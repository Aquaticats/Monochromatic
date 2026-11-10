# Planning: Kopia source watch package

Status:
 design plan only.
 The earlier per-source systemd unit generator shape is rejected.
 The
package should be one supervised daemon using Chokidar directly.

Date of plan:
 2026-05-31.

## Problem

Kopia supports policy schedules through `kopia server start`,
 but that command starts an
HTTP and gRPC server.
 That is more surface area than needed for local automatic snapshots.

The current operational baseline is a user systemd timer that runs:

```shell
kopia snapshot create --all
```

That baseline is robust and simple,
 but it has two limitations:

- Every run rescans all local Kopia sources,
   even when only one source changed.
- The cadence is time-based,
   not file-change-based.

An event-triggered layer should reduce unnecessary rescans without turning Kopia into a
network listener or generating systemd units for every source.

## Goals

- Keep Kopia repository state as the source of truth for watched backup sources.
- Run as one user-level daemon process.
- Use Chokidar directly for recursive filesystem watching.
- Reconcile sources dynamically inside the daemon.
- Trigger `kopia snapshot create <source>` for only the source that changed.
- Keep a periodic `snapshot create --all` timer as a safety net for missed events and source
  changes.
- Make status and logs visible through the daemon log and journald.

## Non-goals

- Do not replace Kopia's retention,
   ignore,
   compression,
   or maintenance policies.
- Do not run `kopia server start`.
- Do not expose a local HTTP service.
- Do not generate per-source systemd units.
- Do not spawn one watcher process per source.
- Do not attempt real-time backup on every write.
   Backups should run after a debounce window.
- Do not watch sources that do not exist locally.

## Current measured shape

Current local Kopia sources discovered from `kopia snapshot list --json --max-results=1`:

- `/home/user/Downloads`
- `/home/user/Seafile/Plain`
- `/var/mnt/encrypted/Archive`

Current recursive watch cost estimate:

- Watched directories:
   10,406.
- Kernel `fs.inotify.max_user_watches`:
   524,288.
- Watch usage:
   about 2 percent of the current kernel watch limit.
- `watchexec` trial RSS:
   about 55 MB.
- Settled idle CPU over a 5 second interval:
   0.0 percent.

The watcher cost tracks directory count,
 not file count.
 The expensive action remains the Kopia
snapshot scan,
 so debouncing matters more than watcher overhead for the current source set.

## Package shape

Create `package/cli/kopia-source-watch/`.

The package exposes a CLI named `kopia-source-watch` with these commands:

- `kopia-source-watch daemon`:
   run the long-lived watcher and snapshot scheduler.
- `kopia-source-watch list`:
   print discovered local Kopia sources and current watch eligibility.
- `kopia-source-watch run-source <path>`:
   run one source snapshot through the same command builder
  used by the daemon.
- `kopia-source-watch run-all`:
   run `kopia snapshot create --all` through the same command builder
  used by the daemon.
- `kopia-source-watch check`:
   validate Kopia,
   Chokidar configuration,
   source discovery,
   and system
  limits.

The package should follow normal workspace package requirements:

- `package.json` with a `bin` entry.
- `mise.toml` with lint,
   type,
   test,
   and build tasks matching sibling CLI packages.
- `README.md` before the package is considered complete.
- Unit tests for source discovery,
   source matching,
   debounce state,
   queue behavior,
   Chokidar event
  handling,
   and command rendering.

## Daemon responsibilities

The daemon owns four loops:

1.  Source discovery loop.
2.  Chokidar event loop.
3.  Debounce timer loop.
4.  Snapshot worker loop.

The daemon should not ask systemd to represent each source.
 Systemd supervises only the daemon.
The daemon represents sources as in-memory state:

```text
source path -> idle | debounce-pending | queued | snapshot-running | rerun-queued | disabled
```

When a filesystem event maps to a source:

1.  If the source is idle,
     start its debounce timer.
2.  If debounce is already pending,
     extend or keep the pending deadline.
3.  If the source is running,
     mark one rerun as queued.
4.  If the source is disabled,
     ignore the event and log the reason.

When a debounce deadline fires:

1.  Add the source to the snapshot queue.
2.  Run at most one Kopia snapshot by default.
3.  Execute `kopia snapshot create <source>`.
4.  If changes arrived while the snapshot ran,
     schedule exactly one follow-up debounce.

Default global snapshot concurrency:
 1.

Reasoning:

- Kopia snapshots are disk and repository intensive.
- Concurrent source snapshots can compete for disk and cache.
- A single worker still preserves per-source targeting while avoiding burst load.

## Source discovery

The daemon should discover sources from Kopia instead of a hand-maintained source list.

Primary source discovery command:

```shell
kopia snapshot list --json --max-results=1
```

The daemon should parse each JSON item and keep entries whose `source.host` and `source.userName`
match the current client.
 If Kopia's default filtering already restricts output to the current
client,
 the explicit filter still protects future command changes.

Source candidates must be validated before watching:

- Source path is absolute.
- Source path exists.
- Source path is a directory.
- Duplicate paths collapse to one source.
- Parent and child source overlap is detected and logged.

Symlink policy should be conservative in the first implementation:

- Use Chokidar with `followSymlinks: false`.
- Watch the symlink itself when a symlink appears under a source.
- Do not follow symlinked directories as separate roots.
- Log symlinked source paths as warnings until a deliberate policy exists.

Discovery cadence:

- Run once at daemon start.
- Run every 15 minutes while the daemon is active.
- Run after `SIGHUP`.
- Run after any successful `run-all` safety-net snapshot,
   if the daemon can observe that action in
  a future integration.

Future enhancement:
 include per-path policies from `kopia policy list --json` when a path policy
exists before the first snapshot.
 The first implementation can rely on snapshot sources because
`kopia snapshot create --all` also operates on files or directories previously backed up by this
user on this computer.

## Watcher backend choice

### Recommended: Chokidar

Use Chokidar as a library inside the daemon.

Source-audited facts from Chokidar 5.0.0:

- `package.json` declares MIT license,
   ESM-only package shape,
   one runtime dependency
  (`readdirp`),
   and Node `>= 20.19.0`.
- README documents recursive watching for path arrays,
   `ignoreInitial`,
   `followSymlinks`,
  `usePolling`,
   `alwaysStat`,
   `depth`,
   `awaitWriteFinish`,
   `ignorePermissionErrors`,
   `atomic`,
  `.add()`,
   `.unwatch()`,
   `.close()`,
   and `.getWatched()`.
- Source `src/index.ts` exposes `.add()`,
   `.unwatch()`,
   async `.close()`,
   and `.getWatched()` on
  `FSWatcher`.
- Source `src/handler.ts` follows the configured `followSymlinks` behavior and recursively adds
  directory watches through the Node filesystem handler.
- README warns that Chokidar creates recursive watchers for everything in scope and recommends
  avoiding broader watch roots than needed.
- README troubleshooting names `EMFILE` and `ENOSPC` as file-handle and inotify-watch exhaustion
  cases,
   and names `fs.inotify.max_user_watches` as the Linux watch limit to inspect or tune.

Pros:

- Open source.
- Library integration keeps source reconciliation,
   event routing,
   debouncing,
   and snapshot queuing
  in one process.
- No generated per-source systemd units.
- No watcher child process.
- `awaitWriteFinish` and `atomic` cover chunked writes and editor atomic-write patterns.
- `.add()` and `.unwatch()` support dynamic source reconciliation without daemon restart.
- `.getWatched()` supports status and diagnostics.

Cons:

- Chokidar 5 requires Node `>= 20.19.0`;
   runtime compatibility with the repo's Node CLI path
  must be tested before implementation commits to Node execution.
- Recursive watching still consumes one or more underlying watches per directory on Linux.
- `awaitWriteFinish` uses polling of file size for pending writes and can reduce responsiveness
  when the stability threshold is high.
- Network or unusual filesystems may need `usePolling: true`,
   which Chokidar documents as more
  resource intensive.

### Alternative: raw Node `fs.watch`

Pros:

- No external package.
- Node documentation exposes a `recursive` option on supported platforms.
- Direct integration with daemon state.

Cons:

- Chokidar exists specifically to normalize raw `fs.watch` and `fs.watchFile` behavior.
- Raw events would push atomic-write,
   chunked-write,
   duplicate-event,
   and recursive edge cases into
  this package.
- This package would need to rebuild features Chokidar already exposes.

### Alternative: watchexec event stream

Pros:

- Open source.
- Already installed in the current environment.
- Uses native filesystem notifications by default.
- Supports recursive watches and structured event output.

Cons:

- Adds a watcher child process.
- Source set changes require restarting the child with a new argument list.
- Event routing crosses a process boundary even though the package can use a library instead.

### Rejected: systemd.path

Pros:

- Native systemd unit type.
- No extra watcher binary or library.

Cons:

- systemd path units use inotify and are not recursive for deep source trees.
- systemd documentation says hidden files whose names start with a dot are generally ignored when
  monitoring paths.
- It pushes source reconciliation back into systemd unit generation,
   which is the rejected shape.

### Rejected: raw inotifywait loop

Pros:

- Open source.
- Installed in the current environment.
- Supports recursive watching.

Cons:

- Requires custom debounce,
   queueing,
   event parsing,
   and shutdown handling.
- Adds low-level process and text-output parsing around behavior Chokidar exposes as a library.

### Rejected as event layer: periodic timer only

Pros:

- Simple and robust.
- No long-running watcher process.
- Handles new sources without reconciliation.

Cons:

- Not event-triggered.
- Rescans inactive sources.
- Higher latency between change and snapshot when the interval is large.

## Chokidar configuration

Initial daemon watcher options:

```ts
{
  persistent: true,
  ignoreInitial: true,
  followSymlinks: false,
  usePolling: false,
  awaitWriteFinish: {
    stabilityThreshold: 30_000,
    pollInterval: 1_000,
  },
  atomic: true,
  alwaysStat: false,
  ignorePermissionErrors: false,
}
```

Rationale:

- `ignoreInitial: true` prevents daemon startup from scheduling snapshots for every existing file.
- `followSymlinks: false` avoids backing up targets outside the configured Kopia source root due
  only to watcher traversal.
- `usePolling: false` keeps the default non-polling backend for local filesystems.
- `awaitWriteFinish` reduces snapshots of partially written large files.
- A 30 second stability threshold is short enough to feed the daemon-level debounce while avoiding
  obvious partial-write triggers.
- `ignorePermissionErrors: false` surfaces watch coverage problems instead of silently hiding them.

The daemon should still apply its own source-level debounce after Chokidar emits an event.
Chokidar's `awaitWriteFinish` protects file completeness;
 daemon debounce protects repository and
disk load.

## Systemd integration

Systemd should supervise exactly one daemon service for this package:

```ini
[Unit]
Description=Watch Kopia sources and trigger targeted snapshots
Documentation=https://kopia.io/docs/

[Service]
Type=simple
ExecStart=<kopia-source-watch> daemon
Restart=on-failure
RestartSec=30s
Nice=10
IOSchedulingClass=best-effort
IOSchedulingPriority=7

[Install]
WantedBy=default.target
```

This unit can live in package documentation or packaging output.
 It should not be generated once
per source.

The hourly safety net remains separate:

```ini
[Timer]
OnActiveSec=1min
OnUnitActiveSec=1h
Unit=kopia-snapshot-all.service
```

The safety net covers:

- missed filesystem events,
- watcher crashes,
- sources that changed before watches finished establishing,
- newly added sources before the daemon discovers them,
- filesystems whose events are not fully visible through inotify.

## Debounce and concurrency defaults

Chokidar write-finish threshold:
 30 seconds.

Reasoning:

- The watcher should avoid events for obvious partial writes.
- The daemon still owns source-level backup debounce.

Source backup debounce:
 10 minutes.

Reasoning:

- Shorter windows increase snapshot scans during active editing or sync bursts.
- Longer windows reduce repository and disk churn.
- The hourly `--all` timer remains the upper-bound safety net.

Busy behavior:
 queue one rerun per source.

Reasoning:

- Restarting a running backup wastes work.
- Ignoring changes while a backup runs can miss a source that changes near the end of a scan.
- Queueing one rerun preserves correctness without unbounded backlog.

## Observability

The daemon should log these events:

- startup configuration,
- Kopia binary path,
- Chokidar version,
- discovered sources,
- skipped sources and reasons,
- source additions and removals,
- watched directory count from `.getWatched()`,
- debounce start and fire,
- snapshot command start and exit,
- queued rerun,
- source discovery errors,
- Chokidar errors,
- `EMFILE` and `ENOSPC` watch-limit errors with the current `fs.inotify.max_user_watches` value.

A lightweight state file can support `kopia-source-watch status` without exposing a network API.
Use `$XDG_STATE_HOME/kopia-source-watch/state.json` or `$HOME/.local/state/kopia-source-watch/state.json`.

## Testing plan

Tests must cover:

- Parsing `kopia snapshot list --json` output with one source,
   multiple sources,
   duplicate sources,
  and non-local host or user entries.
- Rejection of relative paths,
   missing paths,
   files,
   and duplicate paths.
- Parent and child source overlap detection.
- Mapping event paths to the most specific source root.
- Source state transitions for idle,
   debounce-pending,
   queued,
   snapshot-running,
   rerun-queued,
   and
  disabled.
- Snapshot worker behavior with global concurrency 1.
- Chokidar `.add()` and `.unwatch()` calls when sources change.
- Chokidar `add`,
   `addDir`,
   `change`,
   `unlink`,
   and `unlinkDir` event handling.
- Command rendering with argument boundaries preserved.
- Dry-run source discovery and check output.
- Runtime compatibility under the package's selected execution runtime.

Integration tests should use temporary directories and a fake Kopia executable that records argv.
The package should not run destructive or state-mutating verification against the real user
repository.

## Operational rollout

1.  Keep the hourly `kopia snapshot create --all` timer enabled.
2.  Build and test the package.
3.  Run `kopia-source-watch list` and inspect source discovery.
4.  Run `kopia-source-watch check`.
5.  Start the daemon manually with a temporary source and fake Kopia binary.
6.  Start the daemon against real sources after fake-binary behavior passes.
7.  Touch a file in one source and confirm only that source's snapshot command runs after debounce.
8.  Leave the hourly `--all` timer enabled for at least one week.
9.  Reassess whether the fs-change layer reduces enough work to justify keeping it.

## Open questions

- Should `kopia-source-watch` live under `package/cli/` or `package/dev-script/`?
   Current plan
  uses `package/cli/` because the result is an operator-facing command.
- Should the daemon include its own periodic `run-all` loop later,
   or should systemd keep owning
  the safety-net timer?
- Should the daemon expose a polling mode for network or FUSE filesystems?
- Should source-specific snapshots suppress identical snapshot manifests through Kopia policy,
   or
  should the package leave that entirely to user policy?
