use iced::widget::{Space, button, column, container, row, scrollable, stack, text};
use iced::{Element, Length};

use super::{
    message::{ManagerMessage, Message},
    state::WorkflowListState,
};
use crate::services::workflows::Workflow;
use crate::ui::{
    language::{self, Text},
    theme::{self, PastryTheme},
    util::ui_radius,
    widgets::{self, confirm_dialog},
};

pub fn view(state: &WorkflowListState) -> Element<'_, Message> {
    view_list(&state.manager.search, &state.workflows).map(Message::List)
}

fn view_list<'a>(search: &'a str, workflows: &'a [Workflow]) -> Element<'a, ManagerMessage> {
    let search_bar = row![
        container(widgets::search_input_card(
            language::tr(Text::SearchWorkflows),
            search,
            ManagerMessage::SearchChanged,
        ))
        .width(Length::Fill),
        widgets::icon_button_fill(
            widgets::Icon::Add,
            16,
            8,
            ui_radius(8.0),
            ManagerMessage::CreateWorkflow,
            |theme| theme.primary(),
        ),
    ]
    .spacing(10);

    let query = search.trim().to_lowercase();
    let filtered: Vec<&Workflow> = if query.is_empty() {
        workflows.iter().collect()
    } else {
        workflows
            .iter()
            .filter(|workflow| workflow.name.to_lowercase().contains(&query))
            .collect()
    };

    let workflow_list: Element<'_, ManagerMessage> = if filtered.is_empty() {
        widgets::empty_state(
            if workflows.is_empty() {
                language::tr(Text::NoWorkflowsYetCreateFirst)
            } else {
                language::tr(Text::NoWorkflowsMatchSearch)
            },
            40.0,
            true,
        )
    } else {
        let items: Vec<Element<'_, ManagerMessage>> = filtered
            .iter()
            .map(|workflow| view_workflow_item(workflow))
            .collect();

        scrollable(column(items).spacing(10))
            .height(Length::Fill)
            .into()
    };

    let content = column![search_bar, workflow_list]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill);

    content.into()
}

fn view_workflow_item<'a>(workflow: &'a Workflow) -> Element<'a, ManagerMessage> {
    let edit_id = workflow.id.clone();
    let delete_id = workflow.id.clone();
    let toggle_id = workflow.id.clone();
    let is_enabled = workflow.enabled;

    let info = column![
        text(&workflow.name)
            .size(14)
            .style(|theme: &iced::Theme| text::Style {
                color: Some(theme.text())
            }),
    ]
    .spacing(4);

    let buttons = row![
        button(
            text(if workflow.enabled {
                language::tr(Text::Enabled)
            } else {
                language::tr(Text::Disabled)
            })
            .size(13)
        )
        .on_press(ManagerMessage::ToggleEnabled(toggle_id))
        .padding([4, 8])
        .style(move |theme: &iced::Theme, status| {
            let is_hovered = matches!(status, iced::widget::button::Status::Hovered);
            iced::widget::button::Style {
                background: if is_hovered {
                    Some(iced::Background::Color(theme.button_hover_background()))
                } else {
                    None
                },
                text_color: if is_enabled {
                    theme.primary()
                } else {
                    theme.text_secondary()
                },
                border: iced::Border {
                    radius: ui_radius(6.0).into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }),
        widgets::icon_button_fill(
            widgets::Icon::Editor,
            16,
            6,
            ui_radius(6.0),
            ManagerMessage::OpenEditor(edit_id),
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
        .padding(14),
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

pub fn build_page(
    state: &WorkflowListState,
    _theme_mode: theme::ThemeMode,
) -> Element<'_, Message> {
    let back_button = widgets::icon_button_hover(
        widgets::Icon::Back,
        16,
        [4, 8],
        ui_radius(6.0),
        Message::List(ManagerMessage::ClosePage),
        |theme| theme.text(),
    );

    let header_row = row![
        back_button,
        text(language::tr(Text::Workflows)).size(14),
        Space::new().width(Length::Fill)
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    let header =
        widgets::draggable_header(header_row.into(), Message::List(ManagerMessage::StartDrag));

    let manager_view = view(state);
    let content = container(manager_view)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut main_content: Element<'_, Message> = widgets::page_shell(header, content.into());

    if state.manager.delete_confirm_id.is_some() {
        main_content = stack![
            main_content,
            confirm_dialog(
                language::tr(Text::DeleteWorkflow),
                language::tr(Text::DeleteWorkflowConfirm),
                language::tr(Text::Cancel),
                language::tr(Text::Delete),
                Message::List(ManagerMessage::CancelDelete),
                Message::List(ManagerMessage::ConfirmDelete),
            )
        ]
        .into();
    }

    main_content
}
