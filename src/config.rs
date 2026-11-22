use std::{fs, path::Path};

use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use ratatui::style::Color;
use serde::Deserialize;

use crate::theme::Theme;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub connections: Vec<ConnectionEntry>,
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

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionEntry {
    pub name: String,
    pub color: Option<ConnectionColor>,
    #[serde(flatten)]
    pub backend: BackendDetails,
}

#[derive(Copy, Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackendDetails {
    Sqlite(SqliteConnectionDetails),
    Postgres(PostgresConnectionDetails),
    SqlServer(SqlServerConnectionDetails),
}

#[derive(Clone, Debug, Deserialize)]
pub struct SqliteConnectionDetails {
    pub path: String,
    pub is_readonly: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PostgresConnectionDetails {
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SqlServerConnectionDetails {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}
