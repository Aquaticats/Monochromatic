# TypeScript runtime options for the from-scratch monorepo manager

Outcome on 2026-09-16:
every option here is out.
The user killed the TypeScript route because Node single executables are too big,
and judged Bun and Deno not worth further research;
see "Current stack direction" in `doc/planning/monorepo-manager-from-scratch-design.md`.
This file is kept as evidence.

Research date: 2026-09-16.
Input design: `doc/planning/monorepo-manager-from-scratch-design.md`
and `doc/planning/monorepo-manager-route-research/from-scratch-inputs.md`.
Nothing under `/var/home/user/Monochromatic` was modified
(`git status --short` still shows only the pre-existing `M mise.lock`).

## Conventions

- **Verified (measured)**:
  a probe ran in this session on this host; the command and output are quoted.
- **Verified (source)**:
  a cited source file and line range in a pinned checkout.
- **Verified (doc)**:
  a cited documentation page fetched with the user's `curl` command, or a doc file in a pinned checkout.
- **Unverified**:
  inference, or a claim the cited evidence does not settle.
- Runtimes on the host (`mise ls`, `node --version`, `bun --version`, `deno --version`):
  Node `v26.8.2`,
  Bun `1.3.14` (repo pin),
  Deno `2.9.6`.
  Newer releases exist and were not installed:
  Node `v26.9.0` (published 2026-09-16) and Bun `1.4.2` (published 2026-09-05),
  per `gh api repos/<owner>/<repo>/releases`.
- Source checkouts:
  - `~/temp/agent/node-v26.8.2-2026-09-16` (sparse: `lib`, `src`, `doc/api`, `deps/uv`)
  - `~/temp/agent/bun-v1.3.14-2026-09-16` (sparse: `src`, `docs`, `packages/bun-types`)
  - `~/temp/agent/deno-v2.9.6-2026-09-16`
  - `~/temp/agent/linux-v7.2-ogc6-20260827` and `~/temp/agent/systemd-259.7-20260816` (pre-existing)
  - single files under `SCRATCH/src/`:
    notify `8.2.0` `inotify.rs`,
    `@parcel/watcher` `InotifyBackend.cc`,
    Bun `1.4.2` `path_watcher.rs`, `spawn_process.rs`, `bun.d.ts`.
- `SCRATCH` abbreviates `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad`.
  Every probe script named below lives there;
  raw outputs are in `SCRATCH/watch-results.jsonl` and `SCRATCH/spawn-results.jsonl`,
  fetched doc pages in `SCRATCH/docs/`.
- Probe constraints honored:
  no cgroup was created and no process changed cgroup.
  Two probes touched cgroupfs without changing state,
  both described where used:
  a spawn with `CLONE_INTO_CGROUP` into the probe's own current cgroup,
  and a write of `0` to the probe's own `cgroup.procs` (same cgroup, membership unchanged).
  Reflink probes wrote 64 MiB test files under `~/temp/agent/ts-stack-btrfs-probe-2026-09-16`
  because `/tmp` is `tmpfs` (`findmnt --target /tmp`); that directory was removed afterward.

## Shared evidence (independent of runtime)

### Workload size

- `node SCRATCH/repo-census.ts .git node_modules target dist` printed
  `{"dirs":5481,"files":35724,"symlinks":0,"mebibytes":6922,"walkMs":5304}` (verified, measured).
- Excluding only `.git`:
  `{"dirs":99417,"files":873905,"symlinks":2105,"mebibytes":190065,"walkMs":42232}` (verified, measured).
- Current inotify usage by all readable processes (`SCRATCH/inotify-usage.ts`):
  9,398 watches in 104 instances;
  limits `max_user_watches` 524288, `max_user_instances` 8192, `max_queued_events` 16384
  (`cat /proc/sys/fs/inotify/...`, verified, measured).

### cgroup placement cost on this kernel

- The cgroup2 mount has no `favordynmods`:
  `findmnt --types cgroup2` printed
  `rw,nosuid,nodev,noexec,relatime,seclabel,nsdelegate,memory_recursiveprot,memory_hugetlb_accounting`
  (verified, measured).
- Writing to `cgroup.procs` takes the global `cgroup_threadgroup_rwsem` write lock unless favordynmods is on:
  `kernel/cgroup/cgroup.c:3091-3097` selects `CGRP_ATTACH_LOCK_GLOBAL` when `pid || threadgroup`,
  and `:2532-2544` calls `percpu_down_write` (verified, source).
  `include/linux/cgroup-defs.h:89-106` says seeding a cgroup with `CLONE_INTO_CGROUP` does not take that write lock
  (verified, source);
  `Documentation/admin-guide/cgroup-v2.rst:620-624` calls migration "relatively expensive" (verified, source).
- Measured lock cost with a same-cgroup write (`node SCRATCH/migration-cost.ts 30`, 200 ms between writes):
  `{"n":30,"minMs":"5.602","p50Ms":"9.681","maxMs":"15.657"}`;
  back-to-back writes (`SCRATCH/migration-cost-burst.ts`):
  `{"minMs":"0.004","p50Ms":"0.006","maxMs":"9.527"}` (verified, measured).
  A launcher that joins its task cgroup therefore pays roughly 10 ms per sporadic spawn on this host.
  Bun's PR #37466 body reports about 100 ms on its test machine (unverified here).
- glibc `2.43` exports `pidfd_spawn`, `pidfd_spawnp`, `posix_spawnattr_setcgroup_np` (all `GLIBC_2.39`)
  and `posix_spawn_file_actions_addchdir_np` (`nm --dynamic --defined-only /lib64/libc.so.6`, verified, measured);
  `POSIX_SPAWN_SETCGROUP` is `0x100` and `POSIX_SPAWN_SETSID` is `0x80` (`/usr/include/spawn.h:54-63`, verified,
  source).
- Negative control proving the flag reaches `clone3`:
  `node --experimental-ffi SCRATCH/setcgroup-control.ts` printed
  `tmp dir fd rc 9 Bad file descriptor` and `current cgroup fd rc 0 spawned` (verified, measured).
- `systemd-run --scope` path:
  it calls `StartTransientUnit`, waits for the job, then `execvpe`s the command itself
  (`src/run/run.c:2674-2869`, verified, source).
  It expands `${VAR}` in arguments by default (`run.c:79`, `man/systemd-run.xml:189-197`, verified, source),
  so a task launcher must pass `--expand-environment=no`.
  systemd attaches the PID through `cgroup.procs` (`src/shared/cgroup-setup.c:143-173`, verified, source),
  so the ~10 ms lock applies there too (inference).
  `systemctl kill --signal=SIGKILL` uses `cgroup.kill` (`src/core/unit.c:4146-4147`, verified, source).
  Lower bound for one systemd client call:
  `node SCRATCH/dbus-lower-bound.ts` printed `{"p50Ms":"6.97","p90Ms":"10.54"}` for `systemctl --user show` (verified,
  measured).
  The full per-task cost of `systemd-run --user --scope` was not measured because it creates a cgroup (unverified).

### Behaviors identical across the three runtimes

- A JS `SIGCHLD` listener fires for a child started through FFI `pidfd_spawnp`:
  `SCRATCH/sigchld-probe.ts` printed `"sigchldListenerFired":1` for node, bun, and deno (verified, measured).
  Exit handling can be "on `SIGCHLD`, call `waitid(P_PIDFD, fd, WEXITED|WNOHANG)` for each running task".
- An FFI inotify reader detects `IN_Q_OVERFLOW` while the event loop keeps running:
  `SCRATCH/ffi-inotify-probe.ts` printed `"overflowEvents":1` and `"timerTicksIn1500ms":149` for all three
  (verified, measured).
- `fs.watch` registers on a cgroupfs `cgroup.events` file in all three
  (`SCRATCH/cgroup-events-watch.ts` printed `watch on cgroup.events registered: populated 1 | frozen 0`, verified,
  measured);
  that the kernel's populated-change notification reaches the listener is documented
  (`cgroup-v2.rst:418-437`) but not probed (unverified).
- `os.availableParallelism()` and `navigator.hardwareConcurrency` both return 16 in all three
- (`SCRATCH/parallelism-probe.ts`, verified, measured).
- Freeze does not stop wall-clock timers in any runtime (runtime-independent; recorded in the input design).

## Option 1: TypeScript on Node

### Design sketch

- Start: the user runs the daemon under a delegated scope,
  `systemd-run --user --scope -p Delegate=yes --expand-environment=no -- node .../daemon.ts`.
  The daemon writes `0` once to `<root>/daemon/cgroup.procs`,
  enables `+cpu +io +memory +pids` in `<root>/cgroup.subtree_control`,
  and creates `<root>/tasks/<id>/` per task.
- Spawn (primary):
  `node:ffi` binds `pipe2`, `posix_spawnattr_*`, `posix_spawn_file_actions_adddup2`,
  `posix_spawn_file_actions_addchdir_np`, and `pidfd_spawnp`.
  Flags: `SETCGROUP | SETSID | SETSIGDEF | SETSIGMASK`, so the child starts inside its task cgroup,
  in its own session (terminal Ctrl+C reaches only the daemon),
  with default signal dispositions and an empty mask.
  Pipe read ends are wrapped as `new net.Socket({ fd, readable: true, writable: false })`.
  Exit: a `process.on('SIGCHLD')` listener sweeps `waitid(P_PIDFD, ..., WEXITED|WNOHANG)` over running tasks.
- Spawn (stable-API fallback):
  `child_process.spawn('systemd-run', ['--user', '--scope', '--slice=<tool>-tasks.slice', '--expand-environment=no', '-p
  ', 'MemoryMax=...', '--', ...argv], { detached: true })`,
  with freeze, thaw, and kill through `systemctl --user freeze|thaw|kill`.
- Pause, end, limits: `fs.writeFile` to `cgroup.freeze`, `cgroup.kill`, `memory.max`, `pids.max`, `cpu.max`;
  wait for `frozen 1` or `populated 0` by watching `cgroup.events`.
- Watching: one `node:ffi` inotify instance, `inotify_add_watch` per non-excluded directory,
  reads through `fs.read` on the libuv threadpool (one thread held),
  `IN_Q_OVERFLOW` or `ENOSPC` switches to "rescan and rehash",
  and every newly created directory is read with `readdir` right after its watch is added.
- RPC: `node:net` server on `$XDG_RUNTIME_DIR/<tool>/rpc.sock` inside a 0700 directory,
  NDJSON JSON-RPC 2.0 framed by `@monochromatic-dev/mcp-stdio`'s `readLines`,
  one bounded outbound queue per subscriber honoring `write()`/`'drain'`,
  and a subscriber over its bound is dropped and told to resubscribe from a sequence number.
- btrfs: `fs.statfs` for detection, `fs.copyFile(..., COPYFILE_FICLONE_FORCE)` for restore,
  `node:ffi` `ioctl` for `BTRFS_IOC_GET_SUBVOL_INFO` and subvolume or snapshot creation.
- Hashing: `crypto.hash('sha256', ...)`.
- file-enforcer: a child `node package/file-enforcer/core/src/cli.ts` per evaluation, in its own task cgroup.

### R1: per-task cgroup placement, freeze, kill, pidfd

- `child_process` has no cgroup or pre-exec hook:
  `rg cgroup doc/api/child_process.md` in the v26.8.2 checkout finds no match,
  and the `spawn` options offer no pre-exec callback (verified, source);
  libuv issue #1272 asking for a spawn hook is closed (verified, `gh issue view 1272 --repo libuv/libuv`).
- `node:ffi` exists since `v26.1.0` with Stability 1 Experimental (`doc/api/ffi.md:1-35`, verified, source);
  in `v26.8.2` it needs `--experimental-ffi` (`doc/api/cli.md:1257-1267`, verified, source);
  `v26.9.0` release notes list "ffi: enable module by default" (#65475) (verified, `gh release view v26.9.0`).
  Calls are synchronous only; the doc describes no async call mode (`doc/api/ffi.md`, verified, source).
  The installed binary has it: `node --experimental-ffi ... getpid` printed matching PIDs (verified, measured).
- Per-task overhead, `SCRATCH/spawn-probe.ts` via `SCRATCH/spawn-driver.ts`, three rounds of 200 spawns of
- `/usr/bin/true`
  (p50 across rounds, verified, measured):
  - `child_process.spawn` direct: 1.47 to 1.58 ms.
  - `/bin/sh -c 'exec "$0" "$@"'` launcher (without the cgroup write): 2.51 to 2.75 ms.
  - Node launcher using `process.execve` (without the cgroup write): 42.80 to 46.19 ms (40 spawns per round).
  - `node:ffi` `pidfd_spawnp` with `SETCGROUP` into the current cgroup, including a blocking `waitid`: 0.41 to 0.43 ms.
  - A launcher's cgroup join adds the ~10 ms lock measured in "Shared evidence".
- Pipe streaming from FFI-created pipes works without blocking the loop;
  a pidfd cannot be wrapped:
  `node --experimental-ffi SCRATCH/stdio-probe.ts` printed
  `"pidfdSocket":"throws: TypeError [ERR_INVALID_FD_TYPE]: Unsupported fd type: UNKNOWN","stdout":["line-1","line-2","li
  ne-3"],"maxTimerGapMs":22`
  (verified, measured).
- `process.execve` is also Stability 1 Experimental (`doc/api/process.md:1700-1742`, verified, source).
- Reaping: libuv only waits for PIDs it spawned (`deps/uv/src/unix/process.c:131`, verified, source),
  so FFI children stay available to the daemon's `waitid`.

### R2: recursive watching

- `fs.watch(..., { recursive: true })` on Linux is a JS emulation (`lib/fs.js:3139`, verified, source)
  that calls `fs.watch` on every file and directory and uses `readdirSync` and `statSync`
  (`lib/internal/fs/recursive_watch.js:121-185`, verified, source).
  On the checkout with an `ignore` function for the four excluded names,
  `node SCRATCH/repo-watch-scale.ts recursive` printed
  `{"dirs":5481,"syncMs":662,"watches":41205,"rssMiB":243}`:
  one watch per file and directory, and 662 ms of synchronous work (verified, measured).
- Per-directory non-recursive watches:
  `{"dirs":5481,"syncMs":36,"watches":5481,"rssMiB":90}` (verified, measured).
- libuv drops `IN_Q_OVERFLOW` silently:
  the overflow event has `wd == -1`, `find_watcher` returns `NULL`, and the loop `continue`s with the comment
  "Stale event, no watchers left" (`deps/uv/src/unix/linux.c:2617-2619`, verified, source).
  Probe (`SCRATCH/watch-probe.ts`, fixture of 401 directories, 30,000 create and unlink pairs while the JS thread is
  busy,
  then one write to each of 300 files):
  - per-directory mode printed `"burstEvents":16384,"lateModified":300,"lateSeen":0,"errors":[]`;
    positive control without the burst printed `"lateSeen":300` (verified, measured).
  - recursive mode printed `"burstEvents":0,"lateSeen":0,"errors":[]`;
    positive control printed `"lateSeen":300` (verified, measured).
- New-directory race:
  recursive mode rescans with `readdirSync` on a directory event and saw a file created inside a fresh nested directory
  (`"raceSeen":true`);
  per-directory mode has no automatic handling (the daemon adds it) (verified, measured).
- Same-directory burst of 100 distinct writes: 100 of 100 delivered in both modes (verified, measured).
- `@parcel/watcher` does not fix overflow:
  `if ((event->mask & IN_Q_OVERFLOW) == IN_Q_OVERFLOW) { // overflow continue; }`
  (`SCRATCH/src/parcel-watcher-InotifyBackend.cc:120-123`, verified, source).
  chokidar builds on `fs.watch` (inference from its dependency on Node watchers, unverified here).
- Open Node issues found for Linux watch loss: none
  (`gh search issues --repo nodejs/node --state open 'fs.watch'` returned only Windows and single-file issues #61398,
  #53111).
  Closed #58868 fixed spurious recursive rename events on Linux (verified, search output).
- FFI escape hatch detects overflow (see "Shared evidence").

### R3: Unix socket JSON-RPC server

- `node SCRATCH/socket-probe.ts` (200 clients, 1,000 NDJSON notifications each, then a paused client) printed
  `{"socketMode":"755","fanOutMs":191,"firstFalseAfterBytes":262144,"bufferedInUserSpaceBytes":66912256}`
  (verified, measured):
  backpressure is signaled by `write()` returning `false`,
  and ignoring it buffers unbounded memory,
  so per-subscriber bounds are required.
- Socket mode follows the umask (755 here), so the socket must sit in a 0700 directory (verified, measured).
- Path length: a 109-byte path failed with `listen EINVAL` emitted as an `'error'` event
  (first run of the probe, verified, measured);
  the daemon must check the 107-byte limit before binding.
- Peer credentials (`SO_PEERCRED`) have no `node:net` API (absence in `doc/api/net.md`, verified, source);
  directory permissions remain the boundary.

### R4: btrfs

- `node --experimental-ffi SCRATCH/btrfs-probe.ts ~/temp/agent/ts-stack-btrfs-probe-2026-09-16` printed
  `"statfsType":"0x9123683e","copyFile FICLONE_FORCE":{"ok":true,"ms":52.21},"copyFile default":{"ok":true,"ms":0.19},"g
  etSubvolInfo":{"rc":0,"treeid":"257","generation":"478203"},"ffiFiclone":{"rc":0,"ms":0.04}`
  (verified, measured).
  `treeid` 257 matches the `/home` subvolume id recorded in the input research.
- `filefrag -v` showed `last,shared,eof` on the same physical range for all three copies (verified, measured).
- Negative control: `COPYFILE_FICLONE_FORCE` on tmpfs threw `ENOTSUP` (`SCRATCH/ficlone-negative.ts`, verified,
- measured).
- Subvolume and snapshot creation:
  `BTRFS_IOC_SUBVOL_CREATE` and `BTRFS_IOC_SNAP_CREATE_V2` are defined at `include/uapi/linux/btrfs.h:1170` and `:1187`
  (verified, source);
  the FFI call shape is the same as the measured `GET_SUBVOL_INFO` call,
  but creation was not executed (unverified).
  `btrfs` from btrfs-progs is installed (`command -v btrfs`, verified, measured) as a child-process alternative.

### R5: hashing

- `node SCRATCH/hash-probe.ts` (256 MiB buffer, three runs; 35,724 buffers of 4 KiB) printed
  `sha256` 2,322 to 2,334 MiB/s and 97 to 106 ms for the small set,
  `blake2b512` 1,308 to 1,313 MiB/s,
  `nodeCryptoHasBlake3:false`, `nodeCryptoHasXxhash:false` (verified, measured).
- BLAKE3 needs a library:
  `@noble/hashes` (pure JS; release `2.4.0` on 2026-08-27, pushed 2026-09-08)
  or `hash-wasm` (last push 2024-11-19)
  (`SCRATCH/lib-maintenance.ts`, verified, measured).
  SHA-256 at the measured rate is sufficient for a content-hash cache (inference).

### R6: process supervision

- Spawn, streams, exit: see R1.
- Ctrl+C: `SETSID` (FFI) or `detached: true` (`child_process`) keeps tasks out of the terminal's foreground group
  (`doc/api/child_process.md` `options.detached`, verified, doc);
  the daemon's `SIGINT` handler freezes the queue, writes `cgroup.kill` per task, waits for `populated 0`, then exits.
- Graceful end: `process.kill(-pgid, 'SIGTERM')` (the session leader's PID is the group ID after `setsid`),
  then `cgroup.kill` after a timeout (inference from POSIX semantics, unverified by probe).
- Timers: `setTimeout`/`AbortSignal.timeout` for task deadlines, counting only unfrozen time (design).
- libuv threadpool: the FFI inotify reader holds one of the default 4 threads;
  set `UV_THREADPOOL_SIZE` (inference from the probe's use of `fs.read`, unverified at scale).

### R7: file-enforcer interop

- `node SCRATCH/fe-import-probe.ts` printed
  `{"importOk":true,"exportCount":80,"fsPromisesGlob":"function","globMatches":22,"asyncLocalStorage":"ctx","queryReimpo
  rtSeesNewContent":true}`
  (verified, measured).
- In-process reuse keeps the module singletons listed in the input research,
  so evaluation stays in child processes (same conclusion as `from-scratch-inputs.md` "What changes for a long-lived
  daemon").
  Node is the runtime file-enforcer already targets (`mise.no-env.toml:1120-1127` per the input research).

### R8: repository fit

- Node is the documented default:
  `README.md:296-299` ("Node.js for TypeScript task and CLI execution"),
  `doc/philosophy/build-execution.md:1-16`,
  `mise.toml:51-54` (`node = "latest"`) (verified, source).
- `AP4` requires `#!/usr/bin/env node` (`AGENTS.md:1403-1407`, verified, source).
  On `v26.8.2` `node:ffi` needs `--experimental-ffi`, which the `AP4` shebang form cannot pass (only `NODE_OPTIONS` or
  `env -S` could);
  on `v26.9.0` it is on by default (release notes, verified), so the pinned-latest policy resolves it once Mise
  installs 26.9.
- Types: `package/config/typescript/tsconfig.options.json:70-72` uses `"types": ["node"]`;
  `@types/node@26.4.1` ships `ffi.d.ts` (verified, measured `ls`),
  but lacks `getCurrentEventLoop` (added `v26.6.0`), which this design does not need (verified, `rg`).
- `module-test` is runtime-neutral (`package/test-harness/core/README.md:7`, `:30-31`, verified, source);
  tests run through the existing Node `test:unit` template.
- `XRT` (`AGENTS.md:955-956`) prefers cross-runtime patterns;
  `node:ffi` code is Node-specific and Linux/glibc-specific,
  which the Linux-only 0.x scope accepts (inference).
- `SCR` (`AGENTS.md:963-967`) and the `sh` coverage in `doc/planning/load-bearing-code-languages.md:11`
  rule out the `sh -c` launcher variant (verified, source).
- `MXL`, `TSD`, `eslint/no-magic-numbers`: FFI struct offsets need named constants and TSDoc (applies to all three
- options).

### R9: documentation quality (culling triggers)

All pages loaded with text using the user's `curl` command (`node SCRATCH/docs-fetch.ts`, HTTP 200, no JS-shell
markers):
`nodejs.org/api/{child_process,fs,net,ffi,process,crypto,os,worker_threads,cli}.html` (verified, measured).
Triggers:

- Contradiction, `https://nodejs.org/api/fs.html#fswatchfilename-options-listener`:
  history says "v19.1.0 Added recursive support for Linux, AIX and IBMi",
  while Availability says "On IBM i systems, this feature is not supported." (verified, doc).
- Confusion, same section:
  `recursive` "applies ... only on supported platforms (See caveats)",
  but the caveats list no recursive support matrix;
  the `ignore` option has no history entry (verified, doc and `doc/api/fs.md:5276-5320`).
- Omission affecting correctness, same section:
  no mention that inotify queue overflow drops events silently (see R2) (verified, doc absence).
- Contradiction across versions, `https://nodejs.org/api/cli.html` (v26.9.0 page):
  `--no-experimental-ffi` "Added in: v26.1.0",
  while the v26.8.2 docs say `--experimental-ffi` was added in v26.1.0 and the module was flag-gated
  (`doc/api/cli.md:1257-1267` in the v26.8.2 checkout);
  neither page records the default flip made in v26.9.0 (verified, doc and release notes).
- Contradiction, `https://nodejs.org/api/net.html#identifying-paths-for-ipc-connections`:
  "It will throw an error when the length of pathname is greater than ... sun_path",
  while `server.listen` emitted an asynchronous `'error'` event with `EINVAL` (verified, measured).

### R10: maturity of what the design needs

- `node:ffi`: core module, Stability 1, introduced in v26.1.0 (before v26.2.0 of 2026-05-20),
  default-on since v26.9.0 (2026-09-16);
  open #64848 "Remove FFI type aliases while it's still experimental" signals API churn
  (`gh search issues --repo nodejs/node --state open 'ffi'`, verified).
- `koffi` as a non-core FFI fallback: `Koromix/koffi` pushed 2026-09-04, 28 open issues (verified, `gh api`).
- `@noble/hashes` only if BLAKE3 is wanted (see R5).
- No other third-party library is required.

### Yikes

Ranked by severity, most severe first.

1. **Every race-free and loss-free path runs through an experimental, memory-unsafe core module.**
   Placement before exec (`pidfd_spawnp` with `SETCGROUP`) and overflow-aware watching both need `node:ffi`,
   Stability 1, four months old, whose flag default flipped in a minor release today.
   A signature mistake crashes the daemon (`doc/api/ffi.md:15-19`).
   The stable-API fallback exists but costs more:
   `systemd-run --scope` per task (at least 7 ms of client and D-Bus work plus the ~10 ms attach lock, full cost
   unmeasured),
   or a Node `process.execve` launcher (43 to 46 ms, itself Stability 1) plus the lock.
2. **The built-in watcher loses events silently on inotify overflow.**
   Measured 0 of 300 later modifications delivered with no error after a 16,384-event flood,
   caused by `deps/uv/src/unix/linux.c:2617-2619`.
   Without the FFI reader, the only backstop is a periodic full rescan (the stat walk alone took 5.3 s).
3. **Recursive `fs.watch` on Linux is unusable at this size.**
   41,205 watches, 662 ms synchronous setup, 243 MiB RSS;
   the daemon must walk and watch directories itself.
4. **No event-loop integration for pidfds.**
   Exit detection relies on a `SIGCHLD` sweep with synchronous `waitid` (measured working),
   not on readiness of the pidfd itself.
5. **Documentation triggers**:
   the IBM i contradiction and missing overflow caveat in `fs.watch`,
   the FFI flag history contradiction,
   and the IPC path "throw" wording.
6. **No built-in BLAKE3 or xxHash**; SHA-256 at 2.3 GiB/s covers the need.

## Option 2: TypeScript on Bun

Two versions matter:
the repo pins `1.3.14` with the comment "Pinned to 1.3.14 because the Rust rewrite's robustness invites serious doubts."
(`mise.toml:30-33`, verified, source),
and `1.4.x` is the Rust rewrite ("it rewrites Bun from Zig to Rust", `https://bun.com/1.4`, verified, doc),
first released as `1.4.0` on 2026-08-20.
The only published docs describe `1.4.x`.

### Design sketch

- On `1.4.2`:
  `Bun.spawn({ cmd, cgroup: taskDirFd, detached: true, stdio: ['ignore', 'pipe', 'pipe'], onExit })`
  for placement before exec, streams, and exit;
  cgroup files written with `node:fs`;
  per-directory `fs.watch` treating `('change', null)` as "rescan";
  `node:net` for RPC (Bun's native `Bun.listen` does not buffer);
  `Bun.hash.xxHash3` as a fast pre-filter and SHA-256 as the cache key.
- On `1.3.14`:
  the same design with `bun:ffi` `pidfd_spawnp` for placement,
  `Bun.file(fd).stream()` for pipes,
  a `SIGCHLD` sweep for exits,
  and an FFI inotify reader because the built-in watcher drops events.
- file-enforcer runs as Node child processes either way.

### R1: per-task cgroup placement, freeze, kill, pidfd

- `1.4.x` adds `cgroup?: string | number` to `Bun.spawn` and `Bun.spawnSync`:
  `clone3(CLONE_VM|CLONE_VFORK|CLONE_INTO_CGROUP)` on cgroup v2, a pre-exec `cgroup.procs` write otherwise,
  `EBUSY` for frozen cgroups
  (PR #37466 merged 2026-08-11, `gh pr view 37466 --repo oven-sh/bun`;
  `SCRATCH/src/bun-1.4.2-spawn_process.rs:354-356`; `SCRATCH/src/bun-1.4.2-bun.d.ts:7440-7452`)
  (verified, source; not probed because 1.4.2 is not installed).
- `1.3.14` ignores the option silently:
  `bun --eval 'Bun.spawn({ cmd: ["/usr/bin/true"], cgroup: "/nonexistent-cgroup-dir" })'` printed
  `bun 1.3.14 spawn with nonexistent cgroup exit 0` (verified, measured).
- `detached` calls `setsid()` in the vfork child (`src/jsc/bindings/bun-spawn.cpp:198-200` in 1.3.14;
  `bun.d.ts:7399-7411` in 1.4.2) (verified, source).
- Per-task overhead on `1.3.14` (three rounds, p50, verified, measured):
  - `Bun.spawn` direct: 0.47 to 0.49 ms.
  - `sh` launcher without the cgroup write: 1.47 to 1.52 ms.
  - Bun launcher with `process.execve` (available, `typeof process.execve` printed `function`): 58.03 to 61.39 ms.
  - `bun:ffi` `pidfd_spawnp` with `SETCGROUP` into the current cgroup: 0.40 to 0.43 ms.
- Pipes from FFI: `bun SCRATCH/stdio-probe.ts` printed `"stdout":["line-1","line-2","line-3"],"maxTimerGapMs":33`
- (verified, measured).
- `bun:ffi` has no asynchronous calls ("Async functions are not yet supported", `docs/runtime/ffi.mdx:223`, verified,
- source);
  exits come from the `SIGCHLD` sweep (measured working).

### R2: recursive watching

- `1.3.14` built-in watcher (`SCRATCH/watch-probe.ts`, verified, measured):
  - 100 distinct files written in one directory: recursive `"sameDirSeen":1`, per-directory `"sameDirSeen":1`.
  - Recursive mode without any burst: `"lateSeen":3` of 300.
  - Per-directory mode without a burst: 300 of 300; with the flood: 299 of 300 and no overflow signal.
  - New nested directory with a file: `"raceSeen":false` in recursive mode.
- Cause: `shouldEmit` emits only when the timestamp differs by more than 1 ms or when both the event type and path hash
- differ
  (`src/runtime/node/path_watcher.zig:128-145`),
  while its comment claims it suppresses only when both match (verified, source).
- Recursive mode has no `ignore` and walks `node_modules`:
  `bun SCRATCH/repo-watch-scale.ts recursive` printed `{"syncMs":1897,"watches":100532,"rssMiB":144}`;
  per-directory printed `{"syncMs":83,"watches":5481}` (verified, measured).
- Subtree `inotify_add_watch` failures, including `ENOSPC`, are swallowed:
  `if (subpath.len > 0) return .success;` (`path_watcher.zig:492`), and unknown wds such as the overflow event are
  skipped
  (`:617`) (verified, source).
- `1.4.2` source fixes the predicate to "same path hash and same event type within 1 ms"
- (`bun-1.4.2-path_watcher.rs:215-250`),
  reports overflow as `('change', null)` (`:950-967`, PR #33110 merged 2026-06-30),
  and surfaces subtree failures as `'error'` (PR #36415 merged 2026-07-31) (verified, source; not probed).
  Node documents that `filename` may be null for other reasons too, so a null filename is an ambiguous rescan signal
  (`doc/api/fs.md` "Filename argument", verified, doc).
- Open issue #34160 reports super-linear `fs.watch` registration cost on macOS (5,000 watchers in 74 s);
  on Linux `1.3.14` registered 5,481 per-directory watchers in 83 ms (verified, measured).

### R3: Unix socket JSON-RPC server

- `Bun.listen({ unix })` writes are unbuffered:
  `bun SCRATCH/socket-probe.ts` printed
  `{"fanOutMs":20164,"fanOutShortWrites":144400,"fanOutReceivedRatio":0.278,"firstShortWriteAfterBytes":233152}`,
  so 72.2 percent of notifications written without handling short writes were lost (verified, measured).
- `node:net` under Bun buffers and signals backpressure:
  `FORCE_NODE_NET=1 bun SCRATCH/socket-probe.ts` printed
  `{"api":"node:net","fanOutMs":173,"firstFalseAfterBytes":262144}` (verified, measured).
- Socket mode 755 under umask 022 (verified, measured).

### R4: btrfs

- `bun SCRATCH/btrfs-probe.ts ...` printed
  `"statfsType":"0x-6edc97c2","statfsIsBtrfs":false` although the filesystem is btrfs:
  `statfs().type` is signed (-0x6edc97c2 is 0x9123683e as unsigned), unlike Node (verified, measured).
- `COPYFILE_FICLONE_FORCE` 35.44 ms, default copy 0.16 ms, `Bun.write(BunFile)` 0.75 ms,
  FFI `FICLONE` 0.10 ms, `GET_SUBVOL_INFO` rc 0 treeid 257;
  `filefrag -v` showed all copies `shared`;
  `FICLONE_FORCE` on tmpfs threw `ENOTSUP` (verified, measured).

### R5: hashing

- `bun SCRATCH/hash-probe.ts` printed `Bun.hash.xxHash3` 53,242 to 54,995 MiB/s (4 ms for the small set),
  `sha256` 2,227 to 2,298 MiB/s,
  and `Bun.CryptoHasher.algorithms` without BLAKE3 (verified, measured).
- `getHashes()` omits `blake2b512` while `createHash('blake2b512')` works (verified, measured): a compat inconsistency.
- `Bun.hash` functions are "optimized for speed of computation over collision-resistance"
- (`https://bun.com/docs/runtime/hashing`, verified, doc);
  as a cache key it needs a collision-resistant companion (inference).

### R6: process supervision

- `Bun.spawn` `onExit`, `exited`, `kill`, `timeout`, `killSignal`, `signal`
- (`https://bun.com/docs/runtime/child-process`, verified, doc).
- Ctrl+C: `process.on('SIGINT')` (Node-compatible) plus `detached` for tasks.
- Workers, needed only if blocking FFI calls must leave the main thread,
  are "still experimental (particularly for terminating workers)" (`https://bun.com/docs/runtime/workers`, verified,
  doc).

### R7: file-enforcer interop

- `bun SCRATCH/fe-import-probe.ts` printed the same result as Node:
  `importOk:true`, 80 exports, `glob`, `AsyncLocalStorage`, and query re-import working (verified, measured).
- `CM3` bans `bun test`, which misreports under `@monochromatic-dev/test-harness-core` (`AGENTS.md:877-881`, verified,
- source);
  Bun-specific code would still be tested through Node tasks or `bun <file>`.

### R8: repository fit

- Bun is allowed only as documented islands:
  `AGENTS.md:1407`, `doc/philosophy/build-execution.md:15-16`, `bunfig.toml:1-2`, `README.md:298-299` (verified,
  source).
- `XRT` prefers cross-runtime patterns over Bun-specific ones (`AGENTS.md:955-956`, verified, source).
- No Bun type package is installed (`ls node_modules/.pnpm` shows only `@types/node`, verified, measured);
  the shared tsconfig lists only `node` types.
- The pin comment (`mise.toml:32`) conflicts with the features this design needs, which exist only in the Rust line.

### R9: documentation quality (culling triggers)

All fetched pages loaded with text
(`bun.com/docs/runtime/{child-process,ffi,networking/tcp,hashing,file-io,nodejs-compat,workers}`,
`bun.com/guides/read-file/watch`, `bun.com/1.4`) (verified, measured).
Triggers:

- Unversioned docs contradict the pinned runtime:
  `https://bun.com/docs/runtime/child-process` documents `cgroup` ("the spawn fails if the child cannot join the
  cgroup"),
  while 1.3.14 ignores it (verified, doc and measured).
- Self-contradiction, `https://bun.com/docs/runtime/nodejs-compat`:
  "node:net 🟢 Fully implemented" in the same entry that says
  "new net.Socket({ fd }) cannot read from an existing file descriptor" (verified, doc).
- Claim contradicted for the pinned version, same page:
  "node:fs 🟢 Fully implemented. 98% of Node.js's test suite passes",
  while 1.3.14 `fs.watch` delivered 1 of 100 same-directory writes (verified, doc and measured).
- Misleading framing, `https://bun.com/docs/runtime/networking/tcp`:
  "TCP sockets in Bun do not buffer data, so performance-sensitive code should buffer writes itself",
  while unhandled short writes lose data (72.2 percent in the probe) (verified, doc and measured).
- Omission, `https://bun.com/docs/runtime/child-process`:
  the page never mentions `detached` (`rg --count detached` found 0 matches),
  although `bun.d.ts:7399-7411` documents it (verified, doc and source).
- Stated immaturity: `https://bun.com/docs/runtime/ffi` "bun:ffi is experimental, with known bugs and limitations. Do
- not rely on it in production."
- Source comment contradicting code in 1.3.14 (`path_watcher.zig:120-136`) (verified, source).

### R10: maturity and maintenance

- Bun 1.4.0 (2026-08-20) is a whole-runtime port from Zig to Rust;
  1.4.1 followed on 2026-09-04 and 1.4.2 on 2026-09-05 (verified, `gh api repos/oven-sh/bun/releases`).
- Issue intake:
  506 issues created 2026-08-20 to 2026-09-15 versus 298 in the preceding 27 days
  (`gh api 'search/issues?q=repo:oven-sh/bun+is:issue+created:...'`, verified, measured);
  adoption growth may explain part of it (unverified).
- `cgroup` spawn option: merged 2026-08-11, about five weeks old.
- `bun:ffi` and Workers: experimental per docs.

### Yikes

Ranked by severity, most severe first.

1. **Version trap with correctness loss on either side.**
   The pinned 1.3.14 silently ignores `cgroup` (sandbox off without an error)
   and its `fs.watch` delivers 1 of 100 distinct same-directory writes;
   the fixes and native placement exist only in the Rust rewrite the repository explicitly distrusts,
   released four weeks ago, and none of those fixes were probed here.
2. **On 1.3.14 every workaround stacks experimental pieces.**
   Placement and watching need `bun:ffi` ("Do not rely on it in production"),
   there are no asynchronous FFI calls,
   and moving blocking calls off-thread needs Workers, also experimental.
3. **Repository policy conflict.**
   A long-running daemon as a Bun island contradicts `AP4`, `XRT`, and `build-execution.md`,
   and `CM3` already bans `bun test` after a test-harness misreport under Bun.
4. **Native socket API loses data unless the caller buffers**
   (72.2 percent lost in the fan-out probe); `node:net` must be used instead.
5. **Watcher scale and signal ambiguity even on 1.4.2.**
   No `ignore` (100,532 watches recursively), and overflow arrives as a null filename that also has other meanings.
6. **Node-compat drift**:
   signed `statfs().type`, `getHashes()` omitting working algorithms, and documentation that describes a different
   version.

## Option 3: TypeScript on Deno

### Design sketch

- Run as `deno run -A` (required, see R8 and R9).
- Spawn: `Deno.dlopen('libc.so.6')` binding `pidfd_spawnp` with `SETCGROUP`,
  pipes from `pipe2` with `O_NONBLOCK`,
  a single `nonblocking: true` FFI `poll(2)` call over every task's pipe fds plus an `eventfd` for wakeups,
  synchronous FFI `read` on ready fds,
  and exit detection through `Deno.addSignalListener('SIGCHLD')` plus synchronous `waitid(..., WNOHANG)`.
  One blocking-pool thread total, because pending nonblocking calls starve async file I/O (see R6).
- Watching: FFI inotify instance whose fd joins the same `poll` loop.
- RPC: `Deno.listen({ transport: 'unix', path })` or `node:net`.
- btrfs: FFI `open` plus `ioctl` for `FICLONE`, `GET_SUBVOL_INFO`, and subvolume creation
  (Deno does not expose OS file descriptors).
- Hashing: `node:crypto` SHA-256.
- file-enforcer: Node child processes.

### R1: per-task cgroup placement, freeze, kill, pidfd

- `Deno.CommandOptions` has `args`, `cwd`, `clearEnv`, `env`, `uid`, `gid`, `signal`, `stdin`, `stdout`, `stderr`,
  `windowsRawArguments`, `detached`, and no cgroup or pre-exec hook
  (`cli/tsc/dts/lib.deno.ns.d.ts:4140-4211`, verified, source).
  `detached` calls `setsid()` in `pre_exec` (`ext/process/lib.rs:779-783`, verified, source).
- No `execve`: `deno eval 'import process from "node:process"; console.log(typeof process.execve)'` printed `undefined`
  (verified, measured),
  so a Deno launcher must spawn and wait, doubling processes per task.
- Per-task overhead (three rounds, p50, verified, measured):
  - `Deno.Command(...).spawn()` direct: 1.05 to 1.06 ms.
  - `sh` launcher without the cgroup write: 2.05 to 2.14 ms.
  - Deno launcher that spawns and waits: 22.90 to 23.86 ms.
  - FFI `pidfd_spawnp` with `SETCGROUP` into the current cgroup and a nonblocking `waitid`: 0.43 to 0.46 ms.
- `Deno.dlopen` is in the stable namespace (`runtime/js/90_deno_ns.js:264-268`) without an experimental tag
  (`lib.deno.ns.d.ts:6780-6821`) (verified, source).
- FFI pipes and exit: `SCRATCH/stdio-probe.ts` printed
- `"exitStatus":7,"stdout":["line-1","line-2","line-3"],"maxTimerGapMs":26`
  (verified, measured).

### R2: recursive watching

- Built-in `Deno.watchFs` uses one shared notify `RecommendedWatcher` per runtime (`runtime/ops/fs_events.rs:378-475`),
  applies `ignore` only as an event filter after watching (`:412-432`),
  and matches every event against every watched path with `canonicalize` and `same_file::is_same_file`
  (`:250-270`) (verified, source).
  notify follows symlinks by default (`notify/src/config.rs:103-123` at `notify-8.2.0`, verified, source).
- Recursive watch of the checkout with the four exclusions as `ignore`:
  `deno run -A SCRATCH/repo-watch-scale.ts recursive` printed
  `{"syncMs":131294,"watches":100532,"rssMiB":381}`:
  131 seconds blocking the JS thread and every `node_modules` directory watched (verified, measured).
- Many non-recursive watches: event matching cost grows with watched paths.
  `SCRATCH/deno-watch-latency.ts` (10 writes, time until all seen, verified, measured):
  10 paths 11 ms; 1,000 paths in one call 121 ms; 5,480 paths in one call 657 ms; 5,480 separate calls 3,536 ms.
  The `node:fs.watch` polyfill over 401 directories took 9,542 ms to deliver 300 writes (verified, measured).
- Overflow is signaled: the probe after the flood printed `"lateSeen":22,"rescanEvents":1` (verified, measured);
  the queue capacity is 1,024 (`fs_events.rs:53`) and kernel overflow maps to `Flag::Rescan`
  (`notify inotify.rs:212-213`) (verified, source).
- New-directory race: notify adds a watch on `IN_CREATE` of a directory without scanning it
  (`inotify.rs:61-75`, `:279-290`), and the probe printed `"raceSeen":false` (verified, source and measured).
- Open issues: #18348 "`Deno.watchFs` with `recursive: false` still gives events for children" (open since 2023),
  #27559 grouped events with removed files, #27560 symlink events
  (`gh search issues --repo denoland/deno --state open 'watchFs'`, verified).
- FFI inotify reader works and detects overflow (see "Shared evidence").

### R3: Unix socket JSON-RPC server

- `deno run -A SCRATCH/socket-probe.ts` printed
  `{"socketMode":"755","fanOutMs":457,"bytesWrittenAfter1sWithoutReader":233152}`:
  `conn.write()` stays pending while the peer does not read (natural backpressure) (verified, measured).
- `node:net` under Deno printed `{"fanOutMs":284,"firstFalseAfterBytes":262144}` (verified, measured).
- Unix listen needs `allow-read`, `allow-write`, and `allow-net` (`lib.deno_net.d.ts:280-297`, verified, source).

### R4: btrfs

- `deno run -A SCRATCH/btrfs-probe.ts ...` printed `"statfsType":"0x9123683e"`, `COPYFILE_FICLONE_FORCE` 44.04 ms,
  `Deno.copyFileSync` 0.20 ms, FFI `FICLONE` 0.03 ms, `GET_SUBVOL_INFO` rc 0 treeid 257;
  `filefrag -v` showed all copies `shared` (verified, measured).
- `COPYFILE_FICLONE_FORCE` is ignored:
  on tmpfs `deno ... SCRATCH/ficlone-negative.ts` printed `FICLONE_FORCE on tmpfs: no error; destination exists = true`
  (verified, measured);
  the polyfill checks only `COPYFILE_EXCL` (`ext/node/polyfills/_fs/_fs_copy.ts:48-104`, verified, source).
  "Must be a reflink" therefore needs FFI `FICLONE`.
- OS fds are hidden, so fd-based ioctls need FFI `open` (probe used `lib.symbols.open`, verified, measured).

### R5: hashing

- `deno run -A SCRATCH/hash-probe.ts` printed `node:crypto sha256` 2,233 to 2,300 MiB/s,
  `crypto.subtle SHA-256` 1,082 MiB/s,
  and `crypto.subtle BLAKE3` "NotSupportedError: Algorithm 'BLAKE3' is not supported" (verified, measured).

### R6: process supervision

- Nonblocking FFI calls run on the tokio blocking pool (`ext/ffi/call.rs:353`),
  capped at 32 threads on Linux (`runtime/tokio_util.rs:56-64`) (verified, source).
- Starvation measured with `SCRATCH/deno-blocking-pool.ts`:
  8 pending nonblocking `waitid` calls left an async `Deno.readTextFile` at 0.23 ms,
  while 40 pending calls stalled it for 1,891.46 ms until the children exited (verified, measured).
  A naive design (one `waitid` plus two pipe reads per task, 16 concurrent tasks, frozen tasks keeping theirs)
  exhausts the pool and stalls hashing and file I/O (inference from the measurement).
- `Deno.addSignalListener('SIGCHLD')` fires for FFI children (verified, measured), so exits need no blocking thread.
- Ctrl+C: `Deno.addSignalListener('SIGINT')`.

### R7: file-enforcer interop

- With `--no-config`, import failed:
  `Import "@monochromatic-dev/logger-core/ts" not a dependency` (verified, measured).
- From the repository root with `deno run --no-lock --node-modules-dir=manual -A SCRATCH/fe-import-probe.ts`,
  the import succeeded with 80 exports and wrote no `deno.lock` or `node_modules/.deno` (verified, measured).
- Prior repository experience: `doc/troubleshooting/browserslist-deno-root-read.md` records a Deno read prompt for `/`
  while running the root config (verified, source).

### R8: repository fit

- Reading `/proc` or `/sys` needs `--allow-all`:
  `deno run --allow-read SCRATCH/deno-proc-read.ts` printed
  `NotCapable: Requires all access to "/proc/self/cgroup", run again with the --allow-all flag`
  and the same for `/sys/fs/cgroup/cgroup.controllers` (verified, measured);
  source: any path under `/dev`, `/proc`, or `/sys` except `/proc/pressure/` and pipe fds calls
  `check_has_all_permissions`
  (`runtime/permissions/lib.rs:4614-4628`, verified, source).
  The daemon and `doctor` must run with `-A`, so Deno's sandbox brings no protection here.
- No Deno runtime precedent for tools; the shared tsconfig lists only `node` types;
  `AP4` has no Deno shebang allowance; `Deno.*` APIs are runtime-specific under `XRT` (verified, source).
- `module-test` is runtime-neutral (`package/test-harness/core/README.md:7`, `:30-31`, verified, source).

### R9: documentation quality (culling triggers)

All fetched pages loaded with text
(`docs.deno.com/api/deno/~/Deno.{watchFs,Command,CommandOptions,listen,dlopen,ForeignFunction}`,
`docs.deno.com/runtime/fundamentals/{ffi,security,node}/`, `docs.deno.com/runtime/reference/permissions/`,
`docs.deno.com/api/node/fs/~/copyFile`) (verified, measured).
Triggers:

- Contradiction, `https://docs.deno.com/runtime/reference/permissions/`:
  "Reading or writing through symlinks that resolve to these paths requires --allow-all" for `/proc`, `/dev`, `/sys`,
  while direct reads also require it (measured and source, see R8).
- Contradiction, `https://docs.deno.com/api/node/fs/~/copyFile`:
  "COPYFILE_FICLONE_FORCE ... If the platform does not support copy-on-write, then the operation will fail",
  while Deno copies silently (measured, see R4).
- Contradiction, `https://docs.deno.com/api/deno/~/Deno.watchFs`:
  "for directories, will watch the specified directory and all sub directories" for recursive mode,
  while `recursive: false` watchers still receive child events (open #18348; matching by path prefix at
  `fs_events.rs:250-270`).
- Confusion, same page:
  the `ignore` example comment "skip the `.git` directory and `build` output" suggests those directories are not
  watched,
  while watches are still added (measured 100,532 watches with `ignore`).
- Naming contradiction, `https://docs.deno.com/api/deno/~/Deno.dlopen` (`UnsafeCallback` text):
  "use deref() to then allow Deno's process to exit" next to the methods `ref()` and `unref()` (verified, doc).
  The design avoids callbacks.

### R10: maturity and maintenance

- Deno `2.9.6` published 2026-08-27; FFI stable in the namespace.
- notify (watch backend): `notify-9.0.0-rc.5` published 2026-08-30, 91 open issues (verified, `gh api`);
  the design replaces it with FFI inotify.
- No third-party library required.

### Yikes

Ranked by severity, most severe first.

1. **The built-in watcher cannot serve this repository.**
   Recursive mode cannot prune `node_modules` (100,532 watches) and blocked the JS thread for 131 seconds;
   many directory watches cost 657 ms per 10 writes at 5,480 directories.
   A hand-written FFI inotify reader is mandatory, not optional.
2. **Asynchronous FFI starves the runtime's own file I/O.**
   40 pending nonblocking calls stalled `Deno.readTextFile` for 1.9 seconds (32-thread cap).
   The daemon must hand-write a `poll(2)` reactor over FFI for task pipes and inotify,
   in addition to FFI spawn and FFI ioctls.
3. **Most platform work lives outside Deno's API surface.**
   No cgroup or pre-exec spawn hook, no `execve`, no fd access, and `FICLONE_FORCE` silently ignored,
   so spawn, streams, watching, and btrfs all go through raw libc bindings.
4. **The permission model is defeated.**
   `/proc` and `/sys` access requires `-A`, contrary to the documented symlink-only rule.
5. **Repository fit.**
   No Deno precedent for tools, Node-only type configuration, and file-enforcer resolution that depends on the working
   directory.
6. **Documentation contradictions** on permissions, `copyFile` flags, and `watchFs` recursion.

## Cross-option comparison

Node has the fewest and least severe yikes.

- All three reach the same best-case primitives through libc FFI:
  `pidfd_spawnp` with `CLONE_INTO_CGROUP` at about 0.4 ms per spawn,
  an inotify reader that detects overflow,
  `SIGCHLD`-driven exit handling,
  and reflink copies that share extents (all measured).
  The differences are in what fails without FFI, what FFI costs, and how each option fits the repository.
- Node:
  - Its worst yike is dependence on `node:ffi` (experimental, memory-unsafe).
    That module is part of the core runtime, is on by default from v26.9.0,
    and every FFI piece the design needs was measured working on the installed v26.8.2 without extra threads or
    reactors.
  - When FFI is set aside, a stable path still exists:
    `systemd-run --scope` per task, which the user ruled not disqualifying,
    plus per-directory `fs.watch` with a periodic rescan.
    That path is slower and loses overflow detection, but it is safe.
  - Node is the repository's default runtime:
    its types include `ffi.d.ts`, file-enforcer runs in-process or as children unchanged,
    and no Bun-island or Deno exception is needed.
  - Its remaining yikes are a documented engineering cost (walk directories manually)
    or documentation triggers of moderate weight.
- Bun ranks below Node:
  - Its top yike is a real version trap:
    the pinned 1.3.14 measurably loses watch events and silently drops the sandbox option,
    while the fixed and native-cgroup 1.4.x is a four-week-old Rust port the repository explicitly distrusts.
  - Choosing either version violates a stated position or ships known data loss,
    and the workaround stack on 1.3.14 is built on components Bun itself labels "Do not rely on it in production".
  - A policy conflict with the Bun-island rule comes on top.
- Deno ranks last on severity:
  - Its FFI is stable, but three high-severity problems stack up:
    - the built-in watcher is unusable at repository scale;
    - asynchronous FFI starves the runtime's file I/O, forcing a custom `poll` reactor;
    - spawn, streams, watching, and btrfs all bypass Deno's APIs.
  - On top of that, the permission sandbox has to be disabled,
    and the repository has no Deno tooling precedent.
  - Deno needs the most hand-written native-boundary code of the three.

Ranking on disqualifying severity: Node > Bun > Deno.

- Node over Bun:
  Node's worst problem is that its fast path is experimental but works and has a safe fallback.
  Bun's worst problem is that neither available version is acceptable without breaking a stated constraint or shipping
  known loss.
- Bun over Deno:
  Bun 1.4.x would give native cgroup placement without FFI if the user lifts the pin.
  Deno has no native route at all,
  and its FFI route also needs a hand-written reactor to avoid starving its own I/O.

Open question for the user: accept the Node design with `node:ffi` as the primary placement and watching path
(with the `systemd-run --scope` and rescan fallback),
or require stable-only APIs and accept the fallback's per-task and rescan costs.
