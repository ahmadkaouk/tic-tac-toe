use crate::game::GameError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    StdError(#[from] StdError),
    #[error("{0}")]
    GameError(#[from] GameError),
    #[error("A Game in progress already exists between {host} and {guest}")]
    GameInProgress { host: String, guest: String },
    #[error("No pending invitation for {guest} from {host}")]
    NoPendingInvitation { host: String, guest: String },
    #[error("No game in progress between {host} and {guest}")]
    NoGameInProgress { host: String, guest: String },
    #[error("The player {player} is not involved in a game between {host} and {guest}")]
    NotInvolved {
        host: String,
        guest: String,
        player: String,
    },
}
