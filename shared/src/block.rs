use std::collections::HashSet;
use std::fmt::Display;

use crate::{block_result::Event, player::PlayerId};
use chrono::DateTime;
use namada_core::borsh::BorshDeserialize;
use orm::{
    block::BlockInsertDb,
    commits::CommitInsertDb,
    evidences::{EvidenceInsertDb, EvidenceKindDb},
    governance_proposals::{GovernanceProposalInsertDb, GovernanceProposalKindDb},
    governance_votes::{GovernanceProposalVoteInsertDb, GovernanceVoteKindDb},
    players::PlayerUpdateValidatorAddressDb,
    transaction::TransactionDb,
};
use tendermint_rpc::endpoint::block::Response as TendermintBlock;

use super::{
    block_result::BlockResult, commit::Commit, evidence::EvidenceKind, header::BlockHeader, id::Id,
    transaction::Transaction,
};

pub type Epoch = u32;
pub type BlockHeight = u32;

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub hash: Id,
    pub header: BlockHeader,
    pub evidences: Vec<EvidenceKind>,
    pub commit: Option<Commit>,
    pub transactions: Vec<Transaction>,
    pub begin_events: Vec<Event>,
    pub end_events: Vec<Event>,
    pub epoch: Epoch,
}

impl From<TendermintBlock> for Block {
    fn from(value: TendermintBlock) -> Self {
        Block {
            hash: Id::from(value.block_id.hash),
            header: BlockHeader {
                height: value.block.header.height.value() as BlockHeight,
                proposer_address: value
                    .block
                    .header
                    .proposer_address
                    .to_string()
                    .to_lowercase(),
                timestamp: value.block.header.time.to_string(),
                app_hash: Id::from(value.block.header.app_hash),
            },
            evidences: value
                .block
                .evidence
                .into_vec()
                .iter()
                .map(EvidenceKind::from)
                .collect::<Vec<EvidenceKind>>(),
            commit: value.block.last_commit.map(Commit::from),
            transactions: vec![],
            begin_events: vec![],
            end_events: vec![],
            epoch: 0,
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block hash: {}, Block height: {}, Transactions {:#?}",
            self.hash,
            self.header.height,
            self.transactions
                .iter()
                .map(|tx| tx.to_string())
                .collect::<Vec<String>>()
        )
    }
}

impl From<&TendermintBlock> for Block {
    fn from(value: &TendermintBlock) -> Self {
        Block::from(value.clone())
    }
}

impl Block {
    pub fn set_transactions(&mut self, transactions: &[Transaction]) {
        self.transactions = transactions.to_owned();
    }

    pub fn set_block_events(&mut self, block_result: &BlockResult) {
        self.begin_events = block_result.begin_events.to_owned();
        self.end_events = block_result.end_events.to_owned();
    }

    pub fn set_epoch(&mut self, epoch: Epoch) {
        self.epoch = epoch;
    }

    pub fn to_block_db(&self) -> BlockInsertDb {
        let timestamp = DateTime::parse_from_rfc3339(&self.header.timestamp)
            .unwrap()
            .naive_utc();
        BlockInsertDb {
            id: self.hash.to_string(),
            height: self.header.height as i32,
            included_at: timestamp,
            proposer_address: self.header.proposer_address.to_string(),
            epoch: self.epoch as i32,
        }
    }

    pub fn to_transactions_db(&self) -> Vec<TransactionDb> {
        self.transactions
            .iter()
            .filter_map(|transaction| transaction.to_transaction_db(&self.hash.to_string()))
            .collect()
    }

    pub fn to_commits_db(&self) -> Vec<CommitInsertDb> {
        self.commit
            .as_ref()
            .map(|commit| commit.to_commits_db(&self.hash.to_string()))
            .unwrap_or_default()
    }

    pub fn get_proposal_ids_by_proposal_vote(&self) -> HashSet<u64> {
        use namada_governance::VoteProposalData;

        self.transactions
            .iter()
            .filter_map(|transaction| match &transaction.kind {
                crate::transaction::TransactionKind::ProposalVote(data) => {
                    let data = VoteProposalData::try_from_slice(data).unwrap();
                    Some(data.id)
                }
                _ => None,
            })
            .collect()
    }

    pub fn to_proposal_vote_db(
        &self,
        proposals_id: Vec<i32>,
    ) -> Vec<GovernanceProposalVoteInsertDb> {
        use namada_governance::{ProposalVote, VoteProposalData};

        let mut duplicates = HashSet::new();

        self.transactions
            .iter()
            .filter_map(|transaction| match &transaction.kind {
                crate::transaction::TransactionKind::ProposalVote(data) => {
                    let data = VoteProposalData::try_from_slice(data).unwrap();
                    let PlayerId(player_id) = PlayerId::try_from(transaction.memo.clone()?).ok()?;

                    let key = format!("{}-{}", data.voter, data.id);
                    if duplicates.contains(&key) {
                        return None;
                    } else {
                        duplicates.insert(key);
                    }

                    if !proposals_id.contains(&(data.id as i32)) {
                        return None;
                    };

                    Some(GovernanceProposalVoteInsertDb {
                        voter_address: data.voter.to_string(),
                        kind: match data.vote {
                            ProposalVote::Yay => GovernanceVoteKindDb::Yay,
                            ProposalVote::Nay => GovernanceVoteKindDb::Nay,
                            ProposalVote::Abstain => GovernanceVoteKindDb::Abstain,
                        },
                        proposal_id: data.id as i32,
                        transaction_id: transaction.hash.to_string(),
                        player_id,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn to_governance_proposal_db(
        &self,
        mut next_proposal_id: u64,
    ) -> Vec<GovernanceProposalInsertDb> {
        use namada_governance::storage::proposal::{InitProposalData, ProposalType};

        self.transactions
            .iter()
            .filter(|transaction| transaction.ok())
            .filter_map(|transaction| match &transaction.kind {
                crate::transaction::TransactionKind::InitProposal(data) => {
                    let data = InitProposalData::try_from_slice(data).unwrap();
                    let current_id = next_proposal_id;
                    next_proposal_id += 1;

                    Some(GovernanceProposalInsertDb {
                        id: current_id as i32,
                        content: Some(data.content.to_string()),
                        kind: match data.r#type {
                            ProposalType::Default(wasm) => {
                                if wasm.is_some() {
                                    GovernanceProposalKindDb::DefaultWithWasm
                                } else {
                                    GovernanceProposalKindDb::Default
                                }
                            }
                            ProposalType::PGFSteward(_) => GovernanceProposalKindDb::PgfSteward,
                            ProposalType::PGFPayment(_) => GovernanceProposalKindDb::PgfFunding,
                        },
                        author: data.author.to_string(),
                        start_epoch: data.voting_start_epoch.0 as i32,
                        end_epoch: data.voting_end_epoch.0 as i32,
                        grace_epoch: data.grace_epoch.0 as i32,
                        transaction_id: transaction.hash.to_string(),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn to_evidence_db(&self) -> Vec<EvidenceInsertDb> {
        self.evidences
            .iter()
            .map(|evidence| match evidence {
                EvidenceKind::DuplicateVote(address) => EvidenceInsertDb {
                    kind: EvidenceKindDb::DuplicateVote,
                    validator_address: address.to_string(),
                    block_id: self.hash.to_string(),
                },
                EvidenceKind::LightClientAttack(address) => EvidenceInsertDb {
                    kind: EvidenceKindDb::LightClientAttack,
                    validator_address: address.to_string(),
                    block_id: self.hash.to_string(),
                },
            })
            .collect()
    }

    pub fn to_validator_address_db(&self) -> Vec<(String, PlayerUpdateValidatorAddressDb)> {
        use namada_tx::data::pos::BecomeValidator;

        self.transactions
            .iter()
            .filter_map(|transaction| match &transaction.kind {
                crate::transaction::TransactionKind::BecomeValidator(bytes) => {
                    let memo = PlayerId::try_from(transaction.memo.clone()?).ok()?;
                    let data = BecomeValidator::try_from_slice(bytes).unwrap();
                    let update = PlayerUpdateValidatorAddressDb {
                        namada_validator_address: Some(data.address.to_string()),
                    };
                    Some((memo.0, update))
                }
                _ => None,
            })
            .collect()
    }
}
