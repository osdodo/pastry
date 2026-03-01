use super::{
    message::{ManagerMessage, Message},
    state::WorkflowListState,
};

pub fn update(message: Message, state: &mut WorkflowListState) {
    match message {
        Message::List(msg) => match msg {
            ManagerMessage::SearchChanged(search) => {
                state.manager.search = search;
            }
            ManagerMessage::ShowDeleteConfirm(id) => {
                state.manager.delete_confirm_id = Some(id);
            }
            ManagerMessage::ConfirmDelete => {
                if let Some(id) = &state.manager.delete_confirm_id {
                    if let Err(e) = state.workflow_storage.delete(id) {
                        eprintln!("Failed to delete workflow: {}", e);
                    } else {
                        state.reload();
                    }
                }
                state.manager.delete_confirm_id = None;
            }
            ManagerMessage::CancelDelete => {
                state.manager.delete_confirm_id = None;
            }
            ManagerMessage::ToggleEnabled(id) => {
                if let Err(e) = state.workflow_storage.toggle_enabled(&id) {
                    eprintln!("Failed to toggle enabled state: {}", e);
                } else {
                    state.reload();
                }
            }
            _ => {}
        },
    }
}
