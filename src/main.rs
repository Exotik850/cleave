// TODO When opened outside of a terminal, the program should not open the terminal.
// However, when opened inside a terminal, the program should be able to output to the terminal.
// #![windows_subsystem = "windows"]

use std::io::IsTerminal;

use args::Args;
use clap::Parser;
use device_query::Keycode;
use keyboard_types::Code;

mod app;
mod args;
mod context;
mod hotkey;

fn main() -> anyhow::Result<()> {
    let stdout = std::io::stdout();
    let args = stdout.is_terminal().then(Args::parse);
    let mut app = app::App::new(args);
    app.run()?;
    Ok(())
}

fn keycode_to_code(keycode: Keycode) -> Code {
    match keycode {
        Keycode::Key0 => Code::Digit0,
        Keycode::Key1 => Code::Digit1,
        Keycode::Key2 => Code::Digit2,
        Keycode::Key3 => Code::Digit3,
        Keycode::Key4 => Code::Digit4,
        Keycode::Key5 => Code::Digit5,
        Keycode::Key6 => Code::Digit6,
        Keycode::Key7 => Code::Digit7,
        Keycode::Key8 => Code::Digit8,
        Keycode::Key9 => Code::Digit9,
        Keycode::A => Code::KeyA,
        Keycode::B => Code::KeyB,
        Keycode::C => Code::KeyC,
        Keycode::D => Code::KeyD,
        Keycode::E => Code::KeyE,
        Keycode::F => Code::KeyF,
        Keycode::G => Code::KeyG,
        Keycode::H => Code::KeyH,
        Keycode::I => Code::KeyI,
        Keycode::J => Code::KeyJ,
        Keycode::K => Code::KeyK,
        Keycode::L => Code::KeyL,
        Keycode::M => Code::KeyM,
        Keycode::N => Code::KeyN,
        Keycode::O => Code::KeyO,
        Keycode::P => Code::KeyP,
        Keycode::Q => Code::KeyQ,
        Keycode::R => Code::KeyR,
        Keycode::S => Code::KeyS,
        Keycode::T => Code::KeyT,
        Keycode::U => Code::KeyU,
        Keycode::V => Code::KeyV,
        Keycode::W => Code::KeyW,
        Keycode::X => Code::KeyX,
        Keycode::Y => Code::KeyY,
        Keycode::Z => Code::KeyZ,
        Keycode::F1 => Code::F1,
        Keycode::F2 => Code::F2,
        Keycode::F3 => Code::F3,
        Keycode::F4 => Code::F4,
        Keycode::F5 => Code::F5,
        Keycode::F6 => Code::F6,
        Keycode::F7 => Code::F7,
        Keycode::F8 => Code::F8,
        Keycode::F9 => Code::F9,
        Keycode::F10 => Code::F10,
        Keycode::F11 => Code::F11,
        Keycode::F12 => Code::F12,
        Keycode::F13 => Code::F13,
        Keycode::F14 => Code::F14,
        Keycode::F15 => Code::F15,
        Keycode::F16 => Code::F16,
        Keycode::F17 => Code::F17,
        Keycode::F18 => Code::F18,
        Keycode::F19 => Code::F19,
        Keycode::F20 => Code::F20,
        Keycode::Escape => Code::Escape,
        Keycode::Space => Code::Space,
        Keycode::LControl => Code::ControlLeft,
        Keycode::RControl => Code::ControlRight,
        Keycode::LShift => Code::ShiftLeft,
        Keycode::RShift => Code::ShiftRight,
        Keycode::LAlt => Code::AltLeft,
        Keycode::RAlt => Code::AltRight,
        Keycode::Command => Code::MetaLeft,
        Keycode::LOption => Code::MetaLeft,
        Keycode::ROption => Code::MetaRight,
        Keycode::LMeta => Code::MetaLeft,
        Keycode::RMeta => Code::MetaRight,
        Keycode::Enter => Code::Enter,
        Keycode::Up => Code::ArrowUp,
        Keycode::Down => Code::ArrowDown,
        Keycode::Left => Code::ArrowLeft,
        Keycode::Right => Code::ArrowRight,
        Keycode::Backspace => Code::Backspace,
        Keycode::CapsLock => Code::CapsLock,
        Keycode::Tab => Code::Tab,
        Keycode::Home => Code::Home,
        Keycode::End => Code::End,
        Keycode::PageUp => Code::PageUp,
        Keycode::PageDown => Code::PageDown,
        Keycode::Insert => Code::Insert,
        Keycode::Delete => Code::Delete,
        Keycode::Numpad0 => Code::Numpad0,
        Keycode::Numpad1 => Code::Numpad1,
        Keycode::Numpad2 => Code::Numpad2,
        Keycode::Numpad3 => Code::Numpad3,
        Keycode::Numpad4 => Code::Numpad4,
        Keycode::Numpad5 => Code::Numpad5,
        Keycode::Numpad6 => Code::Numpad6,
        Keycode::Numpad7 => Code::Numpad7,
        Keycode::Numpad8 => Code::Numpad8,
        Keycode::Numpad9 => Code::Numpad9,
        Keycode::NumpadSubtract => Code::NumpadSubtract,
        Keycode::NumpadAdd => Code::NumpadAdd,
        Keycode::NumpadDivide => Code::NumpadDivide,
        Keycode::NumpadMultiply => Code::NumpadMultiply,
        Keycode::NumpadEquals => Code::NumpadEqual,
        Keycode::NumpadEnter => Code::NumpadEnter,
        Keycode::NumpadDecimal => Code::NumpadDecimal,
        Keycode::Grave => Code::Backquote,
        Keycode::Minus => Code::Minus,
        Keycode::Equal => Code::Equal,
        Keycode::LeftBracket => Code::BracketLeft,
        Keycode::RightBracket => Code::BracketRight,
        Keycode::BackSlash => Code::Backslash,
        Keycode::Semicolon => Code::Semicolon,
        Keycode::Apostrophe => Code::Quote,
        Keycode::Comma => Code::Comma,
        Keycode::Dot => Code::Period,
        Keycode::Slash => Code::Slash,
    }
}
