use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph, Wrap},
};
use ratatui_table::{Cell, Row, Table, TableState};

use crate::{
    database::{GridCell, QueryStatus},
    keymap::Keymap,
    layout::{Action, Command, components::Component},
};

pub struct DataGrid {
    state: TableState,
    status: Option<QueryStatus>,
    keymap: Keymap,
    row_count: usize,
    rendered_width: u16,
    rendered_height: u16,
}

impl DataGrid {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            state: TableState::default(),
            status: None,
            keymap: keymap,
            row_count: 0,
            rendered_width: 0,
            rendered_height: 0,
        }
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        block: Block,
        header: &Vec<String>,
        items: &Vec<Vec<GridCell>>,
    ) {
        self.rendered_width = block.inner(area).width;
        self.rendered_height = block.inner(area).height;
        self.row_count = items.len();

        match &self.status {
            Some(QueryStatus::ConnectionError(message) | QueryStatus::QueryError(message)) => {
                let paragraph = Paragraph::new(message.as_str())
                    .fg(Color::Red)
                    .wrap(Wrap { trim: true })
                    .block(block);
                frame.render_widget(paragraph, area);

                return;
            }
            Some(QueryStatus::Loading) => {
                let paragraph = Paragraph::new("Loading...")
                    .fg(Color::DarkGray)
                    .wrap(Wrap { trim: true })
                    .block(block);
                frame.render_widget(paragraph, area);

                return;
            }
            None => (),
        }

        let widths = vec![Constraint::Length(20); header.len()];
        let table = Table::lazy_rows(items.len(), &widths, |i| build_row(items, i))
            .widths(widths)
            .header(Row::new(header.clone()).bold())
            .row_highlight_style(Style::new().on_dark_gray())
            .column_highlight_style(Style::new().on_dark_gray())
            .cell_highlight_style(Style::new().reversed())
            .block(block);

        frame.render_stateful_widget(table, area, &mut self.state);
    }

    pub fn set_status(&mut self, status: Option<QueryStatus>) {
        self.status = status;
    }
}

impl Component for DataGrid {
    type Output = Action;

    fn handle_key(&mut self, event: KeyEvent) -> Option<Action> {
        match self.keymap.resolve_event(event) {
            Some(Command::Left) => {
                self.state.select_previous_column();
                None
            }
            Some(Command::Down) => {
                self.state.select_next();
                None
            }
            Some(Command::Right) => {
                self.state.select_next_column();
                None
            }
            Some(Command::Up) => {
                self.state.select_previous();
                None
            }
            Some(Command::HalfPageDown) => {
                let offset = self.state.selected().unwrap_or(0) as u16 + self.rendered_height / 2;
                self.state
                    .select(Some((offset as usize).min(self.row_count)));
                None
            }
            Some(Command::HalfPageUp) => {
                let offset =
                    self.state.selected().unwrap_or(0) as i32 - self.rendered_height as i32 / 2;
                self.state.select(Some(offset.max(0) as usize));
                None
            }
            Some(Command::Copy) => match &self.status {
                Some(QueryStatus::ConnectionError(message)) => Some(Action::Copy(message.clone())),
                Some(QueryStatus::QueryError(message)) => Some(Action::Copy(message.clone())),
                Some(QueryStatus::Loading) => None,
                None => None,
            },
            _ => None,
        }
    }
}

fn build_row(items: &Vec<Vec<GridCell>>, index: usize) -> Row<'_> {
    Row::new(items[index].iter().map(|c| build_cell(c)))
}

fn build_cell<'a>(cell: &'a GridCell) -> Cell<'a> {
    match cell {
        GridCell::Error(message) => Cell::from(message.as_str()).fg(Color::Red),
        GridCell::Null => Cell::from("null").fg(Color::Gray),
        GridCell::String(value) => Cell::from(value.as_str()),
        GridCell::Bool(value) => {
            if *value {
                Cell::from("true")
            } else {
                Cell::from("false")
            }
        }
        GridCell::Integer(value) => Cell::from(value.to_string()),
        GridCell::Float(value) => Cell::from(value.to_string()),
        GridCell::BigNumber(value) => Cell::from(value.as_str()),
        GridCell::Bytes(bytes) => {
            const MAX: usize = 64;
            let hex: String = bytes.iter().take(MAX).map(|b| format!("{b:02x}")).collect();
            let formatted = if bytes.len() > MAX {
                format!("0x{}... ({} B)", hex, bytes.len())
            } else {
                format!("0x{}", hex)
            };

            Cell::from(formatted)
        }
    }
}
