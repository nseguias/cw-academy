use cosmwasm_std::{Deps, StdError, StdResult};

use crate::{
    msg::{BidWinnerResponse, HighestBidderResponse, IsBidClosedResponse, TotalBidResponse},
    state::{BidStatus, BIDS, CONFIG, WINNER},
};

pub fn highest_bid(deps: Deps) -> StdResult<HighestBidderResponse> {
    // if maps is small, might be cleaner to iterate through the bids and collect the highest.
    let winner = WINNER.load(deps.storage)?;

    Ok(HighestBidderResponse {
        addr: winner.0.to_string(),
        total_bid: winner.1.to_owned(),
    })
}

pub fn total_bid(deps: Deps, addr: String) -> StdResult<TotalBidResponse> {
    // if maps is small, might be cleaner to iterate through the bids and collect the highest.
    let bid = BIDS.load(deps.storage, deps.api.addr_validate(&addr)?)?;

    Ok(TotalBidResponse { total_bid: bid })
}

pub fn is_closed(deps: Deps) -> StdResult<IsBidClosedResponse> {
    let cfg = CONFIG.load(deps.storage)?;

    match cfg.status {
        BidStatus::Closed => Ok(IsBidClosedResponse { is_closed: true }),
        BidStatus::Opened => Ok(IsBidClosedResponse { is_closed: false }),
    }
}

pub fn bid_winner(deps: Deps) -> StdResult<BidWinnerResponse> {
    let cfg = CONFIG.load(deps.storage)?;

    // return error if bid is not closed
    if cfg.status != BidStatus::Closed {
        return Err(StdError::generic_err("Bid is not closed"));
    }

    let winner = WINNER.load(deps.storage)?;
    Ok(BidWinnerResponse {
        winner: winner.0.to_string(),
    })
}
