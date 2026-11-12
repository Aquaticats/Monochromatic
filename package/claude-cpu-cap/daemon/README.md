# @monochromatic-dev/claude-cpu-cap-daemon

Caps the CPU use of Claude Code tool commands,
lowers their disk I/O weight so the desktop wins under contention,
and records CPU temperature,
power and fan samples on this host.
The operating notes (what the limits mean,
how to tone the cap down or up) are in
`doc/runbook/claude-cpu-cap.md`;
the evidence behind the settings is in
`doc/handover/claude-cpu-cap-and-thermal-watch.md`.

## What is in the package

- `src/cpu-watch.ts`:
  the watcher entry point,
  run as the user unit `claude-cpu-watch.service`.
- `src/run-pass.ts`:
  one watcher pass (cap,
  power,
  pressure,
  thermal,
  burst mode,
  sample row).
- `src/cap.ts`,
  `src/power.ts`,
  `src/pressure.ts`,
  `src/hwmon.ts`,
  `src/files.ts`,
  `src/samples.ts`,
  `src/streak.ts`:
  the pieces a pass uses.
- `src/io-weight.ts`:
  re-applies the cgroup's default IO weight (10,
  against the kernel default of 100) every pass,
  into `io.bfq.weight` and `io.weight` (see the "IO weight" section).
  A weight is a share against sibling cgroups and costs nothing on an idle disk.
- `src/io-weight-error.ts`:
  the error `src/io-weight.ts` throws when the cgroup has no weight file or a file refuses.
- `src/paths.ts`:
  every location and limit,
  each overridable by environment variable.
- `src/summarize-samples.ts`:
  Tctl per package-power bucket from the sample log.
- `src/analyze-episode.ts`:
  medians,
  minima and maxima of every sample column between two UTC times.
- `systemd/claude-cpu-watch.service`:
  the unit;
  `ExecStart` runs `src/cpu-watch.ts` from this checkout,
  after an `ExecStartPre` that checks the repository link.

## Overrides

`CPU_WATCH_LIMIT_CPU_SOME`,
`_MEMORY_SOME`,
`_THROTTLE_MHZ`,
`_LAG`,
`_TEMP`,
`_BOOST_OFF_MHZ` and `_BURST_STEP_W`
(all prefixed `CPU_WATCH_LIMIT_`) change the limits.
`CPU_WATCH_CAP_FILE`,
`_CGROUP_FILE`,
`_IO_WEIGHT_FILE`,
`_IO_BFQ_WEIGHT_FILE`,
`_HWMON_ROOT`,
`_PROC_ROOT`,
`_SAMPLE_DIR` point the watcher at other locations,
which is how the tests run it on a disposable host.
A limit of 0,
for example `CPU_WATCH_LIMIT_CPU_SOME=0`,
is the positive control that makes an alert fire.

## IO weight

The `claude-code-bash` cgroup has two weight files,
and each is read by a different part of the kernel.
The watcher writes the same default weight into both,
every pass,
and clamps it to each file's range.

### Which file each scheduler reads

- `io.bfq.weight` is read by the BFQ scheduler.
  The range is 1 to 1000 and the kernel default is 100
  (`Documentation/block/bfq-iosched.rst` and `block/bfq-iosched.h`,
  `BFQ_MIN_WEIGHT` and `BFQ_MAX_WEIGHT`,
  Linux v7.2.11).
  The file exists once BFQ is built in or loaded and the kernel has `CONFIG_BFQ_GROUP_IOSCHED`
  (`bfq_init` in `block/bfq-iosched.c` registers the cgroup policy),
  whichever scheduler a disk uses,
  but a disk obeys it only while BFQ is that disk's scheduler.
  A write outside the range fails with `ERANGE` (`block/bfq-cgroup.c`).
- `io.weight` is read by the iocost controller,
  not by BFQ.
  The range is 1 to 10000 and the kernel default is 100
  (`Documentation/admin-guide/cgroup-v2.rst` and `block/blk-iocost.c`,
  `CGROUP_WEIGHT_MIN` and `CGROUP_WEIGHT_MAX`,
  Linux v7.2.11).
  iocost is off until `io.cost.qos` sets `enable=1` for a disk,
  so on a host that never configured it
  (empty `/sys/fs/cgroup/io.cost.qos` and `io.cost.model`) `io.weight` changes nothing.
  A write outside the range fails with `EINVAL`.
- `none`,
  `mq-deadline` and `kyber` read neither file:
  their sources (`block/kyber-iosched.c`,
  `block/mq-deadline.c`,
  Linux v7.2.11) contain no cgroup code.
  iocost is the exception that does not depend on the scheduler:
  `block/blk-iocost.c` mentions no scheduler,
  and `io.cost.qos` turns it on for one disk at a time.
- systemd's `IOWeight=` sets both files,
  which is why a unit with `IOWeight=10` showed the weight in `io.bfq.weight`.

The introduction of the IO section in `cgroup-v2.rst` still says weights need `cfq-iosched` and are unavailable for
blk-mq devices.
That paragraph is stale next to the file descriptions,
so this package follows the file descriptions and the C sources.

### What the watcher does per pass

- Both files exist:
  each is read,
  and rewritten with `default <weight>` when its default line differs.
  `IO-WEIGHT-APPLIED claude-code-bash <file> default <old> -> <new>` is logged per rewrite.
- Only one file exists (a kernel without BFQ cgroup support,
  or one without iocost):
  that file is written,
  and the
  absent one is skipped without a log line.
- The wanted weight is outside a file's range:
  it is moved to the nearest bound,
  and `IO-WEIGHT-CLAMPED` is logged at
  `warn` when the file is rewritten.
  The clamped value counts as correct afterwards,
  so the line is not repeated while the file holds it.
- The cgroup directory does not exist yet (Claude Code creates it in every boot):
  nothing happens (a `debug` line).
- The cgroup directory exists but holds neither file (its io controller is off):
  `IO-WEIGHT-UNAVAILABLE` is logged at
  `warn`,
  naming both paths and where to look.
- A file cannot be read or written:
  the other file is still tried,
  and `IO-WEIGHT-FAILED` is logged at `error`,
  naming
  the file and the system error.
- The two failure lines repeat on every pass (every 2 seconds) while the condition lasts,
  like `CAP-FILE-INVALID`.
  A weight failure never stops the CPU cap,
  the alerts or the sample row of the same pass.

### The measurement behind this (2026-11-01, this host)

- The watcher used to write only `io.weight`.
  The live cgroup then showed `io.weight` `default 10` and `io.bfq.weight` `default 100`.
- `/var/home` is on LUKS over `nvme0n1p6`,
  and `/sys/block/nvme0n1/queue/scheduler` reads
  `none mq-deadline kyber [bfq]`,
  so the disk obeys `io.bfq.weight` only.
  `/sys/fs/cgroup/io.cost.qos` and `io.cost.model` are empty,
  so iocost is not configured.
  The weight the watcher wrote therefore had no effect on that disk.
- Positive control:
  `systemd-run --user --scope --property=IOWeight=10` followed by a read of the scope's
  `io.bfq.weight` printed `default 10`,
  so a write to `io.bfq.weight` is the one that takes effect.

### Disks this does not cover

`sda` uses `kyber` (`none mq-deadline [kyber] bfq`),
which reads neither weight file,
so with iocost off
no cgroup weight limits I/O to that disk.
Two changes would make a weight work there:
switching its scheduler to `bfq` (a udev rule like
`/etc/udev/rules.d/61-nvme-bfq.rules`),
or enabling iocost for it with `enable=1` in `io.cost.qos`.
Both need root,
and both change how every cgroup shares the disk,
so they are out of scope for this package.

## Install

The unit file in `~/.config/systemd/user/` is a regular copy of the file in `systemd/`,
not a link into the repository:
the user manager reads it while loading units,
and a link into a repository that lives on the 860 drive would leave the
unit unloaded whenever the drive is not mounted at manager start or at a daemon reload.
The copy's `ExecStartPre` runs the repository link check installed outside the repository
(`~/.local/share/monochromatic-dev/repository-link/check.mjs`),
so a start before the mount fails with the plain
message and status 75,
and `Restart=on-failure` with `RestartSec=30` and no start limit retries until the mount is
there.
Copy again after every edit of the file in `systemd/`:

```bash
# systemd/claude-cpu-watch.service is the single source of the unit; the installed file is a copy of it.
cp "${HOME}/Monochromatic/package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service" \
  "${HOME}/.config/systemd/user/claude-cpu-watch.service.new"
mv --no-target-directory "${HOME}/.config/systemd/user/claude-cpu-watch.service.new" \
  "${HOME}/.config/systemd/user/claude-cpu-watch.service"
systemctl --user daemon-reload
systemctl --user restart claude-cpu-watch.service
```

`mise run //package/claude-cpu-cap/daemon:check:installed-unit` compares the installed file with the one in `systemd/`
and fails with the difference.

## State that stays on the host

These files hold machine state,
not source,
so they are not in the repo:

- `~/.config/claude-cpu-cap/threads`:
  the current thread cap,
  one integer from 1 to 32.
- `~/.local/share/claude-cpu-cap/samples/`:
  the CSV sample log,
  14 days kept.
