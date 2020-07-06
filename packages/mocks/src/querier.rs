use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, Coin, Decimal, Extern, FullDelegation, HumanAddr, Querier, QuerierResult,
    QueryRequest, SystemError, Validator,
};

use crate::{SwapQuerier, TreasuryQuerier};
use terra_bindings::TerraQueryWrapper;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    canonical_length: usize,
    contract_balance: &[Coin],
) -> Extern<MockStorage, MockApi, TerraMockQuerier> {
    let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
    let custom_querier: TerraMockQuerier = TerraMockQuerier::new(
        MockQuerier::new(&[(&contract_addr, contract_balance)]));

    Extern {
        storage: MockStorage::default(),
        api: MockApi::new(canonical_length),
        querier: custom_querier,
    }
}

pub struct TerraMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    swap: SwapQuerier,
    treasury: TreasuryQuerier,
}

impl Querier for TerraMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl TerraMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                let route_str = route.as_str();
                if route_str == "market" {
                    self.swap.query(query_data)
                } else if route_str == "treasury" {
                    self.treasury.query(query_data)
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl TerraMockQuerier {
    pub fn new(base: MockQuerier<TerraQueryWrapper>) -> Self {
        TerraMockQuerier {
            base,
            swap: SwapQuerier::default(),
            treasury: TreasuryQuerier::default(),
        }
    }

    // set a new balance for the given address and return the old balance
    pub fn update_balance<U: Into<HumanAddr>>(
        &mut self,
        addr: U,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        self.base.update_balance(addr, balance)
    }

    // configure the stacking mock querier
    pub fn with_staking(
        &mut self,
        denom: &str,
        validators: &[Validator],
        delegations: &[FullDelegation],
    ) {
        self.base.update_staking(denom, validators, delegations)
    }

    pub fn with_market(&mut self, rates: &[(&str, &str, Decimal)]) {
        self.swap = SwapQuerier::new(rates);
    }

    pub fn with_treasury(
        &mut self,
        tax_rate: Decimal,
        tax_caps: &[(&str, u128)],
    ) {
        self.treasury = TreasuryQuerier::new(
            tax_rate,
            tax_caps,
        );
    }
}
