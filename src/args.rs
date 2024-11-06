use std::path::PathBuf;

use image::ImageFormat;
use wgpu::core::command::Rect;

use global_hotkey::hotkey::HotKey;
use crate::selection::modes::SelectionMode;

fn parse_region(s: &str) -> Result<Rect<f32>, String> {
    let coords: Vec<f32> = s
        .split(',')
        .map(|s| s.parse().map_err(|_| "Invalid region format"))
        .collect::<Result<Vec<_>, _>>()?;

    if coords.len() != 4 {
        return Err("Region must be in format: x,y,width,height".into());
    }

    Ok(Rect {
        x: coords[0],
        y: coords[1],
        w: coords[2],
        h: coords[3],
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
    pub monitor: Option<u32>, // If not provided, the primary monitor is used
    /// Region to capture in the format: x,y,width,height
    ///
    /// If not provided, the entire screen is captured and the user is prompted to select a region
    /// If provided, the user is not prompted and the region is captured immediately
    #[arg(long, short='r', value_parser=parse_region)]
    pub region: Option<Rect<f32>>,
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

    /// Persistent Daemon Mode
    ///
    /// If true, the app will continue to run in the background even after the hotkey is pressed,
    /// allowing the user to capture the screen multiple times
    ///
    /// Only used when daemon_hotkey is provided
    #[arg(long, short)]
    pub persistent: bool,
}

impl Args {
    pub fn verify(self) -> anyhow::Result<Verified> {
        if self.monitor_list
            && (self.output_dir.is_some()
                || self.image_format.is_some()
                || self.filename.is_some()
                || self.region.is_some()
                || self.scale.is_some()
                || self.daemon_hotkey.is_some())
        {
            anyhow::bail!("Monitor list option cannot be used with other options");
        }
        if let Some(scale) = self.scale {
            if scale <= 0.0 {
                anyhow::bail!("Scale factor must be greater than 0");
            }
        }
        if let Some(region) = self.region {
            if region.w == 0. || region.h == 0. {
                anyhow::bail!("Region width and height must be greater than 0");
            }
        }
        if (self.image_format.is_some() || self.filename.is_some()) && self.output_dir.is_none() {
            anyhow::bail!(
                "Output format and filename is only used when output directory is provided"
            );
        }
        if self.persistent && self.daemon_hotkey.is_none() {
            anyhow::bail!("Persistent daemon mode can only be used with daemon hotkey");
        }
        if self.daemon_hotkey.is_some() && self.delay > 0 {
            anyhow::bail!("Delay cannot be used with daemon hotkey");
        }
        if let Some(hotkey) = &self.daemon_hotkey {
            if hotkey.is_empty() {
                anyhow::bail!("Hotkey cannot be empty");
            }
        }

        let daemon_hotkey = self.daemon_hotkey.map(|s| s.parse()).transpose()?;

        Ok(Verified {
            output_dir: self.output_dir,
            image_format: self.image_format,
            mode: self.mode,
            monitor: self.monitor,
            region: self.region,
            filename: self.filename,
            delay: self.delay,
            monitor_list: self.monitor_list,
            config_path: None,
            scale: self.scale,
            filter: self.filter,
            daemon_hotkey,
            persistent: self.persistent,
        })
    }
}

pub struct Verified {
    pub output_dir: Option<PathBuf>,
    pub image_format: Option<ImageFormat>,
    pub mode: SelectionMode,
    pub monitor: Option<u32>,
    pub region: Option<Rect<f32>>,
    pub filename: Option<String>,
    pub delay: u64,
    pub monitor_list: bool,
    pub config_path: Option<PathBuf>,
    pub scale: Option<f32>,
    pub filter: Option<image::imageops::FilterType>,
    pub daemon_hotkey: Option<HotKey>,
    pub persistent: bool,
}

impl Verified {
    pub fn stay_running(&self) -> bool {
        self.daemon_hotkey.is_some() && self.persistent
    }
}
