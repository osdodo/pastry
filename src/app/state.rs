use iced::Point;

use crate::{
    features::{
        clipboard, color_picker, json, scripts, scripts::SelectScriptDialog, workflow_editor,
        workflow_manager,
    },
    services::clipboard::load_favorite_cards,
    ui::{constants::WINDOW_WIDTH, util::color::Color},
};

#[derive(Debug, Clone)]
pub struct SelectWorkflowDialog {
    pub search: String,
}

impl SelectWorkflowDialog {
    pub fn new() -> Self {
        Self {
            search: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.search.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Main,
    Json,
    ScriptManager,
    WorkflowList,
    WorkflowEditor,
    Settings,
}

pub struct PageState {
    pub current: Page,
}

impl PageState {
    pub fn new(initial: Page) -> Self {
        Self { current: initial }
    }

    pub fn set(&mut self, page: Page) {
        self.current = page;
    }
}

pub struct WindowState {
    pub id: Option<iced::window::Id>,
    pub visible: bool,
    pub animating: bool,
    pub animation_start: Option<std::time::Instant>,
    pub target_visible: bool,
    pub pending_show: bool,
    pub suppress_focus_show: bool,
    pub animation_progress: f32,
    // Width related
    pub current_width: f32,
    // Position related - merged into tuple
    pub position: (f32, f32),
}

impl WindowState {
    pub fn new(start_hidden: bool) -> Self {
        Self {
            id: None,
            visible: false,
            animating: false,
            animation_start: None,
            target_visible: false,
            pending_show: false,
            suppress_focus_show: start_hidden,
            animation_progress: 0.0,
            current_width: WINDOW_WIDTH,
            position: (0.0, 0.0),
        }
    }
}

pub struct State {
    pub window: WindowState,
    pub page: PageState,

    pub pinned: bool,
    pub start_hidden: bool,
    pub web_server_enabled: bool,
    pub web_access_url: Option<String>,
    pub web_qr_svg: Option<String>,

    pub clipboard: clipboard::State,
    pub scripts: scripts::State,
    pub json: json::State,
    pub color_picker: color_picker::State,
    pub workflow_list: workflow_manager::state::WorkflowListState,
    pub workflow_editor: workflow_editor::state::WorkflowEditorState,

    pub show_language_menu: bool,
    pub show_color_picker: bool,
    pub show_select_script_dialog: bool,
    pub show_select_workflow_dialog: bool,
    pub dialog_position: Option<Point>,
    pub cursor_position: Point,
    pub select_script_dialog: SelectScriptDialog,
    pub select_workflow_dialog: SelectWorkflowDialog,
    pub script_target_index: Option<usize>,
    pub workflow_target_index: Option<usize>,
}

impl State {
    pub fn new(start_hidden: bool, web_server_enabled: bool) -> Self {
        let mut clipboard_state = clipboard::State::new();
        let scripts_state = scripts::State::new();

        // Load favorites into clipboard history and populate script names
        if let Ok(cards) = load_favorite_cards() {
            let mut cards: Vec<clipboard::model::CardState> = cards
                .into_iter()
                .map(clipboard::model::CardState::from_data)
                .collect();
            for card in cards.iter_mut() {
                if let Some(script_id) = &card.script_id {
                    card.script_name = scripts_state
                        .scripts
                        .iter()
                        .find(|s| s.id == *script_id)
                        .map(|s| s.name.clone());
                }
            }
            clipboard_state.history.append(&mut cards);
        }

        let mut state = Self {
            window: WindowState::new(start_hidden),
            page: PageState::new(Page::Main),
            clipboard: clipboard_state,
            pinned: false,
            start_hidden,
            web_server_enabled,
            web_access_url: None,
            web_qr_svg: None,
            scripts: scripts_state,
            json: json::State::new(),
            color_picker: color_picker::State::new(Color::new(0.0, 0.0, 0.0, 1.0)),
            workflow_list: workflow_manager::state::WorkflowListState::new(),
            workflow_editor: workflow_editor::state::WorkflowEditorState::new(),
            show_language_menu: false,
            show_color_picker: false,
            show_select_script_dialog: false,
            show_select_workflow_dialog: false,
            dialog_position: None,
            cursor_position: Point::default(),
            select_script_dialog: SelectScriptDialog::new(),
            select_workflow_dialog: SelectWorkflowDialog::new(),
            script_target_index: None,
            workflow_target_index: None,
        };

        state.refresh_web_access();
        state
    }

    pub fn refresh_web_access(&mut self) {
        if !self.web_server_enabled {
            self.web_access_url = None;
            self.web_qr_svg = None;
            return;
        }

        self.web_access_url = crate::web::get_local_ip()
            .map(|ip| format!("http://{}:{}", ip, crate::web::DEFAULT_WEB_SERVER_PORT));
        self.web_qr_svg = self
            .web_access_url
            .as_deref()
            .and_then(crate::web::generate_qr_svg);
    }
}
