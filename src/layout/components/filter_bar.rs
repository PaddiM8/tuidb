use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::ToText;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use crate::layout::Action;
use crate::layout::components::Component;

pub struct FilterBar {
    pub state: Input,
}

impl FilterBar {
    pub fn new() -> Self {
        Self {
            state: Input::default(),
        }
    }

    pub fn text(&self) -> String {
        self.state.to_text().to_string()
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, block: Block, is_focused: bool) {
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
        let width = input_area.width.saturating_sub(1) as usize - 1;
        let scroll = self.state.visual_scroll(width);

        let input = Paragraph::new(self.state.value()).scroll((0, scroll as u16));
        frame.render_widget(input, input_area);

        let x = self.state.visual_cursor().max(scroll) - scroll;

        if is_focused {
            frame.set_cursor_position((input_area.x + x as u16, input_area.y));
        }
    }
}

impl Component for FilterBar {
    type Output = Action;

    fn handle_key(&mut self, event: KeyEvent) -> Option<Action> {
        if event.code == KeyCode::Enter {
            return Some(Action::Filter(self.text()));
        }

        self.state.handle_event(&Event::Key(event));

        None
    }
}
