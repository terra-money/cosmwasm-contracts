use crate::msg::{BalancesResponse, ConfigResponse};
use crate::state::{Config, CONFIG};
use cosmwasm_std::{Deps, Env, StdResult};
use cw20::{BalanceResponse, Cw20QueryMsg};

pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
        legacy_token: config.legacy_token,
        target_token: config.target_token,
        swap_enabled: config.swap_enabled,
    })
}

pub fn balances(deps: Deps, env: Env) -> StdResult<BalancesResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let target_balance: BalanceResponse = deps.querier.query_wasm_smart(
        config.target_token,
        &Cw20QueryMsg::Balance {
            address: env.contract.address.clone().into_string(),
        },
    )?;

    let legacy_balance: BalanceResponse = deps.querier.query_wasm_smart(
        config.legacy_token,
        &Cw20QueryMsg::Balance {
            address: env.contract.address.into_string(),
        },
    )?;

    Ok(BalancesResponse {
        legacy_balance: legacy_balance.balance,
        target_balance: target_balance.balance,
    })
}
