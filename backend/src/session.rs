use crate::{
    game::{
        Game, HostActionMessage, JoinMessage, PlayerAnswerMessage, ReadyMessage,
        RemovePlayerMessage,
    },
    games::{Games, GetGameMessage, InitializeMessage},
    msg::{ClientMessage, ClientRequest, ServerMessage, ServerResponse},
    types::{Answer, HostAction, RemoveReason, ServerError},
};
use actix::{
    Actor, ActorContext, ActorFuture, ActorFutureExt, Addr, AsyncContext, Handler, Message,
    StreamHandler, WrapFuture,
};
use actix_web_actors::ws;
use log::{debug, error, info};
use serde::Serialize;
use std::{pin::Pin, sync::Arc};
use uuid::Uuid;

/// Type alias for numbers that represent Session ID's
pub type SessionId = u32;

pub struct Session {
    /// Unique ID of the session
    pub id: SessionId,
    /// Address to the current game if apart of one
    pub game: Option<Addr<Game>>,
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Session>;

    /// Handle the session being stopped by removing the
    /// session from any games and cleaning up after it
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("Session stopped: {}", self.id);
        // Take the game to attempt removing if present
        if let Some(game) = self.game.take() {
            // Inform game to remove self
            game.do_send(RemovePlayerMessage {
                session_id: self.id,
                target_id: self.id,
                reason: RemoveReason::LostConnection,
            });
        }
    }
}

type RespFut = Pin<Box<dyn ActorFuture<Session, Output = Result<ServerMessage, ServerError>>>>;

impl Session {
    /// Writes a server message by encoding it to json and then sending it
    /// as a text message through the web socket context
    ///
    /// `ctx` The context to write to
    /// `msg` The message to write
    fn write_message<S: Serialize>(ctx: &mut <Self as Actor>::Context, msg: &S) {
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

    fn write_error(ctx: &mut <Self as Actor>::Context, error: ServerError) {
        Self::write_message(ctx, &ServerMessage::Error { error })
    }

    fn initialize(&mut self, ctx: &mut <Self as Actor>::Context, uuid: Uuid) -> RespFut {
        // If already in a game infrom the game that we've left
        if let Some(game) = self.game.take() {
            game.do_send(RemovePlayerMessage {
                session_id: self.id,
                target_id: self.id,
                reason: RemoveReason::Disconnected,
            });
        }

        let msg = InitializeMessage {
            uuid,
            id: self.id,
            addr: ctx.address(),
        };

        Box::pin(
            async move {
                Games::get()
                    .send(msg)
                    .await
                    .expect("Games service was not running")
            }
            .into_actor(self)
            .map(|result, act, _| {
                result.map(|msg| {
                    act.game = Some(msg.game);
                    ServerMessage::Joined {
                        id: act.id,
                        config: msg.config,
                        token: msg.token,
                    }
                })
            }),
        )
    }

    fn connect(&mut self, token: String) -> RespFut {
        // If already in a game infrom the game that we've left
        if let Some(game) = self.game.take() {
            game.do_send(RemovePlayerMessage {
                session_id: self.id,
                target_id: self.id,
                reason: RemoveReason::Disconnected,
            });
        }

        Box::pin(
            async move {
                Games::get()
                    .send(GetGameMessage { token })
                    .await
                    .expect("Games service was not running")
            }
            .into_actor(self)
            .map(|result, act, _| {
                result.map(|game| {
                    act.game = Some(game);
                    ServerMessage::Ok
                })
            }),
        )
    }

    fn join(&mut self, ctx: &mut <Self as Actor>::Context, name: String) -> RespFut {
        let game = self.game.clone();

        let join_msg = JoinMessage {
            id: self.id,
            addr: ctx.address(),
            name,
        };

        Box::pin(
            async move {
                let game = game.ok_or(ServerError::Unexpected)?;

                game.send(join_msg)
                    .await
                    .map_err(|_| ServerError::NotJoinable)
            }
            .into_actor(self)
            .map(|result, act, _| {
                result
                    .map_err(|err| {
                        act.game = None;
                        err
                    })?
                    .map(|msg| ServerMessage::Joined {
                        id: act.id,
                        token: msg.token,
                        config: msg.config,
                    })
            }),
        )
    }

    fn host_action(&mut self, action: HostAction) -> RespFut {
        let game = self.game.clone();

        let action_msg = HostActionMessage {
            session_id: self.id,
            action,
        };

        Box::pin(
            async move {
                let game = game.ok_or(ServerError::Unexpected)?;

                game.send(action_msg)
                    .await
                    .map_err(|_| ServerError::Unexpected)?
                    .map(|_| ServerMessage::Ok)
            }
            .into_actor(self),
        )
    }

    fn answer(&mut self, answer: Answer) -> RespFut {
        let game = self.game.clone();

        let answer_msg = PlayerAnswerMessage {
            session_id: self.id,
            answer,
        };

        Box::pin(
            async move {
                let game = game.ok_or(ServerError::Unexpected)?;

                game.send(answer_msg)
                    .await
                    .map_err(|_| ServerError::Unexpected)?
                    .map(|_| ServerMessage::Ok)
            }
            .into_actor(self),
        )
    }

    fn kick(&mut self, id: SessionId) -> RespFut {
        let game = self.game.clone();

        let remove_msg = RemovePlayerMessage {
            session_id: self.id,
            target_id: id,
            reason: RemoveReason::RemovedByHost,
        };

        Box::pin(
            async move {
                let game = game.ok_or(ServerError::Unexpected)?;

                game.send(remove_msg)
                    .await
                    .map_err(|_| ServerError::Unexpected)?
                    .map(|_| ServerMessage::Ok)
            }
            .into_actor(self),
        )
    }

    fn ready(&mut self) -> RespFut {
        let game = self.game.clone();

        let ready_msg = ReadyMessage {
            session_id: self.id,
        };

        Box::pin(
            async move {
                let game = game.ok_or(ServerError::Unexpected)?;

                game.send(ready_msg)
                    .await
                    .map_err(|_| ServerError::Unexpected)
                    .map(|_| ServerMessage::Ok)
            }
            .into_actor(self),
        )
    }

    /// Handles a recieved client message
    fn handle_message(&mut self, req: ClientRequest, ctx: &mut <Self as Actor>::Context) {
        let fut = match req.msg {
            // Handle initializing new games
            ClientMessage::Initialize { uuid } => self.initialize(ctx, uuid),

            // Handle try connect messages
            ClientMessage::Connect { token } => self.connect(token),

            // Handle join messages
            ClientMessage::Join { name } => self.join(ctx, name),

            ClientMessage::HostAction { action } => self.host_action(action),

            // Handle message for an answer to the current question
            ClientMessage::Answer { answer } => self.answer(answer),

            // Handle message for kicking a player
            ClientMessage::Kick { id } => self.kick(id),

            // Handle client ready messages
            ClientMessage::Ready => self.ready(),
        };

        let fut = fut.map(move |result, _, ctx| {
            let msg = match result {
                Ok(value) => value,
                Err(error) => ServerMessage::Error { error },
            };

            let res: ServerResponse = ServerResponse { rid: req.rid, msg };
            Self::write_message(ctx, &res)
        });

        ctx.spawn(fut);
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
        Self::write_message(ctx, &msg);
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
                ctx.stop();
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
                ctx.close(reason);
                ctx.stop();
                return;
            }
            ws::Message::Binary(_) => {
                error!("Unexpected binary message from socket");
                return;
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
                return;
            }
            ws::Message::Pong(_) => {
                // TODO: Handle pong
                return;
            }
            ws::Message::Nop => return,
        };

        // Decode the recieved client message
        let req = match serde_json::from_slice::<ClientRequest>(text.as_bytes()) {
            Ok(value) => value,
            Err(err) => {
                error!("Unable to decode client message: {}", err);
                Self::write_error(ctx, ServerError::MalformedMessage);
                return;
            }
        };

        Self::handle_message(self, req, ctx);
    }
}

/// Message sent to sessions to inform them that they've been
/// removed from their current game
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClearGameMessage;

impl Handler<ClearGameMessage> for Session {
    type Result = ();

    fn handle(&mut self, _: ClearGameMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.game = None;
    }
}
