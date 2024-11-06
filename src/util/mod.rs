use std::path::{Path, PathBuf};

use anyhow::Context;
use arboard::ImageData;
use image::{imageops::FilterType, GenericImageView, ImageFormat, RgbaImage};
use wgpu::core::command::Rect;

use crate::args::Verified;

// pub(crate) fn crop_and_save(
//     img: &RgbaImage,
//     args: Option<&Verified>,
//     rect: Rect<f32>,
//     output: impl AsRef<Path>,
// ) -> anyhow::Result<()> {
//     let img = crop_image(img, args, rect)?;
//     save_selection(img, args, output)
// }

pub(crate) fn crop_image(
    img: &RgbaImage,
    args: Option<&Verified>,
    selection: Rect<f32>,
) -> anyhow::Result<RgbaImage> {
    let rect = args.and_then(|a| a.region).unwrap_or(selection);
    // Round this to be smaller rather than larger
    let rect = Rect {
        x: rect.x.ceil() as u32,
        y: rect.y.ceil() as u32,
        w: rect.w.floor() as u32,
        h: rect.h.floor() as u32,
    };
    let img = img.view(rect.x, rect.y, rect.w, rect.h);
    Ok(img.to_image())
}

pub(crate) fn save_selection(
    mut image: RgbaImage,
    args: Option<&Verified>,
    save_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    // Handle scaling if requested
    if let Some(scale) = args.and_then(|a| a.scale) {
        image = resize_image(
            &image,
            scale,
            args.and_then(|a| a.filter).unwrap_or(FilterType::Nearest),
        )?;
    }

    // Generate filename and save
    let format = args
        .and_then(|f| f.image_format)
        .unwrap_or(ImageFormat::Png);
    let path = generate_output_path(
        save_path,
        args.and_then(|f| f.filename.as_deref()).unwrap_or("cleave"),
        format,
    );

    Ok(image.save_with_format(path, format)?)
}

pub(crate) fn resize_image(
    image: &RgbaImage,
    scale: f32,
    filter: FilterType,
) -> Result<RgbaImage, image::ImageError> {
    let new_width = (image.width() as f32 * scale).round() as u32;
    let new_height = (image.height() as f32 * scale).round() as u32;
    Ok(image::imageops::resize(
        image, new_width, new_height, filter,
    ))
}

pub(crate) fn generate_output_path(
    dir: impl AsRef<Path>,
    filename: &str,
    format: ImageFormat,
) -> PathBuf {
    let ext = format.extensions_str().first().copied().unwrap_or("png");

    let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S");
    let filename = format!("{filename}-{}.{ext}", timestamp);

    dir.as_ref().join(filename)
}

pub(crate) fn save_to_clipboard(image_data: &RgbaImage) -> Result<(), arboard::Error> {
    let mut clipboard = arboard::Clipboard::new()?;
    let image_data = ImageData {
        width: image_data.width() as usize,
        height: image_data.height() as usize,
        bytes: std::borrow::Cow::Borrowed(image_data.as_raw()),
    };
    if let Err(e) = clipboard.set_image(image_data) {
        eprintln!("Error setting image to clipboard: {:?}", e);
    };
    Ok(())
}

pub(crate) fn load_icon() -> Result<(u32, u32, Vec<u8>), anyhow::Error> {
    let icon_bytes = include_bytes!("../../icon.png");
    let rgba = image::load_from_memory(icon_bytes)?.to_rgba8();
    let (width, height) = rgba.dimensions();
    let rgba = rgba.into_raw();
    Ok((width, height, rgba))
}

pub(crate) fn capture_screen(monitor_id: Option<u32>) -> anyhow::Result<RgbaImage> {
    get_monitor(monitor_id).and_then(|e| Ok(e.capture_image()?))
}

pub(crate) fn get_monitor(monitor_id: Option<u32>) -> anyhow::Result<xcap::Monitor> {
    let mut monitors = xcap::Monitor::all()?;
    let monitor = monitors
        .iter()
        .position(|m| monitor_id.map_or(m.is_primary(), |id| m.id() == id))
        .or_else(|| monitors.iter().position(|m| m.is_primary()))
        .with_context(|| "Could not select monitor")?;
    Ok(monitors.swap_remove(monitor))
}
