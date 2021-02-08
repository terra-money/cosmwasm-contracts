use cosmwasm_std::{
    Api, Binary, Coin, Decimal, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage, Uint128,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use terra_cosmwasm::{SwapResponse, TerraQuerier};

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
    _env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::AssertLimitOrder {
            offer_coin,
            ask_denom,
            minimum_receive,
        } => assert_limit_order(deps, offer_coin, ask_denom, minimum_receive),
    }
}

pub fn assert_limit_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    offer_coin: Coin,
    ask_denom: String,
    minimum_receive: Uint128,
) -> StdResult<HandleResponse> {
    let querier = TerraQuerier::new(&deps.querier);
    let swap_res: SwapResponse = querier.query_swap(offer_coin, ask_denom)?;

    if swap_res.receive.amount < minimum_receive {
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
