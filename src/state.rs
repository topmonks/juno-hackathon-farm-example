use std::collections::BTreeMap;

use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

use crate::{farm::FarmItem, msg::ContractInformationResponse};

type Cooldowns = BTreeMap<(i32, i32), u32>;

#[cw_serde]
pub struct FarmProfile {
    plots: Vec<Vec<FarmItem>>,
    cooldowns: Cooldowns,
    // inventory here *or tokenfactory tokens?*

    // https://docs.junonetwork.io/developer-guides/juno-modules/tokenfactory
    // If you use tokenfactory tokens, are they held by the user, or in the contract on behalf of the user?
    // Maybe a farm marketplace so other users can sell & buy seeds for tokens?

    // upgrades?
}

// addresss: FarmProfile

pub const FARM_PROFILES: Map<&str, FarmProfile> = Map::new("farm_profiles");

// Config configuration Information
pub const INFORMATION: Item<ContractInformationResponse> = Item::new("info");

impl FarmProfile {
    pub fn new() -> Self {
        let initial_plots = 3;

        let mut plots = vec![];
        for _ in 0..initial_plots {
            let mut row = vec![];
            for _ in 0..initial_plots {
                row.push(FarmItem::Meadow);
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

    pub fn get_plot(&self, x: usize, y: usize) -> FarmItem {
        // Reverse order is required since we use the left bottom as 0,0. Same for set_plot()
        if x > self.get_size() || y > self.get_size() {
            // throw error
        }

        let row = self.plots.get(x);
        let col = row.unwrap().get(y);

        return col.unwrap().clone();
    }

    pub fn set_plot(&mut self, x: usize, y: usize, value: FarmItem) {
        // edge cases? what if its air?
        self.plots[x][y] = value;
    }

    pub fn upgrade_size(&mut self, amount: usize) -> FarmProfile {
        for row in &mut self.plots {
            for _ in 0..amount {
                row.push(FarmItem::Meadow);
            }
        }

        let mut new_row = vec![];
        for _ in 0..self.get_size() + amount {
            new_row.push(FarmItem::Meadow);
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
        if self.get_plot(x, y) == FarmItem::Meadow {
            self.set_plot(x, y, FarmItem::Field);
            println!("Tilled plot at {}, {}", x, y);
        }
        self.clone()
    }

    pub fn plant_seed(&mut self, x: usize, y: usize) -> FarmProfile {
        // if self.get_plot(x, y) == FarmItem::Field {
        //     println!("Planted seed at {}, {}", x, y);
        //     self.set_plot(x, y, FarmItem::WheatSeed);
        //     // add to cooldown for 5 blocks. After this is up, the user can harvest

        //     // where we set blockheight+amount
        //     self.cooldowns.insert(
        //         (x.try_into().unwrap(), y.try_into().unwrap()),
        //         FarmItem::value(&FarmItem::WheatSeed),
        //     );
        // } else {
        //     println!("Failed to plant seed at {}, {}", x, y);
        // }
        self.clone()
    }

    pub fn interact(&mut self, x: usize, y: usize, _current_block_height: u32) -> FarmProfile {
        // add a cooldown?

        let plot = self.get_plot(x, y);

        // if plot == FarmItem::WheatSeed {
        //     println!("\nHarvested wheat at {}, {}. Setting to Dirt", x, y);
        //     self.set_plot(x, y, FarmItem::Field);

        //     // give user wheat item here OR mint Wheat tokenfactory token
        // } else {
        //     println!("Nothing to harvest at {}, {}", x, y);
        // }

        self.clone()
    }

    pub fn get_cooldowns(&self, current_block_height: u32) -> Cooldowns {
        // This is when a user can interact with ths plot again in the future
        // &self.cooldowns
        let mut updated_cooldowns: Cooldowns = self.cooldowns.clone();
        for (key, value) in &self.cooldowns {
            if value < &current_block_height {
                updated_cooldowns.remove(key);
            }
        }

        return updated_cooldowns;
    }

    pub fn get_specific_cooldown(&self, x: usize, y: usize) -> u32 {
        *self
            .cooldowns
            .get(&(x.try_into().unwrap(), y.try_into().unwrap()))
            .unwrap_or(&0)
    }

    pub fn get_plots_with_type(&self, item: FarmItem) -> Vec<(usize, usize)> {
        let mut plots = vec![];
        // todo: get all plot coords with a specific type
        plots
    }

    pub fn get_all_unique_plot_items(&self) -> Vec<FarmItem> {
        let mut items = vec![];
        // todo: get every unique item a plot has, what about inventory?
        items
    }
}
