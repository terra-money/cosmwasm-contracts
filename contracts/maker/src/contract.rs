use std::cmp::min;

use cosmwasm_std::{
    to_binary, unauthorized, Api, Binary, Coin, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, Storage, Uint128,
};
use terra_bindings::{SwapMsg, TerraMsg, TerraQuerier};

use crate::msg::{ExchangeRateResponse, HandleMsg, InitMsg, QueryMsg, SimulateResponse};
use crate::state::{config, config_read, State};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        ask: msg.ask,
        offer: msg.offer,
        owner: env.message.sender,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse<TerraMsg>> {
    match msg {
        HandleMsg::Buy { limit } => buy(deps, env, limit),
        HandleMsg::Sell { limit } => sell(deps, env, limit),
    }
}

pub fn buy<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    limit: Option<Uint128>,
) -> StdResult<HandleResponse<TerraMsg>> {
    let state = config_read(&deps.storage).load()?;
    if env.message.sender != state.owner {
        return Err(unauthorized());
    }

    let contract_addr = deps.api.human_address(&env.contract.address)?;
    let mut offer = deps.querier.query_balance(&contract_addr, &state.offer)?;
    if offer.amount == Uint128(0) {
        return Ok(HandleResponse::default());
    }
    if let Some(stop) = limit {
        offer.amount = min(offer.amount, stop);
    }

    Ok(HandleResponse {
        messages: vec![SwapMsg::Trade {
            trader_addr: contract_addr,
            offer_coin: offer,
            ask_denom: state.ask,
        }
        .into()],
        log: vec![],
        data: None,
    })
}

pub fn sell<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    limit: Option<Uint128>,
) -> StdResult<HandleResponse<TerraMsg>> {
    let state = config_read(&deps.storage).load()?;
    if env.message.sender != state.owner {
        return Err(unauthorized());
    }

    let contract_addr = deps.api.human_address(&env.contract.address)?;
    let mut sell = deps.querier.query_balance(&contract_addr, &state.ask)?;
    if sell.amount == Uint128(0) {
        return Ok(HandleResponse::default());
    }
    if let Some(stop) = limit {
        sell.amount = min(sell.amount, stop);
    }

    Ok(HandleResponse {
        messages: vec![SwapMsg::Trade {
            trader_addr: contract_addr,
            offer_coin: sell,
            ask_denom: state.offer,
        }
        .into()],
        log: vec![],
        data: None,
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::ExchangeRate {} => query_rate(deps),
        QueryMsg::Simulate { offer } => query_simulate(deps, offer),
    }
}

fn query_rate<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let state = config_read(&deps.storage).load()?;
    let rate = TerraQuerier::new(&deps.querier).query_exchange_rate(&state.offer, &state.ask)?;
    let resp = ExchangeRateResponse {
        rate,
        ask: state.ask,
        offer: state.offer,
    };
    to_binary(&resp)
}

fn query_simulate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    offer: Coin,
) -> StdResult<Binary> {
    let state = config_read(&deps.storage).load()?;
    let receive =
        TerraQuerier::new(&deps.querier).query_simulate_swap(offer.clone(), &state.ask)?;
    let resp = SimulateResponse {
        sell: offer,
        buy: receive,
    };
    to_binary(&resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { count: 17 };
        let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // beneficiary can release it
        let env = mock_env(&deps.api, "anyone", &coins(2, "token"));
        let msg = HandleMsg::Increment {};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // beneficiary can release it
        let unauth_env = mock_env(&deps.api, "anyone", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let res = handle(&mut deps, unauth_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_env = mock_env(&deps.api, "creator", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let _res = handle(&mut deps, auth_env, msg).unwrap();

        // should now be 5
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
