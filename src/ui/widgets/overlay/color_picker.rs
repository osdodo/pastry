//! Use a color picker as an input element for picking colors.
//!
//! *This API requires the following crate features to be activated: `color_picker`*

use std::collections::HashMap;

use iced::advanced::Renderer as _;
use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::{
    Clipboard, Layout, Overlay, Shell, Widget,
    clipboard::Kind,
    layout::{Limits, Node},
    mouse::{self, Cursor},
    overlay, renderer,
    text::{self, Alignment as TextAlignment, Renderer as _, Text},
    widget::{self, tree::Tree},
};
use iced::widget::{
    Button, Column, Renderer, Row,
    canvas::{self, LineCap, Path, Stroke},
    text::{self as widget_text, Wrapping},
};
use iced::{
    Alignment, Border, Color, Element, Event, Length, Padding, Pixels, Point, Rectangle, Size,
    Vector,
    alignment::{Horizontal, Vertical},
    event, keyboard, touch,
};

use crate::ui::language::{Text as TrText, tr};
use crate::ui::widgets::{
    color_picker,
    core::{
        color::{HexString, Hsv},
        overlay::Position,
    },
    style::{self, Status, color_picker::Style, style_state::StyleState},
};

/// The padding around the elements.
const PADDING: Padding = Padding::new(8.0);
/// The spacing between the element.
const SPACING: Pixels = Pixels(10.0);

/// The step value of the keyboard change of the sat/value color values.
const SAT_VALUE_STEP: f32 = 0.005;
/// The step value of the keyboard change of the hue color value.
const HUE_STEP: i32 = 1;
/// The step value of the keyboard change of the RGBA color values.
const RGBA_STEP: i16 = 1;

/// The overlay of the [`ColorPicker`](crate::widget::ColorPicker).
#[allow(missing_debug_implementations)]
pub struct ColorPickerOverlay<'a, 'b, Message>
where
    Message: Clone,
    'b: 'a,
{
    /// The state of the [`ColorPickerOverlay`].
    state: &'a mut color_picker::State,
    /// The cancel message.
    on_cancel: Message,
    /// The hex copy button of the [`ColorPickerOverlay`].
    hex_copy_button: Button<'static, Message, iced::Theme, Renderer>,
    /// The rgba copy button of the [`ColorPickerOverlay`].
    rgba_copy_button: Button<'static, Message, iced::Theme, Renderer>,
    /// The function that produces a message when the submit button of the [`ColorPickerOverlay`].
    on_submit: &'a dyn Fn(Color) -> Message,
    /// Optional function that produces a message when the color changes during selection (real-time updates).
    on_color_change: Option<&'a dyn Fn(Color) -> Message>,
    /// The position of the [`ColorPickerOverlay`].
    position: Point,
    /// The style of the [`ColorPickerOverlay`].
    class: &'a <iced::Theme as style::color_picker::Catalog>::Class<'b>,
    /// The reference to the tree holding the state of this overlay.
    tree: &'a mut Tree,
    viewport: Rectangle,
}

impl<'a, 'b, Message> ColorPickerOverlay<'a, 'b, Message>
where
    Message: 'static + Clone,
    'b: 'a,
{
    /// Creates a new [`ColorPickerOverlay`] on the given position.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        state: &'a mut color_picker::State,
        on_cancel: Message,
        on_submit: &'a dyn Fn(Color) -> Message,
        on_color_change: Option<&'a dyn Fn(Color) -> Message>,
        position: Point,
        class: &'a <iced::Theme as style::color_picker::Catalog>::Class<'b>,
        tree: &'a mut Tree,
        viewport: Rectangle,
    ) -> Self {
        ColorPickerOverlay {
            state,
            on_cancel: on_cancel.clone(),
            hex_copy_button: Button::new(
                iced::widget::Text::new(tr(TrText::Copy))
                    .size(12)
                    .align_x(Horizontal::Center)
                    .wrapping(Wrapping::None)
                    .width(Length::Fill),
            )
            .width(Length::Fixed(40.0)) // Adjust width as needed
            .on_press(on_cancel.clone()), // Sending a fake message
            rgba_copy_button: Button::new(
                iced::widget::Text::new(tr(TrText::Copy))
                    .size(12)
                    .align_x(Horizontal::Center)
                    .wrapping(Wrapping::None)
                    .width(Length::Fill),
            )
            .width(Length::Fixed(40.0)) // Adjust width as needed
            .on_press(on_cancel.clone()), // Sending a fake message
            on_submit,
            on_color_change,
            position,
            class,
            tree,
            viewport,
        }
    }

    /// Turn this [`ColorPickerOverlay`] into an overlay [`Element`](overlay::Element).
    #[must_use]
    pub fn overlay(self) -> overlay::Element<'a, Message, iced::Theme, Renderer> {
        overlay::Element::new(Box::new(self))
    }

    /// Force redraw all components if the internal state was changed
    fn clear_cache(&self) {
        self.state.clear_cache();
    }

    /// The event handling for the HSV color area.
    fn on_event_hsv_color(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        shell: &mut Shell<Message>,
    ) -> event::Status {
        let mut hsv_color_children = layout.children();

        let hsv_color: Hsv = self.state.color.into();
        let mut color_changed = false;

        let sat_value_bounds = hsv_color_children
            .next()
            .expect("widget: Layout should have a sat/value layout")
            .bounds();
        let hue_bounds = hsv_color_children
            .next()
            .expect("widget: Layout should have a hue layout")
            .bounds();

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
                mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                    let move_value =
                        |value: u16, y: f32| ((i32::from(value) + y as i32).rem_euclid(360)) as u16;

                    if cursor.is_over(hue_bounds) {
                        self.state.color = Color {
                            a: self.state.color.a,
                            ..Hsv {
                                hue: move_value(hsv_color.hue, *y),
                                ..hsv_color
                            }
                            .into()
                        };
                        color_changed = true;
                    }
                }
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(sat_value_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::SatValue;
                    self.state.focus = Focus::SatValue;
                }
                if cursor.is_over(hue_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::Hue;
                    self.state.focus = Focus::Hue;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
                self.state.color_bar_dragged = ColorBarDragged::None;
            }
            _ => {}
        }

        if self.state.color_bar_dragged == ColorBarDragged::SatValue
            && let Some(cursor_position) = cursor.position()
        {
            let sat =
                ((cursor_position.x - sat_value_bounds.x) / sat_value_bounds.width).clamp(0.0, 1.0);
            let value = 1.0
                - ((cursor_position.y - sat_value_bounds.y) / sat_value_bounds.height)
                    .clamp(0.0, 1.0);

            self.state.color = Color {
                a: self.state.color.a,
                ..Hsv {
                    saturation: sat,
                    value,
                    ..hsv_color
                }
                .into()
            };
            color_changed = true;
        }

        if self.state.color_bar_dragged == ColorBarDragged::Hue
            && let Some(cursor_position) = cursor.position()
        {
            let hue = (((cursor_position.x - hue_bounds.x) / hue_bounds.width).clamp(0.0, 1.0)
                * 360.0) as u16;

            self.state.color = Color {
                a: self.state.color.a,
                ..Hsv { hue, ..hsv_color }.into()
            };
            color_changed = true;
        }

        if color_changed {
            self.clear_cache();
            if let Some(on_color_change) = self.on_color_change {
                shell.publish(on_color_change(self.state.color));
            }
            return event::Status::Captured;
        }

        event::Status::Ignored
    }

    /// The event handling for the RGBA color area.
    #[allow(clippy::too_many_lines)]
    fn on_event_rgba_color(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        shell: &mut Shell<Message>,
    ) -> event::Status {
        let mut rgba_color_children = layout.children();
        let mut color_changed = false;

        let mut red_row_children = rgba_color_children
            .next()
            .expect("widget: Layout should have a red row layout")
            .children();
        let _ = red_row_children.next();
        let red_bar_bounds = red_row_children
            .next()
            .expect("widget: Layout should have a red bar layout")
            .bounds();

        let mut green_row_children = rgba_color_children
            .next()
            .expect("widget: Layout should have a green row layout")
            .children();
        let _ = green_row_children.next();
        let green_bar_bounds = green_row_children
            .next()
            .expect("widget: Layout should have a green bar layout")
            .bounds();

        let mut blue_row_children = rgba_color_children
            .next()
            .expect("widget: Layout should have a blue row layout")
            .children();
        let _ = blue_row_children.next();
        let blue_bar_bounds = blue_row_children
            .next()
            .expect("widget: Layout should have a blue bar layout")
            .bounds();

        let mut alpha_row_children = rgba_color_children
            .next()
            .expect("widget: Layout should have an alpha row layout")
            .children();
        let _ = alpha_row_children.next();
        let alpha_bar_bounds = alpha_row_children
            .next()
            .expect("widget: Layout should have an alpha bar layout")
            .bounds();

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
                mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                    let move_value =
                        //|value: f32, y: f32| (value * 255.0 + y).clamp(0.0, 255.0) / 255.0;
                        |value: f32, y: f32| value.mul_add(255.0, y).clamp(0.0, 255.0) / 255.0;

                    if cursor.is_over(red_bar_bounds) {
                        self.state.color = Color {
                            r: move_value(self.state.color.r, *y),
                            ..self.state.color
                        };
                        color_changed = true;
                    }
                    if cursor.is_over(green_bar_bounds) {
                        self.state.color = Color {
                            g: move_value(self.state.color.g, *y),
                            ..self.state.color
                        };
                        color_changed = true;
                    }
                    if cursor.is_over(blue_bar_bounds) {
                        self.state.color = Color {
                            b: move_value(self.state.color.b, *y),
                            ..self.state.color
                        };
                        color_changed = true;
                    }
                    if cursor.is_over(alpha_bar_bounds) {
                        self.state.color = Color {
                            a: move_value(self.state.color.a, *y),
                            ..self.state.color
                        };
                        color_changed = true;
                    }
                }
            },
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(red_bar_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::Red;
                    self.state.focus = Focus::Red;
                }
                if cursor.is_over(green_bar_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::Green;
                    self.state.focus = Focus::Green;
                }
                if cursor.is_over(blue_bar_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::Blue;
                    self.state.focus = Focus::Blue;
                }
                if cursor.is_over(alpha_bar_bounds) {
                    self.state.color_bar_dragged = ColorBarDragged::Alpha;
                    self.state.focus = Focus::Alpha;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
                self.state.color_bar_dragged = ColorBarDragged::None;
            }
            _ => {}
        }

        if self.state.color_bar_dragged == ColorBarDragged::Red
            && let Some(cursor_position) = cursor.position()
        {
            self.state.color.r =
                ((cursor_position.x - red_bar_bounds.x) / red_bar_bounds.width).clamp(0.0, 1.0);
            color_changed = true;
        }

        if self.state.color_bar_dragged == ColorBarDragged::Green
            && let Some(cursor_position) = cursor.position()
        {
            self.state.color.g =
                ((cursor_position.x - green_bar_bounds.x) / green_bar_bounds.width).clamp(0.0, 1.0);
            color_changed = true;
        }

        if self.state.color_bar_dragged == ColorBarDragged::Blue
            && let Some(cursor_position) = cursor.position()
        {
            self.state.color.b =
                ((cursor_position.x - blue_bar_bounds.x) / blue_bar_bounds.width).clamp(0.0, 1.0);
            color_changed = true;
        }

        if self.state.color_bar_dragged == ColorBarDragged::Alpha
            && let Some(cursor_position) = cursor.position()
        {
            self.state.color.a =
                ((cursor_position.x - alpha_bar_bounds.x) / alpha_bar_bounds.width).clamp(0.0, 1.0);
            color_changed = true;
        }

        if color_changed {
            self.clear_cache();
            if let Some(on_color_change) = self.on_color_change {
                shell.publish(on_color_change(self.state.color));
            }
            return event::Status::Captured;
        }

        event::Status::Ignored
    }

    /// The even handling for the keyboard input.
    fn on_event_keyboard(&mut self, event: &Event, shell: &mut Shell<Message>) -> event::Status {
        if self.state.focus == Focus::None {
            return event::Status::Ignored;
        }

        if let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event {
            let mut status = event::Status::Ignored;

            if matches!(key, keyboard::Key::Named(keyboard::key::Named::Tab)) {
                if self.state.keyboard_modifiers.shift() {
                    self.state.focus = self.state.focus.previous();
                } else {
                    self.state.focus = self.state.focus.next();
                }
                // TODO: maybe place this better
                self.clear_cache();
            } else {
                let sat_value_handle = |key_code: &keyboard::Key, color: &mut Color| {
                    let mut hsv_color: Hsv = (*color).into();
                    let mut status = event::Status::Ignored;

                    match key_code {
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                            hsv_color.saturation -= SAT_VALUE_STEP;
                            status = event::Status::Captured;
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                            hsv_color.saturation += SAT_VALUE_STEP;
                            status = event::Status::Captured;
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                            hsv_color.value -= SAT_VALUE_STEP;
                            status = event::Status::Captured;
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                            hsv_color.value += SAT_VALUE_STEP;
                            status = event::Status::Captured;
                        }
                        _ => {}
                    }

                    hsv_color.saturation = hsv_color.saturation.clamp(0.0, 1.0);
                    hsv_color.value = hsv_color.value.clamp(0.0, 1.0);

                    *color = Color {
                        a: color.a,
                        ..hsv_color.into()
                    };
                    status
                };

                let hue_handle = |key_code: &keyboard::Key, color: &mut Color| {
                    let mut hsv_color: Hsv = (*color).into();
                    let mut status = event::Status::Ignored;

                    let mut value = i32::from(hsv_color.hue);

                    match key_code {
                        keyboard::Key::Named(
                            keyboard::key::Named::ArrowLeft | keyboard::key::Named::ArrowDown,
                        ) => {
                            value -= HUE_STEP;
                            status = event::Status::Captured;
                        }
                        keyboard::Key::Named(
                            keyboard::key::Named::ArrowRight | keyboard::key::Named::ArrowUp,
                        ) => {
                            value += HUE_STEP;
                            status = event::Status::Captured;
                        }
                        _ => {}
                    }

                    hsv_color.hue = value.rem_euclid(360) as u16;

                    *color = Color {
                        a: color.a,
                        ..hsv_color.into()
                    };

                    status
                };

                let rgba_bar_handle = |key_code: &keyboard::Key, value: &mut f32| {
                    let mut byte_value = (*value * 255.0) as i16;
                    let mut status = event::Status::Captured;

                    match key_code {
                        keyboard::Key::Named(
                            keyboard::key::Named::ArrowLeft | keyboard::key::Named::ArrowDown,
                        ) => {
                            byte_value -= RGBA_STEP;
                            status = event::Status::Captured;
                        }
                        keyboard::Key::Named(
                            keyboard::key::Named::ArrowRight | keyboard::key::Named::ArrowUp,
                        ) => {
                            byte_value += RGBA_STEP;
                            status = event::Status::Captured;
                        }
                        _ => {}
                    }
                    *value = f32::from(byte_value.clamp(0, 255)) / 255.0;

                    status
                };

                match self.state.focus {
                    Focus::SatValue => status = sat_value_handle(key, &mut self.state.color),
                    Focus::Hue => status = hue_handle(key, &mut self.state.color),
                    Focus::Red => status = rgba_bar_handle(key, &mut self.state.color.r),
                    Focus::Green => status = rgba_bar_handle(key, &mut self.state.color.g),
                    Focus::Blue => status = rgba_bar_handle(key, &mut self.state.color.b),
                    Focus::Alpha => status = rgba_bar_handle(key, &mut self.state.color.a),
                    _ => {}
                }

                if status == event::Status::Captured
                    && let Some(on_color_change) = self.on_color_change
                {
                    shell.publish(on_color_change(self.state.color));
                }
            }

            status
        } else if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event {
            self.state.keyboard_modifiers = *modifiers;
            event::Status::Ignored
        } else {
            event::Status::Ignored
        }
    }
}

impl<'a, Message> Overlay<Message, iced::Theme, Renderer> for ColorPickerOverlay<'a, '_, Message>
where
    Message: 'static + Clone,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        let (max_width, max_height) = if bounds.width > bounds.height {
            (600.0, 300.0)
        } else {
            (360.0, 520.0)
        };

        let limits = Limits::new(Size::ZERO, bounds)
            .shrink(PADDING)
            .width(Length::Fill)
            .height(Length::Fill)
            .max_width(max_width)
            .max_height(max_height);

        let divider = if bounds.width > bounds.height {
            Row::<(), iced::Theme, Renderer>::new()
                .spacing(SPACING)
                .push(Row::new().width(Length::Fill).height(Length::Fill))
                .push(Row::new().width(Length::Fill).height(Length::Fill))
                .layout(self.tree, renderer, &limits)
        } else {
            Column::<(), iced::Theme, Renderer>::new()
                .spacing(SPACING)
                .push(Row::new().width(Length::Fill).height(Length::Fill))
                .push(Row::new().width(Length::Fill).height(Length::Fill))
                .layout(self.tree, renderer, &limits)
        };

        let mut divider_children = divider.children().iter();

        let block1_bounds = divider_children
            .next()
            .expect("Divider should have a first child")
            .bounds();
        let block2_bounds = divider_children
            .next()
            .expect("Divider should have a second child")
            .bounds();

        // ----------- Block 1 ----------------------
        let block1_node = block1_layout(self, renderer, block1_bounds);

        // ----------- Block 2 ----------------------
        let block2_node = block2_layout(self, renderer, block2_bounds);

        let (width, height) = if bounds.width > bounds.height {
            (
                block1_node.size().width + block2_node.size().width + SPACING.0, // + (2.0 * PADDING as f32),
                block2_node.size().height,
            )
        } else {
            (
                block2_node.size().width,
                block1_node.size().height + block2_node.size().height + SPACING.0,
            )
        };

        let mut node =
            Node::with_children(Size::new(width, height), vec![block1_node, block2_node]);

        node.center_and_bounce(self.position, bounds);
        node
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
    ) {
        if event::Status::Captured == self.on_event_keyboard(event, shell) {
            self.clear_cache();
            shell.capture_event();
            shell.request_redraw();
            return;
        }

        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) = event
            && !layout
                .bounds()
                .contains(cursor.position().unwrap_or(Point::ORIGIN))
        {
            shell.publish(self.on_cancel.clone());
        }

        let mut children = layout.children();
        // ----------- Block 1 ----------------------
        let block1_layout = children
            .next()
            .expect("widget: Layout should have a 1. block layout");
        let hsv_color_status = self.on_event_hsv_color(event, block1_layout, cursor, shell);
        // ----------- Block 1 end ------------------

        // ----------- Block 2 ----------------------
        let block2_layout = children
            .next()
            .expect("widget: Layout should have a 2. block layout");

        let mut block2_children = block2_layout.children();

        // ----------- RGBA Color -----------------------
        let rgba_color_layout = block2_children
            .next()
            .expect("widget: Layout should have a RGBA color layout");
        let rgba_color_status = self.on_event_rgba_color(event, rgba_color_layout, cursor, shell);

        // ----------- Hex Text ----------------------
        let _hex_text_layout = block2_children
            .next()
            .expect("widget: Layout should have a hex text layout");

        // ----------- Hex Copy Button -----------------
        let hex_copy_button_layout = block2_children
            .next()
            .expect("widget: Layout should have a hex copy button layout");

        let mut hex_messages = Vec::new();
        self.hex_copy_button.update(
            &mut self.tree.children[0],
            event,
            hex_copy_button_layout,
            cursor,
            renderer,
            clipboard,
            &mut Shell::new(&mut hex_messages),
            &layout.bounds(),
        );
        if !hex_messages.is_empty() {
            clipboard.write(Kind::Standard, self.state.color.as_hex_string());
            shell.publish((self.on_submit)(self.state.color));
        }

        // ----------- RGBA Text ----------------------
        let _rgba_text_layout = block2_children
            .next()
            .expect("widget: Layout should have a rgba text layout");

        // ----------- RGBA Copy Button -----------------
        let rgba_copy_button_layout = block2_children
            .next()
            .expect("widget: Layout should have a rgba copy button layout");

        let mut rgba_messages = Vec::new();
        self.rgba_copy_button.update(
            &mut self.tree.children[1],
            event,
            rgba_copy_button_layout,
            cursor,
            renderer,
            clipboard,
            &mut Shell::new(&mut rgba_messages),
            &layout.bounds(),
        );
        if !rgba_messages.is_empty() {
            let color = self.state.color;
            let rgba_string = format!(
                "rgba({}, {}, {}, {:.2})",
                (color.r * 255.0).round() as u8,
                (color.g * 255.0).round() as u8,
                (color.b * 255.0).round() as u8,
                color.a
            );
            clipboard.write(Kind::Standard, rgba_string);
            shell.publish((self.on_submit)(self.state.color));
        }
        // ----------- Block 2 end ------------------

        if hsv_color_status == event::Status::Captured
            || rgba_color_status == event::Status::Captured
        {
            self.clear_cache();
            shell.capture_event();
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut children = layout.children();

        let mouse_interaction = mouse::Interaction::default();

        // Block 1
        let block1_layout = children
            .next()
            .expect("Graphics: Layout should have a 1. block layout");
        let mut block1_mouse_interaction = mouse::Interaction::default();
        // HSV color
        let mut hsv_color_children = block1_layout.children();
        let sat_value_layout = hsv_color_children
            .next()
            .expect("Graphics: Layout should have a sat/value layout");
        if cursor.is_over(sat_value_layout.bounds()) {
            block1_mouse_interaction = block1_mouse_interaction.max(mouse::Interaction::Pointer);
        }
        let hue_layout = hsv_color_children
            .next()
            .expect("Graphics: Layout should have a hue layout");
        if cursor.is_over(hue_layout.bounds()) {
            block1_mouse_interaction = block1_mouse_interaction.max(mouse::Interaction::Pointer);
        }

        // Block 2
        let block2_layout = children
            .next()
            .expect("Graphics: Layout should have a 2. block layout");

        let mut block2_mouse_interaction = mouse::Interaction::default();
        let mut block2_children = block2_layout.children();
        // RGBA color
        let rgba_color_layout = block2_children
            .next()
            .expect("Graphics: Layout should have a RGBA color layout");
        let mut rgba_color_children = rgba_color_layout.children();

        let f = |layout: Layout<'_>, cursor: Cursor| {
            let mut children = layout.children();

            let _label_layout = children.next();
            let bar_layout = children
                .next()
                .expect("Graphics: Layout should have a bar layout");

            if cursor.is_over(bar_layout.bounds()) {
                mouse::Interaction::ResizingHorizontally
            } else {
                mouse::Interaction::default()
            }
        };
        let red_row_layout = rgba_color_children
            .next()
            .expect("Graphics: Layout should have a red row layout");
        block2_mouse_interaction = block2_mouse_interaction.max(f(red_row_layout, cursor));
        let green_row_layout = rgba_color_children
            .next()
            .expect("Graphics: Layout should have a green row layout");
        block2_mouse_interaction = block2_mouse_interaction.max(f(green_row_layout, cursor));
        let blue_row_layout = rgba_color_children
            .next()
            .expect("Graphics: Layout should have a blue row layout");
        block2_mouse_interaction = block2_mouse_interaction.max(f(blue_row_layout, cursor));
        let alpha_row_layout = rgba_color_children
            .next()
            .expect("Graphics: Layout should have an alpha row layout");
        block2_mouse_interaction = block2_mouse_interaction.max(f(alpha_row_layout, cursor));

        let _hex_text_layout = block2_children.next();

        // Hex Copy Button
        let hex_copy_button_layout = block2_children
            .next()
            .expect("Graphics: Layout should have a hex copy button layout");
        let hex_copy_interaction = self.hex_copy_button.mouse_interaction(
            &self.tree.children[0],
            hex_copy_button_layout,
            cursor,
            &self.viewport,
            renderer,
        );

        let _rgba_text_layout = block2_children.next();

        // RGBA Copy Button
        let rgba_copy_button_layout = block2_children
            .next()
            .expect("Graphics: Layout should have a rgba copy button layout");
        let rgba_copy_interaction = self.rgba_copy_button.mouse_interaction(
            &self.tree.children[1],
            rgba_copy_button_layout,
            cursor,
            &self.viewport,
            renderer,
        );

        mouse_interaction
            .max(block1_mouse_interaction)
            .max(block2_mouse_interaction)
            .max(hex_copy_interaction)
            .max(rgba_copy_interaction)
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let mut children = layout.children();

        // Skip block 1 (HSV color area)
        let _block1_layout = children.next();

        // Block 2 contains the buttons
        if let Some(block2_layout) = children.next() {
            let mut block2_children = block2_layout.children();

            // Skip rgba_colors, hex_text
            let _rgba_layout = block2_children.next();
            let _hex_text_layout = block2_children.next();

            // Operate on hex copy button
            if let Some(hex_copy_layout) = block2_children.next() {
                self.hex_copy_button.operate(
                    &mut self.tree.children[0],
                    hex_copy_layout,
                    renderer,
                    operation,
                );
            }

            // Skip rgba text
            let _rgba_text_layout = block2_children.next();

            // Operate on rgba copy button
            if let Some(rgba_copy_layout) = block2_children.next() {
                self.rgba_copy_button.operate(
                    &mut self.tree.children[1],
                    rgba_copy_layout,
                    renderer,
                    operation,
                );
            }
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        let bounds = layout.bounds();
        let mut children = layout.children();

        let mut style_sheet: HashMap<StyleState, Style> = HashMap::new();
        let _ = style_sheet.insert(
            StyleState::Active,
            style::color_picker::Catalog::style(theme, self.class, Status::Active),
        );
        let _ = style_sheet.insert(
            StyleState::Selected,
            style::color_picker::Catalog::style(theme, self.class, Status::Selected),
        );
        let _ = style_sheet.insert(
            StyleState::Hovered,
            style::color_picker::Catalog::style(theme, self.class, Status::Hovered),
        );
        let _ = style_sheet.insert(
            StyleState::Focused,
            style::color_picker::Catalog::style(theme, self.class, Status::Focused),
        );

        let mut style_state = StyleState::Active;
        if self.state.focus == Focus::Overlay {
            style_state = style_state.max(StyleState::Focused);
        }
        if cursor.is_over(bounds) {
            style_state = style_state.max(StyleState::Hovered);
        }

        if (bounds.width > 0.) && (bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: style_sheet[&style_state].border_radius.into(),
                        width: style_sheet[&style_state].border_width,
                        color: style_sheet[&style_state].border_color,
                    },
                    ..renderer::Quad::default()
                },
                style_sheet[&style_state].background,
            );
        }

        // ----------- Block 1 ----------------------
        let block1_layout = children
            .next()
            .expect("Graphics: Layout should have a 1. block layout");
        block1(renderer, self, block1_layout, cursor, &style_sheet);

        // ----------- Block 2 ----------------------
        let block2_layout = children
            .next()
            .expect("Graphics: Layout should have a 2. block layout");

        block2(
            renderer,
            self,
            block2_layout,
            cursor,
            theme,
            style,
            &bounds,
            &style_sheet,
        );
    }
}

/// Defines the layout of the 1. block of the color picker containing the HSV part.
fn block1_layout<Message>(
    color_picker: &mut ColorPickerOverlay<'_, '_, Message>,
    renderer: &Renderer,
    bounds: Rectangle,
) -> Node
where
    Message: 'static + Clone,
{
    let block1_limits = Limits::new(Size::ZERO, bounds.size())
        .width(Length::Fill)
        .height(Length::Fill);

    let block1_node = Column::<(), iced::Theme, Renderer>::new()
        .spacing(PADDING.y() / 2.) // Average vertical padding
        .push(
            Row::new()
                .width(Length::Fill)
                .height(Length::FillPortion(7)),
        )
        .push(
            Row::new()
                .width(Length::Fill)
                .height(Length::FillPortion(1)),
        )
        .layout(color_picker.tree, renderer, &block1_limits);

    block1_node.move_to(Point::new(bounds.x + PADDING.left, bounds.y + PADDING.top))
}

/// Defines the layout of the 2. block of the color picker containing the RGBA part, Hex and buttons.
fn block2_layout<Message>(
    color_picker: &mut ColorPickerOverlay<'_, '_, Message>,
    renderer: &Renderer,
    bounds: Rectangle,
) -> Node
where
    Message: 'static + Clone,
{
    let block2_limits = Limits::new(Size::ZERO, bounds.size())
        .width(Length::Fill)
        .height(Length::Fill);

    // Layout buttons first to get their dimensions
    // Hex Copy Button (Tree index 0)
    let button_width = 40.0;
    let button_limits = block2_limits.width(Length::Fixed(button_width)); // Loose limits

    let mut hex_copy_button = color_picker.hex_copy_button.layout(
        &mut color_picker.tree.children[0],
        renderer,
        &button_limits,
    );

    // RGBA Copy Button (Tree index 1)
    let mut rgba_copy_button = color_picker.rgba_copy_button.layout(
        &mut color_picker.tree.children[1],
        renderer,
        &button_limits,
    );

    // Calculate height for text rows
    let text_row_height = renderer.default_size().0 + PADDING.y();

    // Text limits (Fill width minus button width and spacing)
    let text_limits = block2_limits
        .width(Length::Fill)
        .height(Length::Fixed(text_row_height))
        .shrink(Size::new(hex_copy_button.bounds().width + SPACING.0, 0.0));

    // Hex Text Layout
    let mut hex_text_layout = Row::<Message, iced::Theme, Renderer>::new()
        .width(Length::Fill)
        .height(Length::Fixed(text_row_height))
        .layout(color_picker.tree, renderer, &text_limits);

    // RGBA Text Layout
    let mut rgba_text_layout = Row::<Message, iced::Theme, Renderer>::new()
        .width(Length::Fill)
        .height(Length::Fixed(text_row_height))
        .layout(color_picker.tree, renderer, &text_limits);

    // RGBA Sliders
    // Available height for sliders
    let occupied_height =
        hex_text_layout.bounds().height + rgba_text_layout.bounds().height + 2.0 * SPACING.0;

    let sliders_limits = block2_limits.shrink(Size::new(0.0, occupied_height));

    let mut rgba_colors: Column<'_, Message, iced::Theme, Renderer> =
        Column::<Message, iced::Theme, Renderer>::new();

    for _ in 0..4 {
        rgba_colors = rgba_colors.push(
            Row::new()
                .align_y(Alignment::Center)
                .spacing(SPACING)
                .padding(PADDING)
                .height(Length::Fill)
                .push(
                    iced::widget::Text::new("X:")
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .push(
                    Row::new()
                        .width(Length::FillPortion(5))
                        .height(Length::Fill),
                )
                .push(
                    iced::widget::Text::new("XXX")
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                ),
        );
    }

    // RGBA Colors Tree (Index 3)
    let mut element: Element<Message, iced::Theme, Renderer> = Element::new(rgba_colors);
    // Ensure tree has enough children. We expect index 3.
    // Indices 0, 1, 2 are buttons (hex, rgba, close).
    while color_picker.tree.children.len() <= 3 {
        color_picker.tree.children.push(Tree::empty());
    }

    let rgba_tree = &mut color_picker.tree.children[3];
    rgba_tree.diff(element.as_widget_mut());

    let mut rgba_colors_layout =
        element
            .as_widget_mut()
            .layout(rgba_tree, renderer, &sliders_limits);

    // Positioning

    // 1. RGBA Sliders (Top)
    let rgba_colors_bounds = rgba_colors_layout.bounds();
    rgba_colors_layout = rgba_colors_layout.move_to(Point::new(
        rgba_colors_bounds.x + PADDING.left,
        rgba_colors_bounds.y + PADDING.top,
    ));
    let rgba_colors_bounds = rgba_colors_layout.bounds();

    // 2. Hex Row (Below Sliders)
    // Hex Text
    let hex_bounds = hex_text_layout.bounds();
    hex_text_layout = hex_text_layout.move_to(Point::new(
        hex_bounds.x + PADDING.left,
        hex_bounds.y + rgba_colors_bounds.height + PADDING.top + SPACING.0,
    ));
    let hex_bounds = hex_text_layout.bounds();

    // Hex Copy Button (Right of Hex Text)
    let hex_copy_bounds = hex_copy_button.bounds();

    // Center button vertically relative to text row
    let button_y_offset = (hex_bounds.height - hex_copy_bounds.height) / 2.0;

    hex_copy_button = hex_copy_button.move_to(Point::new(
        hex_bounds.x + hex_bounds.width + SPACING.0,
        hex_bounds.y + button_y_offset,
    ));

    // 3. RGBA Row (Below Hex Row)
    // RGBA Text
    let rgba_text_bounds = rgba_text_layout.bounds();
    rgba_text_layout = rgba_text_layout.move_to(Point::new(
        rgba_text_bounds.x + PADDING.left,
        rgba_text_bounds.y
            + rgba_colors_bounds.height
            + hex_bounds.height
            + PADDING.top
            + 2.0 * SPACING.0,
    ));
    let rgba_text_bounds = rgba_text_layout.bounds();

    // RGBA Copy Button (Right of RGBA Text)
    let rgba_copy_bounds = rgba_copy_button.bounds();

    // Center button vertically
    let button_y_offset = (rgba_text_bounds.height - rgba_copy_bounds.height) / 2.0;

    rgba_copy_button = rgba_copy_button.move_to(Point::new(
        rgba_text_bounds.x + rgba_text_bounds.width + SPACING.0,
        rgba_text_bounds.y + button_y_offset,
    ));

    Node::with_children(
        Size::new(
            rgba_colors_bounds.width + PADDING.x(),
            rgba_colors_bounds.height
                + hex_bounds.height
                + rgba_text_bounds.height
                + PADDING.y()
                + (2.0 * SPACING.0),
        ),
        vec![
            rgba_colors_layout,
            hex_text_layout,
            hex_copy_button,
            rgba_text_layout,
            rgba_copy_button,
        ],
    )
    .move_to(Point::new(bounds.x, bounds.y))
}

/// Draws the 1. block of the color picker containing the HSV part.
fn block1<Message>(
    renderer: &mut Renderer,
    color_picker: &ColorPickerOverlay<'_, '_, Message>,
    layout: Layout<'_>,
    cursor: Cursor,
    style_sheet: &HashMap<StyleState, Style>,
) where
    Message: Clone + 'static,
{
    // ----------- Block 1 ----------------------
    let hsv_color_layout = layout;

    // ----------- HSV Color ----------------------
    hsv_color(
        renderer,
        color_picker,
        hsv_color_layout,
        cursor,
        style_sheet,
    );

    // ----------- Block 1 end ------------------
}

/// Draws the 2. block of the color picker containing the RGBA part, Hex and buttons.
#[allow(clippy::too_many_arguments)]
fn block2<Message>(
    renderer: &mut Renderer,
    color_picker: &ColorPickerOverlay<'_, '_, Message>,
    layout: Layout<'_>,
    cursor: Cursor,
    theme: &iced::Theme,
    style: &renderer::Style,
    viewport: &Rectangle,
    style_sheet: &HashMap<StyleState, Style>,
) where
    Message: Clone + 'static,
{
    // ----------- Block 2 ----------------------
    let mut block2_children = layout.children();

    // ----------- RGBA Color ----------------------
    let rgba_color_layout = block2_children
        .next()
        .expect("Graphics: Layout should have a RGBA color layout");
    rgba_color(
        renderer,
        rgba_color_layout,
        &color_picker.state.color,
        cursor,
        style,
        style_sheet,
        color_picker.state.focus,
    );

    // ----------- Hex text ----------------------
    let hex_text_layout = block2_children
        .next()
        .expect("Graphics: Layout should have a hex text layout");
    hex_text(
        renderer,
        hex_text_layout,
        &color_picker.state.color,
        cursor,
        style,
        style_sheet,
        color_picker.state.focus,
    );

    // ----------- Hex Copy Button -------------------------
    let hex_copy_button_layout = block2_children
        .next()
        .expect("Graphics: Layout should have a hex copy button layout");

    color_picker.hex_copy_button.draw(
        &color_picker.tree.children[0],
        renderer,
        theme,
        style,
        hex_copy_button_layout,
        cursor,
        viewport,
    );

    // ----------- RGBA text ----------------------
    let rgba_text_layout = block2_children
        .next()
        .expect("Graphics: Layout should have a rgba text layout");
    rgba_text(
        renderer,
        rgba_text_layout,
        &color_picker.state.color,
        cursor,
        style,
        style_sheet,
        color_picker.state.focus,
    );

    // ----------- RGBA Copy Button -------------------------
    let rgba_copy_button_layout = block2_children
        .next()
        .expect("Graphics: Layout should have a rgba copy button layout");

    color_picker.rgba_copy_button.draw(
        &color_picker.tree.children[1],
        renderer,
        theme,
        style,
        rgba_copy_button_layout,
        cursor,
        viewport,
    );

    if color_picker.state.focus == Focus::HexCopy {
        let bounds = hex_copy_button_layout.bounds();
        if (bounds.width > 0.) && (bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: style_sheet[&StyleState::Focused].border_radius.into(),
                        width: 0.0,
                        color: style_sheet[&StyleState::Focused].border_color,
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        }
    }

    if color_picker.state.focus == Focus::RgbaCopy {
        let bounds = rgba_copy_button_layout.bounds();
        if (bounds.width > 0.) && (bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: style_sheet[&StyleState::Focused].border_radius.into(),
                        width: 0.0,
                        color: style_sheet[&StyleState::Focused].border_color,
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        }
    }
    // ----------- Block 2 end ------------------
}

/// Draws the RGBA text representation of the color.
fn rgba_text(
    renderer: &mut Renderer,
    layout: Layout<'_>,
    color: &Color,
    cursor: Cursor,
    _style: &renderer::Style,
    style_sheet: &HashMap<StyleState, Style>,
    _focus: Focus,
) {
    let hsv: Hsv = (*color).into();

    let text_style_state = if cursor.is_over(layout.bounds()) {
        StyleState::Hovered
    } else {
        StyleState::Active
    };

    let bounds = layout.bounds();
    if (bounds.width > 0.) && (bounds.height > 0.) {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: style_sheet[&text_style_state].bar_border_radius.into(),
                    width: 0.0,
                    color: style_sheet[&text_style_state].bar_border_color,
                },
                ..renderer::Quad::default()
            },
            *color,
        );
    }

    let rgba_string = format!(
        "rgba({}, {}, {}, {:.2})",
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
        color.a
    );

    renderer.fill_text(
        Text {
            content: rgba_string,
            bounds: Size::new(bounds.width, bounds.height),
            size: renderer.default_size(),
            font: renderer.default_font(),
            align_x: text::Alignment::Center,
            align_y: Vertical::Center,
            line_height: iced::widget::text::LineHeight::Relative(1.3),
            shaping: iced::widget::text::Shaping::Basic,
            wrapping: Wrapping::default(),
        },
        Point::new(bounds.center_x(), bounds.center_y()),
        Color {
            a: 1.0,
            ..Hsv {
                hue: 0,
                saturation: 0.0,
                value: if hsv.value < 0.5 { 1.0 } else { 0.0 },
            }
            .into()
        },
        bounds,
    );
}

/// Draws the HSV color area.
#[allow(clippy::too_many_lines)]
fn hsv_color<Message>(
    renderer: &mut Renderer,
    color_picker: &ColorPickerOverlay<'_, '_, Message>,
    layout: Layout<'_>,
    _cursor: Cursor,
    style_sheet: &HashMap<StyleState, Style>,
) where
    Message: Clone,
{
    let mut hsv_color_children = layout.children();
    let hsv_color: Hsv = color_picker.state.color.into();

    let sat_value_layout = hsv_color_children
        .next()
        .expect("Graphics: Layout should have a sat/value layout");
    let mut sat_value_style_state = StyleState::Active;
    if color_picker.state.focus == Focus::SatValue {
        sat_value_style_state = sat_value_style_state.max(StyleState::Focused);
    }
    // if cursor.is_over(sat_value_layout.bounds()) {
    //     sat_value_style_state = sat_value_style_state.max(StyleState::Hovered);
    // }

    let sat_value_style = style_sheet
        .get(&sat_value_style_state)
        .expect("Style Sheet not found.");
    let radius = sat_value_style.bar_border_radius;

    let geometry = color_picker.state.sat_value_canvas_cache.draw(
        renderer,
        sat_value_layout.bounds().size(),
        |frame| {
            let column_count = frame.width() as u16;
            let row_count = frame.height() as u16;
            let width = frame.width();
            let height = frame.height();

            for column in 0..column_count {
                for row in 0..row_count {
                    let x = f32::from(column);
                    let y = f32::from(row);

                    // Check rounded corners
                    let mut visible = true;
                    if x < radius && y < radius {
                        visible = (x - radius).powi(2) + (y - radius).powi(2) <= radius.powi(2);
                    } else if x > width - radius && y < radius {
                        visible =
                            (x - (width - radius)).powi(2) + (y - radius).powi(2) <= radius.powi(2);
                    } else if x < radius && y > height - radius {
                        visible = (x - radius).powi(2) + (y - (height - radius)).powi(2)
                            <= radius.powi(2);
                    } else if x > width - radius && y > height - radius {
                        visible = (x - (width - radius)).powi(2) + (y - (height - radius)).powi(2)
                            <= radius.powi(2);
                    }

                    if visible {
                        let saturation = x / width;
                        let value = 1.0 - (y / height);

                        frame.fill_rectangle(
                            Point::new(x, y),
                            Size::new(1.0, 1.0),
                            Color::from(Hsv::from_hsv(hsv_color.hue, saturation, value)),
                        );
                    }
                }
            }

            let stroke = Stroke {
                style: canvas::Style::Solid(
                    Hsv {
                        hue: 0,
                        saturation: 0.0,
                        value: 1.0 - hsv_color.value,
                    }
                    .into(),
                ),
                width: 3.0,
                line_cap: LineCap::Round,
                ..Stroke::default()
            };

            let saturation = hsv_color.saturation * frame.width();
            let value = (1.0 - hsv_color.value) * frame.height();

            // Clamp vertical line (saturation)
            let mut v_y_start = 0.0;
            let mut v_y_end = frame.height();
            if saturation < radius {
                let d = (radius.powi(2) - (saturation - radius).powi(2)).sqrt();
                v_y_start = radius - d;
                v_y_end = frame.height() - v_y_start;
            } else if saturation > frame.width() - radius {
                let d = (radius.powi(2) - (saturation - (frame.width() - radius)).powi(2)).sqrt();
                v_y_start = radius - d;
                v_y_end = frame.height() - v_y_start;
            }

            frame.stroke(
                &Path::line(
                    Point::new(saturation, v_y_start),
                    Point::new(saturation, v_y_end),
                ),
                stroke,
            );

            // Clamp horizontal line (value)
            let mut h_x_start = 0.0;
            let mut h_x_end = frame.width();
            if value < radius {
                let d = (radius.powi(2) - (value - radius).powi(2)).sqrt();
                h_x_start = radius - d;
                h_x_end = frame.width() - h_x_start;
            } else if value > frame.height() - radius {
                let d = (radius.powi(2) - (value - (frame.height() - radius)).powi(2)).sqrt();
                h_x_start = radius - d;
                h_x_end = frame.width() - h_x_start;
            }

            frame.stroke(
                &Path::line(Point::new(h_x_start, value), Point::new(h_x_end, value)),
                stroke,
            );

            let stroke = Stroke {
                style: canvas::Style::Solid(sat_value_style.bar_border_color),
                width: 0.0,
                line_cap: LineCap::Round,
                ..Stroke::default()
            };

            frame.stroke(
                &Path::rounded_rectangle(
                    Point::new(0.0, 0.0),
                    Size::new(frame.size().width, frame.size().height),
                    radius.into(),
                ),
                stroke,
            );
        },
    );

    let translation = Vector::new(sat_value_layout.bounds().x, sat_value_layout.bounds().y);
    renderer.with_translation(translation, |renderer| {
        renderer.draw_geometry(geometry);
    });

    let hue_layout = hsv_color_children
        .next()
        .expect("Graphics: Layout should have a hue layout");
    let mut hue_style_state = StyleState::Active;
    if color_picker.state.focus == Focus::Hue {
        hue_style_state = hue_style_state.max(StyleState::Focused);
    }
    // if cursor.is_over(hue_layout.bounds()) {
    //     hue_style_state = hue_style_state.max(StyleState::Hovered);
    // }

    let hue_style = style_sheet
        .get(&hue_style_state)
        .expect("Style Sheet not found.");
    let radius = hue_style.bar_border_radius;

    let geometry =
        color_picker
            .state
            .hue_canvas_cache
            .draw(renderer, hue_layout.bounds().size(), |frame| {
                let column_count = frame.width() as u16;
                let width = frame.width();
                let height = frame.height();

                for column in 0..column_count {
                    let hue = (f32::from(column) * 360.0 / width) as u16;

                    let hsv_color = Hsv::from_hsv(hue, 1.0, 1.0);
                    let stroke = Stroke {
                        style: canvas::Style::Solid(hsv_color.into()),
                        width: 1.0,
                        line_cap: LineCap::Round,
                        ..Stroke::default()
                    };

                    let x = f32::from(column);
                    let mut y_start = 0.0;
                    let mut y_end = height;

                    if x < radius {
                        let d = (radius.powi(2) - (x - radius).powi(2)).sqrt();
                        y_start = radius - d;
                        y_end = height - y_start;
                    } else if x > width - radius {
                        let d = (radius.powi(2) - (x - (width - radius)).powi(2)).sqrt();
                        y_start = radius - d;
                        y_end = height - y_start;
                    }

                    if y_end > y_start {
                        frame.stroke(
                            &Path::line(Point::new(x, y_start), Point::new(x, y_end)),
                            stroke,
                        );
                    }
                }

                let stroke = Stroke {
                    style: canvas::Style::Solid(Color::BLACK),
                    width: 3.0,
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                };

                let column = f32::from(hsv_color.hue) * frame.width() / 360.0;

                let mut y_start = 0.0;
                let mut y_end = frame.height();
                if column < radius {
                    let d = (radius.powi(2) - (column - radius).powi(2)).sqrt();
                    y_start = radius - d;
                    y_end = frame.height() - y_start;
                } else if column > frame.width() - radius {
                    let d = (radius.powi(2) - (column - (frame.width() - radius)).powi(2)).sqrt();
                    y_start = radius - d;
                    y_end = frame.height() - y_start;
                }

                frame.stroke(
                    &Path::line(Point::new(column, y_start), Point::new(column, y_end)),
                    stroke,
                );

                let stroke = Stroke {
                    style: canvas::Style::Solid(hue_style.bar_border_color),
                    width: 0.0,
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                };

                frame.stroke(
                    &Path::rounded_rectangle(
                        Point::new(0.0, 0.0),
                        Size::new(frame.size().width, frame.size().height),
                        radius.into(),
                    ),
                    stroke,
                );
            });

    let translation = Vector::new(hue_layout.bounds().x, hue_layout.bounds().y);
    renderer.with_translation(translation, |renderer| {
        renderer.draw_geometry(geometry);
    });
}

/// Draws the RGBA color area.
#[allow(clippy::too_many_lines)]
fn rgba_color(
    renderer: &mut Renderer,
    layout: Layout<'_>,
    color: &Color,
    cursor: Cursor,
    style: &renderer::Style,
    style_sheet: &HashMap<StyleState, Style>,
    focus: Focus,
) {
    let mut rgba_color_children = layout.children();

    let f = |renderer: &mut Renderer,
             layout: Layout,
             label: &str,
             color: Color,
             value: f32,
             cursor: Cursor,
             target: Focus| {
        let mut children = layout.children();

        let label_layout = children
            .next()
            .expect("Graphics: Layout should have a label layout");
        let bar_layout = children
            .next()
            .expect("Graphics: Layout should have a bar layout");
        let value_layout = children
            .next()
            .expect("Graphics: Layout should have a value layout");

        // Label
        renderer.fill_text(
            Text {
                content: label.to_owned(),
                bounds: Size::new(label_layout.bounds().width, label_layout.bounds().height),
                size: renderer.default_size(),
                font: renderer.default_font(),
                align_x: TextAlignment::Center,
                align_y: Vertical::Center,
                line_height: widget_text::LineHeight::Relative(1.3),
                shaping: widget_text::Shaping::Basic,
                wrapping: Wrapping::None,
            },
            Point::new(
                label_layout.bounds().center_x(),
                label_layout.bounds().center_y(),
            ),
            style.text_color,
            label_layout.bounds(),
        );

        let bar_bounds = bar_layout.bounds();

        let bar_style_state = if cursor.is_over(bar_bounds) {
            StyleState::Hovered
        } else {
            StyleState::Active
        };

        // Bar background
        let background_bounds = Rectangle {
            x: bar_bounds.x,
            y: bar_bounds.y,
            width: bar_bounds.width * value,
            height: bar_bounds.height,
        };
        if (background_bounds.width > 0.) && (background_bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: background_bounds,
                    border: Border {
                        radius: style_sheet
                            .get(&bar_style_state)
                            .expect("Style Sheet not found.")
                            .bar_border_radius
                            .into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..renderer::Quad::default()
                },
                color,
            );
        }

        // Bar
        if (bar_bounds.width > 0.) && (bar_bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: bar_bounds,
                    border: Border {
                        radius: style_sheet
                            .get(&bar_style_state)
                            .expect("Style Sheet not found.")
                            .bar_border_radius
                            .into(),
                        width: 0.0,
                        color: style_sheet
                            .get(&bar_style_state)
                            .expect("Style Sheet not found.")
                            .bar_border_color,
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        }

        // Value
        renderer.fill_text(
            Text {
                content: format!("{}", (255.0 * value) as u8),
                bounds: Size::new(value_layout.bounds().width, value_layout.bounds().height),
                size: renderer.default_size(),
                font: renderer.default_font(),
                align_x: TextAlignment::Center,
                align_y: Vertical::Center,
                line_height: widget_text::LineHeight::Relative(1.3),
                shaping: widget_text::Shaping::Basic,
                wrapping: Wrapping::None,
            },
            Point::new(
                value_layout.bounds().center_x(),
                value_layout.bounds().center_y(),
            ),
            style.text_color,
            value_layout.bounds(),
        );

        let bounds = layout.bounds();
        if (focus == target) && (bounds.width > 0.) && (bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: style_sheet
                            .get(&StyleState::Focused)
                            .expect("Style Sheet not found.")
                            .border_radius
                            .into(),
                        width: 0.0,
                        color: style_sheet
                            .get(&StyleState::Focused)
                            .expect("Style Sheet not found.")
                            .border_color,
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        }
    };

    // Red
    let red_row_layout = rgba_color_children
        .next()
        .expect("Graphics: Layout should have a red row layout");

    f(
        renderer,
        red_row_layout,
        "R",
        Color::from_rgb(color.r, 0.0, 0.0),
        color.r,
        cursor,
        Focus::Red,
    );

    // Green
    let green_row_layout = rgba_color_children
        .next()
        .expect("Graphics: Layout should have a green row layout");

    f(
        renderer,
        green_row_layout,
        "G",
        Color::from_rgb(0.0, color.g, 0.0),
        color.g,
        cursor,
        Focus::Green,
    );

    // Blue
    let blue_row_layout = rgba_color_children
        .next()
        .expect("Graphics: Layout should have a blue row layout");

    f(
        renderer,
        blue_row_layout,
        "B",
        Color::from_rgb(0.0, 0.0, color.b),
        color.b,
        cursor,
        Focus::Blue,
    );

    // Alpha
    let alpha_row_layout = rgba_color_children
        .next()
        .expect("Graphics: Layout should have an alpha row layout");

    f(
        renderer,
        alpha_row_layout,
        "A",
        Color::from_rgba(0.0, 0.0, 0.0, color.a),
        color.a,
        cursor,
        Focus::Alpha,
    );
}

/// Draws the hex text representation of the color.
fn hex_text(
    renderer: &mut Renderer,
    layout: Layout<'_>,
    color: &Color,
    cursor: Cursor,
    _style: &renderer::Style,
    style_sheet: &HashMap<StyleState, Style>,
    _focus: Focus,
) {
    let hsv: Hsv = (*color).into();

    let hex_text_style_state = if cursor.is_over(layout.bounds()) {
        StyleState::Hovered
    } else {
        StyleState::Active
    };

    let bounds = layout.bounds();
    if (bounds.width > 0.) && (bounds.height > 0.) {
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: style_sheet[&hex_text_style_state].bar_border_radius.into(),
                    width: 0.0,
                    color: style_sheet[&hex_text_style_state].bar_border_color,
                },
                ..renderer::Quad::default()
            },
            *color,
        );
    }

    renderer.fill_text(
        Text {
            content: color.as_hex_string(),
            bounds: Size::new(bounds.width, bounds.height),
            size: renderer.default_size(),
            font: renderer.default_font(),
            align_x: text::Alignment::Center,
            align_y: Vertical::Center,
            line_height: iced::widget::text::LineHeight::Relative(1.3),
            shaping: iced::widget::text::Shaping::Basic,
            wrapping: Wrapping::default(),
        },
        Point::new(bounds.center_x(), bounds.center_y()),
        Color {
            a: 1.0,
            ..Hsv {
                hue: 0,
                saturation: 0.0,
                value: if hsv.value < 0.5 { 1.0 } else { 0.0 },
            }
            .into()
        },
        bounds,
    );
}

/// The state of the [`ColorPickerOverlay`].
#[derive(Debug)]
pub struct State {
    /// The selected color of the [`ColorPickerOverlay`].
    pub(crate) color: Color,
    /// The color used to initialize [`ColorPickerOverlay`].
    pub(crate) initial_color: Color,
    /// The cache of the sat/value canvas of the [`ColorPickerOverlay`].
    pub(crate) sat_value_canvas_cache: canvas::Cache,
    /// The cache of the hue canvas of the [`ColorPickerOverlay`].
    pub(crate) hue_canvas_cache: canvas::Cache,
    /// The dragged color bar of the [`ColorPickerOverlay`].
    pub(crate) color_bar_dragged: ColorBarDragged,
    /// the focus of the [`ColorPickerOverlay`].
    pub(crate) focus: Focus,
    /// The previously pressed keyboard modifiers.
    pub(crate) keyboard_modifiers: keyboard::Modifiers,
}

impl State {
    /// Creates a new State with the given color.
    #[must_use]
    pub fn new(color: Color) -> Self {
        Self {
            color,
            initial_color: color,
            ..Self::default()
        }
    }

    /// Reset cached canvas when internal state is modified.
    ///
    /// If the color has changed, empty all canvas caches
    /// as they (unfortunately) do not depend on the picker state.
    fn clear_cache(&self) {
        self.sat_value_canvas_cache.clear();
        self.hue_canvas_cache.clear();
    }

    /// Synchronize the color with an externally provided value.
    pub(crate) fn force_synchronize(&mut self, color: Color) {
        self.initial_color = color;
        self.color = color;
        self.clear_cache();
    }
}

impl Default for State {
    fn default() -> Self {
        let default_color = Color::from_rgb(0.5, 0.25, 0.25);
        Self {
            color: default_color,
            initial_color: default_color,
            sat_value_canvas_cache: canvas::Cache::default(),
            hue_canvas_cache: canvas::Cache::default(),
            color_bar_dragged: ColorBarDragged::None,
            focus: Focus::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
        }
    }
}

/// Just a workaround to pass the button states from the tree to the overlay
#[allow(missing_debug_implementations)]
pub struct ColorPickerOverlayButtons<'a, Message>
where
    Message: Clone,
{
    /// The hex copy button of the [`ColorPickerOverlay`].
    hex_copy_button: Element<'a, Message, iced::Theme, Renderer>,
    /// The rgba copy button of the [`ColorPickerOverlay`].
    rgba_copy_button: Element<'a, Message, iced::Theme, Renderer>,
    /// The close button of the [`ColorPickerOverlay`].
    close_button: Element<'a, Message, iced::Theme, Renderer>,
}

impl<'a, Message> Default for ColorPickerOverlayButtons<'a, Message>
where
    Message: 'a + Clone,
{
    fn default() -> Self {
        Self {
            hex_copy_button: Button::new(iced::widget::Text::new("Copy")).into(),
            rgba_copy_button: Button::new(iced::widget::Text::new("Copy")).into(),
            close_button: Button::new(iced::widget::Text::new("X")).into(),
        }
    }
}

#[allow(clippy::unimplemented)]
impl<Message> Widget<Message, iced::Theme, Renderer> for ColorPickerOverlayButtons<'_, Message>
where
    Message: Clone,
{
    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(&self.hex_copy_button),
            Tree::new(&self.rgba_copy_button),
            Tree::new(&self.close_button),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[
            &self.hex_copy_button,
            &self.rgba_copy_button,
            &self.close_button,
        ]);
    }

    fn size(&self) -> Size<Length> {
        unimplemented!("This should never be reached!")
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, _limits: &Limits) -> Node {
        unimplemented!("This should never be reached!")
    }

    fn draw(
        &self,
        _state: &Tree,
        _renderer: &mut Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        unimplemented!("This should never be reached!")
    }
}

impl<'a, Message> From<ColorPickerOverlayButtons<'a, Message>>
    for Element<'a, Message, iced::Theme, Renderer>
where
    Message: 'a + Clone,
{
    fn from(overlay: ColorPickerOverlayButtons<'a, Message>) -> Self {
        Self::new(overlay)
    }
}

/// The state of the currently dragged area.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ColorBarDragged {
    /// No area is focussed.
    #[default]
    None,

    /// The saturation/value area is focussed.
    SatValue,

    /// The hue area is focussed.
    Hue,

    /// The red area is focussed.
    Red,

    /// The green area is focussed.
    Green,

    /// The blue area is focussed.
    Blue,

    /// The alpha area is focussed.
    Alpha,
}

/// An enumeration of all focusable element of the [`ColorPickerOverlay`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Focus {
    /// Nothing is in focus.
    #[default]
    None,

    /// The overlay itself is in focus.
    Overlay,

    /// The saturation and value area is in focus.
    SatValue,

    /// The hue bar is in focus.
    Hue,

    /// The red bar is in focus.
    Red,

    /// The green bar is in focus.
    Green,

    /// The blue bar is in focus.
    Blue,

    /// The alpha bar is in focus.
    Alpha,

    /// The hex copy button is in focus.
    HexCopy,

    /// The rgba copy button is in focus.
    RgbaCopy,

    /// The close button is in focus.
    Close,
}

impl Focus {
    /// Gets the next focusable element.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Overlay => Self::SatValue,
            Self::SatValue => Self::Hue,
            Self::Hue => Self::Red,
            Self::Red => Self::Green,
            Self::Green => Self::Blue,
            Self::Blue => Self::Alpha,
            Self::Alpha => Self::HexCopy,
            Self::HexCopy => Self::RgbaCopy,
            Self::RgbaCopy => Self::Close,
            Self::Close | Self::None => Self::Overlay,
        }
    }

    /// Gets the previous focusable element.
    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Overlay => Self::Close,
            Self::SatValue => Self::Overlay,
            Self::Hue => Self::SatValue,
            Self::Red => Self::Hue,
            Self::Green => Self::Red,
            Self::Blue => Self::Green,
            Self::Alpha => Self::Blue,
            Self::HexCopy => Self::Alpha,
            Self::RgbaCopy => Self::HexCopy,
            Self::Close => Self::RgbaCopy,
        }
    }
}
