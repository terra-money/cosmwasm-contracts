use cosmwasm_std::{to_binary, Decimal, QuerierResult, SystemResult, Uint128};
use std::collections::HashMap;

use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery};

#[derive(Clone)]
pub struct TreasuryQuerier {
    tax_rate: Decimal,
    tax_cap: HashMap<String, Uint128>,
}

impl Default for TreasuryQuerier {
    fn default() -> Self {
        TreasuryQuerier {
            tax_rate: Decimal::zero(),
            tax_cap: HashMap::default(),
        }
    }
}

impl TreasuryQuerier {
    pub fn new(tax_rate: Decimal, tax_caps: &[(&str, u128)]) -> Self {
        let mut tax_cap = HashMap::new();
        for (denom, cap) in tax_caps.iter() {
            tax_cap.insert((*denom).to_string(), Uint128::from(*cap));
        }
        TreasuryQuerier { tax_rate, tax_cap }
    }

    pub fn query(&self, request: &TerraQuery) -> QuerierResult {
        match request {
            TerraQuery::TaxRate {} => {
                let res = TaxRateResponse {
                    rate: self.tax_rate,
                };
                SystemResult::Ok(to_binary(&res).into())
            }
            TerraQuery::TaxCap { denom } => {
                let cap = self.tax_cap.get(denom).copied().unwrap_or_default();
                let res = TaxCapResponse { cap };
                SystemResult::Ok(to_binary(&res).into())
            }
            _ => panic!("DO NOT ENTER HERE"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::from_binary;

    #[test]
    fn queries() {
        // set the exchange rates between ETH and BTC (and back)
        let tax_rate = Decimal::percent(2);
        let tax_caps = &[("ETH", 1000u128), ("BTC", 500u128)];

        let querier = TreasuryQuerier::new(tax_rate, tax_caps);

        // test all treasury functions
        let tax_rate_query = TerraQuery::TaxRate {};
        let res = querier.query(&tax_rate_query).unwrap().unwrap();
        let tax_rate_res: TaxRateResponse = from_binary(&res).unwrap();
        assert_eq!(tax_rate_res.rate, tax_rate);

        let tax_cap_query = TerraQuery::TaxCap {
            denom: "ETH".to_string(),
        };
        let res = querier.query(&tax_cap_query).unwrap().unwrap();
        let cap: TaxCapResponse = from_binary(&res).unwrap();
        assert_eq!(cap.cap, Uint128::from(1000u128));
    }
}
