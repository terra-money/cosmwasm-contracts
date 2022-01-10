use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::VestingSchedule;
use cosmwasm_std::Uint128;
use cw20::Denom;
use cw_storage_plus::Map;

pub const VESTING_ACCOUNTS: Map<(&str, &str), VestingAccount> = Map::new("vesting_accounts");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VestingAccount {
    pub master_address: Option<String>,
    pub address: String,
    pub vesting_denom: Denom,
    pub vesting_amount: Uint128,
    pub vesting_schedule: VestingSchedule,
    pub claimed_amount: Uint128,
}

pub fn denom_to_key(denom: Denom) -> String {
    match denom {
        Denom::Cw20(addr) => format!("cw20-{}", addr.to_string()),
        Denom::Native(denom) => format!("native-{}", denom),
    }
}
