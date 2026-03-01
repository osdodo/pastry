use super::types::{NodeKind, Point};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum WorkflowEditorMessage {
    NameChanged(String),
    Save,
    NodePressed(Uuid),
    PortPressed(Uuid, Point),
    PortReleased(Uuid),
    NodeMoved(Uuid, Point),
    CanvasPressed,
    CanvasReleased,
    CanvasPanned(Point),
    CanvasZoomed(f32, Point),
    AddNode(NodeKind, Point),
    RemoveNode(Uuid),
    NodeTitleEdited(Uuid, String),
    ShowContextMenu(Point),
    ShowNodeContextMenu(Uuid, Point),
    HideContextMenu,
    DisconnectPort(Uuid),
    DisconnectNode(Uuid),
    RunGraph,
    ClosePage,
    // Property editing messages
    HotkeyComboEdited(Uuid, String),
    ScriptIdEdited(Uuid, String),
    FileWritePathEdited(Uuid, String),
    FileWriteUseDesktop(Uuid),
    FileWriteUseDownloads(Uuid),
    FileWriteBrowseFolder(Uuid),
    // Inspector visibility toggle
    ToggleInspector(Uuid),
    HotkeyRecording(Uuid, iced::keyboard::Event),
    SaveIndicatorTick,
    StartDrag,
    NoOp,
}
