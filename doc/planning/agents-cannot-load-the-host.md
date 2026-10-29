# Keeping agents from loading the host, in every session

Status:
proposal of 2026-10-21,
nothing here is built or accepted.
Owner decisions are asked in the session that wrote this and recorded in `doc/handover/slint-ide-0x.md`.

## What happened

On the night of 2026-10-20 to 2026-10-21 subagents of one Claude session ran synthetic CPU load on the owner's workstation
for hours and the fans ran at their highest speeds while the owner slept (the owner lost about four hours of sleep).
The stress runs were a flake-hunting method that the delegating session specified:
two systemd units,
`ide-stress-load-354743` and `ide-stress-load-473223`,
ran 12 `stress-ng-cpu` workers each,
and containers ran the whole test suite with `--stress-count 100`.
The first unit had run about 81 minutes when it was found (4865 seconds of process age),
its start time was not logged.

Measured in `~/.local/share/claude-cpu-cap/samples/2026-10-20.csv` and `2026-10-21.csv`
(hourly means,
UTC;
local time is UTC minus 4):

- one-minute load:
  21 at 00:00,
  40 at 01:00,
  33 at 02:00,
  50 at 03:00,
  68 at 04:00,
  123 at 05:00,
  72 at 06:00.
- CPU temperature (Tctl):
  86 at 01:00 and 02:00,
  95 at 03:00,
  92 at 04:00,
  96 at 05:00.
- CPU fan:
  1,184 rpm at 00:00 rising to 1,471 at 03:00 and 1,491 at 05:00;
  ordinary working hours that day sat at 885 to 1,336.

These figures show that the load rose with the stress runs.
They do not split the load between the stress runs and other sessions' work (a translation-repair fan-out
raised the load to 123 on the previous evening,
which was not this session's),
so the share is not established.

## Why the existing protections did not stop it

1. **The rule did not bind.**
   `RXI` in `AGENTS.md` already says stress and load runs go in `podman run --memory=2g --cpus=2 --rm`.
   The delegating session's brief misquoted it ("systemd-run with a memory cap") and suggested `stress-ng` or busy loops.
   A rule that depends on the delegating agent quoting it correctly failed at the first mistake.
2. **The CPU cap does not cover transient units.**
   `claude-cpu-watch.service` (`package/claude-cpu-cap/daemon`) writes `cpu.max` into the one `claude-code-bash`
   cgroup only (`doc/runbook/claude-cpu-cap.md`,
"Known gaps":
units started with `systemd-run --user` get their own cgroup
   in `app.slice` and are outside the cap).
   The repo's own preamble told long background runs to use such units,
so following the advice left the cap.
   Containers (`podman`) are outside it too.
3. **The cap is not set below the machine.**
   `~/.config/claude-cpu-cap/threads` holds 28 and `nproc` reports 28 in an agent shell,
on a 32 thread host,
   so even commands inside the cap may use 28 threads (the owner's decision of 2026-10-13,
per the runbook).
4. **Nothing reacts to heat or synthetic load.**
   The watcher logs CPU pressure and thermal alerts (`ALERT`,
`CLEAR`) and samples Tctl and fan speed,
   but only reports;
it never throttles,
freezes or kills anything.
   The guardrail hook (`package/claude-code-plugin/guardrail`) guards Agent calls and three Bash patterns
   (`bun test`,
`pgrep` wait loops,
`resume`),
none about load.

## Layers, and what each covers

- **A:
  guardrail hook rules** (in this repo).
  Deny Bash commands that start synthetic load:
  `stress-ng`,
  `stress`,
  `--stress-count`,
  `--stress-duration`,
  busy-loop forms (`while true`,
  `while(true)`,
  `yes > /dev/null`,
  `dd if=/dev/zero of=/dev/null`,
  `sha256sum /dev/zero`),
  and `systemd-run --user` commands without `CPUQuota=`;
  deny Agent calls whose prompt tells the subagent to run such tools.
  The denial message gives the alternative (cap the test's own container with `--cpus=2`).
  Covers Claude Code sessions that load the plugin.
  Not verified:
  whether Claude Code runs the hook for a subagent's own Bash calls
  (the plugin's pgrep check was verified in the main session only,
  `package/claude-code-plugin/guardrail/README.md`),
  and whether the hook interface can add text to an Agent prompt (it would let the hook append the ban to every brief).
  Both are checks to run before relying on this layer.
- **B:
  a synthetic-load reaper in `claude-cpu-watch`** (this repo,
  a user service on the machine).
  Every few seconds,
  find processes of the user whose command matches the same denylist and that run inside
  the `claude-code-bash` cgroup or a transient service unit (never inside a desktop `app-*` scope,
  so the owner's own
  terminal is left alone),
  send `SIGTERM`,
  then `SIGKILL` after a grace period,
  and log `REAPED` with the command line.
  Independent of the harness (Claude Code,
  pi,
  codex,
  any script),
  because it looks at processes,
  not at tool calls.
  Misses a load that does not match the denylist (a renamed or inline busy loop).
- **C:
  a thermal and night safety net in `claude-cpu-watch`.**
  When Tctl stays at or above a threshold for a few minutes,
  or the CPU fan stays above a speed,
  lower the cap by writing the `threads` file and freeze the `claude-code-bash` cgroup
  (`cgroup.freeze`) and agent-started units (`systemctl --user freeze`),
  then thaw when the machine cools.
  Optionally a lower cap during sleeping hours.
  Independent of what the load is,
  so it also covers a legitimate but heavy build.
  Costs:
  a frozen build stalls everything that waits on it,
  and a wrong threshold would stall work for no reason.
- **D:
  a lower standing cap.**
  Write a smaller number into `~/.config/claude-cpu-cap/threads` (the owner's knob;
  the runbook says tone down two threads
  at a time).
  Cheap and reversible,
  but covers only the `claude-code-bash` cgroup unless layer A forces every unit and container
  to carry its own quota.
- **E:
  rules and brief discipline** (done as a stopgap).
  `NLG` forbids synthetic host load,
  `NLD` requires every subagent brief to quote the ban and the delegating session to stop
  leftovers before closing a worktree,
  and the `systemd-run` advice now says to add a CPU quota.
  A rule only works while the agent follows it.

## Ranking

A + B > C > D > E alone.

- A and B together beat C:
  they stop the cause (a load nobody needs) before it costs heat,
  and A is in the repo while B needs
  only a small change to a service that already runs;
  C reacts after heat has built and can stall legitimate work.
- C beats D:
  C reacts to what the machine does,
  D only changes a limit that most of the load never passes through.
- D beats E alone:
  a number the system enforces beats a sentence an agent must remember.

## Risks

- B kills processes:
  restrict the match to the denylist and to Claude-owned cgroups and transient units,
  log every kill,
  add an opt-out file for deliberate stress tests,
  and test with a positive control (a `stress-ng` unit must be killed within
  seconds) and a negative control (a long build must survive).
- A can deny a legitimate command that contains a denylisted word;
  the denial message must say why and what to run instead.
- C can stall a long build;
  start with a conservative trigger and a thaw rule,
  and log every freeze.
- None of the layers covers a busy loop written inline in a language runtime;
  layer C is the only one that reacts to it.

## Verification plan, before anything is accepted

1. Start `stress-ng --cpu 2 --timeout 60s` in a `systemd-run --user` unit and show layer B kills it and logs it,
   and that `sleep 60` and a `cargo build` in a unit are not killed.
2. Ask a subagent to run `bun test` (already denied by the hook) and show whether the hook fires for the subagent's Bash call;
   repeat with a denylisted command after layer A exists.
3. Check the hook interface in the installed Claude Code (`claude-code-guide` agent or the installed type definitions)
   for an input-rewriting field before relying on prompt injection.
4. Run layer C against a recorded high-temperature episode (`package/claude-cpu-cap/daemon/src/analyze-episode.ts`)
   to see how often it would have frozen work on ordinary days.
