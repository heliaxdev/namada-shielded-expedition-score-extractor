use diesel::{query_builder::AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::governance_proposals;

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::GovernanceKind"]
pub enum GovernanceProposalKindDb {
    PgfSteward,
    PgfFunding,
    Default,
    DefaultWithWasm,
}

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::GovernanceResult"]
pub enum GovernanceProposalResultDb {
    Passed,
    Rejected,
    Pending,
    Unknown,
    VotingPeriod,
}

#[derive(Serialize, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = governance_proposals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GovernanceProposalDb {
    pub id: i32,
    pub content: Option<String>,
    pub kind: GovernanceProposalKindDb,
    pub author: String,
    pub start_epoch: i32,
    pub end_epoch: i32,
    pub grace_epoch: i32,
    pub yay_votes: String,
    pub nay_votes: String,
    pub abstain_votes: String,
    pub result: GovernanceProposalResultDb,
    pub transaction_id: String,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = governance_proposals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GovernanceProposalInsertDb {
    pub id: i32,
    pub content: Option<String>,
    pub kind: GovernanceProposalKindDb,
    pub author: String,
    pub start_epoch: i32,
    pub end_epoch: i32,
    pub grace_epoch: i32,
    pub transaction_id: String,
}

#[derive(Serialize, AsChangeset, Clone)]
#[diesel(table_name = governance_proposals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GovernanceProposalUpdateStatusDb {
    pub yay_votes: String,
    pub nay_votes: String,
    pub abstain_votes: String,
    pub result: GovernanceProposalResultDb,
}
