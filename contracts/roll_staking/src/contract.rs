use cosmwasm_std::{
    from_slice, log, to_binary, to_vec, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg,
    Decimal, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError, StdResult,
    Storage, Uint128, WasmMsg,
};

use crate::msg::{
    ConfigResponse, HandleMsg, InitMsg, QueryMsg, RollResponse, StakerResponse, TokenHandleMsg,
};

use crate::state::{
    config_read, config_store, load_all_rolls, roll_read, roll_store, staker_read, staker_store,
    ConfigState, RollState, StakerState,
};

use terra_cosmwasm::TerraQuerier;

use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let api = deps.api;
    let state = ConfigState {
        owner: api.canonical_address(&env.message.sender)?,
        staking_token: api.canonical_address(&msg.staking_token)?,
        roll_unit: msg.roll_unit,
        deposit_period: msg.deposit_period.u128().try_into().unwrap(),
        rewards_denom: msg.rewards_denom,
    };

    config_store(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateConfig {
            owner,
            deposit_period,
        } => try_update_config(deps, env, owner, deposit_period),
        HandleMsg::Deposit { amount } => try_deposit(deps, env, amount),
        HandleMsg::Withdraw { amount } => try_withdraw(deps, env, amount),
        HandleMsg::Claim {} => try_claim(deps, env),
        HandleMsg::Distribute { amount } => try_distribute(deps, env, amount),
    }
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    deposit_period: Option<Uint128>,
) -> StdResult<HandleResponse> {
    let api = deps.api;
    config_store(&mut deps.storage).update(|mut state| {
        if api.canonical_address(&env.message.sender)? != state.owner {
            return Err(StdError::unauthorized());
        }

        if let Some(owner) = owner {
            state.owner = api.canonical_address(&owner)?;
        }

        if let Some(deposit_period) = deposit_period {
            state.deposit_period = deposit_period.u128().try_into().unwrap();
        }

        Ok(state)
    })?;

    Ok(HandleResponse::default())
}

// CONTRACT: a user must do call Allow msg in token contract
pub fn try_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    if amount.is_zero() {
        return Err(StdError::generic_err("invalid amount"));
    }

    let staker_addr_raw: CanonicalAddr = deps.api.canonical_address(&env.message.sender)?;

    let config: ConfigState = config_read(&deps.storage).load()?;
    let staker: StakerState = staker_read(&deps.storage, &staker_addr_raw)?;

    // check increased number of rolls
    let before_number_of_roll: u32 = (staker.balance.u128() / config.roll_unit.u128())
        .try_into()
        .unwrap();

    let after_number_of_roll: u32 = ((staker.balance.u128() + amount.u128())
        / config.roll_unit.u128())
    .try_into()
    .unwrap();

    for i in before_number_of_roll..after_number_of_roll {
        roll_store(&mut deps.storage).set(
            &[staker_addr_raw.as_slice(), &i.to_be_bytes()].concat(),
            &to_vec(&RollState {
                owner: staker_addr_raw.clone(),
                creation_time: env.block.time,
            })?,
        );
    }

    let balance: Uint128 = staker.balance + amount;
    let number_of_rolls: Uint128 = Uint128(balance.u128() / config.roll_unit.u128());
    staker_store(&mut deps.storage).set(
        staker_addr_raw.as_slice(),
        &to_vec(&StakerState { balance, ..staker })?,
    );

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&config.staking_token)?,
            send: vec![],
            msg: to_binary(&TokenHandleMsg::TransferFrom {
                owner: env.message.sender.clone(),
                recipient: env.contract.address,
                amount,
            })?,
        })],
        log: vec![
            log("action", "deposit"),
            log("balance", &balance.to_string()),
            log("number_of_rolls", &number_of_rolls.to_string()),
        ],
        data: None,
    })
}

pub fn try_withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    if amount.is_zero() {
        return Err(StdError::generic_err("invalid amount"));
    }

    let staker_addr_raw: CanonicalAddr = deps.api.canonical_address(&env.message.sender)?;

    let config: ConfigState = config_read(&deps.storage).load()?;
    let staker: StakerState = staker_read(&deps.storage, &staker_addr_raw)?;

    // check decreased number of rolls
    let before_number_of_roll: u32 = (staker.balance.u128() / config.roll_unit.u128())
        .try_into()
        .unwrap();

    let after_number_of_roll: u32 = ((staker.balance.u128() - amount.u128())
        / config.roll_unit.u128())
    .try_into()
    .unwrap();

    // remove rolls in DESC(index) order
    for i in after_number_of_roll..before_number_of_roll {
        roll_store(&mut deps.storage)
            .remove(&[staker_addr_raw.as_slice(), &i.to_be_bytes()].concat());
    }

    let balance: Uint128 = (staker.balance - amount)?;
    let number_of_rolls: Uint128 = Uint128(balance.u128() / config.roll_unit.u128());
    staker_store(&mut deps.storage).set(
        staker_addr_raw.as_slice(),
        &to_vec(&StakerState { balance, ..staker })?,
    );

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&config.staking_token)?,
            send: vec![],
            msg: to_binary(&TokenHandleMsg::Transfer {
                recipient: env.message.sender,
                amount,
            })?,
        })],
        log: vec![
            log("action", "withdraw"),
            log("balance", &balance.to_string()),
            log("number_of_rolls", &number_of_rolls.to_string()),
        ],
        data: None,
    })
}

pub fn try_claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let staker_addr_raw: CanonicalAddr = deps.api.canonical_address(&env.message.sender)?;

    let config: ConfigState = config_read(&deps.storage).load()?;
    let staker: StakerState = staker_read(&deps.storage, &staker_addr_raw)?;

    let mut msgs: Vec<CosmosMsg> = vec![];
    let mut claimable_amount: Uint128 = Uint128::zero();
    let mut tax_amount: Uint128 = Uint128::zero();
    if !staker.collected_rewards.is_zero() {
        let collected_rewards_string = staker.collected_rewards.to_string();
        let rewards: Vec<&str> = collected_rewards_string.split('.').collect();
        let integer_part: Uint128 = Uint128::try_from(rewards[0])?;
        let decimal_part: Decimal = if rewards.len() == 1 {
            Decimal::zero()
        } else {
            Decimal::from_str(&("0.".to_string() + rewards[1]))?
        };

        claimable_amount = integer_part;

        let querier = TerraQuerier::new(&deps.querier);
        let tax_rate = querier.query_tax_rate()?.rate;
        let tax_cap = querier.query_tax_cap(&config.rewards_denom)?.cap;
        tax_amount = std::cmp::min(tax_rate * claimable_amount, tax_cap);
        claimable_amount = (claimable_amount - tax_amount)?;

        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: env.message.sender,
            amount: vec![Coin {
                denom: config.rewards_denom.to_string(),
                amount: claimable_amount,
            }],
        }));

        staker_store(&mut deps.storage).set(
            staker_addr_raw.as_slice(),
            &to_vec(&StakerState {
                collected_rewards: decimal_part,
                ..staker
            })?,
        );
    }

    Ok(HandleResponse {
        messages: msgs,
        log: vec![
            log("action", "claim"),
            log(
                "rewards",
                claimable_amount.to_string() + config.rewards_denom.as_str(),
            ),
            log(
                "tax",
                tax_amount.to_string() + config.rewards_denom.as_str(),
            ),
        ],
        data: None,
    })
}

pub fn try_distribute<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    if amount.is_zero() {
        return Err(StdError::generic_err("Invalid amount"));
    }

    let config: ConfigState = config_read(&deps.storage).load()?;
    if config.owner != deps.api.canonical_address(&env.message.sender)? {
        return Err(StdError::unauthorized());
    }

    let rolls_kv = load_all_rolls(&deps.storage);
    let mut rolls: Vec<RollState> = vec![];
    for item in rolls_kv {
        let roll: RollState = from_slice(&item.1)?;
        if roll.creation_time + config.deposit_period < env.block.time {
            rolls.push(roll);
        }
    }

    let number_of_rolls: u32 = rolls.len() as u32;
    if number_of_rolls == 0u32 {
        return Err(StdError::generic_err("No rolls registered"));
    }

    let rewards_per_roll: Decimal = Decimal::from_ratio(amount, number_of_rolls);

    let mut staker: StakerState = Default::default();
    for roll in rolls {
        if !staker.address.is_empty() && staker.address != roll.owner {
            staker_store(&mut deps.storage).set(
                &staker.address.clone().as_slice(),
                &to_vec(&StakerState {
                    collected_rewards: staker.collected_rewards,
                    ..staker
                })?,
            );

            staker = Default::default();
        }

        if staker.address.is_empty() {
            staker = staker_read(&deps.storage, &roll.owner)?;
            staker.address = roll.owner;
        }

        staker.collected_rewards = staker.collected_rewards + rewards_per_roll;
    }

    let staker_address = staker.address.clone();
    if !staker_address.is_empty() {
        staker_store(&mut deps.storage).set(
            &staker_address.as_slice(),
            &to_vec(&StakerState {
                collected_rewards: staker.collected_rewards,
                ..staker
            })?,
        );
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "distribute"),
            log(
                "rewards_per_roll",
                rewards_per_roll.to_string() + config.rewards_denom.as_str(),
            ),
            log("number_of_rolls", number_of_rolls.to_string()),
        ],
        data: None,
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Staker { address } => to_binary(&query_staker(deps, address)?),
        QueryMsg::Roll { address, index } => to_binary(&query_roll(deps, address, index)?),
    }
}

pub fn query_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ConfigResponse> {
    let state: ConfigState = config_read(&deps.storage).load()?;
    let resp = ConfigResponse {
        owner: deps.api.human_address(&state.owner)?,
        staking_token: deps.api.human_address(&state.staking_token)?,
        roll_unit: state.roll_unit,
        deposit_period: state.deposit_period,
        rewards_denom: state.rewards_denom,
    };

    Ok(resp)
}

pub fn query_staker<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<StakerResponse> {
    let state: StakerState = staker_read(&deps.storage, &deps.api.canonical_address(&address)?)?;
    let resp = StakerResponse {
        address: deps.api.human_address(&state.address)?,
        balance: state.balance,
        collected_rewards: state.collected_rewards,
    };

    Ok(resp)
}

pub fn query_roll<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
    index: u32,
) -> StdResult<RollResponse> {
    let state: RollState = roll_read(&deps.storage, &deps.api.canonical_address(&address)?, index)?;
    let resp = RollResponse {
        owner: deps.api.human_address(&state.owner)?,
        creation_time: state.creation_time,
    };

    Ok(resp)
}
