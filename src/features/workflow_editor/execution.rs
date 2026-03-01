use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::{Graph, Node, NodeKind};
use crate::services::clipboard;
use crate::services::scripts;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Json(serde_json::Value),
    #[default]
    Null,
}

pub type NodeId = Uuid;
pub type PortId = Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum SideEffect {
    UpdateLatestCardOutput(String),
}

#[derive(Debug, Default)]
pub struct ExecutionContext {
    // Stores the value present at a specific port (output of a node -> input of another)
    pub port_values: HashMap<PortId, Value>,
    pub logs: Vec<String>,
    pub node_status: HashMap<NodeId, super::types::ExecutionStatus>,
    pub side_effects: Vec<SideEffect>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_port_value(&mut self, port_id: PortId, value: Value) {
        self.port_values.insert(port_id, value);
    }

    pub fn get_port_value(&self, port_id: &PortId) -> Option<&Value> {
        self.port_values.get(port_id)
    }

    pub fn log(&mut self, message: String) {
        self.logs.push(message);
    }

    pub fn set_status(&mut self, node_id: NodeId, status: super::types::ExecutionStatus) {
        self.node_status.insert(node_id, status);
    }
}

pub fn execute_graph(graph: &Graph) -> ExecutionContext {
    let mut context = ExecutionContext::new();

    // 1. Find Trigger Nodes
    let start_nodes: Vec<&Node> = graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Hotkey))
        .collect();

    execute_from_nodes(graph, start_nodes, &mut context);
    context
}

pub fn execute_graph_with_trigger(
    graph: &Graph,
    trigger_kind: NodeKind,
    prop_check: impl Fn(&super::types::NodeProperties) -> bool,
) -> ExecutionContext {
    let mut context = ExecutionContext::new();

    let start_nodes: Vec<&Node> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == trigger_kind && prop_check(&n.properties))
        .collect();

    if start_nodes.is_empty() {
        return context;
    }

    execute_from_nodes(graph, start_nodes, &mut context);
    context
}

fn execute_from_nodes(graph: &Graph, start_nodes: Vec<&Node>, context: &mut ExecutionContext) {
    // We already have mutable access to context.

    // Basic iterative execution (BFS-like) for now
    let mut queue: Vec<Uuid> = start_nodes.iter().map(|n| n.id).collect();
    let mut visited: Vec<Uuid> = Vec::new();

    // Initial Trigger for Start nodes (optional, simulation context)
    for node in &start_nodes {
        context.log(format!("Starting Flow at Node: {}", node.title));
    }

    while let Some(node_id) = queue.pop() {
        if visited.contains(&node_id) {
            continue;
        }

        // Mark running
        context.set_status(node_id, super::types::ExecutionStatus::Running);

        let node = match graph.nodes.iter().find(|n| n.id == node_id) {
            Some(n) => n,
            None => {
                context.set_status(node_id, super::types::ExecutionStatus::Error);
                continue;
            }
        };

        // 2. Execute Node
        execute_node(node, graph, context);
        visited.push(node_id);

        // Mark success
        context.set_status(node_id, super::types::ExecutionStatus::Success);

        // 3. Find next nodes via outputs
        for output in &node.outputs {
            // Find edges connected to this output
            for edge in &graph.edges {
                if edge.source_port == output.id {
                    // Propagate value (simple pass-through for now)
                    if let Some(val) = context.get_port_value(&output.id).cloned() {
                        context.set_port_value(edge.target_port, val);
                    }

                    queue.push(edge.target_node);
                }
            }
        }
    }
}
/// Helper function to format a Value for display
fn format_value(val: &Value) -> String {
    match val {
        Value::String(s) => {
            let char_count = s.chars().count();
            if char_count > 50 {
                let truncated: String = s.chars().take(47).collect();
                format!("\"{}...\" ({} chars)", truncated, char_count)
            } else {
                format!("\"{}\"", s)
            }
        }
        Value::Number(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Json(j) => {
            let s = j.to_string();
            let char_count = s.chars().count();
            if char_count > 50 {
                let truncated: String = s.chars().take(47).collect();
                format!("{}... ({} chars)", truncated, char_count)
            } else {
                s
            }
        }
        Value::Null => "<null>".to_string(),
    }
}

fn value_to_string(value: &Value, null_text: &str) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Boolean(boolean) => boolean.to_string(),
        Value::Json(json) => json.to_string(),
        Value::Null => null_text.to_string(),
    }
}

fn resolve_file_write_path(raw_path: &str) -> PathBuf {
    if raw_path == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from(raw_path));
    }

    if let Some(stripped) = raw_path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(stripped);
    }

    PathBuf::from(raw_path)
}

fn execute_node(node: &Node, _graph: &Graph, context: &mut ExecutionContext) {
    context.log(format!("▶ Executing: {}", node.title));

    match node.kind {
        NodeKind::Hotkey => {
            if let Some(port) = node.outputs.first() {
                context.set_port_value(port.id, Value::String("Hotkey Triggered".to_string()));
            }
        }
        NodeKind::Script => {
            // Get the script ID from node properties
            if let Some(script_id) = &node.properties.script_id {
                // Load the script from storage
                let script_storage = scripts::ScriptStorage::new();
                let all_scripts = script_storage.load();

                if let Some(script) = all_scripts.iter().find(|s| &s.id == script_id) {
                    // Get input value
                    let input_value = node
                        .inputs
                        .first()
                        .and_then(|in_port| context.get_port_value(&in_port.id))
                        .cloned();
                    let input_str = if let Some(value) = input_value {
                        context.log(format!("  ← Input: {}", format_value(&value)));
                        value_to_string(&value, "")
                    } else {
                        String::new()
                    };

                    match scripts::execute_script_blocking(&script.code, &input_str) {
                        Ok(output) => {
                            let val = Value::String(output);
                            context.log(format!("  ✓ Output: {}", format_value(&val)));
                            if let Some(out_port) = node.outputs.first() {
                                context.set_port_value(out_port.id, val);
                            }
                        }
                        Err(e) => {
                            context.log(format!("  ✗ Error: {}", e));
                            if let Some(out_port) = node.outputs.first() {
                                context.set_port_value(out_port.id, Value::Null);
                            }
                        }
                    }
                } else {
                    context.log(format!("Script with ID '{}' not found", script_id));
                }
            } else {
                context.log("No script selected for Script node".into());
            }
        }
        NodeKind::Clipboard => {
            let action = node
                .properties
                .clipboard_action
                .as_deref()
                .unwrap_or("Read");

            if action == "Read" {
                if let Some(out_port) = node.outputs.first() {
                    if let Some(content) = clipboard::get_clipboard_content() {
                        match content {
                            clipboard::ClipboardContent::Text(text) => {
                                let val = Value::String(text);
                                context.log(format!("  ✓ Output: {}", format_value(&val)));
                                context.set_port_value(out_port.id, val);
                            }
                            _ => {
                                context.log("  ⚠ Clipboard contains non-text data".into());
                                context.set_port_value(out_port.id, Value::Null);
                            }
                        }
                    } else {
                        context.log("  ⚠ Clipboard is empty".into());
                        context.set_port_value(out_port.id, Value::Null);
                    }
                }
            } else if action == "Add Card"
                && let Some(in_port) = node.inputs.first()
                && let Some(val) = context.get_port_value(&in_port.id)
            {
                let content = value_to_string(val, "");
                if !content.is_empty() {
                    let clipboard = arboard::Clipboard::new().ok();
                    if let Some(mut cb) = clipboard {
                        if let Err(e) = cb.set_text(content) {
                            context.log(format!("  ✗ Failed to set clipboard: {}", e));
                        } else {
                            context.log("  ✓ Added to clipboard (Card)".into());
                        }
                    }
                }
            }
        }
        NodeKind::FileWrite => {
            if let Some(in_port) = node.inputs.first()
                && let Some(val) = context.get_port_value(&in_port.id)
                && let Some(path) = &node.properties.file_write_path
            {
                let content = value_to_string(val, "null");
                let resolved_path = resolve_file_write_path(path);
                if let Err(e) = std::fs::write(&resolved_path, content) {
                    context.log(format!("  ✗ Failed to write file: {}", e));
                } else {
                    context.log(format!("  ✓ Wrote to {}", resolved_path.display()));
                }
            }
        }
        NodeKind::ClipboardCard => {
            if let Some(in_port) = node.inputs.first()
                && let Some(val) = context.get_port_value(&in_port.id)
            {
                let content = value_to_string(val, "");
                context.log(format!("  ✓ Content set to Card: {}", content));
                context
                    .side_effects
                    .push(SideEffect::UpdateLatestCardOutput(content));
            }
        }
    }
}
