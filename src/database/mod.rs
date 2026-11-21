use crate::{config::BackendDetails, database::sqlite::SqliteConnection};

pub mod sqlite;

pub enum QueryStatus {
    Loading,
    ConnectionError(String),
    QueryError(String),
}

#[derive(Clone, Debug)]
pub struct QueryOptions {
    pub filter: Option<String>,
    pub sorting: Option<QuerySorting>,
}

#[derive(Clone, Debug)]
pub enum QuerySorting {
    Ascending(String),
    Descending(String),
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub header: Vec<String>,
    pub rows: Vec<Vec<GridCell>>,
    pub row_count: usize,
}

impl QueryResult {
    pub fn new(header: Vec<String>, rows: Vec<Vec<GridCell>>, row_count: usize) -> Self {
        Self {
            header,
            rows,
            row_count,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GridCell {
    Error(String),
    Null,
    String(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
    BigNumber(String),
    Bytes(Vec<u8>),
}

pub struct SchemaResult {
    rows: Vec<DatabaseSchemaRow>,
}

impl SchemaResult {
    pub fn new(rows: Vec<DatabaseSchemaRow>) -> Self {
        Self { rows: rows }
    }
}

pub struct DatabaseSchemaRow {
    pub name: String,
    pub type_name: String,
    pub is_nullable: Option<bool>,
}

#[derive(Clone, Debug)]
pub enum DatabaseConnection {
    Sqlite(SqliteConnection),
}

impl DatabaseConnection {
    pub async fn connect(config: &BackendDetails) -> Result<Self, String> {
        match config {
            BackendDetails::Sqlite(c) => {
                let connection = SqliteConnection::connect(c.clone()).await?;

                Ok(DatabaseConnection::Sqlite(connection))
            }
            _ => todo!(),
        }
    }

    pub async fn query_raw(&self, query: &str) -> Result<QueryResult, String> {
        match self {
            DatabaseConnection::Sqlite(c) => c.query_raw(query).await,
        }
    }

    pub async fn query(
        &self,
        table_name: &str,
        options: &QueryOptions,
    ) -> Result<QueryResult, String> {
        match self {
            DatabaseConnection::Sqlite(c) => c.query(table_name, options).await,
        }
    }

    pub async fn tables(&self) -> Result<Vec<String>, String> {
        match self {
            DatabaseConnection::Sqlite(c) => c.tables().await,
        }
    }

    pub async fn schema_for_table(&self, table_name: &str) -> Result<SchemaResult, String> {
        match self {
            DatabaseConnection::Sqlite(c) => c.schema_for_table(table_name).await,
        }
    }

    pub fn is_readonly(&self) -> bool {
        match self {
            DatabaseConnection::Sqlite(c) => c.is_readonly(),
        }
    }
}

pub trait Backend {
    async fn query_raw(&self, query: &str) -> Result<QueryResult, String>;

    async fn query(&self, table_name: &str, options: &QueryOptions) -> Result<QueryResult, String>;

    async fn tables(&self) -> Result<Vec<String>, String>;

    async fn schema_for_table(&self, table_name: &str) -> Result<SchemaResult, String>;

    fn is_readonly(&self) -> bool;
}
