use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ui::language;

pub struct Storage {
    base_dir: PathBuf,
}

impl Storage {
    pub fn new() -> Self {
        let base_dir = if let Ok(dir) = std::env::var("PASTRY_STORAGE_DIR") {
            PathBuf::from(dir)
        } else {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("pastry")
        };

        fs::create_dir_all(&base_dir).ok();

        Self { base_dir }
    }

    fn get_path(&self, filename: &str) -> PathBuf {
        self.base_dir.join(filename)
    }

    pub fn load<T>(&self, filename: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de> + Default,
    {
        let path = self.get_path(filename);

        if !path.exists() {
            return Ok(T::default());
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            language::tr(language::Text::FileReadFailedFmt)
                .replace("{file}", filename)
                .replace("{err}", &e.to_string())
        })?;

        serde_json::from_str(&content).map_err(|e| {
            language::tr(language::Text::JsonParseFailedFmt)
                .replace("{file}", filename)
                .replace("{err}", &e.to_string())
        })
    }

    pub fn save<T>(&self, filename: &str, data: &T) -> Result<(), String>
    where
        T: Serialize,
    {
        let path = self.get_path(filename);

        let json = serde_json::to_string_pretty(data).map_err(|e| {
            language::tr(language::Text::SerializationFailedFmt).replace("{err}", &e.to_string())
        })?;

        fs::write(&path, json).map_err(|e| {
            language::tr(language::Text::FileWriteFailedFmt)
                .replace("{file}", filename)
                .replace("{err}", &e.to_string())
        })?;

        Ok(())
    }

    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
