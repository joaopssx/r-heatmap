# r-heatmap

System thermal and load monitoring utility for Linux terminals.

**Developed by: joaopssx**

## Installation

```bash
cargo install --path .
```

## Usage

```bash
r-heatmap [OPTIONS]
```

### Options

- `-c, --config <PATH>`: Custom configuration file (default: config.toml).
- `-l, --log-level <LEVEL>`: Log verbosity (debug, info, warn, error).
- `--no-gpu`: Skip GPU data collection.

## Configuration

Configuration is managed via `config.toml`:

```toml
[general]
refresh_rate_ms = 500
github_repo = "joaopssx/r-heatmap"

[style]
border_color = "Cyan"
header_color = "Yellow"
sensor_label_contains = ["core", "tctl", "tccd", "cpu", "package", "die"]

[thresholds]
cold = 40.0
warm = 60.0
hot = 80.0
critical = 90.0
```

## Requirements

- Rust stable
- Linux kernel with `hwmon` and `sysfs` support.

## License

MIT (c) joaopssx
