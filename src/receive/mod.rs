use cosmwasm_std::{from_binary, DepsMut, Env, MessageInfo, Response};

mod seed;

use cw721::Cw721ReceiveMsg;
use cw721_base::QueryMsg as Cw721QueryMsg;
use komple_framework_metadata_module::msg::{MetadataResponse, QueryMsg as KompleMetadataQueryMsg};
use komple_framework_token_module::msg::QueryMsg as KompleTokenQueryMsg;
use komple_framework_types::{modules::token::SubModules, shared::query::ResponseWrapper};
use seed::seed;

use crate::{helpers::throw_err, msg::Cw721HookMsg, state::INFORMATION, ContractError};

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = INFORMATION.load(deps.storage)?;

    let collection = config
        .whitelisted_collections
        .iter()
        .find(|c| info.sender.eq(c));

    if collection.is_none() {
        return Err(throw_err(&format!("Unauthorized collection",)));
    }

    let collection = collection.unwrap();

    let submodules: ResponseWrapper<SubModules> = deps.querier.query_wasm_smart(
        collection,
        &Cw721QueryMsg::Extension {
            msg: KompleTokenQueryMsg::SubModules {},
        },
    )?;

    if submodules.data.metadata.is_none() {
        return Err(throw_err(&format!("Missing Komple metadata submodule",)));
    }

    let metadata: MetadataResponse = deps.querier.query_wasm_smart(
        submodules.data.metadata.unwrap(),
        &KompleMetadataQueryMsg::Metadata {
            token_id: msg.token_id.parse::<u32>().unwrap(),
        },
    )?;

    let token_type = metadata
        .metadata
        .attributes
        .iter()
        .find(|a| a.trait_type == "type");

    if token_type.is_none() {
        return Err(throw_err(&format!("Missing metadata type",)));
    }

    let token_type = token_type.unwrap();

    match from_binary(&msg.msg)? {
        Cw721HookMsg::Seed {} => seed(
            deps,
            env,
            msg.sender,
            msg.token_id,
            token_type.value.to_owned(),
        ),
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{testing::mock_info, to_binary, Addr, SystemResult, WasmQuery};
    use cw721::Cw721ReceiveMsg;
    use komple_framework_metadata_module::{
        msg::MetadataResponse,
        state::{MetaInfo, Metadata, Trait},
    };
    use komple_framework_types::{modules::token::SubModules, shared::query::ResponseWrapper};

    use crate::{
        contract::execute,
        msg::{Cw721HookMsg, ExecuteMsg, InstantiateMsg},
        tests::{general_handle_wasm_query, get_komple_addrs, setup_test},
    };

    #[test]
    #[should_panic(expected = "Unauthorized collection")]
    fn unauthorized_collection() {
        let (mut deps, env) = setup_test(Some(InstantiateMsg {
            admin: None,
            whitelisted_collections: None,
        }));

        let collection_addr = "collection_addr";
        let auth_info = mock_info(collection_addr, &vec![]);
        let nft_owner = "nft_owner";
        let msg = ExecuteMsg::Receive(Cw721ReceiveMsg {
            sender: nft_owner.to_string(),
            token_id: "1".to_string(),
            msg: to_binary(&Cw721HookMsg::Seed {}).unwrap(),
        });

        let res = execute(deps.as_mut(), env.to_owned(), auth_info, msg).unwrap();

        println!("{:?}", res);
    }

    #[test]
    fn authorized_collection() {
        let collection_addr = "collection_addr";
        let (mut deps, env) = setup_test(Some(InstantiateMsg {
            admin: None,
            whitelisted_collections: Some(vec![Addr::unchecked(collection_addr)]),
        }));

        deps.querier
            .update_wasm(move |wasm_query| match wasm_query {
                WasmQuery::Smart {
                    contract_addr,
                    msg: _msg,
                } if *contract_addr == collection_addr => SystemResult::Ok(
                    to_binary(&ResponseWrapper::new(
                        "sub_modules",
                        SubModules {
                            metadata: Some(get_komple_addrs().metadata),
                            whitelist: None,
                        },
                    ))
                    .into(),
                ),
                WasmQuery::Smart {
                    contract_addr,
                    msg: _msg,
                } if *contract_addr == get_komple_addrs().metadata => SystemResult::Ok(
                    to_binary(&MetadataResponse {
                        metadata_id: 1,
                        metadata: Metadata {
                            attributes: vec![Trait {
                                trait_type: "type".into(),
                                value: "wheat".into(),
                            }],
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
                _ => general_handle_wasm_query(wasm_query),
            });

        let auth_info = mock_info(collection_addr, &vec![]);
        let nft_owner = "nft_owner";
        let msg = ExecuteMsg::Receive(Cw721ReceiveMsg {
            sender: nft_owner.to_string(),
            token_id: "1".to_string(),
            msg: to_binary(&Cw721HookMsg::Seed {}).unwrap(),
        });

        let res = execute(deps.as_mut(), env.to_owned(), auth_info, msg);

        res.unwrap();

        // assert_eq!(res.is_ok(), true);
    }
}
