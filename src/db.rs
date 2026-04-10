use std::{env, str::FromStr};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("DATABASE_URL is not set")]
    MissingDatabaseUrl,
    #[error("invalid sqlite connection options: {0}")]
    InvalidOptions(String),
    #[error("failed to connect to sqlite")]
    Connect(#[source] sqlx::Error),
    #[error("failed to run sqlite migrations")]
    Migrate(#[source] sqlx::migrate::MigrateError),
}

pub async fn connect_and_migrate_from_env() -> Result<SqlitePool, DbError> {
    let database_url = env::var("DATABASE_URL").map_err(|_| DbError::MissingDatabaseUrl)?;
    let pool = connect(&database_url).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

pub async fn connect(database_url: &str) -> Result<SqlitePool, DbError> {
    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(|err| DbError::InvalidOptions(err.to_string()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        .map_err(DbError::Connect)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(DbError::Migrate)
}
