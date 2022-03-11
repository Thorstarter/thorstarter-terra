#[cfg(test)]
mod testing;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha3::Digest;
use std::convert::TryInto;
use terra_cosmwasm::TerraQuerier;
use thiserror::Error;

// ERRORS
// -----------------------------------------------------

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Unauthorized")]
    Unauthorized {},
    #[error("NoZeroAmount")]
    NoZeroAmount,
    #[error("NoOtherDenoms")]
    NoOtherDenoms,
    #[error("NotConfigured")]
    NotConfigured,
    #[error("DepositNotStarted")]
    DepositNotStarted {},
    #[error("DepositEnded")]
    DepositEnded {},
    #[error("WithdrawNotStarted")]
    WithdrawNotStarted {},
    #[error("WithdrawEnded")]
    WithdrawEnded {},
    #[error("NotFinalized")]
    NotFinalized {},
    #[error("InvalidMerkleProof")]
    InvalidMerkleProof {},
    #[error("OverAllocation")]
    OverAllocation {},
}

// MESSAGES
// -----------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Configure {
        token: String,
        start_time: u64,
        end_deposit_time: u64,
        end_withdraw_time: u64,
        offering_amount: Uint128,
        vesting_initial: Uint128,
        vesting_time: u64,
        merkle_root: String,
        finalized: bool,
    },
    Deposit {
        allocation: Uint128,
        proof: Vec<String>,
    },
    Withdraw {},
    Harvest {},
    Collect {},
    CollectTokens {
        amount: Uint128,
    },
    Migrate {
        new_contract: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    State {},
    UserState { user: String, now: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub owner: String,
    pub token: String,
    pub start_time: u64,
    pub end_deposit_time: u64,
    pub end_withdraw_time: u64,
    pub offering_amount: Uint128,
    pub vesting_initial: Uint128,
    pub vesting_time: u64,
    pub merkle_root: String,
    pub finalized: bool,
    pub total_users: u64,
    pub total_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserStateResponse {
    pub amount: Uint128,
    pub claimed: Uint128,
    pub owed: Uint128,
    pub claimable: Uint128,
}

// STATE
// -----------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr,
    pub token: CanonicalAddr,
    pub start_time: u64,
    pub end_deposit_time: u64,
    pub end_withdraw_time: u64,
    pub offering_amount: Uint128,
    pub vesting_initial: Uint128, // vested initially 1e6 = 100%
    pub vesting_time: u64,        // time past end_time to 100% vested
    pub merkle_root: String,
    pub finalized: bool,
    pub total_users: u64,
    pub total_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct UserState {
    pub amount: Uint128,
    pub claimed: Uint128,
}

pub const STATE: Item<State> = Item::new("state");
pub const USERS_STATE: Map<&Addr, UserState> = Map::new("users");

// CONTRACT
// -----------------------------------------------------

const CONTRACT_NAME: &str = "sale";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let default_addr = deps
        .api
        .addr_canonicalize("terra000000000000000000000000000000000000000")?;
    let empty_string = String::new();
    let state = State {
        owner: sender_addr.clone(),
        token: default_addr,
        start_time: 0,
        end_deposit_time: 0,
        end_withdraw_time: 0,
        offering_amount: Uint128::zero(),
        vesting_initial: Uint128::zero(),
        vesting_time: 0,
        merkle_root: String::new(),
        finalized: false,
        total_users: 0,
        total_amount: Uint128::zero(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "instantiate"),
        ("owner", sender_addr.to_string().as_str()),
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
            token,
            start_time,
            end_deposit_time,
            end_withdraw_time,
            offering_amount,
            vesting_initial,
            vesting_time,
            merkle_root,
            finalized,
        } => configure(
            deps,
            env,
            info,
            token,
            start_time,
            end_deposit_time,
            end_withdraw_time,
            offering_amount,
            vesting_initial,
            vesting_time,
            merkle_root,
            finalized,
        ),
        ExecuteMsg::Deposit { allocation, proof } => deposit(deps, env, info, allocation, proof),
        ExecuteMsg::Withdraw {} => withdraw(deps, env, info),
        ExecuteMsg::Harvest {} => harvest(deps, env, info),
        ExecuteMsg::Collect {} => collect(deps, env, info),
        ExecuteMsg::CollectTokens { amount } => collect_tokens(deps, env, info, amount),
        ExecuteMsg::Migrate { new_contract } => migrate(deps, env, info, new_contract),
    }
}

pub fn configure(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token: String,
    start_time: u64,
    end_deposit_time: u64,
    end_withdraw_time: u64,
    offering_amount: Uint128,
    vesting_initial: Uint128,
    vesting_time: u64,
    merkle_root: String,
    finalized: bool,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let token_addr = deps.api.addr_canonicalize(&token)?;
    if state.owner != sender_addr {
        return Err(ContractError::Unauthorized {});
    }

    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.token = token_addr.clone();
        state.start_time = start_time;
        state.end_deposit_time = end_deposit_time;
        state.end_withdraw_time = end_withdraw_time;
        state.offering_amount = offering_amount;
        state.vesting_initial = vesting_initial;
        state.vesting_time = vesting_time;
        state.merkle_root = merkle_root.clone();
        state.finalized = finalized;
        Ok(state)
    })?;

    Ok(Response::new().add_attributes(vec![
        ("action", "configure"),
        ("token", token.as_str()),
        ("start_time", start_time.to_string().as_str()),
        ("end_deposit_time", end_deposit_time.to_string().as_str()),
        ("end_withdraw_time", end_withdraw_time.to_string().as_str()),
        ("offering_amount", offering_amount.to_string().as_str()),
        ("vesting_initial", vesting_initial.to_string().as_str()),
        ("vesting_time", vesting_time.to_string().as_str()),
        ("merkle_root", merkle_root.as_str()),
        ("finalized", finalized.to_string().as_str()),
    ]))
}

pub fn deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    allocation: Uint128,
    proof: Vec<String>,
) -> Result<Response, ContractError> {
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let sender = deps.api.addr_humanize(&sender_addr)?;
    let state = STATE.load(deps.storage)?;
    if state.start_time == 0 {
        return Err(ContractError::NotConfigured {});
    }
    if env.block.time.seconds() < state.start_time {
        return Err(ContractError::DepositNotStarted {});
    }
    if state.end_deposit_time < env.block.time.seconds() {
        return Err(ContractError::DepositEnded {});
    }

    let user_input = sender.to_string() + "," + &allocation.to_string();
    merkle_verify(state.merkle_root, user_input, proof)?;

    let amount = info
        .funds
        .iter()
        .find(|c| c.denom == "uusd")
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);
    if amount.is_zero() {
        return Err(ContractError::NoZeroAmount {});
    }
    if info.funds.len() > 1 {
        return Err(ContractError::NoOtherDenoms {});
    }

    let mut is_new_user = false;
    USERS_STATE.update(
        deps.storage,
        &sender,
        |maybe_user_state| -> Result<_, ContractError> {
            is_new_user = maybe_user_state.is_none();
            let mut user_state = maybe_user_state.unwrap_or_default();
            // if user_state.amount + amount > allocation {
            //     return Err(ContractError::OverAllocation {});
            // }
            user_state.amount += amount;
            Ok(user_state)
        },
    )?;

    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.total_amount += amount;
        if is_new_user {
            state.total_users += 1;
        }
        Ok(state)
    })?;

    Ok(Response::new().add_attributes(vec![
        ("action", "deposit"),
        ("user", sender.to_string().as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}

pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let sender = deps.api.addr_humanize(&sender_addr)?;
    let state = STATE.load(deps.storage)?;
    if env.block.time.seconds() < state.end_deposit_time {
        return Err(ContractError::WithdrawNotStarted {});
    }
    if state.end_withdraw_time < env.block.time.seconds() {
        return Err(ContractError::WithdrawEnded {});
    }

    let mut amount = Uint128::zero();
    USERS_STATE.update(
        deps.storage,
        &sender,
        |maybe_user_state| -> Result<_, ContractError> {
            let mut user_state = maybe_user_state.unwrap();
            amount = user_state.amount;
            user_state.amount = Uint128::zero();
            Ok(user_state)
        },
    )?;

    let amount_after_tax = deduct_tax(
        deps,
        Coin {
            denom: String::from("uusd"),
            amount: amount,
        },
    )?;
    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.to_string(),
            amount: vec![amount_after_tax.clone()],
        }))
        .add_attributes(vec![
            ("action", "withdraw"),
            ("user", sender.to_string().as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}

pub fn harvest(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let sender = deps.api.addr_humanize(&sender_addr)?;
    let state = STATE.load(deps.storage)?;
    if !state.finalized {
        return Err(ContractError::NotFinalized {});
    }

    let mut amount = Uint128::zero();
    USERS_STATE.update(
        deps.storage,
        &sender,
        |maybe_user_state| -> Result<_, ContractError> {
            let mut user_state = maybe_user_state.unwrap();
            let (_, claimable) = user_vesting(&state, &user_state, env.block.time.seconds());
            amount = claimable.saturating_sub(user_state.claimed);
            if amount.is_zero() {
                return Err(ContractError::NoZeroAmount {});
            }
            user_state.claimed += amount;
            Ok(user_state)
        },
    )?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&state.token)?.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender.to_string(),
                amount: amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "harvest"),
            ("user", sender.to_string().as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}

pub fn collect(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let sender = deps.api.addr_humanize(&sender_addr)?;
    if state.owner != sender_addr {
        return Err(ContractError::Unauthorized {});
    }

    let balance = deps
        .querier
        .query_balance(env.contract.address, "uusd")
        .unwrap();
    let balance_after_tax = deduct_tax(deps, balance)?;
    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.to_string(),
            amount: vec![balance_after_tax.clone()],
        }))
        .add_attributes(vec![
            ("action", "collect"),
            ("user", sender.to_string().as_str()),
            ("amount", balance_after_tax.amount.to_string().as_str()),
        ]))
}

pub fn collect_tokens(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    let sender_addr = deps.api.addr_canonicalize(info.sender.as_str())?;
    let sender = deps.api.addr_humanize(&sender_addr)?;
    if state.owner != sender_addr {
        return Err(ContractError::Unauthorized {});
    }

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
            ("action", "collect_tokens"),
            ("user", sender.to_string().as_str()),
            ("amount", amount.to_string().as_str()),
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
        QueryMsg::UserState { user, now } => to_binary(&query_user_state(deps, user, now)?),
    }
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        owner: deps.api.addr_humanize(&state.owner)?.to_string(),
        token: deps.api.addr_humanize(&state.token)?.to_string(),
        start_time: state.start_time,
        end_deposit_time: state.end_deposit_time,
        end_withdraw_time: state.end_withdraw_time,
        offering_amount: state.offering_amount,
        vesting_initial: state.vesting_initial,
        vesting_time: state.vesting_time,
        merkle_root: state.merkle_root,
        finalized: state.finalized,
        total_users: state.total_users,
        total_amount: state.total_amount,
    })
}

fn query_user_state(deps: Deps, user: String, now: u64) -> StdResult<UserStateResponse> {
    let user_addr = deps.api.addr_canonicalize(&user)?;
    let user = deps.api.addr_humanize(&user_addr)?;
    let state = STATE.load(deps.storage)?;
    let user_state = USERS_STATE
        .may_load(deps.storage, &user)?
        .unwrap_or_default();
    let (owed, claimable) = user_vesting(&state, &user_state, now);
    Ok(UserStateResponse {
        amount: user_state.amount,
        claimed: user_state.claimed,
        owed: owed,
        claimable: claimable,
    })
}

// HELPERS
// -----------------------------------------------------

const ONE: u128 = 1000000_u128;

fn one() -> Uint128 {
    Uint128::from(1000000_u128)
}

fn user_vesting(state: &State, user_state: &UserState, now: u64) -> (Uint128, Uint128) {
    let owed = user_state
        .amount
        .multiply_ratio(state.offering_amount, state.total_amount);
    let vesting_progress = now
        .saturating_sub(state.end_withdraw_time)
        .min(state.vesting_time);
    let claimable = owed.multiply_ratio(state.vesting_initial, one())
        + owed
            .multiply_ratio(one().saturating_sub(state.vesting_initial), one())
            .multiply_ratio(vesting_progress, state.vesting_time);
    return (owed, claimable);
}

fn merkle_verify(
    merkle_root: String,
    user_input: String,
    proof: Vec<String>,
) -> Result<(), ContractError> {
    let mut hash: [u8; 32] = sha3::Keccak256::digest(user_input.as_bytes())
        .as_slice()
        .try_into()
        .expect("Wrong length");
    for p in proof {
        let mut proof_buf: [u8; 32] = [0; 32];
        match hex::decode_to_slice(p, &mut proof_buf) {
            Ok(()) => {}
            _ => return Err(ContractError::InvalidMerkleProof {}),
        }

        hash = if bytes_cmp(hash, proof_buf) == std::cmp::Ordering::Less {
            sha3::Keccak256::digest(&[hash, proof_buf].concat())
                .as_slice()
                .try_into()
                .expect("Wrong length")
        } else {
            sha3::Keccak256::digest(&[proof_buf, hash].concat())
                .as_slice()
                .try_into()
                .expect("Wrong length")
        };
    }

    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(merkle_root, &mut root_buf).unwrap();
    if root_buf != hash {
        return Err(ContractError::InvalidMerkleProof {});
    }
    Ok(())
}

fn bytes_cmp(a: [u8; 32], b: [u8; 32]) -> std::cmp::Ordering {
    let mut i = 0;
    while i < 32 {
        match a[i].cmp(&b[i]) {
            std::cmp::Ordering::Greater => return std::cmp::Ordering::Greater,
            std::cmp::Ordering::Less => return std::cmp::Ordering::Less,
            _ => i += 1,
        }
    }
    std::cmp::Ordering::Equal
}

static DECIMAL_FRACTION: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

pub fn compute_tax(deps: DepsMut, coin: &Coin) -> StdResult<Uint128> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate: Decimal = (terra_querier.query_tax_rate()?).rate;
    let tax_cap: Uint128 = (terra_querier.query_tax_cap(coin.denom.to_string())?).cap;
    Ok(std::cmp::min(
        coin.amount.checked_sub(coin.amount.multiply_ratio(
            DECIMAL_FRACTION,
            DECIMAL_FRACTION * tax_rate + DECIMAL_FRACTION,
        ))?,
        tax_cap,
    ))
}

pub fn deduct_tax(deps: DepsMut, coin: Coin) -> StdResult<Coin> {
    let tax_amount = compute_tax(deps, &coin)?;
    Ok(Coin {
        denom: coin.denom,
        amount: coin.amount.checked_sub(tax_amount)?,
    })
}
