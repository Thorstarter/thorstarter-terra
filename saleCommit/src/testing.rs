use crate::*;
use cosmwasm_std::testing::{
    mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    attr, from_binary, from_slice, Coin, ContractResult as StdContractResult, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SubMsg, SystemError, SystemResult, Timestamp, WasmQuery,
};
use cw20::{BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use std::collections::HashMap;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

const ONE: u128 = 1000000u128;
const ALLOCATION: u128 = 75 * ONE;
const MERKLE_ROOT: &str = "8e70ddd3cba3e4db4073ef0a775c71f601e2b2d5c517ed9718e7d4dd7a2c71a4";
fn test_merkle_proof() -> Vec<String> {
    [
        "90d9e60cde3d83e12292e7535173435d9e3162313b670e36d6e07f88b33a82da".to_string(),
        "d8e28cfabf40072adc132ddeb2c34be910d9a563530fde22830ee9610fe693b7".to_string(),
    ]
    .into()
}

fn test_setup(deposit: bool) -> OwnedDeps<MockStorage, MockApi, CustomMockQuerier> {
    let mut deps = mock_dependencies(&[Coin::new(80 * ONE, "uusd")]);
    deps.querier.infos.insert(
        "token0000".to_string(),
        TokenInfoResponse {
            name: "Mock Token".to_string(),
            symbol: "MTOK".to_string(),
            decimals: 18,
            total_supply: Uint128::from(100u128),
        },
    );
    let mut token_balances = HashMap::new();
    token_balances.insert("addr0000".to_string(), Uint128::from(100u128));
    deps.querier
        .balances
        .insert("token0000".to_string(), token_balances);
    {
        let msg = InstantiateMsg {};
        let info = mock_info("addr0000", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
    {
        let info = mock_info("addr0000", &[]);
        let msg = ExecuteMsg::Configure {
            token: "token0000".to_string(),
            start_time: 10,
            end_deposit_time: 100,
            end_withdraw_time: 200,
            min_price: Uint128::from(0 * ONE),
            offering_amount: Uint128::from(500 * ONE),
            vesting_initial: Uint128::from(100000_u128),
            vesting_time: 200,
            merkle_root: MERKLE_ROOT.to_string(),
            finalized: false,
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    if deposit {
        {
            let msg = ExecuteMsg::Deposit {
                allocation: Uint128::from(ALLOCATION),
                proof: test_merkle_proof(),
            };
            let info = mock_info("addr0001", &[Coin::new(50 * ONE, "uusd")]);
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(40);
            execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        }
        {
            let msg = ExecuteMsg::Configure {
                token: "token0000".to_string(),
                start_time: 10,
                end_deposit_time: 100,
                end_withdraw_time: 200,
                min_price: Uint128::from(0 * ONE),
                offering_amount: Uint128::from(500 * ONE),
                vesting_initial: Uint128::from(100000_u128),
                vesting_time: 200,
                merkle_root: MERKLE_ROOT.to_string(),
                finalized: true,
            };
            let info = mock_info("addr0000", &[]);
            execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        }
    }

    deps
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {};
    let info = mock_info("addr0000", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let value: StateResponse = from_binary(&res).unwrap();
    assert_eq!("addr0000", value.owner.to_string());
    assert_eq!("addr0000", value.token.to_string());
    assert_eq!(0, value.start_time);
    assert_eq!(0, value.end_deposit_time);
    assert_eq!(0, value.end_withdraw_time);
    assert_eq!("0", value.min_price.to_string());
    assert_eq!("0", value.offering_amount.to_string());
    assert_eq!(false, value.finalized);
    assert_eq!("0", value.total_users.to_string());
    assert_eq!("0", value.total_amount.to_string());
    assert_eq!("0", value.total_amount_high.to_string());
}

#[test]
fn test_configure_error_not_owner() {
    let mut deps = test_setup(true);
    let msg = ExecuteMsg::Configure {
        token: "token0000".to_string(),
        start_time: 10,
        end_deposit_time: 100,
        end_withdraw_time: 200,
        min_price: Uint128::from(2 * ONE),
        offering_amount: Uint128::from(500 * ONE),
        vesting_initial: Uint128::from(100000_u128),
        vesting_time: 200,
        merkle_root: MERKLE_ROOT.to_string(),
        finalized: false,
    };
    let info = mock_info("addr0001", &[]);
    let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);
}

#[test]
fn test_configure() {
    let mut deps = test_setup(true);
    let msg = ExecuteMsg::Configure {
        token: "token0000".to_string(),
        start_time: 10,
        end_deposit_time: 100,
        end_withdraw_time: 200,
        min_price: Uint128::from(2 * ONE),
        offering_amount: Uint128::from(500 * ONE),
        vesting_initial: Uint128::from(100000_u128),
        vesting_time: 200,
        merkle_root: MERKLE_ROOT.to_string(),
        finalized: true,
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let value: StateResponse = from_binary(&res).unwrap();
    assert_eq!("addr0000", value.owner.to_string());
    assert_eq!("token0000", value.token.to_string());
    assert_eq!(10, value.start_time);
    assert_eq!(100, value.end_deposit_time);
    assert_eq!(200, value.end_withdraw_time);
    assert_eq!("2000000", value.min_price.to_string());
    assert_eq!("500000000", value.offering_amount.to_string());
    assert_eq!(MERKLE_ROOT, value.merkle_root.to_string());
    assert_eq!(true, value.finalized);
}

#[test]
fn test_deposit_error_not_started() {
    let mut deps = test_setup(false);
    let msg = ExecuteMsg::Deposit {
        allocation: Uint128::from(ALLOCATION),
        proof: test_merkle_proof(),
    };
    let info = mock_info("addr0001", &[Coin::new(50 * ONE, "uusd")]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(5);
    let err = execute(deps.as_mut(), env, info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::DepositNotStarted {}, err);
}

#[test]
fn test_deposit_error_ended() {
    let mut deps = test_setup(false);
    let msg = ExecuteMsg::Deposit {
        allocation: Uint128::from(ALLOCATION),
        proof: test_merkle_proof(),
    };
    let info = mock_info("addr0001", &[Coin::new(50 * ONE, "uusd")]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(105);
    let err = execute(deps.as_mut(), env, info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::DepositEnded {}, err);
}

#[test]
fn test_deposit_error_no_zero() {
    let mut deps = test_setup(false);
    let msg = ExecuteMsg::Deposit {
        allocation: Uint128::from(ALLOCATION),
        proof: test_merkle_proof(),
    };
    let info = mock_info("addr0001", &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(20);
    let err = execute(deps.as_mut(), env, info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::NoZeroAmount {}, err);
}

#[test]
fn test_deposit_error_no_other_denoms() {
    let mut deps = test_setup(false);
    let msg = ExecuteMsg::Deposit {
        allocation: Uint128::from(ALLOCATION),
        proof: test_merkle_proof(),
    };
    let info = mock_info(
        "addr0001",
        &[Coin::new(50 * ONE, "uusd"), Coin::new(ONE, "ukrw")],
    );
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(20);
    let err = execute(deps.as_mut(), env, info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::NoOtherDenoms {}, err);
}

#[test]
fn test_deposit_error_invalid_merkle_proof() {
    let mut deps = test_setup(false);
    let msg = ExecuteMsg::Deposit {
        allocation: Uint128::from(9000 * ONE),
        proof: test_merkle_proof(),
    };
    let info = mock_info("addr0001", &[Coin::new(10 * ONE, "uusd")]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(20);
    let err = execute(deps.as_mut(), env, info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::InvalidMerkleProof {}, err);
}

/*
#[test]
fn test_deposit_error_over_allocation() {
    let mut deps = test_setup(false);
    let msg = ExecuteMsg::Deposit {
        allocation: Uint128::from(ALLOCATION),
        proof: test_merkle_proof(),
    };
    let info = mock_info("addr0001", &[Coin::new(ALLOCATION + 1, "uusd")]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(20);
    let err = execute(deps.as_mut(), env, info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::OverAllocation {}, err);
}
*/

#[test]
fn test_deposit() {
    let mut deps = test_setup(false);
    let msg = ExecuteMsg::Deposit {
        allocation: Uint128::from(ALLOCATION),
        proof: test_merkle_proof(),
    };
    let info = mock_info("addr0001", &[Coin::new(50 * ONE, "uusd")]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(40);
    let rese = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
    assert_eq!(
        rese.attributes,
        vec![
            attr("action", "deposit"),
            attr("user", "addr0001"),
            attr("amount", "50000000"),
        ]
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::UserState {
            user: "addr0001".to_string(),
            now: 150,
        },
    )
    .unwrap();
    let value: UserStateResponse = from_binary(&res).unwrap();
    assert_eq!("50000000", value.amount.to_string());
    assert_eq!("0", value.claimed.to_string());
    assert_eq!("500000000", value.owed.to_string());
    assert_eq!("50000000", value.claimable.to_string());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let value: StateResponse = from_binary(&res).unwrap();
    assert_eq!("1", value.total_users.to_string());
    assert_eq!("50000000", value.total_amount.to_string());

    {
        let msg = ExecuteMsg::Deposit {
            allocation: Uint128::from(ALLOCATION),
            proof: test_merkle_proof(),
        };
        let info = mock_info("addr0001", &[Coin::new(10 * ONE, "uusd")]);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(40);
        execute(deps.as_mut(), env, info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
        let value: StateResponse = from_binary(&res).unwrap();
        assert_eq!("1", value.total_users.to_string());
        assert_eq!("60000000", value.total_amount.to_string());
    }
}

#[test]
fn test_withdraw_error_not_started() {
    let mut deps = test_setup(true);
    let msg = ExecuteMsg::Withdraw {
        amount: Uint128::zero(),
    };
    let info = mock_info("addr0001", &[Coin::new(50 * ONE, "uusd")]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(99);
    let err = execute(deps.as_mut(), env, info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::WithdrawNotStarted {}, err);
}

#[test]
fn test_withdraw_error_ended() {
    let mut deps = test_setup(true);
    let msg = ExecuteMsg::Withdraw {
        amount: Uint128::zero(),
    };
    let info = mock_info("addr0001", &[Coin::new(50 * ONE, "uusd")]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(201);
    let err = execute(deps.as_mut(), env, info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::WithdrawEnded {}, err);
}

#[test]
fn test_withdraw_error_over_amount() {
    let mut deps = test_setup(true);
    let msg = ExecuteMsg::Withdraw {
        amount: Uint128::from(51 * ONE),
    };
    let info = mock_info("addr0001", &[Coin::new(50 * ONE, "uusd")]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(101);
    let err = execute(deps.as_mut(), env, info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::OverAmount {}, err);
}

#[test]
fn test_withdraw_error_over_progress() {
    let mut deps = test_setup(true);
    let msg = ExecuteMsg::Withdraw {
        amount: Uint128::from(26 * ONE),
    };
    let info = mock_info("addr0001", &[Coin::new(50 * ONE, "uusd")]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(150);
    let err = execute(deps.as_mut(), env, info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::OverAmount {}, err);
}

#[test]
fn test_withdraw() {
    let mut deps = test_setup(true);
    let msg = ExecuteMsg::Withdraw {
        amount: Uint128::from(8_u128),
    };
    let info = mock_info("addr0001", &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(101);
    let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "withdraw"),
            attr("user", "addr0001"),
            attr("amount", "8"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr0001".to_string(),
            amount: vec![Coin::new(7_u128, "uusd")],
        }))],
    );
}

#[test]
fn test_harvest_error_not_finalized() {
    let mut deps = test_setup(true);
    {
        let msg = ExecuteMsg::Configure {
            token: "token0000".to_string(),
            start_time: 10,
            end_deposit_time: 100,
            end_withdraw_time: 200,
            min_price: Uint128::zero(),
            offering_amount: Uint128::from(500 * ONE),
            vesting_initial: Uint128::from(100000_u128),
            vesting_time: 200,
            merkle_root: MERKLE_ROOT.to_string(),
            finalized: false,
        };
        let info = mock_info("addr0000", &[]);
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    }
    let msg = ExecuteMsg::Harvest {};
    let info = mock_info("addr0001", &[]);
    let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::NotFinalized {}, err);
}

#[test]
fn test_harvest_error_amount_is_zero() {
    let mut deps = test_setup(true);
    {
        // Harvest all there is to harvest so next claim has 0
        let msg = ExecuteMsg::Harvest {};
        let info = mock_info("addr0001", &[]);
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    }
    let msg = ExecuteMsg::Harvest {};
    let info = mock_info("addr0001", &[]);
    let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::NoZeroAmount {}, err);
}

#[test]
fn test_harvest() {
    let mut deps = test_setup(true);

    {
        // Deposit as other user
        let msg = ExecuteMsg::Deposit {
            allocation: Uint128::from(ALLOCATION),
            proof: test_merkle_proof(),
        };
        let info = mock_info("addr0002", &[Coin::new(350 * ONE, "uusd")]);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(40);
        execute(deps.as_mut(), env, info.clone(), msg).unwrap();
    }
    {
        // Set offering
        let msg = ExecuteMsg::Configure {
            token: "token0000".to_string(),
            start_time: 10,
            end_deposit_time: 100,
            end_withdraw_time: 200,
            min_price: Uint128::zero(),
            offering_amount: Uint128::from(15000 * ONE),
            vesting_initial: Uint128::from(1000000_u128), // 100%
            vesting_time: 1,
            merkle_root: MERKLE_ROOT.to_string(),
            finalized: true,
        };
        let info = mock_info("addr0000", &[]);
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    }

    let msg = ExecuteMsg::Harvest {};
    let info = mock_info("addr0001", &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(40);
    let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "harvest"),
            attr("user", "addr0001"),
            attr("amount", "1875046876"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "token0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0001".to_string(),
                amount: Uint128::from(1875046876_u128),
            })
            .unwrap(),
            funds: vec![],
        }))],
    );
}

#[test]
fn test_harvest_min_price() {
    let mut deps = test_setup(true);

    {
        // Set min price
        let msg = ExecuteMsg::Configure {
            token: "token0000".to_string(),
            start_time: 10,
            end_deposit_time: 100,
            end_withdraw_time: 200,
            min_price: Uint128::from(160 * ONE),
            offering_amount: Uint128::from(500 * ONE),
            vesting_initial: Uint128::from(100000_u128),
            vesting_time: 200,
            merkle_root: MERKLE_ROOT.to_string(),
            finalized: true,
        };
        let info = mock_info("addr0000", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    }

    let msg = ExecuteMsg::Harvest {};
    let info = mock_info("addr0001", &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(40);
    let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "harvest"),
            attr("user", "addr0001"),
            attr("amount", "31250"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "token0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0001".to_string(),
                amount: Uint128::from(31250_u128),
            })
            .unwrap(),
            funds: vec![],
        }))],
    );
}

#[test]
fn test_collect() {
    let mut deps = test_setup(true);
    let msg = ExecuteMsg::Collect {};
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "collect"),
            attr("user", "addr0000"),
            attr("amount", "79207920"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr0000".to_string(),
            amount: vec![Coin::new(79207920_u128, "uusd")],
        }))],
    );
}

pub struct CustomMockQuerier {
    pub base: MockQuerier<TerraQueryWrapper>,
    pub infos: HashMap<String, TokenInfoResponse>,
    pub balances: HashMap<String, HashMap<String, Uint128>>,
}

impl Querier for CustomMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {:?}", e),
                    request: bin_request.into(),
                })
            }
        };
        match &request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if &TerraRoute::Treasury == route {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: Decimal::percent(1),
                            };
                            SystemResult::Ok(StdContractResult::from(to_binary(&res)))
                        }
                        TerraQuery::TaxCap { denom: _ } => {
                            let res = TaxCapResponse {
                                cap: Uint128::from(1000000u128),
                            };
                            SystemResult::Ok(StdContractResult::from(to_binary(&res)))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg: bin_msg,
            }) => {
                if contract_addr.to_string().starts_with("token") {
                    let msg = from_binary(&bin_msg).unwrap();
                    match msg {
                        Cw20QueryMsg::Balance { address } => {
                            SystemResult::Ok(StdContractResult::Ok(
                                to_binary(&BalanceResponse {
                                    balance: match self.balances.get(contract_addr) {
                                        Some(token_balance_map) => {
                                            match token_balance_map.get(&address) {
                                                Some(amount) => *amount,
                                                None => Uint128::zero(),
                                            }
                                        }
                                        None => Uint128::zero(),
                                    },
                                })
                                .unwrap(),
                            ))
                        }
                        Cw20QueryMsg::TokenInfo {} => {
                            match self.infos.get(contract_addr.as_str()) {
                                Some(info) => SystemResult::Ok(StdContractResult::Ok(
                                    to_binary(info).unwrap(),
                                )),
                                None => SystemResult::Err(SystemError::UnsupportedRequest {
                                    kind: contract_addr.to_string(),
                                }),
                            }
                        }
                        _ => SystemResult::Err(SystemError::UnsupportedRequest {
                            kind: stringify!(msg).to_string(),
                        }),
                    }
                } else {
                    panic!("unreachable")
                }
            }
            _ => self.base.handle_query(&request),
        }
    }
}

pub fn mock_dependencies(balances: &[Coin]) -> OwnedDeps<MockStorage, MockApi, CustomMockQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: CustomMockQuerier {
            base: MockQuerier::new(&[(MOCK_CONTRACT_ADDR, balances)]),
            infos: HashMap::new(),
            balances: HashMap::new(),
        },
    }
}
