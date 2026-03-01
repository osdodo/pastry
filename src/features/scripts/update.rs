use iced::Task;

use super::{message::Message, state::State};
use crate::{features::scripts::ManagerMessage, services::scripts::Script};

pub fn update(state: &mut State, msg: Message) -> Task<Message> {
    match msg {
        Message::ClosePage => {
            // Signal that the page should be closed - handled by parent
            Task::none()
        }
        Message::ExternalManager(m) => match m {
            ManagerMessage::SearchChanged(search) => {
                state.manager.search = search;
                Task::none()
            }
            ManagerMessage::StartNew => {
                state.manager.start_new();
                Task::none()
            }
            ManagerMessage::ShowDeleteConfirm(script_id) => {
                state.manager.delete_confirm_id = Some(script_id);
                Task::none()
            }
            ManagerMessage::ConfirmDelete => {
                if let Some(script_id) = &state.manager.delete_confirm_id.clone() {
                    if state.script_storage.delete(script_id).is_ok() {
                        state.scripts = state.script_storage.load();
                    }
                    state.manager.delete_confirm_id = None;
                }
                Task::none()
            }
            ManagerMessage::CancelDelete => {
                state.manager.delete_confirm_id = None;
                Task::none()
            }
            ManagerMessage::StartEdit(script_id) => {
                if let Some(script) = state.scripts.iter().find(|s| s.id == script_id) {
                    state.manager.start_edit(script);
                }
                Task::none()
            }
            ManagerMessage::NameChanged(name) => {
                if let Some(editing) = &mut state.manager.editing_script {
                    editing.name = name;
                }
                Task::none()
            }
            ManagerMessage::CodeChanged(action) => {
                if let Some(editing) = &mut state.manager.editing_script {
                    editing.code_editor.perform(action);
                    editing.code = editing.code_editor.text();
                }
                Task::none()
            }
            ManagerMessage::Save => {
                if let Some(editing) = &mut state.manager.editing_script {
                    editing.code = editing.code_editor.text();

                    if !editing.name.is_empty() && !editing.code.is_empty() {
                        let result = if let Some(id) = &editing.id {
                            state.script_storage.update(
                                id,
                                editing.name.clone(),
                                editing.code.clone(),
                            )
                        } else {
                            let script = Script::new(editing.name.clone(), editing.code.clone());
                            state.script_storage.add(script)
                        };

                        if result.is_ok() {
                            state.scripts = state.script_storage.load();
                            state.manager.cancel_edit();
                        }
                    }
                }
                Task::none()
            }
            ManagerMessage::Cancel => {
                state.manager.cancel_edit();
                Task::none()
            }
        },
        Message::StartDrag => Task::none(),
    }
}
