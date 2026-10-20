use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteJournalMode;
use sqlx::sqlite::SqlitePoolOptions;

pub struct KvStore {
  pool: SqlitePool,
}

impl KvStore {
  pub async fn new(path: &Path) -> anyhow::Result<Self> {
    let options = SqliteConnectOptions::new()
      .filename(path)
      .create_if_missing(true)
      .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new().connect_with(options).await?;

    sqlx::query(
      "CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)",
    )
    .execute(&pool)
    .await?;

    Ok(Self { pool })
  }

  pub async fn get<D: DeserializeOwned>(
    &self,
    key: &str,
  ) -> anyhow::Result<Option<D>> {
    let value: Option<String> = sqlx::query_scalar("SELECT value FROM kv WHERE key = ?")
      .bind(key)
      .fetch_optional(&self.pool)
      .await?;

    match value {
      Some(value) => Ok(Some(serde_json::from_str(&value)?)),
      None => Ok(None),
    }
  }

  pub async fn set<S: Serialize>(
    &self,
    key: &str,
    value: &S,
  ) -> anyhow::Result<bool> {
    let value = serde_json::to_string(value)?;

    let result = sqlx::query(
      "INSERT INTO kv (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(&self.pool)
    .await?;

    Ok(result.rows_affected() > 0)
  }

  pub async fn remove(
    &self,
    key: &str,
  ) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM kv WHERE key = ?")
      .bind(key)
      .execute(&self.pool)
      .await?;

    Ok(result.rows_affected() > 0)
  }
}
