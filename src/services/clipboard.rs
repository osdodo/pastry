use std::{
    borrow::Cow,
    path::Path,
    sync::{Mutex, OnceLock},
};

use arboard::{Clipboard, ImageData};

use crate::platform::svg::render_svg_bytes_to_rgba;
use crate::{
    domain::clipboard::{CardData, ClipType, ImageFormat},
    services::storage::Storage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSourceFormat {
    Png,
    Jpeg,
    Svg,
    Other,
}

#[derive(Debug, Clone)]
pub enum ClipboardContent {
    Text(String),
    Image(Vec<u8>, usize, usize, ImageSourceFormat),
    ImageFile(String, Vec<u8>, usize, usize, ImageSourceFormat),
}

static LAST_CHANGE_COUNT: OnceLock<Mutex<i64>> = OnceLock::new();

fn is_image_file(path: &str) -> bool {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    matches!(
        ext.as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "ico" | "tiff" | "tif" | "svg")
    )
}

fn load_image_from_file(path: &str) -> Option<(Vec<u8>, usize, usize, ImageSourceFormat)> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    let format = match ext.as_deref() {
        Some("png") => ImageSourceFormat::Png,
        Some("jpg" | "jpeg") => ImageSourceFormat::Jpeg,
        Some("svg") => ImageSourceFormat::Svg,
        _ => ImageSourceFormat::Other,
    };

    if ext.as_deref() == Some("svg") {
        let (data, width, height) = load_svg_file(path)?;
        return Some((data, width, height, format));
    }

    let img = image::open(path).ok()?;
    let rgba = img.to_rgba8();
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;
    let data = rgba.into_raw();
    Some((data, width, height, format))
}

fn load_svg_file(path: &str) -> Option<(Vec<u8>, usize, usize)> {
    let svg_data = std::fs::read(path).ok()?;
    let (data, width, height) =
        render_svg_bytes_to_rgba(&svg_data, Some(512), Some(resvg::tiny_skia::Color::WHITE))?;
    Some((data, width as usize, height as usize))
}

#[cfg(target_os = "macos")]
fn get_file_urls_from_clipboard() -> Vec<String> {
    use objc2_app_kit::NSPasteboard;
    use objc2_foundation::{NSString, NSURL};

    let pasteboard = NSPasteboard::generalPasteboard();
    let mut paths = Vec::new();

    if let Some(items) = pasteboard.pasteboardItems() {
        for item in items {
            let file_url_type = NSString::from_str("public.file-url");
            if let Some(url_string) = item.stringForType(&file_url_type) {
                let url_str = url_string.to_string();
                let ns_url_string = NSString::from_str(&url_str);
                if let Some(url) = NSURL::URLWithString(&ns_url_string)
                    && let Some(path) = url.path()
                {
                    paths.push(path.to_string());
                }
            }
        }
    }

    paths
}

#[cfg(target_os = "windows")]
fn get_file_urls_from_clipboard() -> Vec<String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
    use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
    use windows::Win32::System::Ole::CF_HDROP;
    use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};

    let mut paths = Vec::new();

    unsafe {
        if OpenClipboard(Some(HWND::default())).is_err() {
            return paths;
        }

        if let Ok(handle) = GetClipboardData(CF_HDROP.0 as u32) {
            let hdrop_ptr = GlobalLock(windows::Win32::Foundation::HGLOBAL(handle.0));
            if !hdrop_ptr.is_null() {
                let hdrop = HDROP(hdrop_ptr);
                let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
                for i in 0..count {
                    let len = DragQueryFileW(hdrop, i, None);
                    if len > 0 {
                        let mut buffer = vec![0u16; (len + 1) as usize];
                        DragQueryFileW(hdrop, i, Some(&mut buffer));

                        if let Some(pos) = buffer.iter().position(|&c| c == 0) {
                            buffer.truncate(pos);
                        }

                        let path = OsString::from_wide(&buffer);
                        if let Some(path_str) = path.to_str() {
                            paths.push(path_str.to_string());
                        }
                    }
                }

                let _ = GlobalUnlock(windows::Win32::Foundation::HGLOBAL(handle.0));
            }
        }

        let _ = CloseClipboard();
    }

    paths
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn get_file_urls_from_clipboard() -> Vec<String> {
    Vec::new()
}

pub fn should_check_clipboard() -> bool {
    let current_count = get_clipboard_change_count();

    if current_count == -1 {
        return true;
    }

    match LAST_CHANGE_COUNT.get() {
        Some(last_count) => {
            if let Ok(mut count) = last_count.lock()
                && current_count != *count
            {
                *count = current_count;
                return true;
            }
        }
        None => {
            LAST_CHANGE_COUNT.get_or_init(|| Mutex::new(current_count));
            return true;
        }
    }

    false
}

pub fn get_clipboard_content() -> Option<ClipboardContent> {
    let file_paths = get_file_urls_from_clipboard();
    for path in &file_paths {
        if is_image_file(path)
            && let Some((data, width, height, format)) = load_image_from_file(path)
        {
            return Some(ClipboardContent::ImageFile(
                path.clone(),
                data,
                width,
                height,
                format,
            ));
        }
    }

    let mut clipboard = Clipboard::new().ok()?;
    if let Ok(img) = clipboard.get_image() {
        let width = img.width;
        let height = img.height;
        let data = img.bytes.into_owned();
        let expected_size = width * height * 4;

        if data.len() == expected_size && width > 0 && height > 0 {
            return Some(ClipboardContent::Image(
                data,
                width,
                height,
                ImageSourceFormat::Png,
            ));
        }
    }

    if let Ok(text) = clipboard.get_text()
        && !text.is_empty()
    {
        let path = text.trim();
        if is_image_file(path)
            && Path::new(path).exists()
            && let Some((data, width, height, format)) = load_image_from_file(path)
        {
            return Some(ClipboardContent::ImageFile(
                path.to_string(),
                data,
                width,
                height,
                format,
            ));
        }
        return Some(ClipboardContent::Text(text));
    }

    None
}

pub fn set_clipboard_image(data: &[u8], width: usize, height: usize) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let img_data = ImageData {
        width,
        height,
        bytes: Cow::Borrowed(data),
    };
    clipboard.set_image(img_data).map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
fn get_clipboard_change_count() -> i64 {
    use objc2_app_kit::NSPasteboard;
    let pasteboard = NSPasteboard::generalPasteboard();
    pasteboard.changeCount() as i64
}

#[cfg(target_os = "windows")]
fn get_clipboard_change_count() -> i64 {
    use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
    unsafe { GetClipboardSequenceNumber() as i64 }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn get_clipboard_change_count() -> i64 {
    -1
}

const FAVORITES_FILE: &str = "favorites.json";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipboardItem {
    pub content: String,
    pub timestamp: i64,
    pub is_favorite: bool,
    pub clip_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_data: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved_image_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_name: Option<String>,
}

impl From<&CardData> for ClipboardItem {
    fn from(state: &CardData) -> Self {
        let clip_type = match state.clip_type {
            ClipType::PlainText => "PlainText",
            ClipType::RichText => "RichText",
            ClipType::Image => "Image",
            ClipType::File => "File",
        }
        .to_string();

        Self {
            content: state.content.clone(),
            timestamp: state.timestamp.timestamp(),
            is_favorite: state.is_favorite,
            clip_type,
            image_data: None,
            image_width: None,
            image_height: None,
            saved_image_path: state.saved_image_path.clone(),
            script_output: state.script_output.clone(),
            script_id: state.script_id.clone(),
            script_name: state.script_name.clone(),
        }
    }
}

impl ClipboardItem {
    pub fn to_data(&self) -> Option<CardData> {
        use chrono::{Local, TimeZone};

        let timestamp = Local.timestamp_opt(self.timestamp, 0).single()?;

        let clip_type = match self.clip_type.as_str() {
            "PlainText" => ClipType::PlainText,
            "RichText" => ClipType::RichText,
            "Image" => ClipType::Image,
            "File" => ClipType::File,
            _ => ClipType::PlainText,
        };

        let image_format = self.saved_image_path.as_ref().and_then(|saved_path| {
            std::path::Path::new(saved_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| match ext.to_lowercase().as_str() {
                    "svg" => ImageFormat::Svg,
                    "png" => ImageFormat::Png,
                    "jpg" | "jpeg" => ImageFormat::Jpeg,
                    _ => ImageFormat::Other,
                })
        });

        let image_data = if let (Some(data), Some(width), Some(height)) =
            (&self.image_data, self.image_width, self.image_height)
        {
            Some((data.clone(), width as usize, height as usize))
        } else {
            None
        };

        Some(CardData {
            content: self.content.clone(),
            clip_type,
            timestamp,
            is_favorite: self.is_favorite,
            image_data,
            image_format,
            file_path: None,
            script_output: self.script_output.clone(),
            script_id: self.script_id.clone(),
            script_name: self.script_name.clone(),
            saved_image_path: self.saved_image_path.clone(),
        })
    }
}

pub struct ClipboardStorage {
    storage: Storage,
}

impl ClipboardStorage {
    pub fn new() -> Self {
        Self {
            storage: Storage::new(),
        }
    }

    pub fn save_favorites(&self, items: &[ClipboardItem]) -> Result<(), String> {
        let items_vec: Vec<_> = items.to_vec();
        self.storage.save(FAVORITES_FILE, &items_vec)
    }

    pub fn load_favorites(&self) -> Result<Vec<ClipboardItem>, String> {
        self.storage.load(FAVORITES_FILE)
    }
}

pub fn load_favorite_cards() -> Result<Vec<CardData>, String> {
    let storage = ClipboardStorage::new();
    let items = storage.load_favorites()?;
    Ok(items.into_iter().filter_map(|i| i.to_data()).collect())
}

pub fn save_favorites(cards: &[CardData]) -> Result<(), String> {
    let storage = ClipboardStorage::new();
    let favorites: Vec<ClipboardItem> = cards
        .iter()
        .filter(|e| e.is_favorite)
        .map(|e| e.into())
        .collect();
    storage.save_favorites(&favorites)
}
