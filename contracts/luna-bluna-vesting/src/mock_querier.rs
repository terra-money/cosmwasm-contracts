use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Coin, ContractResult, Empty, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, Uint128, WasmQuery,
};
use cw20::BalanceResponse;
use std::collections::HashMap;

use crate::external::handle::AccruedRewardsResponse;

/// mock_dependencies_with_querier is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
/// this uses our CustomQuerier.
pub fn mock_dependencies_with_querier(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]));

    OwnedDeps {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
    rewards_querier: RewardsQuerier,
    balances_querier: BalancesQuerier,
}

#[derive(Clone, Default)]
pub struct RewardsQuerier {
    accrued_rewards: HashMap<String, Uint128>,
}

impl RewardsQuerier {
    pub fn new(accrued_rewards: &[(&String, &Uint128)]) -> Self {
        RewardsQuerier {
            accrued_rewards: balances_to_map(accrued_rewards),
        }
    }
}

#[derive(Clone, Default)]
pub struct BalancesQuerier {
    balances: HashMap<String, Uint128>,
}

impl BalancesQuerier {
    pub fn new(balances: &[(&String, &Uint128)]) -> Self {
        BalancesQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn balances_to_map(rewards: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut rewards_map: HashMap<String, Uint128> = HashMap::new();
    for (key, reward) in rewards.iter() {
        rewards_map.insert(key.to_string(), (*reward).clone());
    }
    rewards_map
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WasmQueryMsg {
    /// Request bAsset reward amount
    AccruedRewards { address: String },
    /// Request cw20 token balance
    Balance { address: String },
}

impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr: _, msg }) => {
                match from_binary(&msg).unwrap() {
                    WasmQueryMsg::AccruedRewards { address } => {
                        match self.rewards_querier.accrued_rewards.get(&address) {
                            Some(v) => SystemResult::Ok(ContractResult::Ok(
                                to_binary(&AccruedRewardsResponse { rewards: v.clone() }).unwrap(),
                            )),
                            None => SystemResult::Ok(ContractResult::Ok(
                                to_binary(&AccruedRewardsResponse {
                                    rewards: Uint128::zero(),
                                })
                                .unwrap(),
                            )),
                        }
                    }
                    WasmQueryMsg::Balance { address } => {
                        match self.balances_querier.balances.get(&address) {
                            Some(v) => SystemResult::Ok(ContractResult::Ok(
                                to_binary(&BalanceResponse { balance: v.clone() }).unwrap(),
                            )),
                            None => SystemResult::Ok(ContractResult::Ok(
                                to_binary(&BalanceResponse {
                                    balance: Uint128::zero(),
                                })
                                .unwrap(),
                            )),
                        }
                    }
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
            rewards_querier: RewardsQuerier::default(),
            balances_querier: BalancesQuerier::default(),
        }
    }

    pub fn with_rewards_querier(&mut self, rewards: &[(&String, &Uint128)]) {
        self.rewards_querier = RewardsQuerier::new(rewards);
    }

    pub fn with_balances_querier(&mut self, balances: &[(&String, &Uint128)]) {
        self.balances_querier = BalancesQuerier::new(balances);
    }
}
