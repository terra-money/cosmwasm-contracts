#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Coin, Decimal, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError, StdResult,
    Uint128,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use terra_cosmwasm::{SwapResponse, TerraQuerier, TerraQueryWrapper};

const DECIMAL_FRACTIONAL: u128 = 1_000_000_000u128;

pub fn reverse_decimal(decimal: Decimal) -> Decimal {
    Decimal::from_ratio(
        DECIMAL_FRACTIONAL,
        decimal * Uint128::new(DECIMAL_FRACTIONAL),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<TerraQueryWrapper>,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::AssertLimitOrder {
            offer_coin,
            ask_denom,
            minimum_receive,
        } => assert_limit_order(deps, offer_coin, ask_denom, minimum_receive),
    }
}

pub fn assert_limit_order(
    deps: DepsMut<TerraQueryWrapper>,
    offer_coin: Coin,
    ask_denom: String,
    minimum_receive: Uint128,
) -> StdResult<Response> {
    let querier = TerraQuerier::new(&deps.querier);
    let swap_res: SwapResponse = querier.query_swap(offer_coin, ask_denom)?;

    if swap_res.receive.amount < minimum_receive {
        return Err(StdError::generic_err(
            "{\"msg\": \"slippage_tolerance assertion\"}",
        ));
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<QueryResponse> {
    Ok(QueryResponse::default())
}
