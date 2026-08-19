// TUI rendering and event handling - by joaopssx
pub mod events;
pub mod layout;
pub mod util;
pub mod widgets;

const TILE_HEIGHT: u16 = 4;

use self::layout::grid;
use self::widgets::{footer, header, tile};
use crate::config::{Config, StyleConfig};
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

    let temps = split_temps(&stats.temps, &stats.disks, &config.style);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(row_height(stats.memory.len())),
            Constraint::Length(row_height(temps.cpus.len())),
            Constraint::Length(row_height(temps.board.len())),
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
    render_reading_grid(f, content_chunks[3], &temps.board, 1, "°C", temp_color);
    render_reading_grid(f, content_chunks[4], &temps.disks, 1, "°C", temp_color);
    render_reading_grid(f, content_chunks[5], &temps.others, 1, "°C", temp_color);
    render_reading_grid(f, content_chunks[6], &stats.fans, 0, " RPM", |r| {
        get_fan_color(r.value, r.max)
    });
    render_reading_grid(f, content_chunks[7], &stats.volts, 3, " V", |r| {
        get_volt_color(r.value, r.min, r.max)
    });
    render_reading_grid(f, content_chunks[8], &stats.gpus, 0, "%", |r| {
        get_usage_color(r.value)
    });
    render_reading_grid(f, content_chunks[9], &stats.gpu_temps, 1, "°C", temp_color);
    render_reading_grid(f, content_chunks[10], &stats.gpu_clocks, 0, " MHz", |r| {
        get_clock_color(r.value, r.max)
    });
    render_reading_grid(f, content_chunks[11], &stats.cores, 1, "%", |r| {
        get_usage_color(r.value)
    });
    footer::render(f, chunks[1], stats, config);
}

struct Temps {
    cpus: Vec<Reading>,
    board: Vec<Reading>,
    disks: Vec<Reading>,
    others: Vec<Reading>,
}

fn split_temps(temps: &[Reading], disks: &[Reading], style: &StyleConfig) -> Temps {
    let mut split = Temps {
        cpus: Vec::new(),
        board: Vec::new(),
        disks: Vec::new(),
        others: Vec::new(),
    };

    for reading in temps {
        if matches(reading, &style.disk_label_contains) {
            split.disks.push(reading.clone());
        } else if matches(reading, &style.board_label_contains) {
            split.board.push(reading.clone());
        } else if matches(reading, &style.sensor_label_contains) {
            split.cpus.push(reading.clone());
        } else {
            split.others.push(reading.clone());
        }
    }

    if split.cpus.is_empty() {
        split.cpus.append(&mut split.others);
    }

    if !style.show_other_sensors {
        split.others.clear();
    }

    split.disks.extend(disks.iter().cloned());

    for row in [
        &mut split.cpus,
        &mut split.board,
        &mut split.disks,
        &mut split.others,
    ] {
        row.sort_by(|a, b| a.label.cmp(&b.label));
    }

    split
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
                let value = format!("{:.*}{}", decimals, reading.value, unit);
                let value = match &reading.note {
                    Some(note) => format!("{value}   {note}"),
                    None => value,
                };

                tile::render_tile(f, col_rect, &reading.label, &value, color(reading));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn temps() -> Vec<Reading> {
        [
            ("coretemp Package id 0", 61.0),
            ("nct6798 SYSTIN", 38.0),
            ("nct6798 PCH_CHIP_TEMP", 52.0),
            ("nvme Composite WD_BLACK SN850X", 44.0),
            ("iwlwifi_1 temp1", 40.0),
        ]
        .iter()
        .map(|(label, value)| Reading::new(label.to_string(), *value))
        .collect()
    }

    #[test]
    fn keeps_the_chipset_out_of_the_cpu_row() {
        let split = split_temps(&temps(), &[], &Config::default().style);

        assert_eq!(split.cpus.len(), 1);
        assert_eq!(split.cpus[0].label, "coretemp Package id 0");
        assert_eq!(split.board.len(), 2);
        assert_eq!(split.board[0].label, "nct6798 PCH_CHIP_TEMP");
        assert_eq!(split.disks.len(), 1);
    }

    #[test]
    fn hides_the_leftovers_unless_they_are_asked_for() {
        let mut config = Config::default();
        let split = split_temps(&temps(), &[], &config.style);
        assert!(split.others.is_empty());

        config.style.show_other_sensors = true;
        let split = split_temps(&temps(), &[], &config.style);

        assert_eq!(split.others.len(), 1);
        assert_eq!(split.others[0].label, "iwlwifi_1 temp1");
    }

    #[test]
    fn falls_back_to_everything_when_no_sensor_matches() {
        let mut config = Config::default();
        config.style.sensor_label_contains = vec!["tctl".to_string()];

        let split = split_temps(&temps(), &[], &config.style);

        assert_eq!(split.cpus.len(), 2);
        assert_eq!(split.board.len(), 2);
        assert_eq!(split.disks.len(), 1);
    }
}
