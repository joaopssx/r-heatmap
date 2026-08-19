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
- `style.sensor_label_contains`: Which discovered sensors go in the CPU row. `computer` is
  in the default list to catch the single sensor Windows reports; it matches nothing on
  Linux. Matching nothing at all falls back to showing everything that was found.
- `style.disk_label_contains`: Which discovered sensors go in the disk row. Optional,
  defaults to `["nvme", "drivetemp"]`. The row is hidden when nothing matches.
- `style.board_label_contains`: Which discovered sensors go in the chipset row. Optional,
  defaults to `["pch", "systin", "chipset", "motherboard", "ambient"]`.
- `style.show_other_sensors`: Show a row with the sensors neither filter claimed, off by
  default. On a laptop that is usually the battery, the ambient probe and the wifi card.
- `thresholds`: Temperature boundaries for heatmap color mapping.

## CORES

The core row is one tile per logical CPU, which is where a single threaded process gives
itself away: one tile pinned at 100% while the rest sit idle looks nothing like a build
spreading over every core.

On Linux the numbers come from `/proc/stat`, which counts jiffies spent in each state
since boot rather than a percentage. A usage figure only exists between two readings, so
each tick subtracts the previous snapshot from the current one and divides the busy jiffies
by the total. Startup takes the first snapshot, which is why the row reads 0% for the
first refresh interval and no longer than that. `iowait` is counted as idle: a core waiting
on the disk is not doing work, and folding it into busy would make an I/O bound copy look
like a CPU bound one.

The aggregate `cpu` line is skipped, so is everything below the per core lines. The status
bar keeps taking its global figure from `sysinfo`, which is a different average of the same
thing, so the two can disagree by a fraction of a percent.

Windows builds the row from `sysinfo` instead, one tile per CPU it reports.

## MEMORY

The memory row sits under the header, before the temperatures, because a temperature
spike reads differently when the machine is also out of RAM. Tiles are colored like any
other usage figure in the interface.

RAM is measured against `MemAvailable` rather than `MemFree`: the kernel hands most of
the free pages to the page cache and gives them back on demand, so `MemFree` on a machine
that has been up for a while looks alarming and means nothing. `MemAvailable` is the
kernel's own estimate of what a new allocation could actually take, which is the number
`free` prints under `disponível`. Swap is `SwapTotal` minus `SwapFree`, and the tile is
dropped when the machine has no swap.

Totals go in the label (`RAM 31.0 GB`) instead of the value, which keeps the label stable
between refreshes — `reading::update` matches by label, so a label carrying a live number
would never match itself.

Linux only: `/proc/meminfo` is the source. Windows already reports total and used memory
through `sysinfo` in the status bar, and the row is empty there.

## TEMPERATURES

Every `hwmon` chip is walked at startup and every `tempN_input` under it becomes a
reading, whatever the chip is. Nothing has to be declared: the config never says which
sensors exist, it only decides where the ones that were found are drawn.

Labels are the chip name, then `tempN_label` when the chip publishes one and the channel
name when it does not, then the device model when the chip hangs off a device that has
one. That is how `nvme Composite KINGSTON SNV2S1000G` and `dell_smm temp1` come out of
the same code, and it is the same label `sysinfo` used to compose, so filters written
against the old behaviour still match.

Discovered sensors are split into four rows, in this order: `disk_label_contains` takes
the disk row, `board_label_contains` the chipset row, `sensor_label_contains` the CPU row,
and the remainder goes to the other row, which is hidden unless `show_other_sensors` is
on. The order is the precedence: the narrower lists are checked first because a drive or a
board label can contain a CPU word.

The chipset row is where the board's own probes go — `PCH_CHIP_TEMP` on Intel boards,
`SYSTIN` on the `nct6775` family, `Ambient` on Dell laptops. They are worth keeping out of
the CPU row rather than hiding: on a box with several NVMe drives and a hot GPU the
chipset can be the part that throttles, and it never shows up in a CPU reading. Note that
`CPUTIN` is the socket temperature the board reports and stays in the CPU row, where it
belongs.

The fallback matters more than the filters: if `sensor_label_contains` matches nothing,
it is ignored and every sensor found is drawn. A machine whose CPU chip calls itself
something nobody guessed shows its temperatures anyway, which is the whole point of
scanning instead of declaring. `-l debug` prints every sensor found with its starting
value, which is the fastest way to write a filter for a chip you have never seen.

Chips are visited in sorted order, so the rows do not shuffle between runs, and tiles
inside a row are sorted by label.

On Windows there is no `hwmon` to walk, so the row is built from `sysinfo` instead, which
falls back to the ACPI thermal zone over WMI and reports one sensor for the whole machine,
labeled `Computer`. `computer` is in the default filter for that reason. Most desktop
firmware does not implement the class, so the row is usually empty there — see WINDOWS.

## DISKS

The disk row takes the discovered sensors that matched `disk_label_contains` on Linux and
whatever `DiskMonitor` found on Windows, and renders both through the same grid. Only one
of the two ever has anything in it. Tiles are colored against `thresholds`, like the CPU row,
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

### AMD

`amdgpu` hangs a full hwmon chip off each card at `device/hwmon/hwmonN/`, so temperature
and clocks are read with the same scanner used for the machine's own chips, only rooted
at the card instead of `/sys/class/hwmon`. Channels are named after the card rather than
the chip — `card1 edge`, `card1 junction`, `card1 mem` for temperatures, `card1 sclk` and
`card1 mclk` for clocks — which is what keeps a discrete card apart from the integrated
one on a hybrid laptop, where both answer as `amdgpu`. Temperatures are colored against
`thresholds`, like every other temperature in the interface.

Clocks come from `freqN_input` in Hz and are shown in MHz. hwmon publishes no ceiling for
them, so the top DPM state is read from `device/pp_dpm_sclk` and `pp_dpm_mclk` (the
domain name in the label is the file name) and the tile is colored by how close the card
is running to it. A card that publishes no DPM table is drawn blue, the same way a
voltage rail with no window is.

The GPU fan and GPU voltage rails are not read here: that same chip is also listed in
`/sys/class/hwmon`, so the fan and voltage rows already pick them up as `amdgpu fan1` and
`amdgpu in0`.

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
  each, with a `scan_platform`/`refresh_platform` pair per system, and `memory` and `cpu`
  are two more on top of the usage figures they already fed the status bar, and `temp`
  discovers every temperature channel on the machine in one pass. Under them,
  `hwmon`, `drm`, `meminfo` and `stat` read `/proc` and `/sys`, and `windows::storage`, `windows::perf` and
  `windows::adapters` do
  the equivalent through Win32. Adding a channel type is still a matter of scanning a new
  prefix. `hwmon::scan_chip` takes a single chip directory, so a chip that lives outside
  `/sys/class/hwmon` — an AMD card's, for instance — goes through the same code.
- **UI**: Terminal interface built with `ratatui`. Rows are laid out top to bottom
  (header, memory, CPU temperatures, chipset, disks, other sensors, fans, voltages, GPU
  usage, GPU temperatures, GPU clocks, cores) — cores last because the row grows with the
  core count and a row with no readings
  is given zero height, and a row that has readings is given four lines per grid row it
  needs, so a machine with sixteen core sensors gets a taller row instead of tiles too
  short to print their value. Every row shares one grid renderer, parameterized by
  decimals, unit and color function. Tiles inside a row are laid out with
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
