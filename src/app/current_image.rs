use cleave_graphics::prelude::GraphicsBundle;
use glam::Vec2;
use image::RgbaImage;

use crate::selection::UserSelection;

use super::context::SelectionUniforms;

pub struct CurrentImage {
    pub image: RgbaImage,
    pub bundle: GraphicsBundle<SelectionUniforms>,
}

impl CurrentImage {
    pub fn capture_image(
        monitor: Option<u32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> anyhow::Result<Self> {
        let img = crate::util::capture_screen(monitor)?;
        let bundle = GraphicsBundle::new(
            img.clone().into(),
            device,
            queue,
            wgpu::PrimitiveTopology::TriangleStrip,
            format,
        );
        Ok(Self { image: img, bundle })
    }

    pub fn update_uniforms(&mut self, time: f32, user: &UserSelection, (w, h): (f32, f32)) {
        self.bundle.uniforms.time = time;

        // println!("{}", self.bundle.uniforms);
        self.bundle.uniforms.screen_size.x = w;
        self.bundle.uniforms.screen_size.y = h;

        let drag = &user.drag;
        let selection = &user.selection;
        self.bundle.uniforms.is_dragging = match (drag, selection) {
            (Some(d), Some(s)) if (d.x != 0. || d.y != 0.) && (s.x != 0. || s.y != 0.) => 3,
            (Some(d), None) if (d.x != 0. || d.y != 0.) => 1,
            (None, Some(s)) if (s.x != 0. || s.y != 0.) => 2,
            _ => 0,
        };
        if let Some(drag) = drag {
            self.bundle.uniforms.drag_start = Vec2::new(drag.x, drag.y);
            self.bundle.uniforms.drag_end = Vec2::new(drag.x + drag.w, drag.y + drag.h);
        } else {
            self.bundle.uniforms.drag_start = Vec2::ZERO;
            self.bundle.uniforms.drag_end = Vec2::ZERO;
        };

        if let Some(selection) = selection {
            self.bundle.uniforms.selection_start = Vec2::new(selection.x, selection.y);
            self.bundle.uniforms.selection_end =
                Vec2::new(selection.x + selection.w, selection.y + selection.h);
        } else {
            self.bundle.uniforms.selection_start = Vec2::ZERO;
            self.bundle.uniforms.selection_end = Vec2::ZERO;
        };
    }
}
