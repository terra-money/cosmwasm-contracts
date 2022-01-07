#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult, Uint128, WasmMsg,
};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, Denom};

use crate::msg::{
    Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, VestingAccountResponse, VestingSchedule,
};
use crate::state::{VestingAccount, VESTING_ACCOUNTS};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::RegisterVestingAccount {
            master_address,
            address,
            vesting_schedule,
        } => {
            // deposit validation
            if info.funds.len() != 1 {
                return Err(StdError::generic_err("must deposit only one type of token"));
            }

            let deposit_coin = info.funds[0].clone();
            register_vesting_account(
                deps,
                env,
                master_address,
                address,
                Denom::Native(deposit_coin.denom),
                deposit_coin.amount,
                vesting_schedule,
            )
        }
        ExecuteMsg::DeregisterVestingAccount {
            address,
            vested_token_recipient,
            left_vesting_token_recipient,
        } => deregister_vesting_account(
            deps,
            env,
            info,
            address,
            vested_token_recipient,
            left_vesting_token_recipient,
        ),
        ExecuteMsg::Claim { recipient } => claim(deps, env, info, recipient),
    }
}

fn register_vesting_account(
    deps: DepsMut,
    env: Env,
    master_address: Option<String>,
    address: String,
    deposit_denom: Denom,
    deposit_amount: Uint128,
    vesting_schedule: VestingSchedule,
) -> StdResult<Response> {
    // vesting_account existence check
    if VESTING_ACCOUNTS.has(deps.storage, address.to_string()) {
        return Err(StdError::generic_err("already exists"));
    }

    // validate vesting schedule
    match vesting_schedule.clone() {
        VestingSchedule::LinearVesting {
            start_time,
            end_time,
            total_amount,
        } => {
            let start_time = start_time
                .parse::<u64>()
                .map_err(|_| StdError::generic_err("invalid start_time"))?;

            let end_time = end_time
                .parse::<u64>()
                .map_err(|_| StdError::generic_err("invalid end_time"))?;

            if start_time < env.block.time.seconds() {
                return Err(StdError::generic_err("invalid start_time"));
            }

            if end_time <= start_time {
                return Err(StdError::generic_err(
                    "end_time must be bigger than start_time",
                ));
            }

            if total_amount != deposit_amount {
                return Err(StdError::generic_err("invalid total_amount"));
            }

            if total_amount.is_zero() {
                return Err(StdError::generic_err(
                    "cannot make zero token vesting account",
                ));
            }
        }
        VestingSchedule::PeriodicVesting {
            start_time,
            end_time,
            vesting_interval,
            amount,
        } => {
            let start_time = start_time
                .parse::<u64>()
                .map_err(|_| StdError::generic_err("invalid start_time"))?;

            let end_time = end_time
                .parse::<u64>()
                .map_err(|_| StdError::generic_err("invalid end_time"))?;

            let vesting_interval = vesting_interval
                .parse::<u64>()
                .map_err(|_| StdError::generic_err("invalid vesting_interval"))?;

            if start_time < env.block.time.seconds() {
                return Err(StdError::generic_err("invalid start_time"));
            }

            if end_time <= start_time {
                return Err(StdError::generic_err(
                    "end_time must be bigger than start_time",
                ));
            }

            if vesting_interval == 0 {
                return Err(StdError::generic_err("vesting_interval must be non-zero"));
            }

            let time_period = end_time - start_time;
            let num_interval = time_period / vesting_interval;
            if time_period != num_interval * vesting_interval {
                return Err(StdError::generic_err(
                    "(end_time - start_time) must be multiple of vesting_interval",
                ));
            }

            if amount.checked_mul(Uint128::from(num_interval))? != deposit_amount {
                return Err(StdError::generic_err(
                    "deposit_amount must be equal with amount * num_interval",
                ));
            }

            if amount.is_zero() {
                return Err(StdError::generic_err(
                    "cannot make zero token vesting account",
                ));
            }
        }
    }

    VESTING_ACCOUNTS.save(
        deps.storage,
        address.to_string(),
        &VestingAccount {
            master_address: master_address.clone(),
            address: address.to_string(),
            vesting_denom: deposit_denom.clone(),
            vesting_amount: deposit_amount,
            vesting_schedule,
            claimed_amount: Uint128::zero(),
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        ("action", "register_vesting_account"),
        (
            "master_address",
            master_address.unwrap_or_default().as_str(),
        ),
        ("address", address.as_str()),
        ("vesting_denom", &format!("{:?}", deposit_denom)),
        ("vesting_amount", &deposit_amount.to_string()),
    ]))
}

fn deregister_vesting_account(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
    vested_token_recipient: Option<String>,
    left_vesting_token_recipient: Option<String>,
) -> StdResult<Response> {
    let sender = info.sender;
    let account = VESTING_ACCOUNTS.may_load(deps.storage, address.to_string())?;
    if account.is_none() {
        return Err(StdError::generic_err("not found"));
    }

    let account = account.unwrap();
    if account.master_address.is_none() || account.master_address.unwrap() != sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    // remove vesting account
    VESTING_ACCOUNTS.remove(deps.storage, address.to_string());

    let mut messages: Vec<CosmosMsg> = vec![];
    let vested_amount = account
        .vesting_schedule
        .vested_amount(env.block.time.seconds())?;
    let claimed_amount = account.claimed_amount;

    // transfer already vested but not claimed amount to
    // a account address or the given `vested_token_recipient` address
    let claimable_amount = vested_amount.checked_sub(claimed_amount)?;
    if !claimable_amount.is_zero() {
        let recipient = vested_token_recipient.unwrap_or_else(|| address.to_string());
        let message: CosmosMsg = match account.vesting_denom.clone() {
            Denom::Native(denom) => BankMsg::Send {
                to_address: recipient,
                amount: vec![Coin {
                    denom,
                    amount: claimable_amount,
                }],
            }
            .into(),
            Denom::Cw20(contract_addr) => WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient,
                    amount: claimable_amount,
                })?,
                funds: vec![],
            }
            .into(),
        };

        messages.push(message);
    }

    // transfer left vesting amount to owner or
    // the given `left_vesting_token_recipient` address
    let left_vesting_amount = account.vesting_amount.checked_sub(vested_amount)?;
    if !left_vesting_amount.is_zero() {
        let recipient = left_vesting_token_recipient.unwrap_or_else(|| sender.to_string());
        let message: CosmosMsg = match account.vesting_denom.clone() {
            Denom::Native(denom) => BankMsg::Send {
                to_address: recipient,
                amount: vec![Coin {
                    denom,
                    amount: claimable_amount,
                }],
            }
            .into(),
            Denom::Cw20(contract_addr) => WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient,
                    amount: claimable_amount,
                })?,
                funds: vec![],
            }
            .into(),
        };

        messages.push(message);
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "deregister_vesting_account"),
        ("address", address.as_str()),
        ("vesting_denom", &format!("{:?}", account.vesting_denom)),
        ("vesting_amount", &account.vesting_amount.to_string()),
        ("vested_amount", &vested_amount.to_string()),
        ("left_vesting_amount", &left_vesting_amount.to_string()),
    ]))
}

fn claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> StdResult<Response> {
    let sender = info.sender;

    let account = VESTING_ACCOUNTS.may_load(deps.storage, sender.to_string())?;
    if account.is_none() {
        return Err(StdError::generic_err("not found"));
    }

    let mut account = account.unwrap();
    let vested_amount = account
        .vesting_schedule
        .vested_amount(env.block.time.seconds())?;
    let claimed_amount = account.claimed_amount;

    let claimable_amount = vested_amount.checked_sub(claimed_amount)?;
    if claimable_amount.is_zero() {
        return Err(StdError::generic_err("no claimable amount left"));
    }

    account.claimed_amount = vested_amount;
    VESTING_ACCOUNTS.save(deps.storage, sender.to_string(), &account)?;

    let recipient = recipient.unwrap_or_else(|| sender.to_string());
    let message: CosmosMsg = match account.vesting_denom.clone() {
        Denom::Native(denom) => BankMsg::Send {
            to_address: recipient,
            amount: vec![Coin {
                denom,
                amount: claimable_amount,
            }],
        }
        .into(),
        Denom::Cw20(contract_addr) => WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient,
                amount: claimable_amount,
            })?,
            funds: vec![],
        }
        .into(),
    };

    Ok(Response::new().add_message(message).add_attributes(vec![
        ("action", "claim"),
        ("address", sender.as_str()),
        ("vesting_denom", &format!("{:?}", account.vesting_denom)),
        ("vesting_amount", &account.vesting_amount.to_string()),
        ("vested_amount", &vested_amount.to_string()),
        ("claim_amount", &claimable_amount.to_string()),
    ]))
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let amount = cw20_msg.amount;
    let _sender = cw20_msg.sender;
    let contract = info.sender;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::RegisterVestingAccount {
            master_address,
            address,
            vesting_schedule,
        }) => register_vesting_account(
            deps,
            env,
            master_address,
            address,
            Denom::Cw20(contract),
            amount,
            vesting_schedule,
        ),
        Err(_) => Err(StdError::generic_err("invalid cw20 hook message")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VestingAccount { address } => to_binary(&vesting_account(deps, env, address)?),
    }
}

fn vesting_account(deps: Deps, env: Env, address: String) -> StdResult<VestingAccountResponse> {
    let account = VESTING_ACCOUNTS.may_load(deps.storage, address)?;
    if account.is_none() {
        return Err(StdError::generic_err("not found"));
    }

    let account = account.unwrap();
    let vested_amount = account
        .vesting_schedule
        .vested_amount(env.block.time.seconds())?;
    Ok(VestingAccountResponse {
        master_address: account.master_address,
        address: account.address,
        vesting_denom: account.vesting_denom,
        vesting_amount: account.vesting_amount,
        vested_amount,
        vesting_schedule: account.vesting_schedule,
        claimable_amount: vested_amount.checked_sub(account.claimed_amount)?,
    })
}
