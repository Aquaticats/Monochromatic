# Node 26.13.0: a process that cannot start a thread under a full `pids.max` aborts or sleeps in `futex_do_wait` for good, so an unbounded parent wait never ends

## Symptom

`package/monorepo-task/core-exec/src/depends.unit.test.ts` ran for over half an hour in the toolchain-update rehearsal of 2026-11-07
(recorded in `doc/planning/toolchain-always-latest.md`, section on the `core-exec` test run, and stopped without a cause).
Run alone on an idle host the same file ends in about 5 s.

The hang reproduces when the file runs in a control group whose task limit (`pids.max`, systemd `TasksMax=`) is smaller than its peak.
The file's peak is 291 tasks, measured on 2026-11-07 with `pids.peak` read inside a `systemd-run --user --scope` run with `TasksMax=2000`.

Two shapes appear in the same run:

```text
Error: spawn /bin/sh EAGAIN at ChildProcess._handle.onexit (node:internal/child_process:315:19)
```

is the parent failing to start a child, and it is reported as an ordinary failed test.
The hang is the other shape: child `node` processes that started but never finish.

## Root cause

Every process and every thread is one task, and `fork` and `clone` fail with `EAGAIN` when a `pids.max` in the cgroup ancestry is full
(the same mechanism as in `doc/troubleshooting/agent-browser-eagain.md`, section "The first shape").

Measured on 2026-11-07 with `TasksMax=150` and `depends.unit.test.ts` as a systemd unit:

- Tasks in the unit later fell to 40 of 150, so the ceiling was no longer full and the processes were still stuck.
- Five `node` processes, among them the marker command `node -e "require('fs').writeFileSync(...)"`, were in state `S` with `wchan=futex_do_wait` and two to four threads
  (a healthy Node holds about eleven).
- One `node` of the test file died with `SIGABRT`; `coredumpctl info` showed the frames
  `abort`, `init_once`, `uv_once`, `uv__work_submit`, `uv_fs_scandir`, `node::fs::ReadDir`
  (a worker-thread start failing inside libuv's one-time pool setup).
- The parent of a stuck child (`node depends.ts`, `ep_poll`, eleven threads) waits for it without end,
  and the test's `nano-spawn` call waits for that parent, because none of these waits had a timeout.

The outcome is a race over which child hits `EAGAIN` where: the first probe at the same cap aborted in about 2 s,
a second ran to the outer `timeout 120` (exit 124), and three further probes at 150, 200 and 250 were still running after 12 s with stuck children.

The libuv and Node sources were not read, so which call sites abort and which leave a partially started pool asleep is not traced here.
Whether the rehearsal's cgroup was at its ceiling is not recorded: the rehearsal logs (`~/temp/agent/tal/`) hold no `TasksMax`, `EAGAIN` or `pids` line,
and the shared per-user ceiling has been reached before (`doc/troubleshooting/agent-browser-eagain.md`).
The pids-ceiling hang is the best match for the symptom, not a confirmed replay.

## Verification

Harness, run from `package/monorepo-task/core-exec` in a scratch worktree, every run inside `systemd-run --user --scope` with `CPUQuota`, `MemoryMax`, `TasksMax` and `IOWeight=10`:

```sh
# Idle control: passes in about 5 s
systemd-run --user --scope --property=TasksMax=500 timeout 300 node src/depends.unit.test.ts

# Ceiling below the file's peak: hung before the fix
systemd-run --user --scope --property=TasksMax=150 timeout 120 node src/depends.unit.test.ts
```

Before the fix: exit 124 at 120 s (the outer bound), four tests passed and the rest were pending or failed with `EAGAIN`.
After the fix (commit message of `fix(monorepo-task-core-exec)` on `fix/monorepo-task-core-exec-depends-test-time`): the same command ends at 60.9 s with exit 1
and 47 `FAIL` records, with no process left in the unit; the 60 s is the test file's own spawn bound.

## Verified workarounds

Bound every wait on a child process, so a child that never exits turns into a failure:

- Spawn the child detached (its own process group) and, when the bound passes, kill the group with `process.kill(-pid, 'SIGKILL')`
  (`spawnBounded` in `package/monorepo-task/core-exec/src/depends-bounded-wait.ts`).
  `SIGKILL` is needed: a child stuck before it installs a handler, or one that ignores `SIGTERM`, keeps the wait alive.
  Killing only the child is not enough with `shell: true`: the other members of a pipeline survive and their open pipe keeps the wait going.
  Tradeoffs: the detached group no longer receives the terminal's interrupt, so the parent passes `SIGINT`, `SIGTERM` and `SIGHUP` on to it;
  Windows has no process groups, so only the child itself ends there (not tested).
- Give test files their own bound on every spawned command (`COMMAND_TIMEOUT_MS` in `depends.unit.test.ts`).
  Tradeoff: under a ceiling the file fails after the bound instead of passing, which is correct but still slow.

## What does not work

- Reading the hang as slowness: the file's own run time is seconds, and the stuck children use no CPU.
- Waiting for a quiet host: the ceiling, not the load, causes it.
- The first full-package run of the day failed in nine files because `dist` was not built; that is unrelated and ended after `buildAndTest`.

## Upstream filing decision

Not filed, and no recommendation to file is made.
The Node and libuv sources were not read, no minimal fix was prototyped, `.out-of-scope/`, the upstream tracker and any trust system were not searched, and the six-constraint check was not run;
default policy is not to file.
Filing would put the poster's account in front of strangers, and trust or blocklist systems exist that can follow a person across projects, so it is the poster's decision.
