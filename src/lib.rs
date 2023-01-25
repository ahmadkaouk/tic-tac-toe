use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use error::ContractError;

mod contract;
mod error;
mod game;
mod msg;
mod state;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: msg::InstantiateMsg,
) -> Result<Response, ContractError> {
    contract::instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: msg::ExecuteMsg,
) -> Result<Response, ContractError> {
    contract::execute(deps, env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: msg::QueryMsg) -> Result<Binary, ContractError> {
    contract::query(deps, env, msg)
}
