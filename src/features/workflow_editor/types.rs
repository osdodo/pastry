use crate::ui::language::{self, Text};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_file_write_path() -> String {
    if let Some(desktop_dir) = dirs::desktop_dir() {
        return desktop_dir.join("output.txt").to_string_lossy().to_string();
    }

    "output.txt".to_string()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: Uuid,
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    String,
    Number,
    Boolean,
    Json,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub kind: NodeKind,
    pub position: Point,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub title: String,
    #[serde(default)]
    pub properties: NodeProperties,
}

/// Node-specific properties
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeProperties {
    // Hotkey properties
    pub hotkey_combo: Option<String>,

    // Script properties
    pub script_id: Option<String>,

    // Clipboard Node properties
    pub clipboard_action: Option<String>,

    // File Write properties
    pub file_write_path: Option<String>,
}

impl Default for NodeProperties {
    fn default() -> Self {
        Self {
            hotkey_combo: None,
            script_id: None,
            clipboard_action: Some("Read".to_string()),
            file_write_path: Some(default_file_write_path()),
        }
    }
}

impl NodeProperties {
    pub fn new_for_kind(kind: &NodeKind) -> Self {
        match kind {
            NodeKind::Hotkey => Self {
                hotkey_combo: Some("Cmd+Shift+X".to_string()),
                ..Default::default()
            },
            NodeKind::Script => Self {
                script_id: None,
                ..Default::default()
            },
            NodeKind::Clipboard => Self {
                clipboard_action: Some("Read".to_string()),
                ..Default::default()
            },
            NodeKind::FileWrite => Self {
                file_write_path: Some(default_file_write_path()),
                ..Default::default()
            },
            NodeKind::ClipboardCard => Self::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeKind {
    // Triggers
    Hotkey,
    // Action
    Script,
    Clipboard,
    FileWrite,
    ClipboardCard,
}

impl NodeKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            NodeKind::Hotkey => language::tr(Text::NodeKindHotkey),
            NodeKind::Script => language::tr(Text::NodeKindScript),
            NodeKind::Clipboard => language::tr(Text::NodeKindClipboard),
            NodeKind::FileWrite => language::tr(Text::NodeKindFileWrite),
            NodeKind::ClipboardCard => language::tr(Text::NodeKindClipboardCard),
        }
    }

    pub fn is_trigger(&self) -> bool {
        matches!(self, NodeKind::Hotkey)
    }

    pub fn get_default_ports(&self) -> (Vec<Port>, Vec<Port>) {
        let mut inputs = vec![];
        let mut outputs = vec![];

        match self {
            NodeKind::Hotkey => {
                outputs.push(Port::new(language::tr(Text::PortFlow), DataType::Any));
            }
            NodeKind::FileWrite | NodeKind::ClipboardCard => {
                inputs.push(Port::new(language::tr(Text::PortIn), DataType::Any));
            }
            NodeKind::Script | NodeKind::Clipboard => {
                inputs.push(Port::new(language::tr(Text::PortIn), DataType::Any));
                outputs.push(Port::new(language::tr(Text::PortOut), DataType::Any));
            }
        }
        (inputs, outputs)
    }
}

impl Node {
    pub fn new(kind: NodeKind, position: Point) -> Self {
        let (inputs, outputs) = kind.get_default_ports();
        Self {
            id: Uuid::new_v4(),
            kind: kind.clone(),
            position,
            inputs,
            outputs,
            title: kind.display_name().to_string(),
            properties: NodeProperties::new_for_kind(&kind),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: Uuid,
    pub source_node: Uuid,
    pub source_port: Uuid,
    pub target_node: Uuid,
    pub target_port: Uuid,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn remove_node(&mut self, id: Uuid) {
        if let Some(pos) = self.nodes.iter().position(|n| n.id == id) {
            self.nodes.remove(pos);
            self.edges
                .retain(|e| e.source_node != id && e.target_node != id);
        }
    }

    // pub fn remove_edge(&mut self, id: Uuid) {
    //     if let Some(pos) = self.edges.iter().position(|e| e.id == id) {
    //         self.edges.remove(pos);
    //     }
    // }

    pub fn find_node_by_port(&self, port_id: Uuid) -> Option<Uuid> {
        for node in &self.nodes {
            if node.inputs.iter().any(|p| p.id == port_id)
                || node.outputs.iter().any(|p| p.id == port_id)
            {
                return Some(node.id);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExecutionStatus {
    #[default]
    Idle,
    Running,
    Success,
    Error,
}

impl Port {
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            data_type,
        }
    }
}
