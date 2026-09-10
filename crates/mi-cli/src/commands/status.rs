use anyhow::Result;
use mi_providers::{shodan::ShodanProvider, Provider};
use sqlx::SqlitePool;

use crate::settings::AppConfig;

pub async fn handle(pool: &SqlitePool, config: &AppConfig) -> Result<()> {
    println!("MI status");
    println!("  database: {} (connected)", config.db_path);

    let asset_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM assets")
        .fetch_one(pool)
        .await?;
    let scope_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scopes")
        .fetch_one(pool)
        .await?;
    println!("  assets tracked: {asset_count}");
    println!("  scopes registered: {scope_count}");

    match &config.shodan_api_key {
        Some(key) => {
            let provider = ShodanProvider::new(key.clone());
            match provider.validate_key().await {
                Ok(info) => println!(
                    "  shodan key: valid  (plan: {}, credits: {})",
                    info.plan.unwrap_or_else(|| "unknown".into()),
                    info.credits_remaining
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".into())
                ),
                Err(e) => println!("  shodan key: invalid or unreachable — {e}"),
            }
        }
        None => println!("  shodan key: not configured (set MI_SHODAN_API_KEY or mi.toml)"),
    }

    Ok(())
}
