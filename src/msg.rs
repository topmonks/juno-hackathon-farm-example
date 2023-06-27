use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw721::Cw721ReceiveMsg;

use crate::state::FarmProfileDto;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub komple_mint_addr: Option<String>,
    pub whitelisted_collections: Option<Vec<KompleCollection>>,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Start {},
    Stop {},
    TillGround { x: u8, y: u8 },
    WaterPlant { x: u8, y: u8 },
    Harvest { x: u8, y: u8 },
    ReceiveNft(Cw721ReceiveMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    Seed { x: u8, y: u8 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ContractInformationResponse)]
    ContractInfo {},

    // Returns a specific users farm profile from state via query
    #[returns(FarmProfileDto)]
    GetFarmProfile { address: String },
}

// === RESPONSES ===
#[cw_serde]
pub struct KompleCollection {
    pub addr: Addr,
    pub id: u32,
}

#[cw_serde]
pub struct ContractInformationResponse {
    pub admin: String,
    pub whitelisted_collections: Vec<KompleCollection>,
    pub komple_mint_addr: Option<String>,
}
