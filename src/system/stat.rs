use crate::system::reading::Reading;
use std::cell::RefCell;
use std::fs;

thread_local! {
    static PREVIOUS: RefCell<Vec<Times>> = const { RefCell::new(Vec::new()) };
}

struct Times {
    name: String,
    busy: f64,
    total: f64,
}

pub fn cores() -> Vec<Reading> {
    let current = read();

    PREVIOUS.with(|previous| {
        let mut previous = previous.borrow_mut();
        let cores = usage(&current, &previous);
        *previous = current;

        cores
    })
}

fn read() -> Vec<Times> {
    match fs::read_to_string("/proc/stat") {
        Ok(text) => parse(&text),
        Err(e) => {
            log::warn!("Could not read /proc/stat: {}", e);
            Vec::new()
        }
    }
}

fn parse(text: &str) -> Vec<Times> {
    text.lines().filter_map(times).collect()
}

fn times(line: &str) -> Option<Times> {
    let mut fields = line.split_whitespace();

    let id = fields.next()?.strip_prefix("cpu")?;
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let values: Vec<f64> = fields.filter_map(|field| field.parse().ok()).collect();
    if values.len() < 5 {
        return None;
    }

    let total: f64 = values.iter().sum();
    let idle = values[3] + values[4];

    Some(Times {
        name: format!("Core {id}"),
        busy: total - idle,
        total,
    })
}

fn usage(current: &[Times], previous: &[Times]) -> Vec<Reading> {
    current
        .iter()
        .map(|core| {
            let value = previous
                .iter()
                .find(|before| before.name == core.name)
                .map_or(0.0, |before| delta(core, before));

            Reading::new(core.name.clone(), value)
        })
        .collect()
}

fn delta(core: &Times, before: &Times) -> f32 {
    let total = core.total - before.total;

    if total <= 0.0 {
        return 0.0;
    }

    ((core.busy - before.busy) / total * 100.0) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: &str = "cpu  100 0 100 800 0 0 0 0 0 0
cpu0 50 0 50 400 0 0 0 0 0 0
cpu1 50 0 50 400 0 0 0 0 0 0
intr 12345
ctxt 6789
";

    const SECOND: &str = "cpu  180 0 120 900 0 0 0 0 0 0
cpu0 130 0 70 400 0 0 0 0 0 0
cpu1 50 0 50 500 0 0 0 0 0 0
";

    #[test]
    fn skips_the_aggregate_line_and_everything_after_it() {
        let cores = parse(FIRST);

        assert_eq!(cores.len(), 2);
        assert_eq!(cores[0].name, "Core 0");
        assert_eq!(cores[1].name, "Core 1");
    }

    #[test]
    fn measures_the_delta_between_two_snapshots() {
        let cores = usage(&parse(SECOND), &parse(FIRST));

        assert_eq!(cores.len(), 2);
        assert_eq!(cores[0].value, 100.0);
        assert_eq!(cores[1].value, 0.0);
    }

    #[test]
    fn reports_nothing_until_there_is_something_to_compare_against() {
        let cores = usage(&parse(FIRST), &[]);

        assert_eq!(cores[0].value, 0.0);
    }

    #[test]
    fn counts_iowait_as_idle() {
        let busy = "cpu0 50 0 50 400 0 0 0 0 0 0";
        let waiting = "cpu0 50 0 50 400 100 0 0 0 0 0";

        let cores = usage(&parse(waiting), &parse(busy));

        assert_eq!(cores[0].value, 0.0);
    }
}
