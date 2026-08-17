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
    matches!(
        color,
        Color::Cyan | Color::Yellow | Color::Green | Color::White
    )
}

pub fn get_fan_color(rpm: f32, max: Option<f32>) -> Color {
    if rpm <= 0.0 {
        return Color::Red;
    }

    match max {
        Some(max) if max > 0.0 => get_usage_color(rpm / max * 100.0),
        _ => Color::Blue,
    }
}

pub fn get_clock_color(mhz: f32, max: Option<f32>) -> Color {
    match max {
        Some(max) if max > 0.0 => get_usage_color(mhz / max * 100.0),
        _ => Color::Blue,
    }
}

pub fn get_volt_color(volts: f32, min: Option<f32>, max: Option<f32>) -> Color {
    let out_of_range = min.is_some_and(|min| volts < min) || max.is_some_and(|max| volts > max);

    if out_of_range {
        Color::Red
    } else if min.is_some() || max.is_some() {
        Color::Green
    } else {
        Color::Blue
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
