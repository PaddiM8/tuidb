use std::rc::Rc;
use std::{fs, path::Path};

use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use ratatui::style::Color;
use serde::Deserialize;
use serde::Serialize;

use crate::layout::CommandKind;
use crate::layout::Mode;
use crate::theme::Theme;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub connections: Vec<ConnectionEntry>,
    pub keymap: Rc<KeymapSet>,
    pub theme: Option<String>,
}

impl Config {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                return Err(error).with_context(|| format!("reading {}", path.display()));
            }
        };

        let config: Config =
            toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;

        Ok(config)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct KeymapSet {
    pub global: Vec<KeybindConfigEntry>,
    pub selection: Vec<KeybindConfigEntry>,
    pub dialog: Vec<KeybindConfigEntry>,
    pub find_dialog: Vec<KeybindConfigEntry>,
    pub editor: Vec<KeybindConfigEntry>,
}

impl Default for KeymapSet {
    fn default() -> Self {
        KeymapSet {
            global: Vec::new(),
            selection: Vec::new(),
            dialog: Vec::new(),
            find_dialog: Vec::new(),
            editor: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeybindConfigEntry {
    pub binding: String,
    #[serde(default)]
    pub mode: Mode,
    pub command: CommandKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionEntry {
    pub name: String,
    pub color: Option<ConnectionColor>,
    #[serde(flatten)]
    pub backend: BackendDetails,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

impl ConnectionColor {
    pub fn into_themed(&self, theme: &Theme) -> Color {
        match self {
            ConnectionColor::Black => theme.black,
            ConnectionColor::Red => theme.red,
            ConnectionColor::Green => theme.green,
            ConnectionColor::Yellow => theme.yellow,
            ConnectionColor::Blue => theme.blue,
            ConnectionColor::Magenta => theme.magenta,
            ConnectionColor::Cyan => theme.cyan,
            ConnectionColor::Gray => theme.gray,
            ConnectionColor::DarkGray => theme.dark_gray,
            ConnectionColor::LightRed => theme.light_red,
            ConnectionColor::LightGreen => theme.light_green,
            ConnectionColor::LightYellow => theme.light_yellow,
            ConnectionColor::LightBlue => theme.light_blue,
            ConnectionColor::LightMagenta => theme.light_magenta,
            ConnectionColor::LightCyan => theme.light_cyan,
            ConnectionColor::White => theme.white,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackendDetails {
    Sqlite(SqliteConnectionDetails),
    Postgres(PostgresConnectionDetails),
    SqlServer(SqlServerConnectionDetails),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqliteConnectionDetails {
    pub path: String,
    pub is_readonly: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostgresConnectionDetails {
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqlServerConnectionDetails {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}
