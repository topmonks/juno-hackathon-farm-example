#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::farm::SlotType;
use crate::msg::{ContractInformationResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::helpers::throw_err;
use crate::receive::receive;
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

    let whitelisted_collections = msg.whitelisted_collections.unwrap_or_default();

    INFORMATION.save(
        deps.storage,
        &ContractInformationResponse {
            admin,
            whitelisted_collections,
        },
    )?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Start {} => {
            let sender = info.sender.to_string();

            if FARM_PROFILES
                .may_load(deps.storage, sender.as_str())?
                .is_some()
            {
                return Err(throw_err("Farm already exists for you"));
            }

            let farm_profile: FarmProfile = FarmProfile::new();
            FARM_PROFILES.save(deps.storage, sender.as_str(), &farm_profile)?;

            Ok(Response::new().add_attribute("action", "start"))
        }

        ExecuteMsg::Stop {} => {
            let sender = info.sender.to_string();
            FARM_PROFILES.remove(deps.storage, sender.as_str());

            Ok(Response::new().add_attribute("action", "stop"))
        }

        ExecuteMsg::TillGround { x, y } => {
            let sender = info.sender.to_string();
            let farm = FARM_PROFILES.may_load(deps.storage, sender.as_str())?;
            if farm.is_none() {
                return Err(throw_err("You do not have a farm"));
            }

            let mut farm = farm.unwrap();
            let plot_value = farm.get_plot(x.into(), y.into());
            if plot_value.r#type != SlotType::Meadow {
                return Err(throw_err(&format!(
                    "Plot [{}, {}] must be meadow to till",
                    x, y
                )));
            }
            let updated_farm = farm.till(x.into(), y.into());
            FARM_PROFILES.save(deps.storage, sender.as_str(), &updated_farm)?;

            Ok(Response::new().add_attribute("action", "tilled"))
        }

        ExecuteMsg::Receive(msg) => receive(deps, env, info, msg),

        ExecuteMsg::PlantSeed { x, y } => {
            let sender = info.sender.to_string();
            let farm = FARM_PROFILES.may_load(deps.storage, sender.as_str())?;
            if farm.is_none() {
                return Err(throw_err("You do not have a farm"));
            }

            let mut farm = farm.unwrap();
            let plot = farm.get_plot(x.into(), y.into());
            let plant = plot.plant;
            if plot.r#type != SlotType::Field || plant.is_some() {
                return Err(throw_err(&format!(
                    "Plot [{}, {}] must be an empty field to plant a seed.",
                    x, y
                )));
            }

            let updated_farm = farm.plant_seed(x.into(), y.into());
            FARM_PROFILES.save(deps.storage, sender.as_str(), &updated_farm)?;

            Ok(Response::new().add_attribute("action", "planted"))
        }

        ExecuteMsg::WaterPlant { x, y } => {
            let sender = info.sender.to_string();
            let farm = FARM_PROFILES.may_load(deps.storage, sender.as_str())?;
            if farm.is_none() {
                return Err(throw_err("You do not have a farm"));
            }

            let mut farm = farm.unwrap();
            let plot = farm.get_plot(x.into(), y.into());
            let plant = plot.plant;
            if plant.is_none() {
                return Err(throw_err(&format!(
                    "Plot [{}, {}] must contain a plant to water.",
                    x, y
                )));
            }

            let uplant = plant.unwrap();
            if uplant.current_stage >= uplant.stages {
                return Err(throw_err(&format!(
                    "Plant [{}, {}] is fully grown and cannot be watered anymore.",
                    x, y
                )));
            }

            let updated_farm = farm.water_plant(x.into(), y.into());
            FARM_PROFILES.save(deps.storage, sender.as_str(), &updated_farm)?;

            Ok(Response::new().add_attribute("action", "watered"))
        }

        ExecuteMsg::Harvest { x, y } => {
            let sender = info.sender.to_string();
            let farm = FARM_PROFILES.may_load(deps.storage, sender.as_str())?;
            if farm.is_none() {
                return Err(throw_err("You do not have a farm"));
            }

            let mut farm = farm.unwrap();
            let plot = farm.get_plot(x.into(), y.into());
            let plant = plot.plant;
            if plant.is_none() {
                return Err(throw_err(&format!(
                    "Plot [{}, {}] must contain a plant to harvest.",
                    x, y
                )));
            }

            let uplant = plant.unwrap();
            if uplant.current_stage != uplant.stages {
                return Err(throw_err(&format!(
                    "Plant [{}, {}] must be fully grown to harvest it.",
                    x, y
                )));
            }

            let updated_farm = farm.harvest(x.into(), y.into());
            FARM_PROFILES.save(deps.storage, sender.as_str(), &updated_farm)?;

            Ok(Response::new().add_attribute("action", "harvested"))
        }
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
