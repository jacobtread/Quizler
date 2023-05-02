//! Contains the definitions of the Client and Server packets

use crate::{
    game::{GameConfig, GameState, TimeSync},
    games::GameToken,
    session::SessionId,
    types::{Answer, HostAction, KickReason, PlayerGameConfig, Question, ServerError},
};
use actix::Message;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Messages recieved from the client
#[derive(Deserialize)]
#[serde(tag = "ty")]
pub enum ClientMessage {
    // Message to initialize the desired game as a host
    Initialize {
        /// The UUID of the game to initialize
        uuid: Uuid,
    },
    // Message to connect self to the game with the associated ID
    Connect {
        // The game token to try and connect to (e.g. W2133)
        token: String,
        // The username to try and connect with
        name: String,
    },
    /// Message indicating the client is ready to play
    ///
    /// (This is done internally by clients once everything has been loaded)
    Ready,
    /// Message for actions from the host session
    HostAction { action: HostAction },
    /// Message to answer the question
    Answer(Answer),
    /// Message for the host to kick a player from the game
    Kick {
        /// The ID of the player to kick
        id: SessionId,
    },
}

/// Messages sent by the server
#[derive(Message, Serialize)]
#[rtype(result = "()")]
#[serde(tag = "ty")]
pub enum ServerMessage {
    /// Message sent to the host after they've initialized
    /// a game
    Initialized {
        /// The uniquely generated game token (e.g A3DLM)
        token: GameToken,
        /// The full game config to be used while playing
        config: Arc<GameConfig>,
    },
    /// Message indicating a complete successful connection
    Connected {
        /// The session ID
        id: SessionId,
        /// The uniquely generated game token (e.g A3DLM)
        token: GameToken,
        /// Copy of the game configuration to send back
        config: PlayerGameConfig,
    },
    /// Message providing information about another player in
    /// the game
    OtherPlayer { id: SessionId, name: String },
    /// Message indicating the current state of the game
    GameState { state: GameState },
    /// Message for syncing the time between the game and clients
    TimeSync(TimeSync),
    /// Question data for the next question
    Question(Arc<Question>),
    /// Updates the player scores with the new scores
    Scores { scores: HashMap<SessionId, u32> },
    /// Server error
    Error { error: ServerError },
    /// Player has been kicked from the game
    Kicked {
        /// The ID of the player that was kicked
        session_id: SessionId,
        /// The reason the player was kicked
        reason: KickReason,
    },
}
