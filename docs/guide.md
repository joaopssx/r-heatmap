# R-Heatmap Reference

## CLI

```text
r-heatmap [OPTIONS]

Options:
  -c, --config <PATH>          Path to the configuration file
                               (default: ./config.toml, then ~/.config/r-heatmap/config.toml)
  -l, --log-level <LOG_LEVEL>  Log level (debug, info, warn, error) [default: info]
      --no-gpu                 Disable GPU monitoring
  -h, --help                   Print help
  -V, --version                Print version
```

## KEYS

- `q`: quit.
- `Ctrl+C`: quit. Raw mode swallows the signal, so it is handled as a key.

## CONFIGURATION

Without `-c`, the lookup order is `./config.toml` and then
`$XDG_CONFIG_HOME/r-heatmap/config.toml`, falling back to `~/.config/r-heatmap/config.toml`.
A file that is missing, unreadable or invalid falls back to the built-in defaults and
logs why, so a broken config never takes the program down.

### Fields

- `general.refresh_rate_ms`: Update frequency in milliseconds. Values below 10 are clamped.
- `general.github_repo`: Text shown on the right side of the status bar.
- `style.border_color`: Primary UI border color.
- `style.header_color`: Information text color.
- `style.sensor_label_contains`: Filter list for hwmon labels.
- `style.disk_label_contains`: Filter list for disk hwmon labels. Optional, defaults to `["nvme", "drivetemp"]`. The disk row is hidden when nothing matches.
- `thresholds`: Temperature boundaries for heatmap color mapping.

## TEMPERATURES

CPU and disk temperatures come from `sysinfo`, which enumerates every `hwmon` chip.
Both rows are driven by substring filters over the sensor label, and the label is
composed by `sysinfo` as chip name, channel label and device model — an NVMe drive
shows up as something like `nvme Composite KINGSTON SNV2S1000G`, which is why
filtering by `"nvme"` is enough.

## FANS

Fan speeds are read straight from `fanN_input` under `/sys/class/hwmon/`, so there is
nothing to configure: every fan the kernel exposes gets a tile, and the row disappears
when no chip reports one. Tiles are colored against `fanN_max` when the chip publishes
it, and a fan sitting at 0 RPM is always red. Some machines expose the same physical
fan through two chips (`dell_ddv` and `dell_smm`, for instance) and will show
duplicated tiles.

## VOLTAGES

Voltages come from `inN_input` under `/sys/class/hwmon/`, converted from millivolts.
Like fans, they need no configuration and the row is hidden when no chip reports one.
A rail is green while it stays inside the `inN_min`/`inN_max` window published by the
chip and red once it leaves it; chips that publish no window are drawn blue, since
there is no way to tell a healthy 12V rail from a sagging one without it. Desktop
super-I/O chips expose a lot of rails, laptops usually expose only the battery.

## GPUS

Every `cardN` under `/sys/class/drm/` that exposes `device/gpu_busy_percent` gets its own
tile, so a hybrid laptop or a multi card box shows all of them instead of whichever came
first. Cards are labeled by node and driver (`card1 amdgpu`), read from `device/uevent`.
Connector entries such as `card0-eDP-1` are skipped, and so are cards without the counter:
`gpu_busy_percent` is an amdgpu interface, so Intel and NVIDIA cards usually report
nothing here and the row stays hidden. `--no-gpu` skips the scan altogether.

## ARCHITECTURE

- **System**: Data retrieval via `sysinfo` and `/sys` filesystem. `sysfs` holds the
  reading primitive: a label, a value and the path it came from, so a refresh rereads a
  single file instead of walking the tree again. `hwmon` builds readings by scanning a
  channel prefix (`fan`, `in`) with a scale factor, `gpu` builds them from the drm nodes.
  Adding another channel type is a matter of scanning a new prefix.
- **UI**: Terminal interface built with `ratatui`. Rows are laid out top to bottom
  (header, temperatures, disks, fans, voltages, GPUs, cores) and a row with no readings
  is given zero height. Fans, voltages and GPUs share one grid renderer, parameterized by
  decimals, unit and color function.
- **Events**: Crossterm-based event loop. Input is polled with whatever is left of the
  current tick, so keys respond immediately regardless of `refresh_rate_ms`.

## LOGGING

Logs are written to `r-heatmap.log` in the working directory and never to stdout, which
is occupied by the alternate screen. The file is appended across runs.

## TROUBLESHOOTING

Check `r-heatmap.log` for initialization errors or sensor detection failures. Ensure the
user has read permissions for entries in `/sys/class/hwmon/`. Startup logs how many fan
and voltage channels were found, which is the quickest way to tell a missing kernel
module from a wrong filter.
