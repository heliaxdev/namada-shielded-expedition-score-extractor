use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::player_ranks;

#[derive(Debug, Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = player_ranks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerRankDb {
    pub id: i32,
    pub ranking: i32,
    pub player_id: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = player_ranks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerRankInsertDb {
    pub ranking: i32,
    pub player_id: String,
}
