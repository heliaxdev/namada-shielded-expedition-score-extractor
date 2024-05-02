use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::stewards;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = stewards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StewardDb {
    pub id: i32,
    pub namada_address: String,
    pub block_height: i32,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = stewards)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StewardInsertDb {
    pub namada_address: String,
    pub block_height: i32,
}
