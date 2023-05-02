use crate::{
    game::{
        Game, HostActionMessage, JoinMessage, PlayerAnswerMessage, ReadyMessage,
        RemovePlayerMessage,
    },
    games::{GameToken, Games, GetGameMessage, InitializeMessage},
    msg::{ClientMessage, ServerMessage},
    types::{KickReason, ServerError},
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Handler, Message, StreamHandler,
    WrapFuture,
};
use actix_web_actors::ws;
use log::{error, info};
use serde::Serialize;
use std::sync::Arc;

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
            ClientMessage::Connect { token } => {
                if self.game.is_some() {
                    return Err(ServerError::UnexpectedMessage);
                }

                // Parse the token ensuring it is valid
                let token: GameToken = token.parse()?;

                // Spawn the connect task
                ctx.spawn(
                    self.games
                        .send(GetGameMessage { token })
                        .into_actor(self)
                        .map(|result, act, ctx| {
                            if let Some(game) = result.expect("Games service was stopped") {
                                // Set the session associated game
                                act.game = Some(game);
                            } else {
                                // Game didn't exsit
                                Self::write_error(ctx, ServerError::InvalidToken);
                            }
                        }),
                );
            }

            // Handle join messages
            ClientMessage::Join { name } => {
                let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;

                // Spawn the join task
                ctx.spawn(
                    game
                        // Send the initliaze message
                        .send(JoinMessage {
                            id: self.id,
                            addr: ctx.address(),
                            name,
                        })
                        .into_actor(self)
                        .map(|msg, act, ctx| {
                            let result = match msg {
                                Ok(value) => value,
                                Err(_) => {
                                    act.game = None;
                                    return;
                                }
                            };

                            let msg = match result {
                                Ok(msg) => ServerMessage::Joined {
                                    id: msg.id,
                                    token: msg.token,
                                    config: msg.config,
                                },
                                // Handle game being stopped
                                Err(error) => ServerMessage::Error { error },
                            };

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
            ws::Message::Ping(ping) => {
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
