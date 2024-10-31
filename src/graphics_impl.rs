use anyhow::{bail, Result};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use wgpu::{
    rwh::{HasDisplayHandle, HasWindowHandle},
    Device, InstanceDescriptor, Operations, Queue, RenderPassColorAttachment, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureView,
};
use winit::dpi::PhysicalSize;

// use crate::DrawCommand;

pub struct Graphics<W> {
    pub device: Device,
    // pipeline: RenderPipeline,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    // pub font_handler: FontHandler,
    pub window: Arc<W>,
}

impl<W> Deref for Graphics<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

pub struct GraphicsOutput {
    output: SurfaceTexture,
    pub view: TextureView,
}

impl GraphicsOutput {
    pub fn finish(self) {
        self.output.present();
    }
}

impl<W> Graphics<W>
where
    W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
{
    pub async fn new(window: W, size: PhysicalSize<u32>) -> Result<Self> {
        let window = Arc::new(window);
        // Create a surface from the window.
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        // Create a surface from the window.
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await;
        let Some(adapter) = adapter else {
            bail!("No adapter found");
        };
        let config = find_config(&surface, &adapter, size);
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits {
                        // max_buffer_size: 786_432_000,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
            )
            .await?;
        surface.configure(&device, &config);
        // let font_handler = FontHandler::new(&window, &device, &queue, config.format);

        Ok(Graphics {
            device,
            queue,
            config,
            size,
            surface,
            window,
            // font_handler,
        })
    }

    fn output(&self) -> Option<GraphicsOutput> {
        let Ok(output) = self.surface.get_current_texture() else {
            println!("No output available");
            self.surface.configure(&self.device, &self.config);
            return None;
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Some(GraphicsOutput { output, view })
    }

    // pub fn resize(&mut self, size: PhysicalSize<u32>) {
    //     if size.width == 0 || size.height == 0 {
    //         return;
    //     }
    //     self.config.width = size.width;
    //     self.config.height = size.height;
    //     self.size = size;
    //     self.surface.configure(&self.device, &self.config);
    // }

    // pub fn render<'a>(&mut self, tbd: impl IntoIterator<Item: DrawCommand>) -> Result<()> {
    //     let mut encoder = self
    //         .device
    //         .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    //     let Some(output) = self.output() else {
    //         bail!("No output available");
    //     };

    //     {
    //         let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
    //             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    //                 view: &output.view,
    //                 resolve_target: None,
    //                 ops: wgpu::Operations {
    //                     load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
    //                     store: wgpu::StoreOp::Store,
    //                 },
    //             })],
    //             ..Default::default()
    //         });
    //         // pass.execute_bundles(tbd);
    //         tbd.into_iter().for_each(|d| d.draw(&mut pass));
    //     }
    //     self.queue.submit(Some(encoder.finish()));
    //     output.finish();
    //     Ok(())
    // }

    pub fn render(&mut self) -> Result<GraphicsPass<W>> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let Some(output) = self.output() else {
            // bail!("No output available");
            println!("No output available");
            return self.render();
        };
        let pass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &output.view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            })
            .forget_lifetime();
        Ok(GraphicsPass {
            graphics: self,
            encoder: Some(encoder),
            output: Some(output),
            pass,
        })
    }
}

pub struct GraphicsPass<'g, 'p, W> {
    graphics: &'g Graphics<W>,
    encoder: Option<wgpu::CommandEncoder>,
    output: Option<GraphicsOutput>,
    pass: wgpu::RenderPass<'p>,
}

impl<'p, W> Deref for GraphicsPass<'_, 'p, W> {
    type Target = wgpu::RenderPass<'p>;
    fn deref(&self) -> &Self::Target {
        &self.pass
    }
}

impl<W> DerefMut for GraphicsPass<'_, '_, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pass
    }
}

impl<W> GraphicsPass<'_, '_, W> {
    pub fn finish(mut self) {
        drop(self.pass);
        let Some(encoder) = self.encoder.take() else {
            return;
        };
        self.graphics.queue.submit(Some(encoder.finish()));
        if let Some(f) = self.output.take() {
            f.finish()
        }
    }
}

fn find_config(
    surface: &Surface,
    adapter: &wgpu::Adapter,
    size: PhysicalSize<u32>,
) -> SurfaceConfiguration {
    let surface_config = surface.get_capabilities(adapter);
    let format = surface_config
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .unwrap_or(&surface_config.formats[0]);

    SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: *format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Immediate,
        desired_maximum_frame_latency: 1,
        alpha_mode: surface_config.alpha_modes[0],
        view_formats: vec![],
    }
}
