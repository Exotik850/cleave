use clap::Parser;
use cleave_daemon::{DeviceEvents, DeviceEventsHandler, HotKey, Keycode, Modifiers};
use std::{collections::HashSet, time::Duration};

#[derive(clap::Parser, Debug)]
struct Args {
    /// The amount of time to sleep between each event loop iteration in milliseconds
    #[arg(short, long, default_value = "100")]
    sleep: u64,

    /// The hotkey to use to start the event loop
    #[arg(short = 'm', long, default_value = "Shift+X")]
    hotkey: HotKey,

    /// Whether or not to stay alive after the hotkey is pressed
    #[arg(short, long)]
    persist: bool,
}

#[derive(Debug)]
struct KeyAction {
    key: Keycode,
    pressed: bool,
}

fn main() -> anyhow::Result<()> {
    let config_path = dirs::config_dir().expect("Could not find config directory");
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
            run_cleave()?;
            pressed.clear();
            mods = Modifiers::empty();
            if !args.persist {
                break;
            }
        }
    }
    Ok(())
}

fn run_cleave() -> anyhow::Result<()> {
    let mut cleave = std::process::Command::new("cleave");
    cleave.args(std::env::args().skip(1));
    match cleave.status() {
        Ok(status) => {
            if !status.success() {
                anyhow::bail!("cleave exited with status: {}", status);
            }
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                anyhow::bail!("Could not find cleave in PATH");
            }
            _ => {
                anyhow::bail!("Could not start cleave: {}", e);
            }
        },
    };
    Ok(())
}
