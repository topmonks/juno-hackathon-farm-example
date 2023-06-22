use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw721::Cw721ReceiveMsg;

use crate::state::FarmProfile;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub whitelisted_collections: Option<Vec<Addr>>,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Start {},
    Stop {},
    TillGround { x: u8, y: u8 },
    PlantSeed { x: u8, y: u8 },
    Receive(Cw721ReceiveMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    Seed {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ContractInformationResponse)]
    ContractInfo {},

    // Returns a specific users farm profile from state via query
    #[returns(FarmProfile)]
    GetFarmProfile { address: String },
}

// === RESPONSES ===
#[cw_serde]
pub struct ContractInformationResponse {
    pub admin: String,
    pub whitelisted_collections: Vec<Addr>,
}
