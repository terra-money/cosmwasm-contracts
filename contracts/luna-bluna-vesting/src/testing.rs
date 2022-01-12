use crate::contract::{execute, instantiate, query, reply};
use crate::external::handle::{HubContractExecuteMsg, RewardContractExecuteMsg};
use crate::mock_querier::mock_dependencies_with_querier;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VestingInfoResponse};
use crate::state::{StakingInfo, VestingSchedule};

use cosmwasm_std::{
    from_binary,
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    to_binary, Addr, Attribute, BankMsg, Coin, ContractResult, Decimal, Reply, Response, StdError,
    SubMsg, SubMsgExecutionResponse, Timestamp, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Denom};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies_with_querier(&[]);

    let msg = InstantiateMsg {
        owner_address: "owner0001".to_string(),
        enable_staking: false,
        staking_info: None,
        vesting_schedule: VestingSchedule {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_interval: "5".to_string(),
            vesting_ratio: Decimal::from_ratio(5u128, 10u128),
        },
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(1000000),
        }],
    );
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            ("action", "create_vesting_account"),
            ("owner_address", "owner0001"),
            ("vesting_denom", "{\"native\":\"uluna\"}",),
            ("vesting_amount", "1000000"),
        ]
    );
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn proper_initialization_enable_staking() {
    let mut deps = mock_dependencies_with_querier(&[]);

    let msg = InstantiateMsg {
        owner_address: "owner0001".to_string(),
        enable_staking: true,
        staking_info: Some(StakingInfo {
            bluna_token: "bluna".to_string(),
            hub_contract: "hub".to_string(),
            reward_contract: "reward".to_string(),
            validator: "validator".to_string(),
        }),
        vesting_schedule: VestingSchedule {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_interval: "5".to_string(),
            vesting_ratio: Decimal::from_ratio(5u128, 10u128),
        },
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(1000000),
        }],
    );
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: "hub".to_string(),
                msg: to_binary(&HubContractExecuteMsg::Bond {
                    validator: "validator".to_string(),
                })
                .unwrap(),
                funds: vec![Coin {
                    denom: "uluna".to_string(),
                    amount: Uint128::new(1000000u128)
                }],
            },
            1,
        )]
    );
    assert_eq!(res.attributes.len(), 0);
}

#[test]
fn invalid_start_time_initialization() {
    let mut deps = mock_dependencies_with_querier(&[]);

    let msg = InstantiateMsg {
        owner_address: "owner0001".to_string(),
        enable_staking: false,
        staking_info: None,
        vesting_schedule: VestingSchedule {
            start_time: "100".to_string(),
            end_time: "100".to_string(),
            vesting_interval: "5".to_string(),
            vesting_ratio: Decimal::from_ratio(5u128, 10u128),
        },
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(1000000),
        }],
    );

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
}

#[test]
fn invalid_end_time_initialization() {
    let mut deps = mock_dependencies_with_querier(&[]);

    let msg = InstantiateMsg {
        owner_address: "owner0001".to_string(),
        enable_staking: false,
        staking_info: None,
        vesting_schedule: VestingSchedule {
            start_time: "100".to_string(),
            end_time: "100".to_string(),
            vesting_interval: "5".to_string(),
            vesting_ratio: Decimal::from_ratio(5u128, 10u128),
        },
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(1000000),
        }],
    );

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
}

#[test]
fn invalid_initialization_enable_staking_without_staking_info() {
    let mut deps = mock_dependencies_with_querier(&[]);

    let msg = InstantiateMsg {
        owner_address: "owner0001".to_string(),
        enable_staking: true,
        staking_info: None,
        vesting_schedule: VestingSchedule {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_interval: "5".to_string(),
            vesting_ratio: Decimal::from_ratio(5u128, 10u128),
        },
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(1000000),
        }],
    );
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap_err();
}

#[test]
fn test_reply() {
    let mut deps = mock_dependencies_with_querier(&[]);
    let msg = InstantiateMsg {
        owner_address: "owner0001".to_string(),
        enable_staking: true,
        staking_info: Some(StakingInfo {
            bluna_token: "bluna".to_string(),
            hub_contract: "hub".to_string(),
            reward_contract: "reward".to_string(),
            validator: "validator".to_string(),
        }),
        vesting_schedule: VestingSchedule {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_interval: "5".to_string(),
            vesting_ratio: Decimal::from_ratio(5u128, 10u128),
        },
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(1000000),
        }],
    );
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let msg: Reply = Reply {
        id: 1,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: vec![],
            data: None,
        }),
    };

    deps.querier
        .with_balances_querier(&[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::new(999999))]);
    let res: Response = reply(deps.as_mut(), env.clone(), msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(
        res.attributes,
        vec![
            ("action", "create_vesting_account"),
            ("owner_address", "owner0001"),
            ("vesting_denom", "{\"cw20\":\"bluna\"}",),
            ("vesting_amount", "999999"),
        ]
    );

    assert_eq!(
        from_binary::<VestingInfoResponse>(
            &query(deps.as_ref(), env, QueryMsg::VestingInfo {}).unwrap()
        )
        .unwrap(),
        VestingInfoResponse {
            owner_address: "owner0001".to_string(),
            vesting_denom: Denom::Cw20(Addr::unchecked("bluna")),
            vesting_amount: Uint128::new(999999u128),
            vested_amount: Uint128::zero(),
            vesting_schedule: VestingSchedule {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_interval: "5".to_string(),
                vesting_ratio: Decimal::from_ratio(5u128, 10u128),
            },
            claimable_amount: Uint128::zero(),
            claimable_staking_rewards: Uint128::zero(),
        }
    );
}

#[test]
fn test_change_owner() {
    let mut deps = mock_dependencies_with_querier(&[]);

    let msg = InstantiateMsg {
        owner_address: "owner0001".to_string(),
        enable_staking: false,
        staking_info: None,
        vesting_schedule: VestingSchedule {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_interval: "5".to_string(),
            vesting_ratio: Decimal::from_ratio(5u128, 10u128),
        },
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(1000000),
        }],
    );
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // unauthorized
    let msg = ExecuteMsg::ChangeOwner {
        new_owner: "owner0002".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
    match res {
        StdError::GenericErr { msg, .. } => assert_eq!(msg, "unauthorized"),
        _ => panic!("DO NOT ENTER"),
    }

    let info = mock_info("owner0001", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_eq!(
        from_binary::<VestingInfoResponse>(
            &query(deps.as_ref(), env, QueryMsg::VestingInfo {}).unwrap()
        )
        .unwrap(),
        VestingInfoResponse {
            owner_address: "owner0002".to_string(),
            vesting_denom: Denom::Native("uluna".to_string()),
            vesting_amount: Uint128::new(1000000u128),
            vested_amount: Uint128::zero(),
            vesting_schedule: VestingSchedule {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_interval: "5".to_string(),
                vesting_ratio: Decimal::from_ratio(5u128, 10u128),
            },
            claimable_amount: Uint128::zero(),
            claimable_staking_rewards: Uint128::zero(),
        }
    );
}

#[test]
fn claim_native() {
    let mut deps = mock_dependencies_with_querier(&[]);
    let msg = InstantiateMsg {
        owner_address: "owner0001".to_string(),
        enable_staking: false,
        staking_info: None,
        vesting_schedule: VestingSchedule {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_interval: "5".to_string(),
            vesting_ratio: Decimal::from_ratio(5u128, 10u128),
        },
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(1000000),
        }],
    );
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // make time to half claimable
    env.block.time = Timestamp::from_seconds(105);

    // valid claim
    let msg = ExecuteMsg::Claim {
        recipient: Some("addr0001".to_string()),
    };

    // permission check
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap_err();
    match res {
        StdError::GenericErr { msg, .. } => assert_eq!(msg, "unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("owner0001", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(BankMsg::Send {
            to_address: "addr0001".to_string(),
            amount: vec![Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(500000u128),
            }],
        }),]
    );
    assert_eq!(
        res.attributes,
        vec![
            Attribute::new("action", "claim"),
            Attribute::new("recipient", "addr0001"),
            Attribute::new("vesting_denom", "{\"native\":\"uluna\"}"),
            Attribute::new("vesting_amount", "1000000"),
            Attribute::new("vested_amount", "500000"),
            Attribute::new("claim_amount", "500000"),
        ],
    );

    // query vesting account
    assert_eq!(
        from_binary::<VestingInfoResponse>(
            &query(deps.as_ref(), env.clone(), QueryMsg::VestingInfo {},).unwrap()
        )
        .unwrap(),
        VestingInfoResponse {
            owner_address: "owner0001".to_string(),
            vesting_denom: Denom::Native("uluna".to_string()),
            vesting_amount: Uint128::new(1000000),
            vested_amount: Uint128::new(500000),
            vesting_schedule: VestingSchedule {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_interval: "5".to_string(),
                vesting_ratio: Decimal::from_ratio(5u128, 10u128),
            },
            claimable_amount: Uint128::zero(),
            claimable_staking_rewards: Uint128::zero(),
        }
    );

    // make time to half claimable
    env.block.time = Timestamp::from_seconds(110);

    let msg = ExecuteMsg::Claim { recipient: None };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(BankMsg::Send {
            to_address: "owner0001".to_string(),
            amount: vec![Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(500000u128),
            }],
        }),]
    );
    assert_eq!(
        res.attributes,
        vec![
            Attribute::new("action", "claim"),
            Attribute::new("recipient", "owner0001"),
            Attribute::new("vesting_denom", "{\"native\":\"uluna\"}"),
            Attribute::new("vesting_amount", "1000000"),
            Attribute::new("vested_amount", "1000000"),
            Attribute::new("claim_amount", "500000"),
        ],
    );

    // query vesting account
    assert_eq!(
        from_binary::<VestingInfoResponse>(
            &query(deps.as_ref(), env, QueryMsg::VestingInfo {},).unwrap()
        )
        .unwrap(),
        VestingInfoResponse {
            owner_address: "owner0001".to_string(),
            vesting_denom: Denom::Native("uluna".to_string()),
            vesting_amount: Uint128::new(1000000),
            vested_amount: Uint128::new(1000000),
            vesting_schedule: VestingSchedule {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_interval: "5".to_string(),
                vesting_ratio: Decimal::from_ratio(5u128, 10u128),
            },
            claimable_amount: Uint128::zero(),
            claimable_staking_rewards: Uint128::zero(),
        }
    );
}

#[test]
fn claim_cw20() {
    let mut deps = mock_dependencies_with_querier(&[]);

    let msg = InstantiateMsg {
        owner_address: "owner0001".to_string(),
        enable_staking: true,
        staking_info: Some(StakingInfo {
            bluna_token: "bluna".to_string(),
            hub_contract: "hub".to_string(),
            reward_contract: "reward".to_string(),
            validator: "validator".to_string(),
        }),
        vesting_schedule: VestingSchedule {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_interval: "5".to_string(),
            vesting_ratio: Decimal::from_ratio(5u128, 10u128),
        },
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(1000000),
        }],
    );
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    let msg: Reply = Reply {
        id: 1,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: vec![],
            data: None,
        }),
    };

    deps.querier
        .with_balances_querier(&[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::new(999999))]);
    let _res: Response = reply(deps.as_mut(), env.clone(), msg).unwrap();
    // make time to half claimable
    env.block.time = Timestamp::from_seconds(105);

    // valid claim
    let msg = ExecuteMsg::Claim {
        recipient: Some("addr0001".to_string()),
    };

    // permission check
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap_err();
    match res {
        StdError::GenericErr { msg, .. } => assert_eq!(msg, "unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("owner0001", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: "bluna".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0001".to_string(),
                amount: Uint128::new(499999u128),
            })
            .unwrap(),
        }),]
    );
    assert_eq!(
        res.attributes,
        vec![
            Attribute::new("action", "claim"),
            Attribute::new("recipient", "addr0001"),
            Attribute::new("vesting_denom", "{\"cw20\":\"bluna\"}"),
            Attribute::new("vesting_amount", "999999"),
            Attribute::new("vested_amount", "499999"),
            Attribute::new("claim_amount", "499999"),
        ],
    );

    // query vesting account
    assert_eq!(
        from_binary::<VestingInfoResponse>(
            &query(deps.as_ref(), env.clone(), QueryMsg::VestingInfo {},).unwrap()
        )
        .unwrap(),
        VestingInfoResponse {
            owner_address: "owner0001".to_string(),
            vesting_denom: Denom::Cw20(Addr::unchecked("bluna")),
            vesting_amount: Uint128::new(999999),
            vested_amount: Uint128::new(499999),
            vesting_schedule: VestingSchedule {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_interval: "5".to_string(),
                vesting_ratio: Decimal::from_ratio(5u128, 10u128),
            },
            claimable_amount: Uint128::zero(),
            claimable_staking_rewards: Uint128::zero(),
        }
    );

    // make time to half claimable
    env.block.time = Timestamp::from_seconds(110);

    let msg = ExecuteMsg::Claim { recipient: None };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: "bluna".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "owner0001".to_string(),
                amount: Uint128::new(500000u128),
            })
            .unwrap(),
        }),]
    );
    assert_eq!(
        res.attributes,
        vec![
            Attribute::new("action", "claim"),
            Attribute::new("recipient", "owner0001"),
            Attribute::new("vesting_denom", "{\"cw20\":\"bluna\"}"),
            Attribute::new("vesting_amount", "999999"),
            Attribute::new("vested_amount", "999999"),
            Attribute::new("claim_amount", "500000"),
        ],
    );

    // query vesting account
    assert_eq!(
        from_binary::<VestingInfoResponse>(
            &query(deps.as_ref(), env, QueryMsg::VestingInfo {},).unwrap()
        )
        .unwrap(),
        VestingInfoResponse {
            owner_address: "owner0001".to_string(),
            vesting_denom: Denom::Cw20(Addr::unchecked("bluna".to_string())),
            vesting_amount: Uint128::new(999999),
            vested_amount: Uint128::new(999999),
            vesting_schedule: VestingSchedule {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_interval: "5".to_string(),
                vesting_ratio: Decimal::from_ratio(5u128, 10u128),
            },
            claimable_amount: Uint128::zero(),
            claimable_staking_rewards: Uint128::zero(),
        }
    );
}

#[test]
fn claim_rewards() {
    let mut deps = mock_dependencies_with_querier(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::new(300u128),
    }]);

    let msg = InstantiateMsg {
        owner_address: "owner0001".to_string(),
        enable_staking: true,
        staking_info: Some(StakingInfo {
            bluna_token: "bluna".to_string(),
            hub_contract: "hub".to_string(),
            reward_contract: "reward".to_string(),
            validator: "validator".to_string(),
        }),
        vesting_schedule: VestingSchedule {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_interval: "5".to_string(),
            vesting_ratio: Decimal::from_ratio(5u128, 10u128),
        },
    };

    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(1000000),
        }],
    );
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    let msg: Reply = Reply {
        id: 1,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: vec![],
            data: None,
        }),
    };

    deps.querier
        .with_balances_querier(&[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::new(999999))]);
    let _res: Response = reply(deps.as_mut(), env.clone(), msg).unwrap();
    // make time to half claimable
    env.block.time = Timestamp::from_seconds(105);

    // register rewards
    deps.querier
        .with_rewards_querier(&[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::new(500u128))]);

    // query vesting account
    assert_eq!(
        from_binary::<VestingInfoResponse>(
            &query(deps.as_ref(), env.clone(), QueryMsg::VestingInfo {},).unwrap()
        )
        .unwrap(),
        VestingInfoResponse {
            owner_address: "owner0001".to_string(),
            vesting_denom: Denom::Cw20(Addr::unchecked("bluna")),
            vesting_amount: Uint128::new(999999),
            vested_amount: Uint128::new(499999),
            vesting_schedule: VestingSchedule {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_interval: "5".to_string(),
                vesting_ratio: Decimal::from_ratio(5u128, 10u128),
            },
            claimable_amount: Uint128::new(499999),
            claimable_staking_rewards: Uint128::new(300 + 500),
        }
    );

    // valid claim
    let msg = ExecuteMsg::ClaimRewards {
        recipient: Some("addr0001".to_string()),
    };

    // permission check
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap_err();
    match res {
        StdError::GenericErr { msg, .. } => assert_eq!(msg, "unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let info = mock_info("owner0001", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(BankMsg::Send {
                to_address: "addr0001".to_string(),
                amount: vec![Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::new(300u128),
                }],
            }),
            SubMsg::new(WasmMsg::Execute {
                contract_addr: "reward".to_string(),
                funds: vec![],
                msg: to_binary(&RewardContractExecuteMsg::ClaimRewards {
                    recipient: Some("addr0001".to_string()),
                })
                .unwrap(),
            }),
        ]
    );
    assert_eq!(res.attributes, vec![("action", "claim_rewards"),],);
}
