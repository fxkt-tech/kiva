//! Casting preferences and ordered modifier state belong to the input adapter.
use bevy::prelude::KeyCode;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) enum CastMode {
    Normal,
    #[default]
    Quick,
}

impl CastMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "normal" => Some(Self::Normal),
            "quick" => Some(Self::Quick),
            _ => None,
        }
    }
}

#[derive(Default)]
pub(super) struct CastInput {
    pub mode: CastMode,
    shift: [bool; 2],
}

impl CastInput {
    pub fn key_changed(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::ShiftLeft => self.shift[0] = pressed,
            KeyCode::ShiftRight => self.shift[1] = pressed,
            _ => {}
        }
    }

    pub fn mode(&self) -> CastMode {
        if self.shift.iter().any(|pressed| *pressed) {
            CastMode::Normal
        } else {
            self.mode
        }
    }

    pub fn clear_modifiers(&mut self) {
        self.shift = [false; 2];
    }

    pub fn sync_shift(&mut self, pressed: bool) {
        self.shift = [pressed, false];
    }
}

#[derive(Clone, Copy)]
pub(super) enum CastHotkey {
    Ability(usize),
    Equipment(usize),
}

impl CastHotkey {
    pub fn from_key(key: KeyCode) -> Option<(Self, &'static str)> {
        use KeyCode::*;
        match key {
            KeyQ => Some((Self::Ability(0), "KeyQ")),
            KeyW => Some((Self::Ability(1), "KeyW")),
            KeyE => Some((Self::Ability(2), "KeyE")),
            KeyR => Some((Self::Ability(3), "KeyR")),
            Digit1 => Some((Self::Equipment(0), "Digit1")),
            Digit2 => Some((Self::Equipment(1), "Digit2")),
            Digit3 => Some((Self::Equipment(2), "Digit3")),
            Digit4 => Some((Self::Equipment(3), "Digit4")),
            Digit5 => Some((Self::Equipment(4), "Digit5")),
            Digit6 => Some((Self::Equipment(5), "Digit6")),
            _ => None,
        }
    }
}
