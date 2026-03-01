#[derive(Debug, Clone)]
pub enum Message {
    List(ManagerMessage),
}

#[derive(Debug, Clone)]
pub enum ManagerMessage {
    SearchChanged(String),
    ShowDeleteConfirm(String),
    ConfirmDelete,
    CancelDelete,
    OpenEditor(String),
    CreateWorkflow,
    StartDrag,
    ClosePage,
    ToggleEnabled(String),
}
