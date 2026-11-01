# TypeScript 7 semantic lint exceeds the Pi worker's 2-GiB memory budget

## Symptom

During auto-mode verification on 2026-10-12,
Plasma displayed memory-shortage notifications and the package-wide Oxlint task ended without diagnostics.
Mise reported:

```text
# proc_f4ce and proc_f064 stderr
[//package/pi-plugin/auto-mode:lint:oxlint] ERROR node exited with non-zero status: killed by SIGTERM
```

The installed notification definition is `/usr/share/knotifications6/oom_notifier.notifyrc`.
It belongs to Plasma and identifies its displayed source as `Linux Kernel`.
The decisive workload evidence is the kernel journal,
not the generic notification title.

## Cause and corrected evidence window

The agent's dedicated `piwork3285273.slice` had `MemoryMax=2147483648`,
no swap,
CPU quota 200%,
and a task limit of 256.
The limit applied to the combined worker processes,
not separately to each TypeScript process.

A relative journal query identified the actual victims:

```text
# Kernel journal, host-local EDT timestamps
Oct 11 20:52:25: CONSTRAINT_MEMCG, task=tsc, pid=4039089, scope=run-p4019559-i58504953.scope
Oct 12 16:34:21: CONSTRAINT_MEMCG, task=tsc, pid=1984453, scope=run-p1983483-i90026146.scope
Oct 12 16:46:00: CONSTRAINT_MEMCG, task=tsc, pid=1988319, scope=run-p1988085-i89963764.scope
```

Every event named this session's slice as `oom_memcg`.
These scopes correspond to `proc_0d52`,
`proc_f4ce`,
and `proc_f064`.
The kernel's diagnostic was `Memory cgroup out of memory: Killed process ... (tsc)`.
The slice recorded three OOM kills.

The earlier calendar-window query excluded the October 11 event in the host's local time zone.
Its empty output did not establish absence of an OOM event.
The relative-window query corrected that evidence gap:

```sh
# Run through the restricted command launcher.
journalctl --no-pager --since='-26h' \
  --grep='oom_memcg=.*/piwork3285273.slice|Memory cgroup out of memory: Killed process'
```

This is distinct from earlier `systemd-run` transport failures before task startup.
The kernel reported cgroup enforcement,
not a machine-wide OOM kill or an ordinary lint finding.
The host probe reported 40,066,496 KiB of `MemAvailable` at the time of the configuration change.

## Verified correction

The user explicitly authorized configuration changes after reporting the notifications.
Only the session's aggregate worker RAM budget was raised,
from 2 GiB to 4 GiB:

```sh
# Executed through the existing restricted launcher.
systemctl --user --runtime set-property piwork3285273.slice MemoryMax=4G
```

Subsequent command scopes also use `--property=MemoryMax=4G`.
The shared slice and each scope retain CPU quota 200%,
256 tasks,
and no swap.
The Pi host itself remains outside the worker slice.
System-wide OOM protection and desktop notifications remain enabled.
The slice setting is runtime configuration,
so it needs rechecking after an environment restart.

`proc_0f9a` asserted the real command's `memory.max=4294967296`,
verified child cgroup inheritance,
and reran the failing full-package lint with one Oxlint thread,
Node old-space capped at 512 MiB,
and `GOMEMLIMIT=768MiB`.
It passed all 77 files with 486 rules,
zero warnings,
and zero errors.
The slice's measured peak was 3,622,907,904 bytes,
which exceeds the former budget and fits the replacement budget.

The worker settings are now encoded in `package/pi-plugin/auto-mode/mise.toml`
for both Oxlint tasks,
through the shared `auto_mode_lint_resources` task variable.
They preserve the existing Node options and do not disable lint rules.
`proc_3c0c` confirmed the persisted task route without caller-supplied memory settings.
It again passed all 77 files with 486 rules,
zero warnings,
and zero errors.
The slice's OOM-kill counter remained at three,
so neither successful run added a kill.

`GOMEMLIMIT` is a [soft Go runtime limit](https://go.dev/doc/gc-guide#Memory_limit),
not the hard combined worker limit.
The cgroup remains the hard resource boundary.
Run memory-heavy checks serially inside that shared budget.
Passing this workload does not guarantee every future workload will fit it.

## Rejected approaches

The full-package run at 2 GiB failed even with the Node and Go worker settings.
Those settings alone were not a verified remedy.
Selected-file lint completed at the smaller budget,
but that did not establish package-wide lint coverage.

Disabling the system OOM service,
muting notifications,
or exempting workers from all memory limits would not address the measured budget mismatch safely.
None was done.
No instruction-source bound,
judgment deadline,
retry allowance,
or lint rule changed.

## Upstream filing decision

- Upstream fault is not established:
  the kernel enforced the configured worker limit.
- An upstream code fix is not required for the verified configuration correction.
- Cgroup memory enforcement and Go's soft limit are supported configuration mechanisms.
- External contribution acceptance was not investigated because no upstream filing is proposed.
- Maintainer willingness was not investigated for the same reason.
- The local correction was exercised against the actual full-package task,
  not only a settings read-back.

No upstream issue or comment is warranted by this incident.
