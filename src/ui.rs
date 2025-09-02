use crate::app::{App, AppScreen, ConnectionField};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, Wrap,
    },
};

/// Helper function to create a centered rect using up certain percentage of the available area
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.area());

    // Main content area
    match app.current_screen {
        AppScreen::ConnectionList => draw_connection_list(f, app, chunks[0]),
        AppScreen::NewConnection => draw_new_connection(f, app, chunks[0]),
        AppScreen::EditConnection => draw_edit_connection(f, app, chunks[0]),
        AppScreen::TableBrowser => draw_table_browser(f, app, chunks[0]),
        AppScreen::QueryEditor => draw_query_editor(f, app, chunks[0]),
        AppScreen::QueryResults => draw_query_results(f, app, chunks[0]),
    }

    // Status bar
    draw_status_bar(f, app, chunks[1]);

    // Help popup
    if app.show_help {
        draw_help_popup(f, app);
    }

    // Error popup
    if app.error_message.is_some() {
        draw_error_popup(f, app);
    }
}

fn draw_connection_list(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    // Title
    let title = Paragraph::new("Database Connections")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Connection list
    let items: Vec<ListItem> = app
        .connections
        .iter()
        .enumerate()
        .map(|(i, conn)| {
            let mut style = Style::default();
            let mut prefix = "  ";

            if Some(i) == app.current_connection {
                style = style.fg(Color::Green).add_modifier(Modifier::BOLD);
                prefix = "● ";
            }

            if i == app.selected_connection_index {
                style = style.bg(Color::Blue).add_modifier(Modifier::BOLD);
            }

            let content = format!(
                "{}{} ({})",
                prefix,
                conn.name,
                conn.database_type.display_name()
            );
            ListItem::new(content).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_connection_index));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Connections (↑↓ to navigate, Enter to connect)"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

fn draw_new_connection(f: &mut Frame, app: &mut App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),  // Title
                Constraint::Length(24), // Form fields (8 rows * 3 height each)
                Constraint::Length(4),  // SSL fields
                Constraint::Min(0),     // Help text
            ]
            .as_ref(),
        )
        .split(area);
    // Title
    let title = Paragraph::new("New Database Connection")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, main_chunks[0]);

    // Form fields area
    let form_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50), // Left column
                Constraint::Percentage(50), // Right column
            ]
            .as_ref(),
        )
        .split(main_chunks[1]);

    // Left column fields
    let left_fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Name
                Constraint::Length(3), // Connection String
                Constraint::Length(3), // Database Type
                Constraint::Length(3), // Host
                Constraint::Length(3), // Port
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(3), // Database
            ]
            .as_ref(),
        )
        .split(form_chunks[0]);

    // Helper function to create field display
    let create_field_display = |f: &mut Frame, field: ConnectionField, title: &str, chunk: Rect| {
        let is_current_field = app.connection_form.current_field == field;
        let is_toggle_field = app.connection_form.is_field_toggle(&field);
        let value = app.connection_form.get_field_value(field.clone());

        let (text, style, display_title) = if is_current_field {
            let text_with_cursor = if is_toggle_field {
                format!("{}|", value)
            } else {
                format!("{}|", value)
            };
            (
                text_with_cursor,
                Style::default().fg(Color::Yellow),
                format!("{} (Active)", title),
            )
        } else {
            (value.to_string(), Style::default(), title.to_string())
        };

        let input = Paragraph::new(text)
            .style(style)
            .block(Block::default().borders(Borders::ALL).title(display_title));
        f.render_widget(input, chunk);
    };

    // Left column fields
    create_field_display(f, ConnectionField::Name, "Name", left_fields[0]);
    create_field_display(
        f,
        ConnectionField::ConnectionString,
        "Connection String",
        left_fields[1],
    );
    create_field_display(
        f,
        ConnectionField::DatabaseType,
        "Database Type (Space to cycle)",
        left_fields[2],
    );
    create_field_display(f, ConnectionField::Host, "Host", left_fields[3]);
    create_field_display(f, ConnectionField::Port, "Port", left_fields[4]);
    create_field_display(f, ConnectionField::Username, "Username", left_fields[5]);
    create_field_display(f, ConnectionField::Password, "Password", left_fields[6]);
    create_field_display(f, ConnectionField::Database, "Database", left_fields[7]);

    // Right column fields

    // SSL section
    let ssl_row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(25), // Use SSL
                Constraint::Percentage(25), // SSL Mode
                Constraint::Percentage(25), // SSL Cert File
                Constraint::Percentage(25), // SSL Key File
            ]
            .as_ref(),
        )
        .split(main_chunks[2]);

    // Create a second row for SSL CA File by splitting the area again
    let ssl_row2_area = Rect {
        x: main_chunks[2].x,
        y: main_chunks[2].y + 1, // Second row
        width: main_chunks[2].width,
        height: 1,
    };

    // SSL fields - first row
    create_field_display(f, ConnectionField::UseSsl, "Use SSL", ssl_row1[0]);

    if app.connection_form.use_ssl {
        create_field_display(f, ConnectionField::SslMode, "SSL Mode", ssl_row1[1]);
        create_field_display(
            f,
            ConnectionField::SslCertFile,
            "SSL Cert File (Ctrl+O)",
            ssl_row1[2],
        );
        create_field_display(
            f,
            ConnectionField::SslKeyFile,
            "SSL Key File (Ctrl+O)",
            ssl_row1[3],
        );
    } else {
        // Show placeholder text when SSL is disabled
        let disabled_text = Paragraph::new("SSL Disabled")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("SSL Mode"));
        f.render_widget(disabled_text, ssl_row1[1]);

        let disabled_text = Paragraph::new("SSL Disabled")
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("SSL Cert File"),
            );
        f.render_widget(disabled_text, ssl_row1[2]);

        let disabled_text = Paragraph::new("SSL Disabled")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("SSL Key File"));
        f.render_widget(disabled_text, ssl_row1[3]);
    }

    // SSL CA File on second row
    if app.connection_form.use_ssl {
        create_field_display(
            f,
            ConnectionField::SslCaFile,
            "SSL CA File (Ctrl+O)",
            ssl_row2_area,
        );
    } else {
        let disabled_text = Paragraph::new("SSL Disabled")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("SSL CA File"));
        f.render_widget(disabled_text, ssl_row2_area);
    }

    // Help text
    let help_text = vec![
        Line::from("Fill either Connection String OR individual fields:"),
        Line::from("  SQLite: sqlite:database.db"),
        Line::from("  PostgreSQL: postgresql://user:password@localhost/dbname"),
        Line::from("  MySQL: mysql://user:password@localhost/dbname"),
        Line::from(""),
        Line::from("Individual fields: Select DB type, then fill Host/Port/User/Pass/DB"),
        Line::from("SSL: Configure SSL certificates and modes"),
        Line::from("Tab: Next field, Shift+Tab: Previous field"),
        Line::from("Enter: Save, Esc: Cancel, Ctrl+O: File dialog, Space: Toggle/Cycle"),
    ];
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });
    f.render_widget(help, main_chunks[3]);
}

fn draw_edit_connection(f: &mut Frame, app: &mut App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),  // Title
                Constraint::Length(24), // Form fields (8 rows * 3 height each)
                Constraint::Length(4),  // SSL fields
                Constraint::Min(0),     // Help text
            ]
            .as_ref(),
        )
        .split(area);

    // Title
    let title = Paragraph::new("Edit Database Connection")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, main_chunks[0]);

    // Form fields area
    let form_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50), // Left column
                Constraint::Percentage(50), // Right column
            ]
            .as_ref(),
        )
        .split(main_chunks[1]);

    // Left column fields
    let left_fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Name
                Constraint::Length(3), // Connection String
                Constraint::Length(3), // Database Type
                Constraint::Length(3), // Host
                Constraint::Length(3), // Port
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(3), // Database
            ]
            .as_ref(),
        )
        .split(form_chunks[0]);

    // Right column fields
    let right_constraints = vec![
        Constraint::Length(3), // Use SSL
    ];

    let right_fields = Layout::default()
        .direction(Direction::Vertical)
        .constraints(&right_constraints)
        .split(form_chunks[1]);

    // Helper function to create field display
    let create_field_display = |f: &mut Frame, field: ConnectionField, title: &str, chunk: Rect| {
        let is_current_field = app.connection_form.current_field == field;
        let is_toggle_field = app.connection_form.is_field_toggle(&field);
        let value = app.connection_form.get_field_value(field.clone());

        let (text, style, display_title) = if is_current_field {
            let text_with_cursor = if is_toggle_field {
                format!("{}|", value)
            } else {
                format!("{}|", value)
            };
            (
                text_with_cursor,
                Style::default().fg(Color::Yellow),
                format!("{} (Active)", title),
            )
        } else {
            (value.to_string(), Style::default(), title.to_string())
        };

        let input = Paragraph::new(text)
            .style(style)
            .block(Block::default().borders(Borders::ALL).title(display_title));
        f.render_widget(input, chunk);
    };

    // Left column fields
    create_field_display(f, ConnectionField::Name, "Name", left_fields[0]);
    create_field_display(
        f,
        ConnectionField::ConnectionString,
        "Connection String",
        left_fields[1],
    );
    create_field_display(
        f,
        ConnectionField::DatabaseType,
        "Database Type (Space to cycle)",
        left_fields[2],
    );
    create_field_display(f, ConnectionField::Host, "Host", left_fields[3]);
    create_field_display(f, ConnectionField::Port, "Port", left_fields[4]);
    create_field_display(f, ConnectionField::Username, "Username", left_fields[5]);
    create_field_display(f, ConnectionField::Password, "Password", left_fields[6]);
    create_field_display(f, ConnectionField::Database, "Database", left_fields[7]);

    // Right column fields
    create_field_display(f, ConnectionField::UseSsl, "Use SSL", right_fields[0]);

    // SSL section
    let ssl_row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(25), // Use SSL
                Constraint::Percentage(25), // SSL Mode
                Constraint::Percentage(25), // SSL Cert File
                Constraint::Percentage(25), // SSL Key File
            ]
            .as_ref(),
        )
        .split(main_chunks[2]);

    // Create a second row for SSL CA File by splitting the area again
    let ssl_row2_area = Rect {
        x: main_chunks[2].x,
        y: main_chunks[2].y + 1, // Second row
        width: main_chunks[2].width,
        height: 1,
    };

    // SSL fields - first row
    create_field_display(f, ConnectionField::UseSsl, "Use SSL", ssl_row1[0]);

    if app.connection_form.use_ssl {
        create_field_display(f, ConnectionField::SslMode, "SSL Mode", ssl_row1[1]);
        create_field_display(
            f,
            ConnectionField::SslCertFile,
            "SSL Cert File (Ctrl+O)",
            ssl_row1[2],
        );
        create_field_display(
            f,
            ConnectionField::SslKeyFile,
            "SSL Key File (Ctrl+O)",
            ssl_row1[3],
        );
    } else {
        // Show placeholder text when SSL is disabled
        let disabled_text = Paragraph::new("SSL Disabled")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("SSL Mode"));
        f.render_widget(disabled_text, ssl_row1[1]);

        let disabled_text = Paragraph::new("SSL Disabled")
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("SSL Cert File"),
            );
        f.render_widget(disabled_text, ssl_row1[2]);

        let disabled_text = Paragraph::new("SSL Disabled")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("SSL Key File"));
        f.render_widget(disabled_text, ssl_row1[3]);
    }

    // SSL CA File on second row
    if app.connection_form.use_ssl {
        create_field_display(
            f,
            ConnectionField::SslCaFile,
            "SSL CA File (Ctrl+O)",
            ssl_row2_area,
        );
    } else {
        let disabled_text = Paragraph::new("SSL Disabled")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("SSL CA File"));
        f.render_widget(disabled_text, ssl_row2_area);
    }

    // Help text
    let help_text = vec![
        Line::from("Edit the connection details:"),
        Line::from("  Fill either Connection String OR individual fields"),
        Line::from("  SQLite: sqlite:database.db"),
        Line::from("  PostgreSQL: postgresql://user:password@localhost/dbname"),
        Line::from("  MySQL: mysql://user:password@localhost/dbname"),
        Line::from(""),
        Line::from("Individual fields: Select DB type, then fill Host/Port/User/Pass/DB"),
        Line::from("SSL: Configure SSL certificates and modes"),
        Line::from("Tab: Next field, Shift+Tab: Previous field"),
        Line::from("Enter: Save, Esc: Cancel, Ctrl+O: File dialog, Space: Toggle/Cycle"),
    ];
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });
    f.render_widget(help, main_chunks[3]);
}

fn draw_table_browser(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    // Tables list
    let table_items: Vec<ListItem> = app
        .tables
        .iter()
        .enumerate()
        .map(|(i, table)| {
            let display_name = if let Some(schema) = &table.schema {
                format!("{}.{}", schema, table.name)
            } else {
                table.name.clone()
            };

            let row_count = table
                .row_count
                .map(|count| format!(" ({})", count))
                .unwrap_or_default();

            let mut style = Style::default();
            if i == app.selected_table_index {
                style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
            }

            ListItem::new(format!("{}{}", display_name, row_count)).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_table_index));

    let selected_table_name = app
        .get_selected_table()
        .map(|t| t.name.as_str())
        .unwrap_or("None");
    let tables_list = List::new(table_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Tables (Selected: {})", selected_table_name)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(tables_list, chunks[0], &mut list_state);

    // Table columns
    let column_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(chunks[1]);

    if !app.table_columns.is_empty() {
        let header = Row::new(vec!["Column", "Type", "Nullable", "PK"])
            .style(Style::default().fg(Color::Yellow))
            .height(1);

        let rows: Vec<Row> = app
            .table_columns
            .iter()
            .map(|col| {
                Row::new(vec![
                    col.name.clone(),
                    col.data_type.clone(),
                    if col.is_nullable { "YES" } else { "NO" }.to_string(),
                    if col.is_primary_key { "YES" } else { "NO" }.to_string(),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Columns"));

        f.render_widget(table, column_chunks[0]);
    } else {
        let empty = Paragraph::new("No columns to display")
            .block(Block::default().borders(Borders::ALL).title("Columns"))
            .alignment(Alignment::Center);
        f.render_widget(empty, column_chunks[0]);
    }

    // Quick actions and sample queries
    let selected_table_name = app
        .get_selected_table()
        .map(|t| t.name.as_str())
        .unwrap_or("table");
    let actions_text = vec![
        Line::from("Quick Actions:"),
        Line::from("  s - Generate SELECT query"),
        Line::from("  q - Open query editor"),
        Line::from(""),
        Line::from("Sample Queries:"),
        Line::from(format!("  SELECT * FROM {} LIMIT 10;", selected_table_name)),
        Line::from(format!("  SELECT COUNT(*) FROM {};", selected_table_name)),
        Line::from(""),
        Line::from("💡 Auto-pagination: Queries automatically limited to 50 rows"),
        Line::from("   Use LIMIT in your queries to override this behavior"),
    ];
    let actions = Paragraph::new(actions_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Actions & Examples"),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(actions, column_chunks[1]);
}

fn draw_query_editor(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(area);

    // Query input with cursor
    let query_with_cursor = if app.current_screen == AppScreen::QueryEditor {
        let mut query = app.query_input.clone();
        query.insert(app.query_cursor_position, '█'); // Block cursor
        query
    } else {
        app.query_input.clone()
    };

    let title = format!(
        "SQL Query (Cursor: {}) | Length: {}",
        app.query_cursor_position,
        app.query_input.len()
    );
    let query_input = Paragraph::new(query_with_cursor)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    f.render_widget(query_input, chunks[0]);

    // Instructions
    let instructions_text = vec![
        Line::from("Press Ctrl+Enter or Enter to execute query, Esc to go back"),
        Line::from("Use Ctrl+C to clear query, 't' for test query"),
        Line::from(""),
        Line::from("💡 Tip: You can type freely here - global shortcuts are disabled"),
    ];
    let instructions = Paragraph::new(instructions_text)
        .block(Block::default().borders(Borders::ALL).title("Instructions"))
        .wrap(Wrap { trim: true });
    f.render_widget(instructions, chunks[1]);
}

fn draw_query_results(f: &mut Frame, app: &App, area: Rect) {
    if let Some(result) = &app.current_query_result {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(area);

        if !result.columns.is_empty() && !result.rows.is_empty() {
            // Results table with pagination
            let current_page_results = app.get_current_page_results();
            let _total_pages = app.get_total_pages();

            // Split the area for table and scrollbar
            let table_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
                .split(chunks[0]);

            // Create header with column highlighting
            let header_cells: Vec<String> = result
                .columns
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    if i == app.selected_column_index {
                        format!(">> {}", col)
                    } else {
                        col.clone()
                    }
                })
                .collect();

            let header = Row::new(header_cells)
                .style(Style::default().fg(Color::Yellow))
                .height(1);

            let visible_rows_count = (table_area[0].height as usize).saturating_sub(3); // Account for borders and header
            let rows: Vec<Row> = current_page_results
                .iter()
                .enumerate() // Add enumeration to track row index
                .skip(app.result_scroll_y)
                .take(visible_rows_count)
                .map(|(visible_row_idx, row)| {
                    let cells: Vec<String> = row
                        .iter()
                        .enumerate()
                        .map(|(i, cell)| {
                            let mut cell_text = if cell.len() > 30 {
                                format!("{}...", &cell[..27])
                            } else {
                                cell.clone()
                            };

                            // Highlight selected column
                            if i == app.selected_column_index {
                                cell_text = format!(">> {}", cell_text);
                            }

                            cell_text
                        })
                        .collect();

                    // Create row with highlighting for selected row
                    let mut row_style = Style::default();
                    // The selected_row_index is absolute within the current page results
                    // visible_row_idx is the index within the visible portion after scrolling
                    // So we need to check if selected_row_index maps to this visible row
                    let absolute_row_idx = app.result_scroll_y + visible_row_idx;
                    if absolute_row_idx == app.selected_row_index {
                        row_style = row_style.bg(Color::Blue).fg(Color::White);
                    }

                    Row::new(cells).style(row_style)
                })
                .collect();

            let widths: Vec<Constraint> = (0..result.columns.len())
                .map(|_| Constraint::Percentage((100 / result.columns.len()) as u16))
                .collect();

            let table = Table::new(rows, widths).header(header).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Query Results"),
            );

            f.render_widget(table, table_area[0]);

            // Add scrollbar
            if current_page_results.len() > visible_rows_count {
                let scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"));

                let mut scrollbar_state = ScrollbarState::default()
                    .content_length(current_page_results.len())
                    .position(app.result_scroll_y);

                f.render_stateful_widget(scrollbar, table_area[1], &mut scrollbar_state);
            }
        } else {
            let empty = Paragraph::new("No results to display")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Query Results"),
                )
                .alignment(Alignment::Center);
            f.render_widget(empty, chunks[0]);
        }

        // Results info with pagination and column selection
        let current_page_results = app.get_current_page_results();
        let total_pages = app.get_total_pages();
        let selected_column = if app.selected_column_index < result.columns.len() {
            &result.columns[app.selected_column_index]
        } else {
            "None"
        };

        let info_text = vec![
            Line::from(format!(
                "Page {}/{} | Rows: {} (showing {}) | Execution time: {:?}",
                app.current_page + 1,
                total_pages.max(1),
                result.rows.len(),
                current_page_results.len(),
                result.execution_time
            )),
            Line::from(format!(
                "Selected column: {} ({}/{})",
                selected_column,
                app.selected_column_index + 1,
                result.columns.len()
            )),
            Line::from(
                "Navigation: ←→ columns, ↑↓ rows, PageUp/Down pages, h/l first/last column, Home/End",
            ),
        ];
        let info = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("Info"))
            .wrap(Wrap { trim: true });
        f.render_widget(info, chunks[1]);
    } else {
        let empty = Paragraph::new("No query results")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Query Results"),
            )
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let spinner = app.get_spinner_char();
    let status_text = if let Some(status) = &app.status_message {
        if app.is_connecting {
            format!("{} {}", spinner, status)
        } else {
            status.clone()
        }
    } else if let Some(conn_index) = app.current_connection {
        let conn_name = &app.connections[conn_index].name;
        let table_info = if app.current_screen == AppScreen::TableBrowser {
            if let Some(table) = app.get_selected_table() {
                format!(" | Table: {}", table.name)
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        format!("Connected to: {}{}", conn_name, table_info)
    } else {
        "No connection".to_string()
    };

    let status_line = match app.current_screen {
        AppScreen::ConnectionList => {
            if app.is_connecting {
                format!("{} | Press Esc to cancel connection", status_text)
            } else {
                format!(
                    "{} | Press 'n' for new connection, 'e' to edit, Enter to connect, 'q' to quit",
                    status_text
                )
            }
        }
        AppScreen::NewConnection => format!(
            "{} | Tab to switch fields, Enter to save, Esc to cancel",
            status_text
        ),
        AppScreen::EditConnection => format!(
            "{} | Tab to switch fields, Enter to save, Esc to cancel",
            status_text
        ),
        AppScreen::TableBrowser => format!(
            "{} | ↑↓ to navigate, 's' for SELECT, 'q' for query editor",
            status_text
        ),
        AppScreen::QueryEditor => format!(
            "{} | Enter/Ctrl+Enter to execute, 't' for test, Esc to go back",
            status_text
        ),
        AppScreen::QueryResults => format!(
            "{} | ←→ columns, ↑↓ rows, PageUp/Down pages, h/l columns, Home/End, Esc to go back",
            status_text
        ),
    };

    let status = Paragraph::new(status_line)
        .style(Style::default().fg(Color::White).bg(Color::Blue))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(status, area);
}

fn draw_help_popup(f: &mut Frame, _app: &App) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(""),
        Line::from("Keyboard Shortcuts:"),
        Line::from(""),
        Line::from("Global:"),
        Line::from("  q - Quit application"),
        Line::from("  h/F1 - Toggle this help"),
        Line::from("  Esc - Go back/Cancel"),
        Line::from(""),
        Line::from("Connection List:"),
        Line::from("  n - New connection"),
        Line::from("  Enter - Connect to selected"),
        Line::from("  d - Delete connection"),
        Line::from("  Esc - Cancel connection (when connecting)"),
        Line::from(""),
        Line::from("Table Browser:"),
        Line::from("  ↑↓ - Navigate tables"),
        Line::from("  s - Generate SELECT query"),
        Line::from("  q - Open query editor"),
        Line::from(""),
        Line::from("Query Editor:"),
        Line::from("  Ctrl+Enter - Execute query"),
        Line::from("  Ctrl+C - Clear query"),
        Line::from("  SQL Generation:"),
        Line::from("    Ctrl+S - SELECT * from current table"),
        Line::from("    Ctrl+I - INSERT statement"),
        Line::from("    Ctrl+D - DELETE statement"),
        Line::from("    Ctrl+U - UPDATE statement"),
        Line::from("    Ctrl+C - CREATE TABLE statement"),
        Line::from("    Ctrl+T - TRUNCATE statement"),
        Line::from(""),
        Line::from("Query Results:"),
        Line::from("  Arrow keys - Navigate/scroll results"),
        Line::from("  PageUp/Down - Change pages"),
        Line::from("  Home/End - First/Last page"),
        Line::from(""),
    ];

    let help_popup = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .style(Style::default().fg(Color::White).bg(Color::Black)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(help_popup, area);
}

fn draw_error_popup(f: &mut Frame, app: &App) {
    if let Some(error_msg) = &app.error_message {
        let area = centered_rect(60, 30, f.area());
        f.render_widget(Clear, area);

        let error_text = vec![
            Line::from(""),
            Line::from(error_msg.clone()),
            Line::from(""),
            Line::from("Press any key to continue..."),
        ];

        let error_popup = Paragraph::new(error_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Error")
                    .style(Style::default().fg(Color::Red).bg(Color::Black)),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(error_popup, area);
    }
}
