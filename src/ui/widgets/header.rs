use crate::config::Config;
use crate::system::SystemStats;
use crate::util::color::parse_color;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::Paragraph,
};

pub fn render(f: &mut Frame, area: Rect, stats: &SystemStats, config: &Config) {
    let title_color = parse_color(&config.style.header_color);

    let sensors: Vec<_> = stats
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

    let max_temp = sensors
        .iter()
        .map(|c| c.temperature().unwrap_or(0.0))
        .fold(0.0, f32::max);

    let content = format!(
        "Sensors: {} | Max Temp: {:.1}°C | Refresh: {}ms",
        sensors.len(),
        max_temp,
        config.general.refresh_rate_ms
    );

    let p = Paragraph::new(content)
        .alignment(Alignment::Center)
        .style(Style::default().fg(title_color).bold());

    f.render_widget(p, area);
}
