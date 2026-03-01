use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Local};
use include_dir::{Dir, include_dir};
use rquickjs::{Coerced, Context, Function, Object, Runtime, function::Rest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::services::storage::Storage;
use crate::ui::language;

static BUILTIN_SCRIPTS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/scripts");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub code: String,
    pub created_at: DateTime<Local>,
}

impl Script {
    pub fn new(name: String, code: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            code,
            created_at: Local::now(),
        }
    }
}

impl Hash for Script {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.code.hash(state);
        self.created_at.hash(state);
    }
}

const SCRIPTS_DIR: &str = "scripts";
const SCRIPTS_META_FILE: &str = "scripts_meta.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScriptMeta {
    id: String,
    name: String,
    created_at: DateTime<Local>,
}

pub struct ScriptStorage {
    storage: Storage,
    scripts_dir: PathBuf,
}

impl ScriptStorage {
    pub fn new() -> Self {
        let storage = Storage::new();
        let scripts_dir = storage.base_dir().join(SCRIPTS_DIR);

        std::fs::create_dir_all(&scripts_dir).ok();

        Self {
            storage,
            scripts_dir,
        }
    }

    pub fn load(&self) -> Vec<Script> {
        let mut scripts = self.load_from_files();

        if scripts.is_empty() {
            scripts = self.load_builtin_scripts();
            for script in &scripts {
                self.save_script_file(script).ok();
            }
            self.save_meta(&scripts).ok();
        }

        scripts
    }

    fn load_from_files(&self) -> Vec<Script> {
        let meta_map: std::collections::HashMap<String, ScriptMeta> =
            if let Ok(metas) = self.storage.load::<Vec<ScriptMeta>>(SCRIPTS_META_FILE) {
                metas.into_iter().map(|m| (m.id.clone(), m)).collect()
            } else {
                std::collections::HashMap::new()
            };

        let mut scripts = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("js") {
                    continue;
                }

                let Ok(code) = std::fs::read_to_string(&path) else {
                    continue;
                };

                let file_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                let (id, name, created_at) = if let Some(meta) = meta_map.get(file_name) {
                    (meta.id.clone(), meta.name.clone(), meta.created_at)
                } else {
                    (
                        file_name.to_string(),
                        display_name_from_id(file_name),
                        Local::now(),
                    )
                };

                scripts.push(Script {
                    id,
                    name,
                    code,
                    created_at,
                });
            }
        }

        scripts.sort_by(|a, b| a.name.cmp(&b.name));
        scripts
    }

    fn load_builtin_scripts(&self) -> Vec<Script> {
        let mut scripts = Vec::new();

        for entry in BUILTIN_SCRIPTS_DIR.files() {
            let Some(name_str) = entry.path().file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name_str.ends_with(".js") {
                continue;
            }
            let Some(code) = entry.contents_utf8() else {
                continue;
            };

            let id = name_str.trim_end_matches(".js").to_string();
            scripts.push(Script {
                id: id.clone(),
                name: display_name_from_id(&id),
                code: code.to_string(),
                created_at: Local::now(),
            });
        }

        scripts.sort_by(|a, b| a.name.cmp(&b.name));
        scripts
    }

    fn save_script_file(&self, script: &Script) -> Result<(), String> {
        let file_path = self.scripts_dir.join(format!("{}.js", script.id));
        std::fs::write(&file_path, &script.code).map_err(|e| {
            language::tr(language::Text::ScriptWriteFailedFmt).replace("{}", &e.to_string())
        })
    }

    fn save_meta(&self, scripts: &[Script]) -> Result<(), String> {
        let metas: Vec<ScriptMeta> = scripts
            .iter()
            .map(|s| ScriptMeta {
                id: s.id.clone(),
                name: s.name.clone(),
                created_at: s.created_at,
            })
            .collect();

        self.storage.save(SCRIPTS_META_FILE, &metas)
    }

    pub fn add(&self, script: Script) -> Result<(), String> {
        self.save_script_file(&script)?;

        let mut scripts = self.load();
        scripts.push(script);
        self.save_meta(&scripts)
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let file_path = self.scripts_dir.join(format!("{}.js", id));
        if file_path.exists() {
            std::fs::remove_file(&file_path).map_err(|e| {
                language::tr(language::Text::ScriptDeleteFailedFmt).replace("{}", &e.to_string())
            })?;
        }

        let mut scripts = self.load();
        scripts.retain(|s| s.id != id);
        self.save_meta(&scripts)
    }

    pub fn update(&self, id: &str, name: String, code: String) -> Result<(), String> {
        let file_path = self.scripts_dir.join(format!("{}.js", id));
        std::fs::write(&file_path, &code).map_err(|e| {
            language::tr(language::Text::ScriptUpdateFailedFmt).replace("{}", &e.to_string())
        })?;

        let mut scripts = self.load();
        if let Some(script) = scripts.iter_mut().find(|s| s.id == id) {
            script.name = name;
            script.code = code;
            self.save_meta(&scripts)
        } else {
            Err(language::tr(language::Text::ScriptNotFound).to_string())
        }
    }
}

pub(crate) fn execute_script_blocking(code: &str, input: &str) -> Result<String, String> {
    let runtime = Runtime::new().map_err(|e| e.to_string())?;
    let context = Context::full(&runtime).map_err(|e| e.to_string())?;

    context.with(|ctx| -> Result<String, String> {
        let global = ctx.globals();

        // Input
        global
            .set("input", input.to_string())
            .map_err(|e| e.to_string())?;
        global.set("output", "").map_err(|e| e.to_string())?;

        // Console.log capturing
        let output = Arc::new(Mutex::new(String::new()));
        let output_clone = output.clone();

        let log_cb = Function::new(ctx.clone(), move |args: Rest<Coerced<String>>| {
            let msg = args
                .iter()
                .map(|v| v.0.clone())
                .collect::<Vec<_>>()
                .join(" ");

            if let Ok(mut out) = output_clone.lock() {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&msg);
            }
        })
        .map_err(|e| e.to_string())?;

        let console = Object::new(ctx.clone()).map_err(|e| e.to_string())?;
        console.set("log", log_cb).map_err(|e| e.to_string())?;
        global.set("console", console).map_err(|e| e.to_string())?;

        // Helpers
        // md5
        global
            .set(
                "md5",
                Function::new(ctx.clone(), |s: String| {
                    format!("{:x}", md5::compute(s.as_bytes()))
                }),
            )
            .unwrap();

        // sha256
        global
            .set(
                "sha256",
                Function::new(ctx.clone(), |s: String| {
                    let mut hasher = Sha256::new();
                    hasher.update(s.as_bytes());
                    hex::encode(hasher.finalize())
                }),
            )
            .unwrap();

        // base64
        global
            .set(
                "base64_encode",
                Function::new(ctx.clone(), |s: String| general_purpose::STANDARD.encode(s)),
            )
            .unwrap();

        global
            .set(
                "base64_decode",
                Function::new(ctx.clone(), |s: String| {
                    String::from_utf8(general_purpose::STANDARD.decode(s).unwrap_or_default())
                        .unwrap_or_default()
                }),
            )
            .unwrap();

        // uuid
        global
            .set(
                "uuid",
                Function::new(ctx.clone(), || uuid::Uuid::new_v4().to_string()),
            )
            .unwrap();

        // Execute
        ctx.eval::<(), _>(code).map_err(|e| e.to_string())?;

        let output_val = global
            .get::<_, Coerced<String>>("output")
            .map(|v| v.0)
            .unwrap_or_default();

        if !output_val.is_empty() {
            return Ok(output_val);
        }

        Ok(output.lock().map(|out| out.clone()).unwrap_or_default())
    })
}

pub async fn execute_script(code: &str, input: &str) -> Result<String, String> {
    let code = code.to_string();
    let input = input.to_string();

    tokio::task::spawn_blocking(move || execute_script_blocking(&code, &input))
        .await
        .map_err(|e| e.to_string())?
}

pub fn localized_display_name(script: &Script) -> String {
    if let Some(text_key) = language::script_text(&script.id) {
        let default_name = display_name_from_id(&script.id);
        if script.name == default_name {
            return language::tr(text_key).to_string();
        }
    }
    script.name.clone()
}

pub fn display_name_from_id(id: &str) -> String {
    id.replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
