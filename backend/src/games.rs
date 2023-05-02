use crate::{
    error::ServerError,
    game::{Game, GameConfig},
    session::SessionRef,
};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, MessageResult};
use rand_core::{OsRng, RngCore};
use serde::Serialize;
use std::{collections::HashMap, fmt::Display, str::FromStr, sync::Arc};
use uuid::Uuid;

/// Central store for storing all the references to the individual
/// games that are currently running
#[derive(Default)]
pub struct Games {
    /// Map of the game tokens to the actual game itself
    games: HashMap<GameToken, Addr<Game>>,
    /// Map of UUID's to game configurations that are preparing to start
    preparing: HashMap<Uuid, GameConfig>,
}

impl Actor for Games {
    type Context = Context<Self>;
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct GameToken([u8; GameToken::LENGTH]);

impl GameToken {
    /// Length of tokens that will be created
    const LENGTH: usize = 5;
    /// Set of chars that can be used as game tokens
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    fn unique_token(games: &HashMap<GameToken, Addr<Game>>) -> GameToken {
        /// Length of the charset
        const RANGE: usize = GameToken::CHARSET.len();

        let mut rand = OsRng;
        let mut token = Self([0u8; Self::LENGTH]);

        loop {
            for at in token.0.iter_mut() {
                loop {
                    // Obtain a random number
                    let var = (rand.next_u32() >> (32 - 6)) as usize;

                    // If the value is in the charset break the loop
                    if var < RANGE {
                        *at = Self::CHARSET[var];
                        break;
                    }
                }
            }

            if !games.contains_key(&token) {
                return token;
            }
        }
    }
}

impl Serialize for GameToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Game tokens are simply serialized as strings by casting the type
        let token = unsafe { std::str::from_utf8_unchecked(&self.0) };
        serializer.serialize_str(token)
    }
}

impl FromStr for GameToken {
    type Err = ServerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != GameToken::LENGTH {
            return Err(ServerError::InvalidToken);
        }

        let bytes = s.as_bytes();

        // Handle invalid characters
        if bytes
            .iter()
            .any(|value| !GameToken::CHARSET.contains(value))
        {
            return Err(ServerError::InvalidToken);
        }

        let mut output = [0u8; GameToken::LENGTH];
        output.copy_from_slice(bytes);
        Ok(Self(output))
    }
}

impl Display for GameToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = unsafe { std::str::from_utf8_unchecked(&self.0) };
        f.write_str(token)
    }
}

/// Requests that the games manager prepares for initalizing
/// of a new game with the provided [`GameConfig`]. Responds
/// with a UUID that the host can use to start the game.
///
/// This request comes from the HTTP API
#[derive(Message)]
#[rtype(result = "Uuid")]
pub struct PrepareGameMessage {
    /// The configuration to store as prepared
    pub config: GameConfig,
}

impl Handler<PrepareGameMessage> for Games {
    type Result = MessageResult<PrepareGameMessage>;

    fn handle(&mut self, msg: PrepareGameMessage, _ctx: &mut Self::Context) -> Self::Result {
        let id = Uuid::new_v4();
        self.preparing.insert(id, msg.config);
        MessageResult(id)
    }
}

/// Message for handling the connection of a host to a preparing game.
/// This creates the actual game and is done through the WebSocket API
#[derive(Message)]
#[rtype(result = "Result<InitializedMessage, ServerError>")]
pub struct InitializeMessage {
    /// The UUID of the prepared game configuration to start
    pub uuid: Uuid,
    /// Reference to the session trying to connect
    pub session_ref: SessionRef,
}

/// Message containing the details of a game that has been successfully
/// connected to by the host (The game has finished being prepared)
pub struct InitializedMessage {
    /// The uniquely generated game token (e.g A3DLM)
    pub token: GameToken,
    /// The full game config to be used while playing
    pub config: Arc<GameConfig>,
    /// The address to the game
    pub game: Addr<Game>,
}

impl Handler<InitializeMessage> for Games {
    type Result = Result<InitializedMessage, ServerError>;

    fn handle(&mut self, msg: InitializeMessage, ctx: &mut Self::Context) -> Self::Result {
        // Find the config data from the pre init list
        let config = self
            .preparing
            .remove(&msg.uuid)
            .ok_or(ServerError::InvalidToken)?;

        let config = Arc::new(config);

        // Create a new game token
        let token = GameToken::unique_token(&self.games);

        let game = Game::new(token, msg.session_ref, config.clone(), ctx.address()).start();
        self.games.insert(token, game.clone());

        Ok(InitializedMessage {
            token,
            config: config.clone(),
            game,
        })
    }
}

/// Message to request an addr to a game
#[derive(Message)]
#[rtype(result = "Option<Addr<Game>>")]
pub struct GetGameMessage {
    /// The raw string token
    pub token: GameToken,
}

impl Handler<GetGameMessage> for Games {
    type Result = Option<Addr<Game>>;

    fn handle(&mut self, msg: GetGameMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.games.get(&msg.token).cloned()
    }
}

/// Message to remove a game
#[derive(Message)]
#[rtype(result = "()")]
pub struct RemoveGameMessage {
    /// The token of the game to remove
    pub token: GameToken,
}

impl Handler<RemoveGameMessage> for Games {
    type Result = ();

    fn handle(&mut self, msg: RemoveGameMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.games.remove(&msg.token);
    }
}
