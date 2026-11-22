use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::ConnectionKey,
    database::{DatabaseConnection, QueryResult, QuerySorting},
};

pub mod components;
pub mod dialogs;
pub mod main_view;
pub mod start_view;

pub trait View {
    fn handle_key(&mut self, event: KeyEvent) -> Vec<Action>;
}

#[derive(Clone, Debug)]
pub struct Command {
    pub kind: CommandKind,
    pub multiplier: u32,
}

#[derive(Clone, Debug)]
pub enum CommandKind {
    Unresolved(Vec<KeyEvent>),

    // Global
    Quit,
    WaitForNextKey(KeyCode),

    // Main
    RunQuery,
    ToggleQueryMode,

    // Editor
    NormalMode,

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
    PageTop,
    PageBottom,

    // Data
    Sort,
    Copy,

    // Dialogs
    CloseDialog,
    FindConnection,
    EditSelectedConnection,
    NewConnection,
}

#[derive(Clone, Debug)]
pub enum Action {
    // Global
    Quit,
    WaitForNextKey(KeyCode),
    Copy(String),

    // Main
    OpenConnection(String),
    OpenTable(String),
    RunRawQuery(String),
    Filter(String),
    Sort(Option<QuerySorting>),
    Db(ConnectionKey, DbEvent),

    // Dialogs
    CloseDialog,
    OpenConnectionsDialog,
}

#[derive(Clone, Debug)]
pub enum DbEvent {
    Connected(DatabaseConnection),
    ConnectionFailed(String),
    Tables(Vec<String>),
    Queried(QueryResult),
    QueryFailed(String),
}
