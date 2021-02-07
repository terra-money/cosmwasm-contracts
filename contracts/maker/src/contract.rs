use std::cmp::min;

use cosmwasm_std::{
    log, to_binary, to_vec, Api, BankMsg, Binary, Coin, CosmosMsg, Env, Extern, HandleResponse,
    HumanAddr, InitResponse, Querier, QueryRequest, StdError, StdResult, Storage, Uint128,
};
use terra_cosmwasm::{
    create_swap_msg, create_swap_send_msg, TerraMsgWrapper, TerraQuerier, TerraQueryWrapper,
};

use crate::msg::{ConfigResponse, HandleMsg, InitMsg, QueryMsg, SimulateResponse};
use crate::state::{config, config_read, State};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        ask: msg.ask,
        offer: msg.offer,
        owner: deps.api.canonical_address(&env.message.sender)?,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    match msg {
        HandleMsg::Buy { limit, recipient } => buy(deps, env, limit, recipient),
        HandleMsg::Sell { limit, recipient } => sell(deps, env, limit, recipient),
        HandleMsg::Send { coin, recipient } => transfer(deps, env, coin, recipient),
    }
}

pub fn transfer<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    coin: Coin,
    to_addr: HumanAddr,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let querier = TerraQuerier::new(&deps.querier);
    let tax_rate = querier.query_tax_rate()?.rate;
    let tax_cap = querier.query_tax_cap(&coin.denom)?.cap;
    let from_addr = env.contract.address;

    let mut expected_tax: Uint128 = tax_rate * coin.amount;
    if expected_tax > tax_cap {
        expected_tax = tax_cap;
    }

    let res = HandleResponse {
        log: vec![log("action", "send"), log("destination", to_addr.as_str())],
        messages: vec![BankMsg::Send {
            from_address: from_addr,
            to_address: to_addr,
            amount: vec![Coin {
                denom: coin.denom,
                amount: (coin.amount - expected_tax)?,
            }],
        }
        .into()],
        data: None,
    };

    Ok(res)
}

pub fn buy<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    limit: Option<Uint128>,
    recipient: Option<HumanAddr>,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let state = config_read(&deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != state.owner {
        return Err(StdError::unauthorized());
    }

    let contract_addr = env.contract.address;
    let mut offer = deps.querier.query_balance(&contract_addr, &state.offer)?;
    if offer.amount == Uint128(0) {
        return Ok(HandleResponse::default());
    }
    if let Some(stop) = limit {
        offer.amount = min(offer.amount, stop);
    }

    let msg: CosmosMsg<TerraMsgWrapper>;
    if let Some(recipient) = recipient {
        msg = create_swap_send_msg(contract_addr, recipient, offer, state.ask);
    } else {
        msg = create_swap_msg(contract_addr, offer, state.ask);
    }

    Ok(HandleResponse {
        messages: vec![msg],
        log: vec![],
        data: None,
    })
}

pub fn sell<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    limit: Option<Uint128>,
    recipient: Option<HumanAddr>,
) -> StdResult<HandleResponse<TerraMsgWrapper>> {
    let state = config_read(&deps.storage).load()?;
    if deps.api.canonical_address(&env.message.sender)? != state.owner {
        return Err(StdError::unauthorized());
    }

    let contract_addr = env.contract.address;
    let mut sell = deps.querier.query_balance(&contract_addr, &state.ask)?;
    if sell.amount == Uint128(0) {
        return Ok(HandleResponse::default());
    }

    if let Some(stop) = limit {
        sell.amount = min(sell.amount, stop);
    }

    let msg: CosmosMsg<TerraMsgWrapper>;
    if let Some(recipient) = recipient {
        msg = create_swap_send_msg(contract_addr, recipient, sell, state.offer);
    } else {
        msg = create_swap_msg(contract_addr, sell, state.offer);
    }

    Ok(HandleResponse {
        messages: vec![msg],
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
        QueryMsg::Simulate { offer } => query_swap(deps, offer),
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

fn query_swap<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    offer: Coin,
) -> StdResult<Binary> {
    let state = config_read(&deps.storage).load()?;
    let ask = if offer.denom == state.ask {
        state.offer
    } else if offer.denom == state.offer {
        state.ask
    } else {
        return Err(StdError::generic_err(format!(
            "Cannot simulate '{}' swap, neither contract's ask nor offer",
            offer.denom
        )));
    };
    let receive = TerraQuerier::new(&deps.querier).query_swap(offer.clone(), ask)?.receive;
    let resp = SimulateResponse {
        sell: offer,
        buy: receive,
    };
    to_binary(&resp)
}

fn query_reflect<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    query: TerraQueryWrapper,
) -> StdResult<Binary> {
    let request: QueryRequest<TerraQueryWrapper> = query.into();
    let raw_request = to_vec(&request)?;
    deps.querier
        .raw_query(&raw_request)
        .map_err(|e| StdError::generic_err(format!("System error: {}", e)))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ConfigResponse;
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{coin, coins, from_binary, CosmosMsg, Decimal, HumanAddr, StdError};

    use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraMsg, TerraQuery};
    use terra_mocks::mock_dependencies;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let env = mock_env("creator", &coins(1000, "earth"));

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
        let env = mock_env("creator", &coins(200, "ETH"));
        let _res = init(&mut deps, env, msg).unwrap();

        // we buy BTC with half the ETH
        let env = mock_env("creator", &[]);
        let contract_addr = env.contract.address.clone();
        let msg = HandleMsg::Buy {
            limit: Some(Uint128(100)),
            recipient: None,
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0] {
            assert_eq!(route, "market");

            match &msg_data {
                TerraMsg::Swap {
                    trader,
                    offer_coin,
                    ask_denom,
                } => {
                    assert_eq!(trader, &contract_addr);
                    assert_eq!(offer_coin, &coin(100, "ETH"));
                    assert_eq!(ask_denom, "BTC");
                }
                _ => panic!("MUST NOT ENTER HERE"),
            }
        } else {
            panic!("Expected swap message, got: {:?}", &res.messages[0]);
        }
    }

    #[test]
    fn buy_send_limit() {
        let mut deps = mock_dependencies(20, &coins(200, "ETH"));

        let msg = InitMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let env = mock_env("creator", &coins(200, "ETH"));
        let _res = init(&mut deps, env, msg).unwrap();

        // we buy BTC with half the ETH
        let env = mock_env("creator", &[]);
        let recipient = HumanAddr("addr0000".to_string());
        let contract_addr = env.contract.address.clone();
        let msg = HandleMsg::Buy {
            limit: Some(Uint128(100)),
            recipient: Some(HumanAddr("addr0000".to_string())),
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0] {
            assert_eq!(route, "market");

            match &msg_data {
                TerraMsg::SwapSend {
                    from_address,
                    to_address,
                    offer_coin,
                    ask_denom,
                } => {
                    assert_eq!(from_address, &contract_addr);
                    assert_eq!(to_address, &recipient);
                    assert_eq!(offer_coin, &coin(100, "ETH"));
                    assert_eq!(ask_denom, "BTC");
                }
                _ => panic!("MUST NOT ENTER HERE"),
            }
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
        let env = mock_env("creator", &coins(200, "ETH"));
        let _res = init(&mut deps, env, msg).unwrap();

        // we buy BTC with half the ETH
        let env = mock_env("someone else", &[]);
        let msg = HandleMsg::Buy {
            limit: Some(Uint128(100)),
            recipient: None,
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
        let env = mock_env("creator", &[]);
        let _res = init(&mut deps, env, msg).unwrap();

        // we sell all the BTC (faked balance above)
        let env = mock_env("creator", &[]);
        let contract_addr = env.contract.address.clone();
        let msg = HandleMsg::Sell {
            limit: None,
            recipient: None,
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0] {
            assert_eq!(route, "market");

            match &msg_data {
                TerraMsg::Swap {
                    trader,
                    offer_coin,
                    ask_denom,
                } => {
                    assert_eq!(trader, &contract_addr);
                    assert_eq!(offer_coin, &coin(120, "BTC"));
                    assert_eq!(ask_denom, "ETH");
                }
                _ => panic!("MUST NOT ENTER HERE"),
            }
        } else {
            panic!("Expected swap message, got: {:?}", &res.messages[0]);
        }
    }

    #[test]
    fn sell_send_no_limit() {
        let mut deps = mock_dependencies(20, &[coin(200, "ETH"), coin(120, "BTC")]);

        let msg = InitMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let env = mock_env("creator", &[]);
        let _res = init(&mut deps, env, msg).unwrap();

        // we sell all the BTC (faked balance above)
        let env = mock_env("creator", &[]);
        let contract_addr = env.contract.address.clone();
        let recipient = HumanAddr("addr0000".to_string());
        let msg = HandleMsg::Sell {
            limit: None,
            recipient: Some(HumanAddr("addr0000".to_string())),
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0] {
            assert_eq!(route, "market");

            match &msg_data {
                TerraMsg::SwapSend {
                    from_address,
                    to_address,
                    offer_coin,
                    ask_denom,
                } => {
                    assert_eq!(from_address, &contract_addr);
                    assert_eq!(to_address, &recipient);
                    assert_eq!(offer_coin, &coin(120, "BTC"));
                    assert_eq!(ask_denom, "ETH");
                }
                _ => panic!("MUST NOT ENTER HERE"),
            }
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
        let env = mock_env("creator", &[]);
        let _res = init(&mut deps, env, msg).unwrap();

        // we sell all the BTC (faked balance above)
        let env = mock_env("creator", &[]);
        let contract_addr = env.contract.address.clone();
        let msg = HandleMsg::Sell {
            limit: Some(Uint128(250)),
            recipient: None,
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0] {
            assert_eq!(route, "market");

            match &msg_data {
                TerraMsg::Swap {
                    trader,
                    offer_coin,
                    ask_denom,
                } => {
                    assert_eq!(trader, &contract_addr);
                    assert_eq!(offer_coin, &coin(133, "BTC"));
                    assert_eq!(ask_denom, "ETH");
                }
                _ => panic!("MUST NOT ENTER HERE"),
            }
        } else {
            panic!("Expected swap message, got: {:?}", &res.messages[0]);
        }
    }

    #[test]
    fn send_with_tax() {
        let mut deps = mock_dependencies(20, &coins(10000, "SDT"));

        // set mock treasury querier
        let tax_rate = Decimal::percent(2);
        let tax_caps = &[("SDT", 10u128), ("UST", 500u128)];

        deps.querier.with_treasury(tax_rate, tax_caps);

        let msg = InitMsg {
            ask: "UST".into(),
            offer: "SDT".into(),
        };
        let env = mock_env("creator", &coins(10000, "SDT"));
        let _res = init(&mut deps, env, msg).unwrap();

        // we buy BTC with half the ETH
        let env = mock_env("creator", &[]);
        let contract_addr = env.contract.address.clone();
        let receiver_addr = HumanAddr::from("receiver");
        let msg = HandleMsg::Send {
            coin: Coin {
                denom: "SDT".to_string(),
                amount: Uint128(10000),
            },
            recipient: receiver_addr.clone(),
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // make sure we produce proper send with tax consideration
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Bank(BankMsg::Send {
            from_address,
            to_address,
            amount,
        }) = &res.messages[0]
        {
            assert_eq!(from_address, &contract_addr);
            assert_eq!(to_address, &receiver_addr);
            assert_eq!(amount, &vec![coin(9990, "SDT")]);
        } else {
            panic!("Expected bank send message, got: {:?}", &res.messages[0]);
        }
    }

    #[test]
    fn send_with_capped_tax() {
        let mut deps = mock_dependencies(20, &coins(10000, "SDT"));

        // set mock treasury querier
        let tax_rate = Decimal::percent(2);
        let tax_caps = &[("SDT", 10u128), ("UST", 500u128)];

        deps.querier.with_treasury(tax_rate, tax_caps);

        let msg = InitMsg {
            ask: "UST".into(),
            offer: "SDT".into(),
        };
        let env = mock_env("creator", &coins(10000, "SDT"));
        let _res = init(&mut deps, env, msg).unwrap();

        // we buy BTC with half the ETH
        let env = mock_env("creator", &[]);
        let contract_addr = env.contract.address.clone();
        let receiver_addr = HumanAddr::from("receiver");
        let msg = HandleMsg::Send {
            coin: Coin {
                denom: "SDT".to_string(),
                amount: Uint128(50),
            },
            recipient: receiver_addr.clone(),
        };
        let res = handle(&mut deps, env, msg).unwrap();

        // make sure tax is capped
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Bank(BankMsg::Send {
            from_address,
            to_address,
            amount,
        }) = &res.messages[0]
        {
            assert_eq!(from_address, &contract_addr);
            assert_eq!(to_address, &receiver_addr);
            assert_eq!(amount, &vec![coin(49, "SDT")]);
        } else {
            panic!("Expected bank send message, got: {:?}", &res.messages[0]);
        }
    }

    #[test]
    fn basic_queries() {
        let mut deps = mock_dependencies(20, &[]);
        // set the exchange rates between ETH and BTC (and back)
        deps.querier.with_market(&[
            ("ETH", "BTC", Decimal::percent(15)),
            ("BTC", "ETH", Decimal::percent(666)),
        ]);

        let msg = InitMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let env = mock_env("creator", &[]);
        let _res = init(&mut deps, env, msg).unwrap();

        // check the config
        let res = query(&mut deps, QueryMsg::Config {}).unwrap();
        let cfg: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(
            cfg,
            ConfigResponse {
                owner: HumanAddr::from("creator"),
                ask: "BTC".to_string(),
                offer: "ETH".to_string(),
            }
        );

        // simulate a forward swap
        let res = query(
            &mut deps,
            QueryMsg::Simulate {
                offer: coin(100, "ETH"),
            },
        )
        .unwrap();
        let cfg: SimulateResponse = from_binary(&res).unwrap();
        assert_eq!(
            cfg,
            SimulateResponse {
                sell: coin(100, "ETH"),
                buy: coin(15, "BTC"),
            }
        );

        // simulate a reverse swap
        let res = query(
            &mut deps,
            QueryMsg::Simulate {
                offer: coin(10, "BTC"),
            },
        )
        .unwrap();
        let cfg: SimulateResponse = from_binary(&res).unwrap();
        assert_eq!(
            cfg,
            SimulateResponse {
                sell: coin(10, "BTC"),
                buy: coin(66, "ETH"),
            }
        );
    }

    #[test]
    fn query_treasury() {
        let mut deps = mock_dependencies(20, &[]);
        // set the exchange rates between ETH and BTC (and back)
        let tax_rate = Decimal::percent(2);
        let tax_caps = &[("ETH", 1000u128), ("BTC", 500u128)];

        deps.querier.with_treasury(tax_rate, tax_caps);

        let msg = InitMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let env = mock_env("creator", &[]);
        let _res = init(&mut deps, env, msg).unwrap();

        // test all treasury functions
        let tax_rate_query = QueryMsg::Reflect {
            query: TerraQueryWrapper {
                route: "treasury".to_string(),
                query_data: TerraQuery::TaxRate {},
            },
        };
        let res = query(&mut deps, tax_rate_query).unwrap();
        let tax_rate_res: TaxRateResponse = from_binary(&res).unwrap();
        assert_eq!(tax_rate_res.rate, tax_rate);

        let tax_cap_query = QueryMsg::Reflect {
            query: TerraQueryWrapper {
                route: "treasury".to_string(),
                query_data: TerraQuery::TaxCap {
                    denom: "ETH".to_string(),
                },
            },
        };
        let res = query(&mut deps, tax_cap_query).unwrap();
        let cap: TaxCapResponse = from_binary(&res).unwrap();
        assert_eq!(cap.cap, Uint128(1000));
    }
}
