use std::{fs, path::Path};

use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use serde::Deserialize;

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
    #[serde(flatten)]
    pub backend: BackendDetails,
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

impl ConnectionEntry {
    pub fn new(name: String, backend: BackendDetails) -> Self {
        Self { name, backend }
    }
}
