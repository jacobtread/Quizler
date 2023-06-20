use crate::{
    game::{Game, GameConfig, GameRef},
    session::{EventTarget, SessionId},
    types::{GameToken, ServerError},
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::{interval, MissedTickBehavior},
};
use uuid::Uuid;

/// Global instance for storing games
static mut GAMES: Option<RwLock<Games>> = None;

/// Central store for storing all the references to the individual
/// games that are currently running
#[derive(Default)]
pub struct Games {
    /// Map of the game tokens to the actual game itself
    games: HashMap<GameToken, GameRef>,
    /// Map of UUID's to game configurations that are preparing to start
    preparing: HashMap<Uuid, PreparingGame>,
}

/// Game state for a game thats been created from the HTTP
/// API but hasn't yet been initialized by a host socket
pub struct PreparingGame {
    /// The config being prepared
    config: GameConfig,
    /// Creation time of this prepared game
    created: Instant,
}

/// Message containing the details of a game that has been successfully
/// connected to by the host (The game has finished being prepared)
pub struct InitializedMessage {
    /// The uniquely generated game token (e.g A3DLM)
    pub token: GameToken,
    /// The full game config to be used while playing
    pub config: Arc<GameConfig>,
    /// Reference to the created game
    pub game: GameRef,
}

impl Games {
    /// Initializes the games global state and starts the
    /// tick_cleanup task
    pub fn init() {
        unsafe {
            GAMES = Some(RwLock::new(Games::default()));
        }

        // Spawn the cleanup future
        tokio::spawn(Self::tick_cleanup());
    }

    /// Handles cleaning up games that have expired from the
    /// preparing set runs every 10 minutes
    async fn tick_cleanup() {
        /// Interval to check for expired game prepares (5mins)
        const PREPARE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 5);

        /// The amount of time that must pass for a prepared game to be
        /// considered expired (10mins)
        const GAME_EXPIRY_TIME: Duration = Duration::from_secs(60 * 10);

        // Create the interval future
        let mut interval = interval(PREPARE_CHECK_INTERVAL);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            // Wait until the desired time has passed
            interval.tick().await;

            // Obtain a write lock and remove all expired games
            let mut games = Self::write().await;
            games.preparing.retain(|_, value| {
                let elapsed = value.created.elapsed();
                elapsed < GAME_EXPIRY_TIME
            });
        }
    }

    /// Aquires a read lock over the games structure
    /// returning the lock guard
    async fn read() -> RwLockReadGuard<'static, Games> {
        match unsafe { &GAMES } {
            Some(value) => value.read().await,
            None => panic!("Global games instance not initialized"),
        }
    }

    /// Aquires a write lock over the games structure
    /// returning the lock guard
    async fn write() -> RwLockWriteGuard<'static, Games> {
        match unsafe { &GAMES } {
            Some(value) => value.write().await,
            None => panic!("Global games instance not initialized"),
        }
    }

    /// Prepares a new Quiz for creation. Stores the uploaded config
    /// with a UUID provided the UUID for the host to connect with
    ///
    /// # Arguments
    /// * config - The config for the quiz
    pub async fn prepare(config: GameConfig) -> Uuid {
        let id = Uuid::new_v4();
        let created = Instant::now();

        let mut games = Self::write().await;
        games
            .preparing
            .insert(id, PreparingGame { config, created });

        id
    }

    /// Initializes a prepared game using the provided host details and
    /// prepare config UUID
    ///
    /// # Arguments
    /// * uuid - The UUID of the prepared config
    /// * host_id - The session ID of the host player
    /// * host_target - The event target for the host player
    pub async fn initialize(
        uuid: Uuid,
        host_id: SessionId,
        host_target: EventTarget,
    ) -> Result<InitializedMessage, ServerError> {
        // Write lock is required for updating state
        let mut games = Self::write().await;

        // Consume the provided prepared config
        let config = games
            .preparing
            .remove(&uuid)
            .ok_or(ServerError::InvalidToken)?
            .config;

        // Create a new game token
        let token = GameToken::unique_token(&games.games);

        // Create the game
        let config = Arc::new(config);
        let game = Game::new(token, host_id, host_target, config.clone());
        let game = Arc::new(RwLock::new(game));

        // Insert the game into the games map
        games.games.insert(token, game.clone());

        Ok(InitializedMessage {
            token,
            config,
            game,
        })
    }

    /// Obtains a cloned Arc for a game with the specific token
    /// if one exists
    ///
    /// # Arguments
    /// * token - The token of the game to get
    pub async fn get_game(token: &GameToken) -> Option<GameRef> {
        Self::read().await.games.get(token).cloned()
    }

    /// Removes the game with the provided [`GameToken`] from
    /// the map of games
    pub async fn remove_game(token: GameToken) {
        Self::write().await.games.remove(&token);
    }
}
