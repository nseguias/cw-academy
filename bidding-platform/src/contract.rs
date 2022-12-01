#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{BidStatus, Config, BIDS, CONFIG, WINNER};
use crate::{execute, query};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:bidding-platform";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// commission rate is 0.1% assuming we're using 6 decimals
const COMMISSION_RATE: u128 = 1000u128;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // bid must be instantiated with only one kind of native tokens
    if info.funds.len() != 1 {
        return Err(ContractError::TooManyOrLittleNativeTokensSent {});
    }

    let contract_owner = msg.contract_owner.unwrap_or(info.sender.to_string());
    let commission = msg.commision.unwrap_or(COMMISSION_RATE);
    let val_contract_owner = deps.api.addr_validate(&contract_owner)?;

    let cfg = Config {
        commodity: msg.commodity,
        contract_owner: val_contract_owner.clone(),
        commission: commission,
        denom: info.funds[0].denom.clone(),
        status: BidStatus::Opened,
    };

    CONFIG.save(deps.storage, &cfg)?;

    WINNER.save(
        deps.storage,
        &(cfg.contract_owner.clone(), info.funds[0].amount),
    )?;

    // contract owner is always the initial bidder and not the sender when they're different
    BIDS.save(
        deps.storage,
        cfg.contract_owner.clone(),
        &info.funds[0].amount.into(),
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("commodity", cfg.commodity)
        .add_attribute("contract_owner", cfg.contract_owner)
        .add_attribute("commission", cfg.commission.to_string())
        .add_attribute("denom", cfg.denom))
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
        ExecuteMsg::Close {} => execute::close(deps, _env, info),
        ExecuteMsg::Retract { receiver } => execute::retract(deps, _env, info, receiver),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::HighestBidder {} => to_binary(&query::highest_bid(deps)?),
        QueryMsg::TotalBid { address: addr } => to_binary(&query::total_bid(deps, addr)?),
        QueryMsg::IsBidClosed {} => to_binary(&query::is_closed(deps)?),
        QueryMsg::BidWinner {} => to_binary(&query::bid_winner(deps)?),
    }
}
