use cosmwasm_std::{
    log, to_binary, BankMsg, BlockInfo, Coin, CosmosMsg, Decimal, Env, HandleResponse, HumanAddr,
    StdError, Uint128, WasmMsg,
};

use crate::contract::{handle, init, query_config, query_roll, query_staker};
use terra_mocks::mock_dependencies;

use crate::msg::{
    ConfigResponse, HandleMsg, InitMsg, RollResponse, StakerResponse, TokenHandleMsg,
};

use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let value: ConfigResponse = query_config(&deps).unwrap();
    assert_eq!("addr0000", value.owner.as_str());
    assert_eq!("staking0000", value.staking_token.as_str());
    assert_eq!(Uint128(100000000u128), value.roll_unit);
    assert_eq!(86400u64, value.deposit_period);
    assert_eq!("uusd", value.rewards_denom.as_str());
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    // update owner
    let env = mock_env("addr0000", &[]);
    let msg = HandleMsg::UpdateConfig {
        owner: Some(HumanAddr("addr0001".to_string())),
        deposit_period: None,
    };

    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let value = query_config(&deps).unwrap();
    assert_eq!("addr0001", value.owner.as_str());
    assert_eq!(86400u64, value.deposit_period);

    // update left items
    let env = mock_env("addr0001", &[]);
    let msg = HandleMsg::UpdateConfig {
        owner: None,
        deposit_period: Some(Uint128::from(100000u128)),
    };

    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let value = query_config(&deps).unwrap();
    assert_eq!("addr0001", value.owner.as_str());
    assert_eq!(100000u64, value.deposit_period);

    // Unauthorzied err
    let env = mock_env("addr0000", &[]);
    let msg = HandleMsg::UpdateConfig {
        owner: None,
        deposit_period: None,
    };

    let res = handle(&mut deps, env, msg);
    match res {
        Err(StdError::Unauthorized { .. }) => {}
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn deposit() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    // deposit will create 1 roll
    let msg = HandleMsg::Deposit {
        amount: Uint128::from(150000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 0);
    let res: HandleResponse = handle(&mut deps, env, msg).unwrap();
    let msg = res.messages.get(0).expect("no message");
    let balance_log = res.log.get(1).expect("no data");
    let number_of_rolls_log = res.log.get(2).expect("no data");

    assert_eq!(
        &log("balance", &Uint128(150000000u128).to_string()),
        balance_log,
    );
    assert_eq!(
        &log("number_of_rolls", &1u32.to_string()),
        number_of_rolls_log,
    );
    assert_eq!(
        &CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: HumanAddr("staking0000".to_string()),
            send: vec![],
            msg: to_binary(&TokenHandleMsg::TransferFrom {
                owner: HumanAddr("addr0000".to_string()),
                recipient: HumanAddr(MOCK_CONTRACT_ADDR.to_string()),
                amount: Uint128(150000000u128),
            })
            .unwrap(),
        }),
        msg
    );

    let roll = query_roll(&deps, HumanAddr("addr0000".to_string()), 0u32).unwrap();
    assert_eq!(
        RollResponse {
            owner: HumanAddr("addr0000".to_string()),
            creation_time: 0u64,
        },
        roll
    );

    let res = query_roll(&deps, HumanAddr("addr0000".to_string()), 1u32).unwrap_err();
    match res {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "No roll data stored");
        }
        _ => panic!("Must return generic error"),
    };

    // deposit more to fill rest part of roll
    let msg = HandleMsg::Deposit {
        amount: Uint128::from(50000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 1);
    let res: HandleResponse = handle(&mut deps, env, msg).unwrap();
    let msg = res.messages.get(0).expect("no message");
    let balance_log = res.log.get(1).expect("no data");
    let number_of_rolls_log = res.log.get(2).expect("no data");

    assert_eq!(
        &log("balance", &Uint128(200000000u128).to_string()),
        balance_log,
    );
    assert_eq!(
        &log("number_of_rolls", &2u32.to_string()),
        number_of_rolls_log,
    );
    assert_eq!(
        &CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: HumanAddr("staking0000".to_string()),
            send: vec![],
            msg: to_binary(&TokenHandleMsg::TransferFrom {
                owner: HumanAddr("addr0000".to_string()),
                recipient: HumanAddr(MOCK_CONTRACT_ADDR.to_string()),
                amount: Uint128(50000000u128),
            })
            .unwrap(),
        }),
        msg
    );

    let roll = query_roll(&deps, HumanAddr("addr0000".to_string()), 1u32).unwrap();
    assert_eq!(
        RollResponse {
            owner: HumanAddr("addr0000".to_string()),
            creation_time: 1u64,
        },
        roll
    );
}

#[test]
fn withdraw() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    // deposit will create 1 roll
    let msg = HandleMsg::Deposit {
        amount: Uint128::from(150000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 0);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    // withdraw but 1 roll is still left
    let msg = HandleMsg::Withdraw {
        amount: Uint128::from(50000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 0);
    let res: HandleResponse = handle(&mut deps, env, msg).unwrap();
    let msg = res.messages.get(0).expect("no message");
    let balance_log = res.log.get(1).expect("no data");
    let number_of_rolls_log = res.log.get(2).expect("no data");

    assert_eq!(
        &log("balance", &Uint128(100000000u128).to_string()),
        balance_log,
    );
    assert_eq!(
        &log("number_of_rolls", &1u32.to_string()),
        number_of_rolls_log,
    );
    assert_eq!(
        &CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: HumanAddr("staking0000".to_string()),
            send: vec![],
            msg: to_binary(&TokenHandleMsg::Transfer {
                recipient: HumanAddr("addr0000".to_string()),
                amount: Uint128(50000000u128),
            })
            .unwrap(),
        }),
        msg
    );

    // withdraw more, then the roll must be removed
    let msg = HandleMsg::Withdraw {
        amount: Uint128::from(50000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 0);
    let res: HandleResponse = handle(&mut deps, env, msg).unwrap();
    let msg = res.messages.get(0).expect("no message");
    let balance_log = res.log.get(1).expect("no data");
    let number_of_rolls_log = res.log.get(2).expect("no data");

    assert_eq!(
        &log("balance", &Uint128(50000000u128).to_string()),
        balance_log,
    );
    assert_eq!(
        &log("number_of_rolls", &0u32.to_string()),
        number_of_rolls_log,
    );
    assert_eq!(
        &CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: HumanAddr("staking0000".to_string()),
            send: vec![],
            msg: to_binary(&TokenHandleMsg::Transfer {
                recipient: HumanAddr("addr0000".to_string()),
                amount: Uint128(50000000u128),
            })
            .unwrap(),
        }),
        msg
    );

    let res = query_roll(&deps, HumanAddr("addr0000".to_string()), 0u32).unwrap_err();
    match res {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "No roll data stored");
        }
        _ => panic!("Must return generic error"),
    };
}

#[test]
fn distribute() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    // distribute in empty roll state
    let msg = HandleMsg::Distribute {
        amount: Uint128::from(1000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 0);
    let res = handle(&mut deps, env, msg).unwrap_err();
    match res {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "No rolls registered");
        }
        _ => panic!("Must return generic error"),
    };

    // deposit will create 1 roll
    let msg = HandleMsg::Deposit {
        amount: Uint128::from(150000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 0);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    // distribute to empty
    let msg = HandleMsg::Distribute {
        amount: Uint128::from(1000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 0);
    let res = handle(&mut deps, env, msg).unwrap_err();
    match res {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "No rolls registered");
        }
        _ => panic!("Must return generic error"),
    };

    // distribute one roll
    let msg = HandleMsg::Distribute {
        amount: Uint128::from(1000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 86401u64);
    let res: HandleResponse = handle(&mut deps, env, msg).unwrap();
    let rewards_per_roll_log = res.log.get(1).expect("no data");
    let number_of_rolls_log = res.log.get(2).expect("no data");

    assert_eq!(
        &log(
            "rewards_per_roll",
            &(Decimal::from_ratio(1000000u128, 1u128).to_string() + "uusd")
        ),
        rewards_per_roll_log,
    );
    assert_eq!(
        &log("number_of_rolls", &1u32.to_string()),
        number_of_rolls_log,
    );

    let staker: StakerResponse = query_staker(&deps, HumanAddr("addr0000".to_string())).unwrap();
    assert_eq!(
        Decimal::from_ratio(1000000u128, 1u128),
        staker.collected_rewards
    );

    // deposit more to create second roll
    let msg = HandleMsg::Deposit {
        amount: Uint128::from(50000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 0);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    // deposit from new account to create a roll
    let msg = HandleMsg::Deposit {
        amount: Uint128::from(100000000u128),
    };
    let env = mock_env_with_block_time("addr0001", &[], 0);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    // distribute to three roll
    let msg = HandleMsg::Distribute {
        amount: Uint128::from(1000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 86401u64);
    let res: HandleResponse = handle(&mut deps, env, msg).unwrap();
    let rewards_per_roll_log = res.log.get(1).expect("no data");
    let number_of_rolls_log = res.log.get(2).expect("no data");

    let expected_rewards_per_roll = Decimal::from_ratio(1000000u128, 3u128);
    assert_eq!(
        &log(
            "rewards_per_roll",
            &(expected_rewards_per_roll.to_string() + "uusd")
        ),
        rewards_per_roll_log,
    );
    assert_eq!(
        &log("number_of_rolls", &3u32.to_string()),
        number_of_rolls_log,
    );

    let staker: StakerResponse = query_staker(&deps, HumanAddr("addr0000".to_string())).unwrap();
    assert_eq!(
        Decimal::from_ratio(1000000u128, 1u128)
            + expected_rewards_per_roll
            + expected_rewards_per_roll,
        staker.collected_rewards
    );

    let staker: StakerResponse = query_staker(&deps, HumanAddr("addr0001".to_string())).unwrap();
    assert_eq!(expected_rewards_per_roll, staker.collected_rewards);
}

#[test]
fn claim() {
    let mut deps = mock_dependencies(20, &[]);
    let tax_rate = Decimal::percent(1);
    let tax_caps = &[("uusd", 1000000u128)];

    deps.querier.with_treasury(tax_rate, tax_caps);

    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    // deposit will create 1 roll
    let msg = HandleMsg::Deposit {
        amount: Uint128::from(150000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 0);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    // distribute one roll
    let msg = HandleMsg::Distribute {
        amount: Uint128::from(1000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 86401u64);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    // claim
    let msg = HandleMsg::Claim {};
    let env = mock_env("addr0000", &[]);
    let res: HandleResponse = handle(&mut deps, env, msg).unwrap();
    let rewards_log = res.log.get(1).expect("no data");
    let tax_log = res.log.get(2).expect("no data");
    let msg = res.messages.get(0).expect("no message");

    assert_eq!(
        &log("rewards", &(990000u32.to_string() + "uusd")),
        rewards_log,
    );
    assert_eq!(
        &log("tax", &(10000u32.to_string() + "uusd")),
        tax_log,
    );

    assert_eq!(
        &CosmosMsg::Bank(BankMsg::Send {
            from_address: HumanAddr(MOCK_CONTRACT_ADDR.to_string()),
            to_address: HumanAddr("addr0000".to_string()),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128(990000u128),
            }],
        }),
        msg
    );
    
    // claim tax cap test
    let msg = HandleMsg::Distribute {
        amount: Uint128::from(200000000u128),
    };
    let env = mock_env_with_block_time("addr0000", &[], 86401u64);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    // computed tax will be 2000000 but it must be capped to 100000
    let msg = HandleMsg::Claim {};
    let env = mock_env("addr0000", &[]);
    let res: HandleResponse = handle(&mut deps, env, msg).unwrap();
    let rewards_log = res.log.get(1).expect("no data");
    let tax_log = res.log.get(2).expect("no data");
    let msg = res.messages.get(0).expect("no message");

    assert_eq!(
        &log("rewards", &(199000000u32.to_string() + "uusd")),
        rewards_log,
    );
    assert_eq!(
        &log("tax", &(1000000u32.to_string() + "uusd")),
        tax_log,
    );

    assert_eq!(
        &CosmosMsg::Bank(BankMsg::Send {
            from_address: HumanAddr(MOCK_CONTRACT_ADDR.to_string()),
            to_address: HumanAddr("addr0000".to_string()),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128(199000000u128),
            }],
        }),
        msg
    );
}

fn mock_env_with_block_time<U: Into<HumanAddr>>(sender: U, sent: &[Coin], time: u64) -> Env {
    let env = mock_env(sender, sent);
    // register time
    return Env {
        block: BlockInfo {
            height: 1,
            time,
            chain_id: "columbus".to_string(),
        },
        ..env
    };
}
