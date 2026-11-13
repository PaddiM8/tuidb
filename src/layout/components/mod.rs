use crate::layout::Job;
use ratatui::crossterm::event::KeyEvent;

pub mod data_grid;
pub mod query_editor;
pub mod sidebar;

pub trait Component {
    fn handle_key(&mut self, key: KeyEvent) -> Option<Job>;
}
