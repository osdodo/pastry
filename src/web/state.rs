use std::sync::Arc;

use base64::Engine;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub clip_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_data_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_width: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_height: Option<usize>,
}

pub fn decode_image_data_url(data_url: &str) -> Option<(Vec<u8>, usize, usize)> {
    let encoded = data_url
        .split_once(',')
        .map(|(_, body)| body)
        .unwrap_or(data_url)
        .trim();

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    let image = image::load_from_memory(&bytes).ok()?;
    let rgba = image.to_rgba8();
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;

    if width == 0 || height == 0 {
        return None;
    }

    Some((rgba.into_raw(), width, height))
}

#[derive(Clone)]
pub struct WebState {
    pub latest_clipboard: Arc<RwLock<Option<ClipboardEntry>>>,
    pub clipboard_sender: tokio::sync::mpsc::UnboundedSender<ClipboardEntry>,
}

impl WebState {
    pub fn new(clipboard_sender: tokio::sync::mpsc::UnboundedSender<ClipboardEntry>) -> Self {
        Self {
            latest_clipboard: Arc::new(RwLock::new(None)),
            clipboard_sender,
        }
    }

    pub async fn update_clipboard(&self, entry: ClipboardEntry) {
        let mut clipboard = self.latest_clipboard.write().await;
        *clipboard = Some(entry);
    }

    pub async fn get_latest(&self) -> Option<ClipboardEntry> {
        let clipboard = self.latest_clipboard.read().await;
        clipboard.clone()
    }
}
