use cosmwasm_std::{Coin, Decimal, Querier, StdResult, Uint128};

use crate::query::{
    ExchangeRateResponse, ExchangeRatesResponse, OracleQuery, RewardsWeightResponse,
    SeigniorageProceedsResponse, SimulateSwapResponse, SwapQuery, TaxCapResponse,
    TaxProceedsResponse, TaxRateResponse, TobinTaxResponse, TreasuryQuery,
};

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

    pub fn query_reward_weight(&self) -> StdResult<Decimal> {
        let request = TreasuryQuery::RewardsWeight {};
        let res: RewardsWeightResponse = self.querier.custom_query(&request.into())?;
        Ok(res.weight)
    }

    pub fn query_seigniorage_proceeds(&self) -> StdResult<Uint128> {
        let request = TreasuryQuery::SeigniorageProceeds {};
        let res: SeigniorageProceedsResponse = self.querier.custom_query(&request.into())?;
        Ok(res.size)
    }

    pub fn query_simulate_swap<T: Into<String>>(&self, offer: Coin, ask: T) -> StdResult<Coin> {
        let request = SwapQuery::Simulate {
            offer,
            ask: ask.into(),
        };
        let res: SimulateSwapResponse = self.querier.custom_query(&request.into())?;
        Ok(res.receive)
    }

    pub fn query_tax_cap<T: Into<String>>(&self, denom: T) -> StdResult<Uint128> {
        let request = TreasuryQuery::TaxCap {
            denom: denom.into(),
        };
        let res: TaxCapResponse = self.querier.custom_query(&request.into())?;
        Ok(res.cap)
    }

    pub fn query_tax_proceeds(&self) -> StdResult<Vec<Coin>> {
        let request = TreasuryQuery::TaxProceeds {};
        let res: TaxProceedsResponse = self.querier.custom_query(&request.into())?;
        Ok(res.proceeds)
    }

    pub fn query_tax_rate(&self) -> StdResult<Decimal> {
        let request = TreasuryQuery::TaxRate {};
        let res: TaxRateResponse = self.querier.custom_query(&request.into())?;
        Ok(res.tax)
    }

    pub fn query_tobin_tax<T: Into<String>>(&self, denom: T) -> StdResult<Decimal> {
        let request = OracleQuery::TobinTax {
            denom: denom.into(),
        };
        let res: TobinTaxResponse = self.querier.custom_query(&request.into())?;
        Ok(res.tax)
    }
}
