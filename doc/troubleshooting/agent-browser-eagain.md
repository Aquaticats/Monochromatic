# agent-browser 0.39.1: "Resource temporarily unavailable" from the user task ceiling or a 30 s read timeout

## Symptom

During heavy parallel `agent-browser` use on 2026-10-18 (local time, UTC-4, so 2026-10-19 in UTC),
`agent-browser` commands failed three times in about 80 seconds with `Resource temporarily unavailable`.
The text came in two different shapes, and they have different causes.

The first shape is a shell diagnostic printed by the npm shim `bin/agent-browser` (a `/bin/sh` script),
at 22:39:35 local (02:39:35 UTC):

```text
/home/user/.local/share/mise/installs/npm-agent-browser/latest/bin/agent-browser: fork: retry: Resource temporarily unavailable
```

A second agent run printed the same line plus `/bin/sh: fork: retry: Resource temporarily unavailable`
at 22:40:19 local, next to a Node `Assertion failed` from
`node::WorkerThreadsTaskRunner::DelayedTaskScheduler::Start()`, which is Node failing to start a thread.
In the 22:39:35 case the command's JSON output still arrived after the diagnostic.

The second shape is an `agent-browser` CLI error at 22:40:52 local (02:40:52 UTC):

```text
✗ Failed to read: Resource temporarily unavailable (os error 11) (the command reached the daemon but no response came back; it may or may not have run, so it was not retried)
✗ Invalid response: EOF while parsing a value at line 1 column 0 (the command reached the daemon but no response came back; ...
```

That command was a shell loop of six `open`, `wait --load networkidle` and `eval` rounds,
started at 22:40:05 and answered at 22:40:52, so it ran about 47 seconds.

Both are `EAGAIN` (errno 11), but only the first shape is a failed `fork`.

## Root cause

### The first shape: the per-user task ceiling was reached

Every process and every thread is one task.
The kernel refuses `fork` and `clone` with `EAGAIN` when a `pids.max` in the task's cgroup ancestry
or the user's `RLIMIT_NPROC` is full.
At the time of the failures both ceilings were `12288`;
the cgroup one was raised to `32768` at 22:59 local and the PAM one at 23:07 local
(see [fedora-44-fold-emulator-user-task-ceiling.md](fedora-44-fold-emulator-user-task-ceiling.md),
whose section "Thread-creation failures at the 12288 ceiling on 2026-10-19" covers the same window for other victims).

Measured afterwards on 2026-10-19 around 03:15 UTC, read-only:

```text
/sys/fs/cgroup/user.slice/user-1000.slice                     pids.max=32768 pids.peak=12289 pids.events: max 329
/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service   pids.max=max   pids.peak=12271 pids.events: max 0
/sys/fs/cgroup/user.slice/.../user@1000.service/app.slice     pids.max=max   pids.peak=10538 pids.events: max 0
/sys/fs/cgroup/user.slice/.../app.slice/claude-code-bash      pids.max=max   pids.peak=5325  pids.events: max 0
```

Three facts follow.

The limit that tripped is on `user-1000.slice`.
It is the only cgroup in the chain with a finite `pids.max`, the only one with a nonzero `max` counter,
and its `pids.peak` of `12289` is the former `TasksMax=12288` plus the one rejected charge.
The shared `claude-code-bash` cgroup never hit a limit of its own,
so a per-Bash-tool cgroup is not the cause.

The kernel log names the forking task's cgroup, not the cgroup whose limit tripped.
`journalctl --dmesg` showed these lines (eleven lines against a counter of 329;
printk rate limiting is the likely reason, not verified):

```text
Oct 18 22:36:59 bazzite kernel: cgroup: fork rejected by pids controller in .../app.slice/dsh-subprocess-282325-372469105f2b.scope
Oct 18 22:39:10 bazzite kernel: cgroup: fork rejected by pids controller in .../app.slice/claude-code-bash
Oct 18 22:39:10 bazzite kernel: cgroup: fork rejected by pids controller in .../app.slice/ide-sampler-baseline.service
Oct 18 22:39:13 bazzite kernel: cgroup: fork rejected by pids controller in .../app.slice/tr-gate-g109.service
Oct 18 22:39:13 bazzite kernel: cgroup: fork rejected by pids controller in .../user.slice/libpod-<id>.scope/container
Oct 18 22:39:23 bazzite kernel: cgroup: fork rejected by pids controller in .../user@1000.service/piwork3285273.slice
```

Rejections in `claude-code-bash`, Podman containers, IDE measurement services and a `piwork` slice at the same moment
are consistent only with a shared ancestor, which is `user-1000.slice`.
The same window has core dumps of `node-MainThread` (eight at 22:38), `mise` (sixteen at 22:39),
`rust-lld`, and `podman`, and a Go `runtime/cgo: pthread_create failed: Resource temporarily unavailable` at 22:40:10.
The `agent-browser` shim was one more victim of a host-wide event, not its cause alone.

### Why agent-browser is a large contributor

Each `agent-browser` session is a daemon plus its own headless Chrome.
The package README describes this in its "Architecture" section
(`README.md:1705` to `README.md:1712` in the installed package directory):

```text
The daemon starts automatically on first command and persists between commands for fast subsequent operations.
After **1 hour** with no commands or dashboard input it saves configured restore state, closes the browser, and exits
```

`README.md:722` to `README.md:745` says each session has its own browser instance.
One session started and measured here (`about:blank`-like data URL, one tab) was 16 processes,
205 threads and about 1.2 GiB resident memory:
the Rust daemon (31 threads), the Chrome browser process (43), a GPU process (29), a network service (17),
a storage service (10), five renderers (11 to 21 each), two crashpad handlers and the zygotes.

`$XDG_RUNTIME_DIR/agent-browser/` held 112 distinct session names by the end of the day;
53 of them were first created in the 22:00 hour and 35 had their state file touched between 22:10 and 22:41.
Those counts are upper bounds on concurrency, because the state files outlive closed sessions
(`agent-browser session list` reported `No active sessions` afterwards).
Taking the 35 as a bound, 35 sessions times about 205 tasks is about 7200 tasks,
on top of an idle user baseline of about 5300 tasks measured the same day,
which is above 12288 before any `mise`, `rust-lld` or Podman fan-out is added.
The instantaneous task count at 22:39 was not captured, so this is arithmetic, not a measurement.

### The second shape: the CLI's own read timeout, possibly triggered by the same starvation

The README says in `README.md:1238`:

```text
The default timeout for standard operations (clicks, waits, fills, etc.) is 25 seconds.
This is intentionally below the CLI's 30-second IPC read timeout so that the daemon returns a proper error
instead of the CLI timing out with EAGAIN.
```

and `README.md:1247` repeats that slow operations can produce `EAGAIN` because the CLI's read timeout expires first.
A timed-out socket read returns `EAGAIN`, which Rust prints as `Resource temporarily unavailable (os error 11)`.
The installed Linux binary contains the strings `Failed to read:`, `Invalid read timeout:` and
`Resource temporarily unavailable`.
The 47 second loop is consistent with a command that waited more than 30 seconds on the daemon.

Whether the daemon stalled because Chrome could not create threads or processes in that window is plausible
(core dumps and a `pthread_create` failure happened within the same minute) but was not measured.
A page that never reaches `networkidle` would give the same message on an idle host.
Do not treat this shape as proof of a task-ceiling hit.

### An earlier reading that was wrong

The report that started this investigation called every instance "apparently from fork/spawn".
Only the shim's `fork: retry:` line is a fork failure.
The `Failed to read ... (os error 11)` line is an IPC read timeout by the README's own description,
and the CLI says so by adding that the command reached the daemon.

### Why a restart of the limits did not fix older sessions

`/proc/self/limits` for this session and for the `claude` process started 2026-10-17 23:35 both still report
`Max processes 12288 12288` after the PAM file was changed to `32768` at 23:07.
`RLIMIT_NPROC` is inherited and fixed per process,
so every `agent-browser` daemon launched from an already-running Claude Code session keeps `12288`,
while the user-wide task count is compared against it.
The cgroup ceiling is now `32768`, but processes started from older sessions are still limited to `12288`
until those sessions are restarted.
The effect on a post-raise session was not exercised because doing so would require exhausting the host.

## Verification

All commands were read-only except the two probe sessions, which this investigation started and closed itself.
No system limit was changed and no other session was touched.

```sh
cat /proc/self/cgroup                                  # .../app.slice/claude-code-bash
cat /proc/self/limits                                  # Max processes 12288 12288
systemctl show user-1000.slice --property=TasksMax,TasksCurrent
sysctl kernel.pid_max kernel.threads-max               # 4194304 and 504531: not constraints
journalctl --dmesg --since "2026-10-17"                # "fork rejected by pids controller" lines
journalctl --user --since "2026-10-18 22:30" --until "2026-10-18 22:50"   # core dumps
ps -eLo comm --no-headers | sort | uniq --count | sort --numeric-sort --reverse
```

Limits that cannot explain the failures:
`kernel.pid_max` is `4194304`, `kernel.threads-max` is `504531` against 6058 tasks host-wide,
`DefaultTasksMax` is `75679`, and `user@1000.service` and `app.slice` have `TasksMax=infinity`.

### Does a per-job systemd scope bound the browser?

Probe, using a throwaway scope for one session that was then closed with `agent-browser close`:

```sh
systemd-run --user --scope --unit=eagain-probe-scope --property=TasksMax=400 \
  agent-browser --session eagain-probe-2 open "data:text/html,<p>s</p>"
cat /proc/<chrome-main-pid>/cgroup
cat /sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/eagain-probe-scope.scope/pids.current
```

Result: the daemon and the renderers stayed in `eagain-probe-scope.scope` (`pids.current=179`, `pids.max=400`),
but the Chrome browser process moved itself into
`.../app.slice/app-org.chromium.Chromium-<pid>.scope`, a sibling transient scope that is not under the probe's scope.
The same happens without `systemd-run`: the browser of the earlier probe session sat in its own
`app-org.chromium.Chromium-<pid>.scope` while its daemon stayed in `claude-code-bash`.
So a `TasksMax` on the wrapping unit caps only the part of the tree created before the browser relocates,
and it does not cap the whole session.
The remaining processes of that tree were split between the two scopes
(the probe scope held 179 tasks, not all of them).
`--slice=<name>.slice` was not tried; the same relocation applies to any parent of the daemon.

## Verified workarounds

Ranked by how directly each addresses the measured multiplier of about 205 tasks per session,
and by what was verified.

### Close sessions promptly and cap idle time (verified in documentation, recommended first)

End each job with `agent-browser --session <name> close`, and start long unattended jobs with
`--idle-timeout 5m` or `AGENT_BROWSER_IDLE_TIMEOUT_MS=300000` (`README.md:1089`).
The default idle timeout is one hour (`README.md:1712`), so a session abandoned after its last command holds
about 205 tasks for up to an hour.
The user-wide task count dropped by several hundred right after closing the probe sessions,
though other jobs moved it too.
Tradeoff: a short idle timeout restarts the browser on the next command,
which loses un-restored tabs unless `--restore` is used (`README.md:846`).

### Run fewer concurrent sessions, reuse one session per job (recommended second)

Reuse one `--session` for a loop of `open` and `eval` calls instead of a session per page or per script;
the README says commands reuse the same daemon (`README.md:494`).
Keep the number of simultaneous sessions to what the host can afford:
with a baseline of about 5300 tasks and a ceiling of `12288`, the headroom for sessions alone is about 30.
Tradeoff: sessions share nothing, so a job that needs isolation cannot be merged;
the cost is serialising independent jobs.

### Restart long-lived agent sessions so they pick up the raised nproc (standing risk)

Sessions started before 23:07 local on 2026-10-18 still carry `12288`.
Restarting them, or an owner-run `prlimit --pid <pid> --nproc=32768:32768` per process,
removes that ceiling for them.
This investigation did not run it because changing limits was out of scope.
Tradeoff: restarting a Claude Code session loses its in-flight context.

### Put browser-heavy jobs in their own systemd unit with TasksMax (partial)

`systemd-run --user --unit=<name> --collect --property=MemoryMax=<cap> --property=TasksMax=<n> <command>`
isolates the job's Node scripts, Podman clients and the `agent-browser` daemon,
and a runaway job then fails inside its own unit before it can starve the rest of the user.
It does not bound Chrome, because Chrome relocates itself into its own `app-org.chromium.Chromium-<pid>.scope`
(see the probe above), so choose `TasksMax` to protect the rest of the host from the job's fan-out,
not to cap the browsers.
It also follows the repo rule to start long background runs in their own named unit with a memory cap.

### Raise the ceiling again (not recommended)

Raising `user-1000.slice` above `32768` only moves the point of failure.
Both ceilings were already raised once on the owner's instruction, and the problem is the multiplier.

## What does not work

- Reading `Failed to read: Resource temporarily unavailable (os error 11)` as a fork failure:
  it is the CLI's read timeout, and an idle host reproduces it with a page that never goes network-idle.
- Setting `AGENT_BROWSER_DEFAULT_TIMEOUT` above `30000`:
  the README says this causes `EAGAIN` on slow operations (`README.md:1247`).
- Retrying the same non-idempotent command on the second shape:
  the CLI itself refuses to resend a command that may have run, and reports `outcome_unknown` (`README.md:1247`).
- Capping a whole session with `systemd-run --scope --property=TasksMax=<n>`:
  Chrome leaves the scope, as measured above.
- Blaming the shared `claude-code-bash` cgroup:
  its `pids.max` is `max` and its `pids.events` `max` counter is `0`.

## Not verified

- The task count at the instant of each failure; only the later `pids.peak` and the kernel log were available.
- Whether the 22:40:52 read timeout was caused by the same starvation or by a slow page.
- How many `agent-browser` sessions were alive at 22:39; the state files outlive sessions.
- Behavior of sessions that still hold `RLIMIT_NPROC=12288` once the user's task count passes that figure.

## Upstream filing decision

No issue is drafted.

1. The cause is local: both ceilings were local configuration, and the load came from parallel local jobs.
2. The only upstream-adjacent item is the wording of a read timeout as `os error 11`, which the README documents.
3. `agent-browser` supports multiple sessions (`README.md:722`) and documents idle shutdown.
4. to 6. were not assessed because constraint 1 fails: documented behavior is not a defect,
   and no fix was prototyped. No `.out-of-scope/` check was needed for a filing that is not made.

Related reading: [cli-git-tag-push-eagain.md](cli-git-tag-push-eagain.md) for the same
task-versus-process counting mistake.
