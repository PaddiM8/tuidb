use crossterm::event::KeyEvent;

use crate::{
    app::ConnectionKey,
    database::{DatabaseConnection, QueryResult},
};

pub mod components;
pub mod dialogs;
pub mod main_view;
pub mod start_view;

pub trait View {
    fn handle_key(&mut self, event: KeyEvent) -> Option<Action>;
}

#[derive(Clone, Debug)]
pub enum Command {
    // Global
    Quit,

    // Main
    RunQuery,
    ToggleQueryMode,

    // Navigation
    ToggleFullscreen,
    FocusLeft,
    FocusUp,
    FocusRight,
    FocusDown,
    Left,
    Up,
    Right,
    Down,
    Submit,
    HalfPageDown,
    HalfPageUp,

    // Dialogs
    CloseDialog,
    FindConnection,
    EditSelectedConnection,
    NewConnection,
}

#[derive(Clone, Debug)]
pub struct QueryOptions {
    pub filter: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Action {
    // Global
    Quit,

    // Main
    OpenConnection(String),
    OpenTable(String),
    RunRawQuery(String),
    RunQuery(QueryOptions),
    Db(ConnectionKey, DbEvent),

    // Dialogs
    CloseDialog,
    OpenConnectionsDialog,
    OpenEditConnectionDialog(Option<String>),
}

#[derive(Clone, Debug)]
pub enum DbEvent {
    Connected(DatabaseConnection),
    ConnectionFailed(String),
    Tables(Vec<String>),
    Queried(QueryResult),
    QueryFailed(String),
}
