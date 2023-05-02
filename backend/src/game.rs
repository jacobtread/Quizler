use crate::{
    error::ServerError,
    games::{GameToken, Games, RemoveGameMessage},
    session::{HostAction, KickMessage, KickReason, ServerMessage, Session, SessionId},
    types::{Answer, AnswerData, Image, ImageRef, Question, QuestionData, Score},
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
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
            return None;
        }

        let elapsed = self.start.elapsed();

        let total_ms = self.length.as_millis() as u32;
        let elapsed_ms = (elapsed.as_millis() as u32).min(total_ms);

        // Update the complete state
        if total_ms == elapsed_ms {
            self.complete = true;
        }

        self.tick = 0;

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

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
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
        matches!(
            self,
            GameState::Starting | GameState::AwaitingAnswers | GameState::Marked
        )
    }
}

impl Game {
    /// Creates a new game instance
    pub fn new(
        token: GameToken,
        host_id: SessionId,
        host_addr: Addr<Session>,
        config: Arc<GameConfig>,
        games: Addr<Games>,
    ) -> Self {
        Self {
            token,
            host: HostSession::new(host_id, host_addr),
            players: Default::default(),
            config,
            state: GameState::Lobby,
            timer: GameTimer::new(),
            question_index: 0,
            games,
        }
    }

    fn tick(&mut self, _ctx: &mut Context<Self>) {
        // Handle states that have timing requirements
        if self.state.requires_timing() {
            // Sync the timer and don't continue the tick until the
            // timer is complete
            let complete = self.sync_timer();
            if !complete {
                return;
            }
        }

        match self.state {
            // Ticking empty states that have no time based actions
            GameState::Lobby | GameState::AwaitingReady | GameState::Finished => {}

            // Starting timer has completed we can now send
            // the first question to the players
            GameState::Starting => {
                self.question(0);
            }

            // Answers have been awaited
            GameState::AwaitingAnswers => {
                self.mark_answers();
                self.marked();
            }

            // Question has been marked, the game can now move
            // to the next question
            GameState::Marked => {
                // Handle reaching the end of the questions
                if self.question_index + 1 >= self.config.questions.len() {
                    // Move to the finished state
                    self.finished();
                    return;
                }

                // Increase the question index
                self.question_index += 1;
                self.question(self.question_index);
            }
        }
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

    /// Sets the current game state to the provided `state` and
    /// sends a state update message to all the clients including
    /// the host
    fn set_state(&mut self, state: GameState) {
        self.state = state;
        self.send_all(ServerMessage::GameState { state });
    }

    /// Resets the game state to lobby and resets the game timer
    /// by skipping it to the end time
    fn reset_state(&mut self) {
        self.set_state(GameState::Lobby);
        self.skip_timer();
    }

    /// Handles progresing the state to [`GameState::Starting`].
    /// This is called when the host starts the game
    fn start(&mut self) {
        const START_DURATION: Duration = Duration::from_secs(5);
        self.set_state(GameState::Starting);
        self.set_timer(START_DURATION);
    }

    /// Handles progressing the state to [`GameState::AwaitingAnswers`]
    /// once all the players have provided the Ready state message
    fn all_ready(&mut self) {
        self.set_state(GameState::AwaitingAnswers);
        let question = self
            .config
            .questions
            .get(self.question_index)
            .expect("Attempted to access a question at an index that does not exist")
            .clone();
        self.set_timer(Duration::from_millis(question.answer_time));
    }

    /// Handles progresing the state to [`GameState::Marked`].
    /// This is called once `mark_answers` has been completed
    fn marked(&mut self) {
        self.set_state(GameState::Marked);
        self.set_timer(Duration::from_millis(self.config.timing.wait_time as u64));
    }

    fn finished(&mut self) {
        self.set_state(GameState::Finished);
    }

    fn question(&mut self, index: usize) {
        // Obtain the current question
        let question = self
            .config
            .questions
            .get(index)
            .cloned()
            .expect("Server attempted to display out of bounds question");

        // Reset ready states for the players
        self.players
            .iter_mut()
            .for_each(|player| player.ready = false);

        // Send the question contents to the clients
        self.send_all(ServerMessage::Question(question));

        // Begin awaiting for ready messages
        self.set_state(GameState::AwaitingReady);
    }

    /// Task for marking the answers
    fn mark_answers(&mut self) {
        // Get the current question
        let question = self
            .config
            .questions
            .get(self.question_index)
            .expect("Attempted to access a question at an index that does not exist")
            .clone();

        let mut scores = HashMap::with_capacity(self.players.len());

        for player in &mut self.players {
            // Mark the player question
            let score: Score =
                Self::mark_answer(player, &question, self.question_index, &self.config.timing);

            // Increase the player score
            player.score += score.value();
            player.results.push(score);

            scores.insert(player.id, player.score);
        }

        // Update everyones scores
        self.send_all(ServerMessage::Scores { scores })
    }

    fn mark_answer(
        player: &PlayerSession,
        question: &Question,
        question_index: usize,
        timing: &GameTiming,
    ) -> Score {
        let answer = match &player.answers[question_index] {
            // Player answered the question
            Some(value) => value,
            // Player didn't answer the question
            None => return Score::Incorrect,
        };

        let elapsed_ms = answer.elapsed.as_millis() as u32;
        let is_bonus = elapsed_ms <= timing.bonus_score_time;

        // Calculate the % amount between the min and max answer times
        let answer_time_percent = 1.0 - ((elapsed_ms as f32) / (question.answer_time as f32));

        let scoring = &question.scoring;

        // The base score from the answer time and the bonus
        let mut base_score = scoring.min_score
            + ((scoring.max_score - scoring.min_score) as f32 * answer_time_percent) as u32;

        // Append bonus score amount
        if is_bonus {
            base_score += scoring.bonus_score;
        }

        use Answer as A;
        use QuestionData as Q;

        match (&answer.answer, &question.data) {
            (A::Single { answer }, Q::Single { answers, .. }) => {
                let is_valid = answers.contains(answer);
                if is_valid {
                    Score::Correct(base_score)
                } else {
                    Score::Incorrect
                }
            }
            (
                A::Multiple { answers },
                Q::Multiple {
                    answers: q_answers, ..
                },
            ) => {
                let mut total = 0;
                let mut correct = 0usize;

                for answer in answers {
                    total += 1;

                    if q_answers.contains(answer) {
                        correct += 1;
                    }
                }

                // % correct out of total answers
                let percent = correct as f32 / total as f32;

                if correct == q_answers.len() {
                    Score::Correct(base_score)
                } else if correct == 0 {
                    Score::Incorrect
                } else {
                    let score = ((base_score as f32) * percent).round() as u32;
                    Score::Partial(score)
                }
            }
            (
                A::ClickableImage { answer: (x, y) },
                Q::ClickableImage {
                    top: (tx, ty),
                    bottom: (bx, by),
                    ..
                },
            ) => {
                if x >= tx && x <= bx && y >= ty && y <= by {
                    Score::Correct(base_score)
                } else {
                    Score::Incorrect
                }
            }
            // Mismatched types shouldn't be possible but
            // will be marked as incorrect
            _ => Score::Incorrect,
        }
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
            player.addr.do_send(ServerMessage::Kicked {
                session_id: player.id,
                reason: KickReason::HostDisconnect,
            });

            // Notify the session that its been kicked
            player.addr.do_send(KickMessage);
        }
    }
}

/// Message to attempt to connect from a new client
#[derive(Message)]
#[rtype(result = "Result<ConnectedMessage, ServerError>")]
pub struct ConnectMessage {
    /// The session ID of the session trying to connect
    pub id: SessionId,
    /// The address of the session connecting
    pub addr: Addr<Session>,
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
        // Cannot join games that are already started or finished
        if !matches!(self.state, GameState::Lobby | GameState::Starting) {
            return Err(ServerError::NotJoinable);
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

        // Create the player
        let game_player =
            PlayerSession::new(msg.id, msg.addr, msg.name, self.config.questions.len());

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

        Ok(ConnectedMessage {
            id: msg.id,
            token: self.token,
            config: PlayerGameConfig(self.config.clone()),
            game: ctx.address(),
        })
    }
}

/// Message from the host to complete an
/// action on the game
#[derive(Message)]
#[rtype(result = "Result<(), ServerError>")]
pub struct HostActionMessage {
    /// The session reference who is attempting
    /// the action (Validated against the host)
    pub session_id: SessionId,
    /// The action
    pub action: HostAction,
}

impl Handler<HostActionMessage> for Game {
    type Result = Result<(), ServerError>;

    fn handle(&mut self, msg: HostActionMessage, _ctx: &mut Self::Context) -> Self::Result {
        // Handle messages that aren't from the game host
        if self.host.id != msg.session_id {
            return Err(ServerError::InvalidPermission);
        }

        match msg.action {
            HostAction::Start => self.start(),
            HostAction::Cancel => self.reset_state(),
            HostAction::Skip => self.skip_timer(),
        };

        Ok(())
    }
}

/// Request to inform that a player is ready
#[derive(Message)]
#[rtype(result = "()")]
pub struct ReadyMessage {
    pub id: SessionId,
}

impl Handler<ReadyMessage> for Game {
    type Result = ();

    fn handle(&mut self, msg: ReadyMessage, _ctx: &mut Self::Context) -> Self::Result {
        // Whether all players are ready
        let mut all_ready = true;

        for player in &mut self.players {
            if player.id == msg.id {
                player.ready = true;
            } else if !player.ready {
                all_ready = false;
            }
        }

        if all_ready {
            self.all_ready();
        }
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
#[rtype(result = "Result<(), ServerError>")]
pub struct RemovePlayerMessage {
    /// Reference of who is attempting to remove the player
    /// (Unless the server is removing)
    pub session_id: SessionId,
    /// The ID of the player to remove
    pub target_id: SessionId,
    /// Reason for the player removal (Sent to clients)
    pub reason: KickReason,
}

impl Handler<RemovePlayerMessage> for Game {
    type Result = Result<(), ServerError>;

    fn handle(&mut self, msg: RemovePlayerMessage, ctx: &mut Self::Context) -> Self::Result {
        // Handle messages that aren't from the game host
        if self.host.id != msg.session_id {
            return Err(ServerError::InvalidPermission);
        }

        // Host is removing itself (Game is stopping)
        if msg.target_id == self.host.id {
            // Stop the game
            ctx.stop();
            return Ok(());
        }

        let kick_msg = Arc::new(ServerMessage::Kicked {
            session_id: msg.target_id,
            reason: msg.reason,
        });

        // Inform each player of the removal
        self.players
            .iter()
            .for_each(|player| player.addr.do_send(kick_msg.clone()));

        // Inform the host of the player removal
        self.host.addr.do_send(kick_msg.clone());

        // Find the player position
        let index = self
            .players
            .iter()
            .position(|player| player.id == msg.target_id)
            .ok_or(ServerError::UnknownPlayer)?;

        // Remove the player
        let target = self.players.remove(index);
        // Tell the session itself that its been kicked
        target.addr.do_send(KickMessage);

        Ok(())
    }
}

/// Message asking to remove a player from the game
#[derive(Message)]
#[rtype(result = "Result<(), ServerError>")]
pub struct PlayerAnswerMessage {
    /// Reference of the session that is answering
    pub session_id: SessionId,
    /// Answer to the question
    pub answer: Answer,
}

impl Handler<PlayerAnswerMessage> for Game {
    type Result = Result<(), ServerError>;

    fn handle(&mut self, msg: PlayerAnswerMessage, _ctx: &mut Self::Context) -> Self::Result {
        let elapsed = self.timer.elapsed();

        // Answers are not being accepted at the current time
        if self.state != GameState::AwaitingAnswers {
            return Err(ServerError::UnexpectedMessage);
        }

        let question = self
            .config
            .questions
            .get(self.question_index)
            .expect("Attempted to access a question at an index that does not exist")
            .clone();

        // Find the player within the game
        let player = self
            .players
            .iter_mut()
            .find(|player| player.id == msg.session_id)
            .ok_or(ServerError::UnknownPlayer)?;

        // Ensure the player hasn't already answered
        if player.answers.len() >= self.question_index {
            return Err(ServerError::AlreadyAnswered);
        }

        // Ensure the answer is the right type of answer
        if !msg.answer.is_valid(&question.data) {
            return Err(ServerError::InvalidAnswer);
        }

        // Set the player answer
        player.answers[self.question_index] = Some(AnswerData {
            answer: msg.answer,
            elapsed,
        });

        Ok(())
    }
}

pub struct HostSession {
    /// The ID of the referenced session
    id: SessionId,
    /// The addr to the session
    addr: Addr<Session>,
}

impl HostSession {
    pub fn new(id: SessionId, addr: Addr<Session>) -> Self {
        Self { id, addr }
    }
}

pub struct PlayerSession {
    /// The ID of the referenced session
    id: SessionId,
    /// The addr to the session
    addr: Addr<Session>,

    /// The player name
    name: String,
    /// The player ready state
    ready: bool,
    /// The players answers and the score they got for them
    answers: Vec<Option<AnswerData>>,
    /// Marked version of each question answer
    results: Vec<Score>,
    /// The player total score
    score: u32,
}

impl PlayerSession {
    pub fn new(id: SessionId, addr: Addr<Session>, name: String, question_len: usize) -> Self {
        // Initialize the empty answers list
        let answers = vec![None; question_len];

        Self {
            id,
            addr,
            name,
            ready: false,
            answers,
            results: Vec::new(),
            score: 0,
        }
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
pub struct GameTiming {
    /// The time to wait before displaying each question (ms)
    pub wait_time: u32,
    /// The time that a bonus score will be granted within
    /// bonus score is disabled if none (ms)
    pub bonus_score_time: u32,
}
