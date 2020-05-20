use std::cmp::min;

use cosmwasm_std::{
    generic_err, to_binary, to_vec, unauthorized, Api, Binary, Coin, Env, Extern, HandleResponse,
    InitResponse, Querier, QueryRequest, StdResult, Storage, Uint128,
};
use terra_bindings::{SwapMsg, TerraMsg, TerraQuerier, TerraQuery};

use crate::msg::{
    ConfigResponse, ExchangeRateResponse, HandleMsg, InitMsg, QueryMsg, SimulateResponse,
};
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
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::ExchangeRate {} => query_rate(deps),
        QueryMsg::Simulate { offer } => query_simulate(deps, offer),
        QueryMsg::Reflect { query } => query_reflect(deps, query),
    }
}

fn query_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let state = config_read(&deps.storage).load()?;
    let resp = ConfigResponse {
        ask: state.ask,
        offer: state.offer,
        owner: deps.api.human_address(&state.owner)?,
    };
    to_binary(&resp)
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

fn query_reflect<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    query: TerraQuery,
) -> StdResult<Binary> {
    let request: QueryRequest<TerraQuery> = query.into();
    let raw_request = to_vec(&request)?;
    let resp = deps
        .querier
        .raw_query(&raw_request)
        .map_err(|e| generic_err(format!("System error: {}", e)))?;
    resp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ConfigResponse;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coin, coins, from_binary, CosmosMsg, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::Config {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!("BTC", value.ask.as_str());
        assert_eq!("ETH", value.offer.as_str());
        assert_eq!("creator", value.owner.as_str());
    }

    #[test]
    fn buy_limit() {
        let mut deps = mock_dependencies(20, &coins(200, "ETH"));

        let msg = InitMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let env = mock_env(&deps.api, "creator", &coins(200, "ETH"));
        let _res = init(&mut deps, env, msg).unwrap();

        // we buy BTC with half the ETH
        let env = mock_env(&deps.api, "creator", &[]);
        let contract_addr = deps.api.human_address(&env.contract.address).unwrap();
        let msg = HandleMsg::Buy {
            limit: Some(Uint128(100)),
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsg::Swap(SwapMsg::Trade {
            trader_addr,
            offer_coin,
            ask_denom,
        })) = &res.messages[0]
        {
            assert_eq!(trader_addr, &contract_addr);
            assert_eq!(offer_coin, &coin(100, "ETH"));
            assert_eq!(ask_denom, "BTC");
        } else {
            panic!("Expected swap message, got: {:?}", &res.messages[0]);
        }
    }

    #[test]
    fn only_owner_can_buy() {
        let mut deps = mock_dependencies(20, &coins(200, "ETH"));

        let msg = InitMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let env = mock_env(&deps.api, "creator", &coins(200, "ETH"));
        let _res = init(&mut deps, env, msg).unwrap();

        // we buy BTC with half the ETH
        let env = mock_env(&deps.api, "someone else", &[]);
        let msg = HandleMsg::Buy {
            limit: Some(Uint128(100)),
        };
        match handle(&mut deps, env, msg).unwrap_err() {
            StdError::Unauthorized { .. } => {}
            e => panic!("Expected unauthorized error, got: {}", e),
        }
    }

    #[test]
    fn sell_no_limit() {
        let mut deps = mock_dependencies(20, &[coin(200, "ETH"), coin(120, "BTC")]);

        let msg = InitMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let env = mock_env(&deps.api, "creator", &[]);
        let _res = init(&mut deps, env, msg).unwrap();

        // we sell all the BTC (faked balance above)
        let env = mock_env(&deps.api, "creator", &[]);
        let contract_addr = deps.api.human_address(&env.contract.address).unwrap();
        let msg = HandleMsg::Sell { limit: None };
        let res = handle(&mut deps, env, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsg::Swap(SwapMsg::Trade {
            trader_addr,
            offer_coin,
            ask_denom,
        })) = &res.messages[0]
        {
            assert_eq!(trader_addr, &contract_addr);
            assert_eq!(offer_coin, &coin(120, "BTC"));
            assert_eq!(ask_denom, "ETH");
        } else {
            panic!("Expected swap message, got: {:?}", &res.messages[0]);
        }
    }

    #[test]
    fn sell_limit_higher_than_balance() {
        let mut deps = mock_dependencies(20, &[coin(200, "ETH"), coin(133, "BTC")]);

        let msg = InitMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let env = mock_env(&deps.api, "creator", &[]);
        let _res = init(&mut deps, env, msg).unwrap();

        // we sell all the BTC (faked balance above)
        let env = mock_env(&deps.api, "creator", &[]);
        let contract_addr = deps.api.human_address(&env.contract.address).unwrap();
        let msg = HandleMsg::Sell {
            limit: Some(Uint128(250)),
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsg::Swap(SwapMsg::Trade {
            trader_addr,
            offer_coin,
            ask_denom,
        })) = &res.messages[0]
        {
            assert_eq!(trader_addr, &contract_addr);
            assert_eq!(offer_coin, &coin(133, "BTC"));
            assert_eq!(ask_denom, "ETH");
        } else {
            panic!("Expected swap message, got: {:?}", &res.messages[0]);
        }
    }
}
