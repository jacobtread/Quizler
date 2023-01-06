use std::collections::HashMap;

use actix::{Actor, ActorContext, Addr, StreamHandler};
use actix_web_actors::ws;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::game::{Game, GameId};

pub struct Session {
    /// Unique ID of the session
    id: SessionId,
    /// Address to the current game if apart of one
    game: Option<SessionGame>,
}

pub struct SessionGame {
    id: GameId,
    addr: Addr<Game>,
}

pub type SessionId = u32;

/// Messages recieved from the client
#[derive(Deserialize)]
#[serde(tag = "ty")]
pub enum ClientMessage {
    // Message to connect self to the game with the associated ID
    Connect {
        // The game token to try and connect to (e.g. W2133)
        try_connect: String,
        // Token provided to the client for connecting with a name
        token: String,
    },
}

/// Messages sent by the server
#[derive(Serialize)]
#[serde(tag = "ty")]
pub enum ServerMessage {
    /// Message indicating a complete successful connection
    Connected {
        // The name of the game
        name: String,
        // The ID of the current session
        id: SessionId,
    },
    /// Message providing information about another player in
    /// the game
    OtherPlayer { id: SessionId, name: String },
    /// Update for the player scores
    ScoreUpdate { scores: HashMap<SessionId, u32> },
}

#[derive(Serialize)]
#[repr(u8)]
pub enum ServerError {
    /// The last proivded message was malformed
    MalformedMessage = 0x0,
    /// The provided username is already in use
    UsernameTaken = 0x1,
}

type ServerResult = Result<ServerMessage, ServerError>;

impl Actor for Session {
    type Context = ws::WebsocketContext<Session>;
}

type SessionContext = ws::WebsocketContext<Session>;

impl Session {
    /// Writes a server message by encoding it to json and then sending it
    /// as a text message through the web socket context
    ///
    /// `ctx` The context to write to
    /// `msg` The message to write
    fn write_message<M: Serialize>(ctx: &mut SessionContext, msg: M) {
        // Serialize the message
        let value = match serde_json::to_string(&msg) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to encode server message as JSON");
                return;
            }
        };

        // Write the text frame
        ctx.text(value);
    }

    /// Handles a recieved client message
    fn handle_message(&mut self, message: ClientMessage, ctx: &mut SessionContext) {}
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // Handle protocol errors
        let message = match item {
            Ok(message) => message,
            Err(err) => {
                error!("Got error while recieving websocket messages: {:?}", err);
                return;
            }
        };

        // Only expect text messages
        let text = match message {
            ws::Message::Text(value) => value,
            ws::Message::Pong(ping) => {
                ctx.pong(&ping);
                return;
            }
            ws::Message::Close(reason) => {
                info!("Session connection closed: {:?}", reason);
                ctx.stop();
                return;
            }
            _ => return,
        };

        let value = match serde_json::from_str::<ClientMessage>(&*text) {
            Ok(value) => value,
            Err(err) => {
                error!("Unable to decode client message: {:?}", err);
                Self::write_message(ctx, ServerError::MalformedMessage);
                return;
            }
        };

        self.handle_message(value, ctx);
    }
}
