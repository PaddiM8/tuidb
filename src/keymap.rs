use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use crossterm::event::{
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};

use crate::layout::Command;

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

fn double(set: &mut HashSet<KeyCode>, key: KeyCode, second_key: KeyCode) -> KeyCombo {
    set.insert(key);

    KeyCombo {
        modifiers: None,
        key: key,
        second_key: Some(second_key),
    }
}

#[derive(Clone)]
struct UnresolvedKey(KeyCode, Instant);

#[derive(Clone)]
pub struct Keymap {
    keybinds: HashMap<KeyCombo, Command>,
    double_keys: HashSet<KeyCode>,
    unresolved_key: Option<UnresolvedKey>,
}

impl Keymap {
    pub fn default_global() -> Self {
        let keybinds = HashMap::from([
            (ctrl(Char('q')), Command::Quit),
            (ctrl(Char('f')), Command::FindConnection),
            (ctrl(Char('o')), Command::ToggleFullscreen),
            (ctrl(Char('h')), Command::FocusLeft),
            (ctrl(Char('k')), Command::FocusUp),
            (ctrl(Char('l')), Command::FocusRight),
            (ctrl(Char('j')), Command::FocusDown),
            (ctrl(Char('n')), Command::ToggleQueryMode),
            (ctrl(KeyCode::Left), Command::FocusLeft),
            (ctrl(KeyCode::Up), Command::FocusUp),
            (ctrl(KeyCode::Right), Command::FocusRight),
            (ctrl(KeyCode::Down), Command::FocusDown),
            (ctrl(KeyCode::Enter), Command::RunQuery),
            (single(KeyCode::F(5)), Command::RunQuery),
        ]);

        Self {
            keybinds,
            double_keys: HashSet::new(),
            unresolved_key: None,
        }
    }

    pub fn default_selection() -> Self {
        let mut double_set = HashSet::new();
        let keybinds = HashMap::from([
            (single(Char('h')), Command::Left),
            (single(Char('k')), Command::Up),
            (single(Char('l')), Command::Right),
            (single(Char('j')), Command::Down),
            (single(Char('y')), Command::Copy),
            (single(Char('s')), Command::Sort),
            (single(KeyCode::Enter), Command::Submit),
            (ctrl(Char('u')), Command::HalfPageUp),
            (ctrl(Char('d')), Command::HalfPageDown),
            (
                double(&mut double_set, Char('g'), Char('g')),
                Command::PageTop,
            ),
            (shift(Char('G')), Command::PageBottom),
        ]);

        Self {
            keybinds,
            double_keys: double_set,
            unresolved_key: None,
        }
    }

    pub fn default_dialog() -> Self {
        let keybinds = HashMap::from([
            (single(KeyCode::Esc), Command::CloseDialog),
            (single(KeyCode::Enter), Command::Submit),
            (ctrl(Char('c')), Command::CloseDialog),
        ]);

        Self {
            keybinds,
            double_keys: HashSet::new(),
            unresolved_key: None,
        }
    }

    pub fn default_connections() -> Self {
        let mut base = Keymap::default_dialog();
        base.keybinds
            .insert(ctrl(Char('e')), Command::EditSelectedConnection);
        base.keybinds
            .insert(ctrl(Char('d')), Command::NewConnection);
        base.keybinds.insert(single(KeyCode::Up), Command::Up);
        base.keybinds.insert(single(KeyCode::Down), Command::Down);
        base.keybinds.insert(ctrl(Char('p')), Command::Up);
        base.keybinds.insert(ctrl(Char('n')), Command::Down);

        base
    }

    pub fn resolve_event(&mut self, event: KeyEvent) -> Option<Command> {
        let modifier = if event.modifiers.is_empty() {
            None
        } else {
            Some(event.modifiers)
        };

        if modifier.is_none() && self.double_keys.contains(&event.code) {
            // TODO: After the duration has passed and no second key was pressed, asynchronously trigger the key press
            if let Some(unresolved_key) = &self.unresolved_key
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
            }
        }

        let key_combo = KeyCombo {
            modifiers: modifier,
            key: event.code,
            second_key: None,
        };

        self.keybinds.get(&key_combo).cloned()
    }
}
