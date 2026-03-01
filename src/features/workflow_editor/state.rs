use super::types::{Graph, Node, NodeKind, Point};
use uuid::Uuid;

#[derive(Debug)]
pub struct WorkflowEditorState {
    pub workflow_id: Option<String>,
    pub name: String,
    pub graph: Graph,
    pub pan: Point,
    pub zoom: f32,
    pub selected_node: Option<Uuid>,
    pub dragging_node: Option<Uuid>,
    pub dragging_edge: Option<(Uuid, Point)>,
    pub context_menu: Option<Point>,
    pub node_context_menu: Option<(Uuid, Point)>,
    pub execution_log: Vec<String>,
    pub node_status: std::collections::HashMap<Uuid, super::types::ExecutionStatus>,
    pub inspector_node: Option<Uuid>,
    pub has_unsaved_changes: bool,
    pub save_indicator_phase: f32,
    pub available_scripts: Vec<crate::services::scripts::Script>,
    pub pending_side_effects: Vec<super::execution::SideEffect>,
}

impl Default for WorkflowEditorState {
    fn default() -> Self {
        Self {
            workflow_id: None,
            name: String::new(),
            graph: Graph::default(),
            pan: Point::default(),
            zoom: 1.0,
            selected_node: None,
            dragging_node: None,
            dragging_edge: None,
            context_menu: None,
            node_context_menu: None,
            execution_log: Vec::new(),
            node_status: std::collections::HashMap::new(),
            inspector_node: None,
            has_unsaved_changes: false,
            save_indicator_phase: 0.0,
            available_scripts: Vec::new(),
            pending_side_effects: Vec::new(),
        }
    }
}

impl WorkflowEditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, kind: NodeKind, position: Point) {
        self.graph.add_node(Node::new(kind, position));
    }

    pub fn remove_node(&mut self, id: Uuid) {
        self.graph.remove_node(id);
    }

    // pub fn remove_edge(&mut self, id: Uuid) {
    //     self.graph.remove_edge(id);
    // }
}
