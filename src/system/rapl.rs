use crate::system::reading::Reading;
use crate::system::sysfs;
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

thread_local! {
    static PREVIOUS: RefCell<Option<Snapshot>> = const { RefCell::new(None) };
}

struct Snapshot {
    taken: Instant,
    zones: Vec<Zone>,
}

struct Zone {
    label: String,
    energy: f64,
    range: f64,
    limit: Option<f32>,
}

pub fn power() -> Vec<Reading> {
    PREVIOUS.with(|previous| {
        let mut previous = previous.borrow_mut();
        let current = read(previous.is_none());

        let power = match previous.as_ref() {
            Some(before) => watts(&current, before),
            None => current
                .zones
                .iter()
                .map(|zone| reading(zone, 0.0))
                .collect(),
        };

        *previous = Some(current);
        power
    })
}

fn read(first: bool) -> Snapshot {
    let zones = match sysfs::class_dir("powercap") {
        Some(root) => zones(&root, first),
        None => Vec::new(),
    };

    Snapshot {
        taken: Instant::now(),
        zones,
    }
}

fn zones(root: &Path, first: bool) -> Vec<Zone> {
    let mut dirs: Vec<PathBuf> = match fs::read_dir(root) {
        Ok(entries) => entries.flatten().map(|entry| entry.path()).collect(),
        Err(e) => {
            log::warn!("Could not read {}: {}", root.display(), e);
            return Vec::new();
        }
    };
    dirs.sort_by_key(|dir| order(dir));

    let mut zones: Vec<Zone> = Vec::new();
    let mut locked = 0;

    for dir in dirs {
        let input = dir.join("energy_uj");
        if !input.is_file() {
            continue;
        }

        let Some(energy) = number(&input) else {
            locked += 1;
            continue;
        };

        let Some(label) = label(&dir) else {
            continue;
        };

        if zones.iter().any(|zone| zone.label == label) {
            continue;
        }

        zones.push(Zone {
            label,
            energy,
            range: number(&dir.join("max_energy_range_uj")).unwrap_or(f64::MAX),
            limit: limit(&dir),
        });
    }

    if locked > 0 && first {
        log::warn!(
            "{} powercap zones are readable only by root, so the power row stays empty. \
             Run as root, or give the energy counters read access: \
             sudo chmod o+r /sys/class/powercap/*/energy_uj",
            locked
        );
    }

    zones
}

fn order(dir: &Path) -> (String, String) {
    let name = dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    match name.split_once(':') {
        Some((driver, zone)) => (driver.to_string(), zone.to_string()),
        None => (name.to_string(), String::new()),
    }
}

fn label(dir: &Path) -> Option<String> {
    let name = sysfs::read_text(&dir.join("name"))?;
    let parent = fs::canonicalize(dir).ok()?.parent()?.to_path_buf();

    match sysfs::read_text(&parent.join("name")) {
        Some(parent) => Some(format!("{parent} {name}")),
        None => Some(name),
    }
}

fn limit(dir: &Path) -> Option<f32> {
    let watts = number(&dir.join("constraint_0_power_limit_uw"))? / 1_000_000.0;

    (watts > 0.0).then_some(watts as f32)
}

fn number(path: &Path) -> Option<f64> {
    sysfs::read_text(path)?.parse().ok()
}

fn watts(current: &Snapshot, before: &Snapshot) -> Vec<Reading> {
    let seconds = current.taken.duration_since(before.taken).as_secs_f64();

    current
        .zones
        .iter()
        .map(|zone| {
            let value = before
                .zones
                .iter()
                .find(|old| old.label == zone.label)
                .map_or(0.0, |old| draw(zone, old, seconds));

            reading(zone, value)
        })
        .collect()
}

fn draw(zone: &Zone, before: &Zone, seconds: f64) -> f32 {
    if seconds <= 0.0 {
        return 0.0;
    }

    let mut used = zone.energy - before.energy;
    if used < 0.0 {
        used += zone.range;
    }

    (used / 1_000_000.0 / seconds) as f32
}

fn reading(zone: &Zone, value: f32) -> Reading {
    let mut reading = Reading::new(zone.label.clone(), value);
    reading.max = zone.limit;

    reading
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn zone(label: &str, energy: f64) -> Zone {
        Zone {
            label: label.to_string(),
            energy,
            range: 262_143_328_850.0,
            limit: Some(30.0),
        }
    }

    fn snapshot(zones: Vec<Zone>, taken: Instant) -> Snapshot {
        Snapshot { taken, zones }
    }

    fn fake_powercap() -> PathBuf {
        let root = std::env::temp_dir().join("r-heatmap-powercap");
        let _ = fs::remove_dir_all(&root);

        let package = root.join("intel-rapl/intel-rapl:0");
        let core = package.join("intel-rapl:0:0");
        let mmio = root.join("intel-rapl-mmio/intel-rapl-mmio:0");
        fs::create_dir_all(&core).unwrap();
        fs::create_dir_all(&mmio).unwrap();

        for (dir, name, energy) in [
            (&package, "package-0", "9000000"),
            (&core, "core", "4000000"),
            (&mmio, "package-0", "9000004"),
        ] {
            fs::write(dir.join("name"), format!("{name}\n")).unwrap();
            fs::write(dir.join("energy_uj"), format!("{energy}\n")).unwrap();
        }
        fs::write(package.join("constraint_0_power_limit_uw"), "30000000\n").unwrap();

        for (link, target) in [
            ("intel-rapl:0", &package),
            ("intel-rapl:0:0", &core),
            ("intel-rapl-mmio:0", &mmio),
        ] {
            std::os::unix::fs::symlink(target, root.join(link)).unwrap();
        }

        root
    }

    #[test]
    fn names_subzones_after_the_package_and_drops_the_second_reading_of_it() {
        let root = fake_powercap();

        let zones = zones(&root, false);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(zones.len(), 2);
        assert_eq!(zones[0].label, "package-0");
        assert_eq!(zones[0].limit, Some(30.0));
        assert_eq!(zones[1].label, "package-0 core");
    }

    #[test]
    fn turns_joules_into_watts() {
        let start = Instant::now();
        let before = snapshot(vec![zone("package-0", 1_000_000.0)], start);
        let current = snapshot(
            vec![zone("package-0", 8_500_000.0)],
            start + Duration::from_millis(500),
        );

        let power = watts(&current, &before);

        assert_eq!(power[0].label, "package-0");
        assert_eq!(power[0].value, 15.0);
        assert_eq!(power[0].max, Some(30.0));
    }

    #[test]
    fn survives_the_counter_wrapping_around() {
        let start = Instant::now();
        let before = snapshot(
            vec![zone("package-0", 262_143_328_850.0 - 1_000_000.0)],
            start,
        );
        let current = snapshot(
            vec![zone("package-0", 4_000_000.0)],
            start + Duration::from_secs(1),
        );

        let power = watts(&current, &before);

        assert_eq!(power[0].value, 5.0);
    }

    #[test]
    fn reports_nothing_for_a_zone_that_just_appeared() {
        let start = Instant::now();
        let before = snapshot(Vec::new(), start);
        let current = snapshot(
            vec![zone("package-0", 4_000_000.0)],
            start + Duration::from_secs(1),
        );

        assert_eq!(watts(&current, &before)[0].value, 0.0);
    }
}
