use std::collections::HashMap;

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::Block,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    config::Config,
    database::DatabaseConnection,
    keymap::Keymap,
    layout::{
        Action, Command, DbEvent, QueryOptions, View,
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

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ConnectionKey(String, String);

pub struct App {
    pub should_quit: bool,
    config: Config,
    action_tx: UnboundedSender<Action>,
    keymap: Keymap,
    active_view: ActiveView,
    active_dialog: ActiveDialog,
    main_views: HashMap<ConnectionKey, MainView>,
    connections: HashMap<ConnectionKey, DatabaseConnection>,
    active_main_view_key: Option<ConnectionKey>,
    start_view: StartView,
    connections_dialog: ConnectionsDialog,
}

impl App {
    pub fn new(config: Config, action_tx: UnboundedSender<Action>) -> Self {
        Self {
            should_quit: false,
            config: config,
            action_tx: action_tx,
            keymap: Keymap::default_global(),
            main_views: HashMap::new(),
            connections: HashMap::new(),
            active_main_view_key: None,
            start_view: StartView::new(),
            active_view: ActiveView::Start,
            active_dialog: ActiveDialog::Connections,
            connections_dialog: ConnectionsDialog::new(Keymap::default_connections()),
        }
    }

    pub fn query_editor_mut(&mut self) -> Option<&mut QueryEditor> {
        self.active_main_view_mut()
            .and_then(|main_view| Some(main_view.query_editor_mut()))
    }

    pub fn handle_key(&mut self, event: KeyEvent) {
        let action = if let ActiveDialog::Connections = self.active_dialog {
            self.connections_dialog.handle_key(event)
        } else if let ActiveView::Start = self.active_view {
            self.start_view.handle_key(event)
        } else if let ActiveView::Main = self.active_view {
            self.active_main_view_mut()
                .and_then(|main_view| main_view.handle_key(event))
        } else {
            None
        };

        if let Some(action) = action {
            self.handle_action(action);
            return;
        }

        if let Some(action) = self.keymap.resolve_event(event) {
            if let Some(built_action) = build_action(action) {
                self.handle_action(built_action);
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        match self.active_view {
            ActiveView::Start => {
                if let ActiveDialog::None = self.active_dialog {
                    self.should_quit = true;
                }

                self.start_view.render(frame)
            }
            ActiveView::Main => {
                let key = self.active_main_view_key.as_ref();
                let main_view = key.and_then(|k| self.main_views.get_mut(k));
                let config =
                    key.and_then(|k| self.config.connections.iter().find(|c| c.name == k.0));

                if let Some(main_view) = main_view
                    && let Some(config) = config
                {
                    main_view.render(frame, config)
                } else {
                    self.connections_dialog.reset();
                    self.set_view(ActiveView::Start);
                    self.set_dialog(ActiveDialog::Connections);
                }
            }
        }

        if let ActiveDialog::Connections = self.active_dialog {
            self.connections_dialog.render(
                frame,
                frame.area(),
                Block::bordered().border_style(Style::new().fg(Color::Cyan)),
                &self.config.connections,
            );
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::CloseDialog => self.set_dialog(ActiveDialog::None),
            Action::OpenConnectionsDialog => {
                self.connections_dialog.reset();
                self.set_dialog(ActiveDialog::Connections)
            }
            Action::OpenConnection(connection_name) => {
                let Some(config) = self
                    .config
                    .connections
                    .iter()
                    .find(|c| c.name == connection_name)
                    .cloned()
                else {
                    return;
                };

                let connection_key = ConnectionKey(connection_name.clone(), String::new());
                self.main_views
                    .insert(connection_key.clone(), MainView::new());
                self.active_main_view_key = Some(connection_key.clone());
                self.set_view(ActiveView::Main);

                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    let action = match DatabaseConnection::connect(&config.backend).await {
                        Ok(connection) => {
                            Action::Db(connection_key, DbEvent::Connected(connection))
                        }
                        Err(message) => {
                            Action::Db(connection_key, DbEvent::ConnectionFailed(message))
                        }
                    };

                    let _ = tx.send(action);
                });
            }
            Action::Db(connection_key, db_event) => self.handle_db_event(connection_key, db_event),
            Action::Quit => self.should_quit = true,
            Action::OpenTable(table_name) => {
                let Some(connection_key) = self.active_main_view_key.as_ref() else {
                    return;
                };

                if let Some(main_view) = self.main_views.get_mut(&connection_key) {
                    main_view.set_table_name(table_name.clone());
                }

                let Some(connection) = self.connections.get(&connection_key).cloned() else {
                    return;
                };

                let tx = self.action_tx.clone();
                let connection_key = connection_key.clone();
                tokio::spawn(async move {
                    let options = QueryOptions { filter: None };

                    let action = match connection.query(&table_name, &options).await {
                        Ok(result) => Action::Db(connection_key, DbEvent::Queried(result)),
                        Err(message) => Action::Db(connection_key, DbEvent::QueryFailed(message)),
                    };

                    let _ = tx.send(action);
                });
            }
            Action::RunRawQuery(query) => {
                let Some(connection_key) = self.active_main_view_key.as_ref() else {
                    return;
                };

                let Some(connection) = self.connections.get(&connection_key).cloned() else {
                    return;
                };

                let tx = self.action_tx.clone();
                let connection_key = connection_key.clone();
                tokio::spawn(async move {
                    let action = match connection.query_raw(&query).await {
                        Ok(result) => Action::Db(connection_key, DbEvent::Queried(result)),
                        Err(message) => Action::Db(connection_key, DbEvent::QueryFailed(message)),
                    };

                    let _ = tx.send(action);
                });
            }
            Action::RunQuery(options) => {
                let Some(connection_key) = self.active_main_view_key.as_ref() else {
                    return;
                };

                let Some(main_view) = self.main_views.get_mut(&connection_key) else {
                    return;
                };

                let Some(table_name) = main_view.table_name().cloned() else {
                    return;
                };

                let Some(connection) = self.connections.get(&connection_key).cloned() else {
                    return;
                };

                let tx = self.action_tx.clone();
                let connection_key = connection_key.clone();
                tokio::spawn(async move {
                    let action = match connection.query(table_name.as_str(), &options).await {
                        Ok(result) => Action::Db(connection_key, DbEvent::Queried(result)),
                        Err(message) => Action::Db(connection_key, DbEvent::QueryFailed(message)),
                    };

                    let _ = tx.send(action);
                });
            }
            Action::OpenEditConnectionDialog(_) => todo!(),
        }
    }

    fn handle_db_event(&mut self, connection_key: ConnectionKey, db_event: DbEvent) {
        let Some(main_view) = self.main_views.get_mut(&connection_key) else {
            return;
        };

        match db_event {
            DbEvent::Connected(connection) => {
                self.connections
                    .insert(connection_key.clone(), connection.clone());

                main_view.set_connection_error(None);

                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    if let Ok(tables) = connection.tables().await {
                        let _ = tx.send(Action::Db(connection_key, DbEvent::Tables(tables)));
                    }
                });
            }
            DbEvent::ConnectionFailed(message) => main_view.set_connection_error(Some(message)),
            DbEvent::Tables(tables) => main_view.set_tables(tables),
            DbEvent::Queried(query_result) => {
                main_view.set_connection_error(None);
                main_view.load_result(query_result);
            }
            DbEvent::QueryFailed(message) => main_view.set_connection_error(Some(message)),
        }
    }

    fn active_main_view_mut(&mut self) -> Option<&mut MainView> {
        match &self.active_main_view_key {
            Some(key) => self.main_views.get_mut(&key),
            None => None,
        }
    }

    fn set_view(&mut self, view: ActiveView) {
        self.active_view = view;
        self.active_dialog = ActiveDialog::None;
    }

    fn set_dialog(&mut self, dialog: ActiveDialog) {
        self.active_dialog = dialog;
    }
}

fn build_action(action: Command) -> Option<Action> {
    match action {
        Command::CloseDialog => Some(Action::CloseDialog),
        Command::FindConnection => Some(Action::OpenConnectionsDialog),
        _ => None,
    }
}
