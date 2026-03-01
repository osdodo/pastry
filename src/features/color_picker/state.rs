use crate::ui::util::color::Color;

#[derive(Debug, Clone)]
pub struct State {
    pub color: Color,
}

impl State {
    pub fn new(color: Color) -> Self {
        Self { color }
    }

    pub fn update_color(&mut self, color: Color) {
        self.color = color;
    }
}
