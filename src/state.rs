use std::collections::BTreeMap;

use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

use crate::{
    farm::{Plant, PlantType, Slot, SlotType},
    msg::ContractInformationResponse,
};

#[cw_serde]
pub struct FarmProfile {
    plots: Vec<Vec<Slot>>,
}

pub const FARM_PROFILES: Map<&str, FarmProfile> = Map::new("farm_profiles");
pub const INFORMATION: Item<ContractInformationResponse> = Item::new("info");

fn create_meadow_plot() -> Slot {
    return Slot {
        r#type: SlotType::Meadow,
        plant: None,
    };
}

fn create_field_plot() -> Slot {
    return {
        Slot {
            r#type: SlotType::Field,
            plant: None,
        }
    };
}

fn create_sunflower_plant() -> Plant {
    return Plant {
        r#type: PlantType::Sunflower,
        stages: 5,
        current_stage: 1,
        dead: false,
    };
}

impl FarmProfile {
    pub fn new() -> Self {
        let initial_plots = 3;

        let mut plots = vec![];
        for _ in 0..initial_plots {
            let mut row = vec![];
            for _ in 0..initial_plots {
                row.push(Slot {
                    r#type: SlotType::Meadow,
                    plant: None,
                });
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

        return col.unwrap().clone();
    }

    pub fn set_plot(&mut self, x: usize, y: usize, value: Slot) {
        self.plots[x][y] = value;
    }

    pub fn upgrade_size(&mut self, amount: usize) -> FarmProfile {
        for row in &mut self.plots {
            for _ in 0..amount {
                row.push(create_meadow_plot());
            }
        }

        let mut new_row = vec![];
        for _ in 0..self.get_size() + amount {
            new_row.push(create_meadow_plot());
        }

        for _ in 0..amount {
            self.plots.push(new_row.clone());
        }

        println!(
            "\nUpgrading farm size to {}x{}",
            self.get_size(),
            self.get_size()
        );

        self.clone()
    }

    pub fn till(&mut self, x: usize, y: usize) -> FarmProfile {
        if self.get_plot(x, y).r#type == SlotType::Meadow {
            self.set_plot(x, y, create_field_plot());
            println!("Tilled plot at {}, {}", x, y);
        }
        self.clone()
    }

    pub fn plant_seed(&mut self, x: usize, y: usize) -> FarmProfile {
        let plot = self.get_plot(x, y);
        let plant = plot.plant;
        if plot.r#type == SlotType::Field && plant.is_none() {
            self.set_plot(
                x,
                y,
                Slot {
                    plant: Some(create_sunflower_plant()),
                    ..plot
                },
            );
        }

        self.clone()
    }

    pub fn water_plant(&mut self, x: usize, y: usize) -> FarmProfile {
        let plot = self.get_plot(x, y);
        let plant = plot.plant;
        let updated_plant = plant.map(|p| {
            if p.stages > p.current_stage {
                return Plant {
                    current_stage: p.current_stage + 1,
                    ..p
                };
            }

            p
        });

        self.set_plot(
            x,
            y,
            Slot {
                plant: updated_plant,
                ..plot
            },
        );

        self.clone()
    }

    pub fn harvest(&mut self, x: usize, y: usize) -> FarmProfile {
        let plot = self.get_plot(x, y);
        let plant = plot.plant;
        let updated_plant = plant.map(|p| {
            if p.stages == p.current_stage {
                return None;
            }

            Some(p)
        });

        self.set_plot(
            x,
            y,
            Slot {
                plant: updated_plant.unwrap(),
                ..plot
            },
        );

        self.clone()
    }
}
