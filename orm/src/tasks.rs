use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::transaction::TransactionKindDb;
use crate::schema::{tasks, unidentified_tasks};

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, diesel_derive_enum::DbEnum,
)]
#[ExistingTypePath = "crate::schema::sql_types::TaskType"]
pub enum TaskTypeDb {
    // crew tasks (tx)
    DelegateStakeOnV0,
    DelegateStakeOnV1,
    ClaimPosRewards,
    ShieldNaan,
    UnshieldNaan,
    ShieldToShielded,
    ShieldAssetOverIbc,
    // pilot tasks (genesis)
    SubmitPreGenesisBondTx,
    // pilot tasks (tx)
    VotePgfStewardProposal,
    VoteUpgradeV0ToV1,
    VoteUpgradeV1ToV2,
    InitPostGenesisValidator,
    // pilot tasks (completable, non-tx)
    StartNode5MinFromGenesis,
    InValidatorSetFor1Epoch,
    SignFirstBlockOfUpgradeToV2,
    // pilot tasks (ongoing, non-tx)
    Keep99PerCentUptime,
    Keep95PerCentUptime,
    Keep99PerCentGovParticipationRate,
    Keep90PerCentGovParticipationRate,
    // manual tasks (either pilot or crew)
    ProvidePublicRpcEndpoint,
    OperateNamadaIndexer,
    OperateNamadaInterface,
    OperateCosmosTestnetRelayer,
    OperateOsmosisTestnetRelayer,
    OperateNobleTestnetRelayer,
    OperateRelayerOnNetWithNfts,
    OperateRelayerOnAnotherNet,
    IntegrateSeInBlockExplorer,
    IntegrateSeInBrowserWallet,
    IntegrateSeInAndroidWallet,
    IntegrateSeInIosWallet,
    IntegrateSeInAnotherWallet,
    SupportShieldedTxsInBlockExplorer,
    SupportShieldedTxsInBrowserWallet,
    SupportShieldedTxsInAndroidWallet,
    SupportShieldedTxsInIosWallet,
    BuildAdditionalFossTooling,
    BuildWebAppWithShieldedActionOnIbcChain,
    OsmosisFrontendShieldedSwaps,
    AnotherAppWithShieldedActionOnIbcChain,
    ReduceMaspProofGenTime,
    IncreaseNoteScanSpeed,
    FindAndProveNamSpecsFlaw,
    OptimizeNamSmExecSpeed,
    FindProtocolSecVulnerability,
}

#[derive(Debug, Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaskDb {
    pub id: i32,
    pub task: TaskTypeDb,
    pub player_id: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaskInsertDb {
    pub task: TaskTypeDb,
    pub player_id: String,
}

#[derive(Debug, Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = unidentified_tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UnidentifiedTaskDb {
    pub id: i32,
    pub tx_kind: TransactionKindDb,
    pub player_id: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = unidentified_tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UnidentifiedTaskInsertDb {
    pub tx_kind: TransactionKindDb,
    pub player_id: String,
}
