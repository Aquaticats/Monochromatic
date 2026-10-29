# Claude CPU cap and degradation watch

This runbook records how Claude sessions on the 16 core, 32 thread host are limited and watched,
and how to tone the limit down or up.
The user decided on 2026-10-13: the whole account may use all 32 threads,
and Claude tool commands (including gate jobs run through `gate-slot.ts`) share one cap,
currently 30 threads (lowered to 28 by the tone-down rule, then raised back by the user).

## What enforces the limits

Whole account: `AllowedCPUs=0-31` on `user.slice` and `user-1000.slice`,
persisted by `systemctl set-property` into `/etc/systemd/system.control`.
Before 2026-10-13 a drop-in from 2026-02-22 held `0-15`, which left the SMT siblings (threads 16 to 31) idle.

Claude tool commands: the `claude-code-bash` cgroup under
`/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/`.
Claude Code creates it anew in each boot (it exists because `CLAUDE_CODE_TOOL_MEMORY_LIMIT` is set)
and has no CPU limit setting, so a user service re-applies `cpu.max` to it.

The service is `claude-cpu-watch.service` (enabled, linger on); its unit file in `~/.config/systemd/user/`
is a symlink to
`package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service`,
which runs `package/claude-cpu-cap/daemon/src/cpu-watch.ts`.
The package README has the install commands.
It reads the thread count from `~/.config/claude-cpu-cap/threads` every 2 seconds
and writes `<threads * 100000> 100000` into the cgroup's `cpu.max` when the value differs.

## Tone the cap down or up

Write the new thread count to the file; the service applies it within seconds:

```bash
# ~/.config/claude-cpu-cap/threads holds one integer from 1 to 32.
echo 28 > ~/.config/claude-cpu-cap/threads
```

Tone down 2 threads at a time while the user explores the CPU, as they asked.
`nproc` inside a new shell reports the cap.

## What the watch reports

The service logs a line only for an event, never for routine samples.
Every line goes through the tagged logger, so it starts with `[level] [timestamp] [tag]` before the event name;
alerts are `warn`, recoveries and starts are `info`.
Read them with `journalctl --user --unit=claude-cpu-watch --follow --lines=0 --output=cat`.

- `ALERT <metric>=<value>`: the metric stayed at or above its limit for 3 samples (6 seconds).
- `CLEAR <metric>`: it stayed below for 5 samples.
- `CAP-APPLIED`: the quota was rewritten (the cgroup is new, or the file changed).
- `WATCHER-STOPPED` or a systemd stop line: nothing is watching until the unit restarts.

Limits: CPU pressure `some avg10` 40, memory pressure `some avg10` 10,
a thermal throttle shortfall of 300 MHz (the user chose a throttling alert on 2026-10-13, then asked for a better value:
Tctl at or above 95 degrees Celsius AND the average clock at least 300 MHz below the boost-off clock of 4490 MHz;
Zen 4 sits at 95 degrees by design, and 125 samples at 95 degrees showed a shortfall of at most 255 MHz),
scheduler lag 250 ms.
IO pressure is not watched on purpose: the user's drives are slow, and the user decided on 2026-10-13
that IO pressure is non-actionable.

## Sample log for judging temperatures

Every 10 seconds the service appends one CSV line to
`~/.local/share/claude-cpu-cap/samples/<local date>.csv` and keeps 14 days.
The columns are UTC time, Tctl, Tccd1, Tccd2, package watts, average clock, CPU pressure, one-minute load and the cap,
then seven columns that held the Corsair fan RPMs and the Corsair temperature reading (`fan1_rpm` to
`corsair_probe_c`).
Those seven are empty in rows from 2026-10-18 on: the user moved every fan from the Corsair controller to motherboard
headers, and the watcher no longer reads CoolerControl or its access token.
In rows from 2026-10-13 to 2026-10-17 the last of them is the controller's temperature probe,
which the user attached to the top of the air cooler (CoolerControl labels it `temp1`),
so it was expected to follow the air leaving the heatsink.
In the first 33 minutes of data it stayed between 25.5 and 26.2 degrees from 42 to 149 W,
so it behaves like a room-temperature reading; the cause is not found (see the handover).
The remaining columns judge how well the CPU hands heat to the cooler:
package watts split between the first chiplet (cores 0 to 7) and the second (cores 8 to 15),
the six motherboard readings of the `gigabyte_wmi` driver, the ACPI zone, the NVMe temperature, a `burst` flag,
and, from 20:55 EDT on 2026-10-13, the discrete graphics card and the integrated graphics (temperature and watts each;
older rows of that day lack these four columns).
From 2026-10-18 the last eleven columns hold the motherboard's `nct6687` readings:
the eight fan speeds (`cpu_fan_rpm`, `pump_fan_rpm`, `sys_fan1_rpm` to `sys_fan6_rpm`, named after the chip's labels;
a header with no fan plugged in reads 0) and the System, VRM MOS and PCH temperatures
(`mb_system_c`, `mb_vrm_c`, `mb_pch_c`).
They come last in the header, so every older row keeps its column positions.
No source names what the six motherboard readings measure on the B650 GAMING X AX (the driver reads six numbered
firmware channels), so they are logged raw;
the third one (`board3_c`) follows the CPU temperature, the fifth moves by about 3 degrees with load,
and the rest stay within 2 degrees.
The ACPI zone is constant at 16.8 degrees.
The six motherboard readings are empty from 2026-10-18, because the new MSI board has no `gigabyte_wmi` driver.
The core split assumes the first eight cores form the first chiplet; 251 of 253 clear rows agree with it
(the chiplet drawing more watts is the one with the hotter CCD).
While package power steps by more than 25 watts between two 2-second passes, the service logs every pass for 90 seconds
(`burst` is 1), because the first seconds after a load step show the die-to-heatsink path best.
Alert lines carry package watts and the average clock, but not fan speed; read fan speeds from the sample rows.

Read the settled temperature per power bucket with
`node package/claude-cpu-cap/daemon/src/summarize-samples.ts <days>`.
A sample counts as settled when the previous 2 minutes stayed in the same 20 watt bucket,
so alert-time spikes do not hide the steady state.
No stress test is run for this; it only records what the machine does under real work (user decision, 2026-10-13).

## Known gaps

- Units started with `systemd-run --user` (`ide7-gate*`, `tr-gate-*`) get their own cgroup in `app.slice`
  and are outside the cap.
- The cap is shared by all sessions, not per session.
- Processes started under the old `0-15` limit kept a narrowed affinity; a reboot clears that.
