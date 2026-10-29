# @monochromatic-dev/claude-cpu-cap-daemon

Caps the CPU use of Claude Code tool commands and records CPU temperature, power and fan samples on this host.
The operating notes (what the limits mean, how to tone the cap down or up) are in
`doc/runbook/claude-cpu-cap.md`; the evidence behind the settings is in
`doc/handover/claude-cpu-cap-and-thermal-watch.md`.

## What is in the package

- `src/cpu-watch.ts`: the watcher entry point, run as the user unit `claude-cpu-watch.service`.
- `src/run-pass.ts`: one watcher pass (cap, power, pressure, thermal, burst mode, sample row).
- `src/cap.ts`, `src/power.ts`, `src/pressure.ts`, `src/hwmon.ts`, `src/files.ts`, `src/samples.ts`,
  `src/streak.ts`: the pieces a pass uses.
- `src/paths.ts`: every location and limit, each overridable by environment variable.
- `src/summarize-samples.ts`: Tctl per package-power bucket from the sample log.
- `src/analyze-episode.ts`: medians, minima and maxima of every sample column between two UTC times.
- `systemd/claude-cpu-watch.service`: the unit; `ExecStart` runs `src/cpu-watch.ts` from this checkout.

## Overrides

`CPU_WATCH_LIMIT_CPU_SOME`, `_MEMORY_SOME`, `_THROTTLE_MHZ`, `_LAG`, `_TEMP`, `_BOOST_OFF_MHZ` and `_BURST_STEP_W`
(all prefixed `CPU_WATCH_LIMIT_`) change the limits.
`CPU_WATCH_CAP_FILE`, `_CGROUP_FILE`, `_HWMON_ROOT`, `_PROC_ROOT`, `_SAMPLE_DIR`
point the watcher at other locations, which is how the tests run it on a disposable host.
A limit of 0, for example `CPU_WATCH_LIMIT_CPU_SOME=0`, is the positive control that makes an alert fire.

## Install

The unit file in `~/.config/systemd/user/` is a symlink to the file in `systemd/`, so edits here reach the host
after a daemon reload and a restart:

```bash
# systemd/claude-cpu-watch.service is the single source of the unit.
ln --symbolic --force "${HOME}/Monochromatic/package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service" \
  "${HOME}/.config/systemd/user/claude-cpu-watch.service"
systemctl --user daemon-reload
systemctl --user restart claude-cpu-watch.service
```

## State that stays on the host

These files hold machine state, not source, so they are not in the repo:

- `~/.config/claude-cpu-cap/threads`: the current thread cap, one integer from 1 to 32.
- `~/.local/share/claude-cpu-cap/samples/`: the CSV sample log, 14 days kept.
