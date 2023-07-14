use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw721::Cw721ReceiveMsg;

use crate::state::{FarmProfile, FarmProfileDto};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub komple_mint_addr: Option<String>,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Start {},
    SetupFarm {
        farm: FarmProfile,
        addr: Addr,
    },
    Stop {},
    TillGround {
        x: u8,
        y: u8,
    },
    WaterPlant {
        x: u8,
        y: u8,
    },
    Harvest {
        x: u8,
        y: u8,
    },
    UpdateContractInformation {
        contract_information: ContractInformation,
    },
    ReceiveNft(Cw721ReceiveMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    Seed { x: u8, y: u8 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ContractInformation)]
    ContractInfo {},

    // Returns a specific users farm profile from state via query
    #[returns(FarmProfileDto)]
    GetFarmProfile { address: String },

    #[returns(Vec<(u64, String)>)]
    Leaderboard {},
}

// === RESPONSES ===
#[cw_serde]
pub struct KompleCollection {
    pub addr: Addr,
    pub id: u32,
}

#[cw_serde]
pub struct ContractInformation {
    pub admin: String,
    pub komple_mint_addr: Option<String>,
}
