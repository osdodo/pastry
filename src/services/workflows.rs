use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::features::workflow_editor::types::Graph;
use crate::services::storage::Storage;

const WORKFLOWS_DIR: &str = "workflows";
const WORKFLOWS_META_FILE: &str = "workflows_meta.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub graph: Graph,
    pub enabled: bool,
}

impl Workflow {
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            graph: Graph::default(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowMeta {
    id: String,
    name: String,
    enabled: bool,
}

pub struct WorkflowStorage {
    storage: Storage,
    workflows_dir: PathBuf,
}

impl WorkflowStorage {
    pub fn new() -> Self {
        let storage = Storage::new();
        let workflows_dir = storage.base_dir().join(WORKFLOWS_DIR);

        std::fs::create_dir_all(&workflows_dir).ok();

        Self {
            storage,
            workflows_dir,
        }
    }

    pub fn load(&self) -> Vec<Workflow> {
        let meta_map: std::collections::HashMap<String, WorkflowMeta> =
            if let Ok(metas) = self.storage.load::<Vec<WorkflowMeta>>(WORKFLOWS_META_FILE) {
                metas.into_iter().map(|m| (m.id.clone(), m)).collect()
            } else {
                std::collections::HashMap::new()
            };

        let mut workflows = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.workflows_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }

                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };

                if let Ok(mut workflow) = serde_json::from_str::<Workflow>(&content) {
                    if let Some(meta) = meta_map.get(&workflow.id) {
                        workflow.enabled = meta.enabled;
                    }
                    workflows.push(workflow);
                }
            }
        }

        workflows.sort_by(|a, b| a.name.cmp(&b.name));
        workflows
    }

    fn save_workflow_file(&self, workflow: &Workflow) -> Result<(), String> {
        let file_path = self.workflows_dir.join(format!("{}.json", workflow.id));
        let content = serde_json::to_string_pretty(workflow)
            .map_err(|e| format!("Failed to serialize workflow: {}", e))?;
        std::fs::write(&file_path, &content)
            .map_err(|e| format!("Failed to write workflow file: {}", e))
    }

    fn save_meta(&self, workflows: &[Workflow]) -> Result<(), String> {
        let metas: Vec<WorkflowMeta> = workflows
            .iter()
            .map(|w| WorkflowMeta {
                id: w.id.clone(),
                name: w.name.clone(),
                enabled: w.enabled,
            })
            .collect();

        self.storage.save(WORKFLOWS_META_FILE, &metas)
    }

    pub fn add(&self, workflow: Workflow) -> Result<(), String> {
        self.save_workflow_file(&workflow)?;
        let mut workflows = self.load();
        workflows.push(workflow);
        self.save_meta(&workflows)
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let file_path = self.workflows_dir.join(format!("{}.json", id));
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to delete workflow file: {}", e))?;
        }

        let mut workflows = self.load();
        workflows.retain(|w| w.id != id);
        self.save_meta(&workflows)
    }

    pub fn update(&self, workflow: &Workflow) -> Result<(), String> {
        self.save_workflow_file(workflow)?;

        let mut workflows = self.load();
        if let Some(existing) = workflows.iter_mut().find(|w| w.id == workflow.id) {
            existing.name = workflow.name.clone();
            existing.graph = workflow.graph.clone();
            existing.enabled = workflow.enabled;
            self.save_meta(&workflows)
        } else {
            Err("Workflow not found".to_string())
        }
    }

    pub fn get(&self, id: &str) -> Option<Workflow> {
        let file_path = self.workflows_dir.join(format!("{}.json", id));
        let content = std::fs::read_to_string(&file_path).ok()?;
        serde_json::from_str::<Workflow>(&content).ok()
    }

    pub fn toggle_enabled(&self, id: &str) -> Result<(), String> {
        let mut workflows = self.load();
        if let Some(workflow) = workflows.iter_mut().find(|w| w.id == id) {
            workflow.enabled = !workflow.enabled;
            self.save_meta(&workflows)
        } else {
            Err("Workflow not found".to_string())
        }
    }
}
