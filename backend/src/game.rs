use crate::{
    error::ServerError,
    games::GameToken,
    session::{ServerMessage, Session, SessionId},
};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, MessageResult, SpawnHandle};
use log::error;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

pub struct Game {
    /// The token this game is stored behind
    token: GameToken,
    /// The host session
    host: HostSession,
    /// Map of session IDs mapped to the session address
    players: Vec<PlayerSession>,
    /// Configuration for the game
    config: Arc<GameConfig>,
    /// The state of the game
    state: GameState,
    /// Spawn handle for the tick task
    task: Option<DelayedTask>,
    /// The index of the current question
    question_index: usize,
    /// Game timer
    timer: GameTimer,
}

pub struct GameTimer {
    last: Instant,
    want: Duration,
}

/// Task that is delayed
pub struct DelayedTask {
    // Spawn handle for the timer update task
    timer_handle: SpawnHandle,
    /// Spawn handle for the delayed task call
    task_handle: SpawnHandle,
    /// Underlying task to execute
    task: Box<dyn FnOnce(&mut Game, &mut Context<Game>)>,
}

impl DelayedTask {
    pub fn finish(self, game: &mut Game, ctx: &mut Context<Game>) {
        // Cancel the timer and task handlers
        ctx.cancel_future(self.timer_handle);
        ctx.cancel_future(self.task_handle);

        // Call the task fn
        (self.task)(game, ctx);
    }

    pub fn cancel(self, ctx: &mut Context<Game>) {
        ctx.cancel_future(self.timer_handle);
        ctx.cancel_future(self.task_handle);
    }
}

impl GameTimer {
    pub fn new() -> Self {
        Self {
            last: Instant::now(),
            want: Duration::from_millis(0),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.last.elapsed()
    }

    pub fn has_elapsed(&self) -> bool {
        let elapsed = self.elapsed();
        elapsed >= self.want
    }

    pub fn set(&mut self, want: Duration) {
        self.last = Instant::now();
        self.want = want;
    }
}

#[derive(Serialize, Clone, Copy)]
#[repr(u8)]
pub enum GameState {
    /// The game is in the lobby
    Lobby = 0x0,
    /// The game is starting
    Starting = 0x1,

    /// The game is waiting for ready from all the players
    AwaitingReady = 0x2,

    /// The game has started and is waiting for answers
    AwaitingAnswers = 0x3,

    /// The game has finished
    Finished = 0x4,
}

const TIMER_INTERVAL: Duration = Duration::from_millis(500);

impl Game {
    pub fn new(
        token: GameToken,
        host_id: SessionId,
        host_addr: Addr<Session>,
        config: Arc<GameConfig>,
    ) -> Self {
        Self {
            token,
            host: HostSession {
                id: host_id,
                addr: host_addr,
            },
            players: Default::default(),
            config,
            state: GameState::Lobby,
            task: None,
            timer: GameTimer::new(),
            question_index: 0,
        }
    }

    /// Spawns the game starting time which will handle when the game
    /// moves from the starting state to the started state.
    ///
    /// `ctx` The game context
    fn starting_task(&mut self, ctx: &mut Context<Self>) {
        // Game starts after 5 seconds unless skipped
        const START_DURATION: Duration = Duration::from_secs(5);

        self.delayed_task(ctx, START_DURATION, |actor, ctx| {
            actor.set_state(GameState::Starting);
            // TODO: Started game logic
        })
    }

    /// Spawns a delayed task to execute after duration where all the clients
    /// have their times updated until said duration
    fn delayed_task<F>(&mut self, ctx: &mut Context<Self>, duration: Duration, f: F)
    where
        F: Fn(&mut Self, &mut Context<Self>) + 'static,
    {
        // Set the timer start point and end duration
        self.timer.set(duration);

        // Intital time update
        let total = self.timer.want.as_millis() as u64;
        self.send_all(ServerMessage::TimeSync { total, elapsed: 0 });

        // Interval handle for updating the timers for all the clients to ensure
        // they are up to date with the server time
        let timer_handle = ctx.run_interval(TIMER_INTERVAL, |actor, ctx| {
            let timer = &actor.timer;
            let (total, elapsed) = if timer.has_elapsed() {
                let total = timer.want.as_millis() as u64;
                (total, total)
            } else {
                // Size down casted to u64 which is probbably even larger than nessicary
                let total = timer.want.as_millis() as u64;
                let elapsed = timer.elapsed().as_millis() as u64;
                (total, elapsed)
            };
            actor.send_all(ServerMessage::TimeSync { total, elapsed })
        });

        // Task handle for finish the task after the desired duration
        let task_handle = ctx.run_later(duration, |actor, ctx| {
            if let Some(task) = actor.task.take() {
                task.finish(actor, ctx);
            }
        });

        // Delayed task for storing the task
        let task = DelayedTask {
            task: Box::new(f),
            task_handle,
            timer_handle,
        };
        self.task = Some(task)
    }

    /// Immediately completes the current delayed task
    fn immediate_task(&mut self, ctx: &mut Context<Self>) {
        if let Some(task) = self.task.take() {
            task.finish(self, ctx);
        }
    }

    /// Begins the question at the provided index
    ///
    /// `ctx`   The game context
    /// `index` The question index
    fn begin_question(&mut self, ctx: &mut Context<Self>, index: usize) {
        self.reset_ready();
        let question = match self.config.questions.get(index) {
            Some(value) => value,
            None => {
                error!("Attempted to begin a question at an index which doesn't exist");
                return;
            }
        };
        self.question_index = index;
        self.send_all(ServerMessage::Question(question.clone()));
    }

    /// Called after all the ready messages have been recieved from all the
    /// clients
    fn ready_question(&mut self, ctx: &mut Context<Self>) {
        self.send_all(ServerMessage::BeginQuestion);
        let question = self.question();
        self.delayed_task(
            ctx,
            Duration::from_millis(question.answer_time),
            Self::mark_answers,
        )
    }

    fn question(&self) -> Arc<Question> {
        self.config
            .questions
            .get(self.question_index)
            .expect("Attempted to access a question at an index that does not exist")
            .clone()
    }

    /// Task for marking the answers
    fn mark_answers(&mut self, ctx: &mut Context<Self>) {
        let question = self.question();

        let scoring = &question.scoring;

        for player in &mut self.players {
            let answer = match player.answers.get(self.question_index) {
                Some(answer) => answer,
                None => {
                    // Player did not answer the question
                    continue;
                }
            };

            let elapsed = self.timer.elapsed();
            let is_bonus = elapsed.as_micros() as u64 <= self.config.timing.bonus_score_time;

            let percent =
                1.0 - ((elapsed.as_millis() as f32) / (question.answer_time as f32)).max(1.0);

            let mut base_score = scoring.min_score
                + ((scoring.max_score - scoring.min_score) as f32 * percent) as u32;

            if is_bonus {
                base_score += scoring.bonus_score;
            }

            let result = match (&question.ty, &answer) {
                (QuestionType::Single { answers, .. }, QuestionAnswer::Single { answer }) => {
                    let valid = answers.contains(answer);

                    if valid {
                        AnswerResult::Correct(base_score)
                    } else {
                        AnswerResult::Incorrect
                    }
                }
                (
                    QuestionType::Multiple {
                        answers: qu_answers,
                        ..
                    },
                    QuestionAnswer::Multiple { answers },
                ) => {
                    let mut correct = 0usize;
                    let mut incorrect = 0usize;
                    for answer in answers {
                        if qu_answers.contains(answer) {
                            correct += 1;
                        } else {
                            incorrect += 1;
                        }
                    }

                    // The percent completion
                    let percent = (correct as f32) / ((correct + incorrect) as f32);

                    let valid = correct == qu_answers.len();

                    if valid {
                        AnswerResult::Correct(base_score)
                    } else {
                        let score = ((base_score as f32) * percent).round() as u32;
                        AnswerResult::Partial(score)
                    }
                }
                (
                    QuestionType::ClickableImage { top, bottom, .. },
                    QuestionAnswer::ClickableImage { answer },
                ) => {
                    // Clicked position is within top and bottom box position
                    let valid = answer.0 >= top.0
                        && answer.0 <= bottom.0
                        && answer.1 >= top.1
                        && answer.1 <= bottom.1;
                    if valid {
                        AnswerResult::Correct(base_score)
                    } else {
                        AnswerResult::Incorrect
                    }
                }
                _ => {
                    error!("Mis matched question and answer types don't know how to mark.");
                    continue;
                }
            };

            player.score += result.score();
            player.results.push(result.clone());

            // Send the result to the player
            player.addr.do_send(ServerMessage::AnswerResult(result));
        }
        // Update everyones scores
        self.update_scores();
    }

    /// Resets the plaeyr ready states of all the players
    fn reset_ready(&mut self) {
        for player in &mut self.players {
            player.ready = false;
        }
    }

    fn cancel_task(&mut self, ctx: &mut Context<Self>) {
        if let Some(task) = self.task.take() {
            task.cancel(ctx);
        }
    }

    fn set_state(&mut self, state: GameState) {
        self.state = state;
        self.send_all(ServerMessage::GameState(state));
    }

    /// Send a message to all clients
    fn send_all(&self, message: ServerMessage) {
        // Wrap the message in an Arc to prevent cloning lots of heap data
        let message = Arc::new(message);

        // Send the message to all the players
        for player in &self.players {
            player.addr.do_send(message.clone());
        }

        // Send the message to the host
        self.host.addr.do_send(message);
    }

    fn update_scores(&self) {
        let mut scores = HashMap::new();
        for player in &self.players {
            scores.insert(player.id, player.score);
        }
        self.send_all(ServerMessage::ScoreUpdate { scores })
    }
}

pub type GameId = u32;

impl Actor for Game {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {}
}

/// Message to attempt to connect from a new client
#[derive(Message)]
#[rtype(result = "Result<ConnectedMessage, ServerError>")]
pub struct TryConnectMessage {
    /// The session ID
    pub id: SessionId,
    /// The name for the connecting player
    pub name: String,
    /// The return address of the session
    pub addr: Addr<Session>,
}

/// Message containing the connected details for a connected player
#[derive(Serialize)]
pub struct ConnectedMessage {
    /// The uniquely generated game token (e.g A3DLM)
    pub token: GameToken,
    /// The session ID
    pub id: SessionId,
    /// Copy of the game configuration to send back
    pub config: PlayerGameConfig,
}

impl Handler<TryConnectMessage> for Game {
    type Result = MessageResult<TryConnectMessage>;

    fn handle(
        &mut self,
        TryConnectMessage { id, name, addr }: TryConnectMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        match self.state {
            GameState::Lobby | GameState::Starting => {}
            _ => return MessageResult(Err(ServerError::NotJoinable)),
        }

        // Error if username is already taken
        if self
            .players
            .iter()
            .find(|player| player.name.eq(&name))
            .is_some()
        {
            return MessageResult(Err(ServerError::UsernameTaken));
        }

        let game_player = PlayerSession {
            id,
            name,
            addr,
            ready: false,
            answers: Vec::new(),
            results: Vec::new(),
            score: 0,
        };

        // Message sent to existing players for this player
        let joiner_message = Arc::new(ServerMessage::OtherPlayer {
            id: game_player.id,
            name: game_player.name.clone(),
        });

        // Notify all players of the existence of eachother
        for player in &self.players {
            player.addr.do_send(joiner_message.clone());

            // Message describing the other player
            game_player.addr.do_send(ServerMessage::OtherPlayer {
                id: player.id,
                name: player.name.clone(),
            });
        }

        // Notify the host of the join
        self.host.addr.do_send(joiner_message);

        self.players.push(game_player);

        MessageResult(Ok(ConnectedMessage {
            id,
            token: self.token,
            config: PlayerGameConfig(self.config.clone()),
        }))
    }
}

/// Message from the host to start the game
#[derive(Message)]
#[rtype(result = "()")]
pub struct StartMessage;

impl Handler<StartMessage> for Game {
    type Result = ();

    fn handle(&mut self, _: StartMessage, ctx: &mut Self::Context) -> Self::Result {
        self.set_state(GameState::Starting);
        // Begin the start time
        self.starting_task(ctx);
    }
}

/// Message from the host to cancel starting the game
#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelMessage;

impl Handler<CancelMessage> for Game {
    type Result = ();

    fn handle(&mut self, _: CancelMessage, ctx: &mut Self::Context) -> Self::Result {
        self.cancel_task(ctx);
        self.set_state(GameState::Lobby);
    }
}

/// Message to skip the current timer
#[derive(Message)]
#[rtype(result = "()")]
pub struct SkipTimerMessage;

impl Handler<SkipTimerMessage> for Game {
    type Result = ();

    fn handle(&mut self, _: SkipTimerMessage, ctx: &mut Self::Context) -> Self::Result {
        self.immediate_task(ctx);
    }
}

/// Request to inform that a player is ready
#[derive(Message)]
#[rtype(result = "Result<(), ServerError>")]
pub struct ReadyMessage {
    pub id: SessionId,
}

impl Handler<ReadyMessage> for Game {
    type Result = MessageResult<ReadyMessage>;

    fn handle(&mut self, msg: ReadyMessage, ctx: &mut Self::Context) -> Self::Result {
        // Whether all players are ready
        let mut all_ready = true;
        let mut found_player = false;
        for player in &mut self.players {
            if player.id == msg.id {
                player.ready = true;
                found_player = true;
            } else if !player.ready {
                all_ready = false;
            }
        }

        if !found_player {
            return MessageResult(Err(ServerError::UnknownPlayer));
        }

        if all_ready {
            self.ready_question(ctx);
        }

        MessageResult(Ok(()))
    }
}

/// Trait implemented by the sessions that are connected to
/// the game for logic to share between both
pub trait GameSession {
    fn id(&self) -> SessionId;

    fn addr(&self) -> &Addr<Session>;
}

pub struct HostSession {
    /// The ID of the session
    id: SessionId,
    /// Address to the session
    addr: Addr<Session>,
}

impl GameSession for HostSession {
    fn id(&self) -> SessionId {
        self.id
    }

    fn addr(&self) -> &Addr<Session> {
        &self.addr
    }
}

pub struct PlayerSession {
    /// The ID of the session
    id: SessionId,
    /// The player name
    name: String,
    /// Address to the session
    addr: Addr<Session>,
    /// The player ready state
    ready: bool,
    /// The players answers and the score they got for them
    answers: Vec<QuestionAnswer>,
    /// Marked version of each question answer
    results: Vec<AnswerResult>,
    /// The player total score
    score: u32,
}

impl GameSession for PlayerSession {
    fn id(&self) -> SessionId {
        self.id
    }

    fn addr(&self) -> &Addr<Session> {
        &self.addr
    }
}

/// Configuration data for a game
#[derive(Serialize)]
pub struct GameConfig {
    /// Basic configuration such as name and subtext
    pub basic: BasicConfig,
    /// Timing data for different game events
    pub timing: GameTiming,
    /// The game questions
    pub questions: Vec<Arc<Question>>,
}

/// Serializable verison of the reference counted game config
/// that only serializes the parts that should be visible to
/// non host users ("timing" and "basic" not "questions")
#[derive(Clone)]
pub struct PlayerGameConfig(Arc<GameConfig>);

impl Serialize for PlayerGameConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut stru = serializer.serialize_struct("GameConfig", 2)?;
        let this = &*self.0;
        stru.serialize_field("basic", &this.basic);
        stru.serialize_field("timing", &this.timing);
        stru.end()
    }
}

#[derive(Serialize, Deserialize)]
pub struct BasicConfig {
    pub name: String,
    pub text: String,
}

#[derive(Serialize)]
pub struct Scoring {
    /// Minimum score awarded for the longest time taken
    pub min_score: u32,
    /// Maximum score awarded for the shortest time taken
    pub max_score: u32,
    /// The amount awarded if scored within the bonus time
    pub bonus_score: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GameTiming {
    /// The time to wait before displaying each question
    pub wait_time: u64,
    /// The time that a bonus score will be granted within
    /// bonus score is disabled if none
    pub bonus_score_time: u64,
}

/// Type for a string which represents a reference to a tmp stored image
pub type ImageRef = Uuid;

pub struct Image {
    /// The file extension for the image
    ext: String,
    /// The actual image data bytes
    data: Vec<u8>,
}

#[derive(Serialize)]
pub struct Question {
    /// The title of the question
    title: String,
    /// The text of the question
    text: String,
    /// Optional UUID from created image
    image: Option<ImageRef>,
    /// The content of the question
    ty: QuestionType,
    /// The time given to answer the question
    answer_time: u64,
    /// The point scoring for the question
    scoring: Scoring,
}

#[derive(Deserialize)]
pub enum QuestionAnswer {
    Single { answer: usize },
    Multiple { answers: Vec<usize> },
    ClickableImage { answer: (f32, f32) },
}

#[derive(Serialize, Clone, Copy)]
pub enum AnswerResult {
    // Answer was 100% correct
    Correct(u32),
    // Answer was incorrect
    Incorrect,
    // Multiple choice has some asnwers right
    Partial(u32),
}
impl AnswerResult {
    pub fn score(&self) -> u32 {
        match self {
            Self::Correct(value) => *value,
            Self::Incorrect => 0,
            Self::Partial(value) => *value,
        }
    }
}

#[derive(Serialize)]
pub enum QuestionType {
    /// Single choice question
    Single {
        /// Vec of indexes of correct answers
        #[serde(skip)]
        answers: Vec<usize>,
        /// Vec of the possible answers
        values: Vec<String>,
    },
    /// Multiple choice question
    Multiple {
        /// Vec of indexes of correct answers
        #[serde(skip)]
        answers: Vec<usize>,
        /// Vec of the possible answers
        values: Vec<String>,
    },
    /// Image where you must click an area
    ClickableImage {
        /// The image url to take clicking on
        image: ImageRef,
        /// Top left box coordinate
        #[serde(skip)]
        top: (f32, f32),
        /// Bottom right box coordinate
        #[serde(skip)]
        bottom: (f32, f32),
    },
}
