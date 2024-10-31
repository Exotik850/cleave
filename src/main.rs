// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use memory_stats::memory_stats;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

mod context;
mod graphics_bundle;
mod graphics_impl;
mod instance;
mod texture;
mod vertex;
use context::{AppContext, Direction, MoveMode};

pub struct Drag {
    start: (f64, f64),
    end: Option<(f64, f64)>,
}

impl Drag {
    fn coords(&self) -> Option<((u32, u32), (u32, u32))> {
        let end = self.end?;
        let (start_x, start_y) = (self.start.0 as u32, self.start.1 as u32);
        let (end_x, end_y) = (end.0 as u32, end.1 as u32);

        let (min_x, max_x) = (start_x.min(end_x), start_x.max(end_x));
        let (min_y, max_y) = (start_y.min(end_y), start_y.max(end_y));
        Some(((min_x, min_y), (max_x, max_y)))
    }
}

pub struct Selection {
    start: (f64, f64),
    end: (f64, f64),
}

impl Selection {
    fn dimensions(&self) -> (f64, f64) {
        let width = (self.end.0 - self.start.0).abs();
        let height = (self.end.1 - self.start.1).abs();
        (width, height)
    }

    fn area(&self) -> f64 {
        let (width, height) = self.dimensions();
        width * height
    }

    // fn aspect_ratio(&self) -> f64 {
    //     let (width, height) = self.dimensions();
    //     width / height
    // }

    // fn center(&self) -> (f64, f64) {
    //     let x = (self.start.0 + self.end.0) / 2.0;
    //     let y = (self.start.1 + self.end.1) / 2.0;
    //     (x, y)
    // }

    fn coords(&self) -> ((u32, u32), (u32, u32)) {
        let (start_x, start_y) = (self.start.0 as u32, self.start.1 as u32);
        let (end_x, end_y) = (self.end.0 as u32, self.end.1 as u32);

        let (min_x, max_x) = (start_x.min(end_x), start_x.max(end_x));
        let (min_y, max_y) = (start_y.min(end_y), start_y.max(end_y));
        ((min_x, min_y), (max_x, max_y))
    }
}

struct App {
    context: Option<AppContext>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
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
                context.update_mouse_position(position);
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
                (ElementState::Pressed, Key::Character(ch)) => {
                    if ch == "m" || ch == "M" {
                        if let Some(usage) = memory_stats() {
                            println!("Current physical memory usage: {}", usage.physical_mem);
                            println!("Current virtual memory usage: {}", usage.virtual_mem);
                        } else {
                            println!("Could not get memory usage");
                        }
                    } else {
                        println!("Unknown key: {:?}", ch);
                    }
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
                    context.set_mode(MoveMode::InverseResize);
                }
                (ElementState::Released, Key::Named(NamedKey::Shift)) => {
                    context.set_mode(MoveMode::Resize);
                }
                (ElementState::Pressed, Key::Named(NamedKey::Control)) => {
                    context.set_mode(MoveMode::Move);
                }
                (ElementState::Released, Key::Named(NamedKey::Control)) => {
                    context.set_mode(MoveMode::Resize);
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

fn main() -> anyhow::Result<()> {
    let mut app = App { context: None };
    let event_loop = winit::event_loop::EventLoop::new()?;
    event_loop.run_app(&mut app)?;
    Ok(())
}
