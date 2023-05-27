//! Contains the definitions of the Client and Server packets

use crate::{
    game::{GameConfig, GameState},
    games::GameToken,
    session::SessionId,
    types::{Answer, HostAction, Question, RemoveReason, Score, ServerError},
};
use actix::Message;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ClientRequest {
    pub rid: u32,
    #[serde(flatten)]
    pub msg: ClientMessage,
}

#[derive(Serialize)]
pub struct ServerResponse {
    pub rid: u32,
    #[serde(flatten)]
    pub msg: ServerMessage,
}

/// Messages recieved from the client
#[derive(Deserialize)]
#[serde(tag = "ty")]
pub enum ClientMessage {
    // Message to initialize the desired game as a host
    Initialize {
        /// The UUID of the game to initialize
        uuid: Uuid,
    },

    // Message to associate the session with the provided game
    Connect {
        /// The game token to try and connect to (e.g. W2133)
        token: String,
    },

    /// Message to attempt to join the game using the provided name
    Join {
        /// The name to attempt to access with
        name: String,
    },

    /// Message indicating the client is ready to play
    ///
    /// (This is done internally by clients once everything has been loaded)
    Ready,
    /// Message for actions from the host session
    HostAction { action: HostAction },
    /// Message to answer the question
    Answer { answer: Answer },
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
    /// Message indicating a complete successful connection
    Joined {
        /// The session ID
        id: SessionId,
        /// The uniquely generated game token (e.g A3DLM)
        token: GameToken,
        /// Copy of the game configuration to send back
        config: Arc<GameConfig>,
    },
    Ok,
    /// Message providing information about another player in
    /// the game
    PlayerData {
        id: SessionId,
        name: String,
    },
    /// Message indicating the current state of the game
    GameState {
        state: GameState,
    },
    /// Message for telling clients the current countdown timer
    Timer {
        value: u32,
    },
    /// Question data for the next question
    Question {
        question: Arc<Question>,
    },
    /// Updates the player scores with the new scores
    Scores {
        scores: HashMap<SessionId, u32>,
    },

    /// Message telling the player the score that they obtained
    Score {
        score: Score,
    },

    /// Server error
    Error {
        error: ServerError,
    },
    /// Player has been kicked from the game
    Kicked {
        /// The ID of the player that was kicked
        id: SessionId,
        /// The reason the player was kicked
        reason: RemoveReason,
    },
}
