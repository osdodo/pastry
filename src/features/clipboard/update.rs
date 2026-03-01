use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

use base64::Engine;
use iced::Task;
use image::imageops::FilterType;

use super::{
    message::Message,
    model::{CardMessage, CardState, ClipType, ImageFormat},
    state::State,
};
use crate::{
    services::{
        clipboard::{self as monitor, ClipboardContent, save_favorites, set_clipboard_image},
        images::{compress_image, save_compressed_image, save_original_image},
    },
    ui::{constants::MAX_HISTORY_SIZE, language},
    web::{ClipboardEntry, update_clipboard},
};

fn persist_favorites(state: &State) {
    let cards: Vec<_> = state
        .history
        .iter()
        .filter(|c| c.is_favorite)
        .map(|c| c.to_favorite_data())
        .collect();
    let _ = save_favorites(&cards);
}

fn sync_text_to_web(text: String) {
    tokio::spawn(async move {
        update_clipboard(ClipboardEntry {
            content: text,
            timestamp: chrono::Local::now(),
            clip_type: "text".to_string(),
            image_data_url: None,
            image_width: None,
            image_height: None,
        })
        .await;
    });
}

const MAX_SYNC_IMAGE_EDGE: u32 = 1600;

fn fit_with_max_edge(width: u32, height: u32, max_edge: u32) -> (u32, u32) {
    if width == 0 || height == 0 {
        return (width, height);
    }

    let current_max = width.max(height);
    if current_max <= max_edge {
        return (width, height);
    }

    let scale = max_edge as f32 / current_max as f32;
    let target_width = (width as f32 * scale).round().max(1.0) as u32;
    let target_height = (height as f32 * scale).round().max(1.0) as u32;

    (target_width, target_height)
}

fn encode_image_as_data_url(
    data: &[u8],
    width: usize,
    height: usize,
) -> Option<(String, usize, usize)> {
    let image = image::RgbaImage::from_raw(width as u32, height as u32, data.to_vec())?;
    let dynamic_image = image::DynamicImage::ImageRgba8(image);

    let (target_width, target_height) = fit_with_max_edge(
        dynamic_image.width(),
        dynamic_image.height(),
        MAX_SYNC_IMAGE_EDGE,
    );
    let resized =
        if target_width != dynamic_image.width() || target_height != dynamic_image.height() {
            dynamic_image.resize(target_width, target_height, FilterType::Triangle)
        } else {
            dynamic_image
        };

    let rgb = resized.to_rgb8();
    let mut encoded = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded, 82);
    encoder
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ColorType::Rgb8.into(),
        )
        .ok()?;
    let base64 = base64::engine::general_purpose::STANDARD.encode(&encoded);
    Some((
        format!("data:image/jpeg;base64,{base64}"),
        rgb.width() as usize,
        rgb.height() as usize,
    ))
}

fn sync_image_to_web(data: Vec<u8>, width: usize, height: usize, hash: u64) {
    tokio::spawn(async move {
        let Some((image_data_url, image_width, image_height)) =
            encode_image_as_data_url(&data, width, height)
        else {
            return;
        };

        update_clipboard(ClipboardEntry {
            content: format!("image:{hash}"),
            timestamp: chrono::Local::now(),
            clip_type: "image".to_string(),
            image_data_url: Some(image_data_url),
            image_width: Some(image_width),
            image_height: Some(image_height),
        })
        .await;
    });
}

fn compute_text_hash(text: &str) -> (u64, ClipType) {
    let clip_type = if text.contains('<') && text.contains('>') {
        ClipType::RichText
    } else {
        ClipType::PlainText
    };

    let tag: u8 = match clip_type {
        ClipType::PlainText => 0,
        ClipType::RichText => 1,
        ClipType::Image => 2,
        ClipType::File => 3,
    };

    let mut hasher = DefaultHasher::new();
    tag.hash(&mut hasher);
    text.hash(&mut hasher);
    (hasher.finish(), clip_type)
}

fn compute_image_hash(data: &[u8], width: usize, height: usize) -> u64 {
    let tag: u8 = 2;
    let mut hasher = DefaultHasher::new();
    tag.hash(&mut hasher);
    data.hash(&mut hasher);
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    hasher.finish()
}

fn clipboard_content_hash(content: &ClipboardContent) -> u64 {
    match content {
        ClipboardContent::Text(text) => compute_text_hash(text).0,
        ClipboardContent::Image(data, width, height, _)
        | ClipboardContent::ImageFile(_, data, width, height, _) => {
            compute_image_hash(data, *width, *height)
        }
    }
}

fn image_source_format_to_image_format(format: monitor::ImageSourceFormat) -> ImageFormat {
    match format {
        monitor::ImageSourceFormat::Png => ImageFormat::Png,
        monitor::ImageSourceFormat::Jpeg => ImageFormat::Jpeg,
        monitor::ImageSourceFormat::Svg => ImageFormat::Svg,
        monitor::ImageSourceFormat::Other => ImageFormat::Other,
    }
}

fn history_contains_hash(state: &State, hash: u64) -> bool {
    state.history.iter().any(|entry| entry.content_hash == hash)
}

fn build_text_entry(state: &State, text: String) -> (Option<CardState>, u64) {
    let (hash, _clip_type) = compute_text_hash(&text);
    if history_contains_hash(state, hash) {
        return (None, hash);
    }

    sync_text_to_web(text.clone());
    (Some(CardState::new_with_hash(text, hash)), hash)
}

fn build_image_entry(
    state: &State,
    data: Vec<u8>,
    width: usize,
    height: usize,
    source_format: monitor::ImageSourceFormat,
    file_path: Option<String>,
) -> (Option<CardState>, u64) {
    let hash = compute_image_hash(&data, width, height);
    if history_contains_hash(state, hash) {
        return (None, hash);
    }

    sync_image_to_web(data.clone(), width, height, hash);
    let img_format = image_source_format_to_image_format(source_format);
    let entry =
        CardState::new_image_with_path_with_hash(data, width, height, img_format, file_path, hash);
    (Some(entry), hash)
}

fn push_history_entry(state: &mut State, entry: CardState) {
    state.history.insert(0, entry);
    if state.history.len() > MAX_HISTORY_SIZE {
        state.history.pop();
    }
}

fn task_set_system_clipboard(
    content: String,
    clip_type: ClipType,
    image_data: Option<(Arc<Vec<u8>>, usize, usize)>,
    target_hash: u64,
) -> Task<Message> {
    Task::perform(
        async move {
            let mut clipboard = arboard::Clipboard::new().ok()?;
            if clip_type == ClipType::Image
                && let Some((data, width, height)) = image_data
            {
                let img_data = arboard::ImageData {
                    width,
                    height,
                    bytes: std::borrow::Cow::Owned(data.as_ref().clone()),
                };
                clipboard.set_image(img_data).ok()?;
            } else {
                clipboard.set_text(content).ok()?;
            }
            Some(target_hash)
        },
        |hash_opt| {
            hash_opt
                .map(Message::SetClipboardHash)
                .unwrap_or(Message::None)
        },
    )
}

fn task_clear_system_clipboard() -> Task<Message> {
    Task::perform(
        async move {
            let mut clipboard = arboard::Clipboard::new().ok()?;
            clipboard.set_text("".to_string()).ok()?;
            Some(0)
        },
        |hash_opt| {
            hash_opt
                .map(Message::SetClipboardHash)
                .unwrap_or(Message::None)
        },
    )
}

fn task_sync_system_clipboard_with_top(state: &mut State) -> Task<Message> {
    if let Some(new_top) = state.history.first() {
        return task_set_system_clipboard(
            new_top.content.clone(),
            new_top.clip_type,
            new_top.image_data.clone(),
            new_top.content_hash,
        );
    }

    state.last_clipboard_hash = 0;
    task_clear_system_clipboard()
}

fn handle_removed_entry(
    state: &mut State,
    removed: CardState,
    removed_index: usize,
) -> Task<Message> {
    if removed.is_favorite {
        persist_favorites(state);
    }

    if let Some(path) = &removed.saved_image_path {
        let _ = std::fs::remove_file(path);
    }

    if removed_index == 0 {
        return task_sync_system_clipboard_with_top(state);
    }

    if removed.content_hash == state.last_clipboard_hash {
        state.last_clipboard_hash = 0;
    }

    Task::none()
}

fn is_cached_image_path(path: &str) -> bool {
    let Some(config_dir) = dirs::config_dir() else {
        return false;
    };
    let images_dir = config_dir.join("pastry").join("images");
    Path::new(path).starts_with(images_dir)
}

fn apply_card_message(state: &mut CardState, message: &CardMessage) {
    match message {
        CardMessage::ToggleFavorite => {
            state.is_favorite = !state.is_favorite;
        }
        CardMessage::Copy => {
            state.is_copied = true;
        }
        CardMessage::ResetCopyIcon => {
            state.is_copied = false;
        }
        CardMessage::ResetScriptOutputCopyIcon => {
            state.script_output_copied = false;
        }
        CardMessage::DeleteScriptOutput => {
            state.script_output = None;
            state.script_output_copied = false;
        }
        CardMessage::ShowDeleteConfirm
        | CardMessage::RunScript
        | CardMessage::RunWorkflow
        | CardMessage::CopyScriptOutput
        | CardMessage::CompressImage
        | CardMessage::ShowJsonFormat
        | CardMessage::ToggleColorPicker(_) => {}
    }
}

pub fn update(state: &mut State, msg: Message) -> Task<Message> {
    match msg {
        Message::ExternalCard(index, inner) => {
            if let Some(entry) = state.history.get_mut(index) {
                // Intercept ToggleFavorite to show confirmation if currently favorited
                if let CardMessage::ToggleFavorite = inner
                    && entry.is_favorite
                {
                    state.unfavorite_confirm_index = Some(index);
                    return Task::none();
                }

                apply_card_message(entry, &inner);
                match inner {
                    CardMessage::ToggleFavorite => {
                        // Extract needed info then release mutable borrow
                        let is_favorite = entry.is_favorite;
                        let saved_path_opt = entry.saved_image_path.clone();
                        let saved_path_opt2 = saved_path_opt.clone();
                        let is_image = matches!(entry.clip_type, ClipType::Image);
                        let image_data_opt = entry.image_data.clone();
                        let image_format = entry.image_format.unwrap_or(ImageFormat::Png);
                        let file_path = entry.file_path.clone();
                        let idx = index;
                        let _ = entry;

                        persist_favorites(state);

                        // Delete image file when unfavoriting
                        if !is_favorite && let Some(path) = saved_path_opt {
                            let _ = std::fs::remove_file(path);
                            if let Some(entry) = state.history.get_mut(idx) {
                                entry.saved_image_path = None;
                            }
                        }

                        // Save original image when favoriting
                        let already_cached = saved_path_opt2
                            .as_deref()
                            .map(is_cached_image_path)
                            .unwrap_or(false);

                        if is_favorite && is_image && !already_cached {
                            let Some((data, width, height)) = image_data_opt else {
                                return Task::none();
                            };
                            let data = data.as_ref().clone();
                            let source_path = file_path
                                .as_ref()
                                .filter(|p| Path::new(p).exists())
                                .cloned()
                                .or(saved_path_opt2);
                            return Task::perform(
                                async move {
                                    save_original_image(
                                        data,
                                        width,
                                        height,
                                        image_format,
                                        source_path,
                                    )
                                    .await
                                    .map(|path| (idx, path))
                                },
                                |result| {
                                    if let Some((index, path)) = result {
                                        Message::ImageSaved(index, path)
                                    } else {
                                        Message::ClearCompress
                                    }
                                },
                            );
                        }
                    }
                    CardMessage::CompressImage => {
                        if let Some((data, width, height)) = entry.image_data.clone() {
                            let format = entry.image_format;
                            let source_path = entry
                                .file_path
                                .clone()
                                .or_else(|| entry.saved_image_path.clone());
                            let idx = index;

                            state.compressing = true;
                            state.compress_message =
                                Some(language::tr(language::Text::Compressing).to_string());

                            return Task::perform(
                                async move {
                                    let data = data.as_ref().clone();
                                    if let Some((c_data, c_width, c_height, c_format)) =
                                        compress_image(
                                            data,
                                            width,
                                            height,
                                            format,
                                            source_path.clone(),
                                        )
                                        .await
                                    {
                                        save_compressed_image(
                                            c_data,
                                            c_width,
                                            c_height,
                                            c_format,
                                            source_path,
                                        )
                                        .await
                                        .map(|path| (idx, path))
                                    } else {
                                        None
                                    }
                                },
                                Message::CompressComplete,
                            );
                        }
                    }
                    CardMessage::ShowDeleteConfirm => {
                        if entry.is_favorite {
                            state.delete_confirm_index = Some(index);
                        } else {
                            let removed = state.history.remove(index);
                            return handle_removed_entry(state, removed, index);
                        }
                    }
                    CardMessage::Copy => {
                        let task = if let Some((data, width, height)) = entry.image_data.as_ref() {
                            let data = Arc::clone(data);
                            let width = *width;
                            let height = *height;
                            Task::perform(
                                async move {
                                    set_clipboard_image(data.as_slice(), width, height).ok()?;
                                    Some(())
                                },
                                |_| Message::None,
                            )
                        } else {
                            let content = entry.content.clone();
                            Task::perform(
                                async move {
                                    let mut clipboard = arboard::Clipboard::new().ok()?;
                                    clipboard.set_text(content).ok()?;
                                    Some(())
                                },
                                |_| Message::None,
                            )
                        };
                        // Reset copy icon after 3 seconds
                        let reset_task = Task::perform(
                            async {
                                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                            },
                            move |_| Message::ExternalCard(index, CardMessage::ResetCopyIcon),
                        );
                        return Task::batch(vec![task, reset_task]);
                    }
                    CardMessage::ResetCopyIcon => {}
                    CardMessage::ResetScriptOutputCopyIcon => {}
                    CardMessage::RunScript => {}
                    CardMessage::RunWorkflow => {}
                    CardMessage::CopyScriptOutput => {
                        if let Some(output) = &entry.script_output {
                            let output_text = output.clone();
                            entry.script_output_copied = true;
                            let mut hasher = DefaultHasher::new();
                            output_text.hash(&mut hasher);
                            let hash = hasher.finish();
                            let text_to_copy = output_text.clone();
                            let copy_task = Task::perform(
                                async move {
                                    let mut clipboard = arboard::Clipboard::new().ok()?;
                                    clipboard.set_text(text_to_copy.clone()).ok()?;
                                    Some(())
                                },
                                |_| Message::None,
                            );
                            let hash_task = Task::done(Message::SetClipboardHash(hash));
                            let reset_task = Task::perform(
                                async {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                },
                                move |_| {
                                    Message::ExternalCard(
                                        index,
                                        CardMessage::ResetScriptOutputCopyIcon,
                                    )
                                },
                            );
                            return Task::batch(vec![copy_task, hash_task, reset_task]);
                        }
                    }
                    CardMessage::DeleteScriptOutput => {
                        if entry.is_favorite {
                            persist_favorites(state);
                        }
                    }
                    CardMessage::ShowJsonFormat => {}
                    CardMessage::ToggleColorPicker(_) => {}
                }
            }
            Task::none()
        }
        Message::ConfirmDelete => {
            if let Some(index) = state.delete_confirm_index.take() {
                let removed = state.history.remove(index);
                return handle_removed_entry(state, removed, index);
            }
            Task::none()
        }
        Message::CancelDelete => {
            state.delete_confirm_index = None;
            Task::none()
        }
        Message::ConfirmUnfavorite => {
            if let Some(index) = state.unfavorite_confirm_index {
                if let Some(entry) = state.history.get_mut(index) {
                    // Perform the unfavorite action
                    entry.is_favorite = false;

                    let saved_path_opt = entry.saved_image_path.clone();

                    // Delete image file logic
                    if let Some(path) = saved_path_opt {
                        let _ = std::fs::remove_file(path);
                        entry.saved_image_path = None;
                    }
                }

                // Persist changes
                persist_favorites(state);
                state.unfavorite_confirm_index = None;
            }
            Task::none()
        }
        Message::CancelUnfavorite => {
            state.unfavorite_confirm_index = None;
            Task::none()
        }
        Message::SearchChanged(text) => {
            state.search_text = text;
            Task::none()
        }
        Message::FilterChanged(filter) => {
            state.filter = filter;
            Task::none()
        }
        Message::Poll => {
            if monitor::should_check_clipboard()
                && let Some(content) = monitor::get_clipboard_content()
            {
                let hash = clipboard_content_hash(&content);
                if hash != state.last_clipboard_hash {
                    return Task::done(Message::ClipboardChanged(content));
                }
            }
            Task::none()
        }
        Message::ClipboardChanged(content) => {
            let (entry_opt, hash) = match content {
                ClipboardContent::Text(text) => build_text_entry(state, text),
                ClipboardContent::Image(data, width, height, format) => {
                    build_image_entry(state, data, width, height, format, None)
                }
                ClipboardContent::ImageFile(file_path, data, width, height, format) => {
                    build_image_entry(state, data, width, height, format, Some(file_path))
                }
            };

            if let Some(entry) = entry_opt {
                push_history_entry(state, entry);
            }
            state.last_clipboard_hash = hash;
            Task::none()
        }
        Message::SetClipboardHash(hash) => {
            state.last_clipboard_hash = hash;
            Task::none()
        }
        Message::ImageSaved(index, path) => {
            if let Some(entry) = state.history.get_mut(index) {
                entry.saved_image_path = Some(path.clone());
            }
            persist_favorites(state);
            Task::none()
        }
        Message::CompressComplete(result) => {
            state.compressing = false;
            match result {
                Some((index, path)) => {
                    let label = language::tr(language::Text::SavedTo);
                    state.compress_message = Some(format!("{} {}", label, path));
                    if let Some(entry) = state.history.get_mut(index)
                        && !entry.is_favorite
                    {
                        entry.saved_image_path = Some(path);
                    }
                    persist_favorites(state);
                }
                None => {
                    state.compress_message =
                        Some(language::tr(language::Text::CompressFailed).to_string());
                }
            }
            Task::perform(
                async {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                },
                |_| Message::ClearCompress,
            )
        }
        Message::ClearCompress => {
            state.compress_message = None;
            Task::none()
        }
        Message::None => Task::none(),
    }
}
