# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Disk temperature row for NVMe and SATA drives, filtered by `style.disk_label_contains`.
- Fan speed tiles read from `fanN_input`, colored against `fanN_max`.
- Voltage tiles read from `inN_input`, colored against the `inN_min`/`inN_max` window.
- `hwmon` scanner shared by fans and voltages, with per channel scale factor.
- Config lookup falls back to `~/.config/r-heatmap/config.toml`.
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

## [0.1.0]

### Added

- Initial release of `r-heatmap`.
- Dynamic TUI with `ratatui`.
- System temperature monitoring with `sysinfo`.
- Configurable TOML support.
- Modular architecture (config, system, ui).
- Status bar with CPU and RAM usage.
- GitHub link in footer.
