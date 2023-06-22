use crate::contract::instantiate;

use crate::msg::InstantiateMsg;
use crate::state::INFORMATION;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{Env, OwnedDeps, QuerierResult, SystemError, SystemResult, WasmQuery};

fn setup_test() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Env) {
    let mut dependencies = mock_dependencies();

    dependencies
        .querier
        .update_wasm(move |x| handle_wasm_query(x));

    let mut env = mock_env();

    (dependencies, env)
}

fn handle_wasm_query(wasm_query: &WasmQuery) -> QuerierResult {
    match wasm_query {
        WasmQuery::Smart { contract_addr, .. } => SystemResult::Err(SystemError::NoSuchContract {
            addr: contract_addr.clone(),
        }),
        WasmQuery::Raw { contract_addr, .. } => SystemResult::Err(SystemError::NoSuchContract {
            addr: contract_addr.clone(),
        }),
        WasmQuery::ContractInfo { contract_addr, .. } => {
            SystemResult::Err(SystemError::NoSuchContract {
                addr: contract_addr.clone(),
            })
        }
        _ => unreachable!(),
    }
}

#[test]
fn proper_initialization() {
    let (mut deps, _env) = setup_test();

    let msg = InstantiateMsg { admin: None };
    let info = mock_info("creator", &vec![]);

    let res = instantiate(deps.as_mut(), mock_env(), info, msg);

    assert_eq!(res.is_ok(), true);

    let information = INFORMATION.load(&deps.storage).unwrap();

    assert_eq!(information.admin, "creator");
}
