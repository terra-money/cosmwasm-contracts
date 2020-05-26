use cosmwasm_std::{to_binary, Coin, Decimal, QuerierResult, Uint128};
use std::collections::HashMap;

use terra_bindings::{
    RewardsWeightResponse, SeigniorageProceedsResponse, TaxCapResponse, TaxProceedsResponse,
    TaxRateResponse, TerraQuery,
};

#[derive(Clone)]
pub struct TreasuryQuerier {
    tax_rate: Decimal,
    tax_proceeds: Vec<Coin>,
    tax_cap: HashMap<String, Uint128>,
    reward_rate: Decimal,
    seigniorage_proceeds: Uint128,
}

impl Default for TreasuryQuerier {
    fn default() -> Self {
        TreasuryQuerier {
            tax_rate: Decimal::zero(),
            tax_proceeds: vec![],
            tax_cap: HashMap::default(),
            reward_rate: Decimal::zero(),
            seigniorage_proceeds: Uint128::zero(),
        }
    }
}

impl TreasuryQuerier {
    pub fn new(
        tax_rate: Decimal,
        tax_proceeds: &[Coin],
        tax_caps: &[(&str, u128)],
        reward_rate: Decimal,
        seigniorage_proceeds: u128,
    ) -> Self {
        let mut tax_cap = HashMap::new();
        for (denom, cap) in tax_caps.iter() {
            tax_cap.insert(denom.to_string(), Uint128(*cap));
        }
        TreasuryQuerier {
            tax_rate,
            tax_proceeds: tax_proceeds.to_vec(),
            tax_cap,
            reward_rate,
            seigniorage_proceeds: Uint128(seigniorage_proceeds),
        }
    }

    pub fn query(&self, request: &TerraQuery) -> QuerierResult {
        match request {
            TerraQuery::TaxRate {} => {
                let res = TaxRateResponse { tax: self.tax_rate };
                Ok(to_binary(&res))
            }
            TerraQuery::TaxCap { denom } => {
                let cap = self.tax_cap.get(denom).copied().unwrap_or_default();
                let res = TaxCapResponse { cap };
                Ok(to_binary(&res))
            }
            TerraQuery::TaxProceeds {} => {
                let res = TaxProceedsResponse {
                    proceeds: self.tax_proceeds.clone(),
                };
                Ok(to_binary(&res))
            }
            TerraQuery::RewardsWeight {} => {
                let res = RewardsWeightResponse {
                    weight: self.reward_rate,
                };
                Ok(to_binary(&res))
            }
            TerraQuery::SeigniorageProceeds {} => {
                let res = SeigniorageProceedsResponse {
                    size: self.seigniorage_proceeds,
                };
                Ok(to_binary(&res))
            }
            _ => panic!("DO NOT ENTER HERE"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{coin, from_binary};

    #[test]
    fn queries() {
        // set the exchange rates between ETH and BTC (and back)
        let tax_rate = Decimal::percent(2);
        let tax_proceeds = vec![coin(10, "ETH"), coin(20, "BTC")];
        let tax_caps = &[("ETH", 1000u128), ("BTC", 500u128)];
        let reward = Decimal::permille(5);
        let seignorage = 777;

        let querier = TreasuryQuerier::new(tax_rate, &tax_proceeds, tax_caps, reward, seignorage);

        // test all treasury functions
        let tax_rate_query = TerraQuery::TaxRate {};
        let res = querier.query(&tax_rate_query).unwrap().unwrap();
        let rate: TaxRateResponse = from_binary(&res).unwrap();
        assert_eq!(rate.tax, tax_rate);

        let tax_cap_query = TerraQuery::TaxCap {
            denom: "ETH".to_string(),
        };
        let res = querier.query(&tax_cap_query).unwrap().unwrap();
        let cap: TaxCapResponse = from_binary(&res).unwrap();
        assert_eq!(cap.cap, Uint128(1000));

        let tax_proceeds_query = TerraQuery::TaxProceeds {};
        let res = querier.query(&tax_proceeds_query).unwrap().unwrap();
        let proceeds: TaxProceedsResponse = from_binary(&res).unwrap();
        assert_eq!(proceeds.proceeds, tax_proceeds);

        let rewards_query = TerraQuery::RewardsWeight {};
        let res = querier.query(&rewards_query).unwrap().unwrap();
        let rewards: RewardsWeightResponse = from_binary(&res).unwrap();
        assert_eq!(rewards.weight, reward);

        let seigniorage_query = TerraQuery::SeigniorageProceeds {};
        let res = querier.query(&seigniorage_query).unwrap().unwrap();
        let proceeds: SeigniorageProceedsResponse = from_binary(&res).unwrap();
        assert_eq!(proceeds.size, Uint128(seignorage));
    }
}
