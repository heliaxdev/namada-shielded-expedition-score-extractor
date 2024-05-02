use anyhow::Context as AnyhowContext;
use borsh::BorshDeserialize;
use either::*;
use namada_core::types::address::MASP;
use namada_core::types::token::Transfer as NamadaTransfer;
use shared::orm::governance_proposals::GovernanceProposalKindDb;
use shared::orm::players::PlayerKindDb;
use shared::orm::schema;
use shared::orm::tasks::{TaskInsertDb, TaskTypeDb, UnidentifiedTaskInsertDb};
use shared::orm::transaction::TransactionKindDb;
use shared::transaction::{Transaction, TransactionKind};

use crate::context::Context;
use crate::db;
use crate::players::{player_exists, process_all_pilots_with_incomplete_tasks, PlayerId};
use crate::sql_ext::ExistsInnerDsl;

pub enum CompletableBy {
    NoOne,
    OnlyCrew,
    OnlyPilots,
    DependsOnPlayerKind,
}

impl CompletableBy {
    pub fn check(task_type: Either<&TransactionKindDb, &TaskTypeDb>) -> Self {
        use TaskTypeDb::*;

        task_type.either(
            |tx_type| match tx_type {
                TransactionKindDb::Unknown => CompletableBy::NoOne,
                _ => CompletableBy::DependsOnPlayerKind,
            },
            |task_type| match task_type {
                DelegateStakeOnV0 | DelegateStakeOnV1 | ClaimPosRewards | ShieldNaan
                | UnshieldNaan | ShieldToShielded | ShieldAssetOverIbc => CompletableBy::OnlyCrew,
                SubmitPreGenesisBondTx
                | StartNode5MinFromGenesis
                | InitPostGenesisValidator
                | InValidatorSetFor1Epoch
                | VotePgfStewardProposal
                | VoteUpgradeV0ToV1
                | VoteUpgradeV1ToV2
                | SignFirstBlockOfUpgradeToV2
                | Keep99PerCentUptime
                | Keep95PerCentUptime
                | Keep99PerCentGovParticipationRate
                | Keep90PerCentGovParticipationRate => CompletableBy::OnlyPilots,
                ProvidePublicRpcEndpoint
                | OperateNamadaIndexer
                | OperateNamadaInterface
                | OperateCosmosTestnetRelayer
                | OperateOsmosisTestnetRelayer
                | OperateNobleTestnetRelayer
                | OperateRelayerOnNetWithNfts
                | OperateRelayerOnAnotherNet
                | IntegrateSeInBlockExplorer
                | IntegrateSeInBrowserWallet
                | IntegrateSeInAndroidWallet
                | IntegrateSeInIosWallet
                | IntegrateSeInAnotherWallet
                | SupportShieldedTxsInBlockExplorer
                | SupportShieldedTxsInBrowserWallet
                | SupportShieldedTxsInAndroidWallet
                | SupportShieldedTxsInIosWallet
                | BuildAdditionalFossTooling
                | BuildWebAppWithShieldedActionOnIbcChain
                | OsmosisFrontendShieldedSwaps
                | AnotherAppWithShieldedActionOnIbcChain
                | ReduceMaspProofGenTime
                | IncreaseNoteScanSpeed
                | FindAndProveNamSpecsFlaw
                | OptimizeNamSmExecSpeed
                | FindProtocolSecVulnerability => CompletableBy::DependsOnPlayerKind,
            },
        )
    }
}

fn compute_insertable_task_from_tx(
    conn: &mut db::Connection,
    cx: &Context,
    transaction: Transaction<PlayerId>,
) -> anyhow::Result<Option<Either<UnidentifiedTaskInsertDb, TaskInsertDb>>> {
    let Some(PlayerId(player_id)) = transaction.memo else {
        return Ok(None);
    };

    if !player_exists(conn, &player_id)? {
        return Ok(None);
    }

    let kind = match &transaction.kind {
        TransactionKind::Bond(_) => {
            use diesel::prelude::*;
            use schema::blocks;
            use schema::transactions;

            let tx_id = transaction.hash.to_string();
            let tx_epoch: i32 = {
                // TODO: try to switch to `.single_value()` and
                // replace `.eq_any()` with `.eq()`
                let block_id_of_tx = transactions::table
                    .filter(transactions::dsl::id.eq(&tx_id))
                    .select(transactions::dsl::block_id);

                blocks::table
                    .filter(blocks::dsl::id.eq_any(block_id_of_tx))
                    .select(blocks::dsl::epoch)
                    .first(conn)
                    .context("Block should have been in database")?
            };

            match () {
                _ if tx_epoch < cx.epochs().v0_to_v1.0 as i32 => {
                    Right(TaskTypeDb::DelegateStakeOnV0)
                }
                _ if tx_epoch < cx.epochs().v1_to_v2.0 as i32 => {
                    Right(TaskTypeDb::DelegateStakeOnV1)
                }
                _ => {
                    return Ok(Some(Left(UnidentifiedTaskInsertDb {
                        player_id,
                        tx_kind: TransactionKindDb::Bond,
                    })))
                }
            }
        }
        TransactionKind::IbcShieldedTransfer(_) => Right(TaskTypeDb::ShieldAssetOverIbc),
        TransactionKind::ShieldedTransfer(data) => {
            let Some(transfer) = NamadaTransfer::try_from_slice(data)
                .map_err(|err| {
                    tracing::warn!(
                        %err,
                        "Failed to deserialize a Namada transfer from an indexed masp tx"
                    );
                })
                .ok()
            else {
                return Ok(None);
            };

            match (transfer.source, transfer.target) {
                (MASP, MASP) => Right(TaskTypeDb::ShieldToShielded),
                (src, MASP) if src == cx.address_book().naan => Right(TaskTypeDb::ShieldNaan),
                (MASP, dst) if dst == cx.address_book().naan => Right(TaskTypeDb::UnshieldNaan),
                (_source, _target) => {
                    return Ok(Some(Left(UnidentifiedTaskInsertDb {
                        player_id,
                        tx_kind: TransactionKindDb::ShieldedTransfer,
                    })))
                }
            }
        }
        TransactionKind::BecomeValidator(_) => Right(TaskTypeDb::InitPostGenesisValidator),
        TransactionKind::ClaimRewards(_) => Right(TaskTypeDb::ClaimPosRewards),
        TransactionKind::ProposalVote(data) => 'proposal_kind: {
            use diesel::prelude::*;
            use namada_governance::VoteProposalData;
            use schema::governance_proposals;
            use schema::governance_votes;

            const REGULAR_PROPOSAL: Either<TransactionKindDb, TaskTypeDb> =
                Left(TransactionKindDb::ProposalVote);
            const V0_TO_V1_PROPOSAL: Either<TransactionKindDb, TaskTypeDb> =
                Right(TaskTypeDb::VoteUpgradeV0ToV1);
            const V1_TO_V2_PROPOSAL: Either<TransactionKindDb, TaskTypeDb> =
                Right(TaskTypeDb::VoteUpgradeV1ToV2);
            const PGF_STEWARD_PROPOSAL: Either<TransactionKindDb, TaskTypeDb> =
                Right(TaskTypeDb::VotePgfStewardProposal);

            let Some(data) = VoteProposalData::try_from_slice(data)
                .map_err(|err| {
                    tracing::warn!(
                        %err,
                        "Failed to deserialize Namada governance proposal data from indexed tx"
                    );
                })
                .ok()
            else {
                return Ok(None);
            };

            if matches!(data.id, 316 | 385) {
                break 'proposal_kind V0_TO_V1_PROPOSAL;
            }

            let maybe_proposal_data = governance_votes::table
                .inner_join(
                    governance_proposals::table
                        .on(governance_votes::dsl::proposal_id.eq(governance_proposals::dsl::id)),
                )
                .filter(governance_proposals::dsl::id.eq(data.id as i32))
                .select((
                    governance_proposals::dsl::author,
                    governance_proposals::dsl::grace_epoch,
                    governance_proposals::dsl::kind,
                ))
                .first::<(String, i32, GovernanceProposalKindDb)>(conn)
                .optional()
                .context("Failed to query proposal data from db")?;

            let Some((proposer_in_db, grace_epoch_in_db, proposal_kind)) = maybe_proposal_data
            else {
                tracing::warn!(
                    %player_id,
                    proposal_data = ?data,
                    "No governance proposal in db matching given data"
                );
                return Ok(None);
            };

            if matches!(proposal_kind, GovernanceProposalKindDb::PgfSteward) {
                break 'proposal_kind PGF_STEWARD_PROPOSAL;
            }

            if proposer_in_db != cx.address_book().upgrade_proposer.to_string() {
                tracing::trace!(
                    %player_id,
                    upgrade_proposer = %cx.address_book().upgrade_proposer,
                    %proposer_in_db,
                    "Governance proposal is not from upgrade proposer"
                );
                break 'proposal_kind REGULAR_PROPOSAL;
            }

            match () {
                _ if grace_epoch_in_db == cx.epochs().v0_to_v1.0 as i32 => V0_TO_V1_PROPOSAL,
                _ if grace_epoch_in_db == cx.epochs().v1_to_v2.0 as i32 => V1_TO_V2_PROPOSAL,
                _ => {
                    tracing::trace!(
                        %player_id,
                        proposal_data = ?data,
                        grace_epoch_in_db,
                        v0_to_v1_epoch = ?cx.epochs().v0_to_v1,
                        v1_to_v2_epoch = ?cx.epochs().v1_to_v2,
                        "Proposal's grace epoch does not match any of the upgrade epochs"
                    );
                    REGULAR_PROPOSAL
                }
            }
        }
        kind => Left(kind.into()),
    };

    match CompletableBy::check(kind.as_ref()) {
        CompletableBy::DependsOnPlayerKind => (),
        CompletableBy::NoOne => {
            tracing::trace!(
                %player_id,
                task_kind = ?kind,
                "Ignoring task that cannot be completed"
            );
            return Ok(None);
        }
        CompletableBy::OnlyCrew => {
            let player_kind = cx.player_kinds().get_or_update(&player_id, conn)?;
            if !matches!(player_kind, PlayerKindDb::Crew) {
                return Ok(Some(Left(UnidentifiedTaskInsertDb {
                    player_id,
                    tx_kind: (&transaction.kind).into(),
                })));
            }
        }
        CompletableBy::OnlyPilots => {
            let player_kind = cx.player_kinds().get_or_update(&player_id, conn)?;
            if !matches!(player_kind, PlayerKindDb::Pilot) {
                return Ok(Some(Left(UnidentifiedTaskInsertDb {
                    player_id,
                    tx_kind: (&transaction.kind).into(),
                })));
            }
        }
    }

    Ok(Some(kind.map_either_with(
        player_id,
        |player_id, tx_kind| UnidentifiedTaskInsertDb { player_id, tx_kind },
        |player_id, task| TaskInsertDb { player_id, task },
    )))
}

#[derive(Debug)]
pub enum TaskInput<'a> {
    /// Transaction input.
    Transaction {
        tx: Transaction<PlayerId>,
        cx: &'a Context,
    },
    /// Pilot input.
    Pilot { cx: &'a Context },
    /// Special tasks input.
    SpecialTasks,
}

pub fn update_task_statuses(conn: &mut db::Connection, input: TaskInput) -> anyhow::Result<()> {
    tracing::debug!(?input, "Attempting to insert task into database");

    match input {
        TaskInput::Transaction { tx, cx } => mark_task_completed_from_tx(conn, cx, tx),
        TaskInput::Pilot { cx } => mark_completed_pilot_tasks(conn, cx),
        TaskInput::SpecialTasks => mark_completed_special_tasks(conn),
    }
}

fn mark_completed_special_tasks(conn: &mut db::Connection) -> anyhow::Result<()> {
    use diesel::prelude::*;

    let affected_rows = diesel::sql_query(
        r#"
        INSERT INTO tasks ( player_id, task )
        SELECT player_id, task
        FROM manual_tasks ON CONFLICT DO NOTHING
        "#,
    )
    .execute(conn)
    .context("Failed to mark special tasks as completed")?;

    tracing::info!(
        no_of_tasks = affected_rows,
        "Special tasks updated in database"
    );

    Ok(())
}

fn mark_completed_pilot_tasks(conn: &mut db::Connection, cx: &Context) -> anyhow::Result<()> {
    let Some(genesis_time) = cx
        .genesis_time()
        .copied()
        .map(Ok)
        .or_else(|| {
            use diesel::prelude::*;
            use schema::blocks;

            blocks::table
                .filter(blocks::dsl::height.eq(1))
                .select(blocks::dsl::included_at)
                .first(conn)
                .optional()
                .context("Failed to read genesis time from db")
                .transpose()
        })
        .transpose()?
    else {
        tracing::info!("No blocks have been committed yet, can't check pilot tasks");
        return Ok(());
    };

    process_all_pilots_with_incomplete_tasks(
        conn,
        TaskTypeDb::StartNode5MinFromGenesis,
        |conn, PlayerId(player_id)| {
            if signed_block_5min_after_genesis(conn, &genesis_time, &player_id)? {
                use diesel::prelude::*;
                use schema::tasks;

                let affected_rows = diesel::insert_into(tasks::table)
                    .values(&TaskInsertDb {
                        task: TaskTypeDb::StartNode5MinFromGenesis,
                        player_id: player_id.clone(),
                    })
                    .on_conflict_do_nothing()
                    .execute(conn)
                    .context("Failed to insert 5min after genesis task into db")?;

                if affected_rows == 0 {
                    tracing::warn!(
                        player_id,
                        upgrade_to_v2_grace_epoch = %cx.epochs().v1_to_v2,
                        "Pilot's \"signed block up to 5mins after genesis\" \
                         task somehow had already been completed"
                    );
                } else {
                    tracing::info!(
                        player_id,
                        genesis_time = ?cx.genesis_time(),
                        "Task completed - pilot signed block up to 5mins after genesis"
                    );
                }
            }
            Ok(())
        },
    )?;

    process_all_pilots_with_incomplete_tasks(
        conn,
        TaskTypeDb::SignFirstBlockOfUpgradeToV2,
        |conn, PlayerId(player_id)| {
            if signed_first_block_of_upgrade_to_v2(conn, cx.epochs().v1_to_v2.0 as i32, &player_id)?
            {
                use diesel::prelude::*;
                use schema::tasks;

                let affected_rows = diesel::insert_into(tasks::table)
                    .values(&TaskInsertDb {
                        task: TaskTypeDb::SignFirstBlockOfUpgradeToV2,
                        player_id: player_id.clone(),
                    })
                    .on_conflict_do_nothing()
                    .execute(conn)
                    .context(
                        "Failed to insert \"first block of upgrade \
                         signed\" task into db",
                    )?;

                if affected_rows == 0 {
                    tracing::warn!(
                        player_id,
                        upgrade_to_v2_grace_epoch = %cx.epochs().v1_to_v2,
                        "Pilot's \"signed first block of upgrade to v2's grace epoch\" \
                         task somehow had already been completed"
                    );
                } else {
                    tracing::info!(
                        player_id,
                        upgrade_to_v2_grace_epoch = %cx.epochs().v1_to_v2,
                        "Task completed - pilot signed first block of upgrade to v2's \
                         grace epoch"
                    );
                }
            }
            Ok(())
        },
    )?;

    process_all_pilots_with_incomplete_tasks(
        conn,
        TaskTypeDb::InValidatorSetFor1Epoch,
        |conn, PlayerId(player_id)| {
            if signed_at_least_one_block(conn, &player_id)? {
                use diesel::prelude::*;
                use schema::tasks;

                let affected_rows = diesel::insert_into(tasks::table)
                    .values(&TaskInsertDb {
                        task: TaskTypeDb::InValidatorSetFor1Epoch,
                        player_id: player_id.clone(),
                    })
                    .on_conflict_do_nothing()
                    .execute(conn)
                    .context("Failed to insert signed block task into db")?;

                if affected_rows == 0 {
                    tracing::warn!(
                        player_id,
                        upgrade_to_v2_grace_epoch = %cx.epochs().v1_to_v2,
                        "Pilot's \"signed at least one block\" \
                         task somehow had already been completed"
                    );
                } else {
                    tracing::info!(
                        player_id,
                        "Task completed - pilot signed at least one block"
                    );
                }
            }
            Ok(())
        },
    )?;

    Ok(())
}

fn signed_at_least_one_block(conn: &mut db::Connection, player_id: &str) -> anyhow::Result<bool> {
    use diesel::prelude::*;
    use schema::commits;
    use schema::players;
    use schema::tm_addresses;

    let validator_addr_with_same_player_id = players::table
        .filter(
            players::dsl::namada_validator_address
                .is_not_null()
                .and(players::dsl::kind.eq(PlayerKindDb::Pilot))
                .and(players::dsl::id.eq(player_id)),
        )
        .select(players::dsl::namada_validator_address);

    let tm_addrs_with_same_player_id = tm_addresses::table
        .filter(
            tm_addresses::dsl::validator_namada_address
                .nullable()
                .eq_any(validator_addr_with_same_player_id),
        )
        .select(tm_addresses::dsl::tm_address);

    let eligible_pilot_nam_addrs: bool = {
        let eligible_pilot_nam_addrs = commits::table
            .filter(commits::dsl::address.eq_any(tm_addrs_with_same_player_id))
            .select(commits::dsl::address);

        eligible_pilot_nam_addrs
            .exists_inner()
            .get(conn)
            .with_context(|| {
                format!(
                    "Failed to check if pilot with id {player_id} signed \
                         at least one block"
                )
            })?
    };

    tracing::info!(
        %player_id,
        eligible_pilot_nam_addrs,
        "Checking if player signed at least one block"
    );

    Ok(eligible_pilot_nam_addrs)
}

fn signed_first_block_of_upgrade_to_v2(
    conn: &mut db::Connection,
    v2_grace_epoch: i32,
    player_id: &str,
) -> anyhow::Result<bool> {
    use diesel::dsl::min;
    use diesel::prelude::*;
    use schema::blocks;
    use schema::commits;
    use schema::players;
    use schema::tm_addresses;

    diesel::alias!(blocks as blocks_alias: BlocksAlias);

    let min_height = blocks_alias
        .filter(blocks_alias.field(blocks::dsl::epoch).eq(v2_grace_epoch))
        .select(min(blocks_alias.field(blocks::dsl::height)));

    let eligible_pilot_tm_addrs = blocks::table
        .inner_join(commits::table)
        .filter(blocks::dsl::height.nullable().eq_any(min_height))
        .group_by(commits::dsl::address)
        .select(commits::dsl::address);

    let validator_addr_with_same_player_id = players::table
        .filter(
            players::dsl::namada_validator_address
                .is_not_null()
                .and(players::dsl::kind.eq(PlayerKindDb::Pilot))
                .and(players::dsl::id.eq(player_id)),
        )
        .select(players::dsl::namada_validator_address);

    let eligible_pilot_nam_addrs: bool = {
        let eligible_pilot_nam_addrs = tm_addresses::table
            .group_by(tm_addresses::dsl::validator_namada_address)
            .filter(
                tm_addresses::dsl::validator_namada_address
                    .nullable()
                    .eq_any(validator_addr_with_same_player_id)
                    .and(tm_addresses::dsl::tm_address.eq_any(eligible_pilot_tm_addrs)),
            )
            .select(tm_addresses::dsl::validator_namada_address);

        eligible_pilot_nam_addrs
            .exists_inner()
            .get(conn)
            .with_context(|| {
                format!(
                    "Failed to check if pilot with id {player_id} signed \
                     first block since upgrade to v2 task"
                )
            })?
    };

    tracing::info!(
        %player_id,
        eligible_pilot_nam_addrs,
        "Checking if player signed first block since upgrade to v2"
    );

    Ok(eligible_pilot_nam_addrs)
}

fn signed_block_5min_after_genesis(
    conn: &mut db::Connection,
    genesis_time: &chrono::NaiveDateTime,
    player_id: &str,
) -> anyhow::Result<bool> {
    use diesel::prelude::*;
    use schema::blocks;
    use schema::commits;
    use schema::players;
    use schema::tm_addresses;

    let five_mins_after_genesis = *genesis_time + chrono::Duration::minutes(5);

    let eligible_pilot_tm_addrs = blocks::table
        .inner_join(commits::table)
        .filter(blocks::dsl::included_at.le(five_mins_after_genesis))
        .group_by(commits::dsl::address)
        .select(commits::dsl::address);

    let validator_addr_with_same_player_id = players::table
        .filter(
            players::dsl::namada_validator_address
                .is_not_null()
                .and(players::dsl::kind.eq(PlayerKindDb::Pilot))
                .and(players::dsl::id.eq(player_id)),
        )
        .select(players::dsl::namada_validator_address);

    let eligible_pilot_nam_addrs: bool = {
        let eligible_pilot_nam_addrs = tm_addresses::table
            .group_by(tm_addresses::dsl::validator_namada_address)
            .filter(
                tm_addresses::dsl::validator_namada_address
                    .nullable()
                    .eq_any(validator_addr_with_same_player_id)
                    .and(tm_addresses::dsl::tm_address.eq_any(eligible_pilot_tm_addrs)),
            )
            .select(tm_addresses::dsl::validator_namada_address);

        eligible_pilot_nam_addrs
            .exists_inner()
            .get(conn)
            .with_context(|| {
                format!(
                    "Failed to check if pilot with id {player_id} signed \
                     blocks up to 5 mins after genesis time"
                )
            })?
    };

    tracing::info!(
        %player_id,
        eligible_pilot_nam_addrs,
        "Checking if player signed blocks up to 5mins after genesis"
    );

    Ok(eligible_pilot_nam_addrs)
}

fn mark_task_completed_from_tx(
    conn: &mut db::Connection,
    cx: &Context,
    input: Transaction<PlayerId>,
) -> anyhow::Result<()> {
    use diesel::result::DatabaseErrorKind;
    use diesel::result::Error;
    use diesel::RunQueryDsl;

    let tx_id = input.hash.clone();

    let Some(task_insertion) = compute_insertable_task_from_tx(conn, cx, input)? else {
        tracing::debug!(
            ?tx_id,
            "No task to be inserted in database from given tx input"
        );
        return Ok(());
    };

    let task_insertion_debug = format!("{task_insertion:?}");

    let affected_rows = task_insertion
        .either_with(
            conn,
            |conn, insertable_unidentified_task| {
                use schema::unidentified_tasks::dsl::*;

                diesel::insert_into(unidentified_tasks)
                    .values(&insertable_unidentified_task)
                    .on_conflict((player_id, tx_kind))
                    .do_nothing()
                    .execute(conn)
            },
            |conn, insertable_task| {
                use schema::tasks::dsl::*;

                diesel::insert_into(tasks)
                    .values(&insertable_task)
                    .on_conflict((player_id, task))
                    .do_nothing()
                    .execute(conn)
            },
        )
        .or_else(|err| {
            // NB: if a uniqueness constraint was violated, we simply ignore the error,
            // since it means a task was already completed. otherwise, we return the
            // error as is
            if matches!(
                &err,
                Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)
            ) {
                Ok(0)
            } else {
                Err(err)
            }
        })
        .context("Failed to execute a database query")?;

    if affected_rows == 0 {
        tracing::debug!(?tx_id, "Task already in database, skipping insertion");
    } else {
        tracing::info!(
            task = ?task_insertion_debug,
            "Task completed - tx task"
        );
    }

    Ok(())
}
