use crossterm::event::{Event, KeyEvent};
use nucleo_matcher::{
    Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::ToText,
    widgets::{Block, Borders, Clear, List, ListState, Paragraph},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    keymap::Keymap,
    layout::{Action, CommandKind, components::Component, dialogs::build_dialog_action},
    theme::Theme,
};

pub struct FindDialog {
    title: String,
    keymap: Keymap,
    matcher: Matcher,
    input: Input,
    state: ListState,
    filtered_indices: Vec<usize>,
    selected_value: Option<String>,
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
            selected_value: None,
        }
    }

    pub fn reset(&mut self) {
        self.state = ListState::default();
        self.input.reset();
        self.filtered_indices.clear();
    }

    pub fn selected_value(&self) -> Option<&str> {
        self.selected_value.as_deref()
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        theme: &Theme,
        area: Rect,
        block: Block,
        items: &Vec<&str>,
    ) {
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
                    .style(theme.style)
                    .borders(Borders::BOTTOM)
                    .border_style(theme.border),
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

        // Filter/fuzzy search using nucleo
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

        if let None = self.selected_value() {
            self.state.select_first();
        }

        self.filtered_indices = scored.iter().map(|(index, _)| index.to_owned()).collect();
        self.selected_value = self
            .state
            .selected()
            .and_then(|row| self.filtered_indices.get(row))
            .and_then(|&index| items.get(index))
            .map(|x| x.to_string());

        let mut results = Vec::new();
        for index in &self.filtered_indices {
            results.push(items[*index]);
        }

        let list = List::new(results)
            .block(Block::new())
            .style(theme.style)
            .highlight_style(theme.item_selected);
        frame.render_stateful_widget(list, list_area, &mut self.state);
    }
}

impl Component for FindDialog {
    type Output = Action;

    fn handle_key(&mut self, event: KeyEvent) -> Vec<Action> {
        let mut actions = Vec::new();
        for command in self.keymap.resolve_event(event) {
            let base_action = build_dialog_action(&command.kind);
            if let Some(base_action) = base_action {
                actions.push(base_action);
                continue;
            }

            match command.kind {
                CommandKind::Unresolved(events) => {
                    for event in events {
                        self.input.handle_event(&Event::Key(event));
                    }
                }
                CommandKind::Up => self.state.select_previous(),
                CommandKind::Down => self.state.select_next(),
                _ => (),
            };
        }

        actions
    }
}
