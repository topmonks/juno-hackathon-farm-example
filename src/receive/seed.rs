use cosmwasm_std::{DepsMut, Env, Response};

use crate::{
    farm::{KomplePlant, PlantType, SlotType},
    helpers::throw_err,
    state::FARM_PROFILES,
    ContractError,
};

pub fn seed(
    deps: DepsMut,
    env: Env,
    sender: String,
    _token_id: String,
    plant_type: PlantType,
    komple: KomplePlant,
    x: u8,
    y: u8,
) -> Result<Response, ContractError> {
    let farm = FARM_PROFILES.may_load(deps.storage, sender.as_str())?;

    if farm.is_none() {
        return Err(ContractError::PlayerDoesNotExist { address: sender });
    }

    let mut farm = farm.unwrap();

    let plot = farm.get_plot(x.into(), y.into());
    if plot.get_real_type(env.block.height) != SlotType::Field || plot.plant.is_some() {
        return Err(throw_err(&format!(
            "Plot [{}, {}] must be an empty field to plant a seed.",
            x, y
        )));
    }

    farm.plant_seed(
        x.into(),
        y.into(),
        &plant_type,
        Some(komple),
        env.block.height,
    );
    FARM_PROFILES.save(deps.storage, sender.as_str(), &farm)?;

    Ok(Response::new().add_attribute("action", "seed"))
}

#[cfg(test)]
mod test {}
