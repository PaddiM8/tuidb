use ratatui::crossterm::event::KeyEvent;

pub mod data_grid;
pub mod filter_bar;
pub mod query_editor;
pub mod sidebar;

pub trait Component {
    type Output;

    fn handle_key(&mut self, key: KeyEvent) -> Option<Self::Output>;
}
