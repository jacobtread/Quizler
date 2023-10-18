use crate::{
    game::GameRef,
    games::{Games, InitializedMessage},
    msg::{ClientMessage, ResponseMessage, ServerEvent, ServerResponse},
    types::{Answer, GameToken, HostAction, RemoveReason, ServerError},
};
use axum::extract::ws::{Message, WebSocket};
use futures_util::future::BoxFuture;
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

/// Structure of a session connected to the server
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
    rx: mpsc::UnboundedReceiver<Arc<ServerEvent>>,
    /// Sender for server events
    tx: EventTarget,
}

// Time intervals to check heartbeats
const HB_INTERVAL: Duration = Duration::from_secs(5);
// Timeout for handling loss of connection
const TIMEOUT: Duration = Duration::from_secs(15);

impl Session {
    /// Handler for starting a new session from the provided websocket
    ///
    /// # Arguments
    /// * socket - The websocket to use for the session
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

    /// Handles processing all events for the session
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

    /// Handles cleaning up the session after processing has
    /// terminated.
    ///
    /// Removes the player from its game if its present
    async fn cleanup(&mut self) {
        debug!("Session stopped: {}", self.id);
        // Take the game to attempt removing if present
        if let Some(game) = self.game.take() {
            let mut lock = game.write().await;

            // Inform game to remove self
            let _ = lock.remove_player(self.id, self.id, RemoveReason::LostConnection);
        }
    }

    /// Handles server events received by this session, processes the
    /// events then sends them to the client
    ///
    /// # Arguments
    /// * event - The event to handle
    async fn handle_event(&mut self, event: Arc<ServerEvent>) -> Result<(), axum::Error> {
        let value = event.as_ref();

        // Ensure we drop our reference to the game when kicked
        if let ServerEvent::Kicked { id, .. } = value {
            if self.id.eq(id) {
                self.game = None;
            }
        }

        self.send(value).await
    }

    /// Handles processing websocket messages, updating heartbeat, and forwading
    /// along parsed messages to handle_request
    ///
    /// # Arguments
    /// * msg - The websocket message
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

                self.send(&ServerResponse(ResponseMessage::Error {
                    error: ServerError::MalformedMessage,
                }))
                .await?;

                return Ok(true);
            }
        };

        self.handle_request(req).await?;

        Ok(true)
    }

    /// Handles processing client messages and sending the response
    /// for the message
    ///
    /// # Arguments
    /// * msg - The client message being processed
    async fn handle_request(&mut self, msg: ClientMessage) -> Result<(), axum::Error> {
        let future: BoxFuture<Result<ResponseMessage, ServerError>> = match msg {
            ClientMessage::Initialize { uuid } => Box::pin(self.initialize(uuid)),
            ClientMessage::Connect { token } => Box::pin(self.connect(token)),
            ClientMessage::Join { name } => Box::pin(self.join(name)),
            ClientMessage::HostAction { action } => Box::pin(self.host_action(action)),
            ClientMessage::Answer { answer } => Box::pin(self.answer(answer)),
            ClientMessage::Kick { id } => Box::pin(self.kick(id)),
            ClientMessage::Ready => Box::pin(self.ready()),
        };

        let res = future.await;

        let msg = match res {
            Ok(value) => value,
            Err(error) => ResponseMessage::Error { error },
        };

        let res: ServerResponse = ServerResponse(msg);
        self.send(&res).await
    }

    /// Converts the provided message to JSON writing it as a text frame
    /// to the websocket
    ///
    /// # Arguments
    /// * msg - The message to send
    async fn send<S: Serialize>(&mut self, msg: &S) -> Result<(), axum::Error> {
        let value = serde_json::to_string(msg).map_err(|err| axum::Error::new(Box::new(err)))?;
        self.socket.send(Message::Text(value)).await
    }

    /// Handler for initialize messages to attempt to initialize a new game.
    /// On success the game reference on this session will be updated.
    ///
    /// # Arguments
    /// * uuid - The UUID of the prepared config
    async fn initialize(&mut self, uuid: Uuid) -> Result<ResponseMessage, ServerError> {
        self.disconnect().await;

        let msg: InitializedMessage = Games::initialize(uuid, self.id, self.tx.clone()).await?;
        self.game = Some(msg.game);

        Ok(ResponseMessage::Joined {
            id: self.id,
            config: msg.config,
            token: msg.token,
        })
    }

    /// Handler for connect messages to attempt to connect to a game.
    /// On success the game reference on this session will be updated.
    ///
    /// # Arguments
    /// * uuid - The UUID of the prepared config
    async fn connect(&mut self, token: String) -> Result<ResponseMessage, ServerError> {
        self.disconnect().await;

        let token: GameToken = token.parse()?;

        let game = Games::get_game(&token)
            .await
            .ok_or(ServerError::InvalidToken)?;

        self.game = Some(game);
        Ok(ResponseMessage::Ok)
    }

    /// Disconnects the session from their current game if they
    /// are already in one
    async fn disconnect(&mut self) {
        // If already in a game infrom the game that we've left
        if let Some(game) = self.game.take() {
            let mut lock = game.write().await;
            let _ = lock.remove_player(self.id, self.id, RemoveReason::Disconnected);
        }
    }

    /// Handler for join messages to attempt to join the game reference
    /// by the session game field
    ///
    /// # Arguments
    /// * name - The name to attempt to join with
    async fn join(&mut self, name: String) -> Result<ResponseMessage, ServerError> {
        let msg = {
            let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
            let mut game = game.write().await;

            game.join(self.id, self.tx.clone(), name)
        }?;

        Ok(ResponseMessage::Joined {
            id: self.id,
            token: msg.token,
            config: msg.config,
        })
    }

    /// Handler for host action messages
    ///
    /// # Arguments
    /// * action - The host action to execute
    async fn host_action(&mut self, action: HostAction) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write().await;

        game.host_action(self.id, action)?;
        Ok(ResponseMessage::Ok)
    }

    /// Handler for answer messages
    ///
    /// # Arguments
    /// * answer - The player answer
    async fn answer(&mut self, answer: Answer) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write().await;

        game.answer(self.id, answer)?;
        Ok(ResponseMessage::Ok)
    }

    /// Handler for kick messages
    ///
    /// # Arguments
    /// * target_id - The ID of the player to kick
    async fn kick(&mut self, target_id: SessionId) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write().await;

        game.remove_player(self.id, target_id, RemoveReason::RemovedByHost)?;
        Ok(ResponseMessage::Ok)
    }

    /// Handler for ready messages
    async fn ready(&mut self) -> Result<ResponseMessage, ServerError> {
        let game = self.game.as_ref().ok_or(ServerError::Unexpected)?;
        let mut game = game.write().await;

        game.ready(self.id);
        Ok(ResponseMessage::Ok)
    }
}
/// Wrapper around the session sender to allow sending server
/// events to the sessions
#[derive(Clone)]
pub struct EventTarget(mpsc::UnboundedSender<Arc<ServerEvent>>);

impl EventTarget {
    /// Sends a server event to the event target
    ///
    /// # Arguments
    /// * event - The server event to send
    pub fn send(&self, event: Arc<ServerEvent>) {
        let _ = self.0.send(event);
    }
}
