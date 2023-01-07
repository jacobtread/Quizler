use std::collections::HashMap;

use actix::{dev::MessageResponse, Actor, Addr, Context, Handler, Message};
use rand_core::{OsRng, RngCore};

use crate::{
    game::{BasicConfig, Game, GameConfig, GameRequest, GameResponse, GameTiming},
    session::{ServerError, ServerMessage, Session, SessionId, SessionRequest},
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

    fn try_connect(game: Addr<Game>, id: u32, name: String, addr: Addr<Session>) {
        tokio::spawn(async move {
            let game = game;
            let res = match game
                .send(GameRequest::TryConnect {
                    id,
                    name,
                    addr: addr.clone(),
                })
                .await
            {
                Ok(value) => value,
                Err(err) => {
                    error!("Failed to send join attempt: {:?}", err);
                    return;
                }
            };

            match res {
                Ok(GameResponse::Connected { id, basic, timing }) => {
                    addr.do_send(SessionRequest::Message(ServerMessage::Connected {
                        id,
                        basic,
                        timing,
                    }));
                }
                Ok(_) => {
                    error!("Unexpected games response message");
                }
                Err(err) => {
                    addr.do_send(SessionRequest::Error(err));
                }
            }
        });
    }
}

#[derive(Message)]
#[rtype(result = "Result<GamesResponse, ServerError>")]
pub enum GamesRequest {
    /// Request from the HTTP API to initialize a new game
    PreInitGame { config: GameConfig },

    /// Message for the host to connect to an un-initialized game
    HostConnect {
        id: u32,
        sess_id: SessionId,
        addr: Addr<Session>,
    },

    /// Message to attempt to connect to a game
    TryConnect {
        token: String,
        id: SessionId,
        name: String,
        addr: Addr<Session>,
    },
}

pub enum GamesResponse {
    /// Pre initialization complete
    PreInitComplete {
        id: u32,
    },

    /// The game was connected to successfully
    Connected {
        /// The game token,
        token: String,
        /// Basic game config information
        basic: BasicConfig,
        /// Timing data for different game events
        timing: GameTiming,
    },

    None,
}

impl Handler<GamesRequest> for Games {
    type Result = Result<GamesResponse, ServerError>;
    fn handle(&mut self, msg: GamesRequest, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            GamesRequest::PreInitGame { config } => {
                let id = self.pre_init_id;
                self.pre_init_id += 1;
                self.pre_init.insert(id, config);
                Ok(GamesResponse::PreInitComplete { id })
            }
            GamesRequest::HostConnect { id, sess_id, addr } => {
                // Find the config data from the pre init list
                let config = self.pre_init.remove(&id).ok_or(ServerError::InvalidToken)?;

                // Clone config data for response
                let timing = config.timing.clone();
                let basic = config.basic.clone();

                // Initialize and store the game
                let token = self.unique_token();
                let game = Game::new(token.clone(), sess_id, addr, config).start();
                self.games.insert(token.clone(), game);

                Ok(GamesResponse::Connected {
                    token,
                    basic,
                    timing,
                })
            }
            GamesRequest::TryConnect {
                token,
                id,
                name,
                addr,
            } => {
                let game = self
                    .games
                    .get(&token)
                    .ok_or(ServerError::InvalidToken)?
                    .clone();
                Self::try_connect(game, id, name, addr);
                Ok(GamesResponse::None)
            }
        }
    }
}

impl Actor for Games {
    type Context = Context<Self>;
}

impl<A, M> MessageResponse<A, M> for GamesResponse
where
    A: Actor,
    M: Message<Result = GamesResponse>,
{
    fn handle(
        self,
        _ctx: &mut <A as Actor>::Context,
        tx: Option<actix::dev::OneshotSender<<M as Message>::Result>>,
    ) {
        if let Some(tx) = tx {
            if tx.send(self).is_err() {
                error!("Failed to send games response");
            }
        }
    }
}
