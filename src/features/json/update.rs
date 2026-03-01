use iced::{Task, widget};
use jsonpath_rust::JsonPath;
use serde_json::Value;

use super::constants::HEADER_HEIGHT;
use super::{message::Message, state::State};
use crate::platform::screen::get_window_height;
use crate::ui::constants::{WINDOW_MARGIN, WINDOW_WIDTH};

pub fn update(state: &mut State, msg: Message) -> Task<Message> {
    match msg {
        Message::ClosePage => Task::none(),
        Message::DeferredLoad(content) => {
            state.set_content(content);
            Task::none()
        }
        Message::ToggleFold(path) => {
            if state.collapsed.contains(&path) {
                state.collapsed.remove(&path);
            } else {
                state.collapsed.insert(path);
            }
            state.update_lines();
            Task::none()
        }
        Message::QueryChanged(query) => {
            state.query = query;
            Task::none()
        }
        Message::QuerySubmitted => {
            if state.query.trim().is_empty() {
                state.display_value = None;
            } else if let Some(parsed) = &state.parsed {
                if let Ok(results) = parsed.query(&state.query) {
                    state.display_value =
                        Some(Value::Array(results.into_iter().cloned().collect()));
                } else {
                    // If invalid query, maybe keep current or show nothing
                    state.display_value = Some(Value::Array(vec![]));
                }
            }
            state.update_lines();
            Task::none()
        }
        Message::CopyText(text) => iced::clipboard::write(text),
        Message::SelectionStarted(line, col) => {
            state.is_selecting = true;
            state.selection_start = Some((line, col));
            state.selection_end = Some((line, col));
            Task::none()
        }
        Message::SelectionUpdated(line, col) => {
            if state.is_selecting {
                state.selection_end = Some((line, col));
            }
            Task::none()
        }
        Message::SelectionEnded => {
            state.is_selecting = false;
            Task::none()
        }
        Message::Scrolled(viewport) => {
            let offset = viewport.absolute_offset();
            state.scroll_offset = offset;
            state.scroll_offset_y = offset.y;
            Task::none()
        }
        Message::Tick => {
            if !state.is_selecting {
                return Task::none();
            }

            let cursor_pos = state.cursor_position;
            let mut delta_x = 0.0;
            let mut delta_y = 0.0;
            let step = 15.0;
            let margin = 40.0;

            // Get window dimensions
            let window_height = get_window_height(WINDOW_MARGIN);

            // Header height
            let header_height = HEADER_HEIGHT;

            // Vertical scrolling
            if cursor_pos.y < header_height + margin {
                delta_y = -step;
            } else if cursor_pos.y > window_height - margin {
                delta_y = step;
            }

            // Horizontal scrolling
            if cursor_pos.x < margin {
                delta_x = -step;
            } else if cursor_pos.x > WINDOW_WIDTH - margin {
                delta_x = step;
            }

            if delta_x != 0.0 || delta_y != 0.0 {
                let current = state.scroll_offset;
                let new_x = (current.x + delta_x).max(0.0);
                let new_y = (current.y + delta_y).max(0.0);

                return widget::operation::scroll_to(
                    state.scrollable_id.clone(),
                    widget::scrollable::AbsoluteOffset {
                        x: Some(new_x),
                        y: Some(new_y),
                    },
                );
            }

            Task::none()
        }
        Message::StartDrag => Task::none(),
    }
}
