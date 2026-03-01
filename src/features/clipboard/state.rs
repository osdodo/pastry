use crate::features::clipboard::model::CardState;
use chrono::{DateTime, Local};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    #[default]
    Recent,
    Favorite,
}

#[derive(Debug, Clone)]
pub struct State {
    pub history: Vec<CardState>,
    pub compressing: bool,
    pub compress_message: Option<String>,
    pub last_clipboard_hash: u64,
    pub search_text: String,
    pub filter: Filter,
    pub delete_confirm_index: Option<usize>,
    pub unfavorite_confirm_index: Option<usize>,
    pub startup_time: DateTime<Local>,
}

impl State {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            compressing: false,
            compress_message: None,
            last_clipboard_hash: 0,
            search_text: String::new(),
            filter: Filter::default(),
            delete_confirm_index: None,
            unfavorite_confirm_index: None,
            startup_time: Local::now(),
        }
    }
}
