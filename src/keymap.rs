use std::collections::HashMap;

use crossterm::event::{
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};

use crate::layout::Command;

type KeyCombo = (Option<KeyModifiers>, KeyCode);

#[derive(Clone)]
pub struct Keymap {
    keybinds: HashMap<KeyCombo, Command>,
}

impl Keymap {
    pub fn default_global() -> Self {
        const CTRL: Option<KeyModifiers> = Some(KeyModifiers::CONTROL);

        let keybinds = HashMap::from([
            ((CTRL, Char('q')), Command::Quit),
            ((CTRL, Char('f')), Command::FindConnection),
            ((CTRL, Char('o')), Command::ToggleFullscreen),
            ((CTRL, Char('h')), Command::FocusLeft),
            ((CTRL, Char('k')), Command::FocusUp),
            ((CTRL, Char('l')), Command::FocusRight),
            ((CTRL, Char('j')), Command::FocusDown),
            ((CTRL, Char('n')), Command::ToggleQueryMode),
            ((CTRL, KeyCode::Left), Command::FocusLeft),
            ((CTRL, KeyCode::Up), Command::FocusUp),
            ((CTRL, KeyCode::Right), Command::FocusRight),
            ((CTRL, KeyCode::Down), Command::FocusDown),
            ((None, KeyCode::F(5)), Command::RunQuery),
            ((CTRL, KeyCode::Enter), Command::RunQuery),
        ]);

        Self { keybinds }
    }

    pub fn default_selection() -> Self {
        const CTRL: Option<KeyModifiers> = Some(KeyModifiers::CONTROL);

        let keybinds = HashMap::from([
            ((None, Char('h')), Command::Left),
            ((None, Char('k')), Command::Up),
            ((None, Char('l')), Command::Right),
            ((None, Char('j')), Command::Down),
            ((None, Char('y')), Command::Copy),
            ((None, KeyCode::Enter), Command::Submit),
            ((CTRL, Char('u')), Command::HalfPageUp),
            ((CTRL, Char('d')), Command::HalfPageDown),
        ]);

        Self { keybinds }
    }

    pub fn default_dialog() -> Self {
        const CTRL: Option<KeyModifiers> = Some(KeyModifiers::CONTROL);

        let keybinds = HashMap::from([
            ((None, KeyCode::Esc), Command::CloseDialog),
            ((None, KeyCode::Enter), Command::Submit),
            ((CTRL, Char('c')), Command::CloseDialog),
        ]);

        Self { keybinds }
    }

    pub fn default_connections() -> Self {
        const CTRL: Option<KeyModifiers> = Some(KeyModifiers::CONTROL);

        let mut base = Keymap::default_dialog();
        base.keybinds
            .insert((CTRL, Char('e')), Command::EditSelectedConnection);
        base.keybinds
            .insert((CTRL, Char('d')), Command::NewConnection);
        base.keybinds.insert((None, KeyCode::Up), Command::Up);
        base.keybinds.insert((None, KeyCode::Down), Command::Down);
        base.keybinds.insert((CTRL, Char('p')), Command::Up);
        base.keybinds.insert((CTRL, Char('n')), Command::Down);

        base
    }

    pub fn resolve_event(&self, event: KeyEvent) -> Option<Command> {
        let modifier = if event.modifiers.is_empty() {
            None
        } else {
            Some(event.modifiers)
        };

        let key_combo = (modifier, event.code);

        self.keybinds.get(&key_combo).cloned()
    }
}
