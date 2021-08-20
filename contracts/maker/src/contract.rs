use std::cmp::min;

use cosmwasm_std::{
    attr, to_binary, to_vec, Addr, BankMsg, Binary, Coin, ContractResult, CosmosMsg, Deps, DepsMut,
    Env, MessageInfo, QueryRequest, QueryResponse, Response, StdError, StdResult, SystemResult,
    Uint128,
};
use terra_cosmwasm::{
    create_swap_msg, create_swap_send_msg, TerraMsgWrapper, TerraQuerier, TerraQueryWrapper,
};

use crate::errors::{MakerError, Unauthorized};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SimulateResponse};
use crate::state::{config, config_read, State};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<TerraMsgWrapper>, MakerError> {
    let state = State {
        ask: msg.ask,
        offer: msg.offer,
        owner: info.sender.into(),
    };

    config(deps.storage).save(&state)?;

    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TerraMsgWrapper>, MakerError> {
    match msg {
        ExecuteMsg::Buy { limit, recipient } => buy(deps, env, info, limit, recipient),
        ExecuteMsg::Sell { limit, recipient } => sell(deps, env, info, limit, recipient),
        ExecuteMsg::Send { coin, recipient } => transfer(deps, env, info, coin, recipient),
    }
}

pub fn transfer(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    coin: Coin,
    to_addr: Addr,
) -> Result<Response<TerraMsgWrapper>, MakerError> {
    let querier = TerraQuerier::new(&deps.querier);
    let tax_rate = querier.query_tax_rate()?.rate;
    let tax_cap = querier.query_tax_cap(&coin.denom)?.cap;

    let mut expected_tax: Uint128 = tax_rate * coin.amount;
    if expected_tax > tax_cap {
        expected_tax = tax_cap;
    }

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "send"),
            attr("destination", to_addr.to_string()),
        ])
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: to_addr.to_string(),
            amount: vec![Coin {
                denom: coin.denom,
                amount: Uint128::from(coin.amount.u128() - expected_tax.u128()),
            }],
        })))
}

pub fn buy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    limit: Option<Uint128>,
    recipient: Option<Addr>,
) -> Result<Response<TerraMsgWrapper>, MakerError> {
    let state = config_read(deps.storage).load()?;
    if info.sender != state.owner {
        return Err(Unauthorized {}.build());
    }

    let contract_addr = env.contract.address;
    let mut offer = deps.querier.query_balance(&contract_addr, &state.offer)?;
    if offer.amount == Uint128::zero() {
        return Ok(Response::default());
    }

    if let Some(stop) = limit {
        offer.amount = min(offer.amount, stop);
    }

    let msg: CosmosMsg<TerraMsgWrapper>;
    if let Some(recipient) = recipient {
        msg = create_swap_send_msg(recipient.to_string(), offer, state.ask);
    } else {
        msg = create_swap_msg(offer, state.ask);
    }

    Ok(Response::new().add_message(msg))
}

pub fn sell(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    limit: Option<Uint128>,
    recipient: Option<Addr>,
) -> Result<Response<TerraMsgWrapper>, MakerError> {
    let state = config_read(deps.storage).load()?;
    if info.sender != state.owner {
        return Err(Unauthorized {}.build());
    }

    let contract_addr = env.contract.address;
    let mut sell = deps.querier.query_balance(&contract_addr, &state.ask)?;
    if sell.amount == Uint128::zero() {
        return Ok(Response::default());
    }

    if let Some(stop) = limit {
        sell.amount = min(sell.amount, stop);
    }

    let msg: CosmosMsg<TerraMsgWrapper>;
    if let Some(recipient) = recipient {
        msg = create_swap_send_msg(recipient.to_string(), sell, state.offer);
    } else {
        msg = create_swap_msg(sell, state.offer);
    }

    Ok(Response::new().add_message(msg))
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::Simulate { offer } => query_swap(deps, env, offer),
        QueryMsg::Reflect { query } => query_reflect(deps, env, query),
    }
}

fn query_config(deps: Deps, _env: Env) -> StdResult<QueryResponse> {
    let state = config_read(deps.storage).load()?;
    let resp = ConfigResponse {
        ask: state.ask,
        offer: state.offer,
        owner: Addr::unchecked(state.owner),
    };
    to_binary(&resp)
}

fn query_swap(deps: Deps, _env: Env, offer: Coin) -> StdResult<QueryResponse> {
    let state = config_read(deps.storage).load()?;
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
    let receive = TerraQuerier::new(&deps.querier)
        .query_swap(offer.clone(), ask)?
        .receive;
    let resp = SimulateResponse {
        sell: offer,
        buy: receive,
    };
    to_binary(&resp)
}

fn query_reflect(deps: Deps, _env: Env, query: TerraQueryWrapper) -> StdResult<Binary> {
    let request: QueryRequest<TerraQueryWrapper> = query.into();
    let raw_request = to_vec(&request)?;
    match deps.querier.raw_query(&raw_request) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {}",
            system_err
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {}",
            contract_err
        ))),
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::ConfigResponse;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, Addr, CosmosMsg, Decimal};

    use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraMsg, TerraQuery, TerraRoute};
    use terra_mocks::mock_dependencies;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!("BTC", value.ask.as_str());
        assert_eq!("ETH", value.offer.as_str());
        assert_eq!("creator", value.owner.to_string().as_str());
    }

    #[test]
    fn buy_limit() {
        let mut deps = mock_dependencies(&coins(200, "ETH"));

        let msg = InstantiateMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let info = mock_info("creator", &coins(200, "ETH"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // we buy BTC with half the ETH
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::Buy {
            limit: Some(Uint128::from(100u128)),
            recipient: None,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0].msg {
            assert_eq!(route, &TerraRoute::Market);

            match &msg_data {
                TerraMsg::Swap {
                    offer_coin,
                    ask_denom,
                } => {
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
        let mut deps = mock_dependencies(&coins(200, "ETH"));

        let msg = InstantiateMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let info = mock_info("creator", &coins(200, "ETH"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // we buy BTC with half the ETH
        let info = mock_info("creator", &[]);
        let recipient = "addr0000".to_string();
        let msg = ExecuteMsg::Buy {
            limit: Some(Uint128::from(100u128)),
            recipient: Some(Addr::unchecked("addr0000".to_string())),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0].msg {
            assert_eq!(route, &TerraRoute::Market);

            match &msg_data {
                TerraMsg::SwapSend {
                    to_address,
                    offer_coin,
                    ask_denom,
                } => {
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
        let mut deps = mock_dependencies(&coins(200, "ETH"));

        let msg = InstantiateMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let info = mock_info("creator", &coins(200, "ETH"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // we buy BTC with half the ETH
        let info = mock_info("someone else", &[]);
        let msg = ExecuteMsg::Buy {
            limit: Some(Uint128::from(100u128)),
            recipient: None,
        };
        match execute(deps.as_mut(), mock_env(), info, msg).unwrap_err() {
            MakerError::Unauthorized { .. } => {}
            e => panic!("Expected unauthorized error, got: {}", e),
        }
    }

    #[test]
    fn sell_no_limit() {
        let mut deps = mock_dependencies(&[coin(200, "ETH"), coin(120, "BTC")]);

        let msg = InstantiateMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // we sell all the BTC (faked balance above)
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::Sell {
            limit: None,
            recipient: None,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0].msg {
            assert_eq!(route, &TerraRoute::Market);

            match &msg_data {
                TerraMsg::Swap {
                    offer_coin,
                    ask_denom,
                } => {
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
        let mut deps = mock_dependencies(&[coin(200, "ETH"), coin(120, "BTC")]);

        let msg = InstantiateMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // we sell all the BTC (faked balance above)
        let info = mock_info("creator", &[]);
        let recipient = Addr::unchecked("addr0000".to_string());
        let msg = ExecuteMsg::Sell {
            limit: None,
            recipient: Some(recipient.clone()),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0].msg {
            assert_eq!(route, &TerraRoute::Market);

            match &msg_data {
                TerraMsg::SwapSend {
                    to_address,
                    offer_coin,
                    ask_denom,
                } => {
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
        let mut deps = mock_dependencies(&[coin(200, "ETH"), coin(133, "BTC")]);

        let msg = InstantiateMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // we sell all the BTC (faked balance above)
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::Sell {
            limit: Some(Uint128::from(250u128)),
            recipient: None,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // make sure we produce proper trade order
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Custom(TerraMsgWrapper { route, msg_data }) = &res.messages[0].msg {
            assert_eq!(route, &TerraRoute::Market);

            match &msg_data {
                TerraMsg::Swap {
                    offer_coin,
                    ask_denom,
                } => {
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
        let mut deps = mock_dependencies(&coins(10000, "SDT"));

        // set mock treasury querier
        let tax_rate = Decimal::percent(2);
        let tax_caps = &[("SDT", 10u128), ("UST", 500u128)];

        deps.querier.with_treasury(tax_rate, tax_caps);

        let msg = InstantiateMsg {
            ask: "UST".into(),
            offer: "SDT".into(),
        };
        let info = mock_info("creator", &coins(10000, "SDT"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // we buy BTC with half the ETH
        let info = mock_info("creator", &[]);
        let receiver_addr = Addr::unchecked("receiver");
        let msg = ExecuteMsg::Send {
            coin: Coin {
                denom: "SDT".to_string(),
                amount: Uint128::from(10000u128),
            },
            recipient: receiver_addr.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // make sure we produce proper send with tax consideration
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &res.messages[0].msg {
            assert_eq!(to_address, &receiver_addr);
            assert_eq!(amount, &vec![coin(9990, "SDT")]);
        } else {
            panic!("Expected bank send message, got: {:?}", &res.messages[0]);
        }
    }

    #[test]
    fn send_with_capped_tax() {
        let mut deps = mock_dependencies(&coins(10000, "SDT"));

        // set mock treasury querier
        let tax_rate = Decimal::percent(2);
        let tax_caps = &[("SDT", 10u128), ("UST", 500u128)];

        deps.querier.with_treasury(tax_rate, tax_caps);

        let msg = InstantiateMsg {
            ask: "UST".into(),
            offer: "SDT".into(),
        };
        let info = mock_info("creator", &coins(10000, "SDT"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // we buy BTC with half the ETH
        let info = mock_info("creator", &[]);
        let receiver_addr = Addr::unchecked("receiver");
        let msg = ExecuteMsg::Send {
            coin: Coin {
                denom: "SDT".to_string(),
                amount: Uint128::from(50u128),
            },
            recipient: receiver_addr.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // make sure tax is capped
        assert_eq!(1, res.messages.len());
        if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &res.messages[0].msg {
            assert_eq!(to_address, &receiver_addr);
            assert_eq!(amount, &vec![coin(49, "SDT")]);
        } else {
            panic!("Expected bank send message, got: {:?}", &res.messages[0]);
        }
    }

    #[test]
    fn basic_queries() {
        let mut deps = mock_dependencies(&[]);
        // set the exchange rates between ETH and BTC (and back)
        deps.querier.with_market(&[
            ("ETH", "BTC", Decimal::percent(15)),
            ("BTC", "ETH", Decimal::percent(666)),
        ]);

        let msg = InstantiateMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // check the config
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let cfg: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(
            cfg,
            ConfigResponse {
                owner: Addr::unchecked("creator"),
                ask: "BTC".to_string(),
                offer: "ETH".to_string(),
            }
        );

        // simulate a forward swap
        let res = query(
            deps.as_ref(),
            mock_env(),
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
            deps.as_ref(),
            mock_env(),
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
        let mut deps = mock_dependencies(&[]);
        // set the exchange rates between ETH and BTC (and back)
        let tax_rate = Decimal::percent(2);
        let tax_caps = &[("ETH", 1000u128), ("BTC", 500u128)];

        deps.querier.with_treasury(tax_rate, tax_caps);

        let msg = InstantiateMsg {
            ask: "BTC".into(),
            offer: "ETH".into(),
        };
        let info = mock_info("creator", &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // test all treasury functions
        let tax_rate_query = QueryMsg::Reflect {
            query: TerraQueryWrapper {
                route: TerraRoute::Treasury,
                query_data: TerraQuery::TaxRate {},
            },
        };
        let res = query(deps.as_ref(), mock_env(), tax_rate_query).unwrap();
        let tax_rate_res: TaxRateResponse = from_binary(&res).unwrap();
        assert_eq!(tax_rate_res.rate, tax_rate);

        let tax_cap_query = QueryMsg::Reflect {
            query: TerraQueryWrapper {
                route: TerraRoute::Treasury,
                query_data: TerraQuery::TaxCap {
                    denom: "ETH".to_string(),
                },
            },
        };
        let res = query(deps.as_ref(), mock_env(), tax_cap_query).unwrap();
        let cap: TaxCapResponse = from_binary(&res).unwrap();
        assert_eq!(cap.cap, Uint128::from(1000u128));
    }
}
