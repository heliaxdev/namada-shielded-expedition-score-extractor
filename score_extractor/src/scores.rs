use std::collections::hash_map::{self, HashMap};
use std::collections::HashSet;
use std::marker::PhantomData;

use anyhow::Context as AnyhowContext;
use either::*;
use shared::orm::players::PlayerKindDb;
use shared::orm::schema;
use shared::orm::tasks::{TaskDb, TaskTypeDb, UnidentifiedTaskDb};
use shared::orm::transaction::TransactionKindDb;

use crate::context::Context;
use crate::db;
use crate::players::{
    process_all_pilots, process_all_pilots_with_nonnull_validator_addr, PilotValidatorAddress,
    PlayerId,
};
use crate::players::{NUMBER_CREW_MEMBERS, NUMBER_PILOTS};
use crate::tasks::CompletableBy;

#[derive(Debug)]
struct UnidentifiedTask(TransactionKindDb);

#[derive(Debug)]
struct IdentifiedTask(TaskTypeDb);

#[derive(Debug)]
struct FixedShare(f64);

#[derive(Debug)]
struct RelativeToCompletionShare(f64);

#[derive(Debug)]
enum PlayerKindCrew {}

#[derive(Debug)]
enum PlayerKindPilot {}

#[derive(Debug)]
struct PoolPrize<P, S> {
    total_shares: S,
    _player_kind: PhantomData<P>,
}

impl<P, S> PoolPrize<P, S> {
    fn new(total_shares: S) -> Self {
        Self {
            total_shares,
            _player_kind: PhantomData,
        }
    }
}

impl PoolPrize<PlayerKindCrew, FixedShare> {
    fn update_score(self, conn: &mut db::Connection, player_id: &str) -> anyhow::Result<()> {
        let FixedShare(total_shares) = self.total_shares;
        let share = (total_shares / NUMBER_CREW_MEMBERS as f64) as i64;
        tracing::info!(player_id, share, "Computed score shares for player");
        set_assign_player_score(conn, player_id, share)
    }
}

impl PoolPrize<PlayerKindPilot, FixedShare> {
    fn update_score(self, conn: &mut db::Connection, player_id: &str) -> anyhow::Result<()> {
        let FixedShare(total_shares) = self.total_shares;
        let share = (total_shares / NUMBER_PILOTS as f64) as i64;
        tracing::info!(player_id, share, "Computed score shares for player");
        set_assign_player_score(conn, player_id, share)
    }
}

impl<P> PoolPrize<P, RelativeToCompletionShare> {
    fn update_score(
        self,
        conn: &mut db::Connection,
        player_id: &str,
        CompletedBy(completed_players): CompletedBy,
    ) -> anyhow::Result<()> {
        let RelativeToCompletionShare(total_shares) = self.total_shares;
        let share = (total_shares / completed_players as f64) as i64;
        tracing::info!(player_id, share, "Computed score shares for player");
        set_assign_player_score(conn, player_id, share)
    }
}

#[derive(Debug, Copy, Clone)]
enum Score {
    Fixed(f64),
    RelativeToCompletion(f64),
}

#[derive(Copy, Clone)]
struct CompletedBy(i64);

enum PoolPrizeKind {
    FixedCrew(PoolPrize<PlayerKindCrew, FixedShare>),
    FixedPilot(PoolPrize<PlayerKindPilot, FixedShare>),
    RelativeCrew(PoolPrize<PlayerKindCrew, RelativeToCompletionShare>),
    RelativePilot(PoolPrize<PlayerKindPilot, RelativeToCompletionShare>),
}

pub fn fetch_pilots_with_nonzero_score(
    conn: &mut db::Connection,
) -> anyhow::Result<HashSet<String>> {
    use diesel::connection::DefaultLoadingMode;
    use diesel::prelude::*;
    use schema::players;

    let set: HashSet<_> = players::table
        .filter(players::dsl::score.gt(0))
        .select(players::dsl::id)
        .load_iter::<String, DefaultLoadingMode>(conn)
        .context("Failed to fetch players with non-zero score from db")?
        .map(|maybe_id| maybe_id.context("Failed to read player id with non-zero score from db"))
        .collect::<anyhow::Result<_>>()?;

    Ok(set)
}

#[inline]
pub fn recompute_task_scores(conn: &mut db::Connection, cx: Context) -> anyhow::Result<()> {
    let pilots_with_nonzero_score = fetch_pilots_with_nonzero_score(conn)?;
    reset_player_scores(conn).context("Failed to reset player scores")?;
    recompute_completed_task_scores(conn, &cx)
        .context("Failed to recompute completed task scores")?;
    recompute_ongoing_task_scores(conn, &cx, &pilots_with_nonzero_score)
        .context("Failed to recompute ongoing task scores")?;
    Ok(())
}

fn reset_player_scores(conn: &mut db::Connection) -> anyhow::Result<()> {
    use diesel::prelude::*;
    use schema::players::dsl::*;

    diesel::update(players).set(score.eq(0)).execute(conn)?;
    tracing::info!("Reset all player scores");

    Ok(())
}

fn set_assign_player_score(
    conn: &mut db::Connection,
    player_id: &str,
    share: i64,
) -> anyhow::Result<()> {
    use diesel::prelude::*;
    use schema::players::dsl::*;

    if share == 0 {
        tracing::info!(player_id, "Skipping setting player score shares of 0");
        return anyhow::Ok(());
    }

    diesel::update(players.filter(id.eq(player_id).and(is_banned.ne(true))))
        .set(score.eq(score + share))
        .execute(conn)
        .with_context(|| {
            format!("Failed to assign share of {share} to the score of {player_id}")
        })?;

    tracing::info!(player_id, share, "Assigned new share to player score");

    Ok(())
}

fn recompute_ongoing_task_scores(
    conn: &mut db::Connection,
    cx: &Context,
    pilots_with_nonzero_score: &HashSet<String>,
) -> anyhow::Result<()> {
    recompute_gov_task_scores(conn, cx)
        .context("Failed to recompute governance participation task scores")?;
    recompute_uptime_task_scores(conn, cx, pilots_with_nonzero_score)
        .context("Failed to recompute uptime task scores")?;
    Ok(())
}

fn recompute_gov_task_scores(conn: &mut db::Connection, cx: &Context) -> anyhow::Result<()> {
    let mut pilots_with_gov_participation_over_90 = HashMap::with_capacity(NUMBER_PILOTS);

    // compute who finished gov participation tasks this round
    process_all_pilots(conn, |transaction_conn, PlayerId(player_id)| {
        let participation_rate =
            compute_governance_participation_rate(transaction_conn, &player_id).with_context(
                || format!("Failed to compute governance participation rate of pilot {player_id}"),
            )?;

        if participation_rate >= 0.90 {
            pilots_with_gov_participation_over_90.insert(player_id, participation_rate);
        }

        Ok(())
    })?;

    let no_gov_participation_rate_over_90 =
        CompletedBy(pilots_with_gov_participation_over_90.len() as _);
    let no_gov_participation_rate_over_99 = CompletedBy(
        pilots_with_gov_participation_over_90
            .values()
            .filter(|&&participation_rate| participation_rate >= 0.99)
            .count() as _,
    );

    // assign scores
    for (player_id, participation_rate) in pilots_with_gov_participation_over_90 {
        update_score(
            conn,
            &player_id,
            no_gov_participation_rate_over_90,
            cx,
            Right(IdentifiedTask(
                TaskTypeDb::Keep90PerCentGovParticipationRate,
            )),
        )?;

        if participation_rate >= 0.99 {
            update_score(
                conn,
                &player_id,
                no_gov_participation_rate_over_99,
                cx,
                Right(IdentifiedTask(
                    TaskTypeDb::Keep99PerCentGovParticipationRate,
                )),
            )?;
        }
    }

    Ok(())
}

fn recompute_uptime_task_scores(
    conn: &mut db::Connection,
    cx: &Context,
    pilots_with_nonzero_score: &HashSet<String>,
) -> anyhow::Result<()> {
    let mut pilots_with_uptime_over_95 = HashMap::with_capacity(NUMBER_PILOTS);

    // compute who finished uptime tasks this round
    process_all_pilots_with_nonnull_validator_addr(
        conn,
        |transaction_conn, PlayerId(player_id), pilot_addr| {
            if !pilots_with_nonzero_score.contains(&player_id) {
                return Ok(());
            }

            let uptime = compute_pilot_uptime(transaction_conn, pilot_addr)
                .with_context(|| format!("Failed to compute uptime of pilot {player_id}"))?;

            if uptime >= 0.95 {
                pilots_with_uptime_over_95.insert(player_id, uptime);
            }

            Ok(())
        },
    )?;

    let no_uptime_over_95 = CompletedBy(pilots_with_uptime_over_95.len() as _);
    let no_uptime_over_99 = CompletedBy(
        pilots_with_uptime_over_95
            .values()
            .filter(|&&uptime| uptime >= 0.99)
            .count() as _,
    );

    // assign scores
    for (player_id, uptime) in pilots_with_uptime_over_95 {
        update_score(
            conn,
            &player_id,
            no_uptime_over_95,
            cx,
            Right(IdentifiedTask(TaskTypeDb::Keep95PerCentUptime)),
        )?;

        if uptime >= 0.99 {
            update_score(
                conn,
                &player_id,
                no_uptime_over_99,
                cx,
                Right(IdentifiedTask(TaskTypeDb::Keep99PerCentUptime)),
            )?;
        }
    }

    Ok(())
}

fn recompute_completed_task_scores(conn: &mut db::Connection, cx: &Context) -> anyhow::Result<()> {
    process_identified_tasks(
        conn,
        |transaction_conn,
         num_completed_players,
         TaskDb {
             task, player_id, ..
         }| {
            tracing::info!(player_id, "Processing player's identified task score");
            update_score(
                transaction_conn,
                &player_id,
                num_completed_players,
                cx,
                Right(IdentifiedTask(task)),
            )
        },
    )
    .context("Failed to recompute completed identified task scores")?;

    process_unidentified_tasks(
        conn,
        |transaction_conn,
         num_completed_players,
         UnidentifiedTaskDb {
             player_id, tx_kind, ..
         }| {
            tracing::info!(player_id, "Updating player's unidentified task score");
            update_score(
                transaction_conn,
                &player_id,
                num_completed_players,
                cx,
                Left(UnidentifiedTask(tx_kind)),
            )
        },
    )
    .context("Failed to recompute completed unidentified task scores")?;

    Ok(())
}

fn update_score(
    conn: &mut db::Connection,
    player_id: &str,
    num_completed_players: CompletedBy,
    cx: &Context,
    task_type: Either<UnidentifiedTask, IdentifiedTask>,
) -> anyhow::Result<()> {
    let player_kind = cx.player_kinds().get_or_update(player_id, conn)?;
    let Some(pool_prize) = get_task_pool_prize(&player_kind, task_type.as_ref()) else {
        return anyhow::Ok(());
    };

    match pool_prize {
        PoolPrizeKind::FixedCrew(prize) => prize.update_score(conn, player_id),
        PoolPrizeKind::FixedPilot(prize) => prize.update_score(conn, player_id),
        PoolPrizeKind::RelativeCrew(prize) => {
            prize.update_score(conn, player_id, num_completed_players)
        }
        PoolPrizeKind::RelativePilot(prize) => {
            prize.update_score(conn, player_id, num_completed_players)
        }
    }
}

fn get_task_pool_prize(
    player_kind: &PlayerKindDb,
    task_type: Either<&UnidentifiedTask, &IdentifiedTask>,
) -> Option<PoolPrizeKind> {
    use PlayerKindDb::*;
    use TaskTypeDb::*;

    const fn relative(points: f64) -> Score {
        Score::RelativeToCompletion(points)
    }

    const fn fixed(points: f64) -> Score {
        Score::Fixed(points)
    }

    let completable_by = &CompletableBy::check(task_type.map_either(
        |UnidentifiedTask(tx_kind)| tx_kind,
        |IdentifiedTask(task_type)| task_type,
    ));

    let cannot_be_assigned_points = matches!(
        (completable_by, player_kind),
        (CompletableBy::NoOne, _)
            | (CompletableBy::OnlyCrew, Pilot)
            | (CompletableBy::OnlyPilots, Crew)
    );

    if cannot_be_assigned_points {
        return None;
    }

    let pool_prize = task_type.either(
        |UnidentifiedTask(tx_type)| match (player_kind, tx_type) {
            (_, TransactionKindDb::Unknown) => None,
            (Crew, _) => Some(relative({
                // = 300_000_000_000.0 / (len(TransactionKindDb) - 1)
                // = 300_000_000_000.0 / 26
                11538461538.461538
            })),
            (Pilot, _) => Some(relative({
                // = 250_000_000_000.0 / (len(TransactionKindDb) - 1)
                // = 250_000_000_000.0 / 26
                9615384615.384615
            })),
        },
        |IdentifiedTask(task_type)| match task_type {
            DelegateStakeOnV0 => Some(fixed(42_857_142_857.14)),
            DelegateStakeOnV1 => Some(fixed(42_857_142_857.14)),
            ClaimPosRewards => Some(relative(42_857_142_857.14)),
            ShieldNaan => Some(relative(42_857_142_857.14)),
            UnshieldNaan => Some(relative(42_857_142_857.14)),
            ShieldToShielded => Some(relative(42_857_142_857.14)),
            ShieldAssetOverIbc => Some(relative(42_857_142_857.14)),
            SubmitPreGenesisBondTx => Some(relative(10_000_000_000.0)),
            StartNode5MinFromGenesis => Some(relative(34_285_714_286.0)),
            InitPostGenesisValidator => Some(relative(34_285_714_286.0)),
            InValidatorSetFor1Epoch => Some(relative(34_285_714_286.0)),
            VotePgfStewardProposal => Some(relative(34_285_714_286.0)),
            VoteUpgradeV0ToV1 => Some(relative(34_285_714_286.0)),
            VoteUpgradeV1ToV2 => Some(relative(34_285_714_286.0)),
            SignFirstBlockOfUpgradeToV2 => Some(relative(34_285_714_286.0)),
            Keep99PerCentUptime => Some(relative(31_250_000_000.0)),
            Keep95PerCentUptime => Some(relative(31_250_000_000.0)),
            Keep99PerCentGovParticipationRate => Some(relative(31_250_000_000.0)),
            Keep90PerCentGovParticipationRate => Some(relative(31_250_000_000.0)),
            ProvidePublicRpcEndpoint => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            OperateNamadaIndexer => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            OperateNamadaInterface => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            OperateCosmosTestnetRelayer => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            OperateOsmosisTestnetRelayer => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            OperateNobleTestnetRelayer => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            OperateRelayerOnNetWithNfts => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            OperateRelayerOnAnotherNet => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            IntegrateSeInBlockExplorer => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            IntegrateSeInBrowserWallet => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            IntegrateSeInAndroidWallet => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            IntegrateSeInIosWallet => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            IntegrateSeInAnotherWallet => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            SupportShieldedTxsInBlockExplorer => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            SupportShieldedTxsInBrowserWallet => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            SupportShieldedTxsInAndroidWallet => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            SupportShieldedTxsInIosWallet => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            BuildAdditionalFossTooling => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            BuildWebAppWithShieldedActionOnIbcChain => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            OsmosisFrontendShieldedSwaps => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            AnotherAppWithShieldedActionOnIbcChain => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            ReduceMaspProofGenTime => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            IncreaseNoteScanSpeed => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            FindAndProveNamSpecsFlaw => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            OptimizeNamSmExecSpeed => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
            FindProtocolSecVulnerability => Some(relative(match player_kind {
                Crew => 66_666_666_667.0,
                Pilot => 62_500_000_000.0,
            })),
        },
    )?;

    tracing::info!(?player_kind, ?task_type, ?pool_prize, "Computed pool prize");

    Some(match (player_kind, pool_prize) {
        (Crew, Score::Fixed(total)) => PoolPrizeKind::FixedCrew(PoolPrize::new(FixedShare(total))),
        (Pilot, Score::Fixed(total)) => {
            PoolPrizeKind::FixedPilot(PoolPrize::new(FixedShare(total)))
        }
        (Crew, Score::RelativeToCompletion(total)) => {
            PoolPrizeKind::RelativeCrew(PoolPrize::new(RelativeToCompletionShare(total)))
        }
        (Pilot, Score::RelativeToCompletion(total)) => {
            PoolPrizeKind::RelativePilot(PoolPrize::new(RelativeToCompletionShare(total)))
        }
    })
}

fn process_identified_tasks<F>(conn: &mut db::Connection, mut process: F) -> anyhow::Result<()>
where
    F: FnMut(&mut db::Connection, CompletedBy, TaskDb) -> anyhow::Result<()>,
{
    use diesel::connection::DefaultLoadingMode;
    use diesel::prelude::*;
    use schema::players;
    use schema::tasks;

    let mut completed_players = HashMap::new();

    tracing::info!("Processing identified tasks");

    tasks::table
        .inner_join(players::table)
        .filter(players::dsl::is_banned.ne(true))
        .select(TaskDb::as_select())
        .load_iter::<_, DefaultLoadingMode>(conn)
        .context("Failed to fetch tasks from the database")?
        .try_for_each(|task| {
            let task = task.context("Failed to deserialize task from database")?;
            let task_completed_player_num = CompletedBy(match completed_players.entry(task.task) {
                hash_map::Entry::Occupied(occupied) => *occupied.get(),
                hash_map::Entry::Vacant(vacant) => {
                    let completed_num: i64 = tasks::table
                        .filter(tasks::dsl::task.eq(task.task))
                        .count()
                        .first(conn)
                        .optional()
                        .context("Failed to query num of players that completed task")?
                        .unwrap_or_default();
                    vacant.insert(completed_num);
                    completed_num
                }
            });
            process(conn, task_completed_player_num, task)
        })?;

    Ok(())
}

fn process_unidentified_tasks<F>(conn: &mut db::Connection, mut process: F) -> anyhow::Result<()>
where
    F: FnMut(&mut db::Connection, CompletedBy, UnidentifiedTaskDb) -> anyhow::Result<()>,
{
    use diesel::connection::DefaultLoadingMode;
    use diesel::prelude::*;
    use schema::players;
    use schema::unidentified_tasks;

    let mut completed_players = HashMap::new();

    tracing::info!("Processing unidentified tasks");

    unidentified_tasks::table
        .inner_join(players::table)
        .filter(players::dsl::is_banned.ne(true))
        .select((UnidentifiedTaskDb::as_select(), players::dsl::kind))
        .load_iter::<(UnidentifiedTaskDb, PlayerKindDb), DefaultLoadingMode>(conn)
        .context("Failed to fetch unidentified tasks from the database")?
        .try_for_each(|db_row| {
            let (task, player_kind) =
                db_row.context("Failed to deserialize row data from database")?;
            let task_completed_player_num = CompletedBy(
                match completed_players.entry((task.tx_kind, player_kind.clone())) {
                    hash_map::Entry::Occupied(occupied) => *occupied.get(),
                    hash_map::Entry::Vacant(vacant) => {
                        let completed_num: i64 = unidentified_tasks::table
                            .inner_join(players::table)
                            .filter(
                                players::dsl::kind
                                    .eq(&player_kind)
                                    .and(unidentified_tasks::dsl::tx_kind.eq(&task.tx_kind)),
                            )
                            .count()
                            .first(conn)
                            .optional()
                            .context("Failed to query num of players that completed task")?
                            .unwrap_or_default();
                        vacant.insert(completed_num);
                        completed_num
                    }
                },
            );
            process(conn, task_completed_player_num, task)
        })?;

    Ok(())
}

fn compute_governance_participation_rate(
    conn: &mut db::Connection,
    player_id: &str,
) -> anyhow::Result<f64> {
    use diesel::dsl::count_star;
    use diesel::prelude::*;
    use schema::governance_proposals;
    use schema::governance_votes;

    tracing::info!(player_id, "Computing governance participation rate");

    let (no_of_votes, total_governance_proposals, participation_rate) = 'result: {
        let total_governance_proposals: i64 = governance_proposals::table
            .select(count_star())
            .first(conn)
            .optional()?
            .unwrap_or_default();

        if total_governance_proposals == 0 {
            break 'result (0, 0, 0.0);
        }

        let no_of_votes: i64 = governance_votes::table
            .filter(governance_votes::dsl::player_id.eq(player_id))
            // FIXME: ideally we count distinct proposal id votes  :-)
            //.distinct_on(governance_votes::proposal_id)
            .select(count_star())
            .first(conn)
            .optional()?
            .unwrap_or_default();

        debug_assert!(no_of_votes <= total_governance_proposals);
        let participation_rate = no_of_votes as f64 / total_governance_proposals as f64;

        (no_of_votes, total_governance_proposals, participation_rate)
    };

    tracing::info!(
        player_id,
        no_of_votes,
        total_governance_proposals,
        participation_rate,
        "Computed pilot governance participation_rate"
    );

    Ok(participation_rate)
}

fn compute_pilot_uptime(
    conn: &mut db::Connection,
    pilot_addr: PilotValidatorAddress,
) -> anyhow::Result<f64> {
    use diesel::prelude::*;
    use schema::commits;
    use schema::players;
    use schema::tm_addresses;

    use crate::sql_ext::CountInnerDsl;

    let PilotValidatorAddress(pilot_addr) = pilot_addr;

    tracing::info!(pilot_addr, "Computing pilot uptime");

    let (signed_blocks, total_blocks, uptime) = {
        const TOTAL_BLOCKS: i64 = 355326;
        const TOTAL_BLOCKS_F64: f64 = 355326.0;

        let signed_blocks: i64 = {
            let validator_addr_with_same_player_id = players::table
                .filter(
                    players::dsl::namada_validator_address
                        .nullable()
                        .eq(&pilot_addr),
                )
                .select(players::dsl::namada_validator_address);

            let tm_addrs_with_same_player_id = tm_addresses::table
                .filter(
                    tm_addresses::dsl::validator_namada_address
                        .nullable()
                        .eq_any(validator_addr_with_same_player_id),
                )
                .select(tm_addresses::dsl::tm_address);

            let eligible_pilot_nam_addrs = commits::table
                .filter(commits::dsl::address.eq_any(tm_addrs_with_same_player_id))
                .select(commits::dsl::address);

            eligible_pilot_nam_addrs
                .count_inner()
                .get(conn)
                .with_context(|| format!("Failed to query no. of blocks signed by {pilot_addr}"))?
        };

        debug_assert!(signed_blocks <= TOTAL_BLOCKS);
        let uptime = signed_blocks as f64 / TOTAL_BLOCKS_F64;

        (signed_blocks, TOTAL_BLOCKS, uptime)
    };

    tracing::info!(
        pilot_addr,
        signed_blocks,
        total_blocks,
        uptime,
        "Computed pilot uptime"
    );

    Ok(uptime)
}
