use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, HumanAddr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub staking_token: HumanAddr,
    pub roll_unit: Uint128,
    pub deposit_period: Uint128,
    pub rewards_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        owner: Option<HumanAddr>,
        deposit_period: Option<Uint128>,
    },
    Deposit {
        amount: Uint128,
    },
    Withdraw {
        amount: Uint128,
    },
    Claim {},
    Distribute {
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Staker { address: HumanAddr },
    Roll { address: HumanAddr, index: u32 },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: HumanAddr,
    pub staking_token: HumanAddr,
    pub roll_unit: Uint128,
    pub deposit_period: u64,
    pub rewards_denom: String,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerResponse {
    pub address: HumanAddr,
    pub balance: Uint128,
    pub collected_rewards: Decimal,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RollResponse {
    pub owner: HumanAddr,
    pub creation_time: u64,
}

///////////////////////////////////////////////
/// Token Contract
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenHandleMsg {
    Transfer {
        recipient: HumanAddr,
        amount: Uint128,
    },
    TransferFrom {
        owner: HumanAddr,
        recipient: HumanAddr,
        amount: Uint128,
    },
}
