use iced::widget::{container, text, text_input};
use iced::{Element, Length};

use crate::ui::theme::PastryTheme;
use crate::ui::util::ui_radius;

pub fn search_input_dialog<'a, Message, F>(
    placeholder: &'a str,
    value: &'a str,
    on_input: F,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
    F: 'a + Fn(String) -> Message,
{
    text_input(placeholder, value)
        .on_input(on_input)
        .padding(8)
        .size(13)
        .style(|theme: &iced::Theme, status| text_input::Style {
            background: iced::Background::Color(theme.input_background()),
            border: iced::Border {
                radius: ui_radius(6.0).into(),
                width: 1.0,
                color: if matches!(status, text_input::Status::Focused { .. }) {
                    theme.primary()
                } else {
                    theme.input_border()
                },
            },
            icon: iced::Color::TRANSPARENT,
            placeholder: theme.text_placeholder(),
            value: theme.text(),
            selection: theme.primary(),
        })
        .into()
}

pub fn search_input_card<'a, Message, F>(
    placeholder: &'a str,
    value: &'a str,
    on_input: F,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
    F: 'a + Fn(String) -> Message,
{
    text_input(placeholder, value)
        .on_input(on_input)
        .padding(8)
        .size(13)
        .style(|theme: &iced::Theme, status| {
            use iced::widget::text_input::Status;

            let is_focused = matches!(status, Status::Focused { .. });
            text_input::Style {
                background: iced::Background::Color(theme.card_background()),
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
        })
        .into()
}

pub fn empty_state<'a, Message>(
    message: &'a str,
    padding: f32,
    center_y: bool,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let card = container(
        text(message)
            .size(13)
            .style(|theme: &iced::Theme| text::Style {
                color: Some(theme.text_secondary()),
            }),
    )
    .padding(padding)
    .center_x(Length::Fill);

    if center_y {
        card.center_y(Length::Fill).into()
    } else {
        card.into()
    }
}
