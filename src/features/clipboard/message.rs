use crate::{
    features::clipboard::{model::CardMessage, state::Filter},
    services::clipboard::ClipboardContent,
};

#[derive(Debug, Clone)]
pub enum Message {
    ExternalCard(usize, CardMessage),
    SearchChanged(String),
    FilterChanged(Filter),
    ConfirmDelete,
    CancelDelete,
    ConfirmUnfavorite,
    CancelUnfavorite,
    Poll,
    ClipboardChanged(ClipboardContent),
    ImageSaved(usize, String),
    CompressComplete(Option<(usize, String)>),
    ClearCompress,
    SetClipboardHash(u64),
    None,
}
