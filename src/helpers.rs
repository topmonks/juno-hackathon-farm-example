use cosmwasm_std::{Deps, MessageInfo};

use crate::{state::INFORMATION, ContractError};

// HELPERS
pub fn admin_error_check(deps: Deps, info: MessageInfo) -> Result<(), ContractError> {
    let contract_info = INFORMATION.load(deps.storage)?;
    if contract_info.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

pub fn throw_err(msg: &str) -> ContractError {
    return ContractError::Std(cosmwasm_std::StdError::generic_err(msg));
}
