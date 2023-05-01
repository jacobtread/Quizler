use crate::{
    error::ServerError,
    game::{
        AnswerResult, ConnectedMessage, Game, GameState, Question, QuestionAnswer, ReadyMessage,
        TryConnectMessage,
    },
    games::{GameToken, Games, GetGameMessage, InitializeMessage, InitializedMessage},
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Handler, Message, StreamHandler,
    WrapFuture,
};
use actix_web_actors::ws;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, future::Future, sync::Arc};
use uuid::Uuid;

pub type SessionId = u32;

pub struct Session {
    /// Unique ID of the session
    id: SessionId,
    /// Address to the current game if apart of one
    game: Option<Addr<Game>>,
}

/// Reference to a session, contains the ID of the
/// session along with the session addr
pub struct SessionRef {
    /// The ID of the referenced session
    pub id: SessionId,
    /// The addr to the session
    pub addr: Addr<Session>,
}

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
    /// Message sent to the host after they've initialized
    /// a game
    Initialized(InitializedMessage),

    /// Message indicating a complete successful connection
    Connected(ConnectedMessage),

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
    fn write_message(ctx: &mut SessionContext, msg: &ServerMessage) {
        // Serialize the message
        let value = match serde_json::to_string(msg) {
            Ok(value) => value,
            Err(err) => {
                error!("Failed to encode server message as JSON: {:?}", err);
                return;
            }
        };

        // Write the text frame
        ctx.text(value);
    }

    fn write_error(ctx: &mut SessionContext, error: ServerError) {
        Self::write_message(ctx, &ServerMessage::Error { error })
    }

    /// Handles a recieved client message
    fn handle_message(&mut self, message: ClientMessage, ctx: &mut SessionContext) {
        // Create a reference to the session
        let session_ref = SessionRef {
            id: self.id,
            addr: ctx.address(),
        };

        match message {
            // Handle initializing new games
            ClientMessage::InitializeGame { uuid } => {
                self.async_message(Self::initialize(session_ref, uuid), ctx);
            }

            // Handle try connect messages
            ClientMessage::TryConnect { token, username } => {
                self.async_message(Self::try_connect(session_ref, token, username), ctx);
            }

            // Handle client ready messages
            ClientMessage::Ready => {
                let game = match &self.game {
                    Some(value) => value.clone(),
                    None => {
                        // Expected the game to exist
                        Self::write_error(ctx, ServerError::Unexpected);
                        return;
                    }
                };
                game.do_send(ReadyMessage { id: session_ref.id });
            }
            _ => todo!(),
        }
    }

    fn async_message<F>(&self, future: F, ctx: &mut SessionContext)
    where
        F: Future<Output = Result<ServerMessage, ServerError>> + 'static,
    {
        let future = future.into_actor(self).map(|result, _act, ctx| {
            // Handle error cases
            let msg = match result {
                Ok(msg) => msg,
                Err(error) => ServerMessage::Error { error },
            };

            // Write the response
            Self::write_message(ctx, &msg);
        });

        ctx.spawn(future);
    }

    async fn initialize(session_ref: SessionRef, uuid: Uuid) -> Result<ServerMessage, ServerError> {
        let games = Games::get();
        let msg: InitializedMessage = games
            .send(InitializeMessage { uuid, session_ref })
            .await
            .expect("Games service was stopped")?;
        Ok(ServerMessage::Initialized(msg))
    }

    async fn try_connect(
        session_ref: SessionRef,
        token: String,
        name: String,
    ) -> Result<ServerMessage, ServerError> {
        // Parse the token
        let token: GameToken = token.parse()?;

        // Attempting to connect to AED32E as Jacob
        debug!("Attempting to connect to {} as {}", token, name);

        // Obtain a addr to the game
        let games = Games::get();
        let game_addr = games
            .send(GetGameMessage { token })
            .await
            .expect("Games service was stopped")
            .ok_or(ServerError::InvalidToken)?;

        // Attempt the connection
        let msg: ConnectedMessage = game_addr
            .send(TryConnectMessage { session_ref, name })
            .await
            .map_err(|_| ServerError::InvalidToken)??;

        Ok(ServerMessage::Connected(msg))
    }
}

impl Handler<ServerMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) -> Self::Result {
        Self::write_message(ctx, &msg);
    }
}

impl Handler<Arc<ServerMessage>> for Session {
    type Result = ();

    fn handle(&mut self, msg: Arc<ServerMessage>, ctx: &mut Self::Context) -> Self::Result {
        Self::write_message(ctx, &*msg);
    }
}

impl Handler<ServerError> for Session {
    type Result = ();

    fn handle(&mut self, msg: ServerError, ctx: &mut Self::Context) -> Self::Result {
        Self::write_error(ctx, msg);
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
                Self::write_error(ctx, ServerError::MalformedMessage);
                return;
            }
        };

        Self::handle_message(self, value, ctx)
    }
}
