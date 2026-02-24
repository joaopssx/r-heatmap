// TUI rendering and event handling - by joaopssx
pub mod events;
pub mod layout;
pub mod widgets;

use self::layout::grid;
use self::widgets::{footer, tile};
use crate::config::Config;
use crate::system::SystemStats;
use crate::util::color::{get_usage_color, parse_color};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};

pub fn render(f: &mut Frame, stats: &SystemStats, config: &Config) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let border_color = parse_color(&config.style.border_color);
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(border_color))
        .title(" R-HEATMAP MONITOR ")
        .title_alignment(Alignment::Center);

    let main_area = main_block.inner(chunks[0]);
    f.render_widget(main_block, chunks[0]);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(main_area);

    render_temperatures(f, content_chunks[0], stats, config);
    render_core_usage_grid(f, content_chunks[1], stats, config);
    footer::render(f, chunks[1], stats, config);
}

fn render_temperatures(f: &mut Frame, area: Rect, stats: &SystemStats, config: &Config) {
    let mut sensors: Vec<_> = stats
        .components
        .iter()
        .filter(|c| {
            config
                .style
                .sensor_label_contains
                .iter()
                .any(|s| c.label().to_lowercase().contains(s))
        })
        .collect();

    if sensors.is_empty() {
        return;
    }
    sensors.sort_by_key(|s| s.label());

    let grid_rects = grid::calculate_grid(area, sensors.len());

    let mut idx = 0;
    for row in grid_rects {
        for col_rect in row {
            if let Some(s) = sensors.get(idx) {
                tile::render_tile(
                    f,
                    col_rect,
                    s.label(),
                    s.temperature().unwrap_or(0.0),
                    "°C",
                    |v| get_temp_level_color(v, config),
                );
                idx += 1;
            }
        }
    }
}

fn render_core_usage_grid(f: &mut Frame, area: Rect, stats: &SystemStats, _config: &Config) {
    let usages = stats.cpu_cores_usage();
    let count = usages.len();
    if count == 0 {
        return;
    }

    let grid_rects = grid::calculate_grid(area, count);

    let mut idx = 0;
    for row in grid_rects {
        for col_rect in row {
            if idx < count {
                let usage = usages[idx];
                let label = format!("Core {}", idx);
                tile::render_tile(f, col_rect, &label, usage, "%", |v| get_usage_color(v));
                idx += 1;
            }
        }
    }
}

fn get_temp_level_color(temp: f32, config: &Config) -> Color {
    if temp < config.thresholds.cold {
        Color::Cyan
    } else if temp < config.thresholds.warm {
        Color::Green
    } else if temp < config.thresholds.hot {
        Color::Yellow
    } else if temp < config.thresholds.critical {
        Color::Rgb(255, 140, 0)
    } else {
        Color::Red
    }
}
