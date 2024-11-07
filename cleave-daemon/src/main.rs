use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::Parser;
use device_query::{DeviceEvents, DeviceEventsHandler, Keycode};
use hotkey::HotKey;
use modifiers::Modifiers;
mod hotkey;
mod modifiers;

#[derive(clap::Parser, Debug)]
struct Args {
    /// The amount of time to sleep between each event loop iteration
    #[arg(short, long, default_value = "100")]
    sleep: u64,

    /// The hotkey to use to start the event loop
    #[arg(short = 'm', long, default_value = "Shift+X")]
    hotkey: HotKey,
}

#[derive(Debug)]
struct KeyAction {
    key: Keycode,
    pressed: bool,
}

fn main() {
    let args: Args = Args::parse();
    let handler = DeviceEventsHandler::new(Duration::from_millis(args.sleep))
        .expect("Could not create event loop");
    let (tx, rx) = std::sync::mpsc::channel();
    let ta = tx.clone();
    let _g1 = handler.on_key_down(move |key| {
        ta.send(KeyAction { key, pressed: true }).unwrap();
    });
    let tb = tx;
    let _g2 = handler.on_key_up(move |key| {
        tb.send(KeyAction {
            key,
            pressed: false,
        })
        .unwrap();
    });

    let mut pressed = HashSet::new();
    let mut mods = Modifiers::empty();
    for event in rx.iter() {
        if let Some(m) = Modifiers::from_keycode(event.key) {
            if event.pressed {
                mods |= m;
            } else {
                mods &= !m;
            }
        }
        if event.pressed {
            pressed.insert(event.key);
        } else {
            pressed.remove(&event.key);
        }
        if args.hotkey.matches(mods, event.key) && event.pressed {
            print!("RUN");
        }
    }
}
