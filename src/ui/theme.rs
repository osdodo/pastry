use iced::theme::Palette;
use iced::{Color, Theme};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

impl ThemeMode {}

static CURRENT_THEME_MODE: std::sync::OnceLock<std::sync::RwLock<ThemeMode>> =
    std::sync::OnceLock::new();

pub fn current() -> ThemeMode {
    if let Some(lock) = CURRENT_THEME_MODE.get() {
        match lock.read() {
            Ok(g) => *g,
            Err(_) => ThemeMode::default(),
        }
    } else {
        CURRENT_THEME_MODE.get_or_init(|| std::sync::RwLock::new(ThemeMode::default()));
        ThemeMode::default()
    }
}

pub fn set_current(mode: ThemeMode) {
    if let Some(lock) = CURRENT_THEME_MODE.get() {
        if let Ok(mut g) = lock.write() {
            *g = mode;
        }
    } else {
        CURRENT_THEME_MODE.get_or_init(|| std::sync::RwLock::new(mode));
    }
}

pub fn init_with_mode(mode: Option<ThemeMode>) {
    let theme_mode = mode.unwrap_or_default();
    set_current(theme_mode);
}

pub fn palette(mode: ThemeMode) -> Palette {
    match mode {
        ThemeMode::Light => Palette {
            background: Color::TRANSPARENT,
            text: Color::from_rgb8(26, 26, 26),
            primary: Color::from_rgb8(108, 85, 246),
            success: Color::from_rgb8(51, 179, 77),
            danger: Color::from_rgb8(217, 51, 51),
            warning: Color::from_rgb8(230, 153, 26),
        },
        ThemeMode::Dark => Palette {
            background: Color::TRANSPARENT,
            text: Color::WHITE,
            primary: Color::from_rgb8(108, 85, 246),
            success: Color::from_rgb8(77, 204, 77),
            danger: Color::from_rgb8(230, 77, 77),
            warning: Color::from_rgb8(230, 179, 51),
        },
    }
}

pub trait PastryTheme {
    fn text(&self) -> Color;
    fn primary(&self) -> Color;
    fn success(&self) -> Color;
    fn danger(&self) -> Color;
    fn page_background(&self) -> Color;
    fn dialog_background(&self) -> Color;
    fn card_background(&self) -> Color;
    fn card_code_background(&self) -> Color;
    fn card_code_background_hover(&self) -> Color;
    fn code_background(&self) -> Color;
    fn text_secondary(&self) -> Color;
    fn text_placeholder(&self) -> Color;
    fn input_background(&self) -> Color;
    fn input_border(&self) -> Color;
    fn button_background(&self) -> Color;
    fn button_hover_background(&self) -> Color;
    fn divider(&self) -> Color;
    fn shadow(&self) -> Color;
    // Workflow editor colors
    fn border_subtle(&self) -> Color;
    fn grid_dot(&self) -> Color;
    fn port_stroke(&self) -> Color;
    fn port_inner(&self) -> Color;
    fn edge_color(&self) -> Color;
    fn node_title(&self) -> Color;
    fn node_bg(&self) -> Color;
    fn edge_shadow(&self) -> Color;
    fn hover_bg(&self) -> Color;
}

impl PastryTheme for Theme {
    fn text(&self) -> Color {
        self.palette().text
    }

    fn primary(&self) -> Color {
        self.palette().primary
    }

    fn success(&self) -> Color {
        Color::from_rgb8(51, 179, 77)
    }

    fn danger(&self) -> Color {
        Color::from_rgb8(217, 51, 51)
    }

    fn page_background(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(38, 38, 38, 0.9)
        } else {
            Color::from_rgba8(250, 250, 250, 0.9)
        }
    }

    fn dialog_background(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(38, 38, 38, 1.0)
        } else {
            Color::from_rgba8(250, 250, 250, 1.0)
        }
    }

    fn card_background(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(51, 51, 51, 0.4)
        } else {
            Color::from_rgba8(255, 255, 255, 0.4)
        }
    }

    fn card_code_background(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(51, 51, 51, 0.6)
        } else {
            Color::from_rgba8(255, 255, 255, 0.6)
        }
    }

    fn card_code_background_hover(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(64, 64, 64, 0.9)
        } else {
            Color::from_rgba8(242, 242, 242, 0.9)
        }
    }

    fn code_background(&self) -> Color {
        Color::from_rgba8(26, 26, 26, 0.9)
    }

    fn text_secondary(&self) -> Color {
        if is_dark(self) {
            Color::from_rgb8(179, 179, 179)
        } else {
            Color::from_rgb8(102, 102, 102)
        }
    }

    fn text_placeholder(&self) -> Color {
        if is_dark(self) {
            Color::from_rgb8(128, 128, 128)
        } else {
            Color::from_rgb8(153, 153, 153)
        }
    }

    fn input_background(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(51, 51, 51, 0.4)
        } else {
            Color::from_rgba8(250, 250, 250, 0.4)
        }
    }

    fn input_border(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(77, 77, 77, 0.5)
        } else {
            Color::from_rgba8(204, 204, 204, 0.5)
        }
    }

    fn button_background(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(26, 26, 26, 0.5)
        } else {
            Color::from_rgba8(250, 250, 250, 0.5)
        }
    }

    fn button_hover_background(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(26, 26, 26, 0.4)
        } else {
            Color::from_rgba8(250, 250, 250, 0.4)
        }
    }

    fn divider(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(77, 77, 77, 0.3)
        } else {
            Color::from_rgba8(204, 204, 204, 0.3)
        }
    }

    fn shadow(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba8(0, 0, 0, 0.3)
        } else {
            Color::from_rgba8(0, 0, 0, 0.15)
        }
    }

    // Workflow editor colors
    fn border_subtle(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba(1.0, 1.0, 1.0, 0.08)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.08)
        }
    }

    fn grid_dot(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba(1.0, 1.0, 1.0, 0.1)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.1)
        }
    }

    fn port_stroke(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba(1.0, 1.0, 1.0, 0.4)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.4)
        }
    }

    fn port_inner(&self) -> Color {
        if is_dark(self) {
            Color::WHITE
        } else {
            Color::from_rgb(0.3, 0.3, 0.3)
        }
    }

    fn edge_color(&self) -> Color {
        if is_dark(self) {
            Color::WHITE
        } else {
            Color::from_rgb(0.3, 0.3, 0.3)
        }
    }

    fn node_title(&self) -> Color {
        if is_dark(self) {
            Color::WHITE
        } else {
            Color::from_rgb(0.1, 0.1, 0.1)
        }
    }

    fn node_bg(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba(0.12, 0.13, 0.16, 0.98)
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.95)
        }
    }

    fn edge_shadow(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba(0.0, 0.0, 0.0, 0.5)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.2)
        }
    }

    fn hover_bg(&self) -> Color {
        if is_dark(self) {
            Color::from_rgba(1.0, 1.0, 1.0, 0.05)
        } else {
            Color::from_rgba(0.0, 0.0, 0.0, 0.05)
        }
    }
}

pub fn is_dark(theme: &Theme) -> bool {
    theme.palette().text.r > 0.5
}
