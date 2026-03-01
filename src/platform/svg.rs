pub fn render_svg_bytes_to_rgba(
    svg_data: &[u8],
    max_size: Option<u32>,
    background: Option<resvg::tiny_skia::Color>,
) -> Option<(Vec<u8>, u32, u32)> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_data, &opt).ok()?;

    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;

    if width == 0 || height == 0 {
        return None;
    }

    let scale = if let Some(max_size) = max_size
        && (width > max_size || height > max_size)
    {
        max_size as f32 / width.max(height) as f32
    } else {
        1.0
    };

    let scaled_width = (width as f32 * scale) as u32;
    let scaled_height = (height as f32 * scale) as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(scaled_width, scaled_height)?;
    pixmap.fill(background.unwrap_or(resvg::tiny_skia::Color::from_rgba8(0, 0, 0, 0)));

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let data = pixmap.take();
    Some((data, scaled_width, scaled_height))
}
