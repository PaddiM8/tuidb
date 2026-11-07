use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, Paragraph},
};

use crate::{
    connection::Connection,
    keymap::Keymap,
    layout::{
        Action, Job, View,
        components::{Component, data_grid::DataGrid, query_editor::QueryEditor, sidebar::Sidebar},
    },
};

pub struct MainView {
    pub should_quit: bool,
    connection: Connection,
    components: Vec<Box<dyn Component>>,
    sidebar_id: usize,
    query_editor_id: usize,
    grid_id: usize,
    keymap: Keymap,
    focused_id: usize,
    is_fullscreen: bool,
}

impl MainView {
    pub fn new(connection: Connection) -> Self {
        let mut components: Vec<Box<dyn Component>> = Vec::new();

        let sidebar_index = components.len();
        components.push(Box::new(Sidebar::new(vec![], Keymap::default_selection())));

        let editor_index = components.len();
        components.push(Box::new(QueryEditor::new()));

        let grid_index = components.len();
        components.push(Box::new(DataGrid::new(
            vec![String::from("test1"), String::from("test2")],
            Keymap::default_selection(),
        )));

        Self {
            should_quit: false,
            connection: connection,
            components: components,
            sidebar_id: sidebar_index,
            query_editor_id: editor_index,
            grid_id: grid_index,
            keymap: Keymap::default_global(),
            focused_id: 0,
            is_fullscreen: false,
        }
    }

    pub fn query_editor_mut(&mut self) -> &mut QueryEditor {
        self.components[self.query_editor_id]
            .as_any_mut()
            .downcast_mut::<QueryEditor>()
            .unwrap()
    }

    fn focused_mut(&mut self) -> &mut dyn Component {
        self.components[self.focused_id].as_mut()
    }

    fn get_block_for_id<'a>(&self, title: &'a str, id: usize) -> Block<'a> {
        let border_style = if self.focused_id == id {
            Style::new().cyan()
        } else {
            Style::new().dark_gray()
        };

        Block::bordered().title(title).border_style(border_style)
    }

    fn handle_key_in_component(&mut self, event: KeyEvent) -> Option<Job> {
        let job = self.focused_mut().handle_key(event);
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
            self.focused_id = self.sidebar_id;
        } else if let Job::FocusUp = job {
            self.focused_id = self.query_editor_id;
        } else if let Job::FocusRight = job {
            self.focused_id = self.query_editor_id;
        } else if let Job::FocusDown = job {
            self.focused_id = self.grid_id;
        } else {
            return false;
        }

        true
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

    fn render(&mut self, frame: &mut Frame) {
        if self.is_fullscreen {
            let focused_block = self.get_block_for_id("", self.focused_id);
            self.focused_mut()
                .render(frame, frame.area(), focused_block);
            return;
        }

        let [content_area, status_bar_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());

        let [sidebar_area, main_area] =
            Layout::horizontal([Constraint::Length(24), Constraint::Fill(1)]).areas(content_area);

        let [editor_area, grid_area] =
            Layout::vertical([Constraint::Length(12), Constraint::Fill(1)]).areas(main_area);

        let connection_name = Paragraph::new(self.connection.name.clone());
        frame.render_widget(connection_name, status_bar_area);

        let sidebar_block = self.get_block_for_id("Tables", self.sidebar_id);
        self.components[self.sidebar_id]
            .as_mut()
            .render(frame, sidebar_area, sidebar_block);

        let editor_block = self.get_block_for_id("Query", self.query_editor_id);
        self.components[self.query_editor_id]
            .as_mut()
            .render(frame, editor_area, editor_block);

        let grid_block = self.get_block_for_id("Result", self.grid_id);
        self.components[self.grid_id]
            .as_mut()
            .render(frame, grid_area, grid_block);
    }
}

fn build_job(action: Action) -> Option<Job> {
    match action {
        _ => None,
    }
}
