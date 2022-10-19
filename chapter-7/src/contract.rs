use crate::msg::InstantiateMsg;
use crate::state::{COUNTER, MIN_DONATION};
use cosmwasm_std::{DepsMut, Response, StdResult};

pub fn instantiate(deps: DepsMut, msg: InstantiateMsg) -> StdResult<Response> {
    COUNTER.save(deps.storage, &0)?;
    MIN_DONATION.save(deps.storage, &msg.min_donation)?;

    Ok(Response::new())
}

pub mod execute {
    use crate::state::{COUNTER, MIN_DONATION};
    use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};
    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let min_donation = MIN_DONATION.load(deps.storage)?;

        let counter = COUNTER.load(deps.storage)?;
        if info
            .funds
            .iter()
            .any(|coin| coin.denom == min_donation.denom && coin.amount >= min_donation.amount)
        {
            COUNTER.save(deps.storage, &(counter + 1))?;
        } else {
        }
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
