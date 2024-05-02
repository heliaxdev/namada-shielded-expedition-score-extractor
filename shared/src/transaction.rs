use std::fmt::Display;

use namada_core::borsh::BorshDeserialize;
use namada_tx::{data::TxType, Tx as NamadaTx};
use orm::transaction::{TransactionDb, TransactionExitStatusDb, TransactionKindDb};

use crate::{
    block_result::{BlockResult, TxAttributes, TxEventStatusCode},
    checksums::Checksums,
};

use super::id::Id;

#[derive(Debug, Clone)]
pub enum TransactionKind {
    Wrapper,
    Protocol,
    TransparentTransfer(Vec<u8>),
    ShieldedTransfer(Vec<u8>),
    Bond(Vec<u8>),
    Redelegation(Vec<u8>),
    Unbond(Vec<u8>),
    Withdraw(Vec<u8>),
    ClaimRewards(Vec<u8>),
    ReactivateValidator(Vec<u8>),
    DeactivateValidator(Vec<u8>),
    IbcEnvelop(Vec<u8>),
    IbcTransparentTransfer(Vec<u8>),
    IbcShieldedTransfer(Vec<u8>),
    ChangeConsensusKey(Vec<u8>),
    ChangeCommission(Vec<u8>),
    ChangeMetadata(Vec<u8>),
    BecomeValidator(Vec<u8>),
    InitAccount(Vec<u8>),
    InitProposal(Vec<u8>),
    ResignSteward(Vec<u8>),
    RevealPublicKey(Vec<u8>),
    UnjailValidator(Vec<u8>),
    UpdateAccount(Vec<u8>),
    UpdateStewardCommissions(Vec<u8>),
    ProposalVote(Vec<u8>),
    Unknown,
}

impl Display for TransactionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wrapper => write!(f, "Wrapper"),
            Self::Protocol => write!(f, "Protocol"),
            Self::TransparentTransfer(_) => write!(f, "Transfer"),
            Self::ShieldedTransfer(_) => write!(f, "Transfer"),
            Self::Bond(_) => write!(f, "Bond"),
            Self::Redelegation(_) => write!(f, "Redelegate"),
            Self::Unbond(_) => write!(f, "Unbond"),
            Self::Withdraw(_) => write!(f, "Withdraw"),
            Self::ClaimRewards(_) => write!(f, "ClaimRewards"),
            Self::ReactivateValidator(_) => write!(f, "ReactivateValidator"),
            Self::DeactivateValidator(_) => write!(f, "DeactivateValidator"),
            Self::ChangeConsensusKey(_) => write!(f, "ChangeConsensusKey"),
            Self::ChangeMetadata(_) => write!(f, "ChangeMetadata"),
            Self::ChangeCommission(_) => write!(f, "ChangeCommission"),
            Self::BecomeValidator(_) => write!(f, "BecomeValidator"),
            Self::IbcEnvelop(_) => write!(f, "IbcEnvelop"),
            Self::IbcTransparentTransfer(_) => write!(f, "IbcTransparentTransfer"),
            Self::IbcShieldedTransfer(_) => write!(f, "IbcShieldedTransfer"),
            Self::InitAccount(_) => write!(f, "InitAccount"),
            Self::InitProposal(_) => write!(f, "InitProposal"),
            Self::ResignSteward(_) => write!(f, "ResignSteward"),
            Self::RevealPublicKey(_) => write!(f, "RevealPublicKey"),
            Self::UnjailValidator(_) => write!(f, "UnjailValidator"),
            Self::UpdateAccount(_) => write!(f, "UpdateAccount"),
            Self::UpdateStewardCommissions(_) => write!(f, "UpdateStewardCommissionswrite"),
            Self::ProposalVote(_) => write!(f, "ProposalVote"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl TransactionKind {
    // pub fn from(tx_kind_name: &str, data: &[u8]) -> Self {
    //     match tx_kind_name {
    //         "tx_transfer" => TransactionKind::Transfer(
    //             namada_sdk::core::types::token::Transfer::try_from_slice(data).unwrap(),
    //         ),
    //         "tx_bond" => TransactionKind::Bond(
    //             namada_sdk::core::types::transaction::pos::Bond::try_from_slice(data).unwrap(),
    //         ),
    //         "tx_redelegation" => TransactionKind::Redelegation(
    //             namada_sdk::core::types::transaction::pos::Redelegation::try_from_slice(data)
    //                 .unwrap(),
    //         ),
    //         "tx_unbond" => TransactionKind::Unbond(
    //             namada_sdk::core::types::transaction::pos::Unbond::try_from_slice(data).unwrap(),
    //         ),
    //         "tx_withdraw" => TransactionKind::Withdraw(
    //             namada_sdk::core::types::transaction::pos::Withdraw::try_from_slice(data).unwrap(),
    //         ),
    //         "tx_claim_rewards" => TransactionKind::ClaimRewards(
    //             namada_sdk::core::types::transaction::pos::ClaimRewards::try_from_slice(data)
    //                 .unwrap(),
    //         ),
    //         "tx_reactivate_validator" => TransactionKind::ReactivateValidator(
    //             namada_sdk::core::types::address::Address::try_from_slice(data).unwrap(),
    //         ),
    //         "tx_deactivate_validator" => TransactionKind::DeactivateValidator(
    //             namada_sdk::core::types::address::Address::try_from_slice(data).unwrap(),
    //         ),
    //         "tx_ibc" => {
    //             let decoded_ibc_tx = namada_sdk::core::ledger::ibc::decode_message(data).unwrap();
    //             TransactionKind::Ibc(decoded_ibc_tx)
    //         }
    //         "tx_change_consensus_key" => TransactionKind::ChangeConsensusKey(
    //             namada_sdk::core::types::transaction::pos::ConsensusKeyChange::try_from_slice(data)
    //                 .unwrap(),
    //         ),
    //         "tx_change_validator_metadata" => TransactionKind::ChangeMetadata(
    //             namada_sdk::core::types::transaction::pos::MetaDataChange::try_from_slice(data)
    //                 .unwrap(),
    //         ),
    //         "tx_change_validator_commission" => TransactionKind::ChangeCommission(
    //             namada_sdk::core::types::transaction::pos::CommissionChange::try_from_slice(data)
    //                 .unwrap(),
    //         ),
    //         "tx_become_validator" => TransactionKind::BecomeValidator(
    //             namada_sdk::core::types::transaction::pos::BecomeValidator::try_from_slice(data)
    //                 .unwrap(),
    //         ),
    //         "tx_init_account" => TransactionKind::InitAccount(
    //             namada_sdk::core::types::transaction::account::InitAccount::try_from_slice(data)
    //                 .unwrap(),
    //         ),
    //         "tx_init_proposal" => TransactionKind::InitProposal(
    //             namada_sdk::core::types::transaction::governance::InitProposalData::try_from_slice(
    //                 data,
    //             )
    //             .unwrap(),
    //         ),
    //         "tx_resign_steward" => TransactionKind::ResignSteward(
    //             namada_sdk::core::types::address::Address::try_from_slice(data).unwrap(),
    //         ),
    //         "tx_reveal_pk" => TransactionKind::RevealPublicKey(
    //             namada_sdk::core::types::key::common::PublicKey::try_from_slice(data).unwrap(),
    //         ),
    //         "tx_unjail_validator" => TransactionKind::UnjailValidator(
    //             namada_sdk::core::types::address::Address::try_from_slice(data).unwrap(),
    //         ),
    //         "tx_update_account" => TransactionKind::UpdateAccount(
    //             namada_sdk::core::types::transaction::account::UpdateAccount::try_from_slice(data)
    //                 .unwrap(),
    //         ),
    //         "tx_update_steward_commission" => TransactionKind::UpdateStewardCommissions(
    //             namada_sdk::core::types::transaction::pgf::UpdateStewardCommission::try_from_slice(data)
    //                 .unwrap(),
    //         ),
    //         "tx_vote_proposal" => TransactionKind::ProposalVote(
    //             namada_sdk::core::types::transaction::governance::VoteProposalData::try_from_slice(data).unwrap(),
    //         ),
    //         _ => TransactionKind::Unknown,
    //     }
    // }

    pub fn from(tx_kind_name: &str, data: &[u8]) -> Self {
        match tx_kind_name {
            "tx_transfer" => {
                let transfer_data =
                    if let Ok(tx) = namada_core::types::token::Transfer::try_from_slice(data) {
                        tx
                    } else {
                        return TransactionKind::Unknown;
                    };

                match transfer_data.shielded {
                    Some(_) => TransactionKind::ShieldedTransfer(data.to_vec()),
                    None => TransactionKind::TransparentTransfer(data.to_vec()),
                }
            }
            "tx_bond" => TransactionKind::Bond(data.to_vec()),
            "tx_redelegation" => TransactionKind::Redelegation(data.to_vec()),
            "tx_unbond" => TransactionKind::Unbond(data.to_vec()),
            "tx_withdraw" => TransactionKind::Withdraw(data.to_vec()),
            "tx_claim_rewards" => TransactionKind::ClaimRewards(data.to_vec()),
            "tx_reactivate_validator" => TransactionKind::ReactivateValidator(data.to_vec()),
            "tx_deactivate_validator" => TransactionKind::DeactivateValidator(data.to_vec()),
            "tx_ibc" => {
                let decoded_ibc_tx = if let Ok(tx) = namada_ibc::decode_message(data) {
                    tx
                } else {
                    return TransactionKind::Unknown;
                };

                match decoded_ibc_tx {
                    namada_ibc::IbcMessage::Envelope(_) => {
                        TransactionKind::IbcEnvelop(data.to_vec())
                    }
                    namada_ibc::IbcMessage::Transfer(_) => {
                        TransactionKind::IbcTransparentTransfer(data.to_vec())
                    }
                    namada_ibc::IbcMessage::ShieldedTransfer(_) => {
                        TransactionKind::IbcShieldedTransfer(data.to_vec())
                    }
                }
            }
            "tx_change_consensus_key" => TransactionKind::ChangeConsensusKey(data.to_vec()),
            "tx_change_validator_metadata" => TransactionKind::ChangeMetadata(data.to_vec()),
            "tx_change_validator_commission" => TransactionKind::ChangeCommission(data.to_vec()),
            "tx_become_validator" => TransactionKind::BecomeValidator(data.to_vec()),
            "tx_init_account" => TransactionKind::InitAccount(data.to_vec()),
            "tx_init_proposal" => TransactionKind::InitProposal(data.to_vec()),
            "tx_resign_steward" => TransactionKind::ResignSteward(data.to_vec()),
            "tx_reveal_pk" => TransactionKind::RevealPublicKey(data.to_vec()),
            "tx_unjail_validator" => TransactionKind::UnjailValidator(data.to_vec()),
            "tx_update_account" => TransactionKind::UpdateAccount(data.to_vec()),
            "tx_update_steward_commission" => {
                TransactionKind::UpdateStewardCommissions(data.to_vec())
            }
            "tx_vote_proposal" => TransactionKind::ProposalVote(data.to_vec()),
            _ => TransactionKind::Unknown,
        }
    }

    pub fn get_bytes(&self) -> Option<&[u8]> {
        match self {
            TransactionKind::Wrapper => None,
            TransactionKind::Protocol => None,
            TransactionKind::TransparentTransfer(bytes) => Some(bytes),
            TransactionKind::ShieldedTransfer(bytes) => Some(bytes),
            TransactionKind::Bond(bytes) => Some(bytes),
            TransactionKind::Redelegation(bytes) => Some(bytes),
            TransactionKind::Unbond(bytes) => Some(bytes),
            TransactionKind::Withdraw(bytes) => Some(bytes),
            TransactionKind::ClaimRewards(bytes) => Some(bytes),
            TransactionKind::ReactivateValidator(bytes) => Some(bytes),
            TransactionKind::DeactivateValidator(bytes) => Some(bytes),
            TransactionKind::IbcEnvelop(bytes) => Some(bytes),
            TransactionKind::IbcTransparentTransfer(bytes) => Some(bytes),
            TransactionKind::IbcShieldedTransfer(bytes) => Some(bytes),
            TransactionKind::ChangeConsensusKey(bytes) => Some(bytes),
            TransactionKind::ChangeCommission(bytes) => Some(bytes),
            TransactionKind::ChangeMetadata(bytes) => Some(bytes),
            TransactionKind::BecomeValidator(bytes) => Some(bytes),
            TransactionKind::InitAccount(bytes) => Some(bytes),
            TransactionKind::InitProposal(bytes) => Some(bytes),
            TransactionKind::ResignSteward(bytes) => Some(bytes),
            TransactionKind::RevealPublicKey(bytes) => Some(bytes),
            TransactionKind::UnjailValidator(bytes) => Some(bytes),
            TransactionKind::UpdateAccount(bytes) => Some(bytes),
            TransactionKind::UpdateStewardCommissions(bytes) => Some(bytes),
            TransactionKind::ProposalVote(bytes) => Some(bytes),
            TransactionKind::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionExitStatus {
    Accepted,
    Applied,
    Rejected,
}

impl Display for TransactionExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accepted => write!(f, "Accepted"),
            Self::Applied => write!(f, "Applied"),
            Self::Rejected => write!(f, "Rejected"),
        }
    }
}

impl TransactionExitStatus {
    pub fn from(tx_attributes: &TxAttributes, tx_kind: &TransactionKind) -> Self {
        match (tx_kind, tx_attributes.code) {
            (TransactionKind::Wrapper, TxEventStatusCode::Ok) => TransactionExitStatus::Accepted,
            (_, TxEventStatusCode::Ok) => TransactionExitStatus::Applied,
            (_, TxEventStatusCode::Fail) => TransactionExitStatus::Rejected,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RawMemo(pub Vec<u8>);

#[derive(Debug, Clone)]
pub struct Transaction<MEMO = RawMemo> {
    pub hash: Id,
    pub inner_hash: Option<Id>,
    pub kind: TransactionKind,
    pub status: TransactionExitStatus,
    pub memo: Option<MEMO>,
    pub gas_used: u64,
    pub index: usize,
}

impl<M> Transaction<M> {
    pub fn try_parse_memo<N>(self) -> anyhow::Result<Transaction<N>>
    where
        N: TryFrom<M, Error = anyhow::Error>,
    {
        let Self {
            hash,
            inner_hash,
            kind,
            status,
            memo,
            gas_used,
            index,
        } = self;
        Ok(Transaction {
            hash,
            inner_hash,
            kind,
            status,
            gas_used,
            index,
            memo: memo.map(|memo| memo.try_into()).transpose()?,
        })
    }
}

impl From<TransactionDb> for Transaction<RawMemo> {
    fn from(db_tx: TransactionDb) -> Self {
        let data = db_tx.associated_data.unwrap_or_default();
        let kind = match db_tx.kind {
            TransactionKindDb::Wrapper => TransactionKind::Wrapper,
            TransactionKindDb::Protocol => TransactionKind::Protocol,
            TransactionKindDb::TransparentTransfer => TransactionKind::TransparentTransfer(data),
            TransactionKindDb::ShieldedTransfer => TransactionKind::ShieldedTransfer(data),
            TransactionKindDb::Bond => TransactionKind::Bond(data),
            TransactionKindDb::Redelegation => TransactionKind::Redelegation(data),
            TransactionKindDb::Unbond => TransactionKind::Unbond(data),
            TransactionKindDb::Withdraw => TransactionKind::Withdraw(data),
            TransactionKindDb::ClaimRewards => TransactionKind::ClaimRewards(data),
            TransactionKindDb::ReactivateValidator => TransactionKind::ReactivateValidator(data),
            TransactionKindDb::DeactivateValidator => TransactionKind::DeactivateValidator(data),
            TransactionKindDb::IbcEnvelop => TransactionKind::IbcEnvelop(data),
            TransactionKindDb::IbcTransparentTransfer => {
                TransactionKind::IbcTransparentTransfer(data)
            }
            TransactionKindDb::IbcShieldedTransfer => TransactionKind::IbcShieldedTransfer(data),
            TransactionKindDb::ChangeConsensusKey => TransactionKind::ChangeConsensusKey(data),
            TransactionKindDb::ChangeCommission => TransactionKind::ChangeCommission(data),
            TransactionKindDb::ChangeMetadata => TransactionKind::ChangeMetadata(data),
            TransactionKindDb::BecomeValidator => TransactionKind::BecomeValidator(data),
            TransactionKindDb::InitAccount => TransactionKind::InitAccount(data),
            TransactionKindDb::InitProposal => TransactionKind::InitProposal(data),
            TransactionKindDb::ResignSteward => TransactionKind::ResignSteward(data),
            TransactionKindDb::RevealPublicKey => TransactionKind::RevealPublicKey(data),
            TransactionKindDb::UnjailValidator => TransactionKind::UnjailValidator(data),
            TransactionKindDb::UpdateAccount => TransactionKind::UpdateAccount(data),
            TransactionKindDb::UpdateStewardCommissions => {
                TransactionKind::UpdateStewardCommissions(data)
            }
            TransactionKindDb::ProposalVote => TransactionKind::ProposalVote(data),
            TransactionKindDb::Unknown => TransactionKind::Unknown,
        };
        Self {
            kind,
            hash: Id::Hash(db_tx.id),
            inner_hash: db_tx.inner_hash.map(Id::Hash),
            status: match db_tx.exit_status {
                TransactionExitStatusDb::Accepted => TransactionExitStatus::Accepted,
                TransactionExitStatusDb::Applied => TransactionExitStatus::Applied,
                TransactionExitStatusDb::Rejected => TransactionExitStatus::Rejected,
            },
            memo: db_tx.memo.map(RawMemo),
            gas_used: db_tx.gas_used as _,
            index: db_tx.index as _,
        }
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.kind, self.status)
    }
}

impl Transaction {
    pub fn deserialize(
        raw_tx_bytes: &[u8],
        tx_code_map: &Checksums,
        block_results: &BlockResult,
        index: usize,
    ) -> Result<(Self, String), String> {
        let transaction = NamadaTx::try_from(raw_tx_bytes).map_err(|e| e.to_string())?;
        let memo = transaction.memo().map(RawMemo);
        match transaction.header().tx_type {
            TxType::Wrapper(_) => {
                let tx_id = Id::from(transaction.header_hash());
                let raw_hash = Id::from(transaction.raw_header_hash());
                let raw_hash_str = raw_hash.to_string();
                let tx_status = block_results.find_tx_hash_result(&tx_id).unwrap();

                let tx_exit = TransactionExitStatus::from(&tx_status, &TransactionKind::Wrapper);
                if tx_exit == TransactionExitStatus::Rejected {
                    return Err("Transaction was rejected".to_string());
                };

                let transaction = Transaction {
                    hash: tx_id,
                    inner_hash: Some(raw_hash),
                    kind: TransactionKind::Wrapper,
                    status: tx_exit,
                    gas_used: tx_status.gas,
                    memo,
                    index,
                };
                Ok((transaction, raw_hash_str))
            }
            TxType::Decrypted(_) => {
                let tx_id = Id::from(transaction.raw_header_hash());
                let raw_hash = tx_id.to_string();
                let tx_status = block_results.find_tx_hash_result(&tx_id).unwrap();

                let tx_code_id = transaction
                    .get_section(transaction.code_sechash())
                    .and_then(|s| s.code_sec())
                    .map(|s| s.code.hash().0)
                    .map(|bytes| String::from_utf8(subtle_encoding::hex::encode(bytes)).unwrap());

                let tx_data = transaction.data().unwrap_or_default();

                let tx_kind = if let Some(id) = tx_code_id {
                    if let Some(tx_kind_name) = tx_code_map.get_name_by_id(&id) {
                        TransactionKind::from(&tx_kind_name, &tx_data)
                    } else {
                        TransactionKind::Unknown
                    }
                } else {
                    TransactionKind::Unknown
                };

                let tx_exit = TransactionExitStatus::from(&tx_status, &tx_kind);
                if tx_exit == TransactionExitStatus::Rejected {
                    return Err("Transaction was rejected".to_string());
                };

                let transaction = Transaction {
                    hash: tx_id,
                    inner_hash: None,
                    kind: tx_kind.clone(),
                    status: tx_exit,
                    gas_used: tx_status.gas,
                    memo,
                    index,
                };

                Ok((transaction, raw_hash))
            }
            TxType::Raw => Err("Raw transaction are not supported.".to_string()),
            TxType::Protocol(_) => Err("Protocol transaction are not supported.".to_string()),
        }
    }

    pub fn ok(&self) -> bool {
        match self.status {
            TransactionExitStatus::Accepted => true,
            TransactionExitStatus::Applied => true,
            TransactionExitStatus::Rejected => false,
        }
    }

    pub fn to_transaction_db(&self, block_id: &str) -> Option<TransactionDb> {
        match self.kind {
            TransactionKind::Unknown => None,
            _ => {
                let tx = TransactionDb {
                    id: self.hash.to_string(),
                    inner_hash: self.inner_hash.as_ref().map(|hash| hash.to_string()),
                    index: self.index as i32,
                    kind: TransactionKindDb::from(&self.kind),
                    associated_data: self.kind.get_bytes().map(Vec::from),
                    exit_status: TransactionExitStatusDb::from(&self.status),
                    gas_used: self.gas_used as i32,
                    block_id: block_id.to_owned(),
                    memo: self.memo.clone().map(|RawMemo(raw)| raw),
                };
                Some(tx)
            }
        }
    }
}

impl From<&TransactionKind> for TransactionKindDb {
    fn from(value: &TransactionKind) -> Self {
        match value {
            TransactionKind::Wrapper => TransactionKindDb::Wrapper,
            TransactionKind::Protocol => TransactionKindDb::Protocol,
            TransactionKind::TransparentTransfer(_) => TransactionKindDb::TransparentTransfer,
            TransactionKind::ShieldedTransfer(_) => TransactionKindDb::ShieldedTransfer,
            TransactionKind::Bond(_) => TransactionKindDb::Bond,
            TransactionKind::Redelegation(_) => TransactionKindDb::Redelegation,
            TransactionKind::Unbond(_) => TransactionKindDb::Unbond,
            TransactionKind::Withdraw(_) => TransactionKindDb::Withdraw,
            TransactionKind::ClaimRewards(_) => TransactionKindDb::ClaimRewards,
            TransactionKind::ReactivateValidator(_) => TransactionKindDb::ReactivateValidator,
            TransactionKind::DeactivateValidator(_) => TransactionKindDb::DeactivateValidator,
            TransactionKind::IbcEnvelop(_) => TransactionKindDb::IbcEnvelop,
            TransactionKind::IbcTransparentTransfer(_) => TransactionKindDb::IbcShieldedTransfer,
            TransactionKind::IbcShieldedTransfer(_) => TransactionKindDb::IbcTransparentTransfer,
            TransactionKind::ChangeConsensusKey(_) => TransactionKindDb::ChangeConsensusKey,
            TransactionKind::ChangeCommission(_) => TransactionKindDb::ChangeCommission,
            TransactionKind::ChangeMetadata(_) => TransactionKindDb::ChangeMetadata,
            TransactionKind::BecomeValidator(_) => TransactionKindDb::BecomeValidator,
            TransactionKind::InitAccount(_) => TransactionKindDb::InitAccount,
            TransactionKind::InitProposal(_) => TransactionKindDb::InitProposal,
            TransactionKind::ResignSteward(_) => TransactionKindDb::ResignSteward,
            TransactionKind::RevealPublicKey(_) => TransactionKindDb::RevealPublicKey,
            TransactionKind::UnjailValidator(_) => TransactionKindDb::UnjailValidator,
            TransactionKind::UpdateAccount(_) => TransactionKindDb::UpdateAccount,
            TransactionKind::UpdateStewardCommissions(_) => {
                TransactionKindDb::UpdateStewardCommissions
            }
            TransactionKind::ProposalVote(_) => TransactionKindDb::ProposalVote,
            TransactionKind::Unknown => TransactionKindDb::Unknown,
        }
    }
}

impl From<&TransactionExitStatus> for TransactionExitStatusDb {
    fn from(value: &TransactionExitStatus) -> Self {
        match value {
            TransactionExitStatus::Accepted => TransactionExitStatusDb::Accepted,
            TransactionExitStatus::Applied => TransactionExitStatusDb::Applied,
            TransactionExitStatus::Rejected => TransactionExitStatusDb::Rejected,
        }
    }
}
