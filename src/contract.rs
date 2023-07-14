use std::collections::HashMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
    WasmMsg,
};
use cw2::set_contract_version;
use komple_framework_mint_module::msg::ExecuteMsg as KompleMintExecuteMsg;

use crate::error::ContractError;
use crate::msg::{ContractInformation, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::helpers::throw_err;
use crate::receive::receive;
use crate::state::{farm_profile_dto, points, FarmProfile, Points, FARM_PROFILES, INFORMATION};

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

    INFORMATION.save(
        deps.storage,
        &ContractInformation {
            admin,
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

            let farm_profile: FarmProfile = FarmProfile::new(env.block.height);
            FARM_PROFILES.save(deps.storage, sender.as_str(), &farm_profile)?;

            Ok(Response::new().add_attribute("action", "start"))
        }

        ExecuteMsg::SetupFarm { farm, addr } => {
            let sender = info.sender.to_string();
            let info = INFORMATION.load(deps.storage)?;

            if sender != info.admin {
                return Err(ContractError::Unauthorized {});
            }

            FARM_PROFILES.save(deps.storage, addr.as_str(), &farm)?;

            Ok(Response::new().add_attribute("action", "setup_farm"))
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
                    farm.till(x.into(), y.into(), env.block.height)?;
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
                    farm.water_plant(x.into(), y.into(), env.block.height)?;
                    FARM_PROFILES.save(deps.storage, sender.as_str(), &farm)?;

                    Ok(Response::new().add_attribute("action", "watered"))
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

                            let harvested = farm.harvest(x.into(), y.into(), env.block.height)?;
                            let mut pts = match points().may_load(deps.storage, &sender.as_str())? {
                                None => Points {
                                    addr: sender.clone(),
                                    plants: HashMap::new(),
                                },
                                Some(p) => p,
                            };
                            pts.add(harvested);

                            FARM_PROFILES.save(deps.storage, sender.as_str(), &farm)?;
                            points().save(deps.storage, sender.as_str(), &pts)?;

                            Ok(Response::new()
                                .add_attribute("action", "harvested")
                                .add_messages(messages))
                        }
                    };
                }
            };
        }
        ExecuteMsg::UpdateContractInformation {
            contract_information,
        } => {
            let sender = info.sender.to_string();
            let info = INFORMATION.load(deps.storage)?;

            if sender != info.admin {
                return Err(ContractError::Unauthorized {});
            }

            INFORMATION.save(deps.storage, &contract_information)?;

            Ok(Response::new().add_attribute("action", "update_contract_information"))
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

            let v = to_binary(&farm_dto)?;
            Ok(v)
        }
        QueryMsg::Leaderboard {} => {
            let res: Result<Vec<(String, u64)>, _> = points()
                .idx
                .total
                .range(deps.storage, None, None, Order::Descending)
                .take(100)
                .map(|res| match res {
                    Ok((_, v)) => {
                        let total: u64 = v.total();
                        let addr: String = v.addr.into();

                        Ok((addr, total))
                    }
                    Err(err) => Err(err),
                })
                .collect();

            match res {
                Ok(v) => Ok(to_binary(&v)?),
                Err(err) => Err(err),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;

    use crate::farm::PlantType;

    use super::*;

    #[test]
    fn save_and_load_points() {
        let mut deps = mock_dependencies();

        let mut new_points = Points {
            addr: "123".into(),
            plants: HashMap::new(),
        };
        new_points.add(PlantType::Sunflower);

        let pts = points();
        pts.save(deps.as_mut().storage, "123", &new_points).unwrap();

        let res: Result<Vec<(String, u64)>, _> = pts
            .idx
            .total
            .range(deps.as_ref().storage, None, None, Order::Descending)
            .map(|res| match res {
                Ok((_, v)) => {
                    let total: u64 = v.total();
                    let addr: String = v.addr.into();

                    Ok((addr, total))
                }
                Err(err) => Err(err),
            })
            .collect();

        assert_eq!(res.unwrap(), vec![("123".to_string(), 1)])
    }
}
