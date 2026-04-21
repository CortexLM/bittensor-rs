//! Example: TUI Dashboard — renders a single frame and exits.
//!
//! Run with: cargo run -p bittensor-examples --example tui_dashboard

use bittensor_tui::prelude::{App, draw};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::stdout;

fn main() -> std::io::Result<()> {
    // Set up the terminal
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // Create the app
    let app = App::new();

    // Render one frame
    terminal.draw(|frame| draw(frame, &app))?;

    // Restore the terminal
    drop(terminal);

    println!("TUI dashboard example complete");
    Ok(())
}
