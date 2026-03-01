use canvas::{Frame, Geometry, Program, Text as CanvasText};
use iced::widget::{Space, button, canvas, column, container, row, scrollable, stack, text};
use iced::{Element, Length};
use iced::{Rectangle, Renderer, Theme};

use crate::{
    app::{Message, Page, State, message::SelectWorkflowMessage, state::SelectWorkflowDialog},
    features::{
        clipboard::view as clipboard_view, color_picker, json, scripts, workflow_editor,
        workflow_manager,
    },
    platform::screen::get_window_height,
    services::workflows::Workflow,
    ui::{
        constants::WINDOW_MARGIN,
        language,
        theme::{self, PastryTheme, ThemeMode},
        util::ui_radius,
        widgets,
    },
};

// Warmup program to initialize Canvas renderer and MONOSPACE font
#[derive(Default)]
struct WarmupCanvas;

impl Program<Message> for WarmupCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        // Draw a tiny invisible text to warm up MONOSPACE font and Canvas renderer
        let txt = CanvasText {
            content: " ".to_string(),
            position: iced::Point::new(0.0, 0.0),
            color: iced::Color::TRANSPARENT,
            size: 12.0.into(),
            ..Default::default()
        };
        frame.fill_text(txt);
        vec![frame.into_geometry()]
    }
}

fn build_backdrop(on_press: Message) -> Element<'static, Message> {
    button(
        container(iced::widget::Space::new())
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(on_press)
    .padding(0)
    .style(|_, _| button::Style {
        background: None,
        ..Default::default()
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn floating_dialog_layout(
    dialog_position: Option<iced::Point>,
) -> (iced::Alignment, iced::Alignment, iced::Padding) {
    let window_height = get_window_height(WINDOW_MARGIN);
    let menu_height = 350.0;

    if let Some(pos) = dialog_position {
        let flip_up = pos.y > window_height - menu_height;
        if flip_up {
            (
                iced::Alignment::Start,
                iced::Alignment::End,
                iced::Padding {
                    top: 0.0,
                    left: (pos.x - 190.0).max(0.0),
                    bottom: window_height - pos.y + 10.0,
                    right: 0.0,
                },
            )
        } else {
            (
                iced::Alignment::Start,
                iced::Alignment::Start,
                iced::Padding {
                    top: pos.y + 20.0,
                    left: (pos.x - 190.0).max(0.0),
                    bottom: 0.0,
                    right: 0.0,
                },
            )
        }
    } else {
        (
            iced::Alignment::Center,
            iced::Alignment::Start,
            iced::Padding::from([100.0, 0.0]),
        )
    }
}

fn build_floating_dialog_overlay<'a>(
    dialog: Element<'a, Message>,
    dialog_position: Option<iced::Point>,
) -> Element<'a, Message> {
    let (align_x, align_y, padding) = floating_dialog_layout(dialog_position);

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(align_x)
        .align_y(align_y)
        .padding(padding)
        .style(|_| container::Style {
            background: None,
            ..Default::default()
        })
        .into()
}

fn stack_with_overlay<'a>(
    base: Element<'a, Message>,
    close_message: Message,
    overlay: Element<'a, Message>,
) -> Element<'a, Message> {
    stack(vec![base, build_backdrop(close_message), overlay]).into()
}

fn view_select_workflow_dialog<'a>(
    dialog: &'a SelectWorkflowDialog,
    workflows: &'a [Workflow],
) -> Element<'a, SelectWorkflowMessage> {
    let search_input = widgets::search_input_dialog(
        language::tr(language::Text::SearchScripts),
        &dialog.search,
        SelectWorkflowMessage::SearchChanged,
    );

    let query = dialog.search.trim().to_lowercase();
    let mut filtered: Vec<_> = workflows
        .iter()
        .filter(|workflow| {
            if query.is_empty() {
                true
            } else {
                workflow.name.to_lowercase().contains(&query)
            }
        })
        .collect();
    filtered.sort_by(|a, b| a.name.cmp(&b.name));

    let workflow_list: Element<'_, SelectWorkflowMessage> = if filtered.is_empty() {
        widgets::empty_state(
            if workflows.is_empty() {
                "No workflows yet"
            } else {
                "No workflows match"
            },
            20.0,
            false,
        )
    } else {
        let items: Vec<Element<'_, SelectWorkflowMessage>> = filtered
            .iter()
            .map(|workflow| {
                let workflow_id = workflow.id.clone();

                let workflow_info = row![
                    text(&workflow.name)
                        .size(13)
                        .style(|theme: &iced::Theme| text::Style {
                            color: Some(theme.text())
                        }),
                    iced::widget::Space::new().width(Length::Fill),
                ]
                .spacing(12)
                .padding(8)
                .align_y(iced::Alignment::Center);

                button(workflow_info)
                    .on_press(SelectWorkflowMessage::SelectWorkflow(workflow_id))
                    .width(Length::Fill)
                    .padding(0)
                    .style(|theme: &iced::Theme, status| button::Style {
                        background: Some(iced::Background::Color(
                            if matches!(status, button::Status::Hovered) {
                                theme.button_hover_background()
                            } else {
                                theme.dialog_background()
                            },
                        )),
                        text_color: theme.text(),
                        border: iced::Border {
                            radius: ui_radius(6.0).into(),
                            ..Default::default()
                        },
                        shadow: iced::Shadow::default(),
                        ..Default::default()
                    })
                    .into()
            })
            .collect();

        scrollable(column(items).spacing(2))
            .height(Length::Fixed(300.0))
            .into()
    };

    let content = column![search_input, workflow_list]
        .spacing(12)
        .padding(12)
        .width(Length::Fixed(200.0));

    widgets::dialog_card(content).into()
}

pub fn view(state: &State) -> Element<'_, Message> {
    let base: Element<'_, Message> = match state.page.current {
        Page::Main => build_main_page(state),
        Page::Json => json::build_page(&state.json, theme::current()).map(Message::Json),
        Page::ScriptManager => {
            scripts::build_page(&state.scripts, theme::current()).map(Message::Scripts)
        }
        Page::WorkflowList => {
            workflow_manager::view::build_page(&state.workflow_list, theme::current())
                .map(Message::WorkflowList)
        }
        Page::WorkflowEditor => {
            workflow_editor::view::build_page(&state.workflow_editor).map(Message::WorkflowEditor)
        }
        Page::Settings => build_settings_page(state),
    };

    let content: Element<'_, Message> = if state.show_select_script_dialog {
        let dialog =
            scripts::view_select_script_dialog(&state.select_script_dialog, &state.scripts.scripts)
                .map(Message::SelectScriptDialog);
        let overlay = build_floating_dialog_overlay(dialog, state.dialog_position);
        stack_with_overlay(base, Message::CloseSelectScriptDialog, overlay)
    } else if state.show_select_workflow_dialog {
        let dialog = view_select_workflow_dialog(
            &state.select_workflow_dialog,
            &state.workflow_list.workflows,
        )
        .map(Message::SelectWorkflowDialog);
        let overlay = build_floating_dialog_overlay(dialog, state.dialog_position);
        stack_with_overlay(base, Message::CloseSelectWorkflowDialog, overlay)
    } else if state.show_language_menu {
        let menu = container(
            iced::widget::column(language::ALL.iter().map(|&lang| {
                let is_selected = language::current() == lang;
                let label = format!("{}", lang);
                button(
                    iced::widget::row![
                        text(label).size(13).style(|theme: &Theme| text::Style {
                            color: Some(theme.text())
                        }),
                        iced::widget::Space::new().width(Length::Fill),
                        if is_selected {
                            text("✓").size(13).style(|theme: &Theme| text::Style {
                                color: Some(theme.primary()),
                            })
                        } else {
                            text("").size(13)
                        }
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center),
                )
                .on_press(Message::LanguageSelected(lang))
                .width(Length::Fill)
                .padding([8, 12])
                .style(|theme, status| button::Style {
                    background: Some(iced::Background::Color(
                        if matches!(status, button::Status::Hovered) {
                            theme.button_hover_background()
                        } else {
                            iced::Color::TRANSPARENT
                        },
                    )),
                    text_color: theme.text(),
                    border: iced::Border {
                        radius: ui_radius(6.0).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
            }))
            .spacing(2),
        )
        .width(100)
        .padding(4)
        .style(|theme| container::Style {
            background: Some(iced::Background::Color(theme.card_background())),
            border: iced::Border {
                radius: ui_radius(8.0).into(),
                width: 1.0,
                color: theme.divider(),
            },
            shadow: iced::Shadow {
                color: theme.shadow(),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        });

        let menu_overlay = container(menu)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::Alignment::End)
            .align_y(iced::Alignment::Start)
            .padding([40, 60]);

        stack_with_overlay(base, Message::ToggleLanguageMenu, menu_overlay.into())
    } else {
        // Add hidden warmup canvas to pre-initialize renderer and fonts
        let warmup = container(
            canvas(WarmupCanvas)
                .width(Length::Fixed(1.0))
                .height(Length::Fixed(1.0)),
        )
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(1.0))
        .style(|_| container::Style {
            background: None,
            ..Default::default()
        });

        stack(vec![base, warmup.into()]).into()
    };

    color_picker::view::color_picker_overlay(
        state.show_color_picker,
        state.color_picker.color,
        content,
    )
}

fn build_main_page(state: &State) -> Element<'_, Message> {
    let pin_color = if state.pinned {
        |theme: &Theme| theme.primary()
    } else {
        |theme: &Theme| theme.text()
    };

    let header_row = row![
        text("Pastry").size(14).style(|theme: &Theme| text::Style {
            color: Some(theme.text())
        }),
        Space::new().width(Length::Fill),
        widgets::icon_button_fill(
            widgets::Icon::Code,
            16,
            4,
            4.0,
            Message::OpenScriptManagerPage,
            |theme| theme.text(),
        ),
        widgets::icon_button_fill(
            widgets::Icon::Flow,
            16,
            4,
            4.0,
            Message::OpenWorkflowListPage,
            |theme| theme.text(),
        ),
        widgets::icon_button_fill(
            widgets::Icon::Pin,
            16,
            4,
            4.0,
            Message::TogglePinned,
            pin_color,
        ),
    ]
    .spacing(8);

    let header = widgets::draggable_header(header_row.into(), Message::StartDrag);

    let clipboard =
        clipboard_view::view(&state.clipboard, theme::current()).map(Message::Clipboard);

    widgets::page_shell(header, clipboard)
}

fn settings_option(label: String, selected: bool, on_press: Message) -> Element<'static, Message> {
    button(
        iced::widget::row![
            text(label).size(13).style(|theme: &Theme| text::Style {
                color: Some(theme.text())
            }),
            iced::widget::Space::new().width(Length::Fill),
            if selected {
                text("✓").size(13).style(|theme: &Theme| text::Style {
                    color: Some(theme.primary()),
                })
            } else {
                text("").size(13)
            }
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center),
    )
    .on_press(on_press)
    .width(Length::Fill)
    .padding([8, 12])
    .style(move |theme, status| button::Style {
        background: Some(iced::Background::Color(if selected {
            let primary = theme.primary();
            iced::Color::from_rgba(
                primary.r,
                primary.g,
                primary.b,
                if theme::is_dark(theme) { 0.2 } else { 0.12 },
            )
        } else if matches!(status, button::Status::Hovered) {
            theme.button_hover_background()
        } else {
            iced::Color::TRANSPARENT
        })),
        text_color: theme.text(),
        border: iced::Border {
            radius: ui_radius(6.0).into(),
            width: if selected { 1.0 } else { 0.0 },
            color: if selected {
                let primary = theme.primary();
                iced::Color::from_rgba(primary.r, primary.g, primary.b, 0.5)
            } else {
                iced::Color::TRANSPARENT
            },
        },
        ..Default::default()
    })
    .into()
}

fn settings_section<'a>(title: String, content: Element<'a, Message>) -> Element<'a, Message> {
    container(
        column(vec![
            text(title)
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.text_placeholder()),
                })
                .into(),
            content,
        ])
        .spacing(8),
    )
    .padding(12)
    .width(Length::Fill)
    .style(|theme| container::Style {
        background: Some(iced::Background::Color(theme.card_background())),
        border: iced::Border {
            radius: ui_radius(10.0).into(),
            width: 1.0,
            color: theme.divider(),
        },
        ..Default::default()
    })
    .into()
}

fn web_sync_content(state: &State) -> Element<'_, Message> {
    let access_url = state
        .web_access_url
        .as_deref()
        .unwrap_or(language::tr(language::Text::NoLanAddressAvailable));

    if let Some(svg_data) = &state.web_qr_svg {
        let status_badge = container(
            row![
                text("●").size(10).style(|theme: &Theme| text::Style {
                    color: Some(theme.success()),
                }),
                text(language::tr(language::Text::WebServiceReady))
                    .size(11)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.text_secondary()),
                    })
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        )
        .padding([4, 10])
        .style(|theme: &Theme| {
            let success = theme.success();
            container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    success.r,
                    success.g,
                    success.b,
                    if theme::is_dark(theme) { 0.2 } else { 0.12 },
                ))),
                border: iced::Border {
                    radius: ui_radius(999.0).into(),
                    width: 1.0,
                    color: iced::Color::from_rgba(success.r, success.g, success.b, 0.45),
                },
                ..Default::default()
            }
        });

        let qr_code = iced::widget::svg(iced::widget::svg::Handle::from_memory(
            svg_data.as_bytes().to_vec(),
        ))
        .width(220)
        .height(220);

        let qr_card = container(
            column(vec![
                container(status_badge).center_x(Length::Fill).into(),
                container(
                    text(language::tr(language::Text::ScanQrToAccess))
                        .size(12)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.text_secondary()),
                        }),
                )
                .center_x(Length::Fill)
                .into(),
                container(qr_code).center_x(Length::Fill).into(),
                container(
                    text(access_url)
                        .size(13)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.text()),
                        }),
                )
                .width(Length::Fill)
                .padding([8, 12])
                .style(|theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(theme.input_background())),
                    border: iced::Border {
                        radius: ui_radius(8.0).into(),
                        width: 1.0,
                        color: theme.input_border(),
                    },
                    ..Default::default()
                })
                .into(),
                container(
                    text(language::tr(language::Text::EnsureSameLan))
                        .size(11)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.text_placeholder()),
                        }),
                )
                .center_x(Length::Fill)
                .into(),
            ])
            .spacing(10),
        )
        .padding(12)
        .width(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(iced::Background::Color(theme.button_background())),
            border: iced::Border {
                radius: ui_radius(12.0).into(),
                width: 1.0,
                color: theme.divider(),
            },
            ..Default::default()
        });

        return qr_card.into();
    }

    container(
        column(vec![
            text(language::tr(language::Text::NoLanAddressAvailable))
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.text_secondary()),
                })
                .into(),
            text(language::tr(language::Text::CheckWifiVpnRetry))
                .size(11)
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.text_placeholder()),
                })
                .into(),
        ])
        .spacing(4),
    )
    .padding(10)
    .width(Length::Fill)
    .style(|theme: &Theme| container::Style {
        background: Some(iced::Background::Color(theme.button_background())),
        border: iced::Border {
            radius: ui_radius(10.0).into(),
            width: 1.0,
            color: theme.divider(),
        },
        ..Default::default()
    })
    .into()
}

fn build_settings_page(state: &State) -> Element<'_, Message> {
    let back_button = widgets::icon_button_hover(
        widgets::Icon::Back,
        16,
        [4, 8],
        6.0,
        Message::CloseSettingsPage,
        |theme| theme.text(),
    );

    let header_row = row![
        back_button,
        text(language::tr(language::Text::Settings)).size(14),
        Space::new().width(Length::Fill)
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let header = widgets::draggable_header(header_row.into(), Message::StartDrag);

    let language_options: Vec<Element<Message>> = language::ALL
        .iter()
        .map(|&lang| {
            let label = format!("{}", lang);
            settings_option(
                label,
                language::current() == lang,
                Message::LanguageSelected(lang),
            )
        })
        .collect();
    let language_section = settings_section(
        language::tr(language::Text::Language).to_string(),
        column(language_options).spacing(2).into(),
    );

    let theme_section = settings_section(
        language::tr(language::Text::Theme).to_string(),
        column(vec![
            settings_option(
                language::tr(language::Text::Light).to_string(),
                theme::current() == ThemeMode::Light,
                Message::SetTheme(ThemeMode::Light),
            ),
            settings_option(
                language::tr(language::Text::Dark).to_string(),
                theme::current() == ThemeMode::Dark,
                Message::SetTheme(ThemeMode::Dark),
            ),
        ])
        .spacing(2)
        .into(),
    );

    let startup_section = settings_section(
        language::tr(language::Text::Startup).to_string(),
        settings_option(
            language::tr(language::Text::StartHidden).to_string(),
            state.start_hidden,
            Message::ToggleStartHidden,
        ),
    );

    let mut web_section_items = vec![settings_option(
        language::tr(language::Text::EnableLanSync).to_string(),
        state.web_server_enabled,
        Message::ToggleWebServer,
    )];
    if state.web_server_enabled {
        web_section_items.push(web_sync_content(state));
    }
    let web_server_section = settings_section(
        language::tr(language::Text::LanSync).to_string(),
        column(web_section_items).spacing(8).into(),
    );

    let content = container(
        column(vec![
            language_section,
            theme_section,
            startup_section,
            web_server_section,
        ])
        .spacing(12),
    )
    .padding(20)
    .width(Length::Fill)
    .height(Length::Fill);

    widgets::page_shell(header, content.into())
}
