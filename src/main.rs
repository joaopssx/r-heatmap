// r-heatmap - System monitor by joaopssx
mod cli;
mod config;
mod logging;
mod system;
mod ui;

use crate::config::Config;
use crate::system::SystemStats;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    let args = cli::parse_args();

    logging::setup_logger(&args.log_level)?;
    color_eyre::install()?;

    let config = config::load_config(args.config.as_deref());

    ui::util::console::use_utf8();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut stats = SystemStats::new(!args.no_gpu);

    let res = run(&mut terminal, &mut stats, &config);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    res
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    stats: &mut SystemStats,
    config: &Config,
) -> Result<()> {
    let tick_rate = Duration::from_millis(config.general.refresh_rate_ms.max(10));
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, stats, config))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if let Some(ui::events::AppEvent::Input(key)) = ui::events::handle_events(timeout) {
            let ctrl_c =
                key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c');

            if key.code == KeyCode::Char('q') || ctrl_c {
                return Ok(());
            }
        }

        if last_tick.elapsed() >= tick_rate {
            stats.refresh();
            last_tick = Instant::now();
        }
    }
}
