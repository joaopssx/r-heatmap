// TUI rendering and event handling - by joaopssx
pub mod events;
pub mod layout;
pub mod util;
pub mod widgets;

use self::layout::grid;
use self::widgets::{footer, header, tile};
use crate::config::Config;
use crate::system::Reading;
use crate::system::SystemStats;
use crate::ui::util::color::{
    get_clock_color, get_fan_color, get_usage_color, get_volt_color, parse_color,
};
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

    let sensors = collect_sensors(stats, &config.style.sensor_label_contains);
    let disks = collect_disks(stats, &config.style.disk_label_contains);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(row_height(stats.memory.len(), 5)),
            Constraint::Length(row_height(sensors.len(), 6)),
            Constraint::Length(row_height(disks.len(), 5)),
            Constraint::Length(row_height(stats.fans.len(), 5)),
            Constraint::Length(row_height(stats.volts.len(), 5)),
            Constraint::Length(row_height(stats.gpus.len(), 5)),
            Constraint::Length(row_height(stats.gpu_temps.len(), 5)),
            Constraint::Length(row_height(stats.gpu_clocks.len(), 5)),
            Constraint::Min(0),
        ])
        .split(main_area);

    header::render(f, content_chunks[0], stats, config);
    render_reading_grid(f, content_chunks[1], &stats.memory, 1, "%", |r| {
        get_usage_color(r.value)
    });
    render_sensor_grid(f, content_chunks[2], &sensors, config);
    render_reading_grid(f, content_chunks[3], &disks, 1, "°C", |r| {
        get_temp_level_color(r.value, config)
    });
    render_reading_grid(f, content_chunks[4], &stats.fans, 0, " RPM", |r| {
        get_fan_color(r.value, r.max)
    });
    render_reading_grid(f, content_chunks[5], &stats.volts, 3, " V", |r| {
        get_volt_color(r.value, r.min, r.max)
    });
    render_reading_grid(f, content_chunks[6], &stats.gpus, 0, "%", |r| {
        get_usage_color(r.value)
    });
    render_reading_grid(f, content_chunks[7], &stats.gpu_temps, 1, "°C", |r| {
        get_temp_level_color(r.value, config)
    });
    render_reading_grid(f, content_chunks[8], &stats.gpu_clocks, 0, " MHz", |r| {
        get_clock_color(r.value, r.max)
    });
    render_reading_grid(f, content_chunks[9], &stats.cores, 1, "%", |r| {
        get_usage_color(r.value)
    });
    footer::render(f, chunks[1], stats, config);
}

fn row_height(count: usize, height: u16) -> u16 {
    if count == 0 { 0 } else { height }
}

fn collect_disks(stats: &SystemStats, filters: &[String]) -> Vec<Reading> {
    let mut disks: Vec<Reading> = collect_sensors(stats, filters)
        .iter()
        .map(|sensor| {
            Reading::new(
                sensor.label().to_string(),
                sensor.temperature().unwrap_or(0.0),
            )
        })
        .collect();

    disks.extend(stats.disks.iter().cloned());
    disks
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
