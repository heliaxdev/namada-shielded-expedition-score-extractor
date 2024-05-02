use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::tm_addresses;
use crate::schema::validators;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = validators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ValidatorDb {
    pub id: i32,
    pub namada_address: String,
    pub voting_power: i32,
    pub max_commission: String,
    pub commission: String,
    pub email: String,
    pub website: Option<String>,
    pub description: Option<String>,
    pub discord_handle: Option<String>,
    pub avatar: Option<String>,
    pub epoch: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = validators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ValidatorInsertDb {
    pub namada_address: String,
    pub voting_power: i32,
    pub max_commission: String,
    pub commission: String,
    pub email: String,
    pub website: Option<String>,
    pub description: Option<String>,
    pub discord_handle: Option<String>,
    pub avatar: Option<String>,
    pub epoch: i32,
}

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = tm_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TmAddressDb {
    pub id: i32,
    pub tm_address: String,
    pub epoch: i32,
    pub validator_namada_address: String,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = tm_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TmAddressInsertDb {
    pub tm_address: String,
    pub epoch: i32,
    pub validator_namada_address: String,
}
