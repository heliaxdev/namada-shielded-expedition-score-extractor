use anyhow::Context;
use shared::orm::schema;
use shared::orm::task_completion_state::TaskCompletionStateInsertDb;

use crate::db;

pub fn compute_task_heights_to_process(
    conn: &mut db::Connection,
) -> anyhow::Result<Option<(i32, i32)>> {
    let (our_height, crawler_height) = read_last_processed_task_heights(conn)?;
    if our_height != crawler_height {
        let starting_height = our_height + 1;
        let ending_height = crawler_height;
        tracing::debug!(
            starting_height,
            ending_height,
            "Processing new transactions over the specified block range"
        );
        Ok(Some((starting_height, ending_height)))
    } else {
        tracing::debug!("No new transactions to process");
        Ok(None)
    }
}

pub fn update_last_processed_tasks_block(
    conn: &mut db::Connection,
    last_block_height: i32,
) -> anyhow::Result<()> {
    use chrono::offset::Utc;
    use diesel::prelude::*;
    use schema::task_completion_state::dsl::*;

    let ts = Utc::now().naive_utc();

    diesel::insert_into(task_completion_state)
        .values(&TaskCompletionStateInsertDb {
            id: 0,
            last_processed_time: ts,
            last_processed_height: last_block_height,
        })
        .on_conflict(id)
        .do_update()
        .set((
            last_processed_height.eq(last_block_height),
            last_processed_time.eq(ts),
        ))
        .execute(conn)
        .context("Failed to update last processed block")?;

    tracing::debug!(
        last_block_height,
        "Updated last procesed block on tasks table"
    );

    Ok(())
}

fn read_last_processed_task_heights(conn: &mut db::Connection) -> anyhow::Result<(i32, i32)> {
    use diesel::dsl::max;
    use diesel::prelude::*;
    use schema::crawler_state::dsl::*;
    use schema::task_completion_state::dsl::*;

    let our_height = task_completion_state
        .select(last_processed_height)
        .first::<i32>(conn)
        .optional()
        .context("Failed to query last processed task completion height")?
        .unwrap_or(0);
    let crawler_height = crawler_state
        .select(max(height))
        .first::<Option<i32>>(conn)
        .context("Failed to query last processed crawler height")?
        .unwrap_or(0);

    tracing::debug!(
        our_height,
        crawler_height,
        "Read last processed block heights"
    );

    Ok((our_height, crawler_height))
}
