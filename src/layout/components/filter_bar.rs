use crossterm::event::{KeyCode, KeyEvent};
use edtui::actions::Action::DeleteWordBackward;
use edtui::events::{KeyEventHandler, KeyEventRegister, KeyInput};
use edtui::{
    EditorEventHandler, EditorMode, EditorState, EditorTheme, EditorView, SyntaxHighlighter,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Paragraph};

use crate::config::ConnectionColor;
use crate::keymap::Keymap;
use crate::layout::components::Component;
use crate::layout::{Action, Command, CommandKind, Mode};
use crate::theme::Theme;

pub struct FilterBar {
    keymap: Keymap,
    state: EditorState,
    events: EditorEventHandler,
}

impl FilterBar {
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
            keymap: keymap,
            state: editor_state,
            events: event_handler,
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

    pub fn render(
        &mut self,
        frame: &mut Frame,
        theme: &Theme,
        connection_color: Option<&ConnectionColor>,
        area: Rect,
        block: Block,
        is_focused: bool,
    ) {
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let label_text = " WHERE ";
        let [label_area, input_area] = Layout::horizontal([
            Constraint::Length(label_text.len() as u16),
            Constraint::Fill(1),
        ])
        .spacing(1)
        .areas(inner);
        let [label_area] = Layout::vertical([Constraint::Length(1)]).areas(label_area);

        // Label
        let mut label = Paragraph::new(label_text).bold();
        if is_focused {
            label = label.bg(connection_color.map_or_else(|| theme.blue, |c| c.into_themed(theme)));
        } else {
            label = label.style(theme.border.reversed()).bg(theme.white);
        };

        frame.render_widget(label, label_area);

        // Input
        let theme = EditorTheme::default().base(theme.style).hide_status_line();
        let syntax_highlighter = SyntaxHighlighter::new("kronuz", "sql").ok();

        let view = EditorView::new(&mut self.state)
            .theme(theme)
            .syntax_highlighter(syntax_highlighter)
            .single_line(true);
        frame.render_widget(view, input_area);
    }
}

impl Component for FilterBar {
    type Output = Action;

    fn handle_key(&mut self, event: KeyEvent) -> Vec<Action> {
        if event.code == KeyCode::Enter {
            return vec![Action::Filter(self.text())];
        }

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
