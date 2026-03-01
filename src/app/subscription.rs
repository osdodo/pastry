use std::time::Duration;

use iced::{Subscription, event, futures::SinkExt, mouse, window as iced_window};
use uuid::Uuid;

use crate::{
    app::{Message, Page::WorkflowEditor, State},
    features::{clipboard, json, workflow_editor},
    platform::{hotkey, tray},
    ui::constants::{CLIPBOARD_CHECK_MS, HOTKEY_DEBOUNCE_MS},
    web::{
        DEFAULT_WEB_SERVER_PORT, WebState, decode_image_data_url, init_web_state, start_web_server,
    },
};

pub fn subscription(state: &State) -> Subscription<Message> {
    let clipboard_sub = iced::time::every(Duration::from_millis(CLIPBOARD_CHECK_MS))
        .map(|_| Message::Clipboard(clipboard::message::Message::Poll));
    let mouse_sub = event::listen_with(|evt, _status, _id| match evt {
        iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
            Some(Message::MouseMoved(position))
        }
        _ => None,
    });
    let window_sub = event::listen_with(|evt, _status, id| match evt {
        iced::Event::Window(iced_window::Event::Opened { .. }) => Some(Message::WindowOpened(id)),
        iced::Event::Window(iced_window::Event::Focused) => Some(Message::ShowWindowFromFocus),
        iced::Event::Window(iced_window::Event::Unfocused) => Some(Message::WindowFocusLost),
        iced::Event::Window(iced_window::Event::Moved(position)) => {
            Some(Message::WindowMoved(position.x, position.y))
        }
        _ => None,
    });
    let hotkey_tray_sub = Subscription::run(|| {
        iced::stream::channel(
            10,
            |mut sender: iced::futures::channel::mpsc::Sender<Message>| async move {
                loop {
                    if let Some(id) = hotkey::check_hotkey_event(HOTKEY_DEBOUNCE_MS) {
                        if id == hotkey::MAIN_HOTKEY_ID {
                            let _ = sender.send(Message::ShowWindow).await;
                        } else {
                            let _ = sender.send(Message::GlobalHotkeyTriggered(id)).await;
                        }
                    }
                    if let Some(evt) = tray::check_events_basic() {
                        match evt {
                            tray::TrayBasicEvent::Show => {
                                let _ = sender.send(Message::ShowWindow).await;
                            }
                            tray::TrayBasicEvent::Settings => {
                                let _ = sender.send(Message::OpenSettingsPage).await;
                            }
                            tray::TrayBasicEvent::Quit => {
                                let _ = sender.send(Message::QuitApp).await;
                            }
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            },
        )
    });

    // Web server subscription
    let web_sub = if state.web_server_enabled {
        let web_port = DEFAULT_WEB_SERVER_PORT;
        Subscription::run_with(("web_server", web_port), |data| {
            let web_port = data.1;
            iced::stream::channel(
                10,
                move |_sender: iced::futures::channel::mpsc::Sender<Message>| {
                    async move {
                        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                        let web_state = WebState::new(tx);

                        // Initialize global web state
                        init_web_state(web_state.clone()).await;

                        // Run web server and clipboard handler concurrently
                        tokio::select! {
                            result = start_web_server(web_state, web_port) => {
                                if let Err(e) = result {
                                    eprintln!("Web server error: {}", e);
                                }
                            }
                            _ = async {
                                while let Some(entry) = rx.recv().await {
                                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                        if entry.clip_type.eq_ignore_ascii_case("image")
                                            && let Some(data_url) = entry.image_data_url.as_deref()
                                            && let Some((data, width, height)) =
                                               decode_image_data_url(data_url)
                                        {
                                            let image = arboard::ImageData {
                                                width,
                                                height,
                                                bytes: std::borrow::Cow::Owned(data),
                                            };
                                            let _ = clipboard.set_image(image);
                                            continue;
                                        }

                                        let _ = clipboard.set_text(&entry.content);
                                    }
                                }
                            } => {}
                        }
                    }
                },
            )
        })
    } else {
        Subscription::none()
    };

    let mut subs = vec![
        clipboard_sub,
        mouse_sub,
        window_sub,
        hotkey_tray_sub,
        web_sub,
    ];
    if state.window.animating {
        let anim_sub = iced::time::every(Duration::from_millis(16)).map(|_| Message::AnimationTick);
        subs.push(anim_sub);
    }

    if state.json.is_selecting {
        subs.push(
            iced::time::every(Duration::from_millis(30))
                .map(|_| Message::Json(json::message::Message::Tick)),
        );
    }

    if state.page.current == WorkflowEditor {
        subs.push(
            event::listen()
                .map(|evt| {
                    if let iced::Event::Keyboard(key_event) = evt {
                        Some(Message::WorkflowEditor(
                            workflow_editor::message::WorkflowEditorMessage::HotkeyRecording(
                                Uuid::nil(),
                                key_event,
                            ),
                        ))
                    } else {
                        None
                    }
                })
                .filter_map(|m| m),
        );

        if state.workflow_editor.has_unsaved_changes {
            subs.push(iced::time::every(Duration::from_millis(80)).map(|_| {
                Message::WorkflowEditor(
                    workflow_editor::message::WorkflowEditorMessage::SaveIndicatorTick,
                )
            }));
        }
    }

    Subscription::batch(subs)
}
