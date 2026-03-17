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
    #[serde(default)]
    pub keys: KeysConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ThemeConfig {
    #[serde(default)]
    pub calendar: CalendarThemeConfig,
    #[serde(default)]
    pub editor: EditorThemeConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CalendarThemeConfig {
    pub weekday: Option<String>,
    pub weekend: Option<String>,
    pub today: Option<String>,
    pub selected: Option<String>,
    pub bracket: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct EditorThemeConfig {
    pub checklist_edit: Option<String>,
    pub task_selected: Option<String>,
    pub date_label: Option<String>,
    pub checklist_item_selected: Option<String>,
    pub field_title: Option<String>,
    pub field_notes: Option<String>,
    pub field_due: Option<String>,
    pub field_checklist: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct KeysConfig {
    pub quit: Option<String>,
    pub view_inbox: Option<String>,
    pub view_today: Option<String>,
    pub view_upcoming: Option<String>,
    pub view_anytime: Option<String>,
    pub view_someday: Option<String>,
    pub select_next: Option<String>,
    pub select_prev: Option<String>,
    pub select_first: Option<String>,
    pub select_last: Option<String>,
    pub new_task: Option<String>,
    pub edit_task: Option<String>,
    pub edit_title: Option<String>,
    pub toggle_task: Option<String>,
    pub refresh: Option<String>,
    pub save_edit: Option<String>,
    pub cancel_edit: Option<String>,
    pub next_focus: Option<String>,
    pub prev_focus: Option<String>,
    pub checklist_edit_toggle: Option<String>,
    pub date_prev_day: Option<String>,
    pub date_next_day: Option<String>,
    pub date_prev_week: Option<String>,
    pub date_next_week: Option<String>,
    pub date_edit_mode: Option<String>,
    pub date_prev_month: Option<String>,
    pub date_next_month: Option<String>,
    pub date_today: Option<String>,
    pub date_tomorrow: Option<String>,
    pub checklist_toggle: Option<String>,
    pub checklist_add: Option<String>,
    pub checklist_next: Option<String>,
    pub checklist_prev: Option<String>,
    pub new_item_above: Option<String>,
    pub new_item_below: Option<String>,
    pub move_item_up: Option<String>,
    pub move_item_down: Option<String>,
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
    let config: Config =
        toml::from_str(&content).with_context(|| format!("invalid config: {}", path.display()))?;
    Ok(config)
}
