use std::env;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

const DB_ENV_VAR: &str = "GTD_TUI_DB_PATH";

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
