use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn calculate_grid(area: Rect, count: usize) -> Vec<Vec<Rect>> {
    if count == 0 {
        return vec![];
    }

    let cols = (count as f32).sqrt().ceil() as u16;
    let rows = (count as f32 / cols as f32).ceil() as u16;

    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100 / rows); rows as usize])
        .split(area);

    row_chunks
        .iter()
        .map(|row_rect| {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(100 / cols); cols as usize])
                .split(*row_rect)
                .to_vec()
        })
        .collect()
}
