#[cfg(unix)]
use crate::system::power_supply;
use crate::system::reading::{self, Reading};
use std::cell::Cell;
use std::time::{Duration, Instant};

const INTERVAL: Duration = Duration::from_secs(5);

thread_local! {
    static LAST: Cell<Option<Instant>> = const { Cell::new(None) };
}

pub struct BatteryMonitor;

impl BatteryMonitor {
    pub fn scan() -> Vec<Reading> {
        let batteries = scan_platform();
        reading::found("batteries", &batteries);
        batteries
    }

    pub fn refresh(batteries: &mut [Reading]) {
        if batteries.is_empty() || !due() {
            return;
        }

        reading::update(batteries, &scan_platform());
    }
}

fn due() -> bool {
    LAST.with(|last| {
        let now = Instant::now();

        match last.get() {
            Some(last_read) if now.duration_since(last_read) < INTERVAL => false,
            _ => {
                last.set(Some(now));
                true
            }
        }
    })
}

#[cfg(unix)]
fn scan_platform() -> Vec<Reading> {
    power_supply::charge()
}

#[cfg(not(unix))]
fn scan_platform() -> Vec<Reading> {
    Vec::new()
}
