use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum SlotType {
    Meadow,
    Field,
}

#[cw_serde]
pub enum PlantType {
    Sunflower,
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
