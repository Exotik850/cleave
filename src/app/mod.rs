
use crate::{
    args::{Args, Verified},
    selection::modes::{Direction, SelectionMode},
};

use current_image::CurrentImage;
use state::CleaveState;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
};

mod context;
mod current_image;
mod state;

use context::CleaveContext;

pub struct App {
    args: Option<Verified>,
    context: Option<CleaveContext>,
    state: CleaveState,
    current_image: Option<CurrentImage>,
}

impl App {
    pub fn new(args: Option<Args>) -> anyhow::Result<Self> {
        Ok(App {
            args: args.map(Args::verify).transpose()?,
            context: None,
            state: Default::default(),
            current_image: None,
        })
    }

    fn start_loop(&mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::new()?;
        Ok(event_loop.run_app(self)?)
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let Some(args) = &self.args else {
            return self.start_loop();
        };

        if args.monitor_list {
            println!("Available monitors:");
            for monitor in xcap::Monitor::all().into_iter().flatten() {
                println!("ID: {}", monitor.id());
            }
            std::process::exit(0);
        }

        if args.delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(args.delay));
        }

        if let Some(output_dir) = &args.output_dir {
            std::fs::create_dir_all(output_dir)?;
        }

        if let Some(region) = args.region {
            let img = crate::util::capture_screen(args.monitor)?;
            let cropped = crate::util::crop_image(&img, Some(args), region)?;
            if let Some(out) = &args.output_dir {
                crate::util::save_selection(cropped, Some(args), out)?;
            } else {
                crate::util::save_to_clipboard(&cropped)?;
            }
            return Ok(());
        }

        self.start_loop()
    }

    fn execute_key_command(
        &mut self,
        event: KeyEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> bool {
        let Some(context) = &mut self.context else {
            return false;
        };
        let KeyEvent {
            logical_key: Key::Named(key),
            state: pressed,
            ..
        } = event
        else {
            return false;
        };
        match (pressed, key) {
            (ElementState::Pressed, NamedKey::Escape) => {
                event_loop.exit();
                context.destroy();
            }
            (ElementState::Pressed, NamedKey::Space) => {
                let Some(c_img) = self.current_image.take() else {
                    eprintln!("No image to crop");
                    return false;
                };
                let Some(rect) = self.state.selection.selection else {
                    eprintln!("No selection to crop");
                    return false;
                };
                let Ok(cropped) = crate::util::crop_image(&c_img.image, self.args.as_ref(), rect)
                else {
                    eprintln!("Could not crop image");
                    return false;
                };
                match self.args.as_ref().and_then(|a| a.output_dir.as_ref()) {
                    Some(path) => {
                        if let Err(e) =
                            crate::util::save_selection(cropped, self.args.as_ref(), path)
                        {
                            eprintln!("{}", e);
                        };
                    }
                    None => {
                        // Save to clipboard
                        if let Err(e) = crate::util::save_to_clipboard(&cropped) {
                            eprintln!("{}", e);
                        };
                    }
                }
                event_loop.exit();
            }
            (ElementState::Pressed, NamedKey::ArrowDown) => {
                self.state.handle_move(Direction::Down);
            }
            (ElementState::Pressed, NamedKey::ArrowUp) => {
                self.state.handle_move(Direction::Up);
            }
            (ElementState::Pressed, NamedKey::ArrowLeft) => {
                self.state.handle_move(Direction::Left);
            }
            (ElementState::Pressed, NamedKey::ArrowRight) => {
                self.state.handle_move(Direction::Right);
            }
            (ElementState::Pressed, NamedKey::Shift) => {
                self.state.set_mode(SelectionMode::InverseResize);
            }
            (ElementState::Released, NamedKey::Shift | NamedKey::Control) => {
                self.state.set_mode(SelectionMode::Move);
            }
            (ElementState::Pressed, NamedKey::Control) => {
                self.state.set_mode(SelectionMode::Resize);
            }
            _ => {}
        }
        true
    }

    fn handle_input(
        &mut self,
        event: &WindowEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        self.state.handle_event(event);
        if let WindowEvent::KeyboardInput { event, .. } = event {
            self.execute_key_command(event.clone(), event_loop);
        }
    }

    fn capture_image(&mut self) {
        let Some(context) = &self.context else {
            return;
        };
        let mut current_image = CurrentImage::capture_image(
            self.args.as_ref().and_then(|a| a.monitor),
            &context.graphics.device,
            &context.graphics.queue,
            context.graphics.config.format,
        )
        .expect("Could not capture image");
        let (w, h) = current_image.image.dimensions();
        let (w, h) = (w as f32, h as f32);
        current_image.update_uniforms(context.total_time, &self.state.selection, (w, h));
        current_image.bundle.update_buffer(&context.graphics.queue);
        context.set_window_visibility(true);
        self.current_image = Some(current_image);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.context.is_some() {
            return;
        }
        let size = crate::util::get_monitor(self.args.as_ref().and_then(|a| a.monitor))
            .expect("Could not find monitor!");
        let context = CleaveContext::new(event_loop, size.width(), size.height())
            .expect("Could not start context");
        self.context = Some(context);
        self.capture_image();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.handle_input(&event, event_loop);
        if let Some(context) = &self.context {
            if !context.graphics.is_visible().unwrap_or(true) && self.current_image.is_none() {
                self.capture_image();
            }
        }
        match event {
            WindowEvent::RedrawRequested => {
                let Some(context) = &mut self.context else {
                    return;
                };

                if id != context.window_id() {
                    return;
                }
                context.update();
                let bund = self.current_image.as_mut().map(|c_img| {
                    c_img.update_uniforms(
                        context.total_time,
                        &self.state.selection,
                        context.size(),
                    );
                    c_img.bundle.update_buffer(&context.graphics.queue);
                    &c_img.bundle
                });
                context.draw(bund);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}
