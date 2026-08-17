use crate::ui::util::color::is_color_light;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn render_tile(f: &mut Frame, area: Rect, label: &str, value: &str, color: Color) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Block::default().bg(color), inner);

    let text_color = if is_color_light(color) {
        Color::Black
    } else {
        Color::White
    };

    let text_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Fill(1),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(format!("{}\n{}", label, value))
            .alignment(Alignment::Center)
            .style(Style::default().fg(text_color).bold()),
        text_chunk[1],
    );
}
