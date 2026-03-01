use std::{f32::consts::TAU, path::Path};

use uuid::Uuid;

use super::message::WorkflowEditorMessage;
use super::state::WorkflowEditorState;
use crate::platform::hotkey;
use crate::services::workflows::WorkflowStorage;
use crate::ui::util;

pub fn update(state: &mut WorkflowEditorState, message: WorkflowEditorMessage) {
    let was_unsaved = state.has_unsaved_changes;

    match message {
        WorkflowEditorMessage::NameChanged(name) => {
            if state.name != name {
                state.name = name;
                state.has_unsaved_changes = true;
            }
        }
        WorkflowEditorMessage::Save => {
            let workflow_storage = WorkflowStorage::new();
            if let Some(id) = &state.workflow_id
                && let Some(existing_workflow) = workflow_storage.get(id)
            {
                let mut workflow = existing_workflow;
                workflow.name = state.name.clone();
                workflow.graph = state.graph.clone();
                if let Err(e) = workflow_storage.update(&workflow) {
                    eprintln!("Failed to save workflow: {}", e);
                } else {
                    state.has_unsaved_changes = false;
                    let all_workflows = workflow_storage.load();
                    hotkey::update_workflow_hotkeys(&all_workflows);
                }
            }
        }
        WorkflowEditorMessage::CanvasPressed => {
            state.selected_node = None;
            state.context_menu = None;
            state.node_context_menu = None;
            state.inspector_node = None;
            state.execution_log.clear();
        }
        WorkflowEditorMessage::NodePressed(id) => {
            state.selected_node = Some(id);
            state.dragging_node = Some(id);
            state.context_menu = None;
            state.node_context_menu = None;
        }
        WorkflowEditorMessage::PortPressed(id, pos) => {
            let mut source_port_id = id;

            if let Some(edge_idx) = state.graph.edges.iter().position(|e| e.target_port == id) {
                let edge = state.graph.edges.remove(edge_idx);
                source_port_id = edge.source_port;
            }

            state.dragging_edge = Some((source_port_id, pos));
            state.context_menu = None;
            state.node_context_menu = None;
        }
        WorkflowEditorMessage::PortReleased(end_port_id) => {
            if let Some((start_port_id, _)) = state.dragging_edge
                && start_port_id != end_port_id
            {
                let edge = super::types::Edge {
                    id: Uuid::new_v4(),
                    source_node: state
                        .graph
                        .find_node_by_port(start_port_id)
                        .unwrap_or(Uuid::nil()),
                    source_port: start_port_id,
                    target_node: state
                        .graph
                        .find_node_by_port(end_port_id)
                        .unwrap_or(Uuid::nil()),
                    target_port: end_port_id,
                };

                if edge.source_node != edge.target_node
                    && edge.source_node != Uuid::nil()
                    && edge.target_node != Uuid::nil()
                {
                    state.graph.add_edge(edge);
                    state.has_unsaved_changes = true;
                }
            }
            state.dragging_edge = None;
        }
        WorkflowEditorMessage::CanvasReleased => {
            state.dragging_node = None;
            state.dragging_edge = None;
        }
        WorkflowEditorMessage::NodeMoved(id, new_pos) => {
            if id.is_nil() {
                if let Some((port_id, _)) = state.dragging_edge {
                    state.dragging_edge = Some((port_id, new_pos));
                }
            } else if let Some(node) = state.graph.nodes.iter_mut().find(|n| n.id == id)
                && node.position != new_pos
            {
                node.position = new_pos;
                state.has_unsaved_changes = true;
            }
        }
        WorkflowEditorMessage::CanvasPanned(delta) => {
            state.pan.x += delta.x;
            state.pan.y += delta.y;
            state.context_menu = None;
            state.node_context_menu = None;
        }
        WorkflowEditorMessage::CanvasZoomed(zoom, _anchor) => {
            // Simple zoom for now, ignoring anchor to keep pan simple
            state.zoom = zoom;
            state.context_menu = None;
            state.node_context_menu = None;
        }
        WorkflowEditorMessage::AddNode(kind, pos) => {
            state.add_node(kind, pos);
            state.context_menu = None;
            state.node_context_menu = None;
            state.has_unsaved_changes = true;
        }
        WorkflowEditorMessage::RemoveNode(id) => {
            state.remove_node(id);
            state.has_unsaved_changes = true;
        }
        WorkflowEditorMessage::NodeTitleEdited(id, new_title) => {
            if let Some(node) = state.graph.nodes.iter_mut().find(|n| n.id == id)
                && node.title != new_title
            {
                node.title = new_title;
                state.has_unsaved_changes = true;
            }
        }
        WorkflowEditorMessage::FileWritePathEdited(id, path) => {
            update_property(state, id, |props| props.file_write_path = Some(path));
        }
        WorkflowEditorMessage::FileWriteUseDesktop(id) => {
            if let Some(desktop_dir) = dirs::desktop_dir() {
                update_property(state, id, |props| {
                    let filename =
                        extract_filename(props.file_write_path.as_deref().unwrap_or("output.txt"));
                    let target = desktop_dir.join(filename);
                    props.file_write_path = Some(target.to_string_lossy().to_string());
                });
            }
        }
        WorkflowEditorMessage::FileWriteUseDownloads(id) => {
            let downloads_dir =
                dirs::download_dir().or_else(|| dirs::home_dir().map(|h| h.join("Downloads")));
            if let Some(downloads_dir) = downloads_dir {
                update_property(state, id, |props| {
                    let filename =
                        extract_filename(props.file_write_path.as_deref().unwrap_or("output.txt"));
                    let target = downloads_dir.join(filename);
                    props.file_write_path = Some(target.to_string_lossy().to_string());
                });
            }
        }
        WorkflowEditorMessage::FileWriteBrowseFolder(id) => {
            let default_name = state
                .graph
                .nodes
                .iter()
                .find(|node| node.id == id)
                .map(|node| {
                    extract_filename(
                        node.properties
                            .file_write_path
                            .as_deref()
                            .unwrap_or("output.txt"),
                    )
                })
                .unwrap_or_else(|| "output.txt".to_string());

            if let Some(folder_path) = pick_file_write_folder() {
                let target_path = Path::new(&folder_path).join(default_name);
                update_property(state, id, |props| {
                    props.file_write_path = Some(target_path.to_string_lossy().to_string())
                });
            }
        }
        WorkflowEditorMessage::ShowContextMenu(pos) => {
            state.context_menu = Some(pos);
            state.node_context_menu = None;
        }
        WorkflowEditorMessage::ShowNodeContextMenu(id, pos) => {
            state.node_context_menu = Some((id, pos));
            state.context_menu = None;
        }
        WorkflowEditorMessage::HideContextMenu => {
            state.context_menu = None;
            state.node_context_menu = None;
        }
        WorkflowEditorMessage::DisconnectPort(port_id) => {
            let original_len = state.graph.edges.len();
            state
                .graph
                .edges
                .retain(|e| e.source_port != port_id && e.target_port != port_id);
            if state.graph.edges.len() != original_len {
                state.has_unsaved_changes = true;
            }
        }
        WorkflowEditorMessage::DisconnectNode(node_id) => {
            let original_len = state.graph.edges.len();
            state
                .graph
                .edges
                .retain(|e| e.source_node != node_id && e.target_node != node_id);
            state.node_context_menu = None;
            if state.graph.edges.len() != original_len {
                state.has_unsaved_changes = true;
            }
        }
        WorkflowEditorMessage::RunGraph => {
            let context = super::execution::execute_graph(&state.graph);
            state.execution_log = context.logs;
            state.node_status = context.node_status;
            state.pending_side_effects = context.side_effects;
        }
        WorkflowEditorMessage::ClosePage => {}
        // Property editing messages
        WorkflowEditorMessage::HotkeyComboEdited(id, value) => {
            update_property(state, id, |props| props.hotkey_combo = Some(value));
        }
        WorkflowEditorMessage::ScriptIdEdited(id, value) => {
            update_property(state, id, |props| props.script_id = Some(value));
        }

        WorkflowEditorMessage::ToggleInspector(node_id) => {
            if state.inspector_node == Some(node_id) {
                state.inspector_node = None;
            } else {
                state.inspector_node = Some(node_id);
            }
        }
        WorkflowEditorMessage::HotkeyRecording(mut id, event) => {
            // If ID is nil, it means it came from global subscription, so we use the currently inspected node
            if id.is_nil() {
                if let Some(inspected_id) = state.inspector_node {
                    id = inspected_id;
                } else {
                    return;
                }
            }

            if let iced::keyboard::Event::KeyPressed { key, modifiers, .. } = event
                && let Some(combo) = util::hotkey::format_hotkey(modifiers, key)
            {
                update_property(state, id, |props| props.hotkey_combo = Some(combo));
            }
        }
        WorkflowEditorMessage::SaveIndicatorTick => {
            if state.has_unsaved_changes {
                state.save_indicator_phase += 0.32;
                if state.save_indicator_phase > TAU {
                    state.save_indicator_phase -= TAU;
                }
            }
        }
        WorkflowEditorMessage::StartDrag => {}
        WorkflowEditorMessage::NoOp => {}
    }

    if !was_unsaved && state.has_unsaved_changes {
        state.save_indicator_phase = 0.0;
    }
}

fn update_property<F>(state: &mut WorkflowEditorState, node_id: Uuid, update_fn: F)
where
    F: FnOnce(&mut super::types::NodeProperties),
{
    if let Some(node) = state.graph.nodes.iter_mut().find(|n| n.id == node_id) {
        let old_props = node.properties.clone();
        update_fn(&mut node.properties);
        if old_props != node.properties {
            state.has_unsaved_changes = true;
        }
    }
}

fn extract_filename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("output.txt")
        .to_string()
}

fn pick_file_write_folder() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        pick_file_write_folder_macos()
    }

    #[cfg(target_os = "linux")]
    {
        pick_file_write_folder_linux()
    }

    #[cfg(target_os = "windows")]
    {
        pick_file_write_folder_windows()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
fn pick_file_write_folder_macos() -> Option<String> {
    let script =
        "set p to POSIX path of (choose folder with prompt \"Select output folder\")\nreturn p";

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8(output.stdout).ok()?;
    let trimmed = path.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(target_os = "linux")]
fn pick_file_write_folder_linux() -> Option<String> {
    let output = std::process::Command::new("zenity")
        .args([
            "--file-selection",
            "--directory",
            "--title=Select output folder",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8(output.stdout).ok()?;
    let trimmed = path.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(target_os = "windows")]
fn pick_file_write_folder_windows() -> Option<String> {
    let script = "$ErrorActionPreference='SilentlyContinue'; Add-Type -AssemblyName System.Windows.Forms; $dlg = New-Object System.Windows.Forms.FolderBrowserDialog; if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { Write-Output $dlg.SelectedPath }";

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8(output.stdout).ok()?;
    let trimmed = path.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
