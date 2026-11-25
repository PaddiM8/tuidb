use std::fmt::Display;

use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, widgets::Block};

use crate::{
    keymap::Keymap,
    layout::{
        Action, Command, CommandKind, components::Component, dialogs::find_dialog::FindDialog,
        main_view::TabId,
    },
    theme::Theme,
};

pub struct TabDialog {
    keymap: Keymap,
    base_dialog: FindDialog<TabDialogEntry>,
}

#[derive(Clone)]
pub struct TabDialogEntry {
    pub tab_id: TabId,
    pub name: String,
}

impl Display for TabDialogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl TabDialog {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap: keymap.clone(),
            base_dialog: FindDialog::new(String::from("Tabs"), keymap),
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
        entries: &Vec<TabDialogEntry>,
    ) {
        let borrowed_entries: Vec<&TabDialogEntry> = entries.iter().collect();
        self.base_dialog
            .render(frame, theme, area, block, &borrowed_entries)
    }
}

impl Component for TabDialog {
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
            CommandKind::Submit if let Some(entry) = self.base_dialog.selected_value() => {
                actions.push(Action::OpenTab(entry.tab_id));
            }
            _ => return None,
        }

        Some(actions)
    }
}
