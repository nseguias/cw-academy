use cosmwasm_std::{Deps, StdResult};

use crate::{msg::HighestBidderResponse, state::WINNER};

pub fn highest_bid(deps: Deps) -> StdResult<HighestBidderResponse> {
    // if maps is small, might be cleaner to iterate through the bids and collect the highest.
    let winner = WINNER.load(deps.storage)?;

    Ok(HighestBidderResponse {
        addr: winner.0.to_string(),
        total_bid: winner.1.to_owned(),
    })
}
