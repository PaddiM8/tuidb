use std::path::Path;

use color_eyre::eyre::{ContextCompat, Result};
use crossterm::{
    event::{
        Event, EventStream, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::supports_keyboard_enhancement,
};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use supports_color::Stream;
use tokio::sync::mpsc;

use crate::{app::App, config::Config, layout::Action, theme::Theme};

mod app;
mod config;
mod database;
mod keymap;
mod layout;
mod theme;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = run(&mut terminal).await;
    ratatui::restore();

    result
}

async fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("could not determine config directory")?
        .join("tuidb");

    let config_path = config_dir.join("config.toml");
    let config = Config::load_from_path(&config_path)?;

    let theme = if let Some(theme_path) = &config.theme {
        Theme::load_from_path(Path::new(theme_path))?
    } else {
        match supports_color::on(Stream::Stdout) {
            Some(support) if support.has_256 || support.has_16m => Theme::default_truecolor(),
            _ => Theme::default_basic(),
        }
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<Action>();
    let mut app = App::new(config, theme, tx.clone());
    let mut reader = EventStream::new();

    let enhanced = supports_keyboard_enhancement().unwrap_or(false);
    if enhanced {
        execute!(
            std::io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
    }

    while !app.should_quit {
        terminal.draw(|frame| app.render(frame))?;

        tokio::select! {
            Some(Ok(ev)) = reader.next() => {
                if let Event::Key(key) = ev && key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
            Some(ev) = rx.recv() => app.handle_action(ev),
            else => break,
        }

        if let Some(query_editor) = app.query_editor_mut()
            && query_editor.is_system_editor_pending()
        {
            drop(reader);
            query_editor.open_system_editor(terminal)?;
            reader = EventStream::new();
        }
    }

    if enhanced {
        let _ = execute!(std::io::stdout(), PopKeyboardEnhancementFlags);
    }

    Ok(())
}
