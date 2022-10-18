use crate::state::COUNTER;
use cosmwasm_std::{DepsMut, Response, StdResult};

pub fn instantiate(deps: DepsMut) -> StdResult<Response> {
    COUNTER.save(deps.storage, &0)?;
    Ok(Response::new())
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
