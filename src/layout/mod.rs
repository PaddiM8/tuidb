use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::ConnectionKey,
    database::{DatabaseConnection, QueryResult, QuerySorting},
    layout::main_view::TabId,
};

pub mod components;
pub mod dialogs;
pub mod main_view;
pub mod start_view;

#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
    Search,
}

impl Mode {
    pub fn all() -> Vec<Self> {
        vec![Mode::Normal, Mode::Insert, Mode::Visual, Mode::Search]
    }
}

pub trait View {
    fn handle_key(&mut self, event: KeyEvent) -> Vec<Action>;

    fn handle_command(&mut self, command: Command) -> Option<Vec<Action>>;
}

#[derive(Clone, Debug)]
pub struct Command {
    pub kind: CommandKind,
    pub multiplier: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandKind {
    Unresolved(Vec<KeyEvent>),
    Nop,

    // Global
    Quit,
    #[serde(skip)]
    WaitForNextKey(Box<CommandKind>),
    RawKey(KeyCode),

    // Main
    RunQuery,
    NewTab,
    CloseCommandBar,

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
    FindTab,
    FindConnection,
}

#[derive(Clone, Debug)]
pub enum Action {
    // Global
    Quit,
    WaitForNextKey(CommandKind),
    HandleCommand(CommandKind),
    HandleCommandString(String),
    Copy(String),

    // Main
    CloseCommandBar,
    OpenTab(TabId),
    OpenConnection(String),
    OpenTable(String),
    RunRawQuery(String),
    Filter(String),
    Sort(Option<QuerySorting>),
    Db(TabId, ConnectionKey, DbEvent),

    // Dialogs
    CloseDialog,
    OpenTabDialog,
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
