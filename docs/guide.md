# R-Heatmap Reference

## CLI

```text
r-heatmap [OPTIONS]

Options:
  -c, --config <PATH>      Path to config.toml
  -l, --log-level <LEVEL>  Logging level (debug, info, warn, error)
  --no-gpu                 Disable GPU statistics
  -h, --help               Print help
  -V, --version            Print version
```

## CONFIGURATION

The software looks for `config.toml` in the working directory by default.

### Fields

- `general.refresh_rate_ms`: Update frequency in milliseconds.
- `style.border_color`: Primary UI border color.
- `style.header_color`: Information text color.
- `style.sensor_label_contains`: Filter list for hwmon labels.
- `style.disk_label_contains`: Filter list for disk hwmon labels. Optional, defaults to `["nvme", "drivetemp"]`. The disk row is hidden when nothing matches.
- `thresholds`: Temperature boundaries for heatmap color mapping.

## ARCHITECTURE

- **System**: Data retrieval via `sysinfo` and `/sys` filesystem.
- **UI**: Terminal interface built with `ratatui`.
- **Events**: Crossterm-based event loop with support for manual quit ('q').

## TROUBLESHOOTING

Check `r-heatmap.log` for initialization errors or sensor detection failures. Ensure the user has read permissions for entries in `/sys/class/hwmon/`.
