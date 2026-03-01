use iced::widget::{Space, container};
use iced::{Element, Length, Theme};

use crate::ui::constants::WINDOW_RADIUS;
use crate::ui::theme::PastryTheme;

pub fn draggable_header<'a, Message: Clone + 'a>(
    header_content: Element<'a, Message>,
    on_drag: Message,
) -> Element<'a, Message> {
    iced::widget::mouse_area(
        container(header_content)
            .padding([12, 12])
            .width(Length::Fill)
            .style(|_| container::Style {
                background: None,
                ..Default::default()
            }),
    )
    .on_press(on_drag)
    .interaction(iced::mouse::Interaction::Grab)
    .into()
}

pub fn page_shell<'a, Message: 'a>(
    header: Element<'a, Message>,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let divider = container(Space::new())
        .width(Length::Fill)
        .height(1)
        .style(|theme: &Theme| container::Style {
            background: Some(iced::Background::Color(theme.divider())),
            ..Default::default()
        })
        .into();

    container(iced::widget::column(vec![header, divider, content]).spacing(0))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(iced::Background::Color(theme.page_background())),
            border: iced::Border {
                radius: WINDOW_RADIUS.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
