use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Input(KeyEvent),
    Tick,
}

pub fn handle_events(tick_rate: Duration) -> Option<AppEvent> {
    if event::poll(tick_rate).ok()?
        && let Event::Key(key) = event::read().ok()?
    {
        return Some(AppEvent::Input(key));
    }
    Some(AppEvent::Tick)
}
