mod app;
mod domain;
mod features;
mod platform;
mod services;
mod ui;
mod web;

use iced::window::{Level, Position};
use iced::{Point, Size, Theme};

use ui::constants::{MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH, WINDOW_MARGIN, WINDOW_WIDTH};

fn main() -> iced::Result {
    let settings = {
        let storage = services::storage::Storage::new();
        storage
            .load::<app::AppSettings>(app::SETTINGS_FILE)
            .unwrap_or_default()
    };

    ui::language::init_with_code(settings.language.as_deref());
    ui::theme::init_with_mode(settings.theme_mode);
    let start_hidden = settings.start_hidden.unwrap_or(false);
    let web_server_enabled = settings.web_server_enabled.unwrap_or(false);

    platform::tray::setup_tray();
    platform::hotkey::setup_hotkey();

    let (screen_width, screen_height) = platform::screen::get_screen_size();
    let window_height = platform::screen::get_window_height(WINDOW_MARGIN);

    iced::application(
        move || app::State::new(start_hidden, web_server_enabled),
        app::update::update,
        app::view::view,
    )
    .title("Pastry")
    .subscription(app::subscription::subscription)
    .theme(|_: &app::State| -> Theme {
        let palette = ui::theme::palette(ui::theme::current());
        Theme::custom("PastryTheme".to_string(), palette)
    })
    .window(iced::window::Settings {
        decorations: false,
        transparent: true,
        level: Level::AlwaysOnTop,
        size: Size::new(WINDOW_WIDTH, window_height),
        min_size: Some(Size::new(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT)),
        position: Position::Specific(Point::new(
            screen_width + 10.0,
            (screen_height - window_height) / 2.0,
        )),
        visible: true,
        ..Default::default()
    })
    .run()
}
