use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, BIDS, CONFIG};
use crate::{execute, query};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:bidding-platform";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// commission rate is 0.5%
const COMMISSION_RATE: &str = "0.005";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let contract_owner = msg.contract_owner.unwrap_or(info.sender.to_string());
    let commission = msg.commision.unwrap_or(COMMISSION_RATE.to_string());
    let val_contract_owner = deps.api.addr_validate(&contract_owner)?;

    let cfg = Config {
        commodity: msg.commodity,
        contract_owner: val_contract_owner.clone(),
        commission: Decimal::from_str(&commission)?,
        highest_bidder: val_contract_owner,
    };
    CONFIG.save(deps.storage, &cfg)?;

    // bid must be instantiated with only one kind of native tokens
    if info.funds.len() != 1 {
        return Err(ContractError::TooManyOrLittleNativeTokensSent {});
    }

    // contract owner is always the initial bidder and not the sender when they're different
    BIDS.save(deps.storage, cfg.contract_owner, &info.funds[0])?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bid {} => execute::bid(deps, _env, info),
        ExecuteMsg::Retract { receiver } => execute::retract(deps, _env, info, receiver),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // TODO: convert to binary
        QueryMsg::HighestBidder {} => &query::highest_bid(deps),
        QueryMsg::TotalBid { address: _ } => todo!(),
    };
    unimplemented!()
}

#[cfg(test)]
mod tests {}
