use crate::system::reading::Reading;
use crate::system::sysfs;
use std::fs;
use std::path::{Path, PathBuf};

pub fn charge() -> Vec<Reading> {
    collect(charge_of)
}

pub fn temperatures() -> Vec<Reading> {
    collect(temp_of)
}

fn collect(read: fn(&str, &Path) -> Option<Reading>) -> Vec<Reading> {
    match sysfs::class_dir("power_supply") {
        Some(root) => collect_root(&root, read),
        None => Vec::new(),
    }
}

fn collect_root(root: &Path, read: fn(&str, &Path) -> Option<Reading>) -> Vec<Reading> {
    let mut supplies: Vec<PathBuf> = match fs::read_dir(root) {
        Ok(entries) => entries.flatten().map(|entry| entry.path()).collect(),
        Err(e) => {
            log::warn!("Could not read {}: {}", root.display(), e);
            return Vec::new();
        }
    };
    supplies.sort();

    supplies
        .iter()
        .filter(|supply| is_battery(supply))
        .filter_map(|supply| {
            let name = supply.file_name()?.to_str()?;
            read(name, supply)
        })
        .collect()
}

fn is_battery(supply: &Path) -> bool {
    sysfs::read_text(&supply.join("type")).as_deref() == Some("Battery")
}

fn charge_of(name: &str, supply: &Path) -> Option<Reading> {
    let capacity = sysfs::read_number(&supply.join("capacity"))?;

    let mut reading = Reading::new(name.to_string(), capacity);
    reading.note = note(supply);

    Some(reading)
}

fn temp_of(name: &str, supply: &Path) -> Option<Reading> {
    Reading::from_file(format!("{name} temp"), supply.join("temp"), 0.1)
}

fn note(supply: &Path) -> Option<String> {
    let status = sysfs::read_text(&supply.join("status"))?;

    match draw(supply) {
        Some(watts) if watts > 0.0 => Some(format!("{status} {watts:.1} W")),
        _ => Some(status),
    }
}

fn draw(supply: &Path) -> Option<f32> {
    if let Some(microwatts) = sysfs::read_number(&supply.join("power_now")) {
        return Some(microwatts / 1_000_000.0);
    }

    let amps = sysfs::read_number(&supply.join("current_now"))? / 1_000_000.0;
    let volts = sysfs::read_number(&supply.join("voltage_now"))? / 1_000_000.0;

    Some(amps * volts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_supplies(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let root = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&root);

        fs::create_dir_all(root.join("AC")).unwrap();
        fs::write(root.join("AC/type"), "Mains\n").unwrap();
        fs::write(root.join("AC/online"), "0\n").unwrap();

        fs::create_dir_all(root.join("BAT0")).unwrap();
        fs::write(root.join("BAT0/type"), "Battery\n").unwrap();
        for (file, content) in files {
            fs::write(root.join("BAT0").join(file), content).unwrap();
        }

        root
    }

    #[test]
    fn reads_charge_and_says_what_the_battery_is_doing() {
        let root = fake_supplies(
            "r-heatmap-battery",
            &[
                ("capacity", "94\n"),
                ("status", "Discharging\n"),
                ("current_now", "607000\n"),
                ("voltage_now", "16111000\n"),
            ],
        );

        let charge = collect_root(&root, charge_of);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(charge.len(), 1);
        assert_eq!(charge[0].label, "BAT0");
        assert_eq!(charge[0].value, 94.0);
        assert_eq!(charge[0].note.as_deref(), Some("Discharging 9.8 W"));
    }

    #[test]
    fn falls_back_to_the_status_alone_when_nothing_is_flowing() {
        let root = fake_supplies(
            "r-heatmap-battery-full",
            &[("capacity", "100\n"), ("status", "Full\n")],
        );

        let charge = collect_root(&root, charge_of);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(charge[0].note.as_deref(), Some("Full"));
    }

    #[test]
    fn reads_tenths_of_a_degree() {
        let root = fake_supplies(
            "r-heatmap-battery-temp",
            &[
                ("capacity", "94\n"),
                ("status", "Full\n"),
                ("temp", "229\n"),
            ],
        );

        let temps = collect_root(&root, temp_of);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(temps[0].label, "BAT0 temp");
        assert!((temps[0].value - 22.9).abs() < 0.01);
    }

    #[test]
    fn a_desktop_with_no_battery_is_not_a_problem() {
        let root = fake_supplies("r-heatmap-battery-none", &[]);
        fs::remove_dir_all(root.join("BAT0")).unwrap();

        let charge = collect_root(&root, charge_of);
        fs::remove_dir_all(&root).unwrap();

        assert!(charge.is_empty());
    }
}
