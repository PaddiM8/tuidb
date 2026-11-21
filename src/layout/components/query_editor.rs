use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent};
use edtui::SyntaxHighlighter;
use edtui::actions::Action::DeleteWordBackward;
use edtui::events::{KeyEventHandler, KeyEventRegister, KeyInput};
use edtui::{
    EditorEventHandler, EditorState, EditorStatusLine, EditorTheme, EditorView,
    actions::Action::DeleteChar,
};
use ratatui::DefaultTerminal;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Block,
};

use crate::keymap::Keymap;
use crate::layout::components::Component;
use crate::layout::{Action, Command};

pub struct QueryEditor {
    pub state: EditorState,
    pub events: EditorEventHandler,
    keymap: Keymap,
    last_j: Option<Instant>,
}

impl QueryEditor {
    pub fn new(keymap: Keymap) -> Self {
        let mut editor_state = EditorState::default();
        editor_state.mode = edtui::EditorMode::Insert;

        let mut key_handler = KeyEventHandler::vim_mode();
        key_handler.insert(
            KeyEventRegister::i(vec![KeyInput::ctrl('w')]),
            DeleteWordBackward(edtui::actions::DeleteWordBackward(1)),
        );
        let event_handler = EditorEventHandler::new(key_handler);

        Self {
            state: editor_state,
            events: event_handler,
            keymap: keymap,
            last_j: None,
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

    pub fn render(&mut self, frame: &mut Frame, area: Rect, block: Block) {
        let transparent = Style::default().bg(Color::Reset).fg(Color::Reset);
        let theme = EditorTheme::default()
            .base(transparent)
            .status_line(
                EditorStatusLine::default()
                    .style_mode(transparent.fg(Color::Green).add_modifier(Modifier::BOLD))
                    .style_line(transparent.fg(Color::DarkGray))
                    .style_search(transparent.fg(Color::Yellow)),
            )
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

    fn handle_key(&mut self, event: KeyEvent) -> Option<Action> {
        if let KeyCode::Char('j') = event.code {
            self.last_j = Some(Instant::now());
        }

        if let KeyCode::Char('k') = event.code
            && let Some(t) = self.last_j
            && t.elapsed() < Duration::from_millis(300)
        {
            self.state.mode = edtui::EditorMode::Normal;
            self.state
                .execute(DeleteChar(edtui::actions::DeleteChar(1)));
            self.last_j = None;
        }

        let action = match self.keymap.resolve_event(event) {
            Some(Command::RunQuery) => Some(Action::RunRawQuery(self.text())),
            _ => None,
        };

        if action.is_some() {
            return action;
        }

        self.events.on_key_event(event, &mut self.state);

        None
    }
}
