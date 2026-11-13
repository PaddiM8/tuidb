use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Paragraph},
};

use crate::{
    config::ConnectionEntry,
    keymap::Keymap,
    layout::{
        Action, Job, View,
        components::{Component, data_grid::DataGrid, query_editor::QueryEditor, sidebar::Sidebar},
    },
};

#[derive(Copy, Clone, PartialEq, Eq)]
enum MainPane {
    Sidebar,
    QueryEditor,
    Grid,
}

pub struct MainView {
    pub should_quit: bool,
    sidebar: Sidebar,
    query_editor: QueryEditor,
    grid: DataGrid,
    keymap: Keymap,
    active: MainPane,
    is_fullscreen: bool,
}

impl MainView {
    pub fn new() -> Self {
        let sidebar = Sidebar::new(vec![], Keymap::default_selection());
        let query_editor = QueryEditor::new();
        let grid = DataGrid::new(
            vec![String::from("test1"), String::from("test2")],
            Keymap::default_selection(),
        );

        Self {
            should_quit: false,
            sidebar: sidebar,
            query_editor: query_editor,
            grid: grid,
            keymap: Keymap::default_global(),
            active: MainPane::QueryEditor,
            is_fullscreen: false,
        }
    }

    pub fn query_editor_mut(&mut self) -> &mut QueryEditor {
        &mut self.query_editor
    }

    fn handle_key_in_component(&mut self, event: KeyEvent) -> Option<Job> {
        let job = match self.active {
            MainPane::Sidebar => self.sidebar.handle_key(event),
            MainPane::QueryEditor => self.query_editor.handle_key(event),
            MainPane::Grid => self.grid.handle_key(event),
        };

        if let Some(job) = job {
            self.try_handle_job_globally(&job);
            return None;
        }

        job
    }

    fn try_handle_job_globally(&mut self, job: &Job) -> bool {
        if let Job::Quit = job {
            self.should_quit = true;
        } else if let Job::ToggleFullscreen = job {
            self.is_fullscreen = !self.is_fullscreen;
        } else if let Job::FocusLeft = job {
            self.active = MainPane::Sidebar;
        } else if let Job::FocusUp = job {
            self.active = MainPane::QueryEditor;
        } else if let Job::FocusRight = job {
            self.active = MainPane::QueryEditor;
        } else if let Job::FocusDown = job {
            self.active = MainPane::QueryEditor;
        } else {
            return false;
        }

        true
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

        let [editor_area, grid_area] =
            Layout::vertical([Constraint::Length(12), Constraint::Fill(1)]).areas(main_area);

        self.render_pane(MainPane::Sidebar, frame, sidebar_area);
        self.render_pane(MainPane::QueryEditor, frame, editor_area);
        self.render_pane(MainPane::Grid, frame, grid_area);

        let connection_name = Paragraph::new::<&str>(connection.name.as_ref());
        frame.render_widget(connection_name, status_bar_area);
    }

    fn render_pane(&mut self, pane: MainPane, frame: &mut Frame, area: Rect) {
        match pane {
            MainPane::Sidebar => {
                let sidebar_block = self.get_block_for_id("Tables", MainPane::Sidebar);
                self.sidebar.render(frame, area, sidebar_block);
            }
            MainPane::QueryEditor => {
                let editor_block = self.get_block_for_id("Query", MainPane::QueryEditor);
                self.query_editor.render(frame, area, editor_block);
            }
            MainPane::Grid => {
                let grid_block = self.get_block_for_id("Result", MainPane::Grid);
                self.grid.render(frame, area, grid_block);
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
    fn handle_key(&mut self, event: KeyEvent) -> Option<Job> {
        let action = self.keymap.resolve_event(event);
        if action.is_none() {
            return self.handle_key_in_component(event);
        }

        let action = action.unwrap();
        if let Some(built_job) = build_job(action) {
            if self.try_handle_job_globally(&built_job) {
                return None;
            } else {
                return Some(built_job);
            }
        }

        return self.handle_key_in_component(event);
    }
}

fn build_job(action: Action) -> Option<Job> {
    match action {
        _ => None,
    }
}
