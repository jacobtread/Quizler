use crate::{
    game::{
        Game, HostActionMessage, JoinMessage, PlayerAnswerMessage, ReadyMessage,
        RemovePlayerMessage,
    },
    games::{Games, GetGameMessage, InitializeMessage},
    msg::{ClientMessage, ClientRequest, ResponseMessage, ServerEvent, ServerResponse},
    types::{Answer, HostAction, RemoveReason, ServerError},
};
use actix::{
    fut::LocalBoxActorFuture, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Handler,
    Message, ResponseActFuture, StreamHandler, WrapFuture,
};
use actix_web_actors::ws;
use log::{debug, error};
use serde::Serialize;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

/// Type alias for numbers that represent Session ID's
pub type SessionId = u32;

pub struct Session {
    /// Unique ID of the session
    pub id: SessionId,
    /// Address to the current game if apart of one
    pub game: Option<Addr<Game>>,
    /// Last time the session was heard from
    pub hb: Instant,
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Session>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Run heartbeat intervals
        ctx.run_interval(Self::HB_INTERVAL, |act, ctx| {
            let elapsed = act.hb.elapsed();
            // Connection lost timeout
            if elapsed >= Self::TIMEOUT {
                ctx.stop();
                return;
            }

            ctx.ping(&[]);
        });
    }

    /// Handle the session being stopped by removing the
    /// session from any games and cleaning up after it
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("Session stopped: {}", self.id);
        // Take the game to attempt removing if present
        if let Some(game) = self.game.take() {
            // Inform game to remove self
            game.do_send(RemovePlayerMessage {
                id: self.id,
                target_id: self.id,
                reason: RemoveReason::LostConnection,
            });
        }
    }
}

type RespFut = LocalBoxActorFuture<Session, Result<ResponseMessage, ServerError>>;

impl Session {
    // Time intervals to check heartbeats
    const HB_INTERVAL: Duration = Duration::from_secs(5);
    // Timeout for handling loss of connection
    const TIMEOUT: Duration = Duration::from_secs(15);

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

    fn disconnect(&mut self) {
        // If already in a game infrom the game that we've left
        if let Some(game) = self.game.take() {
            game.do_send(RemovePlayerMessage {
                id: self.id,
                target_id: self.id,
                reason: RemoveReason::Disconnected,
            });
        }
    }

    fn initialize(&mut self, ctx: &mut <Self as Actor>::Context, uuid: Uuid) -> RespFut {
        self.disconnect();

        let id = self.id;
        let addr = ctx.address();

        async move {
            Games::get()
                .send(InitializeMessage { uuid, id, addr })
                .await
                .expect("Games service was not running")
        }
        .into_actor(self)
        .map(|result, act, _| {
            result.map(|msg| {
                act.game = Some(msg.game);
                ResponseMessage::Joined {
                    id: act.id,
                    config: msg.config,
                    token: msg.token,
                }
            })
        })
        .boxed_local()
    }

    fn connect(&mut self, token: String) -> RespFut {
        self.disconnect();

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
                ResponseMessage::Ok
            })
        })
        .boxed_local()
    }

    fn join(&mut self, ctx: &mut <Self as Actor>::Context, name: String) -> RespFut {
        let game = self.game.clone();
        let id = self.id;
        let addr = ctx.address();

        async move {
            let game = game.ok_or(ServerError::Unexpected)?;

            game.send(JoinMessage { id, addr, name })
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
                .map(|msg| ResponseMessage::Joined {
                    id: act.id,
                    token: msg.token,
                    config: msg.config,
                })
        })
        .boxed_local()
    }

    fn host_action(&mut self, action: HostAction) -> RespFut {
        let game = self.game.clone();
        let id = self.id;

        async move {
            let game = game.ok_or(ServerError::Unexpected)?;

            game.send(HostActionMessage { id, action })
                .await
                .map_err(|_| ServerError::Unexpected)?
                .map(|_| ResponseMessage::Ok)
        }
        .into_actor(self)
        .boxed_local()
    }

    fn answer(&mut self, answer: Answer) -> RespFut {
        let game = self.game.clone();
        let id = self.id;

        async move {
            let game = game.ok_or(ServerError::Unexpected)?;

            game.send(PlayerAnswerMessage { id, answer })
                .await
                .map_err(|_| ServerError::Unexpected)?
                .map(|_| ResponseMessage::Ok)
        }
        .into_actor(self)
        .boxed_local()
    }

    fn kick(&mut self, target_id: SessionId) -> RespFut {
        let game = self.game.clone();
        let id = self.id;

        async move {
            let game = game.ok_or(ServerError::Unexpected)?;

            game.send(RemovePlayerMessage {
                id,
                target_id,
                reason: RemoveReason::RemovedByHost,
            })
            .await
            .map_err(|_| ServerError::Unexpected)?
            .map(|_| ResponseMessage::Ok)
        }
        .into_actor(self)
        .boxed_local()
    }

    fn ready(&mut self) -> RespFut {
        let game = self.game.clone();
        let id = self.id;

        async move {
            let game = game.ok_or(ServerError::Unexpected)?;

            game.send(ReadyMessage { id })
                .await
                .map_err(|_| ServerError::Unexpected)
                .map(|_| ResponseMessage::Ok)
        }
        .into_actor(self)
        .boxed_local()
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
                Err(error) => ResponseMessage::Error { error },
            };

            let res: ServerResponse = ServerResponse { rid: req.rid, msg };
            Self::write_message(ctx, &res)
        });

        ctx.spawn(fut);
    }
}

/// Handle writing server messages
impl Handler<ServerEvent> for Session {
    type Result = ();

    fn handle(&mut self, msg: ServerEvent, ctx: &mut Self::Context) -> Self::Result {
        Self::write_message(ctx, &msg);
    }
}

/// Handle writing shared references to a server message
impl Handler<Arc<ServerEvent>> for Session {
    type Result = ();

    fn handle(&mut self, msg: Arc<ServerEvent>, ctx: &mut Self::Context) -> Self::Result {
        Self::write_message(ctx, &msg);
    }
}

impl Handler<ClientMessage> for Session {
    type Result = ResponseActFuture;

    fn handle(&mut self, msg: ClientMessage, ctx: &mut Self::Context) -> Self::Result {
        todo!()
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

        // Any message is considered a heartbeat
        self.hb = Instant::now();

        // Only expect text messages
        let text = match message {
            ws::Message::Text(value) => value,
            ws::Message::Ping(ping) => {
                ctx.pong(&ping);
                return;
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
                return;
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
                return;
            }
            _ => return,
        };

        // Decode the recieved client message
        let req = match serde_json::from_slice::<ClientRequest>(text.as_bytes()) {
            Ok(value) => value,
            Err(err) => {
                error!("Unable to decode client message: {}", err);
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
