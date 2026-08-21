use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
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

pub fn reread(file: &mut File) -> Option<f32> {
    reread_text(file)?.parse().ok()
}

pub fn reread_text(file: &mut File) -> Option<String> {
    let mut text = String::new();

    file.seek(SeekFrom::Start(0)).ok()?;
    file.read_to_string(&mut text).ok()?;

    Some(text.trim().to_string())
}
