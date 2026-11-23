use crossterm::event::KeyEvent;
use edtui::actions::Action::DeleteWordBackward;
use edtui::events::{KeyEventHandler, KeyEventRegister, KeyInput};
use edtui::{EditorEventHandler, EditorState, EditorTheme, EditorView};
use edtui::{EditorMode, SyntaxHighlighter};
use ratatui::DefaultTerminal;
use ratatui::{Frame, layout::Rect, widgets::Block};

use crate::keymap::Keymap;
use crate::layout::components::{Component, build_edtui_key_handler};
use crate::layout::{Action, Command, CommandKind, Mode};
use crate::theme::Theme;

pub struct QueryEditor {
    state: EditorState,
    events: EditorEventHandler,
    keymap: Keymap,
}

impl QueryEditor {
    pub fn new(keymap: Keymap) -> Self {
        let mut editor_state = EditorState::default();
        editor_state.mode = edtui::EditorMode::Insert;

        let key_handler = build_edtui_key_handler();
        let event_handler = EditorEventHandler::new(key_handler);

        Self {
            state: editor_state,
            events: event_handler,
            keymap: keymap,
        }
    }

    pub fn mode(&self) -> Mode {
        match self.state.mode {
            EditorMode::Normal => Mode::Normal,
            EditorMode::Insert => Mode::Insert,
            EditorMode::Visual => Mode::Visual,
            EditorMode::Search => Mode::Search,
        }
    }

    pub fn text(&self) -> String {
        self.state.lines.to_string()
    }

    pub fn is_system_editor_pending(&self) -> bool {
        edtui::system_editor::is_pending(&self.state)
    }

    pub fn open_system_editor(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        edtui::system_editor::open(&mut self.state, terminal)
    }

    pub fn render(&mut self, frame: &mut Frame, theme: &Theme, area: Rect, block: Block) {
        let theme = EditorTheme::default()
            .base(theme.style)
            .hide_status_line()
            .block(block);
        let syntax_highlighter = SyntaxHighlighter::new("kronuz", "sql").ok();

        let view = EditorView::new(&mut self.state)
            .theme(theme)
            .syntax_highlighter(syntax_highlighter);
        frame.render_widget(view, area);
    }
}

impl Component for QueryEditor {
    type Output = Action;

    fn handle_key(&mut self, event: KeyEvent) -> Vec<Action> {
        self.keymap
            .resolve_event_with_mode(event, self.mode())
            .into_iter()
            .filter_map(|c| self.handle_command(c))
            .flatten()
            .collect()
    }

    fn handle_command(&mut self, command: Command) -> Option<Vec<Action>> {
        let mut actions = Vec::new();
        match command.kind {
            CommandKind::Unresolved(events) => {
                for event in events {
                    self.events.on_key_event(event, &mut self.state);
                }
            }
            CommandKind::WaitForNextKey(fallback) => {
                actions.push(Action::WaitForNextKey(*fallback))
            }
            CommandKind::RawKey(key) => {
                self.events
                    .on_key_event(KeyEvent::from(key), &mut self.state);
            }
            CommandKind::RunQuery => actions.push(Action::RunRawQuery(self.text())),
            CommandKind::NormalMode => {
                self.state.mode = EditorMode::Normal;
            }
            _ => return None,
        }

        Some(actions)
    }
}
