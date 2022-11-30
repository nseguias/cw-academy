use cosmwasm_std::{Deps, StdResult};

use crate::{
    msg::HighestBidderResponse,
    state::{BIDS, CONFIG},
};

pub fn highest_bid(deps: Deps) -> StdResult<HighestBidderResponse> {
    let highest_bidder = CONFIG.load(deps.storage)?.highest_bidder;

    // TODO: probably cleaner to iterate through the bids and collect the highest?
    let highest_bid = BIDS.load(deps.storage, highest_bidder.clone())?.amount;
    Ok(HighestBidderResponse {
        addr: highest_bidder.to_string(),
        total_bid: highest_bid,
    })
}
