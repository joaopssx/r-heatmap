use crate::config::Config;
use crate::system::SystemStats;
use crate::util::color::parse_color;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};

pub fn render(f: &mut Frame, area: Rect, stats: &SystemStats, config: &Config) {
    let cpu = stats.cpu_usage();
    let (used_mem, total_mem) = stats.mem_usage();
    let mem_p = (used_mem as f32 / total_mem as f32) * 100.0;

    let mem_gb = used_mem as f64 / 1024.0 / 1024.0 / 1024.0;
    let total_gb = total_mem as f64 / 1024.0 / 1024.0 / 1024.0;

    let left = format!(
        " CPU: {:.1}% | RAM: {:.1}% ({:.1}/{:.1} GB)",
        cpu, mem_p, mem_gb, total_gb
    );
    let right = format!(" {} ", config.general.github_repo);

    let footer_style = Style::default()
        .bg(parse_color(&config.style.border_color))
        .fg(Color::Black)
        .bold();

    f.render_widget(
        Paragraph::new(left)
            .alignment(Alignment::Left)
            .style(footer_style),
        area,
    );
    f.render_widget(
        Paragraph::new(right)
            .alignment(Alignment::Right)
            .style(footer_style),
        area,
    );
}
