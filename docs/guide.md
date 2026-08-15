# R-Heatmap Reference

## CLI

```text
r-heatmap [OPTIONS]

Options:
  -c, --config <PATH>          Path to the configuration file
                               (default: ./config.toml, then the user config directory)
  -l, --log-level <LOG_LEVEL>  Log level (debug, info, warn, error) [default: info]
      --no-gpu                 Disable GPU monitoring
  -h, --help                   Print help
  -V, --version                Print version
```

## KEYS

- `q`: quit.
- `Ctrl+C`: quit. Raw mode swallows the signal, so it is handled as a key.

## CONFIGURATION

Without `-c`, the lookup order is `./config.toml` and then a per user file:
`$XDG_CONFIG_HOME/r-heatmap/config.toml` on Linux, falling back to
`~/.config/r-heatmap/config.toml`, and `%APPDATA%\r-heatmap\config.toml` on Windows.
A file that is missing, unreadable or invalid falls back to the built-in defaults and
logs why, so a broken config never takes the program down.

### Fields

- `general.refresh_rate_ms`: Update frequency in milliseconds. Values below 10 are clamped.
- `general.github_repo`: Text shown on the right side of the status bar.
- `style.border_color`: Primary UI border color.
- `style.header_color`: Information text color.
- `style.sensor_label_contains`: Filter list for sensor labels. `computer` is in the
  default list to catch the single sensor Windows reports; it matches nothing on Linux.
- `style.disk_label_contains`: Filter list for disk hwmon labels. Optional, defaults to `["nvme", "drivetemp"]`. The disk row is hidden when nothing matches.
- `thresholds`: Temperature boundaries for heatmap color mapping.

## TEMPERATURES

CPU and disk temperatures come from `sysinfo`, which enumerates every `hwmon` chip.
Both rows are driven by substring filters over the sensor label, and the label is
composed by `sysinfo` as chip name, channel label and device model — an NVMe drive
shows up as something like `nvme Composite KINGSTON SNV2S1000G`, which is why
filtering by `"nvme"` is enough.

On Windows `sysinfo` has no `hwmon` to walk and falls back to the ACPI thermal zone over
WMI, which reports one sensor for the whole machine, labeled `Computer`. `computer` is in
the default filter for that reason. Most desktop firmware does not implement the class,
so the row is usually empty there — see WINDOWS.

## DISKS

The disk row takes whatever the sensor filters left behind on Linux and whatever
`DiskMonitor` found on Windows, and renders both through the same grid. Only one of the
two ever has anything in it. Tiles are colored against `thresholds`, like the CPU row,
rather than against the warning point the drive publishes, so both rows read the same
way.

## FANS

Fan speeds are read straight from `fanN_input` under `/sys/class/hwmon/`, so there is
nothing to configure: every fan the kernel exposes gets a tile, and the row disappears
when no chip reports one. Tiles are colored against `fanN_max` when the chip publishes
it, and a fan sitting at 0 RPM is always red. Some machines expose the same physical
fan through two chips (`dell_ddv` and `dell_smm`, for instance) and will show
duplicated tiles. Linux only; see WINDOWS for why.

## VOLTAGES

Voltages come from `inN_input` under `/sys/class/hwmon/`, converted from millivolts.
Like fans, they need no configuration and the row is hidden when no chip reports one.
A rail is green while it stays inside the `inN_min`/`inN_max` window published by the
chip and red once it leaves it; chips that publish no window are drawn blue, since
there is no way to tell a healthy 12V rail from a sagging one without it. Desktop
super-I/O chips expose a lot of rails, laptops usually expose only the battery. Linux
only; see WINDOWS for why.

## GPUS

Every `cardN` under `/sys/class/drm/` that exposes `device/gpu_busy_percent` gets its own
tile, so a hybrid laptop or a multi card box shows all of them instead of whichever came
first. Cards are labeled by node and driver (`card1 amdgpu`), read from `device/uevent`.
Connector entries such as `card0-eDP-1` are skipped, and so are cards without the counter:
`gpu_busy_percent` is an amdgpu interface, so Intel and NVIDIA cards usually report
nothing here and the row stays hidden. Windows has its own source, described below, with
no such vendor limitation. `--no-gpu` skips the scan on both.

## WINDOWS

Each row is a monitor with two implementations picked by `cfg`, so `SystemStats` calls
the same four scans on both systems and the UI never learns which one it got.

Disk temperatures come from `IOCTL_STORAGE_QUERY_PROPERTY` with
`StorageDeviceTemperatureProperty` against `\\.\PhysicalDriveN`. The handle is opened
with no access rights at all, which is what keeps this working without elevation:
the IOCTL is `FILE_ANY_ACCESS`, so it does not need read access to the drive. Drives are
probed from 0 upwards and named from the device descriptor, giving
`disk0 ADATA LEGEND 960`. A drive that reports more than one sensor gets one tile per
channel. NVMe and SATA both answer.

GPU usage comes from `\GPU Engine(*)\Utilization Percentage` through PDH. Instances are
per process and per engine (`pid_9728_luid_0x00000000_0x0000EC9F_phys_0_eng_0_engtype_3D`),
so the values are summed per engine and the busiest engine wins the adapter — the same
number Task Manager puts in its GPU column. The query is opened once and kept, since a
rate counter needs two collections to produce anything.

Adapter names come from `HKLM\SOFTWARE\Microsoft\DirectX`, where each subkey carries
`AdapterLuid`, `Description` and `AdapterType`, and the LUID lines up with the one in the
counter instance once the case is normalized: the registry is written in lowercase and
PDH hands out uppercase. Bit 2 of `AdapterType` marks a software device, which is how
`Microsoft Basic Render Driver` gets dropped instead of showing up as a third idle GPU.
When the key cannot be read the adapters are kept and labeled `GPU 0`, `GPU 1`.

CPU temperature, fans and voltages have no user mode source. On Linux `k10temp` and
`nct6775` do the talking — one reads the CPU's system management network, the other does
port I/O against the super-I/O chip — and both are ring 0 operations. Windows ships no
such driver, so the tools that display these numbers install their own. The only thing
Windows offers on its own is the ACPI thermal zone, read by `sysinfo` over WMI as a
single `Computer` sensor, and most desktop firmware does not implement it.

The console output code page is set to UTF-8 before the alternate screen is entered. A
console left at the OEM default (850 here, 437 on a US install) decodes the box drawing
characters and the degree sign as Latin-1 garbage, and neither crossterm nor ratatui
touch the code page.

On the Linux side, `sysfs::class_dir` resolves a `/sys/class` directory and returns
`None` when it is not there, so a kernel without `hwmon` or `drm` produces an empty scan
rather than a warning about a path that was never going to exist.

## ARCHITECTURE

- **System**: `reading` holds the primitive every row is built from: a label, a value, an
  optional window and where it came from. A reading backed by a `sysfs` file rereads that
  one file on refresh; one produced in a batch — anything on Windows — is refreshed by its
  monitor handing back a fresh set that `reading::update` merges by label, which keeps
  tiles from reordering under the cursor. `disk`, `fan`, `volt` and `gpu` are one monitor
  each, with a `scan_platform`/`refresh_platform` pair per system. Under them, `hwmon` and
  `drm` read `/sys`, and `windows::storage`, `windows::perf` and `windows::adapters` do
  the equivalent through Win32. Adding a channel type is still a matter of scanning a new
  prefix.
- **UI**: Terminal interface built with `ratatui`. Rows are laid out top to bottom
  (header, temperatures, disks, fans, voltages, GPUs, cores) and a row with no readings
  is given zero height. Disks, fans, voltages and GPUs share one grid renderer,
  parameterized by decimals, unit and color function. Tiles inside a row are laid out with
  `Fill`, so the leftover lines are spread instead of piling onto one row and leaving the
  others too short to print their value.
- **Events**: Crossterm-based event loop. Input is polled with whatever is left of the
  current tick, so keys respond immediately regardless of `refresh_rate_ms`.

## LOGGING

Logs are written to `r-heatmap.log` in the working directory and never to stdout, which
is occupied by the alternate screen. The file is appended across runs.

## TROUBLESHOOTING

Check `r-heatmap.log` for initialization errors or sensor detection failures. Ensure the
user has read permissions for entries in `/sys/class/hwmon/`. Startup logs how many
channels each monitor found, which is the quickest way to tell a missing kernel module
from a wrong filter, and `-l debug` lists them one per line with the value they started
at. On Windows the fan and voltage counts are always zero and say nothing about a
problem.

An empty temperature row on Windows is worth checking outside the program first:

```powershell
Get-CimInstance -Namespace root/WMI -ClassName MSAcpi_ThermalZoneTemperature
```

If that fails from an elevated prompt too, the firmware does not expose the class and
there is nothing to read.
