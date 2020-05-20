use cosmwasm_std::{to_binary, Coin, Decimal, QuerierResult, Uint128};
use std::collections::HashMap;

use terra_bindings::{
    RewardsWeightResponse, SeigniorageProceedsResponse, TaxCapResponse, TaxProceedsResponse,
    TaxRateResponse, TreasuryQuery,
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

    pub fn query(&self, request: &TreasuryQuery) -> QuerierResult {
        match request {
            TreasuryQuery::TaxRate {} => {
                let res = TaxRateResponse { tax: self.tax_rate };
                Ok(to_binary(&res))
            }
            TreasuryQuery::TaxCap { denom } => {
                let cap = self.tax_cap.get(denom).copied().unwrap_or_default();
                let res = TaxCapResponse { cap };
                Ok(to_binary(&res))
            }
            TreasuryQuery::TaxProceeds {} => {
                let res = TaxProceedsResponse {
                    proceeds: self.tax_proceeds.clone(),
                };
                Ok(to_binary(&res))
            }
            TreasuryQuery::RewardsWeight {} => {
                let res = RewardsWeightResponse {
                    weight: self.reward_rate,
                };
                Ok(to_binary(&res))
            }
            TreasuryQuery::SeigniorageProceeds {} => {
                let res = SeigniorageProceedsResponse {
                    size: self.seigniorage_proceeds,
                };
                Ok(to_binary(&res))
            }
        }
    }
}
