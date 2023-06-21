use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::FarmProfile;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Start {},
    Stop {},

    TillGround { x: u8, y: u8 },
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
}
