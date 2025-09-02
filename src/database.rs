use anyhow::{Result, anyhow};
use sqlx::{Column, MySql, Pool, Postgres, Row, Sqlite};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DatabaseType {
    SQLite,
    PostgreSQL,
    MySQL,
}

impl DatabaseType {
    pub fn from_url(url: &str) -> Result<Self> {
        if url.starts_with("sqlite:") {
            Ok(DatabaseType::SQLite)
        } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            Ok(DatabaseType::PostgreSQL)
        } else if url.starts_with("mysql://") {
            Ok(DatabaseType::MySQL)
        } else {
            Err(anyhow!("Unsupported database URL format"))
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DatabaseType::SQLite => "SQLite",
            DatabaseType::PostgreSQL => "PostgreSQL",
            DatabaseType::MySQL => "MySQL",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SslConfig {
    pub mode: SslMode,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
    pub ca_file: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SslMode {
    Disable,
    Require,
    VerifyCa,
    VerifyFull,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionConfig {
    pub name: String,
    pub database_type: DatabaseType,
    pub connection_string: String,
    pub ssl_config: Option<SslConfig>,
}

impl ConnectionConfig {
    pub fn new(name: String, connection_string: String) -> Result<Self> {
        let database_type = DatabaseType::from_url(&connection_string)?;
        Ok(Self {
            name,
            database_type,
            connection_string,
            ssl_config: None,
        })
    }

    pub fn with_ssl(mut self, ssl_config: SslConfig) -> Self {
        self.ssl_config = Some(ssl_config);
        self
    }
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub schema: Option<String>,
    pub row_count: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    #[allow(dead_code)]
    pub affected_rows: Option<u64>,
    pub execution_time: std::time::Duration,
    pub total_count: Option<usize>, // Add this field
}

#[derive(Debug)]
pub enum DatabasePool {
    SQLite(Pool<Sqlite>),
    PostgreSQL(Pool<Postgres>),
    MySQL(Pool<MySql>),
}

impl DatabasePool {
    pub async fn connect(config: &ConnectionConfig) -> Result<Self> {
        let connection_string = config.connection_string.clone();

        let pool = match config.database_type {
            DatabaseType::SQLite => {
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect(&connection_string)
                    .await?;
                DatabasePool::SQLite(pool)
            }
            DatabaseType::PostgreSQL => {
                let mut options = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(std::time::Duration::from_secs(120)); // Increase acquire timeout

                // Configure SSL if specified
                if let Some(ssl_config) = &config.ssl_config {
                    options = Self::configure_postgres_ssl(options, ssl_config)?;
                }

                let pool = options.connect(&connection_string).await?;
                DatabasePool::PostgreSQL(pool)
            }
            DatabaseType::MySQL => {
                let mut options = sqlx::mysql::MySqlPoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(std::time::Duration::from_secs(120)); // Increase acquire timeout
                // .connect_timeout(std::time::Duration::from_secs(60)); // Set connect timeout

                // Configure SSL if specified
                if let Some(ssl_config) = &config.ssl_config {
                    options = Self::configure_mysql_ssl(options, ssl_config)?;
                }

                let pool = options.connect(&connection_string).await?;
                DatabasePool::MySQL(pool)
            }
        };

        Ok(pool)
    }

    fn configure_postgres_ssl(
        options: sqlx::postgres::PgPoolOptions,
        ssl_config: &SslConfig,
    ) -> Result<sqlx::postgres::PgPoolOptions> {
        // For now, we'll just configure the SSL mode in the connection string
        // SQLx SSL configuration API may vary by version
        match ssl_config.mode {
            SslMode::Disable => {
                // SSL is disabled by default
            }
            SslMode::Require => {
                // Note: SSL configuration would be handled in the connection string
                // e.g., "postgresql://user:pass@host/db?sslmode=require"
            }
            SslMode::VerifyCa => {
                // Note: SSL configuration would be handled in the connection string
                // e.g., "postgresql://user:pass@host/db?sslmode=verify-ca&sslrootcert=ca.pem"
            }
            SslMode::VerifyFull => {
                // Note: SSL configuration would be handled in the connection string
                // e.g., "postgresql://user:pass@host/db?sslmode=verify-full&sslrootcert=ca.pem"
            }
        }

        Ok(options)
    }

    fn configure_mysql_ssl(
        options: sqlx::mysql::MySqlPoolOptions,
        ssl_config: &SslConfig,
    ) -> Result<sqlx::mysql::MySqlPoolOptions> {
        // For now, we'll just configure the SSL mode in the connection string
        // SQLx SSL configuration API may vary by version
        match ssl_config.mode {
            SslMode::Disable => {
                // SSL is disabled by default
            }
            SslMode::Require | SslMode::VerifyCa | SslMode::VerifyFull => {
                // Note: SSL configuration would be handled in the connection string
                // e.g., "mysql://user:pass@host/db?ssl-mode=REQUIRED"
            }
        }

        Ok(options)
    }

    pub async fn get_tables(&self) -> Result<Vec<TableInfo>> {
        match self {
            DatabasePool::SQLite(pool) => {
                let rows =
                    sqlx::query("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                        .fetch_all(pool)
                        .await?;

                let mut tables = Vec::new();
                for row in rows {
                    let name: String = row.get("name");
                    // Get row count
                    let count_query = format!("SELECT COUNT(*) as count FROM '{}'", name);
                    let count_row = sqlx::query(&count_query).fetch_one(pool).await?;
                    let row_count: i64 = count_row.get("count");

                    tables.push(TableInfo {
                        name,
                        schema: None,
                        row_count: Some(row_count),
                    });
                }
                Ok(tables)
            }
            DatabasePool::PostgreSQL(pool) => {
                let rows = sqlx::query(
                    "SELECT schemaname, tablename FROM pg_tables WHERE schemaname NOT IN ('information_schema', 'pg_catalog') ORDER BY schemaname, tablename"
                )
                .fetch_all(pool)
                .await?;

                let mut tables = Vec::new();
                for row in rows {
                    let schema: String = row.get("schemaname");
                    let name: String = row.get("tablename");

                    // Get row count
                    let count_query =
                        format!("SELECT COUNT(*) as count FROM \"{}\".\"{}\"", schema, name);
                    let count_result = sqlx::query(&count_query).fetch_one(pool).await;
                    let row_count = count_result.ok().map(|r| r.get::<i64, _>("count"));

                    tables.push(TableInfo {
                        name,
                        schema: Some(schema),
                        row_count,
                    });
                }
                Ok(tables)
            }
            DatabasePool::MySQL(pool) => {
                let rows = sqlx::query("SHOW TABLES").fetch_all(pool).await?;

                let mut tables = Vec::new();
                for row in rows {
                    let name: String = row.get(0);

                    // Get row count
                    let count_query = format!("SELECT COUNT(*) as count FROM `{}`", name);
                    let count_result = sqlx::query(&count_query).fetch_one(pool).await;
                    let row_count = count_result.ok().map(|r| r.get::<i64, _>("count"));

                    tables.push(TableInfo {
                        name,
                        schema: None,
                        row_count,
                    });
                }
                Ok(tables)
            }
        }
    }

    pub async fn get_table_columns(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Vec<ColumnInfo>> {
        match self {
            DatabasePool::SQLite(pool) => {
                let query = format!("PRAGMA table_info('{}')", table_name);
                let rows = sqlx::query(&query).fetch_all(pool).await?;

                let mut columns = Vec::new();
                for row in rows {
                    let name: String = row.get("name");
                    let data_type: String = row.get("type");
                    let not_null: i32 = row.get("notnull");
                    let pk: i32 = row.get("pk");

                    columns.push(ColumnInfo {
                        name,
                        data_type,
                        is_nullable: not_null == 0,
                        is_primary_key: pk > 0,
                    });
                }
                Ok(columns)
            }
            DatabasePool::PostgreSQL(pool) => {
                let query = if let Some(schema) = schema {
                    format!(
                        "SELECT column_name, data_type, is_nullable, 
                         CASE WHEN constraint_type = 'PRIMARY KEY' THEN true ELSE false END as is_primary_key
                         FROM information_schema.columns c
                         LEFT JOIN information_schema.key_column_usage kcu ON c.column_name = kcu.column_name AND c.table_name = kcu.table_name
                         LEFT JOIN information_schema.table_constraints tc ON kcu.constraint_name = tc.constraint_name
                         WHERE c.table_schema = '{}' AND c.table_name = '{}'
                         ORDER BY c.ordinal_position",
                        schema, table_name
                    )
                } else {
                    format!(
                        "SELECT column_name, data_type, is_nullable, false as is_primary_key
                         FROM information_schema.columns
                         WHERE table_name = '{}'
                         ORDER BY ordinal_position",
                        table_name
                    )
                };

                let rows = sqlx::query(&query).fetch_all(pool).await?;

                let mut columns = Vec::new();
                for row in rows {
                    let name: String = row.get("column_name");
                    let data_type: String = row.get("data_type");
                    let is_nullable: String = row.get("is_nullable");
                    let is_primary_key: bool = row.get("is_primary_key");

                    columns.push(ColumnInfo {
                        name,
                        data_type,
                        is_nullable: is_nullable == "YES",
                        is_primary_key,
                    });
                }
                Ok(columns)
            }
            DatabasePool::MySQL(pool) => {
                // Use DESCRIBE with better error handling for compatibility
                let query = format!("DESCRIBE `{}`", table_name);

                let rows = sqlx::query(&query).fetch_all(pool).await?;

                let mut columns = Vec::new();
                for row in rows {
                    // Use try_get with fallbacks to handle different data types safely
                    let name = match row.try_get::<String, _>("Field") {
                        Ok(n) => n,
                        Err(_) => {
                            // Try getting as bytes and convert if needed
                            if let Ok(bytes) = row.try_get::<Vec<u8>, _>("Field") {
                                String::from_utf8_lossy(&bytes).to_string()
                            } else {
                                continue; // Skip invalid rows
                            }
                        }
                    };

                    let data_type = match row.try_get::<String, _>("Type") {
                        Ok(t) => t,
                        Err(_) => {
                            // Try getting as bytes and convert if needed
                            if let Ok(bytes) = row.try_get::<Vec<u8>, _>("Type") {
                                String::from_utf8_lossy(&bytes).to_string()
                            } else {
                                "unknown".to_string()
                            }
                        }
                    };

                    let null = match row.try_get::<String, _>("Null") {
                        Ok(n) => n,
                        Err(_) => {
                            // Try getting as bytes and convert if needed
                            if let Ok(bytes) = row.try_get::<Vec<u8>, _>("Null") {
                                String::from_utf8_lossy(&bytes).to_string()
                            } else {
                                "YES".to_string() // Default to nullable if we can't read
                            }
                        }
                    };

                    let key = match row.try_get::<String, _>("Key") {
                        Ok(k) => k,
                        Err(_) => {
                            // Try getting as bytes and convert if needed
                            if let Ok(bytes) = row.try_get::<Vec<u8>, _>("Key") {
                                String::from_utf8_lossy(&bytes).to_string()
                            } else {
                                "".to_string()
                            }
                        }
                    };

                    columns.push(ColumnInfo {
                        name,
                        data_type,
                        is_nullable: null == "YES",
                        is_primary_key: key == "PRI",
                    });
                }
                Ok(columns)
            }
        }
    }

    pub async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        let start_time = std::time::Instant::now();

        match self {
            DatabasePool::SQLite(pool) => {
                let rows = sqlx::query(query).fetch_all(pool).await?;
                let execution_time = start_time.elapsed();

                if rows.is_empty() {
                    return Ok(QueryResult {
                        columns: vec![],
                        rows: vec![],
                        affected_rows: Some(0),
                        execution_time,
                        total_count: Some(0), // Add this
                    });
                }

                let columns: Vec<String> = rows[0]
                    .columns()
                    .iter()
                    .map(|col| col.name().to_string())
                    .collect();

                let mut result_rows = Vec::new();
                for row in rows {
                    let mut row_data = Vec::new();
                    for (i, _) in columns.iter().enumerate() {
                        // Try to get the value as a string, with fallbacks for different types
                        let value = match row.try_get::<String, _>(i) {
                            Ok(s) => s,
                            Err(_) => {
                                // Try other common types if string fails
                                if let Ok(i_val) = row.try_get::<i64, _>(i) {
                                    i_val.to_string()
                                } else if let Ok(f_val) = row.try_get::<f64, _>(i) {
                                    f_val.to_string()
                                } else if let Ok(b_val) = row.try_get::<bool, _>(i) {
                                    b_val.to_string()
                                } else if let Ok(d_val) =
                                    row.try_get::<chrono::DateTime<chrono::Utc>, _>(i)
                                {
                                    d_val.format("%Y-%m-%d %H:%M:%S").to_string()
                                } else {
                                    "NULL".to_string()
                                }
                            }
                        };
                        row_data.push(value);
                    }
                    result_rows.push(row_data);
                }

                Ok(QueryResult {
                    columns,
                    rows: result_rows,
                    affected_rows: None,
                    execution_time,
                    total_count: None, // Will be set by the caller
                })
            }
            DatabasePool::PostgreSQL(pool) => {
                let rows = sqlx::query(query).fetch_all(pool).await?;
                let execution_time = start_time.elapsed();

                if rows.is_empty() {
                    return Ok(QueryResult {
                        columns: vec![],
                        rows: vec![],
                        affected_rows: Some(0),
                        execution_time,
                        total_count: Some(0), // Add this
                    });
                }

                let columns: Vec<String> = rows[0]
                    .columns()
                    .iter()
                    .map(|col| col.name().to_string())
                    .collect();

                let mut result_rows = Vec::new();
                for row in rows {
                    let mut row_data = Vec::new();
                    for (i, _) in columns.iter().enumerate() {
                        // Try to get the value as a string, with fallbacks for different types
                        let value = match row.try_get::<String, _>(i) {
                            Ok(s) => s,
                            Err(_) => {
                                // Try other common types if string fails
                                if let Ok(i_val) = row.try_get::<i64, _>(i) {
                                    i_val.to_string()
                                } else if let Ok(f_val) = row.try_get::<f64, _>(i) {
                                    f_val.to_string()
                                } else if let Ok(b_val) = row.try_get::<bool, _>(i) {
                                    b_val.to_string()
                                } else if let Ok(d_val) =
                                    row.try_get::<chrono::DateTime<chrono::Utc>, _>(i)
                                {
                                    d_val.format("%Y-%m-%d %H:%M:%S").to_string()
                                } else {
                                    "NULL".to_string()
                                }
                            }
                        };
                        row_data.push(value);
                    }
                    result_rows.push(row_data);
                }

                Ok(QueryResult {
                    columns,
                    rows: result_rows,
                    affected_rows: None,
                    execution_time,
                    total_count: None, // Will be set by the caller
                })
            }
            DatabasePool::MySQL(pool) => {
                let rows = sqlx::query(query).fetch_all(pool).await?;
                let execution_time = start_time.elapsed();

                if rows.is_empty() {
                    return Ok(QueryResult {
                        columns: vec![],
                        rows: vec![],
                        affected_rows: Some(0),
                        execution_time,
                        total_count: Some(0), // Add this
                    });
                }

                let columns: Vec<String> = rows[0]
                    .columns()
                    .iter()
                    .map(|col| col.name().to_string())
                    .collect();

                let mut result_rows = Vec::new();
                for row in rows {
                    let mut row_data = Vec::new();
                    for (i, _) in columns.iter().enumerate() {
                        // Try to get the value as a string, with fallbacks for different types
                        let value = match row.try_get::<String, _>(i) {
                            Ok(s) => s,
                            Err(_) => {
                                // Try other common types if string fails
                                if let Ok(i_val) = row.try_get::<i64, _>(i) {
                                    i_val.to_string()
                                } else if let Ok(f_val) = row.try_get::<f64, _>(i) {
                                    f_val.to_string()
                                } else if let Ok(b_val) = row.try_get::<bool, _>(i) {
                                    b_val.to_string()
                                } else if let Ok(d_val) =
                                    row.try_get::<chrono::DateTime<chrono::Utc>, _>(i)
                                {
                                    d_val.format("%Y-%m-%d %H:%M:%S").to_string()
                                } else {
                                    "NULL".to_string()
                                }
                            }
                        };
                        row_data.push(value);
                    }
                    result_rows.push(row_data);
                }

                Ok(QueryResult {
                    columns,
                    rows: result_rows,
                    affected_rows: None,
                    execution_time,
                    total_count: None, // Will be set by the caller
                })
            }
        }
    }
}
