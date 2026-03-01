use std::time::Instant;

use iced::{Point, Size, Task, window as iced_window};
use serde_json::Value;

use crate::{
    app::{AppSettings, Message, Page, SETTINGS_FILE, State, message::SelectWorkflowMessage},
    features::{clipboard, color_picker, json, scripts, workflow_editor, workflow_manager},
    platform::{
        hotkey,
        screen::{get_screen_size, get_window_height, set_window_position},
    },
    services::storage::Storage,
    services::workflows::Workflow,
    services::{
        clipboard::{ClipboardContent, get_clipboard_content, save_favorites},
        scripts::execute_script,
    },
    ui::constants::{ANIMATION_DURATION_MS, WINDOW_MARGIN},
    ui::language,
    ui::theme,
};

fn persist_favorites(state: &State) {
    let cards: Vec<_> = state
        .clipboard
        .history
        .iter()
        .filter(|c| c.is_favorite)
        .map(|c| c.to_favorite_data())
        .collect();
    let _ = save_favorites(&cards);
}

fn persist_settings(state: &State) {
    let _ = Storage::new().save(
        SETTINGS_FILE,
        &AppSettings {
            language: Some(language::to_code(language::current()).to_string()),
            theme_mode: Some(theme::current()),
            start_hidden: Some(state.start_hidden),
            web_server_enabled: Some(state.web_server_enabled),
        },
    );
}

fn start_hide_animation(state: &mut State) {
    state.window.target_visible = false;
    state.window.animating = true;
    state.window.animation_start = Some(Instant::now());
}

fn start_show_animation(state: &mut State) -> Task<Message> {
    if state.window.visible && !state.window.animating {
        return Task::none();
    }
    state.window.target_visible = true;
    state.window.suppress_focus_show = false;
    if let Some(id) = state.window.id {
        state.window.animating = true;
        state.window.animation_start = Some(Instant::now());
        state.window.pending_show = false;
        return iced_window::gain_focus(id);
    }
    state.window.pending_show = true;
    Task::none()
}

fn update_dialog_position_from_cursor(state: &mut State) {
    let pos = state.cursor_position;
    state.dialog_position = (pos.x != 0.0 || pos.y != 0.0).then_some(pos);
}

fn open_select_script_dialog(state: &mut State, index: usize) {
    state.script_target_index = Some(index);
    state.show_select_script_dialog = true;
    state.select_script_dialog.clear();
    update_dialog_position_from_cursor(state);
}

fn open_select_workflow_dialog(state: &mut State, index: usize) {
    state.workflow_target_index = Some(index);
    state.show_select_workflow_dialog = true;
    state.select_workflow_dialog.clear();
    update_dialog_position_from_cursor(state);
}

fn close_select_script_dialog(state: &mut State) {
    state.show_select_script_dialog = false;
    state.script_target_index = None;
    state.dialog_position = None;
}

fn close_select_workflow_dialog(state: &mut State) {
    state.show_select_workflow_dialog = false;
    state.workflow_target_index = None;
    state.dialog_position = None;
}

fn load_workflow_into_editor(state: &mut State, workflow: Workflow) {
    let Workflow {
        id, name, graph, ..
    } = workflow;
    state.workflow_editor.workflow_id = Some(id);
    state.workflow_editor.name = name;
    state.workflow_editor.graph = graph;
    state.workflow_editor.has_unsaved_changes = false;
    state.workflow_editor.save_indicator_phase = 0.0;
    state.workflow_editor.available_scripts = state.scripts.scripts.clone();
    state.page.set(Page::WorkflowEditor);
}

fn update_card_output(card: &mut clipboard::model::CardState, output: String, name: String) {
    card.script_output = Some(output);
    card.script_name = Some(name);
    card.script_output_copied = false;
}

fn apply_output_to_history_card(
    state: &mut State,
    index: usize,
    output: String,
    name: String,
    script_id: Option<String>,
) {
    if let Some(card) = state.clipboard.history.get_mut(index) {
        update_card_output(card, output, name);
        if let Some(script_id) = script_id {
            card.script_id = Some(script_id);
        }

        if card.is_favorite {
            persist_favorites(state);
        }
    }
}

fn apply_output_to_latest_card(state: &mut State, output: String, name: String) {
    if let Some(card) = state.clipboard.history.first_mut() {
        update_card_output(card, output, name);

        if card.is_favorite {
            persist_favorites(state);
        }
    }
}

fn apply_side_effects_to_latest_card(
    state: &mut State,
    side_effects: impl IntoIterator<Item = workflow_editor::execution::SideEffect>,
) {
    for effect in side_effects {
        match effect {
            workflow_editor::execution::SideEffect::UpdateLatestCardOutput(content) => {
                apply_output_to_latest_card(state, content, "Workflow Result".to_string());
            }
        }
    }
}

fn compute_clipboard_hash(content: &ClipboardContent) -> u64 {
    match content {
        ClipboardContent::Text(text) => {
            let clip_type = if text.contains('<') && text.contains('>') {
                clipboard::model::ClipType::RichText
            } else {
                clipboard::model::ClipType::PlainText
            };
            clipboard::model::CardState::compute_text_hash(text, clip_type)
        }
        ClipboardContent::Image(data, width, height, _)
        | ClipboardContent::ImageFile(_, data, width, height, _) => {
            use std::hash::{Hash, Hasher};

            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            2u8.hash(&mut hasher);
            data.hash(&mut hasher);
            width.hash(&mut hasher);
            height.hash(&mut hasher);
            hasher.finish()
        }
    }
}

pub fn update(state: &mut State, msg: Message) -> Task<Message> {
    match msg {
        Message::Clipboard(m) => {
            if let clipboard::message::Message::ExternalCard(
                index,
                clipboard::model::CardMessage::RunScript,
            ) = &m
            {
                open_select_script_dialog(state, *index);
                return Task::none();
            }
            if let clipboard::message::Message::ExternalCard(
                index,
                clipboard::model::CardMessage::RunWorkflow,
            ) = &m
            {
                open_select_workflow_dialog(state, *index);
                return Task::none();
            }
            if let clipboard::message::Message::ExternalCard(
                index,
                clipboard::model::CardMessage::ShowJsonFormat,
            ) = &m
            {
                if let Some(entry) = state.clipboard.history.get(*index) {
                    let trimmed = entry.content.trim();
                    if !trimmed.is_empty()
                        && let Ok(value) = serde_json::from_str::<Value>(trimmed)
                        && let Ok(pretty) = serde_json::to_string_pretty(&value)
                    {
                        // Switch to Json page and load content
                        state.json.prepare_deferred_load();
                        state.page.set(Page::Json);
                        return Task::done(Message::Json(json::Message::DeferredLoad(pretty)));
                    }
                }
                return Task::none();
            }
            if let clipboard::message::Message::ExternalCard(
                _,
                clipboard::model::CardMessage::ToggleColorPicker(color),
            ) = &m
            {
                state.color_picker.update_color(*color);
                state.show_color_picker = true;
                return Task::none();
            }
            clipboard::update::update(&mut state.clipboard, m).map(Message::Clipboard)
        }
        Message::Scripts(m) => match m {
            scripts::Message::ClosePage => {
                state.page.set(Page::Main);
                Task::none()
            }
            scripts::Message::StartDrag => Task::done(Message::StartDrag),
            _ => scripts::update::update(&mut state.scripts, m).map(Message::Scripts),
        },
        Message::WorkflowList(m) => match m {
            workflow_manager::message::Message::List(
                workflow_manager::message::ManagerMessage::StartDrag,
            ) => Task::done(Message::StartDrag),
            workflow_manager::message::Message::List(
                workflow_manager::message::ManagerMessage::ClosePage,
            ) => {
                state.page.set(Page::Main);
                Task::none()
            }
            workflow_manager::message::Message::List(
                workflow_manager::message::ManagerMessage::OpenEditor(id),
            ) => {
                if let Some(workflow) = state.workflow_list.workflow_storage.get(&id) {
                    load_workflow_into_editor(state, workflow);
                }
                Task::none()
            }
            workflow_manager::message::Message::List(
                workflow_manager::message::ManagerMessage::CreateWorkflow,
            ) => {
                let workflow = Workflow::new("New Workflow".to_string());
                if let Err(e) = state.workflow_list.workflow_storage.add(workflow.clone()) {
                    eprintln!("Failed to create workflow: {}", e);
                } else {
                    load_workflow_into_editor(state, workflow);
                }
                Task::none()
            }
            _ => {
                workflow_manager::update::update(m, &mut state.workflow_list);
                Task::none()
            }
        },
        Message::ColorPicker(m) => color_picker::update::update(state, m),
        Message::Json(m) => match m {
            json::Message::ClosePage => {
                state.page.set(Page::Main);
                Task::none()
            }
            json::Message::StartDrag => Task::done(Message::StartDrag),
            _ => json::update::update(&mut state.json, m).map(Message::Json),
        },
        Message::WorkflowEditor(m) => match m {
            workflow_editor::message::WorkflowEditorMessage::ClosePage => {
                state.workflow_list.reload();
                state.page.set(Page::WorkflowList);
                Task::none()
            }
            workflow_editor::message::WorkflowEditorMessage::StartDrag => {
                Task::done(Message::StartDrag)
            }
            workflow_editor::message::WorkflowEditorMessage::Save => {
                workflow_editor::update::update(
                    &mut state.workflow_editor,
                    workflow_editor::message::WorkflowEditorMessage::Save,
                );
                state.workflow_list.reload();

                Task::none()
            }
            _ => {
                workflow_editor::update::update(&mut state.workflow_editor, m);
                let side_effects = std::mem::take(&mut state.workflow_editor.pending_side_effects);
                apply_side_effects_to_latest_card(state, side_effects);

                Task::none()
            }
        },
        Message::GlobalHotkeyTriggered(id) => {
            // Force a clipboard poll to ensure the latest content is in history before executing the workflow
            if let Some(content) = get_clipboard_content() {
                let current_hash = compute_clipboard_hash(&content);

                if current_hash != state.clipboard.last_clipboard_hash {
                    let _ = clipboard::update::update(
                        &mut state.clipboard,
                        clipboard::message::Message::ClipboardChanged(content),
                    );
                }
            }

            let mut side_effects = Vec::new();
            let workflow_list = &state.workflow_list.workflows;

            for workflow in workflow_list {
                if !workflow.enabled {
                    continue;
                }

                // Check if this workflow's hashed ID matches the triggered ID
                if hotkey::hash_id(&workflow.id) == id {
                    let context = workflow_editor::execution::execute_graph_with_trigger(
                        &workflow.graph,
                        workflow_editor::types::NodeKind::Hotkey,
                        |_| true, // Triggered by global hotkey directly, we already know it matches
                    );

                    if !context.logs.is_empty() {
                        side_effects.extend(context.side_effects);
                    }
                }
            }

            for effect in side_effects {
                match effect {
                    workflow_editor::execution::SideEffect::UpdateLatestCardOutput(content) => {
                        apply_output_to_latest_card(state, content, "Workflow Result".to_string());
                    }
                }
            }
            Task::none()
        }
        Message::ToggleLanguageMenu => {
            state.show_language_menu = !state.show_language_menu;
            Task::none()
        }
        Message::SetTheme(mode) => {
            if theme::current() != mode {
                theme::set_current(mode);
                persist_settings(state);
            }
            Task::none()
        }
        Message::LanguageSelected(lang) => {
            language::set_current(lang);
            persist_settings(state);
            state.show_language_menu = false;
            Task::none()
        }
        Message::TogglePinned => {
            state.pinned = !state.pinned;
            Task::none()
        }
        Message::ToggleStartHidden => {
            state.start_hidden = !state.start_hidden;
            persist_settings(state);
            Task::none()
        }
        Message::ToggleWebServer => {
            state.web_server_enabled = !state.web_server_enabled;
            if !state.web_server_enabled {
                std::mem::drop(tokio::spawn(async {
                    crate::web::stop_web_server().await;
                }));
            }
            state.refresh_web_access();
            persist_settings(state);
            Task::none()
        }
        Message::SelectScriptDialog(msg) => match msg {
            scripts::SelectScriptMessage::SearchChanged(search) => {
                state.select_script_dialog.search = search;
                Task::none()
            }
            scripts::SelectScriptMessage::SelectScript(script_id) => {
                if let Some(index) = state.script_target_index {
                    let script_name = state
                        .scripts
                        .scripts
                        .iter()
                        .find(|s| s.id == script_id)
                        .map(|s| s.name.clone())
                        .unwrap_or_else(|| language::tr(language::Text::UnknownScript).to_string());
                    close_select_script_dialog(state);
                    Task::done(Message::ExecuteScript(index, script_id, script_name))
                } else {
                    Task::none()
                }
            }
        },
        Message::CloseSelectScriptDialog => {
            close_select_script_dialog(state);
            Task::none()
        }
        Message::SelectWorkflowDialog(msg) => match msg {
            SelectWorkflowMessage::SearchChanged(search) => {
                state.select_workflow_dialog.search = search;
                Task::none()
            }
            SelectWorkflowMessage::SelectWorkflow(workflow_id) => {
                if let Some(index) = state.workflow_target_index {
                    close_select_workflow_dialog(state);
                    Task::done(Message::ExecuteWorkflow(index, workflow_id))
                } else {
                    Task::none()
                }
            }
        },
        Message::CloseSelectWorkflowDialog => {
            close_select_workflow_dialog(state);
            Task::none()
        }
        Message::ExecuteScript(index, script_id, script_name) => {
            if let Some(entry) = state.clipboard.history.get(index) {
                let input = entry.content.clone();
                if let Some(script) = state.scripts.scripts.iter().find(|s| s.id == script_id) {
                    let code = script.code.clone();
                    let name = script_name.clone();
                    let id = script_id.clone();
                    return Task::perform(
                        async move { execute_script(&code, &input).await },
                        move |result| Message::ScriptExecuted(index, id, name, result),
                    );
                }
            }
            Task::none()
        }
        Message::ScriptExecuted(index, script_id, script_name, result) => {
            let output = match result {
                Ok(output) => output,
                Err(error) => error.to_string(),
            };
            apply_output_to_history_card(state, index, output, script_name, Some(script_id));
            Task::none()
        }
        Message::ExecuteWorkflow(index, workflow_id) => {
            if let Some(workflow) = state.workflow_list.workflow_storage.get(&workflow_id) {
                let workflow_name = workflow.name;
                let context = workflow_editor::execution::execute_graph_with_trigger(
                    &workflow.graph,
                    workflow_editor::types::NodeKind::Clipboard,
                    |_| true,
                );

                for effect in context.side_effects {
                    match effect {
                        workflow_editor::execution::SideEffect::UpdateLatestCardOutput(content) => {
                            apply_output_to_history_card(
                                state,
                                index,
                                content,
                                workflow_name.clone(),
                                None,
                            );
                        }
                    }
                }
            }
            Task::none()
        }
        Message::OpenScriptManagerPage => {
            state.scripts.prepare_deferred_load();
            state.page.set(Page::ScriptManager);
            Task::none()
        }
        Message::OpenWorkflowListPage => {
            state.workflow_list.reload();
            state.workflow_list.manager.delete_confirm_id = None;
            state.page.set(Page::WorkflowList);
            Task::none()
        }
        Message::OpenSettingsPage => {
            state.show_language_menu = false;
            state.page.set(Page::Settings);
            Task::done(Message::ShowWindow)
        }
        Message::CloseSettingsPage => {
            state.page.set(Page::Main);
            Task::none()
        }
        Message::MouseMoved(position) => {
            state.cursor_position = position;
            state.json.cursor_position = position;
            Task::none()
        }
        Message::WindowMoved(x, y) => {
            set_window_position(x, y);
            state.window.position.0 = x;
            state.window.position.1 = y;
            Task::none()
        }
        Message::StartDrag => {
            if let Some(id) = state.window.id {
                return iced_window::drag(id);
            }
            Task::none()
        }
        Message::ShowWindow => start_show_animation(state),
        Message::ShowWindowFromFocus => {
            if state.window.suppress_focus_show {
                state.window.suppress_focus_show = false;
                return Task::none();
            }
            start_show_animation(state)
        }
        Message::QuitApp => std::process::exit(0),
        Message::WindowOpened(id) => {
            state.window.id = Some(id);
            if state.window.pending_show {
                state.window.pending_show = false;
                state.window.target_visible = true;
                state.window.animating = true;
                state.window.animation_start = Some(Instant::now());
                state.window.suppress_focus_show = false;
                return iced_window::gain_focus(id);
            }
            Task::none()
        }
        Message::WindowFocusLost => {
            if !state.pinned && !state.window.animating && state.window.visible {
                start_hide_animation(state);
            }
            Task::none()
        }
        Message::AnimationTick => {
            let mut window_task = Task::none();
            let animating_now = state.window.animating;

            if state.window.animating
                && let Some(start) = state.window.animation_start
            {
                let elapsed = start.elapsed().as_millis() as f32;
                let duration = ANIMATION_DURATION_MS as f32;
                let progress: f32 = (elapsed / duration).min(1.0_f32);
                let eased: f32 = 1.0_f32 - (1.0_f32 - progress).powi(3);
                state.window.animation_progress = if state.window.target_visible {
                    eased
                } else {
                    1.0_f32 - eased
                };
                if progress >= 1.0_f32 {
                    state.window.animating = false;
                    state.window.animation_start = None;
                    state.window.visible = state.window.target_visible;
                }
            }

            if animating_now && let Some(id) = state.window.id {
                let (screen_width, screen_height) = get_screen_size();
                let window_height = get_window_height(WINDOW_MARGIN);

                // Right-edge anchored during show/hide animation
                let x_hidden = screen_width + 10.0;
                let x_visible = screen_width - state.window.current_width - 20.0;
                let x = x_hidden + (x_visible - x_hidden) * state.window.animation_progress;
                let y = (screen_height - window_height) / 2.0;

                window_task = Task::batch([
                    iced_window::resize(id, Size::new(state.window.current_width, window_height)),
                    iced_window::move_to(id, Point::new(x, y)),
                ]);
            }

            window_task
        }
    }
}
