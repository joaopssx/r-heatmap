use ratatui::style::Color;

pub fn parse_color(color: &str) -> Color {
    match color.to_lowercase().as_str() {
        "cyan" => Color::Cyan,
        "blue" => Color::Blue,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "red" => Color::Red,
        "magenta" => Color::Magenta,
        "white" => Color::White,
        _ => Color::Cyan,
    }
}

pub fn is_color_light(color: Color) -> bool {
    match color {
        Color::Cyan | Color::Yellow | Color::Green | Color::White => true,
        _ => false,
    }
}

pub fn get_usage_color(usage: f32) -> Color {
    if usage < 10.0 {
        Color::Rgb(20, 20, 20)
    } else if usage < 30.0 {
        Color::DarkGray
    } else if usage < 50.0 {
        Color::Blue
    } else if usage < 70.0 {
        Color::Green
    } else if usage < 85.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}
