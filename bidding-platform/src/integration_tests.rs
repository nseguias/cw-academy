#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr, Empty, Uint128};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use crate::{
        contract::{execute, instantiate, query},
        msg::{
            BidWinnerResponse, ExecuteMsg, HighestBidderResponse, InstantiateMsg,
            IsBidClosedResponse, QueryMsg, TotalBidResponse,
        },
    };

    // returns an object that can be used with cw-multi-test
    fn bidding_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    #[test]
    fn bidding_process() {
        let owner = Addr::unchecked("owner");
        let alex = Addr::unchecked("alex");
        let ann = Addr::unchecked("ann");

        // an app object is the blockchain simulator. we send initial balance here too!
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &owner, coins(2_000_000u128, "uatom"))
                .unwrap();
            router
                .bank
                .init_balance(storage, &alex, coins(20_000_000u128, "uatom"))
                .unwrap();
            router
                .bank
                .init_balance(storage, &ann, coins(20_000_000u128, "uatom"))
                .unwrap();
        });

        // upload the contract to the blockchain and get back code_id to instantiate the contract
        let code_id = app.store_code(bidding_contract());

        let instantiate_msg = InstantiateMsg {
            commodity: "gold".to_string(),
            contract_owner: None,
            commision: None,
        };

        let contract_addr = app
            .instantiate_contract(
                code_id,
                owner.clone(),
                &instantiate_msg,
                &coins(2_000_000u128, "uatom"),
                "Bidding Platform",
                None,
            )
            .unwrap();

        let bid_msg = ExecuteMsg::Bid {};

        // Alex bids with 15 ATOM
        app.execute_contract(
            alex.clone(),
            contract_addr.clone(),
            &bid_msg,
            &coins(15_000_000u128, "uatom"),
        )
        .unwrap();

        // Highest bidder should be Alex @ 14.985 ATOM

        let res: HighestBidderResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::HighestBidder {})
            .unwrap();

        assert_eq!(res.addr, "alex");
        assert_eq!(res.total_bid, Uint128::from(14_985_000u128));

        // ANN bids with 17 ATOM
        app.execute_contract(
            ann.clone(),
            contract_addr.clone(),
            &bid_msg,
            &coins(17_000_000u128, "uatom"),
        )
        .unwrap();

        // Highest bidder should be Ann @ 16.983 ATOM
        let res: HighestBidderResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::HighestBidder {})
            .unwrap();

        assert_eq!(res.addr, "ann");
        assert_eq!(res.total_bid, Uint128::from(16_983_000u32));

        // Total bid for Alex is 14.985 ATOM
        let res: TotalBidResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::TotalBid {
                    address: "alex".to_string(),
                },
            )
            .unwrap();

        assert_eq!(res.total_bid, Uint128::from(14_985_000u32));

        // ANN bids with 2 ATOM
        app.execute_contract(
            ann.clone(),
            contract_addr.clone(),
            &bid_msg,
            &coins(2_000_000u128, "uatom"),
        )
        .unwrap();

        // Highest bidder should be Ann @ 18_981_000 uATOM
        let res: HighestBidderResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::HighestBidder {})
            .unwrap();

        assert_eq!(res.addr, "ann");
        assert_eq!(res.total_bid, Uint128::from(18_981_000u32));

        // Total bid for Alex is 14.985 ATOM
        let res: TotalBidResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::TotalBid {
                    address: "alex".to_string(),
                },
            )
            .unwrap();

        assert_eq!(res.total_bid, Uint128::from(14_985_000u32));

        // Alex bids with 1 ATOM -> SHOULD FAIL
        let err_res = app.execute_contract(
            alex.clone(),
            contract_addr.clone(),
            &bid_msg,
            &coins(1_000_000u128, "uatom"),
        );

        assert!(err_res.is_err());

        // Alex bids with 5 ATOM
        app.execute_contract(
            alex.clone(),
            contract_addr.clone(),
            &bid_msg,
            &coins(5_000_000u128, "uatom"),
        )
        .unwrap();

        assert!(err_res.is_err());

        // query highest bidder should return Alex @ 19_980_000 uATOM
        let res: HighestBidderResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::HighestBidder {})
            .unwrap();

        assert_eq!(res.addr, "alex");
        assert_eq!(res.total_bid, Uint128::from(19_980_000u128));

        // Total bid for Ann is 18_981_000 uATOM
        let res: TotalBidResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::TotalBid {
                    address: "ann".to_string(),
                },
            )
            .unwrap();

        assert_eq!(res.total_bid, Uint128::from(18_981_000u128));

        // Bid should be opened
        let res: IsBidClosedResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::IsBidClosed {})
            .unwrap();

        assert!(!res.is_closed);

        // querying the bid winner should throw an error
        let res: Result<BidWinnerResponse, cosmwasm_std::StdError> = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::BidWinner {});

        assert!(res.is_err());

        // Owner closes bid
        let close_bid_msg = ExecuteMsg::Close {};

        app.execute_contract(
            owner.clone(),
            contract_addr.clone(),
            &close_bid_msg,
            &vec![],
        )
        .unwrap();

        // Ann retracts bid to herself
        let retract_bid_msg = ExecuteMsg::Retract { receiver: None };
        app.execute_contract(
            ann.clone(),
            contract_addr.clone(),
            &retract_bid_msg,
            &vec![],
        )
        .unwrap();

        // Owner retracts initial bid to an Anon address
        let retract_bid_to_anon_msg = ExecuteMsg::Retract {
            receiver: Some("anon".to_string()),
        };

        app.execute_contract(
            owner.clone(),
            contract_addr.clone(),
            &retract_bid_to_anon_msg,
            &vec![],
        )
        .unwrap();

        // query highest bidder should return Alex @ 19_980_000 uATOM
        let res: HighestBidderResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::HighestBidder {})
            .unwrap();

        assert_eq!(res.addr, "alex");
        assert_eq!(res.total_bid, Uint128::from(19_980_000u128));

        // Total bid for Ann should be 0 ATOM
        let res: TotalBidResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::TotalBid {
                    address: "ann".to_string(),
                },
            )
            .unwrap();

        assert!(res.total_bid.is_zero());

        // Total bid for Owner should be 0 ATOM
        let res: TotalBidResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::TotalBid {
                    address: "owner".to_string(),
                },
            )
            .unwrap();

        assert!(res.total_bid.is_zero());

        // Owner should have 20_019_000 uATOM (Alex bid + Ann's commissions)
        let final_balance_owner = app.wrap().query_all_balances("owner").unwrap();
        assert_eq!(final_balance_owner, coins(20_019_000, "uatom"));

        // Alex should have 0 ATOM as she bought gold with all their tokens
        let final_balance_alex = app.wrap().query_all_balances("alex").unwrap();
        assert_eq!(final_balance_alex, []);

        // Ann
        let final_balance_ann = app.wrap().query_all_balances("ann").unwrap();
        assert_eq!(final_balance_ann, coins(19_981_000, "uatom"));

        // anon should have 1_000_000 uATOM (owner's donated their initial balance)
        let final_balance_anon = app.wrap().query_all_balances("anon").unwrap();
        assert_eq!(final_balance_anon, coins(2_000_000, "uatom"));

        // check that no tokens were lost -> 42_000_000 uATOM were printed!
        assert_eq!(
            final_balance_owner[0].amount
                + Uint128::zero()
                + final_balance_ann[0].amount
                + final_balance_anon[0].amount,
            Uint128::from(42_000_000u128)
        );

        // Bid should be closed
        let res: IsBidClosedResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::IsBidClosed {})
            .unwrap();

        assert!(res.is_closed);

        // Alex should be the bid winner :)
        let res: BidWinnerResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::BidWinner {})
            .unwrap();

        assert_eq!(res.winner, "alex".to_string());
    }
}
