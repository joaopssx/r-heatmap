use crate::config::Config;
use crate::system::SystemStats;
use crate::ui::util::color::parse_color;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::Paragraph,
};

pub fn render(f: &mut Frame, area: Rect, stats: &SystemStats, config: &Config) {
    let title_color = parse_color(&config.style.header_color);

    let max_temp = stats
        .temps
        .iter()
        .map(|reading| reading.value)
        .fold(0.0, f32::max);

    let content = format!(
        "Sensors: {} | Max Temp: {:.1}°C | Refresh: {}ms",
        stats.temps.len(),
        max_temp,
        config.general.refresh_rate_ms
    );

    let p = Paragraph::new(content)
        .alignment(Alignment::Center)
        .style(Style::default().fg(title_color).bold());

    f.render_widget(p, area);
}
