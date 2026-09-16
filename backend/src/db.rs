use std::str::FromStr;
use std::time::Duration;

use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let before = applied_count(pool).await?;
    MIGRATOR.run(pool).await.map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    let after = applied_count(pool).await?;
    Ok(after - before)
}

pub async fn check_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let migrations_table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = '_sqlx_migrations')",
    )
    .fetch_one(pool)
    .await?;

    if !migrations_table_exists {
        return Err(sqlx::Error::Protocol(
            "migration metadata table is missing".to_owned(),
        ));
    }

    let applied: Vec<(i64, bool, Vec<u8>)> = sqlx::query_as(
        "SELECT version, success, checksum FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(pool)
    .await?;
    let expected: Vec<_> = MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .collect();

    for (version, success, checksum) in &applied {
        if !success {
            return Err(sqlx::Error::Protocol(format!(
                "migration {version} is incomplete"
            )));
        }

        let Some(migration) = expected.iter().find(|migration| migration.version == *version) else {
            return Err(sqlx::Error::Protocol(format!(
                "database contains unknown migration {version}"
            )));
        };

        if migration.checksum.as_ref() != checksum {
            return Err(sqlx::Error::Protocol(format!(
                "migration {version} checksum does not match the embedded migration"
            )));
        }
    }

    if let Some(migration) = expected
        .iter()
        .find(|migration| !applied.iter().any(|(version, _, _)| *version == migration.version))
    {
        return Err(sqlx::Error::Protocol(format!(
            "migration {} has not been applied",
            migration.version
        )));
    }

    Ok(())
}

async fn applied_count(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let migrations_table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = '_sqlx_migrations')",
    )
    .fetch_one(pool)
    .await?;

    if !migrations_table_exists {
        return Ok(0);
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await?;
    Ok(count as u64)
}
