# Btrfs cleaner I/O weight

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

## Read-only checks

```bash
# Persistent host policy and last helper result.
systemctl show btrfscleaner.slice --property=IOWeight,ControlGroup
systemctl show btrfs-cleaner-io-weight.timer --property=UnitFileState,ActiveState
systemctl show btrfs-cleaner-io-weight.service --property=Result,ExecMainStatus
cat /sys/fs/cgroup/btrfscleaner.slice/io.weight /sys/fs/cgroup/btrfscleaner.slice/io.bfq.weight
cat /sys/fs/cgroup/btrfscleaner.slice/cgroup.procs
journalctl --unit=btrfs-cleaner-io-weight.service --no-pager --lines=20
```

If a future kernel rejects assignment,
the service fails and the journal retains the error.
If BFQ is unavailable,
the missing `io.bfq.weight` file also fails the helper rather than silently claiming the policy applied.
Future runtime updates must refresh the root-owned Node copy;
repository toolchain updates do not replace it automatically.
