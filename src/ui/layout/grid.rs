use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn rows(count: usize) -> usize {
    if count == 0 {
        return 0;
    }

    let cols = (count as f32).sqrt().ceil();

    (count as f32 / cols).ceil() as usize
}

pub fn calculate_grid(area: Rect, count: usize) -> Vec<Vec<Rect>> {
    if count == 0 {
        return vec![];
    }

    let cols = (count as f32).sqrt().ceil() as u16;
    let rows = rows(count) as u16;

    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1); rows as usize])
        .split(area);

    row_chunks
        .iter()
        .map(|row_rect| {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Fill(1); cols as usize])
                .split(*row_rect)
                .to_vec()
        })
        .collect()
}
