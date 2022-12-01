use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};

use crate::{
    query,
    state::{BIDS, CONFIG, WINNER},
    ContractError,
};

pub fn bid(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // contract owner cannot bid
    if info.sender == cfg.contract_owner {
        return Err(ContractError::Unauthorized {});
    }

    let funds = info.funds.clone();

    // sent too little or too many tokens
    if funds.len() != 1 {
        return Err(ContractError::TooManyOrLittleNativeTokensSent {});
    }

    // TODO: this should use a query
    let highest_bid = query::highest_bid(deps.as_ref())?;
    // let highest_bid = BIDS.load(deps.storage, cfg.contract_owner)?;

    // need to bid in the initial bidder denom
    if funds[0].denom != cfg.denom {
        return Err(ContractError::WrongDenom {});
    }
    // need to bid higher than highest bid
    if funds[0].amount <= highest_bid.total_bid {
        return Err(ContractError::BidTooLow {});
    }

    let winner = WINNER.load(deps.storage)?;

    if winner.0 == info.sender {
        return Err(ContractError::YouAreTheHighestBidder {});
    }

    let bid = BIDS
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(Uint128::from(0u8))
        + funds[0].amount;

    // save winner to state
    WINNER.save(deps.storage, &(info.sender.clone(), bid))?;

    // add new bid to bids map
    BIDS.save(deps.storage, winner.0, &winner.1)?;

    Ok(Response::new()
        .add_attribute("action", "bid")
        .add_attribute("highest_bidder", info.sender)
        .add_attribute("highest_bid", info.funds[0].amount))
}

pub fn retract(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _receiver: Option<String>,
) -> Result<Response, ContractError> {
    unimplemented!()
}
