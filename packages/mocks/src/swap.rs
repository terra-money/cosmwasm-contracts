use cosmwasm_std::{to_binary, Coin, Decimal, QuerierResult, SystemError, SystemResult};
use std::collections::{BTreeMap, HashMap};

use terra_cosmwasm::{SwapResponse, TerraQuery};

// use crate::oracle::rates_to_map;

#[derive(Clone, Default)]
pub struct SwapQuerier {
    rates: HashMap<String, BTreeMap<String, Decimal>>,
}

pub(crate) fn rates_to_map(
    rates: &[(&str, &str, Decimal)],
) -> HashMap<String, BTreeMap<String, Decimal>> {
    let mut rate_map: HashMap<String, BTreeMap<String, Decimal>> = HashMap::new();
    for (offer, ask, rate) in rates.iter() {
        let offer = (*offer).to_string();
        let ask = (*ask).to_string();
        if let Some(sub_map) = rate_map.get_mut(&offer) {
            sub_map.insert(ask, *rate);
        } else {
            let mut sub_map = BTreeMap::new();
            sub_map.insert(ask, *rate);
            rate_map.insert(offer, sub_map);
        }
    }
    rate_map
}

impl SwapQuerier {
    pub fn new(rates: &[(&str, &str, Decimal)]) -> Self {
        SwapQuerier {
            rates: rates_to_map(rates),
        }
    }

    pub fn query(&self, request: &TerraQuery) -> QuerierResult {
        match request {
            TerraQuery::Swap {
                offer_coin,
                ask_denom,
            } => {
                // proper error on not found, serialize result on found
                let rate = self
                    .rates
                    .get(&offer_coin.denom)
                    .and_then(|tree| tree.get(ask_denom));
                let amount = match rate {
                    Some(r) => offer_coin.amount * *r,
                    None => {
                        return SystemResult::Err(SystemError::InvalidRequest {
                            error: format!(
                                "No rate listed for {} to {}",
                                offer_coin.denom, ask_denom,
                            ),
                            request: to_binary(request).unwrap(),
                        })
                    }
                };
                let swap_res = SwapResponse {
                    receive: Coin {
                        amount,
                        denom: ask_denom.clone(),
                    },
                };
                SystemResult::Ok(to_binary(&swap_res).into())
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
    fn forward_swap() {
        let eth2btc = Decimal::percent(15);
        let btc2eth = Decimal::percent(666);

        let querier = SwapQuerier::new(&[("ETH", "BTC", eth2btc), ("BTC", "ETH", btc2eth)]);

        // test forward swap
        let forward_query = TerraQuery::Swap {
            offer_coin: coin(100, "ETH"),
            ask_denom: "BTC".to_string(),
        };
        let res = querier.query(&forward_query).unwrap().unwrap();
        let sim: SwapResponse = from_binary(&res).unwrap();
        assert_eq!(sim.receive, coin(15, "BTC"));
    }

    #[test]
    fn reverse_swap() {
        let eth2btc = Decimal::percent(15);
        let btc2eth = Decimal::percent(666);

        let querier = SwapQuerier::new(&[("ETH", "BTC", eth2btc), ("BTC", "ETH", btc2eth)]);

        // test forward swap
        let forward_query = TerraQuery::Swap {
            offer_coin: coin(50, "BTC"),
            ask_denom: "ETH".to_string(),
        };
        let res = querier.query(&forward_query).unwrap().unwrap();
        let sim: SwapResponse = from_binary(&res).unwrap();
        assert_eq!(sim.receive, coin(333, "ETH"));
    }

    #[test]
    fn unlisted_pair() {
        let eth2btc = Decimal::percent(15);
        let btc2eth = Decimal::percent(666);

        let querier = SwapQuerier::new(&[("ETH", "BTC", eth2btc), ("BTC", "ETH", btc2eth)]);

        // test forward swap
        let forward_query = TerraQuery::Swap {
            offer_coin: coin(100, "ETH"),
            ask_denom: "ATOM".to_string(),
        };
        let res = querier.query(&forward_query);
        match res.unwrap_err() {
            SystemError::InvalidRequest { error, .. } => {
                assert_eq!(error, "No rate listed for ETH to ATOM")
            }
            _ => panic!("unexpected error"),
        }
    }
}
