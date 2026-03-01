use crate::ui::util::color::Color;

#[derive(Debug, Clone)]
pub enum Message {
    CloseColorPicker,
    ColorPickerSubmitted(Color),
}
