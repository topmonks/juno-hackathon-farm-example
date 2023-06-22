use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum SlotType {
    Meadow,
    Field,
}

#[cw_serde]
pub struct Plant {
    pub r#type: String,
}

#[cw_serde]
pub struct Slot {
    pub r#type: SlotType,
    pub plant: Option<Plant>,
}
