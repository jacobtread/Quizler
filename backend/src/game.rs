use std::collections::HashMap;

use actix::{dev::MessageResponse, Actor, Addr, Context, Handler, Message};
use actix_web::cookie::time::Duration;
use serde::{Deserialize, Serialize};

use crate::session::{ServerError, ServerMessage, Session, SessionId, SessionRequest};
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

    fn handle(&mut self, msg: GameRequest, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            GameRequest::TryConnect { id, name, addr } => {
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
    pub scoring: GameScoring,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BasicConfig {
    pub name: String,
    pub text: String,
}

pub struct GameScoring {
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
