# systemd 259 on Fedora 44: a Claude Code session shares one cgroup with everything it spawns, so `systemd-oomd` reaps the session along with its own build

## Symptom

A Claude Code session dies without warning,
 mid-turn,
 taking the terminal window with it.
No error appears in the session and no crash dump is written.
The only record is in the journal:

```text
systemd[2066]: app-ghostty-surface-transient-5181.scope: systemd-oomd killed some process(es) in this unit.
systemd[2066]: app-ghostty-surface-transient-5181.scope: Failed with result 'oom-kill'.
```

The kernel out-of-memory killer is NOT involved,
 and looking for it is the first wrong turn:
`free` reports plenty available at the time,
 and `dmesg` carries no `Out of memory: Killed process`.
Long-running agent sessions are the population that hits this,
 because the trigger is accumulated
memory in one cgroup over days rather than a single large allocation.

## Root cause

`systemd-oomd` kills by memory PRESSURE over a cgroup,
 not by an absolute limit,
 and the cgroup it
picked contains the whole terminal surface.

The kill record names the threshold,
 the measurement and the target:

```text
systemd-oomd[1220]: Considered 1 cgroups for killing, top candidates were:
systemd-oomd[1220]:         Path: /user.slice/.../app.slice/app-ghostty-surface-transient-5181.scope
systemd-oomd[1220]:                 Memory Pressure Limit: 0.00%
systemd-oomd[1220]:                 Pressure: Avg10: 71.20, Avg60: 23.08, Avg300: 5.40, Total: 56s
systemd-oomd[1220]:                 Current Memory Usage: 10.5G
systemd-oomd[1220]: Killed /user.slice/.../app-ghostty-surface-transient-5181.scope due to memory pressure
  for /user.slice/.../app-ghostty-surface-transient-5181.scope being 71.20% > 60.00% for > 5s with reclaim activity
```

Three facts in that record do the work.

`Memory Pressure Limit: 0.00%` is the unit's OWN setting,
 and zero means unset,
 so the effective
threshold came from the daemon's defaults rather than from anything about this scope.
`Pressure: Avg10: 71.20` is a PSI figure:
 the share of the last ten seconds in which something in
the cgroup stalled waiting on memory.
 It is a stall measure,
 not a usage measure,
 which is why
`free` showing headroom is consistent with the kill.
`Current Memory Usage: 10.5G` is the whole scope,
 and this is the part that matters for an agent.

The scope is one terminal surface,
 and everything the session starts is inside it.
 The exit record
shows what that came to over the session's life:

```text
app-ghostty-surface-transient-5181.scope: Consumed 20h 54.147s CPU time over 2d 21h 25min 24.181s
  wall clock time, 18.4G memory peak, 9.7G memory swap peak.
```

An 18.4 GiB peak in one terminal tab is not the agent process.
 It is the agent plus every build,
test run,
 language server and long-lived child it spawned,
 all accounted to one cgroup,
 because a
shell's children inherit its cgroup and nothing moved them out.
 The agent then competes for reclaim
against work it started itself,
 and `systemd-oomd` kills the cgroup with the worst pressure,
 which is
that one.
 The session is collateral damage from its own build.

Swap makes it arrive sooner on this host,
 because zram is compressed RAM rather than disk:

```text
$ zramctl --output NAME,ALGORITHM,DISKSIZE,DATA,COMPR
NAME       ALGORITHM DISKSIZE  DATA COMPR
/dev/zram0 zstd           16G 12.8G  4.2G
```

Paging out does not free the machine's memory here,
 it only compresses it,
 so a working set that
exceeds RAM produces reclaim churn rather than relief,
 and reclaim churn is exactly what PSI counts.

### An earlier reading of the configuration was wrong

Reading `/etc/systemd/oomd.conf.d/` after the fact reports `DefaultMemoryPressureLimit=80%` and
`DefaultMemoryPressureDurationSec=20s`,
 which contradicts the 60 percent in the kill record and
invites the conclusion that the record is misleading.
It is not.
 The override postdates the kill:

```text
$ stat --format='%y  %n' /etc/systemd/oomd.conf.d/20-freeze-hardening.conf
2026-08-16 21:19:03.831114177 -0400  /etc/systemd/oomd.conf.d/20-freeze-hardening.conf

$ systemctl show systemd-oomd --property=ActiveEnterTimestamp
ActiveEnterTimestamp=Sun 2026-08-16 21:19:04 EDT
```

The kill was at 21:06:16 and the hardening landed at 21:19:03,
 thirteen minutes later,
 with the
daemon restarting a second after that.
 What was in force at 21:06 were the built-in defaults,
 and
every ancestor confirms it declined to set its own:

```text
$ systemctl --user show app-ghostty-surface-transient-2534887.scope --property=ManagedOOMMemoryPressureLimit
ManagedOOMMemoryPressureLimit=0
$ systemctl --user show user@1000.service --property=ManagedOOMMemoryPressureLimit
ManagedOOMMemoryPressureLimit=0
```

Check timestamps on configuration before letting it overrule a journal record.
 The journal describes
the moment;
 the configuration describes now.

## Verification

Under systemd 259 (259.7-1.fc44),
 ghostty 1.3.1-1.fc44,
 Claude Code 2.1.233,
 on Fedora 44.

Find the kill and its attribution:

```bash
journalctl --since '3 hours ago' --no-pager --output=short \
  | rg --context 2 'oomd|oom-kill'
```

Read current usage and pressure per terminal scope,
 which shows whether one tab is carrying the
whole session's history:

```bash
for d in /sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/*.scope; do
  printf '%8.2f GiB  %s  %s\n' \
    "$(echo "$(cat "$d/memory.current")/1073741824" | bc -l)" \
    "$(awk '/^some/{print $2}' "$d/memory.pressure" | head -1)" \
    "$(basename "$d")"
done | sort --reverse --numeric-sort | head
```

Confirm which cgroup a spawned command actually lands in,
 which is the whole question:

```bash
cat /proc/self/cgroup
```

Two outcomes distinguish the states.
 `.../app.slice/app-ghostty-surface-transient-N.scope` means the
command shares the session's cgroup and contributes to the pressure that can kill it.
`.../app.slice/claude-code-bash` means it does not.

## Verified workarounds

### Give the agent's spawned commands their own cgroup with a hard limit

Claude Code places Bash tool children in a `claude-code-bash` cgroup when
`CLAUDE_CODE_TOOL_MEMORY_LIMIT` is set in `~/.claude/settings.json`:

```json
{
  "env": {
    "CLAUDE_CODE_TOOL_MEMORY_LIMIT": "8000000000"
  }
}
```

Verified enforced rather than merely declared:

```text
$ cat /sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash/memory.max
8000000000
```

Tradeoff,
 and it is a real one:
 the limit is a HARD `memory.max`,
 so a build that legitimately needs
more than the figure is killed by the kernel inside its own cgroup rather than throttled.
 That is the
intended direction (the build dies,
 the session survives),
 but the number has to be set above the
genuine peak of the heaviest build in the repo or it converts a rare session loss into a frequent
build failure.
 `memory.high` is left at `max`,
 so there is no throttling tier before the wall.

This does not make the session immortal.
 It removes the session's OWN children as the source of
pressure;
 a different process on the machine can still drive the terminal scope's pressure up.

### Put long runs somewhere the terminal cannot take with it

A multi-hour run started from the session should write its output to a file outside the terminal and
have its identity recorded before it starts,
 so a killed session loses only the view:

```bash
RUNS=$(mktemp --directory "${HOME}/temp/agent/<name>.XXXXXXXX")
<command> > "${RUNS}/run.log" 2>&1
```

Tradeoff:
 the harness's own captured output goes empty,
 so progress has to be read from the file.
That is the point,
 since the file is what survives.

### Raise the pressure threshold, knowing what it costs

```text
# /etc/systemd/oomd.conf.d/20-freeze-hardening.conf
DefaultMemoryPressureLimit=80%
DefaultMemoryPressureDurationSec=20s
```

Tradeoff:
 this is the setting that decides whether the desktop is killed or frozen.
 Raising the limit
and lengthening the duration buys tolerance for bursty reclaim,
 and pays for it with longer periods
of a machine that is stalling on memory before anything intervenes.
 It trades a killed session for a
possibly unresponsive desktop,
 which is a preference rather than a fix.

## Seen: a background run killed, its container left running

On 2026-10-06 the journal recorded:

```text
systemd-oomd[2363]: Killed /user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash due to memory pressure for /user.slice/user-1000.slice/user@1000.service/app.slice being 85.14% > 80.00% for > 20s with reclaim activity
```

The session survived,
but a capture loop it had started in the background was in `claude-code-bash` and died with it,
exit status 1 and no message of its own.
The Android emulator container that loop owned was in podman's cgroup and kept running with no owner.
After such a kill,
list containers with `podman ps --all` and stop any the run owned the way the run itself would have,
before starting it again;
the run's own log ends mid-step and does not say it was killed.

Later that day the same cgroup was killed at 17:07:01,
17:08:23 and 17:09:39.
It was at its `memory.max` of 8000000000 bytes,
and its largest processes were other sessions' builds:
two `forbidden-strings` build scripts held about 4.4 GB and 2.1 GB resident.
Every command of this session died with them,
including the restore meant to stop the orphaned emulator.
Run as its own user unit,
`systemd-run --user --unit=<name> --collect --wait --property=MemoryMax=1G <command>`,
the restore finished in 16 seconds with a peak of 64 MB,
and the capture loop was then started the same way with `MemoryMax=2G`,
outside the shared cgroup.

## What does not work

- Detaching a run from the shell with a new session (`setsid`, Node's `spawn` with `detached: true`).
  A new session leaves the process in its parent's cgroup,
  so the daemon's kill of `claude-code-bash` takes it as well.
  On 2026-10-06 an IDE gate started that way at 17:12 died in the 17:14:48 kill
  without writing its exit record;
  the same gate then ran as its own user unit through `systemd-run --user`.
- Looking for the kernel OOM killer.
   It never ran.
   `dmesg` is silent,
   `free` shows headroom,
   and
  both readings are consistent with a PSI-based kill that neither one measures.
- Reading `free` or `memory.current` to judge whether a kill was justified.
   The daemon reads
  `memory.pressure`,
   so usage figures do not reproduce its decision.
   Read the PSI line.
- Reading `/etc/systemd/oomd.conf.d/` and concluding the journal record must be wrong.
   Local
  overrides may postdate the incident;
   check `stat` and the daemon's `ActiveEnterTimestamp`.
- Trusting that a per-tool memory setting protects a session that was already running.
   The setting
  here was configured after the session that later died had launched,
   and that session's children
  were in its terminal scope when the kill came.
   Whether the setting is picked up at session start
  or at each spawn was NOT established,
   so do not reason from either;
   check `/proc/self/cgroup` in
  the session you actually care about.
- Concluding that isolation worked because some long-running process survived the kill.
   It is the
  most inviting wrong answer available,
   and it was reached and discarded during this investigation.
  A `pi` process with four hours of uptime outlived the reap,
   which looked like proof that spawned
  children were safely elsewhere.
   It was not:

```text
$ cat /proc/2919919/cgroup
0::/user.slice/.../app.slice/app-ghostty-surface-transient-2894351.scope
$ ps --no-headers -o lstart= -p 2919919
Sun Aug 16 16:48:19 2026
```

  Scope `2894351` is a DIFFERENT terminal surface from the reaped `5181`.
   The process survived
  because it was in another tab,
   and it is in no `claude-code-bash` cgroup at all despite starting
  six hours after that setting was configured.
   Every surface is its own scope,
   so surviving
  processes prove only that the kill was scoped,
   which was already known from its name.

## Upstream filing decision

Nothing to file,
 and the audit stops at the first constraint.

1.  **Is it really upstream's fault?**
     NO,
     and this ends it.
     `systemd-oomd` did exactly what it is
    documented to do:
     kill the cgroup whose memory pressure exceeded the configured threshold for the
    configured duration.
     Ghostty placing each surface in its own transient scope is also correct,
     and
    is what made the kill legible at all.
     The surprise is entirely in the interaction,
     namely that an
    agent's children share its scope,
     which is ordinary cgroup inheritance rather than a defect in
    any of the three.

Constraints 2 through 6 are not reached,
 since there is no upstream defect for them to be about.
`.out-of-scope/claude-code-upstream-bugs.md` would in any case cover a report against Claude Code.

No duplicate search was run against systemd or ghostty,
 because there is no behavior to report.
