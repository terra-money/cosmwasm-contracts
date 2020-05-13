use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, Decimal, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub offer: String,
    pub ask: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// Buy will try to buy ask and sell offer, up to limit offer tokens, or current balance
    Buy { limit: Option<Uint128> },
    /// Sell is the reverse of buy. Selling ask and buying offer.
    Sell { limit: Option<Uint128> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Exchange rate returns how many ASK we can get for 1 OFFER
    ExchangeRate {},
    // Simulate will try to sell the given number of tokens (denom must be either ask or offer, we trade for the other)
    Simulate { offer: Coin },
}

/// Returns rate of ASK/OFFER
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRateResponse {
    pub rate: Decimal,
    pub ask: String,
    pub offer: String,
}

/// Returns how many coins we could BUY if we SELL the given amount
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulateResponse {
    pub sell: Coin,
    pub buy: Coin,
}
