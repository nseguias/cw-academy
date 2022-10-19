use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

mod contract;
pub mod msg;
mod state;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    contract::instantiate(deps)?;
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: msg::ExecuteMsg,
) -> StdResult<Response> {
    use contract::execute;
    use msg::ExecuteMsg::*;
    match msg {
        Increment {} => execute::increment(deps, info),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use contract::query;
    use msg::QueryMsg::*;
    match msg {
        Value {} => to_binary(&query::value(deps)?),
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{Addr, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use crate::msg::{ExecuteMsg, QueryMsg, ValueResponse};
    use crate::{execute, instantiate, query};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &Empty {},
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        let resp: ValueResponse = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResponse { value: 0 });
    }

    #[test]
    fn increment() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let sender = Addr::unchecked("sender");
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender.clone(),
                &Empty {},
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            sender.clone(),
            contract_addr.clone(),
            &ExecuteMsg::Increment {},
            &[],
        )
        .unwrap();

        let resp: ValueResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResponse { value: 1 });
    }
}
