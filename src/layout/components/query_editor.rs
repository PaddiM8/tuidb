use std::any::Any;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

use crate::layout::Job;
use crate::layout::components::Component;

pub struct QueryEditor {
    pub state: EditorState,
    pub events: EditorEventHandler,
    last_j: Option<Instant>,
}

impl QueryEditor {
    pub fn new() -> Self {
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
}

impl Component for QueryEditor {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<Job> {
        if let KeyCode::Char('j') = key.code {
            self.last_j = Some(Instant::now());
        }

        if let KeyCode::Char('k') = key.code
            && let Some(t) = self.last_j
            && t.elapsed() < Duration::from_millis(300)
        {
            self.state.mode = edtui::EditorMode::Normal;
            self.state
                .execute(DeleteChar(edtui::actions::DeleteChar(1)));
            self.last_j = None;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Enter {
            let job: Option<Job> = match key.code {
                KeyCode::Enter => Some(Job::RunQuery(self.text())),
                _ => None,
            };

            if job.is_some() {
                return job;
            }
        }

        self.events.on_key_event(key, &mut self.state);

        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, block: Block) {
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
