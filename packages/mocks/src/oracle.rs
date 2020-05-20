use cosmwasm_std::{Decimal, QuerierResult};
use std::collections::{BTreeMap, HashMap};

use terra_bindings::OracleQuery;

#[derive(Clone, Default)]
pub struct OracleQuerier {
    // this lets us iterate over all pairs that match the first string
    rates: HashMap<String, BTreeMap<String, Decimal>>,
    // this is based only on one token
    taxes: HashMap<String, Decimal>,
}

pub(crate) fn rates_to_map(
    rates: &[(&str, &str, Decimal)],
) -> HashMap<String, BTreeMap<String, Decimal>> {
    let mut rate_map: HashMap<String, BTreeMap<String, Decimal>> = HashMap::new();
    for (offer, ask, rate) in rates.iter() {
        let offer = offer.to_string();
        let ask = ask.to_string();
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

impl OracleQuerier {
    pub fn new(rates: &[(&str, &str, Decimal)], taxes: &[(&str, Decimal)]) -> Self {
        let mut tax_map = HashMap::new();
        for (denom, tax) in taxes.iter() {
            tax_map.insert(denom.to_string(), *tax);
        }

        OracleQuerier {
            rates: rates_to_map(rates),
            taxes: tax_map,
        }
    }

    pub fn query(&self, request: &OracleQuery) -> QuerierResult {
        match request {
            OracleQuery::TobinTax { .. } => panic!("not implemented"),
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
