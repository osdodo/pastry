use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{Mutex, broadcast, oneshot};
use tokio::time::Duration;
use warp::{Filter, ws::Message};

use super::qr::get_local_ip;
use super::state::{ClipboardEntry, WebState};
use crate::ui::{language, theme};

const RUNTIME_CONFIG_PLACEHOLDER: &str = "__PASTRY_RUNTIME_CONFIG__";
const INDEX_HTML_TEMPLATE: &str = include_str!("./index.html");

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum WsClientMessage {
    #[serde(rename = "push_clipboard")]
    PushClipboard {
        content: String,
        #[serde(default = "default_clip_type")]
        clip_type: String,
        #[serde(default)]
        image_data_url: Option<String>,
        #[serde(default)]
        image_width: Option<usize>,
        #[serde(default)]
        image_height: Option<usize>,
    },
    #[serde(rename = "get_latest")]
    GetLatest,
}

fn default_clip_type() -> String {
    "text".to_string()
}

struct ShutdownHandle {
    id: u64,
    sender: oneshot::Sender<()>,
}

static WEB_SERVER_SHUTDOWN: OnceLock<Mutex<Option<ShutdownHandle>>> = OnceLock::new();
static WEB_SERVER_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

async fn register_shutdown_sender(
    sender: oneshot::Sender<()>,
) -> (u64, Option<oneshot::Sender<()>>) {
    let id = WEB_SERVER_INSTANCE_ID.fetch_add(1, Ordering::Relaxed);
    let lock = WEB_SERVER_SHUTDOWN.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().await;
    let previous = guard.take().map(|handle| handle.sender);
    *guard = Some(ShutdownHandle { id, sender });
    (id, previous)
}

async fn clear_shutdown_sender(id: u64) {
    if let Some(lock) = WEB_SERVER_SHUTDOWN.get() {
        let mut guard = lock.lock().await;
        if guard.as_ref().map(|handle| handle.id) == Some(id) {
            *guard = None;
        }
    }
}

pub async fn stop_web_server() {
    if let Some(lock) = WEB_SERVER_SHUTDOWN.get() {
        let sender = {
            let mut guard = lock.lock().await;
            guard.take().map(|handle| handle.sender)
        };

        if let Some(sender) = sender {
            let _ = sender.send(());
        }
    }
}

fn with_state(state: WebState) -> impl Filter<Extract = (WebState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn with_broadcast(
    tx: broadcast::Sender<String>,
) -> impl Filter<Extract = (broadcast::Sender<String>,), Error = Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

fn to_clipboard_update(entry: &ClipboardEntry) -> String {
    serde_json::json!({
        "type": "clipboard_update",
        "content": entry.content,
        "timestamp": entry.timestamp.to_rfc3339(),
        "clip_type": entry.clip_type,
        "image_data_url": entry.image_data_url,
        "image_width": entry.image_width,
        "image_height": entry.image_height,
    })
    .to_string()
}

fn render_index_html() -> String {
    let current_lang = language::to_code(language::current());
    let current_theme = match theme::current() {
        theme::ThemeMode::Light => "light",
        theme::ThemeMode::Dark => "dark",
    };

    let runtime_config = serde_json::json!({
        "language": current_lang,
        "theme": current_theme,
    })
    .to_string();

    INDEX_HTML_TEMPLATE.replace(RUNTIME_CONFIG_PLACEHOLDER, &runtime_config)
}

pub async fn start_web_server(
    state: WebState,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (shutdown_id, previous_shutdown) = register_shutdown_sender(shutdown_tx).await;
    if let Some(sender) = previous_shutdown {
        let _ = sender.send(());
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Web server listening on http://0.0.0.0:{}", port);
    if let Some(ip) = get_local_ip() {
        println!("Access from LAN: http://{}:{}", ip, port);
    }

    let (tx, _) = broadcast::channel::<String>(64);
    let state_for_broadcast = state.clone();
    let tx_for_broadcast = tx.clone();
    let broadcast_task = tokio::spawn(async move {
        let mut last_key: Option<(String, String, Option<String>)> = None;

        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if let Some(entry) = state_for_broadcast.get_latest().await {
                let key = (
                    entry.content.clone(),
                    entry.clip_type.clone(),
                    entry.image_data_url.clone(),
                );
                if last_key.as_ref() != Some(&key) {
                    last_key = Some(key);
                    let _ = tx_for_broadcast.send(to_clipboard_update(&entry));
                }
            }
        }
    });

    let ws = warp::path!("ws")
        .and(warp::ws())
        .and(with_state(state.clone()))
        .and(with_broadcast(tx.clone()))
        .and_then(
            |ws: warp::ws::Ws, state: WebState, tx: broadcast::Sender<String>| async move {
                Ok::<_, Infallible>(
                    ws.on_upgrade(move |socket| handle_websocket(socket, state, tx)),
                )
            },
        );

    let index = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::html(render_index_html()));

    let health = warp::path!("health")
        .and(warp::get())
        .map(|| warp::reply::with_status("ok".to_string(), warp::http::StatusCode::OK));

    let routes = ws.or(index).or(health).boxed();
    warp::serve(routes)
        .bind(addr)
        .await
        .graceful(async move {
            let _ = shutdown_rx.await;
        })
        .run()
        .await;

    broadcast_task.abort();
    let _ = broadcast_task.await;
    clear_shutdown_sender(shutdown_id).await;
    Ok(())
}

async fn handle_websocket(
    socket: warp::ws::WebSocket,
    state: WebState,
    tx: broadcast::Sender<String>,
) {
    let (mut sink, mut stream) = socket.split();
    let mut rx = tx.subscribe();

    if let Some(entry) = state.get_latest().await {
        let _ = sink.send(Message::text(to_clipboard_update(&entry))).await;
    }

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(text) => {
                        if sink.send(Message::text(text)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            next_msg = stream.next() => {
                match next_msg {
                    Some(Ok(msg)) if msg.is_text() => {
                        if let Ok(text) = msg.to_str()
                            && let Ok(ws_msg) = serde_json::from_str::<WsClientMessage>(text)
                        {
                            match ws_msg {
                                WsClientMessage::PushClipboard {
                                    content,
                                    clip_type,
                                    image_data_url,
                                    image_width,
                                    image_height,
                                } => {
                                    let is_image = clip_type.eq_ignore_ascii_case("image")
                                        && image_data_url.is_some();
                                    let entry = ClipboardEntry {
                                        content,
                                        timestamp: chrono::Local::now(),
                                        clip_type: if is_image {
                                            "image".to_string()
                                        } else {
                                            "text".to_string()
                                        },
                                        image_data_url: if is_image {
                                            image_data_url
                                        } else {
                                            None
                                        },
                                        image_width: if is_image { image_width } else { None },
                                        image_height: if is_image { image_height } else { None },
                                    };
                                    state.update_clipboard(entry.clone()).await;
                                    let _ = state.clipboard_sender.send(entry);
                                }
                                WsClientMessage::GetLatest => {
                                    if let Some(entry) = state.get_latest().await
                                        && sink.send(Message::text(to_clipboard_update(&entry))).await.is_err()
                                    {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(msg)) if msg.is_close() => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => break,
                }
            }
        }
    }
}
