use crate::{
    game::{Game, GameConfig},
    session::{EventTarget, SessionId},
    types::{GameToken, ServerError},
};

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::RwLock, time::interval};
use uuid::Uuid;

static mut GAMES: Option<RwLock<Games>> = None;

/// Central store for storing all the references to the individual
/// games that are currently running
#[derive(Default)]
pub struct Games {
    /// Map of the game tokens to the actual game itself
    games: HashMap<GameToken, Arc<RwLock<Game>>>,
    /// Map of UUID's to game configurations that are preparing to start
    preparing: HashMap<Uuid, PreparingGame>,
}

impl Games {
    async fn get() -> &'static RwLock<Games> {
        match unsafe { &GAMES } {
            Some(value) => value,
            None => panic!("Games not initialized"),
        }
    }

    pub async fn is_game(token: &GameToken) -> bool {
        let lock = Self::get().await.read().await;
        lock.games.contains_key(token)
    }

    pub async fn get_game(token: &GameToken) -> Option<Arc<RwLock<Game>>> {
        let lock = Self::get().await.read().await;
        lock.games.get(token).cloned()
    }

    pub async fn remove_game(token: &GameToken) {
        let mut lock = Self::get().await.write().await;
        lock.games.remove(token);
    }

    pub async fn prepare(config: GameConfig) -> Uuid {
        let id = Uuid::new_v4();
        let created = Instant::now();
        let mut lock = Self::get().await.write().await;

        lock.preparing.insert(id, PreparingGame { config, created });
        id
    }

    async fn take_prepare(id: Uuid) -> Option<PreparingGame> {
        let mut lock = Self::get().await.write().await;

        // Find the config data from the pre init list
        lock.preparing.remove(&id)
    }

    pub async fn initialize(
        uuid: Uuid,
        id: SessionId,
        listener: EventTarget,
    ) -> Result<InitializedMessage, ServerError> {
        let prep = Self::take_prepare(uuid)
            .await
            .ok_or(ServerError::InvalidToken)?;

        let config = prep.config;

        let config = Arc::new(config);

        // Create a new game token
        let token = GameToken::unique_token().await;

        let game = Game::new(token, id, listener, config.clone());

        let game = Arc::new(RwLock::new(game));

        let mut lock = Self::get().await.write().await;
        lock.games.insert(token, game.clone());

        Ok(InitializedMessage { token, config })
    }

    pub fn init() {
        unsafe {
            GAMES = Some(RwLock::new(Games::default()));
        }

        /// Interval to check for expired game prepares (5mins)
        const PREPARE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 5);

        /// The amount of time that must pass for a prepared game to be
        /// considered expired (20mins)
        const GAME_EXPIRY_TIME: Duration = Duration::from_secs(60 * 20);

        tokio::spawn(async move {
            let mut future = interval(PREPARE_CHECK_INTERVAL);

            loop {
                future.tick().await;

                let mut lock = Games::get().await.write().await;
                lock.preparing.retain(|_, value| {
                    let elapsed = value.created.elapsed();
                    elapsed < GAME_EXPIRY_TIME
                });
            }
        });
    }
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
}
