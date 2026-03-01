pub mod color;
pub mod hotkey;

pub fn ui_radius(value: f32) -> f32 {
    if cfg!(target_os = "windows") {
        4.0
    } else {
        value
    }
}
