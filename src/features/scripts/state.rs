use iced::widget::text_editor;

use crate::services::scripts::{Script, ScriptStorage};

pub struct State {
    pub scripts: Vec<Script>,
    pub manager: ScriptManager,
    pub script_storage: ScriptStorage,
    pub is_loading: bool,
}

#[derive(Debug, Clone)]
pub struct SelectScriptDialog {
    pub search: String,
}

impl SelectScriptDialog {
    pub fn new() -> Self {
        Self {
            search: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.search.clear();
    }
}

#[derive(Debug, Clone)]
pub struct ScriptManager {
    pub search: String,
    pub editing_script: Option<EditingScript>,
    pub delete_confirm_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EditingScript {
    pub id: Option<String>,
    pub name: String,
    pub code: String,
    pub code_editor: text_editor::Content,
}

impl ScriptManager {
    pub fn new() -> Self {
        Self {
            search: String::new(),
            editing_script: None,
            delete_confirm_id: None,
        }
    }

    pub fn start_new(&mut self) {
        self.editing_script = Some(EditingScript {
            id: None,
            name: String::new(),
            code: String::new(),
            code_editor: text_editor::Content::new(),
        });
    }

    pub fn start_edit(&mut self, script: &Script) {
        self.editing_script = Some(EditingScript {
            id: Some(script.id.clone()),
            name: script.name.clone(),
            code: script.code.clone(),
            code_editor: text_editor::Content::with_text(&script.code),
        });
    }

    pub fn cancel_edit(&mut self) {
        self.editing_script = None;
    }

    pub fn is_editing(&self) -> bool {
        self.editing_script.is_some()
    }
}

impl State {
    pub fn new() -> Self {
        let script_storage = ScriptStorage::new();
        let scripts = script_storage.load();
        let manager = ScriptManager::new();
        Self {
            scripts,
            manager,
            script_storage,
            is_loading: false,
        }
    }

    /// Prepare for deferred loading - sets loading flag
    pub fn prepare_deferred_load(&mut self) {
        self.is_loading = true;
        self.manager.editing_script = None;
        self.manager.delete_confirm_id = None;
    }
}
