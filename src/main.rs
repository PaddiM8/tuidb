use color_eyre::eyre::{ContextCompat, Result};
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::{app::App, config::Config};
mod app;
mod config;
mod keymap;
mod layout;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(run)?;
    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let path = dirs::config_dir()
        .context("could not determine config directory")?
        .join("tuidb")
        .join("config.toml");
    let config = Config::load_from_path(&path)?;
    let mut app = App::new(config);

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
