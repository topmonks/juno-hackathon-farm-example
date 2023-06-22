use cosmwasm_std::{DepsMut, Env, Response};

use crate::{state::INFORMATION, ContractError};

pub fn seed(
    deps: DepsMut,
    _env: Env,
    _sender: String,
    _token_id: String,
    token_type: String,
) -> Result<Response, ContractError> {
    let _information = INFORMATION.load(deps.storage)?;

    Ok(Response::new().add_attributes(vec![("method", "seed"), ("token_type", &token_type)]))
}

#[cfg(test)]
mod test {}
