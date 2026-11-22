use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Stylize,
    text::Text,
    widgets::{Block, Paragraph, Wrap},
};
use ratatui_table::{Cell, Row, Table, TableState};

use crate::{
    database::{GridCell, QuerySorting, QueryStatus},
    keymap::Keymap,
    layout::{Action, Command, components::Component},
    theme::Theme,
};

pub struct DataGrid {
    state: TableState,
    status: Option<QueryStatus>,
    keymap: Keymap,
    sorting: Option<QuerySorting>,
    visible_column_count: usize,
    total_column_count: usize,
    row_count: usize,
    horizontal_scroll: usize,
    rendered_width: u16,
    rendered_height: u16,
    selected_column_name: Option<String>,
}

impl DataGrid {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            state: TableState::default(),
            status: None,
            keymap: keymap,
            sorting: None,
            visible_column_count: 0,
            total_column_count: 0,
            row_count: 0,
            horizontal_scroll: 0,
            rendered_width: 0,
            rendered_height: 0,
            selected_column_name: None,
        }
    }

    pub fn clear_selection(&mut self) {
        self.horizontal_scroll = 0;
        self.state.select(None);
        self.state.select_column(None);
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        theme: &Theme,
        area: Rect,
        block: Block,
        header: &Vec<String>,
        items: &Vec<Vec<GridCell>>,
    ) {
        self.rendered_width = block.inner(area).width;
        self.rendered_height = block.inner(area).height;
        self.total_column_count = header.len();
        self.row_count = items.len();

        match &self.status {
            Some(QueryStatus::ConnectionError(message) | QueryStatus::QueryError(message)) => {
                let paragraph = Paragraph::new(message.as_str())
                    .fg(theme.red)
                    .wrap(Wrap { trim: true })
                    .block(block);
                frame.render_widget(paragraph, area);

                return;
            }
            Some(QueryStatus::Loading) => {
                let paragraph = Paragraph::new("Loading...")
                    .fg(theme.dark_gray)
                    .wrap(Wrap { trim: true })
                    .block(block);
                frame.render_widget(paragraph, area);

                return;
            }
            None => (),
        }

        const COLUMN_WIDTH: u16 = 20;
        let inner_width = block.inner(area).width;
        let mut columns_to_show = inner_width / (COLUMN_WIDTH + 1);
        let mut edge_column_width = inner_width % (COLUMN_WIDTH + 1);

        if edge_column_width > 1 {
            columns_to_show += 1;
        }

        if columns_to_show as usize > header.len() {
            columns_to_show = header.len() as u16;
            edge_column_width = COLUMN_WIDTH;
        }

        // If the areas width as increased, scroll to the left to show more columns if relevant
        if header.len().saturating_sub(self.horizontal_scroll) < columns_to_show as usize {
            self.horizontal_scroll = header.len().saturating_sub(columns_to_show as usize);
        }

        self.visible_column_count = columns_to_show as usize;
        if header.len().saturating_sub(self.horizontal_scroll) < columns_to_show as usize {
            self.visible_column_count = header.len().saturating_sub(self.horizontal_scroll);
            edge_column_width = COLUMN_WIDTH;
        }

        let mut widths = vec![COLUMN_WIDTH; self.visible_column_count];

        // If the leftover space is not visible, remove the column meant for it.
        // If the last visible column is selected, truncate the first column.
        // Otherwise, truncate the last column.
        let mut right_align_first_column = false;
        if edge_column_width <= 1 {
            self.visible_column_count = self.visible_column_count.saturating_sub(1);
            widths.pop();
        } else if let Some(selected_column) = self.state.selected_column()
            && selected_column == self.visible_column_count.saturating_sub(1)
        {
            widths[0] = edge_column_width;
            right_align_first_column = true;
        } else if self.visible_column_count > 0 {
            widths[self.visible_column_count - 1] = edge_column_width;
        }

        let header_cells = header
            .iter()
            .skip(self.horizontal_scroll)
            .take(self.visible_column_count)
            .enumerate()
            .map(|(i, c)| {
                build_header_cell(
                    c,
                    &self.sorting,
                    widths[i],
                    right_align_first_column && i == 0,
                )
            });
        let constraints = widths.iter().map(|w| Constraint::Length(*w));
        let table = Table::lazy_rows(items.len(), constraints, |i| {
            build_row(
                items,
                i,
                self.horizontal_scroll,
                self.visible_column_count,
                widths.as_slice(),
                right_align_first_column,
                theme,
            )
        })
        .header(Row::new(header_cells).bold())
        .row_highlight_style(theme.item_selected_secondary)
        .column_highlight_style(theme.item_selected_secondary)
        .cell_highlight_style(theme.item_selected)
        .block(block);

        frame.render_stateful_widget(table, area, &mut self.state);

        self.selected_column_name = header
            .get(self.horizontal_scroll + self.state.selected_column().unwrap_or(0))
            .cloned();
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
                if let Some(selected_column) = self.state.selected_column()
                    && selected_column == 0
                {
                    self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
                } else {
                    self.state.select_previous_column();
                }

                None
            }
            Some(Command::Down) => {
                self.state.select_next();
                None
            }
            Some(Command::Right) => {
                if let Some(selected_column) = self.state.selected_column()
                    && selected_column == self.visible_column_count - 1
                {
                    self.horizontal_scroll = (self.horizontal_scroll + 1)
                        .min(self.total_column_count - self.visible_column_count);
                } else {
                    self.state.select_next_column();
                }

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
            Some(Command::Sort) => {
                let Some(column_to_sort) = &self.selected_column_name else {
                    return None;
                };

                match &self.sorting {
                    // Same column: Ascending -> Descending
                    // New column: Ascending -> Ascending
                    Some(QuerySorting::Ascending(currently_sorted)) => {
                        let new_sorting = if column_to_sort == currently_sorted {
                            QuerySorting::Descending(column_to_sort.to_string())
                        } else {
                            QuerySorting::Ascending(column_to_sort.to_string())
                        };

                        self.sorting = Some(new_sorting.clone());
                        Some(Action::Sort(Some(new_sorting)))
                    }
                    // Same column: Descending -> None
                    // New column: Descending -> Descending
                    Some(QuerySorting::Descending(currently_sorted)) => {
                        let new_sorting = if column_to_sort == currently_sorted {
                            None
                        } else {
                            Some(QuerySorting::Descending(column_to_sort.to_string()))
                        };

                        self.sorting = new_sorting.clone();
                        Some(Action::Sort(new_sorting))
                    }
                    // None -> Ascending
                    None => {
                        let new_sorting = QuerySorting::Ascending(column_to_sort.to_string());

                        self.sorting = Some(new_sorting.clone());
                        Some(Action::Sort(Some(new_sorting)))
                    }
                }
            }
            _ => None,
        }
    }
}

fn build_row<'a>(
    items: &'a Vec<Vec<GridCell>>,
    index: usize,
    horizontal_scroll: usize,
    column_count: usize,
    widths: &'a [u16],
    right_align_first_column: bool,
    theme: &Theme,
) -> Row<'a> {
    let cells = items[index]
        .iter()
        .skip(horizontal_scroll)
        .take(column_count)
        .enumerate()
        .map(|(i, c)| build_cell(c, widths[i], right_align_first_column && i == 0, theme));

    Row::new(cells)
}

fn build_header_cell<'a>(
    name: &'a str,
    sorting: &Option<QuerySorting>,
    width: u16,
    is_cut_off_to_left: bool,
) -> Cell<'a> {
    let arrow = match sorting {
        Some(QuerySorting::Ascending(column_name)) if column_name == name => " ▲",
        Some(QuerySorting::Descending(column_name)) if column_name == name => " ▼",
        _ => return build_aligned_cell(name, width, is_cut_off_to_left),
    };

    let width = width.saturating_sub(2) as usize;
    let name: String = name.chars().take(width).collect();
    let formatted = format!("{name:<width$}{arrow}");

    build_aligned_cell(&formatted, width as u16, is_cut_off_to_left)
}

fn build_cell<'a>(
    cell: &'a GridCell,
    width: u16,
    is_cut_off_to_left: bool,
    theme: &Theme,
) -> Cell<'a> {
    match cell {
        GridCell::Error(message) => {
            build_aligned_cell(message.as_str(), width, is_cut_off_to_left).fg(theme.red)
        }
        GridCell::Null => build_aligned_cell("null", width, is_cut_off_to_left).fg(theme.gray),
        GridCell::String(value) => build_aligned_cell(value.as_str(), width, is_cut_off_to_left),
        GridCell::Bool(value) => {
            if *value {
                build_aligned_cell("true", width, is_cut_off_to_left)
            } else {
                build_aligned_cell("false", width, is_cut_off_to_left)
            }
        }
        GridCell::Integer(value) => {
            build_aligned_cell(&value.to_string(), width, is_cut_off_to_left)
        }
        GridCell::Float(value) => build_aligned_cell(&value.to_string(), width, is_cut_off_to_left),
        GridCell::BigNumber(value) => build_aligned_cell(value.as_str(), width, is_cut_off_to_left),
        GridCell::Bytes(bytes) => {
            const MAX: usize = 64;
            let hex: String = bytes.iter().take(MAX).map(|b| format!("{b:02x}")).collect();
            let formatted = if bytes.len() > MAX {
                format!("0x{}... ({} B)", hex, bytes.len())
            } else {
                format!("0x{}", hex)
            };

            build_aligned_cell(&formatted, width, is_cut_off_to_left)
        }
    }
}

fn build_aligned_cell(text: &str, width: u16, is_cut_off_to_left: bool) -> Cell<'static> {
    if is_cut_off_to_left {
        let text = format!("{text:<width$}", text = text, width = width as usize);
        Cell::from(Text::from(text).right_aligned())
    } else {
        Cell::from(text.to_string())
    }
}
