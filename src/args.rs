use std::path::PathBuf;

use image::ImageFormat;

use crate::context::SelectionMode;

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

pub struct Args {
  output_dir: PathBuf,
  image_format: ImageFormat,
  mode: SelectionMode,
  monitor: Option<usize>,
  region: Option<(u32, u32, u32, u32)>,
  filename: Option<String>,
  clipboard: bool,
  delay: u64,
  monitor_list: bool,
  config_path: Option<PathBuf>,
  optimize: bool,
  scale: f32,
  notify: bool,

}