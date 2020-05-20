use cosmwasm_std::{Coin, Decimal, QuerierResult, Uint128};
use std::collections::HashMap;

use terra_bindings::TreasuryQuery;

#[derive(Clone)]
pub struct TreasuryQuerier {
    tax_rate: Decimal,
    tax_proceeds: Vec<Coin>,
    tax_cap: HashMap<String, Uint128>,
    reward_rate: Decimal,
    seigniorage_proceeds: Uint128,
}

impl TreasuryQuerier {
    pub fn new(
        tax_rate: Decimal,
        tax_proceeds: &[Coin],
        tax_caps: &[(String, Uint128)],
        reward_rate: Decimal,
        seigniorage_proceeds: Uint128,
    ) -> Self {
        let mut tax_cap = HashMap::new();
        for (denom, cap) in tax_caps.iter() {
            tax_cap.insert(denom.to_string(), *cap);
        }
        TreasuryQuerier {
            tax_rate,
            tax_proceeds: tax_proceeds.to_vec(),
            tax_cap,
            reward_rate,
            seigniorage_proceeds,
        }
    }

    pub fn query(&self, request: &TreasuryQuery) -> QuerierResult {
        match request {
            TreasuryQuery::TaxRate { .. } => panic!("not implemented"),
            _ => panic!("not implemented"),
            // SwapQuery::Simulate { offer, ask } => {
            //     let pair = (offer.denom.clone(), ask.clone());
            //     // proper error on not found, serialize result on found
            //     let rate = self.rates.get(&pair);
            //     let amount = match rate {
            //         Some(r) => offer.amount * r.clone(),
            //         None => {
            //             return Ok(Err(generic_err(format!(
            //                 "No rate listed for {} to {}",
            //                 pair.0, pair.1
            //             ))))
            //         }
            //     };
            //     let swap_res = SimulateSwapResponse {
            //         receive: Coin {
            //             amount,
            //             denom: ask.clone(),
            //         },
            //     };
            //     Ok(to_binary(&swap_res))
            // }
        }
    }
}
