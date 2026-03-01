use iced::widget::{Space, button, column, container, lazy, row, scrollable, text, text_input};
use iced::{Element, Length};
use iced_selection::text as selectable_text;

use super::{
    message::Message,
    model::{CardMessage, CardState, ClipType, ImageFormat},
    state::{Filter, State},
};
use crate::ui::{
    constants::{BUTTON_RADIUS, CARD_RADIUS, INPUT_RADIUS},
    language::{self, Text},
    theme::{PastryTheme, ThemeMode},
    util::ui_radius,
    widgets,
    widgets::confirm_dialog,
};

fn view_card(state: &CardState) -> Element<'static, CardMessage> {
    let is_favorite = state.is_favorite;
    let favorite_icon = if state.is_favorite {
        widgets::Icon::StarOne
    } else {
        widgets::Icon::Star
    };
    let favorite_color = move |theme: &iced::Theme| {
        if is_favorite {
            theme.primary()
        } else {
            theme.text()
        }
    };
    let is_copied = state.is_copied;
    let copy_icon = if is_copied {
        widgets::Icon::Copied
    } else {
        widgets::Icon::Copy
    };
    let copy_color = move |theme: &iced::Theme| {
        if is_copied {
            theme.primary()
        } else {
            theme.text()
        }
    };

    let time_ago = state.time_ago();
    let is_image = state.clip_type == ClipType::Image;
    let is_json = !is_image && state.is_json;

    let mut header_items: Vec<Element<CardMessage>> = vec![
        text(time_ago)
            .size(10)
            .style(|theme: &iced::Theme| text::Style {
                color: Some(theme.text_secondary()),
            })
            .into(),
        Space::new().width(Length::Fill).into(),
    ];

    header_items.push(
        widgets::icon_button_hover(
            widgets::Icon::Delete,
            16,
            4,
            ui_radius(4.0),
            CardMessage::ShowDeleteConfirm,
            |theme| theme.text(),
        )
        .into(),
    );

    header_items.push(
        widgets::icon_button_hover(
            copy_icon,
            16,
            4,
            ui_radius(4.0),
            CardMessage::Copy,
            copy_color,
        )
        .into(),
    );

    if is_json {
        header_items.push(
            widgets::icon_button_hover(
                widgets::Icon::CodeBrackets,
                16,
                4,
                ui_radius(4.0),
                CardMessage::ShowJsonFormat,
                |theme| theme.text(),
            )
            .into(),
        );
    }

    if is_image {
        header_items.push(
            widgets::icon_button_hover(
                widgets::Icon::Compression,
                16,
                4,
                ui_radius(4.0),
                CardMessage::CompressImage,
                |theme| theme.text(),
            )
            .into(),
        );
    } else {
        header_items.push(
            widgets::icon_button_hover(
                widgets::Icon::Code,
                16,
                4,
                ui_radius(4.0),
                CardMessage::RunScript,
                |theme| theme.text(),
            )
            .into(),
        );
        header_items.push(
            widgets::icon_button_hover(
                widgets::Icon::Flow,
                16,
                4,
                ui_radius(4.0),
                CardMessage::RunWorkflow,
                |theme| theme.text(),
            )
            .into(),
        );
    }

    header_items.push(
        widgets::icon_button_hover(
            favorite_icon,
            16,
            4,
            ui_radius(4.0),
            CardMessage::ToggleFavorite,
            favorite_color,
        )
        .into(),
    );

    let header = row(header_items)
        .spacing(8)
        .align_y(iced::Alignment::Center);

    let content_area: Element<'_, CardMessage> = if state.clip_type == ClipType::Image {
        if let Some(handle) = &state.image_handle {
            let is_svg = state
                .image_format
                .as_ref()
                .map(|f| matches!(f, ImageFormat::Svg))
                .unwrap_or(false);

            if is_svg {
                container(
                    container(
                        iced::widget::image(handle.clone())
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .content_fit(iced::ContentFit::Contain),
                    )
                    .padding(8)
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(iced::Color::WHITE)),
                        border: iced::Border {
                            radius: ui_radius(4.0).into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                )
                .width(Length::Fill)
                .height(Length::Fixed(80.0))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            } else {
                iced::widget::image(handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fixed(80.0))
                    .content_fit(iced::ContentFit::ScaleDown)
                    .into()
            }
        } else {
            text(language::tr(Text::ImageLoadFailed))
                .size(12)
                .style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.danger()),
                })
                .into()
        }
    } else {
        let preview_len = 300;
        let preview: String = state.content.chars().take(preview_len).collect();
        let preview = if state.content.chars().count() > preview_len {
            format!("{}...", preview)
        } else {
            preview
        };

        // Use iced_selection::text for selectable text
        let content_text = selectable_text(preview)
            .size(14)
            .style(move |theme: &iced::Theme| iced_selection::text::Style {
                color: Some(theme.text()),
                selection: theme.primary(),
            })
            .width(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::Glyph);

        if let Some(color) = state.is_color {
            let color_preview = button(Space::new())
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0))
                .style(move |_, _| button::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        color.r, color.g, color.b, color.a,
                    ))),
                    border: iced::Border {
                        radius: ui_radius(8.0).into(),
                        width: 1.0,
                        color: iced::Color::BLACK, // Or theme border?
                    },
                    ..Default::default()
                })
                .on_press(CardMessage::ToggleColorPicker(color));

            row![color_preview, content_text]
                .spacing(10)
                .align_y(iced::Alignment::Center)
                .into()
        } else {
            content_text.into()
        }
    };

    let mut card_items: Vec<Element<'_, CardMessage>> = vec![header.into(), content_area];

    if let Some(output) = &state.script_output {
        let output = output
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string();

        let is_error = output.starts_with("❌")
            || output.contains("失败")
            || output.contains("错误")
            || output.contains("Error")
            || output.contains("error")
            || output.contains("Exception")
            || output.contains("Traceback");

        let base = language::tr(Text::ScriptExecResult);
        let label_text = if let Some(script_name) = &state.script_name {
            format!("{} ({}): ", base, script_name)
        } else {
            base.to_string()
        };

        let output_label = text(label_text)
            .size(10)
            .style(|theme: &iced::Theme| text::Style {
                color: Some(theme.text()),
            });

        let output_text = text(output)
            .size(10)
            .width(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::Glyph)
            .style(move |theme: &iced::Theme| text::Style {
                color: Some(if is_error {
                    theme.danger()
                } else {
                    theme.success()
                }),
            });

        let output_row: Element<'_, CardMessage> = if state.script_output_copied {
            row![
                output_text,
                text(language::tr(Text::Copied))
                    .size(10)
                    .style(|theme: &iced::Theme| text::Style {
                        color: Some(theme.primary()),
                    })
            ]
            .spacing(8)
            .padding(8)
            .align_y(iced::Alignment::Center)
            .into()
        } else {
            row![output_text]
                .padding(8)
                .align_y(iced::Alignment::Center)
                .into()
        };

        let output_area: Element<'_, CardMessage> = if !is_error {
            button(output_row)
                .on_press(CardMessage::CopyScriptOutput)
                .padding(0)
                .width(Length::Fill)
                .style(|theme, status| button::Style {
                    background: Some(iced::Background::Color(
                        if matches!(status, button::Status::Hovered) {
                            theme.card_code_background_hover()
                        } else {
                            theme.card_code_background()
                        },
                    )),
                    text_color: theme.text(),
                    border: iced::Border {
                        radius: ui_radius(4.0).into(),
                        ..Default::default()
                    },
                    shadow: iced::Shadow::default(),
                    snap: false,
                })
                .into()
        } else {
            container(output_row)
                .width(Length::Fill)
                .style(|theme| container::Style {
                    background: Some(iced::Background::Color(theme.card_code_background_hover())),
                    border: iced::Border {
                        radius: ui_radius(4.0).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        };

        let divider = container(container(Space::new().height(1)).width(Length::Fill).style(
            |theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(theme.divider())),
                ..Default::default()
            },
        ))
        .padding([8, 0]);

        card_items.push(divider.into());
        let header_row = row![
            output_label,
            Space::new().width(Length::Fill),
            widgets::icon_button_hover(
                widgets::Icon::Delete,
                12,
                4,
                ui_radius(4.0),
                CardMessage::DeleteScriptOutput,
                |theme| theme.text_secondary(),
            )
        ]
        .align_y(iced::Alignment::Center);

        card_items.push(header_row.into());
        card_items.push(output_area);
    }

    let card_content = iced::widget::column(card_items).spacing(6).padding(10);

    let card = container(card_content)
        .width(Length::Fill)
        .clip(true)
        .style(|theme| container::Style {
            background: Some(iced::Background::Color(theme.card_background())),
            border: iced::Border {
                radius: CARD_RADIUS.into(),
                width: 1.0,
                color: theme.input_border(),
            },
            ..Default::default()
        });

    card.into()
}

pub fn view(state: &State, theme_mode: ThemeMode) -> Element<'_, Message> {
    let search_bar: Element<'_, Message> =
        text_input(language::tr(Text::Search), &state.search_text)
            .on_input(Message::SearchChanged)
            .padding(10)
            .size(14)
            .width(Length::Fill)
            .style(|theme: &iced::Theme, _| text_input::Style {
                background: iced::Background::Color(theme.input_background()),
                border: iced::Border {
                    radius: INPUT_RADIUS.into(),
                    width: 1.0,
                    color: theme.input_border(),
                },
                icon: theme.text_secondary(),
                placeholder: theme.text_placeholder(),
                value: theme.text(),
                selection: theme.primary(),
            })
            .into();

    let filter_button =
        |label: &str, filter: Filter, current: Filter| -> Element<'static, Message> {
            let is_active = filter == current;
            let bg_color = move |theme: &iced::Theme| {
                if is_active {
                    theme.button_background()
                } else {
                    iced::Color::TRANSPARENT
                }
            };

            button(text(label.to_owned()).size(12))
                .on_press(Message::FilterChanged(filter))
                .padding([4, 12])
                .style(move |theme: &iced::Theme, _| button::Style {
                    background: Some(iced::Background::Color(bg_color(theme))),
                    text_color: theme.text(),
                    border: iced::Border {
                        radius: BUTTON_RADIUS.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        };

    let filter_bar: Element<'static, Message> = row![
        filter_button(language::tr(Text::Recent), Filter::Recent, state.filter),
        filter_button(language::tr(Text::Favorite), Filter::Favorite, state.filter)
    ]
    .spacing(4)
    .into();

    let search_area: Element<'_, Message> = column![search_bar, filter_bar]
        .spacing(8)
        .padding(12)
        .into();

    let search = state.search_text.trim().to_lowercase();
    let mut filtered: Vec<(usize, &CardState)> = state
        .history
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            let search_match = search.is_empty()
                || entry.content.to_lowercase().contains(&search)
                || entry
                    .file_path
                    .as_ref()
                    .map(|p| p.to_lowercase().contains(&search))
                    .unwrap_or(false);

            let type_match = match state.filter {
                Filter::Recent => entry.timestamp >= state.startup_time,
                Filter::Favorite => entry.is_favorite,
            };

            search_match && type_match
        })
        .collect();

    let history_list: Element<'_, Message> = if filtered.is_empty() {
        let empty_message = if state.filter == Filter::Favorite {
            language::tr(Text::NoFavoritesYet)
        } else if state.history.is_empty() {
            language::tr(Text::HistoryEmptyHint)
        } else {
            language::tr(Text::NoRecordsMatch)
        };

        container(
            text(empty_message)
                .size(14)
                .style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.text_secondary()),
                }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        let cards: Vec<Element<'_, Message>> = filtered
            .drain(..)
            .map(|(idx, entry)| {
                Element::from(lazy(
                    (theme_mode, language::current(), entry.clone()),
                    |(_, _, entry)| view_card(entry),
                ))
                .map(move |msg| Message::ExternalCard(idx, msg))
            })
            .collect();

        scrollable(column(cards).spacing(10).padding(12).width(Length::Fill))
            .height(Length::Fill)
            .into()
    };

    let mut content_items: Vec<Element<Message>> = vec![search_area];
    if let Some(message) = &state.compress_message {
        let toast_bg = |theme: &iced::Theme| {
            if message.starts_with("✅") {
                theme.success()
            } else if message.starts_with("❌") {
                theme.danger()
            } else {
                theme.primary()
            }
        };
        let toast = container(
            text(message)
                .size(13)
                .style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.text()),
                }),
        )
        .padding([8, 16])
        .style(move |theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(toast_bg(theme))),
            border: iced::Border {
                radius: ui_radius(8.0).into(),
                ..Default::default()
            },
            shadow: iced::Shadow {
                color: theme.shadow(),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            text_color: Some(theme.text()),
            snap: false,
        });
        content_items.push(
            container(toast)
                .width(iced::Length::Fill)
                .padding([0, 12])
                .into(),
        );
    }
    content_items.push(history_list);
    let main_content = column(content_items);

    // Show delete confirmation dialog if needed
    if state.delete_confirm_index.is_some() {
        return confirm_dialog(
            language::tr(language::Text::ConfirmDeleteTitle),
            language::tr(language::Text::ConfirmDeleteMessage),
            language::tr(language::Text::Cancel),
            language::tr(language::Text::Delete),
            Message::CancelDelete,
            Message::ConfirmDelete,
        );
    }

    // Show unfavorite confirmation dialog if needed
    if state.unfavorite_confirm_index.is_some() {
        return confirm_dialog(
            language::tr(language::Text::ConfirmUnfavoriteTitle),
            language::tr(language::Text::ConfirmUnfavoriteMessage),
            language::tr(language::Text::Cancel),
            language::tr(language::Text::ConfirmUnfavoriteTitle),
            Message::CancelUnfavorite,
            Message::ConfirmUnfavorite,
        );
    }

    main_content.into()
}
