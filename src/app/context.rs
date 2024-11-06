use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use image::GenericImageView;
// use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::PhysicalSize,
    window::{Icon, Window, WindowAttributes},
};

// use crate::{graphics_bundle::GraphicsBundle, graphics_impl::Graphics};
use cleave_graphics::prelude::*;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Default, Debug)]
pub struct SelectionUniforms {
    pub screen_size: Vec2,
    pub drag_start: Vec2,
    pub drag_end: Vec2,
    pub selection_start: Vec2,
    pub selection_end: Vec2,
    pub time: f32,
    pub is_dragging: u32, // 0 = None, 1 = Dragging, 2 = Selected, 3 = Both
}

impl std::fmt::Display for SelectionUniforms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "size: {:?}, is_dragging: {}, drag_start: {:?}, drag_end: {:?}, selection_start: {:?}, selection_end: {:?}, time: {}", 
          self.screen_size, self.is_dragging, self.drag_start, self.drag_end, self.selection_start, self.selection_end, self.time)
    }
}

pub struct CleaveContext {
    pub graphics: Graphics<Window>,
    pub total_time: f32,
    last_frame: std::time::Instant,
}

impl CleaveContext {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {
        let (ico_width, ico_height, rgba) = crate::util::load_icon()?;
        let window = event_loop.create_window(
            WindowAttributes::default()
                .with_inner_size(PhysicalSize::new(width, height))
                .with_title("Cleave")
                .with_resizable(false)
                .with_decorations(false)
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
                .with_visible(false)
                .with_window_icon(Some(Icon::from_rgba(rgba, ico_width, ico_height)?)),
        )?;

        let graphics = Graphics::new(window, width, height);
        let graphics = pollster::block_on(graphics)?;

        graphics
            .window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .or_else(|_| {
                graphics
                    .window
                    .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            })?;

        Ok(Self {
            total_time: 0.0,
            last_frame: std::time::Instant::now(),
            graphics,
        })
    }

    pub fn draw<U: Pod + Zeroable + Copy + Clone + Default>(
        &mut self,
        bundle: Option<&GraphicsBundle<U>>,
    ) {
        let mut pass = match self.graphics.render() {
            Ok(pass) => pass,
            Err(err) => {
                eprintln!("Error rendering frame: {:?}", err);
                return;
            }
        };
        if let Some(bundle) = bundle {
            bundle.draw(&mut pass);
        }
        pass.finish();
        self.graphics.request_redraw();
    }

    pub fn update(&mut self) {
        let time = self.last_frame.elapsed().as_secs_f32();
        self.total_time += time;
        self.last_frame = std::time::Instant::now();
    }

    pub fn window_id(&self) -> winit::window::WindowId {
        self.graphics.id()
    }

    pub fn destroy(&self) {
        self.graphics.window.set_minimized(true);
    }

    pub fn set_window_visibility(&self, val: bool) {
        self.graphics.set_visible(val);
    }

    pub fn size(&self) -> (f32, f32) {
        let size = self.graphics.window.outer_size();
        (size.width as f32, size.height as f32)
    }
}
