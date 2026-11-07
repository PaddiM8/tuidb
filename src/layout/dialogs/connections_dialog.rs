use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect, widgets::Block};

use crate::{
    keymap::Keymap,
    layout::{Action, Job, components::Component, dialogs::find_dialog::FindDialog},
};

pub struct ConnectionsDialog {
    keymap: Keymap,
    base_dialog: FindDialog,
}

impl ConnectionsDialog {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap: keymap.clone(),
            base_dialog: FindDialog::new(String::from("Connections"), keymap),
        }
    }
}

impl Component for ConnectionsDialog {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn handle_key(&mut self, event: KeyEvent) -> Option<Job> {
        if let Some(Action::Submit) = self.keymap.resolve_event(event)
            && let Some(connection_name) = self.base_dialog.selected_value()
        {
            return Some(Job::OpenConnection(connection_name));
        }

        self.base_dialog.handle_key(event)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, block: Block) {
        self.base_dialog.render(frame, area, block)
    }
}
