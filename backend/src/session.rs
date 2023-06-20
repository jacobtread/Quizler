use crate::{
    game::GameRef,
    games::Games,
    msg::{ClientMessage, ResponseMessage, ServerEvent, ServerResponse},
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

/// Type alias for numbers that represent Session ID's
pub type SessionId = u32;

/// Atomic provider for session IDs
static SESSION_ID: AtomicU32 = AtomicU32::new(0);

pub struct Session {
    /// Unique ID of the session
    id: SessionId,
    /// Token of the current game this session is in
    game: Option<GameRef>,

    /// Last heartbeat received from the client
    hb: Instant,
    /// The underlying socket connection
    socket: WebSocket,

    /// Receiver for receiving server events
    rx: mpsc::UnboundedReceiver<EventMessage>,
    /// Sender for server events
    tx: EventTarget,
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
            tx: EventTarget(tx),
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
            let mut lock = game.write().await;

            // Inform game to remove self
            let _ = lock.remove_player(self.id, self.id, RemoveReason::LostConnection);
        }
    }

    async fn process(mut self) {
        // Heartbeat interval ticking
        let mut hb_interval = interval(HB_INTERVAL);
        hb_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            select! {
                // Server events
                event = self.rx.recv() => {
                    let event = match event {
                        Some(event) => event,
                        None => break,
                    };

                    if self.handle_event(event).await.is_err() {
                        // Failed to send the response
                        break;
                    }
                }
                // Client requests
                req = self.socket.recv() => {
                    let msg: Message = match req {
                        Some(Ok(value)) => value,
                        // Error while reading body (Skip the message)
                        Some(Err(_)) => continue,
                        // Connection is closed break from processing
                        None => break,
                    };

                    match self.handle_message(msg).await {
                        Ok(false) | Err(_) => break,
                        Ok(true )=> {}
                    }
                }
                // Heartbeat
                _ = hb_interval.tick() => {
                    if !self.heartbeat().await {
                        break;
                    }
                }
            };
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
            let mut lock = game.write().await;
            let _ = lock.remove_player(self.id, self.id, RemoveReason::Disconnected);
        }
    }

    async fn handle_event(&mut self, event: EventMessage) -> Result<(), axum::Error> {
        let value = event.as_ref();

        // Ensure we drop our reference to the game when kicked
        if let ServerEvent::Kicked { id, .. } = value {
            if self.id.eq(id) {
                self.game = None;
            }
        }

        self.send(value).await
    }

    async fn handle_message(&mut self, msg: Message) -> Result<bool, axum::Error> {
        // Update heartbeat
        self.hb = Instant::now();

        // Handle different message types
        let text = match msg {
            Message::Text(value) => value,
            Message::Ping(ping) => {
                // If sending pong failed break
                if self.socket.send(Message::Pong(ping)).await.is_err() {
                    return Ok(false);
                }
                return Ok(true);
            }
            Message::Close(_) => return Ok(false),
            _ => return Ok(true),
        };

        // Decode the recieved client message
        let req = match serde_json::from_str::<ClientMessage>(&text) {
            Ok(value) => value,
            Err(err) => {
                error!("Unable to decode client message: {}", err);

                self.send(&ServerResponse {
                    msg: ResponseMessage::Error {
                        error: ServerError::MalformedMessage,
                    },
                })
                .await?;

                return Ok(true);
            }
        };

        self.handle_request(req).await?;

        Ok(true)
    }

    /// Handles a recieved client message
    async fn handle_request(&mut self, req: ClientMessage) -> Result<(), axum::Error> {
        let res = match req {
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

        let res: ServerResponse = ServerResponse { msg };
        self.send(&res).await
    }

    async fn initialize(&mut self, uuid: Uuid) -> Result<ResponseMessage, ServerError> {
        self.disconnect().await;

        let msg = Games::initialize(uuid, self.id, self.tx.clone()).await?;
        self.game = Some(msg.game);

        Ok(ResponseMessage::Joined {
            id: self.id,
            config: msg.config,
            token: msg.token,
        })
    }

    async fn connect(&mut self, token: String) -> Result<ResponseMessage, ServerError> {
        self.disconnect().await;

        let token: GameToken = token.parse()?;

        let game = Games::get_game(&token)
            .await
            .ok_or(ServerError::InvalidToken)?;

        self.game = Some(game);
        Ok(ResponseMessage::Ok)
    }

    async fn join(&mut self, name: String) -> Result<ResponseMessage, ServerError> {
        let result = {
            let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
            let mut lock = game.write().await;
            lock.join(self.id, self.tx.clone(), name)
        };

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
        let mut lock = game.write().await;

        lock.host_action(self.id, action)?;
        Ok(ResponseMessage::Ok)
    }

    async fn answer(&mut self, answer: Answer) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut lock = game.write().await;

        lock.answer(self.id, answer)?;
        Ok(ResponseMessage::Ok)
    }

    async fn kick(&mut self, target_id: SessionId) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut lock = game.write().await;
        lock.remove_player(self.id, target_id, RemoveReason::RemovedByHost)?;

        Ok(ResponseMessage::Ok)
    }

    async fn ready(&mut self) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut lock = game.write().await;
        lock.ready(self.id);

        Ok(ResponseMessage::Ok)
    }
}

/// Wrapper around server events to allow for owned
/// and shared access
enum EventMessage {
    /// Owned server event
    Owned(ServerEvent),
    /// Server event behind an Arc shared for many players
    Shared(Arc<ServerEvent>),
}

impl EventMessage {
    /// Obtains a reference to the server event stored
    /// in this message
    fn as_ref(&self) -> &ServerEvent {
        match self {
            EventMessage::Owned(value) => value,
            EventMessage::Shared(value) => value.as_ref(),
        }
    }
}

/// Wrapper around the session sender to allow sending server
/// events to the sessions
#[derive(Clone)]
pub struct EventTarget(mpsc::UnboundedSender<EventMessage>);

impl EventTarget {
    /// Sends an owned message over the event sender
    ///
    /// # Arguments
    /// * event - The server event to send
    pub fn send(&self, event: ServerEvent) {
        let _ = self.0.send(EventMessage::Owned(event));
    }

    /// Sends a shared message behind an Arc over the event sender
    ///
    /// # Arguments
    /// * event - The server event to send
    pub fn send_shared(&self, event: Arc<ServerEvent>) {
        let _ = self.0.send(EventMessage::Shared(event));
    }
}
