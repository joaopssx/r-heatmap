# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Disk temperature row for NVMe and SATA drives, filtered by `style.disk_label_contains`.
- Fan speed tiles read from `fanN_input`, colored against `fanN_max`.
- Voltage tiles read from `inN_input`, colored against the `inN_min`/`inN_max` window.
- `hwmon` scanner shared by fans and voltages, with per channel scale factor.
- One tile per GPU reporting `gpu_busy_percent`, labeled by drm node and driver.
- Temperatures are discovered by scanning every `hwmon` chip instead of being read
  through `sysinfo`. The config no longer declares which sensors exist, it only sorts the
  discovered ones into the CPU, disk and other rows, and a `sensor_label_contains` that
  matches nothing is ignored rather than leaving the screen empty.
- Power row in watts, from the RAPL energy counters under `/sys/class/powercap/`, as the
  energy delta between two refreshes over the time between them. Counter wrap is handled
  through `max_energy_range_uj`, tiles are colored against the firmware's long term power
  limit, and the row stays hidden with one explanatory warning when the counters are root
  only, which is the default on most kernels since CVE-2020-8694.
- Current clock of each core, from `cpufreq/scaling_cur_freq`, printed inside the core
  tile next to its usage, so throttling can be read against the temperature row.
- Chipset row, split off from the CPU one, for the board's own probes: `PCH_CHIP_TEMP`,
  `SYSTIN`, `Ambient` and anything else matching `style.board_label_contains`.
- `style.show_other_sensors`, off by default, for the sensors no filter claimed.
- Per core usage is measured from `/proc/stat` on Linux, as the delta between two
  refreshes, with `iowait` counted as idle. The row is rendered like every other one
  instead of through a widget of its own.
- Memory row with RAM and swap pressure, parsed from `/proc/meminfo`. RAM is measured
  against `MemAvailable`, not `MemFree`, so the page cache does not read as used memory.
  The swap tile is dropped on machines with no swap.
- GPU temperature and clock rows on Linux, read from the hwmon chip `amdgpu` hangs off
  each card. Channels are labeled by card, so a hybrid laptop does not mix the discrete
  card with the integrated one. Clocks are colored against the top DPM state published in
  `pp_dpm_sclk`/`pp_dpm_mclk`.
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
- `get_clock_color` is now `get_ratio_color`: GPU clocks and power tiles both color a
  value against a ceiling.
- Rows are as tall as their grid needs, four lines per row of tiles. A row with twelve
  sensors used to get the same six lines as a row with two, which left two lines per tile
  and hid every value behind its own border.
- hwmon chips are visited in sorted order, so fans, voltages and temperatures keep their
  places between runs.
- `sysinfo` components are only read on Windows now, and are no longer refreshed every
  tick on Linux, where that meant walking all of `hwmon` a second time per frame.
- The hwmon scanner takes one chip directory at a time, so chips that are not under
  `/sys/class/hwmon` can go through it.
- `util` moved under `ui`, where both of its files are used.