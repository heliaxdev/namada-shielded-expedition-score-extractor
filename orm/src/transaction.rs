use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::transactions;

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, diesel_derive_enum::DbEnum,
)]
#[ExistingTypePath = "crate::schema::sql_types::TxKind"]
pub enum TransactionKindDb {
    Wrapper,
    Protocol,
    TransparentTransfer,
    ShieldedTransfer,
    Bond,
    Redelegation,
    Unbond,
    Withdraw,
    ClaimRewards,
    ReactivateValidator,
    DeactivateValidator,
    IbcEnvelop,
    IbcTransparentTransfer,
    IbcShieldedTransfer,
    ChangeConsensusKey,
    ChangeCommission,
    ChangeMetadata,
    BecomeValidator,
    InitAccount,
    InitProposal,
    ResignSteward,
    RevealPublicKey,
    UnjailValidator,
    UpdateAccount,
    UpdateStewardCommissions,
    ProposalVote,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::TxExitStatus"]
pub enum TransactionExitStatusDb {
    Applied,
    Accepted,
    Rejected,
}

#[derive(Serialize, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionDb {
    pub id: String,
    pub inner_hash: Option<String>,
    pub index: i32,
    pub kind: TransactionKindDb,
    pub associated_data: Option<Vec<u8>>,
    pub exit_status: TransactionExitStatusDb,
    pub gas_used: i32,
    pub block_id: String,
    pub memo: Option<Vec<u8>>,
}

pub type TransactionInsertDb = TransactionDb;
