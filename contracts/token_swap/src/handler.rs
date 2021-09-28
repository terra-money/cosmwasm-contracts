use crate::state::{Config, CONFIG};
use cosmwasm_std::{
    to_binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
    WasmMsg,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

pub fn swap(
    deps: DepsMut,
    sender: String,
    amount: Uint128,
    recipient: Option<String>,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    if !config.swap_enabled {
        return Err(StdError::generic_err("swap is not enabled"));
    }

    let recipient = recipient.unwrap_or(sender);
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.target_token,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                amount,
                recipient: recipient.clone(),
            })?,
        }))
        .add_attributes([
            ("action", "swap"),
            ("amount", &amount.to_string()),
            ("recipient", &recipient),
        ]))
}

pub fn disable(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let _ = CONFIG.update(deps.storage, |mut config| {
        if config.owner != info.sender {
            return Err(StdError::generic_err("unauthorized"));
        }

        config.swap_enabled = false;
        Ok(config)
    })?;

    Ok(Response::default().add_attributes([("action", "disable")]))
}

pub fn enable(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    let _ = CONFIG.update(deps.storage, |mut config| {
        if config.owner != info.sender {
            return Err(StdError::generic_err("unauthorized"));
        }

        config.swap_enabled = true;
        Ok(config)
    })?;

    Ok(Response::default().add_attributes([("action", "enable")]))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    let target_balance: BalanceResponse = deps.querier.query_wasm_smart(
        config.target_token.to_string(),
        &Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;
    let legacy_balance: BalanceResponse = deps.querier.query_wasm_smart(
        config.legacy_token.to_string(),
        &Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    let recipient = recipient.unwrap_or(config.owner);

    let mut messages: Vec<CosmosMsg> = vec![];
    if !target_balance.balance.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.target_token,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                amount: target_balance.balance,
                recipient: recipient.clone(),
            })?,
        }));
    }

    if !legacy_balance.balance.is_zero() {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.legacy_token,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                amount: legacy_balance.balance,
                recipient: recipient.clone(),
            })?,
        }));
    }

    Ok(Response::new().add_messages(messages).add_attributes([
        ("action", "withdraw"),
        ("legacy_balance", &legacy_balance.balance.to_string()),
        ("target_balance", &target_balance.balance.to_string()),
        ("recipient", &recipient),
    ]))
}
