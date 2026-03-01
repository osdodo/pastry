use iced::Element;
use iced::Theme;

use crate::{
    app::Message,
    features::color_picker::Message as ColorPickerMessage,
    ui::theme::PastryTheme,
    ui::util::color::Color as PastryColor,
    ui::util::ui_radius,
    ui::widgets::{ColorPicker, style},
};

pub fn color_picker_overlay<'a>(
    show: bool,
    color: PastryColor,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let iced_color = iced::Color::from_rgba(color.r, color.g, color.b, color.a);

    ColorPicker::new(
        show,
        iced_color,
        content,
        Message::ColorPicker(ColorPickerMessage::CloseColorPicker),
        move |c| {
            Message::ColorPicker(ColorPickerMessage::ColorPickerSubmitted(PastryColor::new(
                c.r, c.g, c.b, c.a,
            )))
        },
    )
    .style(Box::new(|theme: &Theme, _status| {
        style::color_picker::Style {
            background: iced::Background::Color(theme.dialog_background()),
            border_color: theme.divider(),
            border_radius: ui_radius(10.0),
            border_width: 1.0,
            bar_border_color: theme.palette().text,
            bar_border_radius: ui_radius(5.0),
            bar_border_width: 1.0,
        }
    }))
    .into()
}
