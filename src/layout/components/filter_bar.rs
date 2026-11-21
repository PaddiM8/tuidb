use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent};
use edtui::actions::Action::{DeleteChar, DeleteWordBackward};
use edtui::events::{KeyEventHandler, KeyEventRegister, KeyInput};
use edtui::{
    EditorEventHandler, EditorMode, EditorState, EditorStatusLine, EditorTheme, EditorView,
    SyntaxHighlighter,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::ToText;
use ratatui::widgets::{Block, Paragraph};

use crate::layout::Action;
use crate::layout::components::Component;

pub struct FilterBar {
    state: EditorState,
    events: EditorEventHandler,
    last_j: Option<Instant>,
}

impl FilterBar {
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

    pub fn mode(&self) -> EditorMode {
        self.state.mode
    }

    pub fn text(&self) -> String {
        self.state.lines.to_string()
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, block: Block) {
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let label_text = "WHERE | ";
        let [label_area, input_area] = Layout::horizontal([
            Constraint::Length(label_text.len() as u16),
            Constraint::Fill(1),
        ])
        .areas(inner);
        let [label_area] = Layout::vertical([Constraint::Length(1)]).areas(label_area);

        // Label
        let label = Paragraph::new(label_text).bold();
        frame.render_widget(label, label_area);

        // Input
        let transparent = Style::default().bg(Color::Reset).fg(Color::Reset);
        let theme = EditorTheme::default().base(transparent).hide_status_line();
        let syntax_highlighter = SyntaxHighlighter::new("kronuz", "sql").ok();

        let view = EditorView::new(&mut self.state)
            .theme(theme)
            .syntax_highlighter(syntax_highlighter);
        frame.render_widget(view, input_area);
    }
}

impl Component for FilterBar {
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

        if event.code == KeyCode::Enter {
            return Some(Action::Filter(self.text()));
        }

        self.events.on_key_event(event, &mut self.state);

        None
    }
}
