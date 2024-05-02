use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::blocks;

#[derive(Serialize, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BlockDb {
    pub id: String,
    pub height: i32,
    pub included_at: NaiveDateTime,
    pub proposer_address: String,
    pub epoch: i32,
}

pub type BlockInsertDb = BlockDb;
