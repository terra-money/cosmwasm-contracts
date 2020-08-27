//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests as follows:
//! 1. Copy them over verbatim
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use cosmwasm_std::{
    from_binary, Coin, Decimal, Env, HandleResponse, HandleResult, HumanAddr, InitResponse,
    StdError, Uint128,
};
use cosmwasm_vm::testing::{
    handle, init, mock_dependencies, mock_env, query, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::Instance;
use roll_staking::msg::{
    ConfigResponse, HandleMsg, InitMsg, QueryMsg, RollResponse, StakerResponse,
};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/roll_staking.wasm");
// You can uncomment this line instead to test productionified build from rust-optimizer
// static WASM: &[u8] = include_bytes!("../contract.wasm");

const DEFAULT_GAS_LIMIT: u64 = 500_000;

pub fn mock_instance(
    wasm: &[u8],
    contract_balance: &[Coin],
) -> Instance<MockStorage, MockApi, MockQuerier> {
    // TODO: check_wasm is not exported from cosmwasm_vm
    // let terra_features = features_from_csv("staking,terra");
    // check_wasm(wasm, &terra_features).unwrap();
    let deps = mock_dependencies(20, contract_balance);
    Instance::from_code(wasm, deps, DEFAULT_GAS_LIMIT).unwrap()
}

fn mock_env_height(signer: &HumanAddr, height: u64, time: u64) -> Env {
    let mut env = mock_env(signer, &[]);
    env.block.height = height;
    env.block.time = time;
    env
}

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let res: InitResponse = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let value: ConfigResponse =
        from_binary(&query(&mut deps, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!("addr0000", value.owner.as_str());
    assert_eq!("staking0000", value.staking_token.as_str());
    assert_eq!(Uint128(100000000u128), value.roll_unit);
    assert_eq!(86400u64, value.deposit_period);
    assert_eq!("uusd", value.rewards_denom.as_str());
}

#[test]
fn update_owner() {
    let mut deps = mock_instance(WASM, &[]);
    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res: InitResponse = init(&mut deps, env, msg).unwrap();
    // update owner
    let env = mock_env("addr0000", &[]);
    let msg = HandleMsg::UpdateConfig {
        owner: Some(HumanAddr("addr0001".to_string())),
        deposit_period: None,
    };

    let res: HandleResponse = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let query_result = query(&mut deps, QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&query_result).unwrap();
    assert_eq!("addr0001", value.owner.as_str());
    assert_eq!(86400u64, value.deposit_period);

    // Unauthorzied err
    let env = mock_env("addr0000", &[]);
    let msg = HandleMsg::UpdateConfig {
        owner: None,
        deposit_period: Some(Uint128::from(100000u128)),
    };

    let res: HandleResult = handle(&mut deps, env, msg);
    match res.unwrap_err() {
        StdError::Unauthorized { .. } => {}
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn deposit_test() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res: InitResponse = init(&mut deps, env, msg).unwrap();

    // deposit will create 1 roll
    let msg = HandleMsg::Deposit {
        amount: Uint128::from(150000000u128),
    };
    let env = mock_env_height(&HumanAddr("addr0000".to_string()), 0, 0);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    let query_result = query(
        &mut deps,
        QueryMsg::Staker {
            address: HumanAddr("addr0000".to_string()),
        },
    )
    .unwrap();
    let staker: StakerResponse = from_binary(&query_result).unwrap();
    assert_eq!(HumanAddr("addr0000".to_string()), staker.address);
    assert_eq!(Uint128(150000000u128), staker.balance);
    assert_eq!(Decimal::zero(), staker.collected_rewards);

    let query_result = query(
        &mut deps,
        QueryMsg::Roll {
            address: HumanAddr("addr0000".to_string()),
            index: 0,
        },
    )
    .unwrap();
    let roll: RollResponse = from_binary(&query_result).unwrap();
    assert_eq!(HumanAddr("addr0000".to_string()), roll.owner);
    assert_eq!(0u64, roll.creation_time);
}

#[test]
fn withdraw_test() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res: InitResponse = init(&mut deps, env, msg).unwrap();

    // deposit will create 1 roll
    let msg = HandleMsg::Deposit {
        amount: Uint128::from(150000000u128),
    };
    let env = mock_env_height(&HumanAddr("addr0000".to_string()), 0, 0);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    // withdraw will destory 1 roll
    let msg = HandleMsg::Withdraw {
        amount: Uint128::from(100000000u128),
    };
    let env = mock_env_height(&HumanAddr("addr0000".to_string()), 0, 0);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    let query_result = query(
        &mut deps,
        QueryMsg::Staker {
            address: HumanAddr("addr0000".to_string()),
        },
    )
    .unwrap();
    let staker: StakerResponse = from_binary(&query_result).unwrap();
    assert_eq!(HumanAddr("addr0000".to_string()), staker.address);
    assert_eq!(Uint128(50000000u128), staker.balance);
    assert_eq!(Decimal::zero(), staker.collected_rewards);

    let query_result = query(
        &mut deps,
        QueryMsg::Roll {
            address: HumanAddr("addr0000".to_string()),
            index: 0,
        },
    )
    .unwrap_err();
    match query_result {
        StdError::GenericErr { msg, .. } => {
            assert_eq!(msg, "No roll data stored");
        }
        _ => panic!("Must return generic error"),
    };
}

#[test]
fn distribute_test() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {
        staking_token: HumanAddr("staking0000".to_string()),
        roll_unit: Uint128(100000000u128),
        deposit_period: Uint128(86400u128),
        rewards_denom: "uusd".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res: InitResponse = init(&mut deps, env, msg).unwrap();

    // deposit will create 1 roll
    let msg = HandleMsg::Deposit {
        amount: Uint128::from(150000000u128),
    };
    let env = mock_env_height(&HumanAddr("addr0000".to_string()), 0, 0);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

    // distribute rewards to 1 roll
    let msg = HandleMsg::Distribute {
        amount: Uint128::from(1000000u128),
    };
    let env = mock_env_height(&HumanAddr("addr0000".to_string()), 1u64, 86401u64);
    let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();
    let query_result = query(
        &mut deps,
        QueryMsg::Staker {
            address: HumanAddr("addr0000".to_string()),
        },
    )
    .unwrap();

    let staker: StakerResponse = from_binary(&query_result).unwrap();
    assert_eq!(
        Decimal::from_ratio(1000000u128, 1u128),
        staker.collected_rewards
    );
}

// Skip integration check due to lack of custom querier
// #[test]
// fn claim_test() {
//     let mut deps = mock_instance(WASM, &[]);

//     let msg = InitMsg {
//         staking_token: HumanAddr("staking0000".to_string()),
//         roll_unit: Uint128(100000000u128),
//         deposit_period: Uint128(86400u128),
//         rewards_denom: "uusd".to_string(),
//     };

//     let env = mock_env("addr0000", &[]);
//     let _res: InitResponse = init(&mut deps, env, msg).unwrap();

//     // deposit will create 1 roll
//     let msg = HandleMsg::Deposit {
//         amount: Uint128::from(150000000u128),
//     };
//     let env = mock_env_height(&HumanAddr("addr0000".to_string()), 0, 0);
//     let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

//     let msg = HandleMsg::Distribute {
//         amount: Uint128::from(1000000u128),
//     };
//     let env = mock_env_height(&HumanAddr("addr0000".to_string()), 1u64, 86401u64);
//     let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

//     let msg = HandleMsg::Claim {};
//     let env = mock_env("addr0000", &[]);
//     let _res: HandleResponse = handle(&mut deps, env, msg).unwrap();

//     let query_result = query(
//         &mut deps,
//         QueryMsg::Staker {
//             address: HumanAddr("addr0000".to_string()),
//         },
//     )
//     .unwrap();

//     let staker: StakerResponse = from_binary(&query_result).unwrap();
//     assert_eq!(Decimal::zero(), staker.collected_rewards);
// }
