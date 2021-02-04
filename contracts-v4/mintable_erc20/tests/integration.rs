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
    from_slice, attr, CanonicalAddr, HandleResponse, HumanAddr, InitResponse, Uint128,
    coins,
};
use cosmwasm_storage::{to_length_prefixed, to_length_prefixed_nested};
use cosmwasm_vm::testing::{handle, init, mock_env, mock_instance, query, mock_info,};
use cosmwasm_vm::{Storage, Api};

use mintable_erc20::contract::{
    bytes_to_u128, Constants, KEY_CONSTANTS, KEY_MINTER, KEY_TOTAL_SUPPLY, PREFIX_ALLOWANCES,
    PREFIX_BALANCES, PREFIX_CONFIG,
};
use mintable_erc20::msg::{HandleMsg, InitMsg, InitialBalance, QueryMsg};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/mintable_erc20.wasm");

fn get_constants(storage: &dyn Storage) -> Constants {
    let key = [&to_length_prefixed(PREFIX_CONFIG), KEY_CONSTANTS].concat();
    let data = storage
        .get(&key)
        .0
        .expect("error getting data")
        .expect("no config data stored");
    from_slice(&data).expect("invalid data")
}

fn get_total_supply(storage: &dyn Storage) -> u128 {
    let key = [&to_length_prefixed(PREFIX_CONFIG), KEY_TOTAL_SUPPLY].concat();
    let data = storage
        .get(&key)
        .0
        .expect("error getting data")
        .expect("no decimals data stored");
    bytes_to_u128(&data).unwrap()
}

fn get_owner(storage: &dyn Storage) -> CanonicalAddr {
    let key = [&to_length_prefixed(PREFIX_CONFIG), KEY_MINTER].concat();
    let data = storage
        .get(&key)
        .0
        .expect("error getting data")
        .expect("no decimals data stored");

    let addr_raw: CanonicalAddr = from_slice(&data).expect("invalid data");
    return addr_raw;
}

fn get_balance(storage: &dyn Storage, address: &CanonicalAddr) -> u128 {
    let key = [
        &to_length_prefixed(&PREFIX_BALANCES),
        address.as_slice(),
    ]
    .concat();
    read_u128(storage, &key)
}

fn get_allowance(
    storage: &dyn Storage,
    owner: &CanonicalAddr,
    spender: &CanonicalAddr,
) -> u128 {
    let key = [
        &to_length_prefixed_nested(&[PREFIX_ALLOWANCES, owner.as_slice()]),
        spender.as_slice(),
    ]
    .concat();
    return read_u128(storage, &key);
}

// Reads 16 byte storage value into u128
// Returns zero if key does not exist. Errors if data found that is not 16 bytes
fn read_u128(storage: &dyn Storage, key: &[u8]) -> u128 {
    let result = storage.get(key).0.unwrap();
    match result {
        Some(data) => bytes_to_u128(&data).unwrap(),
        None => 0u128,
    }
}

fn address(index: u8) -> HumanAddr {
    match index {
        0 => HumanAddr("addr0000".to_string()), // contract initializer
        1 => HumanAddr("addr1111".to_string()),
        2 => HumanAddr("addr4321".to_string()),
        3 => HumanAddr("addr5432".to_string()),
        4 => HumanAddr("owner0000".to_string()),
        _ => panic!("Unsupported address index"),
    }
}

fn init_msg() -> InitMsg {
    InitMsg {
        minter: address(4),
        decimals: 5,
        name: "Ash token".to_string(),
        symbol: "ASH".to_string(),
        initial_balances: [
            InitialBalance {
                address: address(1),
                amount: Uint128::from(11u128),
            },
            InitialBalance {
                address: address(2),
                amount: Uint128::from(22u128),
            },
            InitialBalance {
                address: address(3),
                amount: Uint128::from(33u128),
            },
        ]
        .to_vec(),
    }
}

#[test]
fn init_works() {
    let mut deps = mock_instance(WASM, &[]);
    let init_msg = init_msg();
    let info = mock_info(address(0).as_str(), &coins(400, "token"));
    let res: InitResponse = init(&mut deps, mock_env(), info, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let raw_address_1 = deps.api()
            .canonical_address(&address(1))
            .0
            .expect("canonical_address failed");

    let raw_address_2 = deps.api()
            .canonical_address(&address(2))
            .0
            .expect("canonical_address failed");
    let raw_address_3 = deps.api()
            .canonical_address(&address(3))
            .0
            .expect("canonical_address failed");
    let raw_address_4 = deps.api()
            .canonical_address(&address(4))
            .0
            .expect("canonical_address failed");
    
    deps.with_storage(|store| {
        assert_eq!(
            get_constants(store),
            Constants {
                name: "Ash token".to_string(),
                symbol: "ASH".to_string(),
                decimals: 5
            }
        );
        
        assert_eq!(get_total_supply(store), 66);
        assert_eq!(get_balance(store, &raw_address_1), 11);
        assert_eq!(get_balance(store, &raw_address_2), 22);
        assert_eq!(get_balance(store, &raw_address_3), 33);
        assert_eq!(get_owner(store), raw_address_4);
        Ok(())
    }).unwrap()
}

#[test]
fn transfer_works() {
    let mut deps = mock_instance(WASM, &[]);
    let init_msg = init_msg();
    let info1 = mock_info(address(0).as_str(), &coins(400, "token"));
    let res: InitResponse = init(&mut deps, mock_env(), info1, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let sender = address(1);
    let recipient = address(2);
    let sender_raw = deps.api()
        .canonical_address(&sender)
        .0
        .expect("canonical_address failed");
    let recipient_raw = deps.api()
        .canonical_address(&recipient)
        .0
        .expect("canonical_address failed");

    // Before
    deps.with_storage(|storage| {
        assert_eq!(get_balance(storage, &sender_raw), 11);
        assert_eq!(get_balance(storage, &recipient_raw), 22);
        Ok(())
    })
    .unwrap();

    // Transfer
    let transfer_msg = HandleMsg::Transfer {
        recipient: recipient.clone(),
        amount: Uint128::from(1u128),
    };
    let info2 = mock_info(sender.as_str(), &coins(400, "token"));
    let transfer_response: HandleResponse = handle(&mut deps, mock_env(), info2, transfer_msg).unwrap();
    assert_eq!(transfer_response.messages.len(), 0);
    assert_eq!(
        transfer_response.attributes,
        vec![
            attr("action", "transfer"),
            attr("sender", sender.as_str()),
            attr("recipient", recipient.as_str()),
        ]
    );

    // After
    deps.with_storage(|storage| {
        assert_eq!(get_balance(storage, &sender_raw), 10);
        assert_eq!(get_balance(storage, &recipient_raw), 23);
        Ok(())
    })
    .unwrap();
}

#[test]
fn approve_works() {
    let mut deps = mock_instance(WASM, &[]);
    let init_msg = init_msg();
    let info1 = mock_info(address(0).as_str(), &coins(400, "token"));
    let res: InitResponse = init(&mut deps, mock_env(), info1, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let owner = address(1);
    let spender = address(2);
    let owner_raw = deps.api()
        .canonical_address(&owner)
        .0
        .expect("canonical_address failed");
    let spender_raw = deps.api()
        .canonical_address(&spender)
        .0
        .expect("canonical_address failed");

    // Before
    deps.with_storage(|storage| {
        assert_eq!(get_allowance(storage, &owner_raw, &spender_raw), 0);
        Ok(())
    })
    .unwrap();

    // Approve
    let approve_msg = HandleMsg::Approve {
        spender: spender.clone(),
        amount: Uint128::from(42u128),
    };
    let info2 = mock_info(owner.as_str(), &coins(400, "token"));
    let approve_response: HandleResponse = handle(&mut deps, mock_env(), info2, approve_msg).unwrap();
    assert_eq!(approve_response.messages.len(), 0);
    assert_eq!(
        approve_response.attributes,
        vec![
            attr("action", "approve"),
            attr("owner", owner.as_str()),
            attr("spender", spender.as_str()),
        ]
    );

    // After
    deps.with_storage(|storage| {
        assert_eq!(get_allowance(storage, &owner_raw, &spender_raw), 42);
        Ok(())
    })
    .unwrap();
}

#[test]
fn transfer_from_works() {
    let mut deps = mock_instance(WASM, &[]);
    let init_msg = init_msg();
    let info1 = mock_info(address(0).as_str(), &coins(400, "token"));
    let res: InitResponse = init(&mut deps, mock_env(), info1, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let owner = address(1);
    let spender = address(2);
    let recipient = address(3);
    let owner_raw = deps.api()
        .canonical_address(&owner)
        .0
        .expect("canonical_address failed");
    let spender_raw = deps.api()
        .canonical_address(&spender)
        .0
        .expect("canonical_address failed");
    let recipient_raw = deps.api()
        .canonical_address(&recipient)
        .0
        .expect("canonical_address failed");

    // Before
    deps.with_storage(|storage| {
        assert_eq!(get_balance(storage, &owner_raw), 11);
        assert_eq!(get_balance(storage, &recipient_raw), 33);
        assert_eq!(get_allowance(storage, &owner_raw, &spender_raw), 0);
        Ok(())
    })
    .unwrap();

    // Approve
    let approve_msg = HandleMsg::Approve {
        spender: spender.clone(),
        amount: Uint128::from(42u128),
    };
    let info2 = mock_info(owner.as_str(), &coins(400,"token"));
    let approve_response: HandleResponse = handle(&mut deps, mock_env(), info2, approve_msg).unwrap();
    assert_eq!(approve_response.messages.len(), 0);
    assert_eq!(
        approve_response.attributes,
        vec![
            attr("action", "approve"),
            attr("owner", owner.as_str()),
            attr("spender", spender.as_str()),
        ]
    );

    // Transfer from
    let transfer_from_msg = HandleMsg::TransferFrom {
        owner: owner.clone(),
        recipient: recipient.clone(),
        amount: Uint128::from(2u128),
    };
    let info3 = mock_info(spender.as_str(), &coins(400, "token"));
    let transfer_from_response: HandleResponse =
        handle(&mut deps, mock_env(), info3, transfer_from_msg).unwrap();
    
    assert_eq!(transfer_from_response.messages.len(), 0);
    assert_eq!(
        transfer_from_response.attributes,
        vec![
            attr("action", "transfer_from"),
            attr("spender", spender.as_str()),
            attr("sender", owner.as_str()),
            attr("recipient", recipient.as_str()),
        ]
    );

    // After
    deps.with_storage(|storage| {
        assert_eq!(get_balance(storage, &owner_raw), 9);
        assert_eq!(get_balance(storage, &recipient_raw), 35);
        assert_eq!(get_allowance(storage, &owner_raw, &spender_raw), 40);
        Ok(())
    })
    .unwrap();
}

#[test]
fn burn_works() {
    let mut deps = mock_instance(WASM, &[]);
    let init_msg = init_msg();
    let info1 = mock_info(address(0).as_str(), &coins(400, "token"));
    let res: InitResponse = init(&mut deps, mock_env(), info1, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let burner = address(1);
    let burner_raw = deps.api()
        .canonical_address(&burner)
        .0
        .expect("canonical_address failed");

    // Before
    deps.with_storage(|storage| {
        assert_eq!(get_balance(storage, &burner_raw), 11);
        Ok(())
    })
    .unwrap();

    // Burn
    let burn_msg = HandleMsg::Burn {
        amount: Uint128::from(1u128),
        burner: address(1),
    };
    let info2 = mock_info(address(4).as_str(), &coins(400, "token"));
    let burn_response: HandleResponse = handle(&mut deps, mock_env(), info2, burn_msg).unwrap();
    assert_eq!(burn_response.messages.len(), 0);
    assert_eq!(
        burn_response.attributes,
        vec![
            attr("action", "burn"),
            attr("burner", burner.as_str()),
            attr("amount", "1")
        ]
    );

    // After
    deps.with_storage(|storage| {
        assert_eq!(get_balance(storage, &burner_raw), 10);
        Ok(())
    })
    .unwrap();
}

#[test]
fn mint_works() {
    let mut deps = mock_instance(WASM, &[]);
    let init_msg = init_msg();
    let info1 = mock_info(address(0).as_str(), &coins(400, "token"));
    let res: InitResponse = init(&mut deps, mock_env(), info1, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let recipient = address(1);
    let recipient_raw = deps.api()
        .canonical_address(&recipient)
        .0
        .expect("canonical_address failed");

    // Before
    deps.with_storage(|storage| {
        assert_eq!(get_balance(storage, &recipient_raw), 11);
        Ok(())
    })
    .unwrap();

    // Mint
    let mint_msg = HandleMsg::Mint {
        amount: Uint128::from(1u128),
        recipient: address(1),
    };
    let info2 = mock_info(address(4).as_str(), &coins(400, "token"));
    let mint_response: HandleResponse = handle(&mut deps, mock_env(), info2, mint_msg).unwrap();
    assert_eq!(mint_response.messages.len(), 0);
    assert_eq!(
        mint_response.attributes,
        vec![
            attr("action", "mint"),
            attr("recipient", recipient.as_str()),
            attr("amount", "1")
        ]
    );

    // After
    deps.with_storage(|storage| {
        assert_eq!(get_balance(storage, &recipient_raw), 12);
        Ok(())
    })
    .unwrap();
}

#[test]
fn can_query_balance_of_existing_address() {
    let mut deps = mock_instance(WASM, &[]);
    let init_msg = init_msg();
    let info1 = mock_info(address(0).as_str(), &coins(400, "token"));
    let res: InitResponse = init(&mut deps, mock_env(), info1, init_msg).unwrap();
    assert_eq!(0, res.messages.len());

    let query_msg = QueryMsg::Balance {
        address: address(2),
    };
    let query_result = query(&mut deps, mock_env(), query_msg).unwrap();
    assert_eq!(query_result.as_slice(), b"{\"balance\":\"22\"}");
}
