use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::evidences;

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::EvidenceKind"]
pub enum EvidenceKindDb {
    DuplicateVote,
    LightClientAttack,
}

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = evidences)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EvidenceDb {
    pub id: i32,
    pub kind: EvidenceKindDb,
    pub validator_address: String,
    pub block_id: String,
}

#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name = evidences)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EvidenceInsertDb {
    pub kind: EvidenceKindDb,
    pub validator_address: String,
    pub block_id: String,
}
