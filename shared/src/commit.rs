use super::id::Id;
use orm::commits::CommitInsertDb;
use tendermint::block::Commit as TendermintCommit;
use tendermint::block::CommitSig::{BlockIdFlagAbsent, BlockIdFlagCommit, BlockIdFlagNil};

#[derive(Debug, Clone)]
pub struct Commit {
    pub height: u64,
    pub round: u32,
    pub signatures: Vec<CommitSignature>,
}

#[derive(Debug, Clone)]
pub struct CommitSignature {
    pub kind: CommitSignatureKind,
    pub address: Id,
    pub signature: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CommitSignatureKind {
    Empty,
    Commit,
    Nil,
}

impl From<TendermintCommit> for Commit {
    fn from(value: TendermintCommit) -> Self {
        Self {
            height: value.height.value(),
            round: value.round.value(),
            signatures: value
                .signatures
                .iter()
                .filter_map(|signature| match signature {
                    BlockIdFlagAbsent => None,
                    BlockIdFlagCommit {
                        validator_address,
                        timestamp: _,
                        signature,
                    } => Some(CommitSignature {
                        kind: CommitSignatureKind::Commit,
                        address: Id::from(validator_address),
                        signature: signature.as_ref().map(|signature| {
                            let hex_bytes = subtle_encoding::hex::encode(signature.as_bytes());
                            String::from_utf8_lossy(&hex_bytes).to_string()
                        }),
                    }),
                    BlockIdFlagNil {
                        validator_address,
                        timestamp: _,
                        signature: _,
                    } => Some(CommitSignature {
                        kind: CommitSignatureKind::Nil,
                        address: Id::from(validator_address),
                        signature: None,
                    }),
                })
                .collect::<Vec<CommitSignature>>(),
        }
    }
}

impl Commit {
    pub fn to_commits_db(&self, block_id: &str) -> Vec<CommitInsertDb> {
        self.signatures
            .iter()
            .filter_map(|signature| match signature.kind {
                CommitSignatureKind::Commit => Some(CommitInsertDb {
                    signature: signature.signature.clone(),
                    address: signature.address.to_string(),
                    block_id: block_id.to_owned(),
                }),
                _ => None,
            })
            .collect::<Vec<CommitInsertDb>>()
    }
}
