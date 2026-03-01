use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Local};
use serde_json::Value;

pub use crate::domain::clipboard::{CardData, ClipType, ImageFormat};
use crate::platform::svg;
use crate::ui::language;
use crate::ui::util::color::{Color, parse_color};

fn is_json_content(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Check if valid JSON first
    let value = match serde_json::from_str::<Value>(trimmed) {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Only allow objects and arrays
    matches!(value, Value::Object(_) | Value::Array(_))
}

#[derive(Debug, Clone)]
pub struct CardState {
    pub content: String,
    pub clip_type: ClipType,
    pub content_hash: u64,
    pub timestamp: DateTime<Local>,
    pub is_favorite: bool,
    pub image_data: Option<(Arc<Vec<u8>>, usize, usize)>,
    pub image_handle: Option<iced::widget::image::Handle>,
    pub image_format: Option<ImageFormat>,
    pub file_path: Option<String>,
    pub script_output: Option<String>,
    pub script_output_copied: bool,
    pub script_name: Option<String>,
    pub script_id: Option<String>,
    pub saved_image_path: Option<String>,
    pub is_copied: bool,
    pub is_json: bool,
    pub is_color: Option<Color>,
}

type LoadedImage = (
    Vec<u8>,
    u32,
    u32,
    ImageFormat,
    Option<iced::widget::image::Handle>,
);

impl CardState {
    pub fn compute_text_hash(text: &str, clip_type: ClipType) -> u64 {
        let tag: u8 = match clip_type {
            ClipType::PlainText => 0,
            ClipType::RichText => 1,
            ClipType::Image => 2,
            ClipType::File => 3,
        };

        let mut hasher = DefaultHasher::new();
        tag.hash(&mut hasher);
        text.hash(&mut hasher);
        hasher.finish()
    }

    fn compute_image_hash(data: &[u8], width: usize, height: usize) -> u64 {
        let tag: u8 = 2;
        let mut hasher = DefaultHasher::new();
        tag.hash(&mut hasher);
        data.hash(&mut hasher);
        width.hash(&mut hasher);
        height.hash(&mut hasher);
        hasher.finish()
    }

    fn thumbnail_handle(
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Option<iced::widget::image::Handle> {
        let img = image::RgbaImage::from_raw(width, height, data.to_vec())?;
        let max_thumb_size = 200u32;
        let (thumb_width, thumb_height) = if width > max_thumb_size || height > max_thumb_size {
            let scale = max_thumb_size as f32 / (width.max(height) as f32);
            (
                (width as f32 * scale) as u32,
                (height as f32 * scale) as u32,
            )
        } else {
            (width, height)
        };

        let thumb = image::imageops::resize(
            &img,
            thumb_width,
            thumb_height,
            image::imageops::FilterType::Triangle,
        );

        let mut png_bytes: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        thumb.write_to(&mut cursor, image::ImageFormat::Png).ok()?;
        Some(iced::widget::image::Handle::from_bytes(png_bytes))
    }

    fn load_image_from_saved_path(saved_path: &str) -> Option<LoadedImage> {
        let path = Path::new(saved_path);
        if !path.exists() {
            return None;
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());

        let (data, width, height, format) = match ext.as_deref() {
            Some("svg") => {
                let svg_data = std::fs::read(path).ok()?;
                let (data, width, height) = Self::render_svg_to_rgba(&svg_data)?;
                (data, width, height, ImageFormat::Svg)
            }
            Some("png") => {
                let img = image::open(path).ok()?.to_rgba8();
                let (w, h) = img.dimensions();
                (img.into_raw(), w, h, ImageFormat::Png)
            }
            Some("jpg" | "jpeg") => {
                let img = image::open(path).ok()?.to_rgba8();
                let (w, h) = img.dimensions();
                (img.into_raw(), w, h, ImageFormat::Jpeg)
            }
            _ => {
                let img = image::open(path).ok()?.to_rgba8();
                let (w, h) = img.dimensions();
                (img.into_raw(), w, h, ImageFormat::Other)
            }
        };

        let handle = Self::thumbnail_handle(width, height, &data);
        Some((data, width, height, format, handle))
    }

    fn render_svg_to_rgba(svg_data: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
        svg::render_svg_bytes_to_rgba(svg_data, Some(512), Some(resvg::tiny_skia::Color::WHITE))
    }

    pub fn new_with_hash(content: String, content_hash: u64) -> Self {
        let clip_type = if content.contains('<') && content.contains('>') {
            ClipType::RichText
        } else {
            ClipType::PlainText
        };

        let is_json = is_json_content(&content);
        let is_color = parse_color(&content);

        Self {
            content,
            clip_type,
            content_hash,
            timestamp: Local::now(),
            is_favorite: false,
            image_data: None,
            image_handle: None,
            image_format: None,
            file_path: None,
            script_output: None,
            script_output_copied: false,
            script_name: None,
            script_id: None,
            saved_image_path: None,
            is_copied: false,
            is_json,
            is_color,
        }
    }

    pub fn new_image_with_path_with_hash(
        data: Vec<u8>,
        width: usize,
        height: usize,
        format: ImageFormat,
        file_path: Option<String>,
        content_hash: u64,
    ) -> Self {
        let format_str = match format {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpeg => "JPEG",
            ImageFormat::Svg => "SVG",
            ImageFormat::Other => "Image",
        };
        let content = format!("{} {}x{}", format_str, width, height);

        let handle = Self::thumbnail_handle(width as u32, height as u32, &data);

        Self {
            content,
            clip_type: ClipType::Image,
            content_hash,
            timestamp: Local::now(),
            is_favorite: false,
            image_data: Some((Arc::new(data), width, height)),
            image_handle: handle,
            image_format: Some(format),
            file_path,
            script_output: None,
            script_output_copied: false,
            script_name: None,
            script_id: None,
            saved_image_path: None,
            is_copied: false,
            is_json: false,
            is_color: None,
        }
    }

    pub fn to_favorite_data(&self) -> CardData {
        CardData {
            content: self.content.clone(),
            clip_type: self.clip_type,
            timestamp: self.timestamp,
            is_favorite: self.is_favorite,
            image_data: None,
            image_format: self.image_format,
            file_path: None,
            script_output: self.script_output.clone(),
            script_id: self.script_id.clone(),
            script_name: self.script_name.clone(),
            saved_image_path: self.saved_image_path.clone(),
        }
    }

    pub fn from_data(data: CardData) -> Self {
        let image_data = data
            .image_data
            .map(|(bytes, width, height)| (Arc::new(bytes), width, height));

        let content_hash = match data.clip_type {
            ClipType::Image => image_data
                .as_ref()
                .map(|(bytes, width, height)| Self::compute_image_hash(bytes, *width, *height))
                .unwrap_or_else(|| Self::compute_text_hash(&data.content, data.clip_type)),
            _ => Self::compute_text_hash(&data.content, data.clip_type),
        };

        let is_json = data.clip_type != ClipType::Image && is_json_content(&data.content);
        let is_color = parse_color(&data.content);

        let mut state = Self {
            content: data.content,
            clip_type: data.clip_type,
            content_hash,
            timestamp: data.timestamp,
            is_favorite: data.is_favorite,
            image_data,
            image_handle: None,
            image_format: data.image_format,
            file_path: data.file_path,
            script_output: data.script_output,
            script_output_copied: false,
            script_name: data.script_name,
            script_id: data.script_id,
            saved_image_path: data.saved_image_path,
            is_copied: false,
            is_json,
            is_color,
        };

        if state.clip_type == ClipType::Image {
            if let Some((data, width, height)) = state.image_data.as_ref() {
                state.image_handle =
                    Self::thumbnail_handle(*width as u32, *height as u32, data.as_slice());
            } else if let Some(saved_path) = state.saved_image_path.clone()
                && let Some((data, width, height, format, handle)) =
                    Self::load_image_from_saved_path(&saved_path)
            {
                let width = width as usize;
                let height = height as usize;
                state.content_hash = Self::compute_image_hash(&data, width, height);
                state.image_data = Some((Arc::new(data), width, height));
                state.image_format = Some(format);
                state.image_handle = handle;
            }
        }

        state
    }

    pub fn time_ago(&self) -> String {
        let now = Local::now();
        let duration = now.signed_duration_since(self.timestamp);

        if duration.num_seconds() < 60 {
            language::tr(language::Text::JustNow).to_string()
        } else if duration.num_minutes() < 60 {
            language::tr(language::Text::MinutesAgoFmt)
                .replace("{}", &duration.num_minutes().to_string())
        } else if duration.num_hours() < 24 {
            language::tr(language::Text::HoursAgoFmt)
                .replace("{}", &duration.num_hours().to_string())
        } else {
            language::tr(language::Text::DaysAgoFmt).replace("{}", &duration.num_days().to_string())
        }
    }
}

impl Hash for CardState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content_hash.hash(state);
        self.is_favorite.hash(state);
        self.is_copied.hash(state);
        self.script_output.hash(state);
        self.script_output_copied.hash(state);
        self.script_name.hash(state);
        self.script_id.hash(state);
        self.time_ago().hash(state);
        self.is_json.hash(state);
        self.file_path.hash(state);
        self.saved_image_path.hash(state);
        // is_color is not hashed as it's derived from content
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CardMessage {
    ToggleFavorite,
    ShowDeleteConfirm,
    Copy,
    RunScript,
    RunWorkflow,
    CopyScriptOutput,
    ResetCopyIcon,
    ResetScriptOutputCopyIcon,
    CompressImage,
    ShowJsonFormat,
    ToggleColorPicker(Color),
    DeleteScriptOutput,
}
