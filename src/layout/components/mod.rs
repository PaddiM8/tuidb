use edtui::{
    EditorMode,
    actions::{
        Action::{DeleteToEndOfLine, DeleteWordBackward, SwitchMode},
        Chainable,
    },
    events::{KeyEventHandler, KeyEventRegister, KeyInput},
};
use ratatui::crossterm::event::KeyEvent;

use crate::layout::Command;

pub mod data_grid;
pub mod filter_bar;
pub mod query_editor;
pub mod sidebar;

pub trait Component {
    type Output;

    fn handle_key(&mut self, key: KeyEvent) -> Vec<Self::Output>;

    fn handle_command(&mut self, command: Command) -> Option<Vec<Self::Output>>;
}

pub fn build_edtui_key_handler() -> KeyEventHandler {
    let mut key_handler = KeyEventHandler::vim_mode();
    key_handler.insert(
        KeyEventRegister::i(vec![KeyInput::ctrl('w')]),
        DeleteWordBackward(edtui::actions::DeleteWordBackward(1)),
    );
    key_handler.insert(
        KeyEventRegister::n(vec![KeyInput::shift('C')]),
        DeleteToEndOfLine(edtui::actions::delete::DeleteToEndOfLine)
            .chain(SwitchMode(edtui::actions::SwitchMode(EditorMode::Insert))),
    );

    key_handler
}
