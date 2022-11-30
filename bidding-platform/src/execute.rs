use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    state::{BIDS, CONFIG},
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

    let highest_bid = BIDS.load(deps.storage, cfg.contract_owner)?;

    // need to bid in the initial bidder denom
    if funds[0].denom != highest_bid.denom {
        return Err(ContractError::WrongDenom {});
    }
    // need to bid higher than highest bid
    if funds[0].amount <= highest_bid.amount {
        return Err(ContractError::BidTooLow {});
    }

    Ok(Response::new())
}

pub fn retract(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _receiver: Option<String>,
) -> Result<Response, ContractError> {
    unimplemented!()
}
