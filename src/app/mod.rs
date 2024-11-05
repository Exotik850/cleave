use crate::{
    args::{Args, Verified},
    selection::modes::{Direction, SelectionMode},
};

use current_image::CurrentImage;
use state::CleaveState;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
};

mod context;
mod current_image;
mod state;

use context::CleaveContext;

pub struct App {
    args: Option<Verified>,
    current_image: Option<CurrentImage>,
    context: Option<CleaveContext>,
    state: CleaveState,
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
        let stay_running = self.args.as_ref().is_some_and(|d| d.stay_running());
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
                if !stay_running {
                    event_loop.exit();
                    context.destroy();
                }
            }
            (ElementState::Pressed, NamedKey::Space) => {
                let Some(c_img) = self.current_image.take() else {
                    return false;
                };
                let Some(rect) = self.state.selection.selection else {
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
                if !stay_running {
                    event_loop.exit();
                }
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
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.context.is_some() {
            return;
        }
        let context = CleaveContext::new(event_loop).expect("Could not start context");
        if self
            .args
            .as_ref()
            .is_some_and(|a| a.daemon_hotkey.is_some())
        {
            self.state.start_listening();
            context.set_window_visibility(false);
        } else {
            let mut current_image = CurrentImage::capture_image(
                self.args.as_ref().and_then(|a| a.monitor),
                &context.graphics.device,
                &context.graphics.queue,
            )
            .expect("Could not capture image");
            let (w, h) = current_image.image.dimensions();
            let (w, h) = (w as f32, h as f32);
            current_image.update_uniforms(context.total_time, &self.state.selection, (w, h));
            self.current_image = Some(current_image);
        }
        self.context = Some(context);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(context) = &mut self.context else {
            return;
        };

        if id != context.window_id() {
            return;
        }

        // Check if we are in daemon mode
        self.state.handle_event(&event);
        if !self
            .state
            .get_listening(self.args.as_ref().and_then(|a| a.daemon_hotkey))
        {
            return;
        }

        match event {
            WindowEvent::RedrawRequested => {
                context.update();
                if let Some(c_img) = &mut self.current_image {
                    c_img.update_uniforms(
                        context.total_time,
                        &self.state.selection,
                        context.size(),
                    );
                    context.draw(&c_img.bundle);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !self.execute_key_command(event, event_loop) {
                    return;
                }
            }
            WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Left) => self.state.start_drag(),
                (ElementState::Released, MouseButton::Left) => self.state.end_drag(),
                (_, MouseButton::Right) => self.state.cancel_drag(),
                _ => {}
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}
