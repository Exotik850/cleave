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

#[derive(clap::Parser, Debug)]
#[command(version, about, author, long_about=None)]
pub struct Args {
  #[arg(short, long, default_value = "screenshot.png")]
  output_dir: PathBuf,
  #[arg(value_parser=parse_format)]
  image_format: ImageFormat,
  #[arg(short, long, default_value = "move")]
  mode: SelectionMode,
  #[arg(long)]
  monitor: Option<usize>, // If not provided, the primary monitor is used
  #[arg(long, value_parser=parse_region)]
  region: Option<Region>,
  #[arg(long, short='f')]
  filename: Option<String>,
  #[arg(long, short='b')]
  clipboard: bool,
  #[arg(long, short='d')]
  delay: u64,
  #[arg(long, short='l')]
  monitor_list: bool,
  #[arg(long, short='c')]
  config_path: Option<PathBuf>,
  #[arg(long, short='p')]
  optimize: bool,
  #[arg(long, short='s')]
  scale: f32,
  #[arg(long, short='n')]
  notify: bool,
}