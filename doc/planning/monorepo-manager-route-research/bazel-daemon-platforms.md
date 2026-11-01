# Bazel 9 watch daemon design (Route A)

## Status, scope, and evidence labels

- Date: 2026-09-16.
- Status: design research only.
  No Bazel was installed or run.
  Nothing under `/var/home/user/Monochromatic` was modified.
- Purpose: turn Route A of `doc/planning/monorepo-manager-build-routes.md` into a concrete design,
  so any disqualifying ("yikes") problem shows up before a route is chosen.
- Requirements covered:
  WR1 (watch keeps task outputs current),
  WR2 (documented RPC or IPC inspection of current and recent activity),
  WR3 (start, rerun, cancel),
  file-enforcer style generated-file enforcement (FE01 to FE24),
  plus the requirement added mid-research on 2026-09-16:
  sandboxing with at least cgroups is a must,
  and 0.x only needs Linux.
- Evidence labels used on every claim:
  - `[V: source:line]` verified by reading that source or doc;
  - `[M: probe]` measured on this machine (Bazzite 44 Kinoite, Linux 7.2, 16 CPUs, 62 GiB RAM);
  - `[U]` unverified: recall, or inference not checked against a source;
    `[U, inferred]` marks reasoning chained from verified facts.
- Sources and clones under `~/temp/agent/`:
  - `bazel-2026-09-16` at `8e90a0d` (`main`, `.bazelversion` 9.2.0);
    latest release is 9.2.0 (2026-07-13), 9.3.0rc1 exists
    [V: `gh release list --repo bazelbuild/bazel`].
    Where `main` and 9.2.0 might differ, 9.2.0 files were fetched by tag and are cited as `@9.2.0`.
    Versioned docs are cited from `docs/versions/9.1.0/`, the newest snapshot in the clone.
  - `bazel-lib-2026-09-16` at `7ca5f6d`,
    `bazel-watcher-2026-09-16` at `ed00d96`,
    `rules_js-2026-09-16` at `3f5022d`,
    `rules_ts-2026-09-16` at `bfbf401`,
    `rules_lint-2026-09-16` at `4cb8c45`,
    `rules_rust-2026-09-16` at `dd2b107`,
    `rules_kotlin-2026-09-16` at `02f96a1`,
    `rules_android-2026-09-16` at `7bd7cbb`.
  - Fetched without cloning (saved under the scratchpad `bzd/` directory):
    googleapis `google/devtools/build/v1/publish_build_event.proto` and `build_events.proto`,
    target-determinator and bazel-diff READMEs,
    `bazel-contrib/setup-bazel` README (release 0.19.0),
    `rules_multitool` readme,
    `bazel_env.bzl` README and source,
    `aspect-build/rules_oxc` README,
    systemd `docs/CGROUP_DELEGATION.md`,
    Linux `Documentation/admin-guide/cgroup-v2.rst`,
    Node.js `doc/api/child_process.md`,
    Bazel Central Registry module list via the git tree API (1,318 modules; the contents API truncates at 1,000).

## Corrections to existing repository docs

These are recorded here only; the repository was not edited.

- `doc/research/bazel-migration-dx.md` cites the client message "Another command holds the ... lock"
  (`src/main/cpp/blaze_util_posix.cc:740-743`) as the reason a second command waits.
  On current code the client releases its output base lock right after starting the `Run` RPC
  [V: `bazel src/main/cpp/blaze.cc:2090-2096`, same text in `blaze.cc@9.2.0:2005`],
  and serialization happens in the server:
  "Another command (%s) is running. Waiting for it to complete on the server"
  [V: `bazel src/main/java/com/google/devtools/build/lib/runtime/BlazeCommandDispatcher.java:229-265`].
  The conclusion (one command at a time per output base) is unchanged.
- `doc/planning/monorepo-manager-build-routes.md` says the sandbox "prevents the action from accidentally using any
- input files that are not declared".
  The same page also says "Processes can freely access all files on the file system"
  and only hides undeclared inputs from tools that do not know absolute paths
  [V: `bazel docs/docs/sandboxing.mdx:14-22`].
  Filesystem-hiding needs the opt-in `--experimental_use_hermetic_linux_sandbox`
  [V: `bazel src/main/java/com/google/devtools/build/lib/sandbox/SandboxOptions.java:340-348`].

## Architecture overview

### Components

- `watchd`:
  a repository-built long-running Node process (TypeScript, per repository conventions).
  It owns every `bazel` client invocation, the file watcher, the event store, the RPC server,
  the generated-file writer, and the cgroup tree.
- Primary Bazel server:
  one explicit `--output_base`, started lazily by the first client call.
  One command at a time
  [V: `bazel docs/run/client-server.mdx:12-14`].
- Optional query server:
  a second `--output_base` used only for graph queries while a build runs
  [V: `bazel docs/run/scripts.mdx:26-33`].
  Off by default in 0.x (memory, see section "Affected targets and queries").
- Build event ingestion:
  per-invocation `--build_event_json_file` tailed by `watchd` (preferred),
  or `watchd` as a Build Event Service backend (alternative).
- Generated-file writer:
  copies Bazel-built files into the source tree with skip-identical atomic writes,
  and reverts external edits to protected destinations.
- Non-hermetic runners:
  `bazel run` targets for provisioning and home-directory settings, executed inside a `watchd` cgroup.

### Process and cgroup layout (Linux 0.x)

`watchd` runs in its own systemd user unit with `Delegate=yes`
(for example `systemd-run --user --scope --property=Delegate=yes`, or a user service with `Delegate=yes`).
Inside that delegated cgroup `<D>`:

- `<D>/daemon`: `watchd` itself (moved there first, because of the no-internal-processes rule).
- `<D>/client`: every `bazel` client process; `bazel run` targets are `exec`ed by the client,
  so they also land here
  [V: `bazel src/main/cpp/blaze.cc:2190-2219` and `src/main/cpp/blaze_util_posix.cc:335,350-354`].
- `<D>/bazel`: controllers `+cpu +memory +pids` enabled; aggregate `memory.max` and `pids.max` set by `watchd`.
  - `<D>/bazel/server`: the Bazel JVM, placed there with
    `startup --experimental_cgroup_parent=<D>/bazel/server`
    [V: `bazel src/main/java/com/google/devtools/build/lib/runtime/BlazeServerStartupOptions.java:512-530`,
    `src/main/tools/daemonize.cc:308-340`].
  - `<D>/bazel/blaze_<serverpid>_spawns.slice`: created by Bazel one level above the server's own cgroup
    [V: `bazel src/main/java/com/google/devtools/build/lib/sandbox/cgroups/VirtualCgroup.java:93,150-157`].
    - `sandbox_<n>.scope`: one per linux-sandbox action when limits are configured
      [V: `bazel src/main/java/com/google/devtools/build/lib/sandbox/cgroups/VirtualCgroupFactory.java:57-76`,
      `src/main/java/com/google/devtools/build/lib/sandbox/LinuxSandboxedSpawnRunner.java:323-330`].
    - `worker_<n>.scope`: persistent workers when `--experimental_worker_use_cgroups_on_linux` is set
      [V: `bazel src/main/java/com/google/devtools/build/lib/worker/WorkerModule.java:122-130`].
- `<D>/query/server` and its sibling spawns slice: the optional query server.

Measured feasibility of this layout without Bazel:
inside `systemd-run --user --scope --property=Delegate=yes`,
an unprivileged Node process moved itself to a leaf,
enabled `+cpu +memory +pids`,
created `bazel/server` and `bazel/blaze_probe_spawns.slice`,
set `memory.max` (268435456) and `pids.max` (4096),
and froze and thawed a ticking child through `bazel/cgroup.freeze`
(`cgroup.events` reported `frozen 1`; 9 ticks before, 0 during a 1 s freeze, 10 after thaw);
systemd removed the scope afterwards
[M: `bzd/cgroup-probe.ts` run under `systemd-run --user --scope --property=Delegate=yes`].

### Fixed Bazel flags

The analysis cache is discarded when flags change between commands,
so every command `watchd` issues uses one flag set
[V: `bazel docs/advanced/performance/iteration-speed.mdx:72-75`].
`--noallow_analysis_cache_discard` turns a discard into an error
[V: `bazel docs/advanced/performance/iteration-speed.mdx:92-93`].

- Startup (changing any restarts the server
  [V: `bazel src/main/cpp/blaze.cc:112-117` treatise: server killed when startup options differ]):
  `--output_base=<explicit>`,
  `--experimental_cgroup_parent=<D>/bazel/server`,
  `--host_jvm_args=-Xmx<to be measured>`.
  Bazel sets no default `-Xmx`
  [V: no `Xmx` in `bazel src/main/cpp/blaze.cc`, `startup_options.cc`, `bazel_startup_options.cc`],
  so the JVM's own default heap sizing applies [U].
- Command:
  `--watchfs`,
  `--spawn_strategy=linux-sandbox` (fails instead of silently falling back to `processwrapper-sandbox`
  [V: `bazel docs/docs/sandboxing.mdx:91-102`]),
  `--experimental_sandbox_limits=memory=<n>` and `cpu=<n>` (without a limit no per-action cgroup exists, see section
  "Cgroups"),
  `--keep_going`,
  `--build_event_json_file=<run dir>/<invocation>.jsonl`,
  `--build_event_publish_all_actions`,
  `--noallow_analysis_cache_discard`.

Because the cgroup path contains the delegated unit's name,
`watchd` writes these flags into a generated rc file passed with `--bazelrc`,
and agents or humans reach Bazel through `watchd` rather than a bare `bazel` [U, inferred].

## 1. Build events

### What the protocols carry

- The Build Event Service is a generic gRPC publish API:
  `PublishLifecycleEvent` and a bidirectional `PublishBuildToolEventStream`
  [V: `bzd/publish_build_event.proto:48-84`],
  with Bazel's own events wrapped as `google.protobuf.Any bazel_event`
  [V: `bzd/build_events.proto:135-136`].
  BES "treats BEP events as opaque bytes"
  [V: `bazel docs/remote/bep.mdx:91-98`],
  so all progress semantics come from Bazel's `build_event_stream.proto`.
- BEP event kinds useful for WR2
  (id `oneof` [V:
  `bazel src/main/java/com/google/devtools/build/lib/buildeventstream/proto/build_event_stream.proto:269-298`],
  payload `oneof` [V: same file `:1649-1682`]):
  - `BuildStarted` (invocation, command, start time) and `PatternExpanded` (targets requested);
  - `TargetConfigured` and `TargetComplete` (`success`, output groups, `failure_detail`)
    [V: same file `:649-693`];
  - `ActionExecuted`: mnemonic, exit code, stdout/stderr file refs, command line, start and end time, `failure_detail`
    [V: same file `:574-624`];
    by default only "important" actions, including all failed ones, are reported
    [V: same file `:174-176`];
    `--build_event_publish_all_actions` publishes all
    [V: `bazel src/main/java/com/google/devtools/build/lib/buildeventstream/transports/BuildEventStreamOptions.java:161-
    166`];
    whether action cache hits produce `ActionExecuted` events [U];
  - `TestResult`, `TestSummary`, and `TestProgress` (a URI for an active test run)
    [V: same file `:708-804`];
    whether `TestProgress` is emitted for local runs [U];
  - `Progress`: chunks of Bazel's stdout and stderr
    [V: same file `:302-319`];
  - `Aborted` with `USER_INTERRUPTED`, `INCOMPLETE`, `OUT_OF_MEMORY`, and more
    [V: same file `:323-368`];
  - `BuildFinished` with exit code name and number
    [V: same file `:886-932`];
    "predefined exit codes are subject to change ... and are not part of the public API"
    [V: same file `:889-891`].
- Gap:
  there is no action-started or action-running event id in the `oneof`
  [V: same file `:269-298`].
  "What is running right now" at action level exists only as human-readable progress text inside `Progress.stderr`
  [U, inferred].

### Transport choice for `watchd`

- File transport (chosen):
  `--build_event_json_file` and `--build_event_binary_file` are documented
  [V: `bazel docs/remote/bep.mdx:64-87`].
  The writer thread flushes at least every 250 ms
  (`EXPERIMENTAL_BEP_FILE_FLUSH_MILLIS`, default 250)
  [V: `bazel src/main/java/com/google/devtools/build/lib/buildeventstream/transports/FileTransport.java:99-101,136-150`]
  .
  `watchd` tails the JSON lines file per invocation.
  A slow reader cannot fail the build [U, inferred: no ack path in `FileTransport`].
- BES backend (alternative):
  `--bes_backend` accepts only `grpc://` or `grpcs://`
  [V: `bazel src/main/java/com/google/devtools/build/lib/buildeventservice/BuildEventServiceOptions.java:34-46`];
  `--bes_proxy=unix:/path` reaches a Unix socket
  [V: same file `:209-219`].
  Failure behavior when `watchd` is the backend:
  - upload failure ends the invocation with exit 38 (transient) or 45 (persistent)
    [V: `bazel src/main/java/com/google/devtools/build/lib/buildeventservice/BuildEventServiceUploader.java:339-354`;
    `bazel docs/run/scripts.mdx:62,65`];
  - retries: `--build_event_upload_max_retries` default 4
    [V: `bazel src/main/java/com/google/devtools/build/lib/buildeventstream/BuildEventProtocolOptions.java:64-70`],
    backoff `initial_delay * 1.6^attempt`, reset when acks arrive
    [V: `BuildEventServiceUploader.java:556-582,734-740`];
  - `--bes_timeout` default `0s` means wait forever for upload after the build
    [V: `BuildEventServiceOptions.java:50-62`];
  - `--bes_upload_mode`: `wait_for_upload_complete` (default) blocks at the end of the invocation,
    `nowait_for_upload_complete` blocks at the start of the next one,
    `fully_async` may lose events
    [V: `BuildEventServiceOptions.java:182-205`];
  - the uploader's command queue is an unbounded `LinkedBlockingDeque`,
    so a slow backend grows memory rather than stalling actions
    [V: `BuildEventServiceUploader.java:146`; stall claim U, inferred].
  The BES route couples build success to daemon health, so it is kept for a future remote viewer only.
- Overhead flags:
  `--build_event_publish_all_actions` (more events),
  `--bes_outerr_buffer_size` default 10240 and `--bes_outerr_chunk_size` default 1 MiB
  [V: `BuildEventServiceOptions.java:143-167`],
  `--build_event_max_named_set_of_file_entries` default 5000
  [V: `BuildEventStreamOptions.java:171-175`].
  Measured overhead: none [U].

## 2. Affected targets and queries

- Design decision:
  `watchd` does not compute affected targets before building.
  It asks Bazel to build or test the watched pattern and lets Skyframe skip unchanged work,
  which is how `ibazel` works (it reruns the same command on any watched change)
  [V: `bazel-watcher internal/ibazel/ibazel.go:300-372`].
  No-op build latency on this repository: unmeasured [U].
- `rdeps` for WR2 answers ("what would this file affect?"):
  `watchd` keeps an in-memory reverse-dependency index built from
  `bazel query --output=streamed_proto 'deps(//...)'` (plus `buildfiles(//...)`),
  refreshed only when `BUILD`, `.bzl`, or `MODULE.bazel` files change,
  serialized between builds exactly like `ibazel`'s `QUERY` state
  [V pattern: `bazel-watcher internal/ibazel/ibazel.go:71-73,327-345`].
  `query` (not `cquery`) avoids analysis-cache interference;
  `ibazel` notes "cquery can be affected by the majority of command line option"
  [V: `bazel-watcher internal/ibazel/ibazel.go:722-727`].
  Query cost and output size on this repository: unmeasured [U].
- Queries during a build:
  a query on the same output base waits for the running command
  [V: `BlazeCommandDispatcher.java:229-265`],
  or fails with exit 9 under `--noblock_for_lock`
  [V: `bazel docs/run/scripts.mdx:54`].
  A separate `--output_base` runs a second server
  [V: `bazel docs/run/scripts.mdx:26-33`; `docs/advanced/performance/iteration-speed.mdx:88-90`].
- Cost of a second output base:
  - a second JVM with its own analysis cache [V: `iteration-speed.mdx:17-25`];
    heap size is not quantified by the memory docs, which only show `--host_jvm_args=-Xmx2g` as a cap example
    [V: `bazel docs/advanced/performance/memory.mdx:11-14`];
  - fetched repositories are shared through the repo contents cache, default `{--repository_cache}/contents`,
    "shareable across workspaces"
    [V: `bazel docs/versions/9.1.0/reference/command-line-reference.mdx:357-362`];
  - machine headroom: 62 GiB total, 17 GiB available, 20 GiB swap in use at measurement
    [M: `free --gibi`].
- target-determinator and bazel-diff:
  both compare two git revisions
  (target-determinator "between two git commits" [V: `bzd/target-determinator-README.md:3`];
  bazel-diff checks out each revision and hashes the graph [V: `bzd/bazel-diff-README.md:61-67`]).
  They answer CI questions about commits, not uncommitted edits in a live loop, so the daemon does not use them.

## 3. Cancellation

- Client signal path:
  SIGINT and SIGTERM both call `CancelServer()`;
  the third SIGINT kills the server
  [V: `bazel src/main/cpp/blaze_util_posix.cc:147-176`].
  The signal handler writes to a pipe; a cancel thread sends the `Cancel` RPC
  once the command id is known
  [V: `bazel src/main/cpp/blaze.cc:1838-1930`],
  with a 10 s deadline and a printed "Could not interrupt server" on failure
  [V: `blaze.cc:1932-1946`].
- Server path:
  `Cancel` interrupts the command thread
  [V: `bazel src/main/java/com/google/devtools/build/lib/server/CommandManager.java:99-111`];
  after 10 s without completion a slow-interrupt warning is logged
  [V: `CommandManager.java:200-221`].
- Exit code 8 means "Build Interrupted but we terminated with an orderly shutdown"
  [V: `bazel docs/run/scripts.mdx:53`; `src/main/java/com/google/devtools/build/lib/util/ExitCode.java:52`].
- In-flight actions:
  local spawns call `subprocess.destroyAndWait()` on interrupt
  [V: `bazel src/main/java/com/google/devtools/build/lib/exec/local/LocalSpawnRunner.java:413-421`];
  linux-sandbox sends SIGTERM, then SIGKILL after `kill_delay_secs`
  [V: `bazel src/main/tools/linux-sandbox.cc:195-213`];
  `--local_termination_grace_seconds` defaults to 15
  [V: `bazel src/main/java/com/google/devtools/build/lib/exec/local/LocalExecutionOptions.java:30-38`];
  that this flag feeds the sandbox `kill_delay_secs` on interrupt [U, inferred].
  Persistent workers cannot currently be interrupted
  [V: `bazel src/main/java/com/google/devtools/build/lib/server/GrpcCommandServerImpl.java:107-111` comment].
- Action cache:
  entries are written in post-execution processing after an action finishes
  [V: `bazel src/main/java/com/google/devtools/build/lib/skyframe/ActionExecutionFunction.java:853-873`],
  so completed actions survive a cancel and interrupted ones rerun next time [U, inferred].
- Client disappearance:
  if the client's stream is cancelled, the server restores the interrupt flag on its next output write
  [V: `GrpcCommandServerImpl.java:101-129`].
  The server also blocks writing output until the client reads
  [V: same lines], so `watchd` must drain client stdout and stderr continuously [U, inferred].
- Time to stop on this repository: unmeasured [U].
- `watchd` cancel ladder:
  1. send one SIGINT to the client (never three);
  2. after a configurable deadline, `cgroup.kill` on `<D>/bazel`,
     which is the same outcome as Bazel's own third-Ctrl-C server kill
     [V: `blaze_util_posix.cc:151-163`],
     documented as costly because the analysis cache is lost
     [V: `iteration-speed.mdx:77-86`].

## 4. Watching

- `--watchfs` only changes how Bazel detects changes at the start of the next command
  [V: `bazel src/main/java/com/google/devtools/build/lib/skyframe/LocalDiffAwareness.java:38-42,52-63`];
  it never starts a build.
  - Linux: Java `WatchService` over inotify, registering every directory
    [V: `bazel src/main/java/com/google/devtools/build/lib/skyframe/WatchServiceDiffAwareness.java:43,264-330`].
    On inotify overflow Bazel throws `BrokenDiffAwarenessException`
    [V: `WatchServiceDiffAwareness.java:176-190`] and falls back to scanning [U].
  - macOS: FSEvents [V: `LocalDiffAwareness.java:115-118`].
  - Windows: no-op unless `--experimental_windows_watchfs`
    [V: `LocalDiffAwareness.java:52-75`; `WatchServiceDiffAwareness.java:99`].
- Limits here:
  `fs.inotify.max_user_watches` 524288, `max_user_instances` 8192, `max_queued_events` 16384
  [M: `/proc/sys/fs/inotify/*`];
  the repository has 99,415 directories, 3,450 after excluding `.git`, `node_modules`, `target`, `.gradle`, `build`,
  `dist`
  [M: `fd --type directory`].
  `watchd`'s watcher and Bazel's `--watchfs` each consume watches [U, inferred].
- `ibazel` as design reference:
  - watch sets come from `kind('source file', deps(set(%s)))` and `buildfiles(set(%s))` queries
    [V: `bazel-watcher internal/ibazel/ibazel.go:71-73,553-640`];
  - only write, create, rename, remove events count
    [V: `ibazel.go:296-298`];
  - 100 ms debounce [V: `ibazel.go:107`];
  - the loop is synchronous: `RUN` blocks in `commandToRun`, events queue until it returns
    [V: `ibazel.go:356-372`], so `ibazel` cannot cancel a build on a new change [U, inferred].
- `watchd` loop:
  - watch the workspace minus `.bazelignore` paths with the repository's existing watcher code
    (file-enforcer FE21 and FE23 supervision);
  - classify each change: graph file (refresh index), source in index (build),
    protected destination (revert), `watchd`-written file (suppress echo by recorded content hash);
  - debounce, then either queue the next invocation or supersede the running one via the cancel ladder,
    per work kind (configurable).

## 5. Generated files in the source tree

### bazel-lib `write_source_files` facts

- Updating requires `bazel run //:target`; the generated `diff_test` fails when out of date,
  and a missing destination produces a failing test with instructions
  [V: `bazel-lib lib/write_source_files.bzl:18-28`; `lib/private/write_source_file.bzl:125-190`].
- Destination must be in the user workspace
  [V: `lib/private/write_source_file.bzl:97-100`],
  and in the target's own package unless `check_that_out_file_exists = False`
  [V: `lib/private/write_source_file.bzl:102-104`].
- `out_file` is a string attribute on purpose, so `ibazel`'s source-file query does not watch it
  and loop forever
  [V: `lib/private/write_source_file.bzl:196-203`].
  Editing a destination therefore triggers nothing under `ibazel`,
  except through the `diff_test`, which takes the destination as `file2`
  [V: `lib/private/write_source_file.bzl:184-190`] if tests are in the watched set [U, inferred].
- The Linux and macOS updater is a bash script (`#!/usr/bin/env bash`) that always runs
  `rm -Rf "$out"` then `cp -f "$in" "$out"`, with no content comparison
  [V: `lib/private/write_source_file.bzl:223-310`].
  It is not atomic and changes every destination's inode and mtime on every run [U, inferred].
- Windows uses a generated `.bat` with `RUNFILES_MANIFEST_ONLY=1` and `copy`/`robocopy`,
  and notes "there's no sandboxing in windows"
  [V: `lib/private/write_source_file.bzl:313-396`].
- The target runs in the client after the server command finishes
  [V: `bazel src/main/cpp/blaze.cc:2190-2219`],
  so it runs outside the sandbox and outside Bazel's cache [U, inferred].

### `watchd` design for generation

- Generators are ordinary Bazel actions (for example `js_run_binary`) producing files in `bazel-bin`,
  sandboxed and cgroup-limited like any action.
- A manifest target emits the destination map `{ out_path, label }`;
  `watchd` builds it, reads output paths from `TargetComplete` output groups,
  and writes destinations itself with FE05 semantics (skip identical, atomic rename) and FE06 create-only,
  instead of calling the `write_source_files` script.
  `diff_test` targets stay for CI.
- Protected destinations (FE22):
  an external edit to a destination triggers a rebuild of its label (an action cache hit when inputs are unchanged)
  and a rewrite from `bazel-bin`, plus the existing desktop notification.
- Echo suppression (FE23): record written content hashes; ignore matching watch events.
- In-place transforms (FE10, FE11, FE14 `manageCargoManifests`) read the destination as an input;
  the generator must be idempotent, and `watchd` stops with an error after repeated non-converging rewrites.
- Cross-package discovery (FE03, FE07 globs like `package/*/*/Cargo.toml`):
  a `BUILD` `glob()` "does not match files in subpackages"
  [V: `bazel docs/versions/9.1.0/reference/be/functions.mdx:269-281`].
  Options: a generated label list (a source-tree write that must precede analysis),
  or a repository rule using `repository_ctx.watch_tree` over `package/`
  [V: `bazel docs/versions/9.1.0/rules/lib/builtins/repository_ctx.mdx:340-346`].
  A module extension cannot watch paths outside the workspace
  [V: same file `:332`].
- Bootstrap hazard:
  files file-enforcer manages are also inputs to repository rules,
  for example `Cargo.toml` manifests read by `crate_universe`
  [V: `rules_rust crate_universe/extensions.bzl:63-64`]
  and `package.json` files read during pnpm lock handling
  [V: `rules_js docs/pnpm.md:140-146`].
  A broken generated manifest can break loading of every package that depends on that repository,
  including a generator that needs it [U, inferred].
  Generator targets must not depend on repositories derived from their own outputs [U, inferred].

## 6. Non-hermetic operations

- Execution tags
  [V: `bazel docs/versions/9.1.0/reference/be/common-definitions.mdx:83`]:
  `no-sandbox` (never sandboxed, still cacheable),
  `no-cache` (never cached locally or remotely, but "Skyframe or the persistent action cache are not affected"),
  `local` (no remote, no sandbox),
  `requires-network` and `block-network`.
  So even a `no-cache` action with unchanged inputs does not rerun within Bazel's own caches,
  and Bazel never observes state outside declared outputs [U, inferred].
- `--sandbox_writable_path` makes an existing directory writable inside the sandbox
  [V: `bazel src/main/java/com/google/devtools/build/lib/sandbox/SandboxOptions.java:167-175`];
  linux-sandbox otherwise makes the whole filesystem read-only except the sandbox directory
  and kills all processes the action spawned
  [V: `bazel docs/docs/sandboxing.mdx:77-86`].
  Writing `~/.config/JetBrains/...` from a build action would need that flag and would still be invisible to caching
  [U, inferred].
- Mapping of file-enforcer features:
  - FE18 and FE19 via Meta Package Manager:
    a `bazel run //tools:provision` target, triggered by `watchd` on its own staleness check
    (hash of the package list plus an MPM probe of installed state),
    executed in `<D>/client` with cgroup limits and no filesystem sandbox.
    Privileged installs go through the package manager's own elevation, outside the user cgroup [U].
  - FE15 JetBrains LSP4IJ settings in the home directory:
    same `bazel run` pattern; `write_source_files` cannot target it
    [V: `lib/private/write_source_file.bzl:97-100`].
  - Root config files (`mise.toml`, `CLAUDE.md`, and others):
    generator actions plus the `watchd` writer; needs a root `BUILD.bazel` or `check_that_out_file_exists = False`.
  - FE16 and FE17 command execution with platform dispatch:
    build actions if hermetic, else `bazel run` targets [U, inferred].
  - FE09 staleness manifest:
    replaced by the action cache for hermetic generation; still needed inside `watchd` for `bazel run` work.
- What breaks:
  caching correctness for anything outside declared inputs and outputs,
  and sandbox-plus-cgroup coverage for `bazel run` work (only the `watchd` cgroup applies).

## 7. Platforms and CI

0.x is Linux-only; the macOS and Windows findings below matter for HC5 later.

- Linux:
  `linux-sandbox` uses user, mount, PID, network, and IPC namespaces
  [V: `bazel docs/docs/sandboxing.mdx:77-86`];
  nested use (for example inside Docker without `--privileged`) falls back to `processwrapper-sandbox`
  [V: `sandboxing.mdx:91-97`].
- Ubuntu 24.x hosts:
  AppArmor's `unprivileged_userns` profile denies linux-sandbox mounts;
  issue bazelbuild/bazel#24081 is open,
  and the workaround PR #26434 ("Use busybox workaround to run linux-sandbox on Ubuntu 24.04+") is open
  [V: `gh issue view 24081`, `gh pr view 26434`].
  Whether GitHub-hosted `ubuntu-24.04` images enable that restriction [U].
  Whether GitHub runners have a systemd user manager for `systemd-run --user` [U].
- macOS: `darwin-sandbox` via `sandbox-exec` [V: `sandboxing.mdx:88-89`]; no cgroups
  [V: `VirtualCgroup.java:82-85` returns `NULL` off Linux].
- Windows:
  - no sandbox in the docs' strategy list [V: `sandboxing.mdx:50-97`];
    `--experimental_use_windows_sandbox` needs an external `BazelSandbox.exe`
    [V: `SandboxOptions.java:197-217`];
  - symlinks need Developer Mode or admin, `--enable_runfiles` is opt-in,
    and long paths need a short `--output_user_root`
    [V: `bazel docs/configure/windows.mdx:19-49`];
  - `genrule`, `sh_*`, and `run_shell` need Bash [V: `windows.mdx:69-116`];
  - rules_js CI: "The root workspace only builds cleanly on macOS for now; Windows has broad pre-existing failures"
    [V: `rules_js .github/workflows/ci-workflows.yaml:288-292`];
  - rules_lint CI: "TODO(alex): lots of places are missing windows..." with `windows-latest` commented out
    [V: `rules_lint .github/workflows/ci.yaml:224-225`];
  - bazel-lib skips integration tests on Windows [V: `bazel-lib .github/workflows/ci.yaml:49-50`];
  - Node's `subprocess.kill('SIGINT')` terminates forcefully on Windows
    [V: `bzd/node-child_process.md:1748-1754`],
    while the Bazel client cancels only on `CTRL_C_EVENT` or `CTRL_BREAK_EVENT`
    [V: `bazel src/main/cpp/blaze_util_windows.cc:299-322`],
    so graceful cancel needs console control events [U, inferred].
- Bazelisk: reads `.bazelversion` and can track the latest LTS or rolling release
  [V: `bazel docs/install/bazelisk.mdx:31-38`].
- `bazel-contrib/setup-bazel` 0.19.0:
  `bazelisk-cache` keyed by `.bazelversion`,
  `disk-cache` keyed by `BUILD` file contents,
  `repository-cache` keyed by `MODULE.bazel`,
  `external-cache` for repositories over 10 MB,
  default output base `D:/_bazel` on Windows
  [V: `bzd/setup-bazel-README.md:21-25,110-148,208-237`].
  GitHub Actions cache size limits and eviction [U].

## 8. Tool provisioning

Root `mise.toml` `[tools]` [V: `mise.toml` `[tools]` section read 2026-09-16].
Bazel toolchains pin exact versions; the repository's `PIN` rule says "Pin tool versions only with clear justification"
[V: `CLAUDE.md` rule `PIN`], and most tools are `latest`.

- Bazel itself: Bazelisk, may float [V: `docs/install/bazelisk.mdx:31-38`].
- `node` (`latest`): rules_js node toolchain from `.nvmrc` or explicit versions
  [V: `rules_js npm/extensions.bzl:14`; `MODULE.bazel:163-171`];
  floating `latest` support [U].
- `pnpm` (`latest`): rules_js `pnpm_version_from = "//:package.json"`
  [V: `rules_js npm/extensions.bzl:19`];
  developers still run pnpm outside Bazel in the typical flow, or `bazel run -- @pnpm//:pnpm`
  [V: `rules_js docs/pnpm.md:126-154`].
- `rust` nightly with `clippy`, `rust-src`, `llvm-tools-preview`:
  rules_rust accepts one version per channel as `nightly/<date>`, no floating nightly
  [V: `rules_rust rust/extensions.bzl:236-241`];
  llvm-tools repository support exists [V: `rules_rust rust/private/repository_utils.bzl:282`];
  component parity [U].
- `cargo-nextest`, `cargo-fuzz`: no rules_rust integration (prior research, `doc/research/bazel-migration-dx.md`);
- stays Mise or MPM, or prebuilt binaries.
- `java` (`temurin-21`): rules_java remote JDKs [U: distribution and version choices not checked];
  Gradle builds kept outside Bazel still need a JDK from Mise or MPM.
- `android-sdk`: rules_android resolves the SDK from `ANDROID_HOME`
  [V: `rules_android rules/android_sdk_repository/rule.bzl:116`], so it is not provisioned;
  the BCR module `hermetic_android_toolchains` 0.4.0 exists [V: BCR
  `modules/hermetic_android_toolchains/metadata.json`], not evaluated.
- `zig` 0.15.2: `rules_zig` release v0.16.0 exists and takes a `zig_version`
  [V: `gh api repos/aherrmann/rules_zig` README lines 132-145, latest release v0.16.0];
  0.15.2 availability [U]; the repository uses Zig inside a Cargo build script (`libghostty-vt-sys`),
  which needs the binary plumbed into `cargo_build_script` [U].
- `cmake`: `rules_foreign_cc` is in the BCR [V: BCR list]; plumbing into `opusic-sys` build scripts [U].
- Prebuilt CLIs (`dprint`, `ripgrep`, `fd`, `fastmod`, `hyperfine`, `caddy`, `opentofu`, `hcloud`, `llama.cpp`,
- `harper-cli`, `betterleaks`, `rcodesign`, `slint-lsp`, `slint-viewer`):
  `rules_multitool` lockfile entries need URL plus `sha256` per OS and CPU
  [V: `bzd` fetch of `bazel-contrib/rules_multitool/readme.md`, "Usage" section];
  `bazel_env.bzl` puts Bazel-managed tools on `PATH` through direnv
  [V: `buildbuddy-io/bazel_env.bzl` README lines 3-6] using bash trampolines
  [V: `bazel_env.bzl` source lines 444-447].
- npm CLIs (`typescript-language-server`, `wrangler`, `pagefind`, `socket`): pnpm dev dependencies through rules_js [U,
- inferred].
- `sops`, `age`: secrets tooling outside the build graph; Mise or MPM [U, inferred].
- `uv`, `pipx:slopo`: `rules_python` and `rules_uv` exist in the BCR [V: BCR list]; not evaluated.
- `bun`: no BCR module matches `bun` [V: `bzd/bcr-modules-full.txt` search].
- `watchexec`: dropped; `watchd` replaces it.
- oxlint: still no rules_lint linter (prior research); BCR `aspect_rules_oxc` 0.1.0-rc1 is a transpiler, not a linter
  [V: `aspect-build/rules_oxc` README lines 1-12].
- Persistent workers: rules_ts has none
  [V: 0 matches for `worker` across rules_ts `.bzl` and `.md` files];
  rules_kotlin sets `supports-workers: 1`
  [V: `rules_kotlin kotlin/internal/toolchains.bzl:92`].

## 9. Fedora Atomic specifics

- Host: Bazzite 44 (Kinoite), SELinux enforcing, user context `unconfined_t`, home on btrfs, `/home` resolves to
- `/var/home`
  [M: `/etc/os-release`, `/sys/fs/selinux/enforce`, `id --context`, `readlink --canonicalize /home`,
  `stat --file-system`].
- User namespaces:
  the docs' only prerequisite is `kernel.unprivileged_userns_clone` not being 0 where that file exists
  [V: `bazel docs/docs/sandboxing.mdx:128-138`];
  the file does not exist here, `user.max_user_namespaces` is 254589,
  and `unshare --user --map-root-user --mount --pid --net --ipc --uts --fork -- /usr/bin/true` succeeded from the agent
  shell
  [M].
  The agent shell is in the initial user namespace (`user:[4026531837]`) [M],
  so linux-sandbox is not nested there [U, inferred].
  A real linux-sandbox run was not attempted [U].
- cgroups:
  cgroup v2 only, mounted with `nsdelegate,memory_recursiveprot`
  [M: `/proc/self/mounts`];
  `user@1000.service` delegates `cpu io memory pids dmem`, owned by the user
  [M: `cgroup.controllers`, `cgroup.subtree_control`, `stat`];
  `/usr/bin/systemd-run` exists (systemd 259) [M].
  Delegated nested layout, limits, and freeze work for an unprivileged process [M: probe in section "Architecture
  overview"].
- Bazel binaries arrive through Bazelisk into the user cache, so no rpm-ostree layering is needed [U].

## 10. Cgroups

### Bazel's cgroup surface

- `--experimental_cgroup_parent=<path>` (startup, client-only):
  starts the server in that cgroup for each supported controller;
  "It is not an error if the specified cgroup is not writable"
  [V: `bazel src/main/java/com/google/devtools/build/lib/runtime/BlazeServerStartupOptions.java:506-531`].
  `daemonize` writes the PID to `cgroup.procs` and on failure prints a debug message,
  "Falling back to running without cgroups"
  [V: `bazel src/main/tools/daemonize.cc:308-340`].
- `--experimental_run_in_user_cgroup` (startup, Linux):
  runs the server via `systemd-run --user --scope` after a `/bin/true` probe, without `Delegate=yes`
  [V: `BlazeServerStartupOptions.java:538-547`; `daemonize.cc:277-297`].
  Combined with the server climbing one level for its spawns slice
  [V: `VirtualCgroup.java:150-157`],
  Bazel would create cgroups inside the systemd-managed `app.slice` and write its `cgroup.subtree_control`
  [V: `bazel src/main/java/com/google/devtools/build/lib/sandbox/cgroups/controller/v2/UnifiedController.java:36-48`],
  which systemd's delegation rules forbid
  ("Never create your own cgroups below arbitrary cgroups systemd manages", "Never *write* to any of the attributes of
  a cgroup systemd created")
  [V: `bzd/CGROUP_DELEGATION.md:386-400`].
  `watchd` therefore uses `--experimental_cgroup_parent` inside its own delegated unit and never this flag.
- `--experimental_sandbox_memory_limit_mb` and `--experimental_sandbox_limits=<name>=<value>`:
  "each Linux sandbox will be limited";
  "Requires cgroups v1 or v2 and permissions for the users to the cgroups dir"
  [V: `SandboxOptions.java:350-375`; `@9.2.0` `SandboxOptions.java:346-369`].
  Only `cpu` and `memory` are applied
  [V: `VirtualCgroupFactory.java:62-94`; `VirtualCgroup.java:171-189`].
  No `pids` or `io` limit per action [U, inferred from those lines].
- `--experimental_sandbox_enforce_resources_regexp`:
  actions whose mnemonic matches get their declared resources as limits
  [V: `@9.2.0` `SandboxOptions.java:389-400`; `LinuxSandboxedSpawnRunner.java:323-327`].
- `--incompatible_use_new_cgroup_implementation`:
  in 9.2.0 a real flag, default true, and `--experimental_sandbox_limits` requires it
  [V: `@9.2.0` `SandboxOptions.java:358-387`];
  on `main` a deprecated no-op
  [V: `bazel src/main/java/com/google/devtools/build/lib/bazel/rules/BazelRulesModule.java:88-94`].
- `--experimental_worker_use_cgroups_on_linux`:
  runs each worker in its own cgroup without limits, for memory accounting;
  documentation category `UNDOCUMENTED`
  [V: `bazel src/main/java/com/google/devtools/build/lib/worker/WorkerOptions.java:285-295`];
  absent from the 9.1.0 command-line reference
  [M: `bzd/flag-doc-check.ts`].
  Worker limits apply only with `--experimental_worker_sandbox_hardening`
  [V: `WorkerModule.java:122-130`],
  and cgroup use silently falls back to `ps` when unavailable
  [V: `WorkerModule.java:186-191`].
- Documentation coverage:
  in the 9.1.0 docs snapshot only `reference/command-line-reference.mdx` contains the word `cgroup`;
  the unversioned docs contain none
  [M: `rg --files-with-matches 'cgroup'` over `docs/`].
  The other cgroup flags do appear as reference entries
  (`experimental_cgroup_parent` line 153, `experimental_run_in_user_cgroup` line 159,
  `experimental_sandbox_enforce_resources_regexp` line 3769, `experimental_sandbox_limits` line 3775,
  `experimental_sandbox_memory_limit_mb` line 3781, `experimental_worker_sandbox_hardening` line 3847,
  `incompatible_use_new_cgroup_implementation` line 3871)
  [M: `bzd/flag-doc-check.ts` against `docs/versions/9.1.0/reference/command-line-reference.mdx`].

### Per-action cgroups

- Exist for `linux-sandbox` spawns only:
  `LinuxSandboxedSpawnRunner` builds a `VirtualCgroupFactory("sandbox_", ..., alwaysCreate = false)`
  [V: `LinuxSandboxedSpawnRunner.java:159-166`],
  creates a child per spawn and passes `-C <dir>` to linux-sandbox
  [V: `LinuxSandboxedSpawnRunner.java:323-330`; `LinuxSandboxCommandLineBuilder.java:302-304`],
  which writes the PID into `cgroup.procs`
  [V: `bazel src/main/tools/linux-sandbox.cc:184-187`],
  and removes the cgroup after the action because of the kernel's 65535 memory cgroup limit
  [V: `LinuxSandboxedSpawnRunner.java:470-476`].
- Created only when some limit is non-zero
  [V: `VirtualCgroupFactory.java:57-71`].
- Not created for `processwrapper-sandbox`, `local`/`standalone`, `no-sandbox` or `local` tagged actions,
  or `bazel run` targets
  [V: no `cgroup` references in `ProcessWrapperSandboxedSpawnRunner.java` or `LocalSpawnRunner.java`; `bazel run` exec
  in client at `blaze.cc:2190-2219`].
- Silent degradation:
  if the server's parent cgroup is not writable, the virtual root becomes `NULL` with an info-level log
  [V: `VirtualCgroup.java:81-100,158-168`],
  and failed limit writes only log warnings
  [V: `VirtualCgroupFactory.java:77-94`].
  Issue bazelbuild/bazel#26062 ("bazel should fail if cgroups cannot be created and experimental_sandbox_limits or
  experimental_sandbox_memory_limit_mb is set")
  is open and marked stale on 2026-08-28;
  a Bazel maintainer wrote "we don't want to fail the build if cgroups aren't created properly because it turns out
  that user environments affect this too easily"
  and later "it doesn't seem like we use this internally either"
  [V: `gh issue view 26062 --comments`, comments 2025-05-20 and 2025-06-11].
- `watchd` enforcement plan:
  - own the delegated tree and set aggregate `memory.max` and `pids.max` on `<D>/bazel`,
    which does not depend on Bazel [M: probe];
  - after each server start, check `/proc/<server pid>/cgroup` equals `<D>/bazel/server`
    (server PID from `<output_base>/server/server.pid.txt` [U: file name]);
  - run a canary test target on each server start whose action asserts `/proc/self/cgroup` ends in `sandbox_<n>.scope`
    and that `memory.max` matches, and refuse to report "sandboxed" otherwise
    (the #26062 reporter used the same recursive-test approach [V: issue comment 2025-05-27]).

### Freezing a running build

- Kernel semantics:
  writing `1` to `cgroup.freeze` freezes the cgroup and all descendants; completion is signalled by `frozen 1` in
  `cgroup.events`;
  frozen processes can still be killed by fatal signals
  [V: `bzd/cgroup-v2.rst:1025-1046`].
- Freezing only the server leaf is not enough:
  limited actions live in the sibling `blaze_<pid>_spawns.slice`
  [V: `VirtualCgroup.java:93,150-157`; `linux-sandbox.cc:184-187`],
  so they keep running.
  `watchd` freezes `<D>/bazel`, the common parent [M: hierarchical freeze probe].
- State corruption:
  no mechanism found.
  A freeze pauses writes rather than aborting them [U, inferred],
  and Bazel's own supported recovery path kills the server outright on the third SIGINT
  [V: `blaze_util_posix.cc:151-163`].
  Bazel documents nothing about freezing [M: no `cgroup` mention outside the flag reference].
- Hazards:
  - linux-sandbox timeouts use wall-clock `alarm(opt.timeout_secs)`
    [V: `linux-sandbox.cc:408`], and process-tools uses `setitimer(ITIMER_REAL)`
    [V: `bazel src/main/tools/process-tools.cc:177-188`],
    so a test whose timeout passes during a freeze is killed on thaw as a timeout [U, inferred].
    Failed tests rerun next time under `--cache_test_results=auto`
    [V: `bazel docs/versions/9.1.0/reference/command-line-reference.mdx:3424-3432`].
  - The client's `Cancel` RPC has a 10 s deadline [V: `blaze.cc:1939`];
    cancelling a frozen build fails, so `watchd` thaws before cancelling.
  - Every other command on that output base waits while the frozen command holds the server lock
    [V: `BlazeCommandDispatcher.java:229-265`].
  - Kernel inotify queues can overflow during a long freeze (limit 16384 [M]),
    which makes `--watchfs` throw `BrokenDiffAwarenessException`
    [V: `WatchServiceDiffAwareness.java:176-190`].
  - BEP action timings include the frozen time [U, inferred].

## RPC surface for WR2 and WR3

- Transport (0.x):
  Unix domain socket at `$XDG_RUNTIME_DIR/monochromatic/watchd.sock`, mode 0600,
  JSON-RPC 2.0 with newline-delimited framing, the framing `@monochromatic-dev/mcp-stdio` already implements for stdio.
- Inspection (WR2):
  - `daemon.status`: state (`idle`, `debouncing`, `querying`, `building`, `cancelling`, `writing`, `frozen`),
    queue, current invocation id, Bazel server PID, output base, cgroup verification result.
  - `invocations.list { limit, before }`: recent invocations from a ring buffer persisted as JSON lines.
  - `invocation.get { id }`: command, targets, per-target results, failed actions with `failure_detail`,
    completed actions with timings, test results, exit code name, cancel reason.
  - `events.subscribe { kinds, sinceSeq }`: sequenced notifications
    (`invocation.started`, `target.completed`, `action.completed`, `test.result`, `progress.text`,
    `invocation.finished`, `file.written`, `file.reverted`, `watch.changed`, `cgroup.degraded`).
  - `graph.affected { paths }`: answers from the reverse-dependency index.
- Control (WR3):
  - `work.start { kind: build | test | generate | provision, targets }`: returns a queued invocation id.
  - `work.rerun { invocationId }`: same targets; tests with `--cache_test_results=no` when forced.
  - `work.cancel { invocationId }`: cancel ladder in section "Cancellation".
  - `work.pause` and `work.resume`: freeze and thaw `<D>/bazel`, with the hazards in section "Freezing a running build".
  - `daemon.shutdown`: cancel, `bazel shutdown`, remove the socket.
- Current-activity limit:
  "running now" is reported as the invocation, its completed events, and the latest progress text,
  because BEP has no action-started events [V: `build_event_stream.proto:269-298`].

## Evidence limits

- No Bazel command was run, so query cost, no-op build latency, server memory, cancel latency,
  and the canary cgroup check are all unmeasured.
- The cgroup probe exercised the kernel and systemd layout, not Bazel's use of it.
- GitHub-hosted runner details (AppArmor, systemd user manager, cgroup delegation) were not checked.
- Rule-set Windows statements come from CI configuration comments, not from runs.

## Yikes candidates

### Y1. Documentation rule (HC7) is already failed, and the design leans harder on undocumented behavior

- Evidence:
  Bazel exited the vet on a documentation confusion trigger (`doc/planning/monorepo-manager-build-routes.md`,
  "Inherited gaps").
  This design additionally depends on behavior found only in source or one-line flag help:
  cancel propagation and the 10 s cancel deadline [V: `blaze.cc:1838-1946`],
  BES failure exit codes and retry policy [V: `BuildEventServiceUploader.java:339-354,556-582`],
  cgroup placement and silent fallback [V: `VirtualCgroup.java:81-168`; `daemonize.cc:308-340`],
  an `UNDOCUMENTED` worker cgroup flag [V: `WorkerOptions.java:285-295`],
  `write_source_files` internals [V: `lib/private/write_source_file.bzl:223-310`],
  and freeze behavior, which Bazel does not document at all.
  The only docs pages mentioning cgroups are flag reference entries [M].
- Why disqualifying: HC7 culls on any confusion; the parts the daemon needs most are the least documented.

### Y2. Bazel's cgroup sandboxing is experimental, conditional, and fails open

- Evidence:
  per-action cgroups exist only for linux-sandbox actions and only when a limit is set [V:
  `VirtualCgroupFactory.java:57-71`; `LinuxSandboxedSpawnRunner.java:159-166,323-330`];
  only `cpu` and `memory` [V: `VirtualCgroupFactory.java:62-94`];
  unwritable cgroups degrade silently [V: `VirtualCgroup.java:96-99`; `daemonize.cc:333`];
  upstream declines to fail builds and the issue is stale [V: bazelbuild/bazel#26062];
  the flag set is churning (`incompatible_use_new_cgroup_implementation` became a no-op on `main`) [V:
  `BazelRulesModule.java:88-94`];
  the built-in user-cgroup mode breaks systemd delegation rules [V: `daemonize.cc:295`; `VirtualCgroup.java:150-157`;
  `bzd/CGROUP_DELEGATION.md:386-400`].
- Why it could force redesign:
  with cgroups a hard requirement, the guarantee must come from `watchd` (delegated tree, aggregate limits, canary
  verification),
  and per-action limits remain a best-effort Bazel feature that can switch itself off without an error.

### Y3. Large parts of the work escape both the sandbox and per-action cgroups

- Evidence:
  `bazel run` targets execute in the client process after the server finishes [V: `blaze.cc:2190-2219`];
  `processwrapper-sandbox`, `local`, and `no-sandbox` paths create no cgroups [V: no references in those runners];
  persistent workers need extra flags, one undocumented [V: `WorkerOptions.java:285-295`; `WorkerModule.java:122-130`],
  and rules_kotlin uses workers [V: `toolchains.bzl:92`].
  File-enforcer's home-directory settings (FE15), OS packages (FE18, FE19), and source-tree writes (FE05, FE22) all
  land on the `bazel run` or `watchd` side.
- Why it could force redesign:
  if "sandboxing" means filesystem isolation per task and not only cgroup containment, those features cannot satisfy it
  under Bazel; only `watchd`'s cgroup applies.

### Y4. File-enforcer mostly moves into `watchd` instead of into Bazel

- Evidence:
  no cross-package globs [V: `functions.mdx:269-281`];
  no destinations outside the workspace [V: `write_source_file.bzl:97-100`; `repository_ctx.mdx:332`];
  `write_source_files` unconditionally deletes and copies through bash [V: `write_source_file.bzl:223-310`], which
  fights echo suppression and FE05;
  generated manifests feed repository rules [V: `crate_universe/extensions.bzl:63-64`;
  `rules_js docs/pnpm.md:140-146`], creating bootstrap hazards [U, inferred].
- Why it could force redesign:
  FE05, FE09 (for `bazel run` work), FE20 to FE23 are rebuilt in `watchd`,
  so Route A keeps most of Route B's file-enforcer work and adds Starlark generators and discovery rules on top.

### Y5. WR2 "what is it doing now" has no structured action-level source

- Evidence:
  no action-started event id in BEP [V: `build_event_stream.proto:269-298`];
  actions are reported at completion, and by default only important ones [V: same file `:174-176`].
- Why it could disqualify:
  if WR2 requires listing currently running actions, the only source is unstructured progress text.

### Y6. One command per output base, on a machine already short of memory

- Evidence:
  one invocation per server [V: `client-server.mdx:12-14`; `BlazeCommandDispatcher.java:229-265`];
  concurrent queries need a second server and JVM [V: `scripts.mdx:26-33`];
  17 GiB available and 20 GiB swap in use [M: `free --gibi`];
  server heap is not capped by default [V: no `Xmx` in client sources].
- Why it could force redesign:
  WR3 start requests queue behind running builds or cancel them,
  and a query server plus Gradle, rust-analyzer, and editors may not fit; unmeasured.

### Y7. Pausing a build via cgroup freeze has correctness side effects

- Evidence:
  freezing the server's own cgroup misses limited actions [V: `VirtualCgroup.java:93,150-157`];
  wall-clock timers kill tests on thaw [V: `linux-sandbox.cc:408`; `process-tools.cc:177-188`; consequence U, inferred];
  cancel fails while frozen [V: `blaze.cc:1939`].
- Why it matters: not disqualifying if `work.pause` is optional or scoped to non-test work; no corruption mechanism was
- found.

### Y8. Hermetic toolchains require pinning, which conflicts with the repository's `PIN` rule

- Evidence:
  rules_rust takes dated nightlies only [V: `rust/extensions.bzl:236-241`];
  `rules_multitool` needs `sha256` per platform [V: readme "Usage"];
  the Android SDK is not provisioned [V: `rule.bzl:116`];
  `mise.toml` uses `latest` for most tools and `CLAUDE.md` rule `PIN` requires justification for pins [V].
- Why it needs a decision: either the policy changes to "lockfile plus bump automation", or Mise or MPM keeps
- provisioning and Bazel is not hermetic for those tools.

### Y9. Linux CI may not get linux-sandbox or delegated cgroups

- Evidence:
  AppArmor blocks linux-sandbox on Ubuntu 24.x, issue #24081 and fix PR #26434 both open [V];
  runner image settings and systemd user sessions unchecked [U].
- Why it could disqualify: "sandboxing with at least cgroups is a must" would hold locally but not in CI without
- runner-level workarounds.

### Y10. Windows and macOS (after 0.x)

- Evidence:
  rules_js root workspace fails on Windows CI [V: `ci-workflows.yaml:288-292`];
  rules_lint lacks Windows CI [V: `ci.yaml:224-225`];
  no Windows sandbox [V: `sandboxing.mdx:50-97`; `SandboxOptions.java:197-217`];
  cgroups are Linux-only [V: `VirtualCgroup.java:82-85`];
  `--watchfs` is a no-op on Windows without an experimental flag [V: `LocalDiffAwareness.java:52-75`];
  graceful cancel needs console control events [V: `blaze_util_windows.cc:299-322`;
  `bzd/node-child_process.md:1748-1754`].
- Why it matters: out of scope for 0.x, but HC5 later requires Windows and macOS runners, and the cgroup requirement
- has no equivalent there.
