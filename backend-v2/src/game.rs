use crate::{
    games::Games,
    msg::ServerEvent,
    session::{EventTarget, SessionId},
    types::{
        Answer, AnswerData, GameToken, HostAction, ImStr, Image, ImageRef, NameFiltering, Question,
        QuestionData, RemoveReason, Score, ScoreCollection, ServerError,
    },
};
use log::debug;
use rustrict::CensorStr;
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::RwLock, task::AbortHandle, time::sleep};
use uuid::Uuid;

/// Reference to a game behind an Arc and a RwLock
pub type GameRef = Arc<RwLock<Game>>;

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
    task_handle: Option<AbortHandle>,

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

    /// The game has completely stopped
    Stopped,
}

impl Game {
    /// Creates a new game instance
    pub fn new(
        token: GameToken,
        host_id: SessionId,
        host_addr: EventTarget,
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
    fn timed_next_state(&mut self, duration: Duration) {
        let token = self.token;
        let handle = tokio::spawn(async move {
            sleep(duration).await;
            let game = Games::get_game(&token).await;
            if let Some(game) = game {
                let lock = &mut *game.write().await;
                lock.task_handle = None;
                lock.next_state();
            }
        });

        self.task_handle = Some(handle.abort_handle());

        // Send timer message with the duration time
        self.send_all(ServerEvent::Timer {
            value: duration.as_millis() as u32,
        });
    }

    /// Cancels a expected task if the handle is present
    fn cancel_task(&mut self) {
        if let Some(task_handle) = self.task_handle.take() {
            task_handle.abort();
        }
    }

    /// Moves the game to the next state based on its current state
    fn next_state(&mut self) {
        // If a task handle still exists cancel it
        self.cancel_task();

        match self.state {
            // Next state after lobby is starting
            GameState::Lobby => {
                const START_DURATION: Duration = Duration::from_secs(5);

                self.set_state(GameState::Starting);
                self.timed_next_state(START_DURATION);
            }

            // Next state after starting is question
            GameState::Starting => {
                self.question();
            }

            // Next state after awaiting ready is pre question
            GameState::AwaitingReady => {
                const START_DURATION: Duration = Duration::from_secs(5);
                self.set_state(GameState::PreQuestion);
                self.timed_next_state(START_DURATION);
            }

            // Next state after pre-question is awaiting answers
            GameState::PreQuestion => {
                // Await answers for the question
                self.set_state(GameState::AwaitingAnswers);

                // Assign the question start time
                self.start_time = Instant::now();

                let question = &self.config.questions[self.question_index];
                self.timed_next_state(Duration::from_millis(question.answer_time));
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

            GameState::Stopped => {}
        }
    }

    /// Send a message to all clients
    fn send_all(&self, message: ServerEvent) {
        // Wrap the message in an Arc to prevent cloning lots of heap data
        let message = Arc::new(message);

        // Send the message to all the players
        for player in &self.players {
            player.addr.send_shared(message.clone());
        }

        // Send the message to the host
        self.host.addr.send_shared(message);
    }

    /// Sets the current game state to the provided `state` and
    /// sends a state update message to all the clients including
    /// the host
    fn set_state(&mut self, state: GameState) {
        self.state = state;
        self.send_all(ServerEvent::GameState { state });
    }

    /// Resets the game state and all the player data to its initial values
    fn reset_completely(&mut self) {
        self.cancel_task();

        self.question_index = 0;

        for player in self.players.iter_mut() {
            // Reset the player answers
            player.answers.reset();

            // Reset the player score
            player.score = 0;
        }

        self.set_state(GameState::Lobby);
    }

    /// Handles updating state post removing a player
    fn on_remove(&mut self) {
        self.update_ready();

        // Reset the game if everyone disconected while in progress
        if self.state != GameState::Finished && self.players.is_empty() {
            self.reset_completely();
        }
    }

    /// Updates the current state checking if all the players are ready
    /// then if they are progresses the state to [`GameState::AwaitingAnswers`]
    fn update_ready(&mut self) {
        // Ignore if we aren't expecting ready states
        if self.state != GameState::AwaitingReady {
            return;
        }

        // Check all players are ready
        let all_ready = self.players.iter().all(|player| player.ready);
        if !all_ready || !self.host.ready {
            return;
        }

        self.next_state()
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

    fn question(&mut self) {
        // Reset ready states for the players
        self.players
            .iter_mut()
            .for_each(|player| player.ready = false);

        // Reset host ready state
        self.host.ready = false;

        // Obtain the current question
        let question = self.config.questions[self.question_index].clone();

        // Send the question contents to the clients
        self.send_all(ServerEvent::Question { question });

        // Begin awaiting for ready messages
        self.set_state(GameState::AwaitingReady);
    }

    /// Task for marking the answers
    fn mark_answers(&mut self) {
        // Get the current question
        let question = &self.config.questions[self.question_index];

        let scores: Vec<(SessionId, u32)> = self
            .players
            .iter_mut()
            .map(|player| {
                let answer = player.answers.get_answer(self.question_index);
                let score = answer.mark(question);

                // Increase the player score
                player.score += score.value();

                player.addr.send(ServerEvent::Score { score });

                (player.id, player.score)
            })
            .collect();
        let scores = ScoreCollection(scores);

        // Update everyones scores
        self.send_all(ServerEvent::Scores { scores });

        // Set state to marked
        self.set_state(GameState::Marked);
    }

    fn stop(&mut self) {
        // Don't try and stop the game twice
        if let GameState::Stopped = &self.state {
            return;
        }

        debug!("Game stopped: {}", self.token);

        let token = self.token;

        tokio::spawn(async move {
            // Remove the game from the list of games
            Games::remove_game(&token).await;
        });

        // Tell all the players they've been kicked
        for player in &self.players {
            // Send the visual kick message
            player.addr.send(ServerEvent::Kicked {
                id: player.id,
                reason: RemoveReason::HostDisconnect,
            });
        }

        self.host.addr.send(ServerEvent::Kicked {
            id: self.host.id,
            reason: RemoveReason::HostDisconnect,
        });

        self.state = GameState::Stopped;
    }

    pub fn try_join(
        &mut self,
        id: SessionId,
        listener: EventTarget,
        name: String,
    ) -> Result<JoinedMessage, ServerError> {
        // Trim name padding
        let name = name.trim();

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
        let game_player =
            PlayerSession::new(id, listener, Box::from(name), self.config.questions.len());

        // Message sent to existing players for this player
        let joiner_message = Arc::new(ServerEvent::PlayerData {
            id: game_player.id,
            name: game_player.name.clone(),
        });

        // Notify all players of the existence of eachother
        for player in &self.players {
            player.addr.send_shared(joiner_message.clone());

            // Message describing the other player
            game_player.addr.send(ServerEvent::PlayerData {
                id: player.id,
                name: player.name.clone(),
            });
        }

        // Notify the host of the join
        self.host.addr.send_shared(joiner_message);

        self.players.push(game_player);

        Ok(JoinedMessage {
            token: self.token,
            config: self.config.clone(),
        })
    }

    pub fn host_action(&mut self, id: SessionId, action: HostAction) -> Result<(), ServerError> {
        // Handle messages that aren't from the game host
        if self.host.id != id {
            return Err(ServerError::InvalidPermission);
        }

        match action {
            HostAction::Reset => self.reset_completely(),
            HostAction::Next => self.next_state(),
        };

        Ok(())
    }

    pub fn ready(&mut self, id: SessionId) {
        if id == self.host.id {
            self.host.ready = true;
        } else {
            let player = self.players.iter_mut().find(|player| player.id == id);
            if let Some(player) = player {
                player.ready = true;
            }
        }

        self.update_ready();
    }

    pub fn get_image(&self, uuid: Uuid) -> Option<Image> {
        self.config.images.get(&uuid).cloned()
    }

    pub fn remove_player(
        &mut self,
        id: SessionId,
        target_id: SessionId,
        mut reason: RemoveReason,
    ) -> Result<(), ServerError> {
        // Handle messages that aren't from the game host
        if target_id != id && self.host.id != id {
            return Err(ServerError::InvalidPermission);
        }

        // Host is removing itself (Game is stopping)
        if target_id == self.host.id {
            // Stop the game
            self.stop();
            return Ok(());
        }

        // Find the player position
        let index = self
            .players
            .iter()
            .position(|player| player.id == target_id)
            .ok_or(ServerError::UnknownPlayer)?;

        // Replace host remove reason for non hosts
        if RemoveReason::RemovedByHost == reason && id != self.host.id {
            reason = RemoveReason::Disconnected;
        }

        let kick_msg = Arc::new(ServerEvent::Kicked {
            id: target_id,
            reason,
        });

        // Inform each player of the removal
        self.players
            .iter()
            .for_each(|player| player.addr.send_shared(kick_msg.clone()));

        // Inform the host of the player removal
        self.host.addr.send_shared(kick_msg);

        // Remove the player
        self.players.remove(index);

        self.on_remove();

        Ok(())
    }

    pub fn answer(&mut self, id: SessionId, answer: Answer) -> Result<(), ServerError> {
        let elapsed = self.start_time.elapsed();

        // Answers are not being accepted at the current time
        if self.state != GameState::AwaitingAnswers {
            return Err(ServerError::UnexpectedMessage);
        }

        let question = &self.config.questions[self.question_index];

        // Find the player within the game
        let player = self
            .players
            .iter_mut()
            .find(|player| player.id == id)
            .ok_or(ServerError::UnknownPlayer)?;

        // Ensure the answer is the right type of answer
        if !answer.is_valid(&question.data) {
            return Err(ServerError::InvalidAnswer);
        }

        // Set the player answer
        player
            .answers
            .set_answer(self.question_index, AnswerData { elapsed, answer });

        // If all the players have answered we can advance the state
        let all_answered = self
            .players
            .iter()
            .all(|player| player.answers.has_answer(self.question_index));

        if all_answered {
            self.next_state();
        }

        Ok(())
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        debug!("Game dropped: {}", self.token);
    }
}

/// Message containing the connected details for a connected player
pub struct JoinedMessage {
    /// The uniquely generated game token (e.g A3DLM)
    pub token: GameToken,
    /// Copy of the game configuration to send back
    pub config: Arc<GameConfig>,
}

pub struct HostSession {
    /// The ID of the referenced session
    id: SessionId,
    /// The addr to the session
    addr: EventTarget,
    /// The player ready state
    ready: bool,
}

impl HostSession {
    pub fn new(id: SessionId, addr: EventTarget) -> Self {
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
    addr: EventTarget,
    /// The player ready state
    ready: bool,

    /// The player name
    name: ImStr,
    /// The players answers and the score they got for them
    answers: PlayerAnswers,
    /// The player total score
    score: u32,
}

impl PlayerSession {
    pub fn new(id: SessionId, addr: EventTarget, name: ImStr, question_len: usize) -> Self {
        Self {
            id,
            addr,
            name,
            ready: false,
            answers: PlayerAnswers::new(question_len),
            score: 0,
        }
    }
}

/// Structure storing the player answers. Fixed length to
/// the total number of questions in the game
struct PlayerAnswers {
    /// The actual player answers
    values: Box<[PlayerAnswer]>,
}

impl PlayerAnswers {
    /// Creates a new player answers structure of the
    /// provided length
    ///
    /// # Arguments
    /// * length - The length of the answers
    fn new(length: usize) -> Self {
        // Create all the answers collecting into the boxed slice
        let values: Box<[PlayerAnswer]> = (0..length).map(|_| PlayerAnswer::default()).collect();
        Self { values }
    }

    /// Resets the state of each player answer replacing the
    /// score and answer data with None
    fn reset(&mut self) {
        self.values.iter_mut().for_each(|value| {
            value.data = None;
            value.score = None;
        })
    }

    /// Sets the player answer at the provided index to the
    /// provided value
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    /// * answer - The answer to set the value to
    fn set_answer(&mut self, index: usize, answer: AnswerData) {
        debug_assert!(index < self.values.len());
        self.values[index].data = Some(answer);
    }

    /// Provides mutable access to the player answer at the provided
    /// index
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    fn get_answer(&mut self, index: usize) -> &mut PlayerAnswer {
        debug_assert!(index < self.values.len());
        &mut self.values[index]
    }

    /// Checks if theres an answer stored at the provided index
    ///
    /// # Arguments
    /// * index - The index of the answer within the values array
    fn has_answer(&self, index: usize) -> bool {
        debug_assert!(index < self.values.len());
        self.values[index].data.is_some()
    }
}

#[derive(Default)]
struct PlayerAnswer {
    /// The answer provided by the player
    data: Option<AnswerData>,
    /// The score provided by the server
    score: Option<Score>,
}

impl PlayerAnswer {
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
                // Trim extra whitespace
                let answer = answer.trim();

                let correct = if *ignore_case {
                    answers
                        .iter()
                        .any(|value| answer.eq_ignore_ascii_case(value))
                } else {
                    answers.iter().any(|value| answer.eq(value.as_ref()))
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

/// Configuration data for a game
#[derive(Serialize)]
pub struct GameConfig {
    /// The name of the game
    pub name: ImStr,
    /// Text displayed under the game name
    pub text: ImStr,
    /// Maximum number of players allowed in this game
    pub max_players: usize,
    /// Filtering on names
    #[serde(skip)]
    pub filtering: NameFiltering,
    /// The game questions
    #[serde(skip)]
    pub questions: Box<[Arc<Question>]>,
    /// Map of uploaded image UUIDs to their respective
    /// image data
    #[serde(skip)]
    pub images: HashMap<ImageRef, Image>,
}
