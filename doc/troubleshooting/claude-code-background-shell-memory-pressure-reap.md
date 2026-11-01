# Claude Code reaps idle background shells on memory-pressure stalls

## Symptom

A background Bash task that only waits
(`while kill -0 <pid>; do sleep 30; done; ...`)
is stopped with
"stopped because the system is running low on memory",
while `free` reports 20 GB or more available.
Seen four times on 2026-09-17 (02:15 to 03:33 UTC) on the 64 GB machine running the translation-repair passes,
each time within minutes of arming the waiter.

## Cause

Claude Code 2.1.274 runs on Bun,
which installs a memory-pressure watcher backed by a PSI trigger on `/proc/pressure/memory`
(the bundle names `memoryPressurePsiTrigger` and `memoryPressureWatcherHasOsBackend`).
The harness listens for the runtime's `memoryPressure` event and,
when the session has been idle for 10 minutes (`uds=600000` in the bundle),
reaps running local shell tasks under the telemetry name `task_local_shell_pressure_reap`.
The kernel side was real but small:
the 16 GiB zram swap was full (15.4 GiB stored in 5.7 GiB),
so every further reclaim went to the 48 GiB disk swap file,
and `/proc/pressure/memory` showed short stalls
(`some avg10` up to about 2 percent) during those bursts.
The watcher treats any stall as critical.

## Fix

Start Claude Code with the documented switch in its environment:

```sh
CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP=1
```

The bundle's own help text says setting it from a shell command inside the session has no effect,
so it needs a restart.
Until then a `CronCreate` job every 25 minutes is the wake that survives,
since cron jobs are not shell tasks.

Reducing the stalls themselves:
the zram was widened from `min(ram / 2, 16384)` to `min(ram / 2, 32768)` in `/etc/systemd/zram-generator.conf`
(zram-generator 1.2.1 on Fedora 44),
so at the measured 2.7 to 1 ratio 32 GiB of pages live in about 12 GiB of RAM instead of on disk.

## Resizing a live zram device

The setup unit's `ExecStop` resets the device,
so `systemctl restart systemd-zram-setup@zram0.service` after `swapoff /dev/zram0`
recreates it at the new size.
Two traps met on the way:

- Starting `dev-zram0.swap` afterwards re-runs the setup on the already initialised device,
  which fails with
  `Failed to configure compression algorithm into /sys/block/zram0/comp_algorithm`
  and `Device or resource busy`.
  The restart of the setup unit alone is enough;
  the swap unit comes up as its dependent.
- `zramctl --reset /dev/zram0` on this kernel (7.2.0) removes the device node entirely,
  after which the setup unit fails on its `dev-zram0.device` dependency.
  Recreate it with `cat /sys/class/zram-control/hot_add` (as root),
  then `systemctl reset-failed` both units and `systemctl start dev-zram0.swap`.

Verified 2026-09-17 03:48 UTC:
`swapon --show` lists `/dev/zram0` at 31.2 GiB with priority 100 above the disk file at priority 10.

## Not reviewed

The reap logic and its threshold were read out of the Claude Code bundle's minified strings,
not from published documentation;
the env switch's name and its startup-only scope come from the bundle's own help text.
