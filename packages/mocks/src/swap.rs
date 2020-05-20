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

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::{coin, from_binary, StdError};

    #[test]
    fn forward_swap() {
        let eth2btc = Decimal::percent(15);
        let btc2eth = Decimal::percent(666);

        let querier = SwapQuerier::new(&[("ETH", "BTC", eth2btc), ("BTC", "ETH", btc2eth)]);

        // test forward swap
        let forward_query = SwapQuery::Simulate {
            offer: coin(100, "ETH"),
            ask: "BTC".to_string(),
        };
        let res = querier.query(&forward_query).unwrap().unwrap();
        let sim: SimulateSwapResponse = from_binary(&res).unwrap();
        assert_eq!(sim.receive, coin(15, "BTC"));
    }

    #[test]
    fn reverse_swap() {
        let eth2btc = Decimal::percent(15);
        let btc2eth = Decimal::percent(666);

        let querier = SwapQuerier::new(&[("ETH", "BTC", eth2btc), ("BTC", "ETH", btc2eth)]);

        // test forward swap
        let forward_query = SwapQuery::Simulate {
            offer: coin(50, "BTC"),
            ask: "ETH".to_string(),
        };
        let res = querier.query(&forward_query).unwrap().unwrap();
        let sim: SimulateSwapResponse = from_binary(&res).unwrap();
        assert_eq!(sim.receive, coin(333, "ETH"));
    }

    #[test]
    fn unlisted_pair() {
        let eth2btc = Decimal::percent(15);
        let btc2eth = Decimal::percent(666);

        let querier = SwapQuerier::new(&[("ETH", "BTC", eth2btc), ("BTC", "ETH", btc2eth)]);

        // test forward swap
        let forward_query = SwapQuery::Simulate {
            offer: coin(100, "ETH"),
            ask: "ATOM".to_string(),
        };
        let res = querier.query(&forward_query).unwrap();
        match res.unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "No rate listed for ETH to ATOM"),
            _ => panic!("unexpected error"),
        }
    }
}
