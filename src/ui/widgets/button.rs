use iced::Color;
use iced::Theme;
use iced::widget::button;

use super::icon::{Icon, icon_svg};
use crate::ui::theme::PastryTheme;

pub fn icon_button_hover<Message, F>(
    icon: Icon,
    icon_size: u32,
    padding: impl Into<iced::Padding>,
    radius: f32,
    on_press: Message,
    color: F,
) -> iced::widget::Button<'static, Message>
where
    Message: Clone + 'static,
    F: Fn(&Theme) -> Color + 'static,
{
    button(icon_svg(icon, icon_size, color))
        .on_press(on_press)
        .padding(padding)
        .style(move |theme, status| {
            let is_hovered = matches!(status, button::Status::Hovered);
            button::Style {
                background: if is_hovered {
                    Some(iced::Background::Color(theme.button_hover_background()))
                } else {
                    None
                },
                text_color: theme.text(),
                border: if is_hovered {
                    iced::Border {
                        radius: radius.into(),
                        ..Default::default()
                    }
                } else {
                    iced::Border::default()
                },
                ..Default::default()
            }
        })
}

pub fn icon_button_fill<Message, F>(
    icon: Icon,
    icon_size: u32,
    padding: impl Into<iced::Padding>,
    radius: f32,
    on_press: Message,
    color: F,
) -> iced::widget::Button<'static, Message>
where
    Message: Clone + 'static,
    F: Fn(&Theme) -> Color + 'static,
{
    button(icon_svg(icon, icon_size, color))
        .on_press(on_press)
        .padding(padding)
        .style(move |theme, status| {
            let background = if matches!(status, button::Status::Hovered) {
                theme.button_hover_background()
            } else {
                theme.button_background()
            };
            button::Style {
                background: Some(iced::Background::Color(background)),
                text_color: theme.text(),
                border: iced::Border {
                    radius: radius.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
}
