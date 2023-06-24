use std::str::FromStr;

use cosmwasm_schema::cw_serde;

use crate::ContractError;

#[cw_serde]
pub enum SlotType {
    Meadow,
    Field,
}

#[cw_serde]
pub enum PlantType {
    Sunflower,
    Wheat,
}

impl FromStr for PlantType {
    type Err = ContractError;

    fn from_str(s: &str) -> Result<Self, ContractError> {
        match s {
            "sunflower" => Ok(PlantType::Sunflower),
            "wheat" => Ok(PlantType::Wheat),
            name => Err(ContractError::UnknownPlant {
                name: name.to_string(),
            }),
        }
    }
}

impl ToString for PlantType {
    fn to_string(&self) -> String {
        match self {
            PlantType::Sunflower => String::from("sunflower"),
            PlantType::Wheat => String::from("wheat"),
        }
    }
}

#[cw_serde]
pub struct Plant {
    pub r#type: PlantType,
    pub current_stage: u8,
    pub stages: u8,
    pub dead: bool,
}

#[cw_serde]
pub struct Slot {
    pub r#type: SlotType,
    pub plant: Option<Plant>,
}
