use std::collections::HashMap;

use actix::{Actor, Addr, Context};

use crate::session::{Session, SessionId};

pub struct Game {
    /// The ID of the game
    id: GameId,
    /// Map of session IDs mapped to the session address
    players: HashMap<SessionId, GameSession>,
}

pub struct GameSession {
    /// The ID of the session
    id: SessionId,
    /// The player name
    name: String,
    /// Address to the session
    addr: Addr<Session>,
}

pub type GameId = u32;

impl Actor for Game {
    type Context = Context<Self>;
}
