use crate::{
    error::ServerError,
    game::{
        AnswerResult, CancelMessage, ConnectedMessage, Game, GameState, Question, QuestionAnswer,
        ReadyMessage, RemovePlayerMessage, SkipTimerMessage, StartMessage, TryConnectMessage,
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
    pub id: SessionId,
    /// Address to the current game if apart of one
    pub game: Option<Addr<Game>>,
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
    Initialize {
        /// The UUID of the game to initialize
        uuid: Uuid,
    },

    // Message to connect self to the game with the associated ID
    Connect {
        // The game token to try and connect to (e.g. W2133)
        token: String,
        // The username to try and connect with
        username: String,
    },

    /// Message indicating the client is ready to play
    ///
    /// (This is done internally by clients once everything has been loaded)
    Ready,
    /// Message to start the game
    Start,
    /// Message to cancel starting the game
    Cancel,
    /// Message to answer the question
    Answer(QuestionAnswer),
    /// Message for the host to skip the current question
    Skip,
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
        total: u32,
        /// The time that has already passed
        elapsed: u32,
    },

    /// Question data for the next question
    Question(Arc<Question>),

    /// Result message for showing the results of a player
    AnswerResult(AnswerResult),

    /// Update for the player scores
    ScoreUpdate { scores: HashMap<SessionId, u32> },

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

/// Message send to sessions to inform them that they've
/// been removed from their game
#[derive(Message, Debug, Copy, Clone, Serialize)]
#[rtype(result = "()")]
pub struct KickMessage;

impl Handler<KickMessage> for Session {
    type Result = ();

    fn handle(&mut self, _: KickMessage, ctx: &mut Self::Context) -> Self::Result {
        // Clear the active game so we don't attempt to send
        // a LostConnection message
        self.game = None;

        // Session is stopped now that they aren't in a game
        ctx.stop();
    }
}

#[derive(Message, Debug, Copy, Clone, Serialize)]
#[rtype(result = "()")]
#[repr(u8)]
pub enum KickReason {
    /// Player was manually kicked by the host
    RemovedByHost = 0x1,
    /// The host diconnected ending the game
    HostDisconnect = 0x2,
    /// Connection was lost to the player
    LostConnection = 0x3,
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Session>;

    /// Handle the session being stopped by removing the
    /// session from any games and cleaning up after it
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // Take the game to attempt removing if present
        if let Some(game) = self.game.take() {
            // Inform game to remove self
            game.do_send(RemovePlayerMessage {
                session_ref: None,
                target_id: self.id,
                reason: KickReason::LostConnection,
            });
        }
    }
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
            ClientMessage::Initialize { uuid } => {
                self.async_message(Self::initialize(session_ref, uuid), ctx);
            }

            // Handle try connect messages
            ClientMessage::Connect { token, username } => {
                self.async_message(Self::try_connect(session_ref, token, username), ctx);
            }

            // Handle message to start game
            ClientMessage::Start => {
                let game = match &self.game {
                    Some(value) => value.clone(),
                    None => {
                        // Expected the game to exist
                        Self::write_error(ctx, ServerError::Unexpected);
                        return;
                    }
                };
                game.do_send(StartMessage { session_ref });
            }

            // Handle message to cancel starting game
            ClientMessage::Cancel => {
                let game = match &self.game {
                    Some(value) => value.clone(),
                    None => {
                        // Expected the game to exist
                        Self::write_error(ctx, ServerError::Unexpected);
                        return;
                    }
                };
                game.do_send(CancelMessage { session_ref });
            }

            // Handle message for an answer to the current question
            ClientMessage::Answer(_answer) => todo!(),

            // Handle message for kicking a player
            ClientMessage::Kick { id } => {
                let game = match &self.game {
                    Some(value) => value.clone(),
                    None => {
                        // Expected the game to exist
                        Self::write_error(ctx, ServerError::Unexpected);
                        return;
                    }
                };
                game.do_send(RemovePlayerMessage {
                    session_ref: Some(session_ref),
                    target_id: id,
                    reason: KickReason::RemovedByHost,
                });
            }

            // Handle message for skipping the current question
            ClientMessage::Skip => {
                let game = match &self.game {
                    Some(value) => value.clone(),
                    None => {
                        // Expected the game to exist
                        Self::write_error(ctx, ServerError::Unexpected);
                        return;
                    }
                };
                game.do_send(SkipTimerMessage { session_ref });
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

/// Handle writing server messages
impl Handler<ServerMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) -> Self::Result {
        Self::write_message(ctx, &msg);
    }
}

/// Handle writing shared references to a server message
impl Handler<Arc<ServerMessage>> for Session {
    type Result = ();

    fn handle(&mut self, msg: Arc<ServerMessage>, ctx: &mut Self::Context) -> Self::Result {
        Self::write_message(ctx, &*msg);
    }
}

/// Handle server error messages sent to the session by forwarding
/// them on to be written as server error messages
impl Handler<ServerError> for Session {
    type Result = ();

    fn handle(&mut self, msg: ServerError, ctx: &mut Self::Context) -> Self::Result {
        Self::write_error(ctx, msg);
    }
}

/// Stream handler for processing incoming websocket messages, and
/// responding accordingly. Text packet messages are decoded and
/// send onto the [`Session::handle_message`] function to be proccessed
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
