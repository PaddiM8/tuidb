use crossterm::event::{Event, KeyEvent};
use ratatui::{prelude::*, text::ToText, widgets::*};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    keymap::Keymap,
    layout::{Action, Command, CommandKind, components::Component},
};

pub struct CommandBar {
    state: Input,
    keymap: Keymap,
}

impl CommandBar {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            state: Input::default(),
            keymap,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let width = area.width.saturating_sub(2) as usize - 1;
        let scroll = self.state.visual_scroll(width);

        let input = Paragraph::new(format!(":{}", self.state.value())).scroll((0, scroll as u16));
        frame.render_widget(input, area);

        let x = self.state.visual_cursor().max(scroll) - scroll + 1;
        frame.set_cursor_position((area.x + x as u16, area.y));
    }
}

impl Component for CommandBar {
    type Output = Action;

    fn handle_key(&mut self, event: KeyEvent) -> Vec<Action> {
        let actions: Vec<Action> = self
            .keymap
            .resolve_event(event)
            .into_iter()
            .filter_map(|c| self.handle_command(c))
            .flatten()
            .collect();
        if actions.len() == 0 {
            self.state.handle_event(&Event::Key(event));

            Vec::new()
        } else {
            actions
        }
    }

    fn handle_command(&mut self, command: Command) -> Option<Vec<Action>> {
        let mut actions = Vec::new();
        match command.kind {
            CommandKind::CloseCommandBar => {
                actions.push(Action::CloseCommandBar);
            }
            CommandKind::Submit => {
                actions.push(Action::HandleCommandString(
                    self.state.to_text().to_string(),
                ));
                actions.push(Action::CloseCommandBar);
            }
            _ => (),
        }

        Some(actions)
    }
}
