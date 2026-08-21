use crate::system::reading::Reading;
use crate::system::sysfs;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

const ROOT: &str = "/sys/devices/system/cpu";

thread_local! {
    static FILES: RefCell<HashMap<String, Option<File>>> = RefCell::new(HashMap::new());
}

pub fn annotate(cores: &mut [Reading]) {
    FILES.with(|files| annotate_with(Path::new(ROOT), &mut files.borrow_mut(), cores));
}

fn annotate_with(root: &Path, files: &mut HashMap<String, Option<File>>, cores: &mut [Reading]) {
    for core in cores {
        let Some(id) = core.label.strip_prefix("Core ") else {
            continue;
        };

        let file = files
            .entry(id.to_string())
            .or_insert_with(|| open(root, id));

        core.note = file
            .as_mut()
            .and_then(sysfs::reread)
            .map(|khz| format!("{:.2} GHz", khz / 1_000_000.0));
    }
}

fn open(root: &Path, id: &str) -> Option<File> {
    File::open(root.join(format!("cpu{id}/cpufreq/scaling_cur_freq"))).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_the_clock_of_each_core() {
        let root = std::env::temp_dir().join("r-heatmap-cpufreq");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("cpu0/cpufreq")).unwrap();
        fs::write(root.join("cpu0/cpufreq/scaling_cur_freq"), "3600000\n").unwrap();

        let mut files = HashMap::new();
        let mut cores = vec![
            Reading::new("Core 0".to_string(), 12.0),
            Reading::new("Core 1".to_string(), 30.0),
        ];

        annotate_with(&root, &mut files, &mut cores);
        fs::write(root.join("cpu0/cpufreq/scaling_cur_freq"), "800000\n").unwrap();
        annotate_with(&root, &mut files, &mut cores);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(cores[0].note.as_deref(), Some("0.80 GHz"));
        assert_eq!(cores[1].note, None);
        assert_eq!(files.len(), 2);
    }
}
