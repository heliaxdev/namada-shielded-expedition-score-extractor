use orm::chain_params::{ChainParametersDb, ChainParametersInsertDb};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainParameters {
    pub epoch: u32,
    pub total_native_token_supply: String,
    pub total_staked_native_token: String,
    pub max_validators: u32,
    pub pos_inflation: String,
    pub pgf_treasury_inflation: String,
    pub pgf_steward_inflation: String,
    pub pgf_treasury: String,
}

impl From<&ChainParametersDb> for ChainParameters {
    fn from(value: &ChainParametersDb) -> Self {
        Self {
            epoch: value.id as u32,
            total_native_token_supply: value.total_native_token_supply.clone(),
            total_staked_native_token: value.total_staked_native_token.clone(),
            max_validators: value.max_validators as u32,
            pos_inflation: value.pos_inflation.clone(),
            pgf_treasury_inflation: value.pgf_treasury_inflation.clone(),
            pgf_steward_inflation: value.pgf_steward_inflation.clone(),
            pgf_treasury: value.pgf_treasury.clone(),
        }
    }
}

impl ChainParameters {
    pub fn to_chain_parameters_db(&self) -> ChainParametersInsertDb {
        ChainParametersInsertDb {
            id: self.epoch as i32,
            total_native_token_supply: self.total_native_token_supply.clone(),
            total_staked_native_token: self.total_staked_native_token.clone(),
            max_validators: self.max_validators as i32,
            pos_inflation: self.pos_inflation.clone(),
            pgf_steward_inflation: self.pgf_steward_inflation.clone(),
            pgf_treasury_inflation: self.pgf_treasury_inflation.clone(),
            pgf_treasury: self.pgf_treasury.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PosParameters {
    pub pos_inflation: String,
    pub max_validator: u32,
    pub epoch: u32,
}

#[derive(Debug, Clone)]
pub struct PgfParameters {
    pub pgf_treasury_inflation: String,
    pub pgf_steward_inflation: String,
    pub epoch: u32,
}
