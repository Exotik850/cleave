use device_query::{DeviceQuery, Keycode};
pub use keyboard_types::{Code, Modifiers};
use std::{borrow::Borrow, fmt::Display, hash::Hash, str::FromStr};

use crate::keycode_to_code;

#[cfg(target_os = "macos")]
pub const CMD_OR_CTRL: Modifiers = Modifiers::SUPER;
#[cfg(not(target_os = "macos"))]
pub const CMD_OR_CTRL: Modifiers = Modifiers::CONTROL;

#[derive(thiserror::Error, Debug)]
pub enum HotKeyParseError {
    #[error("Couldn't recognize \"{0}\" as a valid key for hotkey, if you feel like it should be, please report this to https://github.com/tauri-apps/muda")]
    UnsupportedKey(String),
    #[error("Found empty token while parsing hotkey: {0}")]
    EmptyToken(String),
    #[error("Invalid hotkey format: \"{0}\", an hotkey should have the modifiers first and only one main key, for example: \"Shift + Alt + K\"")]
    InvalidFormat(String),
}

/// A keyboard shortcut that consists of an optional combination
/// of modifier keys (provided by [`Modifiers`](crate::hotkey::Modifiers)) and
/// one key ([`Code`](crate::hotkey::Code)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HotKey {
    /// The hotkey modifiers.
    pub mods: Modifiers,
    /// The hotkey key.
    pub key: Code,
    /// The hotkey id.
    pub id: u32,
}

impl HotKey {
    /// Creates a new hotkey to define keyboard shortcuts throughout your application.
    /// Only [`Modifiers::ALT`], [`Modifiers::SHIFT`], [`Modifiers::CONTROL`], and [`Modifiers::SUPER`]
    pub fn new(mods: Option<Modifiers>, key: Code) -> Self {
        let mut mods = mods.unwrap_or_else(Modifiers::empty);
        if mods.contains(Modifiers::META) {
            mods.remove(Modifiers::META);
            mods.insert(Modifiers::SUPER);
        }

        Self {
            mods,
            key,
            id: mods.bits() << 16 | key as u32,
        }
    }

    pub fn check(&self, codes: impl IntoIterator<Item = Keycode>) -> bool {
        let mut mods = Modifiers::empty();
        let mut code = None;
        for key in codes {
            match key {
                Keycode::LShift | Keycode::RShift => mods |= Modifiers::SHIFT,
                Keycode::LControl | Keycode::RControl => mods |= Modifiers::CONTROL,
                Keycode::LAlt | Keycode::RAlt => mods |= Modifiers::ALT,
                Keycode::LMeta | Keycode::RMeta => mods |= Modifiers::SUPER,
                other => {
                    code = Some(other);
                }
            }
        }

        if code.is_none() {
            return false;
        }

        self.matches(mods, keycode_to_code(code.unwrap()))
    }

    /// Returns the id associated with this hotKey
    /// which is a hash of the string represention of modifiers and key within this hotKey.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns `true` if this [`Code`] and [`Modifiers`] matches this hotkey.
    pub fn matches(&self, modifiers: impl Borrow<Modifiers>, key: impl Borrow<Code>) -> bool {
        // Should be a const but const bit_or doesn't work here.
        let base_mods = Modifiers::SHIFT | Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER;
        let modifiers = modifiers.borrow();
        let key = key.borrow();
        self.mods == *modifiers & base_mods && self.key == *key
    }

    /// Converts this hotkey into a string.
    pub fn into_string(self) -> String {
        let mut hotkey = String::new();
        if self.mods.contains(Modifiers::SHIFT) {
            hotkey.push_str("shift+")
        }
        if self.mods.contains(Modifiers::CONTROL) {
            hotkey.push_str("control+")
        }
        if self.mods.contains(Modifiers::ALT) {
            hotkey.push_str("alt+")
        }
        if self.mods.contains(Modifiers::SUPER) {
            hotkey.push_str("super+")
        }
        hotkey.push_str(&self.key.to_string());
        hotkey
    }
}

impl Display for HotKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.into_string())
    }
}

// HotKey::from_str is available to be backward
// compatible with tauri and it also open the option
// to generate hotkey from string
impl FromStr for HotKey {
    type Err = HotKeyParseError;
    fn from_str(hotkey_string: &str) -> Result<Self, Self::Err> {
        parse_hotkey(hotkey_string)
    }
}

impl TryFrom<&str> for HotKey {
    type Error = HotKeyParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_hotkey(value)
    }
}

impl TryFrom<String> for HotKey {
    type Error = HotKeyParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        parse_hotkey(&value)
    }
}

fn parse_hotkey(hotkey: &str) -> Result<HotKey, HotKeyParseError> {
    let tokens = hotkey.split('+').collect::<Vec<&str>>();

    let mut mods = Modifiers::empty();
    let mut key = None;

    match tokens.len() {
        // single key hotkey
        1 => {
            key = Some(parse_key(tokens[0])?);
        }
        // modifiers and key comobo hotkey
        _ => {
            for raw in tokens {
                let token = raw.trim();

                if token.is_empty() {
                    return Err(HotKeyParseError::EmptyToken(hotkey.to_string()));
                }

                if key.is_some() {
                    // At this point we have parsed the modifiers and a main key, so by reaching
                    // this code, the function either received more than one main key or
                    //  the hotkey is not in the right order
                    // examples:
                    // 1. "Ctrl+Shift+C+A" => only one main key should be allowd.
                    // 2. "Ctrl+C+Shift" => wrong order
                    return Err(HotKeyParseError::InvalidFormat(hotkey.to_string()));
                }

                match token.to_uppercase().as_str() {
                    "OPTION" | "ALT" => {
                        mods |= Modifiers::ALT;
                    }
                    "CONTROL" | "CTRL" => {
                        mods |= Modifiers::CONTROL;
                    }
                    "COMMAND" | "CMD" | "SUPER" => {
                        mods |= Modifiers::SUPER;
                    }
                    "SHIFT" => {
                        mods |= Modifiers::SHIFT;
                    }
                    #[cfg(target_os = "macos")]
                    "COMMANDORCONTROL" | "COMMANDORCTRL" | "CMDORCTRL" | "CMDORCONTROL" => {
                        mods |= Modifiers::SUPER;
                    }
                    #[cfg(not(target_os = "macos"))]
                    "COMMANDORCONTROL" | "COMMANDORCTRL" | "CMDORCTRL" | "CMDORCONTROL" => {
                        mods |= Modifiers::CONTROL;
                    }
                    _ => {
                        key = Some(parse_key(token)?);
                    }
                }
            }
        }
    }

    Ok(HotKey::new(
        Some(mods),
        key.ok_or_else(|| HotKeyParseError::InvalidFormat(hotkey.to_string()))?,
    ))
}

fn parse_key(key: &str) -> Result<Code, HotKeyParseError> {
    use Code::*;
    match key.to_uppercase().as_str() {
        "BACKQUOTE" | "`" => Ok(Backquote),
        "BACKSLASH" | "\\" => Ok(Backslash),
        "BRACKETLEFT" | "[" => Ok(BracketLeft),
        "BRACKETRIGHT" | "]" => Ok(BracketRight),
        "PAUSE" | "PAUSEBREAK" => Ok(Pause),
        "COMMA" | "," => Ok(Comma),
        "DIGIT0" | "0" => Ok(Digit0),
        "DIGIT1" | "1" => Ok(Digit1),
        "DIGIT2" | "2" => Ok(Digit2),
        "DIGIT3" | "3" => Ok(Digit3),
        "DIGIT4" | "4" => Ok(Digit4),
        "DIGIT5" | "5" => Ok(Digit5),
        "DIGIT6" | "6" => Ok(Digit6),
        "DIGIT7" | "7" => Ok(Digit7),
        "DIGIT8" | "8" => Ok(Digit8),
        "DIGIT9" | "9" => Ok(Digit9),
        "EQUAL" | "=" => Ok(Equal),
        "KEYA" | "A" => Ok(KeyA),
        "KEYB" | "B" => Ok(KeyB),
        "KEYC" | "C" => Ok(KeyC),
        "KEYD" | "D" => Ok(KeyD),
        "KEYE" | "E" => Ok(KeyE),
        "KEYF" | "F" => Ok(KeyF),
        "KEYG" | "G" => Ok(KeyG),
        "KEYH" | "H" => Ok(KeyH),
        "KEYI" | "I" => Ok(KeyI),
        "KEYJ" | "J" => Ok(KeyJ),
        "KEYK" | "K" => Ok(KeyK),
        "KEYL" | "L" => Ok(KeyL),
        "KEYM" | "M" => Ok(KeyM),
        "KEYN" | "N" => Ok(KeyN),
        "KEYO" | "O" => Ok(KeyO),
        "KEYP" | "P" => Ok(KeyP),
        "KEYQ" | "Q" => Ok(KeyQ),
        "KEYR" | "R" => Ok(KeyR),
        "KEYS" | "S" => Ok(KeyS),
        "KEYT" | "T" => Ok(KeyT),
        "KEYU" | "U" => Ok(KeyU),
        "KEYV" | "V" => Ok(KeyV),
        "KEYW" | "W" => Ok(KeyW),
        "KEYX" | "X" => Ok(KeyX),
        "KEYY" | "Y" => Ok(KeyY),
        "KEYZ" | "Z" => Ok(KeyZ),
        "MINUS" | "-" => Ok(Minus),
        "PERIOD" | "." => Ok(Period),
        "QUOTE" | "'" => Ok(Quote),
        "SEMICOLON" | ";" => Ok(Semicolon),
        "SLASH" | "/" => Ok(Slash),
        "BACKSPACE" => Ok(Backspace),
        "CAPSLOCK" => Ok(CapsLock),
        "ENTER" => Ok(Enter),
        "SPACE" => Ok(Space),
        "TAB" => Ok(Tab),
        "DELETE" => Ok(Delete),
        "END" => Ok(End),
        "HOME" => Ok(Home),
        "INSERT" => Ok(Insert),
        "PAGEDOWN" => Ok(PageDown),
        "PAGEUP" => Ok(PageUp),
        "PRINTSCREEN" => Ok(PrintScreen),
        "SCROLLLOCK" => Ok(ScrollLock),
        "ARROWDOWN" | "DOWN" => Ok(ArrowDown),
        "ARROWLEFT" | "LEFT" => Ok(ArrowLeft),
        "ARROWRIGHT" | "RIGHT" => Ok(ArrowRight),
        "ARROWUP" | "UP" => Ok(ArrowUp),
        "NUMLOCK" => Ok(NumLock),
        "NUMPAD0" | "NUM0" => Ok(Numpad0),
        "NUMPAD1" | "NUM1" => Ok(Numpad1),
        "NUMPAD2" | "NUM2" => Ok(Numpad2),
        "NUMPAD3" | "NUM3" => Ok(Numpad3),
        "NUMPAD4" | "NUM4" => Ok(Numpad4),
        "NUMPAD5" | "NUM5" => Ok(Numpad5),
        "NUMPAD6" | "NUM6" => Ok(Numpad6),
        "NUMPAD7" | "NUM7" => Ok(Numpad7),
        "NUMPAD8" | "NUM8" => Ok(Numpad8),
        "NUMPAD9" | "NUM9" => Ok(Numpad9),
        "NUMPADADD" | "NUMADD" | "NUMPADPLUS" | "NUMPLUS" => Ok(NumpadAdd),
        "NUMPADDECIMAL" | "NUMDECIMAL" => Ok(NumpadDecimal),
        "NUMPADDIVIDE" | "NUMDIVIDE" => Ok(NumpadDivide),
        "NUMPADENTER" | "NUMENTER" => Ok(NumpadEnter),
        "NUMPADEQUAL" | "NUMEQUAL" => Ok(NumpadEqual),
        "NUMPADMULTIPLY" | "NUMMULTIPLY" => Ok(NumpadMultiply),
        "NUMPADSUBTRACT" | "NUMSUBTRACT" => Ok(NumpadSubtract),
        "ESCAPE" | "ESC" => Ok(Escape),
        "F1" => Ok(F1),
        "F2" => Ok(F2),
        "F3" => Ok(F3),
        "F4" => Ok(F4),
        "F5" => Ok(F5),
        "F6" => Ok(F6),
        "F7" => Ok(F7),
        "F8" => Ok(F8),
        "F9" => Ok(F9),
        "F10" => Ok(F10),
        "F11" => Ok(F11),
        "F12" => Ok(F12),
        "AUDIOVOLUMEDOWN" | "VOLUMEDOWN" => Ok(AudioVolumeDown),
        "AUDIOVOLUMEUP" | "VOLUMEUP" => Ok(AudioVolumeUp),
        "AUDIOVOLUMEMUTE" | "VOLUMEMUTE" => Ok(AudioVolumeMute),
        "MEDIAPLAY" => Ok(MediaPlay),
        "MEDIAPAUSE" => Ok(MediaPause),
        "MEDIAPLAYPAUSE" => Ok(MediaPlayPause),
        "MEDIASTOP" => Ok(MediaStop),
        "MEDIATRACKNEXT" => Ok(MediaTrackNext),
        "MEDIATRACKPREV" | "MEDIATRACKPREVIOUS" => Ok(MediaTrackPrevious),
        "F13" => Ok(F13),
        "F14" => Ok(F14),
        "F15" => Ok(F15),
        "F16" => Ok(F16),
        "F17" => Ok(F17),
        "F18" => Ok(F18),
        "F19" => Ok(F19),
        "F20" => Ok(F20),
        "F21" => Ok(F21),
        "F22" => Ok(F22),
        "F23" => Ok(F23),
        "F24" => Ok(F24),

        _ => Err(HotKeyParseError::UnsupportedKey(key.to_string())),
    }
}

pub fn wait_until_pressed(hotkey: HotKey) {
    let state = device_query::DeviceState::new();
    println!("Waiting for hotkey: {}", hotkey);
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if hotkey.check(state.get_keys()) {
            break;
        }
    }
}

// struct KeyCatcher {
//    pressed: HashSet<Code>,
// }
