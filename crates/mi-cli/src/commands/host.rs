use anyhow::{Context, Result};
use mi_core::ScopeGuard;
use mi_models::{Asset, AssetKind, Confidence, Observation};
use mi_providers::{shodan::ShodanProvider, Provider};
use sqlx::SqlitePool;

use crate::settings::AppConfig;

pub async fn handle(pool: &SqlitePool, config: &AppConfig, ip: String) -> Result<()> {
    let guard = ScopeGuard::new(pool);
    guard.check(&ip).await?;

    let key = config
        .shodan_api_key
        .clone()
        .context("no Shodan API key configured — set MI_SHODAN_API_KEY or add it to mi.toml")?;
    let provider = ShodanProvider::new(key);

    let result = provider
        .host_info(&ip)
        .await
        .with_context(|| format!("Shodan lookup failed for {ip}"))?;

    let asset = Asset::new(AssetKind::Ip, ip.clone(), Confidence::High);

    let normalized = serde_json::json!({
        "org": result.raw.get("org"),
        "os": result.raw.get("os"),
        "ports": result.raw.get("ports"),
        "hostnames": result.raw.get("hostnames"),
    });

    let obs = Observation::new(asset.id, "shodan", result.raw.clone(), normalized.clone());

    mi_storage::insert_asset(pool, &asset).await?;
    mi_storage::insert_observation(pool, &obs).await?;

    println!("Host: {ip}");
    println!("  org:       {}", field(&normalized["org"]));
    println!("  os:        {}", field(&normalized["os"]));
    println!("  ports:     {}", field(&normalized["ports"]));
    println!("  hostnames: {}", field(&normalized["hostnames"]));
    println!("Stored as asset {}", asset.id);

    Ok(())
}

fn field(v: &serde_json::Value) -> String {
    if v.is_null() { "unknown".to_string() } else { v.to_string() }
}
