use chrono::{DateTime, Local};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipType {
    RichText,
    PlainText,
    Image,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Svg,
    Other,
}

#[derive(Debug, Clone)]
pub struct CardData {
    pub content: String,
    pub clip_type: ClipType,
    pub timestamp: DateTime<Local>,
    pub is_favorite: bool,
    pub image_data: Option<(Vec<u8>, usize, usize)>,
    pub image_format: Option<ImageFormat>,
    pub file_path: Option<String>,
    pub script_output: Option<String>,
    pub script_id: Option<String>,
    pub script_name: Option<String>,
    pub saved_image_path: Option<String>,
}
