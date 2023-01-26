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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        game::{Game, GameError},
        msg::{AllGamesListResponse, GamesInfo, GamesResponse},
    };
    use cosmwasm_std::{from_binary, StdError};
    use cw_multi_test::{App, ContractWrapper, Executor};

    // A macro rule to get an attribute value from an event
    macro_rules! attribute {
        ($event:expr, $key:expr) => {
            $event
                .attributes
                .iter()
                .find(|attr| attr.key == $key)
                .unwrap()
                .value
        };
    }

    #[test]
    fn proper_instantiation() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        let resp: AllGamesListResponse = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::AllGamesList {})
            .unwrap();

        assert_eq!(resp, AllGamesListResponse { games: vec![] });
    }

    #[test]
    fn send_invitation() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        let resp = app
            .execute_contract(
                Addr::unchecked("sender"),
                contract_addr.clone(),
                &ExecuteMsg::Invite {
                    guest: "guest".to_string(),
                },
                &[],
            )
            .unwrap();

        let event = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

        assert_eq!(attribute!(event, "action"), "invite");
        assert_eq!(attribute!(event, "host"), "sender");
        assert_eq!(attribute!(event, "guest"), "guest");

        let resp: GamesResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr,
                &QueryMsg::Games {
                    host: "sender".to_string(),
                    guest: "guest".to_string(),
                },
            )
            .unwrap();

        assert_eq!(resp.info.host, "sender");
        assert_eq!(resp.info.guest, "guest");
        assert_eq!(resp.info.pending_invitation, true);
    }

    #[test]
    fn invalid_invitation_game_in_progress() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Accept {
                host: "sender".to_string(),
            },
            &[],
        )
        .unwrap();

        let err = app
            .execute_contract(
                Addr::unchecked("sender"),
                contract_addr,
                &ExecuteMsg::Invite {
                    guest: "guest".to_string(),
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::GameInProgress {
                host: "sender".to_string(),
                guest: "guest".to_string()
            },
            err.downcast().unwrap()
        );
    }

    #[test]
    fn accept_invitation() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        let resp = app
            .execute_contract(
                Addr::unchecked("guest"),
                contract_addr.clone(),
                &ExecuteMsg::Accept {
                    host: "sender".to_string(),
                },
                &[],
            )
            .unwrap();

        let event = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

        assert_eq!(attribute!(event, "action"), "accept invitation");
        assert_eq!(attribute!(event, "host"), "sender");
        assert_eq!(attribute!(event, "guest"), "guest");

        let resp: GamesResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr,
                &QueryMsg::Games {
                    host: "sender".to_string(),
                    guest: "guest".to_string(),
                },
            )
            .unwrap();

        assert_eq!(resp.info.host, "sender");
        assert_eq!(resp.info.guest, "guest");
        assert_eq!(resp.info.pending_invitation, false);
        assert_eq!(resp.info.current_game.unwrap().board(), &[Player::None; 9]);
        assert_eq!(resp.info.current_game.unwrap().turn(), Player::X);
    }

    #[test]
    fn no_pending_invitation() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        // reject invitationj
        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Reject {
                host: "sender".to_string(),
            },
            &[],
        )
        .unwrap();

        let err = app
            .execute_contract(
                Addr::unchecked("guest"),
                contract_addr,
                &ExecuteMsg::Accept {
                    host: "sender".to_string(),
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::NoPendingInvitation {
                host: "sender".to_string(),
                guest: "guest".to_string()
            },
            err.downcast().unwrap()
        );
    }

    #[test]
    fn reject_invitation() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        let resp = app
            .execute_contract(
                Addr::unchecked("guest"),
                contract_addr.clone(),
                &ExecuteMsg::Reject {
                    host: "sender".to_string(),
                },
                &[],
            )
            .unwrap();

        let event = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

        assert_eq!(attribute!(event, "action"), "reject invitation");
        assert_eq!(attribute!(event, "host"), "sender");
        assert_eq!(attribute!(event, "guest"), "guest");

        let resp: GamesResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr,
                &QueryMsg::Games {
                    host: "sender".to_string(),
                    guest: "guest".to_string(),
                },
            )
            .unwrap();

        assert_eq!(resp.info.host, "sender");
        assert_eq!(resp.info.guest, "guest");
        assert_eq!(resp.info.pending_invitation, false);
        assert_eq!(resp.info.current_game, None);
    }

    #[test]
    fn invalid_reject() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Reject {
                host: "sender".to_string(),
            },
            &[],
        )
        .unwrap();

        let err = app
            .execute_contract(
                Addr::unchecked("guest"),
                contract_addr,
                &ExecuteMsg::Reject {
                    host: "sender".to_string(),
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::NoPendingInvitation {
                host: "sender".to_string(),
                guest: "guest".to_string()
            },
            err.downcast().unwrap()
        );
    }

    #[test]
    fn valid_play() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        // invite
        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        // accept invitation
        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Accept {
                host: "sender".to_string(),
            },
            &[],
        )
        .unwrap();

        // play
        let resp = app
            .execute_contract(
                Addr::unchecked("guest"),
                contract_addr.clone(),
                &ExecuteMsg::Play {
                    host: "sender".to_string(),
                    guest: "guest".to_string(),
                    cell: 4,
                },
                &[],
            )
            .unwrap();

        let event = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

        assert_eq!(attribute!(event, "action"), "play");
        assert_eq!(attribute!(event, "host"), "sender");
        assert_eq!(attribute!(event, "guest"), "guest");
        assert_eq!(attribute!(event, "cell"), "4");

        let resp: GamesResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr,
                &QueryMsg::Games {
                    host: "sender".to_string(),
                    guest: "guest".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            GamesInfo {
                host: "sender".to_string(),
                guest: "guest".to_string(),
                host_role: Player::O,
                guest_role: Player::X,
                current_game: Some(Game {
                    board: [
                        Player::None,
                        Player::None,
                        Player::None,
                        Player::None,
                        Player::X,
                        Player::None,
                        Player::None,
                        Player::None,
                        Player::None
                    ],
                    turn: Player::O,
                }),
                pending_invitation: false,
                completed_games: vec![]
            },
            resp.info
        );
    }

    #[test]
    fn not_your_turn() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        // invite
        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        // accept invitation
        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Accept {
                host: "sender".to_string(),
            },
            &[],
        )
        .unwrap();

        // play
        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Play {
                host: "sender".to_string(),
                guest: "guest".to_string(),
                cell: 4,
            },
            &[],
        )
        .unwrap();

        let err = app
            .execute_contract(
                Addr::unchecked("guest"),
                contract_addr,
                &ExecuteMsg::Play {
                    host: "sender".to_string(),
                    guest: "guest".to_string(),
                    cell: 5,
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::GameError((GameError::NotYourTurn).into()),
            err.downcast().unwrap()
        );
    }

    #[test]
    fn invalid_cell() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        // invite
        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        // accept invitation
        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Accept {
                host: "sender".to_string(),
            },
            &[],
        )
        .unwrap();

        // play
        let err = app
            .execute_contract(
                Addr::unchecked("guest"),
                contract_addr,
                &ExecuteMsg::Play {
                    host: "sender".to_string(),
                    guest: "guest".to_string(),
                    cell: 10,
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::GameError((GameError::InvalidMove(10)).into()),
            err.downcast().unwrap()
        );
    }

    #[test]
    fn cell_already_taken() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        // invite
        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        // accept invitation
        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Accept {
                host: "sender".to_string(),
            },
            &[],
        )
        .unwrap();

        // play
        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Play {
                host: "sender".to_string(),
                guest: "guest".to_string(),
                cell: 4,
            },
            &[],
        )
        .unwrap();

        let err = app
            .execute_contract(
                Addr::unchecked("sender"),
                contract_addr,
                &ExecuteMsg::Play {
                    host: "sender".to_string(),
                    guest: "guest".to_string(),
                    cell: 4,
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::GameError((GameError::InvalidMove(4)).into()),
            err.downcast().unwrap()
        );
    }

    #[test]
    fn game_not_found() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        let err = app
            .execute_contract(
                Addr::unchecked("sender"),
                contract_addr,
                &ExecuteMsg::Play {
                    host: "sender".to_string(),
                    guest: "guest".to_string(),
                    cell: 4,
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::StdError(
                (StdError::NotFound {
                    kind: "tic_tac_toe::state::Games".to_string()
                })
                .into()
            ),
            err.downcast().unwrap()
        );
    }

    #[test]
    fn player_not_in_game() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        // invite
        app.execute_contract(
            Addr::unchecked("host"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        // accept invitation
        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Accept {
                host: "host".to_string(),
            },
            &[],
        )
        .unwrap();

        // play
        let err = app
            .execute_contract(
                Addr::unchecked("player"),
                contract_addr,
                &ExecuteMsg::Play {
                    host: "host".to_string(),
                    guest: "guest".to_string(),
                    cell: 4,
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::NotInvolved {
                player: "player".to_string(),
                host: "host".to_string(),
                guest: "guest".to_string()
            },
            err.downcast().unwrap()
        );
    }

    #[test]
    fn game_over_winner_x() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        // invite
        app.execute_contract(
            Addr::unchecked("host"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        // accept invitation
        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Accept {
                host: "host".to_string(),
            },
            &[],
        )
        .unwrap();

        // play
        play(&mut app, contract_addr.clone(), "host", 0);
        play(&mut app, contract_addr.clone(), "guest", 1);
        play(&mut app, contract_addr.clone(), "host", 3);
        play(&mut app, contract_addr.clone(), "guest", 5);
        play(&mut app, contract_addr.clone(), "host", 6);

        let resp = app
            .wrap()
            .query_wasm_smart(
                contract_addr,
                &QueryMsg::Games {
                    host: "host".to_string(),
                    guest: "guest".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            GamesResponse {
                info: GamesInfo {
                    host: "host".to_string(),
                    guest: "guest".to_string(),
                    host_role: Player::X,
                    guest_role: Player::O,
                    pending_invitation: false,
                    current_game: None,
                    completed_games: vec![Game {
                        board: [
                            Player::X,
                            Player::O,
                            Player::None,
                            Player::X,
                            Player::None,
                            Player::O,
                            Player::X,
                            Player::None,
                            Player::None,
                        ],
                        turn: Player::O,
                    }]
                },
            },
            resp
        );
        assert!(resp.info.completed_games[0].is_over());
        assert_eq!(Player::X, resp.info.completed_games[0].winner().unwrap());
    }

    #[test]
    fn game_over_with_draw() {
        let mut app = App::default();
        let contract_addr = contract_address(&mut app);

        // invite
        app.execute_contract(
            Addr::unchecked("host"),
            contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: "guest".to_string(),
            },
            &[],
        )
        .unwrap();

        // accept invitation
        app.execute_contract(
            Addr::unchecked("guest"),
            contract_addr.clone(),
            &ExecuteMsg::Accept {
                host: "host".to_string(),
            },
            &[],
        )
        .unwrap();

        // play
        play(&mut app, contract_addr.clone(), "host", 0);
        play(&mut app, contract_addr.clone(), "guest", 4);
        play(&mut app, contract_addr.clone(), "host", 8);
        play(&mut app, contract_addr.clone(), "guest", 3);
        play(&mut app, contract_addr.clone(), "host", 5);
        play(&mut app, contract_addr.clone(), "guest", 2);
        play(&mut app, contract_addr.clone(), "host", 6);
        play(&mut app, contract_addr.clone(), "guest", 7);
        play(&mut app, contract_addr.clone(), "host", 1);

        let resp = app
            .wrap()
            .query_wasm_smart(
                contract_addr,
                &QueryMsg::Games {
                    host: "host".to_string(),
                    guest: "guest".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            GamesResponse {
                info: GamesInfo {
                    host: "host".to_string(),
                    guest: "guest".to_string(),
                    host_role: Player::X,
                    guest_role: Player::O,
                    pending_invitation: false,
                    current_game: None,
                    completed_games: vec![Game {
                        board: [
                            Player::X,
                            Player::X,
                            Player::O,
                            Player::O,
                            Player::O,
                            Player::X,
                            Player::X,
                            Player::O,
                            Player::X,
                        ],
                        turn: Player::O,
                    }]
                },
            },
            resp
        );
        assert!(resp.info.completed_games[0].is_over());
        assert!(resp.info.completed_games[0].winner().is_none());
    }

    fn invite(app: &mut App, contract_addr: Addr, host: &str, guest: &str) {
        app.execute_contract(
            Addr::unchecked(host),
            contract_addr,
            &ExecuteMsg::Invite {
                guest: guest.to_string(),
            },
            &[],
        )
        .unwrap();
    }

    fn accept(app: &mut App, contract_addr: Addr, host: &str, guest: &str) {
        app.execute_contract(
            Addr::unchecked(guest),
            contract_addr,
            &ExecuteMsg::Accept {
                host: host.to_string(),
            },
            &[],
        )
        .unwrap();
    }

    fn init_game(app: &mut App, contract_addr: Addr, host: &str, guest: &str) {
        invite(app, contract_addr.clone(), host, guest);
        accept(app, contract_addr.clone(), host, guest);
    }

    fn play(app: &mut App, contract_addr: Addr, player: &str, cell: usize) {
        app.execute_contract(
            Addr::unchecked(player),
            contract_addr,
            &ExecuteMsg::Play {
                host: "host".to_string(),
                guest: "guest".to_string(),
                cell,
            },
            &[],
        )
        .unwrap();
    }

    fn contract_address(app: &mut App) -> Addr {
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        let sender = Addr::unchecked("Owner");

        app.instantiate_contract(code_id, sender, &InstantiateMsg {}, &[], "Contract", None)
            .unwrap()
    }
}
