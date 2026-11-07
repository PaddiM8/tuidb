use std::collections::HashMap;

use crossterm::event::{
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};

use crate::layout::Action;

type KeyCombo = (Option<KeyModifiers>, KeyCode);

#[derive(Clone)]
pub struct Keymap {
    keybinds: HashMap<KeyCombo, Action>,
}

impl Keymap {
    pub fn new() -> Self {
        Self {
            keybinds: HashMap::new(),
        }
    }

    pub fn default_global() -> Self {
        const CTRL: Option<KeyModifiers> = Some(KeyModifiers::CONTROL);

        let keybinds = HashMap::from([
            ((None, Char('q')), Action::Quit),
            ((CTRL, Char('f')), Action::FindConnection),
            ((CTRL, Char('o')), Action::ToggleFullscreen),
            ((CTRL, Char('h')), Action::FocusLeft),
            ((CTRL, Char('k')), Action::FocusUp),
            ((CTRL, Char('l')), Action::FocusRight),
            ((CTRL, Char('j')), Action::FocusDown),
        ]);

        Self { keybinds }
    }

    pub fn default_selection() -> Self {
        const CTRL: Option<KeyModifiers> = Some(KeyModifiers::CONTROL);

        let keybinds = HashMap::from([
            ((None, Char('h')), Action::Left),
            ((None, Char('k')), Action::Up),
            ((None, Char('l')), Action::Right),
            ((None, Char('j')), Action::Down),
            ((CTRL, Char('u')), Action::HalfPageUp),
            ((CTRL, Char('d')), Action::HalfPageDown),
        ]);

        Self { keybinds }
    }

    pub fn default_dialog() -> Self {
        const CTRL: Option<KeyModifiers> = Some(KeyModifiers::CONTROL);

        let keybinds = HashMap::from([
            ((None, KeyCode::Esc), Action::CloseDialog),
            ((None, KeyCode::Enter), Action::Submit),
            ((CTRL, Char('c')), Action::CloseDialog),
        ]);

        Self { keybinds }
    }

    pub fn default_connections() -> Self {
        const CTRL: Option<KeyModifiers> = Some(KeyModifiers::CONTROL);

        let mut base = Keymap::default_dialog();
        base.keybinds
            .insert((CTRL, Char('e')), Action::EditSelectedConnection);
        base.keybinds
            .insert((CTRL, Char('d')), Action::NewConnection);
        base.keybinds.insert((None, KeyCode::Up), Action::Up);
        base.keybinds.insert((None, KeyCode::Down), Action::Down);
        base.keybinds.insert((CTRL, Char('p')), Action::Up);
        base.keybinds.insert((CTRL, Char('n')), Action::Down);

        base
    }

    pub fn resolve_event(&self, event: KeyEvent) -> Option<Action> {
        let modifier = if event.modifiers.is_empty() {
            None
        } else {
            Some(event.modifiers)
        };

        let key_combo = (modifier, event.code);

        self.keybinds.get(&key_combo).cloned()
    }
}
