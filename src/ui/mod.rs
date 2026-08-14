// TUI rendering and event handling - by joaopssx
pub mod events;
pub mod layout;
pub mod widgets;

use self::layout::grid;
use self::widgets::{footer, header, tile};
use crate::config::Config;
use crate::system::SystemStats;
use crate::system::sysfs::Reading;
use crate::util::color::{get_fan_color, get_usage_color, get_volt_color, parse_color};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};
use sysinfo::Component;

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

    let disks = collect_sensors(stats, &config.style.disk_label_contains);
    let disks_height = if disks.is_empty() { 0 } else { 5 };
    let fans_height = if stats.fans.is_empty() { 0 } else { 5 };
    let volts_height = if stats.volts.is_empty() { 0 } else { 5 };
    let gpus_height = if stats.gpus.is_empty() { 0 } else { 5 };

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(6),
            Constraint::Length(disks_height),
            Constraint::Length(fans_height),
            Constraint::Length(volts_height),
            Constraint::Length(gpus_height),
            Constraint::Min(0),
        ])
        .split(main_area);

    let sensors = collect_sensors(stats, &config.style.sensor_label_contains);

    header::render(f, content_chunks[0], stats, config);
    render_sensor_grid(f, content_chunks[1], &sensors, config);
    render_sensor_grid(f, content_chunks[2], &disks, config);
    render_reading_grid(f, content_chunks[3], &stats.fans, 0, " RPM", |r| {
        get_fan_color(r.value, r.max)
    });
    render_reading_grid(f, content_chunks[4], &stats.volts, 3, " V", |r| {
        get_volt_color(r.value, r.min, r.max)
    });
    render_reading_grid(f, content_chunks[5], &stats.gpus, 0, "%", |r| {
        get_usage_color(r.value)
    });
    render_core_usage_grid(f, content_chunks[6], stats, config);
    footer::render(f, chunks[1], stats, config);
}

fn collect_sensors<'a>(stats: &'a SystemStats, filters: &[String]) -> Vec<&'a Component> {
    let mut sensors: Vec<_> = stats
        .components
        .iter()
        .filter(|c| {
            let label = c.label().to_lowercase();
            filters.iter().any(|f| label.contains(f))
        })
        .collect();

    sensors.sort_by_key(|s| s.label());
    sensors
}

fn render_sensor_grid(f: &mut Frame, area: Rect, sensors: &[&Component], config: &Config) {
    if sensors.is_empty() {
        return;
    }

    let grid_rects = grid::calculate_grid(area, sensors.len());

    let mut idx = 0;
    for row in grid_rects {
        for col_rect in row {
            if let Some(s) = sensors.get(idx) {
                let temp = s.temperature().unwrap_or(0.0);
                tile::render_tile(
                    f,
                    col_rect,
                    s.label(),
                    &format!("{:.1}°C", temp),
                    get_temp_level_color(temp, config),
                );
                idx += 1;
            }
        }
    }
}

fn render_reading_grid<F>(
    f: &mut Frame,
    area: Rect,
    readings: &[Reading],
    decimals: usize,
    unit: &str,
    color: F,
) where
    F: Fn(&Reading) -> Color,
{
    if readings.is_empty() {
        return;
    }

    let grid_rects = grid::calculate_grid(area, readings.len());

    let mut idx = 0;
    for row in grid_rects {
        for col_rect in row {
            if let Some(reading) = readings.get(idx) {
                tile::render_tile(
                    f,
                    col_rect,
                    &reading.label,
                    &format!("{:.*}{}", decimals, reading.value, unit),
                    color(reading),
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
                tile::render_tile(
                    f,
                    col_rect,
                    &label,
                    &format!("{:.1}%", usage),
                    get_usage_color(usage),
                );
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
