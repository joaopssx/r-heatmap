use crate::system::reading::Reading;
use crate::system::sysfs;
use std::path::Path;

pub fn annotate(cores: &mut [Reading]) {
    annotate_root(Path::new("/sys/devices/system/cpu"), cores);
}

fn annotate_root(root: &Path, cores: &mut [Reading]) {
    for core in cores {
        let Some(id) = core.label.strip_prefix("Core ") else {
            continue;
        };

        let path = root.join(format!("cpu{id}/cpufreq/scaling_cur_freq"));
        core.note = sysfs::read_number(&path).map(|khz| format!("{:.2} GHz", khz / 1000000.0));
    }
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

        let mut cores = vec![
            Reading::new("Core 0".to_string(), 12.0),
            Reading::new("Core 1".to_string(), 30.0),
        ];
        annotate_root(&root, &mut cores);
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(cores[0].note.as_deref(), Some("3.60 GHz"));
        assert_eq!(cores[1].note, None);
    }
}
