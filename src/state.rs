use std::collections::BTreeMap;

use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

use crate::{
    farm::{Slot, SlotType},
    msg::ContractInformationResponse,
};

type Cooldowns = BTreeMap<(i32, i32), u32>;

#[cw_serde]
pub struct FarmProfile {
    plots: Vec<Vec<Slot>>,
    cooldowns: Cooldowns,
}

// addresss: FarmProfile

pub const FARM_PROFILES: Map<&str, FarmProfile> = Map::new("farm_profiles");

// Config configuration Information
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

        FarmProfile {
            plots,
            cooldowns: Cooldowns::new(),
        }
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

        // add 0..amount to the bottom
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
        self.clone()
    }
}
