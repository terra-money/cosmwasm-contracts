use crate::state::{StakingInfo, VestingSchedule};
use cosmwasm_std::Uint128;
use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub owner_address: String,
    pub enable_staking: bool,
    pub staking_info: Option<StakingInfo>,
    pub vesting_schedule: VestingSchedule,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ChangeOwner { new_owner: String },
    Claim { recipient: Option<String> },
    ClaimRewards { recipient: Option<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    VestingInfo {},
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct VestingInfoResponse {
    pub owner_address: String,
    pub vesting_denom: Denom,
    pub vesting_amount: Uint128,
    pub vested_amount: Uint128,
    pub vesting_schedule: VestingSchedule,
    pub claimable_amount: Uint128,
    pub claimable_staking_rewards: Uint128,
}
