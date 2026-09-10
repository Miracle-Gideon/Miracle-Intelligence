//! mi-storage: SQLite persistence + repository pattern.
//!
//! Domain types (mi-models) never derive sqlx traits directly — this crate
//! owns the mapping between DB rows and domain structs, so mi-models stays
//! free of any storage-layer dependency.

use chrono::{DateTime, Utc};
use mi_models::{Asset, AssetKind, Confidence, Observation, Scope, ScopeCategory, ScopeStatus};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::FromRow;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("invalid data in row: {0}")]
    InvalidRow(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Opens (creating if needed) the SQLite DB at `path` and runs pending migrations.
pub async fn init_db(path: &str) -> Result<SqlitePool> {
    let url = format!("sqlite://{path}?mode=rwc");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

// ---------------------------------------------------------------------
// Assets
// ---------------------------------------------------------------------

#[derive(FromRow)]
struct AssetRow {
    id: String,
    kind: String,
    value: String,
    org_id: Option<String>,
    confidence: String,
    first_seen: String,
    last_seen: String,
}

impl AssetRow {
    fn into_domain(self) -> Result<Asset> {
        Ok(Asset {
            id: Uuid::parse_str(&self.id)
                .map_err(|e| StorageError::InvalidRow(format!("asset id: {e}")))?,
            kind: AssetKind::parse(&self.kind)
                .ok_or_else(|| StorageError::InvalidRow(format!("asset kind: {}", self.kind)))?,
            value: self.value,
            org_id: self
                .org_id
                .map(|s| Uuid::parse_str(&s))
                .transpose()
                .map_err(|e| StorageError::InvalidRow(format!("org_id: {e}")))?,
            first_seen: parse_dt(&self.first_seen)?,
            last_seen: parse_dt(&self.last_seen)?,
            confidence: Confidence::parse(&self.confidence).ok_or_else(|| {
                StorageError::InvalidRow(format!("confidence: {}", self.confidence))
            })?,
        })
    }
}

pub async fn insert_asset(pool: &SqlitePool, asset: &Asset) -> Result<()> {
    sqlx::query(
        "INSERT INTO assets (id, kind, value, org_id, confidence, first_seen, last_seen)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(asset.id.to_string())
    .bind(asset.kind.as_str())
    .bind(&asset.value)
    .bind(asset.org_id.map(|u| u.to_string()))
    .bind(asset.confidence.as_str())
    .bind(asset.first_seen.to_rfc3339())
    .bind(asset.last_seen.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_asset(pool: &SqlitePool, id: Uuid) -> Result<Option<Asset>> {
    let row: Option<AssetRow> = sqlx::query_as("SELECT * FROM assets WHERE id = ?")
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?;
    row.map(|r| r.into_domain()).transpose()
}

pub async fn list_assets(pool: &SqlitePool, kind: Option<AssetKind>) -> Result<Vec<Asset>> {
    let rows: Vec<AssetRow> = match kind {
        Some(k) => {
            sqlx::query_as("SELECT * FROM assets WHERE kind = ? ORDER BY last_seen DESC")
                .bind(k.as_str())
                .fetch_all(pool)
                .await?
        }
        None => {
            sqlx::query_as("SELECT * FROM assets ORDER BY last_seen DESC")
                .fetch_all(pool)
                .await?
        }
    };
    rows.into_iter().map(|r| r.into_domain()).collect()
}

// ---------------------------------------------------------------------
// Observations
// ---------------------------------------------------------------------

pub async fn insert_observation(pool: &SqlitePool, obs: &Observation) -> Result<()> {
    sqlx::query(
        "INSERT INTO observations (id, asset_id, provider, raw, normalized, observed_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(obs.id.to_string())
    .bind(obs.asset_id.to_string())
    .bind(&obs.provider)
    .bind(obs.raw.to_string())
    .bind(obs.normalized.to_string())
    .bind(obs.observed_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------
// Scopes
// ---------------------------------------------------------------------

#[derive(FromRow)]
struct ScopeRow {
    id: String,
    pattern: String,
    category: String,
    status: String,
    authorized_by: String,
    authorized_until: Option<String>,
    created_at: String,
}

impl ScopeRow {
    fn into_domain(self) -> Result<Scope> {
        Ok(Scope {
            id: Uuid::parse_str(&self.id)
                .map_err(|e| StorageError::InvalidRow(format!("scope id: {e}")))?,
            pattern: self.pattern,
            category: ScopeCategory::parse(&self.category).ok_or_else(|| {
                StorageError::InvalidRow(format!("scope category: {}", self.category))
            })?,
            status: ScopeStatus::parse(&self.status)
                .ok_or_else(|| StorageError::InvalidRow(format!("scope status: {}", self.status)))?,
            authorized_by: self.authorized_by,
            authorized_until: self.authorized_until.map(|s| parse_dt(&s)).transpose()?,
            created_at: parse_dt(&self.created_at)?,
        })
    }
}

pub async fn insert_scope(pool: &SqlitePool, scope: &Scope) -> Result<()> {
    sqlx::query(
        "INSERT INTO scopes (id, pattern, category, status, authorized_by, authorized_until, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(scope.id.to_string())
    .bind(&scope.pattern)
    .bind(scope.category.as_str())
    .bind(scope.status.as_str())
    .bind(&scope.authorized_by)
    .bind(scope.authorized_until.map(|d| d.to_rfc3339()))
    .bind(scope.created_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_scopes(pool: &SqlitePool) -> Result<Vec<Scope>> {
    let rows: Vec<ScopeRow> = sqlx::query_as("SELECT * FROM scopes ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    rows.into_iter().map(|r| r.into_domain()).collect()
}

pub async fn remove_scope(pool: &SqlitePool, id: Uuid) -> Result<bool> {
    let result = sqlx::query("DELETE FROM scopes WHERE id = ?")
        .bind(id.to_string())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

fn parse_dt(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| StorageError::InvalidRow(format!("timestamp '{s}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mi_models::{AssetKind, Confidence, ScopeCategory};

    async fn test_pool() -> SqlitePool {
        // In-memory DB, fresh per test — migration round-trip check.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn asset_round_trips() {
        let pool = test_pool().await;
        let asset = Asset::new(AssetKind::Domain, "example.com", Confidence::High);
        insert_asset(&pool, &asset).await.unwrap();

        let fetched = get_asset(&pool, asset.id).await.unwrap().unwrap();
        assert_eq!(fetched.value, "example.com");
        assert_eq!(fetched.kind, AssetKind::Domain);
    }

    #[tokio::test]
    async fn scope_round_trips_and_deletes() {
        let pool = test_pool().await;
        let scope = Scope::new("example.com", ScopeCategory::Domain, "test-suite", None);
        insert_scope(&pool, &scope).await.unwrap();

        let scopes = list_scopes(&pool).await.unwrap();
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].pattern, "example.com");

        let deleted = remove_scope(&pool, scope.id).await.unwrap();
        assert!(deleted);
        assert_eq!(list_scopes(&pool).await.unwrap().len(), 0);
    }
}
