use crate::system::sysfs::{self, Reading};
use std::fs;
use std::path::{Path, PathBuf};

pub struct GpuMonitor;

impl GpuMonitor {
    pub fn scan() -> Vec<Reading> {
        let gpus = scan_root(Path::new("/sys/class/drm"));
        log::info!("Found {} GPUs reporting usage", gpus.len());
        gpus
    }
}

fn scan_root(root: &Path) -> Vec<Reading> {
    let mut gpus = Vec::new();

    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Could not read {}: {}", root.display(), e);
            return gpus;
        }
    };

    let mut cards: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| is_card(path))
        .collect();
    cards.sort();

    for card in cards {
        let name = match card.file_name().and_then(|f| f.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let label = match driver(&card) {
            Some(driver) => format!("{name} {driver}"),
            None => name,
        };

        if let Some(reading) = Reading::new(label, card.join("device/gpu_busy_percent"), 1.0) {
            gpus.push(reading);
        }
    }

    gpus
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

    #[test]
    fn only_lists_cards_reporting_usage() {
        let root = std::env::temp_dir().join("r-heatmap-drm");
        let _ = fs::remove_dir_all(&root);

        for card in ["card0", "card1", "card1-eDP-1"] {
            fs::create_dir_all(root.join(card).join("device")).unwrap();
        }
        fs::write(root.join("card1/device/gpu_busy_percent"), "42\n").unwrap();
        fs::write(
            root.join("card1/device/uevent"),
            "DRIVER=amdgpu\nPCI_ID=1002:1636\n",
        )
        .unwrap();

        let gpus = scan_root(&root);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(gpus.len(), 1);
        assert_eq!(gpus[0].label, "card1 amdgpu");
        assert_eq!(gpus[0].value, 42.0);
    }
}
