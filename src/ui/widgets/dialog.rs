use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Length};

use crate::ui::theme::PastryTheme;
use crate::ui::util::ui_radius;

pub fn dialog_card<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Container<'a, Message>
where
    Message: 'a,
{
    container(content).style(|theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(theme.dialog_background())),
        border: iced::Border {
            radius: ui_radius(12.0).into(),
            width: 1.0,
            color: theme.divider(),
        },
        shadow: iced::Shadow {
            color: theme.shadow(),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    })
}

pub fn confirm_dialog<'a, Message: Clone + 'a>(
    title: &'a str,
    message: &'a str,
    cancel_text: &'a str,
    confirm_text: &'a str,
    on_cancel: Message,
    on_confirm: Message,
) -> Element<'a, Message> {
    let dialog = dialog_card(
        column![
            text(title)
                .size(16)
                .style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.text())
                }),
            Space::new().height(Length::Fixed(16.0)),
            text(message)
                .size(13)
                .style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.text_secondary())
                }),
            Space::new().height(Length::Fixed(24.0)),
            row![
                Space::new().width(Length::Fill),
                button(text(cancel_text).size(12))
                    .on_press(on_cancel)
                    .padding([6, 16])
                    .style(|theme: &iced::Theme, status| button::Style {
                        background: Some(iced::Background::Color(
                            if matches!(status, button::Status::Hovered) {
                                theme.button_hover_background()
                            } else {
                                theme.button_background()
                            }
                        )),
                        text_color: theme.text(),
                        border: iced::Border {
                            radius: ui_radius(6.0).into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                button(text(confirm_text).size(12))
                    .on_press(on_confirm)
                    .padding([6, 16])
                    .style(|theme: &iced::Theme, _| button::Style {
                        background: Some(iced::Background::Color(theme.danger())),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            radius: ui_radius(6.0).into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            ]
            .spacing(8)
        ]
        .width(Length::Fixed(320.0))
        .padding(24),
    );

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .align_y(iced::Alignment::Start)
        .padding(100)
        .into()
}
