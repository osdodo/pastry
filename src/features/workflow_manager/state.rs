use crate::services::workflows::{Workflow, WorkflowStorage};

#[derive(Debug, Clone)]
pub struct WorkflowManager {
    pub search: String,
    pub delete_confirm_id: Option<String>,
}

impl WorkflowManager {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            delete_confirm_id: None,
        }
    }
}

pub struct WorkflowListState {
    pub workflows: Vec<Workflow>,
    pub manager: WorkflowManager,
    pub workflow_storage: WorkflowStorage,
}

impl WorkflowListState {
    pub fn new() -> Self {
        let workflow_storage = WorkflowStorage::new();
        let workflows = workflow_storage.load();
        crate::platform::hotkey::update_workflow_hotkeys(&workflows);
        let manager = WorkflowManager::new();
        Self {
            workflows,
            manager,
            workflow_storage,
        }
    }

    pub fn reload(&mut self) {
        self.workflows = self.workflow_storage.load();
        crate::platform::hotkey::update_workflow_hotkeys(&self.workflows);
    }
}

impl Default for WorkflowListState {
    fn default() -> Self {
        Self::new()
    }
}
