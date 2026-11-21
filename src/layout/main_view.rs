use std::{
    ops::Add,
    time::{Duration, Instant},
};

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Paragraph},
};

use crate::{
    config::ConnectionEntry,
    database::{GridCell, QueryOptions, QueryResult, QuerySorting, QueryStatus},
    keymap::Keymap,
    layout::{
        Action, Command, View,
        components::{
            Component,
            data_grid::DataGrid,
            filter_bar::FilterBar,
            query_editor::QueryEditor,
            sidebar::{Sidebar, SidebarEvent},
        },
    },
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum MainPane {
    Sidebar,
    Query,
    Grid,
}

struct StatusMessage(String, Instant);

pub struct MainView {
    tables: Vec<String>,
    active_table_name: Option<String>,
    last_active_table_name: Option<String>,
    table_header: Vec<String>,
    table_rows: Vec<Vec<GridCell>>,
    query_options: QueryOptions,
    sidebar: Sidebar,
    query_editor: QueryEditor,
    filter_bar: FilterBar,
    grid: DataGrid,
    keymap: Keymap,
    active: MainPane,
    status_message: Option<StatusMessage>,
    is_fullscreen: bool,
}

impl MainView {
    pub fn new() -> Self {
        let sidebar = Sidebar::new(Keymap::default_selection());
        let query_editor = QueryEditor::new(Keymap::default_global());
        let filter_bar = FilterBar::new();
        let grid = DataGrid::new(Keymap::default_selection());

        Self {
            active_table_name: None,
            last_active_table_name: None,
            tables: Vec::new(),
            table_header: Vec::new(),
            table_rows: Vec::new(),
            query_options: QueryOptions {
                filter: None,
                sorting: None,
            },
            sidebar: sidebar,
            query_editor: query_editor,
            filter_bar: filter_bar,
            grid: grid,
            keymap: Keymap::default_global(),
            active: MainPane::Query,
            status_message: None,
            is_fullscreen: false,
        }
    }

    pub fn query_editor_mut(&mut self) -> &mut QueryEditor {
        &mut self.query_editor
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(StatusMessage(
            message,
            Instant::now().add(Duration::from_secs(5)),
        ));
    }

    fn handle_key_in_component(&mut self, event: KeyEvent) -> Option<Action> {
        match self.active {
            MainPane::Sidebar => match self.sidebar.handle_key(event) {
                Some(SidebarEvent::SubmittedIndex(index)) => {
                    self.active = MainPane::Grid;
                    self.tables.get(index).cloned().map(Action::OpenTable)
                }
                None => None,
            },
            MainPane::Query => {
                if self.is_full_editor() {
                    self.query_editor.handle_key(event)
                } else {
                    self.filter_bar.handle_key(event)
                }
            }
            MainPane::Grid => self.grid.handle_key(event),
        }
    }

    pub fn table_name(&self) -> Option<&String> {
        self.active_table_name.as_ref()
    }

    pub fn set_table_name(&mut self, table_name: String) {
        self.active_table_name = Some(table_name.clone());
        self.last_active_table_name = Some(table_name);
    }

    pub fn load_result(&mut self, result: QueryResult) {
        self.grid.clear_selection();
        self.table_header = result.header;
        self.table_rows = result.rows;
    }

    pub fn set_query_status(&mut self, status: Option<QueryStatus>) {
        self.grid.set_status(status);
    }

    pub fn set_tables(&mut self, tables: Vec<String>) {
        self.tables = tables;
    }

    pub fn query_options(&self) -> &QueryOptions {
        &self.query_options
    }

    pub fn set_filter(&mut self, filter: Option<String>) {
        self.query_options.filter = filter;
    }

    pub fn set_sorting(&mut self, sorting: Option<QuerySorting>) {
        self.query_options.sorting = sorting;
    }

    pub fn render(&mut self, frame: &mut Frame, connection: &ConnectionEntry) {
        if self.is_fullscreen {
            self.render_pane(self.active, frame, frame.area());
            return;
        }

        let [content_area, status_bar_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());

        let [sidebar_area, main_area] =
            Layout::horizontal([Constraint::Length(24), Constraint::Fill(1)]).areas(content_area);

        let show_full_editor = self.is_full_editor();
        let editor_area_height = if show_full_editor { 12 } else { 3 };
        let [editor_area, grid_area] =
            Layout::vertical([Constraint::Length(editor_area_height), Constraint::Fill(1)])
                .areas(main_area);

        self.render_pane(MainPane::Sidebar, frame, sidebar_area);
        self.render_pane(MainPane::Grid, frame, grid_area);
        self.render_pane(MainPane::Query, frame, editor_area);

        let status = if let Some(status_message) = &self.status_message
            && Instant::now() <= status_message.1
        {
            &format!("{} | {}", connection.name, status_message.0)
        } else {
            &connection.name
        };

        let status = Paragraph::new::<&str>(status);
        frame.render_widget(status, status_bar_area);
    }

    fn is_full_editor(&self) -> bool {
        self.active_table_name.is_none()
    }

    fn render_pane(&mut self, pane: MainPane, frame: &mut Frame, area: Rect) {
        match pane {
            MainPane::Sidebar => {
                let sidebar_block = self.get_block_for_id("Tables", MainPane::Sidebar);
                self.sidebar
                    .render(frame, area, sidebar_block, &self.tables);
            }
            MainPane::Query => {
                if self.is_full_editor() {
                    let editor_block = self.get_block_for_id("Query", MainPane::Query);
                    self.query_editor.render(frame, area, editor_block);
                } else {
                    let filter_block = self.get_block_for_id("Filter", MainPane::Query);
                    self.filter_bar.render(
                        frame,
                        area,
                        filter_block,
                        self.active == MainPane::Query,
                    );
                }
            }
            MainPane::Grid => {
                let title = if let Some(table_name) = &self.active_table_name {
                    format!("Rows ({})", table_name)
                } else {
                    String::from("Rows")
                };

                let grid_block = self.get_block_for_id(&title, MainPane::Grid);
                self.grid.render(
                    frame,
                    area,
                    grid_block,
                    &self.table_header,
                    &self.table_rows,
                );
            }
        }
    }

    fn get_block_for_id<'a>(&self, title: &'a str, pane: MainPane) -> Block<'a> {
        let border_style = if self.active == pane {
            Style::new().cyan()
        } else {
            Style::new().dark_gray()
        };

        Block::bordered().title(title).border_style(border_style)
    }
}

impl View for MainView {
    fn handle_key(&mut self, event: KeyEvent) -> Option<Action> {
        let command = self.keymap.resolve_event(event);
        if let Some(Command::Quit) = command {
            return Some(Action::Quit);
        }

        match command {
            Some(Command::ToggleFullscreen) => self.is_fullscreen = !self.is_fullscreen,
            Some(Command::ToggleQueryMode) => {
                if self.active_table_name.is_none() {
                    self.active_table_name = self
                        .last_active_table_name
                        .clone()
                        .or(self.tables.first().cloned());
                } else {
                    self.active_table_name = None;
                    self.sidebar.deselect();
                    self.active = MainPane::Query;
                }
            }
            Some(Command::FocusLeft) => {
                self.sidebar.focus();
                self.active = MainPane::Sidebar;
            }
            Some(Command::FocusUp) => self.active = MainPane::Query,
            Some(Command::FocusRight) => self.active = MainPane::Query,
            Some(Command::FocusDown) => self.active = MainPane::Grid,
            _ => return self.handle_key_in_component(event),
        }

        None
    }
}
