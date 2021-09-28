use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cw20::{BalanceResponse, Cw20QueryMsg};
use std::collections::HashMap;

/// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new(MockQuerier::<Empty>::new(&[(
        MOCK_CONTRACT_ADDR,
        contract_balance,
    )]));

    OwnedDeps {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier,
    token_querier: TokenQuerier,
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(String, &[(String, Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(
    balances: &[(String, &[(String, Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut contract_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut balance_map: HashMap<String, Uint128> = HashMap::new();
        for (acc_addr, balance) in balances.iter() {
            balance_map.insert(acc_addr.clone(), *balance);
        }

        contract_map.insert(contract_addr.clone(), balance_map);
    }
    contract_map
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match from_binary(msg).unwrap() {
                    Cw20QueryMsg::Balance { address } => {
                        match self.token_querier.balances.get(contract_addr) {
                            Some(balances_map) => match balances_map.get(&address) {
                                Some(balance) => SystemResult::Ok(ContractResult::from(to_binary(
                                    &BalanceResponse { balance: *balance },
                                ))),
                                None => SystemResult::Err(SystemError::InvalidRequest {
                                    error: "No account found in token balance map".to_string(),
                                    request: msg.as_slice().into(),
                                }),
                            },
                            None => SystemResult::Err(SystemError::InvalidRequest {
                                error: "No token contract found in balance map".to_string(),
                                request: msg.as_slice().into(),
                            }),
                        }
                    }
                    _ => SystemResult::Err(SystemError::InvalidRequest {
                        error: "Not supported query".to_string(),
                        request: msg.as_slice().into(),
                    }),
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
        }
    }

    // configure the token owner mock querier
    pub fn with_token_balances(&mut self, balances: &[(String, &[(String, Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }
}
