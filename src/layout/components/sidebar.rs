use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::*};

use crate::{
    keymap::Keymap,
    layout::{CommandKind, components::Component},
    theme::Theme,
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

    pub fn render(
        &mut self,
        frame: &mut Frame,
        theme: &Theme,
        area: Rect,
        block: Block,
        items: &Vec<String>,
    ) {
        let list = List::new(items.iter().map(|s| s.as_str()))
            .block(block)
            .highlight_style(theme.item_selected);

        if self.state.selected().is_some() {
            self.last_selected = self.state.selected();
        }

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}

impl Component for Sidebar {
    type Output = SidebarEvent;

    fn handle_key(&mut self, event: KeyEvent) -> Vec<SidebarEvent> {
        let mut actions = Vec::new();
        for command in self.keymap.resolve_event(event) {
            match command.kind {
                CommandKind::Down => {
                    for _ in 0..command.multiplier {
                        self.state.select_next();
                    }
                }
                CommandKind::Up => {
                    for _ in 0..command.multiplier {
                        self.state.select_previous();
                    }
                }
                CommandKind::Submit => {
                    if let Some(index) = self.state.selected() {
                        actions.push(SidebarEvent::SubmittedIndex(index));
                    }
                }
                _ => (),
            }
        }

        actions
    }
}
