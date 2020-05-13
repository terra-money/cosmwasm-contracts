use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, Decimal, QueryRequest};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// TerraQuery is an override of QueryRequest::Custom to access Terra-specific modules
pub enum TerraQuery {
    Swap(SwapQuery),
    // TODO: add for treasury and oracle
}

/// This contains all queries that can be made to the swap module
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwapQuery {
    // ExchangeRate will return the rate between just this pair.
    ExchangeRate { offer: String, ask: String },
    // ExchangeRates will return the exchange rate between offer denom and all supported asks
    ExchangeRates { offer: String },
    // Delegations will return all delegations by the delegator,
    // or just those to the given validator (if set)
    Simulate { offer: Coin, ask: String },
}

// This is a simpler way to making queries
impl Into<QueryRequest<TerraQuery>> for TerraQuery {
    fn into(self) -> QueryRequest<TerraQuery> {
        QueryRequest::Custom(self)
    }
}

// This is a simpler way to making queries
impl Into<QueryRequest<TerraQuery>> for SwapQuery {
    fn into(self) -> QueryRequest<TerraQuery> {
        QueryRequest::Custom(TerraQuery::Swap(self))
    }
}

/// ExchangeRateResponse is data format returned from SwapRequest::ExchangeRate query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRateResponse {
    pub ask: String,
    pub rate: Decimal,
}

/// ExchangeRatesResponse is data format returned from SwapRequest::ExchangeRates query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRatesResponse {
    pub rates: Vec<ExchangeRateResponse>,
}

/// SimulateSwapResponse is data format returned from SwapRequest::Simulate query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulateSwapResponse {
    pub receive: Coin,
}
