use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::chain_parameters;

#[derive(Serialize, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = chain_parameters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChainParametersDb {
    pub id: i32,
    pub total_native_token_supply: String,
    pub total_staked_native_token: String,
    pub max_validators: i32,
    pub pos_inflation: String,
    pub pgf_steward_inflation: String,
    pub pgf_treasury_inflation: String,
    pub pgf_treasury: String,
}

pub type ChainParametersInsertDb = ChainParametersDb;
