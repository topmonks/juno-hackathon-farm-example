use std::collections::BTreeMap;

pub enum Item {
    Grass,
    Dirt,
    Seed,
    Cow,
}

type Cooldowns = BTreeMap<(i32, i32), i32>;

// where str is the address
type Farms = BTreeMap<String, Farm>;

#[derive(Clone)]
pub struct Farm {
    plots: Vec<Vec<Item>>,
    cooldowns: Cooldowns,
}

// add Debug trait to Item
impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Grass => write!(f, "Grass"),
            Item::Dirt => write!(f, "Dirt"),
            Item::Seed => write!(f, "Seed"),
            Item::Cow => write!(f, "Cow"),
        }
    }
}

// impl eq trait for Item
impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Item::Grass, Item::Grass) => true,
            (Item::Dirt, Item::Dirt) => true,
            (Item::Seed, Item::Seed) => true,
            (Item::Cow, Item::Cow) => true,
            _ => false,
        }
    }
}

// impl copy for Item
impl Clone for Item {
    fn clone(&self) -> Self {
        match self {
            Item::Grass => Item::Grass,
            Item::Dirt => Item::Dirt,
            Item::Seed => Item::Seed,
            Item::Cow => Item::Cow,
        }
    }
}

impl Copy for Item {}

// Main
fn main() {

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

    // these are backwards, Y is up and down.
    let farm2 = farm2.clone().till(2, 1);

    // let updated_farm = farm2.clone().upgrade_size(1);
    // farms_state.insert(addr2.to_string(), updated_farm);

    // print the updated farm
    // let farm2 = farms_state.get(addr2).unwrap();
    println!("Farm size: {}", farm2.get_size());
    println!("Farm plots: {}", farm2.get_plots());

    // let farm_item = farm2.get_plot(1, 1);
    // println!("Farm item: {:?}", farm_item);
}

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
        self.plots[y][x]
    }

    pub fn set_plot(&mut self, x: usize, y: usize, value: Item) {
        // reversed since we are doing top bottom left as 0,0. Same for get_plot()
        let size = self.get_size()-1;
        self.plots[size-y][x] = value;
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

    pub fn get_cooldown(&self, x: usize, y: usize) -> i32 {
        *self.cooldowns.get(&(x as i32, y as i32)).unwrap_or(&0)
    }

    pub fn find_plots_with_type(&self, item: Item) -> Vec<(usize, usize)> {
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