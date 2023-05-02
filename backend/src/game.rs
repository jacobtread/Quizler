use crate::{
    error::ServerError,
    games::{GameToken, Games, RemoveGameMessage},
    session::{KickMessage, KickReason, ServerMessage, SessionId, SessionRef},
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use bytes::Bytes;
use log::error;
use mime::Mime;
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
    /// The index of the current question
    question_index: usize,

    /// Game timer
    timer: GameTimer,
    /// Address to the games manager
    games: Addr<Games>,
}

pub struct GameTimer {
    /// The start time for the timer
    start: Instant,
    /// The duration of time the timer is waiting for
    length: Duration,
    /// Whether the game timer has already emitted
    /// completion
    complete: bool,
    /// The current number of ticks that have been processed
    /// (We only produce sync messages every 5 ticks (500ms))
    tick: u8,
}

impl GameTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            length: Duration::from_millis(0),
            complete: false,
            tick: 0,
        }
    }

    pub fn set(&mut self, want: Duration) {
        self.start = Instant::now();
        self.length = want;
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn sync(&mut self) -> Option<TimeSync> {
        self.tick += 1;

        if self.complete || self.tick != 5 {
            self.tick = 0;
            return None;
        }

        let elapsed = self.start.elapsed();

        let total_ms = self.length.as_millis() as u32;
        let elapsed_ms = (elapsed.as_millis() as u32).min(total_ms);

        // Update the complete state
        if total_ms == elapsed_ms {
            self.complete = true;
        }

        // Create the time sync data
        Some(TimeSync {
            total: total_ms,
            elapsed: elapsed_ms,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct TimeSync {
    pub total: u32,
    pub elapsed: u32,
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

    /// The questions have been marked
    Marked = 0x4,

    /// The game has finished
    Finished = 0x5,
}

impl GameState {
    fn requires_timing(&self) -> bool {
        match self {
            GameState::Starting | GameState::AwaitingAnswers | GameState::Marked => true,
            _ => false,
        }
    }
}

impl Game {
    fn tick(&mut self, _ctx: &mut Context<Self>) {
        let state = self.state;
        let complete = self.sync_timer();

        // Handle states that have timing requirements
        if state.requires_timing() && !complete {
            return;
        }

        match state {
            // Ticking in the lobby does nothing...
            GameState::Lobby => {}
            // Ticking the starting timer
            GameState::Starting => {
                self.ready_question();
            }
            // Ticking await ready does nothing...
            GameState::AwaitingReady => {}

            // Answers have been awaited
            GameState::AwaitingAnswers => {
                self.mark_answers();
                self.marked();
            }

            // Question has been marked, the game can now move
            // to the next question
            GameState::Marked => {
                self.ready_question();
            }

            GameState::Finished => todo!(),
        }
    }

    fn set_state(&mut self, state: GameState) {
        self.state = state;
        self.send_all(ServerMessage::GameState { state });
    }

    fn reset_state(&mut self) {
        self.set_state(GameState::Lobby);
        self.skip_timer();
    }

    fn start(&mut self) {
        const START_DURATION: Duration = Duration::from_secs(5);
        self.set_state(GameState::Starting);
        self.set_timer(START_DURATION);
    }

    /// Handles progressing the state to [`GameState::AwaitingAnswers`]
    /// once all the players have provided the Ready state message
    fn all_ready(&mut self) {
        self.set_state(GameState::AwaitingAnswers);
        let question = self.question();
        self.set_timer(Duration::from_millis(question.answer_time));
    }

    fn marked(&mut self) {
        self.set_state(GameState::Marked);
        self.set_timer(Duration::from_millis(self.config.timing.wait_time));
    }

    fn finished(&mut self) {
        self.set_state(GameState::Finished);
    }

    fn ready_question(&mut self) {
        // Handle reaching the end of the questions
        if self.question_index + 1 >= self.config.questions.len() {
            self.finished();
            return;
        }

        // Increase the question index
        self.question_index += 1;

        // Obtain the current question
        let question = self.config.questions[self.question_index].clone();

        // Reset ready states for the players
        self.reset_ready();

        // Send the question contents to the clients
        self.send_all(ServerMessage::Question(question));

        // Begin awaiting for ready messages
        self.set_state(GameState::AwaitingReady);
    }

    /// Syncronizes the timers between the clients and the server
    /// returnig whether the current timer is complete
    fn sync_timer(&mut self) -> bool {
        if let Some(sync) = self.timer.sync() {
            self.send_all(ServerMessage::TimeSync(sync));
        }
        self.timer.complete
    }

    /// Skips the current timer ahead to the ending
    fn skip_timer(&mut self) {
        let total_ms = self.timer.length.as_millis() as u32;
        self.send_all(ServerMessage::TimeSync(TimeSync {
            total: total_ms,
            elapsed: total_ms,
        }));
        self.timer.complete = true;
    }

    /// Sets the current timer waiting duration
    fn set_timer(&mut self, duration: Duration) {
        // Set timer duration
        self.timer.set(duration);

        // Send initialize time sync message
        self.send_all(ServerMessage::TimeSync(TimeSync {
            total: duration.as_millis() as u32,
            elapsed: 0,
        }));
    }

    pub fn new(
        token: GameToken,
        host_ref: SessionRef,
        config: Arc<GameConfig>,
        games: Addr<Games>,
    ) -> Self {
        Self {
            token,
            host: HostSession {
                session_ref: host_ref,
            },
            players: Default::default(),
            config,
            state: GameState::Lobby,
            timer: GameTimer::new(),
            question_index: 0,
            games,
        }
    }

    fn question(&self) -> Arc<Question> {
        self.config
            .questions
            .get(self.question_index)
            .expect("Attempted to access a question at an index that does not exist")
            .clone()
    }

    /// Task for marking the answers
    fn mark_answers(&mut self) {
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
                    // TODO: Handle min max for questions

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
            player
                .session_ref
                .addr
                .do_send(ServerMessage::AnswerResult(result));
        }

        // Update everyones scores
        self.update_scores();
    }

    /// Resets the plaeyr ready states of all the players
    fn reset_ready(&mut self) {
        self.players
            .iter_mut()
            .for_each(|player| player.ready = false);
    }

    /// Send a message to all clients
    fn send_all(&self, message: ServerMessage) {
        // Wrap the message in an Arc to prevent cloning lots of heap data
        let message = Arc::new(message);

        // Send the message to all the players
        for player in &self.players {
            player.session_ref.addr.do_send(message.clone());
        }

        // Send the message to the host
        self.host.session_ref.addr.do_send(message);
    }

    fn update_scores(&self) {
        let mut scores = HashMap::with_capacity(self.players.len());
        for player in &self.players {
            scores.insert(player.session_ref.id, player.score);
        }
        self.send_all(ServerMessage::ScoreUpdate { scores })
    }
}

impl Actor for Game {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        const TICK_INTERVAL: Duration = Duration::from_millis(100);

        // Run the tick function every 100ms
        ctx.run_interval(TICK_INTERVAL, Self::tick);
    }

    /// Handle stopping of a game actor
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // Remove the game from the list of games
        self.games.do_send(RemoveGameMessage { token: self.token });

        // Tell all the players they've been kicked
        for player in &self.players {
            // Send the visual kick message
            player.session_ref.addr.do_send(ServerMessage::Kicked {
                session_id: player.session_ref.id,
                reason: KickReason::HostDisconnect,
            });

            // Notify the session that its been kicked
            player.session_ref.addr.do_send(KickMessage);
        }
    }
}

/// Message to attempt to connect from a new client
#[derive(Message)]
#[rtype(result = "Result<ConnectedMessage, ServerError>")]
pub struct ConnectMessage {
    /// Reference to the session trying to connect
    pub session_ref: SessionRef,
    /// The name for the connecting player
    pub name: String,
}

/// Message containing the connected details for a connected player
pub struct ConnectedMessage {
    /// The session ID
    pub id: SessionId,
    /// The uniquely generated game token (e.g A3DLM)
    pub token: GameToken,
    /// Copy of the game configuration to send back
    pub config: PlayerGameConfig,
    /// Address to the game
    pub game: Addr<Game>,
}

impl Handler<ConnectMessage> for Game {
    type Result = Result<ConnectedMessage, ServerError>;

    fn handle(&mut self, msg: ConnectMessage, ctx: &mut Self::Context) -> Self::Result {
        match self.state {
            GameState::Lobby | GameState::Starting => {}
            _ => return Err(ServerError::NotJoinable),
        }

        // Error if username is already taken
        if self
            .players
            .iter()
            .find(|player| player.name.eq(&msg.name))
            .is_some()
        {
            return Err(ServerError::UsernameTaken);
        }

        let game_player = PlayerSession {
            name: msg.name,
            session_ref: msg.session_ref,
            ready: false,
            answers: Vec::new(),
            results: Vec::new(),
            score: 0,
        };

        // Message sent to existing players for this player
        let joiner_message = Arc::new(ServerMessage::OtherPlayer {
            id: game_player.session_ref.id,
            name: game_player.name.clone(),
        });

        // Notify all players of the existence of eachother
        for player in &self.players {
            player.session_ref.addr.do_send(joiner_message.clone());

            // Message describing the other player
            game_player
                .session_ref
                .addr
                .do_send(ServerMessage::OtherPlayer {
                    id: player.session_ref.id,
                    name: player.name.clone(),
                });
        }

        // Notify the host of the join
        self.host.session_ref.addr.do_send(joiner_message);

        let id = game_player.session_ref.id;

        self.players.push(game_player);

        Ok(ConnectedMessage {
            id,
            token: self.token,
            config: PlayerGameConfig(self.config.clone()),
            game: ctx.address(),
        })
    }
}

/// Message from the host to start the game
#[derive(Message)]
#[rtype(result = "()")]
pub struct StartMessage {
    /// The session reference who is attempting
    /// to start the game
    pub session_ref: SessionRef,
}

impl Handler<StartMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: StartMessage, _ctx: &mut Self::Context) -> Self::Result {
        let host = &self.host.session_ref;

        // Handle messages that aren't from the game host
        if host.id != msg.session_ref.id {
            msg.session_ref.addr.do_send(ServerError::InvalidPermission);
            return;
        }

        self.start();
    }
}

/// Message from the host to cancel starting the game
#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelMessage {
    /// The session reference who is attempting to
    /// cancel starting the game
    pub session_ref: SessionRef,
}

impl Handler<CancelMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: CancelMessage, _ctx: &mut Self::Context) -> Self::Result {
        // Handle messages that aren't from the game host
        if self.host.session_ref.id != msg.session_ref.id {
            msg.session_ref.addr.do_send(ServerError::InvalidPermission);
            return;
        }

        self.reset_state();
    }
}

/// Message to skip the current timer
#[derive(Message)]
#[rtype(result = "()")]
pub struct SkipTimerMessage {
    pub session_ref: SessionRef,
}

impl Handler<SkipTimerMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: SkipTimerMessage, _ctx: &mut Self::Context) -> Self::Result {
        // Handle messages that aren't from the game host
        if self.host.session_ref.id != msg.session_ref.id {
            msg.session_ref.addr.do_send(ServerError::InvalidPermission);
            return;
        }

        self.skip_timer();
    }
}

/// Request to inform that a player is ready
#[derive(Message)]
#[rtype(result = "Result<(), ServerError>")]
pub struct ReadyMessage {
    pub id: SessionId,
}

impl Handler<ReadyMessage> for Game {
    type Result = Result<(), ServerError>;

    fn handle(&mut self, msg: ReadyMessage, _ctx: &mut Self::Context) -> Self::Result {
        // Whether all players are ready
        let mut all_ready = true;
        let mut found_player = false;
        for player in &mut self.players {
            if player.session_ref.id == msg.id {
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
            self.all_ready();
        }

        Ok(())
    }
}

/// Message asking to remove a player from the game
#[derive(Message)]
#[rtype(result = "Option<Image>")]
pub struct GetImageMessage {
    pub uuid: Uuid,
}

impl Handler<GetImageMessage> for Game {
    type Result = Option<Image>;

    fn handle(&mut self, msg: GetImageMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.config.images.get(&msg.uuid).cloned()
    }
}

/// Message asking to remove a player from the game
#[derive(Message)]
#[rtype(result = "()")]
pub struct RemovePlayerMessage {
    /// Reference of who is attempting to remove the player
    /// (Unless the server is removing)
    pub session_ref: Option<SessionRef>,
    /// The ID of the player to remove
    pub target_id: SessionId,
    /// Reason for the player removal (Sent to clients)
    pub reason: KickReason,
}

impl Handler<RemovePlayerMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: RemovePlayerMessage, ctx: &mut Self::Context) -> Self::Result {
        if let Some(session_ref) = &msg.session_ref {
            // Handle messages that aren't from the game host
            if self.host.session_ref.id != session_ref.id {
                session_ref.addr.do_send(ServerError::InvalidPermission);
                return;
            }

            // Host is removing itself (Game is stopping)
            if msg.target_id == session_ref.id {
                // Stop the game
                ctx.stop();
                return;
            }
        }

        let kick_msg = Arc::new(ServerMessage::Kicked {
            session_id: msg.target_id,
            reason: msg.reason,
        });

        // Inform each player of the removal
        self.players
            .iter()
            .for_each(|player| player.session_ref.addr.do_send(kick_msg.clone()));

        // Inform the host of the player removal
        self.host.session_ref.addr.do_send(kick_msg.clone());

        // Find the player position
        let index = self
            .players
            .iter()
            .position(|player| player.session_ref.id == msg.target_id);

        let index = match index {
            Some(value) => value,
            None => {
                // Send the error message to the return addr
                if let Some(session_ref) = msg.session_ref {
                    session_ref.addr.do_send(ServerError::UnknownPlayer);
                }
                return;
            }
        };

        // Remove the player
        let target = self.players.remove(index);
        // Tell the session itself that its been kicked
        target.session_ref.addr.do_send(KickMessage);
    }
}

/// Message asking to remove a player from the game
#[derive(Message)]
#[rtype(result = "Result<(), ServerError>")]
pub struct PlayerAnswerMessage {
    /// Reference of the session that is answering
    pub session_ref: SessionRef,
    /// Answer to the question
    pub answer: QuestionAnswer,
}

impl Handler<PlayerAnswerMessage> for Game {
    type Result = Result<(), ServerError>;

    fn handle(&mut self, msg: PlayerAnswerMessage, _ctx: &mut Self::Context) -> Self::Result {
        let question = self.question();

        // Find the player within the game
        let player = self
            .players
            .iter_mut()
            .find(|player| player.session_ref.id == msg.session_ref.id)
            .ok_or(ServerError::UnknownPlayer)?;

        // Ensure the player hasn't already answered
        if player.answers.len() >= self.question_index {
            return Err(ServerError::AlreadyAnswered);
        }

        // Ensure the answer is the right type of answer
        if !msg.answer.is_valid(&question.ty) {
            return Err(ServerError::InvalidAnswer);
        }

        // Add to player answers
        player.answers.push(msg.answer);

        Ok(())
    }
}

pub struct HostSession {
    /// Reference to the session
    session_ref: SessionRef,
}

pub struct PlayerSession {
    /// Reference to the session
    session_ref: SessionRef,
    /// The player name
    name: String,
    /// The player ready state
    ready: bool,
    /// The players answers and the score they got for them
    answers: Vec<QuestionAnswer>,
    /// Marked version of each question answer
    results: Vec<AnswerResult>,
    /// The player total score
    score: u32,
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
    /// Map of uploaded image UUIDs to their respective
    /// image data
    #[serde(skip)]
    pub images: HashMap<ImageRef, Image>,
}

/// Serializable verison of the reference counted game config
/// that only serializes the parts that should be visible to
/// non host users (only "basic")
#[derive(Clone)]
pub struct PlayerGameConfig(Arc<GameConfig>);

impl Serialize for PlayerGameConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut stru = serializer.serialize_struct("GameConfig", 2)?;
        let this = &*self.0;
        stru.serialize_field("basic", &this.basic)?;
        stru.end()
    }
}

#[derive(Serialize, Deserialize)]
pub struct BasicConfig {
    pub name: String,
    pub text: String,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub struct Image {
    /// Mime type for the image
    pub mime: Mime,
    /// The actual image data bytes
    pub data: Bytes,
}

#[derive(Serialize, Deserialize)]
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
#[serde(tag = "ty")]
pub enum QuestionAnswer {
    Single { answer: usize },
    Multiple { answers: Vec<usize> },
    ClickableImage { answer: (f32, f32) },
}
impl QuestionAnswer {
    pub fn is_valid(&self, qt: &QuestionType) -> bool {
        match (self, qt) {
            (Self::Single { .. }, QuestionType::Single { .. })
            | (Self::Multiple { .. }, QuestionType::Multiple { .. })
            | (Self::ClickableImage { .. }, QuestionType::ClickableImage { .. }) => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Clone, Copy)]
#[serde(tag = "ty", content = "value")]
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

#[derive(Serialize, Deserialize)]
#[serde(tag = "ty")]
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
        /// The optional minimum number of required answers
        min: Option<usize>,
        /// The optional maximum number of required answers
        max: Option<usize>,
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
