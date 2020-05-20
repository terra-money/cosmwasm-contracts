use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, Decimal, Querier, QueryRequest, StdResult, Uint128};

/// TerraQuery is an override of QueryRequest::Custom to access Terra-specific modules
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TerraQuery {
    Swap(SwapQuery),
    Oracle(OracleQuery),
    Treasury(TreasuryQuery),
}

// This is a simpler way to making queries
impl Into<QueryRequest<TerraQuery>> for TerraQuery {
    fn into(self) -> QueryRequest<TerraQuery> {
        QueryRequest::Custom(self)
    }
}

/// This contains all queries that can be made to the swap module
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwapQuery {
    // Delegations will return all delegations by the delegator,
    // or just those to the given validator (if set)
    Simulate { offer: Coin, ask: String },
}

// This is a simpler way to making queries
impl Into<QueryRequest<TerraQuery>> for SwapQuery {
    fn into(self) -> QueryRequest<TerraQuery> {
        QueryRequest::Custom(TerraQuery::Swap(self))
    }
}

/// SimulateSwapResponse is data format returned from SwapRequest::Simulate query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulateSwapResponse {
    pub receive: Coin,
}

/// This contains all queries that can be made to the oracle module
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OracleQuery {
    // ExchangeRate will return the rate between just this pair.
    ExchangeRate { offer: String, ask: String },
    // ExchangeRates will return the exchange rate between offer denom and all supported asks
    ExchangeRates { offer: String },
    // Return the tobin tax charged on exchanges with this token
    // (TODO: define if this applies to the offer or the ask?)
    TobinTax { denom: String},
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
    pub tax: Decimal,
}

// This is a simpler way to making queries
impl Into<QueryRequest<TerraQuery>> for OracleQuery {
    fn into(self) -> QueryRequest<TerraQuery> {
        QueryRequest::Custom(TerraQuery::Oracle(self))
    }
}

/// This contains all queries that can be made to the treasury module
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TreasuryQuery {
    TaxRate {},
    TaxProceeds{},
    // TODO: review
    TaxCap { denom: String },
    RewardWeight {},
    SeigniorageProceeds {},
}

/// TaxRateResponse is data format returned from TreasuryRequest::TaxRate query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaxRateResponse {
    pub tax: Decimal,
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

/***** TODO move to traits *********/

/// This is a helper wrapper to easily use our custom queries
pub struct TerraQuerier<'a, Q: Querier> {
    querier: &'a Q,
}

impl<'a, Q: Querier> TerraQuerier<'a, Q> {
    pub fn new(querier: &'a Q) -> Self {
        TerraQuerier { querier }
    }

    pub fn query_exchange_rate<T: Into<String>>(&self, offer: T, ask: T) -> StdResult<Decimal> {
        let request = OracleQuery::ExchangeRate {
            offer: offer.into(),
            ask: ask.into(),
        };
        let res: ExchangeRateResponse = self.querier.custom_query(&request.into())?;
        Ok(res.rate)
    }

    pub fn query_exchange_rates<T: Into<String>>(
        &self,
        offer: T,
    ) -> StdResult<Vec<ExchangeRateResponse>> {
        let request = OracleQuery::ExchangeRates {
            offer: offer.into(),
        };
        let res: ExchangeRatesResponse = self.querier.custom_query(&request.into())?;
        Ok(res.rates)
    }

    pub fn query_simulate_swap<T: Into<String>>(&self, offer: Coin, ask: T) -> StdResult<Coin> {
        let request = SwapQuery::Simulate {
            offer,
            ask: ask.into(),
        };
        let res: SimulateSwapResponse = self.querier.custom_query(&request.into())?;
        Ok(res.receive)
    }
}
