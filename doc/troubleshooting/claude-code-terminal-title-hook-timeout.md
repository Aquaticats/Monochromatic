# Claude Code 2.1.283 `cctt` hook times out on Btrfs commit stalls and Node 26 thread-start hangs

## Status

Diagnosed on 2026-09-27.
The fix is not chosen yet;
the open decisions are in "Open decisions".
This file records the evidence so a later session does not re-derive it.

## Symptom

Claude Code 2.1.283 printed this notice on a prompt submit
(verbatim,
including Claude Code's own dash):

```text
UserPromptSubmit hook [cctt] timed out after 30s — output discarded. Raise the hook's "timeout" to allow more time.
```

`cctt` is the terminal-title hook.
`.claude/settings.local.json` registers it,
with no `timeout` field,
on `SessionStart`,
`InstructionsLoaded`,
`UserPromptSubmit`,
`PreToolUse`,
`PermissionRequest`,
`PostToolUse`,
`PostToolUseFailure`,
`Notification`,
`SubagentStart`,
`SubagentStop`,
`Stop`,
`TeammateIdle`,
`TaskCompleted`,
`ConfigChange`,
`WorktreeCreate`,
`WorktreeRemove`,
`PreCompact`,
and `SessionEnd`.
The command resolves to `node_modules/.bin/cctt`,
a shim that execs Node v26.10.0 (mise) on
`package/claude-code-plugin/terminal-title/bundle/node/index.mjs`.

The Claude Code hooks reference (fetched 2026-09-27 from `https://code.claude.com/docs/en/hooks.md`)
sets the default timeouts:

```text
Defaults: 600 for `command`, `http`, and `mcp_tool`; [...] Claude Code lowers the `command`, `http`,
and `mcp_tool` default to 30 on UserPromptSubmit
```

So `UserPromptSubmit` is only the event where a stall becomes visible.
On `PreToolUse` and `PostToolUse` a stalled `cctt` holds the tool call for up to 600 seconds
without any notice.

## Root cause

Two separate incidents share the symptom.
They stay separate:
their triggers,
times,
and evidence differ.

### Incident A: 2026-09-27 00:28:05 EDT, Btrfs commit stall blocks the logger's file creation

This incident produced the notice in "Symptom".

#### Chain

1.  The committed bundle predates the lazy default logger.
    `git log` shows `bundle/node/index.mjs` last changed in `ece5b7553` (2026-07-15),
    while `b333197d5` (2026-09-06),
    `fix(module-logger): build the default logger on first use, not at import`,
    landed later.
    The bundle was never rebuilt after that fix.
2.  The stale bundle creates a log file on every run,
    even when nothing is logged.
    Traced under a pseudo-terminal so `/dev/tty` opens like in a real session:

    ```text
    mkdir("/var/home/user/Monochromatic/node_modules/.monochromatic", 0777) = -1 EEXIST (File exists)
    openat(AT_FDCWD, "/dev/tty", O_WRONLY|O_CREAT|O_TRUNC|O_CLOEXEC, 0666) = 22
    openat(AT_FDCWD, ".../node_modules/.monochromatic/2026-09-27T04-39-15.782Z.log.jsonl",
           O_WRONLY|O_CREAT|O_APPEND|O_CLOEXEC, 0666 <unfinished ...>
    write(22, "\33]0;\342\234\263 Received prompt: timing "..., 38) = 38
    ```

    The created file holds only the verification probe:
    `{"test":true,"timestamp":1790483955782}`.
3.  The probe comes from the file sink's verification,
    `package/logger/core/src/sink/file.ts:190-233`:

    ```ts
    // package/logger/core/src/sink/file.ts
    await mkdir(
      LOG_DIR,
      { recursive: true, },
    );
    // ...
    const testData = `{"test":true,"timestamp":${Date.now()}}\n`;
    await appendFile(
      filePath,
      testData,
    );
    ```

4.  `/var/home` is Btrfs (`bazzite_bazzite`,
    UUID `1bb3d23e-ec16-473a-b932-0deb3c450b94`)
    with qgroups on:
    `/sys/fs/btrfs/<uuid>/qgroups/` reports `enabled=1`,
    `mode=qgroup`,
    `inconsistent=1`.
    Its `commit_stats` reported `max_commit_ms 246996`
    and `total_commit_ms 42758935` over `91112` commits.
    Creating a file joins the running transaction,
    so `mkdir` and `openat(O_CREAT)` wait while a long commit runs.
5.  Node keeps the event loop alive while that filesystem request is pending,
    so `cctt` outlives the 30-second budget.

#### Evidence for the stall window

Every logger file named from `2026-09-27T04-28-05.139Z` to `2026-09-27T04-28-47.379Z`
has a modification time between `00:28:52.469` and `00:28:52.509` EDT,
and several are empty.
File creation in `node_modules/.monochromatic/` was blocked until 00:28:52.47.

The stall reached beyond this directory.
journald stamped a Chromium message whose own clock reads `0927/002841.654826`
at `00:28:52.471401`,
and recorded no other entries between `00:28:04.343` and `00:28:52.471`.

#### Positive control

Delaying only `mkdir`/`mkdirat` by 8 seconds reproduces a stall-length hook run:

```sh
# $SCRATCH/ups.json holds {"hook_event_name":"UserPromptSubmit",...,"prompt":"timing probe"}
/usr/bin/time --format='%e s wall' cctt < "$SCRATCH/ups.json"
# 0.02 s wall
/usr/bin/time --format='%e s wall' strace --follow-forks --quiet=all --output=/dev/null \
  --inject=mkdir,mkdirat:delay_enter=8s cctt < "$SCRATCH/ups.json"
# 8.08 s wall
```

A full syscall trace showed that the only filesystem writes `cctt` makes,
apart from `/dev/null` in the shim and `/dev/tty`,
are the logger's `mkdir` and its `.log.jsonl` file.

#### Current source avoids the write on the success path

Running the current source directly under a pseudo-terminal:

```sh
script --quiet --return --command "strace --follow-forks --trace=mkdir,mkdirat,openat,write \
  --output=$SCRATCH/strace-src.txt node package/claude-code-plugin/terminal-title/src/index.ts \
  < $SCRATCH/ups.json" /dev/null
```

The trace contains the `/dev/tty` open and the OSC write,
and no `mkdir` or log-file open.
When `/dev/tty` fails (no controlling terminal),
`package/claude-code-plugin/source/src/handler/terminal-title/index.ts:193`
logs a debug record,
and that first record still creates a log file.

#### Other hook plugins

The same pseudo-terminal `strace` harness,
fed a `UserPromptSubmit` payload to `cccr`
and a `PreToolUse` `Bash` payload (`git status`) to `ccgr` and `ccbof`:

- `cccr` (bundle last changed in `ece5b7553`):
  no `mkdir` and no log file.
- `ccgr` and `ccbof` (bundles last changed in `ece5b7553`):
  each created a log file holding only the probe record.
- `ccgr` and `ccbof` current source,
  run with `node package/claude-code-plugin/<name>/src/index.ts`:
  normal hook output and no log file.
- `ccssh` was not run,
  because its housekeeping mutates the workspace.
- `ccsr`'s bundle was rebuilt in `d80bc4216` (2026-09-25),
  after the lazy-logger fix.

#### Stall history

The logger names each file with the time taken just before `open(O_CREAT)`,
so `birthtime - name time` measures how long file creation waited.
Counts of files whose creation waited more than 30 seconds,
by local day,
over 528,490 files (`$SCRATCH/stall-history.ts`):

- 2026-07-05 to 2026-09-23:
  between 0 and 23 per day,
  maximum wait 229.1 seconds (2026-08-29).
- 2026-09-24:
  54,
  maximum 52.7 seconds.
- 2026-09-25:
  6,
  maximum 37.0 seconds.
- 2026-09-26:
  321,
  maximum 180.7 seconds.
- 2026-09-27 (to 00:40):
  68,
  maximum 72.2 seconds.

Hook-created log files per day rose over the same span:
21,488 on 2026-09-23,
41,678 on 2026-09-25,
and 49,003 on 2026-09-26.
The directory holds 528,490 files using 11 GB.
Whether that file churn lengthens commits is not measured.

The Btrfs stall itself is tracked in
[Bazzite desktop stalls](bazzite-desktop-input-stalls.md)
and the [Bazzite exit handover](../handover/leave-bazzite-cachyos-btrfs.md);
this file covers only how the hooks are exposed to it.

### Incident B: 2026-09-26 00:55:54 EDT, Node 26.10.0 hangs at startup when a platform worker thread cannot be created

A `cctt` process started at 00:55:54 on 2026-09-26 was still alive on 2026-09-27:
PID 2224691,
reparented to `systemd --user`,
3 threads,
cgroup `app-ghostty-surface-transient-4113802.scope`.
`eu-stack --pid=2224691` showed the main thread inside platform setup,
before any JavaScript ran:

```text
#3  pthread_cond_wait@@GLIBC_2.3.2
#4  uv_cond_wait
#5  node::WorkerThreadsTaskRunner::WorkerThreadsTaskRunner(int, node::PlatformDebugLogLevel)
#6  node::NodePlatform::NodePlatform(int, v8::TracingController*, v8::PageAllocator*)
#7  node::V8Platform::Initialize(int)
```

Only one `node-V8Worker` thread existed.
The kernel log shows a thread-creation refusal in that same scope one second later:

```text
Sep 26 00:55:55 bazzite kernel: cgroup: fork rejected by pids controller in
  /user.slice/user-1000.slice/user@1000.service/app.slice/app-ghostty-surface-transient-4113802.scope
```

At that time both `TasksMax` on `user-1000.slice`
(`/etc/systemd/system/user-.slice.d/20-freeze-hardening.conf`)
and `nproc` (`/etc/security/limits.d/90-freeze-hardening.conf`) were 8192.
`user-1000.slice/pids.events.local` reports `max 23347`.
Both were raised to 12288 at 01:15:58 the same night;
see [Fedora 44 user task ceiling](fedora-44-fold-emulator-user-task-ceiling.md).

Node v26.10.0 `src/node_platform.cc:256-277` stops creating workers on the first failure
but still waits for the full count:

```cpp
// src/node_platform.cc (v26.10.0)
  for (int i = 0; i < thread_pool_size; i++) {
    // ...
    if (uv_thread_create(t.get(), PlatformWorkerThread, worker_data.get()) !=
        0) {
      break;
    }
    worker_data.release();
    threads_.push_back(std::move(t));
  }

  // Wait for platform workers to initialize before continuing with the
  // bootstrap.
  while (pending_platform_workers > 0) {
    platform_workers_ready.Wait(lock);
  }
```

`pending_platform_workers` starts at `thread_pool_size`,
and only started workers decrement it,
so one failed `uv_thread_create` makes the wait endless.
The same code is at `src/node_platform.cc:258-279` on `main` (fetched 2026-09-27).

Why the Claude Code timeout did not end this process is unknown.
`/proc/2224691/status` showed SIGTERM neither blocked nor ignored,
but caught (`SigCgt: 0000000100004002`).
On 2026-09-27 `kill -TERM 2224691` ended it at once
(`ps -p 2224691` printed no process right after),
so a timeout that delivers SIGTERM does end this hang;
the 2026-09-26 process apparently never received one.

Incident B did not cause incident A.
The kernel logged no pids rejection after 2026-09-26 02:21:15,
and `user-1000.slice/pids.peak` is 8539,
below the current 12288 limits.

### Readings that were wrong

- Finding the day-old orphan first suggested the thread-limit hang explained today's notice.
  The limit history and the absence of rejections after 2026-09-26 02:21:15 rule that out
  for incident A.
- A first trace ran without a controlling terminal,
  where the `/dev/tty` open fails and the debug record alone forces the file sink,
  which suggested the real success path might not touch the disk.
  The pseudo-terminal trace in "Chain" shows the stale bundle writes the probe file
  on the success path too.

## Verification

- Claude Code 2.1.283;
  Node v26.10.0 (`/home/user/.local/share/mise/installs/node/26.10.0/bin/node`);
  kernel 7.2.0-ogc6.1.fc44;
  repository HEAD `d2bb74ac4`;
  bundle last changed in `ece5b7553`.
- Harnesses:
  the `time` and `strace --inject` commands in "Positive control",
  the `script` plus `strace` command in "Current source avoids the write on the success path",
  and `$SCRATCH/hook-kill-probe.ts` for the hook-mode catalog.

### Runs that complete promptly

- Bundle,
  no injected delay:
  0.02 seconds.
- Current source under a pseudo-terminal:
  no filesystem write besides `/dev/tty`.

### Runs that stall

- Filesystem stall variant:
  bundle with `mkdir` delayed 8 seconds took 8.08 seconds;
  bundle during the 2026-09-27 00:28 Btrfs stall waited about 47 seconds.
- Thread-creation variant:
  Node never finishes startup (PID 2224691,
  alive for more than 24 hours).

### Hook-mode catalog

`claude -p` with `claude-haiku-4-5-20251001` in a throwaway directory,
`UserPromptSubmit` hooks running `sleep`,
`ps` sampled every 250 ms:

- Synchronous,
  `timeout: 3`:
  the `sleep` process was gone about 3 seconds after it started.
- `asyncRewake: true`,
  `timeout: 3`:
  gone after about 3 seconds.
- `async: true`,
  `timeout: 3`:
  alive until `claude -p` exited,
  matching the documented "doesn't enforce `timeout`".
- `asyncRewake: true`,
  `timeout: 20`,
  alone:
  the first streamed `assistant` event arrived at 23.8 seconds,
  after the hook was killed at about 23 seconds.
  In `-p` mode it held the prompt,
  although the docs say it "runs in the background".
- `async: true` alone:
  the first streamed `assistant` event arrived at 4.0 seconds while the hook was still alive.

Interactive sessions were not probed.

## Workarounds

None is applied yet.
Each candidate names what it was verified against and what it costs.

### Rebuild the bundle from current source

Verified by running the current source directly:
the success path makes no filesystem write.
Cost:
the no-terminal path still creates a log file through the debug record;
the bundle is committed output and must be rebuilt again after later logger changes.

### Run `cctt` hooks with `async: true`

Verified non-blocking in `-p` mode.
Cost:
no timeout is enforced,
so an incident B hang lives until the session ends
(interactive teardown not probed);
title writes from separate processes can land out of order after a stall releases them together.

### Set a short explicit `timeout`

Verified that the process is killed at the timeout for synchronous hooks.
Cost:
each event still blocks for up to that long during a stall.

## What does not work

- Raising `timeout`,
  as the notice suggests:
  measured create stalls reached 180.7 seconds,
  and incident B never ends,
  so a longer budget only lengthens the blocked prompt or tool call.
- `asyncRewake` as a non-blocking hook with an enforced timeout:
  in `-p` mode it held the prompt until the hook ended.

## Upstream filing decision

### Claude Code

`.out-of-scope/claude-code-upstream-bugs.md` exempts Claude Code defects from upstream filing,
including the `asyncRewake` behavior in the hook-mode catalog.
Nothing is filed.

### Node.js

No `.out-of-scope/` entry covers Node.js.
Tracker searches on 2026-09-27
(`WorkerThreadsTaskRunner`,
`uv_thread_create hang`,
`hang startup thread create EAGAIN`,
`pids limit hangs`,
`pending_platform_workers`,
and the PR search `platform worker thread creation failure`)
found no duplicate.
The closest,
nodejs/node#53919,
reports an assertion in `DelayedTaskScheduler::Start()`,
a different failure.

1.  Upstream's fault:
    yes;
    the constructor waits for workers it never started.
2.  Upstream can fix it:
    yes;
    count only started workers or fail startup with an error.
3.  Supported use case:
    running under a thread or process limit is ordinary (containers with `--pids-limit`);
    pending a documentation citation.
4.  Contribution welcome:
    `CONTRIBUTING.md` and AI-assistance policy not yet read.
5.  Likely to fix:
    no negative signal found;
    the #53919 thread not yet read in full.
6.  Prototype:
    not yet;
    it needs a Node source build in a bounded container.

Constraints 4 to 6 are open,
so the draft stays unfiled.

~~~md
<!-- do not file as-is: constraints 4 to 6 are open -->
Title: Startup hangs forever when a platform worker thread fails to start

`WorkerThreadsTaskRunner::WorkerThreadsTaskRunner` (src/node_platform.cc:258-279 on main)
breaks out of the worker-creation loop on the first `uv_thread_create` failure, then waits
until `pending_platform_workers` reaches zero. Workers that were never created never
decrement it, so the process blocks in `uv_cond_wait` before any JavaScript runs.

Reproduction: run `node -e 0` in a cgroup whose `pids.max` admits the main thread, the
delayed-task scheduler and one worker, but not the rest of the pool.

Observed on v26.10.0 (Linux 7.2, pids controller refusal logged by the kernel):
main thread in `WorkerThreadsTaskRunner::WorkerThreadsTaskRunner` -> `uv_cond_wait`,
one `node-V8Worker` thread present, process alive for more than 24 hours.

Suggested fix: after the loop, subtract the workers that were not started from
`pending_platform_workers` (or wait only for `threads_.size() - 1`), and either continue
with the smaller pool or fail startup with a clear error.
~~~

## Open decisions

The user is choosing between fixes in an interview session;
record each answer here as it lands.
