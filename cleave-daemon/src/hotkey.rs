use std::{borrow::Borrow, fmt::Display, str::FromStr};

pub use crate::modifiers::Modifiers;
pub use device_query::Keycode;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotKey {
    /// The hotkey modifiers.
    pub mods: Modifiers,
    /// The hotkey key.
    pub key: Keycode,
}

impl HotKey {
    /// Creates a new hotkey to define keyboard shortcuts throughout your application.
    /// Only [`Modifiers::ALT`], [`Modifiers::SHIFT`], [`Modifiers::CONTROL`], and [`Modifiers::SUPER`]
    pub fn new(mods: Option<Modifiers>, key: Keycode) -> Self {
        let mods = mods.unwrap_or_default();
        Self { mods, key }
    }

    /// Returns `true` if this [`Code`] and [`Modifiers`] matches this hotkey.
    pub fn matches(&self, modifiers: impl Borrow<Modifiers>, key: impl Borrow<Keycode>) -> bool {
        // Should be a const but const bit_or doesn't work here.
        let base_mods = Modifiers::SHIFT | Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER;
        let modifiers = modifiers.borrow();
        let key = key.borrow();
        (self.mods == (*modifiers & base_mods)) && (self.key == *key)
    }

    /// Converts this hotkey into a string.
    pub fn into_string(self) -> String {
        let mut hotkey = String::new();
        let state = self.mods;
        if state.contains(Modifiers::SHIFT) {
            hotkey.push_str("shift+");
        }
        if state.contains(Modifiers::CONTROL) {
            hotkey.push_str("control+");
        }
        if state.contains(Modifiers::ALT) {
            hotkey.push_str("alt+");
        }
        if state.contains(Modifiers::SUPER) {
            hotkey.push_str("super+");
        }
        hotkey.push_str(&format!("{:?}", self.key).to_lowercase());
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
                    "META" => {
                        mods |= Modifiers::META;
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

fn parse_key(key: &str) -> Result<Keycode, HotKeyParseError> {
    use Keycode::*;
    match key.to_uppercase().as_str() {
        "BACKQUOTE" | "`" => Ok(Grave),
        "BACKSLASH" | "\\" => Ok(BackSlash),
        "BRACKETLEFT" | "[" => Ok(LeftBracket),
        "BRACKETRIGHT" | "]" => Ok(RightBracket),
        // "PAUSE" | "PAUSEBREAK" => Ok(),
        "COMMA" | "," => Ok(Comma),
        "DIGIT0" | "0" => Ok(Key0),
        "DIGIT1" | "1" => Ok(Key1),
        "DIGIT2" | "2" => Ok(Key2),
        "DIGIT3" | "3" => Ok(Key3),
        "DIGIT4" | "4" => Ok(Key4),
        "DIGIT5" | "5" => Ok(Key5),
        "DIGIT6" | "6" => Ok(Key6),
        "DIGIT7" | "7" => Ok(Key7),
        "DIGIT8" | "8" => Ok(Key8),
        "DIGIT9" | "9" => Ok(Key9),
        "EQUAL" | "=" => Ok(Equal),
        "KEYA" | "A" => Ok(A),
        "KEYB" | "B" => Ok(B),
        "KEYC" | "C" => Ok(C),
        "KEYD" | "D" => Ok(D),
        "KEYE" | "E" => Ok(E),
        "KEYF" | "F" => Ok(F),
        "KEYG" | "G" => Ok(G),
        "KEYH" | "H" => Ok(H),
        "KEYI" | "I" => Ok(I),
        "KEYJ" | "J" => Ok(J),
        "KEYK" | "K" => Ok(K),
        "KEYL" | "L" => Ok(L),
        "KEYM" | "M" => Ok(M),
        "KEYN" | "N" => Ok(N),
        "KEYO" | "O" => Ok(O),
        "KEYP" | "P" => Ok(P),
        "KEYQ" | "Q" => Ok(Q),
        "KEYR" | "R" => Ok(R),
        "KEYS" | "S" => Ok(S),
        "KEYT" | "T" => Ok(T),
        "KEYU" | "U" => Ok(U),
        "KEYV" | "V" => Ok(V),
        "KEYW" | "W" => Ok(W),
        "KEYX" | "X" => Ok(X),
        "KEYY" | "Y" => Ok(Y),
        "KEYZ" | "Z" => Ok(Z),
        "MINUS" | "-" => Ok(Minus),
        "PERIOD" | "." => Ok(Dot),
        "QUOTE" | "'" => Ok(Apostrophe),
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
        // "PRINTSCREEN" => Ok(Keycode::),
        // "SCROLLLOCK" => Ok(ScrollLock),
        "ARROWDOWN" | "DOWN" => Ok(Down),
        "ARROWLEFT" | "LEFT" => Ok(Left),
        "ARROWRIGHT" | "RIGHT" => Ok(Right),
        "ARROWUP" | "UP" => Ok(Up),
        // "NUMLOCK" => Ok(),
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
        "NUMPADEQUAL" | "NUMEQUAL" => Ok(NumpadEquals),
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
        // "AUDIOVOLUMEDOWN" | "VOLUMEDOWN" => Ok(Keycode::),
        // "AUDIOVOLUMEUP" | "VOLUMEUP" => Ok(AudioVolumeUp),
        // "AUDIOVOLUMEMUTE" | "VOLUMEMUTE" => Ok(AudioVolumeMute),
        // "MEDIAPLAY" => Ok(Media),
        // "MEDIAPAUSE" => Ok(MediaPause),
        // "MEDIAPLAYPAUSE" => Ok(MediaPlayPause),
        // "MEDIASTOP" => Ok(MediaStop),
        // "MEDIATRACKNEXT" => Ok(MediaTrackNext),
        // "MEDIATRACKPREV" | "MEDIATRACKPREVIOUS" => Ok(MediaTrackPrevious),
        "F13" => Ok(F13),
        "F14" => Ok(F14),
        "F15" => Ok(F15),
        "F16" => Ok(F16),
        "F17" => Ok(F17),
        "F18" => Ok(F18),
        "F19" => Ok(F19),
        "F20" => Ok(F20),
        // "F21" => Ok(F21),
        // "F22" => Ok(F22),
        // "F23" => Ok(F23),
        // "F24" => Ok(F24),
        _ => Err(HotKeyParseError::UnsupportedKey(key.to_string())),
    }
}
