use iced::Point;
use iced::widget::{self, scrollable};
use serde_json::Value;
use std::collections::HashSet;

use super::lines::{Line, render_json_lines};

pub struct State {
    pub content: String,
    pub parsed: Option<Value>,
    pub collapsed: HashSet<String>,
    pub query: String,
    pub scrollable_id: widget::Id,
    pub selection_start: Option<(usize, usize)>,
    pub selection_end: Option<(usize, usize)>,
    pub is_selecting: bool,
    pub scroll_offset: scrollable::AbsoluteOffset,
    pub scroll_offset_y: f32,
    pub lines: Vec<Line>,
    pub cursor_position: Point,
    pub is_loading: bool,
    pub display_value: Option<Value>,
}

impl State {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            parsed: None,
            collapsed: HashSet::new(),
            query: String::new(),
            scrollable_id: widget::Id::unique(),
            selection_start: None,
            selection_end: None,
            is_selecting: false,
            scroll_offset: scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
            scroll_offset_y: 0.0,
            lines: Vec::new(),
            cursor_position: Point::default(),
            is_loading: false,
            display_value: None,
        }
    }

    /// Prepare for deferred loading - clears state and sets loading flag
    pub fn prepare_deferred_load(&mut self) {
        self.content.clear();
        self.parsed = None;
        self.collapsed.clear();
        self.lines.clear();
        self.selection_start = None;
        self.selection_end = None;
        self.is_selecting = false;
        self.scroll_offset = scrollable::AbsoluteOffset { x: 0.0, y: 0.0 };
        self.scroll_offset_y = 0.0;
        self.is_loading = true;
        self.display_value = None;
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.parsed = serde_json::from_str::<Value>(&self.content).ok();
        self.collapsed.clear();
        self.selection_start = None;
        self.selection_end = None;
        self.is_selecting = false;
        self.is_loading = false;
        self.display_value = None;
        self.update_lines();
    }

    pub fn update_lines(&mut self) {
        let value = self.display_value.as_ref().or(self.parsed.as_ref());
        self.lines = render_json_lines(&self.content, &value.cloned(), &self.collapsed);
    }
}
