use crate::{
    games::Games,
    msg::{ClientMessage, ClientRequest, ResponseMessage, ServerEvent, ServerResponse},
    types::{Answer, GameToken, HostAction, RemoveReason, ServerError},
};

use axum::extract::ws::{Message, WebSocket};
use futures_util::StreamExt;
use log::error;
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    select,
    sync::mpsc,
    time::{interval, MissedTickBehavior},
};
use uuid::Uuid;

pub enum SocketEvent {
    Event(Arc<ServerEvent>),
    Message(Message),
    Heartbeat,
}

/// Type alias for numbers that represent Session ID's
pub type SessionId = u32;

/// Atomic provider for session IDs
static SESSION_ID: AtomicU32 = AtomicU32::new(0);

///
#[derive(Clone)]
pub struct EventListener(mpsc::UnboundedSender<ServerEvent>);

pub struct Session {
    /// Unique ID of the session
    id: SessionId,
    /// Token of the current game this session is in
    game: Option<GameToken>,

    /// Last heartbeat received from the client
    hb: Instant,
    /// The underlying socket connection
    socket: WebSocket,

    /// Receiver for receiving server events
    rx: mpsc::UnboundedReceiver<ServerEvent>,
    /// Sender for server events
    tx: mpsc::UnboundedSender<ServerEvent>,
}

// Time intervals to check heartbeats
const HB_INTERVAL: Duration = Duration::from_secs(5);
// Timeout for handling loss of connection
const TIMEOUT: Duration = Duration::from_secs(15);

impl Session {
    fn spawn(socket: WebSocket) {
        let (tx, rx) = mpsc::unbounded_channel::<ServerEvent>();
        let id = SESSION_ID.fetch_add(1, Ordering::AcqRel);
        let hb = Instant::now();
        let this = Self {
            id,
            game: None,
            hb,
            socket,
            rx,
            tx,
        };
        tokio::spawn(this.process());
    }

    /// Heartbeat returns false if connection is failed
    async fn heartbeat(&mut self) -> bool {
        let elapsed = self.hb.elapsed();
        if elapsed >= TIMEOUT {
            // Connection lost timeout
            false
        } else if self
            .socket
            .send(Message::Ping(Vec::with_capacity(0)))
            .await
            .is_err()
        {
            // Failed to send pong message
            false
        } else {
            true
        }
    }

    async fn cleanup(&mut self) {
        todo!("Proper cleanup remove from game")
        // debug!("Session stopped: {}", self.id);
        // // Take the game to attempt removing if present
        // if let Some(game) = self.game.take() {
        //     // Inform game to remove self
        //     game.do_send(RemovePlayerMessage {
        //         id: self.id,
        //         target_id: self.id,
        //         reason: RemoveReason::LostConnection,
        //     });
        // }
    }

    async fn process(mut self) {
        // Heartbeat interval ticking
        let hb_interval = interval(HB_INTERVAL);
        hb_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            let event = select! {
                // Server events
                event = self.rx.recv() => {
                    let event = match event {
                        Some(event) => event,
                        None => break,
                    };
                    SocketEvent::Event(event)
                }
                // Client requests
                req = self.socket.recv() => {
                    let req: Message = match req {
                        Some(Ok(value)) => value,
                        // Error while reading body (Skip the message)
                        Some(Err(error)) => continue,
                        // Connection is closed break from processing
                        None => break,
                    };

                    SocketEvent::Message(req)
                }
                // Heartbeat
                _ = hb_interval.tick() => {
                    SocketEvent::Heartbeat
                }
            };

            match event {
                SocketEvent::Event(event) => {
                    // Handle self kicked
                    if let ServerEvent::Kicked { id, reason } = &event {
                        if id == self.id {
                            self.game = None;
                        }
                    }

                    // TODO: Handle error
                    self.send(event.as_ref()).await;
                }
                SocketEvent::Message(msg) => {
                    // Update heartbeat
                    self.hb = Instant::now();

                    // Handle different message types
                    let text = match msg {
                        Message::Text(value) => value,
                        Message::Ping(ping) => {
                            self.socket.send(Message::Pong(ping)).await;
                        }
                        Message::Close(reason) => {
                            break;
                        }
                        _ => continue,
                    };

                    // Decode the recieved client message
                    let req = match serde_json::from_str::<ClientRequest>(&text) {
                        Ok(value) => value,
                        Err(err) => {
                            error!("Unable to decode client message: {}", err);
                            return;
                        }
                    };

                    self.handle_message(req).await;
                }
                SocketEvent::Heartbeat => {
                    if !self.heartbeat() {
                        break;
                    }
                }
            }
        }
        self.cleanup().await;
    }

    async fn send<S: Serialize>(&mut self, msg: &S) -> Result<(), axum::Error> {
        let value = serde_json::to_string(msg)?;
        self.socket.send(Message::Text(value)).await
    }

    async fn handle_event(&mut self, event: ServerEvent) {}

    async fn handle_message(&mut self, req: ClientRequest) -> Result<ServerResponse, ServerError> {}

    async fn disconnect(&mut self) {
        // If already in a game infrom the game that we've left
        if let Some(game) = self.game.take() {
            let game = Games::get_game(&game).await;
            if let Some(game) = game {
                let lock = game.write().await;
                lock.remove_player(self.id, self.id, RemoveReason::Disconnected);
            }
        }
    }

    async fn initialize(&mut self, uuid: Uuid) -> Result<ServerResponse, ServerError> {
        self.disconnect();

        let listener = EventListener(self.tx.clone());

        let msg = Games::initialize(uuid, self.id, listener).await?;
        self.game = msg.token;

        Ok(ResponseMessage::Joined {
            id: self.id,
            config: msg.config,
            token: msg.token,
        })
    }

    async fn connect(&mut self, token: String) -> Result<ServerResponse, ServerError> {
        self.disconnect();

        let token: GameToken = token.parse().map_err(|_| ServerError::InvalidToken)?;

        let game = Games::is_game(&token).await?;

        if (game) {
            self.game = Some(token);
            Ok(ResponseMessage::Ok)
        } else {
            Err(ServerError::InvalidToken)
        }
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
            Self::send(ctx, &res)
        });

        ctx.spawn(fut);
    }
}
