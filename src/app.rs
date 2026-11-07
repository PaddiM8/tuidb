use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::Block,
};

use crate::{
    connection::Connection,
    keymap::Keymap,
    layout::{
        Action, Job, View,
        components::{Component, query_editor::QueryEditor},
        dialogs::connections_dialog::ConnectionsDialog,
        main_view::MainView,
        start_view::StartView,
    },
};

#[derive(Copy, Clone, Eq, PartialEq)]
enum ActiveView {
    Start,
    Main,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ActiveDialog {
    None,
    Connections,
}

pub struct App {
    pub should_quit: bool,
    keymap: Keymap,
    active_view: ActiveView,
    active_dialog: ActiveDialog,
    main_view: Option<MainView>,
    start_view: StartView,
    connections_dialog: ConnectionsDialog,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            keymap: Keymap::default_global(),
            main_view: None,
            start_view: StartView::new(),
            active_view: ActiveView::Start,
            active_dialog: ActiveDialog::Connections,
            connections_dialog: ConnectionsDialog::new(Keymap::default_connections()),
        }
    }

    pub fn query_editor_mut(&mut self) -> Option<&mut QueryEditor> {
        if let Some(main_view) = &mut self.main_view {
            Some(main_view.query_editor_mut())
        } else {
            None
        }
    }

    pub fn handle_key(&mut self, event: KeyEvent) {
        if let Some(action) = self.keymap.resolve_event(event) {
            if let Some(built_job) = build_job(action) {
                self.handle_job(built_job);
            }

            return;
        }

        let job = if let ActiveDialog::Connections = self.active_dialog {
            self.connections_dialog.handle_key(event)
        } else if let ActiveView::Start = self.active_view {
            self.start_view.handle_key(event)
        } else if let ActiveView::Main = self.active_view {
            self.main_view.as_mut().unwrap().handle_key(event)
        } else {
            None
        };

        if let Some(job) = job {
            self.handle_job(job);
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        match self.active_view {
            ActiveView::Start => self.start_view.render(frame),
            ActiveView::Main => self.main_view.as_mut().unwrap().render(frame),
        }

        if let ActiveDialog::Connections = self.active_dialog {
            self.connections_dialog.render(
                frame,
                frame.area(),
                Block::bordered().border_style(Style::new().fg(Color::Cyan)),
            );
        }
    }

    fn set_view(&mut self, view: ActiveView) {
        self.active_view = view;
        self.active_dialog = ActiveDialog::None;
    }

    fn set_dialog(&mut self, dialog: ActiveDialog) {
        self.active_dialog = dialog;
    }

    fn handle_job(&mut self, job: Job) {
        match job {
            Job::CloseDialog => self.set_dialog(ActiveDialog::None),
            Job::OpenConnectionsDialog => self.set_dialog(ActiveDialog::Connections),
            Job::OpenConnection(connection_name) => {
                self.main_view = Some(MainView::new(Connection::new(connection_name)));
                self.set_view(ActiveView::Main);
            }
            _ => (),
        }
    }
}

fn build_job(action: Action) -> Option<Job> {
    match action {
        Action::CloseDialog => Some(Job::CloseDialog),
        Action::FindConnection => Some(Job::OpenConnectionsDialog),
        _ => None,
    }
}
