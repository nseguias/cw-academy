use cosmwasm_std::{Deps, StdResult};

use crate::{
    msg::{HighestBidderResponse, TotalBidResponse},
    state::{BIDS, WINNER},
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
