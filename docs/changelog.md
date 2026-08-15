# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Disk temperature row for NVMe and SATA drives, filtered by `style.disk_label_contains`.
- Fan speed tiles read from `fanN_input`, colored against `fanN_max`.
- Voltage tiles read from `inN_input`, colored against the `inN_min`/`inN_max` window.
- `hwmon` scanner shared by fans and voltages, with per channel scale factor.
- One tile per GPU reporting `gpu_busy_percent`, labeled by drm node and driver.
- Config lookup falls back to `~/.config/r-heatmap/config.toml`, or to
  `%APPDATA%\r-heatmap\config.toml` on Windows, where `HOME` does not exist.
- Windows support. CPU, cores and RAM work as they do on Linux. Disk temperatures are
  read from every physical drive with `IOCTL_STORAGE_QUERY_PROPERTY`, covering NVMe and
  SATA, on a handle opened with no access rights so it needs no elevation. GPU usage
  comes from the `GPU Engine` performance counters, summed per engine with the busiest
  one taken as the adapter figure, which is what Task Manager reports. Adapters are named
  from `HKLM\SOFTWARE\Microsoft\DirectX` and software ones are dropped, so
  `Microsoft Basic Render Driver` does not show up as an idle GPU. CPU temperature, fans
  and voltages have no user mode source and those rows stay hidden.
- `-l debug` lists every reading a monitor found instead of only the count.
- Console output code page is set to UTF-8 on Windows, otherwise the borders and the
  degree sign are decoded against the OEM code page.
- `computer` in the default sensor filter, the label Windows reports for the thermal
  zone. It matches no hwmon label.
- `Ctrl+C` quits, since raw mode swallows the signal.

### Changed

- `-c/--config` is honored. It was parsed and ignored.
- `--no-gpu` is honored. It was parsed and ignored.
- Logs go to `r-heatmap.log` only. Writing to stdout corrupted the alternate screen.
- Input is polled with the time left in the current tick instead of sleeping a full
  refresh interval, so keys no longer lag behind `refresh_rate_ms`.
- The terminal is restored even when the render loop fails.
- Header widget is rendered. It existed but was never called, which also left
  `style.header_color` unused.
- Tiles take a formatted value and a color, so temperatures, usage, RPM and volts can
  share the same widget.
- GPU detection lists every card instead of returning the first one found, and no longer
  reads `/sys` from inside the render loop. The status bar dropped its GPU field, which
  could only ever show one card.
- Reading primitive moved from `hwmon` to `sysfs`, now that GPUs use it too.
- `hwmon` and `gpu` resolve their `/sys/class` root through `sysfs::class_dir` and give
  up quietly when it is not a directory, instead of warning about a missing path on
  systems that never had one.
- The reading primitive moved out of `sysfs` into `reading`, now that a reading can come
  from a batch query instead of a file, and every row is a monitor with one
  implementation per system behind it.
- The temperature row collapses when empty, like every other row. It was reserving six
  lines on machines with no CPU sensor.
- Tiles in a row are sized with `Fill` instead of a rounded percentage. The leftover
  lines used to pile onto a single row, leaving the rest one line short and hiding the
  value under the label.
- The disk row is rendered from readings rather than straight from `sysinfo` components,
  so both sources can feed it.

## [0.1.0]

### Added

- Initial release of `r-heatmap`.
- Dynamic TUI with `ratatui`.
- System temperature monitoring with `sysinfo`.
- Configurable TOML support.
- Modular architecture (config, system, ui).
- Status bar with CPU and RAM usage.
- GitHub link in footer.
