//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests.
//! 1. First copy them over verbatum,
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
    coin, coins, from_binary, Coin, CosmosMsg, HandleResponse, InitResponse, Uint128,
};
use cosmwasm_vm::testing::{
    handle, init, mock_dependencies, mock_env, query, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_vm::{Api, Instance};

use terra_bindings::{TerraMsg, TerraMsgWrapper};

use maker::msg::{ConfigResponse, HandleMsg, InitMsg, QueryMsg};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/maker.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");

const DEFAULT_GAS_LIMIT: u64 = 500_000;

// TODO: improve the whole state of this
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

#[test]
fn proper_initialization() {
    let mut deps = mock_instance(WASM, &[]);

    let msg = InitMsg {
        ask: "BTC".into(),
        offer: "ETH".into(),
    };
    let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res: InitResponse<TerraMsg> = init(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&mut deps, QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("BTC", value.ask.as_str());
    assert_eq!("ETH", value.offer.as_str());
    assert_eq!("creator", value.owner.as_str());
}

#[test]
fn buy_limit() {
    let mut deps = mock_instance(WASM, &coins(200, "ETH"));

    let msg = InitMsg {
        ask: "BTC".into(),
        offer: "ETH".into(),
    };
    let env = mock_env(&deps.api, "creator", &coins(200, "ETH"));
    let _res: InitResponse<TerraMsgWrapper> = init(&mut deps, env, msg).unwrap();

    // we buy BTC with half the ETH
    let env = mock_env(&deps.api, "creator", &[]);
    let contract_addr = deps.api.human_address(&env.contract.address).unwrap();
    let msg = HandleMsg::Buy {
        limit: Some(Uint128(100)),
        recipient: None,
    };
    let res: HandleResponse<TerraMsgWrapper> = handle(&mut deps, env, msg).unwrap();

    // make sure we produce proper trade order
    assert_eq!(1, res.messages.len());
    if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0] {
        assert_eq!(route, "market");

        match msg_data {
            TerraMsg::Swap {
                trader,
                offer_coin,
                ask_denom,
            } => {
                assert_eq!(trader, &contract_addr);
                assert_eq!(offer_coin, &coin(100, "ETH"));
                assert_eq!(ask_denom, "BTC");
            }
            _ => panic!("Should not enter here")
        }
    } else {
        panic!("Expected swap message, got: {:?}", &res.messages[0]);
    }
}
