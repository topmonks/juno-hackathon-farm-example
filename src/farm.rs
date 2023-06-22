use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cw_serde]
pub enum FarmItem {
    Meadow,
    Field,
}

#[cw_serde]
pub struct Plant {
    pub r#type: String,
}

#[cw_serde]
pub struct Slot {
    pub r#type: FarmItem,
    pub plant: Option<Plant>,
}

// impl FarmItem {
//     // cooldown interaction block wait times for each Enum Value
//     pub fn value(&self) -> u32 {
//         match *self {
//             // 0 means there is no interaction cooldown
//             FarmItem::Meadow => 0,
//             FarmItem::Field => 10000,
//         }
//     }

//     pub fn pair(&self) -> FarmItem {
//         match *self {
//             // items which are the final form of the seed
//             FarmItem::Meadow => FarmItem::Meadow,
//             FarmItem::Field => FarmItem::Field,
//         }
//     }
// }

// impl Copy for FarmItem {}

// impl Clone for FarmItem {
//     fn clone(&self) -> Self {
//         match self {
//             FarmItem::Meadow => FarmItem::Meadow,
//             FarmItem::Field => FarmItem::Field,
//         }
//     }
// }

// impl seralize and deserialize for FarmItem
// impl Serialize for FarmItem {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         match self {
//             FarmItem::Meadow => serializer.serialize_str("Meadow"),
//             FarmItem::Field => serializer.serialize_str("Field"),
//         }
//     }
// }

// impl<'de> Deserialize<'de> for FarmItem {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s = String::deserialize(deserializer)?;
//         match s.as_str() {
//             "Meadow" => Ok(FarmItem::Meadow),
//             "Field" => Ok(FarmItem::Field),
//             _ => Err(serde::de::Error::custom("invalid FarmItem")),
//         }
//     }
// }
