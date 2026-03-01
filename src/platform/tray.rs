use std::sync::{OnceLock, RwLock};

use crate::ui::language;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};

static MENU_SHOW_ID: OnceLock<RwLock<MenuId>> = OnceLock::new();
static MENU_SETTINGS_ID: OnceLock<RwLock<MenuId>> = OnceLock::new();
static MENU_QUIT_ID: OnceLock<RwLock<MenuId>> = OnceLock::new();
pub fn setup_tray() {
    init_linux_gtk();

    let menu = Menu::new();
    let show_item = MenuItem::new(language::tr(language::Text::ShowClipboard), true, None);
    let settings_item = MenuItem::new(language::tr(language::Text::Settings), true, None);
    let quit_item = MenuItem::new(language::tr(language::Text::Quit), true, None);

    if let Some(lock) = MENU_SHOW_ID.get() {
        if let Ok(mut id) = lock.write() {
            *id = show_item.id().clone();
        }
    } else {
        let _ = MENU_SHOW_ID.set(RwLock::new(show_item.id().clone()));
    }
    if let Some(lock) = MENU_QUIT_ID.get() {
        if let Ok(mut id) = lock.write() {
            *id = quit_item.id().clone();
        }
    } else {
        let _ = MENU_QUIT_ID.set(RwLock::new(quit_item.id().clone()));
    }
    if let Some(lock) = MENU_SETTINGS_ID.get() {
        if let Ok(mut id) = lock.write() {
            *id = settings_item.id().clone();
        }
    } else {
        let _ = MENU_SETTINGS_ID.set(RwLock::new(settings_item.id().clone()));
    }

    let _ = menu.append(&show_item);
    let _ = menu.append(&settings_item);
    let _ = menu.append(&quit_item);

    let icon = load_icon();
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Pastry")
        .with_icon(icon)
        .build()
        .unwrap_or_else(|_| panic!("{}", language::tr(language::Text::TrayBuildFailed)));

    Box::leak(Box::new(tray));
}

#[cfg(target_os = "linux")]
fn init_linux_gtk() {
    gtk::init().unwrap_or_else(|err| panic!("Failed to initialize GTK: {err}"));
}

#[cfg(not(target_os = "linux"))]
fn init_linux_gtk() {}

fn load_icon() -> tray_icon::Icon {
    const LOGO_BYTES: &[u8] = include_bytes!("../../assets/logo.png");
    let img = image::load_from_memory_with_format(LOGO_BYTES, image::ImageFormat::Png)
        .unwrap_or_else(|_| panic!("{}", language::tr(language::Text::ImageLoadFailed)))
        .to_rgba8();

    let target = tray_icon_target_size();
    let (src_width, src_height) = img.dimensions();

    // Scale to fit target size while maintaining aspect ratio
    let scale = (target as f32 / src_width.max(src_height) as f32).min(1.0);
    let scaled_width = ((src_width as f32) * scale).round() as u32;
    let scaled_height = ((src_height as f32) * scale).round() as u32;

    let resized = image::imageops::resize(
        &img,
        scaled_width,
        scaled_height,
        image::imageops::FilterType::Lanczos3,
    );

    let mut canvas = image::RgbaImage::from_pixel(target, target, image::Rgba([0, 0, 0, 0]));

    let offset_x = ((target - scaled_width) / 2) as i64;
    let offset_y = ((target - scaled_height) / 2) as i64;

    image::imageops::overlay(&mut canvas, &resized, offset_x, offset_y);
    let data = canvas.into_raw();

    let mut rgba_data = Vec::with_capacity(data.len());
    for chunk in data.chunks(4) {
        let a = chunk[3];
        if a > 0 {
            rgba_data.extend_from_slice(&[255, 255, 255, a]);
        } else {
            rgba_data.extend_from_slice(&[0, 0, 0, 0]);
        }
    }

    tray_icon::Icon::from_rgba(rgba_data, target, target)
        .unwrap_or_else(|_| panic!("{}", language::tr(language::Text::IconCreateFailed)))
}

fn tray_icon_target_size() -> u32 {
    #[cfg(target_os = "macos")]
    {
        // Use larger size for Retina displays (modern macOS status bar supports higher resolution)
        64
    }
    #[cfg(target_os = "windows")]
    {
        32
    }
    #[cfg(target_os = "linux")]
    {
        24
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        24
    }
}

#[derive(Debug, Clone)]
pub enum TrayBasicEvent {
    Show,
    Settings,
    Quit,
}

pub fn check_events_basic() -> Option<TrayBasicEvent> {
    let menu_receiver = MenuEvent::receiver();
    if let Ok(event) = menu_receiver.try_recv() {
        let id = event.id.clone();
        if let Some(lock) = MENU_SHOW_ID.get()
            && let Ok(show_id) = lock.read()
            && id == show_id.clone()
        {
            return Some(TrayBasicEvent::Show);
        }
        if let Some(lock) = MENU_SETTINGS_ID.get()
            && let Ok(settings_id) = lock.read()
            && id == settings_id.clone()
        {
            return Some(TrayBasicEvent::Settings);
        }
        if let Some(lock) = MENU_QUIT_ID.get()
            && let Ok(quit_id) = lock.read()
            && id == quit_id.clone()
        {
            return Some(TrayBasicEvent::Quit);
        }
    }
    None
}
