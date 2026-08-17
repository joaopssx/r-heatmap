use crate::system::reading::Reading;
use std::fs;

pub fn pools() -> Vec<Reading> {
    match fs::read_to_string("/proc/meminfo") {
        Ok(text) => parse(&text),
        Err(e) => {
            log::warn!("Could not read /proc/meminfo: {}", e);
            Vec::new()
        }
    }
}

fn parse(text: &str) -> Vec<Reading> {
    ram(text).into_iter().chain(swap(text)).collect()
}

fn ram(text: &str) -> Option<Reading> {
    let total = field(text, "MemTotal")?;
    let available = field(text, "MemAvailable")?;

    pool("RAM", total, total - available)
}

fn swap(text: &str) -> Option<Reading> {
    let total = field(text, "SwapTotal")?;
    let free = field(text, "SwapFree")?;

    pool("Swap", total, total - free)
}

fn pool(name: &str, total: f32, used: f32) -> Option<Reading> {
    if total <= 0.0 {
        return None;
    }

    let gb = total / 1024.0 / 1024.0;
    let label = format!("{name} {gb:.1} GB");

    Some(Reading::new(label, used.max(0.0) / total * 100.0))
}

fn field(text: &str, name: &str) -> Option<f32> {
    text.lines()
        .find_map(|line| line.strip_prefix(name)?.strip_prefix(':'))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|kb| kb.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEMINFO: &str = "MemTotal:       32611368 kB
MemFree:         1204032 kB
MemAvailable:   24458526 kB
Buffers:          412672 kB
SwapCached:        13120 kB
SwapTotal:       8388604 kB
SwapFree:        7549743 kB
";

    #[test]
    fn measures_ram_against_available_not_free() {
        let pools = parse(MEMINFO);

        assert_eq!(pools.len(), 2);
        assert_eq!(pools[0].label, "RAM 31.1 GB");
        assert!((pools[0].value - 25.0).abs() < 0.1);
    }

    #[test]
    fn reads_swap_past_the_cached_line() {
        let pools = parse(MEMINFO);

        assert_eq!(pools[1].label, "Swap 8.0 GB");
        assert!((pools[1].value - 10.0).abs() < 0.1);
    }

    #[test]
    fn skips_swap_when_there_is_none() {
        let pools =
            parse("MemTotal: 1024 kB\nMemAvailable: 512 kB\nSwapTotal: 0 kB\nSwapFree: 0 kB\n");

        assert_eq!(pools.len(), 1);
        assert_eq!(pools[0].label, "RAM 0.0 GB");
    }
}
