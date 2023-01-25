use crate::game::{Game, Player};
use cosmwasm_std::Addr;
use cw_storage_plus::Map;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Games {
    pub pending_invition: bool,
    pub host: Player,
    pub current: Option<Game>,
    pub completed: Vec<Game>,
}

pub const GAMES: Map<(&Addr, &Addr), Games> = Map::new("games");
