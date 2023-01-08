use actix::{dev::MessageResponse, Actor, Addr, Context, Handler, Message};
use serde::{Deserialize, Serialize};

use crate::{
    error::ServerError,
    session::{ServerMessage, Session, SessionId, SessionRequest},
};
use log::error;

pub struct Game {
    /// The token this game is stored behind
    token: String,
    /// The host session
    host: HostSession,
    /// Map of session IDs mapped to the session address
    players: Vec<GameSession>,
    /// Configuration for the game
    config: GameConfig,
    /// The state of the game
    state: GameState,
}

#[derive(Serialize)]
#[repr(u8)]
pub enum GameState {
    /// The game is in the lobby
    Lobby = 0x0,
    /// The game is starting
    Starting = 0x1,
    /// The game has started
    Started = 0x2,
    /// The game has finished
    Finished = 0x3,
}

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
        }
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

impl Handler<GameRequest> for Game {
    type Result = Result<GameResponse, ServerError>;

    fn handle(&mut self, msg: GameRequest, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            GameRequest::TryConnect { id, name, addr } => {
                match self.state {
                    GameState::Started | GameState::Starting => {
                        return Err(ServerError::AlreadyStarted)
                    }
                    GameState::Finished => return Err(ServerError::AlreadyFinished),
                    _ => {}
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

                let game_player = GameSession { id, name, addr };

                // Notify existing players and host of joined player
                for player in &self.players {
                    player.notify_other(&game_player);
                }
                self.host.notify_other(&game_player);

                self.players.push(game_player);

                let config = &self.config;
                Ok(GameResponse::Connected {
                    id,
                    token: self.token.clone(),
                    basic: config.basic.clone(),
                    timing: config.timing.clone(),
                })
            }
            // Not yet implemented
            GameRequest::SkipTimer => Ok(GameResponse::None),
        }
    }
}

pub struct HostSession {
    /// The ID of the session
    id: SessionId,
    /// Address to the session
    addr: Addr<Session>,
}

impl HostSession {
    pub fn notify_other(&self, other: &GameSession) {
        self.addr
            .do_send(SessionRequest::Message(ServerMessage::OtherPlayer {
                id: other.id,
                name: other.name.clone(),
            }))
    }
}

pub struct GameSession {
    /// The ID of the session
    id: SessionId,
    /// The player name
    name: String,
    /// Address to the session
    addr: Addr<Session>,
}

impl GameSession {
    pub fn notify_other(&self, other: &GameSession) {
        self.addr
            .do_send(SessionRequest::Message(ServerMessage::OtherPlayer {
                id: other.id,
                name: other.name.clone(),
            }))
    }
}

pub type GameId = u32;

impl Actor for Game {
    type Context = Context<Self>;
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
    pub wait_time: u32,
    /// The time that a bonus score will be granted within
    /// bonus score is disabled if none
    pub bonus_score_time: u32,
}

/// Type for a string which represents a reference to a tmp stored image
pub type ImageRef = String;

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
    answer_time: u32,
    /// The point scoring for the question
    scoring: Scoring,
}

pub enum QuestionType {
    /// Single choice question
    Single {
        /// Vec of indexes of correct answers
        answers: Vec<u32>,
        /// Vec of the possible answers
        values: Vec<String>,
    },
    /// Multiple choice question
    Multiple {
        /// Vec of indexes of correct answers
        answers: Vec<u32>,
        /// Vec of the possible answers
        values: Vec<String>,
    },
    /// Image where you must click an area
    ClickableImage {
        /// The image url to take clicking on
        image: ImageRef,
        /// Top left box coordinate
        top: (f32, f32),
        /// Bottom right box coordinate
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
        ctx: &mut <A as Actor>::Context,
        tx: Option<actix::dev::OneshotSender<<M as Message>::Result>>,
    ) {
        if let Some(tx) = tx {
            if tx.send(self).is_err() {
                error!("Failed to send game response");
            }
        }
    }
}
