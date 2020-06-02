use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, Decimal, QueryRequest, Uint128};

/// TerraQueryWrapper is an override of QueryRequest::Custom to access Terra-specific modules
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TerraQueryWrapper {
    pub route: String,
    pub query_data: TerraQuery,
}

/// TerraQuery is defines avaliable query datas
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TerraQuery {
    Swap { offer_coin: Coin, ask_denom: String },
    ExchangeRate { offer: String, ask: String },
    ExchangeRates { offer: String },
    TobinTax { denom: String },
    TaxRate {},
    TaxProceeds {},
    TaxCap { denom: String },
    RewardsWeight {},
    SeigniorageProceeds {},
}

// This is a simpler way to making queries
impl Into<QueryRequest<TerraQueryWrapper>> for TerraQueryWrapper {
    fn into(self) -> QueryRequest<TerraQueryWrapper> {
        QueryRequest::Custom(self)
    }
}

/// SwapResponse is data format returned from SwapRequest::Simulate query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SwapResponse {
    pub receive: Coin,
}

/// ExchangeRateResponse is data format returned from OracleRequest::ExchangeRate query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRateResponse {
    pub ask: String,
    pub rate: Decimal,
}

/// ExchangeRatesResponse is data format returned from OracleRequest::ExchangeRates query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeRatesResponse {
    pub rates: Vec<ExchangeRateResponse>,
}

/// TobinTaxResponse is data format returned from OracleRequest::TobinTax query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TobinTaxResponse {
    pub rate: Decimal,
}

/// TaxRateResponse is data format returned from TreasuryRequest::TaxRate query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaxRateResponse {
    pub rate: Decimal,
}

/// TaxProceedsResponse is data format returned from TreasuryRequest::TaxProceeds query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaxProceedsResponse {
    pub proceeds: Vec<Coin>,
}

/// TaxCapResponse is data format returned from TreasuryRequest::TaxCap query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaxCapResponse {
    // TODO: verify
    pub cap: Uint128,
}

/// RewardsWeightResponse is data format returned from TreasuryRequest::RewardsWeight query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardsWeightResponse {
    pub weight: Decimal,
}

/// SeigniorageProceedsResponse is data format returned from TreasuryRequest::SeigniorageProceeds query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SeigniorageProceedsResponse {
    // TODO: verify what this is
    pub size: Uint128,
}
