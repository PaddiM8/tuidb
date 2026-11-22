use crossterm::event::KeyEvent;
use ratatui::{Frame, widgets::Block};

use crate::{
    layout::{Action, View},
    theme::Theme,
};

pub struct StartView {}

impl StartView {
    pub fn new() -> Self {
        Self {}
    }
}

impl StartView {
    pub fn render(&mut self, frame: &mut Frame, theme: &Theme) {
        frame.render_widget(Block::new().style(theme.style), frame.area());
    }
}

impl View for StartView {
    fn handle_key(&mut self, _: KeyEvent) -> Vec<Action> {
        Vec::new()
    }
}
