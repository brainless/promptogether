use sqlx::migrate::Migrator;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect(database_url)
        .await?;

    sqlx::raw_sql("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")
        .execute(&pool)
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
    let applied = applied_count(pool).await?;
    let known = MIGRATOR.migrations.len() as u64;
    if applied < known {
        return Err(sqlx::Error::Protocol(format!(
            "{known} migrations known but only {applied} applied"
        )));
    }
    Ok(())
}

async fn applied_count(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await?;
    Ok(count as u64)
}
