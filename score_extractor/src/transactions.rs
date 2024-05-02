use anyhow::Context;
use shared::orm::schema;
use shared::orm::transaction::TransactionDb;
use shared::transaction::{RawMemo, Transaction};

use crate::db;
use crate::last_state;

/// Processes a batch of transactions and returns the next height to process.
pub fn process_last_transactions<F, M>(
    conn: &mut db::Connection,
    mut process: F,
) -> anyhow::Result<Option<i32>>
where
    F: FnMut(&mut db::Connection, Transaction<M>) -> anyhow::Result<()>,
    M: TryFrom<RawMemo, Error = anyhow::Error>,
{
    use diesel::connection::DefaultLoadingMode;
    use diesel::prelude::*;
    use schema::blocks;
    use schema::transactions;

    let Some((starting_height, mut ending_height)) =
        last_state::compute_task_heights_to_process(conn)?
    else {
        tracing::info!("No new transactions to process");
        return Ok(None);
    };

    if ending_height - starting_height > 1000 {
        ending_height = starting_height + 1000
    }

    let blocks_ids_range = blocks::table
        .filter(
            blocks::dsl::height
                .ge(starting_height)
                .and(blocks::dsl::height.le(ending_height)),
        )
        .select(blocks::dsl::id);
    let transactions_in_range = transactions::table
        .filter(transactions::dsl::block_id.eq_any(blocks_ids_range))
        .select(TransactionDb::as_select());

    let mut processed_txs_counter = 0;
    const PRINT_STEP: usize = 15;

    tracing::info!(
        starting_height,
        ending_height,
        "Processing new transactions in the given block range"
    );

    transactions_in_range
        .load_iter::<TransactionDb, DefaultLoadingMode>(conn)
        .context("Failed to fetch transactions from the database")?
        .try_for_each(|transaction| {
            let transaction: Transaction<RawMemo> = transaction
                .context("Failed to deserialize transaction from database")?
                .into();
            let transaction: Transaction<M> = transaction.try_parse_memo()?;
            let result = process(conn, transaction);
            processed_txs_counter += 1;
            if processed_txs_counter % PRINT_STEP == 0 {
                tracing::info!(
                    starting_height,
                    ending_height,
                    processed_tx_count = processed_txs_counter,
                    "Still processing transactions"
                );
            }
            result
        })?;

    tracing::info!(
        starting_height,
        ending_height,
        "Finished processing all transactions in the given block range"
    );

    Ok(Some(ending_height))
}
