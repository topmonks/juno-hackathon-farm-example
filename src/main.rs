use std::{collections::BTreeMap, ops::Sub};

pub enum Item {
    Grass,
    Dirt,
    WheatSeed,
    WheatHarvestable,
    Cow,
}
impl Item {
    // cooldown interaction block wait times
    fn value(&self) -> u32 {
        match *self {
            // 0 means there is no interaction cooldown
            Item::Grass => 0,
            Item::Dirt => 10000, // save a last interation time? reset this every till / plant, etc?
            Item::WheatSeed => 10,
            Item::WheatHarvestable => 0,
            Item::Cow => 100,
        }
    }


    fn pair(&self) -> Item {
        match *self {
            // items which are the final form of the seed
            Item::Grass => Item::Grass,
            Item::Dirt => Item::Dirt,
            Item::WheatSeed => Item::WheatHarvestable,
            Item::WheatHarvestable => Item::WheatHarvestable,
            Item::Cow => Item::Cow, // add ReadyToMilkCow?
        }
    }
}


type Cooldowns = BTreeMap<(i32, i32), u32>;

// where str is the address
type Farms = BTreeMap<String, Farm>;


// type Statistics? impl leaderboard queries

#[derive(Clone, Debug)]
pub struct Farm {
    plots: Vec<Vec<Item>>,
    cooldowns: Cooldowns,
    // inventory here OR use tokenfactory tokens. Using TFTokens, it would replace items to be denoms
}

// add Debug trait to Item
impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Grass => write!(f, "Grass"),
            Item::Dirt => write!(f, "Dirt"),
            Item::WheatSeed => write!(f, "Wheat Seed"),
            Item::WheatHarvestable => write!(f, "Wheat"),
            Item::Cow => write!(f, "Cow"),
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Item::Grass, Item::Grass) => true,
            (Item::Dirt, Item::Dirt) => true,
            (Item::WheatSeed, Item::WheatSeed) => true,
            (Item::WheatHarvestable, Item::WheatHarvestable) => true,
            (Item::Cow, Item::Cow) => true,
            _ => false,
        }
    }
}

impl Copy for Item {}

impl Clone for Item {
    fn clone(&self) -> Self {
        match self {
            Item::Grass => Item::Grass,
            Item::Dirt => Item::Dirt,
            Item::WheatSeed => Item::WheatSeed,
            Item::WheatHarvestable => Item::WheatHarvestable,
            Item::Cow => Item::Cow,
        }
    }
}


// Main
fn main() {

    let mut current_block_height: u32 = 0;

    let addr1 = "juno1reece";
    let addr2 = "juno1alice";

    let mut farms_state = Farms::new();
    let my_farm = Farm::new();

    farms_state.insert(addr1.to_string(), my_farm.clone());
    farms_state.insert(addr2.to_string(), my_farm);


    let farm1 = farms_state.get(addr1).unwrap();
    println!("Farm size: {}", farm1.get_size());
    println!("Farm plots: {}", farm1.get_plots());

    let farm2 = farms_state.get(addr2).unwrap();
    // println!("Farm size: {}", farm2.get_size());
    // println!("Farm plots: {}", farm2.get_plots());

    let farm2 = farm2.clone().till(0, 0);
    let farm2 = farm2.clone().till(2, 0);
    let farm2 = farm2.clone().till(1, 1);
    let farm2 = farm2.clone().till(2, 2);
    // let farm2 = farm2.clone().till(88, 88); // fails

    // let updated_farm = farm2.clone().upgrade_size(1);
    // farms_state.insert(addr2.to_string(), updated_farm);

    // print the updated farm
    // let farm2 = farms_state.get(addr2).unwrap();
    println!("Farm size: {}", farm2.get_size());
    println!("Farm plots: {}", farm2.get_plots());
    println!("Farm All Unique Items: {:?}", farm2.get_all_unique_plot_items());
    println!("Query Farm Specific dirt plots: {:?}", farm2.get_plots_with_type(Item::Dirt));

    let check = farm2.get_plot(0, 2);
    println!("Farm item @ (0,2): {:?}", check);

    // let farm_item = farm2.get_plot(1, 1);
    // println!("Farm item: {:?}", farm_item);

    // update farm2 to farm_state
    farms_state.insert(addr1.to_string(), farm2.clone());

    let user_farm = query_farm(&farms_state, addr1, current_block_height);
    println!("User Farm: {:?}", user_farm);

    // plant_seed
    let farm2 = farm2.clone().plant_seed(0, 0); // top right
    println!("Farm plots: {}", farm2.get_plots());
    // / get cooldowns
    println!("Farm plot cooldowns: {:?}", farm2.get_cooldowns());
    
    // NOTE: on query, we need to show wheat when getting plots when the cooldown is over
    // interact with farm2 0, 0 
    let farm2 = farm2.clone().interact(0, 0, current_block_height);
    println!("Farm plots: {}", farm2.get_plots());
    println!("Farm plot cooldowns: {:?}", farm2.get_cooldowns());
    current_block_height += 11;

    //  save farm to state
    farms_state.insert(addr1.to_string(), farm2.clone());

    // interact with farm2 0, 0
    // let farm2 = farm2.clone().interact(0, 0, current_block_height);
    // println!("Farm plots: {}", farm2.get_plots());

    // query_farm
    let user_farm = query_farm(&farms_state, addr1, current_block_height);
    println!("User Farm: {:?}", user_farm);

    // interact now and harest the wheat since more blocks have passed
    let farm2 = farm2.clone().interact(0, 0, current_block_height);
    // save to state
    farms_state.insert(addr1.to_string(), farm2.clone());

    // query_farm
    let user_farm = query_farm(&farms_state, addr1, current_block_height);
    println!("User Farm: {:?}", user_farm);


}

pub fn query_farm(farm_state: &Farms, addr: &str, current_block_height: u32) -> Farm {
    let farm = farm_state.get(addr).unwrap().clone();
    // loop over each plot and check if cooldown is over
    // if it is, replace the item with the next stage
    let mut temp_farm = farm.clone();
    for (i, row) in farm.plots.iter().enumerate() {
        for (j, item) in row.iter().enumerate() {
            let current_cooldown = farm.cooldowns.get(&(i as i32, j as i32));

            if current_cooldown.is_some() {
                if current_block_height >= current_cooldown.unwrap().to_owned() {
                    temp_farm.plots[i][j] = Item::pair(item);
                }
            } else {
                temp_farm.plots[i][j] = item.to_owned();
            }
        }
    }

    temp_farm
}

// These return a farm object right now since I can't easily/simplify save in func state. This is just a POC
// will do when it comes to CW
impl Farm {
    pub fn new() -> Self {
        let initial_plots = 3;

        let mut plots = vec![];
        for _ in 0..initial_plots {
            let mut row = vec![];
            for _ in 0..initial_plots {
                row.push(Item::Grass);
            }
            plots.push(row);
        }

        Farm {
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

    pub fn get_plot(&self, x: usize, y: usize) -> Item {
        // required since we use the left bottom as 0,0. Same for set_plot()
        if x > self.get_size() || y > self.get_size() {
            // throw error
        }
        // self.plots.get(y).unwrap().get(x).unwrap().clone()
        self.plots[y][x]
    }

    pub fn set_plot(&mut self, x: usize, y: usize, value: Item) {
        // reversed since we are doing top bottom left as 0,0. Same for get_plot()
        // let size = self.get_size()-1;

        // nvm top left is the 0,0
        self.plots[y][x] = value;
    }

    pub fn upgrade_size(&mut self, amount: usize) -> Farm {
        for row in &mut self.plots {
            for _ in 0..amount {
                row.push(Item::Grass);
            }
        }

        for _ in 0..amount {
            let mut row = vec![];
            for _ in 0..self.get_size() + amount {
                row.push(Item::Grass);
            }
            self.plots.push(row);
        }

        self.clone()
    }

    pub fn till(&mut self, x: usize, y: usize) -> Farm {
        if self.get_plot(x, y) == Item::Grass {
            self.set_plot(x, y, Item::Dirt);
        }
        self.clone()
    }
    // Till All?

    pub fn plant_seed(&mut self, x: usize, y: usize) -> Farm {
        if self.get_plot(x, y) == Item::Dirt {
            println!("Planted seed at {}, {}", x, y);
            self.set_plot(x, y, Item::WheatSeed);
            // add to cooldown for 5 blocks. After this is up, the user can harvest

            // where we set blockheight+amount
            self.cooldowns.insert((x.try_into().unwrap(), y.try_into().unwrap()), Item::value(&Item::WheatSeed));
        } else {
            println!("Failed to plant seed at {}, {}", x, y);
        }
        self.clone()
    }

    pub fn interact(&mut self, x: usize, y: usize, current_block_height: u32) -> Farm {
        // check cooldown for the plot, if active, require user to wait. Else, if seeds, give Wheat
        let on_cooldown = self.get_cooldown(x, y) > current_block_height;
        if on_cooldown {
            println!("Cooldown active for {}, {} for {} more blocks", x, y, self.get_cooldown(x, y) - current_block_height);
        } else {
            let plot = self.get_plot(x, y);

            if plot == Item::WheatSeed {
                println!("Harvested wheat at {}, {}. Setting to Dirt", x, y);
                self.set_plot(x, y, Item::Dirt);

                // give user wheat item here OR mint Wheat tokenfactory token
            } else {
                println!("Nothing to harvest at {}, {}", x, y);
            }
        }

        self.clone()
    }

    pub fn get_cooldowns(&self) -> &Cooldowns {
        // This is when a user can interact with ths plot again in the future
        &self.cooldowns
    }

    pub fn get_cooldown(&self, x: usize, y: usize) -> u32 {
        *self.cooldowns.get(&(x.try_into().unwrap(), y.try_into().unwrap())).unwrap_or(&0)
    }

    pub fn get_plots_with_type(&self, item: Item) -> Vec<(usize, usize)> {
        let mut plots = vec![];
        for x in 0..self.get_size() {
            for y in 0..self.get_size() {
                if self.get_plot(x, y) == item {
                    plots.push((x, y));
                }
            }
        }
        plots
    }

    pub fn get_all_unique_plot_items(&self) -> Vec<Item> {
        let mut items = vec![];
        for x in 0..self.get_size() {
            for y in 0..self.get_size() {
                let item = self.get_plot(x, y);
                if !items.contains(&item) {
                    items.push(item);
                }
            }
        }
        items
    }

}