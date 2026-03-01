use iced::Task;

use crate::app;

use super::message::Message;

pub fn update(state: &mut app::State, message: Message) -> Task<app::Message> {
    match message {
        Message::CloseColorPicker => {
            state.show_color_picker = false;
            Task::none()
        }
        Message::ColorPickerSubmitted(_color) => {
            state.show_color_picker = false;
            Task::none()
        }
    }
}
