use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Different types of items that can be placed on the farm
#[derive(std::fmt::Debug, JsonSchema, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum FarmItem {
    Air, // Air / Void
    Grass,
    Dirt,
    WheatSeed,
    WheatHarvestable,
    Cow,
}

impl FarmItem {
    // cooldown interaction block wait times for each Enum Value
    pub fn value(&self) -> u32 {
        match *self {
            // 0 means there is no interaction cooldown
            FarmItem::Air => 0,
            FarmItem::Grass => 0,
            FarmItem::Dirt => 10000,
            FarmItem::WheatSeed => 10,
            FarmItem::WheatHarvestable => 0,
            FarmItem::Cow => 100,
        }
    }

    pub fn pair(&self) -> FarmItem {
        match *self {
            // items which are the final form of the seed
            FarmItem::Air => FarmItem::Air,
            FarmItem::Grass => FarmItem::Grass,
            FarmItem::Dirt => FarmItem::Dirt,
            FarmItem::WheatSeed => FarmItem::WheatHarvestable,
            FarmItem::WheatHarvestable => FarmItem::WheatHarvestable,
            FarmItem::Cow => FarmItem::Cow, // add ReadyToMilkCow?
        }
    }
}

impl Copy for FarmItem {}

impl Clone for FarmItem {
    fn clone(&self) -> Self {
        match self {
            FarmItem::Air => FarmItem::Air,
            FarmItem::Grass => FarmItem::Grass,
            FarmItem::Dirt => FarmItem::Dirt,
            FarmItem::WheatSeed => FarmItem::WheatSeed,
            FarmItem::WheatHarvestable => FarmItem::WheatHarvestable,
            FarmItem::Cow => FarmItem::Cow,
        }
    }
}

// impl seralize and deserialize for FarmItem
impl Serialize for FarmItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FarmItem::Air => serializer.serialize_str("Air"),
            FarmItem::Grass => serializer.serialize_str("Grass"),
            FarmItem::Dirt => serializer.serialize_str("Dirt"),
            FarmItem::WheatSeed => serializer.serialize_str("WheatSeed"),
            FarmItem::WheatHarvestable => serializer.serialize_str("WheatHarvestable"),
            FarmItem::Cow => serializer.serialize_str("Cow"),
        }
    }
}

impl<'de> Deserialize<'de> for FarmItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Air" => Ok(FarmItem::Air),
            "Grass" => Ok(FarmItem::Grass),
            "Dirt" => Ok(FarmItem::Dirt),
            "WheatSeed" => Ok(FarmItem::WheatSeed),
            "WheatHarvestable" => Ok(FarmItem::WheatHarvestable),
            "Cow" => Ok(FarmItem::Cow),
            _ => Err(serde::de::Error::custom("invalid FarmItem")),
        }
    }
}
