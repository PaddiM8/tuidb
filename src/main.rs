use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::app::App;
mod app;
mod connection;
mod keymap;
mod layout;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(run)?;
    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new();
    while !app.should_quit {
        terminal.draw(|frame| app.render(frame))?;

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                app.handle_key(key);
            }
            _ => {}
        }

        if let Some(query_editor) = app.query_editor_mut()
            && query_editor.is_system_editor_pending()
        {
            query_editor.open_system_editor(terminal)?;
        }
    }

    Ok(())
}
