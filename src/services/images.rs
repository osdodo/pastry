use std::fs;
use std::path::Path;

use image::{ImageFormat as ImgFormat, RgbaImage};

use crate::domain::clipboard as clipboard_model;

pub async fn compress_image(
    data: Vec<u8>,
    width: usize,
    height: usize,
    format: Option<clipboard_model::ImageFormat>,
    file_path: Option<String>,
) -> Option<(Vec<u8>, usize, usize, clipboard_model::ImageFormat)> {
    let format = format.unwrap_or(clipboard_model::ImageFormat::Png);

    match format {
        clipboard_model::ImageFormat::Svg => compress_svg(data, width, height, file_path).await,
        clipboard_model::ImageFormat::Png => compress_png(data, width, height).await,
        clipboard_model::ImageFormat::Jpeg | clipboard_model::ImageFormat::Other => {
            compress_jpeg(data, width, height).await
        }
    }
}

async fn compress_svg(
    data: Vec<u8>,
    width: usize,
    height: usize,
    file_path: Option<String>,
) -> Option<(Vec<u8>, usize, usize, clipboard_model::ImageFormat)> {
    if let Some(path) = file_path
        && let Ok(svg_content) = fs::read_to_string(&path)
    {
        let compressed = compress_svg_content(&svg_content);
        return Some((
            compressed.into_bytes(),
            width,
            height,
            clipboard_model::ImageFormat::Svg,
        ));
    }

    let img = RgbaImage::from_raw(width as u32, height as u32, data)?;
    let mut compressed_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut compressed_bytes);
    img.write_to(&mut cursor, ImgFormat::Png).ok()?;

    let compressed_img =
        image::load_from_memory_with_format(&compressed_bytes, ImgFormat::Png).ok()?;
    let rgba_img = compressed_img.to_rgba8();
    Some((
        rgba_img.into_raw(),
        width,
        height,
        clipboard_model::ImageFormat::Png,
    ))
}

async fn compress_png(
    data: Vec<u8>,
    width: usize,
    height: usize,
) -> Option<(Vec<u8>, usize, usize, clipboard_model::ImageFormat)> {
    let img = RgbaImage::from_raw(width as u32, height as u32, data)?;
    let rgb_img = image::DynamicImage::ImageRgba8(img).to_rgb8();

    let mut compressed_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut compressed_bytes);

    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 85);
    if encoder
        .encode(
            rgb_img.as_raw(),
            width as u32,
            height as u32,
            image::ColorType::Rgb8.into(),
        )
        .is_err()
    {
        return None;
    }

    let compressed_img =
        image::load_from_memory_with_format(&compressed_bytes, ImgFormat::Jpeg).ok()?;
    let rgba_img = compressed_img.to_rgba8();
    Some((
        rgba_img.into_raw(),
        width,
        height,
        clipboard_model::ImageFormat::Jpeg,
    ))
}

async fn compress_jpeg(
    data: Vec<u8>,
    width: usize,
    height: usize,
) -> Option<(Vec<u8>, usize, usize, clipboard_model::ImageFormat)> {
    let img = RgbaImage::from_raw(width as u32, height as u32, data)?;
    let scale = 0.7;
    let new_width = ((width as f32) * scale) as u32;
    let new_height = ((height as f32) * scale) as u32;

    let resized = image::imageops::resize(
        &img,
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,
    );

    let rgb_img = image::DynamicImage::ImageRgba8(resized).to_rgb8();
    let mut compressed_bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut compressed_bytes);

    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 85);
    if encoder
        .encode(
            rgb_img.as_raw(),
            new_width,
            new_height,
            image::ColorType::Rgb8.into(),
        )
        .is_err()
    {
        return None;
    }

    let compressed_img =
        image::load_from_memory_with_format(&compressed_bytes, ImgFormat::Jpeg).ok()?;
    let rgba_img = compressed_img.to_rgba8();
    Some((
        rgba_img.into_raw(),
        new_width as usize,
        new_height as usize,
        clipboard_model::ImageFormat::Jpeg,
    ))
}

fn compress_svg_content(svg: &str) -> String {
    let mut result = String::with_capacity(svg.len());
    let mut in_tag = false;
    let mut in_quotes = false;
    let mut in_doctype = false;
    let mut quote_char = ' ';
    let mut last_char_was_space = false;
    let mut i = 0;
    let bytes = svg.as_bytes();

    fn peek_next_nonspace(bytes: &[u8], mut j: usize) -> Option<char> {
        while j < bytes.len() {
            let c = bytes[j] as char;
            if !c.is_whitespace() {
                return Some(c);
            }
            j += 1;
        }
        None
    }

    while i < bytes.len() {
        let ch = bytes[i] as char;

        if !in_quotes
            && i + 3 < bytes.len()
            && bytes[i] == b'<'
            && bytes[i + 1] == b'!'
            && bytes[i + 2] == b'-'
            && bytes[i + 3] == b'-'
        {
            i += 4;
            while i + 2 < bytes.len() {
                if bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'>' {
                    i += 3;
                    break;
                }
                i += 1;
            }
            continue;
        }

        if !in_quotes
            && i + 8 < bytes.len()
            && bytes[i] == b'<'
            && bytes[i + 1] == b'!'
            && (bytes[i + 2] == b'D' || bytes[i + 2] == b'd')
            && (bytes[i + 3] == b'O' || bytes[i + 3] == b'o')
            && (bytes[i + 4] == b'C' || bytes[i + 4] == b'c')
            && (bytes[i + 5] == b'T' || bytes[i + 5] == b't')
            && (bytes[i + 6] == b'Y' || bytes[i + 6] == b'y')
            && (bytes[i + 7] == b'P' || bytes[i + 7] == b'p')
            && (bytes[i + 8] == b'E' || bytes[i + 8] == b'e')
        {
            in_doctype = true;
            in_tag = true;
            result.push(ch);
            i += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            if !in_quotes {
                in_quotes = true;
                quote_char = ch;
                result.push(ch);
            } else if ch == quote_char {
                in_quotes = false;
                result.push(ch);
            } else {
                result.push(ch);
            }
            last_char_was_space = false;
            i += 1;
            continue;
        }

        if in_quotes {
            result.push(ch);
            i += 1;
            continue;
        }

        if ch == '<' {
            in_tag = true;
            result.push(ch);
            last_char_was_space = false;
        } else if ch == '>' {
            in_tag = false;
            in_doctype = false;
            result.push(ch);
            last_char_was_space = false;
        } else if ch.is_whitespace() {
            if in_doctype {
                result.push(ch);
            } else if in_tag {
                let next = peek_next_nonspace(bytes, i + 1);
                let prev = result.chars().rev().find(|c| !c.is_whitespace());
                let skip = matches!(prev, Some('<') | Some('/') | Some('='))
                    || matches!(
                        next,
                        Some('>') | Some('/') | Some('=') | Some('"') | Some('\'')
                    );
                if !skip && !last_char_was_space {
                    result.push(' ');
                    last_char_was_space = true;
                }
            }
        } else {
            result.push(ch);
            last_char_was_space = false;
        }

        i += 1;
    }

    result
}

pub async fn save_compressed_image(
    data: Vec<u8>,
    width: usize,
    height: usize,
    format: clipboard_model::ImageFormat,
    original_file_path: Option<String>,
) -> Option<String> {
    let extension = match format {
        clipboard_model::ImageFormat::Png => "png",
        clipboard_model::ImageFormat::Jpeg => "jpg",
        clipboard_model::ImageFormat::Svg => "svg",
        clipboard_model::ImageFormat::Other => "jpg",
    };

    let (save_dir, filename) = if let Some(ref original_path) = original_file_path {
        let path = Path::new(original_path);
        let dir = if format == clipboard_model::ImageFormat::Svg {
            None
        } else {
            path.parent().map(|p| p.to_path_buf())
        };
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("compressed");
        let filename = format!("{}_compressed.{}", stem, extension);
        (dir, filename)
    } else {
        use chrono::Local;
        let now = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        let filename = format!("compressed_{}.{}", timestamp, extension);
        (None, filename)
    };

    let save_dir = save_dir.or_else(dirs::download_dir)?;
    let file_path = save_dir.join(filename);

    match format {
        clipboard_model::ImageFormat::Svg => {
            let svg_text = String::from_utf8(data).ok()?;
            fs::write(&file_path, svg_text).ok()?;
        }
        clipboard_model::ImageFormat::Png => {
            let img = image::RgbaImage::from_raw(width as u32, height as u32, data)?;
            img.save_with_format(&file_path, image::ImageFormat::Png)
                .ok()?;
        }
        clipboard_model::ImageFormat::Jpeg | clipboard_model::ImageFormat::Other => {
            let img = image::RgbaImage::from_raw(width as u32, height as u32, data)?;
            let rgb_img = image::DynamicImage::ImageRgba8(img).to_rgb8();
            rgb_img
                .save_with_format(&file_path, image::ImageFormat::Jpeg)
                .ok()?;
        }
    }

    Some(file_path.to_string_lossy().to_string())
}

pub async fn save_original_image(
    data: Vec<u8>,
    width: usize,
    height: usize,
    format: clipboard_model::ImageFormat,
    original_file_path: Option<String>,
) -> Option<String> {
    let images_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("pastry")
        .join("images");

    if fs::create_dir_all(&images_dir).is_err() {
        return None;
    }

    if format == clipboard_model::ImageFormat::Svg {
        if let Some(ref original_path) = original_file_path {
            let source_path = Path::new(original_path);
            if source_path.exists() {
                let filename = source_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("image.svg");
                let dest_path = images_dir.join(filename);

                fs::copy(source_path, &dest_path).ok()?;
                return Some(dest_path.to_string_lossy().to_string());
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    let extension = match format {
        clipboard_model::ImageFormat::Png => "png",
        clipboard_model::ImageFormat::Jpeg => "jpg",
        clipboard_model::ImageFormat::Svg => "svg",
        clipboard_model::ImageFormat::Other => "png",
    };

    let filename = if let Some(ref original_path) = original_file_path {
        let path = Path::new(original_path);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
        format!("{}.{}", stem, extension)
    } else {
        use chrono::Local;
        let now = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        format!("image_{}.{}", timestamp, extension)
    };

    let file_path = images_dir.join(filename);

    match format {
        clipboard_model::ImageFormat::Png => {
            let img = image::RgbaImage::from_raw(width as u32, height as u32, data)?;
            img.save_with_format(&file_path, image::ImageFormat::Png)
                .ok()?;
        }
        clipboard_model::ImageFormat::Jpeg | clipboard_model::ImageFormat::Other => {
            let img = image::RgbaImage::from_raw(width as u32, height as u32, data)?;
            let rgb_img = image::DynamicImage::ImageRgba8(img).to_rgb8();
            rgb_img
                .save_with_format(&file_path, image::ImageFormat::Jpeg)
                .ok()?;
        }
        clipboard_model::ImageFormat::Svg => {
            return None;
        }
    }

    Some(file_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_svg_content() {
        let input = r#"
<svg width="100" height="100">
  <!-- This is a comment -->
  <g>
    <rect x="10" y="10" width="30" height="30" />
  </g>
</svg>
"#;
        let compressed = compress_svg_content(input);

        assert!(!compressed.contains("<!--"));
        assert!(!compressed.contains('\n'));
        assert!(!compressed.contains("  "));
        assert!(compressed.contains("><g>"));
        assert!(compressed.contains("><rect"));

        let input2 = "< g > < / g >";
        let compressed2 = compress_svg_content(input2);
        assert_eq!(compressed2, "<g></g>");

        let input3 = "<rect  x = \"10\" />";
        let compressed3 = compress_svg_content(input3);
        assert_eq!(compressed3, "<rect x=\"10\"/>");
    }
}
