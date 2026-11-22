use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crossterm::event::{
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};

use crate::layout::{Command, CommandKind};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct KeyCombo {
    modifiers: Option<KeyModifiers>,
    key: KeyCode,
    second_key: Option<KeyCode>,
}

fn single(key: KeyCode) -> KeyCombo {
    KeyCombo {
        modifiers: None,
        key: key,
        second_key: None,
    }
}

fn ctrl(key: KeyCode) -> KeyCombo {
    KeyCombo {
        modifiers: Some(KeyModifiers::CONTROL),
        key: key,
        second_key: None,
    }
}

fn shift(key: KeyCode) -> KeyCombo {
    KeyCombo {
        modifiers: Some(KeyModifiers::SHIFT),
        key: key,
        second_key: None,
    }
}

fn double(set: &mut HashSet<KeyEvent>, key: KeyCode, second_key: KeyCode) -> KeyCombo {
    set.insert(KeyEvent::from(key));

    KeyCombo {
        modifiers: None,
        key: key,
        second_key: Some(second_key),
    }
}

#[derive(Clone)]
pub struct Keymap {
    keybinds: HashMap<KeyCombo, CommandKind>,
    double_keys: HashSet<KeyEvent>,
    buffer: Vec<KeyEvent>,
    last_unresolved_key_press: Option<Instant>,
}

impl Keymap {
    pub fn default_global() -> Self {
        let keybinds = HashMap::from([
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

        Self {
            keybinds,
            double_keys: HashSet::new(),
            buffer: Vec::new(),
            last_unresolved_key_press: None,
        }
    }

    pub fn default_selection() -> Self {
        let mut double_set = HashSet::new();
        let keybinds = HashMap::from([
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
                double(&mut double_set, Char('g'), Char('g')),
                CommandKind::PageTop,
            ),
            (shift(Char('G')), CommandKind::PageBottom),
        ]);

        Self {
            keybinds,
            double_keys: double_set,
            buffer: Vec::new(),
            last_unresolved_key_press: None,
        }
    }

    pub fn default_dialog() -> Self {
        let keybinds = HashMap::from([
            (single(KeyCode::Esc), CommandKind::CloseDialog),
            (single(KeyCode::Enter), CommandKind::Submit),
            (ctrl(Char('c')), CommandKind::CloseDialog),
        ]);

        Self {
            keybinds,
            double_keys: HashSet::new(),
            buffer: Vec::new(),
            last_unresolved_key_press: None,
        }
    }

    pub fn default_connections() -> Self {
        let mut base = Keymap::default_dialog();
        base.keybinds
            .insert(ctrl(Char('e')), CommandKind::EditSelectedConnection);
        base.keybinds
            .insert(ctrl(Char('d')), CommandKind::NewConnection);
        base.keybinds.insert(single(KeyCode::Up), CommandKind::Up);
        base.keybinds
            .insert(single(KeyCode::Down), CommandKind::Down);
        base.keybinds.insert(ctrl(Char('p')), CommandKind::Up);
        base.keybinds.insert(ctrl(Char('n')), CommandKind::Down);

        base
    }

    pub fn resolve_event(&mut self, event: KeyEvent) -> Option<Command> {
        let modifier = if event.modifiers.is_empty() {
            None
        } else {
            Some(event.modifiers)
        };

        if self
            .last_unresolved_key_press
            .is_some_and(|i| i.elapsed() > Duration::from_millis(750))
        {
            self.buffer.clear();
            self.last_unresolved_key_press = None;
        }

        if modifier.is_none() && event.code.as_char().is_some_and(|c| c.is_ascii_digit()) {
            self.buffer.push(event);
            self.last_unresolved_key_press = Some(Instant::now());
            return None;
        }

        let digits: String = self
            .buffer
            .iter()
            .take_while(|e| e.code.as_char().is_some_and(|c| c.is_ascii_digit()))
            .map(|e| e.code.as_char().unwrap())
            .collect();
        let other: Option<&KeyEvent> = self.buffer.iter().skip(digits.len()).last();

        if other.is_none() && self.double_keys.contains(&event) {
            self.buffer.push(event);
            self.last_unresolved_key_press = Some(Instant::now());

            return None;
            // TODO: After the duration has passed and no second key was pressed, asynchronously trigger the key press
            /*if let Some(unresolved_key) = &self.unresolved_key
                && unresolved_key.1.elapsed() < Duration::from_millis(750)
            {
                let key_combo = KeyCombo {
                    modifiers: None,
                    key: unresolved_key.0,
                    second_key: Some(event.code),
                };

                return self.keybinds.get(&key_combo).cloned();
            } else {
                self.unresolved_key = Some(UnresolvedKey(event.code.clone(), Instant::now()));
            }*/
        }

        let key_combo = if let Some(other) = other {
            KeyCombo {
                modifiers: modifier,
                key: other.code,
                second_key: Some(event.code),
            }
        } else {
            KeyCombo {
                modifiers: modifier,
                key: event.code,
                second_key: None,
            }
        };

        self.last_unresolved_key_press = None;
        self.buffer.clear();

        if let Some(kind) = self.keybinds.get(&key_combo).cloned() {
            Some(Command {
                kind: kind,
                multiplier: digits.parse().unwrap_or(1),
            })
        } else {
            None
        }
    }
}
