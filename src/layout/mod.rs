use crossterm::event::KeyEvent;
use ratatui::Frame;

pub mod components;
pub mod dialogs;
pub mod main_view;
pub mod start_view;

pub trait View {
    fn handle_key(&mut self, event: KeyEvent) -> Option<Job>;

    fn render(&mut self, frame: &mut Frame);
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
    OpenConnection(String),
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
