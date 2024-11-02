use crate::args::Args;
use clap::Parser;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

use crate::context::{AppContext, Direction, SelectionMode};

pub struct App {
    args: Option<Args>,
    context: Option<AppContext>,
}

impl App {
    pub fn new(args: Option<Args>) -> Self {
        App {
            args,
            context: None,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        if let Some(args) = &self.args {
            if let Some(output_dir) = &args.output_dir {
                std::fs::create_dir_all(output_dir)?;
            }
        }

        let event_loop = winit::event_loop::EventLoop::new()?;
        event_loop.run_app(self)?;
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.context.is_some() {
            return;
        }

        let context = AppContext::new(event_loop).expect("Could not start context");
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

        match event {
            WindowEvent::RedrawRequested => {
                context.draw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                context.update_mouse_position(position.x, position.y);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        logical_key: key,
                        ..
                    },
                ..
            } => match (state, key) {
                (ElementState::Pressed, Key::Named(NamedKey::Escape)) => {
                    event_loop.exit();
                    context.destroy();
                }
                (ElementState::Pressed, Key::Named(NamedKey::Space)) => {
                    context.hide_window();
                    context.save_selection_to_clipboard();
                    event_loop.exit();
                }
                (ElementState::Pressed, Key::Named(NamedKey::ArrowDown)) => {
                    context.handle_move(Direction::Down);
                }
                (ElementState::Pressed, Key::Named(NamedKey::ArrowUp)) => {
                    context.handle_move(Direction::Up);
                }
                (ElementState::Pressed, Key::Named(NamedKey::ArrowLeft)) => {
                    context.handle_move(Direction::Left);
                }
                (ElementState::Pressed, Key::Named(NamedKey::ArrowRight)) => {
                    context.handle_move(Direction::Right);
                }
                (ElementState::Pressed, Key::Named(NamedKey::Shift)) => {
                    context.set_mode(SelectionMode::InverseResize);
                }
                (ElementState::Released, Key::Named(NamedKey::Shift)) => {
                    context.set_mode(SelectionMode::Resize);
                }
                (ElementState::Pressed, Key::Named(NamedKey::Control)) => {
                    context.set_mode(SelectionMode::Move);
                }
                (ElementState::Released, Key::Named(NamedKey::Control)) => {
                    context.set_mode(SelectionMode::Resize);
                }
                _ => {}
            },
            WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Left) => context.start_drag(),
                (ElementState::Released, MouseButton::Left) => context.end_drag(),
                (_, MouseButton::Right) => context.cancel_drag(),
                _ => {}
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}
