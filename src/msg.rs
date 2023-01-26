use cosmwasm_schema::cw_serde;

use crate::game::{Game, Player};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// Invite a player to play a game.
    Invite {
        /// The address of the player to invite.
        guest: String,
    },
    /// Accept an invitation to play a game.
    Accept {
        /// The address of the player who invited you.
        host: String,
    },
    /// Reject an invitation to play a game.
    Reject {
        /// The address of the player who invited you.
        host: String,
    },
    /// Play a move in the game.
    Play {
        /// The address of the host of the game.
        host: String,
        /// The address of the guest of the game.
        guest: String,
        /// The cell to play in.
        cell: usize,
    },
}

#[cw_serde]
pub enum QueryMsg {
    /// Get all the games between two players.
    Games {
        /// The address of the host of the game.
        host: String,
        /// The address of the guest of the game.
        guest: String,
    },
    /// Get all the games for all players.
    AllGamesList {},
}

/// The information about games between two players.
#[cw_serde]
pub struct GamesInfo {
    pub host: String,
    pub guest: String,
    pub host_role: Player,
    pub guest_role: Player,
    pub pending_invitation: bool,
    pub current_game: Option<Game>,
    pub completed_games: Vec<Game>,
}

/// All the games between two players.
#[cw_serde]
pub struct GamesResponse {
    pub info: GamesInfo,
}

/// A list of games.
#[cw_serde]
pub struct AllGamesListResponse {
    pub games: Vec<GamesInfo>,
}
