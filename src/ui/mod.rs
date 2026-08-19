// TUI rendering and event handling - by joaopssx
pub mod events;
pub mod layout;
pub mod util;
pub mod widgets;

const TILE_HEIGHT: u16 = 4;

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

    let temps = split_temps(stats, config);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(row_height(stats.memory.len())),
            Constraint::Length(row_height(temps.cpus.len())),
            Constraint::Length(row_height(temps.disks.len())),
            Constraint::Length(row_height(temps.others.len())),
            Constraint::Length(row_height(stats.fans.len())),
            Constraint::Length(row_height(stats.volts.len())),
            Constraint::Length(row_height(stats.gpus.len())),
            Constraint::Length(row_height(stats.gpu_temps.len())),
            Constraint::Length(row_height(stats.gpu_clocks.len())),
            Constraint::Min(0),
        ])
        .split(main_area);

    let temp_color = |r: &Reading| get_temp_level_color(r.value, config);

    header::render(f, content_chunks[0], stats, config);
    render_reading_grid(f, content_chunks[1], &stats.memory, 1, "%", |r| {
        get_usage_color(r.value)
    });
    render_reading_grid(f, content_chunks[2], &temps.cpus, 1, "°C", temp_color);
    render_reading_grid(f, content_chunks[3], &temps.disks, 1, "°C", temp_color);
    render_reading_grid(f, content_chunks[4], &temps.others, 1, "°C", temp_color);
    render_reading_grid(f, content_chunks[5], &stats.fans, 0, " RPM", |r| {
        get_fan_color(r.value, r.max)
    });
    render_reading_grid(f, content_chunks[6], &stats.volts, 3, " V", |r| {
        get_volt_color(r.value, r.min, r.max)
    });
    render_reading_grid(f, content_chunks[7], &stats.gpus, 0, "%", |r| {
        get_usage_color(r.value)
    });
    render_reading_grid(f, content_chunks[8], &stats.gpu_temps, 1, "°C", temp_color);
    render_reading_grid(f, content_chunks[9], &stats.gpu_clocks, 0, " MHz", |r| {
        get_clock_color(r.value, r.max)
    });
    render_reading_grid(f, content_chunks[10], &stats.cores, 1, "%", |r| {
        get_usage_color(r.value)
    });
    footer::render(f, chunks[1], stats, config);
}

struct Temps {
    cpus: Vec<Reading>,
    disks: Vec<Reading>,
    others: Vec<Reading>,
}

fn split_temps(stats: &SystemStats, config: &Config) -> Temps {
    let mut temps = Temps {
        cpus: Vec::new(),
        disks: Vec::new(),
        others: Vec::new(),
    };

    for reading in &stats.temps {
        if matches(reading, &config.style.disk_label_contains) {
            temps.disks.push(reading.clone());
        } else if matches(reading, &config.style.sensor_label_contains) {
            temps.cpus.push(reading.clone());
        } else {
            temps.others.push(reading.clone());
        }
    }

    if temps.cpus.is_empty() {
        temps.cpus.append(&mut temps.others);
    }

    if !config.style.show_other_sensors {
        temps.others.clear();
    }

    temps.disks.extend(stats.disks.iter().cloned());

    for row in [&mut temps.cpus, &mut temps.disks, &mut temps.others] {
        row.sort_by(|a, b| a.label.cmp(&b.label));
    }

    temps
}

fn matches(reading: &Reading, filters: &[String]) -> bool {
    let label = reading.label.to_lowercase();

    filters.iter().any(|filter| label.contains(filter))
}

fn row_height(count: usize) -> u16 {
    grid::rows(count) as u16 * TILE_HEIGHT
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
