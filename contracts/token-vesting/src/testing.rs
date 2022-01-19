use crate::contract::{execute, instantiate, query};
use crate::msg::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, VestingAccountResponse, VestingData,
    VestingSchedule,
};

use cosmwasm_std::{
    from_binary,
    testing::{mock_dependencies, mock_env, mock_info},
    to_binary, Addr, Attribute, BankMsg, Coin, Response, StdError, SubMsg, Timestamp, Uint128,
    WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, Denom};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {};

    let info = mock_info("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
}

#[test]
fn register_vesting_account_with_native_token() {
    let mut deps = mock_dependencies(&[]);
    let _res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0000", &[]),
        InstantiateMsg {},
    )
    .unwrap();

    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // zero amount vesting token
    let msg = ExecuteMsg::RegisterVestingAccount {
        master_address: None,
        address: "addr0001".to_string(),
        vesting_schedule: VestingSchedule::LinearVesting {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_amount: Uint128::zero(),
        },
    };

    // invalid zero amount
    let info = mock_info("addr0000", &[Coin::new(0u128, "uusd")]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);
    match res.unwrap_err() {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "assert(vesting_amount > 0)")
        }
        _ => panic!("should not enter"),
    }

    // normal amount vesting token
    let msg = ExecuteMsg::RegisterVestingAccount {
        master_address: None,
        address: "addr0001".to_string(),
        vesting_schedule: VestingSchedule::LinearVesting {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_amount: Uint128::new(1000000u128),
        },
    };

    // invalid amount
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res.unwrap_err() {
        StdError::GenericErr { msg, .. } => assert_eq!(msg, "must deposit only one type of token"),
        _ => panic!("should not enter"),
    }

    // invalid amount
    let info = mock_info(
        "addr0000",
        &[Coin::new(100u128, "uusd"), Coin::new(10u128, "ukrw")],
    );
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res.unwrap_err() {
        StdError::GenericErr { msg, .. } => assert_eq!(msg, "must deposit only one type of token"),
        _ => panic!("should not enter"),
    }

    // invalid amount
    let info = mock_info("addr0000", &[Coin::new(10u128, "uusd")]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone());
    match res.unwrap_err() {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "assert(deposit_amount == vesting_amount)")
        }
        _ => panic!("should not enter"),
    }

    // valid amount
    let info = mock_info("addr0000", &[Coin::new(1000000u128, "uusd")]);
    let res: Response = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            ("action", "register_vesting_account"),
            ("master_address", "",),
            ("address", "addr0001"),
            ("vesting_denom", "{\"native\":\"uusd\"}"),
            ("vesting_amount", "1000000"),
        ]
    );

    // query vesting account
    assert_eq!(
        from_binary::<VestingAccountResponse>(
            &query(
                deps.as_ref(),
                env,
                QueryMsg::VestingAccount {
                    address: "addr0001".to_string(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountResponse {
            address: "addr0001".to_string(),
            vestings: vec![VestingData {
                master_address: None,
                vesting_denom: Denom::Native("uusd".to_string()),
                vesting_amount: Uint128::new(1000000),
                vested_amount: Uint128::zero(),
                vesting_schedule: VestingSchedule::LinearVesting {
                    start_time: "100".to_string(),
                    end_time: "110".to_string(),
                    vesting_amount: Uint128::new(1000000u128),
                },
                claimable_amount: Uint128::zero(),
            }],
        }
    );
}

#[test]
fn register_vesting_account_with_cw20_token() {
    let mut deps = mock_dependencies(&[]);
    let _res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0000", &[]),
        InstantiateMsg {},
    )
    .unwrap();
    let info = mock_info("token0000", &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // zero amount vesting token
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::new(1000000u128),
        msg: to_binary(&Cw20HookMsg::RegisterVestingAccount {
            master_address: None,
            address: "addr0001".to_string(),
            vesting_schedule: VestingSchedule::LinearVesting {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_amount: Uint128::zero(),
            },
        })
        .unwrap(),
    });

    // invalid zero amount
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    match res.unwrap_err() {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "assert(vesting_amount > 0)")
        }
        _ => panic!("should not enter"),
    }

    // invariant amount
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::new(1000000u128),
        msg: to_binary(&Cw20HookMsg::RegisterVestingAccount {
            master_address: None,
            address: "addr0001".to_string(),
            vesting_schedule: VestingSchedule::LinearVesting {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_amount: Uint128::new(999000u128),
            },
        })
        .unwrap(),
    });

    // invalid amount
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    match res.unwrap_err() {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "assert(deposit_amount == vesting_amount)")
        }
        _ => panic!("should not enter"),
    }

    // valid amount
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::new(1000000u128),
        msg: to_binary(&Cw20HookMsg::RegisterVestingAccount {
            master_address: None,
            address: "addr0001".to_string(),
            vesting_schedule: VestingSchedule::LinearVesting {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_amount: Uint128::new(1000000u128),
            },
        })
        .unwrap(),
    });

    // valid amount
    let res: Response = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            ("action", "register_vesting_account"),
            ("master_address", "",),
            ("address", "addr0001"),
            ("vesting_denom", "{\"cw20\":\"token0000\"}"),
            ("vesting_amount", "1000000"),
        ]
    );

    // query vesting account
    assert_eq!(
        from_binary::<VestingAccountResponse>(
            &query(
                deps.as_ref(),
                env,
                QueryMsg::VestingAccount {
                    address: "addr0001".to_string(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountResponse {
            address: "addr0001".to_string(),
            vestings: vec![VestingData {
                master_address: None,
                vesting_denom: Denom::Cw20(Addr::unchecked("token0000")),
                vesting_amount: Uint128::new(1000000),
                vested_amount: Uint128::zero(),
                vesting_schedule: VestingSchedule::LinearVesting {
                    start_time: "100".to_string(),
                    end_time: "110".to_string(),
                    vesting_amount: Uint128::new(1000000u128),
                },
                claimable_amount: Uint128::zero(),
            }],
        }
    );
}

#[test]
fn claim_native() {
    let mut deps = mock_dependencies(&[]);
    let _res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0000", &[]),
        InstantiateMsg {},
    )
    .unwrap();

    // init env to time 100
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // valid amount
    let msg = ExecuteMsg::RegisterVestingAccount {
        master_address: None,
        address: "addr0001".to_string(),
        vesting_schedule: VestingSchedule::LinearVesting {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_amount: Uint128::new(1000000u128),
        },
    };

    let info = mock_info("addr0000", &[Coin::new(1000000u128, "uusd")]);
    let _ = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // make time to half claimable
    env.block.time = Timestamp::from_seconds(105);

    // claim not found denom
    let msg = ExecuteMsg::Claim {
        denoms: vec![
            Denom::Native("ukrw".to_string()),
            Denom::Native("uusd".to_string()),
        ],
        recipient: None,
    };

    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    match res.unwrap_err() {
        StdError::GenericErr { msg, .. } => assert_eq!(
            msg,
            "vesting entry is not found for denom {\"native\":\"ukrw\"}"
        ),
        _ => panic!("should not enter"),
    }

    // valid claim
    let msg = ExecuteMsg::Claim {
        denoms: vec![Denom::Native("uusd".to_string())],
        recipient: None,
    };

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(BankMsg::Send {
            to_address: "addr0001".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(500000u128),
            }],
        }),]
    );
    assert_eq!(
        res.attributes,
        vec![
            Attribute::new("action", "claim"),
            Attribute::new("address", "addr0001"),
            Attribute::new("vesting_denom", "{\"native\":\"uusd\"}"),
            Attribute::new("vesting_amount", "1000000"),
            Attribute::new("vested_amount", "500000"),
            Attribute::new("claim_amount", "500000"),
        ],
    );

    // query vesting account
    assert_eq!(
        from_binary::<VestingAccountResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::VestingAccount {
                    address: "addr0001".to_string(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountResponse {
            address: "addr0001".to_string(),
            vestings: vec![VestingData {
                master_address: None,
                vesting_denom: Denom::Native("uusd".to_string()),
                vesting_amount: Uint128::new(1000000),
                vested_amount: Uint128::new(500000),
                vesting_schedule: VestingSchedule::LinearVesting {
                    start_time: "100".to_string(),
                    end_time: "110".to_string(),
                    vesting_amount: Uint128::new(1000000u128),
                },
                claimable_amount: Uint128::zero(),
            }],
        }
    );

    // make time to half claimable
    env.block.time = Timestamp::from_seconds(110);

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(BankMsg::Send {
            to_address: "addr0001".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::new(500000u128),
            }],
        }),]
    );
    assert_eq!(
        res.attributes,
        vec![
            Attribute::new("action", "claim"),
            Attribute::new("address", "addr0001"),
            Attribute::new("vesting_denom", "{\"native\":\"uusd\"}"),
            Attribute::new("vesting_amount", "1000000"),
            Attribute::new("vested_amount", "1000000"),
            Attribute::new("claim_amount", "500000"),
        ],
    );

    // query vesting account
    assert_eq!(
        from_binary::<VestingAccountResponse>(
            &query(
                deps.as_ref(),
                env,
                QueryMsg::VestingAccount {
                    address: "addr0001".to_string(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountResponse {
            address: "addr0001".to_string(),
            vestings: vec![],
        }
    );
}

#[test]
fn claim_cw20() {
    let mut deps = mock_dependencies(&[]);
    let _res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0000", &[]),
        InstantiateMsg {},
    )
    .unwrap();

    // init env to time 100
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // valid amount
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::new(1000000u128),
        msg: to_binary(&Cw20HookMsg::RegisterVestingAccount {
            master_address: None,
            address: "addr0001".to_string(),
            vesting_schedule: VestingSchedule::LinearVesting {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_amount: Uint128::new(1000000u128),
            },
        })
        .unwrap(),
    });

    // valid amount
    let info = mock_info("token0001", &[]);
    let _ = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // make time to half claimable
    env.block.time = Timestamp::from_seconds(105);

    // claim not found denom
    let msg = ExecuteMsg::Claim {
        denoms: vec![
            Denom::Cw20(Addr::unchecked("token0002")),
            Denom::Cw20(Addr::unchecked("token0001")),
        ],
        recipient: None,
    };

    let info = mock_info("addr0001", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);
    match res.unwrap_err() {
        StdError::GenericErr { msg, .. } => assert_eq!(
            msg,
            "vesting entry is not found for denom {\"cw20\":\"token0002\"}"
        ),
        _ => panic!("should not enter"),
    }

    // valid claim
    let msg = ExecuteMsg::Claim {
        denoms: vec![Denom::Cw20(Addr::unchecked("token0001"))],
        recipient: None,
    };

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: "token0001".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0001".to_string(),
                amount: Uint128::new(500000u128),
            })
            .unwrap(),
        }),]
    );
    assert_eq!(
        res.attributes,
        vec![
            Attribute::new("action", "claim"),
            Attribute::new("address", "addr0001"),
            Attribute::new("vesting_denom", "{\"cw20\":\"token0001\"}"),
            Attribute::new("vesting_amount", "1000000"),
            Attribute::new("vested_amount", "500000"),
            Attribute::new("claim_amount", "500000"),
        ],
    );

    // query vesting account
    assert_eq!(
        from_binary::<VestingAccountResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::VestingAccount {
                    address: "addr0001".to_string(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountResponse {
            address: "addr0001".to_string(),
            vestings: vec![VestingData {
                master_address: None,
                vesting_denom: Denom::Cw20(Addr::unchecked("token0001")),
                vesting_amount: Uint128::new(1000000),
                vested_amount: Uint128::new(500000),
                vesting_schedule: VestingSchedule::LinearVesting {
                    start_time: "100".to_string(),
                    end_time: "110".to_string(),
                    vesting_amount: Uint128::new(1000000u128),
                },
                claimable_amount: Uint128::zero(),
            }],
        }
    );

    // make time to half claimable
    env.block.time = Timestamp::from_seconds(110);

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(WasmMsg::Execute {
            contract_addr: "token0001".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0001".to_string(),
                amount: Uint128::new(500000u128),
            })
            .unwrap(),
        }),]
    );
    assert_eq!(
        res.attributes,
        vec![
            Attribute::new("action", "claim"),
            Attribute::new("address", "addr0001"),
            Attribute::new("vesting_denom", "{\"cw20\":\"token0001\"}"),
            Attribute::new("vesting_amount", "1000000"),
            Attribute::new("vested_amount", "1000000"),
            Attribute::new("claim_amount", "500000"),
        ],
    );

    // query vesting account
    assert_eq!(
        from_binary::<VestingAccountResponse>(
            &query(
                deps.as_ref(),
                env,
                QueryMsg::VestingAccount {
                    address: "addr0001".to_string(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountResponse {
            address: "addr0001".to_string(),
            vestings: vec![],
        }
    );
}

#[test]
fn query_vesting_account() {
    let mut deps = mock_dependencies(&[]);
    let _res = instantiate(
        deps.as_mut(),
        mock_env(),
        mock_info("addr0000", &[]),
        InstantiateMsg {},
    )
    .unwrap();

    // init env to time 100
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(100);

    // native vesting
    let msg = ExecuteMsg::RegisterVestingAccount {
        master_address: None,
        address: "addr0001".to_string(),
        vesting_schedule: VestingSchedule::LinearVesting {
            start_time: "100".to_string(),
            end_time: "110".to_string(),
            vesting_amount: Uint128::new(1000000u128),
        },
    };

    let info = mock_info("addr0000", &[Coin::new(1000000u128, "uusd")]);
    let _ = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::new(1000000u128),
        msg: to_binary(&Cw20HookMsg::RegisterVestingAccount {
            master_address: None,
            address: "addr0001".to_string(),
            vesting_schedule: VestingSchedule::LinearVesting {
                start_time: "100".to_string(),
                end_time: "110".to_string(),
                vesting_amount: Uint128::new(1000000u128),
            },
        })
        .unwrap(),
    });

    // valid amount
    let info = mock_info("token0001", &[]);
    let _ = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // half claimable
    env.block.time = Timestamp::from_seconds(105);

    // query all entry
    assert_eq!(
        from_binary::<VestingAccountResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::VestingAccount {
                    address: "addr0001".to_string(),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountResponse {
            address: "addr0001".to_string(),
            vestings: vec![
                VestingData {
                    master_address: None,
                    vesting_denom: Denom::Cw20(Addr::unchecked("token0001")),
                    vesting_amount: Uint128::new(1000000),
                    vested_amount: Uint128::new(500000),
                    vesting_schedule: VestingSchedule::LinearVesting {
                        start_time: "100".to_string(),
                        end_time: "110".to_string(),
                        vesting_amount: Uint128::new(1000000u128),
                    },
                    claimable_amount: Uint128::new(500000),
                },
                VestingData {
                    master_address: None,
                    vesting_denom: Denom::Native("uusd".to_string()),
                    vesting_amount: Uint128::new(1000000),
                    vested_amount: Uint128::new(500000),
                    vesting_schedule: VestingSchedule::LinearVesting {
                        start_time: "100".to_string(),
                        end_time: "110".to_string(),
                        vesting_amount: Uint128::new(1000000u128),
                    },
                    claimable_amount: Uint128::new(500000),
                }
            ],
        }
    );

    // query one entry
    assert_eq!(
        from_binary::<VestingAccountResponse>(
            &query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::VestingAccount {
                    address: "addr0001".to_string(),
                    start_after: None,
                    limit: Some(1),
                },
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountResponse {
            address: "addr0001".to_string(),
            vestings: vec![VestingData {
                master_address: None,
                vesting_denom: Denom::Cw20(Addr::unchecked("token0001")),
                vesting_amount: Uint128::new(1000000),
                vested_amount: Uint128::new(500000),
                vesting_schedule: VestingSchedule::LinearVesting {
                    start_time: "100".to_string(),
                    end_time: "110".to_string(),
                    vesting_amount: Uint128::new(1000000u128),
                },
                claimable_amount: Uint128::new(500000),
            },],
        }
    );

    // query one entry after first one
    assert_eq!(
        from_binary::<VestingAccountResponse>(
            &query(
                deps.as_ref(),
                env,
                QueryMsg::VestingAccount {
                    address: "addr0001".to_string(),
                    start_after: Some(Denom::Cw20(Addr::unchecked("token0001"))),
                    limit: Some(1),
                },
            )
            .unwrap()
        )
        .unwrap(),
        VestingAccountResponse {
            address: "addr0001".to_string(),
            vestings: vec![VestingData {
                master_address: None,
                vesting_denom: Denom::Native("uusd".to_string()),
                vesting_amount: Uint128::new(1000000),
                vested_amount: Uint128::new(500000),
                vesting_schedule: VestingSchedule::LinearVesting {
                    start_time: "100".to_string(),
                    end_time: "110".to_string(),
                    vesting_amount: Uint128::new(1000000u128),
                },
                claimable_amount: Uint128::new(500000),
            }],
        }
    );
}
