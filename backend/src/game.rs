use crate::{
    games::{GameToken, Games, RemoveGameMessage},
    msg::ServerMessage,
    session::{ClearGameMessage, Session, SessionId},
    types::{
        Answer, AnswerData, HostAction, Image, ImageRef, NameFiltering, Question, QuestionData,
        RemoveReason, Score, ServerError,
    },
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message, SpawnHandle};
use log::debug;
use rustrict::CensorStr;
use serde::Serialize;
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

    /// Spawn handle for delayed tasks
    task_handle: Option<SpawnHandle>,

    /// Start time updated for each question
    start_time: Instant,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// The game is in the lobby
    Lobby,

    /// The game is starting
    Starting,

    /// The game is waiting for ready from all the players
    AwaitingReady,

    /// The question is about to start
    PreQuestion,

    /// The game has started and is waiting for answers
    AwaitingAnswers,

    /// The questions have been marked
    Marked,

    /// The game has finished
    Finished,
}

impl Game {
    /// Creates a new game instance
    pub fn new(
        token: GameToken,
        host_id: SessionId,
        host_addr: Addr<Session>,
        config: Arc<GameConfig>,
    ) -> Self {
        Self {
            token,
            host: HostSession::new(host_id, host_addr),
            players: Default::default(),
            config,
            state: GameState::Lobby,
            question_index: 0,
            task_handle: None,
            start_time: Instant::now(),
        }
    }

    /// Creates a new delayed task to move to the next state once the provided
    /// duration has passed. This updates the timer state for clients aswell
    ///
    /// `duration` The duration to wait before moving states
    /// `ctx`      The actor context
    fn timed_next_state(&mut self, duration: Duration, ctx: &mut Context<Self>) {
        self.task_handle = Some(ctx.run_later(duration, |act, ctx| {
            // Clear the task handle
            act.task_handle = None;

            // Move to the next state
            act.next_state(ctx);
        }));

        // Send timer message with the duration time
        self.send_all(ServerMessage::Timer {
            value: duration.as_millis() as u32,
        });
    }

    /// Moves the game to the next state based on its current state
    fn next_state(&mut self, ctx: &mut Context<Self>) {
        // If a task handle still exists cancel it
        if let Some(task_handle) = self.task_handle.take() {
            ctx.cancel_future(task_handle);
        }

        match self.state {
            // Next state after lobby is starting
            GameState::Lobby => {
                const START_DURATION: Duration = Duration::from_secs(5);

                self.set_state(GameState::Starting);
                self.timed_next_state(START_DURATION, ctx);
            }

            // Next state after starting is question
            GameState::Starting => {
                self.question();
            }

            // Next state after awaiting ready is pre question
            GameState::AwaitingReady => {
                const START_DURATION: Duration = Duration::from_secs(5);
                self.set_state(GameState::PreQuestion);
                self.timed_next_state(START_DURATION, ctx);
            }

            // Next state after pre-question is awaiting answers
            GameState::PreQuestion => {
                // Await answers for the question
                self.set_state(GameState::AwaitingAnswers);

                // Assign the question start time
                self.start_time = Instant::now();

                let question = self.current_question();
                self.timed_next_state(Duration::from_millis(question.answer_time), ctx);
            }

            // Next state after awaiting answers is marking
            GameState::AwaitingAnswers => {
                self.mark_answers();
            }

            // Next state after marking is the next question
            GameState::Marked => {
                // Move to the next question
                self.next_question();
            }

            // Next state after finished is a reset game
            GameState::Finished => {
                self.reset_completely();
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

    /// Sets the current game state to the provided `state` and
    /// sends a state update message to all the clients including
    /// the host
    fn set_state(&mut self, state: GameState) {
        self.state = state;
        self.send_all(ServerMessage::GameState { state });
    }

    /// Resets the game state and all the player data to its initial values
    fn reset_completely(&mut self) {
        self.question_index = 0;

        for player in self.players.iter_mut() {
            // Fill the answers and scores with None
            player.answers.fill(PlayerAnswer::default());

            // Reset the player score
            player.score = 0;
        }

        self.set_state(GameState::Lobby);
    }

    /// Handles updating state post removing a player
    fn on_remove(&mut self, ctx: &mut Context<Self>) {
        self.update_ready(ctx);

        // Reset the game if everyone disconected while in progress
        if self.state != GameState::Finished && self.players.is_empty() {
            self.reset_completely();
        }
    }

    /// Updates the current state checking if all the players are ready
    /// then if they are progresses the state to [`GameState::AwaitingAnswers`]
    fn update_ready(&mut self, ctx: &mut Context<Self>) {
        // Ignore if we aren't expecting ready states
        if self.state != GameState::AwaitingReady {
            return;
        }

        // Check all players are ready
        let all_ready = self.players.iter().all(|player| player.ready);
        if !all_ready || !self.host.ready {
            return;
        }

        self.next_state(ctx)
    }

    fn next_question(&mut self) {
        // Handle reaching the end of the questions
        if self.question_index + 1 >= self.config.questions.len() {
            // Move to the finished state
            self.set_state(GameState::Finished);
            return;
        }

        // Increase the question index
        self.question_index += 1;
        self.question();
    }

    fn current_question(&self) -> Arc<Question> {
        self.config.questions[self.question_index].clone()
    }

    fn question(&mut self) {
        // Obtain the current question
        let question = self.current_question();

        // Reset ready states for the players
        self.players
            .iter_mut()
            .for_each(|player| player.ready = false);

        // Reset host ready state
        self.host.ready = false;

        // Send the question contents to the clients
        self.send_all(ServerMessage::Question { question });

        // Begin awaiting for ready messages
        self.set_state(GameState::AwaitingReady);
    }

    /// Task for marking the answers
    fn mark_answers(&mut self) {
        // Get the current question
        let question = self.current_question();

        let mut scores = HashMap::with_capacity(self.players.len());

        for player in &mut self.players {
            let answer = &mut player.answers[self.question_index];
            let score = answer.mark(&question);

            // Increase the player score
            player.score += score.value();

            player.addr.do_send(ServerMessage::Score { score });

            scores.insert(player.id, player.score);
        }

        // Update everyones scores
        self.send_all(ServerMessage::Scores { scores });

        // Set state to marked
        self.set_state(GameState::Marked);
    }
}

impl Actor for Game {
    type Context = Context<Self>;

    /// Handle stopping of a game actor
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("Game stopped: {}", self.token);

        // Remove the game from the list of games
        Games::get().do_send(RemoveGameMessage { token: self.token });

        // Tell all the players they've been kicked
        for player in &self.players {
            // Send the visual kick message
            player.addr.do_send(ServerMessage::Kicked {
                id: player.id,
                reason: RemoveReason::HostDisconnect,
            });

            // Notify the session that its been kicked
            player.addr.do_send(ClearGameMessage);
        }
    }
}

/// Message to attempt to connect from a new client
#[derive(Message)]
#[rtype(result = "Result<JoinedMessage, ServerError>")]
pub struct JoinMessage {
    /// The session ID of the session trying to connect
    pub id: SessionId,
    /// The address of the session connecting
    pub addr: Addr<Session>,
    /// The name for the connecting player
    pub name: String,
}

/// Message containing the connected details for a connected player
pub struct JoinedMessage {
    /// The uniquely generated game token (e.g A3DLM)
    pub token: GameToken,
    /// Copy of the game configuration to send back
    pub config: Arc<GameConfig>,
}

impl Handler<JoinMessage> for Game {
    type Result = Result<JoinedMessage, ServerError>;

    fn handle(&mut self, msg: JoinMessage, _ctx: &mut Self::Context) -> Self::Result {
        // Trim name padding
        let name = msg.name.trim();

        // Name filtering
        if let Some(filter_type) = self.config.filtering.type_of() {
            if name.is(filter_type) {
                return Err(ServerError::InappropriateName);
            }
        }

        // Cannot join games that are already started or finished
        if !matches!(self.state, GameState::Lobby | GameState::Starting) {
            return Err(ServerError::NotJoinable);
        }

        // Game already at max capacity
        if self.players.len() >= self.config.max_players {
            return Err(ServerError::CapacityReached);
        }

        // Error if username is already taken
        if self
            .players
            .iter()
            .any(|player| player.name.eq_ignore_ascii_case(name))
        {
            return Err(ServerError::UsernameTaken);
        }

        // Create the player
        let game_player = PlayerSession::new(
            msg.id,
            msg.addr,
            name.to_string(),
            self.config.questions.len(),
        );

        // Message sent to existing players for this player
        let joiner_message = Arc::new(ServerMessage::PlayerData {
            id: game_player.id,
            name: game_player.name.clone(),
        });

        // Notify all players of the existence of eachother
        for player in &self.players {
            player.addr.do_send(joiner_message.clone());

            // Message describing the other player
            game_player.addr.do_send(ServerMessage::PlayerData {
                id: player.id,
                name: player.name.clone(),
            });
        }

        // Notify the host of the join
        self.host.addr.do_send(joiner_message);

        self.players.push(game_player);

        Ok(JoinedMessage {
            token: self.token,
            config: self.config.clone(),
        })
    }
}

/// Message from the host to complete an
/// action on the game
#[derive(Message)]
#[rtype(result = "Result<(), ServerError>")]
pub struct HostActionMessage {
    /// The ID of the session sending the action
    pub id: SessionId,
    /// The action
    pub action: HostAction,
}

impl Handler<HostActionMessage> for Game {
    type Result = Result<(), ServerError>;

    fn handle(&mut self, msg: HostActionMessage, ctx: &mut Self::Context) -> Self::Result {
        // Handle messages that aren't from the game host
        if self.host.id != msg.id {
            return Err(ServerError::InvalidPermission);
        }

        match msg.action {
            HostAction::Reset => self.reset_completely(),
            HostAction::Next => self.next_state(ctx),
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

    fn handle(&mut self, msg: ReadyMessage, ctx: &mut Self::Context) -> Self::Result {
        if msg.id == self.host.id {
            self.host.ready = true;
        } else {
            let player = self.players.iter_mut().find(|player| player.id == msg.id);
            if let Some(player) = player {
                player.ready = true;
            }
        }

        self.update_ready(ctx);
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
    pub id: SessionId,
    /// The ID of the player to remove
    pub target_id: SessionId,
    /// Reason for the player removal (Sent to clients)
    pub reason: RemoveReason,
}

impl Handler<RemovePlayerMessage> for Game {
    type Result = Result<(), ServerError>;

    fn handle(&mut self, msg: RemovePlayerMessage, ctx: &mut Self::Context) -> Self::Result {
        // Handle messages that aren't from the game host
        if msg.target_id != msg.id && self.host.id != msg.id {
            return Err(ServerError::InvalidPermission);
        }

        // Host is removing itself (Game is stopping)
        if msg.target_id == self.host.id {
            // Stop the game
            ctx.stop();
            return Ok(());
        }

        // Find the player position
        let index = self
            .players
            .iter()
            .position(|player| player.id == msg.target_id)
            .ok_or(ServerError::UnknownPlayer)?;

        let mut reason = msg.reason;

        // Replace host remove reason for non hosts
        if RemoveReason::RemovedByHost == reason && msg.id != self.host.id {
            reason = RemoveReason::Disconnected;
        }

        let kick_msg = Arc::new(ServerMessage::Kicked {
            id: msg.target_id,
            reason,
        });

        // Inform each player of the removal
        self.players
            .iter()
            .for_each(|player| player.addr.do_send(kick_msg.clone()));

        // Inform the host of the player removal
        self.host.addr.do_send(kick_msg);

        // Remove the player
        let target = self.players.remove(index);
        // Tell the session itself that its been kicked
        target.addr.do_send(ClearGameMessage);

        self.on_remove(ctx);

        Ok(())
    }
}

/// Message asking to remove a player from the game
#[derive(Message)]
#[rtype(result = "Result<(), ServerError>")]
pub struct PlayerAnswerMessage {
    /// The ID of the session who answered
    pub id: SessionId,
    /// Answer to the question
    pub answer: Answer,
}

impl Handler<PlayerAnswerMessage> for Game {
    type Result = Result<(), ServerError>;

    fn handle(&mut self, msg: PlayerAnswerMessage, ctx: &mut Self::Context) -> Self::Result {
        let elapsed = self.start_time.elapsed();

        // Answers are not being accepted at the current time
        if self.state != GameState::AwaitingAnswers {
            return Err(ServerError::UnexpectedMessage);
        }

        let question = self.current_question();

        // Find the player within the game
        let player = self
            .players
            .iter_mut()
            .find(|player| player.id == msg.id)
            .ok_or(ServerError::UnknownPlayer)?;

        // Ensure the answer is the right type of answer
        if !msg.answer.is_valid(&question.data) {
            return Err(ServerError::InvalidAnswer);
        }

        // Set the player answer
        player.answers[self.question_index].answer(AnswerData {
            answer: msg.answer,
            elapsed,
        });

        // If all the players have answered we can skip the timer
        let all_answered = self
            .players
            .iter()
            .all(|player| player.answers[self.question_index].has_answer());

        if all_answered {
            self.next_state(ctx);
        }

        Ok(())
    }
}

pub struct HostSession {
    /// The ID of the referenced session
    id: SessionId,
    /// The addr to the session
    addr: Addr<Session>,
    /// The player ready state
    ready: bool,
}

impl HostSession {
    pub fn new(id: SessionId, addr: Addr<Session>) -> Self {
        Self {
            id,
            addr,
            ready: false,
        }
    }
}

pub struct PlayerSession {
    /// The ID of the referenced session
    id: SessionId,
    /// The addr to the session
    addr: Addr<Session>,
    /// The player ready state
    ready: bool,

    /// The player name
    name: String,
    /// The players answers and the score they got for them
    answers: Vec<PlayerAnswer>,
    /// The player total score
    score: u32,
}

#[derive(Default, Clone)]
struct PlayerAnswer {
    /// The answer provided by the player
    data: Option<AnswerData>,
    /// The score provided by the server
    score: Option<Score>,
}

impl PlayerAnswer {
    #[inline]
    fn has_answer(&self) -> bool {
        self.data.is_some()
    }

    #[inline]
    fn answer(&mut self, answer: AnswerData) {
        self.data = Some(answer);
    }

    fn mark(&mut self, question: &Question) -> Score {
        let score = self.get_score(question);
        self.score = Some(score);
        score
    }

    fn get_score(&self, question: &Question) -> Score {
        let answer = match &self.data {
            Some(value) => value,
            None => return Score::Incorrect,
        };

        let elapsed_ms = answer.elapsed.as_millis() as u32;
        let is_bonus = elapsed_ms <= question.bonus_score_time;

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
                let is_valid = answers
                    .get(*answer)
                    .map(|value| value.correct)
                    .unwrap_or(false);

                if is_valid {
                    Score::Correct { value: base_score }
                } else {
                    Score::Incorrect
                }
            }
            (A::Multiple { answers: indexes }, Q::Multiple { answers, .. }) => {
                let count_answers = indexes.len();

                // The total number of actual correct answers
                let count_expected = answers.iter().filter(|value| value.correct).count();

                // Didn't provide enough answer or provided too many
                if count_answers < 1 || count_answers > count_expected {
                    return Score::Incorrect;
                }

                // Count the number of provided correct answers
                let count_correct = indexes
                    .iter()
                    .filter_map(|index| answers.get(*index))
                    .filter(|value| value.correct)
                    .count();

                if count_correct < 1 {
                    Score::Incorrect
                } else if count_correct == count_expected {
                    Score::Correct { value: base_score }
                } else {
                    // % correct out of total answers
                    let percent = count_correct as f32 / count_expected as f32;
                    let score = ((base_score as f32) * percent).round() as u32;
                    Score::Partial {
                        value: score,
                        count: count_correct as u32,
                        total: count_expected as u32,
                    }
                }
            }

            (
                A::TrueFalse { answer },
                Q::TrueFalse {
                    answer: actual_answer,
                },
            ) => {
                if *answer == *actual_answer {
                    Score::Correct { value: base_score }
                } else {
                    Score::Incorrect
                }
            }

            (
                A::Typer { answer },
                Q::Typer {
                    answers,
                    ignore_case,
                },
            ) => {
                let correct = if *ignore_case {
                    answers
                        .iter()
                        .any(|value| answer.eq_ignore_ascii_case(value))
                } else {
                    answers.iter().any(|value| answer.eq(value))
                };

                if correct {
                    Score::Correct { value: base_score }
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

impl PlayerSession {
    pub fn new(id: SessionId, addr: Addr<Session>, name: String, question_len: usize) -> Self {
        // Initialize the empty answers list
        let answers = vec![PlayerAnswer::default(); question_len];
        Self {
            id,
            addr,
            name,
            ready: false,
            answers,
            score: 0,
        }
    }
}

/// Configuration data for a game
#[derive(Serialize)]
pub struct GameConfig {
    /// The name of the game
    pub name: String,
    /// Text displayed under the game name
    pub text: String,
    /// Maximum number of players allowed in this game
    pub max_players: usize,
    /// Filtering on names
    #[serde(skip)]
    pub filtering: NameFiltering,
    /// The game questions
    #[serde(skip)]
    pub questions: Vec<Arc<Question>>,
    /// Map of uploaded image UUIDs to their respective
    /// image data
    #[serde(skip)]
    pub images: HashMap<ImageRef, Image>,
}
