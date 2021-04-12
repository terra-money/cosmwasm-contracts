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

use cosmwasm_std::{coin, coins, from_binary, CosmosMsg, Response, Uint128};
use cosmwasm_vm::testing::{
    execute, instantiate, mock_env, mock_info, mock_instance_with_options, query,
    MockInstanceOptions,
};

use terra_cosmwasm::{TerraMsg, TerraMsgWrapper, TerraRoute};

use maker::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

// This line will test the output of cargo wasm
static WASM: &[u8] = include_bytes!("../../../target/wasm32-unknown-unknown/release/maker.wasm");
// You can uncomment this line instead to test productionified build from cosmwasm-opt
// static WASM: &[u8] = include_bytes!("../contract.wasm");

#[test]
fn proper_initialization() {
    let mut deps = mock_instance_with_options(
        WASM,
        MockInstanceOptions {
            supported_features: [
                "terra".to_string(),
                "staking".to_string(),
                "stargate".to_string(),
            ]
            .iter()
            .cloned()
            .collect(),
            ..MockInstanceOptions::default()
        },
    );

    let msg = InstantiateMsg {
        ask: "BTC".into(),
        offer: "ETH".into(),
    };
    let info = mock_info("creator", &coins(1000, "earth"));

    // we can just call .unwrap() to assert this was a success
    let res: Response<TerraMsgWrapper> = instantiate(&mut deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&mut deps, mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("BTC", value.ask.as_str());
    assert_eq!("ETH", value.offer.as_str());
    assert_eq!("creator", value.owner.to_string().as_str());
}

#[test]
fn buy_limit() {
    let mut deps = mock_instance_with_options(
        WASM,
        MockInstanceOptions {
            supported_features: [
                "terra".to_string(),
                "staking".to_string(),
                "stargate".to_string(),
            ]
            .iter()
            .cloned()
            .collect(),
            contract_balance: Some(&coins(200, "ETH")),
            ..MockInstanceOptions::default()
        },
    );

    let msg = InstantiateMsg {
        ask: "BTC".into(),
        offer: "ETH".into(),
    };
    let info = mock_info("creator", &coins(200, "ETH"));
    let _res: Response<TerraMsgWrapper> = instantiate(&mut deps, mock_env(), info, msg).unwrap();

    // we buy BTC with half the ETH
    let info = mock_info("creator", &[]);
    let msg = ExecuteMsg::Buy {
        limit: Some(Uint128(100)),
        recipient: None,
    };
    let res: Response<TerraMsgWrapper> = execute(&mut deps, mock_env(), info, msg).unwrap();

    // make sure we produce proper trade order
    assert_eq!(1, res.messages.len());
    if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0] {
        assert_eq!(route, &TerraRoute::Market);

        match msg_data {
            TerraMsg::Swap {
                offer_coin,
                ask_denom,
            } => {
                assert_eq!(offer_coin, &coin(100, "ETH"));
                assert_eq!(ask_denom, "BTC");
            }
            _ => panic!("Should not enter here"),
        }
    } else {
        panic!("Expected swap message, got: {:?}", &res.messages[0]);
    }
}
