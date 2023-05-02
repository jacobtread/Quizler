use crate::{
    error::ServerError,
    game::{
        ConnectMessage, Game, GameConfig, GameState, HostActionMessage, PlayerAnswerMessage,
        PlayerGameConfig, ReadyMessage, RemovePlayerMessage, TimeSync,
    },
    games::{GameToken, Games, GetGameMessage, InitializeMessage},
    types::{Answer, Question},
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Handler, Message, StreamHandler,
    WrapFuture,
};
use actix_web_actors::ws;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Type alias for numbers that represent Session ID's
pub type SessionId = u32;

pub struct Session {
    /// Unique ID of the session
    pub id: SessionId,
    /// Address to the current game if apart of one
    pub game: Option<Addr<Game>>,
    /// Address to the games store
    pub games: Addr<Games>,
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

/// Actions that can be executed by the host
/// session of a game

#[derive(Debug, Copy, Clone, Deserialize)]
#[repr(u8)]
pub enum HostAction {
    /// Begin the starting process
    Start = 0x1,
    /// Cancel the starting process
    Cancel = 0x2,
    /// Skip the current waiting timer state
    Skip = 0x3,
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
                session_id: self.id,
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
                error!("Failed to encode server message as JSON: {}", err);
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
    fn handle_message(
        &mut self,
        message: ClientMessage,
        ctx: &mut SessionContext,
    ) -> Result<(), ServerError> {
        match message {
            // Handle initializing new games
            ClientMessage::Initialize { uuid } => {
                if self.game.is_some() {
                    return Err(ServerError::UnexpectedMessage);
                }

                // Spawn the initialization task
                ctx.spawn(
                    self.games
                        // Send the initliaze message
                        .send(InitializeMessage {
                            uuid,
                            id: self.id,
                            addr: ctx.address(),
                        })
                        .into_actor(self)
                        .map(|msg, act, ctx| {
                            // Handle games service being stopped
                            let result = msg.expect("Games service was not running");

                            // Transform the output message
                            let msg = match result {
                                Ok(msg) => {
                                    act.game = Some(msg.game);
                                    ServerMessage::Initialized {
                                        config: msg.config,
                                        token: msg.token,
                                    }
                                }
                                Err(error) => ServerMessage::Error { error },
                            };

                            // Write the response
                            Self::write_message(ctx, &msg);
                        }),
                );
            }

            // Handle try connect messages
            ClientMessage::Connect { token, name } => {
                if self.game.is_some() {
                    return Err(ServerError::UnexpectedMessage);
                }

                let id = self.id;
                let addr = ctx.address();

                let games = self.games.clone();

                // Spawn the connect task
                ctx.spawn(
                    async move {
                        // Parse the token
                        let token: GameToken = token.parse()?;

                        // Attempting to connect to AED32E as Jacob
                        debug!("Attempting to connect to {} as {}", token, name);

                        // Obtain a addr to the game
                        let game_addr = games
                            .send(GetGameMessage { token })
                            .await
                            .expect("Games service was stopped")
                            .ok_or(ServerError::InvalidToken)?;
                        // Attempt the connection
                        game_addr
                            .send(ConnectMessage { id, addr, name })
                            .await
                            .map_err(|_| ServerError::InvalidToken)?
                    }
                    .into_actor(self)
                    .map(|result, act, ctx| {
                        // Transform the output message
                        let msg = match result {
                            Ok(msg) => {
                                act.game = Some(msg.game);

                                ServerMessage::Connected {
                                    id: msg.id,
                                    token: msg.token,
                                    config: msg.config,
                                }
                            }
                            Err(error) => ServerMessage::Error { error },
                        };

                        // Write the response
                        Self::write_message(ctx, &msg);
                    }),
                );
            }

            ClientMessage::HostAction { action } => {
                let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
                // Spawn the answer task
                ctx.spawn(
                    game
                        // Send the initliaze message
                        .send(HostActionMessage {
                            session_id: self.id,
                            action,
                        })
                        .into_actor(self)
                        .map(|msg, _, ctx| {
                            let msg = match msg {
                                Ok(Err(error)) => ServerMessage::Error { error },
                                // Handle game being stopped
                                Err(_) => ServerMessage::Error {
                                    error: ServerError::Unexpected,
                                },
                                _ => return,
                            };

                            // Write the response
                            Self::write_message(ctx, &msg);
                        }),
                );
            }

            // Handle message for an answer to the current question
            ClientMessage::Answer(answer) => {
                let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
                // Spawn the answer task
                ctx.spawn(
                    game
                        // Send the initliaze message
                        .send(PlayerAnswerMessage {
                            session_id: self.id,
                            answer,
                        })
                        .into_actor(self)
                        .map(|msg, _z, ctx| {
                            let msg = match msg {
                                Ok(Err(error)) => ServerMessage::Error { error },
                                // Handle game being stopped
                                Err(_) => ServerMessage::Error {
                                    error: ServerError::Unexpected,
                                },
                                _ => return,
                            };

                            // Write the response
                            Self::write_message(ctx, &msg);
                        }),
                );
            }

            // Handle message for kicking a player
            ClientMessage::Kick { id } => {
                let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
                game.do_send(RemovePlayerMessage {
                    session_id: self.id,
                    target_id: id,
                    reason: KickReason::RemovedByHost,
                });
            }

            // Handle client ready messages
            ClientMessage::Ready => {
                let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
                game.do_send(ReadyMessage { id: self.id });
            }
        }
        Ok(())
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
                error!("Got error while recieving websocket messages: {}", err);
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
        let value = match serde_json::from_slice::<ClientMessage>(text.as_bytes()) {
            Ok(value) => value,
            Err(err) => {
                error!("Unable to decode client message: {}", err);
                Self::write_error(ctx, ServerError::MalformedMessage);
                return;
            }
        };

        if let Err(error) = Self::handle_message(self, value, ctx) {
            Self::write_error(ctx, error);
        }
    }
}
