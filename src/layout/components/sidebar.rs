use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::*};

use crate::{
    keymap::Keymap,
    layout::{Command, components::Component},
};

pub enum SidebarEvent {
    SubmittedIndex(usize),
}

pub struct Sidebar {
    state: ListState,
    keymap: Keymap,
    last_selected: Option<usize>,
}

impl Sidebar {
    pub fn new(keymap: Keymap) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            keymap,
            last_selected: None,
        }
    }

    pub fn focus(&mut self) {
        if self.state.selected().is_none() {
            self.state.select(self.last_selected.or(Some(0)));
        }
    }

    pub fn deselect(&mut self) {
        self.state.select(None);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, block: Block, items: &Vec<String>) {
        let list = List::new(items.iter().map(|s| s.as_str()))
            .block(block)
            .highlight_style(Style::new().on_dark_gray());

        if self.state.selected().is_some() {
            self.last_selected = self.state.selected();
        }

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}

impl Component for Sidebar {
    type Output = SidebarEvent;

    fn handle_key(&mut self, event: KeyEvent) -> Option<SidebarEvent> {
        match self.keymap.resolve_event(event) {
            Some(Command::Down) => {
                self.state.select_next();
                None
            }
            Some(Command::Up) => {
                self.state.select_previous();
                None
            }
            Some(Command::Submit) => self
                .state
                .selected()
                .map(|i| SidebarEvent::SubmittedIndex(i)),
            _ => None,
        }
    }
}
