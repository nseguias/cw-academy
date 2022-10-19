use crate::state::COUNTER;
use cosmwasm_std::{DepsMut, Response, StdResult};

pub fn instantiate(deps: DepsMut) -> StdResult<Response> {
    COUNTER.save(deps.storage, &0)?;
    Ok(Response::new())
}

pub mod execute {
    use crate::state::COUNTER;
    use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};
    pub fn increment(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let counter = COUNTER.load(deps.storage)? + 1;
        COUNTER.save(deps.storage, &counter)?;
        let res: Response = Response::new()
            .add_attribute("action", "incerment")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", counter.to_string());
        Ok(res)
    }
}

pub mod query {
    use crate::msg::ValueResponse;
    use crate::state::COUNTER;
    use cosmwasm_std::{Deps, StdResult};

    pub fn value(deps: Deps) -> StdResult<ValueResponse> {
        let counter = COUNTER.load(deps.storage)?;
        Ok(ValueResponse { value: counter })
    }
}
