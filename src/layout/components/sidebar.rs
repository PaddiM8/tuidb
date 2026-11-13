use std::any::Any;

use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::*};

use crate::{
    keymap::Keymap,
    layout::{
        components::Component,
        {Action, Job},
    },
};

pub struct Sidebar {
    items: Vec<String>,
    state: ListState,
    keymap: Keymap,
}

impl Sidebar {
    pub fn new(items: Vec<String>, keymap: Keymap) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            items,
            state,
            keymap,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, block: Block) {
        let list = List::new(self.items.iter().map(|s| s.as_str()))
            .block(block)
            .highlight_style(Style::new().reversed());
        frame.render_stateful_widget(list, area, &mut self.state);
    }
}

impl Component for Sidebar {
    fn handle_key(&mut self, event: KeyEvent) -> Option<Job> {
        match self.keymap.resolve_event(event) {
            Some(Action::Down) => {
                self.state.select_previous();
                None
            }
            Some(Action::Up) => {
                self.state.select_next();
                None
            }
            Some(Action::Submit) => self
                .state
                .selected()
                .map(|i| Job::OpenTable(self.items[i].clone())),
            _ => None,
        }
    }
}
