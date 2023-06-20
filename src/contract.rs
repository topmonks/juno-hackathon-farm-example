#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::farm::FarmItem;
use crate::msg::{ContractInformationResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::helpers::throw_err;
use crate::state::{FarmProfile, FARM_PROFILES, INFORMATION};

const CONTRACT_NAME: &str = "crates.io:farm_template";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = msg.admin.unwrap_or_else(|| info.sender.into_string());
    deps.api.addr_validate(&admin)?;

    INFORMATION.save(deps.storage, &ContractInformationResponse { admin })?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Start {} => {
            let sender = info.sender.to_string();

            // check if address already has a Farm
            if FARM_PROFILES
                .may_load(deps.storage, sender.as_str())?
                .is_some()
            {
                return Err(throw_err("Farm already exists for you"));
            }

            // create a fresh farm for the user with default plots and cooldowns (None)
            let farm_profile = FarmProfile::new();

            // save this to the users profile
            FARM_PROFILES.save(deps.storage, sender.as_str(), &farm_profile)?;

            Ok(Response::new().add_attribute("action", "start"))
        }

        ExecuteMsg::Stop {} => {
            let sender = info.sender.to_string();
            FARM_PROFILES.remove(deps.storage, sender.as_str());

            Ok(Response::new().add_attribute("action", "stop"))
        }

        // For UI/UX, how could a user till multiple at the same time?
        ExecuteMsg::TillGround { x, y } => {
            let sender = info.sender.to_string();

            let farm = FARM_PROFILES.may_load(deps.storage, sender.as_str())?;
            if farm.is_none() {
                return Err(throw_err("You do not have a farm"));
            }

            let mut farm = farm.unwrap();

            // check if the plot is already tilled
            let plot_value = farm.get_plot(x.into(), y.into());
            if plot_value == FarmItem::Air {
                return Err(throw_err("Plot at x,y does not exist"));
            }

            if plot_value != FarmItem::Grass {
                return Err(throw_err(&format!(
                    "Plot [{}, {}] must be grass to till",
                    x, y
                )));
            }

            // till the plot
            let updated_farm = farm.till(x.into(), y.into());

            // save to state
            FARM_PROFILES.save(deps.storage, sender.as_str(), &updated_farm)?;

            Ok(Response::new().add_attribute("action", "tilled"))
        } // What other features / upgrades will you add?
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => {
            let info = INFORMATION.load(deps.storage)?;
            let v = to_binary(&info)?;
            Ok(v)
        }

        QueryMsg::GetFarmProfile { address } => {
            let farms = FARM_PROFILES.may_load(deps.storage, address.as_str())?;
            let v = to_binary(&farms)?;
            Ok(v)
        }
    }
}

#[cfg(test)]
mod tests {}
