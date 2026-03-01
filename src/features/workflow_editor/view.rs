use iced::border::Radius;
use iced::mouse::{self, Cursor};
use iced::widget::canvas::{
    Action, Canvas, Frame, Geometry, Path, Program, Stroke, Text as CanvasText,
};
use iced::widget::{
    button, column, container, mouse_area, pick_list, row, stack, text as w_text, text_input,
};
use iced::{Color, Element, Length, Point, Rectangle, Size, Theme, Vector};
use std::path::PathBuf;
use uuid::Uuid;

use super::message::WorkflowEditorMessage;
use super::state::WorkflowEditorState;
use super::types;
use crate::platform::screen::get_window_height;
use crate::services::scripts;
use crate::ui::{
    constants::{WINDOW_MARGIN, WINDOW_WIDTH},
    language::{self, Text},
    theme::PastryTheme,
    util::ui_radius,
    widgets,
};

const NODE_WIDTH: f32 = 150.0;
const NODE_HEIGHT: f32 = 80.0;
const HEADER_HEIGHT: f32 = 30.0;
const PORT_RADIUS: f32 = 5.0;
const EDITOR_TOP_BAR_HEIGHT: f32 = 56.0;

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptOption {
    pub id: String,
    pub name: String,
}

impl std::fmt::Display for ScriptOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Default)]
struct InteractionState {
    last_cursor_pos: Option<Point>,
    is_panning: bool,
    is_dragging_node: bool,
}

struct WorkflowEditorProgram<'a> {
    state: &'a WorkflowEditorState,
}

impl<'a> Program<WorkflowEditorMessage> for WorkflowEditorProgram<'a> {
    type State = InteractionState;

    fn draw(
        &self,
        _interaction: &Self::State,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Background - transparent to show the page background
        // No background fill needed here as the page container already provides the background

        // Apply Pan and Zoom
        let pan = self.state.pan;
        let zoom = self.state.zoom;

        frame.push_transform();
        frame.translate(Vector::new(pan.x, pan.y));
        frame.scale(zoom);

        // Helper to find port position
        // get_port_pos is now a top-level function

        // Draw Edges
        for edge in &self.state.graph.edges {
            let start_node = self
                .state
                .graph
                .nodes
                .iter()
                .find(|n| n.id == edge.source_node);
            let end_node = self
                .state
                .graph
                .nodes
                .iter()
                .find(|n| n.id == edge.target_node);

            if let (Some(start), Some(end)) = (start_node, end_node) {
                // Find port indices
                let start_idx = start
                    .outputs
                    .iter()
                    .position(|p| p.id == edge.source_port)
                    .unwrap_or(0);
                let end_idx = end
                    .inputs
                    .iter()
                    .position(|p| p.id == edge.target_port)
                    .unwrap_or(0);

                let start_pos_node = Point::new(start.position.x, start.position.y);
                let end_pos_node = Point::new(end.position.x, end.position.y);

                let start_pos = get_port_pos(start_pos_node, false, start_idx);
                let end_pos = get_port_pos(end_pos_node, true, end_idx);

                draw_edge(&mut frame, start_pos, end_pos, theme.edge_color(), theme);
            }
        }

        // Draw Phantom Edge
        if let Some((start_port_id, end_mouse_pos)) = self.state.dragging_edge {
            // Find start port position
            let mut start_pos = Point::ORIGIN;
            for node in &self.state.graph.nodes {
                if let Some(idx) = node.outputs.iter().position(|p| p.id == start_port_id) {
                    start_pos =
                        get_port_pos(Point::new(node.position.x, node.position.y), false, idx);
                    break;
                }
                if let Some(idx) = node.inputs.iter().position(|p| p.id == start_port_id) {
                    start_pos =
                        get_port_pos(Point::new(node.position.x, node.position.y), true, idx);
                    break;
                }
            }
            // Convert internal Point to iced Point
            let end_pos = Point::new(end_mouse_pos.x, end_mouse_pos.y);
            draw_edge(
                &mut frame,
                start_pos,
                end_pos,
                Color::from_rgb(0.5, 0.5, 1.0),
                theme,
            );
        }

        // Draw Nodes
        for node in &self.state.graph.nodes {
            draw_node(
                &mut frame,
                node,
                self.state.selected_node == Some(node.id),
                self.state.node_status.get(&node.id).cloned(),
                &self.state.available_scripts,
                theme,
            );
        }

        frame.pop_transform();

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        interaction: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Action<WorkflowEditorMessage>> {
        let cursor_pos = cursor.position_in(bounds)?;

        let pan = self.state.pan;
        let zoom = self.state.zoom;
        let world_pos = Point::new((cursor_pos.x - pan.x) / zoom, (cursor_pos.y - pan.y) / zoom);

        match event {
            iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let scroll_amount = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * 20.0,
                    mouse::ScrollDelta::Pixels { y, .. } => *y,
                };
                let new_zoom = (zoom * (1.0 + scroll_amount * 0.001)).clamp(0.5_f32, 5.0_f32);
                let logic_cursor_pos = types::Point {
                    x: cursor_pos.x,
                    y: cursor_pos.y,
                };
                return Some(Action::publish(WorkflowEditorMessage::CanvasZoomed(
                    new_zoom,
                    logic_cursor_pos,
                )));
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                interaction.last_cursor_pos = Some(cursor_pos);

                // 2. Check Ports Second
                for node in self.state.graph.nodes.iter().rev() {
                    let node_pos = Point::new(node.position.x, node.position.y);

                    // Inputs
                    for (i, port) in node.inputs.iter().enumerate() {
                        let port_y = node_pos.y + HEADER_HEIGHT + 20.0 + (i as f32 * 20.0);
                        let port_center = Point::new(node_pos.x, port_y);
                        if port_center.distance(world_pos) <= PORT_RADIUS * 2.0 {
                            // Larger Hitbox
                            return Some(Action::publish(WorkflowEditorMessage::PortPressed(
                                port.id,
                                types::Point {
                                    x: port_center.x,
                                    y: port_center.y,
                                },
                            )));
                        }
                    }
                    // Outputs
                    for (i, port) in node.outputs.iter().enumerate() {
                        let port_y = node_pos.y + HEADER_HEIGHT + 20.0 + (i as f32 * 20.0);
                        let port_center = Point::new(node_pos.x + NODE_WIDTH, port_y);
                        if port_center.distance(world_pos) <= PORT_RADIUS * 2.0 {
                            return Some(Action::publish(WorkflowEditorMessage::PortPressed(
                                port.id,
                                types::Point {
                                    x: port_center.x,
                                    y: port_center.y,
                                },
                            )));
                        }
                    }
                }

                // 3. Check Node Body
                for node in self.state.graph.nodes.iter().rev() {
                    let node_rect = Rectangle::new(
                        Point::new(node.position.x, node.position.y),
                        Size::new(NODE_WIDTH, NODE_HEIGHT),
                    );
                    if node_rect.contains(world_pos) {
                        interaction.is_dragging_node = true;
                        interaction.is_panning = false;
                        return Some(Action::publish(WorkflowEditorMessage::NodePressed(node.id)));
                    }
                }

                interaction.is_panning = true;
                interaction.is_dragging_node = false;
                return Some(Action::publish(WorkflowEditorMessage::CanvasPressed));
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                interaction.is_panning = false;
                interaction.is_dragging_node = false;
                interaction.last_cursor_pos = None;

                // If dragging edge, check if we dropped on a port
                if self.state.dragging_edge.is_some() {
                    for node in self.state.graph.nodes.iter() {
                        let node_pos = Point::new(node.position.x, node.position.y);
                        for (i, port) in node.inputs.iter().enumerate() {
                            let port_y = node_pos.y + HEADER_HEIGHT + 20.0 + (i as f32 * 20.0);
                            let port_center = Point::new(node_pos.x, port_y);
                            if port_center.distance(world_pos) <= PORT_RADIUS * 2.0 {
                                return Some(Action::publish(WorkflowEditorMessage::PortReleased(
                                    port.id,
                                )));
                            }
                        }
                        for (i, port) in node.outputs.iter().enumerate() {
                            let port_y = node_pos.y + HEADER_HEIGHT + 20.0 + (i as f32 * 20.0);
                            let port_center = Point::new(node_pos.x + NODE_WIDTH, port_y);
                            if port_center.distance(world_pos) <= PORT_RADIUS * 2.0 {
                                return Some(Action::publish(WorkflowEditorMessage::PortReleased(
                                    port.id,
                                )));
                            }
                        }
                    }
                }

                return Some(Action::publish(WorkflowEditorMessage::CanvasReleased));
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // Always update mouse pos if dragging edge
                if self.state.dragging_edge.is_some() {
                    let p = types::Point {
                        x: world_pos.x,
                        y: world_pos.y,
                    };
                    return Some(Action::publish(WorkflowEditorMessage::NodeMoved(
                        Uuid::nil(),
                        p,
                    )));
                }

                if let Some(last_pos) = interaction.last_cursor_pos {
                    let delta = cursor_pos - last_pos;
                    interaction.last_cursor_pos = Some(cursor_pos);

                    if interaction.is_panning {
                        return Some(Action::publish(WorkflowEditorMessage::CanvasPanned(
                            types::Point {
                                x: delta.x,
                                y: delta.y,
                            },
                        )));
                    } else if interaction.is_dragging_node
                        && let Some(id) = self.state.selected_node
                        && let Some(node) = self.state.graph.nodes.iter().find(|n| n.id == id)
                    {
                        let world_delta = delta * (1.0 / zoom);
                        let new_pos = types::Point {
                            x: node.position.x + world_delta.x,
                            y: node.position.y + world_delta.y,
                        };
                        return Some(Action::publish(WorkflowEditorMessage::NodeMoved(
                            id, new_pos,
                        )));
                    }
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                // Check if we clicked on a port first
                for node in &self.state.graph.nodes {
                    let node_pos = Point::new(node.position.x, node.position.y);
                    // Inputs
                    for (i, port) in node.inputs.iter().enumerate() {
                        let port_pos = get_port_pos(node_pos, true, i);
                        if port_pos.distance(world_pos) <= PORT_RADIUS + 2.0 {
                            return Some(Action::publish(WorkflowEditorMessage::DisconnectPort(
                                port.id,
                            )));
                        }
                    }
                    // Outputs
                    for (i, port) in node.outputs.iter().enumerate() {
                        let port_pos = get_port_pos(node_pos, false, i);
                        if port_pos.distance(world_pos) <= PORT_RADIUS + 2.0 {
                            return Some(Action::publish(WorkflowEditorMessage::DisconnectPort(
                                port.id,
                            )));
                        }
                    }
                }

                // Check if we clicked on a node
                for node in self.state.graph.nodes.iter().rev() {
                    let node_rect = Rectangle::new(
                        Point::new(node.position.x, node.position.y),
                        Size::new(NODE_WIDTH, NODE_HEIGHT),
                    );
                    if node_rect.contains(world_pos) {
                        return Some(Action::publish(WorkflowEditorMessage::ShowNodeContextMenu(
                            node.id,
                            types::Point {
                                x: world_pos.x,
                                y: world_pos.y,
                            },
                        )));
                    }
                }

                return Some(Action::publish(WorkflowEditorMessage::ShowContextMenu(
                    types::Point {
                        x: world_pos.x,
                        y: world_pos.y,
                    },
                )));
            }
            _ => {}
        }
        None
    }
}

fn draw_node(
    frame: &mut Frame,
    node: &types::Node,
    is_selected: bool,
    status: Option<types::ExecutionStatus>,
    available_scripts: &[scripts::Script],
    theme: &Theme,
) {
    let position = Point::new(node.position.x, node.position.y);

    // Node Shadow (subtle glow if selected)
    if is_selected {
        let shadow_rect = Path::rounded_rectangle(
            position - Vector::new(2.0, 2.0),
            Size::new(NODE_WIDTH + 4.0, NODE_HEIGHT + 4.0),
            Radius::from(ui_radius(10.0)),
        );
        let mut shadow_color = theme.primary();
        shadow_color.a = 0.2;
        frame.fill(&shadow_rect, shadow_color);
    }

    // Node Body
    let node_rect = Path::rounded_rectangle(
        position,
        Size::new(NODE_WIDTH, NODE_HEIGHT),
        Radius::from(ui_radius(12.0)),
    );
    frame.fill(&node_rect, theme.node_bg());

    // Kind Color (used for ports)
    let kind_color = match node.kind {
        types::NodeKind::Clipboard => Color::from_rgb(0.2, 0.8, 0.8),
        _ => Color::from_rgb(0.4, 0.6, 1.0),
    };

    // Border
    let border_color = if is_selected {
        theme.primary()
    } else {
        theme.border_subtle()
    };

    frame.stroke(
        &node_rect,
        Stroke::default()
            .with_color(border_color)
            .with_width(if is_selected { 2.0 } else { 1.0 }),
    );

    // Execution Progress
    if let Some(status) = status {
        let color = match status {
            types::ExecutionStatus::Running => Color::from_rgb(1.0, 1.0, 0.0),
            types::ExecutionStatus::Success => Color::from_rgb(0.0, 1.0, 0.0),
            types::ExecutionStatus::Error => Color::from_rgb(1.0, 0.0, 0.0),
            _ => Color::TRANSPARENT,
        };

        if color != Color::TRANSPARENT {
            let bar_height = 3.0;
            let bar_margin = 12.0;
            let status_rect = Path::rounded_rectangle(
                position + Vector::new(bar_margin, NODE_HEIGHT - 12.0),
                Size::new(NODE_WIDTH - bar_margin * 2.0, bar_height),
                Radius::from(ui_radius(bar_height / 2.0)),
            );
            frame.fill(&status_rect, color);
        }
    }

    // Title
    frame.fill_text(CanvasText {
        content: node.title.clone(),
        position: position + Vector::new(12.0, 12.0),
        color: theme.node_title(),
        size: 13.0.into(),
        ..Default::default()
    });

    if let Some(summary) = node_summary_text(node, available_scripts) {
        frame.fill_text(CanvasText {
            content: ellipsize(&summary, 24),
            position: position + Vector::new(12.0, 32.0),
            color: theme.text_secondary(),
            size: 10.0.into(),
            ..Default::default()
        });
    }

    // Ports
    for (i, _) in node.inputs.iter().enumerate() {
        draw_port(frame, position, true, i, None, theme);
    }
    for (i, _) in node.outputs.iter().enumerate() {
        draw_port(frame, position, false, i, Some(kind_color), theme);
    }
}

fn node_summary_text(node: &types::Node, available_scripts: &[scripts::Script]) -> Option<String> {
    match node.kind {
        types::NodeKind::Hotkey => {
            let value = node
                .properties
                .hotkey_combo
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("—");
            Some(format!("{}: {}", language::tr(Text::Hotkey), value))
        }
        types::NodeKind::Script => {
            let value = node
                .properties
                .script_id
                .as_deref()
                .and_then(|id| available_scripts.iter().find(|s| s.id == id))
                .map(scripts::localized_display_name)
                .or_else(|| node.properties.script_id.clone())
                .unwrap_or_else(|| "—".to_string());
            Some(format!("{}: {}", language::tr(Text::Script), value))
        }
        types::NodeKind::Clipboard => {
            let value = node
                .properties
                .clipboard_action
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("—");
            Some(value.to_string())
        }
        types::NodeKind::FileWrite => {
            let value = node
                .properties
                .file_write_path
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("—");
            Some(format!("{}: {}", language::tr(Text::FilePath), value))
        }
        types::NodeKind::ClipboardCard => None,
    }
}

fn ellipsize(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }

    let keep = max_chars.saturating_sub(1);
    let mut result = text.chars().take(keep).collect::<String>();
    result.push('…');
    result
}

fn draw_port(
    frame: &mut Frame,
    node_pos: Point,
    is_input: bool,
    index: usize,
    color: Option<Color>,
    theme: &Theme,
) {
    let port_pos = get_port_pos(node_pos, is_input, index);
    let port_circle = Path::circle(port_pos, PORT_RADIUS);
    let inner_circle = Path::circle(port_pos, PORT_RADIUS * 0.5);

    frame.fill(&port_circle, theme.grid_dot());

    let port_stroke_color = color.unwrap_or_else(|| theme.port_stroke());
    frame.stroke(
        &port_circle,
        Stroke::default()
            .with_color(port_stroke_color)
            .with_width(1.0),
    );

    let inner_color = color.unwrap_or_else(|| theme.port_inner());
    frame.fill(&inner_circle, inner_color);
}

fn draw_edge(frame: &mut Frame, start: Point, end: Point, color: Color, theme: &Theme) {
    let path = Path::new(|p| {
        p.move_to(start);
        let dist = (end.x - start.x).abs().max(10.0) * 0.5;
        let c1 = Point::new(start.x + dist, start.y);
        let c2 = Point::new(end.x - dist, end.y);
        p.bezier_curve_to(c1, c2, end);
    });

    // Draw a subtle background line for depth
    frame.stroke(
        &path,
        Stroke::default()
            .with_color(theme.edge_shadow())
            .with_width(4.5),
    );

    frame.stroke(&path, Stroke::default().with_color(color).with_width(2.0));
}

/// Helper function to create a property input field
fn property_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    node_id: Uuid,
    message_fn: impl Fn(Uuid, String) -> WorkflowEditorMessage + 'a,
) -> Element<'a, WorkflowEditorMessage> {
    column![
        w_text(label)
            .size(11)
            .font(iced::Font::with_name("Inter"))
            .style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.text_secondary()),
            }),
        text_input(placeholder, value)
            .on_input(move |s| message_fn(node_id, s))
            // This is a bit of a hack: text_input doesn't emit key events directly.
            // But we can listen for general key events if we know this input is focused.
            // However, iced doesn't expose focus state easily to general subscriptions.
            // A better way for hotkey recording:
            // Instead of a text input, use a custom widget or a button that enters "recording mode".
            // For now, let's keep it simple: if the user types here, we trust them.
            // IF the user wants auto-record, we should replace this with a specialized widget.
            .padding(10)
            .size(13)
            .style(|theme: &Theme, status| {
                let mut style = iced::widget::text_input::default(theme, status);
                style.border.radius = ui_radius(10.0).into();
                style.border.width = 1.0;
                style.border.color = theme.input_border();
                style.background = iced::Background::Color(theme.dialog_background());
                style.value = theme.text();
                style.placeholder = theme.text_placeholder();
                if matches!(status, iced::widget::text_input::Status::Focused { .. }) {
                    style.border.color = theme.primary();
                    style.background = iced::Background::Color(theme.dialog_background());
                }
                style
            }),
    ]
    .spacing(6)
    .into()
}

fn quick_path_button<'a>(
    label: &'a str,
    message: WorkflowEditorMessage,
) -> Element<'a, WorkflowEditorMessage> {
    button(w_text(label).size(11))
        .on_press(message)
        .padding([4, 10])
        .style(|theme: &Theme, status| {
            let mut style = button::Style {
                background: Some(iced::Background::Color(theme.button_background())),
                text_color: theme.text(),
                border: iced::Border {
                    radius: ui_radius(8.0).into(),
                    width: 1.0,
                    color: theme.input_border(),
                },
                ..Default::default()
            };

            if matches!(status, iced::widget::button::Status::Hovered) {
                style.background = Some(iced::Background::Color(theme.hover_bg()));
            }

            style
        })
        .into()
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

fn file_write_path_hint(raw_path: &str) -> (Text, bool) {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return (Text::FilePathHintEmpty, true);
    }

    if trimmed == "~" || trimmed.ends_with('/') || trimmed.ends_with('\\') {
        return (Text::FilePathHintLooksLikeFolder, true);
    }

    let resolved = resolve_file_write_path(trimmed);
    let parent = resolved
        .parent()
        .filter(|value| !value.as_os_str().is_empty());
    if let Some(parent_dir) = parent
        && !parent_dir.exists()
    {
        return (Text::FilePathHintParentMissing, true);
    }

    (Text::FilePathHintValid, false)
}

/// Add node-specific properties to the inspector
fn add_node_properties<'a>(
    node: &'a types::Node,
    node_id: Uuid,
    available_scripts: &'a [scripts::Script],
) -> Vec<Element<'a, WorkflowEditorMessage>> {
    let props = &node.properties;

    let prop_elements: Vec<Element<'a, WorkflowEditorMessage>> = match node.kind {
        types::NodeKind::Hotkey => {
            let _is_recording = false; // We need state for this. For now let's just make it a "Click to Record" button that we can't fully implement without more state

            // To properly implement this, we need to know WHICH node is currently recording.
            // But we don't have that field in state exposed here easily without passing it down?
            // Wait, we generate the view based on `state.inspector_node`.
            // We can add a `recording_hotkey_node: Option<Uuid>` to WorkflowEditorState?
            // For now, let's just use a special style of text input that traps keys?
            // Actually, Iced 0.13 removed keyboard subscription from widgets easily.
            // We need to use a subscription at the top level and route events.

            // Let's rely on the global subscription.
            // We will add a "Record" button. When clicked, it sets a state `recording_hotkey_node = Some(id)`.
            // Then the global subscription listens for keys and sends `HotkeyRecording`.

            vec![
                column![
                    w_text(language::tr(Text::Hotkey))
                        .size(11)
                        .font(iced::Font::with_name("Inter"))
                        .style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.text_secondary()),
                        }),
                    text_input(
                        language::tr(Text::ClickToRecord),
                        props.hotkey_combo.as_deref().unwrap_or("")
                    )
                    .on_input(move |s| WorkflowEditorMessage::HotkeyComboEdited(node_id, s)) // fallback
                    // We can't easily detect focus here to start recording.
                    // So instead, let's keep the text input for manual entry,
                    // but maybe adding a record button next to it?
                    .padding(10)
                    .size(13)
                    .style(|theme: &Theme, status| {
                        let mut style = iced::widget::text_input::default(theme, status);
                        style.border.radius = ui_radius(10.0).into();
                        style.border.width = 1.0;
                        style.border.color = theme.input_border();
                        style.background = iced::Background::Color(theme.dialog_background());
                        style.value = theme.text();
                        style.placeholder = theme.text_placeholder();
                        if matches!(status, iced::widget::text_input::Status::Focused { .. }) {
                            style.border.color = theme.primary();
                            style.background = iced::Background::Color(theme.dialog_background());
                        }
                        style
                    }),
                    w_text(language::tr(Text::HotkeyRecordHint))
                        .size(10)
                        .style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.text_secondary()),
                        }),
                ]
                .spacing(6)
                .into(),
            ]
        }
        types::NodeKind::Script => {
            let options: Vec<ScriptOption> = available_scripts
                .iter()
                .map(|s| ScriptOption {
                    id: s.id.clone(),
                    name: scripts::localized_display_name(s),
                })
                .collect();

            let selected_option = props
                .script_id
                .as_ref()
                .and_then(|id| options.iter().find(|o| o.id == *id).cloned());

            vec![
                column![
                    w_text(language::tr(Text::Script))
                        .size(11)
                        .font(iced::Font::with_name("Inter"))
                        .style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.text_secondary()),
                        }),
                    pick_list(options, selected_option, move |selected| {
                        WorkflowEditorMessage::ScriptIdEdited(node_id, selected.id)
                    })
                    .padding(10)
                    .width(Length::Fill)
                    .text_size(12)
                    .style(|theme: &Theme, status| {
                        let mut style = iced::widget::pick_list::default(theme, status);
                        style.border.radius = ui_radius(10.0).into();
                        style.border.width = 1.0;
                        style.border.color = theme.input_border();
                        style.background = iced::Background::Color(theme.dialog_background());
                        style
                    })
                    .menu_style(|theme: &Theme| iced::overlay::menu::Style {
                        text_color: theme.text(),
                        background: iced::Background::Color(theme.dialog_background()),
                        border: iced::Border {
                            width: 1.0,
                            color: theme.divider(),
                            radius: ui_radius(12.0).into(),
                        },
                        selected_text_color: Color::WHITE,
                        selected_background: iced::Background::Color(theme.primary()),
                        shadow: iced::Shadow::default(),
                    }),
                ]
                .spacing(6)
                .into(),
            ]
        }

        types::NodeKind::Clipboard => vec![],
        types::NodeKind::FileWrite => {
            let file_path = props.file_write_path.as_deref().unwrap_or("");
            let (path_hint_text, has_path_issue) = file_write_path_hint(file_path);
            vec![
                column![
                    w_text(language::tr(Text::QuickPath))
                        .size(11)
                        .font(iced::Font::with_name("Inter"))
                        .style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.text_secondary()),
                        }),
                    row![
                        quick_path_button(
                            language::tr(Text::Desktop),
                            WorkflowEditorMessage::FileWriteUseDesktop(node_id),
                        ),
                        quick_path_button(
                            language::tr(Text::Downloads),
                            WorkflowEditorMessage::FileWriteUseDownloads(node_id),
                        ),
                        quick_path_button(
                            language::tr(Text::BrowseFolder),
                            WorkflowEditorMessage::FileWriteBrowseFolder(node_id),
                        ),
                    ]
                    .spacing(8),
                    property_input(
                        language::tr(Text::FilePath),
                        language::tr(Text::FilePathPlaceholder),
                        file_path,
                        node_id,
                        WorkflowEditorMessage::FileWritePathEdited,
                    ),
                    w_text(language::tr(path_hint_text))
                        .size(10)
                        .style(move |theme: &Theme| iced::widget::text::Style {
                            color: Some(if has_path_issue {
                                theme.danger()
                            } else {
                                theme.success()
                            }),
                        }),
                ]
                .spacing(6)
                .into(),
            ]
        }
        types::NodeKind::ClipboardCard => vec![],
    };

    if prop_elements.is_empty() {
        vec![]
    } else {
        prop_elements
    }
}

pub fn workflow_editor_view<'a>(
    state: &'a WorkflowEditorState,
) -> Element<'a, WorkflowEditorMessage> {
    let canvas: Element<WorkflowEditorMessage> = Canvas::new(WorkflowEditorProgram { state })
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    // Logs Panel
    let logs_panel: Element<WorkflowEditorMessage> = if !state.execution_log.is_empty() {
        let logs: Element<WorkflowEditorMessage> = column(
            state
                .execution_log
                .iter()
                .map(|log| {
                    w_text(log)
                        .size(12)
                        .style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.text_secondary()),
                        })
                        .into()
                })
                .collect::<Vec<Element<WorkflowEditorMessage>>>(),
        )
        .spacing(4)
        .into();

        mouse_area(
            container(
                column![
                    w_text(language::tr(Text::ExecutionLogs))
                        .size(14)
                        .style(|theme: &Theme| {
                            iced::widget::text::Style {
                                color: Some(theme.text()),
                            }
                        }),
                    iced::widget::scrollable(logs).height(Length::Fill)
                ]
                .spacing(12),
            )
            .width(300)
            .height(400)
            .padding(16)
            .style(|theme: &Theme| container::Style {
                background: Some(iced::Background::Color(theme.dialog_background())),
                border: iced::Border {
                    color: theme.divider(),
                    radius: ui_radius(16.0).into(),
                    width: 1.0,
                },
                shadow: iced::Shadow {
                    color: theme.shadow(),
                    offset: Vector::new(0.0, 10.0),
                    blur_radius: 30.0,
                },
                ..Default::default()
            }),
        )
        .on_press(WorkflowEditorMessage::NoOp)
        .into()
    } else {
        iced::widget::Space::new().width(0).height(0).into()
    };

    // Stack structure:
    // Base: Canvas
    // Overlay 1: Inspector (Right)
    // Overlay 2: Run Button (Bottom Right, inside Log panel path or separate?)
    // Overlay 3: Logs Panel (Bottom)
    // Overlay 4: Context Menu (Absolute)

    // Let's restructure
    // Base is canvas
    // Then we use a generic overlay column/row structure?
    // Using Stack is easiest.

    // If inspector is there, it needs to be processed.
    // Ideally, Inspector is "Right Dock", Logs is "Bottom Dock".
    // Canvas is behind everything.

    // Re-doing the layout composition
    let canvas_layer = canvas;

    // UI Layer (Inspector + Logs + Button)
    // We can use a Column for the main UI structure (Top: Space, Bottom: Logs)
    // And a Row for (Left: Space, Right: Inspector)

    // But they overlap...
    // Let's stick to Stack.

    let inspector_layer: Element<WorkflowEditorMessage> = if let Some(selected_id) =
        state.inspector_node
    {
        if let Some(node) = state.graph.nodes.iter().find(|n| n.id == selected_id) {
            let icon = match node.kind {
                types::NodeKind::Hotkey => widgets::Icon::Command,
                types::NodeKind::FileWrite => widgets::Icon::FileWrite,
                types::NodeKind::ClipboardCard => widgets::Icon::Clipboard,
                _ => widgets::Icon::Code,
            };

            let mut inspector_content = column![
                // Header: Icon + (Title Input & Node Kind) + Close Button
                row![
                    container(widgets::icon_svg(icon, 14, |theme| theme.primary()))
                        .padding(6)
                        .style(|_| container::Style {
                            background: Some(iced::Background::Color(Color::from_rgba(
                                1.0, 1.0, 1.0, 0.03
                            ))),
                            border: iced::Border {
                                radius: ui_radius(8.0).into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    column![
                        text_input(language::tr(Text::NodeTitle), &node.title)
                            .on_input(move |s| WorkflowEditorMessage::NodeTitleEdited(
                                selected_id,
                                s
                            ))
                            .size(14)
                            .width(Length::Fill)
                            .style(|theme: &Theme, status| {
                                let mut style = iced::widget::text_input::default(theme, status);
                                style.background = iced::Background::Color(Color::TRANSPARENT);
                                style.border.width = 0.0;
                                style.value = theme.text();
                                style.placeholder = theme.text_placeholder();
                                style
                            })
                    ]
                    .width(Length::Fill)
                    .spacing(0),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
                container(iced::widget::Space::new())
                    .width(Length::Fill)
                    .height(1)
                    .style(|theme: &Theme| container::Style {
                        background: Some(iced::Background::Color(theme.divider())),
                        ..Default::default()
                    }),
            ]
            .spacing(12);

            if matches!(node.kind, types::NodeKind::Clipboard) {
                inspector_content = inspector_content.push(
                    w_text(language::tr(Text::ClipboardNodeDescription))
                        .size(11)
                        .font(iced::Font::with_name("Inter"))
                        .style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.text_secondary()),
                        }),
                );
            }

            // Add node-specific properties
            let prop_elements = add_node_properties(node, selected_id, &state.available_scripts);

            // Build the inspector content with properties
            let inspector_with_props: Element<'_, WorkflowEditorMessage> =
                if prop_elements.is_empty() {
                    inspector_content.into()
                } else {
                    let mut col = inspector_content.push(iced::widget::Space::new().height(1));

                    for el in prop_elements {
                        col = col.push(el).push(iced::widget::Space::new().height(4));
                    }
                    col.into()
                };

            mouse_area(
                container(
                    container(inspector_with_props)
                        .width(300)
                        .height(Length::Shrink),
                )
                .width(300)
                .padding(16)
                .style(|theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(theme.dialog_background())),
                    border: iced::Border {
                        width: 1.0,
                        color: theme.divider(),
                        radius: ui_radius(20.0).into(),
                    },
                    shadow: iced::Shadow {
                        color: theme.shadow(),
                        offset: Vector::new(0.0, 10.0),
                        blur_radius: 30.0,
                    },
                    ..Default::default()
                }),
            )
            .on_press(WorkflowEditorMessage::NoOp)
            .into()
        } else {
            iced::widget::Space::new().width(0).height(0).into()
        }
    } else {
        iced::widget::Space::new().width(0).height(0).into()
    };

    let run_button = button(
        row![
            widgets::icon_svg(widgets::Icon::Code, 16, |_| { Color::WHITE }),
            w_text(language::tr(Text::Run)).size(14)
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .on_press(WorkflowEditorMessage::RunGraph)
    .padding([10, 24])
    .style(|theme: &Theme, _| button::Style {
        background: Some(iced::Background::Color(theme.primary())),
        text_color: Color::WHITE,
        border: iced::Border {
            radius: ui_radius(20.0).into(), // Pill shape
            ..Default::default()
        },
        ..Default::default()
    });

    let ui_overlay = row![
        iced::widget::Space::new().width(Length::Fill),
        column![
            inspector_layer,
            iced::widget::Space::new().height(Length::Fill),
            logs_panel,
            container(run_button)
                .width(Length::Fill)
                .align_x(iced::Alignment::End)
        ]
        .spacing(16)
        .width(300)
    ]
    .padding(20);

    let content_el: Element<WorkflowEditorMessage> = stack![canvas_layer, ui_overlay].into();

    let mut root = stack![content_el];

    if let Some(pos) = state.context_menu {
        let screen_x = pos.x * state.zoom + state.pan.x;
        let screen_y = pos.y * state.zoom + state.pan.y;

        let menu_items = vec![
            (types::NodeKind::Hotkey, widgets::Icon::Command),
            (types::NodeKind::Clipboard, widgets::Icon::Clipboard),
            (types::NodeKind::Script, widgets::Icon::CodeBrackets),
            (types::NodeKind::FileWrite, widgets::Icon::FileWrite),
            (types::NodeKind::ClipboardCard, widgets::Icon::Clipboard),
        ];

        let mut menu_content = column(vec![]).spacing(2);

        // Initial Triggers label
        menu_content = menu_content.push(
            container(
                w_text(language::tr(Text::Triggers))
                    .size(10)
                    .font(iced::Font::with_name("Inter"))
                    .style(|theme: &Theme| iced::widget::text::Style {
                        color: Some(theme.text_secondary()),
                    }),
            )
            .padding([8, 12]),
        );

        let mut current_group = 0; // 0: Trigger, 1: Data, 2: Transform, 3: Output

        for (kind, icon) in menu_items {
            let item_group = if kind.is_trigger() {
                0
            } else if matches!(kind, types::NodeKind::Clipboard) {
                1
            } else if matches!(kind, types::NodeKind::Script) {
                2
            } else {
                3
            };

            if item_group > current_group {
                let label = match item_group {
                    1 => language::tr(Text::DataSource),
                    2 => language::tr(Text::Transform),
                    _ => language::tr(Text::OutputGroup),
                };

                // Add separator
                menu_content = menu_content.push(
                    container(iced::widget::Space::new().width(Length::Fill).height(0.5))
                        .padding([1, 0])
                        .style(|theme: &Theme| container::Style {
                            background: Some(iced::Background::Color(theme.divider())),
                            ..Default::default()
                        }),
                );

                // Add Label
                menu_content = menu_content.push(
                    container(
                        w_text(label)
                            .size(10)
                            .font(iced::Font::with_name("Inter"))
                            .style(|theme: &Theme| iced::widget::text::Style {
                                color: Some(theme.text_secondary()),
                            }),
                    )
                    .padding([8, 12]),
                );
                current_group = item_group;
            }

            let label = kind.display_name();
            menu_content = menu_content.push(
                button(
                    row![
                        widgets::icon_svg(icon, 14, |theme| theme.text_secondary()),
                        w_text(label)
                            .size(13)
                            .style(|theme: &Theme| iced::widget::text::Style {
                                color: Some(theme.text()),
                            }),
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center),
                )
                .on_press(WorkflowEditorMessage::AddNode(kind, pos))
                .width(Length::Fill)
                .style(|theme: &Theme, status| {
                    let mut style = iced::widget::button::text(theme, status);
                    if matches!(status, iced::widget::button::Status::Hovered) {
                        style.background = Some(iced::Background::Color(theme.hover_bg()));
                        style.border.radius = ui_radius(6.0).into();
                    }
                    style
                })
                .padding([8, 12]),
            );
        }

        root = root.push(render_context_menu(
            menu_content.width(180),
            screen_x,
            screen_y,
            188.0,
            340.0,
        ));
    }

    if let Some((node_id, pos)) = state.node_context_menu {
        let screen_x = pos.x * state.zoom + state.pan.x;
        let screen_y = pos.y * state.zoom + state.pan.y;

        let menu_col = column![
            button(
                row![
                    widgets::icon_svg(widgets::Icon::Editor, 14, |theme| theme.text_secondary()),
                    w_text(language::tr(Text::EditDetails))
                        .size(13)
                        .style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.text()),
                        }),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            )
            .on_press(WorkflowEditorMessage::ToggleInspector(node_id))
            .width(Length::Fill)
            .style(|theme: &Theme, status| {
                let mut style = iced::widget::button::text(theme, status);
                if matches!(status, iced::widget::button::Status::Hovered) {
                    style.background = Some(iced::Background::Color(theme.hover_bg()));
                    style.border.radius = ui_radius(6.0).into();
                }
                style
            })
            .padding([8, 12]),
            button(
                row![
                    widgets::icon_svg(widgets::Icon::Delete, 14, |theme| theme.danger()),
                    w_text(language::tr(Text::DeleteNode))
                        .size(13)
                        .style(|theme: &Theme| {
                            iced::widget::text::Style {
                                color: Some(theme.danger()),
                            }
                        }),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            )
            .on_press(WorkflowEditorMessage::RemoveNode(node_id))
            .width(Length::Fill)
            .style(|theme: &Theme, status| {
                let mut style = iced::widget::button::text(theme, status);
                if matches!(status, iced::widget::button::Status::Hovered) {
                    style.background = Some(iced::Background::Color(theme.hover_bg()));
                    style.border.radius = ui_radius(6.0).into();
                }
                style
            })
            .padding([8, 12]),
            button(
                row![
                    widgets::icon_svg(widgets::Icon::Back, 14, |theme| theme.text_secondary()),
                    w_text(language::tr(Text::ClearConnections))
                        .size(13)
                        .style(|theme: &Theme| iced::widget::text::Style {
                            color: Some(theme.text()),
                        }),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            )
            .on_press(WorkflowEditorMessage::DisconnectNode(node_id))
            .width(Length::Fill)
            .style(|theme: &Theme, status| {
                let mut style = iced::widget::button::text(theme, status);
                if matches!(status, iced::widget::button::Status::Hovered) {
                    style.background = Some(iced::Background::Color(theme.hover_bg()));
                    style.border.radius = ui_radius(6.0).into();
                }
                style
            })
            .padding([8, 12]),
        ]
        .width(160)
        .spacing(2);

        root = root.push(render_context_menu(
            menu_col, screen_x, screen_y, 168.0, 150.0,
        ));
    }

    root.into()
}

fn render_context_menu<'a>(
    content: impl Into<Element<'a, WorkflowEditorMessage>>,
    screen_x: f32,
    screen_y: f32,
    menu_width: f32,
    menu_height: f32,
) -> Element<'a, WorkflowEditorMessage> {
    let window_height = get_window_height(WINDOW_MARGIN);
    let editor_height = (window_height - EDITOR_TOP_BAR_HEIGHT).max(120.0);
    let edge_gap = 10.0;
    let max_x = (WINDOW_WIDTH - menu_width - edge_gap).max(edge_gap);
    let max_y = (editor_height - menu_height - edge_gap).max(edge_gap);
    let clamped_x = screen_x.clamp(edge_gap, max_x);
    let clamped_y = screen_y.clamp(edge_gap, max_y);

    let menu = container(content)
        .width(Length::Fixed(menu_width))
        .padding(4)
        .style(|theme: &Theme| container::Style {
            background: Some(iced::Background::Color(theme.dialog_background())),
            border: iced::Border {
                width: 1.0,
                color: theme.divider(),
                radius: ui_radius(12.0).into(),
            },
            shadow: iced::Shadow {
                color: theme.shadow(),
                offset: Vector::new(0.0, 10.0),
                blur_radius: 25.0,
            },
            ..Default::default()
        });

    let backdrop = button(
        container(iced::widget::Space::new())
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(WorkflowEditorMessage::HideContextMenu)
    .style(|_, _| button::Style {
        background: None,
        ..Default::default()
    })
    .width(Length::Fill)
    .height(Length::Fill);

    let menu_layer = container(menu)
        .padding(iced::Padding {
            top: clamped_y,
            left: clamped_x,
            bottom: 0.0,
            right: 0.0,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::Alignment::Start)
        .align_y(iced::Alignment::Start);

    stack![backdrop, menu_layer].into()
}

fn get_port_pos(node_pos: Point, is_input: bool, index: usize) -> Point {
    let port_y = node_pos.y + HEADER_HEIGHT + 20.0 + (index as f32 * 20.0);
    let x = if is_input {
        node_pos.x
    } else {
        node_pos.x + NODE_WIDTH
    };
    Point::new(x, port_y)
}

fn build_save_button(
    has_unsaved_changes: bool,
    save_indicator_phase: f32,
) -> Element<'static, WorkflowEditorMessage> {
    let button_content: Element<'static, WorkflowEditorMessage> = if has_unsaved_changes {
        let pulse = (save_indicator_phase.sin() + 1.0) * 0.5;
        let mut dot_color = iced::Color::from_rgb8(255, 107, 107);
        dot_color.a = 0.35 + 0.65 * pulse;
        let dot_size = 8.0 + 3.0 * pulse;

        row![
            w_text(language::tr(Text::Save)).size(13),
            w_text("●")
                .size(dot_size)
                .style(move |_theme: &Theme| iced::widget::text::Style {
                    color: Some(dot_color),
                })
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
    } else {
        w_text(language::tr(Text::Save)).size(13).into()
    };

    button(button_content)
        .on_press(WorkflowEditorMessage::Save)
        .padding([6, 14])
        .style(move |theme: &Theme, status| {
            if has_unsaved_changes {
                let mut bg = theme.primary();
                bg.a = if matches!(status, button::Status::Hovered) {
                    0.26
                } else {
                    0.18
                };

                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: theme.text(),
                    border: iced::Border {
                        radius: ui_radius(6.0).into(),
                        width: 1.0,
                        color: theme.primary(),
                    },
                    ..Default::default()
                }
            } else {
                let mut bg = theme.button_background();
                bg.a = if matches!(status, button::Status::Hovered) {
                    0.35
                } else {
                    0.18
                };

                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: if matches!(status, button::Status::Hovered) {
                        theme.text()
                    } else {
                        theme.text_secondary()
                    },
                    border: iced::Border {
                        radius: ui_radius(6.0).into(),
                        width: 0.0,
                        color: iced::Color::TRANSPARENT,
                    },
                    ..Default::default()
                }
            }
        })
        .into()
}

/// Build the complete node editor page with header, divider, and content
pub fn build_page(state: &WorkflowEditorState) -> Element<'_, WorkflowEditorMessage> {
    let back_button = widgets::icon_button_hover(
        widgets::Icon::Back,
        16,
        [4, 8],
        6.0,
        WorkflowEditorMessage::ClosePage,
        |theme| theme.text(),
    );

    let header_row = row![
        back_button,
        text_input(language::tr(Text::WorkflowName), &state.name)
            .on_input(WorkflowEditorMessage::NameChanged)
            .padding(8)
            .size(13)
            .width(Length::Fixed(200.0))
            .style(|theme: &iced::Theme, status| {
                use iced::widget::text_input::Status;
                let is_focused = matches!(status, Status::Focused { .. });
                iced::widget::text_input::Style {
                    background: iced::Background::Color(theme.dialog_background()),
                    border: iced::Border {
                        radius: ui_radius(8.0).into(),
                        width: 1.0,
                        color: if is_focused {
                            theme.primary()
                        } else {
                            theme.divider()
                        },
                    },
                    icon: iced::Color::TRANSPARENT,
                    placeholder: theme.text_secondary(),
                    value: theme.text(),
                    selection: theme.primary(),
                }
            }),
        iced::widget::Space::new().width(Length::Fill),
        build_save_button(state.has_unsaved_changes, state.save_indicator_phase),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let header = widgets::draggable_header(header_row.into(), WorkflowEditorMessage::StartDrag);

    let editor_view = workflow_editor_view(state);

    let content = container(editor_view)
        .padding(0)
        .width(Length::Fill)
        .height(Length::Fill);

    widgets::page_shell(header, content.into())
}
