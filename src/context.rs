use anyhow::Context;
use arboard::ImageData;
use glam::Vec2;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};
// use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::PhysicalSize,
    window::{Icon, Window, WindowAttributes},
};

use crate::{graphics_bundle::GraphicsBundle, graphics_impl::Graphics, Drag, Selection};

pub enum MoveMode {
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

pub struct AppContext {
    size: PhysicalSize<u32>,
    mouse_position: (f64, f64),
    current_drag: Option<Drag>,
    img: ImageBuffer<Rgb<u8>, Vec<u8>>,
    // pixels: Pixels<'static>,
    total_time: f32,
    last_frame: std::time::Instant,
    graphics: Graphics<Window>,
    bundle: GraphicsBundle,
    selection: Option<Selection>,
    mode: MoveMode,
}

impl AppContext {
    pub fn start_drag(&mut self) {
        if self.current_drag.is_some() {
            return;
        }
        self.current_drag = Some(Drag {
            start: self.mouse_position,
            end: None,
        });
    }

    pub fn end_drag(&mut self) {
        if let Some(drag) = self.current_drag.take() {
            self.selection = Some(Selection {
                start: drag.start,
                end: self.mouse_position,
            });
        }
    }

    pub fn cancel_drag(&mut self) {
        self.current_drag = None;
        self.selection = None;
    }

    pub fn update_mouse_position(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.mouse_position = (position.x, position.y);
        if let Some(drag) = &mut self.current_drag {
            drag.end = Some(self.mouse_position);
        }
    }

    pub fn save_selection_to_clipboard(&self) {
        let Some(selection) = &self.selection else {
            return;
        };

        let ((min_x, min_y), (max_x, max_y)) = selection.coords();

        let (width, height) = selection.dimensions();
        let width = width as usize;
        let height = height as usize;

        let mut image_data = Vec::new();
        for y in min_y..max_y {
            let row_start = (y * self.size.width + min_x) as usize * 4;
            let row_end = (y * self.size.width + max_x) as usize * 4;
            let row = &self.img.as_raw()[row_start..row_end];
            image_data.extend_from_slice(row);
        }

        let mut clipboard = arboard::Clipboard::new().unwrap();
        if selection.area() as usize != image_data.len() / 4 {
            eprintln!(
                "Invalid selection size {:?} (w h p)",
                (width, height, image_data.len() / 4)
            );
            return;
        }
        let image_data = ImageData {
            width,
            height,
            bytes: std::borrow::Cow::Owned(image_data),
        };
        let _ = clipboard.set_image(image_data);
    }

    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<Self> {
        let monitor = xcap::Monitor::all()?
            .into_iter()
            .find(|m| m.is_primary())
            .with_context(|| "Could not get primary monitor")?;
        let img = DynamicImage::ImageRgba8(monitor.capture_image()?).to_rgb8();
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

        let graphics = Graphics::new(window, size);
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
            size,
            img,
            bundle,
            total_time: 0.0,
            last_frame: std::time::Instant::now(),
            current_drag: None,
            // window,
            graphics,
            mouse_position: (0.0, 0.0),
            selection: None,
            mode: MoveMode::Resize,
        })
    }

    pub fn handle_move(&mut self, dir: Direction) {
        let Some(selection) = &mut self.selection else {
            return;
        };

        let (dx, dy) = match dir {
            Direction::Up => (0.0, -1.0),
            Direction::Down => (0.0, 1.0),
            Direction::Left => (-1.0, 0.0),
            Direction::Right => (1.0, 0.0),
        };

        match self.mode {
            MoveMode::Move => {
                selection.start.0 = (selection.start.0 + dx).clamp(0.0, self.size.width as f64);
                selection.start.1 = (selection.start.1 + dy).clamp(0.0, self.size.height as f64);
                selection.end.0 = (selection.end.0 + dx).clamp(0.0, self.size.width as f64);
                selection.end.1 = (selection.end.1 + dy).clamp(0.0, self.size.height as f64);
            }
            MoveMode::Resize => {
                selection.end.0 = (selection.end.0 + dx).clamp(0.0, self.size.width as f64);
                selection.end.1 = (selection.end.1 + dy).clamp(0.0, self.size.height as f64);
            }
            MoveMode::InverseResize => {
                selection.start.0 = (selection.start.0 + dx).clamp(0.0, self.size.width as f64);
                selection.start.1 = (selection.start.1 + dy).clamp(0.0, self.size.height as f64);
            }
        }
    }

    pub fn draw(&mut self) {
        let time = self.last_frame.elapsed().as_secs_f32();
        self.total_time += time;
        self.last_frame = std::time::Instant::now();
        self.bundle.update_selection(
            self.total_time,
            &self.graphics.queue,
            self.current_drag.as_ref(),
            self.selection.as_ref(),
            Vec2::new(self.size.width as f32, self.size.height as f32),
        );

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

    pub fn window_id(&self) -> winit::window::WindowId {
        self.graphics.id()
    }

    pub fn destroy(&self) {
        self.graphics.window.set_minimized(true);
    }

    pub fn hide_window(&self) {
        self.graphics.set_visible(false);
    }

    pub fn set_mode(&mut self, mode: MoveMode) {
        self.mode = mode
    }
}

fn draw_rectangle_outline(
    img_width: u32,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
    frame: &mut [u8],
) {
    for x in min_x..=max_x {
        let top_index = (min_y * img_width + x) as usize * 4;
        let bottom_index = (max_y * img_width + x) as usize * 4;
        frame[top_index..top_index + 4].copy_from_slice(&[255, 0, 0, 255]); // Red color
        frame[bottom_index..bottom_index + 4].copy_from_slice(&[255, 0, 0, 255]);
    }
    for y in min_y..=max_y {
        let left_index = (y * img_width + min_x) as usize * 4;
        let right_index = (y * img_width + max_x) as usize * 4;
        frame[left_index..left_index + 4].copy_from_slice(&[255, 0, 0, 255]);
        frame[right_index..right_index + 4].copy_from_slice(&[255, 0, 0, 255]);
    }
}
