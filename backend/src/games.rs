use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler, Message, MessageResult, ResponseFuture};
use rand_core::{OsRng, RngCore};

use crate::{
    error::ServerError,
    game::{BasicConfig, ConnectedMessage, Game, GameConfig, GameTiming},
    session::{ServerMessage, Session, SessionId},
};
use log::error;

/// Central store for storing all the references to the individual
/// games that are currently running
pub struct Games {
    /// Map of the game tokens to the actual game itself
    games: HashMap<String, Addr<Game>>,

    /// The next ID for pre init values
    pre_init_id: u32,

    /// Uninitialized games
    pre_init: HashMap<u32, GameConfig>,
}

impl Actor for Games {
    type Context = Context<Self>;
}

impl Games {
    /// The length of game tokens
    const TOKEN_LENGTH: usize = 5;

    /// Generates a unique token not used by any other games stored
    /// in the games map
    fn unique_token(&self) -> String {
        loop {
            let token = Self::random_token();
            if !self.games.contains_key(&token) {
                return token;
            }
        }
    }

    /// Generates a random token from the charset
    fn random_token() -> String {
        /// Available chars to create the token from
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        /// The number of chars in the charset
        const RANGE: u32 = 36;

        let mut rand = OsRng;
        let mut out = String::with_capacity(Self::TOKEN_LENGTH);

        // Loop until the string length is finished
        for _ in 0..Self::TOKEN_LENGTH {
            // Loop until a valid random is found
            loop {
                let var = rand.next_u32() >> (32 - 6);
                if var < RANGE {
                    out.push(char::from(CHARSET[var as usize]));
                    break;
                }
            }
        }

        out
    }
}

/// Request from the HTTP API to initialize a new game
#[derive(Message)]
#[rtype(result = "Result<PreInitCompleteMessage, ServerError>")]
pub struct PreInitGameMessage {
    config: GameConfig,
}

pub struct PreInitCompleteMessage {
    id: u32,
}

impl Handler<PreInitGameMessage> for Games {
    type Result = MessageResult<PreInitGameMessage>;

    fn handle(&mut self, msg: PreInitGameMessage, ctx: &mut Self::Context) -> Self::Result {
        let id = self.pre_init_id;
        self.pre_init_id += 1;
        self.pre_init.insert(id, msg.config);
        MessageResult(Ok(PreInitCompleteMessage { id }))
    }
}

/// Message for the host to connect to an un-initialized game

#[derive(Message)]
#[rtype(result = "Result<HostConnectedMessage, ServerError>")]
pub struct HostConnectMessage {
    id: u32,
    sess_id: SessionId,
    addr: Addr<Session>,
}

/// The game was connected to successfully
pub struct HostConnectedMessage {
    /// The game token,
    token: String,
    /// Basic game config information
    basic: BasicConfig,
    /// Timing data for different game events
    timing: GameTiming,
}

impl Handler<HostConnectMessage> for Games {
    type Result = MessageResult<HostConnectMessage>;

    fn handle(&mut self, msg: HostConnectMessage, ctx: &mut Self::Context) -> Self::Result {
        let HostConnectMessage { id, sess_id, addr } = msg;

        // Find the config data from the pre init list
        let config = match self.pre_init.remove(&id) {
            Some(value) => value,
            None => return MessageResult(Err(ServerError::InvalidToken)),
        };

        // Clone config data for response
        let timing = config.timing.clone();
        let basic = config.basic.clone();

        // Initialize and store the game
        let token = self.unique_token();
        let game = Game::new(token.clone(), sess_id, addr, config).start();
        self.games.insert(token.clone(), game);

        MessageResult(Ok(HostConnectedMessage {
            token,
            basic,
            timing,
        }))
    }
}

/// Message to attempt to connect to a game
#[derive(Message)]
#[rtype(result = "Result<(), ServerError>")]
pub struct TryConnectMessage {
    token: String,
    id: SessionId,
    name: String,
    addr: Addr<Session>,
}
impl Handler<TryConnectMessage> for Games {
    type Result = ResponseFuture<Result<(), ServerError>>;

    fn handle(&mut self, msg: TryConnectMessage, ctx: &mut Self::Context) -> Self::Result {
        let TryConnectMessage {
            token,
            id,
            name,
            addr,
        } = msg;

        let game = self.games.get(&token).cloned();

        Box::pin(async move {
            let game = game.ok_or(ServerError::InvalidToken)?;

            let msg = super::game::TryConnectMessage {
                id,
                name,
                addr: addr.clone(),
            };

            let result = match game.send(msg).await {
                Ok(value) => value,
                Err(err) => {
                    error!("Failed to send join attempt: {:?}", err);
                    return Err(ServerError::NotJoinable);
                }
            };

            let ConnectedMessage {
                token,
                id,
                basic,
                timing,
            } = result?;

            addr.do_send(ServerMessage::Connected {
                token,
                id,
                basic,
                timing,
            });

            Ok(())
        })
    }
}
