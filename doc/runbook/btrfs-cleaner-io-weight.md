# Btrfs cleaner and transaction I/O weight

## Host policy at 2026-11-03T02:11:58-05:00

The user requested permanent I/O weight `10` for `btrfs-cleaner`.
The host now places its cleaner kernel threads in the root-level `btrfscleaner.slice`.
Both `io.weight` and `io.bfq.weight` read `default 10`.
The observed schedulers on `nvme0n1` and `sda` are both `bfq`.

The slice contains the cleaners themselves.
Its configuration sets no CPU quota or memory limit on filesystem work.
The helper that assigns threads has separate resource limits.

## Persistent files

These files are local host configuration:

- `/etc/systemd/system/btrfscleaner.slice` sets `IOWeight=10`.
- `/etc/systemd/system/btrfs-cleaner-io-weight.service` runs the assignment helper as root,
  requires the slice,
  and starts after `local-fs.target`.
- `/etc/systemd/system/btrfs-cleaner-io-weight.timer` is enabled in `timers.target.wants`.
  It runs at boot with `OnBootSec=10s`,
  then `OnUnitInactiveSec=30s`,
  with `AccuracySec=1s`.
- `/usr/local/libexec/btrfs-cleaner-io-weight/apply.ts` reconciles the weight files,
  then moves matching kernel threads by writing their PIDs to the slice's `cgroup.procs`.
- `/usr/local/libexec/btrfs-cleaner-io-weight/node` is a root-owned copy of the repository's
  locked Node `26.13.0` binary.
  The service does not depend on a home directory,
  repository checkout,
  or login session being available at boot.

The helper requires the exact process name `btrfs-cleaner`,
parent PID `2`,
and the kernel-thread flag in `/proc/<pid>/stat`.
It skips threads already assigned to the slice and tolerates processes disappearing during enumeration.
Later mounts are covered by subsequent timer activations.
There is an assignment delay until the next helper run;
the timer is reconciliation,
not a kernel thread-creation hook.

The service uses `CPUQuota=25%`,
`MemoryMax=128M`,
`TasksMax=16`,
`IOWeight=10`,
and `TimeoutStartSec=30s`.
It has `NoNewPrivileges=yes`,
`ProtectSystem=strict`,
`ProtectHome=yes`,
and write access to `/sys/fs/cgroup/btrfscleaner.slice`.

## Verification

On kernel `7.2.15-ogc1.1.fc45.x86_64` and systemd `262.1-1.fc45`:

- `systemd-analyze verify` accepted the installed slice,
  service,
  and timer without diagnostics.
- The service completed with `Result=success` and `ExecMainStatus=0`.
- Cleaner PIDs `2901` and `5763` both reported `0::/btrfscleaner.slice` in `/proc/<pid>/cgroup`.
- The slice's `cgroup.procs` contained both PIDs,
  and both weight files reported `default 10`.
- `systemctl show` reported `IOWeight=10` for the slice,
  and `UnitFileState=enabled` plus `ActiveState=active` for the timer.
- Scheduled repeat activations also completed successfully,
  including the activation ending at `2026-11-03T02:13:30-05:00`.
  Both weight files and cleaner membership remained correct.
- Disposable procfs-shaped fixtures covered assignment,
  unrelated names,
  userspace lookalikes,
  existing membership,
  disappearing processes,
  weight correction,
  and a repeated pass without additional assignment.
- The scoped Markdown lint task passed,
  and Pandoc's GFM renderer produced inspectable HTML without warnings.

The fixture harness and installation inputs are under
`${HOME}/temp/agent/btrfs-cleaner-io-weight.TNx5pe88`.
Installed files are authoritative;
scratch files are verification artifacts.
No reboot or I/O throughput benchmark was performed.
This verifies cleaner membership and configured weights,
not attribution of every downstream Btrfs worker's I/O to the cleaner slice.

## Transaction threads at 2026-11-08T16:30:06-05:00

The owner:
"Set IOWeight of btrfs transactions to 10.
Permanently.
Like how we set btrfs cleaner's IOWeight to 10."

The `btrfs-transaction` kernel threads now sit in the root-level `btrfstransaction.slice`,
whose `io.weight` and `io.bfq.weight` read `default 10`.
It is a slice of its own,
so the transaction threads and the cleaners each hold a weight of `10`
and do not split one.

The mechanism is the cleaner's:

- `/etc/systemd/system/btrfstransaction.slice` sets `IOWeight=10`.
- `/etc/systemd/system/btrfs-transaction-io-weight.service` and its `.timer` copy the cleaner's units,
  with the same resource limits and the same schedule.
- The service runs the same helper,
  `/usr/local/libexec/btrfs-cleaner-io-weight/apply.ts`,
  which gained two optional arguments after the fixture paths:
  the thread name and the slice name.
  Without them it still places `btrfs-cleaner`,
  so the cleaner's own unit did not change.

### What the weight reaches

The `btrfs-transaction` thread makes the periodic commit:
`transaction_kthread` calls `btrfs_commit_transaction` in `fs/btrfs/disk-io.c` of Linux `v7.2`.
A commit that something else asks for runs in the thread of whoever asked:
`create_snapshot` calls `btrfs_commit_transaction` itself,
in `fs/btrfs/ioctl.c` of the same version.
So a snapshot's commit keeps the weight of the process that took the snapshot,
and this setting does not touch it.
The running kernel is `7.2.15-ogc1.1.fc45`;
the source read is mainline `v7.2`,
not that build.

The slice's `io.stat` counted real disk work within seconds of the move
(`wios=798` and `wbytes=12967936` on `259:0`),
so at least some of the thread's writes are charged to the slice.

### The risk to watch

Btrfs holds other work while a commit finishes.
Measured on this host,
one snapshot's commit held a `create`,
`write`,
`fsync` and `unlink` loop in another folder for 664 ms against a 7 ms median.
A commit given less disk time under load can hold that work for longer.
`/sys/fs/btrfs/<uuid>/commit_stats` records commit counts and durations.

### Comparison of the two weights at 2026-11-08T17:03:25-05:00

The slice alternated between the default weight `100` and weight `10`,
in two-minute windows in an order that spreads slow drift over both,
under the agents' normal load.
A small file was created,
written,
synced and removed in another folder every two seconds.
The windows,
one JSON object per line,
are in `${HOME}/temp/agent/btrfs-transaction-io-weight/alternating-windows-result.jsonl`.

- `/var/mnt/encrypted`:
  at `100`,
  477 commits averaged 835 ms,
  with window averages from 178 to 8038 ms.
  At `10`,
  516 commits averaged 750 ms,
  with window averages from 287 to 6080 ms.
- The system filesystem:
  at `100`,
  19 commits averaged 5496 ms.
  At `10`,
  17 commits averaged 16546 ms.
  A few commits of over a minute dominate both figures:
  one of 65652 ms at `100`,
  and a window averaging 84560 ms over three commits at `10`.
- The synced file:
  the median of the window medians was 132 ms at `100` and 256 ms at `10`,
  the median of the window 95th percentiles was 12627 ms and 8722 ms,
  and the slowest was 32165 ms and 17069 ms.

The differences are smaller than the spread between windows of one weight,
so no effect in either direction is shown.
The two arms each held windows whose median synced file took over ten seconds,
so this weight does not remove those stalls.
An earlier comparison of one window each,
548 ms against 1018 ms on `/var/mnt/encrypted`,
was noise.
The run restored weight `10` and restarted the timer.

### Verification of the transaction threads

- Disposable procfs-shaped fixtures passed for both thread names:
  assignment,
  userspace lookalikes,
  the other thread's name left alone,
  existing membership,
  disappearing processes,
  weight correction,
  and a repeated pass.
- `systemd-analyze verify` accepted the slice,
  service and timer with no diagnostics.
- Both services ended with `Result=success` and `ExecMainStatus=0` after the helper was replaced,
  so the cleaner's run of the rewritten helper is proven too.
- Transaction PIDs `2831` and `5634` report `0::/btrfstransaction.slice`,
  and cleaner PIDs `2830` and `5633` stayed in `btrfscleaner.slice`.
- The timer reports `UnitFileState=enabled` and `ActiveState=active`.

The fixture harness,
installation inputs and the previous helper (`apply.ts.before`) are under
`${HOME}/temp/agent/btrfs-transaction-io-weight`.
No reboot was performed,
so the boot path is the cleaner's path by construction and not observed for this slice.

## Read-only checks

```bash
# Persistent host policy and last helper result.
systemctl show btrfscleaner.slice btrfstransaction.slice --property=IOWeight,ControlGroup
systemctl show btrfs-cleaner-io-weight.timer btrfs-transaction-io-weight.timer --property=UnitFileState,ActiveState
systemctl show btrfs-cleaner-io-weight.service btrfs-transaction-io-weight.service --property=Result,ExecMainStatus
cat /sys/fs/cgroup/btrfscleaner.slice/io.weight /sys/fs/cgroup/btrfscleaner.slice/io.bfq.weight
cat /sys/fs/cgroup/btrfstransaction.slice/io.weight /sys/fs/cgroup/btrfstransaction.slice/io.bfq.weight
cat /sys/fs/cgroup/btrfscleaner.slice/cgroup.procs /sys/fs/cgroup/btrfstransaction.slice/cgroup.procs
journalctl --unit=btrfs-cleaner-io-weight.service --unit=btrfs-transaction-io-weight.service --no-pager --lines=20
```

If a future kernel rejects assignment,
the service fails and the journal retains the error.
If BFQ is unavailable,
the missing `io.bfq.weight` file also fails the helper rather than silently claiming the policy applied.
Future runtime updates must refresh the root-owned Node copy;
repository toolchain updates do not replace it automatically.
