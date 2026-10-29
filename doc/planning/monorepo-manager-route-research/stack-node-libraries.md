# Node library options for the from-scratch monorepo manager

Outcome on 2026-09-16:
the Node design here is out.
The user killed the TypeScript route because Node single executables are too big;
see "Current stack direction" in `doc/planning/monorepo-manager-from-scratch-design.md`.
This file is kept as evidence.

Research date: 2026-09-16.
Input design: `doc/planning/monorepo-manager-from-scratch-design.md`.
Prior research this corrects: `doc/planning/monorepo-manager-route-research/stack-typescript.md`.
Nothing under `/var/home/user/Monochromatic` was modified
(`git status --short` still shows only the pre-existing `M mise.lock`).

## Conventions

- **Verified (measured)**:
  a probe ran in this session on this host; the command, script, and output are named.
- **Verified (source)**:
  a cited source file and line in an installed package or a pinned checkout.
- **Verified (doc)**:
  a cited documentation file or page.
- **Unverified**:
  inference, or a claim the cited evidence does not settle.
- `SCRATCH` abbreviates `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad`.
  Every probe script lives in `SCRATCH/node-libs/`;
  libraries were installed there with `pnpm add` (pnpm `12.3.4`),
  and raw outputs are in `SCRATCH/node-libs/*.jsonl`.
- Host:
  Node `v26.8.2`,
  systemd `259 (259.8-1.fc44)` (`systemctl --version`),
  `DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus`,
  Watchman not installed (`command -v watchman` exit 1),
  `cgexec` not installed (`command -v cgexec` exit 1).
- Package metadata comes from `https://registry.npmjs.org/<name>` and `gh api repos/<owner>/<repo>`
  (`SCRATCH/node-libs/lib-meta.ts`, output `lib-meta.jsonl`).
  Transitive counts come from `pnpm list --json --depth Infinity --prod` (`dep-count.ts`);
  pnpm prints a shared subtree once,
  so a count is a lower bound where two libraries share dependencies.
- Probe constraints honored:
  - No cgroup, unit, or scope was created.
    The only `StartTransientUnit` call used the job mode `probe-invalid-mode`,
    which systemd rejects at `src/core/dbus-manager.c:1142-1144` before it loads or creates a unit.
  - `GetUnitByPIDFD` was called with an fd of `/dev/null` to test fd passing;
    it is a read-only query.
  - Watch probes on the checkout only registered inotify watches (at most 41,209 for about 2 seconds, one process at a
  - time).
  - Event probes wrote only to fixtures under `SCRATCH/node-libs/ev/`, which is `tmpfs`;
    inotify events come from the VFS layer, so the filesystem difference is assumed not to matter (unverified).

## Repository incumbents

### `chokidar`

- Catalog: `'chokidar': '>=5.0.0'` (`pnpm-workspace.yaml:71`);
  the lockfile resolves `chokidar@5.0.0` and `readdirp@5.1.1` (`node_modules/.pnpm`, verified, measured).
- `package/file-enforcer/core` (`package.json:33`):
  one chokidar watcher per directory derived by `watchDirs` (`src/watch/watch-filter.ts:28-60`),
  with `atomic: true`, `awaitWriteFinish: false`, `depth: 0`, `followSymlinks: false`, `ignoreInitial: true`
  (`src/watch/watch-dir.ts:231-241`),
  restarted at most 3 times 50 ms apart (`src/watch/watch-supervisor.ts:11`, `:16`, `:236-292`)
  (verified, source).
  Adopted in commit `90c54c3d6` (2026-06-06), "migrate watch loop to chokidar".
- `watch-directory-chokidar-regression.unit.test.ts` pins two live behaviors through the built `dist`:
  a write to a tracked read emits `{ kind: 'source' }` (`:404-449`),
  and a write to a managed destination emits `{ kind: 'protected' }` (`:451-496`),
  each after waiting for chokidar's `ready` (commit `a4728812b` "wait for chokidar readiness").
  It pins nothing about scale, overflow, bursts, or new directories (verified, source).
- `package/watch-restart/cli` (`package.json:35`):
  recursive chokidar with `awaitWriteFinish` defaulting to `{ stabilityThreshold: 50, pollInterval: 10 }`
  (`src/watcher-types.ts:36-56`).
  Its atomic-save test is skipped with
  "Empirical flake rate ~20% on Linux EVEN IN ISOLATION ... chokidar occasionally reports zero events on the
  destination path"
  (`src/watcher.unit.test.ts:213-223`, verified, source).
- The 2026-05 dev-server evaluation that picked chokidar compared single-directory use,
  not whole-repository watching
  (`package-paused/desktop-daemon/editord/HANDOVER.custom-dev-server-watcher.md:62-141`, verified, source).

### `@homebridge/dbus-native`

- Catalog `'>=0.7.9'` (`pnpm-workspace.yaml:27`), used by `package/kwin/key-helper` as a D-Bus service.
- It replaced `dbus-next` because `dbus-next@0.10.2` was unmaintained and its optional native `usocket` pulled a
  `node-gyp` and `request` advisory chain (`doc/troubleshooting/dependencies.md:1765-1849`, verified, source).
- key-helper declares the server surface locally because the shipped `index.d.ts` types only the client path
  (`package/kwin/key-helper/src/dbus-native.d.ts:1-10`, verified, source).

### `@monochromatic-dev/mcp-stdio`

- `private: true`, `LGPL-3.0-or-later` (`package/mcp/stdio/package.json`).
- Reusable exports: `readLines`, JSON-RPC 2.0 types and error codes, `isJsonRpcMessage`, `isPlainObject`
  (`src/index.ts`).
- MCP-specific parts:
  `serve` binds stdin and stdout and logs every outbound frame with `console.error` (`src/transport.ts:156`);
  `createSerialRequestQueue` runs one handler at a time per connection and lets a cancelled running entry finish
  (`src/transport-queue.ts:95-110`);
  the README lists "subscriptions" and "progress notifications" as deliberately omitted (`README.md:162-163`).
- `isJsonRpcMessage` accepts only object `params` (`src/json-rpc.ts:177-186`),
  while JSON-RPC 2.0 also allows by-position array params (inference from the specification, section 4.2).
- `processStdoutWriter` waits for `'drain'` when `write()` returns `false` (`src/transport.ts:90-98`);
  the pattern applies to any `Writable`, including a socket (verified, source).
- `readLines` cost is linear in lines per chunk:
  10,000 lines (2.4 MiB) 14 ms, 40,000 (9.7 MiB) 20 ms, 160,000 (38.6 MiB) 80 ms
  (`node rpc-probe.ts readlines`, verified, measured).

### Process libraries

- `nano-spawn` (catalog `'>=2.1.0'`, `pnpm-workspace.yaml:102`) is a dependency of 35 manifests, including the root;
  `execa` (catalog `'>=10.0.1'`, `:80`) only of the root `package.json:44`
  (`rg` over `package.json` files, verified, measured).
- Recorded nano-spawn behaviors:
  final newline stripped (`doc/troubleshooting/nano-spawn-final-newline.md`),
  and `timeout` is not a hard deadline (`doc/troubleshooting/nano-spawn-timeout-is-not-a-hard-deadline.md`).

## Library inventory

Each entry: latest version and publish date, license, direct and transitive dependencies,
repository push date, open issues and pull requests, stars.
Documentation problems are collected in "Documentation problems recorded".

### Watching

- `chokidar` 5.0.0, 2025-11-25, MIT, 1 direct (`readdirp`), 1 transitive; `paulmillr/chokidar` pushed 2026-08-16, 48
- open, 12,240 stars.
- `readdirp` 5.1.1, 2026-08-06, MIT, 0 dependencies; pushed 2026-08-06, 0 open.
- `@parcel/watcher` 2.6.0, 2026-07-20, MIT, 4 direct plus 12 optional platform binaries, 17 transitive;
  install script `node scripts/build-from-source.js`;
  `parcel-bundler/watcher` pushed 2026-08-04, 83 open, 838 stars.
- `watcher` 2.3.1, 2024-04-06, no `license` field in `package.json` (MIT `license` file), 3 direct, 5 transitive;
  `fabiospampinato/watcher` pushed 2026-07-17, 15 open, 140 stars.
- `watchpack` 2.5.2, 2026-06-11, MIT, 1 direct (`graceful-fs`); `webpack/watchpack` pushed 2026-09-13, 13 open, 400
- stars.
- `node-watch` 0.7.4, 2023-08-01, MIT, 0 dependencies; pushed 2026-01-14, 8 open, 341 stars.
- `nsfw` 2.3.1, 2026-04-02, MIT, 1 direct; install script `node-gyp rebuild` (no prebuilt binary);
  `axosoft/node-simple-file-watcher` pushed 2026-06-22, 67 open, 930 stars.
- `fb-watchman` 2.0.2, 2022-09-21, Apache-2.0, 1 direct (`bser`), 2 transitive;
  `facebook/watchman` pushed 2026-09-16, 258 open, 13,703 stars.
  The latest Watchman release `v2026.09.14.00` has 0 assets;
  the newest release with assets is `v2026.07.27.00`, with a `fc42` RPM and a Linux zip
  (`gh api repos/facebook/watchman/releases`, verified, measured).
  `mise registry` lists no `watchman` (verified, measured).

### D-Bus and cgroups

- `@homebridge/dbus-native` 0.7.9, 2026-08-15, MIT, 6 direct, 15 transitive; `homebridge/dbus-native` pushed
- 2026-08-15, 0 open, 16 stars;
  `engines.node` `>=8.0`.
- `dbus-native` 0.15.2, 2026-08-20, MIT, 1 direct (`xml2js`); `sidorares/dbus-native` pushed 2026-09-12, 15 open, 269
- stars;
  `engines.node` `>=22.12.0`.
- `dbus-next` 0.10.2, 2021-10-10, MIT, 8 direct including optional native `usocket`, 99 transitive;
  `dbusjs/node-dbus-next` pushed 2024-06-05, 51 open.
- `@jellybrick/dbus-next` 0.11.3, 2026-08-05, MIT, 2 direct, 8 transitive; `JellyBrick/node-dbus-next` pushed
- 2026-08-05, 1 open, 3 stars.
- `@balena/systemd` 0.5.11, 2026-02-19, Apache-2.0, 1 direct (`@mapbox/node-pre-gyp`), 54 transitive, Rust build
- fallback;
  `balena-io-modules/systemd` pushed 2026-09-07, 8 open, 1 star.
- cgroup packages:
  `https://registry.npmjs.org/-/v1/search?text=cgroup` returned only metric readers and container-id parsers,
  plus `simple-sandbox` 0.3.25 (2022-08-21, native namespaces and cgroups);
  no maintained package creates, freezes, or kills cgroups (verified, measured).

### JSON-RPC

- `vscode-jsonrpc` 9.0.2, 2026-08-28, MIT, 0 dependencies; `Microsoft/vscode-languageserver-node` pushed 2026-09-10, 88
- open (repository-wide), 1,788 stars.
- `json-rpc-2.0` 1.8.0, 2026-08-28, MIT, 0 dependencies; `shogowada/json-rpc-2.0` pushed 2026-08-28, 5 open, 246 stars.
- `jayson` 4.3.0, 2025-12-22, MIT, 12 direct including `@types/node`, `@types/ws`, `commander`, `ws`, `uuid`,
- `stream-json`;
  `tedeh/jayson` pushed 2025-12-22, 2 open, 726 stars.

### Processes

- `execa` 10.0.1, 2026-07-31, MIT, 12 direct, 17 transitive, `engines.node` `>=22`; `sindresorhus/execa` pushed
- 2026-07-31, 2 open, 7,603 stars.
- `tinyexec` 1.3.1, 2026-09-03, MIT, 0 dependencies; `tinylibs/tinyexec` pushed 2026-09-15, 11 open, 379 stars.
- `nano-spawn` 2.1.0, 2026-04-01, MIT, 0 dependencies; `sindresorhus/nano-spawn` pushed 2026-04-01, 3 open, 592 stars.
- `node-pty` 1.1.0, 2025-12-22, MIT, native (`node scripts/prebuild.js || node-gyp rebuild`); pushed 2026-09-14, 62
- open.

### Hashing

- `hash-wasm` 4.12.0, 2024-11-19, MIT, 0 dependencies; `Daninet/hash-wasm` pushed 2024-11-19, 13 open, 1,156 stars.
- `@node-rs/xxhash` 1.7.8, 2026-09-10, MIT, 13 optional platform binaries; `napi-rs/node-rs` pushed 2026-09-16, 38 open.
- `xxhash-wasm` 1.1.0, 2024-11-19, MIT, 0 dependencies; pushed 2024-11-19, 1 open.
- `@noble/hashes` 2.4.0, 2026-08-27, MIT, 0 dependencies; pushed 2026-09-08, 4 open.
- `blake3-jit` 1.1.0, 2026-04-17, MIT, 0 dependencies; `Brooooooklyn/blake3-jit` pushed 2026-09-16, 2 open, 35 stars.
- `blake3` 3.0.0 and `blake3-wasm` 3.0.0, 2022-10-09, MIT; `connor4312/blake3` pushed 2023-02-28, 16 open.
  Both are uninstallable:
  `blake3@3.0.0` depends on `blake3-wasm@2.1.7` and `blake3-wasm@3.0.0` on `@c4312/blake3-internal@2.1.7`,
  and neither version was ever published
  (`pnpm add` failed with `ERR_PNPM_NO_MATCHING_VERSION`; `pnpm view blake3-wasm versions time` lists only 2.1.5 and
  3.0.0;
  verified, measured).

## C1: recursive watching of the repository

### How each library watches on Linux

- `chokidar` 5.0.0 (verified, source):
  - `fs.watch` on every file (`handler.js:393`) and every directory (`:549`), shared per path (`:155-221`).
  - A directory event re-reads that directory with `readdirp`, throttled to once per 1,000 ms per directory with one
  - trailing re-read
    (`handler.js:453`, `:495-517`).
  - A new directory is read before its `fs.watch` is registered (`handler.js:545-549`),
    the race tracked as open issue #1471.
  - `change` events for a path within 50 ms of a previous emitted `change` are dropped with no trailing event
    (`index.js:525-528`, `_throttle` at `:569-596`), open issue #1455 "Event throttling discards updates".
  - `_handleError` emits `error` except for `ENOENT` and `ENOTDIR` (`index.js:552-563`).
  - With `atomic` (default when not polling) it ignores paths matching `DOT_RE` (`index.js:15`, `:658`)
    and delays `unlink` by 100 ms (`index.js:486-500`).
  - Overflow handling inherits libuv, which skips `IN_Q_OVERFLOW` (`deps/uv/src/unix/linux.c:2617-2619`, per the prior
  - research).
- `@parcel/watcher` 2.6.0 (verified, source, files shipped in `src/`):
  - One inotify fd on a native thread, `inotify_add_watch` per non-ignored directory only
    (`src/linux/InotifyBackend.cc:67-79`).
  - `IN_Q_OVERFLOW` is skipped with `// overflow` then `continue` (`InotifyBackend.cc:120-123`).
  - On `IN_CREATE` or `IN_MOVED_TO` of a directory it adds a watch for that directory only and does not read it
    (`InotifyBackend.cc:167-184`), so files already inside and nested subdirectories are never watched;
    open issues #97 "folders created via `mkdir -p` not watched recursively" and #243.
  - A failed `inotify_add_watch` during `subscribe` throws (`:73-76`),
    but a failure for a directory created later only removes it from the tree (`:179-183`).
  - Linux builds define `WATCHMAN` (`binding.gyp:39-51`), and the `default` backend returns Watchman when it is
  - available
    (`src/Backend.cc:39-43`).
  - `getEventsSince` uses `DirTree::getCached(root)` (`src/DirTree.cc:31-47`, `src/shared/BruteForceBackend.cc:6-16`),
    so with a live subscription on the same root it diffs the in-memory tree instead of the disk.
  - Linux crawls with `src/unix/legacy.cc` (`binding.gyp:45`).
  - Ignore globs are converted with `picomatch.makeRe` and matched with `std::regex` (`wrapper.js:26-44`);
    open issues #244 (regex slowdown) and #250 (stack overflow on long paths).
  - Debounce: first event after 500 ms of quiet is sent at once, later ones batched in 50 ms windows
  - (`src/Debounce.hh:9-10`, `Debounce.cc:70-99`).
- `watcher` 2.3.1 (verified, source):
  per-directory `fs.watch` on Linux (`dist/constants.js:11` `HAS_NATIVE_RECURSION = IS_MAC || IS_WINDOWS`),
  default `depth` 20, `debounce` 300 ms, `limit` 10,000,000 (`dist/constants.js:4-6`).
- `watchpack` 2.5.2 (verified, source):
  per-directory `fs.watch` (`lib/watchEventSource.js:18`, `:340-352`),
  errors reported with `console.error` rather than an event (`lib/DirectoryWatcher.js:677`, `:689`, `:699`),
  initial `FS_ACCURACY` of 2,000 ms (`lib/DirectoryWatcher.js:26`).
- `node-watch` 0.7.4 (verified, source):
  detects Node's recursive `fs.watch` (`lib/has-native-recursive.js`) and then passes `recursive: true`
  without an ignore list; its `filter` skip prunes only in the non-native branch (`lib/watch.js:391-437`).
- `nsfw` 2.3.1 (verified, source):
  native inotify loop with no `IN_Q_OVERFLOW` branch (`rg IN_Q_OVERFLOW src/linux` found none),
  exclusions as absolute paths only (`README.md:87`).
- Watchman via `fb-watchman` (verified, doc):
  on `IN_Q_OVERFLOW` Watchman recursively rescans the root and marks all files changed
  (`website/docs/troubleshooting.md`, section "Recrawl", fetched from `facebook/watchman` `main`).

### Setup cost on the checkout

`node --max-old-space-size=2048 SCRATCH/node-libs/watch-scale.ts <lib> /var/home/user/Monochromatic`,
excluding `node_modules`, `.git`, `target`, and `dist` by basename,
two runs each (output `repo-watch-scale.jsonl`, verified, measured).
The watch count is read from `/proc/self/fdinfo`;
the positive control on a 101-directory, 500-file fixture gave chokidar 601 watches and every other library 101.

- `chokidar`:
  `ready` after 1,776 and 1,739 ms, 41,209 watches, maximum event-loop delay 231 and 226 ms, RSS 404 and 403 MiB.
- `@parcel/watcher` with glob ignores (`**/node_modules` and so on):
  `subscribe` resolved after 1,527 and 1,521 ms, 5,481 watches, maximum delay 39 ms both runs, RSS 97 MiB.
- `@parcel/watcher` with one `RegExp` ignore:
  466 and 486 ms, 5,481 watches, maximum delay 38 and 39 ms, RSS 97 and 96 MiB.
- `watcher`:
  `ready` after 1,103 and 1,145 ms, 5,483 watches (it also watches parent directories), maximum delay 40 and 39 ms, RSS
  224 and 247 MiB.
- `watchpack`:
  no `ready` event; watch count stable after 563 and 579 ms, 5,481 watches, maximum delay 107 and 115 ms, RSS 209 MiB.
- `node-watch` was not run on the checkout:
  `node node-watch-skip.ts` on the fixture registered 664 watches where pruning `node_modules` and `dist` would give
  601,
  so on the checkout it would try to watch every path outside `.git`
  (99,417 directories plus 873,905 files per the prior census, more than `max_user_watches` 524,288; inference).
- The deepest non-excluded directory is 17 levels below the root (`node max-depth.ts`, verified, measured),
  inside `watcher`'s default `depth` of 20 today.

### Event delivery on fixtures

`node SCRATCH/node-libs/watch-events.ts <lib> <absolute fixture root>` (output `watch-events.jsonl`, verified,
measured).
Each scenario starts a fresh watcher, waits 2.5 seconds, discards events seen so far, then acts.
A first run without that discard reported watchpack's initial-scan events as scenario events;
that run is kept as `watch-events-v1-contaminated.jsonl` and not used.

- Single write to an existing file (positive control): delivered by all five.
- Two writes to one file 20 ms apart, any event after the second write:
  chokidar no (1 event), `@parcel/watcher` yes (2), `watcher` yes (1), `watchpack` yes (2), `node-watch` yes (1).
- `mkdirSync(new/a/b, { recursive: true })` then an immediate write inside:
  `@parcel/watcher` missed the file; the other four reported it.
  Chokidar's documented race (#1471) did not reproduce here.
- A tree with `tree/x/y/pre.txt` renamed into the root, then `tree/x/y/post.txt` written 1 second later:
  `@parcel/watcher` reported only `tree` and missed both files; the other four reported all three.
- 300 later writes without a flood: 300 of 300 for all five.
- 30,000 synchronous create and unlink pairs while the JavaScript thread was busy (about 0.7 to 0.95 seconds), then 300
- writes:
  chokidar 0, `watcher` 0, `watchpack` 0, `node-watch` 0, all with no `error` event;
  `@parcel/watcher` 300 of 300.
  The flood in the same directory as the later writes gave the same results.
- `watchpack` reported 104 to 302 `change` events for files written up to about 2 seconds before `startTime`
  (`preActionEvents`), explained as `"scan (file)"` (`watchpack-trace.ts`);
  after the flood it delivered 0 (`changesBeforeFlood` 302, `changes` 302).
- Inotify watches per fixture: chokidar and `node-watch` 303 (one per file), `@parcel/watcher` and `watchpack` 3,
- `watcher` 5.
- A raw `fs.watch` control on the flood directory received exactly 16,384 events,
  the `max_queued_events` limit (`watchpack-debug.ts raw`).

### `@parcel/watcher` snapshot query under a live subscription

`node parcel-snapshot.ts live` subscribed, wrote a snapshot, moved in `tree/x/y`, wrote `tree/x/y/hidden.txt`,
then called `getEventsSince` in the same process:
it returned only `create .../tree`.
`node parcel-snapshot.ts fresh` in a new process with the same snapshot returned
`tree`, `tree/x`, `tree/x/y`, and `tree/x/y/hidden.txt` (verified, measured).
A rescan through `getEventsSince` therefore needs a process without a subscription on that root.

### Loss detection without FFI

`node canary-probe.ts <root> control|flood` (verified, measured):
a `worker_threads` worker overwrites a different pre-created `canary-<n>` file every 20 ms inside the chokidar-watched
root,
and the main thread records which sequence numbers produced `change` events.

- Control, no flood: 48 written, 48 seen, 0 missing, and 300 of 300 later writes seen.
- Flood: 90 written, 56 seen, 34 missing (first missing 31, 32, 33), and 0 of 300 later writes seen.
- A first variant that created new `canary-<n>` files found no gap (93 written, 93 seen):
  chokidar's directory re-read reports every file present after the flood,
  so only modification canaries reveal loss.
- Because libuv shares one inotify instance per loop, an overflow anywhere drops canary events too (inference from the
- measured gap).
  The canary does not locate lost paths; a gap means "rescan".

### Designs and disqualifying problems

#### chokidar 5 (incumbent)

- Design:
  one `watch(root, { ignored, ignoreInitial: true, followSymlinks: false })`;
  the daemon re-hashes each changed path again at least 60 ms after its last event to cover the 50 ms throttle,
  re-reads each `addDir` directory after a short delay to cover #1471,
  and runs a worker-thread modification canary that triggers a stat and hash rescan on a gap (all unverified at
  repository scale).
- Pros:
  incumbent in two packages with regression tests;
  1 transitive dependency and no native code;
  its directory re-reads recovered `mkdir -p` and moved-in trees;
  every loss except overflow leaves an event on the path for the daemon to re-verify.
- Cons:
  41,209 watches (7.9 percent of `max_user_watches`), 403 to 404 MiB RSS, 226 to 231 ms blocking at startup;
  drops the final change within 50 ms (measured);
  silent loss after overflow (measured);
  the repository already records about 20 percent atomic-save loss with `awaitWriteFinish` (`watch-restart`).

#### `@parcel/watcher` 2.6.0

- Design:
  `subscribe(root, cb, { backend: 'inotify', ignore: [RegExp] })`;
  on every directory `create`, a second `subscribe` for that directory plus a daemon walk that emits synthetic creates;
  the same canary for overflow;
  rescans through `getEventsSince` in a child process.
- Pros:
  5,481 watches, 466 to 486 ms setup off the JavaScript thread, 96 to 97 MiB RSS;
  its native reader drained the flood that emptied every libuv-based watcher;
  no change throttle.
- Cons:
  new directories stay unwatched, so files there produce no event at all (measured);
  overflow is dropped in C++ (source);
  silent switch to Watchman when installed unless `backend` is set;
  glob ignores cost 3.1 to 3.3 times the `RegExp` setup and have an open stack-overflow issue (#250);
  snapshot queries are stale under a live subscription (measured);
  open crash issues #49, #258, #264; native binary.

#### `watcher` 2.3.1

- Pros: 5,483 watches, correct results in every non-flood scenario, 0 native code.
- Cons:
  silent overflow loss;
  1,103 to 1,145 ms setup and 224 to 247 MiB RSS;
  silent default `depth` 20 and `debounce` 300 ms;
  last release 2024-04-06;
  rejected by the repository in 2026-05 for `watch-restart` (commit `af70baa0a`).

#### `watchpack` 2.5.2

- Disqualifying: no `error` event (errors go to `console.error`), no `ready` event,
  false `change` events for files written shortly before start, silent overflow loss (all measured or source).

#### `node-watch` 0.7.4

- Disqualifying: on Node 26 its `skip` does not prune watches (measured 664 versus 601),
  so the checkout's dependency directories would exceed the inotify watch limit (inference).

#### `nsfw` 2.3.1

- Disqualifying: compiles with `node-gyp` at install, has no overflow handling, and excludes only absolute paths,
  so every package's `node_modules` must be listed and kept current.

#### Watchman through `fb-watchman`

- Pros: the only option that turns `IN_Q_OVERFLOW` into a documented recrawl.
- Disqualifying for 0.x:
  an external daemon the user must install and keep running;
  no current release assets, the newest RPM targets Fedora 42 while the host is Fedora 44,
  and `mise` has no `watchman` entry;
  its idle watches persist 5 days by default (`idle_reap_age_seconds`, per the editord handover `:365-370`);
  `fb-watchman` was last published 2022-09-21.

#### Own inotify reader through `node:ffi` (prior research)

- Pros: sees `IN_Q_OVERFLOW` directly (measured in the prior research).
- Cons: Stability 1 experimental module and hand-written directory walking, which the user ranks after a documented
- library path.

#### C1 ranking

chokidar > `@parcel/watcher` > `watcher` > `node:ffi` reader > Watchman > `watchpack` > `nsfw` > `node-watch`.

- chokidar over `@parcel/watcher`:
  chokidar's losses each leave an event the daemon can re-verify,
  while `@parcel/watcher` leaves whole new subtrees with no event at all;
  chokidar is also the incumbent (`RCI`).
  The price is about 307 MiB more RSS and about 1.3 seconds more startup.
- `@parcel/watcher` over `watcher`:
  lower setup time and memory, and it survived the measured flood; both need new-directory handling or the canary.
- `watcher` over the `node:ffi` reader:
  a documented library with correct event results beats experimental hand-written code per the user's rule.
- `node:ffi` reader over Watchman:
  in-process with no external install, although only Watchman documents overflow recovery.
- Watchman over `watchpack`:
  Watchman at least recovers from overflow; `watchpack` hides errors and reports false changes.
- `watchpack` over `nsfw`: `nsfw` needs a compiler toolchain at install and also drops overflow.
- `nsfw` over `node-watch`: `node-watch` would exhaust the watch limit on this checkout.

## C2: per-task cgroup placement, freeze, and kill

### systemd D-Bus facts

- `StartTransientUnit(s name, s mode, a(sv) properties, a(sa(sv)) aux)` (`src/core/dbus-manager.c:3158-3162`).
  The name and mode are checked first (`:1138-1144`), then the unit is loaded and made transient,
  then properties are applied (`:1025-1084`), then the start job is queued (`:1167`) (verified, source).
- Scopes take existing processes through `PIDs` (`au`) or `PIDFDs` (`src/core/dbus-scope.c:88`, `:143`),
  and support `RuntimeMaxUSec` (`:79`) (verified, source).
- Services take stdio as fds (`StandardInputFileDescriptor` and so on, `src/core/dbus-service.c:700-703`),
  as named fds, or as paths (`StandardOutputFile`, `StandardOutputFileToAppend`, `src/core/dbus-execute.c:3335-3395`).
  A path that turns out to be an `AF_UNIX` socket is connected to instead of opened (`src/core/exec-invoke.c:267-295`)
  (verified, source).
- `FreezeUnit` and `ThawUnit` reply only when the freezer operation completes, and fail with
  "Unit has a pending job", "Unit is not active", or "Unit is frozen by a parent slice"
  (`src/core/dbus-unit.c:797-852`, verified, source).
  `systemctl freeze` documents that a unit is thawed automatically before a job runs against it
  (`man/systemctl.xml:632-645`).
- `KillUnit(name, whom, signal)` with `SIGKILL` writes `cgroup.kill` (`src/core/unit.c:4145-4152`, verified, source).
- `systemd-run --scope` prints "Running as unit: ..." through `log_info` unless `--quiet` (`src/run/run.c:2843-2847`,
- `:2259-2268`),
  expands `${VAR}` unless `--expand-environment=no` (`:2849-2866`), and then `execvpe`s the command with its own
  environment (`:2869`)
  (verified, source).

### D-Bus library probe

`node SCRATCH/node-libs/dbus-probe.ts <lib>` against the user manager (output `dbus-probe.jsonl`, verified, measured):
`Properties.Get(Manager, Version)`, `GetUnitByPID`, the invalid-mode `StartTransientUnit` carrying
`ExecStart` `a(sasb)`, `Environment` `as`, `MemoryMax` `t`, and `AddRef` `b`,
50 timed `GetUnitByPID` calls, then `GetUnitByPIDFD(h)` and one more call on the same connection.

- `@homebridge/dbus-native` 0.7.9:
  `Version` came back undecoded as `[[{"type":"s","child":[]}],["259.8-1.fc44"]]`;
  the transient call reached systemd's mode check ("Job mode probe-invalid-mode is invalid.") as a plain object, not an
  `Error`;
  p50 0.47 and 0.50 ms, p90 0.57 and 0.59 ms (two runs);
  `h` failed locally with "Unknown data type format: h", and the connection kept working.
- `dbus-native` 0.15.2:
  `Version` decoded to `"259.8-1.fc44"`; errors are `DBusError`; p50 0.43 and 0.45 ms, p90 0.52 and 0.57 ms;
  `h` was marshalled without an fd and systemd answered "Bad message"; the connection kept working.
- `@jellybrick/dbus-next` 0.11.3:
  p50 0.52 ms, p90 0.68 ms;
  `h` printed "Sending file descriptors is not supported in current bus connection",
  the call never settled within 3 seconds, and the next call on the connection never settled either.
- `dbus-next` 0.10.2 (built without `usocket`):
  p50 0.47 ms, p90 0.60 ms;
  `h` never settled, and the next call rejected with "Tried to write a message to a closed stream";
  in a run without an `error` listener that emitted `error` crashed the process.
- `@balena/systemd` 0.5.11 was not probed:
  its README lists only `GetUnit`, `StartUnit`, `StopUnit`, `RestartUnit`, `ActiveState`, and `PartOf`,
  with no transient units, freeze, or kill (verified, doc).
- The invalid-mode check only proves that systemd accepted the call signature `ssa(sv)a(sa(sv))`;
  systemd did not parse the property variants (unverified whether each library marshals them correctly).
- No pure JavaScript D-Bus library can pass fds:
  Node has no `SCM_RIGHTS` API (`nodejs/node#53391`, closed as not planned on 2025-09-11, `gh issue view`),
  which `dbus-native` states in `lib/constants.js:86-98`,
  and `dbus-next` passes fds only through `usocket` (`lib/connection.js:48-57`) (verified, source).
- For comparison, one `systemctl --user show` process took 6.97 ms p50 in the prior research.

### Option designs

#### Option P1: `systemd-run --user --scope` launcher plus D-Bus control

- Spawn (design sketch, not run):

  ```ts
  // daemon task launcher sketch
  spawn('systemd-run', [
    '--user', '--scope', '--quiet', '--expand-environment=no',
    `--unit=${tool}-task-${id}.scope`, `--slice=${tool}-tasks.slice`,
    '-p', `MemoryMax=${memoryMax}`, '--', ...argv,
  ], { detached: true, env, cwd, stdio: ['ignore', 'pipe', 'pipe'] });
  ```

  systemd-run joins the new scope and then `execvpe`s the task, so the task starts inside its cgroup (verified, source).
- Stdio and terminal:
  the task inherits the daemon's pipes and exact `env` and `cwd`;
  `detached: true` gives it its own session, so Ctrl+C in the daemon's terminal reaches only the daemon.
  Node's `'pipe'` stdio is a socketpair (`deps/uv/src/unix/process.c:207`), so `echo x > /dev/stdout` inside a task
  fails:
  `spawnSync('sh', ['-c', 'echo via-dev-stdout > /dev/stdout; ...'])` printed
  `sh: line 1: /dev/stdout: No such device or address` and `/proc/self/fd/1 -> socket:[...]` (verified, measured).
  Repository sources mention `/dev/std*` only in 3 `wg-quicker` comments (`rg`, verified, measured).
- Exit: the `exit` event of the spawned PID, which becomes the task after `execvpe`.
- Freeze, thaw, kill: `FreezeUnit`, `ThawUnit`, and `KillUnit(name, 'all', 9)` through `@homebridge/dbus-native`;
  a global pause freezes `<tool>-tasks.slice`.
- Cost: one `systemd-run` process per task plus the cgroup attach lock the prior research measured at about 10 ms;
  the total was not measured because it creates a scope (unverified).
- Cons:
  the scopes live under a systemd slice, not under a daemon-owned delegated root,
  so the design's "daemon creates one cgroup per task below its delegated root" does not apply,
  and writing `cgroup.freeze` directly would bypass systemd's `FreezerState`;
  `FreezeUnit` fails with "Unit has a pending job" while a stop job runs (source).

#### Option P2: transient service through D-Bus from JavaScript

- `StartTransientUnit` of `<tool>-task-<id>.service` with `Type=exec`, `ExecStart`, full `Environment`,
- `WorkingDirectory`, `AddRef=true`,
  and `StandardOutputFile` and `StandardErrorFile` pointing at daemon-created FIFOs or `AF_UNIX` sockets, because fds
  cannot be passed.
- The task is a child of the user manager: exit status comes from unit properties (`ExecMainStatus`, `Result`) or
- `JobRemoved`,
  and the daemon must pass its entire environment explicitly.
- A socket path breaks `/dev/stdout` reopening the same way pipes do (inference);
  a FIFO path avoids that but Node has no `mkfifo` API, so it needs the `mkfifo` command or FFI (verified by absence in
  `node:fs`, inference).
- Not probed: every step creates a unit (unverified).
- Ranked after P1 because it adds signal handling, environment marshalling, and FIFO plumbing for the same placement.

#### Option P3: delegated subtree with direct cgroupfs writes

- Freeze and kill through `cgroup.freeze` and `cgroup.kill` in daemon-owned cgroups (prior research).
- Placement before exec without FFI needs a launcher that joins a cgroup and execs:
  none is installed (`cgexec` absent), a Node `process.execve` launcher is Stability 1 and 42.80 to 46.19 ms (prior
  research),
  and a post-spawn `cgroup.procs` write lets the task fork before it moves (inference).
- With `node:ffi` `pidfd_spawnp` and `POSIX_SPAWN_SETCGROUP` it is the fastest path (0.41 to 0.43 ms, prior research).

#### C2 rankings

Placement: P1 > P3 with `node:ffi` > P2 > P3 without FFI.

- P1 over P3 with `node:ffi`: P1 uses documented, stable interfaces;
  P3's speed matters only if P1's unmeasured per-task cost proves too high.
- P3 with `node:ffi` over P2: P2 is unprobed and needs FIFO plumbing and property-based exit tracking.
- P2 over P3 without FFI: P2 still places the task before exec, while a post-spawn write can let children escape.

D-Bus client: `@homebridge/dbus-native` > `dbus-native` > `systemctl` CLI > `@jellybrick/dbus-next` > `dbus-next` >
`@balena/systemd`.

- `@homebridge/dbus-native` over `dbus-native`:
  both completed every needed call at 0.43 to 0.50 ms p50;
  the incumbent avoids a second D-Bus library.
  `dbus-native` has the better API (decoded variants, `DBusError`, `bigint`, 1 dependency),
  so migrating key-helper to it is a separate decision worth taking.
- `dbus-native` over the `systemctl` CLI: about 0.45 ms per call against 6.97 ms for a process.
- `systemctl` over `@jellybrick/dbus-next`: the fork wedges its connection after an fd-typed call.
- `@jellybrick/dbus-next` over `dbus-next`: `dbus-next` is unmaintained since 2021 and was removed from this repository
- for advisories.
- `dbus-next` over `@balena/systemd`: `@balena/systemd` lacks the needed methods entirely.

## C3: JSON-RPC 2.0 over a Unix socket

### Probe

`node --expose-gc SCRATCH/node-libs/rpc-probe.ts vscode|json-rpc-2.0` on an abstract Unix socket,
server and 200 clients in one process, two runs (verified, measured).

- `vscode-jsonrpc` 9.0.2:
  a request cancelled after 50 ms rejected on the client with code `-32800`,
  and the server's `CancellationToken` saw it 51.1 to 51.3 ms after the handler started.
  Fan-out of 1,000 notifications to each of 200 clients took 3,580 and 3,613 ms.
  100,000 notifications to one client that never reads: 343 write promises settled within 3 seconds, and the heap grew
  81 MiB.
- `json-rpc-2.0` 1.8.0 with `readLines` and newline-delimited frames:
  the same fan-out took 992 and 912 ms.
  It has no server-side cancellation; `.timeout()` only rejects the client promise (`README.md` "With timeout").

### Source facts

- `vscode-jsonrpc` frames with `Content-Length` headers (`SocketMessageReader` and `SocketMessageWriter`,
- `lib/node/main.d.ts:32`).
  Every write goes through a `Semaphore` whose waiting array has no bound (`lib/common/semaphore.js`),
  and a write resolves on the socket write callback (`lib/node/ril.js:79-97`).
  `createClientPipeTransport` listens and closes its server after the first connection (`lib/node/main.js:213-220`),
  so a multi-client daemon builds its own `net.Server` and one connection per socket (verified, source).
- `jayson` 4.3.0's TCP server writes responses with no delimiter and ignores `write()` backpressure
  (`lib/server/tcp.js`), parses with `stream-json`, and has no server-to-client notifications (verified, source).

### Designs

- R1, `json-rpc-2.0` plus `mcp-stdio` `readLines`:
  one `JSONRPCServerAndClient` per socket, newline-delimited frames,
  a daemon-owned bounded queue per subscriber that waits for `'drain'` (the `processStdoutWriter` pattern),
  and a hand-written cancel notification mapping request ids to `AbortController` through `addMethodAdvanced`.
- R2, `vscode-jsonrpc`:
  built-in cancellation and `$/progress`, but header framing, a fan-out 3.6 to 4.0 times slower in this probe,
  and an unbounded write queue that the daemon must bound by counting its own unsettled promises.
- R3, hand-written dispatch on `mcp-stdio` types:
  no new dependency, but request id tracking, method registry, and error mapping are repository code.
- R4, `jayson`: disqualified by the missing server notifications and missing frame delimiters.

Ranking: R1 > R2 > R3 > R4.

- R1 over R2: subscription fan-out and bounded subscriber queues are the daemon's main traffic,
  and R1 measured 3.6 to 4.0 times faster with a write path the daemon controls;
  cancellation is the only feature R2 adds.
- R2 over R3: R2 ships tested cancellation and progress.
- R3 over R4: R4 cannot push notifications at all.

## C4: process supervision

### Probe

`node SCRATCH/node-libs/spawn-lib-probe.ts`:
the task prints a line and exits at once, leaving a `sleep 3` grandchild in its own session that inherits stdout
(verified, measured).

- `child_process` `exit` event: 40 ms.
- `child_process` `close` event: 3,035 ms.
- `execa` default: 3,038 ms; with `buffer: false`: 3,034 ms.
- `nano-spawn`: 3,033 ms.
- `tinyexec`: 3,035 ms.

### Source facts

- `nano-spawn` merges `process.env` into any `env` it is given (`source/options.js:16`),
  and `tinyexec` does the same and prepends `node_modules/.bin` unless `nodePath: false` (`dist/main.mjs:44-53`);
  neither can pass an exact environment (verified, source).
- `tinyexec` settles on `close` (`dist/main.mjs:328`) and implements `timeout` with `AbortSignal.timeout` (`:317`),
  which sends Node's `killSignal`, `SIGTERM` by default (verified, source).
- `execa` defaults: `extendEnv: true`, `cleanup: true`, `forceKillAfterDelay: true`, `killDescendants: false`
  (`lib/arguments/options.js:53-68`), `maxBuffer` 100,000,000 (`lib/arguments/specific.js:101`).
  `cleanup` is skipped for `detached` subprocesses (`lib/terminate/cleanup.js:5-8`),
  and `killDescendants` signals only the process group (`lib/terminate/kill-descendants.js:7-33`) (verified, source).

### Designs

- S1, `child_process.spawn` directly:
  exact `env`, `detached: true`, task status from `exit`, streams read to their own end.
- S2, `execa` with `extendEnv: false`, `buffer: false`, `detached: true`:
  richer errors, but its completion waits for stdio close, `cleanup` is off when detached, and it adds 17 transitive
  dependencies.
- S3, `nano-spawn` (incumbent) or `tinyexec`:
  0 dependencies, but they cannot drop inherited environment variables, which undermines a cache key over the declared
  environment.

Ranking: S1 > S2 > S3.

- S1 over S2: the daemon needs the `exit` time separately from stdio close, and cgroup kill replaces `execa`'s
- process-group kill.
- S2 over S3: S2 can pass an exact environment.
- `nano-spawn` stays appropriate for short helper commands such as `git` queries, where inheriting the environment is
- intended.

## C5: hashing

`node SCRATCH/node-libs/hash-bench.ts`, 128 MiB random buffer and 35,724 buffers of 4 KiB, three runs, in memory
(verified, measured; ranges are minimum to maximum):

- `node:crypto` `sha256`: 1,641 to 1,883 MiB/s; small set 108 to 121 ms.
- `node:crypto` `blake2b512`: 1,063 to 1,111 MiB/s; 158 to 167 ms.
- `hash-wasm` BLAKE3: 672 to 689 MiB/s; 205 to 210 ms.
- `blake3-jit`: 1,129 to 1,225 MiB/s; 117 to 124 ms.
- `@noble/hashes` BLAKE3: 64 to 65 MiB/s; 2,207 to 2,218 ms.
- `hash-wasm` XXH3-64: 6,037 to 6,331 MiB/s; 28 to 30 ms. XXH3-128: 6,011 to 6,360 MiB/s; 29 to 32 ms.
- `@node-rs/xxhash` XXH3-64: 2,428 to 2,467 MiB/s; 61 to 64 ms. XXH3-128: 2,451 to 2,456 MiB/s; 66 ms.
- `xxhash-wasm` h64: 931 to 8,501 MiB/s (unstable); 15 to 17 ms.
- The three BLAKE3 implementations produced the same digest (`blake3Agree: true`).
- The prior research measured `sha256` at 2,322 to 2,334 MiB/s with a 256 MiB buffer,
  so run-to-run spread on this host exceeds 30 percent; only differences of more than 2 times are treated as real.

Ranking: `node:crypto` `sha256` > `hash-wasm` XXH3-128 as an optional pre-filter > `blake3-jit` > `@node-rs/xxhash` >
`@noble/hashes` BLAKE3.

- `sha256` over an XXH3 pre-filter: the design doc measured a full sequential rehash of tracked files at a 437 ms
- median,
  so a pre-filter has no measured need and adds a dependency last released 2024-11-19.
- XXH3-128 over `blake3-jit`: 4.9 to 5.6 times the throughput if a pre-filter is ever needed; 128 bits keeps collisions
- unlikely (inference).
- `blake3-jit` over `@node-rs/xxhash`: a cryptographic hash usable as the key itself.
- `@node-rs/xxhash` over `@noble/hashes` BLAKE3: the latter is 25 to 29 times slower than `sha256`.

## Documentation problems recorded

Recorded without culling, per the user's decision.

- `chokidar` 5.0.0:
  - `README.md` says "events are not reported twice", while later distinct changes within 50 ms are dropped
  - (`index.js:525-528`, #1455, measured).
  - `README.md` describes the `atomic` default in terms of `useFsEvents`, which exists nowhere in the v5 code
  - (`rg useFsEvents` matches only `README.md`).
  - The troubleshooting section calls `ENOSPC` file-handle exhaustion, prints "ENOSP",
    and says `fs.watch` handle exhaustion "can't seem to be solved by ... OS tuning" beside the `max_user_watches`
    command.
  - `README.md` does not state that every file gets its own watch (measured 41,209).
- `@parcel/watcher` 2.6.0:
  - `README.md` says Linux queries use "fts (brute force)", while Linux builds use `src/unix/legacy.cc`
  - (`binding.gyp:39-46`).
  - `README.md` says `ignore` takes "paths or glob patterns", while `index.d.ts` and `wrapper.js:12-25` also accept
  - `RegExp`.
  - `README.md` does not mention that new directories are not crawled, that overflow is dropped,
    or that `getEventsSince` reuses a live subscription's in-memory tree.
- `watcher` 2.3.1: no `license` field in `package.json`;
  `readme.md` says chokidar is "not very actively maintened" (chokidar pushed 2026-08-16);
  the silent `depth` 20 default appears only in the options list.
- `watchpack` 2.5.2: `README.md` documents no `ready` or `error` event,
  and does not say that files modified within `FS_ACCURACY` (2,000 ms) before `startTime` are reported as changed
  (measured).
- `node-watch` 0.7.4: `README.md:71-72` says Linux lacks native recursion and `skip` makes watching efficient,
  while on Node 26 it uses Node's recursion and `skip` no longer prunes watches (measured).
- Watchman: `website/docs/install.md` says to download RPM and DEB files from the latest release,
  while the latest release has 0 assets and the newest RPM is for Fedora 42.
- `@homebridge/dbus-native` 0.7.9: `README.md:33-34` clones `sidorares/node-dbus`, a different repository;
  variant results are returned undecoded with no documentation of the shape (measured);
  `index.d.ts` types only the client path (repository note in `dbus-native.d.ts`); `engines.node` `>=8.0`.
- `dbus-native` 0.15.2: `README.md` calls `docs/api.md` "the complete reference", but the npm tarball ships no `docs/`
- directory.
- `@jellybrick/dbus-next` 0.11.3: `README.md` says the earliest supported Node is 6.3.0 while `engines.node` is
- `>=12.20.0`,
  and links documentation to the upstream `acrisci.github.io` site.
- `@balena/systemd` 0.5.11: the `test` script is `echo 'No tests yet'`.
- systemd 259:
  - `man/org.freedesktop.systemd1.xml:580-582`, `:2181-2183`, `:2213`, `:2225` mark `FreezeUnit`, `ThawUnit`, `Freeze`,
  - `Thaw`,
    `FreezerState`, and `CanFreeze` as "not documented".
  - `KillUnit` prose names the argument `who` (`:1333`) while the signature says `whom` (`dbus-manager.c:3099`),
    and says "will fail of no matching process" (`:1342`).
  - `StartTransientUnit` prose says `aux` "is currently unused" (`:1602`), while the source parses auxiliary units
    (`dbus-manager.c:1086`, `:1162`).
- `vscode-jsonrpc` 9.0.2: the 54-line `README.md` never mentions sockets, cancellation, or progress;
  its example `NotificationType<string, void>` passes two type arguments to a class declared `NotificationType<P>`
  (`lib/common/messages.d.ts:274`); the listen-once behavior of `createClientPipeTransport` is undocumented.
- `jayson` 4.3.0: `README.md:346` says notifications set `id` to `null`, while `:385` says `id` is omitted since 3.6.1.
- `execa` 10.0.1: `cleanup` is documented as "Kill the subprocess when the current process exits"
- (`types/arguments/options.d.ts:326-330`),
  while `lib/terminate/cleanup.js:5-8` skips detached subprocesses;
  `readme.md` links `docs/*.md` pages the npm tarball does not ship.
- `tinyexec` 1.3.1: `README.md:61` says `timeout` means the process "will be forcibly killed",
  while the source sends Node's default `SIGTERM` (`dist/main.mjs:317`).
- `nano-spawn` 2.1.0: final-newline stripping and non-hard `timeout`, already recorded in `doc/troubleshooting/`.
- `hash-wasm` 4.12.0: `README.md` benchmarks are for 4.10.0 and include no BLAKE3 or XXH3 figures.
- `@node-rs/xxhash` 1.7.8: `README.md` types inputs as `BufferLike` (including `ArrayBuffer` and `number[]`),
  while the shipped `index.d.ts` accepts `Uint8Array | string`;
  the package description "Fastest xxhash implementation in Node.js" is contradicted on this host by `hash-wasm` XXH3
  at 2.4 to 2.6 times its throughput.
- `blake3` and `blake3-wasm` 3.0.0: published manifests depend on versions that were never published.

## Corrections to prior research

Claims in `stack-typescript.md` that are wrong or incomplete given these libraries.

1.  `:198-201` "`@parcel/watcher` does not fix overflow" is correct but incomplete:
    it also never watches directories created or moved in after start (measured),
    its `getEventsSince` is stale under a live subscription (measured),
    and its native reader drained a flood that emptied every libuv-based watcher (measured 300 of 300 against 0).
2.  `:201` "chokidar builds on `fs.watch` (inference ..., unverified here)":
    verified, with one watch per file and per directory (41,209 on the checkout), 1,739 to 1,776 ms to `ready`,
    226 to 231 ms of main-thread blocking, 403 to 404 MiB RSS, silent overflow loss,
    and dropped changes within 50 ms; chokidar is therefore no cheaper than the recursive `fs.watch` it measured
    (41,205 watches).
3.  `:340-342` (yikes 3) "the daemon must walk and watch directories itself" is wrong:
    `@parcel/watcher`, `watcher`, and `watchpack` already watch per directory with pruning (5,481 to 5,483 watches),
    each with its own defects.
4.  `:336-339` (yikes 2) "Without the FFI reader, the only backstop is a periodic full rescan" is wrong:
    a worker-thread modification canary detected the loss window without FFI (34 of 90 canaries missing during a flood,
    0 of 48 without).
    It detects loss; it still does not locate the lost paths.
5.  `:329-335` (yikes 1) "Every race-free and loss-free path runs through ... `node:ffi`" is too broad:
    loss detection has the canary, and freeze, thaw, and kill have D-Bus calls at 0.43 to 0.50 ms p50;
    only cheap placement before exec still needs `node:ffi`.
6.  `:128-130` fallback "freeze, thaw, and kill through `systemctl --user freeze|thaw|kill`":
    the incumbent `@homebridge/dbus-native` does the same over D-Bus without a process per call,
    `FreezeUnit` replies after the freeze completes (`dbus-unit.c:797-852`),
    and `KillUnit` with `SIGKILL` uses `cgroup.kill` (`unit.c:4145-4152`).
7.  `:131-132` "Pause, end, limits: `fs.writeFile` to `cgroup.freeze`, `cgroup.kill`" applies only to a daemon-owned
7. delegated subtree;
    with the `systemd-run --scope` fallback the scopes are systemd-owned under a slice, so direct writes would bypass
    systemd's state.
    The design doc's process model (`monorepo-manager-from-scratch-design.md:352-363`) assumes the delegated subtree.
8.  `:137-140` NDJSON with `mcp-stdio` `readLines`: still sound, but incomplete.
    Only `readLines`, the types, and the drain-aware writer pattern are reusable;
    `serve` and the serial queue are stdio and MCP specific;
    `vscode-jsonrpc` offers built-in cancellation with a fan-out 3.6 to 4.0 times slower and an unbounded write queue,
    and `json-rpc-2.0` offers dispatch with no server cancellation.
9.  `:252-256` Ctrl+C and exit handling miss two Node-wide facts:
    `'pipe'` stdio is a socketpair, so `> /dev/stdout` inside tasks fails with `ENXIO` (measured),
    which the FFI `pipe2` design avoided and the `systemd-run` fallback does not;
    and every spawn library settles on stdio close, 3 seconds after `exit` when a grandchild keeps stdout open
    (measured).
10. `:243-247` BLAKE3 "needs a library: `@noble/hashes` ... or `hash-wasm`" omits that `@noble/hashes` BLAKE3 runs at
10. 64 to 65 MiB/s,
    that `blake3-jit` reaches 1,129 to 1,225 MiB/s,
    and that the `blake3` and `blake3-wasm` packages cannot be installed.
11. `:322-323` "No other third-party library is required" held only for the all-FFI design;
    the library design uses chokidar and `@homebridge/dbus-native` (incumbents) and `json-rpc-2.0`.
12. `:716-727` "That path is slower and loses overflow detection":
    the non-FFI path loses the `IN_Q_OVERFLOW` signal but not loss detection (canary),
    and its freeze and kill calls are sub-millisecond over D-Bus.
13. The design doc summary (`monorepo-manager-from-scratch-design.md:190-195`) says Node needs `node:ffi` for
13. watch-overflow detection;
    that is no longer required.

## Revised Node design

Libraries per capability, each chosen by the ranking in its section.

- Watching: `chokidar` 5 (incumbent).
  - One watcher on the repository root with an `ignored` function pruning dependency and output directories.
  - A coalescing window before scheduling, with content hashing as the source of truth.
  - A trailing re-hash of every changed path at least 60 ms after its last event, covering the 50 ms change throttle.
  - A delayed re-read of each new directory, covering the scan-then-watch race (#1471).
  - A `worker_threads` modification canary; a sequence gap triggers a full stat and hash rescan.
  - The watcher module belongs next to file-enforcer's watch code so both share one owner (`RCI`).
  - If 403 MiB RSS or 41,209 watches prove too costly, switch to `@parcel/watcher` with a `RegExp` ignore,
    `backend: 'inotify'`, and a per-new-directory subscription plus walk.
- Placement: `systemd-run --user --scope --quiet --expand-environment=no --unit=... --slice=...` spawned with
- `child_process`.
  Measure its per-task cost first; if it is too high, the fast path is `node:ffi` `pidfd_spawnp` into a delegated
  subtree,
  which changes the process model to the one in the design doc.
- Freeze, thaw, kill, unit state: `@homebridge/dbus-native` calling `FreezeUnit`, `ThawUnit`, `KillUnit`, `GetUnit`,
  with a slice-level freeze for a global pause, retries on "Unit has a pending job",
  and a decoder for its raw variant shape.
  Consider migrating key-helper and this daemon to `dbus-native` 0.15.2 as one follow-up decision.
- RPC: `json-rpc-2.0` with `mcp-stdio` `readLines`, newline-delimited frames on a socket in a 0700 directory under
- `$XDG_RUNTIME_DIR`,
  a bounded per-subscriber queue with drain-aware writes, and a hand-written cancel notification backed by
  `AbortController`.
- Processes: `child_process.spawn` with an exact `env`, `detached: true`, task status from `exit`,
  and cgroup emptiness (unit inactive) as the end of the task tree;
  `nano-spawn` only for helper commands.
- Hashing: `node:crypto` `sha256`; no hashing library.
- `node:ffi` becomes optional: needed only if `systemd-run` placement is too slow or a precise overflow signal is
- required.

## Remaining yikes

Ranked by severity, most severe first.

1.  **Per-task placement cost without FFI is still unmeasured.**
    `systemd-run --scope` starts a process, talks to systemd, and pays the attach lock for every task;
    measuring it creates scopes, which this research did not do.
2.  **No Node library reports inotify overflow.**
    chokidar, `watcher`, `watchpack`, and `node-watch` lost 300 of 300 writes with no error, and `@parcel/watcher`
    drops the signal in C++.
    The canary detects loss windows but was measured only on a fixture, and each detection costs a full rescan.
3.  **The incumbent watcher is heavy and lossy at repository scale.**
    41,209 watches, 403 to 404 MiB RSS, 226 to 231 ms of startup blocking,
    final changes within 50 ms dropped,
    and a recorded atomic-save flake in `watch-restart`.
    The daemon-side re-verification that covers the throttle is unverified.
4.  **The alternative watcher goes blind in new subtrees.**
    `@parcel/watcher` misses files in `mkdir -p` directories and moved-in trees without any event;
    its workaround multiplies subscriptions.
5.  **Tasks cannot reopen `/dev/stdout` or `/dev/stderr`.**
    Node stdio pipes are sockets; fixing it needs per-task log files, `mkfifo`, or FFI `pipe2`.
6.  **Two process models do not mix.**
    systemd-owned scopes under a slice (no FFI) and daemon-owned delegated cgroups (FFI) need different freeze, kill,
    and doctor checks.
7.  **D-Bus from JavaScript cannot pass fds.**
    Transient services need FIFOs or sockets for stdio;
    two `dbus-next` variants wedge their connection after an fd-typed call;
    the incumbent returns raw variants and non-`Error` errors;
    systemd leaves the freeze methods undocumented.
8.  **Library-built RPC still needs daemon code for its hard parts.**
    `json-rpc-2.0` has no cancellation, and `vscode-jsonrpc` queues writes without bound (81 MiB for 100,000
    notifications to a paused client).
9.  **Maintenance signals are uneven.**
    `hash-wasm` and `xxhash-wasm` were last released 2024-11-19, `fb-watchman` 2022-09-21, `dbus-next` 2021-10-10;
    `@homebridge/dbus-native` has 16 stars and `@jellybrick/dbus-next` 3.
10. **Documentation problems**, recorded without culling, in "Documentation problems recorded".
