use cosmwasm_std::{
    attr, to_binary, Binary, CosmosMsg, Env, HandleResponse, HumanAddr, InitResponse, StdResult, 
    Deps, DepsMut, MessageInfo
};

use crate::msg::{
    CustomMsgWrapper, CustomResponse, HandleMsg, InitMsg,
    OwnerResponse, QueryMsg, SpecialQuery,
};

use crate::state::{config, config_read, State};
use crate::errors::MaskError;

pub fn init(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InitMsg,
) -> StdResult<InitResponse<CustomMsgWrapper>> {
    let state = State {
        owner: deps.api.canonical_address(&info.sender)?,
    };

    config(deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse<CustomMsgWrapper>, MaskError> {
    match msg {
        HandleMsg::ReflectMsg { msgs } => try_reflect(deps, env, info, msgs),
        HandleMsg::ChangeOwner { owner } => try_change_owner(deps, env, info, owner),
    }
}

pub fn try_reflect(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<CustomMsgWrapper>>,
) -> Result<HandleResponse<CustomMsgWrapper>, MaskError> {
    let state = config(deps.storage).load()?;
    if deps.api.canonical_address(&info.sender)? != state.owner {
        return Err(MaskError::Unauthorized{});
    }
    if msgs.is_empty() {
        return Err(MaskError::MessagesEmpty{});
    }
    let res = HandleResponse {
        messages: msgs,
        attributes: vec![attr("action", "reflect")],
        data: None,
    };
    Ok(res)
}

pub fn try_change_owner(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: HumanAddr,
) -> Result<HandleResponse<CustomMsgWrapper>, MaskError> {
    let api = deps.api;
    config(deps.storage).update(|mut state| {
        if api.canonical_address(&info.sender)? != state.owner {
            return Err(MaskError::Unauthorized{});
        }
        state.owner = api.canonical_address(&owner)?;
        Ok(state)
    })?;
    Ok(HandleResponse {
        attributes: vec![attr("action", "change_owner"), attr("owner", owner)],
        ..HandleResponse::default()
    })
}

pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Owner {} => to_binary(&query_owner(deps)?),
        QueryMsg::ReflectCustom { text } => to_binary(&query_custom(deps, text)?),
    }
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let state = config_read(deps.storage).load()?;
    let resp = OwnerResponse {
        owner: deps.api.human_address(&state.owner)?,
    };
    Ok(resp)
}

fn query_custom(
    deps: Deps,
    text: String,
) -> StdResult<CustomResponse> {
    let req = SpecialQuery::Capitalized{text}.into();

    deps.querier.custom_query(&req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::CustomMsg;
    use crate::testing::mock_dependencies_with_custom_querier;
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coin, coins, BankMsg, Binary, StakingMsg};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let value = query_owner(deps.as_ref()).unwrap();
        assert_eq!("creator", value.owner.as_str());
    }

    #[test]
    fn reflect() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let payload = vec![BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: HumanAddr::from("friend"),
            amount: coins(1, "token"),
        }
        .into()];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let info = mock_info("creator", &[]);
        let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(payload, res.messages);
    }

    #[test]
    fn reflect_requires_owner() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let payload = vec![BankMsg::Send {
            from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            to_address: HumanAddr::from("friend"),
            amount: coins(1, "token"),
        }
        .into()];
        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let info = mock_info("unauthorized", &[]);
        let err = handle(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, MaskError::Unauthorized);
    }

    #[test]
    fn reflect_reject_empty_msgs() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let payload = vec![];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let info = mock_info("creator", &[]);
        let err = handle(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, MaskError::MessagesEmpty);
    }

    #[test]
    fn reflect_multiple_messages() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let payload = vec![
            BankMsg::Send {
                from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
                to_address: HumanAddr::from("friend"),
                amount: coins(1, "token"),
            }
            .into(),
            // make sure we can pass through custom native messages
            CustomMsgWrapper {
                route: "mask".to_string(),
                msg_data: CustomMsg::Raw(Binary(b"{\"foo\":123}".to_vec())),
            }
            .into(),
            CustomMsgWrapper {
                route: "mask".to_string(),
                msg_data: CustomMsg::Debug("Hi, Dad!".to_string()),
            }
            .into(),
            StakingMsg::Delegate {
                validator: HumanAddr::from("validator"),
                amount: coin(100, "ustake"),
            }
            .into(),
        ];

        let msg = HandleMsg::ReflectMsg {
            msgs: payload.clone(),
        };
        let info = mock_info("creator", &[]);
        let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(payload, res.messages);
    }

    #[test]
    fn transfer() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let new_owner = HumanAddr::from("friend");
        let msg = HandleMsg::ChangeOwner {
            owner: new_owner.clone(),
        };
        let info = mock_info("creator", &[]);
        let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should change state
        assert_eq!(0, res.messages.len());
        let value = query_owner(deps.as_ref()).unwrap();
        assert_eq!("friend", value.owner.as_str());
    }

    #[test]
    fn transfer_requires_owner() {
        let mut deps = mock_dependencies_with_custom_querier(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        let new_owner = HumanAddr::from("friend");
        let msg = HandleMsg::ChangeOwner {
            owner: new_owner.clone(),
        };
        let info = mock_info("unauthorized", &[]);
        let err = handle(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, MaskError::Unauthorized);
    }

    #[test]
    fn dispatch_custom_query() {
        let deps = mock_dependencies_with_custom_querier(&[]);

        // we don't even initialize, just trigger a query
        let value = query_custom(deps.as_ref(), "demo one".to_string()).unwrap();
        assert_eq!(value.msg, "DEMO ONE");
    }
}
