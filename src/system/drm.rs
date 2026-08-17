use crate::system::hwmon;
use crate::system::reading::Reading;
use crate::system::sysfs;
use std::fs;
use std::path::{Path, PathBuf};

pub fn usage() -> Vec<Reading> {
    collect(usage_of)
}

pub fn temperatures() -> Vec<Reading> {
    collect(temps_of)
}

pub fn clocks() -> Vec<Reading> {
    collect(clocks_of)
}

fn collect(read: fn(&str, &Path) -> Vec<Reading>) -> Vec<Reading> {
    match sysfs::class_dir("drm") {
        Some(root) => collect_root(&root, read),
        None => Vec::new(),
    }
}

fn collect_root(root: &Path, read: fn(&str, &Path) -> Vec<Reading>) -> Vec<Reading> {
    let mut readings = Vec::new();

    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Could not read {}: {}", root.display(), e);
            return readings;
        }
    };

    let mut cards: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| is_card(path))
        .collect();
    cards.sort();

    for card in cards {
        if let Some(node) = card.file_name().and_then(|f| f.to_str()) {
            readings.extend(read(node, &card));
        }
    }

    readings
}

fn usage_of(node: &str, card: &Path) -> Vec<Reading> {
    let label = match driver(card) {
        Some(driver) => format!("{node} {driver}"),
        None => node.to_string(),
    };

    Reading::from_file(label, card.join("device/gpu_busy_percent"), 1.0)
        .into_iter()
        .collect()
}

fn temps_of(node: &str, card: &Path) -> Vec<Reading> {
    match hwmon::chip_dir(&card.join("device")) {
        Some(chip) => hwmon::scan_chip(&chip, node, "temp", 0.001),
        None => Vec::new(),
    }
}

fn clocks_of(node: &str, card: &Path) -> Vec<Reading> {
    let Some(chip) = hwmon::chip_dir(&card.join("device")) else {
        return Vec::new();
    };

    let mut clocks = hwmon::scan_chip(&chip, node, "freq", 1e-6);
    for clock in &mut clocks {
        let domain = clock.label.rsplit(' ').next().unwrap_or_default();
        clock.max = top_state(&card.join(format!("device/pp_dpm_{domain}")));
    }

    clocks
}

fn top_state(path: &Path) -> Option<f32> {
    let states = sysfs::read_text(path)?;

    states
        .lines()
        .filter_map(|line| {
            line.split_whitespace()
                .find(|token| token.to_lowercase().ends_with("mhz"))
        })
        .filter_map(|state| state[..state.len() - 3].parse::<f32>().ok())
        .max_by(f32::total_cmp)
}

fn is_card(path: &Path) -> bool {
    match path.file_name().and_then(|f| f.to_str()) {
        Some(name) => match name.strip_prefix("card") {
            Some(id) => !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()),
            None => false,
        },
        None => false,
    }
}

fn driver(card: &Path) -> Option<String> {
    let uevent = sysfs::read_text(&card.join("device/uevent"))?;

    uevent
        .lines()
        .find_map(|line| line.strip_prefix("DRIVER=").map(|d| d.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_drm(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&root);

        for card in ["card0", "card1", "card1-eDP-1"] {
            fs::create_dir_all(root.join(card).join("device")).unwrap();
        }
        fs::create_dir_all(root.join("card1/device/hwmon/hwmon4")).unwrap();
        fs::write(root.join("card1/device/gpu_busy_percent"), "42\n").unwrap();
        fs::write(
            root.join("card1/device/uevent"),
            "DRIVER=amdgpu\nPCI_ID=1002:1636\n",
        )
        .unwrap();

        root
    }

    fn write_chip(root: &Path, files: &[(&str, &str)]) {
        for (name, content) in files {
            fs::write(root.join("card1/device/hwmon/hwmon4").join(name), content).unwrap();
        }
    }

    #[test]
    fn only_lists_cards_reporting_usage() {
        let root = fake_drm("r-heatmap-drm-usage");

        let gpus = collect_root(&root, usage_of);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(gpus.len(), 1);
        assert_eq!(gpus[0].label, "card1 amdgpu");
        assert_eq!(gpus[0].value, 42.0);
    }

    #[test]
    fn reads_amdgpu_temperatures() {
        let root = fake_drm("r-heatmap-drm-temps");
        write_chip(
            &root,
            &[
                ("name", "amdgpu\n"),
                ("temp1_input", "48000\n"),
                ("temp1_label", "edge\n"),
                ("temp2_input", "51000\n"),
                ("temp2_label", "junction\n"),
            ],
        );

        let temps = collect_root(&root, temps_of);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(temps.len(), 2);
        assert_eq!(temps[0].label, "card1 edge");
        assert!((temps[0].value - 48.0).abs() < 0.01);
        assert_eq!(temps[1].label, "card1 junction");
    }

    #[test]
    fn reads_clocks_against_the_top_dpm_state() {
        let root = fake_drm("r-heatmap-drm-clocks");
        write_chip(
            &root,
            &[
                ("name", "amdgpu\n"),
                ("freq1_input", "1850000000\n"),
                ("freq1_label", "sclk\n"),
            ],
        );
        fs::write(
            root.join("card1/device/pp_dpm_sclk"),
            "0: 500Mhz\n1: 1200Mhz *\n2: 2200Mhz\n",
        )
        .unwrap();

        let clocks = collect_root(&root, clocks_of);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(clocks.len(), 1);
        assert_eq!(clocks[0].label, "card1 sclk");
        assert_eq!(clocks[0].value, 1850.0);
        assert_eq!(clocks[0].max, Some(2200.0));
    }
}
