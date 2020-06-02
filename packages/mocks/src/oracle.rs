use cosmwasm_std::{generic_err, to_binary, Decimal, QuerierResult};
use std::collections::{BTreeMap, HashMap};

use terra_bindings::{ExchangeRateResponse, ExchangeRatesResponse, TerraQuery, TobinTaxResponse};

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

impl OracleQuerier {
    pub fn new(rates: &[(&str, &str, Decimal)], taxes: &[(&str, Decimal)]) -> Self {
        let mut tax_map = HashMap::new();
        for (denom, tax) in taxes.iter() {
            tax_map.insert((*denom).to_string(), *tax);
        }

        OracleQuerier {
            rates: rates_to_map(rates),
            taxes: tax_map,
        }
    }

    pub fn query(&self, request: &TerraQuery) -> QuerierResult {
        match request {
            TerraQuery::ExchangeRate { offer, ask } => {
                // proper error on not found, serialize result on found
                let rate = self.rates.get(offer).and_then(|tree| tree.get(ask));
                if rate.is_none() {
                    return Ok(Err(generic_err(format!(
                        "No rate listed for {} to {}",
                        offer, ask
                    ))));
                }
                let oracle_res = ExchangeRateResponse {
                    ask: ask.to_string(),
                    rate: *rate.unwrap(),
                };
                Ok(to_binary(&oracle_res))
            }
            TerraQuery::ExchangeRates { offer } => {
                // proper error on not found, serialize result on found
                let stored = self.rates.get(offer);
                let rates = match stored {
                    Some(tree) => tree
                        .iter()
                        .map(|(ask, r)| ExchangeRateResponse {
                            ask: ask.to_string(),
                            rate: *r,
                        })
                        .collect(),
                    None => vec![],
                };
                let oracle_res = ExchangeRatesResponse { rates };
                Ok(to_binary(&oracle_res))
            }
            TerraQuery::TobinTax { denom } => {
                // proper error on not found, serialize result on found
                let rate = *self.taxes.get(denom).unwrap_or(&Decimal::zero());
                let oracle_res = TobinTaxResponse { rate };
                Ok(to_binary(&oracle_res))
            }
            _ => panic!("DO NOT ENTER HERE"),
        }
    }
}
