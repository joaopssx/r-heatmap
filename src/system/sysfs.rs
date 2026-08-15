use std::fs;
use std::path::{Path, PathBuf};

pub fn class_dir(name: &str) -> Option<PathBuf> {
    let dir = Path::new("/sys/class").join(name);
    dir.is_dir().then_some(dir)
}

pub fn read_text(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

pub fn read_number(path: &Path) -> Option<f32> {
    read_text(path)?.parse().ok()
}
