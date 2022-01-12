#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Attribute, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};

use serde_json::to_string;

use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Denom};

use crate::external::handle::{
    AccruedRewardsResponse, HubContractExecuteMsg, RewardContractExecuteMsg, RewardContractQueryMsg,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VestingInfoResponse};
use crate::state::{Config, VestingInfo, CONFIG, VESTING_INFO};

const VESTING_DENOM: &str = "uluna";
const REWARDS_DENOM: &str = "uusd";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // validate owner address
    deps.api.addr_validate(&msg.owner_address)?;

    // deposit validation
    if info.funds.len() != 1 || info.funds[0].denom != VESTING_DENOM {
        return Err(StdError::generic_err(format!(
            "only {} is allowed to be deposited",
            VESTING_DENOM.clone()
        )));
    }

    // validate vesting schedule with vesting amount
    let vesting_token = info.funds[0].clone();
    msg.vesting_schedule
        .validate(env.block.time.seconds(), vesting_token.amount)?;

    let mut messages: Vec<SubMsg> = vec![];
    let mut attrs: Vec<Attribute> = vec![];
    if msg.enable_staking {
        if msg.staking_info.is_none() {
            return Err(StdError::generic_err(
                "must provide staking_info to enable staking",
            ));
        }

        let staking_info = msg.staking_info.clone().unwrap();
        deps.api.addr_validate(&staking_info.bluna_token)?;
        deps.api.addr_validate(&staking_info.hub_contract)?;
        deps.api.addr_validate(&staking_info.reward_contract)?;

        messages.push(SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: staking_info.hub_contract,
                msg: to_binary(&HubContractExecuteMsg::Bond {
                    validator: staking_info.validator,
                })?,
                funds: info.funds.clone(),
            },
            1,
        ));

        VESTING_INFO.save(
            deps.storage,
            &VestingInfo {
                vesting_denom: Denom::Cw20(Addr::unchecked(&staking_info.bluna_token)),
                vesting_amount: Uint128::zero(), // this will be filled at reply
                vesting_schedule: msg.vesting_schedule,
                claimed_amount: Uint128::zero(),
            },
        )?;
    } else {
        attrs.extend(
            vec![
                ("action", "create_vesting_account"),
                ("owner_address", &msg.owner_address),
                (
                    "vesting_denom",
                    &to_string(&Denom::Native(VESTING_DENOM.to_string())).unwrap(),
                ),
                ("vesting_amount", &vesting_token.amount.to_string()),
            ]
            .into_iter()
            .map(|v| v.into()),
        );

        VESTING_INFO.save(
            deps.storage,
            &VestingInfo {
                vesting_denom: Denom::Native(VESTING_DENOM.to_string()),
                vesting_amount: vesting_token.amount,
                vesting_schedule: msg.vesting_schedule,
                claimed_amount: Uint128::zero(),
            },
        )?;
    }

    // store config
    CONFIG.save(
        deps.storage,
        &Config {
            owner_address: msg.owner_address,
            staking_enabled: msg.enable_staking,
            staking_info: msg.staking_info,
        },
    )?;

    Ok(Response::new()
        .add_attributes(attrs)
        .add_submessages(messages))
}

/// This will check converted bluna amount and set
/// the amount as vesting amount.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    if msg.id != 1 {
        return Err(StdError::generic_err("unauthorized"));
    }

    let config: Config = CONFIG.load(deps.storage)?;
    let staking_info = config.staking_info.unwrap();
    let bluna_token = staking_info.bluna_token;
    let response: BalanceResponse = deps.querier.query_wasm_smart(
        bluna_token.to_string(),
        &Cw20QueryMsg::Balance {
            address: env.contract.address.to_string(),
        },
    )?;

    VESTING_INFO.update(deps.storage, |mut v| -> StdResult<_> {
        v.vesting_amount = response.balance;
        Ok(v)
    })?;

    Ok(Response::new().add_attributes(vec![
        ("action", "create_vesting_account"),
        ("owner_address", &config.owner_address),
        (
            "vesting_denom",
            &to_string(&Denom::Cw20(Addr::unchecked(bluna_token))).unwrap(),
        ),
        ("vesting_amount", &response.balance.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ChangeOwner { new_owner } => change_owner(deps, info, new_owner),
        ExecuteMsg::Claim { recipient } => claim(deps, env, info, recipient),
        ExecuteMsg::ClaimRewards { recipient } => claim_rewards(deps, env, info, recipient),
    }
}

fn change_owner(deps: DepsMut, info: MessageInfo, new_owner: String) -> StdResult<Response> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if config.owner_address != info.sender.to_string() {
        return Err(StdError::generic_err("unauthorized"));
    }

    config.owner_address = new_owner.to_string();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "change_owner"),
        ("new_owner", new_owner.as_str()),
    ]))
}

fn claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> StdResult<Response> {
    let sender = info.sender;
    let recipient = recipient.unwrap_or_else(|| sender.to_string());

    // permission check
    let config: Config = CONFIG.load(deps.storage)?;
    if config.owner_address != sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut vesting_info: VestingInfo = VESTING_INFO.load(deps.storage)?;
    let vested_amount = vesting_info
        .vesting_schedule
        .vested_amount(env.block.time.seconds(), vesting_info.vesting_amount)?;
    let claimed_amount = vesting_info.claimed_amount;

    let claimable_amount = vested_amount.checked_sub(claimed_amount)?;
    if claimable_amount.is_zero() {
        return Err(StdError::generic_err("nothing to claim"));
    }

    vesting_info.claimed_amount = vested_amount;
    VESTING_INFO.save(deps.storage, &vesting_info)?;

    // depends on vesting_denom, make native or cw20 transfer message
    let message: CosmosMsg = match vesting_info.vesting_denom.clone() {
        Denom::Native(denom) => BankMsg::Send {
            to_address: recipient.clone(),
            amount: vec![Coin {
                denom,
                amount: claimable_amount,
            }],
        }
        .into(),
        Denom::Cw20(contract_addr) => WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount: claimable_amount,
            })?,
            funds: vec![],
        }
        .into(),
    };

    Ok(Response::new()
        .add_message(message)
        .add_attributes(vec![("action", "claim"), ("recipient", recipient.as_str())])
        .add_attributes(vec![
            (
                "vesting_denom",
                &to_string(&vesting_info.vesting_denom).unwrap(),
            ),
            ("vesting_amount", &vesting_info.vesting_amount.to_string()),
            ("vested_amount", &vested_amount.to_string()),
            ("claim_amount", &claimable_amount.to_string()),
        ]))
}

fn claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
) -> StdResult<Response> {
    let sender = info.sender;
    let recipient = recipient.unwrap_or_else(|| sender.to_string());

    // permission check
    let config: Config = CONFIG.load(deps.storage)?;
    if config.owner_address != sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    if !config.staking_enabled {
        return Err(StdError::generic_err("staking disabled"));
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    // check reward_denom balance
    let existing_rewards: Coin = deps
        .querier
        .query_balance(env.contract.address.to_string(), REWARDS_DENOM)?;
    if !existing_rewards.amount.is_zero() {
        messages.push(
            BankMsg::Send {
                to_address: recipient.clone(),
                amount: vec![existing_rewards],
            }
            .into(),
        );
    }

    // check reward_contract rewards
    let staking_info = config.staking_info.unwrap();
    let response: AccruedRewardsResponse = deps.querier.query_wasm_smart(
        Addr::unchecked(staking_info.reward_contract.to_string()),
        &RewardContractQueryMsg::AccruedRewards {
            address: env.contract.address.to_string(),
        },
    )?;
    if !response.rewards.is_zero() {
        messages.push(
            WasmMsg::Execute {
                contract_addr: staking_info.reward_contract,
                msg: to_binary(&RewardContractExecuteMsg::ClaimRewards {
                    recipient: Some(recipient),
                })?,
                funds: vec![],
            }
            .into(),
        );
    }

    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![("action", "claim_rewards")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VestingInfo {} => to_binary(&vesting_account(deps, env)?),
    }
}

fn vesting_account(deps: Deps, env: Env) -> StdResult<VestingInfoResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    let vesting_info: VestingInfo = VESTING_INFO.load(deps.storage)?;

    let vested_amount = vesting_info
        .vesting_schedule
        .vested_amount(env.block.time.seconds(), vesting_info.vesting_amount)?;
    let claimable_amount = vested_amount.checked_sub(vesting_info.claimed_amount)?;

    let mut claimable_staking_rewards: Uint128 = Uint128::zero();
    if config.staking_enabled {
        let staking_info = config.staking_info.unwrap();

        let existing_rewards: Coin = deps
            .querier
            .query_balance(env.contract.address.to_string(), REWARDS_DENOM)?;
        claimable_staking_rewards += existing_rewards.amount;

        let response: AccruedRewardsResponse = deps.querier.query_wasm_smart(
            Addr::unchecked(staking_info.reward_contract),
            &RewardContractQueryMsg::AccruedRewards {
                address: env.contract.address.to_string(),
            },
        )?;
        claimable_staking_rewards += response.rewards;
    }

    Ok(VestingInfoResponse {
        owner_address: config.owner_address,
        vesting_denom: vesting_info.vesting_denom,
        vesting_amount: vesting_info.vesting_amount,
        vested_amount,
        vesting_schedule: vesting_info.vesting_schedule,
        claimable_amount,
        claimable_staking_rewards,
    })
}
