use crossterm::event::KeyEvent;

pub mod components;
pub mod dialogs;
pub mod main_view;
pub mod start_view;

pub trait View {
    fn handle_key(&mut self, event: KeyEvent) -> Option<Job>;
}

#[derive(Clone, Debug)]
pub enum Action {
    // Global
    Quit,

    // Main
    RunQuery,

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
pub enum Job {
    // Global
    Quit,

    // Main
    OpenConnection(usize),
    OpenTable(String),
    RunQuery(String),

    // Navigation
    ToggleFullscreen,
    FocusLeft,
    FocusUp,
    FocusRight,
    FocusDown,
    HalfPageDown,
    HalfPageUp,

    // Dialogs
    CloseDialog,
    OpenConnectionsDialog,
    OpenEditConnectionDialog(Option<String>),
}
