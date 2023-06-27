#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use komple_framework_mint_module::msg::ExecuteMsg as KompleMintExecuteMsg;

use crate::error::ContractError;
use crate::farm::SlotType;
use crate::msg::{ContractInformationResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::helpers::throw_err;
use crate::receive::receive;
use crate::state::{farm_profile_dto, FarmProfile, FARM_PROFILES, INFORMATION};

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
            komple_mint_addr: msg.komple_mint_addr,
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

            return match farm {
                None => Err(throw_err("You do not have a farm")),
                Some(mut farm) => {
                    let slot = farm.get_plot(x.into(), y.into());
                    if !slot.can_till(env.block.height) {
                        return Err(throw_err(&format!(
                            "Plot [{}, {}] must be meadow or field with dead plant to till",
                            x, y
                        )));
                    }

                    farm.till(x.into(), y.into(), env.block.height);
                    FARM_PROFILES.save(deps.storage, sender.as_str(), &farm)?;

                    Ok(Response::new().add_attribute("action", "tilled"))
                }
            };
        }

        ExecuteMsg::ReceiveNft(msg) => receive(deps, env, info, msg),

        ExecuteMsg::WaterPlant { x, y } => {
            let sender = info.sender.to_string();
            let farm: Option<FarmProfile> =
                FARM_PROFILES.may_load(deps.storage, sender.as_str())?;

            return match farm {
                None => Err(throw_err("You do not have a farm")),
                Some(mut farm) => {
                    let plot = farm.get_plot(x.into(), y.into());
                    let plant = plot.plant;

                    return match plant {
                        None => Err(throw_err(&format!(
                            "Plot [{}, {}] must contain a plant to water.",
                            x, y
                        ))),
                        Some(plant) => {
                            if !plant.can_water(env.block.height) {
                                if plant.can_harvest(env.block.height) {
                                    return Err(throw_err(&format!(
                                        "Plant [{}, {}] is fully grown and cannot be watered anymore.",
                                        x, y
                                    )));
                                }

                                if plant.is_dead(env.block.height) {
                                    return Err(throw_err(&format!(
                                        "Plant [{}, {}] is dead and cannot be watered anymore.",
                                        x, y
                                    )));
                                }

                                return Err(throw_err(&format!(
                                    "Plant [{}, {}] cannot be watered.",
                                    x, y
                                )));
                            }

                            farm.water_plant(x.into(), y.into(), env.block.height);
                            FARM_PROFILES.save(deps.storage, sender.as_str(), &farm)?;

                            Ok(Response::new().add_attribute("action", "watered"))
                        }
                    };
                }
            };
        }

        ExecuteMsg::Harvest { x, y } => {
            let sender = info.sender.to_string();
            let farm = FARM_PROFILES.may_load(deps.storage, sender.as_str())?;

            return match farm {
                None => Err(throw_err("You do not have a farm")),
                Some(mut farm) => {
                    let plot = farm.get_plot(x.into(), y.into());
                    let plant = plot.plant;

                    return match plant {
                        None => Err(throw_err(&format!(
                            "Plot [{}, {}] must contain a plant to harvest.",
                            x, y
                        ))),
                        Some(plant) => {
                            if !plant.can_harvest(env.block.height) {
                                return Err(throw_err(&format!(
                                    "Plant [{}, {}] must be fully grown and watered to harvest it.",
                                    x, y
                                )));
                            }
                            let information = INFORMATION.load(deps.storage)?;

                            let mut messages: Vec<CosmosMsg> = vec![];

                            if let Some(komple_mint_addr) = information.komple_mint_addr {
                                if let Some(komple) = plant.komple {
                                    let admin_mint_nft = WasmMsg::Execute {
                                        contract_addr: komple_mint_addr,
                                        msg: to_binary::<KompleMintExecuteMsg>(
                                            &KompleMintExecuteMsg::AdminMint {
                                                collection_id: komple.collection_id,
                                                recipient: info.sender.into_string(),
                                                metadata_id: Some(komple.metadata_id),
                                            },
                                        )?,
                                        funds: vec![],
                                    };

                                    messages.push(admin_mint_nft.into());
                                }
                            }

                            farm.harvest(x.into(), y.into(), env.block.height);
                            FARM_PROFILES.save(deps.storage, sender.as_str(), &farm)?;

                            Ok(Response::new()
                                .add_attribute("action", "harvested")
                                .add_messages(messages))
                        }
                    };
                }
            };
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => {
            let info = INFORMATION.load(deps.storage)?;
            let v = to_binary(&info)?;
            Ok(v)
        }
        QueryMsg::GetFarmProfile { address } => {
            let farm = FARM_PROFILES.may_load(deps.storage, address.as_str())?;
            let farm_dto = farm_profile_dto(&farm, env.block.height);

            // get possible actions
            let v = to_binary(&farm_dto)?;
            Ok(v)
        }
    }
}

#[cfg(test)]
mod tests {}
