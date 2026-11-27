use std::fmt::Display;

use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::*};

use crate::{
    keymap::Keymap,
    layout::{Command, CommandKind, components::Component},
    theme::Theme,
};

pub enum SelectableListEvent {
    SubmittedIndex(usize),
}

pub struct SelectableList {
    state: ListState,
    keymap: Keymap,
    last_selected: Option<usize>,
}

impl SelectableList {
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

    pub fn select(&mut self, index: usize) {
        self.state.select(Some(index));
    }

    pub fn deselect(&mut self) {
        self.state.select(None);
    }

    pub fn render<T: Display>(
        &mut self,
        frame: &mut Frame,
        theme: &Theme,
        area: Rect,
        block: Block,
        items: &[T],
    ) {
        let list = List::new(items.iter().map(|s| s.to_string()))
            .block(block)
            .highlight_style(theme.item_selected);

        if self.state.selected().is_some() {
            self.last_selected = self.state.selected();
        }

        frame.render_stateful_widget(list, area, &mut self.state);
    }
}

impl Component for SelectableList {
    type Output = SelectableListEvent;

    fn handle_key(&mut self, event: KeyEvent) -> Vec<SelectableListEvent> {
        self.keymap
            .resolve_event(event)
            .into_iter()
            .filter_map(|c| self.handle_command(c))
            .flatten()
            .collect()
    }

    fn handle_command(&mut self, command: Command) -> Option<Vec<SelectableListEvent>> {
        let mut actions = Vec::new();
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
                    actions.push(SelectableListEvent::SubmittedIndex(index));
                }
            }
            _ => return None,
        }

        Some(actions)
    }
}
