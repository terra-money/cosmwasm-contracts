use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

use crate::msg::{
    AllowanceResponse, BalanceResponse, HandleMsg, InitMsg, MinterResponse, QueryMsg,
    TotalSupplyResposne,
};
use crate::errors::ContractError;
use cosmwasm_std::{
    from_slice, attr, to_binary, to_vec, Binary, CanonicalAddr, Env, HandleResponse,
    HumanAddr, InitResponse, StdError, StdResult, Storage, Uint128, Deps, DepsMut,
    MessageInfo,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Constants {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

pub const PREFIX_CONFIG: &[u8] = b"config";
pub const PREFIX_BALANCES: &[u8] = b"balances";
pub const PREFIX_ALLOWANCES: &[u8] = b"allowances";

pub const KEY_MINTER: &[u8] = b"minter";
pub const KEY_CONSTANTS: &[u8] = b"constants";
pub const KEY_TOTAL_SUPPLY: &[u8] = b"total_supply";

pub fn init(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> Result<InitResponse, ContractError> {
    let mut total_supply: u128 = 0;
    {
        // Initial balances
        let mut balances_store = PrefixedStorage::new(deps.storage, PREFIX_BALANCES);
        for row in msg.initial_balances {
            let raw_address = deps.api.canonical_address(&row.address)?;
            let amount_raw = row.amount.u128();
            println!("address: {} -- amount {} ",raw_address,amount_raw);
            balances_store.set(raw_address.as_slice(), &amount_raw.to_be_bytes());
            total_supply += amount_raw;
            println!("total supply {}", total_supply);
        }
    }

    // Check name, symbol, decimals
    if !is_valid_name(&msg.name) {
        return Err(ContractError::InvalidName{});
    }
    if !is_valid_symbol(&msg.symbol) {
        return Err(ContractError::InvalidSymbol{});
    }
    if msg.decimals > 18 {
        return Err(ContractError::InvalidDecimals{});
    }

    let mut config_store = PrefixedStorage::new(deps.storage, PREFIX_CONFIG);
    let constants = to_vec(&Constants {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
    })?;

    let minter_address_raw = deps.api.canonical_address(&msg.minter)?;
    config_store.set(KEY_MINTER, &to_vec(&minter_address_raw)?);
    config_store.set(KEY_CONSTANTS, &constants);
    config_store.set(KEY_TOTAL_SUPPLY, &total_supply.to_be_bytes());

    Ok(InitResponse::default())
}

pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        HandleMsg::Approve { spender, amount } => try_approve(deps, env, info, &spender, &amount),
        HandleMsg::Transfer { recipient, amount } => try_transfer(deps, env, info, &recipient, &amount),
        HandleMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => try_transfer_from(deps, env, info, &owner, &recipient, &amount),
        HandleMsg::Burn { amount, burner } => try_burn(deps, env, info, &amount, &burner),
        HandleMsg::Mint { amount, recipient } => try_mint(deps, env, info, &amount, &recipient),
    }
}

pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Minter {} => {
            let minter_address_raw: CanonicalAddr = read_minter(deps.storage);
            let out = to_binary(&MinterResponse {
                address: deps
                    .api
                    .human_address(&minter_address_raw)
                    .expect("invalid data"),
            })?;
            Ok(out)
        }
        QueryMsg::Balance { address } => {
            let address_key = deps.api.canonical_address(&address)?;
            let balance = read_balance(deps.storage, &address_key)?;
            let out = to_binary(&BalanceResponse {
                balance: Uint128::from(balance),
            })?;
            Ok(out)
        }
        QueryMsg::Allowance { owner, spender } => {
            let owner_key = deps.api.canonical_address(&owner)?;
            let spender_key = deps.api.canonical_address(&spender)?;
            let allowance = read_allowance(deps.storage, &owner_key, &spender_key)?;
            let out = to_binary(&AllowanceResponse {
                allowance: Uint128::from(allowance),
            })?;
            Ok(out)
        }
        QueryMsg::TotalSupply {} => {
            let config_store = ReadonlyPrefixedStorage::new(deps.storage, PREFIX_CONFIG);
            let data = config_store
                .get(KEY_TOTAL_SUPPLY)
                .expect("no total supply data stored");
            let total_supply: Uint128 = Uint128(bytes_to_u128(&data).unwrap());
            let out = to_binary(&TotalSupplyResposne { total_supply })?;
            Ok(out)
        }
    }
}

fn try_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: &HumanAddr,
    amount: &Uint128,
) -> Result<HandleResponse, ContractError> {
    let sender_address_raw = &info.sender;
    let recipient_address_raw = deps.api.canonical_address(recipient)?;
    let amount_raw = amount.u128();

    perform_transfer(
        deps.storage,
        &deps.api.canonical_address(&sender_address_raw)?,
        &recipient_address_raw,
        amount_raw,
    )?;

    let res = HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "transfer"),
            attr("sender", &info.sender),
            attr("recipient", &recipient),
        ],
        data: None,
    };
    Ok(res)
}

fn try_transfer_from(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo, 
    owner: &HumanAddr,
    recipient: &HumanAddr,
    amount: &Uint128,
) -> Result<HandleResponse, ContractError> {
    let spender_address_raw = &deps.api.canonical_address(&info.sender)?;
    let owner_address_raw = deps.api.canonical_address(owner)?;
    let recipient_address_raw = deps.api.canonical_address(recipient)?;
    let amount_raw = amount.u128();

    let mut allowance = read_allowance(deps.storage, &owner_address_raw, &spender_address_raw)?;
    if allowance < amount_raw {
        return Err(ContractError::InsufficientAllowance{});
    }
    allowance -= amount_raw;
    write_allowance(
        deps.storage,
        &owner_address_raw,
        &spender_address_raw,
        allowance,
    )?;
    perform_transfer(
        deps.storage,
        &owner_address_raw,
        &recipient_address_raw,
        amount_raw,
    )?;

    let res = HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "transfer_from"),
            attr("spender", &info.sender),
            attr("sender", owner.as_str()),
            attr("recipient", recipient.as_str()),
        ],
        data: None,
    };
    Ok(res)
}

fn try_approve(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    spender: &HumanAddr,
    amount: &Uint128,
) -> Result<HandleResponse, ContractError> {
    let owner_address_raw = deps.api.canonical_address(&info.sender)?;
    let spender_address_raw = deps.api.canonical_address(spender)?;
    write_allowance(
        deps.storage,
        &owner_address_raw,
        &spender_address_raw,
        amount.u128(),
    )?;
    let res = HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "approve"),
            attr("owner", &info.sender),
            attr("spender", spender.as_str()),
        ],
        data: None,
    };
    Ok(res)
}

/// Burn tokens
///
/// Remove `amount` tokens from the system irreversibly, from signer account
///
/// @param amount the amount of money to burn
/// @param burner the address of the asset holder
fn try_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: &Uint128,
    burner: &HumanAddr,
) -> Result<HandleResponse, ContractError> {
    // only the owner can do burning
    let config_store = ReadonlyPrefixedStorage::new(deps.storage, PREFIX_CONFIG);

    let data = config_store.get(KEY_MINTER).expect("no owner data stored");

    let owner_address_raw: CanonicalAddr = from_slice(&data).expect("invalid address data");
    
    if deps.api.canonical_address(&info.sender)? != owner_address_raw {
        return Err(ContractError::Unauthorized{});
    }

    let burner_address_raw = &deps.api.canonical_address(burner)?;
    let amount_raw = amount.u128();

    let mut account_balance = read_balance(deps.storage, burner_address_raw)?;

    if account_balance < amount_raw {
        return Err(ContractError::InsufficientFunds{});
    }
    account_balance -= amount_raw;

    let mut balances_store = PrefixedStorage::new(deps.storage, PREFIX_BALANCES);
    balances_store.set(
        burner_address_raw.as_slice(),
        &account_balance.to_be_bytes(),
    );

    let mut config_store = PrefixedStorage::new(deps.storage, PREFIX_CONFIG);
    let data = config_store
        .get(KEY_TOTAL_SUPPLY)
        .expect("no total supply data stored");
    let mut total_supply = bytes_to_u128(&data).unwrap();

    total_supply -= amount_raw;

    config_store.set(KEY_TOTAL_SUPPLY, &total_supply.to_be_bytes());

    let res = HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "burn"),
            attr("burner", burner.as_str()),
            attr("amount", &amount.to_string()),
        ],
        data: None,
    };

    Ok(res)
}

/// Mint tokens
///
/// Append `amount` tokens to the system and the account, from signer account
///
/// @param amount the amount of money to mint
/// @param the recipient address for the minted asset
fn try_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: &Uint128,
    recipient: &HumanAddr,
) -> Result<HandleResponse, ContractError> {
    // only the minter can do minting
    let config_store = ReadonlyPrefixedStorage::new(deps.storage, PREFIX_CONFIG);
    let data = config_store.get(KEY_MINTER).expect("no minter data stored");

    let minter_address_raw: CanonicalAddr = from_slice(&data).expect("invalid address data");
    if deps.api.canonical_address(&info.sender)? != minter_address_raw {
        return Err(ContractError::Unauthorized{});
    }

    let recipient_address_raw = &deps.api.canonical_address(recipient)?;
    let amount_raw = amount.u128();

    let mut account_balance = read_balance(deps.storage, recipient_address_raw)?;
    account_balance += amount_raw;

    let mut balances_store = PrefixedStorage::new(deps.storage, PREFIX_BALANCES);
    balances_store.set(
        recipient_address_raw.as_slice(),
        &account_balance.to_be_bytes(),
    );

    let mut config_store = PrefixedStorage::new(deps.storage, PREFIX_CONFIG);
    let data = config_store
        .get(KEY_TOTAL_SUPPLY)
        .expect("no total supply data stored");
    let mut total_supply = bytes_to_u128(&data).unwrap();

    total_supply += amount_raw;

    config_store.set(KEY_TOTAL_SUPPLY, &total_supply.to_be_bytes());

    let res = HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "mint"),
            attr("recipient", recipient.as_str()),
            attr("amount", &amount.to_string()),
        ],
        data: None,
    };

    Ok(res)
}

fn perform_transfer(
    store: &mut dyn Storage,
    from: &CanonicalAddr,
    to: &CanonicalAddr,
    amount: u128,
) -> StdResult<()> {
    let mut balances_store = PrefixedStorage::new(store, PREFIX_BALANCES);

    let mut from_balance = read_u128_pre(&balances_store, from.as_slice())?;
    if from_balance < amount {
        return Err(StdError::generic_err(format!(
            "Insufficient funds: balance={}, required={}",
            from_balance, amount
        )));
    }
    from_balance -= amount;
    balances_store.set(from.as_slice(), &from_balance.to_be_bytes());

    let mut to_balance = read_u128_pre(&balances_store, to.as_slice())?;
    to_balance += amount;
    balances_store.set(to.as_slice(), &to_balance.to_be_bytes());

    Ok(())
}

// Converts 16 bytes value into u128
// Errors if data found that is not 16 bytes
pub fn bytes_to_u128(data: &[u8]) -> StdResult<u128> {
    match data[0..16].try_into() {
        Ok(bytes) => Ok(u128::from_be_bytes(bytes)),
        Err(_) => Err(StdError::generic_err(
            "Corrupted data found. 16 byte expected.",
        )),
    }
}

fn read_minter(storage: &dyn Storage) -> CanonicalAddr {
    let config_storage = ReadonlyPrefixedStorage::new(storage, PREFIX_CONFIG);
    let data = config_storage
        .get(KEY_MINTER)
        .expect("no config data stored");
    from_slice(&data).expect("invalid data")
}

pub fn read_u128(store: &dyn Storage, key: &[u8]) -> StdResult<u128> {
    let result = store.get(key);
    match result {
        Some(data) => bytes_to_u128(&data),
        None => Ok(0u128),
    }
}

pub fn read_u128_pre(store: &PrefixedStorage, key: &[u8]) -> StdResult<u128> {
    let result = store.get(key);
    match result {
        Some(data) => bytes_to_u128(&data),
        None => Ok(0u128),
    }
}

pub fn read_u128_ropre(store: &ReadonlyPrefixedStorage, key: &[u8]) -> StdResult<u128> {
    let result = store.get(key);
    match result {
        Some(data) => bytes_to_u128(&data),
        None => Ok(0u128),
    }
}

fn read_balance(store: &dyn Storage, owner: &CanonicalAddr) -> StdResult<u128> {
    let balance_store = ReadonlyPrefixedStorage::new(store, PREFIX_BALANCES);
    read_u128_ropre(&balance_store, owner.as_slice())
}

fn read_allowance(
    store: &dyn Storage,
    owner: &CanonicalAddr,
    spender: &CanonicalAddr,
) -> StdResult<u128> {
    let owner_store = ReadonlyPrefixedStorage::multilevel(store, &[PREFIX_ALLOWANCES, owner.as_slice()]);
    read_u128_ropre(&owner_store, spender.as_slice())
}

fn write_allowance(
    store: &mut dyn Storage,
    owner: &CanonicalAddr,
    spender: &CanonicalAddr,
    amount: u128,
) -> StdResult<()> {
    let mut owner_store = PrefixedStorage::multilevel(store, &[PREFIX_ALLOWANCES, owner.as_slice()]);
    owner_store.set(spender.as_slice(), &amount.to_be_bytes());
    Ok(())
}

fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.len() < 3 || bytes.len() > 30 {
        return false;
    }
    true
}

fn is_valid_symbol(symbol: &str) -> bool {
    let bytes = symbol.as_bytes();
    if bytes.len() < 3 || bytes.len() > 6 {
        return false;
    }
    for byte in bytes.iter() {
        if !((*byte >= 65 && *byte <= 90) || (*byte >= 97 && *byte <= 122)) {
            return false;
        }
    }
    true
}
