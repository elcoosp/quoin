//! Keyboard accelerators: parsed from strings, stored in memory.

use std::fmt;
use std::str::FromStr;

/// Keyboard code re-exported from `keyboard-types`, mirroring muda.
pub type Code = keyboard_types::Code;

/// Modifier flags re-exported from `keyboard-types`, mirroring muda.
pub type Modifiers = keyboard_types::Modifiers;

/// A keyboard shortcut (modifiers + key), mirroring `muda::Accelerator`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accelerator {
    /// Modifier keys held.
    pub mods: Option<Modifiers>,
    /// Main key.
    pub key: Code,
}

impl Accelerator {
    /// Creates an accelerator from modifiers + key.
    pub fn new(mods: Option<Modifiers>, key: Code) -> Self {
        Self { mods, key }
    }
}

fn parse_modifiers_single(token: &str) -> Option<Modifiers> {
    match token.to_ascii_lowercase().as_str() {
        "ctrl" | "control" | "cmd" | "command" | "cmdorctrl" => Some(Modifiers::CONTROL),
        "shift" => Some(Modifiers::SHIFT),
        "alt" | "option" | "opt" => Some(Modifiers::ALT),
        "super" | "meta" => Some(Modifiers::META),
        _ => None,
    }
}

/// Accepts both `Code` variant names (`KeyA`, `Digit1`, `ArrowUp`) and
/// convenient aliases (`a`, `1`, `up`).
fn parse_code(token: &str) -> Option<Code> {
    let lower = token.to_ascii_lowercase();
    let aliased = match lower.as_str() {
        "a" => "keya",
        "b" => "keyb",
        "c" => "keyc",
        "d" => "keyd",
        "e" => "keye",
        "f" => "keyf",
        "g" => "keyg",
        "h" => "keyh",
        "i" => "keyi",
        "j" => "keyj",
        "k" => "keyk",
        "l" => "keyl",
        "m" => "keym",
        "n" => "keyn",
        "o" => "keyo",
        "p" => "keyp",
        "q" => "keyq",
        "r" => "keyr",
        "s" => "keys",
        "t" => "keyt",
        "u" => "keyu",
        "v" => "keyv",
        "w" => "keyw",
        "x" => "keyx",
        "y" => "keyy",
        "z" => "keyz",
        "0" => "digit0",
        "1" => "digit1",
        "2" => "digit2",
        "3" => "digit3",
        "4" => "digit4",
        "5" => "digit5",
        "6" => "digit6",
        "7" => "digit7",
        "8" => "digit8",
        "9" => "digit9",
        "up" => "arrowup",
        "down" => "arrowdown",
        "left" => "arrowleft",
        "right" => "arrowright",
        "space" => "space",
        "enter" => "enter",
        "escape" | "esc" => "escape",
        "tab" => "tab",
        "backspace" => "backspace",
        "delete" => "delete",
        other => other,
    };
    let mut owned = aliased.to_string();
    owned[..1].make_ascii_uppercase();
    Code::from_str(&owned).ok()
}

impl FromStr for Accelerator {
    type Err = String;

    /// Parses forms like `"Ctrl+KeyA"`, `"Shift+Alt+KeyQ"` or `"KeyP"`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mods = Modifiers::empty();
        let mut saw_modifier = false;
        let mut key: Option<Code> = None;
        for token in s.split(['+', '|']) {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            if let Some(m) = parse_modifiers_single(token) {
                mods |= m;
                saw_modifier = true;
            } else {
                key = Some(parse_code(token).ok_or_else(|| format!("unknown key `{token}`"))?);
            }
        }
        let Some(key) = key else {
            return Err(format!("accelerator `{s}` has no key"));
        };
        Ok(Self {
            mods: if saw_modifier { Some(mods) } else { None },
            key,
        })
    }
}

impl fmt::Display for Accelerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(mods) = self.mods {
            if mods.contains(Modifiers::CONTROL) {
                write!(f, "Ctrl+")?;
            }
            if mods.contains(Modifiers::SHIFT) {
                write!(f, "Shift+")?;
            }
            if mods.contains(Modifiers::ALT) {
                write!(f, "Alt+")?;
            }
            if mods.contains(Modifiers::META) {
                write!(f, "Super+")?;
            }
        }
        write!(f, "{}", self.key)
    }
}
