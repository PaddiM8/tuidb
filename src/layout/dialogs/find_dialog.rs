use crossterm::event::{Event, KeyEvent};
use nucleo_matcher::{
    Matcher, Utf32Str,
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
    state: ListState,
    filtered_indices: Vec<usize>,
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
            state: state,
            filtered_indices: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.state = ListState::default();
        self.input.reset();
        self.filtered_indices.clear();
    }

    pub fn selected(&self) -> Option<usize> {
        if let Some(index) = self.state.selected() {
            self.filtered_indices.get(index).copied()
        } else {
            None
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, block: Block, items: &Vec<&str>) {
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
        let pattern = Pattern::parse(
            &self.input.to_text().to_string(),
            CaseMatching::Smart,
            Normalization::Smart,
        );

        let mut buffer = Vec::new();
        let mut scored: Vec<(usize, u32)> = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                pattern
                    .score(Utf32Str::new(&item, &mut buffer), &mut self.matcher)
                    .map(|score| (index, score))
            })
            .collect();

        // Sort by the score
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        self.filtered_indices = scored.iter().map(|(index, _)| index.to_owned()).collect();

        if let None = self.selected() {
            self.state.select_first();
        }

        let mut results = Vec::new();
        for index in &self.filtered_indices {
            results.push(items[*index]);
        }

        let list = List::new(results)
            .block(Block::new())
            .highlight_style(Style::new().reversed());
        frame.render_stateful_widget(list, list_area, &mut self.state);
    }
}

impl Component for FindDialog {
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
}
