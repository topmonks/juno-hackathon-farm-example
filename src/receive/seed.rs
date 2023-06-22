use cosmwasm_std::{DepsMut, Env, Response};

use crate::{state::INFORMATION, ContractError};

pub fn seed(
    deps: DepsMut,
    env: Env,
    sender: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let information = INFORMATION.load(deps.storage)?;

    Ok(Response::new().add_attributes(vec![("method", "seed")]))
}

#[cfg(test)]
mod test {}
