use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_db_path")]
    pub db_path: String,

    /// BYOK: no default is ever bundled. Set via mi.toml or MI_SHODAN_API_KEY.
    /// The full encrypted multi-key pool is a Phase 2 addition — this is a
    /// single key read straight from config/env to keep Phase 1 small.
    pub shodan_api_key: Option<String>,
}

fn default_db_path() -> String {
    "mi.db".to_string()
}

pub fn load_config(path_override: Option<&str>) -> anyhow::Result<AppConfig> {
    let mut builder = Config::builder().set_default("db_path", "mi.db")?;

    let file_name = path_override.unwrap_or("mi");
    builder = builder.add_source(File::with_name(file_name).required(false));
    builder = builder.add_source(Environment::with_prefix("MI").separator("_"));

    let cfg = builder.build()?;
    Ok(cfg.try_deserialize()?)
}
