use crate::contract::{execute, instantiate};

use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::INFORMATION;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    to_binary, Addr, DepsMut, Env, OwnedDeps, QuerierResult, SystemError, SystemResult, WasmQuery,
};
use komple_framework_metadata_module::msg::MetadataResponse;
use komple_framework_metadata_module::state::{MetaInfo, Metadata};

pub struct KompleAddrs {
    pub metadata: Addr,
    pub mint: Addr,
}
pub fn get_komple_addrs() -> KompleAddrs {
    KompleAddrs {
        metadata: Addr::unchecked("komple_metadata"),
        mint: Addr::unchecked("komple_mint"),
    }
}

pub fn setup_test(
    instantiate_msg: Option<InstantiateMsg>,
) -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Env) {
    let mut dependencies = mock_dependencies();

    dependencies
        .querier
        .update_wasm(|x| general_handle_wasm_query(x));

    let env = mock_env();

    if let Some(instantiate_msg) = instantiate_msg {
        let info = mock_info("creator", &vec![]);

        let _res = instantiate(dependencies.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    }

    (dependencies, env)
}

pub fn init_farm(addr: &str, deps: DepsMut) -> () {
    let msg = ExecuteMsg::Start {};
    let info = mock_info(addr, &vec![]);

    let _res = execute(deps, mock_env(), info, msg).unwrap();
}

pub fn till(addr: &str, x: u8, y: u8, deps: DepsMut) -> () {
    let msg = ExecuteMsg::TillGround { x, y };
    let info = mock_info(addr, &vec![]);

    let _res = execute(deps, mock_env(), info, msg).unwrap();
}

pub fn general_handle_wasm_query(wasm_query: &WasmQuery) -> QuerierResult {
    match wasm_query {
        WasmQuery::Smart {
            contract_addr,
            msg: _msg,
        } if *contract_addr == get_komple_addrs().metadata => SystemResult::Ok(
            to_binary(&MetadataResponse {
                metadata_id: 1,
                metadata: Metadata {
                    attributes: vec![],
                    meta_info: MetaInfo {
                        image: None,
                        external_url: None,
                        description: None,
                        animation_url: None,
                        youtube_url: None,
                    },
                },
            })
            .into(),
        ),
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
    let (mut deps, _env) = setup_test(None);

    let msg = InstantiateMsg {
        admin: None,
        komple_mint_addr: None,
    };
    let info = mock_info("creator", &vec![]);

    let res = instantiate(deps.as_mut(), mock_env(), info, msg);

    assert_eq!(res.is_ok(), true);

    let information = INFORMATION.load(&deps.storage).unwrap();

    assert_eq!(information.admin, "creator");
}
