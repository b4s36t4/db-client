use crate::app::{App, AppScreen, ConnectionField};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn handle_key_event(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // Clear messages on any key press when error is showing
    if app.error_message.is_some() {
        app.clear_messages();
        return Ok(());
    }

    // Global key handlers (only when not in input fields)
    if !is_input_field_active(app) {
        match key_event.code {
            KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('h') | KeyCode::F(1) => {
                app.show_help = !app.show_help;
                return Ok(());
            }
            KeyCode::Esc => {
                if app.is_connecting {
                    app.cancel_connection();
                    return Ok(());
                }
            }
            _ => {}
        }
    }

    // Screen-specific key handlers
    match app.current_screen {
        AppScreen::ConnectionList => handle_connection_list_keys(app, key_event).await,
        AppScreen::NewConnection => handle_new_connection_keys(app, key_event),
        AppScreen::EditConnection => handle_edit_connection_keys(app, key_event),
        AppScreen::TableBrowser => handle_table_browser_keys(app, key_event).await,
        AppScreen::QueryEditor => handle_query_editor_keys(app, key_event).await,
        AppScreen::QueryResults => handle_query_results_keys(app, key_event),
    }
}

fn is_input_field_active(app: &App) -> bool {
    matches!(
        app.current_screen,
        AppScreen::NewConnection | AppScreen::EditConnection | AppScreen::QueryEditor
    )
}

async fn handle_connection_list_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('n') => {
            app.current_screen = AppScreen::NewConnection;
            app.connection_form = Default::default();
        }
        KeyCode::Up => {
            app.previous_connection();
        }
        KeyCode::Down => {
            app.next_connection();
        }
        KeyCode::Enter => {
            if !app.connections.is_empty() && !app.is_connecting {
                if let Err(e) = app.start_connection(app.selected_connection_index) {
                    app.error_message = Some(format!("Failed to start connection: {}", e));
                }
            }
        }
        KeyCode::Char('e') => {
            if !app.connections.is_empty() && !app.is_connecting {
                if let Err(e) = app.start_editing_connection(app.selected_connection_index) {
                    app.error_message = Some(format!("Failed to start editing connection: {}", e));
                }
            }
        }
        KeyCode::Char('d') => {
            if !app.connections.is_empty() {
                let index_to_remove = app.selected_connection_index;
                let _ = app.remove_connection(index_to_remove).await;
                // Adjust selected index if necessary
                if app.selected_connection_index >= app.connections.len()
                    && !app.connections.is_empty()
                {
                    app.selected_connection_index = app.connections.len() - 1;
                }
                // Save connections to disk
                if let Err(e) = app.save_connections() {
                    app.error_message = Some(format!("Failed to save connections: {}", e));
                }
            }
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
    Ok(())
}

fn handle_new_connection_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        KeyCode::Tab => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                app.connection_form.previous_field();
            } else {
                app.connection_form.next_field();
            }
        }
        KeyCode::Enter => {
            if !app.connection_form.name.is_empty() {
                match app.save_edited_connection() {
                    Ok(()) => {
                        app.status_message = Some("Connection updated successfully".to_string());
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Failed to update connection: {}", e));
                    }
                }
            }
        }
        KeyCode::Esc => {
            app.current_screen = AppScreen::ConnectionList;
        }
        KeyCode::Char(c) => {
            // Handle toggle fields
            if app.connection_form.is_toggle_field() {
                match app.connection_form.current_field {
                    ConnectionField::UseSsl => {
                        if c == 'y' || c == 'Y' || c == ' ' || c == '\n' {
                            app.connection_form.toggle_ssl();
                        }
                    }
                    ConnectionField::SslMode => {
                        if c == ' ' || c == '\n' {
                            app.connection_form.cycle_ssl_mode();
                        }
                    }
                    ConnectionField::DatabaseType => {
                        if c == ' ' || c == '\n' {
                            app.connection_form.cycle_database_type();
                        }
                    }
                    _ => {}
                }
                return Ok(());
            }

            // Handle file selection shortcuts
            #[cfg(not(target_arch = "wasm32"))]
            match app.connection_form.current_field {
                ConnectionField::SslCertFile => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) && c == 'o' {
                        if let Some(path) = App::select_ssl_certificate_file() {
                            app.connection_form.ssl_cert_file = path;
                        }
                        return Ok(());
                    }
                }
                ConnectionField::SslKeyFile => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) && c == 'o' {
                        if let Some(path) = App::select_ssl_key_file() {
                            app.connection_form.ssl_key_file = path;
                        }
                        return Ok(());
                    }
                }
                ConnectionField::SslCaFile => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) && c == 'o' {
                        if let Some(path) = App::select_ssl_ca_file() {
                            app.connection_form.ssl_ca_file = path;
                        }
                        return Ok(());
                    }
                }
                _ => {}
            }

            // Handle regular character input
            if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                let mut current_value = app.connection_form.get_current_field_value().to_string();
                current_value.push(c);
                app.connection_form.set_current_field_value(current_value);
            }
        }
        KeyCode::Backspace => {
            if !app.connection_form.is_toggle_field() {
                let mut current_value = app.connection_form.get_current_field_value().to_string();
                current_value.pop();
                app.connection_form.set_current_field_value(current_value);
            }
        }
        KeyCode::Left => {
            // Could add cursor position tracking for connection fields in the future
        }
        KeyCode::Right => {
            // Could add cursor position tracking for connection fields in the future
        }
        KeyCode::Home => {
            // Could add cursor position tracking for connection fields in the future
        }
        KeyCode::End => {
            // Could add cursor position tracking for connection fields in the future
        }
        _ => {}
    }
    Ok(())
}

fn handle_edit_connection_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    // For editing, we use the same logic as new connection but with different save behavior
    match key_event.code {
        KeyCode::Tab => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                app.connection_form.previous_field();
            } else {
                app.connection_form.next_field();
            }
        }
        KeyCode::Enter => {
            if !app.connection_form.name.is_empty() {
                match app.save_edited_connection() {
                    Ok(()) => {
                        app.status_message = Some("Connection updated successfully".to_string());
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Failed to update connection: {}", e));
                    }
                }
            }
        }
        KeyCode::Esc => {
            app.current_screen = AppScreen::ConnectionList;
            app.editing_connection_index = None; // Reset editing state
        }
        KeyCode::Char(c) => {
            // Handle toggle fields
            if app.connection_form.is_toggle_field() {
                match app.connection_form.current_field {
                    ConnectionField::UseSsl => {
                        if c == 'y' || c == 'Y' || c == ' ' || c == '\n' {
                            app.connection_form.toggle_ssl();
                        }
                    }
                    ConnectionField::SslMode => {
                        if c == ' ' || c == '\n' {
                            app.connection_form.cycle_ssl_mode();
                        }
                    }
                    ConnectionField::DatabaseType => {
                        if c == ' ' || c == '\n' {
                            app.connection_form.cycle_database_type();
                        }
                    }
                    _ => {}
                }
                return Ok(());
            }

            // Handle file selection shortcuts
            #[cfg(not(target_arch = "wasm32"))]
            match app.connection_form.current_field {
                ConnectionField::SslCertFile => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) && c == 'o' {
                        if let Some(path) = crate::app::App::select_ssl_certificate_file() {
                            app.connection_form.ssl_cert_file = path;
                        }
                        return Ok(());
                    }
                }
                ConnectionField::SslKeyFile => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) && c == 'o' {
                        if let Some(path) = crate::app::App::select_ssl_key_file() {
                            app.connection_form.ssl_key_file = path;
                        }
                        return Ok(());
                    }
                }
                ConnectionField::SslCaFile => {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) && c == 'o' {
                        if let Some(path) = crate::app::App::select_ssl_ca_file() {
                            app.connection_form.ssl_ca_file = path;
                        }
                        return Ok(());
                    }
                }
                _ => {}
            }

            // Handle regular character input
            if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                let mut current_value = app.connection_form.get_current_field_value().to_string();
                current_value.push(c);
                app.connection_form.set_current_field_value(current_value);
            }
        }
        KeyCode::Backspace => {
            if !app.connection_form.is_toggle_field() {
                let mut current_value = app.connection_form.get_current_field_value().to_string();
                current_value.pop();
                app.connection_form.set_current_field_value(current_value);
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_table_browser_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.current_screen = AppScreen::ConnectionList;
        }
        KeyCode::Up => {
            app.previous_table();
            if let Err(e) = app.refresh_table_columns().await {
                app.error_message = Some(format!("Failed to load columns: {}", e));
            }
        }
        KeyCode::Down => {
            app.next_table();
            if let Err(e) = app.refresh_table_columns().await {
                app.error_message = Some(format!("Failed to load columns: {}", e));
            }
        }
        KeyCode::Char('s') => {
            let query = app.generate_select_query();
            app.query_input = query;
            app.query_cursor_position = app.query_input.len();
            app.current_screen = AppScreen::QueryEditor;
        }
        KeyCode::Char('q') => {
            app.current_screen = AppScreen::QueryEditor;
        }
        KeyCode::Char('r') => {
            if let Err(e) = app.refresh_tables().await {
                app.error_message = Some(format!("Failed to refresh tables: {}", e));
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_query_editor_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.current_screen = AppScreen::TableBrowser;
        }
        KeyCode::Enter if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            if !app.query_input.trim().is_empty() {
                app.status_message = Some("Executing query...".to_string());
                match app.execute_query(&app.query_input.clone()).await {
                    Ok(_) => {
                        app.status_message = Some("Query executed successfully!".to_string());
                        // Force a small delay to show the success message
                        tokio::time::timeout(
                            tokio::time::Duration::from_millis(500),
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)),
                        )
                        .await
                        .ok();
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Query execution failed: {}", e));
                        app.status_message = None;
                    }
                }
            } else {
                app.error_message = Some("Cannot execute empty query".to_string());
            }
        }
        KeyCode::Char('e') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            // Alternative: Ctrl+E to execute query
            if !app.query_input.trim().is_empty() {
                app.status_message = Some("Executing query...".to_string());
                match app.execute_query(&app.query_input.clone()).await {
                    Ok(_) => {
                        app.status_message = Some("Query executed successfully!".to_string());
                        // Force a small delay to show the success message
                        tokio::time::timeout(
                            tokio::time::Duration::from_millis(500),
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)),
                        )
                        .await
                        .ok();
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Query execution failed: {}", e));
                        app.status_message = None;
                    }
                }
            } else {
                app.error_message = Some("Cannot execute empty query".to_string());
            }
        }

        // SQL Generation Shortcuts (must come before general character handler)
        KeyCode::Char('s') => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+S: Generate SELECT * for current table
                if let Some(table) = app.get_selected_table() {
                    let query = app.generate_select_star_statement(&table.name, Some(100));
                    app.query_input = query;
                    app.query_cursor_position = app.query_input.len();
                }
            } else {
                app.insert_char_in_query('s');
            }
        }
        KeyCode::Char('i') => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+I: Generate INSERT statement
                if let Some(table) = app.get_selected_table() {
                    if !app.table_columns.is_empty() {
                        let sample_values = vec!["'value1'".to_string(), "'value2'".to_string()];
                        let column_names = app
                            .table_columns
                            .iter()
                            .map(|c| c.name.clone())
                            .collect::<Vec<_>>();
                        let query = app.generate_insert_statement(
                            &table.name,
                            &column_names,
                            &sample_values,
                        );
                        app.query_input = query;
                        app.query_cursor_position = app.query_input.len();
                    }
                }
            } else {
                app.insert_char_in_query('i');
            }
        }
        KeyCode::Char('d') => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+D: Generate DELETE statement
                if let Some(table) = app.get_selected_table() {
                    let query = app.generate_delete_statement(&table.name, None);
                    app.query_input = query;
                    app.query_cursor_position = app.query_input.len();
                }
            } else {
                app.insert_char_in_query('d');
            }
        }
        KeyCode::Char('u') => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+U: Generate UPDATE statement
                if let Some(table) = app.get_selected_table() {
                    let query =
                        app.generate_update_statement(&table.name, "column1 = 'new_value'", None);
                    app.query_input = query;
                    app.query_cursor_position = app.query_input.len();
                }
            } else {
                app.insert_char_in_query('u');
            }
        }
        KeyCode::Char('c') => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    // Ctrl+Shift+C: Clear query (original Ctrl+C functionality)
                    app.clear_query();
                } else {
                    // Ctrl+C: Generate CREATE TABLE statement
                    if let Some(table) = app.get_selected_table() {
                        let query = app.generate_create_table_statement(
                            &format!("{}_copy", table.name),
                            &app.table_columns,
                        );
                        app.query_input = query;
                        app.query_cursor_position = app.query_input.len();
                    }
                }
            } else {
                app.insert_char_in_query('c');
            }
        }
        KeyCode::Char('t') => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+T: Generate TRUNCATE statement
                if let Some(table) = app.get_selected_table() {
                    let query = app.generate_truncate_statement(&table.name);
                    app.query_input = query;
                    app.query_cursor_position = app.query_input.len();
                }
            } else {
                // Regular 't': Test query
                app.query_input = "SELECT 1 as test;".to_string();
                app.query_cursor_position = app.query_input.len();
                app.status_message =
                    Some("Test query loaded. Press Enter or Ctrl+Enter to execute".to_string());
            }
        }
        KeyCode::Char(c) => {
            // Only allow printable characters and common SQL characters
            if c.is_ascii_graphic()
                || c.is_ascii_whitespace()
                || c == ';'
                || c == ','
                || c == '('
                || c == ')'
            {
                app.insert_char_in_query(c);
            }
        }
        KeyCode::Backspace => {
            app.delete_char_in_query();
        }
        KeyCode::Left => {
            app.move_cursor_left();
        }
        KeyCode::Right => {
            app.move_cursor_right();
        }
        KeyCode::Home => {
            app.move_cursor_to_start();
        }
        KeyCode::End => {
            app.move_cursor_to_end();
        }
        KeyCode::Enter => {
            // Check if this is a single line query (no newlines)
            if !app.query_input.contains('\n') && !app.query_input.trim().is_empty() {
                // Execute single-line query on Enter
                app.status_message = Some("Executing query...".to_string());
                match app.execute_query(&app.query_input.clone()).await {
                    Ok(_) => {
                        app.status_message = Some("Query executed successfully!".to_string());
                        // Force a small delay to show the success message
                        tokio::time::timeout(
                            tokio::time::Duration::from_millis(500),
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)),
                        )
                        .await
                        .ok();
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Query execution failed: {}", e));
                        app.status_message = None;
                    }
                }
            } else {
                // Insert newline for multi-line queries
                app.insert_char_in_query('\n');
            }
        }
        KeyCode::Tab => {
            app.insert_char_in_query('\t');
        }
        KeyCode::Delete => {
            // Delete character at cursor position
            if app.query_cursor_position < app.query_input.len() {
                app.query_input.remove(app.query_cursor_position);
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_query_results_keys(app: &mut App, key_event: KeyEvent) -> Result<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.current_screen = AppScreen::QueryEditor;
        }
        KeyCode::Up => {
            // First try to navigate rows, then scroll if at top
            if app.selected_row_index > 0 {
                app.previous_row();
            } else if app.result_scroll_y > 0 {
                app.result_scroll_y -= 1;
            }
        }
        KeyCode::Down => {
            // First try to navigate rows, then scroll if at bottom
            let current_results = app.get_current_page_results();
            if app.selected_row_index < current_results.len().saturating_sub(1) {
                app.next_row();
            } else if app.result_scroll_y < current_results.len().saturating_sub(1) {
                app.result_scroll_y += 1;
            }
        }
        KeyCode::Left => {
            app.previous_column();
        }
        KeyCode::Right => {
            app.next_column();
        }
        KeyCode::PageUp => {
            app.previous_page();
        }
        KeyCode::PageDown => {
            app.next_page();
        }
        KeyCode::Home => {
            app.result_scroll_x = 0;
            app.result_scroll_y = 0;
            app.selected_column_index = 0;
            app.selected_row_index = 0; // Reset row selection
            app.current_page = 0;
        }
        KeyCode::End => {
            if let Some(result) = &app.current_query_result {
                app.selected_column_index = result.columns.len().saturating_sub(1);
                app.current_page = app.get_total_pages().saturating_sub(1);
                let current_results = app.get_current_page_results();
                app.selected_row_index = current_results.len().saturating_sub(1);
                app.result_scroll_y = current_results.len().saturating_sub(1);
            }
        }
        KeyCode::Char('h') => {
            app.selected_column_index = 0;
        }
        KeyCode::Char('l') => {
            if let Some(result) = &app.current_query_result {
                app.selected_column_index = result.columns.len().saturating_sub(1);
            }
        }
        _ => {}
    }
    Ok(())
}
