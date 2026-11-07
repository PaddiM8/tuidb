use std::any::Any;

use crate::layout::Job;
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect, widgets::Block};

pub mod data_grid;
pub mod query_editor;
pub mod sidebar;

pub trait Component {
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn handle_key(&mut self, key: KeyEvent) -> Option<Job>;

    fn render(&mut self, frame: &mut Frame, area: Rect, block: Block);
}
