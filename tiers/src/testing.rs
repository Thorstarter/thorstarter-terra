use crate::*;
use cosmwasm_std::testing::{
    mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{
    attr, from_binary, from_slice, Coin, ContractResult as StdContractResult, OwnedDeps, Querier,
    QuerierResult, SubMsg, SystemError, SystemResult,
};
use cw20::{BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use std::collections::HashMap;
use std::marker::PhantomData;
use terra_cosmwasm::TerraQueryWrapper;

const SEVEN_DAYS: u64 = 7 * 24 * 60 * 60;

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
        //custom_query_type: PhantomData,
    }
}

fn test_setup() -> OwnedDeps<MockStorage, MockApi, CustomMockQuerier> {
    let mut deps = mock_dependencies(&[]);
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
    let msg = InstantiateMsg {
        token: "token0000".to_string(),
        locked_period: 604800,
    };
    let info = mock_info("addr0000", &[]);
    instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    deps
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        token: "token0000".to_string(),
        locked_period: SEVEN_DAYS,
    };
    let info = mock_info("addr0000", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let value: StateResponse = from_binary(&res).unwrap();
    assert_eq!("addr0000", value.owner.to_string());
    assert_eq!("token0000", value.token.to_string());
    assert_eq!(false, value.paused);
    assert_eq!(604800, value.locked_period);
    assert_eq!("0", value.total_balance.to_string());
}

#[test]
fn test_configure() {
    let mut deps = test_setup();
    let msg = ExecuteMsg::Configure {
        paused: true,
        locked_period: 123,
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "configure"),
            attr("paused", "true"),
            attr("locked_period", "123"),
        ]
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let value: StateResponse = from_binary(&res).unwrap();
    assert_eq!(true, value.paused);
    assert_eq!(123, value.locked_period);

    let msg = ExecuteMsg::Configure {
        paused: true,
        locked_period: 123,
    };
    let info = mock_info("addr0001", &[]);
    let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);
}

#[test]
fn test_transfer_ownership() {
    let mut deps = test_setup();
    let msg = ExecuteMsg::TransferOwnership {
        owner: Addr::unchecked("addr0007".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "transfer_ownership"),
            attr("owner", "addr0007"),
        ]
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let value: StateResponse = from_binary(&res).unwrap();
    assert_eq!("addr0007", value.owner);

    let msg = ExecuteMsg::TransferOwnership {
        owner: Addr::unchecked("addr0009".to_string()),
    };
    let info = mock_info("addr0000", &[]);
    let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err);
}

#[test]
fn test_bond() {
    let mut deps = test_setup();
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0001".to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let info = mock_info("token0000", &[]);
    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::UserState {
            user: "addr0001".to_string(),
        },
    )
    .unwrap();
    let value: UserStateResponse = from_binary(&res).unwrap();
    assert_eq!("100", value.balance.to_string());

    let res = query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap();
    let value: StateResponse = from_binary(&res).unwrap();
    assert_eq!("100", value.total_balance.to_string());

    deps.querier.balances.get_mut("token0000").unwrap().insert(
        mock_env().contract.address.to_string(),
        Uint128::from(150u128),
    );

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0001".to_string(),
        amount: Uint128::from(50u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let info = mock_info("token0000", &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "bond"),
            attr("user", "addr0001"),
            attr("tokens", "50"),
            attr("change", "33"),
            attr("balance", "133"),
        ]
    );
}

#[test]
fn test_unbond() {
    // Bond some amount
    let mut deps = test_setup();
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0001".to_string(),
        amount: Uint128::from(50u128),
        msg: to_binary(&Cw20HookMsg::Bond {}).unwrap(),
    });
    let info = mock_info("token0000", &[]);
    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    // Mock tokens in contract to equal bonded amount
    deps.querier.balances.get_mut("token0000").unwrap().insert(
        mock_env().contract.address.to_string(),
        Uint128::from(50u128),
    );

    // Error when not enough balance
    let msg = ExecuteMsg::Unbond {
        amount: Uint128::from(51u128),
    };
    let info = mock_info("addr0001", &[]);
    let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::UnbondBalanceTooLow {}, err);

    // Error when before 7 days
    let msg = ExecuteMsg::Unbond {
        amount: Uint128::from(25u128),
    };
    let info = mock_info("addr0001", &[]);
    let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::UnbondBefore7Days {}, err);

    let msg = ExecuteMsg::Unbond {
        amount: Uint128::from(20u128),
    };
    let mut env = mock_env();
    env.block.time = env.block.time.plus_seconds(SEVEN_DAYS);
    let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "unbond"),
            attr("user", "addr0001"),
            attr("change", "20"),
            attr("tokens", "20"),
            attr("balance", "30"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "token0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0001".to_string(),
                amount: Uint128::from(20u128),
            })
            .unwrap(),
            funds: vec![],
        }))],
    );

    // Test pausing
    let mut deps = test_setup();
    let msg = ExecuteMsg::Configure {
        paused: true,
        locked_period: 0,
    };
    let info = mock_info("addr0000", &[]);
    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::Unbond {
        amount: Uint128::from(25u128),
    };
    let info = mock_info("addr0001", &[]);
    let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
    assert_eq!(ContractError::Paused {}, err);
}
