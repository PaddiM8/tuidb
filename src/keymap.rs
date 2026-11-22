use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    time::{Duration, Instant},
};

use color_eyre::eyre::{Context, Result};
use crossterm::event::{
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};

use crate::{
    config::KeybindConfigEntry,
    layout::{Command, CommandKind, Mode},
};

#[derive(Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyCombo {
    modifiers: Option<KeyModifiers>,
    key: KeyCode,
    second_key: Option<KeyCode>,
    mode: Mode,
}

fn single(key: KeyCode) -> KeyCombo {
    KeyCombo {
        modifiers: None,
        key: key,
        second_key: None,
        mode: Mode::Normal,
    }
}

fn ctrl(key: KeyCode) -> KeyCombo {
    KeyCombo {
        modifiers: Some(KeyModifiers::CONTROL),
        key: key,
        second_key: None,
        mode: Mode::Normal,
    }
}

fn shift(key: KeyCode) -> KeyCombo {
    KeyCombo {
        modifiers: Some(KeyModifiers::SHIFT),
        key: key,
        second_key: None,
        mode: Mode::Normal,
    }
}

fn double(
    double_keys_by_mode: &mut HashMap<Mode, HashSet<KeyEvent>>,
    key: KeyCode,
    second_key: KeyCode,
) -> KeyCombo {
    double_with_mode(double_keys_by_mode, key, second_key, Mode::Normal)
}

fn double_with_mode(
    double_keys_by_mode: &mut HashMap<Mode, HashSet<KeyEvent>>,
    key: KeyCode,
    second_key: KeyCode,
    mode: Mode,
) -> KeyCombo {
    if let Some(set) = double_keys_by_mode.get_mut(&mode) {
        set.insert(KeyEvent::from(key));
    } else {
        double_keys_by_mode.insert(mode, HashSet::from([KeyEvent::from(key)]));
    }

    KeyCombo {
        modifiers: None,
        key: key,
        second_key: Some(second_key),
        mode: mode,
    }
}

#[derive(Clone)]
pub struct Keymap {
    keybinds: HashMap<KeyCombo, CommandKind>,
    double_keys_by_mode: HashMap<Mode, HashSet<KeyEvent>>,
    buffer: Vec<KeyEvent>,
    last_unresolved_key_press: Option<Instant>,
}

fn add_keybinds(
    target: &mut HashMap<KeyCombo, CommandKind>,
    double_keys_by_mode: &mut HashMap<Mode, HashSet<KeyEvent>>,
    keybinds: &Vec<KeybindConfigEntry>,
) {
    for keybind in keybinds {
        let split_by_plus: Vec<&str> = keybind.binding.split('+').collect();

        let mut modifiers: Option<KeyModifiers> = None;
        let keys: Vec<&str> = if split_by_plus.len() > 1 {
            modifiers = match split_by_plus[0].to_lowercase().as_ref() {
                "ctrl" => Some(KeyModifiers::CONTROL),
                "shift" => Some(KeyModifiers::SHIFT),
                "alt" => Some(KeyModifiers::ALT),
                _ => None,
            };

            split_by_plus[1].split(' ').collect()
        } else {
            split_by_plus[0].split(' ').collect()
        };

        let key = parse_keycode(keys[0]);
        if keys.len() > 1 {
            if let Some(set) = double_keys_by_mode.get_mut(&keybind.mode) {
                set.insert(KeyEvent::from(key));
            } else {
                double_keys_by_mode.insert(keybind.mode, HashSet::from([KeyEvent::from(key)]));
            }
        }

        let key_combo = KeyCombo {
            modifiers: modifiers,
            key: key,
            second_key: keys.get(1).map(|key| parse_keycode(key)),
            mode: keybind.mode,
        };

        target.insert(key_combo, keybind.command.clone());
    }
}

fn parse_keycode(value: &str) -> KeyCode {
    if value == "" {
        return KeyCode::Null;
    }

    match value.to_lowercase().as_str() {
        "enter" => KeyCode::Enter,
        "esc" => KeyCode::Esc,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "tab" => KeyCode::Tab,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backspace" => KeyCode::Backspace,
        "insert" => KeyCode::Insert,
        "delete" => KeyCode::Delete,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        _ => KeyCode::Char(value.chars().nth(0).unwrap_or('\0')),
    }
}

impl Keymap {
    pub fn build_global(bindings: &Vec<KeybindConfigEntry>) -> Self {
        let mut keybinds = HashMap::from([
            (ctrl(Char('q')), CommandKind::Quit),
            (ctrl(Char('f')), CommandKind::FindConnection),
            (ctrl(Char('o')), CommandKind::ToggleFullscreen),
            (ctrl(Char('h')), CommandKind::FocusLeft),
            (ctrl(Char('k')), CommandKind::FocusUp),
            (ctrl(Char('l')), CommandKind::FocusRight),
            (ctrl(Char('j')), CommandKind::FocusDown),
            (ctrl(Char('n')), CommandKind::ToggleQueryMode),
            (ctrl(KeyCode::Left), CommandKind::FocusLeft),
            (ctrl(KeyCode::Up), CommandKind::FocusUp),
            (ctrl(KeyCode::Right), CommandKind::FocusRight),
            (ctrl(KeyCode::Down), CommandKind::FocusDown),
            (ctrl(KeyCode::Enter), CommandKind::RunQuery),
            (single(KeyCode::F(5)), CommandKind::RunQuery),
        ]);

        let mut double_keys_by_mode = HashMap::new();
        add_keybinds(&mut keybinds, &mut double_keys_by_mode, bindings);

        Self {
            keybinds,
            double_keys_by_mode: double_keys_by_mode,
            buffer: Vec::new(),
            last_unresolved_key_press: None,
        }
    }

    pub fn build_selection(bindings: &Vec<KeybindConfigEntry>) -> Self {
        let mut double_keys_by_mode = HashMap::new();
        let mut keybinds = HashMap::from([
            (single(Char('h')), CommandKind::Left),
            (single(Char('k')), CommandKind::Up),
            (single(Char('l')), CommandKind::Right),
            (single(Char('j')), CommandKind::Down),
            (single(Char('y')), CommandKind::Copy),
            (single(Char('s')), CommandKind::Sort),
            (single(KeyCode::Enter), CommandKind::Submit),
            (ctrl(Char('u')), CommandKind::HalfPageUp),
            (ctrl(Char('d')), CommandKind::HalfPageDown),
            (
                double(&mut double_keys_by_mode, Char('g'), Char('g')),
                CommandKind::PageTop,
            ),
            (shift(Char('G')), CommandKind::PageBottom),
        ]);

        add_keybinds(&mut keybinds, &mut double_keys_by_mode, bindings);

        Self {
            keybinds,
            double_keys_by_mode: HashMap::new(),
            buffer: Vec::new(),
            last_unresolved_key_press: None,
        }
    }

    pub fn build_dialog(bindings: &Vec<KeybindConfigEntry>) -> Self {
        let mut keybinds = HashMap::from([
            (single(KeyCode::Esc), CommandKind::CloseDialog),
            (single(KeyCode::Enter), CommandKind::Submit),
            (ctrl(Char('c')), CommandKind::CloseDialog),
        ]);

        let mut double_keys_by_mode = HashMap::new();
        add_keybinds(&mut keybinds, &mut double_keys_by_mode, bindings);

        Self {
            keybinds,
            double_keys_by_mode: HashMap::new(),
            buffer: Vec::new(),
            last_unresolved_key_press: None,
        }
    }

    pub fn build_find_dialog(bindings: &Vec<KeybindConfigEntry>) -> Self {
        let mut base = Keymap::build_dialog(bindings);
        base.keybinds.insert(single(KeyCode::Up), CommandKind::Up);
        base.keybinds
            .insert(single(KeyCode::Down), CommandKind::Down);
        base.keybinds.insert(ctrl(Char('p')), CommandKind::Up);
        base.keybinds.insert(ctrl(Char('n')), CommandKind::Down);

        base
    }

    pub fn build_editor(bindings: &Vec<KeybindConfigEntry>) -> Self {
        Keymap::build_global(bindings)
    }

    pub fn resolve_event(&mut self, event: KeyEvent) -> Vec<Command> {
        self.resolve_event_inner(event, Mode::Normal, true)
    }

    pub fn resolve_event_with_mode(&mut self, event: KeyEvent, mode: Mode) -> Vec<Command> {
        self.resolve_event_inner(event, mode, true)
    }

    fn resolve_event_inner(&mut self, event: KeyEvent, mode: Mode, buffer: bool) -> Vec<Command> {
        let modifier = if event.modifiers.is_empty() {
            None
        } else {
            Some(event.modifiers)
        };

        // Clear the buffer if the last unresolved key press was too long ago to count
        if self
            .last_unresolved_key_press
            .is_some_and(|i| i.elapsed() > Duration::from_millis(750))
        {
            self.buffer.clear();
            self.last_unresolved_key_press = None;
        }

        // If it's a digit, just add it to the buffer and return, since we need to wait
        // for an actual command to do anythin with it
        if buffer && modifier.is_none() && event.code.as_char().is_some_and(|c| c.is_ascii_digit())
        {
            self.buffer.push(event);
            self.last_unresolved_key_press = Some(Instant::now());

            return Vec::new();
        }

        let digits: String = self
            .buffer
            .iter()
            .take_while(|e| e.code.as_char().is_some_and(|c| c.is_ascii_digit()))
            .map(|e| e.code.as_char().unwrap())
            .collect();
        let other: Option<&KeyEvent> = self.buffer.iter().skip(digits.len()).last();

        // If the pressed key could be followed by a second key, add it to the buffer and return,
        // to wait for a potential second key press
        // TODO: After the duration has passed and no second key was pressed, asynchronously trigger the key press
        if buffer
            && other.is_none()
            && let Some(double_keys) = self.double_keys_by_mode.get(&mode)
            && double_keys.contains(&event)
        {
            self.buffer.push(event);
            self.last_unresolved_key_press = Some(Instant::now());

            return vec![Command {
                kind: CommandKind::WaitForNextKey(event.code),
                multiplier: 1,
            }];
        }

        let key_combo = if let Some(other) = other {
            KeyCombo {
                modifiers: modifier,
                key: other.code,
                second_key: Some(event.code),
                mode: mode,
            }
        } else {
            KeyCombo {
                modifiers: modifier,
                key: event.code,
                second_key: None,
                mode: mode,
            }
        };

        self.last_unresolved_key_press = None;
        self.buffer.clear();

        if let Some(kind) = self.keybinds.get(&key_combo).cloned() {
            vec![Command {
                kind: kind,
                multiplier: digits.parse().unwrap_or(1),
            }]
        } else if let Some(second_key) = key_combo.second_key {
            let mut commands = self.resolve_event_inner(KeyEvent::from(key_combo.key), mode, false);
            commands.extend(self.resolve_event_inner(KeyEvent::from(second_key), mode, false));

            commands
        } else {
            let mut events: Vec<KeyEvent> = digits
                .chars()
                .map(|c| KeyCode::Char(c))
                .map(|c| KeyEvent::from(c))
                .collect();
            events.push(event);

            vec![Command {
                kind: CommandKind::Unresolved(events),
                multiplier: 1,
            }]
        }
    }
}
