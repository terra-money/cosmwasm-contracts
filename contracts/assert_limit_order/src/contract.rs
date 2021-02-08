use cosmwasm_std::{
    Api, Binary, Decimal, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult,
    Storage, Uint128,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};

const DECIMAL_FRACTIONAL: Uint128 = Uint128(1_000_000_000u128);

pub fn reverse_decimal(decimal: Decimal) -> Decimal {
    Decimal::from_ratio(DECIMAL_FRACTIONAL, decimal * DECIMAL_FRACTIONAL)
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::AssertLimitOrder {
            offer_amount,
            ask_denom,
            ask_prev_balance,
            belief_price,
            slippage_tolerance,
        } => assert_limit_order(
            deps,
            env,
            offer_amount,
            ask_denom,
            ask_prev_balance,
            belief_price,
            slippage_tolerance,
        ),
    }
}

pub fn assert_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    offer_amount: Uint128,
    ask_denom: String,
    ask_prev_balance: Uint128,
    belief_price: Decimal,
    slippage_tolerance: Decimal,
) -> StdResult<HandleResponse> {
    let ask_cur_balance = deps
        .querier
        .query_balance(&env.message.sender, ask_denom.as_str())?;
    let swap_return = (ask_cur_balance.amount - ask_prev_balance)?;

    let expected_return = offer_amount * reverse_decimal(belief_price);
    let minimum_return = (expected_return - expected_return * slippage_tolerance)?;
    if swap_return < minimum_return {
        return Err(StdError::generic_err(
            "{\"msg\": \"slippage_tolerance assertion\"}",
        ));
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _msg: QueryMsg,
) -> StdResult<Binary> {
    Ok(Binary::default())
}
