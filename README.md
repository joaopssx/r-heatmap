# r-heatmap

System thermal and load monitoring utility for Linux terminals.

**Developed by: joaopssx**

## What it shows

- CPU package and core temperatures, colored by threshold.
- Disk temperatures (NVMe and SATA).
- Fan speeds in RPM.
- Voltage rails, when the chip exposes them.
- Per core usage, plus a status bar with global CPU, RAM and GPU usage.

Rows with nothing to report are hidden instead of showing empty tiles, so a laptop
without voltage sensors simply does not get a voltage row.

## Installation

```bash
cargo install --path .
```

The trailing dot is required. The binary lands in `~/.cargo/bin`, which needs to be
on your `PATH`.

## Usage

```bash
r-heatmap [OPTIONS]
```

Press `q` or `Ctrl+C` to quit.

### Options

- `-c, --config <PATH>`: Custom configuration file.
- `-l, --log-level <LEVEL>`: Log verbosity (debug, info, warn, error). Default `info`.
- `--no-gpu`: Skip GPU data collection.

## Configuration

Without `-c`, the config is looked up in `./config.toml` first, then in
`~/.config/r-heatmap/config.toml` (or `$XDG_CONFIG_HOME/r-heatmap/config.toml`). A
missing or invalid file is not fatal: the built-in defaults are used and the reason
goes to the log.

```toml
[general]
refresh_rate_ms = 500
github_repo = "joaopssx/r-heatmap"

[style]
border_color = "Cyan"
header_color = "Yellow"
sensor_label_contains = ["core", "tctl", "tccd", "cpu", "package", "die"]
disk_label_contains = ["nvme", "drivetemp"]

[thresholds]
cold = 40.0
warm = 60.0
hot = 80.0
critical = 90.0
```

Fans and voltages have no filter list: every channel the kernel exposes is shown.

## Logging

Everything goes to `r-heatmap.log` in the current directory, never to the terminal —
a stray log line would corrupt the interface. Start with `-l debug` when a sensor is
missing.

## Requirements

- Rust stable
- Linux kernel with `hwmon` and `sysfs` support.

NVMe drives report their temperature out of the box. SATA drives need the `drivetemp`
module:

```bash
sudo modprobe drivetemp
echo drivetemp | sudo tee /etc/modules-load.d/drivetemp.conf
```

## License

MIT (c) joaopssx
