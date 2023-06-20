use crate::{
    game::{Game, GameConfig},
    session::{EventListener, SessionId},
    types::{GameToken, ServerError},
};

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::RwLock, time::interval};
use uuid::Uuid;

static GAMES: RwLock<Games> = RwLock::const_new(Games::default());

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
    pub async fn is_game(token: &GameToken) -> bool {
        let lock = GAMES.read().await;
        lock.games.contains_key(token)
    }

    pub async fn get_game(token: &GameToken) -> Option<Arc<RwLock<Game>>> {
        let lock = GAMES.read().await;
        lock.games.get(token).cloned()
    }

    pub async fn remove_game(token: &GameToken) {
        let lock = GAMES.write().await;
        lock.games.remove(token);
    }

    pub async fn prepare(config: GameConfig) -> Uuid {
        let lock = GAMES.write().await;
        let id = Uuid::new_v4();
        let created = Instant::now();
        lock.preparing.insert(id, PreparingGame { config, created });
        id
    }

    async fn take_prepare(id: Uuid) -> Option<PreparingGame> {
        let lock = GAMES.write().await;

        // Find the config data from the pre init list
        lock.preparing.remove(&id)
    }

    pub async fn initialize(
        uuid: Uuid,
        id: SessionId,
        listener: EventListener,
    ) -> Result<InitializedMessage, ServerError> {
        let prep = Self::take_prepare(uuid)
            .await
            .ok_or(ServerError::InvalidToken)?;

        let config = prep.config;

        let config = Arc::new(config);

        // Create a new game token
        let token = GameToken::unique_token().await;

        let game = Game::new(token, id, listener, config.clone()).start();

        let game = Arc::new(RwLock::new(game));

        let lock = GAMES.write().await;
        lock.games.insert(token, game.clone());

        Ok(InitializedMessage { token, config })
    }

    pub fn init() {
        /// Interval to check for expired game prepares (5mins)
        const PREPARE_CHECK_INTERVAL: Duration = Duration::from_secs(60 * 5);

        /// The amount of time that must pass for a prepared game to be
        /// considered expired (20mins)
        const GAME_EXPIRY_TIME: Duration = Duration::from_secs(60 * 20);

        tokio::spawn(async move {
            let future = interval(PREPARE_CHECK_INTERVAL);

            loop {
                future.tick().await;

                let lock = GAMES.write().await;
                lock.preparing.retain(|_, value| {
                    let elapsed = value.created.elapsed();
                    elapsed < GAME_EXPIRY_TIME
                });
            }
        })
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
