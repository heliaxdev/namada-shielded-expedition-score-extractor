use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::governance_votes;

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::VoteKind"]
pub enum GovernanceVoteKindDb {
    Nay,
    Yay,
    Abstain,
}

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = governance_votes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GovernanceProposalVoteDb {
    pub id: i32,
    pub voter_address: String,
    pub kind: GovernanceVoteKindDb,
    pub proposal_id: i32,
    pub transaction_id: String,
    pub player_id: String,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = governance_votes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GovernanceProposalVoteInsertDb {
    pub voter_address: String,
    pub kind: GovernanceVoteKindDb,
    pub proposal_id: i32,
    pub transaction_id: String,
    pub player_id: String,
}
