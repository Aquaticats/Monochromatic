# Claude CPU cap and degradation watch

This runbook records how Claude sessions on the 16 core,
32 thread host are limited and watched,
and how to tone the limit down or up.
The user decided on 2026-10-13:
the whole account may use all 32 threads,
and Claude tool commands (including gate jobs run through `gate-slot.ts`) share one cap,
currently 30 threads (lowered to 28 by the tone-down rule,
then raised back by the user).

## What enforces the limits

Whole account:
`AllowedCPUs=0-31` on `user.slice` and `user-1000.slice`,
persisted by `systemctl set-property` into `/etc/systemd/system.control`.
Before 2026-10-13 a drop-in from 2026-02-22 held `0-15`,
which left the SMT siblings (threads 16 to 31) idle.

Claude tool commands:
the `claude-code-bash` cgroup under
`/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/`.
Claude Code creates it anew in each boot (it exists because `CLAUDE_CODE_TOOL_MEMORY_LIMIT` is set)
and has no CPU limit setting,
so a user service re-applies `cpu.max` to it.

The service is `claude-cpu-watch.service` (enabled,
linger on);
its unit file in `~/.config/systemd/user/`
is a symlink to
`package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service`,
which runs `package/claude-cpu-cap/daemon/src/cpu-watch.ts`.
The package README has the install commands.
It reads the thread count from `~/.config/claude-cpu-cap/threads` every 2 seconds
and writes `<threads * 100000> 100000` into the cgroup's `cpu.max` when the value differs.

## Disk I/O weight

The same service writes `default 10` into the cgroup's `io.bfq.weight` and `io.weight` (the kernel default is 100 in
both) whenever either differs.
The weight is a share against sibling cgroups,
so it costs nothing on an idle disk and only matters while the NVMe is
contended.
`io.bfq.weight` is the file the BFQ scheduler reads,
and `/etc/udev/rules.d/61-nvme-bfq.rules` sets BFQ on `nvme0n1`;
`io.weight` is read only by the iocost controller,
which is not configured here,
so before 2026-11-01 (when the
service wrote `io.weight` alone) the weight had no effect on the NVMe.
Under `kyber` (the scheduler of `sda`) both files are accepted but ignored.
The package README (`package/claude-cpu-cap/daemon/README.md`,
"IO weight") has the kernel sources,
the ranges
and the log lines (`IO-WEIGHT-APPLIED`,
`IO-WEIGHT-CLAMPED`,
`IO-WEIGHT-UNAVAILABLE`,
`IO-WEIGHT-FAILED`).
To check the live cgroup,
read both files;
each starts with `default 10` once the service has run:

```bash
# claude-code-bash cgroup of user 1000.
cd /sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash && cat io.bfq.weight io.weight
```
Terminal scopes get the same weight from
`~/.config/systemd/user/app-ghostty-surface-transient-.scope.d/10-io-weight.conf` (`IOWeight=10`),
which covers other
agent harnesses,
builds and commits run from a terminal.
Since the interim OdyTTY switch on 2026-11-02,
the host also has `app-odytty@.service.d/10-io-weight.conf` and `app-odytty-.scope.d/10-io-weight.conf`
under `~/.config/systemd/user/`.
The former sets `IOWeight=10` in `[Service]` for desktop-entry launches;
the latter sets it in `[Scope]` for AppImage scope launches.
Disposable service and scope launches inherited `10`,
and the running OdyTTY service's `io.weight` and `io.bfq.weight` both read `default 10` after the live update.
The Ghostty drop-in remains available for explicit Ghostty launches.

### Installed terminal coverage at 2026-11-03T01:00:12-05:00

The user requested the existing Ghostty `IOWeight=10` policy for all terminals.
The host's installed `TerminalEmulator` desktop entries now have persistent drop-ins
under `~/.config/systemd/user/`:

- Ghostty:
  `com.mitchellh.ghostty`.
- OdyTTY:
  `odytty`.
- Konsole:
  `org.kde.konsole`.
- Kitty:
  `kitty` and its URL launcher `kitty-open`.
- Rio:
  `rio`.
- Contour:
  `org.contourterminal.Contour`.
- Monochromatic Terminal:
  `monochromatic.terminal`.
- The Konsole-based VSCodium launcher:
  `vscodium-pod-shell`.

For each desktop ID after `systemd-escape`,
`app-<id>@.service.d/10-io-weight.conf` and `app-<id>-.service.d/10-io-weight.conf`
set `IOWeight=10` in `[Service]`;
`app-<id>-.scope.d/10-io-weight.conf` sets it in `[Scope]`.
The existing Ghostty surface-scope drop-in remains in place.
The change added 25 files,
retained the existing OdyTTY files,
and reloaded the user manager without restarting terminals.
Only OdyTTY was running during the process inventory;
its unit property and both kernel weight files already reported `10`.

Verification used disposable units in `app.slice` with systemd `262.1-1.fc45`.
Before installation,
Rio template-service,
dash-service,
and scope fixtures each reported `default 100` in both weight files.
After installation,
all 28 fixtures across the desktop entries and Ghostty's surface prefix reported `default 10`.
The fixture commands did not explicitly pass `IOWeight`,
so these checks exercised drop-in inheritance.
Scope processes moved by the existing network-exemption service retained both weights at `10`.
No fixture units remained after verification.
The discarded initial fixture omitted `--slice=app.slice` and read an unavailable `io.weight` file
in its automatically created template slice;
the corrected fixture matched the desktop launch layout.
This verifies configuration,
not a disk-performance improvement or each emulator's graphical startup.

Coverage follows the installed desktop IDs.
Future installations with different IDs need corresponding drop-ins.
Direct process launches inherit their parent's cgroup unless a launcher creates another unit;
these drop-ins do not match arbitrary executable names or separately created job units.
The host files remain local configuration,
like the original Ghostty policy.
The installation manifest and disposable probe scripts are in `${HOME}/temp/agent/terminal-io-weight*`.

### Policy evidence and reversal

Evidence and measurements:
`doc/planning/agent-search-io-throttle.md`.
To undo:
delete the applicable terminal drop-ins and run `systemctl --user daemon-reload`,
and set `DEFAULT_IO_WEIGHT` in
`package/claude-cpu-cap/daemon/src/constants.ts` to 100.

## Tone the cap down or up

Write the new thread count to the file;
the service applies it within seconds:

```bash
# ~/.config/claude-cpu-cap/threads holds one integer from 1 to 32.
echo 28 > ~/.config/claude-cpu-cap/threads
```

Tone down 2 threads at a time while the user explores the CPU,
as they asked.
`nproc` inside a new shell reports the cap.

## What the watch reports

The service logs a line only for an event,
never for routine samples.
Every line goes through the tagged logger,
so it starts with `[level] [timestamp] [tag]` before the event name;
alerts are `warn`,
recoveries and starts are `info`.
Read them with `journalctl --user --unit=claude-cpu-watch --follow --lines=0 --output=cat`.

- `ALERT <metric>=<value>`:
  the metric stayed at or above its limit for 3 samples (6 seconds).
- `CLEAR <metric>`:
  it stayed below for 5 samples.
- `CAP-APPLIED`:
  the quota was rewritten (the cgroup is new,
  or the file changed).
- `WATCHER-STOPPED` or a systemd stop line:
  nothing is watching until the unit restarts.

Limits:
CPU pressure `some avg10` 40,
memory pressure `some avg10` 10,
a thermal throttle shortfall of 300 MHz (the user chose a throttling alert on 2026-10-13,
then asked for a better value:
Tctl at or above 95 degrees Celsius AND the average clock at least 300 MHz below the boost-off clock of 4490 MHz;
Zen 4 sits at 95 degrees by design,
and 125 samples at 95 degrees showed a shortfall of at most 255 MHz),
scheduler lag 250 ms.
IO pressure is not watched on purpose:
the user's drives are slow,
and the user decided on 2026-10-13
that IO pressure is non-actionable.

## Sample log for judging temperatures

Every 10 seconds the service appends one CSV line to
`~/.local/share/claude-cpu-cap/samples/<local date>.csv` and keeps 14 days.
The columns are UTC time,
Tctl,
Tccd1,
Tccd2,
package watts,
average clock,
CPU pressure,
one-minute load and the cap,
then seven columns that held the Corsair fan RPMs and the Corsair temperature reading (`fan1_rpm` to
`corsair_probe_c`).
Those seven are empty in rows from 2026-10-18 on:
the user moved every fan from the Corsair controller to motherboard
headers,
and the watcher no longer reads CoolerControl or its access token.
In rows from 2026-10-13 to 2026-10-17 the last of them is the controller's temperature probe,
which the user attached to the top of the air cooler (CoolerControl labels it `temp1`),
so it was expected to follow the air leaving the heatsink.
In the first 33 minutes of data it stayed between 25.5 and 26.2 degrees from 42 to 149 W,
so it behaves like a room-temperature reading;
the cause is not found (see the handover).
The remaining columns judge how well the CPU hands heat to the cooler:
package watts split between the first chiplet (cores 0 to 7) and the second (cores 8 to 15),
the six motherboard readings of the `gigabyte_wmi` driver,
the ACPI zone,
the NVMe temperature,
a `burst` flag,
and,
from 20:55 EDT on 2026-10-13,
the discrete graphics card and the integrated graphics (temperature and watts each;
older rows of that day lack these four columns).
From 2026-10-18 the last eleven columns hold the motherboard's `nct6687` readings:
the eight fan speeds (`cpu_fan_rpm`,
`pump_fan_rpm`,
`sys_fan1_rpm` to `sys_fan6_rpm`,
named after the chip's labels;
a header with no fan plugged in reads 0) and the System,
VRM MOS and PCH temperatures
(`mb_system_c`,
`mb_vrm_c`,
`mb_pch_c`).
They come last in the header,
so every older row keeps its column positions.
No source names what the six motherboard readings measure on the B650 GAMING X AX (the driver reads six numbered
firmware channels),
so they are logged raw;
the third one (`board3_c`) follows the CPU temperature,
the fifth moves by about 3 degrees with load,
and the rest stay within 2 degrees.
The ACPI zone is constant at 16.8 degrees.
The six motherboard readings are empty from 2026-10-18,
because the new MSI board has no `gigabyte_wmi` driver.
The core split assumes the first eight cores form the first chiplet;
251 of 253 clear rows agree with it
(the chiplet drawing more watts is the one with the hotter CCD).
While package power steps by more than 25 watts between two 2-second passes,
the service logs every pass for 90 seconds
(`burst` is 1),
because the first seconds after a load step show the die-to-heatsink path best.
Alert lines carry package watts and the average clock,
but not fan speed;
read fan speeds from the sample rows.

Read the settled temperature per power bucket with
`node package/claude-cpu-cap/daemon/src/summarize-samples.ts <days>`.
A sample counts as settled when the previous 2 minutes stayed in the same 20 watt bucket,
so alert-time spikes do not hide the steady state.
No stress test is run for this;
it only records what the machine does under real work (user decision,
2026-10-13).

## Known gaps

- Units started with `systemd-run --user` (`ide7-gate*`,
  `tr-gate-*`) get their own cgroup in `app.slice`
  and are outside the cap.
- The cap is shared by all sessions,
  not per session.
- Processes started under the old `0-15` limit kept a narrowed affinity;
  a reboot clears that.
