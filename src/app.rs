use std::{collections::HashMap, time::Duration};

use arboard::Clipboard;
use crossterm::event::KeyEvent;
use ratatui::{Frame, widgets::Block};
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{
    config::Config,
    database::{DatabaseConnection, QueryOptions, QueryStatus},
    keymap::{DUAL_KEYBIND_TIMEOUT_MS, Keymap},
    layout::{
        Action, Command,
        CommandKind::{self},
        DbEvent, View,
        components::{Component, query_editor::QueryEditor},
        dialogs::{
            connections_dialog::ConnectionsDialog,
            tab_dialog::{TabDialog, TabDialogEntry},
        },
        main_view::{MainView, TabId},
        start_view::StartView,
    },
    theme::Theme,
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
    Tabs,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ConnectionKey(String, String);

pub struct App {
    pub should_quit: bool,
    config: Config,
    theme: Theme,
    action_tx: UnboundedSender<Action>,
    wait_for_next_key_handle: Option<JoinHandle<()>>,
    keymap: Keymap,
    clipboard: Option<Clipboard>,
    active_view: ActiveView,
    active_dialog: ActiveDialog,
    main_views: HashMap<ConnectionKey, MainView>,
    connections: HashMap<ConnectionKey, DatabaseConnection>,
    active_main_view_key: Option<ConnectionKey>,
    start_view: StartView,
    connections_dialog: ConnectionsDialog,
    tab_dialog: TabDialog,
}

impl App {
    pub fn new(config: Config, theme: Theme, action_tx: UnboundedSender<Action>) -> Self {
        let keymap = Keymap::build_global(&config.keymap.global);
        let connections_keymap = Keymap::build_find_dialog(&config.keymap.find_dialog);
        let tab_keymap = Keymap::build_find_dialog(&config.keymap.find_dialog);

        Self {
            should_quit: false,
            config: config,
            theme: theme,
            action_tx: action_tx,
            wait_for_next_key_handle: None,
            keymap: keymap,
            clipboard: Clipboard::new().ok(),
            main_views: HashMap::new(),
            connections: HashMap::new(),
            active_main_view_key: None,
            start_view: StartView::new(),
            active_view: ActiveView::Start,
            active_dialog: ActiveDialog::Connections,
            connections_dialog: ConnectionsDialog::new(connections_keymap),
            tab_dialog: TabDialog::new(tab_keymap),
        }
    }

    pub fn query_editor_mut(&mut self) -> Option<&mut QueryEditor> {
        self.active_main_view_mut()
            .and_then(|main_view| main_view.active_query_editor_mut())
    }

    pub fn handle_key(&mut self, event: KeyEvent) {
        if let Some(handle) = &self.wait_for_next_key_handle {
            handle.abort();
            self.wait_for_next_key_handle = None;
        }

        let actions = if let ActiveDialog::Connections = self.active_dialog {
            self.connections_dialog.handle_key(event)
        } else if let ActiveDialog::Tabs = self.active_dialog {
            self.tab_dialog.handle_key(event)
        } else if let ActiveView::Start = self.active_view {
            self.start_view.handle_key(event)
        } else if let ActiveView::Main = self.active_view {
            if let Some(main_view) = self.active_main_view_mut() {
                main_view.handle_key(event)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        if actions.len() > 0 {
            for action in actions {
                self.handle_action(action);
            }

            return;
        }

        for command in self.keymap.resolve_event(event) {
            if let Some(built_action) = build_action(command) {
                self.handle_action(built_action);
            }
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        let actions = if let ActiveDialog::Connections = self.active_dialog {
            self.connections_dialog.handle_command(command.clone())
        } else if let ActiveDialog::Tabs = self.active_dialog {
            self.tab_dialog.handle_command(command.clone())
        } else if let ActiveView::Start = self.active_view {
            self.start_view.handle_command(command.clone())
        } else if let ActiveView::Main = self.active_view {
            self.active_main_view_mut()
                .and_then(|main_view| main_view.handle_command(command.clone()))
        } else {
            None
        };

        if let Some(actions) = actions {
            for action in actions {
                self.handle_action(action);
            }
        } else if let Some(action) = build_action(command) {
            self.handle_action(action);
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.buffer_mut().set_style(area, self.theme.style);

        match self.active_view {
            ActiveView::Start => {
                if let ActiveDialog::None = self.active_dialog {
                    self.should_quit = true;
                }

                self.start_view.render(frame, &self.theme)
            }
            ActiveView::Main => {
                let key = self.active_main_view_key.as_ref();
                let main_view = key.and_then(|k| self.main_views.get_mut(k));
                let config =
                    key.and_then(|k| self.config.connections.iter().find(|c| c.name == k.0));

                if let Some(main_view) = main_view
                    && let Some(config) = config
                {
                    main_view.render(frame, config, &self.theme)
                } else {
                    self.connections_dialog.reset();
                    self.set_view(ActiveView::Start);
                    self.set_dialog(ActiveDialog::Connections);
                }
            }
        }

        match self.active_dialog {
            ActiveDialog::Connections => {
                self.connections_dialog.render(
                    frame,
                    &self.theme,
                    frame.area(),
                    Block::bordered().border_style(self.theme.border_selected),
                    &self.config.connections,
                );
            }
            ActiveDialog::Tabs => {
                let Some(main_view) = self.active_main_view() else {
                    return;
                };

                let current_tab_id = main_view.active_tab_id();
                let entries: Vec<TabDialogEntry> = main_view
                    .tabs()
                    .iter()
                    .rev()
                    .map(|c| TabDialogEntry {
                        tab_id: c.tab_id(),
                        name: c.name(),
                    })
                    // Don't include the current tab
                    .filter(|e| current_tab_id.is_none_or(|t| t != e.tab_id))
                    .collect();
                self.tab_dialog.render(
                    frame,
                    &self.theme,
                    frame.area(),
                    Block::bordered().border_style(self.theme.border_selected),
                    &entries,
                );
            }
            ActiveDialog::None => (),
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Copy(text) => {
                if let Some(clipboard) = &mut self.clipboard {
                    if clipboard.set_text(text).is_ok() {
                        self.set_status_message(String::from("Copied to clipboard"));
                    } else {
                        self.set_status_message(String::from("Failed to copy to clipboard"));
                    }
                }
            }
            Action::WaitForNextKey(fallback) => {
                let tx = self.action_tx.clone();
                self.wait_for_next_key_handle = Some(tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(DUAL_KEYBIND_TIMEOUT_MS + 1)).await;
                    let _ = tx.send(Action::HandleCommand(fallback));
                }));
            }
            Action::HandleCommand(command_kind) => self.handle_command(Command {
                kind: command_kind,
                multiplier: 1,
            }),
            Action::OpenTab(tab_id) => {
                let Some(main_view) = self.active_main_view_mut() else {
                    return;
                };

                main_view.open_tab(tab_id);
                self.set_dialog(ActiveDialog::None);
            }
            Action::CloseDialog => self.set_dialog(ActiveDialog::None),
            Action::OpenTabDialog => {
                self.tab_dialog.reset();
                self.set_dialog(ActiveDialog::Tabs)
            }
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
                self.main_views.insert(
                    connection_key.clone(),
                    MainView::new(self.config.keymap.clone()),
                );
                self.active_main_view_key = Some(connection_key.clone());
                self.set_view(ActiveView::Main);

                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    let action = match DatabaseConnection::connect(&config.backend).await {
                        Ok(connection) => {
                            Action::Db(TabId(0), connection_key, DbEvent::Connected(connection))
                        }
                        Err(message) => {
                            Action::Db(TabId(0), connection_key, DbEvent::ConnectionFailed(message))
                        }
                    };

                    let _ = tx.send(action);
                });
            }
            Action::Db(tab_id, connection_key, db_event) => {
                self.handle_db_event(tab_id, connection_key, db_event)
            }
            Action::Quit => self.should_quit = true,
            Action::OpenTable(table_name) => {
                let Some(connection_key) = self.active_main_view_key.as_ref() else {
                    return;
                };

                let tab_id = if let Some(main_view) = self.main_views.get_mut(&connection_key) {
                    main_view.open_table(table_name.clone())
                } else {
                    return;
                };

                let Some(connection) = self.connections.get(&connection_key).cloned() else {
                    return;
                };

                let tx = self.action_tx.clone();
                let connection_key = connection_key.clone();
                tokio::spawn(async move {
                    let options = QueryOptions {
                        filter: None,
                        sorting: None,
                    };

                    let action = match connection.query(&table_name, &options).await {
                        Ok(result) => Action::Db(tab_id, connection_key, DbEvent::Queried(result)),
                        Err(message) => {
                            Action::Db(tab_id, connection_key, DbEvent::QueryFailed(message))
                        }
                    };

                    let _ = tx.send(action);
                });
            }
            Action::RunRawQuery(query) => {
                let Some(connection_key) = self.active_main_view_key.as_ref() else {
                    return;
                };

                let tab_id = if let Some(main_view) = self.main_views.get_mut(&connection_key) {
                    main_view.set_query_status(Some(QueryStatus::Loading));
                    main_view.active_tab_id()
                } else {
                    return;
                };

                let Some(tab_id) = tab_id else {
                    return;
                };

                let Some(connection) = self.connections.get(&connection_key).cloned() else {
                    return;
                };

                let tx = self.action_tx.clone();
                let connection_key = connection_key.clone();
                tokio::spawn(async move {
                    let action = match connection.query_raw(&query).await {
                        Ok(result) => Action::Db(tab_id, connection_key, DbEvent::Queried(result)),
                        Err(message) => {
                            Action::Db(tab_id, connection_key, DbEvent::QueryFailed(message))
                        }
                    };

                    let _ = tx.send(action);
                });
            }
            Action::Filter(filter) => {
                let Some(connection_key) = self.active_main_view_key.as_ref() else {
                    return;
                };

                let Some(main_view) = self.main_views.get_mut(&connection_key) else {
                    return;
                };

                main_view.set_query_status(Some(QueryStatus::Loading));
                main_view.set_filter(Some(filter));

                let Some(table_name) = main_view.table_name().cloned() else {
                    return;
                };

                let Some(tab_id) = main_view.active_tab_id() else {
                    return;
                };

                let Some(connection) = self.connections.get(&connection_key).cloned() else {
                    return;
                };

                let tx = self.action_tx.clone();
                let connection_key = connection_key.clone();
                let Some(options) = main_view.query_options() else {
                    return;
                };

                let options = options.clone();
                tokio::spawn(async move {
                    let action = match connection.query(table_name.as_str(), &options).await {
                        Ok(result) => Action::Db(tab_id, connection_key, DbEvent::Queried(result)),
                        Err(message) => {
                            Action::Db(tab_id, connection_key, DbEvent::QueryFailed(message))
                        }
                    };

                    let _ = tx.send(action);
                });
            }
            Action::Sort(sorting) => {
                let Some(connection_key) = self.active_main_view_key.as_ref() else {
                    return;
                };

                let Some(main_view) = self.main_views.get_mut(&connection_key) else {
                    return;
                };

                main_view.set_query_status(Some(QueryStatus::Loading));
                main_view.set_sorting(sorting);

                let Some(table_name) = main_view.table_name().cloned() else {
                    return;
                };

                let Some(tab_id) = main_view.active_tab_id() else {
                    return;
                };

                let Some(connection) = self.connections.get(&connection_key).cloned() else {
                    return;
                };

                let tx = self.action_tx.clone();
                let connection_key = connection_key.clone();
                let Some(options) = main_view.query_options().clone() else {
                    return;
                };

                let options = options.clone();
                tokio::spawn(async move {
                    let action = match connection.query(table_name.as_str(), &options).await {
                        Ok(result) => Action::Db(tab_id, connection_key, DbEvent::Queried(result)),
                        Err(message) => {
                            Action::Db(tab_id, connection_key, DbEvent::QueryFailed(message))
                        }
                    };

                    let _ = tx.send(action);
                });
            }
        }
    }

    fn handle_db_event(&mut self, tab_id: TabId, connection_key: ConnectionKey, db_event: DbEvent) {
        let Some(main_view) = self.main_views.get_mut(&connection_key) else {
            return;
        };

        match db_event {
            DbEvent::Connected(connection) => {
                self.connections
                    .insert(connection_key.clone(), connection.clone());

                main_view.set_query_status(None);

                let Some(tab_id) = main_view.active_tab_id() else {
                    return;
                };

                let tx = self.action_tx.clone();
                tokio::spawn(async move {
                    if let Ok(tables) = connection.tables().await {
                        let _ =
                            tx.send(Action::Db(tab_id, connection_key, DbEvent::Tables(tables)));
                    }
                });
            }
            DbEvent::ConnectionFailed(message) => {
                main_view.set_query_status(Some(QueryStatus::ConnectionError(message)))
            }
            DbEvent::Tables(tables) => main_view.set_tables(tables),
            DbEvent::Queried(query_result) => {
                main_view.set_query_status(None);
                main_view.load_result(tab_id, query_result);
            }
            DbEvent::QueryFailed(message) => {
                main_view.set_query_status(Some(QueryStatus::QueryError(message)))
            }
        }
    }

    fn active_main_view(&self) -> Option<&MainView> {
        match &self.active_main_view_key {
            Some(key) => self.main_views.get(&key),
            None => None,
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

    fn set_status_message(&mut self, message: String) {
        if let Some(main_view) = self.active_main_view_mut() {
            main_view.set_status_message(message);
        }
    }
}

fn build_action(command: Command) -> Option<Action> {
    match command.kind {
        CommandKind::Quit => Some(Action::Quit),
        CommandKind::CloseDialog => Some(Action::CloseDialog),
        CommandKind::FindTab => Some(Action::OpenTabDialog),
        CommandKind::FindConnection => Some(Action::OpenConnectionsDialog),
        _ => None,
    }
}
