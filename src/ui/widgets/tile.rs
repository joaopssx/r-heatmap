use crate::util::color::is_color_light;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn render_tile<F>(f: &mut Frame, area: Rect, label: &str, value: f32, unit: &str, color_fn: F)
where
    F: Fn(f32) -> ratatui::style::Color,
{
    let color = color_fn(value);
    let border_color = ratatui::style::Color::DarkGray;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Block::default().bg(color), inner);

    let text_color = if is_color_light(color) {
        ratatui::style::Color::Black
    } else {
        ratatui::style::Color::White
    };

    let text = format!("{}\n{:.1}{}", label, value, unit);

    let text_chunk = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Fill(1),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(text_color).bold()),
        text_chunk[1],
    );
}
