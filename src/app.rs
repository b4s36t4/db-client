use crate::database::{
    ColumnInfo, ConnectionConfig, DatabasePool, QueryResult, SslConfig, SslMode, TableInfo,
};
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    ConnectionList,
    NewConnection,
    EditConnection,
    TableBrowser,
    QueryEditor,
    QueryResults,
}

#[derive(Debug)]
pub struct App {
    pub current_screen: AppScreen,
    pub should_quit: bool,
    pub connections: Vec<ConnectionConfig>,
    pub selected_connection_index: usize,
    pub current_connection: Option<usize>,
    pub database_pool: Option<DatabasePool>,

    // Connection form state
    pub connection_form: ConnectionForm,
    pub editing_connection_index: Option<usize>, // Index of connection being edited

    // Table browser state
    pub tables: Vec<TableInfo>,
    pub selected_table_index: usize,
    pub table_columns: Vec<ColumnInfo>,

    // Query editor state
    pub query_input: String,
    pub query_cursor_position: usize,
    pub query_history: Vec<String>,
    #[allow(dead_code)]
    pub query_history_index: Option<usize>,

    // Query results state
    pub current_query_result: Option<QueryResult>,
    pub result_scroll_x: usize,
    pub result_scroll_y: usize,
    pub selected_column_index: usize,
    pub current_page: usize,
    pub results_per_page: usize,
    pub selected_row_index: usize,

    // UI state
    pub show_help: bool,
    pub error_message: Option<String>,
    pub status_message: Option<String>,
    pub is_connecting: bool,  // Loading state for connection
    pub spinner_frame: usize, // Animation frame for loading spinner
    pub connection_task: Option<tokio::task::JoinHandle<Result<DatabasePool, anyhow::Error>>>, // Handle for connection task
    pub cancel_token: Option<tokio_util::sync::CancellationToken>, // Token to cancel connection
}

#[derive(Debug, Clone)]
pub struct ConnectionForm {
    pub name: String,
    pub connection_string: String,
    pub current_field: ConnectionField,

    // Individual connection fields
    pub database_type: crate::database::DatabaseType,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,

    // SSL configuration
    pub use_ssl: bool,
    pub ssl_mode: SslMode,
    pub ssl_cert_file: String,
    pub ssl_key_file: String,
    pub ssl_ca_file: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionField {
    Name,
    ConnectionString,
    DatabaseType,
    Host,
    Port,
    Username,
    Password,
    Database,

    UseSsl,
    SslMode,
    SslCertFile,
    SslKeyFile,
    SslCaFile,
}

impl ConnectionForm {
    pub fn next_field(&mut self) {
        self.current_field = match self.current_field {
            ConnectionField::Name => ConnectionField::ConnectionString,
            ConnectionField::ConnectionString => ConnectionField::DatabaseType,
            ConnectionField::DatabaseType => ConnectionField::Host,
            ConnectionField::Host => ConnectionField::Port,
            ConnectionField::Port => ConnectionField::Username,
            ConnectionField::Username => ConnectionField::Password,
            ConnectionField::Password => ConnectionField::Database,
            ConnectionField::Database => ConnectionField::UseSsl,
            ConnectionField::UseSsl => {
                if self.use_ssl {
                    ConnectionField::SslMode
                } else {
                    ConnectionField::Name
                }
            }
            ConnectionField::SslMode => ConnectionField::SslCertFile,
            ConnectionField::SslCertFile => ConnectionField::SslKeyFile,
            ConnectionField::SslKeyFile => ConnectionField::SslCaFile,
            ConnectionField::SslCaFile => ConnectionField::Name,
        };
    }

    pub fn previous_field(&mut self) {
        self.current_field = match self.current_field {
            ConnectionField::Name => ConnectionField::SslCaFile,
            ConnectionField::ConnectionString => ConnectionField::Name,
            ConnectionField::DatabaseType => ConnectionField::ConnectionString,
            ConnectionField::Host => ConnectionField::DatabaseType,
            ConnectionField::Port => ConnectionField::Host,
            ConnectionField::Username => ConnectionField::Port,
            ConnectionField::Password => ConnectionField::Username,
            ConnectionField::Database => ConnectionField::Password,
            ConnectionField::UseSsl => ConnectionField::Database,
            ConnectionField::SslMode => ConnectionField::UseSsl,
            ConnectionField::SslCertFile => ConnectionField::SslMode,
            ConnectionField::SslKeyFile => ConnectionField::SslCertFile,
            ConnectionField::SslCaFile => ConnectionField::SslKeyFile,
        };
    }

    pub fn toggle_ssl(&mut self) {
        self.use_ssl = !self.use_ssl;
        if !self.use_ssl {
            // Reset SSL fields when disabled
            self.ssl_cert_file.clear();
            self.ssl_key_file.clear();
            self.ssl_ca_file.clear();
        }
    }

    pub fn cycle_ssl_mode(&mut self) {
        self.ssl_mode = match self.ssl_mode {
            SslMode::Disable => SslMode::Require,
            SslMode::Require => SslMode::VerifyCa,
            SslMode::VerifyCa => SslMode::VerifyFull,
            SslMode::VerifyFull => SslMode::Disable,
        };
    }

    pub fn get_current_field_value(&self) -> &str {
        self.get_field_value(self.current_field.clone())
    }

    pub fn get_field_value(&self, field: ConnectionField) -> &str {
        match field {
            ConnectionField::Name => &self.name,
            ConnectionField::ConnectionString => &self.connection_string,
            ConnectionField::DatabaseType => self.database_type.display_name(),
            ConnectionField::Host => &self.host,
            ConnectionField::Port => &self.port,
            ConnectionField::Username => &self.username,
            ConnectionField::Password => &self.password,
            ConnectionField::Database => &self.database,

            ConnectionField::UseSsl => {
                if self.use_ssl {
                    "Yes"
                } else {
                    "No"
                }
            }
            ConnectionField::SslMode => match self.ssl_mode {
                SslMode::Disable => "Disable",
                SslMode::Require => "Require",
                SslMode::VerifyCa => "Verify CA",
                SslMode::VerifyFull => "Verify Full",
            },
            ConnectionField::SslCertFile => &self.ssl_cert_file,
            ConnectionField::SslKeyFile => &self.ssl_key_file,
            ConnectionField::SslCaFile => &self.ssl_ca_file,
        }
    }

    pub fn set_current_field_value(&mut self, value: String) {
        match self.current_field {
            ConnectionField::Name => self.name = value,
            ConnectionField::ConnectionString => self.connection_string = value,
            ConnectionField::Host => self.host = value,
            ConnectionField::Port => self.port = value,
            ConnectionField::Username => self.username = value,
            ConnectionField::Password => self.password = value,
            ConnectionField::Database => self.database = value,
            ConnectionField::SslCertFile => self.ssl_cert_file = value,
            ConnectionField::SslKeyFile => self.ssl_key_file = value,
            ConnectionField::SslCaFile => self.ssl_ca_file = value,
            _ => {} // Toggle fields don't accept string input
        }
    }

    pub fn is_toggle_field(&self) -> bool {
        matches!(
            self.current_field,
            ConnectionField::UseSsl | ConnectionField::SslMode | ConnectionField::DatabaseType
        )
    }

    pub fn is_field_toggle(&self, field: &ConnectionField) -> bool {
        matches!(
            field,
            ConnectionField::UseSsl | ConnectionField::SslMode | ConnectionField::DatabaseType
        )
    }

    pub fn cycle_database_type(&mut self) {
        self.database_type = match self.database_type {
            crate::database::DatabaseType::SQLite => crate::database::DatabaseType::PostgreSQL,
            crate::database::DatabaseType::PostgreSQL => crate::database::DatabaseType::MySQL,
            crate::database::DatabaseType::MySQL => crate::database::DatabaseType::SQLite,
        };
        // Update default port when database type changes
        self.port = match self.database_type {
            crate::database::DatabaseType::SQLite => "".to_string(),
            crate::database::DatabaseType::PostgreSQL => "5432".to_string(),
            crate::database::DatabaseType::MySQL => "3306".to_string(),
        };
    }

    pub fn build_connection_string(&self) -> Option<String> {
        // If connection string is already provided, use it
        if !self.connection_string.is_empty() {
            return Some(self.connection_string.clone());
        }

        // Build from individual fields
        if self.host.is_empty() {
            return None; // Host is required
        }

        match self.database_type {
            crate::database::DatabaseType::SQLite => {
                // SQLite uses file path, not host/port
                Some(format!("sqlite:{}", self.host))
            }
            crate::database::DatabaseType::PostgreSQL => {
                let port = if self.port.is_empty() {
                    "5432"
                } else {
                    &self.port
                };
                // URL encode username, password, and database name to handle special characters
                let encoded_username = urlencoding::encode(&self.username);
                let encoded_password = urlencoding::encode(&self.password);
                let encoded_database = urlencoding::encode(&self.database);

                if self.username.is_empty() {
                    Some(format!(
                        "postgresql://{}:{}/{}",
                        self.host, port, encoded_database
                    ))
                } else if self.password.is_empty() {
                    Some(format!(
                        "postgresql://{}@{}:{}/{}",
                        encoded_username, self.host, port, encoded_database
                    ))
                } else {
                    Some(format!(
                        "postgresql://{}:{}@{}:{}/{}",
                        encoded_username, encoded_password, self.host, port, encoded_database
                    ))
                }
            }
            crate::database::DatabaseType::MySQL => {
                let port = if self.port.is_empty() {
                    "3306"
                } else {
                    &self.port
                };
                // URL encode username, password, and database name to handle special characters
                let encoded_username = urlencoding::encode(&self.username);
                let encoded_password = urlencoding::encode(&self.password);
                let encoded_database = urlencoding::encode(&self.database);

                if self.username.is_empty() {
                    Some(format!(
                        "mysql://{}:{}/{}",
                        self.host, port, encoded_database
                    ))
                } else if self.password.is_empty() {
                    Some(format!(
                        "mysql://{}@{}:{}/{}",
                        encoded_username, self.host, port, encoded_database
                    ))
                } else {
                    Some(format!(
                        "mysql://{}:{}@{}:{}/{}",
                        encoded_username, encoded_password, self.host, port, encoded_database
                    ))
                }
            }
        }
    }
}

impl Default for ConnectionForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            connection_string: String::new(),
            current_field: ConnectionField::Name,
            database_type: crate::database::DatabaseType::PostgreSQL, // Default to PostgreSQL
            host: "localhost".to_string(),
            port: "5432".to_string(), // Default PostgreSQL port
            username: String::new(),
            password: String::new(),
            database: String::new(),
            use_ssl: false,
            ssl_mode: SslMode::Disable,
            ssl_cert_file: String::new(),
            ssl_key_file: String::new(),
            ssl_ca_file: String::new(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            current_screen: AppScreen::ConnectionList,
            should_quit: false,
            connections: Self::default_connections(),
            selected_connection_index: 0,
            current_connection: None,
            database_pool: None,
            connection_form: ConnectionForm::default(),
            editing_connection_index: None,
            tables: Vec::new(),
            selected_table_index: 0,
            table_columns: Vec::new(),
            query_input: String::new(),
            query_cursor_position: 0,
            query_history: Vec::new(),
            query_history_index: None,
            current_query_result: None,
            result_scroll_x: 0,
            result_scroll_y: 0,
            selected_column_index: 0,
            current_page: 0,
            results_per_page: 50,
            selected_row_index: 0, // Add this field
            show_help: false,
            error_message: None,
            status_message: None,
            is_connecting: false,
            spinner_frame: 0,
            connection_task: None,
            cancel_token: None,
        };

        // Try to load saved connections, ignore errors
        let _ = app.load_connections();

        app
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    fn default_connections() -> Vec<ConnectionConfig> {
        vec![
            ConnectionConfig {
                name: "Sample SQLite".to_string(),
                database_type: crate::database::DatabaseType::SQLite,
                connection_string: "sqlite::memory:".to_string(),
                ssl_config: None,
            },
            ConnectionConfig {
                name: "Local PostgreSQL".to_string(),
                database_type: crate::database::DatabaseType::PostgreSQL,
                connection_string: "postgresql://user:password@localhost/dbname".to_string(),
                ssl_config: None,
            },
            ConnectionConfig {
                name: "Local MySQL".to_string(),
                database_type: crate::database::DatabaseType::MySQL,
                connection_string: "mysql://user:password@localhost/dbname".to_string(),
                ssl_config: None,
            },
        ]
    }

    pub fn start_connection(&mut self, connection_index: usize) -> Result<()> {
        if connection_index >= self.connections.len() {
            return Err(anyhow::anyhow!("Invalid connection index"));
        }

        // Cancel any existing connection attempt
        self.cancel_connection();

        let config = self.connections[connection_index].clone();
        let cancel_token = tokio_util::sync::CancellationToken::new();

        self.status_message = Some(format!("Connecting to {}...", config.name));
        self.is_connecting = true;
        self.cancel_token = Some(cancel_token.clone());

        let task =
            tokio::spawn(
                async move { Self::perform_connection(config, cancel_token.clone()).await },
            );

        self.connection_task = Some(task);
        Ok(())
    }

    async fn perform_connection(
        config: ConnectionConfig,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<DatabasePool, anyhow::Error> {
        // Add timeout for the entire connection process
        let timeout_duration = tokio::time::Duration::from_secs(120);

        tokio::select! {
            result = tokio::time::timeout(timeout_duration, DatabasePool::connect(&config)) => {
                match result {
                    Ok(pool) => {
                        pool
                    }
                    Err(e) => {
                        Err(anyhow::anyhow!("Connection failed: {}", e))
                    }
                }
            }
            _ = cancel_token.cancelled() => {
                Err(anyhow::anyhow!("Connection cancelled"))
            }
        }
    }

    pub async fn refresh_tables(&mut self) -> Result<()> {
        if let Some(pool) = &self.database_pool {
            match pool.get_tables().await {
                Ok(tables) => {
                    self.tables = tables;
                    self.selected_table_index = 0;
                    if !self.tables.is_empty() {
                        self.refresh_table_columns().await?;
                    }
                    Ok(())
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load tables: {}", e));
                    Err(e)
                }
            }
        } else {
            Err(anyhow::anyhow!("No database connection"))
        }
    }

    pub async fn refresh_table_columns(&mut self) -> Result<()> {
        if let Some(pool) = &self.database_pool {
            if let Some(table) = self.tables.get(self.selected_table_index) {
                match pool
                    .get_table_columns(&table.name, table.schema.as_deref())
                    .await
                {
                    Ok(columns) => {
                        self.table_columns = columns;
                        Ok(())
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load table columns: {}", e));
                        Err(e)
                    }
                }
            } else {
                Ok(())
            }
        } else {
            Err(anyhow::anyhow!("No database connection"))
        }
    }

    pub async fn execute_query(&mut self, query: &str) -> Result<()> {
        if let Some(pool) = &self.database_pool {
            self.status_message = Some("Executing query...".to_string());

            // For SELECT queries, first get the total count without LIMIT
            let total_count = if query.trim().to_uppercase().starts_with("SELECT") {
                let count_query = self.generate_count_query(query);
                match pool.execute_query(&count_query).await {
                    Ok(count_result) => {
                        if let Some(first_row) = count_result.rows.first() {
                            first_row
                                .first()
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(0)
                        } else {
                            0
                        }
                    }
                    Err(_) => 0, // If count fails, default to 0
                }
            } else {
                0
            };

            // Auto-add LIMIT if it's a SELECT query without one
            let modified_query = self.auto_limit_query(query);

            match pool.execute_query(&modified_query).await {
                Ok(mut result) => {
                    // Store the total count in the result
                    result.total_count = Some(total_count);
                    self.current_query_result = Some(result);
                    self.current_screen = AppScreen::QueryResults;
                    self.result_scroll_x = 0;
                    self.result_scroll_y = 0;
                    self.selected_column_index = 0;
                    self.selected_row_index = 0; // Reset row selection
                    self.current_page = 0;
                    self.status_message = Some("Query executed successfully".to_string());
                    self.error_message = None;

                    // Add to history if not already there
                    if !self.query_history.contains(&query.to_string()) {
                        self.query_history.push(query.to_string());
                        if self.query_history.len() > 50 {
                            self.query_history.remove(0);
                        }
                    }

                    Ok(())
                }
                Err(e) => {
                    self.error_message = Some(format!("Query failed: {}", e));
                    self.status_message = None;
                    Err(e)
                }
            }
        } else {
            Err(anyhow::anyhow!("No database connection"))
        }
    }

    pub fn add_connection(&mut self, name: String, connection_string: String) -> Result<()> {
        let config = ConnectionConfig::new(name, connection_string)?;
        self.connections.push(config);
        Ok(())
    }

    pub async fn remove_connection(&mut self, index: usize) -> Result<()> {
        if index < self.connections.len() {
            self.connections.remove(index);
            if let Some(current) = self.current_connection {
                if current == index {
                    self.current_connection = None;
                    self.database_pool = None;
                    self.current_screen = AppScreen::ConnectionList;
                } else if current > index {
                    self.current_connection = Some(current - 1);
                }
            }
        }
        Ok(())
    }

    pub fn start_editing_connection(&mut self, index: usize) -> Result<()> {
        if index >= self.connections.len() {
            return Err(anyhow::anyhow!("Invalid connection index"));
        }

        let config = &self.connections[index];

        // Populate form with existing connection data
        self.connection_form.name = config.name.clone();
        self.connection_form.connection_string = config.connection_string.clone();
        self.connection_form.database_type = config.database_type.clone();

        // Parse connection string to populate individual fields if possible
        // For now, we'll keep it simple and just set the connection string
        // More sophisticated parsing could be added later

        // Set SSL config if present
        if let Some(ssl_config) = &config.ssl_config {
            self.connection_form.use_ssl = true;
            self.connection_form.ssl_mode = ssl_config.mode.clone();
            if let Some(cert_file) = &ssl_config.cert_file {
                self.connection_form.ssl_cert_file = cert_file.clone();
            }
            if let Some(key_file) = &ssl_config.key_file {
                self.connection_form.ssl_key_file = key_file.clone();
            }
            if let Some(ca_file) = &ssl_config.ca_file {
                self.connection_form.ssl_ca_file = ca_file.clone();
            }
        } else {
            self.connection_form.use_ssl = false;
        }

        // Reset form state
        self.connection_form.current_field = ConnectionField::Name;
        self.editing_connection_index = Some(index);
        self.current_screen = AppScreen::EditConnection;

        Ok(())
    }

    pub fn save_edited_connection(&mut self) -> Result<()> {
        let index = match self.editing_connection_index {
            Some(idx) => idx,
            None => return Err(anyhow::anyhow!("No connection being edited")),
        };

        if index >= self.connections.len() {
            return Err(anyhow::anyhow!("Invalid connection index"));
        }

        // Build connection string from individual fields or use provided string
        let connection_string = match self.connection_form.build_connection_string() {
            Some(cs) => cs,
            None => {
                return Err(anyhow::anyhow!(
                    "Please provide either a connection string or fill in the individual fields (at least Host is required)"
                ));
            }
        };

        // Create connection config with SSL settings
        let mut config =
            match ConnectionConfig::new(self.connection_form.name.clone(), connection_string) {
                Ok(config) => config,
                Err(e) => {
                    return Err(anyhow::anyhow!("Invalid connection: {}", e));
                }
            };

        // Add SSL configuration if enabled
        if self.connection_form.use_ssl {
            let ssl_config = SslConfig {
                mode: self.connection_form.ssl_mode.clone(),
                cert_file: if self.connection_form.ssl_cert_file.is_empty() {
                    None
                } else {
                    Some(self.connection_form.ssl_cert_file.clone())
                },
                key_file: if self.connection_form.ssl_key_file.is_empty() {
                    None
                } else {
                    Some(self.connection_form.ssl_key_file.clone())
                },
                ca_file: if self.connection_form.ssl_ca_file.is_empty() {
                    None
                } else {
                    Some(self.connection_form.ssl_ca_file.clone())
                },
            };

            config = config.with_ssl(ssl_config);
        }

        // Update the connection
        self.connections[index] = config;

        // Save connections to disk
        if let Err(e) = self.save_connections() {
            return Err(anyhow::anyhow!("Failed to save connections: {}", e));
        }

        // Reset editing state
        self.editing_connection_index = None;
        self.current_screen = AppScreen::ConnectionList;
        Ok(())
    }

    pub fn next_table(&mut self) {
        if !self.tables.is_empty() {
            self.selected_table_index = (self.selected_table_index + 1) % self.tables.len();
        }
    }

    pub fn previous_table(&mut self) {
        if !self.tables.is_empty() {
            if self.selected_table_index == 0 {
                self.selected_table_index = self.tables.len() - 1;
            } else {
                self.selected_table_index -= 1;
            }
        }
    }

    pub fn get_selected_table(&self) -> Option<&TableInfo> {
        self.tables.get(self.selected_table_index)
    }

    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.status_message = None;
    }

    pub fn update_spinner(&mut self) {
        if self.is_connecting {
            self.spinner_frame = (self.spinner_frame + 1) % 4;
        }
    }

    pub fn get_spinner_char(&self) -> char {
        if self.is_connecting {
            match self.spinner_frame {
                0 => '|',
                1 => '/',
                2 => '-',
                3 => '\\',
                _ => '|',
            }
        } else {
            ' '
        }
    }

    pub fn cancel_connection(&mut self) {
        if let Some(cancel_token) = &self.cancel_token {
            cancel_token.cancel();
        }
        if let Some(task) = self.connection_task.take() {
            task.abort();
        }
        self.is_connecting = false;
        self.status_message = Some("Connection cancelled".to_string());
        self.connection_task = None;
        self.cancel_token = None;
    }

    pub async fn check_connection_task(&mut self) {
        if let Some(task) = self.connection_task.take() {
            if task.is_finished() {
                // Connection task completed, get the result
                match task.await {
                    Ok(Ok(pool)) => {
                        self.database_pool = Some(pool);
                        self.current_connection = Some(self.selected_connection_index);
                        self.current_screen = AppScreen::TableBrowser;
                        self.status_message = Some(format!(
                            "Connected to {}",
                            self.connections[self.selected_connection_index].name
                        ));
                        self.error_message = None;
                        self.is_connecting = false;

                        // Load tables
                        if let Err(e) = self.refresh_tables().await {
                            self.error_message = Some(format!("Failed to load tables: {}", e));
                        }
                    }
                    Ok(Err(e)) => {
                        self.error_message = Some(format!("Connection failed: {}", e));
                        self.status_message = None;
                        self.is_connecting = false;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Connection task panicked: {}", e));
                        self.status_message = None;
                        self.is_connecting = false;
                    }
                }

                self.connection_task = None;
                self.cancel_token = None;
            } else {
                // Task is still running, put it back
                self.connection_task = Some(task);
            }
        }
    }

    pub fn generate_select_query(&self) -> String {
        if let Some(table) = self.get_selected_table() {
            let table_name = if let Some(schema) = &table.schema {
                format!(r"`{}`.`{}`", schema, table.name)
            } else {
                format!(r"`{}`", table.name)
            };
            format!("SELECT * FROM {} LIMIT 100;", table_name)
        } else {
            "SELECT 1;".to_string()
        }
    }

    pub fn insert_char_in_query(&mut self, c: char) {
        self.query_input.insert(self.query_cursor_position, c);
        self.query_cursor_position += 1;
    }

    pub fn delete_char_in_query(&mut self) {
        if self.query_cursor_position > 0 {
            self.query_cursor_position -= 1;
            self.query_input.remove(self.query_cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.query_cursor_position > 0 {
            self.query_cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.query_cursor_position < self.query_input.len() {
            self.query_cursor_position += 1;
        }
    }

    pub fn move_cursor_to_start(&mut self) {
        self.query_cursor_position = 0;
    }

    pub fn move_cursor_to_end(&mut self) {
        self.query_cursor_position = self.query_input.len();
    }

    pub fn clear_query(&mut self) {
        self.query_input.clear();
        self.query_cursor_position = 0;
    }

    pub fn next_connection(&mut self) {
        if !self.connections.is_empty() {
            self.selected_connection_index =
                (self.selected_connection_index + 1) % self.connections.len();
        }
    }

    pub fn previous_connection(&mut self) {
        if !self.connections.is_empty() {
            if self.selected_connection_index == 0 {
                self.selected_connection_index = self.connections.len() - 1;
            } else {
                self.selected_connection_index -= 1;
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_selected_connection(&self) -> Option<&ConnectionConfig> {
        self.connections.get(self.selected_connection_index)
    }

    pub fn next_column(&mut self) {
        if let Some(result) = &self.current_query_result {
            if self.selected_column_index < result.columns.len().saturating_sub(1) {
                self.selected_column_index += 1;
            }
        }
    }

    pub fn previous_column(&mut self) {
        if self.selected_column_index > 0 {
            self.selected_column_index -= 1;
        }
    }

    pub fn next_page(&mut self) {
        let total_pages = self.get_total_pages();
        if self.current_page < total_pages.saturating_sub(1) {
            self.current_page += 1;
            self.result_scroll_y = 0; // Reset vertical scroll when changing pages
            self.selected_row_index = 0; // Reset row selection when changing pages
        }
    }

    pub fn previous_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.result_scroll_y = 0; // Reset vertical scroll when changing pages
            self.selected_row_index = 0; // Reset row selection when changing pages
        }
    }

    // Add row navigation methods
    pub fn next_row(&mut self) {
        if let Some(_result) = &self.current_query_result {
            let current_page_results = self.get_current_page_results();
            if self.selected_row_index < current_page_results.len().saturating_sub(1) {
                self.selected_row_index += 1;
                // Auto-scroll if selected row goes out of view
                if self.selected_row_index >= self.result_scroll_y + 10 {
                    // Assuming visible height is ~10 rows
                    self.result_scroll_y = self.selected_row_index.saturating_sub(9);
                }
            }
        }
    }

    pub fn previous_row(&mut self) {
        if self.selected_row_index > 0 {
            self.selected_row_index -= 1;
            // Auto-scroll if selected row goes out of view
            if self.selected_row_index < self.result_scroll_y {
                self.result_scroll_y = self.selected_row_index;
            }
        }
    }

    pub fn get_current_page_results(&self) -> Vec<Vec<String>> {
        if let Some(result) = &self.current_query_result {
            let start = self.current_page * self.results_per_page;
            let end = std::cmp::min(start + self.results_per_page, result.rows.len());
            if start < result.rows.len() {
                result.rows[start..end].to_vec()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    pub fn get_total_pages(&self) -> usize {
        if let Some(result) = &self.current_query_result {
            // Use total_count if available, otherwise fall back to current rows
            let total_rows = result.total_count.unwrap_or(result.rows.len());
            if total_rows == 0 {
                0
            } else {
                (total_rows + self.results_per_page - 1) / self.results_per_page
            }
        } else {
            0
        }
    }

    pub fn auto_limit_query(&self, query: &str) -> String {
        let query_upper = query.to_uppercase();
        if !query_upper.contains("LIMIT") && query_upper.contains("SELECT") {
            format!(
                "{} LIMIT {}",
                query.trim_end_matches(';'),
                self.results_per_page
            )
        } else {
            query.to_string()
        }
    }

    pub fn save_connections(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("rata-db");

        fs::create_dir_all(&config_dir)?;

        let config_file = config_dir.join("connections.json");
        let json = serde_json::to_string_pretty(&self.connections)?;
        fs::write(config_file, json)?;

        Ok(())
    }

    pub fn load_connections(&mut self) -> Result<()> {
        let config_file = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("rata-db")
            .join("connections.json");

        if config_file.exists() {
            let content = fs::read_to_string(config_file)?;
            let connections: Vec<ConnectionConfig> = serde_json::from_str(&content)?;
            self.connections = connections;
        }

        Ok(())
    }

    // Add helper functions for SQL generation
    pub fn generate_count_query(&self, query: &str) -> String {
        let query_upper = query.trim().to_uppercase();

        // Remove existing LIMIT clause
        let query_without_limit = if let Some(limit_pos) = query_upper.rfind("LIMIT") {
            query[..limit_pos].trim()
        } else {
            query.trim()
        };

        // Remove trailing semicolon
        let query_clean = query_without_limit.trim_end_matches(';');

        // Extract FROM clause and everything after it
        if let Some(from_pos) = query_upper.find("FROM") {
            let from_clause = &query_clean[from_pos..];
            format!("SELECT COUNT(*) {}", from_clause)
        } else {
            // If no FROM clause found, just wrap the entire query
            format!("SELECT COUNT(*) FROM ({})", query_clean)
        }
    }

    pub fn generate_insert_statement(
        &self,
        table_name: &str,
        columns: &[String],
        values: &[String],
    ) -> String {
        let columns_str = columns.join(", ");
        let values_str = values
            .iter()
            .map(|v| {
                if v == "NULL" {
                    "NULL".to_string()
                } else {
                    format!("'{}'", v.replace("'", "''"))
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "INSERT INTO {} ({}) VALUES ({});",
            table_name, columns_str, values_str
        )
    }

    pub fn generate_create_table_statement(
        &self,
        table_name: &str,
        columns: &[ColumnInfo],
    ) -> String {
        let column_definitions: Vec<String> = columns
            .iter()
            .map(|col| {
                let mut def = format!("{} {}", col.name, col.data_type);
                if !col.is_nullable {
                    def.push_str(" NOT NULL");
                }
                if col.is_primary_key {
                    def.push_str(" PRIMARY KEY");
                }
                def
            })
            .collect();

        format!(
            "CREATE TABLE {} (\n  {}\n);",
            table_name,
            column_definitions.join(",\n  ")
        )
    }

    #[allow(dead_code)]
    pub fn generate_alter_table_add_column(&self, table_name: &str, column: &ColumnInfo) -> String {
        let mut def = format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            table_name, column.name, column.data_type
        );

        if !column.is_nullable {
            def.push_str(" NOT NULL");
        }

        if column.is_primary_key {
            def.push_str(" PRIMARY KEY");
        }

        def.push(';');
        def
    }

    #[allow(dead_code)]
    pub fn generate_drop_table_statement(&self, table_name: &str) -> String {
        format!("DROP TABLE {};", table_name)
    }

    pub fn generate_select_star_statement(&self, table_name: &str, limit: Option<usize>) -> String {
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        format!("SELECT * FROM {}{};", table_name, limit_clause)
    }

    pub fn generate_delete_statement(
        &self,
        table_name: &str,
        where_clause: Option<&str>,
    ) -> String {
        match where_clause {
            Some(where_cl) => format!("DELETE FROM {} WHERE {};", table_name, where_cl),
            None => format!("DELETE FROM {};", table_name),
        }
    }

    pub fn generate_update_statement(
        &self,
        table_name: &str,
        set_clause: &str,
        where_clause: Option<&str>,
    ) -> String {
        match where_clause {
            Some(where_cl) => format!(
                "UPDATE {} SET {} WHERE {};",
                table_name, set_clause, where_cl
            ),
            None => format!("UPDATE {} SET {};", table_name, set_clause),
        }
    }

    #[allow(dead_code)]
    // Additional helper functions for common database operations
    pub fn generate_index_statement(
        &self,
        table_name: &str,
        index_name: &str,
        columns: &[String],
    ) -> String {
        let columns_str = columns.join(", ");
        format!(
            "CREATE INDEX {} ON {} ({});",
            index_name, table_name, columns_str
        )
    }

    #[allow(dead_code)]
    pub fn generate_view_statement(&self, view_name: &str, select_query: &str) -> String {
        format!("CREATE VIEW {} AS {};", view_name, select_query)
    }

    pub fn generate_truncate_statement(&self, table_name: &str) -> String {
        format!("TRUNCATE TABLE {};", table_name)
    }

    #[allow(dead_code)]
    pub fn generate_rename_table_statement(&self, old_name: &str, new_name: &str) -> String {
        format!("ALTER TABLE {} RENAME TO {};", old_name, new_name)
    }

    #[allow(dead_code)]
    pub fn generate_add_foreign_key_statement(
        &self,
        table_name: &str,
        column: &str,
        reference_table: &str,
        reference_column: &str,
    ) -> String {
        format!(
            "ALTER TABLE {} ADD CONSTRAINT fk_{}_{} FOREIGN KEY ({}) REFERENCES {}({});",
            table_name, table_name, column, column, reference_table, reference_column
        )
    }

    #[allow(dead_code)]
    pub fn generate_analyze_statement(&self, table_name: &str) -> String {
        format!("ANALYZE {};", table_name)
    }

    #[allow(dead_code)]
    pub fn generate_vacuum_statement(&self) -> String {
        "VACUUM;".to_string()
    }

    #[allow(dead_code)]
    pub fn generate_backup_statement(&self, table_name: &str, backup_table: &str) -> String {
        format!(
            "CREATE TABLE {} AS SELECT * FROM {};",
            backup_table, table_name
        )
    }

    // File selection helpers

    #[cfg(not(target_arch = "wasm32"))]
    pub fn select_ssl_certificate_file() -> Option<String> {
        FileDialog::new()
            .add_filter("Certificate Files", &["crt", "pem", "cer", "der"])
            .add_filter("All Files", &["*"])
            .set_title("Select SSL Certificate")
            .pick_file()
            .map(|path| path.to_string_lossy().to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn select_ssl_key_file() -> Option<String> {
        FileDialog::new()
            .add_filter("Key Files", &["key", "pem"])
            .add_filter("All Files", &["*"])
            .set_title("Select SSL Private Key")
            .pick_file()
            .map(|path| path.to_string_lossy().to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn select_ssl_ca_file() -> Option<String> {
        FileDialog::new()
            .add_filter("CA Certificate Files", &["crt", "pem", "cer"])
            .add_filter("All Files", &["*"])
            .set_title("Select SSL CA Certificate")
            .pick_file()
            .map(|path| path.to_string_lossy().to_string())
    }
}
