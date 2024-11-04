use std::path::PathBuf;

use image::ImageFormat;

use crate::context::SelectionMode;

#[derive(Debug, Copy, Clone)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

fn parse_region(s: &str) -> Result<Region, String> {
    let coords: Vec<u32> = s
        .split(',')
        .map(|s| s.parse().map_err(|_| "Invalid region format"))
        .collect::<Result<Vec<_>, _>>()?;

    if coords.len() != 4 {
        return Err("Region must be in format: x,y,width,height".into());
    }

    Ok(Region {
        x: coords[0],
        y: coords[1],
        width: coords[2],
        height: coords[3],
    })
}

fn parse_format(s: &str) -> Result<ImageFormat, String> {
    match s {
        "bmp" => Ok(ImageFormat::Bmp),
        "gif" => Ok(ImageFormat::Gif),
        "ico" => Ok(ImageFormat::Ico),
        "jpeg" => Ok(ImageFormat::Jpeg),
        "png" => Ok(ImageFormat::Png),
        "tiff" => Ok(ImageFormat::Tiff),
        "webp" => Ok(ImageFormat::WebP),
        _ => Err("Invalid image format".into()),
    }
}

fn parse_filter(s: &str) -> Result<image::imageops::FilterType, String> {
    match s {
        "Nearest" => Ok(image::imageops::FilterType::Nearest),
        "Triangle" => Ok(image::imageops::FilterType::Triangle),
        "CatmullRom" => Ok(image::imageops::FilterType::CatmullRom),
        "Gaussian" => Ok(image::imageops::FilterType::Gaussian),
        "Lanczos3" => Ok(image::imageops::FilterType::Lanczos3),
        _ => Err("Invalid filter type".into()),
    }
}

/// Cleave - A GPU-accelerated screen capture tool
#[derive(clap::Parser, Debug)]
#[command(
    name = "cleave",
    author,
    version,
    about,
    long_about = "A lightweight, GPU-accelerated screen capture tool that allows users to quickly select and copy portions of their screen"
)]
pub struct Args {
    /// Output directory for the captured image
    ///
    /// If not provided, the capture is copied to the clipboard
    #[arg(short, long)]
    pub output_dir: Option<PathBuf>,
    /// Output format for the captured image
    ///
    /// Supported formats: bmp, gif, ico, jpeg, png, tiff, webp
    ///
    /// Only used when output_dir is provided
    #[arg(long="format", value_parser=parse_format)]
    pub image_format: Option<ImageFormat>,
    /// Selection mode for the capture tool
    #[arg(short, long, default_value = "move")]
    pub mode: SelectionMode,
    /// Monitor index to capture
    ///
    /// If not provided, the primary monitor is used
    #[arg(long)]
    pub monitor: Option<usize>, // If not provided, the primary monitor is used
    /// Region to capture in the format: x,y,width,height
    ///
    /// If not provided, the entire screen is captured and the user is prompted to select a region
    /// If provided, the user is not prompted and the region is captured immediately
    #[arg(long, short='r', value_parser=parse_region)]
    pub region: Option<Region>,
    /// Filename for the captured image
    ///
    /// If not provided, the image is saved with a timestamp: 'cleave-YYYY-MM-DD-HH-MM-SS'
    /// Only used when output_dir is provided
    #[arg(long, short = 'f')]
    pub filename: Option<String>,
    /// Delay in milliseconds before capturing the screen
    ///
    /// If not provided, the screen is captured immediately
    #[arg(long, short = 'd', default_value = "0")]
    pub delay: u64,
    /// List available monitors and exit
    #[arg(long, short = 'l')]
    pub monitor_list: bool,
    // /// Path to the configuration file
    // ///
    // /// If not provided, the default configuration is used
    // #[arg(long, short = 'c')]
    // pub config_path: Option<PathBuf>,
    // TODO: Implement these features
    // /// Optimize the captured image when applicable
    // #[arg(long, short='p')]
    // optimize: bool,
    /// Scale the captured image by a factor
    #[arg(long, short = 's')]
    pub scale: Option<f32>,
    /// Filter to use when scaling the image
    ///
    /// Supported filters: Nearest, Triangle, CatmullRom, Gaussian, Lanczos3
    ///
    /// Only used when scale is provided
    #[arg(long, short = 'q', value_parser=parse_filter)]
    pub filter: Option<image::imageops::FilterType>,

    /// Daemon Mode Hotkey
    ///
    /// If provided, app runs in the background and captures the screen whenever the user presses a hotkey
    #[arg(long)]
    pub daemon_hotkey: Option<String>,
}

impl Args {
    pub fn verify(&self) -> Result<(), String> {
        if self.monitor_list
            && (self.output_dir.is_some()
                || self.image_format.is_some()
                || self.filename.is_some()
                || self.region.is_some()
                || self.scale.is_some()
                || self.daemon_hotkey.is_some())
        {
            return Err("Monitor list option cannot be used with other options".into());
        }
        if let Some(scale) = self.scale {
            if scale <= 0.0 {
                return Err("Scale factor must be greater than 0".into());
            }
        }
        if let Some(region) = self.region {
            if region.width == 0 || region.height == 0 {
                return Err("Region width and height must be greater than 0".into());
            }
        }
        if (self.image_format.is_some() || self.filename.is_some()) && self.output_dir.is_none() {
            return Err(
                "Output format and filename is only used when output directory is provided".into(),
            );
        }

        Ok(())
    }
}
