use serde::{Deserialize, Serialize};

use crate::ui::theme::ThemeMode;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub language: Option<String>,
    pub theme_mode: Option<ThemeMode>,
    pub start_hidden: Option<bool>,
    pub web_server_enabled: Option<bool>,
}

pub const SETTINGS_FILE: &str = "settings.json";
