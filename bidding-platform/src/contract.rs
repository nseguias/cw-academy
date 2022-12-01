use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, BIDS, CONFIG, WINNER};
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

    // bid must be instantiated with only one kind of native tokens
    if info.funds.len() != 1 {
        return Err(ContractError::TooManyOrLittleNativeTokensSent {});
    }

    let contract_owner = msg.contract_owner.unwrap_or(info.sender.to_string());
    let commission = msg.commision.unwrap_or(COMMISSION_RATE.to_string());
    let val_contract_owner = deps.api.addr_validate(&contract_owner)?;

    let cfg = Config {
        commodity: msg.commodity,
        contract_owner: val_contract_owner.clone(),
        commission: Decimal::from_str(&commission)?,
        denom: info.funds[0].denom.clone(),
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
        ExecuteMsg::Retract { receiver } => execute::retract(deps, _env, info, receiver),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // TODO: convert to binary
        QueryMsg::HighestBidder {} => to_binary(&query::highest_bid(deps)?),
        QueryMsg::TotalBid { address: _ } => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        contract::{execute, instantiate, query},
        msg::{ExecuteMsg, InstantiateMsg},
        ContractError,
    };
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Uint128,
    };

    // Two fake addresses we will use to mock_info
    pub const ADDR1: &str = "addr1";
    pub const ADDR2: &str = "addr2";
    pub const ADDR3: &str = "addr3";
    pub const COMMODITY: &str = "gold";
    pub const DENOM: &str = "uatom";
    const COMMISSION_RATE: &str = "0.005";

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[coin(100, DENOM)]);
        let msg = InstantiateMsg {
            commodity: COMMODITY.to_string(),
            contract_owner: None,
            commision: None,
        };
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes.len(), 5);
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "instantiate"),
                attr("commodity", COMMODITY),
                attr("contract_owner", ADDR1),
                attr("commission", COMMISSION_RATE),
                attr("denom", DENOM)
            ]
        );
    }

    #[test]
    fn test_bid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[coin(100, DENOM)]);
        let msg = InstantiateMsg {
            commodity: "gold".to_string(),
            contract_owner: None,
            commision: None,
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let execute_msg = ExecuteMsg::Bid {};

        // bids with less amount than highest bid should fail
        let execute_info = mock_info(ADDR2, &[coin(1, DENOM)]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info,
            execute_msg.clone(),
        );
        assert_eq!(res.unwrap_err(), ContractError::BidTooLow {});

        // bids with equal amount than highest bid should fail
        let execute_info = mock_info(ADDR2, &[coin(100, DENOM)]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info,
            execute_msg.clone(),
        );
        assert_eq!(res.unwrap_err(), ContractError::BidTooLow {});

        // bids with equal amount than highest bid should fail
        let execute_info = mock_info(ADDR2, &[coin(100, DENOM)]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info,
            execute_msg.clone(),
        );
        assert_eq!(res.unwrap_err(), ContractError::BidTooLow {});

        // bids with higher amount than highest bid should work
        let execute_info = mock_info(ADDR2, &[coin(101, DENOM)]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info,
            execute_msg.clone(),
        );
        assert_eq!(
            res.unwrap().attributes,
            vec![
                attr("action", "bid"),
                attr("highest_bidder", ADDR2),
                attr("highest_bid", "101")
            ]
        );

        // query highest bidder should return new bidder addr2 & 101
        let res = query::highest_bid(deps.as_ref()).unwrap();
        assert_eq!(res.total_bid, Uint128::from(101u8));
        assert_eq!(res.addr, ADDR2);

        // bids with wrong denom should fail
        let execute_info = mock_info(ADDR2, &[coin(101, "wrong_denom")]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info,
            execute_msg.clone(),
        );
        assert_eq!(res.unwrap_err(), ContractError::WrongDenom {});

        // bid again with higher amount than highest bid should work
        let execute_info = mock_info(ADDR3, &[coin(500, DENOM)]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info,
            execute_msg.clone(),
        );
        assert_eq!(
            res.unwrap().attributes,
            vec![
                attr("action", "bid"),
                attr("highest_bidder", ADDR3),
                attr("highest_bid", "500")
            ]
        );

        // query highest bidder should return new bidder addr3 & 500
        let res = query::highest_bid(deps.as_ref()).unwrap();
        assert_eq!(res.total_bid, Uint128::from(500u32));
        assert_eq!(res.addr, ADDR3);
    }
}
