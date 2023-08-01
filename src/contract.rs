use std::collections::HashMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Storage, WasmMsg,
};
use cw2::set_contract_version;
use komple_framework_mint_module::msg::ExecuteMsg as KompleMintExecuteMsg;
use nois::{int_in_range, NoisCallback, ProxyExecuteMsg};

use crate::error::ContractError;
use crate::farm::KomplePlant;
use crate::msg::{ContractInformation, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::helpers::throw_err;
use crate::receive::receive;
use crate::state::{
    farm_profile_dto, points, FarmProfile, NoiseJob, Points, FARM_PROFILES, INFORMATION, NOIS_JOBS,
    NOIS_JOB_LAST_ID, NOIS_PROXY,
};

const CONTRACT_NAME: &str = "crates.io:farm_template";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn next_job_id(store: &mut dyn Storage) -> StdResult<String> {
    let last_id = (NOIS_JOB_LAST_ID.may_load(store)?).unwrap_or(0);
    let next_id = last_id + 1;
    NOIS_JOB_LAST_ID.save(store, &next_id)?;

    Ok(next_id.to_string())
}

fn mint_seeds(
    plant: KomplePlant,
    recipient: String,
    seeds: i32,
    storage: &dyn Storage,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let information = INFORMATION.load(storage)?;
    let admin_mint_nft = match information.komple_mint_addr {
        None => Err(throw_err("Komple mint addr missing.")),
        Some(komple_mint_addr) => Ok(WasmMsg::Execute {
            contract_addr: komple_mint_addr,
            msg: to_binary::<KompleMintExecuteMsg>(&KompleMintExecuteMsg::AdminMint {
                collection_id: plant.collection_id,
                recipient,
                metadata_id: Some(plant.metadata_id),
            })?,
            funds: vec![],
        }),
    }?;

    let mut messages: Vec<CosmosMsg> = vec![];
    for _i in 0..seeds {
        messages.push(admin_mint_nft.clone().into());
    }

    Ok(messages)
}

fn noise_job(
    noise_job: NoiseJob,
    randomness: [u8; 32],
    storage: &dyn Storage,
) -> Result<Vec<CosmosMsg>, ContractError> {
    match noise_job {
        NoiseJob::MintSeeds { plant, recipient } => {
            let seeds = int_in_range(randomness, 2, 5);

            mint_seeds(plant, recipient, seeds, storage)
        }
    }
}

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

    NOIS_JOB_LAST_ID.save(deps.storage, &0)?;

    match msg.nois_proxy {
        None => (),
        Some(addr) => {
            let nois_proxy_addr = deps.api.addr_validate(&addr)?;
            NOIS_PROXY.save(deps.storage, &nois_proxy_addr)?;
        }
    }

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

            match farm {
                None => Err(throw_err("You do not have a farm")),
                Some(mut farm) => {
                    farm.till(x.into(), y.into(), env.block.height)?;
                    FARM_PROFILES.save(deps.storage, sender.as_str(), &farm)?;

                    Ok(Response::new().add_attribute("action", "tilled"))
                }
            }
        }

        ExecuteMsg::ReceiveNft(msg) => receive(deps, env, info, msg),

        ExecuteMsg::NoisReceive { callback } => {
            let proxy = NOIS_PROXY.load(deps.storage)?;
            ensure_eq!(info.sender, proxy, throw_err("Unauthorized sender"));

            let NoisCallback {
                job_id, randomness, ..
            } = callback;

            let randomness: [u8; 32] = randomness
                .to_array()
                .map_err(|_| throw_err("Invalid randomness"))?;

            let job = NOIS_JOBS.load(deps.storage, &job_id)?;
            NOIS_JOBS.remove(deps.storage, &job_id);
            let messages = noise_job(job, randomness, deps.storage)?;

            Ok(Response::new().add_messages(messages))
        }

        ExecuteMsg::WaterPlant { x, y } => {
            let sender = info.sender.to_string();
            let farm: Option<FarmProfile> =
                FARM_PROFILES.may_load(deps.storage, sender.as_str())?;

            match farm {
                None => Err(throw_err("You do not have a farm")),
                Some(mut farm) => {
                    farm.water_plant(x.into(), y.into(), env.block.height)?;
                    FARM_PROFILES.save(deps.storage, sender.as_str(), &farm)?;

                    Ok(Response::new().add_attribute("action", "watered"))
                }
            }
        }

        ExecuteMsg::Harvest { x, y } => {
            let sender = info.sender.to_string();
            let farm = FARM_PROFILES.may_load(deps.storage, sender.as_str())?;

            match farm {
                None => Err(throw_err("You do not have a farm")),
                Some(mut farm) => {
                    let plot = farm.get_plot(x.into(), y.into());
                    let plant = plot.plant;

                    match plant {
                        None => Err(throw_err(&format!(
                            "Plot [{}, {}] must contain a plant to harvest.",
                            x, y
                        ))),
                        Some(plant) => {
                            let messages = match plant.komple {
                                None => Err(throw_err("Plant komple missing.")),
                                Some(komple) => {
                                    let nois_proxy = NOIS_PROXY.may_load(deps.storage)?;
                                    match nois_proxy {
                                        None => mint_seeds(
                                            komple,
                                            info.sender.into_string(),
                                            2,
                                            deps.storage,
                                        ),
                                        Some(nois_proxy) => {
                                            let job = NoiseJob::MintSeeds {
                                                plant: komple,
                                                recipient: info.sender.into_string(),
                                            };
                                            let job_id = next_job_id(deps.storage)?;
                                            NOIS_JOBS.save(deps.storage, &job_id, &job)?;

                                            let mut messages: Vec<CosmosMsg> = vec![];
                                            let msg = WasmMsg::Execute {
                                                contract_addr: nois_proxy.into(),
                                                msg: to_binary(
                                                    &ProxyExecuteMsg::GetNextRandomness { job_id },
                                                )?,
                                                funds: info.funds.clone(),
                                            };
                                            messages.push(msg.into());
                                            Ok(messages)
                                        }
                                    }
                                }
                            }?;

                            let harvested = farm.harvest(x.into(), y.into(), env.block.height)?;
                            let mut pts = match points().may_load(deps.storage, sender.as_str())? {
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
                    }
                }
            }
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
                        let addr: String = v.addr;

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
                    let addr: String = v.addr;

                    Ok((addr, total))
                }
                Err(err) => Err(err),
            })
            .collect();

        assert_eq!(res.unwrap(), vec![("123".to_string(), 1)])
    }
}
