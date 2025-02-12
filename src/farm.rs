use std::str::FromStr;

use cosmwasm_schema::cw_serde;

use crate::{params, ContractError};

#[cw_serde]
pub enum SlotType {
    Meadow,
    Field,
}

#[cw_serde]
#[derive(Eq, Hash)]
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
pub struct KomplePlant {
    pub metadata_id: u32,
    pub collection_id: u32,
}

#[cw_serde]
pub struct Plant {
    pub r#type: PlantType,
    pub stages: u64,
    pub growth_period: u64,
    pub created_at: u64,
    pub watered_at: Vec<u64>,
    pub komple: Option<KomplePlant>,
}

impl Plant {
    pub fn get_current_stage(&self, block: u64) -> u64 {
        let passed_time = block - self.created_at;

        passed_time / self.growth_period + 1
    }

    pub fn is_dead(&self, block: u64) -> bool {
        let watered_stages: u64 = self.watered_at.len().try_into().unwrap();
        let current_stage = self.get_current_stage(block);

        watered_stages + 1 < current_stage
    }

    pub fn can_water(&self, block: u64) -> bool {
        let watered_stages: u64 = self.watered_at.len().try_into().unwrap();
        let current_stage = self.get_current_stage(block);

        watered_stages < self.stages && watered_stages == current_stage - 1
    }

    pub fn can_harvest(&self, block: u64) -> bool {
        let watered_stages: u64 = self.watered_at.len().try_into().unwrap();
        let current_stage = self.get_current_stage(block);

        self.stages == current_stage && watered_stages == self.stages
    }
}

#[cw_serde]
pub struct Slot {
    pub r#type: SlotType,
    pub plant: Option<Plant>,
    pub created_at: u64,
}

impl Slot {
    pub fn is_field_turned_meadow(&self, block: u64) -> bool {
        self.r#type == SlotType::Field
            && match &self.plant {
                None => block - self.created_at > params::FIELD_TURNS_MEADOW,
                Some(_) => false,
            }
    }

    pub fn get_real_type(&self, block: u64) -> SlotType {
        if self.is_field_turned_meadow(block) {
            return SlotType::Meadow;
        }

        self.r#type.clone()
    }

    pub fn can_till(&self, block: u64) -> bool {
        match self.get_real_type(block) {
            SlotType::Meadow => true,
            SlotType::Field => match &self.plant {
                None => false,
                Some(plant) => plant.is_dead(block),
            },
        }
    }
}
