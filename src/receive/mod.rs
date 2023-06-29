use cosmwasm_std::{from_binary, DepsMut, Env, MessageInfo, Response};

mod seed;

use cw721::Cw721ReceiveMsg;
use cw721_base::QueryMsg as Cw721QueryMsg;
use komple_framework_metadata_module::msg::{MetadataResponse, QueryMsg as KompleMetadataQueryMsg};
use komple_framework_mint_module::msg::{CollectionsResponse, QueryMsg as KompleMintQueryMsg};
use komple_framework_token_module::msg::QueryMsg as KompleTokenQueryMsg;
use komple_framework_types::{modules::token::SubModules, shared::query::ResponseWrapper};
use seed::seed;

use crate::{
    farm::KomplePlant, helpers::throw_err, msg::Cw721HookMsg, state::INFORMATION, ContractError,
};

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = INFORMATION.load(deps.storage)?;

    if config.komple_mint_addr.is_none() {
        return Err(throw_err(&format!("Komple mint address not provided",)));
    }

    let collections: ResponseWrapper<Vec<CollectionsResponse>> = deps.querier.query_wasm_smart(
        config.komple_mint_addr.unwrap(),
        &KompleMintQueryMsg::Collections {
            blacklist: false,
            start_after: None,
            limit: None,
        },
    )?;

    let collection = collections.data.iter().find(|c| info.sender.eq(&c.address));

    if collection.is_none() {
        return Err(throw_err(&format!("Unauthorized collection",)));
    }

    let collection = collection.unwrap();

    let submodules: ResponseWrapper<SubModules> = deps.querier.query_wasm_smart(
        &collection.address,
        &Cw721QueryMsg::Extension {
            msg: KompleTokenQueryMsg::SubModules {},
        },
    )?;

    if submodules.data.metadata.is_none() {
        return Err(throw_err(&format!("Missing Komple metadata submodule",)));
    }

    let metadata: ResponseWrapper<MetadataResponse> = deps.querier.query_wasm_smart(
        submodules.data.metadata.unwrap(),
        &KompleMetadataQueryMsg::Metadata {
            token_id: msg.token_id.parse::<u32>().unwrap(),
        },
    )?;

    let plant_type = metadata
        .data
        .metadata
        .attributes
        .iter()
        .find(|a| a.trait_type == "type");

    if plant_type.is_none() {
        return Err(throw_err(&format!("Missing metadata type",)));
    }

    let plant_type = plant_type.unwrap().value.parse()?;

    let komple = KomplePlant {
        metadata_id: metadata.data.metadata_id,
        collection_id: collection.collection_id,
    };

    match from_binary(&msg.msg)? {
        Cw721HookMsg::Seed { x, y } => seed(
            deps,
            env,
            msg.sender,
            msg.token_id,
            plant_type,
            komple,
            x,
            y,
        ),
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{testing::mock_info, to_binary, SystemResult, WasmQuery};
    use cw721::Cw721ReceiveMsg;
    use komple_framework_metadata_module::{
        msg::MetadataResponse,
        state::{MetaInfo, Metadata, Trait},
    };
    use komple_framework_mint_module::msg::CollectionsResponse;
    use komple_framework_types::{modules::token::SubModules, shared::query::ResponseWrapper};

    use crate::{
        contract::execute,
        msg::{Cw721HookMsg, ExecuteMsg, InstantiateMsg},
        tests::{general_handle_wasm_query, get_komple_addrs, init_farm, setup_test, till},
    };

    #[test]
    #[should_panic(expected = "Unauthorized collection")]
    fn unauthorized_collection() {
        let (mut deps, env) = setup_test(Some(InstantiateMsg {
            admin: None,
            komple_mint_addr: Some(get_komple_addrs().mint.to_string()),
        }));

        deps.querier
            .update_wasm(move |wasm_query| match wasm_query {
                WasmQuery::Smart {
                    contract_addr,
                    msg: _msg,
                } if *contract_addr == get_komple_addrs().mint => SystemResult::Ok(
                    to_binary(&ResponseWrapper::new(
                        "collections",
                        vec![CollectionsResponse {
                            address: "collection".to_string(),
                            collection_id: 1,
                        }],
                    ))
                    .into(),
                ),
                _ => general_handle_wasm_query(wasm_query),
            });

        let collection_addr = "collection_addr";
        let auth_info = mock_info(collection_addr, &vec![]);
        let nft_owner = "nft_owner";
        let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: nft_owner.to_string(),
            token_id: "1".to_string(),
            msg: to_binary(&Cw721HookMsg::Seed { x: 0, y: 0 }).unwrap(),
        });

        let res = execute(deps.as_mut(), env.to_owned(), auth_info, msg).unwrap();

        println!("{:?}", res);
    }

    #[test]
    fn authorized_collection() {
        let collection_addr = "collection_addr";
        let (mut deps, env) = setup_test(Some(InstantiateMsg {
            admin: None,
            komple_mint_addr: Some(get_komple_addrs().mint.to_string()),
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
                    to_binary(&ResponseWrapper::new(
                        "metadata",
                        MetadataResponse {
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
                        },
                    ))
                    .into(),
                ),
                WasmQuery::Smart {
                    contract_addr,
                    msg: _msg,
                } if *contract_addr == get_komple_addrs().mint => SystemResult::Ok(
                    to_binary(&ResponseWrapper::new(
                        "collections",
                        vec![CollectionsResponse {
                            address: collection_addr.to_string(),
                            collection_id: 1,
                        }],
                    ))
                    .into(),
                ),
                _ => general_handle_wasm_query(wasm_query),
            });

        let auth_info = mock_info(collection_addr, &vec![]);
        let nft_owner = "nft_owner";
        init_farm(nft_owner, deps.as_mut());
        till(nft_owner, 0, 0, deps.as_mut());

        let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            sender: nft_owner.to_string(),
            token_id: "1".to_string(),
            msg: to_binary(&Cw721HookMsg::Seed { x: 0, y: 0 }).unwrap(),
        });

        let res = execute(deps.as_mut(), env.to_owned(), auth_info, msg);

        assert_eq!(res.is_ok(), true);
    }
}
