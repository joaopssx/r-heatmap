use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub enum AppEvent {
    Input(KeyCode),
    Tick,
}

pub fn handle_events(tick_rate: Duration) -> Option<AppEvent> {
    if event::poll(tick_rate).ok()? {
        if let Event::Key(key) = event::read().ok()? {
            return Some(AppEvent::Input(key.code));
        }
    }
    Some(AppEvent::Tick)
}
