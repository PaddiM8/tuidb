use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, widgets::Block};

use crate::{
    config::ConnectionEntry,
    keymap::Keymap,
    layout::{
        Action, Command, CommandKind, components::Component, dialogs::find_dialog::FindDialog,
    },
    theme::Theme,
};

pub struct ConnectionsDialog {
    keymap: Keymap,
    base_dialog: FindDialog<String>,
}

impl ConnectionsDialog {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap: keymap.clone(),
            base_dialog: FindDialog::new(String::from("Connections"), keymap),
        }
    }

    pub fn reset(&mut self) {
        self.base_dialog.reset();
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        theme: &Theme,
        area: Rect,
        block: Block,
        connections: &Vec<ConnectionEntry>,
    ) {
        let connection_names: Vec<&String> = connections.iter().map(|c| &c.name).collect();
        self.base_dialog
            .render(frame, theme, area, block, &connection_names)
    }
}

impl Component for ConnectionsDialog {
    type Output = Action;

    fn handle_key(&mut self, event: KeyEvent) -> Vec<Action> {
        let mut actions: Vec<Action> = self
            .keymap
            .resolve_event(event)
            .into_iter()
            .filter_map(|c| self.handle_command(c))
            .flatten()
            .collect();
        actions.extend(self.base_dialog.handle_key(event));

        actions
    }

    fn handle_command(&mut self, command: Command) -> Option<Vec<Action>> {
        let mut actions = Vec::new();
        match command.kind {
            CommandKind::Unresolved(events) => actions.extend(
                events
                    .into_iter()
                    .flat_map(|e| self.base_dialog.handle_key(e)),
            ),
            CommandKind::Submit
                if let Some(connection_name) = self.base_dialog.selected_value() =>
            {
                actions.push(Action::OpenConnection(connection_name.to_string()));
            }
            _ => return None,
        }

        Some(actions)
    }
}
