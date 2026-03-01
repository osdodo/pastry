//! Use a color picker as an input element for picking colors.
//!
//! *This API requires the following crate features to be activated: `color_picker`*

use std::ops::{Deref, DerefMut};

use iced::advanced::{
    Clipboard, Layout, Shell, Widget,
    layout::{Limits, Node},
    mouse::{self, Cursor},
    overlay, renderer,
    widget::{
        Operation,
        tree::{self, Tag, Tree},
    },
};
use iced::{Color, Element, Event, Length, Point, Rectangle, Renderer, Vector};

use super::overlay::color_picker::{self, ColorPickerOverlay, ColorPickerOverlayButtons};
pub use crate::ui::widgets::style::{self, Status, color_picker::Style};

//TODO: Remove ignore when Null is updated. Temp fix for Test runs
/// An input element for picking colors.
///
/// # Example
/// ```ignore
/// # use iced_aw::ColorPicker;
/// # use iced::{Color, widget::{button, Button, Text}};
/// #
/// #[derive(Clone, Debug)]
/// enum Message {
///     Open,
///     Cancel,
///     Submit(Color),
/// }
///
/// let color_picker = ColorPicker::new(
///     true,
///     Color::default(),
///     Button::new(Text::new("Pick color"))
///         .on_press(Message::Open),
///     Message::Cancel,
///     Message::Submit,
/// );
/// ```
#[allow(missing_debug_implementations)]
pub struct ColorPicker<'a, Message>
where
    Message: Clone,
{
    /// Show the picker.
    show_picker: bool,
    /// The color to show.
    color: Color,
    /// The underlying element.
    underlay: Element<'a, Message, iced::Theme, Renderer>,
    /// The message that is sent if the cancel button of the [`ColorPickerOverlay`] is pressed.
    on_cancel: Message,
    /// The function that produces a message when the submit button of the [`ColorPickerOverlay`] is pressed.
    on_submit: Box<dyn Fn(Color) -> Message>,
    /// Optional function that produces a message when the color changes during selection (real-time updates).
    on_color_change: Option<Box<dyn Fn(Color) -> Message>>,
    /// The style of the [`ColorPickerOverlay`].
    class: <iced::Theme as style::color_picker::Catalog>::Class<'a>,
    /// The buttons of the overlay.
    overlay_state: Element<'a, Message, iced::Theme, Renderer>,
}

impl<'a, Message> ColorPicker<'a, Message>
where
    Message: 'a + Clone,
{
    /// Creates a new [`ColorPicker`] wrapping around the given underlay.
    ///
    /// It expects:
    ///     * if the overlay of the color picker is visible.
    ///     * the initial color to show.
    ///     * the underlay [`Element`] on which this [`ColorPicker`]
    ///         will be wrapped around.
    ///     * a message that will be send when the cancel button of the [`ColorPicker`]
    ///         is pressed.
    ///     * a function that will be called when the submit button of the [`ColorPicker`]
    ///         is pressed, which takes the picked [`Color`] value.
    pub fn new<U, F>(
        show_picker: bool,
        color: Color,
        underlay: U,
        on_cancel: Message,
        on_submit: F,
    ) -> Self
    where
        U: Into<Element<'a, Message, iced::Theme, Renderer>>,
        F: 'static + Fn(Color) -> Message,
    {
        Self {
            show_picker,
            color,
            underlay: underlay.into(),
            on_cancel,
            on_submit: Box::new(on_submit),
            on_color_change: None,
            class: <iced::Theme as style::color_picker::Catalog>::default(),
            overlay_state: Element::new(ColorPickerOverlayButtons::default()),
        }
    }

    /// Sets a callback that will be called whenever the color changes during selection (real-time updates).
    #[must_use]
    pub fn on_color_change<F>(mut self, on_color_change: F) -> Self
    where
        F: 'static + Fn(Color) -> Message,
    {
        self.on_color_change = Some(Box::new(on_color_change));
        self
    }

    /// Sets the style of the [`ColorPicker`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&iced::Theme, Status) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }

    /// Sets the class of the input of the [`ColorPicker`].
    #[must_use]
    pub fn class(
        mut self,
        class: impl Into<<iced::Theme as style::color_picker::Catalog>::Class<'a>>,
    ) -> Self {
        self.class = class.into();
        self
    }
}

/// The state of the [`ColorPicker`].
#[derive(Debug, Default)]
pub struct State {
    /// The state of the overlay.
    pub(crate) overlay_state: color_picker::State,
    /// Was overlay shown during the previous render?
    pub(crate) old_show_picker: bool,
}

impl State {
    /// Creates a new [`State`].
    #[must_use]
    pub fn new(color: Color) -> Self {
        Self {
            overlay_state: color_picker::State::new(color),
            old_show_picker: false,
        }
    }

    /// Synchronize with the provided color if it was changed or picker was reopened
    ///
    /// Keep the overlay state in sync. While overlay is open, it "owns" the value
    /// (there is no other way the user can update its value). When it is reopened,
    /// reset the color to the initial one.
    fn synchronize(&mut self, show_picker: bool, color: Color) {
        if show_picker && (!self.old_show_picker || self.overlay_state.initial_color != color) {
            self.overlay_state.force_synchronize(color);
        }
        self.old_show_picker = show_picker;
    }
}

impl Deref for State {
    type Target = color_picker::State;

    fn deref(&self) -> &Self::Target {
        &self.overlay_state
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.overlay_state
    }
}

impl<'a, Message> Widget<Message, iced::Theme, Renderer> for ColorPicker<'a, Message>
where
    Message: 'static + Clone,
{
    fn tag(&self) -> Tag {
        Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.color))
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.underlay), Tree::new(&self.overlay_state)]
    }

    fn diff(&self, tree: &mut Tree) {
        let color_picker_state = tree.state.downcast_mut::<State>();

        color_picker_state.synchronize(self.show_picker, self.color);

        tree.diff_children(&[&self.underlay, &self.overlay_state]);
    }

    fn size(&self) -> iced::Size<Length> {
        self.underlay.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.underlay
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.underlay.as_widget_mut().update(
            &mut state.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.underlay.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        self.underlay.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        self.underlay
            .as_widget_mut()
            .operate(&mut state.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, Renderer>> {
        let picker_state: &mut State = tree.state.downcast_mut();

        if !self.show_picker {
            return self.underlay.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            );
        }

        let bounds = layout.bounds();
        let position = Point::new(bounds.center_x(), bounds.center_y());

        Some(
            ColorPickerOverlay::new(
                picker_state,
                self.on_cancel.clone(),
                &self.on_submit,
                self.on_color_change.as_deref(),
                position,
                &self.class,
                &mut tree.children[1],
                *viewport,
            )
            .overlay(),
        )
    }
}

impl<'a, Message> From<ColorPicker<'a, Message>> for Element<'a, Message, iced::Theme, Renderer>
where
    Message: 'static + Clone,
{
    fn from(color_picker: ColorPicker<'a, Message>) -> Self {
        Element::new(color_picker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMessage {
        Cancel,
        Submit(Color),
    }

    type TestColorPicker<'a> = ColorPicker<'a, TestMessage>;

    fn create_test_button() -> iced::widget::Button<'static, TestMessage, iced::Theme> {
        iced::widget::button(iced::widget::text::Text::new("Pick"))
    }

    #[test]
    fn color_picker_new_with_picker_hidden() {
        let color = Color::from_rgb(0.5, 0.5, 0.5);
        let button = create_test_button();

        let picker = TestColorPicker::new(
            false,
            color,
            button,
            TestMessage::Cancel,
            TestMessage::Submit,
        );

        assert!(!picker.show_picker);
        assert_eq!(picker.color, color);
    }

    #[test]
    fn color_picker_new_with_picker_shown() {
        let color = Color::from_rgb(0.3, 0.6, 0.9);
        let button = create_test_button();

        let picker = TestColorPicker::new(
            true,
            color,
            button,
            TestMessage::Cancel,
            TestMessage::Submit,
        );

        assert!(picker.show_picker);
        assert_eq!(picker.color, color);
    }

    #[test]
    fn color_picker_state_new() {
        let color = Color::from_rgb(0.5, 0.5, 0.5);
        let state = State::new(color);

        assert!(!state.old_show_picker);
    }

    #[test]
    fn color_picker_state_default() {
        let state = State::default();

        assert!(!state.old_show_picker);
    }
}
