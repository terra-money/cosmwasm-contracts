use cosmwasm_std::{generic_err, to_binary, Coin, Decimal, QuerierResult};
use std::collections::{BTreeMap, HashMap};

use terra_bindings::{SimulateSwapResponse, SwapQuery};

use crate::oracle::rates_to_map;

#[derive(Clone, Default)]
pub struct SwapQuerier {
    rates: HashMap<String, BTreeMap<String, Decimal>>,
}

impl SwapQuerier {
    pub fn new(rates: &[(&str, &str, Decimal)]) -> Self {
        SwapQuerier {
            rates: rates_to_map(rates),
        }
    }

    pub fn query(&self, request: &SwapQuery) -> QuerierResult {
        match request {
            SwapQuery::Simulate { offer, ask } => {
                // proper error on not found, serialize result on found
                let rate = self.rates.get(&offer.denom).and_then(|tree| tree.get(ask));
                let amount = match rate {
                    Some(r) => offer.amount * *r,
                    None => {
                        return Ok(Err(generic_err(format!(
                            "No rate listed for {} to {}",
                            offer.denom, ask,
                        ))))
                    }
                };
                let swap_res = SimulateSwapResponse {
                    receive: Coin {
                        amount,
                        denom: ask.clone(),
                    },
                };
                Ok(to_binary(&swap_res))
            }
        }
    }
}
