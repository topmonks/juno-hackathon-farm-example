use cosmwasm_std::{from_binary, DepsMut, Env, MessageInfo, Response};

mod seed;

use cw721::Cw721ReceiveMsg;
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

    match from_binary(&msg.msg)? {
        Cw721HookMsg::Seed {} => seed(deps, env, msg.sender, msg.token_id),
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{testing::mock_info, to_binary, Addr};
    use cw721::Cw721ReceiveMsg;

    use crate::{
        contract::execute,
        msg::{Cw721HookMsg, ExecuteMsg, InstantiateMsg},
        tests::setup_test,
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

        let auth_info = mock_info(collection_addr, &vec![]);
        let nft_owner = "nft_owner";
        let msg = ExecuteMsg::Receive(Cw721ReceiveMsg {
            sender: nft_owner.to_string(),
            token_id: "1".to_string(),
            msg: to_binary(&Cw721HookMsg::Seed {}).unwrap(),
        });

        let res = execute(deps.as_mut(), env.to_owned(), auth_info, msg);

        assert_eq!(res.is_ok(), true);
    }
}
