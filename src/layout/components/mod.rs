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
