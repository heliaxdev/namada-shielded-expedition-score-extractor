use anyhow::Context as AnyhowContext;
use clap::Parser;
use clap_verbosity_flag::{InfoLevel, LevelFilter, Verbosity};
use diesel::result::Error as DieselErr;
use namada_core::types::address::Address as NamadaAddress;
use namada_core::types::storage::Epoch as NamadaEpoch;
use score_extractor::context::{
    CometBftUrl, Context, DatabaseUrl, Epochs, GenesisTime, UpgradeProposer,
};
use score_extractor::db;
use score_extractor::last_state;
use score_extractor::players;
use score_extractor::scores;
use score_extractor::tasks;
use score_extractor::transactions;
use shared::player::PlayerId;
use tokio::signal;
use tokio::sync::oneshot;
use tokio::time;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(clap::Parser)]
pub struct CmdlineArgs {
    /// URL to a Postgres database
    #[clap(long, env)]
    pub database_url: String,
    /// URL to a CometBFT node
    #[clap(long, env)]
    pub cometbft_url: String,
    /// Time when the Namada chain started
    #[clap(long, env)]
    pub namada_genesis_time: Option<chrono::NaiveDateTime>,
    /// Node proposing the upgrade from v0 to v1, and v1 to v2
    #[clap(long, env)]
    pub upgrade_proposer: NamadaAddress,
    /// Epoch when the upgrade from v0 to v1 happens
    #[clap(long, env)]
    pub v0_to_v1_upgrade_epoch: NamadaEpoch,
    /// Epoch when the upgrade from v1 to v2 happens
    #[clap(long, env)]
    pub v1_to_v2_upgrade_epoch: NamadaEpoch,
    /// Sleep duration between score computations
    #[clap(long, env, value_parser = parse_dur)]
    pub sleep_duration: time::Duration,
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,
}

const VERSION_STRING: &str = env!("VERGEN_GIT_SHA");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let CmdlineArgs {
        database_url,
        cometbft_url,
        namada_genesis_time,
        upgrade_proposer,
        v0_to_v1_upgrade_epoch: v0_to_v1,
        v1_to_v2_upgrade_epoch: v1_to_v2,
        verbosity,
        sleep_duration,
    } = CmdlineArgs::parse();

    let log_level = match verbosity.log_level_filter() {
        LevelFilter::Off => None,
        LevelFilter::Error => Some(Level::ERROR),
        LevelFilter::Warn => Some(Level::WARN),
        LevelFilter::Info => Some(Level::INFO),
        LevelFilter::Debug => Some(Level::DEBUG),
        LevelFilter::Trace => Some(Level::TRACE),
    };
    if let Some(log_level) = log_level {
        let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
        tracing::subscriber::set_global_default(subscriber)
            .context("setting default subscriber failed")?;
    }

    tracing::info!(version = %VERSION_STRING, "Starting score extractor");

    let context = Context::new(
        Epochs { v0_to_v1, v1_to_v2 },
        UpgradeProposer(upgrade_proposer),
        GenesisTime(namada_genesis_time),
        DatabaseUrl(database_url),
        CometBftUrl(cometbft_url),
    )
    .await?;

    let mut interval = {
        let mut ticker = time::interval(sleep_duration);
        ticker.tick().await; // skip first tick
        ticker
    };
    let mut ctrl_c = ctrl_c_receiver();

    update_database(&context).await;
    loop {
        tokio::select! {
            _ = &mut ctrl_c => {
                tracing::info!("Interrupt signal received, exiting");
                break Ok(());
            }
            _ = sleep(sleep_duration, &mut interval) => {
                update_database(&context).await;
            }
        }
    }
}

async fn update_database(context: &Context) {
    tracing::info!("Checking for new database updates");
    if let Err(err) = update_player_tasks(context).await {
        tracing::error!(reason = ?err, "Failed to update player tasks");
    }
    if let Err(err) = update_scores(context).await {
        tracing::error!(reason = ?err, "Failed to update player scores");
    }
    if let Err(err) = update_rankings(context).await {
        tracing::error!(reason = ?err, "Failed to update player rankings");
    }
    tracing::info!("All database updates concluded");
}

async fn sleep(dur: time::Duration, interval: &mut time::Interval) {
    tracing::debug!(idle_duration = ?dur, "Idling");
    interval.tick().await;
}

fn ctrl_c_receiver() -> oneshot::Receiver<()> {
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Error receiving interrupt signal");
        tx.send(()).expect("Error transmitting interrupt signal");
    });
    rx
}

async fn update_scores(context: &Context) -> anyhow::Result<()> {
    tracing::info!("Recomputing scores in the database");
    let cloned_cx = context.clone();
    context
        .db_connection_pool()
        .with(|conn| {
            conn.build_transaction()
                .read_write()
                .run(|conn| scores::recompute_task_scores(conn, cloned_cx))
        })
        .await??;
    Ok(())
}

async fn update_player_tasks(context: &Context) -> anyhow::Result<()> {
    tracing::info!("Attempting to process new tasks");
    process_new_tasks(context)
        .await
        .context("Failed to process new tasks")?;
    Ok(())
}

async fn process_new_tasks(context: &Context) -> anyhow::Result<()> {
    let cloned_cx = context.clone();
    context
        .db_connection_pool()
        .with(move |conn| {
            conn.build_transaction()
                .read_write()
                .run(|transaction_conn| {
                    let cx = cloned_cx;

                    let new_block_height = process_new_transactions(transaction_conn, &cx)
                        .map_err(|err| {
                            tracing::error!(?err, "Database error");
                            DieselErr::RollbackTransaction
                        })?;

                    process_non_tx_tasks(transaction_conn, &cx).map_err(|err| {
                        tracing::error!(?err, "Database error");
                        DieselErr::RollbackTransaction
                    })?;

                    if let Some(block) = new_block_height {
                        last_state::update_last_processed_tasks_block(transaction_conn, block)
                            .map_err(|err| {
                                tracing::error!(?err, "Database error");
                                DieselErr::RollbackTransaction
                            })?;
                    }
                    Ok::<_, DieselErr>(())
                })
        })
        .await??;

    Ok(())
}

fn process_new_transactions(
    conn: &mut db::Connection,
    cx: &Context,
) -> anyhow::Result<Option<i32>> {
    tracing::info!("Processing new transactions");

    transactions::process_last_transactions::<_, PlayerId>(conn, |process_conn, transaction| {
        tasks::update_task_statuses(
            process_conn,
            tasks::TaskInput::Transaction {
                tx: transaction,
                cx,
            },
        )
    })
    .context("Failed to process last transactions")
}

fn process_non_tx_tasks(conn: &mut db::Connection, cx: &Context) -> anyhow::Result<()> {
    tracing::info!("Processing non-transaction tasks");

    tasks::update_task_statuses(conn, tasks::TaskInput::Pilot { cx })
        .context("Failed to process pilot tasks")?;

    tasks::update_task_statuses(conn, tasks::TaskInput::SpecialTasks)
        .context("Failed to process special tasks")?;

    Ok(())
}

fn parse_dur(dur: &str) -> anyhow::Result<time::Duration> {
    duration_str::parse_std(dur).context("Failed to parse duration string")
}

async fn update_rankings(context: &Context) -> anyhow::Result<()> {
    tracing::info!("Recomputing player rankings in the database");
    context
        .db_connection_pool()
        .with(|conn| {
            conn.build_transaction()
                .read_write()
                .run(players::update_rankings)
        })
        .await??;
    Ok(())
}
