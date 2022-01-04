#[cfg(test)]
mod testing;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{BalanceResponse as CW20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ERRORS
// -----------------------------------------------------

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Paused")]
    Paused {},
    #[error("UnbondBalanceTooLow")]
    UnbondBalanceTooLow {},
    #[error("UnbondBefore7Days")]
    UnbondBefore7Days {},
    // See https://docs.rs/thiserror/1.0.21/thiserror/
}

// MESSAGES
// -----------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub token: String,
    pub locked_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Configure { paused: bool, locked_period: u64 },
    TransferOwnership { owner: Addr },
    Unbond { amount: Uint128 },
    UnbondNow { amount: Uint128 },
    Migrate { new_contract: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Bond {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    State {},
    UserState { user: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub owner: String,
    pub token: String,
    pub paused: bool,
    pub locked_period: u64,
    pub total_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserStateResponse {
    pub last_deposit: u64,
    pub balance: Uint128,
}

// STATE
// -----------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr,
    pub token: CanonicalAddr,
    pub paused: bool,
    pub locked_period: u64,
    pub total_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct UserState {
    pub last_deposit: u64,
    pub balance: Uint128,
}

pub const STATE: Item<State> = Item::new("state");
pub const USERS_STATE: Map<&Addr, UserState> = Map::new("users");

// CONTRACT
// -----------------------------------------------------

const CONTRACT_NAME: &str = "ts-tiers";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let token_addr = deps.api.addr_canonicalize(&msg.token)?;
    let state = State {
        owner: sender_addr.clone(),
        token: token_addr.clone(),
        paused: false,
        locked_period: msg.locked_period,
        total_balance: Uint128::zero(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "instantiate"),
        ("owner", sender_addr.to_string().as_str()),
        ("token", token_addr.to_string().as_str()),
        ("locked_period", msg.locked_period.to_string().as_str()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Configure {
            paused,
            locked_period,
        } => configure(deps, env, info, paused, locked_period),
        ExecuteMsg::TransferOwnership { owner } => transfer_ownership(deps, env, info, owner),
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Unbond { amount } => unbond(deps, env, info, amount),
        ExecuteMsg::UnbondNow { amount } => unbond_now(deps, env, info, amount),
        ExecuteMsg::Migrate { new_contract } => migrate(deps, env, info, new_contract),
    }
}

pub fn configure(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    paused: bool,
    locked_period: u64,
) -> Result<Response, ContractError> {
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let state = STATE.load(deps.storage)?;
    if state.owner != sender_addr {
        return Err(ContractError::Unauthorized {});
    }

    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.paused = paused;
        state.locked_period = locked_period;
        Ok(state)
    })?;

    Ok(Response::new().add_attributes(vec![
        ("action", "configure"),
        ("paused", paused.to_string().as_str()),
        ("locked_period", locked_period.to_string().as_str()),
    ]))
}

pub fn transfer_ownership(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Addr,
) -> Result<Response, ContractError> {
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let state = STATE.load(deps.storage)?;
    if state.owner != sender_addr {
        return Err(ContractError::Unauthorized {});
    }

    let owner_addr = deps.api.addr_canonicalize(owner.as_str())?;
    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.owner = owner_addr;
        Ok(state)
    })?;

    Ok(Response::new().add_attributes(vec![
        ("action", "transfer_ownership"),
        ("owner", owner.to_string().as_str()),
    ]))
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let state = STATE.load(deps.storage)?;

    match from_binary(&msg.msg) {
        Ok(Cw20HookMsg::Bond {}) => {
            if state.token != sender_addr {
                return Err(ContractError::Unauthorized {});
            }
            let cw20_sender_addr = deps.api.addr_canonicalize(&msg.sender)?;
            bond(deps, env, cw20_sender_addr, msg.amount)
        }
        Err(_) => Err(ContractError::Std(StdError::generic_err(
            "missing message data",
        ))),
    }
}

pub fn bond(
    deps: DepsMut,
    env: Env,
    sender_addr: CanonicalAddr,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_humanize(&sender_addr)?;
    let state = STATE.load(deps.storage)?;
    if state.paused {
        return Err(ContractError::Paused {});
    }

    let new_user_state =
        USERS_STATE.update(deps.storage, &sender, |maybe_user_state| -> StdResult<_> {
            let mut user_state = maybe_user_state.unwrap_or_default();
            user_state.balance += amount;
            user_state.last_deposit = env.block.time.seconds();
            Ok(user_state)
        })?;

    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.total_balance += amount;
        Ok(state)
    })?;

    Ok(Response::new().add_attributes(vec![
        ("action", "bond"),
        ("user", sender.to_string().as_str()),
        ("tokens", amount.to_string().as_str()),
        ("balance", new_user_state.balance.to_string().as_str()),
    ]))
}

pub fn unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let contract_addr = deps
        .api
        .addr_canonicalize(env.contract.address.to_string().as_str())?;
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let sender = deps.api.addr_humanize(&sender_addr)?;
    let state = STATE.load(deps.storage)?;
    if state.paused {
        return Err(ContractError::Paused {});
    }

    let new_user_state = USERS_STATE.update(
        deps.storage,
        &sender,
        |maybe_user_state| -> Result<_, ContractError> {
            let mut user_state = maybe_user_state.unwrap();
            if user_state.balance < amount {
                return Err(ContractError::UnbondBalanceTooLow {});
            }
            if env.block.time.seconds() < user_state.last_deposit + state.locked_period {
                return Err(ContractError::UnbondBefore7Days {});
            }
            user_state.balance -= amount;
            Ok(user_state)
        },
    )?;

    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.total_balance -= amount;
        Ok(state)
    })?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&state.token)?.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender.to_string(),
                amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "unbond"),
            ("user", sender.to_string().as_str()),
            ("change", amount.to_string().as_str()),
            ("balance", new_user_state.balance.to_string().as_str()),
        ]))
}

pub fn unbond_now(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let sender = deps.api.addr_humanize(&sender_addr)?;
    let state = STATE.load(deps.storage)?;

    let new_user_state = USERS_STATE.update(
        deps.storage,
        &sender,
        |maybe_user_state| -> Result<_, ContractError> {
            let mut user_state = maybe_user_state.unwrap();
            if user_state.balance < amount {
                return Err(ContractError::UnbondBalanceTooLow {});
            }
            user_state.balance -= amount;
            Ok(user_state)
        },
    )?;

    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.total_balance -= amount;
        Ok(state)
    })?;

    Ok(Response::new()
        .add_messages(vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&state.token)?.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: sender.to_string(),
                    amount: amount / Uint128::from(2u128),
                })?,
                funds: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&state.token)?.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: deps.api.addr_humanize(&state.owner)?.to_string(),
                    amount: amount - (amount / Uint128::from(2u128)),
                })?,
                funds: vec![],
            }),
        ])
        .add_attributes(vec![
            ("action", "unbond_now"),
            ("user", sender.to_string().as_str()),
            ("change", amount.to_string().as_str()),
            ("balance", new_user_state.balance.to_string().as_str()),
        ]))
}

pub fn migrate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _new_contract: String,
) -> Result<Response, ContractError> {
    Err(ContractError::Std(StdError::generic_err("unimplemented")))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::UserState { user } => to_binary(&query_user_state(deps, user)?),
    }
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        owner: deps.api.addr_humanize(&state.owner)?.to_string(),
        token: deps.api.addr_humanize(&state.token)?.to_string(),
        paused: state.paused,
        locked_period: state.locked_period,
        total_balance: state.total_balance,
    })
}

fn query_user_state(deps: Deps, user: String) -> StdResult<UserStateResponse> {
    let user_addr = deps.api.addr_canonicalize(&user)?;
    let user = deps.api.addr_humanize(&user_addr)?;
    let user_state = USERS_STATE
        .may_load(deps.storage, &user)?
        .unwrap_or_default();
    Ok(UserStateResponse {
        last_deposit: user_state.last_deposit,
        balance: user_state.balance,
    })
}

// HELPERS
// -----------------------------------------------------

pub fn balance_of(
    deps: &DepsMut,
    token: &CanonicalAddr,
    owner: &CanonicalAddr,
) -> StdResult<Uint128> {
    let balance: CW20BalanceResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.addr_humanize(token)?.to_string(),
            msg: to_binary(&Cw20QueryMsg::Balance {
                address: deps.api.addr_humanize(owner)?.to_string(),
            })?,
        }))?;
    Ok(Uint128::from(balance.balance))
}

/*
pub fn total_supply(deps: Deps, token: String) -> StdResult<Uint128> {
    let token_info: TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: token,
            msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))?;

    Ok(Uint128::from(token_info.total_supply))
}
*/
