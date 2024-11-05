use std::sync::{atomic::AtomicBool, mpsc::TryRecvError};

use crate::{args::Args, hotkey::HotKey};
use device_query::{DeviceEvents, DeviceEventsHandler, DeviceQuery, KeyboardCallback, Keycode};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
};

use crate::context::{AppContext, Direction, SelectionMode};

#[derive(Debug)]
struct KeyAction {
    key: Keycode,
    pressed: bool,
}

struct Daemon<Up: ?Sized, Down: ?Sized> {
    pressed: Vec<Keycode>,
    rx: std::sync::mpsc::Receiver<KeyAction>,
    _event_handler: DeviceEventsHandler,
    hotkey: HotKey,
    key_up: device_query::CallbackGuard<Up>,
    key_down: device_query::CallbackGuard<Down>,
}

impl<U: ?Sized, D: ?Sized> Daemon<U, D> {
    fn clear_buffer(&mut self) {
        for action in self.rx.iter() {
            println!("{:?}", action);
            if action.pressed {
                self.pressed.push(action.key);
            } else {
                self.pressed.retain(|&x| x != action.key);
            }
        }
    }
}

pub struct App<Up, Down> {
    args: Option<Args>,
    daemon: Option<Daemon<Up, Down>>,
    context: Option<AppContext>,
}

impl<U, D> App<U, D> {
    pub fn new(args: Option<Args>) -> Self {
        App {
            args,
            context: None,
            daemon: None,
        }
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let mut start_daemon = None;

        if let Some(args) = &self.args {
            if let Err(e) = args.verify() {
                eprintln!("{}", e);
                std::process::exit(1);
            }

            if args.monitor_list {
                println!("Available monitors:");
                for monitor in xcap::Monitor::all().into_iter().flatten() {
                    println!("ID: {}", monitor.id());
                }
                std::process::exit(0);
            }

            if let Some(hotkey) = &args.daemon_hotkey {
                // Wait until the hotkey is pressed
                let hotkey: crate::hotkey::HotKey = hotkey.parse()?;
                // crate::hotkey::wait_until_pressed(hotkey);
                start_daemon = Some(hotkey);
            }

            if args.delay > 0 {
                std::thread::sleep(std::time::Duration::from_millis(args.delay));
            }

            if let Some(output_dir) = &args.output_dir {
                std::fs::create_dir_all(output_dir)?;
            }
        }
        let daemon = start_daemon.map(|hotkey| {
            let (tx, rx) = std::sync::mpsc::channel();
            let _event_handler =
                device_query::DeviceEventsHandler::new(std::time::Duration::from_millis(10))
                    .expect("Could not start event loop");
            let txa = tx.clone();
            let key_down = _event_handler.on_key_down(move |key| {
                let _ = txa.send(KeyAction {
                    key: *key,
                    pressed: true,
                });
            });
            let key_up = _event_handler.on_key_up(move |key| {
                let _ = tx.send(KeyAction {
                    key: *key,
                    pressed: false,
                });
            });
            Daemon {
                _event_handler,
                rx,
                pressed: Vec::new(),
                key_up,
                key_down,
                hotkey,
            }
        });

        // match daemon {
        //     Some(daemon) => loop {
        //         if let Ok(_) = daemon.rx.try_recv() {
        //             self.start_loop()?;
        //             if daemon.stay_running {
        //                 continue;
        //             }
        //             break;
        //         }
        //     },
        //     None => self.start_loop()?,
        // }
        self.daemon = daemon;

        let event_loop = EventLoop::new()?;
        Ok(event_loop.run_app(&mut self)?)
    }
}

impl<U, D> ApplicationHandler for App<U, D> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.context.is_some() {
            return;
        }
        let mut context = AppContext::new(event_loop).expect("Could not start context");

        if let Some(args) = &self.args {
            let Some(arg_context) = context.set_args(args) else {
                event_loop.exit();
                return;
            };
            context = arg_context;
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
        if let Some(daemon) = self.daemon.as_mut() {
            daemon.clear_buffer();
            if !daemon.hotkey.check(daemon.pressed.iter().copied()) {
                return;
            }
            daemon.pressed.clear();
        }
        if id != context.window_id() {
            return;
        }

        let stay_running = self.args.as_ref().is_some_and(|d| d.stay_running());

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
                    if !stay_running {
                        event_loop.exit();
                        context.destroy();
                    }
                }
                (ElementState::Pressed, Key::Named(NamedKey::Space)) => {
                    context.set_window_visibility(false);
                    if let Err(e) = context.save_selection(self.args.as_ref()) {
                        eprintln!("{}", e);
                    };
                    if !stay_running {
                        event_loop.exit();
                    }
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
