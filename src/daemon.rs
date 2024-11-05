use std::sync::{Arc, RwLock};

use device_query::{DeviceEvents, DeviceEventsHandler, Keycode};

use crate::hotkey::HotKey;

#[derive(Debug)]
pub(crate) struct KeyAction {
    key: Keycode,
    pressed: bool,
}

pub(crate) struct Daemon {
    pressed: Arc<RwLock<Vec<Keycode>>>,

    // pub rx: std::sync::mpsc::Receiver<KeyAction>,
    _event_handler: DeviceEventsHandler,
    _key_up: device_query::CallbackGuard<Keycode>,
    _key_down: device_query::CallbackGuard<Keycode>,
    hotkey: HotKey,
    listening: bool,
}

impl Daemon {
    pub fn start(hotkey: HotKey) -> Self {
        let _event_handler =
            device_query::DeviceEventsHandler::new(std::time::Duration::from_millis(10))
                .expect("Could not start event loop");
        let pressed: Arc<RwLock<Vec<Keycode>>> = Default::default();
        let pa: Arc<_> = pressed.clone();
        let _key_down = _event_handler.on_key_down(move |key| {
            let mut pressed = pa.write().unwrap();
            pressed.push(key);
        });
        let pb: Arc<_> = pressed.clone();
        let _key_up = _event_handler.on_key_up(move |key| {
            let mut pressed = pb.write().unwrap();
            pressed.retain(|&k| k != key);
        });
        Daemon {
            _event_handler,
            _key_up,
            _key_down,
            // rx,
            pressed,
            hotkey,
            listening: true,
        }
    }

    fn is_pressed(&self) -> bool {
        let pressed = dbg!(self.pressed.read().unwrap());
        self.hotkey.check(pressed.iter().copied())
    }

    fn clear_buffer(&mut self) {
        let mut pressed = self.pressed.write().unwrap();
        pressed.clear();
    }

    pub fn start_listening(&mut self) {
        self.listening = true;
    }

    pub fn get_pressed(&mut self) -> bool {
        let val = self.listening && self.is_pressed();
        if val {
            self.clear_buffer();
            self.listening = false;
        }
        val
    }
}
