use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection, SqliteJournalMode};
use sqlx::{Connection, Row};
use std::path::Path;

pub async fn open(path: &Path) -> Result<SqliteConnection, Box<dyn std::error::Error>> {
    if !path.is_file() {
        return Err(format!("concept database missing at {}", path.display()).into());
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        // Keep committed content in the database file itself, not a WAL sidecar.
        .journal_mode(SqliteJournalMode::Delete)
        .foreign_keys(true);
    Ok(SqliteConnection::connect_with(&options).await?)
}

pub async fn existing_slug(
    db: &mut SqliteConnection,
    slug: &str,
    name: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query("SELECT slug FROM concepts WHERE slug = ? OR lower(name) = lower(?)")
        .bind(slug)
        .bind(name)
        .fetch_optional(db)
        .await?;
    Ok(row.map(|row| row.get("slug")))
}

pub async fn save(
    db: &mut SqliteConnection,
    slug: &str,
    name: &str,
    definition: &str,
    post_slugs: &[String],
    overwrite: bool,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    if overwrite {
        sqlx::query("UPDATE concepts SET definition = ? WHERE slug = ?")
            .bind(definition)
            .bind(slug)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM concept_posts WHERE concept_slug = ?")
            .bind(slug)
            .execute(&mut *tx)
            .await?;
    } else {
        sqlx::query("INSERT INTO concepts (slug, name, definition) VALUES (?, ?, ?)")
            .bind(slug)
            .bind(name)
            .bind(definition)
            .execute(&mut *tx)
            .await?;
    }
    for post_slug in post_slugs {
        sqlx::query("INSERT INTO concept_posts (concept_slug, post_slug) VALUES (?, ?)")
            .bind(slug)
            .bind(post_slug)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}
