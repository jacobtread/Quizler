use crate::{
    games::Games,
    msg::{ClientMessage, ClientRequest, ResponseMessage, ServerEvent, ServerResponse},
    types::{Answer, GameToken, HostAction, RemoveReason, ServerError},
};

use axum::extract::ws::{Message, WebSocket};
use log::{debug, error};
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
pub struct EventListener(mpsc::UnboundedSender<Arc<ServerEvent>>);

impl EventListener {
    pub fn do_send(&self, msg: Arc<ServerEvent>) {
        self.0.send(msg);
    }
}

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
    rx: mpsc::UnboundedReceiver<Arc<ServerEvent>>,
    /// Sender for server events
    tx: mpsc::UnboundedSender<Arc<ServerEvent>>,
}

// Time intervals to check heartbeats
const HB_INTERVAL: Duration = Duration::from_secs(5);
// Timeout for handling loss of connection
const TIMEOUT: Duration = Duration::from_secs(15);

impl Session {
    pub async fn start(socket: WebSocket) {
        let (tx, rx) = mpsc::unbounded_channel();
        let id = SESSION_ID.fetch_add(1, Ordering::AcqRel);
        debug!("Starting new session {}", id);
        let hb = Instant::now();
        let this = Self {
            id,
            game: None,
            hb,
            socket,
            rx,
            tx,
        };
        this.process().await;
    }

    /// Heartbeat returns false if connection is failed
    async fn heartbeat(&mut self) -> bool {
        let elapsed = self.hb.elapsed();
        if elapsed >= TIMEOUT {
            // Connection lost timeout
            false
        } else {
            self.socket
                .send(Message::Ping(Vec::with_capacity(0)))
                .await
                .is_ok()
        }
    }

    async fn cleanup(&mut self) {
        debug!("Session stopped: {}", self.id);
        // Take the game to attempt removing if present
        if let Some(game) = self.game.take() {
            let game = Games::get_game(&game).await;

            if let Some(game) = game {
                let mut lock = game.write().await;

                // Inform game to remove self
                let _ = lock.remove_player(self.id, self.id, RemoveReason::LostConnection);
            }
        }
    }

    async fn process(mut self) {
        // Heartbeat interval ticking
        let mut hb_interval = interval(HB_INTERVAL);
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
                    if let ServerEvent::Kicked { id, reason } = event.as_ref() {
                        if *id == self.id {
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
                            continue;
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
                    if !self.heartbeat().await {
                        break;
                    }
                }
            }
        }
        self.cleanup().await;
    }

    async fn send<S: Serialize>(&mut self, msg: &S) -> Result<(), axum::Error> {
        let value = serde_json::to_string(msg).map_err(|err| axum::Error::new(Box::new(err)))?;
        self.socket.send(Message::Text(value)).await
    }

    async fn disconnect(&mut self) {
        // If already in a game infrom the game that we've left
        if let Some(game) = self.game.take() {
            let game = Games::get_game(&game).await;
            if let Some(game) = game {
                let mut lock = game.write().await;
                lock.remove_player(self.id, self.id, RemoveReason::Disconnected);
            }
        }
    }

    async fn initialize(&mut self, uuid: Uuid) -> Result<ResponseMessage, ServerError> {
        self.disconnect();

        let listener = EventListener(self.tx.clone());

        let msg = Games::initialize(uuid, self.id, listener).await?;
        self.game = Some(msg.token);

        Ok(ResponseMessage::Joined {
            id: self.id,
            config: msg.config,
            token: msg.token,
        })
    }

    async fn connect(&mut self, token: String) -> Result<ResponseMessage, ServerError> {
        self.disconnect();

        let token: GameToken = token.parse().map_err(|_| ServerError::InvalidToken)?;

        let game = Games::is_game(&token).await;

        if game {
            self.game = Some(token);
            Ok(ResponseMessage::Ok)
        } else {
            Err(ServerError::InvalidToken)
        }
    }

    async fn join(&mut self, name: String) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let game = Games::get_game(&game)
            .await
            .ok_or(ServerError::InvalidToken)?;

        let listener = EventListener(self.tx.clone());

        let mut lock = game.write().await;
        let result = lock.try_join(self.id, listener, name);

        match result {
            Ok(msg) => Ok(ResponseMessage::Joined {
                id: self.id,
                token: msg.token,
                config: msg.config,
            }),
            Err(err) => {
                self.game = None;
                Err(err)
            }
        }
    }

    async fn host_action(&mut self, action: HostAction) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let game = Games::get_game(&game)
            .await
            .ok_or(ServerError::InvalidToken)?;

        let mut lock = game.write().await;
        lock.host_action(self.id, action)?;
        Ok(ResponseMessage::Ok)
    }

    async fn answer(&mut self, answer: Answer) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let game = Games::get_game(&game)
            .await
            .ok_or(ServerError::InvalidToken)?;

        let mut lock = game.write().await;
        lock.answer(self.id, answer)?;
        Ok(ResponseMessage::Ok)
    }

    async fn kick(&mut self, target_id: SessionId) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let game = Games::get_game(&game)
            .await
            .ok_or(ServerError::InvalidToken)?;

        let mut lock = game.write().await;
        lock.remove_player(self.id, target_id, RemoveReason::RemovedByHost)?;

        Ok(ResponseMessage::Ok)
    }

    async fn ready(&mut self) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let game = Games::get_game(&game)
            .await
            .ok_or(ServerError::InvalidToken)?;

        let mut lock = game.write().await;
        lock.ready(self.id);
        Ok(ResponseMessage::Ok)
    }

    /// Handles a recieved client message
    async fn handle_message(&mut self, req: ClientRequest) {
        let res = match req.msg {
            // Handle initializing new games
            ClientMessage::Initialize { uuid } => self.initialize(uuid).await,

            // Handle try connect messages
            ClientMessage::Connect { token } => self.connect(token).await,

            // Handle join messages
            ClientMessage::Join { name } => self.join(name).await,

            ClientMessage::HostAction { action } => self.host_action(action).await,

            // Handle message for an answer to the current question
            ClientMessage::Answer { answer } => self.answer(answer).await,

            // Handle message for kicking a player
            ClientMessage::Kick { id } => self.kick(id).await,

            // Handle client ready messages
            ClientMessage::Ready => self.ready().await,
        };

        let msg = match res {
            Ok(value) => value,
            Err(error) => ResponseMessage::Error { error },
        };

        let res: ServerResponse = ServerResponse { rid: req.rid, msg };
        self.send(&res).await;
    }
}
