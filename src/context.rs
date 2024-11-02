use std::path::{Path, PathBuf};

use anyhow::Context;
use arboard::ImageData;
use glam::{DVec2, Vec2};
use image::{
    imageops::FilterType, GenericImageView, ImageBuffer, ImageError, ImageFormat, Rgba, RgbaImage,
};
// use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::PhysicalSize,
    window::{Icon, Window, WindowAttributes},
};

// use crate::{graphics_bundle::GraphicsBundle, graphics_impl::Graphics};
use cleave_graphics::prelude::*;

use crate::args::Args;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SelectionMode {
    Move,          // Move the selection
    InverseResize, // Make the selection smaller
    Resize,        // Make the selection larger
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Default, Debug)]
pub struct SelectionUniforms {
    screen_size: Vec2,
    drag_start: Vec2,
    drag_end: Vec2,
    selection_start: Vec2,
    selection_end: Vec2,
    time: f32,
    is_dragging: u32, // 0 = None, 1 = Dragging, 2 = Selected, 3 = Both
}

impl std::fmt::Display for SelectionUniforms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "size: {:?}, is_dragging: {}, drag_start: {:?}, drag_end: {:?}, selection_start: {:?}, selection_end: {:?}, time: {}", 
          self.screen_size, self.is_dragging, self.drag_start, self.drag_end, self.selection_start, self.selection_end, self.time)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Drag {
    start: Vec2,
    end: Option<Vec2>,
}

#[derive(Clone, Copy, Debug)]
pub struct Selection {
    start: Vec2,
    end: Vec2,
}

pub struct UserSelection {
    drag: Option<Drag>,
    selection: Option<Selection>,
}

impl UserSelection {
    fn new() -> Self {
        Self {
            drag: None,
            selection: None,
        }
    }

    fn sel_coords(&self) -> Option<((u32, u32), (u32, u32))> {
        let selection = self.selection.as_ref()?;
        let (start_x, start_y) = (selection.start.x, selection.start.y);
        let (end_x, end_y) = (selection.end.x, selection.end.y);

        let (min_x, max_x) = (start_x.min(end_x).ceil(), start_x.max(end_x).floor());
        let (min_y, max_y) = (start_y.min(end_y).ceil(), start_y.max(end_y).floor());
        Some(((min_x as u32, min_y as u32), (max_x as u32, max_y as u32)))
    }

    // fn sel_dimensions(&self) -> Option<(f32, f32)> {
    //     let selection = self.selection.as_ref()?;
    //     let width = (selection.end.x - selection.start.x).abs();
    //     let height = (selection.end.y - selection.start.y).abs();
    //     Some((width, height))
    // }

    // fn get_
}

fn resize_image(
    image: &RgbaImage,
    scale: f32,
    filter: FilterType,
) -> Result<RgbaImage, ImageError> {
    let new_width = (image.width() as f32 * scale).round() as u32;
    let new_height = (image.height() as f32 * scale).round() as u32;
    Ok(image::imageops::resize(
        image, new_width, new_height, filter,
    ))
}

fn generate_output_path(dir: &Path, filename: &str, format: ImageFormat) -> PathBuf {
    let ext = format.extensions_str().first().copied().unwrap_or("png");

    let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S");
    let filename = format!("{filename}-{}.{ext}", timestamp);

    dir.join(filename)
}

pub struct AppContext {
    mouse_position: DVec2,
    selection: UserSelection,
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    total_time: f32,
    last_frame: std::time::Instant,
    graphics: Graphics<Window>,
    bundle: GraphicsBundle<SelectionUniforms>,
    mode: SelectionMode,
}

impl AppContext {
    pub fn start_drag(&mut self) {
        if let Some(drag) = self.selection.drag.as_mut() {
            if drag.start != Vec2::ZERO {
                return;
            }
        };
        self.selection.drag = Some(Drag {
            start: self.mouse_position.as_vec2(),
            end: Some(self.mouse_position.as_vec2()),
        });
    }

    pub fn end_drag(&mut self) {
        self.selection.selection = None;
        if let Some(drag) = self.selection.drag.take() {
            let end_pos = drag.end.unwrap_or(drag.start); // Use end if set, otherwise use start
            self.selection.selection = Some(Selection {
                start: drag.start,
                end: end_pos,
            });
        }
    }

    pub fn cancel_drag(&mut self) {
        self.selection.drag = None;
        self.selection.selection = None;
    }

    fn get_selection_data(&self, ax: u32, ay: u32, bx: u32, by: u32) -> RgbaImage {
        let img = self.image.view(ax, ay, bx.abs_diff(ax), by.abs_diff(ay));
        img.to_image()
    }

    fn save_to_clipboard(&self, image_data: RgbaImage) {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        let image_data = ImageData {
            width: image_data.width() as usize,
            height: image_data.height() as usize,
            bytes: std::borrow::Cow::Owned(image_data.to_vec()),
        };
        let _ = clipboard.set_image(image_data);
    }

    pub fn save_selection(&self, args: Option<&Args>) -> anyhow::Result<()> {
        // Get dimensions and validate image data
        let ((ax, ay), (bx, by)) = if let Some(region) = args.and_then(|a| a.region) {
            (
                (region.x, region.y),
                ((region.x + region.width), (region.y + region.height)),
            )
        } else {
            self.selection
                .sel_coords()
                .ok_or_else(|| anyhow::anyhow!("No selection made"))?
        };

        let mut image_data = self.get_selection_data(ax, ay, bx, by);

        // Handle scaling if requested
        if let Some(scale) = args.and_then(|a| a.scale) {
            image_data = resize_image(
                &image_data,
                scale,
                args.and_then(|a| a.filter).unwrap_or(FilterType::Nearest),
            )?;
        }

        // Save to clipboard if no output directory specified
        let Some(output_dir) = args.and_then(|f| f.output_dir.as_deref()) else {
            return {
                self.save_to_clipboard(image_data);
                Ok(())
            };
        };

        // Generate filename and save
        let format = args
            .and_then(|f| f.image_format)
            .unwrap_or(ImageFormat::Png);
        let path = generate_output_path(
            output_dir,
            args.and_then(|f| f.filename.as_deref()).unwrap_or("cleave"),
            format,
        );

        Ok(image_data.save_with_format(path, format)?)
    }

    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<Self> {
        let monitor = xcap::Monitor::all()?
            .into_iter()
            .find(|m| m.is_primary())
            .with_context(|| "Could not get primary monitor")?;
        let img = monitor.capture_image()?;
        let size = PhysicalSize::new(monitor.width(), monitor.height());

        let icon_bytes = include_bytes!("../icon.png");
        let rgba = image::load_from_memory(icon_bytes)?.to_rgba8();
        let (width, height) = rgba.dimensions();
        let rgba = rgba.into_raw();

        let window = event_loop.create_window(
            WindowAttributes::default()
                .with_inner_size(size)
                .with_title("Cleave")
                .with_resizable(false)
                .with_decorations(false)
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
                .with_visible(false)
                .with_window_icon(Some(Icon::from_rgba(rgba, width, height)?)),
        )?;

        let graphics = Graphics::new(window, size.width, size.height);
        let graphics = pollster::block_on(graphics)?;

        let bundle = GraphicsBundle::new(
            img.clone().into(),
            &graphics.device,
            &graphics.queue,
            wgpu::PrimitiveTopology::TriangleStrip,
            graphics.config.format,
        );

        graphics.window.set_visible(true);
        let _ = graphics
            .window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined);

        // let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        // let pixels = Pixels::new(size.width, size.height, surface_texture)?;

        Ok(Self {
            image: img,
            bundle,
            total_time: 0.0,
            last_frame: std::time::Instant::now(),
            selection: UserSelection::new(),
            // window,
            graphics,
            mouse_position: DVec2::new(0.0, 0.0),
            mode: SelectionMode::Resize,
        })
    }

    pub fn handle_move(&mut self, dir: Direction) -> Option<()> {
        let (dx, dy) = match dir {
            Direction::Up => (0.0, -1.0),
            Direction::Down => (0.0, 1.0),
            Direction::Left => (-1.0, 0.0),
            Direction::Right => (1.0, 0.0),
        };

        let selection = self.selection.selection.as_mut()?;

        match self.mode {
            SelectionMode::Move => {
                selection.start.x = (selection.start.x + dx).clamp(0.0, self.image.width() as f32);
                selection.start.y = (selection.start.y + dy).clamp(0.0, self.image.height() as f32);
                selection.end.x = (selection.end.x + dx).clamp(0.0, self.image.width() as f32);
                selection.end.y = (selection.end.y + dy).clamp(0.0, self.image.height() as f32);
            }
            SelectionMode::Resize => {
                selection.end.x = (selection.end.x + dx).clamp(0.0, self.image.width() as f32);
                selection.end.y = (selection.end.y + dy).clamp(0.0, self.image.height() as f32);
            }
            SelectionMode::InverseResize => {
                selection.start.x = (selection.start.x + dx).clamp(0.0, self.image.width() as f32);
                selection.start.y = (selection.start.y + dy).clamp(0.0, self.image.height() as f32);
            }
        }

        Some(())
    }

    pub fn draw(&mut self) {
        let time = self.last_frame.elapsed().as_secs_f32();
        self.total_time += time;
        self.last_frame = std::time::Instant::now();

        self.update_uniforms();
        self.bundle.update_buffer(&self.graphics.queue);

        let mut pass = match self.graphics.render() {
            Ok(pass) => pass,
            Err(err) => {
                eprintln!("Error rendering frame: {:?}", err);
                return;
            }
        };
        self.bundle.draw(&mut pass);
        pass.finish();
        self.graphics.request_redraw();
    }

    fn update_uniforms(&mut self) {
        self.bundle.uniforms.time = self.total_time;
        self.bundle.uniforms.screen_size.x = self.image.width() as f32;
        self.bundle.uniforms.screen_size.y = self.image.height() as f32;

        let drag = self.selection.drag;
        let selection = self.selection.selection;
        self.bundle.uniforms.is_dragging = match (drag, selection) {
            (Some(d), Some(s)) if d.start != Vec2::ZERO || s.start != Vec2::ZERO => 3,
            (Some(d), None) if d.start != Vec2::ZERO => 1,
            (None, Some(s)) if s.start != Vec2::ZERO => 2,
            _ => 0,
        };

        if let Some(drag) = drag {
            self.bundle.uniforms.drag_start = drag.start;
            self.bundle.uniforms.drag_end = drag.end.unwrap_or_default();
        } else {
            self.bundle.uniforms.drag_start = Vec2::ZERO;
            self.bundle.uniforms.drag_end = Vec2::ZERO;
        };

        if let Some(selection) = selection {
            self.bundle.uniforms.selection_start = selection.start;
            self.bundle.uniforms.selection_end = selection.end;
        } else {
            self.bundle.uniforms.selection_start = Vec2::ZERO;
            self.bundle.uniforms.selection_end = Vec2::ZERO;
        };
    }

    pub fn window_id(&self) -> winit::window::WindowId {
        self.graphics.id()
    }

    pub fn destroy(&self) {
        self.graphics.window.set_minimized(true);
    }

    pub fn hide_window(&self) {
        self.graphics.set_visible(false);
    }

    pub fn set_mode(&mut self, mode: SelectionMode) {
        self.mode = mode
    }

    pub fn update_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_position = DVec2::new(x, y);
        if let Some(drag) = self.selection.drag.as_mut() {
            drag.end = Some(self.mouse_position.as_vec2());
        }
    }

    pub fn set_args(mut self, args: &crate::args::Args) -> Option<Self> {
        // self.bundle.uniforms.screen_size.x = args.width as f32;
        // self.bundle.uniforms.screen_size.y = args.height as f32;
        if let Some(region) = args.region {
            self.selection.selection = Some(Selection {
                start: Vec2::new(region.x as f32, region.y as f32),
                end: Vec2::new(
                    (region.x + region.width) as f32,
                    (region.y + region.height) as f32,
                ),
            });
            if let Err(e) = self.save_selection(Some(args)) {
                eprintln!("Error saving selection: {:?}", e);
            };
            return None;
        }

        Some(self)
    }
}
