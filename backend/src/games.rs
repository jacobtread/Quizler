use std::collections::HashMap;

use actix::Addr;

use crate::game::Game;

/// Central store for storing all the references to the individual
/// games that are currently running
pub struct Games {
    /// Map of the game tokens to the actual game itself
    values: HashMap<String, Addr<Game>>,
}
