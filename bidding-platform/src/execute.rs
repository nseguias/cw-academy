use cosmwasm_std::{coins, BankMsg, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::{
    query,
    state::{BidStatus, BIDS, CONFIG, WINNER},
    ContractError,
};

const DECIMALS: u32 = 6;

pub fn bid(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // contract owner cannot bid
    if info.sender == cfg.contract_owner {
        return Err(ContractError::Unauthorized {});
    }

    // can only bid on open bids
    if cfg.status != BidStatus::Opened {
        return Err(ContractError::BidClosed {});
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

    // bid without commision
    let gross_bid = funds[0].amount;

    // commission
    let commission =
        gross_bid * Uint128::from(cfg.commission) / (Uint128::from(10u128.pow(DECIMALS)));

    // bid including commission
    let net_bid = gross_bid - commission;

    // bid (minos commision) has to be higher than highest bid
    if net_bid <= highest_bid.total_bid {
        return Err(ContractError::BidTooLow {});
    }

    let winner = WINNER.load(deps.storage)?;

    // winner shouldn't be allowed to bid himself
    if winner.0 == info.sender {
        return Err(ContractError::YouAreTheHighestBidder {});
    }

    // create bank message to be sent to contract owner with the commision paid by the bidder
    let bank_msg = BankMsg::Send {
        to_address: cfg.contract_owner.to_string(),
        amount: coins(commission.into(), cfg.denom),
    };

    // save new winner to state
    WINNER.save(deps.storage, &(info.sender.clone(), net_bid))?;

    // add new bid to bids map
    BIDS.save(deps.storage, info.sender.clone(), &net_bid)?;

    Ok(Response::new()
        .add_attribute("action", "bid")
        .add_attribute("highest_bidder", info.sender)
        .add_attribute("highest_bid", net_bid)
        .add_message(bank_msg))
}

pub fn close(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;

    // only contract owner can close bid
    if info.sender != cfg.contract_owner {
        return Err(ContractError::Unauthorized {});
    }

    // cannot close a bid that's not open
    if cfg.status != BidStatus::Opened {
        return Err(ContractError::BidClosed {});
    }

    // update status to Closed and save to storage
    cfg.status = BidStatus::Closed;
    CONFIG.save(deps.storage, &cfg)?;

    // // create bank message to send winning bid tokens to contract owner
    // let winner = WINNER.load(deps.storage)?;

    // let bank_msg = BankMsg::Send {
    //     to_address: cfg.contract_owner.to_string(),
    //     amount: coins(winner.1.into(), cfg.denom),
    // };

    let bid = BIDS.load(deps.storage, info.sender.clone())?;

    // create bank message to send contract owner tokens to their address
    let bank_msg = BankMsg::Send {
        to_address: cfg.contract_owner.to_string(),
        amount: coins(bid.into(), cfg.denom),
    };

    // winner balance is set to 0 so he won't be able to retract any funds
    BIDS.save(deps.storage, info.sender, &Uint128::zero())?;

    // TODO: here goes the logic to transfer commodity IRL to the bid winner

    Ok(Response::new().add_message(bank_msg))
}

pub fn retract(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    receiver: Option<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // users cannot retract any bids until the bid is closed by the owner
    if cfg.status != BidStatus::Closed {
        return Err(ContractError::BidStillOpen {});
    }

    let winner = WINNER.load(deps.storage)?;
    if winner.0 == info.sender {
        return Err(ContractError::WinnerCannotRetractBid {});
    }

    let bid = BIDS
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(Uint128::zero());

    if bid == Uint128::zero() {
        return Err(ContractError::NothingToRetract {});
    };
    let recipient = receiver.unwrap_or(info.sender.to_string());

    // create bank message to send funds to losing bidders (minus commissions)
    let bank_msg = BankMsg::Send {
        to_address: recipient,
        amount: coins(bid.into(), cfg.denom),
    };

    // set bid balance to 0 so users cannot retract funds more than once
    BIDS.save(deps.storage, info.sender, &Uint128::zero())?;

    Ok(Response::new().add_message(bank_msg))
}
