use anyhow::{anyhow, Context};
use shared::orm::players::PlayerKindDb;
use shared::orm::schema;
use shared::orm::tasks::TaskTypeDb;
pub use shared::player::PlayerId;

use crate::db;

// NB: numbers pulled from csv files
pub const NUMBER_PILOTS: usize = 10470;
pub const NUMBER_CREW_MEMBERS: usize = 129238;

pub struct PilotValidatorAddress(pub String);

pub fn process_all_pilots_with_incomplete_tasks<F>(
    conn: &mut db::Connection,
    task_type: TaskTypeDb,
    mut process: F,
) -> anyhow::Result<()>
where
    F: FnMut(&mut db::Connection, PlayerId) -> anyhow::Result<()>,
{
    use diesel::connection::DefaultLoadingMode;
    use diesel::dsl::not;
    use diesel::prelude::*;
    use schema::players::dsl::*;
    use schema::tasks;

    let players_who_completed_task = tasks::table
        .filter(tasks::dsl::task.eq(task_type))
        .select(tasks::dsl::player_id);

    players
        .filter(
            kind.eq(PlayerKindDb::Pilot)
                .and(score.gt(0))
                .and(not(id.eq_any(players_who_completed_task)))
                .and(is_banned.ne(true)),
        )
        .select(id)
        .load_iter::<String, DefaultLoadingMode>(conn)
        .context("Failed to fetch pilot data from database")?
        .try_for_each(|database_response| {
            let player_id =
                database_response.context("Failed to deserialize pilot from database")?;
            process(conn, PlayerId(player_id))
        })?;

    Ok(())
}

pub fn process_all_pilots_with_nonnull_validator_addr<F>(
    conn: &mut db::Connection,
    mut process: F,
) -> anyhow::Result<()>
where
    F: FnMut(&mut db::Connection, PlayerId, PilotValidatorAddress) -> anyhow::Result<()>,
{
    use diesel::connection::DefaultLoadingMode;
    use diesel::prelude::*;
    use schema::players::dsl::*;

    players
        .filter(
            namada_validator_address
                .is_not_null()
                .and(kind.eq(PlayerKindDb::Pilot))
                .and(is_banned.ne(true)),
        )
        .select((id, namada_validator_address))
        .load_iter::<(String, Option<String>), DefaultLoadingMode>(conn)
        .context("Failed to fetch pilot data from database")?
        .try_for_each(|database_response| {
            let (player_id, validator_addr) =
                database_response.context("Failed to deserialize pilot from database")?;
            let validator_addr = validator_addr
                .ok_or_else(|| anyhow!("Validator address must be present given the SQL query"))?;
            process(
                conn,
                PlayerId(player_id),
                PilotValidatorAddress(validator_addr),
            )
        })?;

    Ok(())
}

pub fn process_all_pilots<F>(conn: &mut db::Connection, mut process: F) -> anyhow::Result<()>
where
    F: FnMut(&mut db::Connection, PlayerId) -> anyhow::Result<()>,
{
    use diesel::connection::DefaultLoadingMode;
    use diesel::prelude::*;
    use schema::players::dsl::*;

    players
        .filter(kind.eq(PlayerKindDb::Pilot).and(is_banned.ne(true)))
        .select(id)
        .load_iter::<String, DefaultLoadingMode>(conn)
        .context("Failed to fetch pilot data from database")?
        .try_for_each(|database_response| {
            let player_id =
                database_response.context("Failed to deserialize pilot from database")?;
            process(conn, PlayerId(player_id))
        })?;

    Ok(())
}

pub fn player_exists(conn: &mut db::Connection, player_id: &str) -> anyhow::Result<bool> {
    use diesel::dsl::exists;
    use diesel::prelude::*;
    use schema::players::dsl::*;

    diesel::select(exists(players.filter(id.eq(player_id))))
        .get_result::<bool>(conn)
        .with_context(|| format!("Failed to check if player with id {player_id} exists"))
}

fn reset_rankings(conn: &mut db::Connection) -> anyhow::Result<()> {
    use diesel::prelude::*;
    use schema::player_ranks;

    diesel::delete(player_ranks::table)
        .execute(conn)
        .context("Failed to delete existing player rankings")?;

    // NB: this is necessary to avoid overflowing the id
    // counter of the `player_ranks` table
    diesel::sql_query(
        r#"
        ALTER SEQUENCE player_ranks_id_seq RESTART WITH 1
        "#,
    )
    .execute(conn)
    .context("Failed to reset the id counter of player rankings")?;

    Ok(())
}

fn set_rankings_for_player_kind(
    conn: &mut db::Connection,
    player_kind: PlayerKindDb,
) -> anyhow::Result<()> {
    use diesel::prelude::*;

    let affected_rows = diesel::sql_query(
        r#"
        INSERT INTO player_ranks ( ranking, player_id )
        SELECT
          ROW_NUMBER() OVER (ORDER BY score DESC, internal_id)
            AS rank_index,
          id
            AS player_id
        FROM players WHERE players.id in
            (SELECT id FROM players WHERE kind = $1)
        "#,
    )
    .bind::<schema::sql_types::PlayerKind, _>(&player_kind)
    .execute(conn)
    .with_context(|| format!("Failed to set new player rankings for {player_kind}"))?;

    tracing::info!(
        %player_kind,
        no_of_players = affected_rows,
        "Updated player rankings in the database"
    );

    Ok(())
}

pub fn update_rankings(conn: &mut db::Connection) -> anyhow::Result<()> {
    reset_rankings(conn).context("Failed to reset player rankings")?;

    set_rankings_for_player_kind(conn, PlayerKindDb::Pilot)
        .context("Failed to update pilot rankings")?;
    set_rankings_for_player_kind(conn, PlayerKindDb::Crew)
        .context("Failed to update crew rankings")?;

    Ok(())
}
