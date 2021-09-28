use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub legacy_token: String,
    pub target_token: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),

    ////////////////////////
    /// Owner Operations ///
    ////////////////////////
    Enable {},
    Disable {},
    Withdraw {
        recipient: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    // Swap legacy token to target token
    Swap { recipient: Option<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Balances {},
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct ConfigResponse {
    pub owner: String,
    pub legacy_token: String,
    pub target_token: String,
    pub swap_enabled: bool,
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct BalancesResponse {
    pub legacy_balance: Uint128,
    pub target_balance: Uint128,
}
