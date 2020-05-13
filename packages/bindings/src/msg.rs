use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, CosmosMsg, HumanAddr};

/// TerraMsg is an override of CosmosMsg::Custom to add support for Terra's custom message types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TerraMsg {
    Swap(SwapMsg),
}

/// SwapMsg captures all possible messages we can return to terra's native swap module
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum SwapMsg {
    Trade {
        trader_addr: HumanAddr,
        offer_coin: Coin,
        ask_denom: String,
    },
}

// this is a helper to be able to return these as CosmosMsg easier
impl Into<CosmosMsg<TerraMsg>> for TerraMsg {
    fn into(self) -> CosmosMsg<TerraMsg> {
        CosmosMsg::Custom(self)
    }
}

// and another helper, so we can return SwapMsg::Trade{..}.into() as a CosmosMsg
impl Into<CosmosMsg<TerraMsg>> for SwapMsg {
    fn into(self) -> CosmosMsg<TerraMsg> {
        CosmosMsg::Custom(TerraMsg::Swap(self))
    }
}
