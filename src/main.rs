// r-heatmap - System monitor by joaopssx
mod cli;
mod config;
mod logging;
mod system;
mod ui;
mod util;

use color_eyre::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;

fn main() -> Result<()> {
    let args = cli::parse_args();

    logging::setup_logger(&args.log_level)?;
    color_eyre::install()?;

    let config = config::load_config();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut stats = system::SystemStats::new();

    loop {
        terminal.draw(|f| ui::render(f, &stats, &config))?;

        if let Some(event) = ui::events::handle_events(Duration::from_millis(100)) {
            match event {
                ui::events::AppEvent::Input(code) => {
                    if code == crossterm::event::KeyCode::Char('q') {
                        break;
                    }
                }
                ui::events::AppEvent::Tick => {}
            }
        }

        stats.refresh();
        std::thread::sleep(Duration::from_millis(
            config.general.refresh_rate_ms.max(10),
        ));
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
