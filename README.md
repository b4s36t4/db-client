# Rata-DB - Terminal Database Client

A modern, developer-friendly TUI (Terminal User Interface) database client built with Rust, supporting MySQL, PostgreSQL, and SQLite databases.

## Features

- **Multi-Database Support**: Connect to MySQL, PostgreSQL, and SQLite databases
- **Beautiful TUI Interface**: Clean, modern terminal interface built with Ratatui
- **Interactive Navigation**: Keyboard-driven navigation with visual feedback
- **Connection Management**: Save and manage multiple database connections
- **Table Browser**: Browse tables, view schemas, and column information
- **SQL Query Editor**: Write and execute SQL queries with cursor position tracking
- **Results Viewer**: View query results in a tabular format with scrolling support
- **Error Handling**: Comprehensive error messages and status updates
- **Demo Database**: Built-in demo SQLite database with sample data

## Installation

### Prerequisites

- Rust (1.70 or later)
- Cargo

### Build from Source

```bash
git clone <repository-url>
cd rata-db
cargo build --release
```

The binary will be available at `target/release/rata-db`

## Usage

### Starting the Application

```bash
# Run the application
./target/release/rata-db

# Or during development
cargo run
```

### Creating Demo Database

To create a demo SQLite database with sample data:

```bash
./target/release/rata-db --create-demo
# Or during development
cargo run -- --create-demo
```

This creates a `demo.db` file with sample tables (`users`, `orders`, `categories`) and data.

## Interface Guide

### Connection Management

- **Navigation**: Use ↑/↓ arrow keys to navigate connections
- **Connect**: Press `Enter` to connect to selected database
- **New Connection**: Press `n` to create a new connection
- **Delete Connection**: Press `d` to delete selected connection
- **Connected Status**: Connected databases show a green ● indicator

### Connection Form

- **Field Navigation**: Use `Tab` to switch between name and connection string fields
- **Cursor Visualization**: Active field shows cursor position with `|`
- **Save**: Press `Enter` to save the connection
- **Cancel**: Press `Esc` to cancel

#### Connection String Examples

```
# SQLite
sqlite:database.db
sqlite::memory:

# PostgreSQL
postgresql://user:password@localhost/database_name

# MySQL
mysql://user:password@localhost/database_name
```

### Table Browser

- **Navigation**: Use ↑/↓ to navigate between tables
- **Column View**: Selected table's columns are shown on the right
- **Quick SELECT**: Press `s` to generate a SELECT query for the current table
- **Query Editor**: Press `q` to open the query editor
- **Refresh**: Press `r` to refresh the table list

### Query Editor

- **Cursor Tracking**: Shows cursor position in the title bar
- **Visual Cursor**: Block cursor (█) shows current position in query text
- **Execute**: Press `Ctrl+Enter` to execute the query
- **Clear**: Press `Ctrl+C` to clear the query
- **Navigation**: Use arrow keys, Home, End for cursor movement
- **Multi-line**: Press `Enter` for new lines, `Tab` for indentation

### Query Results

- **Scrolling**: Use arrow keys to scroll through results
- **Pagination**: Use `Page Up`/`Page Down` for faster scrolling
- **Column Navigation**: Use ←/→ to scroll horizontally through columns
- **Home**: Press `Home` to go to top-left of results

### Global Shortcuts

- **Help**: Press `h` or `F1` to toggle help popup
- **Quit**: Press `q` in connection list or `Ctrl+Q` anywhere
- **Back/Cancel**: Press `Esc` to go back or cancel current action
- **Error Dismissal**: Press any key to dismiss error messages

## Database Support

### SQLite
- Full support for local SQLite databases
- In-memory databases supported
- Schema browsing and table information
- All standard SQL operations

### PostgreSQL
- Connection pooling with configurable connections
- Schema-aware table browsing
- Full PostgreSQL SQL support
- Proper handling of schemas and namespaces

### MySQL
- Connection pooling support
- Table and column metadata retrieval
- Standard MySQL SQL operations
- Proper handling of MySQL-specific features

## Architecture

The application is built with a modular architecture:

- **`main.rs`**: Application entry point and terminal setup
- **`app.rs`**: Application state management and business logic
- **`database.rs`**: Database abstraction layer with SQLx integration
- **`ui.rs`**: User interface rendering with Ratatui widgets
- **`event.rs`**: Keyboard event handling and navigation
- **`demo.rs`**: Demo database creation and sample queries

### Key Technologies

- **[Ratatui](https://github.com/ratatui-org/ratatui)**: Terminal UI framework
- **[SQLx](https://github.com/launchbadge/sqlx)**: Async SQL toolkit
- **[Tokio](https://tokio.rs/)**: Async runtime
- **[Crossterm](https://github.com/crossterm-rs/crossterm)**: Cross-platform terminal manipulation
- **[Anyhow](https://github.com/dtolnay/anyhow)**: Error handling

## Development

### Running in Development

```bash
# Run with auto-reload
cargo watch -x run

# Run tests
cargo test

# Check for issues
cargo clippy

# Format code
cargo fmt
```

### Project Structure

```
src/
├── main.rs          # Entry point and terminal setup
├── app.rs           # Application state and logic
├── database.rs      # Database connection and queries
├── ui.rs            # User interface components
├── event.rs         # Event handling and navigation
└── demo.rs          # Demo database creation
```

## Troubleshooting

### Connection Issues

1. **SQLite**: Ensure the database file exists and is readable
2. **PostgreSQL/MySQL**: Verify connection string format and credentials
3. **Network**: Check if the database server is accessible

### Performance

- Large result sets are automatically limited for display
- Connection pooling optimizes database performance
- Async operations prevent UI blocking

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
