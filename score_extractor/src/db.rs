use std::env;

use anyhow::{anyhow, Context};
use deadpool_diesel::postgres::Object as ConnectionManager;
use deadpool_diesel::postgres::Pool as DbPool;
pub use diesel::pg::PgConnection as Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[derive(Clone)]
pub struct Pool {
    db: DbPool,
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../orm/migrations/");

impl Pool {
    pub async fn new(db_url: String) -> anyhow::Result<Self> {
        let max_pool_size = env::var("DATABASE_POOL_SIZE")
            .unwrap_or_else(|_| 8.to_string())
            .parse::<usize>()
            .unwrap_or(8_usize);
        let pool_manager = deadpool_diesel::Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);

        let db = DbPool::builder(pool_manager)
            .max_size(max_pool_size)
            .build()
            .context("Failed to build Postgres connection pool")?;

        let db_connection_pool = Self { db };
        db_connection_pool.with(run_pending_migrations).await??;

        Ok(db_connection_pool)
    }

    async fn db_connection(&self) -> anyhow::Result<ConnectionManager> {
        self.db
            .get()
            .await
            .context("Failed to get connection to database from pool")
    }

    pub async fn with<O, R>(&self, op: O) -> anyhow::Result<R>
    where
        O: FnOnce(&mut Connection) -> R + Send + 'static,
        R: Send + 'static,
    {
        let conn_manager = self.db_connection().await?;
        let op_result = conn_manager
            .interact(op)
            .await
            .map_err(|_| anyhow!("Failed to interact with database connection manager"))?;
        Ok(op_result)
    }
}

fn run_pending_migrations(conn: &mut Connection) -> anyhow::Result<()> {
    _ = conn
        .run_pending_migrations(MIGRATIONS)
        .map_err(|err| anyhow!("Failed to run pending database migrations: {err}"))?;
    Ok(())
}
