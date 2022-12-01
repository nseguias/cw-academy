use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{StdError, StdResult, Uint128};

#[cw_serde]
#[cfg_attr(test, derive(Default))]
pub struct InstantiateMsg {
    pub commodity: String,
    pub contract_owner: Option<String>,
    pub commision: Option<u128>,
}

impl InstantiateMsg {
    pub fn validate(&self) -> StdResult<()> {
        // validate commodity name
        if !self.has_valid_name() {
            return Err(StdError::generic_err(
                "Commodity name is not in the expected format (3-50 UTF-8 bytes)",
            ));
        }
        Ok(())
    }
    fn has_valid_name(&self) -> bool {
        let bytes = self.commodity.as_bytes();
        if bytes.len() < 3 || bytes.len() > 50 {
            return false;
        }
        true
    }
    // TODO: validate contract owner
    // TODO: validate commission
}

#[cw_serde]
pub enum ExecuteMsg {
    // any user other than the contract owner can raise their bid by sending tokens to the contract with the Bid {} message
    Bid {},
    // only owner can close bid
    Close {},
    // sends all senders bids (minus commissions) to the receiver account (if provided) or to the original bidder
    Retract { receiver: Option<String> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the total bid of the given address, 0 if unset.
    #[returns(TotalBidResponse)]
    TotalBid { address: String },
    /// Returns the highest bidder and their total bid amount.
    #[returns(HighestBidderResponse)]
    HighestBidder {},
    /// Returns true if bid is closed.
    #[returns(IsBidClosedResponse)]
    IsBidClosed {},
    /// Returns the highest bidder and their total bid amount.
    #[returns(BidWinnerResponse)]
    BidWinner {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct TotalBidResponse {
    pub total_bid: Uint128,
}

#[cw_serde]
pub struct HighestBidderResponse {
    pub addr: String,
    pub total_bid: Uint128,
}

#[cw_serde]
pub struct IsBidClosedResponse {
    pub is_closed: bool,
}

#[cw_serde]
pub struct BidWinnerResponse {
    pub winner: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_instantiate_msg_name() {
        // Too short
        let mut msg = InstantiateMsg {
            commodity: str::repeat("a", 2),
            ..InstantiateMsg::default()
        };
        assert!(!msg.has_valid_name());

        // In the correct length range
        msg.commodity = str::repeat("a", 3);
        assert!(msg.has_valid_name());

        // Too long
        msg.commodity = str::repeat("a", 51);
        assert!(!msg.has_valid_name());
    }
}
