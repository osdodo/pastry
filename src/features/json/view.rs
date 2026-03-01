use iced::widget::canvas::{
    Action, Event, Fill, Frame, Geometry, Path, Program, Text as CanvasText,
};
use iced::widget::{Space, container, row, scrollable, text_input};
use iced::{Element, Length};
use iced::{Point, Rectangle, Renderer, Theme, keyboard, mouse};

use super::{lines::Line, message::Message, state::State};
use crate::ui::{
    theme,
    theme::{PastryTheme, ThemeMode},
    util::ui_radius,
    widgets,
};

#[derive(Debug, Default, Clone, Copy)]
struct ProgramState {
    is_dragging: bool,
}

/// Build the complete JSON viewer page with header, divider, and content
pub fn build_page(state: &State, theme_mode: ThemeMode) -> Element<'_, Message> {
    let _palette = theme::palette(theme_mode);
    let back_button = widgets::icon_button_hover(
        widgets::Icon::Back,
        16,
        [4, 8],
        ui_radius(6.0),
        Message::ClosePage,
        |theme| theme.text(),
    );

    let header_row = row![back_button, Space::new().width(Length::Fill)]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    let header = widgets::draggable_header(header_row.into(), Message::StartDrag);

    let program = JsonCanvasProgram::new(state.lines.clone(), state, theme_mode);
    let total_height = program.total_height();
    let total_width = program.total_width();

    let json_area: Element<'_, Message> = container(
        scrollable(
            container(
                iced::widget::canvas(program)
                    .width(Length::Fixed(total_width))
                    .height(Length::Fixed(total_height)),
            )
            .padding(super::constants::CANVAS_PADDING),
        )
        .id(state.scrollable_id.clone())
        .on_scroll(Message::Scrolled)
        .direction(scrollable::Direction::Both {
            vertical: scrollable::Scrollbar::default(),
            horizontal: scrollable::Scrollbar::default(),
        })
        .height(Length::Fill)
        .width(Length::Fill),
    )
    .style(|_| container::Style {
        border: iced::Border {
            radius: ui_radius(8.0).into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .height(Length::Fill)
    .width(Length::Fill)
    .into();

    let query_input = text_input("Type a query...", &state.query)
        .on_input(Message::QueryChanged)
        .on_submit(Message::QuerySubmitted)
        .padding(10)
        .width(Length::Fill)
        .style(|theme: &Theme, status| text_input::Style {
            background: iced::Background::Color(theme.input_background()),
            border: iced::Border {
                color: if matches!(status, text_input::Status::Focused { .. }) {
                    theme.primary()
                } else {
                    theme.input_border()
                },
                width: 1.0,
                radius: ui_radius(6.0).into(),
            },
            icon: theme.text_secondary(),
            placeholder: theme.text_placeholder(),
            value: theme.text(),
            selection: theme.primary(),
        });

    let footer = container(row![query_input].align_y(iced::Alignment::Center))
        .padding([12, 12])
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: None,
            ..Default::default()
        });

    let footer_divider = container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(1)
        .style(|theme: &Theme| container::Style {
            background: Some(iced::Background::Color(theme.divider())),
            ..Default::default()
        });

    let page_content = container(
        iced::widget::column(vec![json_area, footer_divider.into(), footer.into()]).spacing(0),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    widgets::page_shell(header, page_content)
}

fn highlight_segments(s: &str, palette: iced::theme::Palette) -> Vec<(String, iced::Color)> {
    let mut spans: Vec<(String, iced::Color)> = Vec::new();
    let mut i: usize = 0;
    let len = s.len();
    while i < len {
        let ch = s[i..].chars().next().unwrap();
        let ch_len = ch.len_utf8();
        if ch == '"' {
            let start = i;
            i += ch_len;
            while i < len {
                let c2 = s[i..].chars().next().unwrap();
                let l2 = c2.len_utf8();
                if c2 == '\\' {
                    i += l2;
                    if i < len {
                        let esc = s[i..].chars().next().unwrap();
                        i += esc.len_utf8();
                    }
                    continue;
                }
                if c2 == '"' {
                    i += l2;
                    break;
                }
                i += l2;
            }

            // Check if it's a key (followed by colon)
            let mut is_key = false;
            let mut j = i;
            while j < len {
                let c = s[j..].chars().next().unwrap();
                if c.is_whitespace() {
                    j += c.len_utf8();
                } else if c == ':' {
                    is_key = true;
                    break;
                } else {
                    break;
                }
            }

            let color = if is_key {
                palette.text
            } else {
                palette.success
            };
            spans.push((s[start..i].to_string(), color));
        } else if ch.is_ascii_digit() || ch == '-' {
            let start = i;
            i += ch_len;
            while i < len {
                let c2 = s[i..].chars().next().unwrap();
                if c2.is_ascii_digit()
                    || c2 == '.'
                    || c2 == 'e'
                    || c2 == 'E'
                    || c2 == '+'
                    || c2 == '-'
                {
                    i += c2.len_utf8();
                } else {
                    break;
                }
            }
            spans.push((s[start..i].to_string(), palette.warning));
        } else if s[i..].starts_with("true") {
            spans.push(("true".to_string(), palette.success));
            i += 4;
        } else if s[i..].starts_with("false") {
            spans.push(("false".to_string(), palette.danger));
            i += 5;
        } else if s[i..].starts_with("null") {
            spans.push(("null".to_string(), palette.danger));
            i += 4;
        } else {
            spans.push((ch.to_string(), palette.text));
            i += ch_len;
        }
    }
    spans
}

#[derive(Debug, Clone)]
struct JsonDrawLine {
    text: String,
    path: Option<String>,
    collapsed: bool,
}

struct JsonCanvasProgram {
    lines: Vec<JsonDrawLine>,
    line_height: f32,
    toggle_width: f32,
    line_no_width: f32,
    char_width: f32,
    selection_start: Option<(usize, usize)>,
    selection_end: Option<(usize, usize)>,
    max_width: f32,
    scroll_offset_y: f32,
    theme_mode: ThemeMode,
}

impl JsonCanvasProgram {
    fn new(lines: Vec<Line>, state: &State, theme_mode: ThemeMode) -> Self {
        let total = lines.len();
        let digits = total.to_string().len().max(1) as f32;
        let line_height = super::constants::LINE_HEIGHT;
        let char_width = super::constants::CHAR_WIDTH;

        // Defer syntax highlighting - only store raw line data
        let mut max_chars = 0;
        let draw_lines: Vec<JsonDrawLine> = lines
            .into_iter()
            .map(|l| {
                max_chars = max_chars.max(l.text.chars().count());
                JsonDrawLine {
                    text: l.text,
                    path: l.path,
                    collapsed: l.collapsed,
                }
            })
            .collect();

        let line_no_width = digits * char_width + super::constants::LINE_NO_PADDING;
        let max_width = super::constants::TOGGLE_WIDTH
            + line_no_width
            + super::constants::TEXT_PADDING_LEFT
            + (max_chars as f32 * char_width)
            + 20.0; // Extra padding

        Self {
            lines: draw_lines,
            line_height,
            toggle_width: super::constants::TOGGLE_WIDTH,
            line_no_width,
            char_width,
            selection_start: state.selection_start,
            selection_end: state.selection_end,
            max_width,
            scroll_offset_y: state.scroll_offset_y,
            theme_mode,
        }
    }

    fn total_width(&self) -> f32 {
        self.max_width
    }

    fn total_height(&self) -> f32 {
        self.lines.len() as f32 * self.line_height + 20.0
    }

    fn hit_test(&self, p: Point) -> (usize, usize) {
        let line_idx = (p.y / self.line_height).floor().max(0.0) as usize;
        let line_idx = line_idx.min(self.lines.len().saturating_sub(1));

        let start_x = self.toggle_width + self.line_no_width + super::constants::TEXT_PADDING_LEFT;
        let char_idx = ((p.x - start_x) / self.char_width).round().max(0.0) as usize;

        if line_idx < self.lines.len() {
            let line_len = self.lines[line_idx].text.chars().count();
            (line_idx, char_idx.min(line_len))
        } else {
            (line_idx, 0)
        }
    }

    fn extract_text(&self, start: (usize, usize), end: (usize, usize)) -> String {
        let mut result = String::new();
        for i in start.0..=end.0 {
            if i >= self.lines.len() {
                break;
            }
            let line = &self.lines[i];
            let chars: Vec<char> = line.text.chars().collect();

            let s = if i == start.0 { start.1 } else { 0 };
            let e = if i == end.0 { end.1 } else { chars.len() };

            if s < chars.len() {
                let end_idx = e.min(chars.len());
                if s < end_idx {
                    result.push_str(&chars[s..end_idx].iter().collect::<String>());
                }
            }

            if i < end.0 {
                result.push('\n');
            }
        }
        result
    }
}

fn normalize_selection(
    start: (usize, usize),
    end: (usize, usize),
) -> ((usize, usize), (usize, usize)) {
    if start.0 < end.0 || (start.0 == end.0 && start.1 <= end.1) {
        (start, end)
    } else {
        (end, start)
    }
}

impl Program<Message> for JsonCanvasProgram {
    type State = ProgramState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let palette = theme::palette(self.theme_mode);
        // Helper to access trait methods if needed, though we can just use palette for canvas mostly.
        // Actually we can't easily construct Theme from mode here without depending on iced internal, but we have palette.
        // Let's use palette for standard colors. For custom colors (like text_secondary), we might need to duplicate logic or lookup.
        // Given we are in a draw method, we can't use the `theme` argument passed to draw easily because it matches `_theme` which is `Theme`.
        // Wait, `_theme` IS passed to draw! We should use that!

        // But `highlight_segments` needs palette. And `_theme` has palette.

        let mut frame = Frame::new(renderer, bounds.size());

        // Virtual scrolling: calculate visible range with buffer
        let buffer_lines = 5; // Extra lines above/below viewport for smooth scrolling
        let visible_height = bounds.height;
        let first_visible = (self.scroll_offset_y / self.line_height).floor() as usize;
        let visible_count = (visible_height / self.line_height).ceil() as usize + buffer_lines * 2;
        let start_idx = first_visible.saturating_sub(buffer_lines);
        let end_idx = (start_idx + visible_count).min(self.lines.len());

        // Draw selection (only for visible lines)
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (s, e) = normalize_selection(start, end);

            for idx in start_idx..end_idx {
                if idx >= s.0 && idx <= e.0 {
                    let y = (idx as f32) * self.line_height;
                    let line = &self.lines[idx];
                    let start_x = self.toggle_width
                        + self.line_no_width
                        + super::constants::TEXT_PADDING_LEFT;
                    let line_chars = line.text.chars().count();

                    let col_start = if idx == s.0 { s.1 } else { 0 };
                    let col_end = if idx == e.0 { e.1 } else { line_chars };

                    let draw_col_end = if idx < e.0 {
                        col_end.max(line_chars) + 1
                    } else {
                        col_end
                    };

                    let x = start_x + (col_start as f32) * self.char_width;
                    let w = ((draw_col_end.saturating_sub(col_start)) as f32) * self.char_width;

                    if w > 0.0 {
                        frame.fill_rectangle(
                            Point::new(x, y),
                            iced::Size::new(w, self.line_height),
                            Fill::from(iced::Color::from_rgba(0.2, 0.4, 0.8, 0.3)),
                        );
                    }
                }
            }
        }

        let font_size = 12.0;
        // Only render visible lines (virtual scrolling)
        for idx in start_idx..end_idx {
            let line = &self.lines[idx];
            let y = (idx as f32) * self.line_height;
            let base_y = y + (self.line_height - font_size) / 2.0;

            // Toggle triangle
            if line.path.is_some() {
                let cx = self.toggle_width / 2.0;
                let cy = y + self.line_height * 0.5;
                let size = 4.0;
                let path = if line.collapsed {
                    Path::new(|b| {
                        b.move_to(iced::Point::new(cx - size * 0.5, cy - size));
                        b.line_to(iced::Point::new(cx + size * 0.5, cy));
                        b.line_to(iced::Point::new(cx - size * 0.5, cy + size));
                        b.close();
                    })
                } else {
                    Path::new(|b| {
                        b.move_to(iced::Point::new(cx - size, cy - size * 0.5));
                        b.line_to(iced::Point::new(cx, cy + size * 0.5));
                        b.line_to(iced::Point::new(cx + size, cy - size * 0.5));
                        b.close();
                    })
                };
                frame.fill(&path, Fill::from(palette.text));
            }

            // Line number
            let ln_txt = CanvasText {
                content: format!("{}", idx + 1),
                position: iced::Point::new(self.toggle_width, base_y),
                // We need text_secondary which is custom.
                // We can reproduce it or add it to palette?
                // Let's rely on palette.text with some opacity or just gray.
                color: if matches!(self.theme_mode, ThemeMode::Dark) {
                    iced::Color::from_rgb8(179, 179, 179)
                } else {
                    iced::Color::from_rgb8(102, 102, 102)
                },
                size: font_size.into(),
                font: iced::Font::MONOSPACE,
                ..Default::default()
            };
            frame.fill_text(ln_txt);

            // Syntax highlight on-demand (only for visible lines)
            let segments = highlight_segments(&line.text, palette);
            let mut x =
                self.toggle_width + self.line_no_width + super::constants::TEXT_PADDING_LEFT;
            for (seg, color) in segments.iter() {
                let is_ascii = seg.is_ascii();
                let txt = CanvasText {
                    content: seg.clone(),
                    position: iced::Point::new(x, base_y),
                    color: *color,
                    size: font_size.into(),
                    font: if is_ascii {
                        iced::Font::MONOSPACE
                    } else {
                        iced::Font::DEFAULT
                    },
                    ..Default::default()
                };
                frame.fill_text(txt);

                let seg_width: f32 = seg
                    .chars()
                    .map(|c| {
                        if c.is_ascii() {
                            self.char_width
                        } else {
                            self.char_width * 2.0
                        }
                    })
                    .sum();
                x += seg_width;
            }
        }
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let cursor_pos = cursor.position_in(bounds);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(p) = cursor_pos {
                    let idx = (p.y / self.line_height).floor() as usize;
                    // Check toggle first
                    if idx < self.lines.len()
                        && p.x < self.toggle_width + 12.0
                        && let Some(path) = &self.lines[idx].path
                    {
                        return Some(Action::publish(Message::ToggleFold(path.clone())));
                    }

                    // Start selection
                    let (line, col) = self.hit_test(p);
                    state.is_dragging = true;
                    return Some(Action::publish(Message::SelectionStarted(line, col)));
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging
                    && let Some(p) = cursor_pos
                {
                    let (line, col) = self.hit_test(p);
                    return Some(Action::publish(Message::SelectionUpdated(line, col)));
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    return Some(Action::publish(Message::SelectionEnded));
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(c),
                modifiers,
                ..
            }) if c == "c" && modifiers.command() => {
                // Extract text
                if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                    let (s, e) = normalize_selection(start, end);
                    let text = self.extract_text(s, e);
                    if !text.is_empty() {
                        return Some(Action::publish(Message::CopyText(text)));
                    }
                }
            }
            _ => {}
        }
        None
    }
}
