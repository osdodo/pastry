pub const WINDOW_WIDTH: f32 = 500.0;
pub const MIN_WINDOW_WIDTH: f32 = 500.0;
pub const MIN_WINDOW_HEIGHT: f32 = 500.0;
pub const ANIMATION_DURATION_MS: u64 = 200;
pub const HOTKEY_DEBOUNCE_MS: u64 = 300;
pub const CLIPBOARD_CHECK_MS: u64 = 800;
pub const MAX_HISTORY_SIZE: usize = 100;
pub const WINDOW_MARGIN: f32 = 50.0;
pub const WINDOW_RADIUS: f32 = if cfg!(target_os = "windows") {
    0.0
} else {
    20.0
};
pub const CARD_RADIUS: f32 = if cfg!(target_os = "windows") {
    0.0
} else {
    12.0
};
pub const BUTTON_RADIUS: f32 = if cfg!(target_os = "windows") {
    0.0
} else {
    10.0
};
pub const INPUT_RADIUS: f32 = if cfg!(target_os = "windows") {
    0.0
} else {
    12.0
};
