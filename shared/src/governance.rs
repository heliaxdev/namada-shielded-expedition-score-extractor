use orm::{
    governance_proposals::{GovernanceProposalKindDb, GovernanceProposalResultDb},
    governance_votes::GovernanceVoteKindDb,
};
use serde::{Deserialize, Serialize};

use crate::{block::Epoch, id::Id};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProposalResult {
    Passed,
    Rejected,
    Pending,
    Unknown,
    VotingPeriod,
}

impl From<&GovernanceProposalResultDb> for ProposalResult {
    fn from(value: &GovernanceProposalResultDb) -> Self {
        match value {
            GovernanceProposalResultDb::Passed => ProposalResult::Passed,
            GovernanceProposalResultDb::Rejected => ProposalResult::Rejected,
            GovernanceProposalResultDb::Pending => ProposalResult::Pending,
            GovernanceProposalResultDb::Unknown => ProposalResult::Unknown,
            GovernanceProposalResultDb::VotingPeriod => ProposalResult::VotingPeriod,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProposalKind {
    PgfSteward,
    PgfFunding,
    Default,
    DefaultWithWasm,
}

impl From<&GovernanceProposalKindDb> for ProposalKind {
    fn from(value: &GovernanceProposalKindDb) -> Self {
        match value {
            GovernanceProposalKindDb::PgfSteward => ProposalKind::PgfSteward,
            GovernanceProposalKindDb::PgfFunding => ProposalKind::PgfFunding,
            GovernanceProposalKindDb::Default => ProposalKind::Default,
            GovernanceProposalKindDb::DefaultWithWasm => ProposalKind::DefaultWithWasm,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProposalVoteKind {
    Nay,
    Yay,
    Abstain,
}

impl From<&GovernanceVoteKindDb> for ProposalVoteKind {
    fn from(value: &GovernanceVoteKindDb) -> Self {
        match value {
            GovernanceVoteKindDb::Nay => ProposalVoteKind::Nay,
            GovernanceVoteKindDb::Yay => ProposalVoteKind::Yay,
            GovernanceVoteKindDb::Abstain => ProposalVoteKind::Abstain,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Proposal {
    pub id: u64,
    pub content: Option<String>,
    pub kind: ProposalKind,
    pub author: Id,
    pub start_epoch: Epoch,
    pub end_epoch: Epoch,
    pub grace_epoch: Epoch,
    pub result: ProposalResult,
    pub yay_votes: String,
    pub nay_votes: String,
    pub abstain_votes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProposalVote {
    pub id: u64,
    pub kind: ProposalVoteKind,
    pub proposal_id: u64,
    pub voter_address: Id,
    pub player_id: Id,
}
