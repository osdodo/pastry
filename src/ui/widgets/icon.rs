use iced::Color;
use iced::widget::Svg;
use iced::widget::svg;
use iced::widget::svg::Handle;

#[derive(Debug, Clone, Copy)]
pub enum Icon {
    Add,
    Back,
    Close,
    Code,
    CodeBrackets,
    Compression,
    Copy,
    Copied,
    Delete,
    Editor,
    Pin,
    Star,
    StarOne,
    FileWrite,
    Flow,
    Command,
    Clipboard,
}

pub fn icon_handle(icon: Icon) -> Handle {
    match icon {
        Icon::Add => Handle::from_memory(include_bytes!("../../../assets/add.svg").as_slice()),
        Icon::Back => Handle::from_memory(include_bytes!("../../../assets/back.svg").as_slice()),
        Icon::Close => Handle::from_memory(include_bytes!("../../../assets/close.svg").as_slice()),
        Icon::Code => Handle::from_memory(include_bytes!("../../../assets/code.svg").as_slice()),
        Icon::CodeBrackets => {
            Handle::from_memory(include_bytes!("../../../assets/code-brackets.svg").as_slice())
        }
        Icon::Compression => {
            Handle::from_memory(include_bytes!("../../../assets/compression.svg").as_slice())
        }
        Icon::Copy => Handle::from_memory(include_bytes!("../../../assets/copy.svg").as_slice()),
        Icon::Copied => {
            Handle::from_memory(include_bytes!("../../../assets/copied.svg").as_slice())
        }
        Icon::Delete => {
            Handle::from_memory(include_bytes!("../../../assets/delete.svg").as_slice())
        }
        Icon::Editor => Handle::from_memory(include_bytes!("../../../assets/edit.svg").as_slice()),
        Icon::Pin => Handle::from_memory(include_bytes!("../../../assets/pin.svg").as_slice()),
        Icon::Star => Handle::from_memory(include_bytes!("../../../assets/star.svg").as_slice()),
        Icon::StarOne => {
            Handle::from_memory(include_bytes!("../../../assets/star-one.svg").as_slice())
        }
        Icon::FileWrite => {
            Handle::from_memory(include_bytes!("../../../assets/file-write.svg").as_slice())
        }
        Icon::Flow => Handle::from_memory(include_bytes!("../../../assets/flow.svg").as_slice()),
        Icon::Command => {
            Handle::from_memory(include_bytes!("../../../assets/command.svg").as_slice())
        }
        Icon::Clipboard => {
            Handle::from_memory(include_bytes!("../../../assets/clipboard.svg").as_slice())
        }
    }
}

pub fn icon_svg<F>(icon: Icon, size: u32, color: F) -> Svg<'static>
where
    F: Fn(&iced::Theme) -> Color + 'static,
{
    svg(icon_handle(icon))
        .width(size)
        .height(size)
        .style(move |theme, _status| svg::Style {
            color: Some(color(theme)),
        })
}
