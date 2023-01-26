use crate::{
    contract,
    game::{Game, GameError, Player},
    msg::{AllGamesListResponse, ExecuteMsg, GamesInfo, GamesResponse, InstantiateMsg, QueryMsg}, error::ContractError,
};
use anyhow::Error;
use cosmwasm_std::{Addr, StdError};
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};

/// This is a helper struct to make testing easier.
pub struct GameMock {
    app: App,
    contract_addr: Addr,
}
impl GameMock {
    /// Creates a new GameMock instance.
    pub fn new() -> Self {
        let mut app = App::default();
        let code = ContractWrapper::new(contract::execute, contract::instantiate, contract::query);
        let code_id = app.store_code(Box::new(code));
        let sender = Addr::unchecked("Owner");

        let contract_addr = app
            .instantiate_contract(code_id, sender, &InstantiateMsg {}, &[], "Contract", None)
            .unwrap();
        Self { app, contract_addr }
    }

    /// Initializes a game with the given host and guest.
    pub fn init_game(&mut self, host: &str, guest: &str) {
        self.invite(host, guest).unwrap();
        self.accept(host, guest).unwrap();
    }

    /// Simulates a player sending an invitation to another player.
    pub fn invite(&mut self, host: &str, guest: &str) -> Result<AppResponse, Error> {
        self.app.execute_contract(
            Addr::unchecked(host),
            self.contract_addr.clone(),
            &ExecuteMsg::Invite {
                guest: guest.to_string(),
            },
            &[],
        )
    }

    /// Sends an acceptance of an invitation.
    pub fn accept(&mut self, host: &str, guest: &str) -> Result<AppResponse, Error> {
        self.app.execute_contract(
            Addr::unchecked(guest),
            self.contract_addr.clone(),
            &ExecuteMsg::Accept {
                host: host.to_string(),
            },
            &[],
        )
    }

    /// Sends a rejection of an invitation.
    pub fn reject(&mut self, host: &str, guest: &str) -> Result<AppResponse, Error> {
        self.app.execute_contract(
            Addr::unchecked(guest),
            self.contract_addr.clone(),
            &ExecuteMsg::Reject {
                host: host.to_string(),
            },
            &[],
        )
    }

    /// Executes a play by the given player.
    pub fn play(&mut self, player: &str, cell: usize) -> Result<AppResponse, Error> {
        self.app.execute_contract(
            Addr::unchecked(player),
            self.contract_addr.clone(),
            &ExecuteMsg::Play {
                host: "host".to_string(),
                guest: "guest".to_string(),
                cell,
            },
            &[],
        )
    }

    /// Queries the contract for the games of the given host and guest.
    pub fn query_games(&self, host: &str, guest: &str) -> Result<GamesResponse, StdError> {
        self.app.wrap().query_wasm_smart(
            self.contract_addr.clone(),
            &QueryMsg::Games {
                host: host.to_string(),
                guest: guest.to_string(),
            },
        )
    }

    pub fn query_all_games(&self) -> Result<AllGamesListResponse, StdError> {
        self.app
            .wrap()
            .query_wasm_smart(self.contract_addr.clone(), &QueryMsg::AllGamesList {})
    }
}
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
    let game_mock = GameMock::new();

    let resp = game_mock.query_all_games().unwrap();
    assert_eq!(resp, AllGamesListResponse { games: vec![] });
}

#[test]
fn send_invitation() {
    let mut game_mock = GameMock::new();
    let resp = game_mock.invite("host", "guest").unwrap();

    let event = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
    assert_eq!(attribute!(event, "action"), "invite");
    assert_eq!(attribute!(event, "host"), "host");
    assert_eq!(attribute!(event, "guest"), "guest");

    let resp = game_mock.query_games("host", "guest").unwrap();
    assert_eq!(resp.info.host, "host");
    assert_eq!(resp.info.guest, "guest");
    assert_eq!(resp.info.pending_invitation, true);
}

#[test]
fn invalid_invitation_game_in_progress() {
    let mut game_mock = GameMock::new();
    game_mock.init_game("host", "guest");

    let err = game_mock.invite("host", "guest").unwrap_err();
    assert_eq!(
        ContractError::GameInProgress {
            host: "host".to_string(),
            guest: "guest".to_string()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn accept_invitation() {
    let mut game_mock = GameMock::new();
    game_mock.invite("host", "guest").unwrap();

    let resp = game_mock.accept("host", "guest").unwrap();
    let event = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
    assert_eq!(attribute!(event, "action"), "accept invitation");
    assert_eq!(attribute!(event, "host"), "host");
    assert_eq!(attribute!(event, "guest"), "guest");

    let resp = game_mock.query_games("host", "guest").unwrap();
    assert_eq!(resp.info.host, "host");
    assert_eq!(resp.info.guest, "guest");
    assert_eq!(resp.info.pending_invitation, false);
    assert_eq!(resp.info.current_game.unwrap().board(), &[Player::None; 9]);
    assert_eq!(resp.info.current_game.unwrap().turn(), Player::X);
}

#[test]
fn no_pending_invitation() {
    let mut game_mock = GameMock::new();
    game_mock.invite("host", "guest").unwrap();
    game_mock.reject("host", "guest").unwrap();

    let err = game_mock.accept("host", "guest").unwrap_err();
    assert_eq!(
        ContractError::NoPendingInvitation {
            host: "host".to_string(),
            guest: "guest".to_string()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn reject_invitation() {
    let mut game_mock = GameMock::new();
    game_mock.invite("host", "guest").unwrap();

    let resp = game_mock.reject("host", "guest").unwrap();
    let event = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
    assert_eq!(attribute!(event, "action"), "reject invitation");
    assert_eq!(attribute!(event, "host"), "host");
    assert_eq!(attribute!(event, "guest"), "guest");

    let resp = game_mock.query_games("host", "guest").unwrap();
    assert_eq!(resp.info.host, "host");
    assert_eq!(resp.info.guest, "guest");
    assert_eq!(resp.info.pending_invitation, false);
    assert_eq!(resp.info.current_game, None);
}

#[test]
fn invalid_reject() {
    let mut game_mock = GameMock::new();
    game_mock.init_game("host", "guest");

    let err = game_mock.reject("host", "guest").unwrap_err();
    assert_eq!(
        ContractError::NoPendingInvitation {
            host: "host".to_string(),
            guest: "guest".to_string()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn valid_play() {
    let mut game_mock = GameMock::new();
    game_mock.init_game("host", "guest");

    // play
    let resp = game_mock.play("host", 4).unwrap();

    let event = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
    assert_eq!(attribute!(event, "action"), "play");
    assert_eq!(attribute!(event, "host"), "host");
    assert_eq!(attribute!(event, "guest"), "guest");
    assert_eq!(attribute!(event, "cell"), "4");

    let resp: GamesResponse = game_mock.query_games("host", "guest").unwrap();
    assert_eq!(
        GamesInfo {
            host: "host".to_string(),
            guest: "guest".to_string(),
            host_role: Player::X,
            guest_role: Player::O,
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
    let mut game_mock = GameMock::new();
    game_mock.init_game("host", "guest");

    // play
    game_mock.play("host", 4).unwrap();

    let err = game_mock.play("host", 5).unwrap_err();
    assert_eq!(
        ContractError::GameError((GameError::NotYourTurn).into()),
        err.downcast().unwrap()
    );
}

#[test]
fn invalid_cell() {
    let mut game_mock = GameMock::new();
    game_mock.init_game("host", "guest");

    // play
    let err = game_mock.play("host", 10).unwrap_err();
    assert_eq!(
        ContractError::GameError((GameError::InvalidMove(10)).into()),
        err.downcast().unwrap()
    );
}

#[test]
fn cell_already_taken() {
    let mut game_mock = GameMock::new();
    game_mock.init_game("host", "guest");

    game_mock.play("host", 4).unwrap();
    let err = game_mock.play("guest", 4).unwrap_err();

    assert_eq!(
        ContractError::GameError((GameError::InvalidMove(4)).into()),
        err.downcast().unwrap()
    );
}

#[test]
fn game_not_found() {
    let mut game_mock = GameMock::new();
    let err = game_mock.play("host", 0).unwrap_err();

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
    let mut game_mock = GameMock::new();
    game_mock.init_game("host", "guest");

    let err = game_mock.play("player", 0).unwrap_err();
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
    let mut game_mock = GameMock::new();
    game_mock.init_game("host", "guest");

    // play
    game_mock.play("host", 0).unwrap();
    game_mock.play("guest", 1).unwrap();
    game_mock.play("host", 3).unwrap();
    game_mock.play("guest", 5).unwrap();
    game_mock.play("host", 6).unwrap();

    let resp = game_mock.query_games("host", "guest").unwrap();

    assert!(resp.info.completed_games[0].is_over());
    assert_eq!(Player::X, resp.info.completed_games[0].winner().unwrap());
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
}

#[test]
fn game_over_with_draw() {
    let mut game_mock = GameMock::new();
    game_mock.init_game("host", "guest");

    // play
    game_mock.play("host", 0).unwrap();
    game_mock.play("guest", 4).unwrap();
    game_mock.play("host", 8).unwrap();
    game_mock.play("guest", 3).unwrap();
    game_mock.play("host", 5).unwrap();
    game_mock.play("guest", 2).unwrap();
    game_mock.play("host", 6).unwrap();
    game_mock.play("guest", 7).unwrap();
    game_mock.play("host", 1).unwrap();

    let resp = game_mock.query_games("host", "guest").unwrap();
    assert!(resp.info.completed_games[0].is_over());
    assert!(resp.info.completed_games[0].winner().is_none());
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
}
