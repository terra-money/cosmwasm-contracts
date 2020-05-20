use cosmwasm_std::{generic_err, to_binary, Coin, Decimal, QuerierResult};
use std::collections::HashMap;

use terra_bindings::{SimulateSwapResponse, SwapQuery};

/// TokenPair is (offer, ask) denoms
pub type TokenPair = (String, String);

#[derive(Clone, Default)]
pub struct SwapQuerier {
    rates: HashMap<TokenPair, Decimal>,
}

impl SwapQuerier {
    pub fn new(rates: &[(TokenPair, Decimal)]) -> Self {
        let mut map = HashMap::new();
        for (pair, rate) in rates.iter() {
            map.insert(pair.clone(), *rate);
        }
        SwapQuerier { rates: map }
    }

    pub fn query(&self, request: &SwapQuery) -> QuerierResult {
        match request {
            SwapQuery::Simulate { offer, ask } => {
                let pair = (offer.denom.clone(), ask.clone());
                // proper error on not found, serialize result on found
                let rate = self.rates.get(&pair);
                let amount = match rate {
                    Some(r) => offer.amount * *r,
                    None => {
                        return Ok(Err(generic_err(format!(
                            "No rate listed for {} to {}",
                            pair.0, pair.1
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
