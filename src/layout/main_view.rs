use std::{
    fmt::Display,
    ops::Add,
    rc::Rc,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent};
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
            command_bar::CommandBar,
            data_grid::DataGrid,
            filter_bar::FilterBar,
            query_editor::QueryEditor,
            selectable_list::{SelectableList, SelectableListEvent},
        },
    },
    theme::Theme,
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum MainPane {
    TableList,
    TabList,
    Query,
    Grid,
}

struct StatusMessage(String, Instant);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub usize);

pub enum Tab {
    Query(QueryTab),
    Filter(FilterTab),
}

pub struct QueryTab {
    id: TabId,
    name: Option<String>,
    query_editor: QueryEditor,
    grid: DataGrid,
    table_header: Vec<String>,
    table_rows: Vec<Vec<GridCell>>,
}

pub struct FilterTab {
    id: TabId,
    table_name: String,
    query_options: QueryOptions,
    filter_bar: FilterBar,
    grid: DataGrid,
    table_header: Vec<String>,
    table_rows: Vec<Vec<GridCell>>,
}

impl Tab {
    pub fn tab_id(&self) -> TabId {
        match self {
            Tab::Query(tab) => tab.id,
            Tab::Filter(tab) => tab.id,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Tab::Query(tab) => tab.name.clone().unwrap_or(format!("Tab {}", tab.id.0)),
            Tab::Filter(tab) => tab.table_name.clone(),
        }
    }

    fn as_query(&self) -> Option<&QueryTab> {
        match self {
            Tab::Query(tab) => Some(tab),
            _ => None,
        }
    }

    fn as_query_mut(&mut self) -> Option<&mut QueryTab> {
        match self {
            Tab::Query(tab) => Some(tab),
            _ => None,
        }
    }

    fn as_filter_mut(&mut self) -> Option<&mut FilterTab> {
        match self {
            Tab::Filter(tab) => Some(tab),
            _ => None,
        }
    }

    fn as_filter(&self) -> Option<&FilterTab> {
        match self {
            Tab::Filter(tab) => Some(tab),
            _ => None,
        }
    }

    fn grid_mut(&mut self) -> &mut DataGrid {
        match self {
            Tab::Query(tab) => &mut tab.grid,
            Tab::Filter(tab) => &mut tab.grid,
        }
    }

    fn new_query(id: TabId, keymap_set: &KeymapSet) -> Self {
        Tab::Query(QueryTab {
            id: id,
            name: None,
            query_editor: QueryEditor::new(Keymap::build_editor(&keymap_set.editor)),
            grid: DataGrid::new(Keymap::build_selection(&keymap_set.selection)),
            table_header: Vec::new(),
            table_rows: Vec::new(),
        })
    }

    fn new_filter(id: TabId, table_name: String, keymap_set: &KeymapSet) -> Self {
        Tab::Filter(FilterTab {
            id: id,
            table_name: table_name,
            query_options: QueryOptions {
                filter: None,
                sorting: None,
            },
            filter_bar: FilterBar::new(Keymap::build_editor(&keymap_set.editor)),
            grid: DataGrid::new(Keymap::build_selection(&keymap_set.selection)),
            table_header: Vec::new(),
            table_rows: Vec::new(),
        })
    }
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

pub struct MainView {
    tabs: Vec<Tab>,
    tables: Vec<String>,
    last_active_table_name: Option<String>, // FIXME: This should be used when pressing ctrl+h in the query editor
    reported_row_count: Option<usize>,
    table_list: SelectableList,
    tab_list: SelectableList,
    keymap: Keymap,
    keymap_set: Rc<KeymapSet>,
    active: MainPane,
    status_message: Option<StatusMessage>,
    is_fullscreen: bool,
    next_tab_id: TabId,
    command_bar: Option<CommandBar>,
}

impl MainView {
    pub fn new(keymap_set: Rc<KeymapSet>) -> Self {
        let table_list = SelectableList::new(Keymap::build_selection(&keymap_set.selection));
        let tab_list = SelectableList::new(Keymap::build_selection(&keymap_set.selection));

        Self {
            tabs: vec![Tab::new_query(TabId(0), &keymap_set)],
            last_active_table_name: None,
            tables: Vec::new(),
            reported_row_count: None,
            table_list: table_list,
            tab_list: tab_list,
            keymap: Keymap::build_global(&keymap_set.global),
            keymap_set: keymap_set.clone(),
            active: MainPane::Query,
            status_message: None,
            is_fullscreen: false,
            next_tab_id: TabId(1),
            command_bar: None,
        }
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(StatusMessage(
            message,
            Instant::now().add(Duration::from_secs(5)),
        ));
    }

    fn active_query_editor(&self) -> Option<&QueryEditor> {
        self.tabs
            .last()
            .and_then(|t| t.as_query())
            .map(|e| &e.query_editor)
    }

    pub fn active_query_editor_mut(&mut self) -> Option<&mut QueryEditor> {
        self.tabs
            .last_mut()
            .and_then(|t| t.as_query_mut())
            .map(|e| &mut e.query_editor)
    }

    fn active_filter_bar(&self) -> Option<&FilterBar> {
        self.tabs
            .last()
            .and_then(|t| t.as_filter())
            .map(|e| &e.filter_bar)
    }

    fn active_filter_bar_mut(&mut self) -> Option<&mut FilterBar> {
        self.tabs
            .last_mut()
            .and_then(|t| t.as_filter_mut())
            .map(|e| &mut e.filter_bar)
    }

    pub fn active_tab_id(&self) -> Option<TabId> {
        self.tabs.last().map(|t| t.tab_id())
    }

    pub fn table_name(&self) -> Option<&String> {
        self.tabs
            .last()
            .and_then(|t| t.as_filter())
            .map(|t| &t.table_name)
    }

    pub fn open_tab(&mut self, tab_id: TabId) {
        let tab = self
            .tabs
            .iter()
            .position(|t| t.tab_id() == tab_id)
            .map(|i| self.tabs.remove(i));
        if let Some(tab) = tab {
            self.tabs.push(tab);
            self.tab_list.select(0);
        }
    }

    pub fn open_table(&mut self, table_name: String) -> TabId {
        let existing_tab = self
            .tabs
            .iter()
            .position(|t| {
                matches!(
                    t.as_filter().map(|t| t.table_name == table_name),
                    Some(true)
                )
            })
            .map(|i| self.tabs.remove(i));
        if let Some(existing_tab) = existing_tab {
            let tab_id = existing_tab.tab_id();
            self.tabs.push(existing_tab);
            self.active = MainPane::Grid;

            tab_id
        } else {
            let tab_id = self.next_tab_id;
            self.tabs
                .push(Tab::new_filter(tab_id, table_name, &self.keymap_set));
            self.next_tab_id = TabId(self.next_tab_id.0 + 1);
            self.active = MainPane::Grid;

            tab_id
        }
    }

    pub fn load_result(&mut self, tab_id: TabId, result: QueryResult) {
        let Some(tab) = self.tabs.iter_mut().find(|t| t.tab_id() == tab_id) else {
            return;
        };

        match tab {
            Tab::Query(query_tab) => {
                query_tab.table_header = result.header;
                query_tab.table_rows = result.rows;
            }
            Tab::Filter(filter_tab) => {
                filter_tab.table_header = result.header;
                filter_tab.table_rows = result.rows;
            }
        }

        self.reported_row_count = Some(result.row_count);
    }

    pub fn set_query_status(&mut self, status: Option<QueryStatus>) {
        if let Some(grid) = self.tabs.last_mut().map(Tab::grid_mut) {
            grid.set_status(status);
        }
    }

    pub fn set_tables(&mut self, tables: Vec<String>) {
        self.tables = tables;
    }

    pub fn query_options(&self) -> Option<&QueryOptions> {
        self.tabs
            .last()
            .and_then(|t| t.as_filter())
            .map(|t| &t.query_options)
    }

    pub fn set_filter(&mut self, filter: Option<String>) {
        if let Some(filter_tab) = self.tabs.last_mut().and_then(|t| t.as_filter_mut()) {
            filter_tab.query_options.filter = filter;
        }
    }

    pub fn tabs(&self) -> &Vec<Tab> {
        &self.tabs
    }

    pub fn set_sorting(&mut self, sorting: Option<QuerySorting>) {
        if let Some(filter_tab) = self.tabs.last_mut().and_then(|t| t.as_filter_mut()) {
            filter_tab.query_options.sorting = sorting;
        }
    }

    pub fn mode(&self) -> Mode {
        match self.active {
            MainPane::TableList => Mode::Normal,
            MainPane::TabList => Mode::Normal,
            MainPane::Query => {
                if let Some(query_editor) = self.active_query_editor() {
                    query_editor.mode()
                } else if let Some(filter_bar) = self.active_filter_bar() {
                    filter_bar.mode()
                } else {
                    Mode::Normal
                }
            }
            MainPane::Grid => Mode::Normal,
        }
    }

    pub fn close_command_bar(&mut self) {
        self.command_bar = None;
    }

    pub fn close_tab(&mut self) {
        self.tabs.pop();
    }

    pub fn close_all_tabs(&mut self) {
        self.tabs.clear();
        self.tabs.push(Tab::new_query(TabId(0), &self.keymap_set));
        self.next_tab_id = TabId(1);
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
        let [table_list_area, tab_list_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Max(24)]).areas(sidebar_area);

        let editor_area_height = if self.active_query_editor().is_some() {
            12
        } else {
            3
        };
        let [editor_area, grid_area] =
            Layout::vertical([Constraint::Length(editor_area_height), Constraint::Fill(1)])
                .areas(main_area);

        self.render_pane(
            MainPane::TableList,
            frame,
            theme,
            connection.color.as_ref(),
            table_list_area,
        );
        self.render_pane(
            MainPane::TabList,
            frame,
            theme,
            connection.color.as_ref(),
            tab_list_area,
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

        if let Some(command_bar) = &mut self.command_bar {
            command_bar.render(frame, status_bar_area);
        } else {
            self.render_status_bar(frame, status_bar_area, theme, &connection);
        }
    }

    fn render_status_bar(
        &self,
        frame: &mut Frame,
        status_bar_area: Rect,
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
        let mode = if let Some(query_editor) = self.active_query_editor() {
            query_editor.mode()
        } else if let Some(filter_bar) = self.active_filter_bar() {
            filter_bar.mode()
        } else {
            Mode::Normal
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
            MainPane::TableList => {
                let table_list_block =
                    self.get_block_for_id("Tables", MainPane::TableList, theme, connection_color);
                self.table_list
                    .render(frame, theme, area, table_list_block, &self.tables);
            }
            MainPane::TabList => {
                let tab_list_block =
                    self.get_block_for_id("Tabs", MainPane::TabList, theme, connection_color);
                let tabs: Vec<&Tab> = self.tabs.iter().rev().collect();
                self.tab_list
                    .render(frame, theme, area, tab_list_block, &tabs);
            }
            MainPane::Query => {
                let query_block =
                    self.get_block_for_id("Query", MainPane::Query, theme, connection_color);
                let filter_block =
                    self.get_block_for_id("", MainPane::Query, theme, connection_color);
                let is_active = self.active == MainPane::Query;

                if let Some(query_editor) = self.active_query_editor_mut() {
                    query_editor.render(frame, theme, area, query_block);
                } else if let Some(filter_bar) = self.active_filter_bar_mut() {
                    filter_bar.render(
                        frame,
                        theme,
                        connection_color,
                        area,
                        filter_block,
                        is_active,
                    );
                }
            }
            MainPane::Grid => {
                let title = if let Some(table_name) = self.table_name() {
                    format!("Rows ({})", table_name)
                } else {
                    String::from("Rows")
                };

                let grid_block =
                    self.get_block_for_id(&title, MainPane::Grid, theme, connection_color);

                let (grid, header, rows) = match self.tabs.last_mut() {
                    Some(Tab::Query(query_tab)) => (
                        &mut query_tab.grid,
                        &query_tab.table_header,
                        &query_tab.table_rows,
                    ),
                    Some(Tab::Filter(filter_tab)) => (
                        &mut filter_tab.grid,
                        &filter_tab.table_header,
                        &filter_tab.table_rows,
                    ),
                    None => return,
                };

                grid.render(frame, theme, area, grid_block, &header, &rows);
            }
        }
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

    fn handle_key_in_component(&mut self, event: KeyEvent) -> Vec<Action> {
        let mut actions = Vec::new();
        match self.active {
            MainPane::TableList => {
                let table_actions: Vec<Action> = self
                    .table_list
                    .handle_key(event)
                    .into_iter()
                    .filter_map(|c| self.handle_command_in_table_list(c))
                    .collect();
                actions.extend(table_actions);
            }
            MainPane::TabList => {
                let tab_list_actions: Vec<Action> = self
                    .tab_list
                    .handle_key(event)
                    .into_iter()
                    .filter_map(|c| self.handle_command_in_tab_list(c))
                    .collect();
                actions.extend(tab_list_actions);
            }
            MainPane::Query => {
                if let Some(query_tab) = self.tabs.last_mut().and_then(|t| t.as_query_mut()) {
                    actions.extend(query_tab.query_editor.handle_key(event))
                } else if let Some(filter_tab) =
                    self.tabs.last_mut().and_then(|t| t.as_filter_mut())
                {
                    actions.extend(filter_tab.filter_bar.handle_key(event))
                }
            }
            MainPane::Grid => actions.extend(
                self.tabs
                    .last_mut()
                    .map(Tab::grid_mut)
                    .map_or_default(|grid| grid.handle_key(event)),
            ),
        }

        actions
    }

    fn handle_command_in_component(&mut self, command: Command) -> Option<Vec<Action>> {
        match self.active {
            MainPane::TableList => None,
            MainPane::TabList => None,
            MainPane::Query => {
                if let Some(query_editor) = self.active_query_editor_mut() {
                    query_editor.handle_command(command)
                } else if let Some(filter_bar) = self.active_filter_bar_mut() {
                    filter_bar.handle_command(command)
                } else {
                    None
                }
            }
            MainPane::Grid => self
                .tabs
                .last_mut()
                .map(Tab::grid_mut)
                .and_then(|grid| grid.handle_command(command)),
        }
    }

    fn handle_command_in_table_list(&mut self, command: SelectableListEvent) -> Option<Action> {
        match command {
            SelectableListEvent::SubmittedIndex(index) => {
                self.active = MainPane::Grid;
                self.tables.get(index).cloned().map(Action::OpenTable)
            }
        }
    }

    fn handle_command_in_tab_list(&mut self, command: SelectableListEvent) -> Option<Action> {
        match command {
            SelectableListEvent::SubmittedIndex(index) => {
                if let Some(tab) = self.tabs.get(self.tabs.len() - index - 1) {
                    self.open_tab(tab.tab_id());
                }
            }
        }

        None
    }
}

impl View for MainView {
    fn handle_key(&mut self, event: KeyEvent) -> Vec<Action> {
        if let Some(command_bar) = &mut self.command_bar {
            return command_bar.handle_key(event);
        }

        if let KeyCode::Char(':') = event.code {
            self.command_bar = Some(CommandBar::new(Keymap::build_command_bar(
                &self.keymap_set.command_bar,
            )));

            return Vec::new();
        }

        let commands = self.keymap.resolve_event_with_mode(event, self.mode());
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
            CommandKind::NewTab => {
                self.tabs
                    .push(Tab::new_query(self.next_tab_id, &self.keymap_set));
                self.next_tab_id = TabId(self.next_tab_id.0 + 1);
                self.table_list.deselect();
                self.active = MainPane::Query;
            }
            CommandKind::FocusLeft => {
                self.table_list.focus();
                self.active = MainPane::TableList;
            }
            CommandKind::FocusUp => {
                if self.active == MainPane::TabList {
                    self.active = MainPane::TableList;
                } else {
                    self.active = MainPane::Query;
                }
            }
            CommandKind::FocusRight => self.active = MainPane::Grid,
            CommandKind::FocusDown => {
                if self.active == MainPane::TableList {
                    self.active = MainPane::TabList;
                } else {
                    self.active = MainPane::Grid;
                }
            }
            CommandKind::FocusQuery => self.active = MainPane::Query,
            _ => return self.handle_command_in_component(command),
        }

        Some(actions)
    }
}
