use crate::{
    error::ServerError,
    game::{
        AnswerResult, ConnectedMessage, Game, GameId, GameState, Question, QuestionAnswer,
        TryConnectMessage,
    },
    games::{GameToken, Games, GetGameMessage, HostConnectedMessage},
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

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
    // Message to initialize the desired game as a host
    InitializeGame {
        /// The UUID of the game to initialize
        uuid: Uuid,
    },
    // Message to connect self to the game with the associated ID
    TryConnect {
        // The game token to try and connect to (e.g. W2133)
        token: String,
        // The username to try and connect with
        username: String,
    },
    /// Message indicating the client is ready to play
    Ready,
    /// Message to start the game
    Start,
    /// Message to cancel starting the game
    Cancel,
    /// Message to answer the question
    Answer(QuestionAnswer),
}

/// Messages sent by the server
#[derive(Message, Serialize)]
#[rtype(result = "()")]
#[serde(tag = "ty")]
pub enum ServerMessage {
    /// Message indicating a complete successful connection
    Connected(ConnectedMessage),

    /// Message sent to the host after they've connected to
    /// and create a game
    HostConnected(HostConnectedMessage),

    /// Message providing information about another player in
    /// the game
    OtherPlayer { id: SessionId, name: String },

    /// Message indicating the current state of the game
    GameState(GameState),

    /// Message for syncing the time between the game and clients
    TimeSync {
        /// The total time that is being waited for
        total: u64,
        /// The time that has already passed
        elapsed: u64,
    },

    /// Question data for the next question
    Question(Arc<Question>),

    /// Result message for showing the results of a player
    AnswerResult(AnswerResult),

    /// Message to begin the question displaying the answers
    /// at the bottom for the user to choose
    BeginQuestion,

    /// Update for the player scores
    ScoreUpdate { scores: HashMap<SessionId, u32> },

    /// Server error
    Error { error: ServerError },
}

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
                error!("Failed to encode server message as JSON: {:?}", err);
                return;
            }
        };

        // Write the text frame
        ctx.text(value);
    }

    /// Handles a recieved client message
    fn handle_message(&mut self, message: ClientMessage, ctx: &mut SessionContext) {
        match message {
            ClientMessage::TryConnect { token, username } => {
                let addr = ctx.address();
                let id = self.id;

                // Spawn the connection attempt
                actix_rt::spawn(async move {
                    // Copy of the address used to send the return message
                    let ret_addr = addr.clone();

                    // Attempt to connect
                    let _ = match Self::try_connect(addr, id, token, username).await {
                        Ok(msg) => ret_addr.do_send(ServerMessage::Connected(msg)),
                        Err(err) => ret_addr.do_send(err),
                    };
                });
            }
            ClientMessage::Ready => todo!(),
            _ => todo!(),
        }
    }

    async fn try_connect(
        addr: Addr<Session>,
        id: SessionId,
        token: String,
        name: String,
    ) -> Result<ConnectedMessage, ServerError> {
        // Parse the token
        let token: GameToken = token.parse()?;

        // Attempting to connect to AED32E as Jacob (1)
        debug!("Attempting to connect to {} as {} ({})", token, name, id);

        // Obtain a addr to the game
        let games = Games::get();
        let game_addr = games
            .send(GetGameMessage { token })
            .await
            .expect("Games service was stopped")
            .ok_or(ServerError::InvalidToken)?;

        // Attempt the connection
        game_addr
            .send(TryConnectMessage { addr, id, name })
            .await
            .map_err(|_| ServerError::InvalidToken)?
    }
}

impl Handler<ServerMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) -> Self::Result {
        Self::write_message(ctx, msg);
    }
}
impl Handler<Arc<ServerMessage>> for Session {
    type Result = ();

    fn handle(&mut self, msg: Arc<ServerMessage>, ctx: &mut Self::Context) -> Self::Result {
        Self::write_message(ctx, msg);
    }
}

impl Handler<ServerError> for Session {
    type Result = ();

    fn handle(&mut self, msg: ServerError, ctx: &mut Self::Context) -> Self::Result {
        Self::write_message(ctx, ServerMessage::Error { error: msg });
    }
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

        // Decode the recieved client message
        let value = match serde_json::from_str::<ClientMessage>(&*text) {
            Ok(value) => value,
            Err(err) => {
                error!("Unable to decode client message: {:?}", err);
                Self::write_message(ctx, ServerError::MalformedMessage);
                return;
            }
        };

        // Handle the client message
        self.handle_message(value, ctx);
    }
}
