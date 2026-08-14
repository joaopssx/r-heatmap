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

## FANS

Fan speeds are read straight from `fanN_input` under `/sys/class/hwmon/`, so there is nothing to configure: every fan the kernel exposes gets a tile, and the row disappears when no chip reports one. Tiles are colored against `fanN_max` when the chip publishes it, and a fan sitting at 0 RPM is always red. Some machines expose the same physical fan through two chips (`dell_ddv` and `dell_smm`, for instance) and will show duplicated tiles.

## VOLTAGES

Voltages come from `inN_input` under `/sys/class/hwmon/`, converted from millivolts. Like fans, they need no configuration and the row is hidden when no chip reports one. A rail is green while it stays inside the `inN_min`/`inN_max` window published by the chip and red once it leaves it; chips that publish no window are drawn blue, since there is no way to tell a healthy 12V rail from a sagging one without it. Desktop super-I/O chips expose a lot of rails, laptops usually expose only the battery.

## ARCHITECTURE

- **System**: Data retrieval via `sysinfo` and `/sys` filesystem.
- **UI**: Terminal interface built with `ratatui`.
- **Events**: Crossterm-based event loop with support for manual quit ('q').

## TROUBLESHOOTING

Check `r-heatmap.log` for initialization errors or sensor detection failures. Ensure the user has read permissions for entries in `/sys/class/hwmon/`.
