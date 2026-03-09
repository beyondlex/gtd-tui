use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

const DB_ENV_VAR: &str = "GTD_TUI_DB_PATH";

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ThemeConfig {
    #[serde(default)]
    pub calendar: CalendarThemeConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CalendarThemeConfig {
    pub weekday: Option<String>,
    pub weekend: Option<String>,
    pub today: Option<String>,
    pub selected: Option<String>,
}

pub fn db_path() -> Result<PathBuf> {
    if let Ok(value) = env::var(DB_ENV_VAR) {
        return Ok(PathBuf::from(value));
    }

    let home = env::var("HOME").map_err(|_| anyhow!("HOME is not set"))?;
    Ok(PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("gtd-tui")
        .join("gtd.db"))
}

pub fn config_path() -> Result<PathBuf> {
    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(dir).join("gtd-tui").join("config.toml"));
    }
    let home = env::var("HOME").map_err(|_| anyhow!("HOME is not set"))?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("gtd-tui")
        .join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("invalid config: {}", path.display()))?;
    Ok(config)
}
