use crate::{
    config::SqliteConnectionDetails,
    database::{
        Backend, DatabaseSchemaRow, GridCell, QueryOptions, QueryResult, QuerySorting, SchemaResult,
    },
};

use sqlx::{
    AssertSqlSafe, Column, Pool, Row, Sqlite, TypeInfo, ValueRef,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow},
};

#[derive(Clone, Debug)]
pub struct SqliteConnection {
    config: SqliteConnectionDetails,
    pool: Pool<Sqlite>,
}

impl SqliteConnection {
    pub async fn connect(config: SqliteConnectionDetails) -> Result<Self, String> {
        let options = SqliteConnectOptions::new()
            .filename(&config.path)
            .read_only(config.is_readonly.unwrap_or(false));
        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .map_err(|e| e.to_string())?;

        Ok(Self {
            config: config,
            pool: pool,
        })
    }
}

impl Backend for SqliteConnection {
    async fn query_raw(&self, query: &str) -> Result<QueryResult, std::string::String> {
        let sqlite_rows = sqlx::query(AssertSqlSafe(query))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let columns: Vec<String> = match sqlite_rows.first() {
            Some(row) => row.columns().iter().map(|c| c.name().to_string()).collect(),
            None => Vec::new(),
        };

        let mut mapped_rows: Vec<Vec<GridCell>> = Vec::new();
        for sqlite_row in sqlite_rows {
            let mut mapped_row = Vec::new();
            for i in 0..sqlite_row.len() {
                let cell =
                    to_cell(&sqlite_row, i).unwrap_or_else(|e| GridCell::Error(e.to_string()));
                mapped_row.push(cell);
            }

            mapped_rows.push(mapped_row)
        }

        Ok(QueryResult::new(columns, mapped_rows))
    }

    async fn query(
        &self,
        table_name: &str,
        options: &QueryOptions,
    ) -> Result<QueryResult, std::string::String> {
        let mut query_str = format!("SELECT * FROM {}", enquote(table_name));

        // Filter
        if let Some(filter) = &options.filter {
            query_str.push_str("WHERE ");
            query_str.push_str(filter);
        };

        // Sorting
        match &options.sorting {
            Some(QuerySorting::Ascending(column_name)) => {
                query_str.push_str(&format!("ORDER BY {} ASC", enquote(column_name)))
            }
            Some(QuerySorting::Descending(column_name)) => {
                query_str.push_str(&format!("ORDER BY {} DESC", enquote(column_name)))
            }
            None => (),
        }

        // Limit
        query_str.push_str(" LIMIT 1000");

        // Execute
        let mut result = self.query_raw(query_str.as_ref()).await?;

        // Column names
        if result.header.len() == 0 {
            let columns: Vec<String> =
                sqlx::query_scalar("SELECT name FROM pragma_table_info(?) ORDER BY cid")
                    .bind(table_name)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;
            result.header = columns;
        }

        Ok(result)
    }

    async fn tables(&self) -> Result<Vec<String>, String> {
        sqlx::query_scalar(
            r#"
            SELECT name
            FROM sqlite_master
            WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%'
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
    }

    async fn schema_for_table(&self, table_name: &str) -> Result<SchemaResult, String> {
        let rows: Vec<(String, String, bool)> = sqlx::query_as(
            r#"
            SELECT name, type, "notnull"
            FROM pragma_table_info(?)
            ORDER BY cid
            "#,
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let schema_rows = rows
            .into_iter()
            .map(|row| DatabaseSchemaRow {
                name: row.0,
                type_name: row.1,
                is_nullable: Some(!row.2),
            })
            .collect();

        Ok(SchemaResult::new(schema_rows))
    }

    fn is_readonly(&self) -> bool {
        self.config.is_readonly.unwrap_or(false)
    }
}

fn to_cell(row: &SqliteRow, i: usize) -> Result<GridCell, sqlx::Error> {
    let raw = match row.try_get_raw(i) {
        Ok(raw) => raw,
        Err(e) => return Ok(GridCell::Error(e.to_string())),
    };

    if raw.is_null() {
        return Ok(GridCell::Null);
    }

    let cell = match raw.type_info().name() {
        "INTEGER" => row.try_get(i).map(GridCell::Integer)?,
        "REAL" => row.try_get(i).map(GridCell::Float)?,
        "TEXT" => row.try_get(i).map(GridCell::String)?,
        "BLOB" => row.try_get(i).map(GridCell::Bytes)?,
        _ => row.try_get(i).map(GridCell::String)?,
    };

    return Ok(cell);
}

fn enquote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
