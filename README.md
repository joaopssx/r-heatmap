# r-heatmap

System thermal and load monitoring utility for Linux and Windows terminals.

**Developed by: joaopssx**

## What it shows

- RAM and swap pressure, read from `/proc/meminfo` on Linux.
- Every temperature the machine exposes, found on its own, split into CPU, chipset, disk
  and everything else.
- Fan speeds in RPM.
- Voltage rails, when the chip exposes them.
- CPU power draw in watts, from the RAPL energy counters.
- Usage of every GPU that reports it, one tile per card.
- GPU temperature and clocks on AMD cards, straight from the `amdgpu` driver.
- Per core usage, measured from `/proc/stat` between refreshes, with the current clock of
  that core next to it, plus a status bar with global CPU and RAM usage.

Rows with nothing to report are hidden instead of showing empty tiles, so a laptop
without voltage sensors simply does not get a voltage row. Which rows you get depends on
the system: see [Requirements](#requirements).

## Installation

```bash
cargo install --path .
```

The trailing dot is required. The binary lands in `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin`
on Windows), which needs to be on your `PATH`.

## Usage

```bash
r-heatmap [OPTIONS]
```

Press `q` or `Ctrl+C` to quit.

### Options

- `-c, --config <PATH>`: Custom configuration file.
- `-l, --log-level <LEVEL>`: Log verbosity (debug, info, warn, error). Default `info`.
- `--no-gpu`: Skip GPU detection entirely, hiding the GPU row.

## Configuration

Without `-c`, the config is looked up in `./config.toml` first, then in
`~/.config/r-heatmap/config.toml` (or `$XDG_CONFIG_HOME/r-heatmap/config.toml`) on Linux
and in `%APPDATA%\r-heatmap\config.toml` on Windows. A missing or invalid file is not
fatal: the built-in defaults are used and the reason goes to the log.

```toml
[general]
refresh_rate_ms = 500
github_repo = "joaopssx/r-heatmap"

[style]
border_color = "Cyan"
header_color = "Yellow"
sensor_label_contains = ["core", "tctl", "tccd", "cpu", "package", "die", "computer"]
disk_label_contains = ["nvme", "drivetemp"]
board_label_contains = ["pch", "systin", "chipset", "motherboard", "ambient"]
show_other_sensors = false

[thresholds]
cold = 40.0
warm = 60.0
hot = 80.0
critical = 90.0
```

Nothing in there declares a sensor. Every `hwmon` chip is scanned at startup and the
filters only decide which row a discovered sensor lands in: `disk_label_contains` picks
the disk row, `board_label_contains` the chipset row, `sensor_label_contains` the CPU row,
and whatever is left is hidden unless `show_other_sensors` is on. If no sensor matches `sensor_label_contains` the filter is
ignored and everything found is shown, so an unknown chip name never leaves the screen
empty.

Fans and voltages have no filter list either: every channel the kernel exposes is shown.

## Logging

Everything goes to `r-heatmap.log` in the current directory, never to the terminal —
a stray log line would corrupt the interface. Start with `-l debug` when a sensor is
missing.

## Requirements

Rust stable, plus one of:

### Linux

A kernel with `hwmon` and `sysfs` support, which is everything but an embedded build.

AMD GPUs need nothing installed: `amdgpu` publishes usage, temperature and clocks in
`sysfs`, and all three are read from there. Intel and NVIDIA cards publish none of it
through the open driver, so their rows stay hidden.

The power row needs the RAPL energy counters, which most kernels keep root only since
CVE-2020-8694 — the counter leaks enough timing to be a side channel. Without access the
row is hidden and the log says so. To see it as a normal user:

```bash
sudo chmod o+r /sys/class/powercap/*/energy_uj
```

That lasts until reboot; a udev rule makes it stick. Running the whole program as root
works too.

NVMe drives report their temperature out of the box. SATA drives need the `drivetemp`
module:

```bash
sudo modprobe drivetemp
echo drivetemp | sudo tee /etc/modules-load.d/drivetemp.conf
```

### Windows

Windows 10 or 11. Nothing to install, no elevation.

Working: CPU usage, per core usage and RAM, from the same `sysinfo` calls used on Linux.
Disk temperatures, read off every physical drive with `IOCTL_STORAGE_QUERY_PROPERTY`,
which covers NVMe and SATA. GPU usage, from the same performance counters Task Manager
reads, one tile per hardware adapter and named after it.

Not working, and not fixable from user mode: CPU temperature, fan speeds and voltages.
On Linux those come from `k10temp`, `nct6775` and friends — kernel modules that talk to
the CPU's internal registers and to the super-I/O chip on the board. Windows ships no
equivalent driver, so every tool that shows those numbers (HWiNFO, LibreHardwareMonitor,
the vendor utilities) installs a signed kernel driver of its own. This one does not, so
the three rows stay hidden.

The one exception Windows does expose is the ACPI thermal zone, which `sysinfo` reads
over WMI and which shows up as a single `Computer` sensor. Most desktop boards do not
implement it. Check yours:

```powershell
Get-CimInstance -Namespace root/WMI -ClassName MSAcpi_ThermalZoneTemperature
```

`Sem suporte` / `Not supported` means the temperature row stays empty.

## License

MIT (c) joaopssx
