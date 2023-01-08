use std::time::{Duration, Instant};

use actix::{
    dev::MessageResponse, Actor, Addr, AsyncContext, Context, Handler, Message, SpawnHandle,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::ServerError,
    games,
    session::{ServerMessage, Session, SessionId, SessionRequest},
};
use log::error;

pub struct Game {
    /// The token this game is stored behind
    token: String,
    /// The host session
    host: HostSession,
    /// Map of session IDs mapped to the session address
    players: Vec<PlayerSession>,
    /// Configuration for the game
    config: GameConfig,
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
    pub fn new(token: String, host_id: u32, host_addr: Addr<Session>, config: GameConfig) -> Self {
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

    fn question(&self) -> &Question {
        match self.config.questions.get(self.question_index) {
            Some(value) => value,
            None => {
                panic!("Attempted to access a question at an index that does not exist");
            }
        }
    }

    /// Task for marking the answers
    fn mark_answers(&mut self, ctx: &mut Context<Self>) {
        let question = self.question();
        for player in &mut self.players {
            let answer = match player.answers.get(self.question_index) {
                Some(answer) => answer,
                None => {
                    // Player did not answer the question
                    continue;
                }
            };

            let score = match (&question.ty, answer) {
                (QuestionType::Single { answers, .. }, QuestionAnswer::Single { answer }) => {
                    let valid = answers.contains(answer);
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

                    let valid = correct == qu_answers.len();
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
                }
                _ => {
                    error!("Mis matched question and answer types don't know how to mark.");
                    continue;
                }
            }
        }
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
        for player in &self.players {
            player.send(message.clone());
        }
        self.host.send(message);
    }
}

#[derive(Message)]
#[rtype(result = "Result<GameResponse, ServerError>")]
pub enum GameRequest {
    /// Message to attempt to connect a new client
    TryConnect {
        id: SessionId,
        name: String,
        addr: Addr<Session>,
    },

    /// Message from the host to start the game
    Start,

    /// Message to cancel starting the game
    Cancel,

    /// Request to inform that a player is ready
    Ready { id: SessionId },

    /// Message to skip the current timer
    SkipTimer,
}

pub enum GameResponse {
    Connected {
        /// The game token
        token: String,
        /// The session ID
        id: u32,
        /// Basic game config information
        basic: BasicConfig,
        /// Timing data for different game events
        timing: GameTiming,
    },
    None,
}

pub type GameId = u32;

impl Actor for Game {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {}
}

impl Handler<GameRequest> for Game {
    type Result = Result<GameResponse, ServerError>;

    fn handle(&mut self, msg: GameRequest, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            GameRequest::TryConnect { id, name, addr } => {
                match self.state {
                    GameState::Lobby | GameState::Starting => {}
                    _ => return Err(ServerError::NotJoinable),
                }

                // Error if username is already taken
                if self
                    .players
                    .iter()
                    .find(|player| player.name.eq(&name))
                    .is_some()
                {
                    return Err(ServerError::UsernameTaken);
                }

                let game_player = PlayerSession {
                    id,
                    name,
                    addr,
                    ready: false,
                    answers: Vec::new(),
                };

                // Message sent to existing players for this player
                let joiner_message = ServerMessage::OtherPlayer {
                    id: game_player.id,
                    name: game_player.name.clone(),
                };

                // Notify all players of the existence of eachother
                for player in &self.players {
                    player.send(joiner_message.clone());

                    // Message describing the other player
                    game_player.send(ServerMessage::OtherPlayer {
                        id: player.id,
                        name: player.name.clone(),
                    });
                }

                // Notify the host of the join
                self.host.send(joiner_message);

                self.players.push(game_player);

                let config = &self.config;
                Ok(GameResponse::Connected {
                    id,
                    token: self.token.clone(),
                    basic: config.basic.clone(),
                    timing: config.timing.clone(),
                })
            }

            GameRequest::Start => {
                self.set_state(GameState::Starting);
                // Begin the start time
                self.starting_task(ctx);
                Ok(GameResponse::None)
            }

            GameRequest::Cancel => {
                self.cancel_task(ctx);
                self.set_state(GameState::Lobby);
                // TODO: Reset all other state
                Ok(GameResponse::None)
            }

            // Not yet implemented
            GameRequest::SkipTimer => {
                self.immediate_task(ctx);

                // Reset the timer future
                Ok(GameResponse::None)
            }
            GameRequest::Ready { id } => {
                // Whether all players are ready
                let mut all_ready = true;
                let mut found_player = false;
                for player in &mut self.players {
                    if player.id == id {
                        player.ready = true;
                        found_player = true;
                    } else if !player.ready {
                        all_ready = false;
                    }
                }

                if !found_player {
                    return Err(ServerError::UnknownPlayer);
                }

                if all_ready {
                    self.ready_question(ctx);
                }

                Ok(GameResponse::None)
            }
        }
    }
}

/// Trait implemented by the sessions that are connected to
/// the game for logic to share between both
pub trait GameSession {
    fn id(&self) -> SessionId;

    fn addr(&self) -> &Addr<Session>;

    fn send(&self, message: ServerMessage) {
        self.addr().do_send(SessionRequest::Message(message));
    }
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
    /// The players answers
    answers: Vec<QuestionAnswer>,
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
pub struct GameConfig {
    /// Basic configuration such as name and subtext
    pub basic: BasicConfig,
    /// Timing data for different game events
    pub timing: GameTiming,
    /// Scoring point values
    pub scoring: Scoring,
    /// The game questions
    pub questions: Vec<Question>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BasicConfig {
    pub name: String,
    pub text: String,
}

#[derive(Clone, Serialize)]
pub struct Scoring {
    /// The minimum amount awarded for getting the
    /// question correct
    pub min: u32,

    /// The maximum amount to award for getting the
    /// question right
    pub max: u32,

    /// The amount awarded if scored within the bonus time
    pub bonus_score: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameTiming {
    /// The time to wait before displaying each question
    pub wait_time: u64,
    /// The time that a bonus score will be granted within
    /// bonus score is disabled if none
    pub bonus_score_time: u64,
}

/// Type for a string which represents a reference to a tmp stored image
pub type ImageRef = String;

#[derive(Clone, Serialize)]
pub struct Question {
    /// The title of the question
    title: String,
    /// The text of the question
    text: String,

    /// Optional image
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

#[derive(Clone, Serialize)]
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

impl<A, M> MessageResponse<A, M> for GameResponse
where
    A: Actor,
    M: Message<Result = GameResponse>,
{
    fn handle(
        self,
        _ctx: &mut <A as Actor>::Context,
        tx: Option<actix::dev::OneshotSender<<M as Message>::Result>>,
    ) {
        if let Some(tx) = tx {
            if tx.send(self).is_err() {
                error!("Failed to send game response");
            }
        }
    }
}
