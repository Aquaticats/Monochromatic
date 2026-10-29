# From-scratch monorepo manager: design inputs

Research date: 2026-09-16.
Scope: read-only research feeding a concrete design for the from-scratch route in
`doc/planning/monorepo-manager-build-routes.md` ("Route B").
Nothing under `/var/home/user/Monochromatic` was modified.

## Conventions used here

- **Verified** means a cited file and line range (repository),
  a cited official doc or source file (external),
  or a measurement command run in this session with its output quoted.
- **Unverified** means inference, recall, or a claim the cited source does not settle.
- External sources were read from shallow clones or single-file downloads:
  - `/home/user/temp/agent/bsp-2026-09-16` (build-server-protocol/build-server-protocol)
  - `/home/user/temp/agent/lsp-2026-09-16` (microsoft/language-server-protocol, `gh-pages`)
  - `/home/user/temp/agent/watchman-2026-09-16` (facebook/watchman)
  - `/home/user/temp/agent/pueue-2026-09-16` (Nukesor/pueue)
  - single files under
  - `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad/src/`
    (Tilt CLI source and cancel extension, Turborepo and Nx doc sources, Bazel caching doc, REAPI proto,
    btrfs-progs docs and `cmds/subvolume.c`, Linux `fs/btrfs/ioctl.c`, `fs/btrfs/send.c`,
    `Documentation/admin-guide/cgroup-v2.rst`, `Documentation/userspace-api/landlock.rst`,
    OpenZFS man pages, systemd man sources and `docs/CGROUP_DELEGATION.md`, bubblewrap README,
    process-wrap `src/std/process_group.rs`).
  Below, `SRC/` abbreviates that scratchpad `src/` directory.
- Web pages fetched through a summarizing fetcher are cited by URL and marked "(fetched page)";
  their quotes are less exact than file line citations.
- Process note:
  `btrfs --version` was run once as a probe before re-reading the "do not run third-party tools" instruction;
  no other third-party tool was run.

## Stack framing (per coordinator correction)

The implementation language of the daemon is **not decided**.
Every "reuse" finding in Part 1 is conditional on a TypeScript on Node stack.
Only Linux support is required for 0.x;
macOS and Windows appear as notes only.
Tension to record, not resolve:
the vet's frozen constraint HC5 still lists macOS and Windows CI runners
(`doc/audit/tech-monorepo-manager-vet-2026-09-16.md:244-248`, verified).

## Part 1: repository inventory

### Requirement codes in scope

- FE01 to FE24 are defined in `doc/audit/tech-monorepo-manager-vet-2026-09-16.md:97-193` (verified).
  FE14 (Cargo manifests) and FE15 (JetBrains LSP4IJ) are pluggable;
  FE18 and FE19 may be delegated to Meta Package Manager (`:195-217`, verified).
- WR1 to WR3 are at `:221-227` (verified).
- Recorded inferences:
  inspection means current and recent activity;
  control means at least start or rerun and cancel (`doc/planning/mise-removal-coverage.md:52-58`, verified).

### file-enforcer internals (reuse conditional on TypeScript on Node)

#### Entry point and packaging

- CLI is a top-level-await script, not a `bin`:
  `package/file-enforcer/core/src/cli.ts:21-53` parses `process.argv` by hand (`--watch` plus one positional),
  discovers `file-enforcer.config.ts` upward with `findRoot({ marker: fileNamed(...) })` (`:40-44`),
  then `await import(configPath)` and optionally `startWatching` (verified).
- `package.json` has no `bin`, declares `"sideEffects": false` (`package/file-enforcer/core/package.json:1-46`,
- verified),
  yet `src/io/staleness-manifest.ts:146-149` registers a `process.on('beforeExit')` hook at import time (verified).
- Root tasks invoke source directly:
  `sync:files` runs `node package/file-enforcer/core/src/cli.ts` and `watch:sync:files` adds `--watch`
  (`mise.no-env.toml:1120-1127`, verified).

#### Watch loop (`src/watch/`)

- `startWatching(configPath): Promise<never>` (`src/watch/watch.ts:44-286`, verified):
  - rerun = `invalidatePaths` + tracker `reset()` + `await import(`${absoluteConfig}?v=${Date.now()}`)`,
    then tear down every watcher and rebuild them (`:79-93`).
  - events accumulate in two sets and are debounced (`DEBOUNCE_MS = 100`, `src/watch/watch-dir.ts:27`)
    into one `WatchRerunBatch` (`watch.ts:123-171`).
  - returns only via `lifecycle.failure` rejection (`:282-285`); there is no stop handle.
- Rerun queue (`src/watch/watch-rerun-queue.ts`, verified):
  - public surface is `enqueue`, `pendingCount`, `running` (`:42-57`); FIFO, strictly serial drain (`:225-270`).
  - no cancel, no identity for a batch, no record of what ran or how it ended;
    handler errors go to `onError` and draining continues (`:257-268`).
- Supervisor (`src/watch/watch-supervisor.ts`, verified):
  per-directory restart loop, `WATCHER_RESTART_LIMIT = 3`, `WATCHER_RESTART_DELAY_MS = 50` (`:11-16`),
  throws a fail-closed error after the limit (`:236-292`).
- Lifecycle (`src/watch/watch-lifecycle.ts:62-178`, verified):
  abort-controller registry, single debounce timer, one-shot `fail()` that rejects a shared `failure` promise.
  Failure is process-wide; there is no restart of watch mode after failure.
- Directory watcher (`src/watch/watch-dir.ts:62-285`, verified):
  one chokidar instance per directory with `depth: 0`, `atomic: true`, `ignoreInitial: true` (`:231-241`).
- Watched set and classification (`src/watch/watch-filter.ts`, verified):
  - `watchDirs` = parent dirs of tracked reads, writes, config, plus nearest existing glob static roots (`:43-66`).
  - `classifyEvent` returns `protected` when a tracked destination's `floor(mtimeMs) > recorded write time`,
    when no write time exists, or when `stat` fails; equal-or-older mtime is an `ignore` echo (`:149-179`).
  - Sources match tracked reads, the config, glob static roots, or a fresh `expandGlob` of every tracked glob per event
  - (`:81-99`, `:186-190`).
- Protected destinations (FE22):
  the "revert" is the ordinary rerun rewriting managed content;
  `notifyWriteProtection` logs "reverting to enforced content" and sends a desktop notification
  (`src/watch/notify.ts:83-89`, verified).
  The macOS branch interpolates the path into an AppleScript string without escaping (`:135-143`, verified);
  irrelevant for Linux-only 0.x, but a syntax-boundary defect if reused (AGENTS.md `SYB`, line 1093).
- Known gaps recorded by the package itself (`package/file-enforcer/core/TODO.md`, verified):
  no SIGINT/SIGTERM shutdown (`:28-32`), whole-config rerun on any change (`:41-50`),
  fixed debounce with burst splitting (`:24-27`), `exec()` without stdin, env, cwd (`:72-75`).

#### Tracking (`src/tracker.ts`, `src/tracker-capture.ts`)

- Module-level mutable singletons: `_reads`, `_writes`, `_writeTimestamps`, `_globs` (`src/tracker.ts:21-38`, verified).
  `reset()` clears all but write timestamps (`:88-92`).
- Per-builder dependency capture uses `AsyncLocalStorage` (`src/tracker-capture.ts:62`, `:159-185`, verified).
  Only reads routed through `cat`/`addWatchedPaths` are captured (`src/tracker.ts:118-125`, `:241-245`, verified).
- Undeclared reads are real in the root config:
  it imports `glob`, `lstat`, `readFile` from `node:fs/promises` and `nano-spawn` directly
  (`file-enforcer.config.ts:1-13`, verified),
  and `generateMiseToml` globs `package/*/*/node_modules/.bin` outside the tracker (`file-enforcer.config.ts:676-689`,
  verified).

#### Staleness cache (`src/io/staleness-*`)

- Keyed per destination (`single:<abs dest>`) or destination glob (`each:<abs glob>`)
  (`src/io/staleness-manifest.ts:196-214`, verified).
- Manifest location: nearest ancestor `node_modules/.cache/file-enforcer/staleness-manifest.json`
  (`src/io/staleness-manifest.ts:163-182`, `src/io/staleness-types.ts:16-24`, verified).
- **Source freshness is `stat` size plus `mtimeMs`, not content hashes**
  (`src/io/staleness-stamps.ts:76-97`, `:179-194`, verified).
  `sourceSetHash` hashes that metadata JSON "for human inspection and equality debugging"
  (`src/io/staleness-hash.ts:43-56`, `src/io/staleness-types.ts:131-134`, verified).
- Destinations check size, mtime, **and** a SHA-256 of current content (`src/io/staleness-destination-match.ts:30-76`,
- verified).
- The only implicit dependency is the config file path itself (`src/context.ts:74-91`, verified);
  modules imported by the config are not stamped (inference from that code, unverified by test).
- Lazy builders skip when fresh and otherwise capture reads, write, then record (`src/io/write-lazy.ts:101-156`,
- verified).
  The root config uses eager string content, so it records no entries (`TODO.md:46-50`, verified).
- In-process manifest cache is loaded once and reused (`src/io/staleness-manifest.ts:229-246`, verified).
- Cross-process merge: under the lock, read disk, then spread `{...disk, ...thisProcess}` and write atomically
  (`src/io/staleness-manifest-persist.ts:142-174`, verified);
  the process's in-memory copy wins per key even when disk is newer.
- Lock (`src/io/staleness-manifest-lock*.ts`, verified):
  - `mkdir <manifest>.lock` as the lock, 10 ms poll, 5 s timeout (`staleness-manifest-lock.ts:21-31`, `:132-178`).
  - owner file with `pid` and `createdAt` (`staleness-manifest-lock-owner.ts:134-151`);
    liveness by `process.kill(pid, 0)`, `EPERM` counts as alive (`:220-242`).
  - recovery: dead owner is recoverable; absent owner falls back to 60 s directory age
  - (`staleness-manifest-lock-recovery.ts:14`, `:83-103`).

#### Other process-lifetime caches that matter for a daemon

- Unbounded read cache `readCache: Map<string, string>` (`src/io/cache.ts:10`, verified), invalidated only by watch
- events.
- Active config path holder (`src/context.ts:14`, verified).
- Package manager and root detection memoized for the process (`src/package/manager.ts:92-94`, `:187-189`, verified).
- Notification tool detection memoized (`src/watch/notify.ts:67`, verified).
- `findRootCached` memoizes rejections for the process lifetime
- (`package/fs-path/core/src/root-discovery.ts:293-315`, verified).

#### What changes for a long-lived daemon that reports activity

All conditional on keeping TypeScript on Node.

- Rerun isolation.
  Node loads a module again for each distinct query string (https://nodejs.org/api/esm.html, fetched page),
  so `?v=Date.now()` creates a new module record per rerun while transitive imports stay cached.
  Whether old records are ever collected is not documented on that page (unverified).
  Running each config evaluation in a child process gives fresh transitive imports,
  bounded memory, isolation of the module singletons listed above,
  and a real cancellation point (kill the child); a dynamic `import()` cannot be aborted (unverified, recall).
- Instance scoping.
  Tracker sets, read cache, manifest cache, active config path, and lifecycle are module-global;
  a daemon hosting file-enforcer beside other tasks needs them per run or per config.
- Lifecycle.
  `startWatching` needs a stop handle, signal-driven shutdown, and restart after failure instead of `Promise<never>`.
- Activity events.
  The queue exposes only `pendingCount()` and `running()`; a daemon needs typed events:
  batch accepted (paths, protected paths), run started, run finished (outcome, duration, destinations written,
  protected reverts),
  watcher restart, watcher failure.
- Logging is not an event bus:
  `LogRecord` is `{ level, message, timestamp }` with no tag or structured fields
  (`package/logger/core/README.md:258-268`, verified),
  so recent activity cannot be reconstructed from logs.
- Watcher ownership.
  Per-rerun teardown and recreation of every chokidar watcher (`watch.ts:91-92`) should become subscriptions
  on one repository-wide watcher service.
- Cache semantics.
  The staleness manifest is a metadata stamp cache for file-enforcer builders, not a content-hash task cache.
  It can stay as FE09, but the task cache needs its own content-addressed key (Part 3).
- Lock interaction with pause.
  A frozen or stopped holder still appears live to `process.kill(pid, 0)`, so waiters time out after 5 s
  with `StalenessManifestPersistenceError` (`staleness-manifest-lock.ts:165-172`, verified code path; the freeze
  scenario is inference).

### `@monochromatic-dev/watch-restart-cli`

- Scope: watch paths, restart one child (`package/watch-restart/cli/README.md:5-16`, verified).
- Process-group ownership default:
  spawn `detached: true` and signal `-pid` (`src/child.ts:736-750`, verified);
  stop sends the configured signal, waits `stopTimeout`, escalates to `SIGKILL` (`src/child.ts:763-813`, verified).
- Handle exposes only `stop()` (`src/start.ts:195-200`, verified); no status, no rerun, no pause.
- No Windows-specific handling (rg for `win32|windows|taskkill` found only path-separator notes, verified).
- In-memory content-hash filter pre-populated during initial scan (`README.md:368-373`, verified).
- Reuse value (TS on Node): restart-driver semantics and filter composition for persistent tasks such as dev servers.

### `@monochromatic-dev/monorepo-task-core-exec`

- Binaries: `task-command`, `task-append`, `task-depends`, `task-oxlint`, `task-pnpm`, `task-tsc`
  (`package/monorepo-task/core-exec/package.json:8-15`, verified).
- `task-depends` staleness is timestamp-based: `strategy(sources) > strategy(outputs)` with `sh:` timestamp commands
  (`README.md:258-307`, verified).
- Mise's own `sources`/`outputs` default to hashing `(path, size)` plus mtime comparison
  (`TROUBLESHOOTING.mise-sources.md:16-37`, verified as the repository's reading of mise source).
- Reuse value: `task-oxlint` and `task-tsc` wrappers are the real lint and type-check entry points
  (`mise.no-env.toml:589-611`, verified).
  The timestamp model is not reusable as a content-hash cache.

### `@monochromatic-dev/mcp-stdio` (JSON-RPC 2.0 framing)

- Newline-delimited JSON-RPC 2.0; `readLines` accepts any `AsyncIterable<Uint8Array>` (`src/line-reader.ts:22-77`,
- verified).
- `serve({ server, input, output })` takes injectable input and writer (`src/transport.ts:131-141`, verified),
  so a `net.Socket` could be wired in (unverified, not tested).
- Outbound types are responses only: `JsonRpcOutbound = JsonRpcResponse | JsonRpcErrorResponse`
- (`src/json-rpc.ts:108-116`, verified);
  server-to-client notifications (needed for subscriptions) are not modeled.
- Serial execution queue with MCP cancellation semantics:
  queued-and-cancelled entries are dropped, running ones finish and their frame is withheld
  (`src/transport-queue.ts:94-128`, verified).
  That is the wrong default for build control, where cancel must stop work.
- Deliberately omits progress notifications and subscriptions (`README.md:153-166`, verified).
- Reuse value: message validation guard (`src/json-rpc.ts:177-187`), line framing, backpressure-aware writer
- (`src/transport.ts:70-101`).

### `@monochromatic-dev/fs-path-core` root markers

- `MISE_MONOREPO` matches a `mise.toml` containing a `[monorepo]` header line (`src/root-marker.ts:94-120`, verified).
- Factories `fileNamed`, `directoryNamed`, `packageNamed`; `PNPM_WORKSPACE`, `GIT_REPOSITORY`
- (`src/root-marker.ts:137-260`, verified).
- 25 TypeScript files reference `MISE_MONOREPO` (rg `MISE_MONOREPO\b`, verified);
  41 source files reference `MISE_*` env vars, `'mise', [`, or `mise run ` (rg + `wc --lines`, verified).
  The replacement needs its own root marker and an equivalent of `MISE_MONOREPO_ROOT`/`MISE_CONFIG_ROOT` for task
  processes.

### Node fanout scripts in `mise.no-env.toml`

- `fanout` (`mise.no-env.toml:282-306`, verified):
  runs `mise tasks --all --hidden --json`, selects direct children one `:` level deeper than `MISE_TASK_NAME`,
  spawns all as `mise run <child>` in parallel, fails listing failed children.
- `fanout_packages` (`:307-346`, verified):
  selects every `//package/...:<suffix>` task; bounded worker pool of `availableParallelism()`,
  except `watch*` suffixes which run unbounded so every watcher stays alive.
- `dispatch_workspace_node` (`:347-419`, verified):
  resolves monorepo root, runs a package entry from `dist/final/node/<entry>.mjs` or `src/<entry>.ts`,
  and rebuilds the oxlint config when any config or plugin source is newer than outputs (mtime check).
- These encode today's implicit graph: hierarchical names, fan-out by name prefix, one prerequisite special-cased by
- mtime.

### How tasks are defined across `mise.toml` files

- 178 `mise.toml` files under `package/` (plus 13 in `package-paused`, 2 in `package-deprecated`, 1 root)
  (rg `--files` + `uniq --count`, verified).
- 1,777 `[tasks.*]` headers across package files; 1,324 `extends =` lines, 41 `depends =`, 103 `shell =`, 28 `dir =`,
- 22 `usage =`
  (rg counts, verified).
- Most common package task names (rg + `uniq --count`, verified):
  `lint` 165, `build` 139, `format:oxlint` 138, `lint:oxlint` 123, `lint:types` 104,
  `watch:build:js` 85, `watch:build` 85, `test:unit` 83, `build:js` 74, `watch:build:js:node` 67,
  `buildAndTest` 54, `build:js:node` 49, `run` 29, `watch:build:js:browser` 26, `test` 23,
  `lint:clippy` 19, `build:js:browser` 18, `lint:rust` 17, `format:clippy` 17.
- Templates live in the root (`mise.no-env.toml:510-619`, verified):
  aggregate names run `vars.fanout`; leaves run `rolldown --configLoader native --config rolldown.<target>.config.ts`,
  `rolldown --watch ...`, the `test:unit` Node runner, or task-util wrappers.
- `test:unit` template excludes files containing `.expensive.` unless `--all` (`mise.no-env.toml:549-579`, verified).
- Policy comment: "never use depends or post depends, use run only" (`mise.no-env.toml:623-624`, verified).
  The 41 `depends` lines are intra-package (for example `package/test-harness/core/mise.toml:7`, verified);
  no cross-package build dependency is declared.
- Mise metadata reports no tasks with sources, outputs, post-dependencies, wait-for, or task-local tools
  (`doc/planning/mise-removal-coverage.md:86-92`, verified).
- Native manifests under `package/`: 160 `package.json`, 19 `Cargo.toml`, 3 `build.gradle.kts`, 2 `settings.gradle.kts`
  (rg `--files`, verified); no `build.zig` matched the query.
- Root `mise.toml` is generated from `mise.no-env.toml` by file-enforcer (`file-enforcer.config.ts:652-724`, verified).
- Monorepo discovery config: `config_roots = ["package/*/*"]` (`mise.no-env.toml:14-24`, verified).

### Repository constraints from `AGENTS.md` that shape the design

- Languages:
  approved-language rule is a proposal, not an accepted rule (`doc/planning/load-bearing-code-languages.md:3-17`,
  `:117-128`, verified);
  it states TypeScript is the default for repository automation (`:79-82`) and that no universal allowlist exists
  (`:108-115`).
  Existing rule `SCR`: never write bash or PowerShell scripts; inline mise logic as Node TypeScript or a package bin
  (`AGENTS.md:956-960`, verified).
- Cross-runtime: `XRT` prefer cross-runtime patterns over Bun-specific ones (`AGENTS.md:948-949`, verified).
- Home paths: `HOM` derive from injected home or runtime homedir (`AGENTS.md:951-954`, verified).
- Logging: `LOG` log extensively, never remove (`AGENTS.md:1064`); `TLG` tagged loggers only, raw console for CLI
- output (`:1073`);
  `LG1` tag every function boundary (`:1080`); `LG2` use catch bindings (`:1086`) (verified by tag line).
- Max lines: `MXL` never bypass (`AGENTS.md:997-1003`, verified);
  oxlint `eslint/max-lines` is 300 with blank lines and comments skipped
  (`package/config/oxlint/src/rule/style.ts:47-54`, verified);
  `**/*.config.*` files are exempt (`package/config/oxlint/src/overrides.ts:130-135`, verified);
  Rust has the same 300-line budget (`MXR`, `AGENTS.md:1005-1013`, verified).
- Docs in code: `TSD` TSDoc on all declarations (`AGENTS.md:1121`), `RDC` rustdoc on all Rust items (`:1015-1018`)
- (verified).
- Errors: `PP4` custom error classes and `@throws` (`AGENTS.md:1281`), never `process.exit`
- (`unicorn/no-process-exit`), never swallow (`LG2`),
  `PP8` throw on unreachable (`:1299`), no `switch` (`no-restricted-syntax/no-switch`) (verified by tag line).
- Config: `AD2` switch to TypeScript config when config needs logic (`AGENTS.md:1730`), `AD3` direct async execution
- over descriptor or interpreter (`:1735`) (verified).
  The user relaxed FE01: configuration need not be TypeScript (`tech-monorepo-manager-vet-2026-09-16.md:97-101`,
  verified).
- CLI packaging: `AP4` `#!/usr/bin/env node` shebang for `bin` packages (`AGENTS.md:1396`); `CM3` builds via tasks
- (`:860`) (verified).
- Workspace imports: `ST3` `/ts` source subpath (`AGENTS.md:1167`, verified).
- Package completeness: `PKG` README, zero lint errors, tests on every exported path (`AGENTS.md:1420`, verified).
- Agent spawning guard: `SPG` (`AGENTS.md:965-969`, verified); relevant if tasks ever launch agents.
- Heavy runs isolated: `RXI` (`AGENTS.md:825`, verified); a whole-repo build-and-test daemon is itself such a workload.

## Part 2: prior art for inspecting and controlling a long-running build or watch process

### Build Server Protocol

- Transport and framing: "The base protocol is identical to the language server base protocol"
  (`/home/user/temp/agent/bsp-2026-09-16/docs/specification.md:83-91`, verified).
  Discovery: connection files with `argv`; the client spawns `argv` and talks over stdin/stdout,
  and the tool may bridge to a running daemon over Unix sockets or named pipes
  (`docs/overview/server-discovery.md:127-184`, verified).
  Every client gets "a fresh BSP connection" (`:143-145`, verified).
- Lifetime is client-managed (`docs/specification.md:101-106`, verified); a build-tool daemon behind `argv` is outside
- the protocol.
- Task notifications (`website/generated/specification.md`, verified):
  - `build/taskStart`, `build/taskProgress`, `build/taskFinish` share `taskId` (`:1748-1772`).
  - `TaskStartParams { taskId, originId?, eventTime?, message?, dataKind?, data? }` (`:1776-1797`).
  - `TaskProgressParams` adds `total?`, `progress?`, `unit?` (`:1839-1869`).
  - `TaskFinishParams` adds `status: StatusCode` (`:1899-1923`); `StatusCode` is `Ok = 1`, `Error = 2`, `Cancelled = 3`
  - (`:353-364`).
  - `TaskId { id, parents? }` enables tree rendering (`:327-341`).
  - `CompileParams.originId`: "The server may include this id in triggered notifications or responses" (`:1029-1031`).
- Subscription model: none; notifications flow to the one connected client.
- Cancellation: `$/cancelRequest` is defined only in
- `spec/src/main/resources/META-INF/smithy/bsp/extensions/cancel.smithy:16-26` (verified);
  rg over `website/generated` and `docs` finds no `cancelRequest` (verified).
  `buildTarget/run`: "Cancelling this request must kill the running process"
  (`website/generated/specification.md:1098`, verified).
- Documentation quality notes (verified):
  - Prose says spawned-by-request tasks "should reference the request's `originId` parent" and names a `parent` field
  - (`:1767-1769`),
    while `TaskId` has `parents` and says "OriginId should not be included in the parents field" (`:332-339`).
  - `TaskFinishParams.eventTime` is documented as "when the event started" (`:1907`).
  - Generated Markdown contains unrendered `[[bsp#BuildTargetCompile]]` links (`:1806`, `:1932`).
  - Status section says "not an approved standard" and "subject to change" (`docs/specification.md:52-54`).

### Watchman

- Transport: "The server will create a unix domain socket"; locate via `watchman get-sockname` or `$WATCHMAN_SOCK`
  (`/home/user/temp/agent/watchman-2026-09-16/website/docs/socket-interface.md:11-18`, verified).
  Windows is a supported install target (`website/docs/install.md:13`, `:30-42`, verified)
  but the socket-interface page documents no Windows transport (rg, verified).
- Framing: JSON PDUs are single-line compact JSON terminated by newline; requests are arrays, responses objects;
  BSER binary alternative (`socket-interface.md:22-52`, verified).
- Subscription model: `subscribe` and `log-level` let the server "unilaterally send any number of PDUs"
  (`socket-interface.md:22-25`, verified).
  Initial result then results after the settle period, each carrying a `clock` (`website/docs/cmd/subscribe.md:42-58`,
  verified);
  a `since` clockspec resumes from a prior clock (`:61-75`).
  `defer` and `drop` hold or discard notifications while a named state (`state-enter`) is asserted (`:120-196`,
  verified).
- Clocks: abstract clock ids avoid timestamp races (`website/docs/clockspec.md:13-22`, verified);
  named cursors need an exclusive lock and cannot roll back (`:24-40`).
  Queries older than the prune clock return a fresh instance (`website/docs/config.md:216-222`, verified).
- Triggers: output goes to the Watchman log by default; settle before running;
  "Watchman will only run a single instance of the trigger process at a time", then re-evaluates from the spawn-time
  clock
  (`website/docs/cmd/trigger.md:8-28`, verified).
- Cancellation: no documented way to cancel a running trigger process;
  rg for `cancel|kill|sigterm` in docs finds only subscription and watch cancellation (verified).
- Errors: `error` and `warning` fields in responses (`socket-interface.md:54-69`, verified).
- Documentation quality note: `config.md:221` refers to `is_fresh_instance` "elsewhere in this document",
  while it is described in other pages (`file-query.md:80`, `cmd/query.md:49`) (verified).

### Tilt

- Transport: `tilt get` and `tilt trigger` talk to "the Tilt HTTP server", `--host` default `localhost`, `--port`
- default `10350`
  (https://docs.tilt.dev/cli/tilt_get.html and https://docs.tilt.dev/cli/tilt_trigger.html, fetched pages).
- API server: "a full-fledged Kubernetes API server" usable with `kubectl` and a kubeconfig; in-memory types over HTTP
  (https://github.com/tilt-dev/tilt-apiserver README, fetched page).
  Object reference is autogenerated at https://api.tilt.dev, all `tilt.dev/v1alpha1` (fetched page).
- Subscription model: `tilt get -w/--watch` and `--watch-only` stream changes (Kubernetes watch semantics) (fetched
- page).
- Inspection object: `UIResource` status has `buildHistory`, `currentBuild`, `pendingBuildSince`, `queued`,
  `runtimeStatus`, `updateStatus`, `disableStatus`, `triggerMode`, `waiting { reason, on }`
  (https://api.tilt.dev/interface/ui-resource-v1alpha1.html, fetched page).
- Control:
  - `tilt trigger` posts `{"manifest_names":[...], "build_reason": ...}` to the `trigger` endpoint via `apiPostJson`,
    not to the Kubernetes-style API (`SRC/tilt-cli-trigger.go:61-63`, verified).
  - `tilt disable` "will stop the running process (if any) and delete any objects owned by that resource"
    (https://docs.tilt.dev/disable_resources.html, fetched page).
  - Cancel of a running `local_resource` is not core:
    the `cancel` extension adds a button that "kills a running process by sending a TERM signal",
    is written in bash with `jq`, and lists "Windows Support (this currently only works on Unix)" as future work
    (`SRC/tilt-cancel-README.md:5-9`, `:11-14`, `:41-44`, verified);
    it reads `.status.running.pid` from `tilt get cmd -o json` (`SRC/tilt-cancel-reconcile.sh:17`, `:50`, verified).
  - `UIButton` clicks surface as `status.lastClickedAt` (https://api.tilt.dev/interface/ui-button-v1alpha1.html,
  - fetched page),
    so controllers watch objects rather than receive calls.
- Documentation quality notes:
  `tilt trigger` docs do not say what happens if the resource is already building (fetched page, absence);
  cancellation lives in an extension, not core docs.

### Gradle Tooling API

- Transport: JVM library; "The Tooling API always uses the Gradle daemon"
  (https://docs.gradle.org/current/userguide/tooling_api.html, fetched page).
  The user guide page does not describe the client-daemon wire protocol (absence on fetched page).
- Events: `LongRunningOperation.addProgressListener(org.gradle.tooling.events.ProgressListener, Set<OperationType>)`
  filters by `OperationType` (`BUILD_PHASE`, `FILE_DOWNLOAD`, `GENERIC`, `PROBLEMS`, `PROJECT_CONFIGURATION`, `ROOT`,
  `TASK`, `TEST`, `TEST_METADATA`, `TEST_OUTPUT`, `TRANSFORM`, `WORK_ITEM`)
  (https://docs.gradle.org/current/javadoc/org/gradle/tooling/LongRunningOperation.html and
  `.../events/OperationType.html`, fetched pages).
  `OperationDescriptor` has `getName`, `getDisplayName`, `getParent` (`.../events/OperationDescriptor.html`, fetched
  page).
- Cancellation: `withCancellationToken`; `CancellationTokenSource.cancel()` is best effort, and a cancelled operation
- reports
  `BuildCancelledException` (`.../CancellationTokenSource.html`, fetched page).
- Compatibility: supports running builds for the last five major Gradle releases (tooling API guide, fetched page).
- Relevance: the repository's Kotlin package builds with Gradle; the daemon persists after builds
  ("long-lived background process", 3 h idle stop, reused only with identical Java home and JVM args,
  `--no-daemon` may still spawn a single-use daemon)
  (https://docs.gradle.org/current/userguide/gradle_daemon.html, fetched page).

### LSP `$/cancelRequest` and `$/progress`

- Framing: `Content-Length` header, `\r\n\r\n`, UTF-8 JSON-RPC body
  (`/home/user/temp/agent/lsp-2026-09-16/_specifications/lsp/3.17/specification.md:26-64`, verified).
- Transport options recommended as CLI args: `stdio`, `pipe` (named pipe or socket file), `socket` (port) (`:704-708`,
- verified).
- `$/` methods are implementation dependent; notifications may be ignored; `$/` requests get `MethodNotFound`
- (`:332-334`, verified).
- Cancellation: `CancelParams { id }`; "A request that got canceled still needs to return from the server and send a
- response back";
  advised error `RequestCancelled` (`:336-353`, verified); `RequestCancelled = -32800` (`:303`, verified).
- Progress: `$/progress { token, value }`; "The token is different than the request ID which allows to report progress
- out of band" (`:355-385`, verified).
- Work done progress: begin/report/end with `cancellable`, `message`, `percentage`
  (`_specifications/lsp/3.17/types/workDoneProgress.md:14-98`, verified);
  client-initiated via `workDoneToken`, server-initiated via `window/workDoneProgress/create` (`:112-113`, `:189-191`);
  cancelling client-initiated progress is cancelling the request (`:165-166`);
  `window/workDoneProgress/cancel` cancels server-initiated progress "for any number of reasons"
  (`_specifications/lsp/3.17/window/workDoneProgressCancel.md:3`, verified).
- Ordering: responses roughly in request order, reordering allowed when results are unaffected
- (`specification.md:420-424`, verified).
- Scope limit: "one server serves one tool", no sharing between tools (`:412`, verified).
- Machine-readable spec: `_specifications/lsp/3.17/metaModel/metaModel.json` (file exists, verified).

### pueue (added per user requirement)

- Architecture: client `pueue`, daemon `pueued`, shared `pueue_lib`; daemon runs a task loop and a message loop
  (`/home/user/temp/agent/pueue-2026-09-16/docs/Architecture.md:18-76`, verified).
- Transport: Unix socket by default (`use_unix_socket: true`), default mode `0o700`, path `pueue_<username>.socket`,
  optional TCP plus TLS with shared secret file (`pueue_lib/src/settings.rs:39-55`, `:82-83`, `:184-195`, `:301-316`,
  verified).
  Handshake: authorization (secret and certificate), then the daemon's version as confirmation
  (`docs/Architecture.md:69-76`, verified).
- Framing: u64 big-endian length header then CBOR (`ciborium`) payload in 1,280-byte chunks
  (`pueue_lib/src/network/protocol.rs:1-85`, `:97-140`, verified).
  The doc comment says "u64 as 4bytes" (`:61`, `:94`) while the code writes an 8-byte `u64` header (`:66-67`, `:102`)
  (verified).
- Requests include `Add`, `Start`, `Restart`, `Pause`, `Kill`, `Send`, `Group`, `Parallel`, `Status`, `Log`, `Stream`,
- `Reset`, `DaemonShutdown`
  (`pueue_lib/src/message/request.rs:27-79`, verified); signals `SigInt`, `SigKill`, `SigTerm`, `SigCont`, `SigStop`
  (`:190-203`).
  `Stream` gives a continuous log stream of tasks (`:71-72`, `:277-289`); there is no general event subscription
  (verified by variant list).
- Scheduling: next task = queued, group running, group has free slot (`parallel_tasks == 0` means unlimited),
- dependencies succeeded;
  order by higher priority first, then lower id (`pueue/src/daemon/process_handler/spawn.rs:25-106`, verified).
  Priority is set at add (`pueue/src/client/cli.rs:77-81`) or on restart (`pueue_lib/src/message/request.rs:174-175`),
  not in place (verified by rg).
- Pause: a group pause marks the group `Paused`; with `wait` running tasks continue, otherwise each running task gets
- `ProcessAction::Pause`
  (`pueue/src/daemon/process_handler/pause.rs:10-69`, verified), which maps to `SIGSTOP` and resume to `SIGCONT`
  (`pueue/src/process_helper/unix.rs:28-35`, verified).
- Process tree handling: tasks are spawned with `process_wrap` `ProcessGroup::leader()`
- (`pueue/src/process_helper/unix.rs:113`, `:130`, verified);
  that wrapper signals with `killpg(pgid, sig)` and reaps with `waitpid(-pgid)`
  (`SRC/process-wrap-process_group.rs:103-104`, `:108-123`, verified).
  Windows uses a `JobObject` wrapper (`pueue/src/process_helper/windows.rs:316`, verified).
- Documentation quality notes (verified):
  user docs live in the GitHub wiki (`README.md:121-128`);
  the protocol is documented only through `pueue_lib` rustdoc and source;
  `docs/Architecture.md:21` links `pueue-lib` as a separate repository while it is in-tree;
  README declares the project "feature-complete" (`README.md:19-20`).

## Part 3: input hashing prior art

### Turborepo

Source: `SRC/turbo-caching.mdx` and `SRC/turbo-configuration.mdx` (vercel/turborepo `apps/docs/content/docs/...`).

- Two hashes, global and task; either change misses (`turbo-caching.mdx:216-218`, verified).
- Global hash inputs (`:220-230`, verified):
  resolved task definition from root and package `turbo.json`;
  lockfile changes affecting the workspace root;
  source files of internal packages the root depends on;
  `globalDependencies` file contents;
  values of `globalEnv` variables;
  behavior-changing flags (`--cache-dir`, `--framework-inference`, `--env-mode`);
  passthrough arguments (`turbo build -- --arg=value`).
- Package hash inputs (`:249-256`, verified):
  package `turbo.json`; **lockfile changes that affect the package** (a lockfile slice);
  package `package.json`; files, defaulting to "all source-controlled files in the package directory".
- `inputs` (`turbo-configuration.mdx:875-934`, verified):
  `package.json`, `turbo.json`, and lockfiles are always inputs;
  specifying `inputs` opts out of `.gitignore` handling; `$TURBO_DEFAULT$` restores defaults; `$TURBO_ROOT$` roots a
  glob at the repo.
- Deferred hashing `mode: "jit"` hashes after dependencies complete; `--dry=json` then reports `hash: null`
- (`:936-996`, verified).
- Environment: `env` and `globalEnv` affect hashes (`:60-70`, `:744-810`); `passThroughEnv` values do not (`:72-90`,
- `:812-831`);
  `envMode` default `"strict"` filters task env to declared variables (`:255-270`) (verified).
- Tool versions: not listed in either hash input table (`turbo-caching.mdx:220-256`, verified by absence).
- Watch: without `watchUsingTaskInputs`, `turbo watch` reruns all tasks in a changed package;
  root config, lockfile, and `globalDependencies` changes rerun everything (`turbo-configuration.mdx:392-420`,
  verified).
  Persistent tasks are restarted only when `interruptible` (`:1116-1124`, verified).
- Daemon: deprecated for `turbo run`, still used by `turbo watch` and the LSP (`:249-253`, verified).

### Nx

Source: `SRC/nx-inputs.mdoc` and `SRC/nx-how-caching-works.mdoc` (nrwl/nx `astro-docs/...`).

- Hash may include project and dependency source files, workspace configuration, external dependency versions,
  runtime values such as OS and CPU architecture, and command arguments (`nx-how-caching-works.mdoc:9-18`, verified).
- Input kinds (`nx-inputs.mdoc`, verified):
  project configuration always (`:17-19`); command arguments (`:21`);
  file sets with `!` exclusion and `^` dependency scoping (`:27-107`); `.gitignore`d files never hashed (`:82`);
  JSON subset inputs that hash parsed content (`:111-132`);
  `{ "env": "API_KEY" }` (`:134-144`);
  `{ "runtime": "node --version" }`, "often used to include versions of tools" (`:146-156`);
  working directory (`:158-173`);
  `externalDependencies`, defaulting to all workspace external dependencies when unspecified (`:175-221`);
  `dependentTasksOutputFiles` with optional `transitive` (`:223-237`);
  inputs of continuous dependencies (`:239-245`).
- `namedInputs` in `nx.json` or project config; conventions `default`, `production`, `sharedGlobals`,
  where `sharedGlobals` may hold "the OS ... or the version of Node" (`:287-376`, verified).
- Lockfile: the inputs page never says "lockfile" (rg, verified); external dependency versions are the documented unit.

### Bazel action key and REAPI

- Each action "has inputs, output names, a command line, and environment variables";
  the action cache maps action hashes to results; outputs live in a CAS
  (`SRC/bazel-remote-caching.mdx:15-35`, verified).
- REAPI `Action` = `command_digest`, `input_root_digest`, `timeout`, `do_not_cache`, `salt`, `platform`
  (`SRC/remote_execution.proto:674-740`, verified);
  timeout is in the key so a short timeout cannot hit a long-timeout result (`:706-714`);
  `salt` namespaces poisoned results (`:723-729`).
- `Command` = `arguments`, lexicographically sorted `environment_variables`, `output_paths`, and more (`:749-871`,
- verified).
- Environment: only `--action_env` allow-listed variables enter the action definition
- (`bazel-remote-caching.mdx:333-342`, verified).
- Tool versions: "Bazel currently does not track tools outside a workspace",
  so different `/usr/bin` compilers "wrongly share cache hits" (`:344-350`, verified).
- Lockfile slices: not covered by these sources (unverified).

## Part 4: daemon requirements research (Linux-only 0.x)

### Stated requirements recorded here

From coordinator messages relaying the user:

- One long-running process in its own terminal; prints logs; reacts to Ctrl+C; TUI later; clients control it over RPC.
- Watches the whole repository by default; on every change builds everything affected and runs the default test set for
- everything affected.
- Caching always on.
- Uses btrfs or ZFS features to speed up change comparison when available; the development repo is on btrfs.
- Every task has a priority (default 0); concurrency defaults to available parallelism with an env override;
  clients can change priority, pause, resume, end tasks.
- Everything in 0.x is Linux only.
- Sandboxing is a must, at least cgroups.
- Heavy suites are test files matching `*.expensive.*.test.*`.
- Filesystem-enhanced caching supports btrfs only in 0.x; ZFS is a brief note.
  The tool may create btrfs subvolumes and snapshots whenever it wishes.
- A `doctor` command tells the user which commands to run or files to edit, and why, to unblock capabilities.

### Measured environment

- Repository filesystem: `stat --file-system --format="%T"` printed `btrfs`;
  `findmnt` shows `/var/home` from subvolume `/home` (`subvolid=257`) with options
  `rw,relatime,seclabel,ssd,discard=async,space_cache=v2,subvolid=257,subvol=/home` (no `user_subvol_rm_allowed`)
  (verified).
- Subvolume roots have inode 256 (`SRC/btrfs/ch-subvolume-intro.rst:1-5`, verified).
  Measured inodes: `/var/home` 256, `/var/home/user` 257, `/var/home/user/Monochromatic` 2595895,
  so the repository is a plain directory inside the `/home` subvolume (verified).
- Kernel `7.2.0-ogc6.1.fc44.x86_64`; cgroup v2 root controllers `cpuset cpu io memory hugetlb pids rdma misc dmem`;
  this shell runs in `user@1000.service/app.slice/...` (verified).
- User manager delegation on this Fedora host (`systemctl cat user@.service`, verified):
  upstream unit `Delegate=pids memory cpu`, `DelegateSubgroup=init.scope`;
  Fedora drop-in `00-uresourced.conf` adds `Delegate=cpu io memory`;
  a local drop-in `20-freeze-hardening.conf` sets `ManagedOOMMemoryPressure=kill` at 80% for 20 s.
  `user@1000.service` and `app.slice` both list `cpu io memory pids dmem` as available and enabled controllers;
  `app.slice` is owned by `user` with mode 755 (verified).
- Active LSMs include `landlock`; `/proc/sys/user/max_user_namespaces` is 254589; systemd 259 (verified).
- `os.availableParallelism()` printed 16; uid 1000 (verified).
- inotify limits: `max_user_watches` 524288, `max_user_instances` 8192 (verified).
- Directory counts: 99,416 directories excluding only `.git`;
  5,480 excluding `node_modules`, `.git`, `target`, `dist` (`find ... | wc --lines`, verified).
- `$XDG_RUNTIME_DIR` is `/run/user/1000`, owner `user`, mode 700 (verified).
- Heavy suites: 4 files match `*.expensive.*` under `package/`, all 4 also match `*.expensive.*.test.*` (rg, verified);
  test file suffix counts include 694 `.unit.test.ts`, 8 `.browser.test.ts`, 3 `.integration.test.ts`, 2
  `.matrix.test.ts`, 1 `.bench.test.ts` (rg, verified).

### Whole-repository change detection on Linux

- inotify "monitoring of directories is not recursive", has per-user watch limits, reports `IN_Q_OVERFLOW` on queue
- overflow,
  races with newly created subdirectories, and misses `mmap` writes and remote filesystems
  (https://man7.org/linux/man-pages/man7/inotify.7.html, fetched page).
- fanotify: unprivileged use since Linux 5.13 requires `FAN_REPORT_FID` and forbids `FAN_MARK_MOUNT` and
- `FAN_MARK_FILESYSTEM`;
  directory entry events need `FAN_REPORT_DIR_FID`/`FAN_REPORT_NAME` (Linux 5.9)
  (https://man7.org/linux/man-pages/man2/fanotify_init.2.html, fetched page).
  A whole-filesystem mark therefore needs `CAP_SYS_ADMIN`.
- Watchman precedent for overflow and bursts: fresh-instance results when history is gone (`config.md:216-222`),
  `state-enter` with `defer`/`drop` for operations like checkouts (`cmd/subscribe.md:120-196`) (verified).

### btrfs features

- `btrfs subvolume find-new <subvolume> <last_gen>`: "List the recently modified files in a subvolume, after *last_gen*
- generation"
  is its entire documentation (`SRC/btrfs/btrfs-subvolume.rst:127-128`, verified); no privilege or output format is
  documented there.
  Implementation (`SRC/btrfs/subvolume.c:1209-1305`, verified):
  runs `BTRFS_IOC_TREE_SEARCH` with `min_transid = oldest_gen` and `max_type = BTRFS_EXTENT_DATA_KEY`,
  prints only items where `sh.type == BTRFS_EXTENT_DATA_KEY && found_gen >= oldest_gen` (`:1274-1280`),
  and ends with "transid marker was %llu" (`:1303`).
  Consequences: data-extent writes only, so deletions, renames, and metadata-only changes are not listed (inference
  from the filter, unverified by experiment);
  the code uses `>=` where the doc says "after".
- `BTRFS_IOC_TREE_SEARCH` and `_V2` return `-EPERM` without `CAP_SYS_ADMIN` (`SRC/btrfs/linux-ioctl.c:1616-1625`,
- `:1646-1656`, verified).
  So `find-new` needs root.
- `BTRFS_IOC_GET_SUBVOL_INFO` returns the subvolume's "Latest transaction id" `generation`
- (`SRC/btrfs/btrfs-ioctl.rst:343-372`)
  and has no capability check in its kernel handler (`SRC/btrfs/linux-ioctl.c:1966-2135`, verified by reading the
  function bodies).
  It is subvolume-wide; for this repository that is the whole `/home` subvolume.
  Whether unrelated writes such as `relatime` updates bump it is unverified.
- Generation is "an internal counter which is updated every transaction" (`SRC/btrfs/btrfs-subvolume.rst:144-146`,
- verified).
- Snapshots:
  "Subvolume creation is not restricted, but snapshots are limited to own subvolumes only",
  and the source must be the subvolume root inode (`SRC/btrfs/linux-ioctl.c:1203-1217`, verified).
  Deletion by a non-root owner needs the `user_subvol_rm_allowed` mount option (`SRC/btrfs/btrfs-subvolume.rst:94-96`;
  kernel `SRC/btrfs/linux-ioctl.c:2383-2399`, verified); the measured mount lacks it.
- `btrfs send --no-data` gives metadata differences between read-only snapshots
  (`SRC/btrfs/btrfs-send.rst:12-29`, `:58-63`, verified)
  but `btrfs_ioctl_send` requires `CAP_SYS_ADMIN` (`SRC/btrfs/linux-send.c:7957-7971`, verified);
  the send man page does not state that (absence, verified).
- Reflinks: shallow copies sharing blocks; `cp --reflink=always`; no cross-filesystem reflink;
  cross-mount reflink within one filesystem works since 5.18; NOCOW and checksum status must match
  (`SRC/btrfs/Reflink.rst:4-33`, verified).
  `FICLONE` permissions are covered in "btrfs unprivileged operations matrix".

### btrfs unprivileged operations matrix (btrfs-only for 0.x)

The user allows the tool to create subvolumes and snapshots whenever it wishes.
Each entry: operation, unprivileged outcome, evidence.

- **Create a subvolume in a directory the user can write.**
  Allowed: kernel comment "Subvolume creation is not restricted" (`SRC/btrfs/linux-ioctl.c:1203-1206`, verified);
  `btrfs_mksubvol` applies the ordinary `may_create_dentry` directory permission check
  (`SRC/btrfs/linux-ioctl.c:881-897`, verified);
  docs: "There are no restrictions for subvolume creation" (`SRC/btrfs/ch-subvolume-intro.rst:80-85`, verified).
  Creation "needs to flush dirty data that belong to the subvolume" (`ch-subvolume-intro.rst:195-200`, verified).
- **Snapshot a subvolume the user owns.**
  Allowed only when `inode_owner_or_capable` on the source and the source is the subvolume root inode
  (`SRC/btrfs/linux-ioctl.c:1203-1217`, verified);
  `btrfs subvolume snapshot`: "If *source* is not a subvolume, btrfs returns an error"
  (`SRC/btrfs/btrfs-subvolume.rst:255-260`, verified).
  Snapshots are not recursive: nested subvolumes appear as empty stubs with inode 2 (`ch-subvolume-intro.rst:87-91`,
  `:184-188`, verified).
  Inode numbers are not filesystem-unique; use `subvolumeid:inodenumber` (`:190-193`, verified).
- **Read-only snapshots.**
  `snapshot -r` makes the new snapshot read-only (`btrfs-subvolume.rst:262-266`, verified);
  toggling flags later needs `inode_owner_or_capable` (`SRC/btrfs/linux-ioctl.c:1313-1325`, verified);
  flipping read-only snapshots back to read-write breaks incremental-send assumptions (`ch-subvolume-intro.rst:57-61`,
  verified).
  Documentation inconsistency: `btrfs-subvolume.rst:45-47` says snapshots are created read-write by default,
  while `ch-subvolume-intro.rst:52-53` says snapshots have the *ro* property "as *true*" (verified).
- **Delete a non-empty subvolume or snapshot.**
  Needs `CAP_SYS_ADMIN` or the `user_subvol_rm_allowed` mount option plus write and exec access
  (`SRC/btrfs/linux-ioctl.c:2383-2412`, verified);
  option doc: "Allow subvolumes to be deleted by their respective owner. Otherwise, only the root user can do that"
  (`SRC/btrfs/ch-mount-options.rst:483-497`, verified).
  Recursive delete likewise needs `CAP_SYS_ADMIN` or the option (`btrfs-subvolume.rst:116-122`, verified).
- **Delete an empty subvolume without the option.**
  "Since 4.18, the rmdir(2) syscall can delete an empty subvolume just like an ordinary directory"
  (`ch-mount-options.rst:492-495`, verified);
  kernel `btrfs_rmdir` returns `ENOTEMPTY` above empty size, and for a subvolume root calls `btrfs_delete_subvolume`
  (`SRC/btrfs/linux-inode.c:4917-4934`, verified);
  runtime detection: `rmdir_subvol` feature file (`SRC/btrfs/btrfs-man5.rst:161-165`), present at
  `/sys/fs/btrfs/features/rmdir_subvol` on this host (measured).
  Cost: every file in the snapshot must be unlinked first (inference).
- **List changed files since a generation (`find-new`).**
  Not possible unprivileged: it uses `BTRFS_IOC_TREE_SEARCH`, which returns `-EPERM` without `CAP_SYS_ADMIN`
  (`SRC/btrfs/subvolume.c:1209-1244`, `SRC/btrfs/linux-ioctl.c:1616-1625`, verified),
  and even as root it reports only `EXTENT_DATA` items (`subvolume.c:1274-1280`, verified).
- **Read a subvolume generation.**
  Possible unprivileged through `BTRFS_IOC_GET_SUBVOL_INFO` (`generation`, "Latest transaction id of this subvolume",
  `SRC/btrfs/btrfs-ioctl.rst:343-372`), whose kernel handler has no capability check
  (`SRC/btrfs/linux-ioctl.c:1966-2135`, verified by reading).
  Useful only as a subvolume-wide "something changed" signal; whether atime-only activity bumps it is unverified.
- **Send stream between snapshots (`send --no-data`).**
  Root only (`SRC/btrfs/linux-send.c:7957-7971`, verified).
- **Reflink copies for output restoration.**
  `FICLONE` needs the source open for reading and the destination open for writing, fails with `EXDEV` when not on the
  same mounted filesystem,
  `EINVAL` or `EOPNOTSUPP` when unsupported, available as a generic ioctl since Linux 4.5
  (https://man7.org/linux/man-pages/man2/ioctl_ficlone.2.html, fetched page).
  btrfs doc: `cp --reflink=always`, cross-mount reflink within one filesystem works since 5.18, NOCOW status must match
  (`SRC/btrfs/Reflink.rst:4-33`, verified).
  No capability is required beyond those file permissions (inference from the man page's error list).
- **Convert an existing directory (such as the repository checkout) into a subvolume.**
  Required before snapshotting it, because snapshots accept only a subvolume root (`linux-ioctl.c:1209-1217`,
  `btrfs-subvolume.rst:260`, verified).
  There is no in-place conversion documented in the pages read (absence; unverified beyond these pages).
  Renaming a non-subvolume inode across subvolumes returns `-EXDEV` (`SRC/btrfs/linux-inode.c:8588-8590`, verified),
  so moving files into a new subvolume is a copy; `cp --reflink=always` makes that copy share extents.
  Measured: this checkout is a plain directory (inode 2595895) inside the `/home` subvolume (subvolid 257).

### ZFS (note only)

ZFS is out of scope for 0.x filesystem-enhanced caching per the user.
Recorded for later: `zfs diff` reports removed, created, modified, renamed paths (`SRC/zfs/zfs-diff.8:37-54`, verified);
on Linux, `snapshot` delegation "Must also have the mount ability" while `mount` cannot be delegated
(`SRC/zfs/zfs-allow.8:100-110`, `:210`, verified).

### Scheduling, priorities, pause, resume, end

#### Concurrency default

- `os.availableParallelism()` wraps libuv `uv_available_parallelism()` (https://nodejs.org/api/os.html, fetched page),
  which on Linux "inspects the calling thread's CPU affinity mask" (https://docs.libuv.org/en/v1.x/misc.html, fetched
  page).
  Neither page mentions cgroup `cpu.max` quotas (absence on fetched pages), so a quota-limited daemon cgroup may be
  overestimated (inference).

#### Pause via signals (pueue's approach)

- `SIGKILL` and `SIGSTOP` cannot be caught, blocked, or ignored; `SIGCONT` continues a stopped process;
  some blocking calls may fail with `EINTR` after stop and continue on Linux
  (https://man7.org/linux/man-pages/man7/signal.7.html, fetched page).
- Process groups (https://man7.org/linux/man-pages/man2/setpgid.2.html, fetched page):
  fork children inherit the process group;
  background process groups reading the terminal get `SIGTTIN` and stop;
  when a group becomes orphaned with a stopped member, each member gets `SIGHUP` then `SIGCONT`.
- Node `detached: true` makes the child "the leader of a new process group and session";
  "child processes of child processes will not be terminated when attempting to kill their parent"
  (https://nodejs.org/api/child_process.html, fetched page).
  Node docs show no pidfd or cgroup placement option (absence on fetched page).
- Pitfalls for process-group pause (inference from the cited semantics, unverified by experiment):
  descendants that call `setsid` or `setpgid` escape `killpg`;
  a detached long-lived helper (for example the Gradle daemon, which persists and is reused across builds per the
  Gradle daemon page) is outside the task's group;
  stop and continue is observable to the task and its parent (job-control state);
  timers and deadlines keep running in wall-clock terms while stopped.
- pidfd: `pidfd_open` (Linux 5.3) avoids PID reuse races; readable on exit, `EPOLLHUP` when reaped;
  `pidfd_send_signal` signals through it (https://man7.org/linux/man-pages/man2/pidfd_open.2.html, fetched page).

#### Pause and end via cgroup v2 (`SRC/linux/cgroup-v2.rst`, verified)

- "cgroup is a mechanism to organize processes hierarchically and distribute system resources along the hierarchy"
- (`:116-118`).
- "On creation, all processes are put in the cgroup that the parent process belongs to";
  "Migration of a process doesn't affect already existing descendant processes" (`:124-129`).
  A task must be placed in its cgroup before it spawns children.
- `cgroup.freeze`: writing `1` freezes the cgroup and all descendants until unfrozen; completion shows `frozen 1` in
- `cgroup.events`;
  frozen processes "can be killed by a fatal signal"; processes moved in stop, moved out run (`:1025-1049`).
- `cgroup.kill`: writing `1` SIGKILLs every process in the subtree, handles concurrent forks, protected against
- migrations;
  fails with `EOPNOTSUPP` in threaded cgroups (`:1051-1064`).
- Limits: `cpu.max` as `$MAX $PERIOD` (`:1214-1225`); `memory.max` hard limit invoking the OOM killer inside the cgroup
- (`:1403-1411`);
  `memory.oom.group` kills the cgroup as a unit (`:1477-1493`); `pids.max` hard process limit, fork-bomb protection
  (`:2435-2458`).
- No internal process constraint: non-root cgroups with domain controllers enabled in `cgroup.subtree_control` cannot
- hold processes (`:507-534`).
- Delegation to a less privileged user = write access to the directory plus `cgroup.procs`, `cgroup.threads`,
- `cgroup.subtree_control` (`:543-557`);
  containment: migrating a process needs write access to `cgroup.procs` of the common ancestor (`:575-590`).
- What cgroups do not provide:
  the controller list is CPU, memory, IO, PID, cpuset, device, RDMA, DMEM, HugeTLB, misc, perf_event (`:60-80`,
  `:1089-3000`);
  none restricts filesystem reads.
  The device controller only gates device-file `mknod`, read, and write via cgroup BPF (`:2788-2808`).
  (The doc has no sentence saying "no filesystem access control"; the conclusion is from the definition and controller
  list.)

#### systemd delegation for an unprivileged daemon

- `Delegate=` "Turns on delegation of further resource control partitioning to processes of the unit";
  for `User=` services the cgroup is made accessible to that user;
  takes a boolean or controller list; defaults to false; delegated controllers are enabled for parent and sibling units
  too
  (`SRC/systemd/systemd.resource-control.xml:1421-1464`, verified).
- `DelegateSubgroup=` places main processes in a subgroup, avoiding manual moves; it names services' `ExecStart=`
- (`:1468-1489`, verified);
  whether it applies to transient scopes is unverified.
- `systemd-run --scope`: the command is executed by `systemd-run` itself as parent, "inherit[s] the execution
- environment of the caller",
  and runs synchronously (`SRC/systemd/systemd-run.xml:76-81`, verified).
  This fits a daemon that owns its terminal and Ctrl+C.
- systemd delegation doc (`SRC/systemd/CGROUP_DELEGATION.md`, verified):
  systemd won't touch the delegated subtree (`:178-200`);
  controllers are made available but not enabled; write `+memory` etc. to `cgroup.subtree_control` (`:257-261`);
  "move your main daemon process out of that cgroup (and into a sub-cgroup) before you can start further processes"
  (`:298-303`);
  never create cgroups below arbitrary systemd-managed cgroups and never write attributes of cgroups systemd created
  (`:386-397`).
- Fedora default on this host: the user manager itself receives `cpu io memory pids` (measured above).
  An `app.slice/run-*.scope` created with `systemd-run --user --scope -p Delegate=yes` should inherit availability of
  those controllers (inference, unverified by experiment).

#### Restricting file reads if needed

- Landlock (`SRC/linux/landlock.rst`, verified):
  "Landlock empowers any process, including unprivileged ones, to securely restrict themselves" (`:13-19`);
  rules restrict "the thread enforcing it, and its future children" (`:31-34`);
  unprivileged enforcement requires `no_new_privs` (`:276-281`);
  restrictions cannot be removed once applied (`:318-324`);
  `clone(2)` children inherit the domain (`:392-400`);
  limits include 16 stacked layers and unrestrictable pipe or socket fds via `/proc/<pid>/fd` (`:640-676`).
  Landlock denies; it does not report reads for input discovery (inference; audit logging flags exist per ABI notes at
  `:257`, reading audit output needs privileges, unverified).
- User and mount namespaces: since Linux 3.8 unprivileged processes can create user namespaces,
  gaining capabilities inside to create other namespace types; block-device mounts still need init-namespace
  `CAP_SYS_ADMIN`
  (https://man7.org/linux/man-pages/man7/user_namespaces.7.html, fetched page).
- bubblewrap: uses user namespaces "allowing any user to use the tool"; setuid mode "has been removed"
- (`SRC/linux/bubblewrap-README.md:12-23`, verified);
  uses `PR_SET_NO_NEW_PRIVS` (`:32-40`, verified).

#### Control socket safety

- Abstract Unix sockets: "socket permissions have no meaning for abstract sockets"; `SO_PEERCRED` returns peer
- credentials on connected stream sockets;
  some systems ignore socket file permissions (https://man7.org/linux/man-pages/man7/unix.7.html, fetched page).
- Node IPC paths: Linux `sun_path` limit typically 107 bytes; Linux abstract sockets via leading `\0`
  (https://nodejs.org/api/net.html, fetched page).
- `XDG_RUNTIME_DIR` "MUST be owned by the user", mode 0700, bound to login, for "communication and synchronization
- purposes"
  (https://specifications.freedesktop.org/basedir/latest/, fetched page); measured mode 700 here.

#### macOS and Windows notes only

- macOS: no cgroups; pueue's Unix path uses process groups there (`pueue/src/process_helper/unix.rs:88-99`, verified).
- Windows: job objects manage processes "as a unit"; children join the job by default unless breakaway limits are set;
  `TerminateJobObject` and `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` end whole trees
  (https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects, fetched page);
  that page documents no suspend or freeze operation (absence).

### Mount option scope measured on this host

- Doc: "Most mount options apply to the whole filesystem and only options in the first mounted subvolume will take
- effect" (`SRC/btrfs/ch-mount-options.rst:9-14`, verified).
- `/etc/fstab` lists `/home` and `/var` on UUID `1bb3d23e-...` with `compress=zstd:1`, and notes "Updated by
- bootc-fstab-edit.service" (measured, readable without root).
- `/proc/self/mountinfo` super options for every mount of superblock `0:36` (`/sysroot`, `/etc`, `/var`, `/var/home`)
- are
  `rw,seclabel,ssd,discard=async,space_cache=v2,...` with **no** `compress` (measured).
  The first mount is `/sysroot` from `rootflags=subvol=root` on the kernel command line (measured `/proc/cmdline`).
- Conclusion (inference from the doc note plus measurements): adding `user_subvol_rm_allowed` only to the `/home` fstab
- line would likely not take effect at boot;
  the option would need to be in the first mount's flags (`rootflags=`), or be applied at runtime by remount.
- Kernel remount path: `btrfs_reconfigure` copies the new context's `mount_opt` into the filesystem unless it is a
- bind-mount reconfigure
  (`SRC/btrfs/linux-super.c:1492-1523`, `:1400-1406`, verified by reading); `user_subvol_rm_allowed` is an ordinary
  flag parameter (`:247`, `:546`).
  A runtime `mount -o remount,user_subvol_rm_allowed` was not tested (unverified).
- Changing kernel arguments on rpm-ostree systems: `rpm-ostree kargs --append=...` or `--replace=KEY=VALUE=NEWVALUE`,
- applied to a new deployment and requiring reboot
  (Fedora CoreOS "Modifying Kernel Arguments", https://docs.fedoraproject.org/en-US/fedora-coreos/kernel-args/, seen
  only as a search-result summary, unverified).
  Whether bootc-managed hosts honor that path is unverified.

### doctor: environment prerequisites

Each item: what it unblocks, how to detect it without root, exact remediation, reason, evidence.
Commands that need root are marked `sudo`.

#### Prior art for doctor output structure

- `flutter doctor`: one line per validator with a leading box `[✓]` success, `[✗]` missing, `[!]` partial or not
- available, `[☠]` crash,
  colored green, red, yellow; message lines prefixed `•` information, `!` hint, `✗` error
  (`SRC/flutter-doctor_validator.dart:139-174`, verified).
- `brew doctor`: exits non-zero when problems are found; prints each finding as a warning after a one-time banner
  "Please note that these warnings are just used to help the Homebrew maintainers...";
  prints "Your system is ready to brew." when clean; `--list-checks` lists checks runnable individually
  (`SRC/brew-doctor.rb:17-24`, `:59-101`, verified; https://docs.brew.sh/Manpage, fetched page).
- `mise doctor`: inline sections (`version`, `activated`, `shims_on_path`, ...), then numbered warnings and numbered
- problems,
  "No problems found" when clean, exit 1 on problems; `--json` emits the same data with `errors` and `warnings` arrays
  (`SRC/mise-doctor-mod.rs:44-54`, `:300-320`, `:323-327`, `:380-400`, verified; https://mise.jdx.dev/cli/doctor.html,
  fetched page).

#### Filesystem acceleration (optional capabilities; hashing always works without them)

- **Repository is on btrfs.**
  Detect: `statfs` type, as `stat --file-system --format=%T <repo>` printed `btrfs` (measured).
  Remediation: none; filesystem-enhanced caching stays off.
  Reason: every feature below is btrfs-specific.
- **Repository root is a subvolume** (unblocks snapshots of the checkout).
  Detect: inode number of the repository root is 256 (`ch-subvolume-intro.rst:1-5`, `:172-173`, verified); here it is
  2595895 (measured).
  Remediation (daemon stopped, no open editors; paths shown for this host; unprivileged):
  `btrfs subvolume create /var/home/user/Monochromatic.subvolume`,
  `cp --archive --reflink=always /var/home/user/Monochromatic/. /var/home/user/Monochromatic.subvolume/`,
  `mv /var/home/user/Monochromatic /var/home/user/Monochromatic.before-subvolume`,
  `mv /var/home/user/Monochromatic.subvolume /var/home/user/Monochromatic`,
  then verify inode 256 and remove the old directory when satisfied.
  Reason: snapshots accept only subvolume roots; files cannot be renamed across subvolumes (`EXDEV`), so a reflink copy
  is the cheap move.
  Unverified: `cp --archive` behavior for hard links and xattrs across this copy, and effects on Git worktrees that
  record absolute paths.
- **Subvolume deletion by owner** (unblocks cheap cleanup of snapshots).
  Detect: `user_subvol_rm_allowed` in the super options field of `/proc/self/mountinfo` for the repository mount
  (measured absent).
  Remediation, runtime: `sudo mount -o remount,user_subvol_rm_allowed /var/home` (untested; kernel reading above).
  Remediation, persistent: add `user_subvol_rm_allowed` to the first mount of that filesystem;
  on this host that is `rootflags=subvol=root` on the kernel command line, for example
  `sudo rpm-ostree kargs --replace=rootflags=subvol=root,user_subvol_rm_allowed` then reboot (unverified for bootc);
  on conventional hosts add it to every `/etc/fstab` btrfs entry sharing the UUID, then `sudo systemctl daemon-reload`
  (fstab header comment, measured) and reboot.
  Then re-read `/proc/self/mountinfo` to confirm.
  Reason: owner deletion of non-empty subvolumes is otherwise root-only (`ch-mount-options.rst:483-497`,
  `linux-ioctl.c:2383-2399`).
  Fallback without it: unlink contents then `rmdir` the empty subvolume (kernel 4.18+, `rmdir_subvol` feature present).
- **`rmdir_subvol` feature** (fallback deletion).
  Detect: `/sys/fs/btrfs/features/rmdir_subvol` exists (measured present; `btrfs-man5.rst:161-165`).
  Remediation: kernel 4.18 or newer. Reason: see previous item.
- **Reflink between cache and repository** (unblocks cheap output restoration).
  Detect: create a probe file in the cache directory and `FICLONE` it into the repository; success, `EXDEV`, `EINVAL`,
  or `EOPNOTSUPP`
  distinguishes the cases (ioctl_ficlone(2), fetched page); also check `uname --kernel-release` is at least 5.18 when
  the cache is on a different mount point (`Reflink.rst:21-26`).
  Remediation: place the cache on the same btrfs filesystem (a sibling subvolume outside the repository subvolume, so
  snapshots of the checkout exclude it);
  keep NOCOW status consistent, for example do not `chattr +C` the cache directory (`Reflink.rst:27-33`).
  Reason: reflink shares extents instead of copying bytes.
- **Nested subvolumes inside the checkout** (correctness of snapshots).
  Detect: walk for directories with inode 256 below the repository root.
  Remediation: none required; report that their contents are absent from snapshots (`ch-subvolume-intro.rst:87-91`).
  Reason: snapshot barrier.

#### Sandbox (required: at least cgroups)

- **cgroup v2 unified hierarchy.**
  Detect: `stat --file-system --format=%T /sys/fs/cgroup` prints `cgroup2fs` (measured); `/proc/self/cgroup` has a
  single `0::/path` line (cgroups(7), fetched page; measured).
  Remediation: boot a systemd configuration that mounts cgroup v2 (the specific kernel argument was not researched;
  unverified).
  Reason: freezer, kill, and delegation files used below are v2 interfaces (`SRC/linux/cgroup-v2.rst:1025-1064`,
  `:537-590`).
- **Daemon runs in a delegated cgroup.**
  Detect: resolve own cgroup from `/proc/self/cgroup`; check the directory is owned by the current uid and
  `cgroup.procs`, `cgroup.subtree_control` are writable (`access(W_OK)`)
  (`cgroup-v2.rst:543-557`, `:575-590`).
  Remediation: start the daemon as `systemd-run --user --scope -p Delegate=yes -- <tool> daemon`
  (`SRC/systemd/systemd-run.xml:76-81`; `SRC/systemd/systemd.resource-control.xml:1421-1464`).
  Reason: systemd forbids creating cgroups below units it did not delegate (`SRC/systemd/CGROUP_DELEGATION.md:386-397`).
- **Daemon process moved to a leaf before creating task cgroups.**
  Detect: own cgroup has no child cgroups with processes of the daemon, and `cgroup.subtree_control` write succeeds.
  Remediation: automatic (the daemon moves itself into `daemon/`); doctor reports if it cannot.
  Reason: no-internal-process constraint (`cgroup-v2.rst:507-534`; `CGROUP_DELEGATION.md:298-303`).
- **Required controllers delegated** (`cpu`, `memory`, `pids`; `io` optional).
  Detect: read `cgroup.controllers` of the daemon's cgroup (measured `cpu io memory pids dmem` for `user@1000.service`
  and `app.slice`).
  Remediation: `sudo systemctl edit user@.service` or create
  `/etc/systemd/system/user@.service.d/60-<tool>-delegate.conf` containing
  `[Service]` and `Delegate=cpu io memory pids`, then `sudo systemctl daemon-reload` and log out and back in
  (drop-in directories: `SRC/systemd/systemd.unit.xml:203-249`; reload: `:598`; `Delegate=` list form:
  `systemd.resource-control.xml:1436-1444`).
  This host already has `Delegate=pids memory cpu` upstream plus Fedora's `Delegate=cpu io memory` drop-in (measured
  `systemctl cat user@.service`).
  Reason: limits require controllers to be available and then enabled with `+cpu +memory +pids` in
  `cgroup.subtree_control` (`CGROUP_DELEGATION.md:257-261`).
- **Freeze and kill interfaces.**
  Detect: `cgroup.freeze` and `cgroup.kill` exist in the delegated cgroup (measured present under `app.slice`).
  Remediation: newer kernel (versions not verified from the kernel doc; recall says 5.2 and 5.14, unverified).
  Reason: pause and end must cover whole process trees (`cgroup-v2.rst:1025-1064`).
- **User manager available outside a graphical login (SSH, CI).**
  Detect: `systemctl --user show-environment` succeeds (unverified as the best probe).
  Remediation: `sudo loginctl enable-linger <user>` (`SRC/systemd/loginctl.xml:186-194`).
  Reason: `systemd-run --user` needs a running user manager.

#### Optional read restriction and watcher limits

- **Landlock** (only if file-read restriction is enabled).
  Detect: `landlock_create_ruleset(NULL, 0, LANDLOCK_CREATE_RULESET_VERSION)` returns an ABI version
  (`SRC/linux/landlock.rst:514-524`);
  `/sys/kernel/security/lsm` lists `landlock` (measured, readable without root).
  Remediation: kernel built with `CONFIG_SECURITY_LANDLOCK=y` and booted with `lsm=landlock,<existing list>`
  (`landlock.rst:845-859`).
  Reason: unprivileged self-restriction inherited by children (`landlock.rst:13-19`, `:392-400`).
- **Unprivileged user namespaces** (only if a bubblewrap or mount-namespace sandbox is enabled).
  Detect: `/proc/sys/user/max_user_namespaces` is greater than 0 (measured 254589; namespaces(7), fetched page).
  Remediation: `/etc/sysctl.d/60-<tool>-userns.conf` with `user.max_user_namespaces = <N>`, then `sudo sysctl --system`
  (`SRC/systemd/sysctl.d.xml:25`; the `sysctl --system` flag is procps recall, unverified).
  Reason: bubblewrap requires unprivileged user namespaces since setuid mode was removed
  (`SRC/linux/bubblewrap-README.md:12-23`).
- **inotify watch budget.**
  Detect: compare `/proc/sys/fs/inotify/max_user_watches` (measured 524288) and `max_user_instances` (measured 8192)
  with the count of non-ignored directories (measured 5,480) plus other watchers' usage (unmeasured).
  Remediation: `/etc/sysctl.d/60-<tool>-inotify.conf` with `fs.inotify.max_user_watches = <N>`, then
  `sudo sysctl --system`.
  Reason: inotify is non-recursive and limited per user (inotify(7), fetched page).
- **Control socket directory.**
  Detect: `$XDG_RUNTIME_DIR` set, owned by the user, mode 0700 (measured `/run/user/1000`, 700).
  Remediation: run inside a systemd-logind session or enable linger.
  Reason: the spec requires 0700 and user ownership (basedir spec, fetched page); the socket grants task execution.
- **Parallelism versus CPU quota** (informational).
  Detect: read `cpu.max` of the daemon cgroup (`cgroup-v2.rst:1214-1225`) and compare with the affinity-based default.
  Remediation: set the concurrency env var. Reason: libuv's default reads affinity, not quota.

## Design implications

- **Per-task cgroup is the process-tree primitive.**
  Spawn every task into its own leaf cgroup under a delegated subtree; pause = `cgroup.freeze`, end = `cgroup.kill`,
  limits = `cpu.max`, `memory.max` plus `memory.oom.group`, `pids.max`.
  This covers descendants that escape process groups, which rules out pueue-style `killpg` `SIGSTOP` as the pause
  mechanism.
- **Start the daemon with `systemd-run --user --scope -p Delegate=yes`** so it keeps its terminal and Ctrl+C,
  then move itself into a leaf (for example `daemon/`) before enabling controllers and creating `tasks/<id>/` siblings.
  Fail closed with a clear diagnostic when the cgroup is not delegated, since sandboxing is a must.
- **Place tasks in their cgroup before exec**, because migration does not move existing descendants.
  This needs either `clone3` with `CLONE_INTO_CGROUP` or a tiny spawn helper that writes its pid to `cgroup.procs`,
  applies Landlock if configured, sets `no_new_privs`, then execs.
  Node's documented `child_process` API offers neither, so a TypeScript on Node daemon needs a helper binary (language
  approval question).
- **Define pause semantics explicitly**: paused queued tasks are held; paused running tasks are frozen;
  wall-clock timeouts and test timers keep running (freeze does not stop time), so timeouts should count only unfrozen
  time or be suspended.
- **Priority is a queue ordering, not preemption.**
  Adopt pueue's order (priority desc, then id asc) but allow in-place priority changes for queued tasks, which pueue
  lacks.
  Decide how priority propagates to a task's queued dependencies (priority inversion is otherwise possible; inference).
- **Concurrency default** `availableParallelism()` or `sched_getaffinity` count, overridable by env var, and capped by
- the daemon cgroup's `cpu.max` when set.
- **Transport**: filesystem Unix socket inside `$XDG_RUNTIME_DIR` (mode 0700), never an abstract socket; optionally
- verify `SO_PEERCRED` uid.
  No named pipes or TCP in 0.x.
- **Framing**: newline-delimited JSON-RPC 2.0 (Watchman JSON PDUs and `mcp-stdio` precedent) rather than LSP
- `Content-Length`;
  simpler for `socat`-style debugging. Server-to-client notifications must be added to the message model.
- **Event model**: BSP-style lifecycle notifications (`task/started`, `task/progress`, `task/finished`) with `taskId`,
- `parents`, `originId`,
  status including `cancelled`, plus a queryable snapshot per task modeled on Tilt `UIResource`
  (current run, recent history, queued since, waiting reason, paused, priority).
- **Subscriptions with resumable cursors**: each event carries a monotonically increasing sequence (Watchman clock
- analogue);
  clients subscribe `since` a sequence; if history was trimmed, reply with a fresh-instance flag and a full snapshot.
- **Control is explicit methods, not request cancellation**: `task/run`, `task/rerun`, `task/cancel`, `task/pause`,
- `task/resume`, `task/setPriority`.
  Keep LSP `$/cancelRequest` only for cancelling a slow RPC call, answering it with `-32800`.
- **Coalescing policy per task**: never overlap runs of the same task; after a run, rerun once with changes accumulated
- during it
  (Watchman trigger semantics, matching file-enforcer's serial rerun queue).
  Persistent tasks declare a restart policy (Turborepo `interruptible` analogue, watch-restart signal escalation).
- **Burst handling**: provide a hold state (Watchman `state-enter` analogue) usable around `pnpm install`,
- `git checkout`, and rebases.
- **Change detection**: inotify on non-ignored directories (5,480 today versus 99,416 with `node_modules`, `target`,
- `dist`),
  treating `IN_Q_OVERFLOW` as "rescan and rehash".
  Content hashes remain the source of truth for cache decisions; watchers only schedule work.
- **btrfs is an optional accelerator, not a change source**:
  `find-new` needs root and lists only data-extent writes, so it cannot replace watching or hashing for an unprivileged
  daemon.
  Unprivileged `GET_SUBVOL_INFO` generation can gate "nothing changed since generation N" once the checkout is its own
  subvolume.
- **Snapshots as consistent build inputs**: after the one-time conversion, take a read-only snapshot per build wave and
- hash or build from it,
  so user edits during a run cannot tear inputs; exclude caches and outputs by keeping them in sibling subvolumes
  (snapshots are not recursive).
  Snapshot cleanup needs `user_subvol_rm_allowed` (currently absent) or unlink-then-`rmdir`; `doctor` must explain both.
- **Reflink for cache restore** is the most useful unprivileged btrfs feature: materialize cached outputs by reflink
- from a same-filesystem cache,
  falling back to copy on `EXDEV`, `EINVAL`, `EOPNOTSUPP`, or NOCOW mismatches.
- **Cache identity must not use raw inode numbers**, since snapshots duplicate them; use content hashes, or
- `subvolid:inode` where identity matters.
- **ZFS is out of scope for 0.x.**
- **`doctor` is a first-class command** with flutter-style per-capability status, numbered problems and warnings like
- `mise doctor`,
  non-zero exit on required failures like `brew doctor`, and `--json`; every finding names detection evidence, the
  exact command or file, and the reason.
  Required: cgroup v2, delegated cgroup, `cpu`/`memory`/`pids` controllers, `cgroup.freeze`, `cgroup.kill`, private
  runtime directory.
  Optional: btrfs, subvolume checkout, `user_subvol_rm_allowed`, reflink reachability, Landlock, user namespaces,
  inotify budget.
- **Cache key contents** (union of Turborepo, Nx, REAPI evidence):
  resolved task definition; argv; declared input file contents (content hashes, not mtime);
  declared env names and values in strict mode (sorted); tool identity and version via explicit runtime inputs
  (Bazel shows untracked tools produce wrong hits); lockfile slice per package; hashes of dependency outputs;
  platform (OS, arch); a salt or cache-format version; timeout if timeouts can change outcomes.
- **Undeclared-read enforcement is lint-level** as the user accepted; the root `file-enforcer.config.ts` already
- contains direct reads that such a rule must flag or allow-list.
  Landlock can later turn "declared inputs" into "readable paths" for tasks that opt in.
- **Default test set** excludes `*.expensive.*.test.*`; today's `test:unit` template's `.expensive.` substring rule is
- compatible with all 4 current heavy files.
- **Task graph must be built, not translated**: 1,324 of 1,777 package task headers extend templates, and no
- cross-package dependencies,
  sources, or outputs are declared today; the graph needs derivation from `package.json` workspace dependencies, Cargo,
  and Gradle, plus explicit inputs.
- **file-enforcer inside the daemon (TypeScript on Node only)**: run each config evaluation as a child task in its own
- cgroup rather than a cache-busting `import()`;
  keep FE09's staleness manifest for builder skipping; emit typed events from the rerun queue; replace module
  singletons with per-run context when embedding in-process.
- **Activity must not be derived from logs**: `LogRecord` has no structured fields; define a separate typed event
- stream and keep tagged logs for humans.
- **Protocol documentation is a deliverable**: publish a machine-readable schema (LSP `metaModel.json` precedent) and
- test examples against the implementation,
  avoiding the BSP `parent`/`parents` contradiction, BSP's undocumented cancel, and pueue's header-size comment drift.

## Risks

- **Scope**: task graph, content-hash cache, watcher, scheduler with priorities and pause, cgroup sandbox, spawn
- helper, RPC, and documentation are all built; nothing is borrowed from a build tool.
- **Systemd dependency**: the sandbox relies on a delegated user cgroup; CI runners without a systemd user session
- (unverified) or users who forget `systemd-run` get no sandbox, and failing closed blocks work.
- **Spawn race and language constraint**: tasks must enter their cgroup before exec; a Node daemon needs a native or
- non-Node helper, which reopens the approved-language question.
- **Freeze side effects**: frozen tasks keep file locks alive; file-enforcer's manifest lock sees a live pid and other
- writers time out after 5 s; test timers and network timeouts fire after resume.
- **Shared daemons across tasks**: Gradle (and likely Kotlin, unverified) daemons outlive or precede tasks;
- `cgroup.kill` on one task can kill a daemon another task reuses, while running with `--no-daemon` costs startup time.
- **Whole-repo rebuild and test storms**: every change triggers affected builds plus tests; `pnpm install` or branch
- switches can queue most of the repository; without a hold state and coalescing the machine saturates (see `RXI`).
- **Watcher limits**: inotify is non-recursive with overflow and new-directory races; `node_modules` inclusion
- multiplies directories about eighteenfold (99,416 versus 5,480).
- **Filesystem accelerators mostly need root**: `find-new`, `send`, and fanotify filesystem marks need `CAP_SYS_ADMIN`;
- the unprivileged options are coarse (`GET_SUBVOL_INFO`) or need a one-time checkout conversion.
- **Checkout conversion is disruptive**: converting the repository into a subvolume replaces every inode, may confuse
- editors, Git worktrees with absolute paths, and running watchers, and temporarily doubles metadata.
- **Snapshot accumulation**: without `user_subvol_rm_allowed`, each snapshot must be emptied file by file before
- `rmdir`, which is write-heavy; with the option, persistence on this bootc host appears to require kernel `rootflags`,
- not `/etc/fstab` (measured fstab `compress` not in effect).
- **Host-specific remediation**: `doctor` advice differs between bootc or rpm-ostree hosts and conventional fstab
- hosts; the rpm-ostree `kargs` path is only search-summary evidence so far.
- **Stale cache hits from undeclared inputs** remain possible under lint-only enforcement, including tool versions and
- ambient env not declared as inputs.
- **Control socket is code execution**: anyone who can connect can run tasks as the user; permissions and peer checks
- are security-critical.
- **HC5 conflict**: the frozen vet constraint lists macOS and Windows CI runners, while 0.x is Linux-only; the gap must
- be reconciled in the requirements doc.
- **Documentation gate applies to the repository itself**: the same HC7 standard that culled every market tool now
- applies to repository-written protocol and CLI docs, and drift between docs and code is common in the prior art
- examined.
- **Migration coupling**: 41 source files and 25 `MISE_MONOREPO` references depend on mise names, env vars, or markers;
- replacing them is part of cutover.
