use std::sync::OnceLock;

static SCREEN_INFO: OnceLock<(f32, f32)> = OnceLock::new();
static WINDOW_HEIGHT: OnceLock<f32> = OnceLock::new();
static WINDOW_POSITION: OnceLock<(f32, f32)> = OnceLock::new();

pub fn get_screen_size() -> (f32, f32) {
    if let Some(&size) = SCREEN_INFO.get() {
        return size;
    }
    let size = get_screen_size_platform();
    SCREEN_INFO.set(size).ok();
    size
}

pub fn get_window_height(window_margin: f32) -> f32 {
    if let Some(&height) = WINDOW_HEIGHT.get() {
        return height;
    }
    let (_screen_width, screen_height) = get_screen_size();
    let height = (screen_height * 0.7).min(screen_height - window_margin * 2.0);
    WINDOW_HEIGHT.set(height).ok();
    height
}

pub fn set_window_position(x: f32, y: f32) {
    WINDOW_POSITION.set((x, y)).ok();
}

#[cfg(target_os = "macos")]
fn get_screen_size_platform() -> (f32, f32) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;

    if let Some(mtm) = MainThreadMarker::new()
        && let Some(screen) = NSScreen::mainScreen(mtm)
    {
        let frame = screen.frame();
        return (frame.size.width as f32, frame.size.height as f32);
    }
    (1920.0, 1080.0)
}

#[cfg(target_os = "windows")]
fn get_screen_size_platform() -> (f32, f32) {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);
        if width > 0 && height > 0 {
            return (width as f32, height as f32);
        }
    }
    (1920.0, 1080.0)
}

#[cfg(target_os = "linux")]
fn get_screen_size_platform() -> (f32, f32) {
    if let Some(size) = get_x11_screen_size() {
        return size;
    }
    (1920.0, 1080.0)
}

#[cfg(target_os = "linux")]
fn get_x11_screen_size() -> Option<(f32, f32)> {
    use x11rb::connection::Connection;
    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let setup = conn.setup();
    let screen = setup.roots.get(screen_num as usize)?;
    Some((
        screen.width_in_pixels as f32,
        screen.height_in_pixels as f32,
    ))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn get_screen_size_platform() -> (f32, f32) {
    (1920.0, 1080.0)
}
