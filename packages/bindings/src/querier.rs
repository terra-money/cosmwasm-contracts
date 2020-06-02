use cosmwasm_std::{Coin, Decimal, Querier, StdResult, Uint128};

use crate::query::{
    ExchangeRateResponse, ExchangeRatesResponse, RewardsWeightResponse,
    SeigniorageProceedsResponse, SwapResponse, TaxCapResponse, TaxProceedsResponse,
    TaxRateResponse, TerraQuery, TerraQueryWrapper, TobinTaxResponse,
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
        let request = TerraQueryWrapper {
            route: "oracle".to_string(),
            query_data: TerraQuery::ExchangeRate {
                offer: offer.into(),
                ask: ask.into(),
            },
        };
        let res: ExchangeRateResponse = self.querier.custom_query(&request.into())?;
        Ok(res.rate)
    }

    pub fn query_exchange_rates<T: Into<String>>(
        &self,
        offer: T,
    ) -> StdResult<Vec<ExchangeRateResponse>> {
        let request = TerraQueryWrapper {
            route: "oracle".to_string(),
            query_data: TerraQuery::ExchangeRates {
                offer: offer.into(),
            },
        };
        let res: ExchangeRatesResponse = self.querier.custom_query(&request.into())?;
        Ok(res.rates)
    }

    pub fn query_reward_weight(&self) -> StdResult<Decimal> {
        let request = TerraQueryWrapper {
            route: "treasury".to_string(),
            query_data: TerraQuery::RewardsWeight {},
        };
        let res: RewardsWeightResponse = self.querier.custom_query(&request.into())?;
        Ok(res.weight)
    }

    pub fn query_seigniorage_proceeds(&self) -> StdResult<Uint128> {
        let request = TerraQueryWrapper {
            route: "treasury".to_string(),
            query_data: TerraQuery::SeigniorageProceeds {},
        };
        let res: SeigniorageProceedsResponse = self.querier.custom_query(&request.into())?;
        Ok(res.size)
    }

    pub fn query_swap<T: Into<String>>(&self, offer_coin: Coin, ask_denom: T) -> StdResult<Coin> {
        let request = TerraQueryWrapper {
            route: "market".to_string(),
            query_data: TerraQuery::Swap {
                offer_coin,
                ask_denom: ask_denom.into(),
            },
        };
        let res: SwapResponse = self.querier.custom_query(&request.into())?;
        Ok(res.receive)
    }

    pub fn query_tax_cap<T: Into<String>>(&self, denom: T) -> StdResult<Uint128> {
        let request = TerraQueryWrapper {
            route: "treasury".to_string(),
            query_data: TerraQuery::TaxCap {
                denom: denom.into(),
            },
        };
        let res: TaxCapResponse = self.querier.custom_query(&request.into())?;
        Ok(res.cap)
    }

    pub fn query_tax_proceeds(&self) -> StdResult<Vec<Coin>> {
        let request = TerraQueryWrapper {
            route: "treasury".to_string(),
            query_data: TerraQuery::TaxProceeds {},
        };
        let res: TaxProceedsResponse = self.querier.custom_query(&request.into())?;
        Ok(res.proceeds)
    }

    pub fn query_tax_rate(&self) -> StdResult<Decimal> {
        let request = TerraQueryWrapper {
            route: "treasury".to_string(),
            query_data: TerraQuery::TaxRate {},
        };
        let res: TaxRateResponse = self.querier.custom_query(&request.into())?;
        Ok(res.rate)
    }

    pub fn query_tobin_tax<T: Into<String>>(&self, denom: T) -> StdResult<Decimal> {
        let request = TerraQueryWrapper {
            route: "oracle".to_string(),
            query_data: TerraQuery::TobinTax {
                denom: denom.into(),
            },
        };
        let res: TobinTaxResponse = self.querier.custom_query(&request.into())?;
        Ok(res.rate)
    }
}
