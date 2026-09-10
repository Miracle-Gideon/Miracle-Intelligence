use anyhow::Result;

use crate::cli::ConfigAction;
use crate::settings::AppConfig;

pub async fn handle(config: &AppConfig, action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => {
            println!("db_path: {}", config.db_path);
            match &config.shodan_api_key {
                Some(k) => println!("shodan_api_key: {}", mask(k)),
                None => println!("shodan_api_key: (not set — see `mi config path`)"),
            }
        }
        ConfigAction::Path => {
            println!("Configuration is resolved in this order (later overrides earlier):");
            println!("  1. built-in defaults");
            println!("  2. ./mi.toml, if present");
            println!("  3. environment variables prefixed MI_ (e.g. MI_SHODAN_API_KEY, MI_DB_PATH)");
        }
    }
    Ok(())
}

fn mask(key: &str) -> String {
    if key.len() <= 4 {
        "*".repeat(key.len())
    } else {
        format!("{}{}", "*".repeat(key.len() - 4), &key[key.len() - 4..])
    }
}
