use crossterm::event::{Event, KeyEvent};
use nucleo_matcher::{
    Matcher,
    pattern::{CaseMatching, Normalization, Pattern},
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::ToText,
    widgets::{Block, Borders, Clear, List, ListState, Paragraph},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    keymap::Keymap,
    layout::{Action, Job, components::Component, dialogs::build_dialog_job},
};

pub struct FindDialog {
    title: String,
    keymap: Keymap,
    matcher: Matcher,
    input: Input,
    items: Vec<String>,
    state: ListState,
}

impl FindDialog {
    pub fn new(title: String, keymap: Keymap) -> Self {
        let mut state = ListState::default();
        state.select_first();

        Self {
            title: title,
            keymap: keymap,
            matcher: Matcher::new(nucleo_matcher::Config::DEFAULT),
            input: Input::new(String::new()),
            items: vec![
                String::from("Connection 1"),
                String::from("Connection 2"),
                String::from("Abc"),
                String::from("Con"),
                String::from("Lorem ipsum"),
            ],
            state: state,
        }
    }

    pub fn selected_value(&self) -> Option<String> {
        if let Some(index) = self.state.selected() {
            Some(self.items[index].clone())
        } else {
            None
        }
    }
}

impl Component for FindDialog {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn handle_key(&mut self, event: KeyEvent) -> Option<Job> {
        if let Some(action) = self.keymap.resolve_event(event) {
            let base_job = build_dialog_job(&action);
            if base_job.is_some() {
                return base_job;
            }

            match action {
                Action::Up => self.state.select_previous(),
                Action::Down => self.state.select_next(),
                _ => (),
            }

            return None;
        }

        self.input.handle_event(&Event::Key(event));

        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, block: Block) {
        let area = super::centered(area, area.width.min(40), area.height.min(15));
        frame.render_widget(
            Block::new().style(Style::default().add_modifier(Modifier::DIM)),
            frame.area(),
        );

        frame.render_widget(Clear, area);

        let block = block.title::<&str>(self.title.as_ref());

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let [input_area, list_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(inner);

        // Input
        let width = input_area.width.saturating_sub(2) as usize - 1;
        let scroll = self.input.visual_scroll(width);

        let input = Paragraph::new(format!("> {}", self.input.value()))
            .scroll((0, scroll as u16))
            .block(
                Block::new()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::new().fg(Color::DarkGray)),
            );
        frame.render_widget(input, input_area);

        let x = self.input.visual_cursor().max(scroll) - scroll + 2;
        frame.set_cursor_position((input_area.x + x as u16, input_area.y));

        // List
        let results: Vec<&str> = Pattern::parse(
            &self.input.to_text().to_string(),
            CaseMatching::Smart,
            Normalization::Smart,
        )
        .match_list(&self.items, &mut self.matcher)
        .into_iter()
        .map(|(s, _score)| s.as_ref())
        .collect();

        let list = List::new(results)
            .block(Block::new())
            .highlight_style(Style::new().reversed());
        frame.render_stateful_widget(list, list_area, &mut self.state);
    }
}
