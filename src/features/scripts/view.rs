use iced::widget::{
    Space, button, column, container, row, scrollable, text, text_editor, text_input,
};
use iced::{Element, Length};

use super::{
    message::{ManagerMessage, Message, SelectScriptMessage},
    state::{ScriptManager, SelectScriptDialog, State},
};
use crate::{
    services::scripts,
    ui::widgets::confirm_dialog,
    ui::{
        language::{self, Text},
        theme::{self, PastryTheme},
        util::ui_radius,
        widgets,
    },
};

pub fn view(state: &State) -> Element<'_, Message> {
    {
        let element: Element<'_, ManagerMessage> =
            if state.manager.is_editing() && state.manager.editing_script.is_some() {
                view_edit_form(&state.manager)
            } else {
                view_list(&state.manager, &state.scripts)
            };
        element.map(Message::ExternalManager)
    }
}

pub fn view_select_script_dialog<'a>(
    dialog: &'a SelectScriptDialog,
    scripts: &'a [scripts::Script],
) -> Element<'a, SelectScriptMessage> {
    let search_input = widgets::search_input_dialog(
        language::tr(Text::SearchScripts),
        &dialog.search,
        SelectScriptMessage::SearchChanged,
    );

    let query = dialog.search.trim().to_lowercase();
    let filtered: Vec<&scripts::Script> = if query.is_empty() {
        scripts.iter().collect()
    } else {
        scripts
            .iter()
            .filter(|script| {
                let display_name = scripts::localized_display_name(script).to_lowercase();
                display_name.contains(&query) || script.name.to_lowercase().contains(&query)
            })
            .collect()
    };

    let script_list: Element<'_, SelectScriptMessage> = if filtered.is_empty() {
        widgets::empty_state(
            if scripts.is_empty() {
                language::tr(Text::NoScriptsYetAdd)
            } else {
                language::tr(Text::NoScriptsMatch)
            },
            20.0,
            false,
        )
    } else {
        let items: Vec<Element<'_, SelectScriptMessage>> = filtered
            .iter()
            .map(|script| {
                let run_id = script.id.clone();

                let script_info = row![
                    text(scripts::localized_display_name(script))
                        .size(13)
                        .style(|theme: &iced::Theme| text::Style {
                            color: Some(theme.text())
                        }),
                    iced::widget::Space::new().width(Length::Fill),
                ]
                .spacing(12)
                .padding(8)
                .align_y(iced::Alignment::Center);

                button(script_info)
                    .on_press(SelectScriptMessage::SelectScript(run_id))
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

    let content = column![search_input, script_list]
        .spacing(12)
        .padding(12)
        .width(Length::Fixed(200.0));

    widgets::dialog_card(content).into()
}

fn view_list<'a>(
    manager: &'a ScriptManager,
    scripts: &'a [scripts::Script],
) -> Element<'a, ManagerMessage> {
    let search_bar = row![
        container(widgets::search_input_card(
            language::tr(Text::SearchScripts),
            &manager.search,
            ManagerMessage::SearchChanged,
        ))
        .width(Length::Fill),
        widgets::icon_button_fill(
            widgets::Icon::Add,
            16,
            8,
            ui_radius(6.0),
            ManagerMessage::StartNew,
            |theme| theme.text(),
        ),
    ]
    .spacing(10);

    let query = manager.search.trim().to_lowercase();
    let filtered: Vec<&scripts::Script> = if query.is_empty() {
        scripts.iter().collect()
    } else {
        scripts
            .iter()
            .filter(|script| {
                let display_name = scripts::localized_display_name(script).to_lowercase();
                display_name.contains(&query) || script.name.to_lowercase().contains(&query)
            })
            .collect()
    };

    let script_list: Element<'_, ManagerMessage> = if filtered.is_empty() {
        widgets::empty_state(
            if scripts.is_empty() {
                language::tr(Text::NoScriptsYetNew)
            } else {
                language::tr(Text::NoScriptsMatch)
            },
            40.0,
            true,
        )
    } else {
        let items: Vec<Element<'_, ManagerMessage>> = filtered
            .iter()
            .map(|script| view_script_item(script))
            .collect();

        scrollable(column(items).spacing(10))
            .height(Length::Fill)
            .into()
    };

    let content = column![search_bar, script_list]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill);

    // Show delete confirmation dialog if needed
    if let Some(_confirm_id) = &manager.delete_confirm_id {
        return confirm_dialog(
            language::tr(language::Text::ConfirmDeleteTitle),
            language::tr(language::Text::ConfirmDeleteMessage),
            language::tr(language::Text::Cancel),
            language::tr(language::Text::Delete),
            ManagerMessage::CancelDelete,
            ManagerMessage::ConfirmDelete,
        );
    }

    content.into()
}

fn view_script_item(script: &scripts::Script) -> Element<'static, ManagerMessage> {
    let edit_id = script.id.clone();
    let delete_id = script.id.clone();

    let info = container(
        column![
            text(scripts::localized_display_name(script))
                .size(14)
                .style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.text())
                })
        ]
        .spacing(0),
    )
    .padding([2, 0]);

    let buttons = row![
        widgets::icon_button_fill(
            widgets::Icon::Editor,
            16,
            6,
            ui_radius(6.0),
            ManagerMessage::StartEdit(edit_id),
            |theme| theme.text(),
        ),
        widgets::icon_button_fill(
            widgets::Icon::Delete,
            16,
            6,
            ui_radius(6.0),
            ManagerMessage::ShowDeleteConfirm(delete_id),
            |theme| theme.text(),
        ),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    container(
        row![
            info,
            iced::widget::Space::new().width(Length::Fill),
            buttons
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center)
        .padding([10, 14]),
    )
    .width(Length::Fill)
    .style(|theme| container::Style {
        background: Some(iced::Background::Color(theme.card_background())),
        border: iced::Border {
            radius: ui_radius(8.0).into(),
            width: 1.0,
            color: theme.divider(),
        },
        ..Default::default()
    })
    .into()
}

fn view_edit_form<'a>(manager: &'a ScriptManager) -> Element<'a, ManagerMessage> {
    let editing = if let Some(e) = manager.editing_script.as_ref() {
        e
    } else {
        return view_list(manager, &[]);
    };
    let is_new = editing.id.is_none();

    let header = row![
        text(if is_new {
            language::tr(Text::NewScript)
        } else {
            language::tr(Text::EditScript)
        })
        .size(14)
        .style(|theme: &iced::Theme| text::Style {
            color: Some(theme.text())
        }),
        iced::widget::Space::new().width(Length::Fill),
        button(widgets::icon_svg(widgets::Icon::Close, 16, |theme| {
            theme.text()
        }))
        .on_press(ManagerMessage::Cancel)
        .padding(4)
        .style(|theme: &iced::Theme, _| button::Style {
            background: None,
            text_color: theme.text(),
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            snap: false,
        }),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let is_builtin = editing
        .id
        .as_ref()
        .and_then(|id| language::script_text(id))
        .is_some();

    let name_field = if is_builtin {
        let display_name = if let Some(id) = &editing.id {
            if let Some(text_key) = language::script_text(id) {
                language::tr(text_key).to_string()
            } else {
                editing.name.clone()
            }
        } else {
            editing.name.clone()
        };

        column![
            text(language::tr(Text::ScriptName))
                .size(12)
                .style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.text_secondary())
                }),
            container(
                text(display_name)
                    .size(13)
                    .style(|theme: &iced::Theme| text::Style {
                        color: Some(theme.text())
                    })
            )
            .padding(10)
            .width(Length::Fill)
            .style(|theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(theme.input_background())),
                border: iced::Border {
                    radius: ui_radius(8.0).into(),
                    width: 1.0,
                    color: theme.divider(),
                },
                ..Default::default()
            })
        ]
        .spacing(6)
    } else {
        column![
            text(language::tr(Text::ScriptName))
                .size(12)
                .style(|theme: &iced::Theme| text::Style {
                    color: Some(theme.text_secondary())
                }),
            text_input("", &editing.name)
                .on_input(ManagerMessage::NameChanged)
                .padding(10)
                .size(13)
                .style(|theme: &iced::Theme, status| {
                    use iced::widget::text_input::Status;
                    let is_focused = matches!(status, Status::Focused { .. });
                    iced::widget::text_input::Style {
                        background: iced::Background::Color(theme.input_background()),
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
                }),
        ]
        .spacing(6)
    };

    let code_field = column![
        text(language::tr(Text::ScriptCode))
            .size(12)
            .style(|theme: &iced::Theme| text::Style {
                color: Some(theme.text_secondary())
            }),
        text_editor(&editing.code_editor)
            .on_action(ManagerMessage::CodeChanged)
            .height(Length::Fill)
            .padding(10)
            .highlight_with::<iced::highlighter::Highlighter>(
                iced::highlighter::Settings {
                    theme: iced::highlighter::Theme::Base16Eighties,
                    token: "js".to_string(),
                },
                |highlight: &iced::highlighter::Highlight, _theme: &iced::Theme| highlight
                    .to_format()
            )
            .style(|theme: &iced::Theme, status| {
                use iced::widget::text_editor::Status;
                let is_focused = matches!(status, Status::Focused { .. });
                iced::widget::text_editor::Style {
                    background: iced::Background::Color(theme.code_background()),
                    border: iced::Border {
                        radius: ui_radius(8.0).into(),
                        width: 1.0,
                        color: if is_focused {
                            theme.primary()
                        } else {
                            theme.divider()
                        },
                    },
                    placeholder: theme.text_secondary(),
                    value: iced::Color::WHITE,
                    selection: theme.primary(),
                }
            }),
    ]
    .spacing(6)
    .height(Length::Fill);

    let can_save = !editing.name.trim().is_empty() && !editing.code_editor.text().trim().is_empty();

    let save_button = button(
        container(text(language::tr(Text::Save)).size(13))
            .padding([8, 20])
            .center_x(Length::Fill),
    )
    .width(Length::Fill);

    let save_button = if can_save {
        save_button.on_press(ManagerMessage::Save)
    } else {
        save_button
    };

    let save_button = save_button.style(move |theme: &iced::Theme, status| button::Style {
        background: Some(iced::Background::Color(if !can_save {
            theme.button_background()
        } else if matches!(status, button::Status::Hovered) {
            theme.button_hover_background()
        } else {
            theme.button_background()
        })),
        text_color: if can_save {
            iced::Color::WHITE
        } else {
            theme.text_secondary()
        },
        border: iced::Border {
            radius: ui_radius(6.0).into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let content = column![header, name_field, code_field, save_button]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill);

    content.into()
}

/// Build the complete script manager page with header, divider, and content
pub fn build_page(state: &State, _theme_mode: theme::ThemeMode) -> Element<'_, Message> {
    let back_button = widgets::icon_button_hover(
        widgets::Icon::Back,
        16,
        [4, 8],
        ui_radius(6.0),
        Message::ClosePage,
        |theme| theme.text(),
    );

    let header_row = row![
        back_button,
        text(language::tr(Text::ScriptManagerTitle)).size(14),
        Space::new().width(Length::Fill)
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let header = widgets::draggable_header(header_row.into(), Message::StartDrag);

    let manager_view = view(state);
    let content = container(manager_view)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

    widgets::page_shell(header, content.into())
}
