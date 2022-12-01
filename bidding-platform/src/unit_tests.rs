#[cfg(test)]
mod tests {
    use crate::{
        contract::{execute, instantiate, query},
        msg::{
            ExecuteMsg, HighestBidderResponse, InstantiateMsg, IsBidClosedResponse, QueryMsg,
            TotalBidResponse,
        },
        ContractError,
    };
    use cosmwasm_std::{
        attr, coin, from_binary,
        testing::{mock_dependencies, mock_env, mock_info},
        BankMsg, CosmosMsg, Uint128,
    };

    // three fake addresses we will use to mock_info
    pub const ADDR1: &str = "addr1";
    pub const ADDR2: &str = "addr2";
    pub const ADDR3: &str = "addr3";
    pub const ADDR4: &str = "addr4";

    pub const COMMODITY: &str = "gold";
    pub const DENOM: &str = "uatom";

    // commision is 0,1% or 0.001 with 6 decimals
    const COMMISSION_RATE: u128 = 1000u128;

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
                attr("commission", COMMISSION_RATE.to_string()),
                attr("denom", DENOM)
            ]
        );
    }

    #[test]
    fn test_bid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[coin(1_000_000, DENOM)]);
        let msg = InstantiateMsg {
            commodity: "gold".to_string(),
            contract_owner: None,
            commision: None,
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let execute_msg = ExecuteMsg::Bid {};

        // bids with less amount than highest bid should fail
        let execute_info = mock_info(ADDR2, &[coin(1_000, DENOM)]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info,
            execute_msg.clone(),
        );
        assert_eq!(res.unwrap_err(), ContractError::BidTooLow {});

        // bids with equal amount than highest bid should fail
        let execute_info = mock_info(ADDR2, &[coin(1_000_000, DENOM)]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info,
            execute_msg.clone(),
        );
        assert_eq!(res.unwrap_err(), ContractError::BidTooLow {});

        // bids with higher amount than highest bid should work
        let execute_info = mock_info(ADDR2, &[coin(10_000_000, DENOM)]);
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
                attr("highest_bid", "9990000")
            ]
        );

        // query highest bidder should return new bidder addr2 & 9990000 (10_000_000 - 10_000)
        let query_msg = QueryMsg::HighestBidder {};
        let res: HighestBidderResponse =
            from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        assert_eq!(res.total_bid, Uint128::from(9990000u128));
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
        let execute_info = mock_info(ADDR3, &[coin(100_000_000, DENOM)]);
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
                attr("highest_bid", "99900000")
            ]
        );

        // query highest bidder should return new bidder addr3 & 500
        let query_msg = QueryMsg::HighestBidder {};
        let res: HighestBidderResponse =
            from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        assert_eq!(res.total_bid, Uint128::from(99900000u32));
        assert_eq!(res.addr, ADDR3);
    }

    #[test]
    fn test_close_bid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[coin(1_000_000, DENOM)]);
        let msg = InstantiateMsg {
            commodity: "gold".to_string(),
            contract_owner: None,
            commision: None,
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let execute_msg = ExecuteMsg::Bid {};

        let execute_info = mock_info(ADDR2, &[coin(10_000_000, DENOM)]);

        // bid with highest amount and unwrap() guarantees success
        execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        // query highest bidder should return new bidder addr2 & 9990000 (10_000_000 - 10_000)
        let query_msg = QueryMsg::HighestBidder {};
        let res: HighestBidderResponse =
            from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        assert_eq!(res.total_bid, Uint128::from(9990000u128));
        assert_eq!(res.addr, ADDR2);

        // closing a bid as a non-contract owner should fail
        let close_bid_msg = ExecuteMsg::Close {};
        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            close_bid_msg.clone(),
        );
        assert_eq!(res.unwrap_err(), ContractError::Unauthorized {});

        // closing a bid as the contract owner should work
        let execute_info = mock_info(ADDR1, &[]);

        // query if bid is closed should return false
        let query_msg = QueryMsg::IsBidClosed {};
        let res: IsBidClosedResponse =
            from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        assert!(!res.is_closed);

        // unwrap guarantees success
        execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            close_bid_msg.clone(),
        )
        .unwrap();

        // closing a bid that's closed as the contract owner should fail
        let res = execute(deps.as_mut(), env.clone(), execute_info, close_bid_msg);
        assert_eq!(res.unwrap_err(), ContractError::BidClosed {});

        // query total bid for ADDR1 after closing should be 0
        let query_msg = QueryMsg::TotalBid {
            address: ADDR1.to_string(),
        };
        let res: TotalBidResponse =
            from_binary(&query(deps.as_ref(), env.clone(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(res.total_bid, Uint128::zero());

        // query total bid for ADDR2 (bid winner) should return 9990000
        let query_msg = QueryMsg::TotalBid {
            address: ADDR2.to_string(),
        };
        let res: TotalBidResponse =
            from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        assert_eq!(res.total_bid, Uint128::from(9990000u128));

        // query highest bidder should still return ADDR2 & 9990000 (10_000_000 - 10_000)
        let query_msg = QueryMsg::HighestBidder {};
        let res: HighestBidderResponse =
            from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        assert_eq!(res.total_bid, Uint128::from(9990000u128));
        assert_eq!(res.addr, ADDR2);

        // query if bid is closed should return true
        let query_msg = QueryMsg::IsBidClosed {};
        let res: IsBidClosedResponse =
            from_binary(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        assert!(res.is_closed);
    }

    #[test]
    fn test_retract_bid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[coin(1_000_000, DENOM)]);
        let msg = InstantiateMsg {
            commodity: "gold".to_string(),
            contract_owner: None,
            commision: None,
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // win bid as ADDR2
        let execute_msg = ExecuteMsg::Bid {};
        let execute_info = mock_info(ADDR2, &[coin(10_000_000, DENOM)]);

        // bid with highest amount and unwrap() guarantees success
        execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        // win bid as ADDR3
        let execute_info = mock_info(ADDR3, &[coin(100_000_000, DENOM)]);

        // bid with highest amount and unwrap() guarantees success
        execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        // win bod as ADDR4
        let execute_info = mock_info(ADDR4, &[coin(1_000_000_000, DENOM)]);

        // bid with highest amount and unwrap() guarantees success
        execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            execute_msg.clone(),
        )
        .unwrap();

        // closing a bid as the contract owner
        let close_bid_msg = ExecuteMsg::Close {};
        let execute_info = mock_info(ADDR1, &[]);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            close_bid_msg.clone(),
        )
        .unwrap();

        // create a bank message for owner
        let bank_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: ADDR1.to_string(),
            amount: vec![coin(1_000_000u128.into(), DENOM)],
        });
        assert_eq!(res.messages[0].msg, bank_msg);

        // retracting bid for ADDR2 without passing receiver should work
        let retract_bid_msg = ExecuteMsg::Retract { receiver: None };
        let execute_info = mock_info(ADDR2, &[]);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            retract_bid_msg.clone(),
        )
        .unwrap();

        // create a bank message for ADDR2
        let bank_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: ADDR2.to_string(),
            amount: vec![coin(9_990_000u128.into(), DENOM)],
        });
        assert_eq!(res.messages[0].msg, bank_msg);

        // retracting bid for ADDR2 (again) should fail
        let execute_info = mock_info(ADDR2, &[]);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            retract_bid_msg.clone(),
        );

        assert_eq!(res.unwrap_err(), ContractError::NothingToRetract {});

        // retracting bid as contract owner without passing receiver should fail (balance 0 already)
        let execute_info = mock_info(ADDR1, &[]);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            retract_bid_msg.clone(),
        );

        assert_eq!(res.unwrap_err(), ContractError::NothingToRetract {});

        // retracting bid as ADDR3 (bid winner) to a different receiver should work
        let retract_bid_msg = ExecuteMsg::Retract {
            receiver: Some("addr5".to_string()),
        };
        let execute_info = mock_info(ADDR3, &[]);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            retract_bid_msg.clone(),
        )
        .unwrap();

        // create a bank message for ADDR3
        let bank_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr5".to_string(),
            amount: vec![coin(99_900_000u128.into(), DENOM)],
        });
        assert_eq!(res.messages[0].msg, bank_msg);

        // retracting bid as ADDR4 (bid winner) without passing receiver should fail
        let execute_info = mock_info(ADDR4, &[]);

        let res = execute(
            deps.as_mut(),
            env.clone(),
            execute_info.clone(),
            retract_bid_msg.clone(),
        );

        assert_eq!(res.unwrap_err(), ContractError::WinnerCannotRetractBid {});
    }
}
