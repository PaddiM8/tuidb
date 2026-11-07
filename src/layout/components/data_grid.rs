use std::any::Any;

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::Block,
};
use ratatui_table::{Cell, Row, Table, TableState};

use crate::{
    keymap::Keymap,
    layout::{Action, Job, components::Component},
};

pub struct DataGrid {
    header: Vec<String>,
    items: Vec<Vec<String>>,
    state: TableState,
    keymap: Keymap,
    rendered_width: u16,
    rendered_height: u16,
}

impl DataGrid {
    pub fn new(header: Vec<String>, keymap: Keymap) -> Self {
        Self {
            header: header,
            items: vec![
                vec![
                    String::from("a1"),
                    String::from("a2"),
                    String::from("a3"),
                    String::from("a4"),
                ],
                vec![
                    String::from("b1"),
                    String::from("b2"),
                    String::from("b3"),
                    String::from("b4"),
                ],
                vec![
                    String::from("c1"),
                    String::from("c2"),
                    String::from("c3"),
                    String::from("c4"),
                ],
                vec![
                    String::from("d1"),
                    String::from("d2"),
                    String::from("d3"),
                    String::from("d4"),
                ],
                vec![
                    String::from("e1"),
                    String::from("e2"),
                    String::from("e3"),
                    String::from("e4"),
                ],
                vec![
                    String::from("f1"),
                    String::from("f2"),
                    String::from("f3"),
                    String::from("f4"),
                ],
                vec![
                    String::from("g1"),
                    String::from("g2"),
                    String::from("g3"),
                    String::from("g4"),
                ],
                vec![
                    String::from("h1"),
                    String::from("h2"),
                    String::from("h3"),
                    String::from("h4"),
                ],
                vec![
                    String::from("i1"),
                    String::from("i2"),
                    String::from("i3"),
                    String::from("i4"),
                ],
                vec![
                    String::from("j1"),
                    String::from("j2"),
                    String::from("j3"),
                    String::from("j4"),
                ],
                vec![
                    String::from("k1"),
                    String::from("k2"),
                    String::from("k3"),
                    String::from("k4"),
                ],
            ],
            state: TableState::default(),
            keymap: keymap,
            rendered_width: 0,
            rendered_height: 0,
        }
    }
}

impl Component for DataGrid {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_key(&mut self, event: KeyEvent) -> Option<Job> {
        match self.keymap.resolve_event(event) {
            Some(Action::Left) => {
                self.state.select_previous_column();
                None
            }
            Some(Action::Down) => {
                self.state.select_next();
                None
            }
            Some(Action::Right) => {
                self.state.select_next_column();
                None
            }
            Some(Action::Up) => {
                self.state.select_previous();
                None
            }
            Some(Action::HalfPageDown) => {
                let offset = self.state.selected().unwrap_or(0) as u16 + self.rendered_height / 2;
                self.state
                    .select(Some((offset as usize).min(self.items.len())));
                None
            }
            Some(Action::HalfPageUp) => {
                let offset =
                    self.state.selected().unwrap_or(0) as i32 - self.rendered_height as i32 / 2;
                self.state.select(Some(offset.max(0) as usize));
                None
            }
            _ => None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, block: Block) {
        let widths = [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ];

        self.rendered_width = block.inner(area).width;
        self.rendered_height = block.inner(area).height;

        let table = Table::lazy_rows(self.items.len(), widths, |i| build_row(&self.items, i))
            .widths(widths)
            .header(Row::new(self.header.clone()).bold())
            .row_highlight_style(Style::new().on_dark_gray())
            .column_highlight_style(Style::new().on_dark_gray())
            .cell_highlight_style(Style::new().reversed())
            .block(block);

        frame.render_stateful_widget(table, area, &mut self.state);
    }
}

fn build_row(items: &Vec<Vec<String>>, index: usize) -> Row<'_> {
    Row::new(items[index].iter().map(|c| Cell::from(c.as_str())))
}
