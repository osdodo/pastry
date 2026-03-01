#[derive(Debug, Clone)]
pub enum Message {
    ExternalManager(ManagerMessage),
    ClosePage,
    StartDrag,
}

#[derive(Debug, Clone)]
pub enum ManagerMessage {
    SearchChanged(String),
    StartNew,
    StartEdit(String),
    ShowDeleteConfirm(String),
    ConfirmDelete,
    CancelDelete,
    NameChanged(String),
    CodeChanged(iced::widget::text_editor::Action),
    Save,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum SelectScriptMessage {
    SearchChanged(String),
    SelectScript(String),
}
