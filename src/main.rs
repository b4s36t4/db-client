mod app;
mod database;
mod demo;
mod event;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::{
    io,
    time::{Duration, Instant},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Check if we should create demo database
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--create-demo" {
        println!("Creating demo database...");
        demo::create_demo_database().await?;
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();

    // Add demo database to connections if it exists
    if std::path::Path::new("demo.db").exists() {
        let _ = app.add_connection(
            "Demo SQLite Database".to_string(),
            "sqlite:demo.db".to_string(),
        );
    }

    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                event::handle_key_event(&mut app, key).await?;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            app.update_spinner(); // Update spinner animation

            // Check if connection task has completed
            app.check_connection_task().await;
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
