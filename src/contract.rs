use crate::game::Player;
use crate::state::GAMES;
use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response};

pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default().add_attribute("action", "instantiate"))
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::Invite { guest } => {
            let guest_addr = api.addr_validate(&guest)?;
            exec::invite(deps, info, &guest_addr)
        }
        ExecuteMsg::Accept { host } => {
            let host_addr = api.addr_validate(&host)?;
            exec::accept(deps, info, &host_addr)
        }
        ExecuteMsg::Reject { host } => {
            let host_addr = api.addr_validate(&host)?;
            exec::reject(deps, info, &host_addr)
        }
        ExecuteMsg::Play { host, guest, cell } => {
            let host_addr = api.addr_validate(&host)?;
            let guest_addr = api.addr_validate(&guest)?;
            exec::play(deps, info, &host_addr, &guest_addr, cell)
        }
    }
}

mod exec {
    use super::*;
    use crate::{game::Game, state::Games};
    use cosmwasm_std::ensure;
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    pub fn invite(
        deps: DepsMut,
        info: MessageInfo,
        guest_addr: &Addr,
    ) -> Result<Response, ContractError> {
        let games = GAMES.load(deps.storage, (&info.sender, guest_addr));

        let games = if let Ok(mut games) = games {
            // Ensure that there is no game in progress. Otherwise, return an error.
            ensure!(
                games.current.is_none(),
                ContractError::GameInProgress {
                    host: info.sender.to_string(),
                    guest: guest_addr.to_string()
                }
            );
            // Set pending_invition to true. The game will be created when the guest accepts the invitation
            games.pending_invition = true;
            games
        } else {
            Games {
                pending_invition: true,
                host: get_host_role(&info.sender, guest_addr),
                current: None,
                completed: vec![],
            }
        };

        GAMES.save(deps.storage, (&info.sender, guest_addr), &games)?;

        Ok(Response::default()
            .add_attribute("action", "invite")
            .add_attribute("host", info.sender.to_string())
            .add_attribute("guest", guest_addr.to_string()))
    }

    pub fn accept(
        deps: DepsMut,
        info: MessageInfo,
        host_addr: &Addr,
    ) -> Result<Response, ContractError> {
        let mut games = GAMES.load(deps.storage, (host_addr, &info.sender))?;

        ensure!(
            games.pending_invition,
            ContractError::NoPendingInvitation {
                host: host_addr.to_string(),
                guest: info.sender.to_string()
            }
        );

        games.pending_invition = false;
        games.current = Some(Game::new());

        GAMES.save(deps.storage, (host_addr, &info.sender), &games)?;
        Ok(Response::default()
            .add_attribute("action", "accept invitation")
            .add_attribute("host", host_addr.to_string())
            .add_attribute("guest", info.sender.to_string()))
    }

    pub fn reject(
        deps: DepsMut,
        info: MessageInfo,
        host_addr: &Addr,
    ) -> Result<Response, ContractError> {
        let mut games = GAMES.load(deps.storage, (host_addr, &info.sender))?;

        ensure!(
            games.pending_invition,
            ContractError::NoPendingInvitation {
                host: host_addr.to_string(),
                guest: info.sender.to_string()
            }
        );
        games.pending_invition = false;

        GAMES.save(deps.storage, (host_addr, &info.sender), &games)?;

        Ok(Response::default()
            .add_attribute("action", "reject invitation")
            .add_attribute("host", host_addr.to_string())
            .add_attribute("guest", info.sender.to_string()))
    }

    pub fn play(
        deps: DepsMut,
        info: MessageInfo,
        host_addr: &Addr,
        guest_addr: &Addr,
        cell: usize,
    ) -> Result<Response, ContractError> {
        let mut games = GAMES.load(deps.storage, (host_addr, guest_addr))?;

        let game = games
            .current
            .as_mut()
            .ok_or(ContractError::NoGameInProgress {
                host: host_addr.to_string(),
                guest: guest_addr.to_string(),
            })?;

        let player = if info.sender == *host_addr {
            games.host
        } else if info.sender == *guest_addr {
            if games.host == Player::O {
                Player::X
            } else {
                Player::O
            }
        } else {
            return Err(ContractError::NotInvolved {
                host: host_addr.to_string(),
                guest: guest_addr.to_string(),
                player: info.sender.to_string(),
            });
        };

        game.play(player, cell)?;

        if game.is_over() {
            games.completed.push(*game);
            games.current = None;
        }

        GAMES.save(deps.storage, (host_addr, guest_addr), &games)?;

        Ok(Response::default()
            .add_attribute("action", "play")
            .add_attribute("host", host_addr.to_string())
            .add_attribute("guest", guest_addr.to_string())
            .add_attribute("cell", cell.to_string()))
    }

    /// Get the host role based on the hash of the inviter and guest addresses.
    ///
    /// The first bit of the hash of the two addresses is used to determine the host symbol. If the first bit is 0,
    /// the host symbol is O, otherwise it is X.
    fn get_host_role(host_addr: &Addr, guest_addr: &Addr) -> Player {
        let concat = format!("{host_addr}{guest_addr}");
        let mut hasher = DefaultHasher::new();
        concat.hash(&mut hasher);
        let hash = hasher.finish().to_string();

        let first_bit = hash.as_bytes()[0] & 1;
        if first_bit == 0 {
            Player::O
        } else {
            Player::X
        }
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Games { host, guest } => {
            let host_addr = deps.api.addr_validate(&host)?;
            let guest_addr = deps.api.addr_validate(&guest)?;
            Ok(to_binary(&query::games(deps, &host_addr, &guest_addr)?)?)
        }
        QueryMsg::AllGamesList {} => Ok(to_binary(&query::all_games_list(deps)?)?),
    }
}

mod query {
    use super::*;
    use crate::msg::{AllGamesListResponse, GamesInfo, GamesResponse};
    use cosmwasm_std::{Order, StdResult};

    pub fn games(
        deps: Deps,
        host_addr: &Addr,
        guest_addr: &Addr,
    ) -> Result<GamesResponse, ContractError> {
        let games = GAMES.load(deps.storage, (host_addr, guest_addr))?;

        let game_info = GamesInfo {
            host: host_addr.to_string(),
            guest: guest_addr.to_string(),
            host_role: games.host,
            guest_role: if games.host == Player::O {
                Player::X
            } else {
                Player::O
            },
            pending_invitation: games.pending_invition,
            current_game: games.current,
            completed_games: games.completed,
        };
        Ok(GamesResponse { info: game_info })
    }

    pub fn all_games_list(deps: Deps) -> Result<AllGamesListResponse, ContractError> {
        let games: StdResult<Vec<_>> = GAMES
            .range(deps.storage, None, None, Order::Ascending)
            .map(|game| {
                let (key, value) = game?;
                Ok(GamesInfo {
                    host: key.0.to_string(),
                    guest: key.1.to_string(),
                    host_role: value.host,
                    guest_role: if value.host == Player::O {
                        Player::X
                    } else {
                        Player::O
                    },
                    pending_invitation: value.pending_invition,
                    current_game: value.current,
                    completed_games: value.completed,
                })
            })
            .collect();

        Ok(AllGamesListResponse { games: games? })
    }
}
