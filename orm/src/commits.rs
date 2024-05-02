use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::commits;

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = commits)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CommitDb {
    pub id: i32,
    pub signature: Option<String>,
    pub address: String,
    pub block_id: String,
}

#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name = commits)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CommitInsertDb {
    pub signature: Option<String>,
    pub address: String,
    pub block_id: String,
}
