use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_slice, CanonicalAddr, Decimal, Order, ReadonlyStorage, StdError, StdResult, Storage,
    Uint128, KV,
};
use cosmwasm_storage::{
    singleton, singleton_read, PrefixedStorage, ReadonlyPrefixedStorage, ReadonlySingleton,
    Singleton,
};

static CONFIG_KEY: &[u8] = b"config";
static ROLL_KEY: &[u8] = b"roll";
static STAKER_KEY: &[u8] = b"staker";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigState {
    pub owner: CanonicalAddr,
    pub staking_token: CanonicalAddr,
    pub roll_unit: Uint128,
    pub deposit_period: u64,
    pub rewards_denom: String,
}

pub fn config_store<S: Storage>(storage: &mut S) -> Singleton<S, ConfigState> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, ConfigState> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RollState {
    pub owner: CanonicalAddr,
    pub creation_time: u64,
}

pub fn roll_store<S: Storage>(storage: &mut S) -> PrefixedStorage<S> {
    PrefixedStorage::new(ROLL_KEY, storage)
}

pub fn roll_read<S: Storage>(
    storage: &S,
    owner: &CanonicalAddr,
    index: u32,
) -> StdResult<RollState> {
    let roll_storage = ReadonlyPrefixedStorage::new(ROLL_KEY, storage);
    let result = roll_storage.get(&[owner.as_slice(), &index.to_be_bytes()].concat());
    match result {
        Some(data) => from_slice(&data),
        None => Err(StdError::generic_err("No roll data stored")),
    }
}

pub fn load_all_rolls<S: Storage>(storage: &S) -> Vec<KV> {
    ReadonlyPrefixedStorage::new(ROLL_KEY, storage)
        .range(None, None, Order::Ascending)
        .collect()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct StakerState {
    pub address: CanonicalAddr,
    pub balance: Uint128,
    pub collected_rewards: Decimal,
}

pub fn staker_store<S: Storage>(storage: &mut S) -> PrefixedStorage<S> {
    PrefixedStorage::new(STAKER_KEY, storage)
}

pub fn staker_read<S: Storage>(storage: &S, address: &CanonicalAddr) -> StdResult<StakerState> {
    let roll_storage = ReadonlyPrefixedStorage::new(STAKER_KEY, storage);
    let result = roll_storage.get(address.as_slice());
    match result {
        Some(data) => from_slice(&data),
        None => Ok(StakerState {
            address: address.clone(),
            balance: Uint128::zero(),
            collected_rewards: Decimal::zero(),
        }),
    }
}
