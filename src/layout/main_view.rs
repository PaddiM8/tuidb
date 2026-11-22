use std::{
    ops::Add,
    time::{Duration, Instant},
};

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Paragraph},
};
use thousands::Separable;

use crate::{
    config::{ConnectionColor, ConnectionEntry, KeymapSet},
    database::{GridCell, QueryOptions, QueryResult, QuerySorting, QueryStatus},
    keymap::Keymap,
    layout::{
        Action, Command, CommandKind, Mode, View,
        components::{
            Component,
            data_grid::DataGrid,
            filter_bar::FilterBar,
            query_editor::QueryEditor,
            sidebar::{Sidebar, SidebarEvent},
        },
    },
    theme::Theme,
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
    reported_row_count: Option<usize>,
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
    pub fn new(keymap_set: &KeymapSet) -> Self {
        let sidebar = Sidebar::new(Keymap::build_selection(&keymap_set.selection));
        let query_editor = QueryEditor::new(Keymap::build_editor(&keymap_set.editor));
        let filter_bar = FilterBar::new(Keymap::build_editor(&keymap_set.editor));
        let grid = DataGrid::new(Keymap::build_selection(&keymap_set.selection));

        Self {
            active_table_name: None,
            last_active_table_name: None,
            tables: Vec::new(),
            table_header: Vec::new(),
            table_rows: Vec::new(),
            reported_row_count: None,
            query_options: QueryOptions {
                filter: None,
                sorting: None,
            },
            sidebar: sidebar,
            query_editor: query_editor,
            filter_bar: filter_bar,
            grid: grid,
            keymap: Keymap::build_global(&keymap_set.global),
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

    fn handle_key_in_component(&mut self, event: KeyEvent) -> Vec<Action> {
        let mut actions = Vec::new();
        match self.active {
            MainPane::Sidebar => {
                actions.extend(self.handle_key_in_sidebar(event));
            }
            MainPane::Query => {
                if self.is_full_editor() {
                    actions.extend(self.query_editor.handle_key(event))
                } else {
                    actions.extend(self.filter_bar.handle_key(event))
                }
            }
            MainPane::Grid => actions.extend(self.grid.handle_key(event)),
        }

        actions
    }

    fn handle_command_in_component(&mut self, command: Command) -> Option<Vec<Action>> {
        match self.active {
            MainPane::Sidebar => None,
            MainPane::Query => {
                if self.is_full_editor() {
                    self.query_editor.handle_command(command)
                } else {
                    self.filter_bar.handle_command(command)
                }
            }
            MainPane::Grid => self.grid.handle_command(command),
        }
    }

    fn handle_key_in_sidebar(&mut self, event: KeyEvent) -> Vec<Action> {
        let mut actions = Vec::new();
        for command in self.sidebar.handle_key(event) {
            if let Some(action) = self.handle_command_in_sidebar(command) {
                actions.push(action);
            }
        }

        actions
    }

    fn handle_command_in_sidebar(&mut self, command: SidebarEvent) -> Option<Action> {
        match command {
            SidebarEvent::SubmittedIndex(index) => {
                self.active = MainPane::Grid;
                self.grid.clear_selection();

                self.tables.get(index).cloned().map(Action::OpenTable)
            }
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
        self.table_header = result.header;
        self.table_rows = result.rows;
        self.reported_row_count = Some(result.row_count);
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

    pub fn render(&mut self, frame: &mut Frame, connection: &ConnectionEntry, theme: &Theme) {
        if self.is_fullscreen {
            self.render_pane(
                self.active,
                frame,
                theme,
                connection.color.as_ref(),
                frame.area(),
            );
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

        self.render_pane(
            MainPane::Sidebar,
            frame,
            theme,
            connection.color.as_ref(),
            sidebar_area,
        );
        self.render_pane(
            MainPane::Grid,
            frame,
            theme,
            connection.color.as_ref(),
            grid_area,
        );
        self.render_pane(
            MainPane::Query,
            frame,
            theme,
            connection.color.as_ref(),
            editor_area,
        );

        self.render_status_bar(status_bar_area, frame, theme, &connection);
    }

    fn render_status_bar(
        &self,
        status_bar_area: Rect,
        frame: &mut Frame,
        theme: &Theme,
        connection: &ConnectionEntry,
    ) {
        let row_count_text = if let Some(row_count) = self.reported_row_count {
            format!("{} rows", row_count.separate_with_spaces())
        } else {
            String::new()
        };

        let [mode_area, connection_area, status_area, row_count_area] = Layout::horizontal([
            Constraint::Length(8),
            Constraint::Max(connection.name.len().min(32) as u16),
            Constraint::Fill(1),
            Constraint::Max(row_count_text.len().min(32) as u16),
        ])
        .spacing(1)
        .areas(status_bar_area);

        // Mode
        let mode = if self.is_full_editor() {
            self.query_editor.mode()
        } else {
            self.filter_bar.mode()
        };
        let mode_widget = match mode {
            Mode::Normal => Paragraph::new::<&str>(" NORMAL ").bg(theme.cyan),
            Mode::Insert => Paragraph::new::<&str>(" INSERT ").bg(theme.green),
            Mode::Visual => Paragraph::new::<&str>(" VISUAL ").bg(theme.red),
            Mode::Search => Paragraph::new::<&str>(" SEARCH ").bg(theme.yellow),
        };
        frame.render_widget(mode_widget, mode_area);

        // Connection
        let connection_widget = Paragraph::new::<&str>(&connection.name);
        frame.render_widget(connection_widget, connection_area);

        // Status
        if let Some(status_message) = &self.status_message
            && Instant::now() <= status_message.1
        {
            let status = Paragraph::new::<&str>(status_message.0.as_str());
            frame.render_widget(status, status_area);
        };

        // Row count
        let row_count_widget = Paragraph::new::<&str>(&row_count_text).right_aligned();
        frame.render_widget(row_count_widget, row_count_area);
    }

    fn render_pane(
        &mut self,
        pane: MainPane,
        frame: &mut Frame,
        theme: &Theme,
        connection_color: Option<&ConnectionColor>,
        area: Rect,
    ) {
        match pane {
            MainPane::Sidebar => {
                let sidebar_block =
                    self.get_block_for_id("Tables", MainPane::Sidebar, theme, connection_color);
                self.sidebar
                    .render(frame, theme, area, sidebar_block, &self.tables);
            }
            MainPane::Query => {
                if self.is_full_editor() {
                    let editor_block =
                        self.get_block_for_id("Query", MainPane::Query, theme, connection_color);
                    self.query_editor.render(frame, theme, area, editor_block);
                } else {
                    let filter_block =
                        self.get_block_for_id("", MainPane::Query, theme, connection_color);
                    self.filter_bar.render(
                        frame,
                        theme,
                        connection_color,
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

                let grid_block =
                    self.get_block_for_id(&title, MainPane::Grid, theme, connection_color);
                self.grid.render(
                    frame,
                    theme,
                    area,
                    grid_block,
                    &self.table_header,
                    &self.table_rows,
                );
            }
        }
    }

    fn is_full_editor(&self) -> bool {
        self.active_table_name.is_none()
    }

    fn get_block_for_id<'a>(
        &self,
        title: &'a str,
        pane: MainPane,
        theme: &Theme,
        connection_color: Option<&ConnectionColor>,
    ) -> Block<'a> {
        let border_style = if self.active == pane {
            connection_color.map_or_else(
                || theme.border_selected,
                |c| Style::new().fg(c.into_themed(theme)),
            )
        } else {
            theme.border
        };

        Block::bordered()
            .title(title)
            .style(theme.style)
            .border_style(border_style)
    }
}

impl View for MainView {
    fn handle_key(&mut self, event: KeyEvent) -> Vec<Action> {
        let commands = self.keymap.resolve_event(event);
        if commands.len() > 0 {
            commands
                .into_iter()
                .flat_map(|c| {
                    if let Some(actions) = self.handle_command(c) {
                        actions
                    } else {
                        Vec::new()
                    }
                })
                .collect()
        } else {
            self.handle_key_in_component(event)
        }
    }

    fn handle_command(&mut self, command: Command) -> Option<Vec<Action>> {
        let mut actions = Vec::new();
        match command.kind {
            CommandKind::Unresolved(events) => actions.extend(
                events
                    .into_iter()
                    .flat_map(|e| self.handle_key_in_component(e)),
            ),
            CommandKind::Quit => actions.push(Action::Quit),
            CommandKind::ToggleFullscreen => self.is_fullscreen = !self.is_fullscreen,
            CommandKind::ToggleQueryMode => {
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
            CommandKind::FocusLeft => {
                self.sidebar.focus();
                self.active = MainPane::Sidebar;
            }
            CommandKind::FocusUp => self.active = MainPane::Query,
            CommandKind::FocusRight => self.active = MainPane::Query,
            CommandKind::FocusDown => self.active = MainPane::Grid,
            _ => return self.handle_command_in_component(command),
        }

        Some(actions)
    }
}
