use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

use crate::{
    farm::{KomplePlant, Plant, PlantType, Slot, SlotType},
    helpers::throw_err,
    msg::ContractInformation,
    params, ContractError,
};

fn plant_dto(plant: &Option<Plant>, block: u64) -> Option<PlantDto> {
    plant.as_ref().map(|plant| PlantDto {
        created_at: plant.created_at,
        growth_period: plant.growth_period,
        komple: plant.komple.clone(),
        stages: plant.stages,
        r#type: plant.r#type.clone(),
        watered_at: plant.watered_at.clone(),
        can_harvest: plant.can_harvest(block),
        can_water: plant.can_water(block),
        current_stage: plant.get_current_stage(block),
        is_dead: plant.is_dead(block),
    })
}

fn slot_dto(slot: &Slot, block: u64) -> SlotDto {
    SlotDto {
        plant: plant_dto(&slot.plant, block),
        r#type: slot.get_real_type(block),
        can_till: slot.can_till(block),
        created_at: slot.created_at,
    }
}

#[cw_serde]
pub struct FarmProfile {
    plots: Vec<Vec<Slot>>,
}

pub fn farm_profile_dto(farm_profile: &Option<FarmProfile>, block: u64) -> Option<FarmProfileDto> {
    farm_profile.as_ref().map(|farm_profile| FarmProfileDto {
        plots: farm_profile
            .plots
            .iter()
            .map(|rows| rows.iter().map(|slot| slot_dto(slot, block)).collect())
            .collect(),
        blocks: block,
    })
}

#[cw_serde]
pub struct PlantDto {
    pub r#type: PlantType,
    pub stages: u64,
    pub growth_period: u64,
    pub created_at: u64,
    pub watered_at: Vec<u64>,
    pub komple: Option<KomplePlant>,
    pub can_water: bool,
    pub can_harvest: bool,
    pub current_stage: u64,
    pub is_dead: bool,
}

#[cw_serde]
pub struct SlotDto {
    pub r#type: SlotType,
    pub plant: Option<PlantDto>,
    pub can_till: bool,
    pub created_at: u64,
}

#[cw_serde]
pub struct FarmProfileDto {
    plots: Vec<Vec<SlotDto>>,
    blocks: u64,
}

#[cw_serde]
pub enum NoiseJob {
    MintSeeds {
        plant: KomplePlant,
        recipient: String,
    },
}

pub const FARM_PROFILES: Map<&str, FarmProfile> = Map::new("farm_profiles");
pub const INFORMATION: Item<ContractInformation> = Item::new("info");
pub const NOIS_PROXY: Item<Addr> = Item::new("nois_proxy");
pub const NOIS_JOB_LAST_ID: Item<u64> = Item::new("nois_job_last_id");
pub const NOIS_JOBS: Map<&str, NoiseJob> = Map::new("nois_jobs");

#[cw_serde]
pub struct Points {
    pub addr: String,
    pub plants: HashMap<String, u64>,
}

impl Points {
    pub fn add(&mut self, plant_type: PlantType) {
        let plant_type_str = plant_type.to_string();
        let plant_points = self.plants.get(&plant_type_str).unwrap_or(&0);
        self.plants.insert(plant_type_str, plant_points + 1);
    }

    pub fn total(&self) -> u64 {
        self.plants
            .values()
            .copied()
            .reduce(|acc, next| acc + next)
            .unwrap_or(0)
    }
}

pub struct PointsIndexes<'a> {
    pub total: MultiIndex<'a, u64, Points, &'a str>,
}

impl<'a> IndexList<Points> for PointsIndexes<'a> {
    fn get_indexes(
        &'_ self,
    ) -> Box<dyn Iterator<Item = &'_ dyn cw_storage_plus::Index<Points>> + '_> {
        let v: Vec<&dyn Index<Points>> = vec![&self.total];
        Box::new(v.into_iter())
    }
}

pub fn points<'a>() -> IndexedMap<'a, &'a str, Points, PointsIndexes<'a>> {
    IndexedMap::new(
        "points",
        PointsIndexes {
            total: MultiIndex::new(|_k, t| t.total(), "points", "points_total"),
        },
    )
}

fn create_meadow_plot(block: u64) -> Slot {
    Slot {
        r#type: SlotType::Meadow,
        plant: None,
        created_at: block,
    }
}

fn create_field_plot(block: u64) -> Slot {
    Slot {
        r#type: SlotType::Field,
        plant: None,
        created_at: block,
    }
}

fn create_plant(plant_type: &PlantType, komple: Option<KomplePlant>, block: u64) -> Plant {
    match plant_type {
        PlantType::Sunflower => Plant {
            r#type: PlantType::Sunflower,
            stages: 5,
            komple,
            growth_period: params::GROWTH_PERIOD_SUNFLOWER,
            created_at: block,
            watered_at: vec![block],
        },
        PlantType::Wheat => Plant {
            r#type: PlantType::Wheat,
            stages: 4,
            komple,
            growth_period: params::GROWTH_PERIOD_WHEET,
            created_at: block,
            watered_at: vec![block],
        },
    }
}

impl FarmProfile {
    pub fn new(block: u64) -> Self {
        let initial_plots = 9;

        let mut plots = vec![];
        for _ in 0..initial_plots {
            let mut row = vec![];
            for _ in 0..initial_plots {
                row.push(create_meadow_plot(block));
            }
            plots.push(row);
        }

        FarmProfile { plots }
    }

    pub fn get_size(&self) -> usize {
        self.plots.len()
    }

    pub fn get_plots(&self) -> String {
        let mut output = String::new();
        for row in &self.plots {
            output.push_str(&format!("\n {:?}", row));
        }
        output
    }

    pub fn get_plot(&self, x: usize, y: usize) -> Slot {
        if x > self.get_size() || y > self.get_size() {
            // throw error
        }

        let row = self.plots.get(x);
        let col = row.unwrap().get(y);

        col.unwrap().clone()
    }

    pub fn set_plot(&mut self, x: usize, y: usize, value: Slot) {
        self.plots[x][y] = value;
    }

    pub fn upgrade_size(&mut self, amount: usize, block: u64) {
        for row in &mut self.plots {
            for _ in 0..amount {
                row.push(create_meadow_plot(block));
            }
        }

        let mut new_row = vec![];
        for _ in 0..self.get_size() + amount {
            new_row.push(create_meadow_plot(block));
        }

        for _ in 0..amount {
            self.plots.push(new_row.clone());
        }

        println!(
            "\nUpgrading farm size to {}x{}",
            self.get_size(),
            self.get_size()
        );
    }

    pub fn till(&mut self, x: usize, y: usize, block: u64) -> Result<(), ContractError> {
        let plot = self.get_plot(x, y);
        if !plot.can_till(block) {
            return Err(throw_err(&format!(
                "Plot [{}, {}] must be meadow or field with dead plant to till",
                x, y
            )));
        }

        self.set_plot(x, y, create_field_plot(block));

        Ok(())
    }

    pub fn plant_seed(
        &mut self,
        x: usize,
        y: usize,
        plant_type: &PlantType,
        komple: Option<KomplePlant>,
        block: u64,
    ) {
        let plot = self.get_plot(x, y);
        if plot.get_real_type(block) == SlotType::Field && plot.plant.is_none() {
            self.set_plot(
                x,
                y,
                Slot {
                    plant: Some(create_plant(plant_type, komple, block)),
                    ..plot
                },
            );
        }
    }

    pub fn water_plant(&mut self, x: usize, y: usize, block: u64) -> Result<(), ContractError> {
        let plot = self.get_plot(x, y);
        let updated_plant = match plot.plant {
            None => Err(throw_err(&format!(
                "Plot [{}, {}] must contain a plant to water.",
                x, y
            ))),
            Some(plant) => {
                if !plant.can_water(block) {
                    if plant.can_harvest(block) {
                        return Err(throw_err(&format!(
                            "Plant [{}, {}] is fully grown and cannot be watered anymore.",
                            x, y
                        )));
                    }

                    if plant.is_dead(block) {
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

                let mut p2 = plant.clone();
                p2.watered_at.push(block);

                Ok(p2)
            }
        }?;

        self.set_plot(
            x,
            y,
            Slot {
                plant: Some(updated_plant),
                ..plot
            },
        );

        Ok(())
    }

    pub fn harvest(&mut self, x: usize, y: usize, block: u64) -> Result<PlantType, ContractError> {
        let plot = self.get_plot(x, y);

        match plot.plant {
            None => Err(throw_err(&format!(
                "Plot [{}, {}] must contain a plant to harvest.",
                x, y
            ))),
            Some(plant) => {
                if !plant.can_harvest(block) {
                    return Err(throw_err(&format!(
                        "Plant [{}, {}] must be fully grown and watered to harvest it.",
                        x, y
                    )));
                }

                self.set_plot(x, y, create_field_plot(block));

                Ok(plant.r#type)
            }
        }
    }
}
