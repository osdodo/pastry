use iced::Point;

use crate::{
    features::{clipboard, color_picker, json, scripts, workflow_editor, workflow_manager},
    ui::{language::Language, theme::ThemeMode},
};

#[derive(Debug, Clone)]
pub enum Message {
    Clipboard(clipboard::Message),
    Json(json::Message),
    Scripts(scripts::Message),
    WorkflowList(workflow_manager::message::Message),
    WorkflowEditor(workflow_editor::message::WorkflowEditorMessage),
    ColorPicker(color_picker::Message),
    SelectScriptDialog(scripts::SelectScriptMessage),
    CloseSelectScriptDialog,
    SelectWorkflowDialog(SelectWorkflowMessage),
    CloseSelectWorkflowDialog,
    ExecuteScript(usize, String, String),
    ScriptExecuted(usize, String, String, Result<String, String>),
    ExecuteWorkflow(usize, String),
    ToggleLanguageMenu,
    LanguageSelected(Language),
    TogglePinned,
    ToggleStartHidden,
    ToggleWebServer,
    SetTheme(ThemeMode),
    OpenScriptManagerPage,
    OpenWorkflowListPage,
    OpenSettingsPage,
    CloseSettingsPage,
    StartDrag,
    MouseMoved(Point),
    ShowWindow,
    ShowWindowFromFocus,
    QuitApp,
    WindowOpened(iced::window::Id),
    WindowFocusLost,
    WindowMoved(f32, f32),
    AnimationTick,
    GlobalHotkeyTriggered(u32),
}

#[derive(Debug, Clone)]
pub enum SelectWorkflowMessage {
    SearchChanged(String),
    SelectWorkflow(String),
}
