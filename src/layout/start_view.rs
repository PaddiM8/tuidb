use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    widgets::{Block, Clear},
};

use crate::layout::{Job, View};

pub struct StartView {}

impl StartView {
    pub fn new() -> Self {
        Self {}
    }
}

impl StartView {
    pub fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(Clear, frame.area());
    }
}

impl View for StartView {
    fn handle_key(&mut self, _: KeyEvent) -> Option<Job> {
        None
    }
}
