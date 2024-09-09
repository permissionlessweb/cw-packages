use cosmwasm_std::{entry_point, DepsMut, Empty, Env, MessageInfo, Never, Response};

#[entry_point]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: Empty) -> Result<Response, Never> {
    Ok(Response::new())
}
