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
    coins, from_slice, BankMsg, CosmosMsg, HandleResponse, HumanAddr, InitResponse,
    ContractResult,
};
use cosmwasm_vm::testing::{handle, init, mock_env, mock_instance, query, mock_info, MOCK_CONTRACT_ADDR};
use cw_mask::msg::{HandleMsg, InitMsg, OwnerResponse, QueryMsg, CustomMsgWrapper};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/cw_mask.wasm");
// You can uncomment this line instead to test productionified build from rust-optimizer
// static WASM: &[u8] = include_bytes!("../contract.wasm");

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res: InitResponse = init(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&mut deps, mock_env(), QueryMsg::Owner {}).unwrap();
    let value: OwnerResponse = from_slice(res.as_slice()).unwrap();
    assert_eq!("creator", value.owner.as_str());
}

#[test]
fn reflect() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: InitResponse = init(&mut deps, mock_env(), info, msg).unwrap();

    let payload = vec![CosmosMsg::Bank(BankMsg::Send {
        from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
        to_address: HumanAddr::from("friend"),
        amount: coins(1, "token"),
    })];
    let msg = HandleMsg::ReflectMsg {
        msgs: payload.clone(),
    };
    let info = mock_info("creator", &[]);
    let res = handle(&mut deps, mock_env(), info, msg).unwrap();

    // should return payload
    assert_eq!(payload, res.messages);
}

#[test]
fn reflect_requires_owner() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: InitResponse = init(&mut deps, mock_env(), info, msg).unwrap();

    let payload = vec![BankMsg::Send {
        from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
        to_address: HumanAddr::from("friend"),
        amount: coins(1, "token"),
    }
    .into()];
    let msg = HandleMsg::ReflectMsg {
        msgs: payload.clone(),
    };
    let info = mock_info("unauthorized", &[]);
    let res: ContractResult<HandleResponse<CustomMsgWrapper>> = handle(&mut deps, mock_env(), info, msg);
    let err = res.unwrap_err();
    assert!(err.contains("Unauthorized"));
}

#[test]
fn transfer() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: InitResponse = init(&mut deps, mock_env(), info, msg).unwrap();

    let new_owner = HumanAddr::from("friend");
    let msg = HandleMsg::ChangeOwner {
        owner: new_owner.clone(),
    };
    let info = mock_info("creator", &[]);
    let res: HandleResponse = handle(&mut deps, mock_env(), info, msg).unwrap();
    // should change state
    assert_eq!(0, res.messages.len());
    let res = query(&mut deps, mock_env(), QueryMsg::Owner {}).unwrap();
    let value: OwnerResponse = from_slice(res.as_slice()).unwrap();
    assert_eq!("friend", value.owner.as_str());
}

#[test]
fn transfer_requires_owner() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {};
    let info = mock_info("creator", &coins(2, "token"));
    let _res: InitResponse = init(&mut deps, mock_env(), info, msg).unwrap();

    let info = mock_info("unauthorized", &[]);
    let new_owner = HumanAddr::from("friend");
    let msg = HandleMsg::ChangeOwner {
        owner: new_owner.clone(),
    };
    let res: ContractResult<HandleResponse> = handle(&mut deps, mock_env(), info, msg);
    let err = res.unwrap_err();
    assert!(err.contains("Unauthorized"));
}
