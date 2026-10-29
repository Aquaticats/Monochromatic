# Daemon stack deep dive: languages beyond TypeScript, Rust, Kotlin, Go, and Zig

Outcome on 2026-09-16:
the user judged Python,
OCaml,
and .NET not worth further research.
This file is kept as evidence.

Date: 2026-09-16.
Scope: read-only design research for the daemon in
`doc/planning/monorepo-manager-from-scratch-design.md`.
Nothing under the repository was modified;
nothing was installed;
no cgroup was created.

## Conventions

- **verified**:
   read in the cited source or page during this session,
   or measured on the development machine.
- **unverified**:
   recall or inference;
   the reason is stated.
- `SCRATCH` means
   `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad`.
  Saved documentation HTML is in `SCRATCH/others/docs/`,
   extracted text in `SCRATCH/others/text/`,
   probe scripts in `SCRATCH/others/`.
- Source clones (shallow, some sparse) under `~/temp/agent/`:
  - `dotnet/runtime` at `8970fe8a2fb3` (2026-09-16)
  - `ocaml/ocaml` at `8102ebbfa2a3` (2026-09-15)
  - `ocaml-multicore/eio` at `2ae74d1cceb3` (2026-09-11)
  - `whitequark/ocaml-inotify` at `f36de4316f8b` (2024-07-26)
  - `ocaml/dune` at `215adfd0ee1a` (2026-09-16)
  - `python/cpython` at `d8a107249106` (2026-09-16)
  - glibc GitHub mirror `bminor/glibc` at `04e750e75b73` (2026-01-20)
- Documentation gate (R9), per `doc/audit/tech-monorepo-manager-vet-2026-09-16.md:289-321`:
   each relied-on page fetched once with
   `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0'`;
   status,
   byte count,
   extracted text length,
   an expected keyword,
   and challenge markers recorded in `SCRATCH/others/probe-p1.tsv`,
   `probe-p2.tsv`,
   and `probe-p3.tsv` (driver `SCRATCH/others/docprobe.ts`).
  Any confusion or contradiction found while reading is recorded as a trigger with its URL.
- Library maintenance metadata came from `gh api repos/<owner>/<repo>` (plus `releases/latest`, `tags`, `commits`) on
   2026-09-16
   (`SCRATCH/others/ghmeta-*.tsv`, verified).

## Cross-cutting findings

These apply to every option in this document,
and some apply to the options other agents cover.

### X1: glibc has race-free cgroup placement for spawn, documented only in NEWS and a header

- glibc 2.39 added `posix_spawnattr_setcgroup_np`,
   `POSIX_SPAWN_SETCGROUP`,
   and `pidfd_spawn`/`pidfd_spawnp`
   (`NEWS:815` "Version 2.39",
   `NEWS:831-840`,
   verified).
- The host has glibc 2.43 (`ldd --version`, measured);
   declarations at `/usr/include/bits/spawn_ext.h:27-63`
   and `POSIX_SPAWN_SETCGROUP 0x100` at `/usr/include/spawn.h:63` (verified).
  The header comment reads "Sore the cgroupsv2 the attribute structure" (`spawn_ext.h:33`, verified).
- Implementation:
   `sysdeps/unix/sysv/linux/spawni.c:397-437` passes `CLONE_INTO_CGROUP | CLONE_PIDFD | CLONE_VM | CLONE_VFORK`
   to `clone3` and has no fallback for `POSIX_SPAWN_SETCGROUP` when `clone3` is unavailable,
   returning `ENOTSUP` (verified).
- No code runs in the child besides glibc's own helper,
   so the caller's runtime never executes managed or OCaml code after the fork.
  This is what makes the call usable from garbage-collected runtimes.
- Documentation:
   the glibc manual mentions `pidfd_spawn` once in passing (`manual/process.texi:141`, verified)
   and documents neither function;
   `https://man7.org/linux/man-pages/man3/pidfd_spawn.3.html` returns 404 (measured);
   `man --where posix_spawnattr_setcgroup_np pidfd_spawn` finds no entry (measured).
  The kernel side is documented:
   `CLONE_INTO_CGROUP` "(since Linux 5.7)",
   fd opened "using either the O_RDONLY or the O_PATH flag",
   `EBUSY` when the target cgroup has a domain controller enabled
   (<https://man7.org/linux/man-pages/man2/clone.2.html>, verified).
  Under the vet's rule that "a feature named only in a changelog, issue, blog post,
  or source code counts as undocumented",
   any design whose primary cgroup placement is this glibc call carries an R9 trigger.
- Probe from Python `ctypes` (`SCRATCH/others/spawn_probe.py`, measured):
   `pidfd_spawn` with `POSIX_SPAWN_SETCGROUP | POSIX_SPAWN_SETPGROUP`,
   a 336-byte attribute buffer,
   and an fd for the cgroup this shell already occupies returned 0;
   the child reported the expected cgroup and its own process group;
   `os.waitid(os.P_PIDFD, ...)` returned its exit.
  Control (`SCRATCH/others/spawn_probe_control.py`):
   the same call with an fd for a non-cgroup directory returned `EBADF`,
   so the flag reaches the kernel.
  Unverified:
   placement into a different,
   delegated cgroup,
   because creating one was out of scope.
- Runtime requirement:
   glibc 2.39 or newer on every machine,
   including CI runners (runner glibc versions unverified).

### X2: self-exec trampoline uses only documented APIs

- The daemon spawns its own executable as
   `<daemon> __enter-cgroup <cgroup dir> -- <argv>`.
  The trampoline writes its pid to `<cgroup dir>/cgroup.procs`,
   then `execve`s the task command.
- No task code runs before placement,
   exec keeps the pid,
   so the parent's normal child tracking still applies.
- Cost:
   one extra executable start per task.
  Measured for Python only (see P3 R1);
   unmeasured for .NET NativeAOT and OCaml because their toolchains are not installed and installing was out of scope.
- Rejected variant:
   a `sh -c 'echo $$ > cgroup.procs && exec "$@"'` wrapper is repository logic in an unapproved language
   (`doc/planning/load-bearing-code-languages.md:84-94`).

### X3: `systemd-run --user --scope` per task

- Documented placement path for any language
   (`SRC/systemd/systemd-run.xml:76-81`, as cited in `from-scratch-inputs.md:705-707`).
- The resulting scope is created by systemd,
   and `CGROUP_DELEGATION.md:386-397` forbids writing attributes of cgroups systemd created,
   so pause and end would go through systemd instead of `cgroup.freeze` and `cgroup.kill`
   (the `systemctl --user freeze` and `kill` pages were not read in this session; unverified).

### X4: Ctrl+C and process groups

- The terminal delivers `SIGINT` to its foreground process group,
   so tasks must start in their own process group
   (`POSIX_SPAWN_SETPGROUP` with pgid 0 in X1,
   `setpgid` in X2).
- Background process groups that read the terminal receive `SIGTTIN`
   (setpgid(2), cited in `from-scratch-inputs.md:660-663`),
   so task stdin is `/dev/null`.

### X5: watcher algorithm shared by every option

- One inotify instance;
   one watch per non-excluded directory;
   exclusions (`node_modules`, `.git`, `target`, `dist`) checked during the initial walk
   and on each `IN_CREATE | IN_ISDIR` or `IN_MOVED_TO | IN_ISDIR`.
- New-directory race:
   after adding a watch for a new directory,
   scan it and treat every entry found as changed,
   because events before the watch existed are lost (inotify(7), cited in `from-scratch-inputs.md:545-547`).
- Overflow:
   `IN_Q_OVERFLOW` arrives with wd -1;
   the daemon marks everything dirty,
   rewalks,
   and lets content hashes decide what changed
   (Watchman fresh-instance precedent, `from-scratch-inputs.md:552-553`).
- Measured limits:
   `max_queued_events` 16384,
   `max_user_watches` 524288,
   `max_user_instances` 8192;
   5,480 directories after exclusions (`from-scratch-inputs.md:537-538`).
  A slow reader overflows at 16,384 queued events,
   so the reader must never share a thread with CPU-bound work.

### X6: cgroup and btrfs kernel interfaces

- `cgroup.events` carries `populated` and `frozen`;
   "a value change in this file generates a file modified event"
   (<https://docs.kernel.org/admin-guide/cgroup-v2.html>, sections "cgroup.events" and "[Un]populated Notification",
verified).
  Every option can wait for freeze completion and task-tree exit through its inotify path.
- btrfs UAPI (verified on host headers):
   `BTRFS_IOC_SUBVOL_CREATE` (`/usr/include/linux/btrfs.h:1136`),
   `BTRFS_IOC_SNAP_CREATE_V2` (`:1153`),
   `BTRFS_IOC_GET_SUBVOL_INFO` (`:1213`),
   `BTRFS_IOC_SNAP_DESTROY_V2` (`:1219`),
   `struct btrfs_ioctl_vol_args` with a 4088-byte name (`:36-39`),
   `BTRFS_SUBVOL_RDONLY` (`:47`),
   `BTRFS_SUPER_MAGIC 0x9123683E` (`/usr/include/linux/magic.h:28`).
- Privilege matrix unchanged from `from-scratch-inputs.md:586-639`.

## Part 1: plausibility screen

### Plausible (kept)

- **C# on .NET 10 with NativeAOT.**
  Evidence:
   the relied-on Microsoft Learn API pages load with plain `curl` with text present
   (`probe-p1.tsv`),
   the runtime ships `UnixDomainSocketEndPoint`,
   `PosixSignalRegistration`,
   Unix file modes,
   and `System.IO.Hashing`,
   and NativeAOT supports P/Invoke to libc
   (<https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/interop>, verified).
- **OCaml 5 with Eio.**
  Evidence:
   Dune,
   an OCaml build system,
   already implements a Linux inotify watcher with queue-overflow handling (`src/dune_scheduler/inotify.ml:136`, `:150`)
   and an RPC server over Unix-domain sockets (`src/rpc/csexp_rpc.ml:27`),
   and Eio 1.5 (2026-08-19) provides pidfd-based process management and Unix-socket listening (verified).
- **Python 3.14 (CPython, standard library first).**
  Evidence:
   the standard library documents `fcntl.FICLONE` (3.12),
   `os.pidfd_open`,
   `os.waitid` with `P_PIDFD`,
   `loop.add_reader`,
   and `asyncio.start_unix_server`
   (docs.python.org pages in `probe-p1.tsv`/`probe-p2.tsv`, verified),
   and `ctypes` reached glibc's cgroup spawn on this host (X1 probe).

### Excluded

- **Swift on Linux.**
  The relied-on documentation fails the plain-curl gate:
   <https://developer.apple.com/documentation/foundation/process> returns a JavaScript shell
   ("requires JavaScript", 260 characters of text, keyword absent),
   and <https://swiftpackageindex.com/swiftlang/swift-subprocess/documentation/subprocess>
   plus the SwiftNIO `NIOPosix` page return HTTP 403 with a Cloudflare "Just a moment..." challenge
   (`probe-p1.tsv`, measured).
- **Java without Kotlin.**
  It runs on the same JVM with the same `ProcessBuilder`,
   `WatchService`,
   and FFM APIs as the Kotlin/JVM option another agent covers,
   so it adds no capability,
   while the repository has 65 Kotlin files,
   a detekt rule set at `package/linter/kotlin/detekt.yml`,
   and no Java files (measured with `rg --files`).
- **C++.**
  It reaches the kernel through the same libc calls as Rust,
   which additionally has `CommandExt::pre_exec` in its standard library,
   while the repository has one C++ file (`package/file-manager/variant-qt/src/qt_log.cpp`)
   and Rust lint infrastructure (`clippy.toml`, `package/linter/rust`);
   separately,
   the language's official documentation is a purchased ISO standard
   ("You can purchase the official standard at the ISO Store",
   <https://isocpp.org/std/the-standard>, verified),
   so consumed behavior would come from non-official references.
- **Haskell.**
  IO sequencing needs `do`-notation or monadic binds,
   which the repository's non-TypeScript skill forbids by default ("point-free chains, monad stacks, `do`-notation",
   `.claude/skills/dum-dum-non-ts/SKILL.md:65-67`, verified),
   and the `process` package offers process-group and session flags but no pre-exec hook
   (<https://hackage.haskell.org/package/process/docs/System-Process.html>, verified).
- **Nim.**
  The maintained JSON-RPC library is badged "Stability:
  experimental" and "is designed to automatically generate marshalling and parameter checking code"
   (`status-im/nim-json-rpc` README, verified),
   which conflicts with the skill's default ban on macros and code generation (`SKILL.md:58-61`),
   and `std/osproc` has no pre-exec hook among its `ProcessOption` values
   (<https://nim-lang.org/docs/osproc.html>, verified).
- **Crystal.**
  "A Crystal program by default executes a single fiber at a time, thus concurrent only, while parallelism is opt-in"
   (<https://crystal-lang.org/reference/1.21/guides/concurrency.html>, verified),
   so content hashing would stall the watcher and RPC fibers unless opt-in parallelism is adopted,
   and its `Process` API has `exec` but no pre-exec hook (API page, verified).
- **Elixir and Gleam (BEAM).**
  The `erlang` module reference contains no `execve`,
   `ioctl`,
   or `inotify`
   (absence on <https://www.erlang.org/doc/apps/erts/erlang.html>, verified),
   and Elixir's `file_system` package delegates Linux watching to the external `inotify-tools` program
   (<https://hexdocs.pm/file_system/readme.html>, verified),
   so cgroup placement,
   watching,
   and reflinks would all live in native code or external programs,
   making it a two-language design with BEAM as orchestrator only.

## Option P1: C# on .NET 10 NativeAOT

### Shape

- One NativeAOT executable with `daemon`,
   `doctor`,
   and client subcommands.
- Scheduler state owned by one logical actor fed by a `System.Threading.Channels` channel
   (<https://learn.microsoft.com/en-us/dotnet/core/extensions/channels>, loaded).
- Dedicated OS threads for the inotify reader and for a `poll(2)` loop over task pidfds,
   because the runtime has no public API to await readiness of an arbitrary file descriptor.
  The runtime's own `FileSystemWatcher` does the same,
   running "a dedicated thread" over a shared inotify instance
   (`src/libraries/System.IO.FileSystem.Watcher/src/System/IO/FileSystemWatcher.Linux.cs:39-43`, verified).
- No third-party NuGet packages.

### R1: cgroup placement, freeze, kill, limits

- The public `Process` class cannot place a child:
   `SystemNative_ForkAndExecProcess` calls `vfork` or `fork`,
   optionally `setsid` when detached,
   then `execve`,
   with no caller hook
   (`src/native/libs/System.Native/pal_process.c:815-830`, `:884`, `:936-937`, verified).
- Primary design (X1):
  - Startup:
     read `/proc/self/cgroup`,
     check delegation,
     create `<root>/daemon`,
     move the daemon there,
     enable `+cpu +memory +pids +io` in `<root>/cgroup.subtree_control`.
  - Per task:
     create `<root>/task-<id>` with `Directory.CreateDirectory`,
     write `memory.max`,
     `pids.max`,
     optional `cpu.max`,
     and `memory.oom.group`,
     open the directory with a P/Invoke to `open(O_RDONLY | O_DIRECTORY | O_CLOEXEC)`,
     then P/Invoke `posix_spawnattr_init`,
     `posix_spawnattr_setflags(SETCGROUP | SETPGROUP | SETSIGMASK | SETSIGDEF)`,
     `posix_spawnattr_setcgroup_np`,
     `posix_spawn_file_actions_addopen` (stdin `/dev/null`),
     `adddup2` (stdout and stderr pipes),
     `posix_spawn_file_actions_addchdir_np`,
     `addclosefrom_np` (`/usr/include/spawn.h:206-228`, verified),
     and `pidfd_spawnp`.
  - `argv`,
     `envp`,
     the 336-byte attribute block (`spawn.h:29-39`),
     and the 80-byte file-actions block (`spawn.h:44-50`)
     live in `NativeMemory` buffers,
     because the `CLONE_VM` child reads them while other managed threads keep running.
- Reaping interaction,
   verified in source:
   .NET reaps only children started by `Process`
   ("By default we only reap managed processes started using the 'Process' class. This allows other code
to start processes without .NET reaping them",
   `pal_signal.c:376-377`),
   except when the process is pid 1 or `SIGCHLD` was originally ignored,
   where it calls `waitpid(-1)` (`pal_signal.c:384`; `ProcessWaitState.Unix.cs:688-706`).
  In a container where the daemon is pid 1,
   the runtime would steal exit statuses from `pidfd` waits.
  A foreign exited child also makes every `SIGCHLD` fall back to checking all `Process` instances
  (`ProcessWaitState.Unix.cs:622-653`).
- Fallback (X2):
   `Process.Start(self, "__enter-cgroup", dir, "--", ...)`,
   trampoline writes its pid to `cgroup.procs` and P/Invokes `execve`;
   `Process` then tracks the same pid after exec.
  Startup cost of a NativeAOT trampoline:
   unmeasured.
- Freeze,
   thaw,
   kill:
   `File.WriteAllText` to `cgroup.freeze` and `cgroup.kill`;
   completion observed through `cgroup.events` (X6).

### R2: watching

- `FileSystemWatcher` is unusable for this repository:
   with `IncludeSubdirectories` it recurses into every child directory with no exclusion hook
   (`FileSystemWatcher.Linux.cs:1176-1218`, verified),
   so it would add a watch for each of the 99,416 directories that exist when only `.git` is excluded.
  On overflow it stops the shared inotify instance and restarts watchers,
   rewalking the tree (`:536-548`, `:1147-1154`, verified).
  Its API page says nothing about Linux behavior
   (<https://learn.microsoft.com/en-us/dotnet/api/system.io.filesystemwatcher?view=net-10.0>, no "Linux"
or "inotify" text, verified).
- Design:
   P/Invoke `inotify_init1`,
   `inotify_add_watch`,
   and `read` into a `NativeMemory` buffer on a dedicated thread,
   parse `struct inotify_event` by hand,
   apply X5.
  The runtime's `Interop.Sys.INotifyAddWatch` is internal,
   so the daemon declares its own.

### R3: RPC

- `Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified)` bound to a `UnixDomainSocketEndPoint`
   (<https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.unixdomainsocketendpoint?view=net-10.0>, loaded)
   inside a directory created with `Directory.CreateDirectory(path, UnixFileMode)`
   (`src/libraries/System.Private.CoreLib/src/System/IO/Directory.cs:53`, verified).
- Peer uid check:
   `Socket.GetRawSocketOption(1, 17, span)` (`Socket.cs:2298`, verified;
   `SOL_SOCKET 1`, `SO_PEERCRED 17` at `/usr/include/asm-generic/socket.h:9`, `:30`).
- Framing:
   split on `\n`,
   parse with `JsonDocument`/`JsonNode`
   (<https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/use-dom>, loaded),
   write with `Utf8JsonWriter`.
  `JsonSerializer` and StreamJsonRpc are avoided because the skill forbids reflection
  and code generation by default (`SKILL.md:58-61`),
   and reflection-based serialization is limited under NativeAOT
   ("No runtime code generation", "Requires trimming",
   <https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/>, verified).
- Subscriptions:
   per connection,
   a bounded channel;
   on overflow the server ends the subscription with an error carrying the last delivered sequence number,
   and the client resumes (Watchman clock model from the design doc).

### R4: btrfs

- `FICLONE`:
   P/Invoke `ioctl(int, ulong, int)` with `0x40049409` (`_IOW(0x94, 9, int)`, same definition as `pal_io.c:81-82`).
  Declaring the variadic `ioctl` with a fixed signature relies on the x86-64 System V convention for integer arguments
   (inference, unverified on other architectures).
- `File.Copy` already tries `FICLONE` first (`pal_io.c:1453-1481`, verified),
   but the API page does not document it
   (<https://learn.microsoft.com/en-us/dotnet/api/system.io.file.copy?view=net-10.0>, no "reflink" or "clone",
verified),
   so the design must not rely on it.
- `statfs`:
   P/Invoke with a byte buffer,
   read `f_type` at offset 0,
   compare with `0x9123683E`.
- Subvolumes and snapshots:
   ioctls from X6 with hand-laid `btrfs_ioctl_vol_args` and `_v2` buffers,
   or the `btrfs` CLI through `Process` for these rare operations.

### R5: hashing

- `System.IO.Hashing.XxHash128` or `XxHash3`
   (Microsoft package built from `src/libraries/System.IO.Hashing`),
   or in-box `SHA256`.
- True multi-core hashing with `Parallel.ForEachAsync`.
- Throughput unmeasured (no NativeAOT build performed).

### R6: supervision and Ctrl+C

- `PosixSignalRegistration.Create(PosixSignal.SIGINT, ...)` with `Cancel = true`
   ("SIGINT and SIGQUIT can be canceled on both Windows and on Unix platforms; SIGTERM can only be canceled on Unix",
   <https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.posixsignalregistration.create?view=net-10.0>,
verified).
- Shutdown:
   stop accepting,
   hold the queue,
   write `cgroup.kill` for every task cgroup,
   wait for `populated 0`,
   remove the directories,
   unlink the socket.
- Exit detection:
   the poll thread wakes on pidfd readability,
   then P/Invoke `waitid(P_PIDFD, fd, siginfo, WEXITED)`.
- Startup sweep kills leftover `task-*` cgroups from a crashed daemon.
- `doctor` refuses pid 1 (the reaping hazard in R1).

### R8: repository fit

- Present today:
   zero `.cs` files,
   no .NET in `mise.toml`,
   no .NET in `.github/workflows` (`rg`, measured).
  `mise registry` lists `core:dotnet` (measured),
   and a user-level SDK 10.0.300 exists at `~/.local/share/mise/installs/dotnet/`,
   outside the repository configuration.
- Would have to be added:
  - SDK pin in `mise.toml`;
     NativeAOT build prerequisites "clang zlib-devel zlib-ng-devel zlib-ng-compat-devel" on Fedora
     (NativeAOT overview page, verified);
     `clang`, `/usr/include/zlib.h`, and `/usr/lib64/libz.so` exist on this host (measured) and must also exist in CI.
  - `.csproj`,
     `Directory.Build.props` (nullable, warnings as errors, `GenerateDocumentationFile` for doc-comment enforcement),
     NuGet lock file,
     file-enforcer management of `.csproj` metadata in the style of `manageCargoManifests`
     (`file-enforcer.config.ts:20`, `:115`).
  - Analyzer equivalents of repository rules with no built-in Roslyn counterpart,
     such as the max-lines budget,
     built like the Kotlin rule set in `package/linter/kotlin`.
  - Test framework,
     mise tasks,
     CI setup,
     forbidden-strings coverage.
  - Skill compliance:
     every P/Invoke is FFI and the daemon is async,
     both "stop and ask" constructs (`SKILL.md:91-102`).

### R9: documentation

Pages loaded with text present:
all Microsoft Learn pages in `probe-p1.tsv`,
`probe-p2.tsv`,
and `probe-p3.tsv`.
Triggers:

- **Contradiction on the interop mechanism the design depends on.**
  <https://learn.microsoft.com/en-us/dotnet/standard/native-interop/pinvoke-source-generation>:
   "Using DllImport isn't an option for platforms that require full Native AOT scenarios
   and therefore using other approaches (for example, source generation) is more appropriate."
  <https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/interop>:
   "native code interop works similarly in Native AOT and non-AOT deployments"
   and "The P/Invoke calls in AOT-compiled binaries are bound lazily at runtime by default".
  A reader cannot tell whether `[DllImport]` is supported under NativeAOT on Linux
  or whether the `[LibraryImport]` source generator is required,
   and the generator is code generation that the skill forbids by default.
- **Version applicability absent without JavaScript.**
  The "Applies to" section of
   <https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.posixsignalregistration.create?view=net-10.0>
   and of the `XxHash128` page is an empty `moniker-applies-to-table`
   with an empty `<thead>` in the served HTML (verified in raw HTML).
- **Unversioned URL lands on a release candidate.**
  <https://learn.microsoft.com/en-us/dotnet/api/system.io.hashing.xxhash3> redirects to `?view=net-11.0-pp`
   and shows "Package: System.IO.Hashing v11.0.0-rc.1.26425.128" (verified).
- Consumed behavior only in source:
   `File.Copy` reflinking and `FileSystemWatcher` recursion without exclusions;
   the design avoids both.
- Shared X1 trigger for the primary cgroup placement.
- Checked and dismissed:
   "Some information relates to prerelease product"
   and "Access to this page requires authorization" appear in extracted text,
   but sit in elements with class `is-hidden` (verified in raw HTML).

### R10: maturity

- `dotnet/runtime`:
   release v10.0.12 on 2026-09-08,
   tag `v11.0.0-rc.1.26425.128`,
   last commit 2026-09-16,
   18,276 stars,
   7,901 open issues (verified).
- `System.IO.Hashing` ships from the same repository.
- Not used:
   `microsoft/vs-streamjsonrpc` v2.25.29 (2026-06-15),
   `xoofx/Blake3.NET` 3.0.2 (2026-07-16).

### Yikes

1. **R9 cull trigger on the platform itself.**
    Microsoft's own pages contradict each other about whether `DllImport` works under NativeAOT,
     and every kernel feature in this design is a P/Invoke.
    Switching to JIT deployment avoids the NativeAOT question but not the fact that the confusion was hit.
2. **Every kernel feature is hand-declared FFI with hand-computed layouts.**
    cgroup spawn,
     inotify,
     pidfd waits,
     `FICLONE`,
     `statfs`,
     and btrfs ioctls all need P/Invoke signatures and byte offsets typed from C headers,
     including a variadic `ioctl` declared as fixed-arity;
     nothing checks them against the headers at build time.
3. **Primary placement depends on a glibc API documented only in NEWS (X1)**,
     and lazily bound P/Invokes fail at the first spawn rather than at startup on glibc older than 2.39.
4. **No public readiness API for arbitrary fds.**
    inotify and pidfd handling need dedicated threads and hand-written `poll` loops.
5. **`FileSystemWatcher` cannot exclude directories**,
     so the watcher is written from scratch on raw inotify.
6. **pid 1 reaping hazard**:
     in containers the runtime's `waitpid(-1)` steals task exit statuses.
7. **Whole new toolchain and lint stack**,
     including native build prerequisites on every build machine.
8. **JSON without reflection or code generation**
     means hand-mapping every RPC message through a DOM.

## Option P2: OCaml 5 with Eio

### Shape

- `Eio_main.run` on the io_uring backend (`kernel.io_uring_disabled` is 0 on this host, measured).
- Control plane (scheduler,
   RPC,
   watcher) as fibers in one domain;
   hashing in worker domains through `Eio.Domain_manager`.
- Unix-level escape hatches through `Eio_unix`.

### R1: cgroup placement, freeze, kill, limits

- Eio states the constraint in its own interface:
   "we cannot run any OCaml code in the forked child process ...
   Therefore, the fork call and all child actions need to be written in C"
   (`lib_eio/unix/fork_action.mli:4-13`, verified).
  The available actions are `execve`,
   `chdir`,
   `fchdir`,
   `inherit_fds`,
   `setpgid`,
   `setuid`,
   `setgid`,
   and `login_tty` (`fork_action.mli`, verified),
   and the module is exposed only as `Eio_unix.Private.Fork_action` (`lib_eio/unix/eio_unix.mli:137-148`, verified).
- Eio's Linux spawn calls `clone3` with `CLONE_PIDFD` only (`lib_eio_linux/eio_stubs.c:227-251`)
   using a local `clone_args` struct that ends at `tls` (`:48-57`),
   so adding `CLONE_INTO_CGROUP` means forking Eio's C stubs (verified).
- `Unix.fork` "fails if the OCaml process is multi-core (any domain has been spawned)"
   (`otherlibs/unix/unix.mli:507-511`; `fork.c:39-41`, verified).
- Paths that remain:
  - **Chosen: X2 trampoline.**
     `Eio_unix.Process.spawn_unix ~pgid:0` (`lib_eio_linux/process.ml:43-61`, verified) runs the daemon executable,
     which writes its pid with `Unix.openfile`/`Unix.write` into `cgroup.procs`,
     then `Unix.execvpe`.
    Public APIs only;
     per-task trampoline cost unmeasured.
  - Alternative:
     `ctypes-foreign` bindings to `pidfd_spawnp` (X1),
     then `Eio_unix.await_readable` on the pidfd and `Unix.waitpid [WNOHANG]`.
    Triggers X1 and the ctypes R9 trigger.
  - Rejected:
     a C fork action (repository C code plus a `Private` API),
     or patching Eio's `clone3`.
- Freeze and kill:
   write `cgroup.freeze` and `cgroup.kill` through `Eio.Path`;
   `Eio.Process.spawn` only sends `SIGKILL` to the direct child when its switch is released
   (<https://ocaml.org/p/eio/latest/doc/eio/Eio/Process/index.html>, verified),
   so descendants rely on `cgroup.kill`.

### R2: watching

- `inotify` 2.6 provides `create`,
   `add_watch`,
   `read`,
   and a documented `Q_overflow` "associated with a watch descriptor -1"
   (<https://ocaml.org/p/inotify/latest/doc/inotify/Inotify/index.html>, verified).
- `inotify-eio` wraps it by awaiting readability then reading (`lib/eio_inotify.ml:25-31`, verified).
- Dune's precedent reads the plain `Inotify` API on a dedicated system thread (`src/dune_scheduler/inotify.ml:150`)
   and turns `Q_overflow` into a queue-overflow event (`:136`, verified).
  To avoid the `inotify-eio` documentation trigger in R9,
   the design follows Dune:
   plain `Inotify` in `Eio_unix.run_in_systhread` (`lib_eio/unix/eio_unix.mli:70`, verified),
   then X5.

### R3: RPC

- `Eio.Net.listen` on a `` `Unix path `` address
   (<https://ocaml.org/p/eio/latest/doc/eio/Eio/Net/index.html>, loaded).
- Line framing with `Eio.Buf_read.of_flow ~max_size`,
   which "will raise an exception if the buffer would need to grow above max_size"
   (Eio README on ocaml.org, verified).
- JSON through `yojson` 3.0.0 (`Yojson.Safe`);
   JSON-RPC envelopes hand-written.
- `Unix.mkdir dir 0o700` for the socket directory.
  `Unix.getsockopt` variants cover bool,
   int,
   optional int,
   float,
   and error options only (`unix.mli:1612-1645`; no `PEERCRED` anywhere in `unix.mli`, verified),
   so peer credentials need FFI or are skipped in favor of directory permissions.
- Subscriptions:
   a bounded `Eio.Stream` per client with the same overflow-and-resume rule as P1.

### R4: btrfs

- The `Unix` library has no `ioctl` and no `statfs` (`rg` on `unix.mli`, verified).
- Chosen to avoid FFI:
   filesystem type from `/proc/self/mountinfo`;
   reflink restores through batched `cp --reflink=always` child processes;
   snapshots through the `btrfs` CLI (`/usr/bin/btrfs` present, measured).
- Alternative:
   `ctypes-foreign` bindings to `ioctl` and `statfs`.

### R5: hashing

- Standard library `Digest` gained BLAKE2b in OCaml 5.2.0 (`Changes:3312` under `Changes:2990`, verified);
   `digestif` 1.3.1 offers C-backed BLAKE2 and SHA-2.
- Parallel hashing across domains.
- Throughput unmeasured (no OCaml toolchain on host).

### R6: supervision and Ctrl+C

- `Sys.set_signal Sys.sigint (Signal_handle ...)` broadcasting an `Eio.Condition`,
   which the Eio README presents as the one safe operation in a handler
   ("you can safely call Eio.Condition.broadcast", verified).
- Exit status through `Eio.Process.await` (trampoline path) or pidfd readiness (ctypes path).
- Shutdown and startup sweep as in P1.

### R8: repository fit

- Present today:
   no OCaml files,
   no OCaml in CI.
  `mise registry` lists `opam` (`github:ocaml/opam`) but no OCaml compiler and no Dune entry (measured).
- Would have to be added:
   an opam switch or Dune package management per machine and CI,
   `dune-project` and opam lock files,
   `ocamlformat`,
   odoc-based documentation checks,
   equivalents of max-lines and doc-comment rules,
   test stanzas,
   file-enforcer management,
   forbidden-strings coverage.
- Skill fit for a TypeScript-only reader:
   pattern matching as control flow,
   functors,
   and Eio's capability types conflict with rule 1 (`SKILL.md:62-74`);
   FFI and the effects-based async runtime are "stop and ask" constructs (`SKILL.md:91-102`);
   `ppx`-based testing is metaprogramming.

### R9: documentation

Pages loaded with text present:
ocaml.org manual (`Unix`, `Sys`, parallelism, C interface),
Eio package pages,
`inotify`,
`inotify-eio`,
`ctypes-foreign`,
`yojson`,
`digestif`,
Dune and opam manuals (`probe-p1.tsv` to `probe-p3.tsv`).
Triggers:

- **`inotify-eio` describes itself as the wrong library.**
  <https://ocaml.org/p/inotify-eio/latest/doc/inotify-eio/Eio_inotify/index.html>:
   "An Lwt wrapper for Inotify module"
   (source `lib/eio_inotify.mli:1`, verified).
- **`ctypes-foreign` documents parameters a function does not have.**
  <https://ocaml.org/p/ctypes-foreign/latest/doc/ctypes-foreign/Foreign/index.html>:
   `funptr` "raises Dl.DL_error if name is not found in ?from and ?stub is false",
   but its signature has neither `?from` nor `?stub`;
   `foreign`'s `?check_errno` warns that "a function that succeeds is allowed to change errno. So use this option
   with caution",
   leaving reliable `errno` retrieval unexplained.
- **Probable trigger for a TypeScript-only reader.**
  `Eio.Process.spawn` is typed `sw:Switch.t -> [> 'tag mgr_ty ] Std.r -> ... -> 'tag ty Std.r`,
   with `'a t = ([> [> `Generic ] ty ] as 'a) Std.r`,
   and the page does not explain these types (verified).
  Whether this counts is the user's call.
- The chosen design avoids the first two by using plain `Inotify`,
   the trampoline,
   and CLIs;
   the X1 trigger applies only to the ctypes alternative.

### R10: maturity

- `ocaml/ocaml`:
   release 5.5.1 on 2026-09-04,
   last commit 2026-09-15.
- `ocaml-multicore/eio`:
   v1.5 on 2026-08-19,
   last commit 2026-09-11,
   724 stars,
   60 open issues.
- `ocaml-multicore/ocaml-uring`:
   v2.15.0 on 2026-07-04.
- `whitequark/ocaml-inotify`:
   v2.6 on 2024-07-26,
   no commits since,
   42 stars.
- `haesbaert/ocaml-iomux` (dependency of `inotify-eio`):
   v0.4 on 2025-09-17,
   30 stars.
- `yallop/ocaml-ctypes`:
   GitHub latest release 0.21.1 (2023-07-20) while ocaml.org lists `ctypes-foreign` 0.24.0,
   last commit 2025-11-06,
   106 open issues.
- `ocaml-community/yojson`:
   3.0.0 on 2025-05-29,
   last commit 2025-08-01.
- `mirage/digestif`:
   v1.3.1 on 2026-07-14.
- `ocaml/dune`:
   3.24.2 on 2026-08-03.

### Yikes

1. **No in-language cgroup placement.**
    Eio's own interface says child actions must be C and hides them under `Private`,
     Eio's `clone3` stub cannot carry a cgroup,
     and `Unix.fork` is unavailable once domains exist;
     what remains is an extra exec per task,
     libffi,
     or repository C.
2. **The kernel surface is missing from the standard library.**
    No `ioctl`,
     `statfs`,
     inotify,
     or peer credentials,
     so btrfs work goes through CLI subprocesses or FFI.
    The strongest precedent,
     Dune,
     solves the same problems with C stubs
     (`otherlibs/stdune/src/spawn_stubs.c` via `src/dune_engine/process.ml:1055`; `src/rpc/csexp_rpc.ml:19-21`, `:301`),
     so following precedent means writing C.
3. **R9 triggers in two libraries the natural design would use**
     (`inotify-eio`, `ctypes-foreign`),
     and a probable trigger in Eio's core type signatures.
4. **Toolchain is outside mise**:
     no compiler or Dune entry in the registry,
     so every machine and CI job builds an opam switch.
5. **Thin library maintenance**:
     the inotify binding has been idle since 2024-07,
     and ctypes' GitHub releases lag its opam releases by several versions.
6. **Evidence gap**:
     no OCaml measurement was possible without installing a toolchain,
     so trampoline cost and hashing throughput are unknown.

## Option P3: Python 3.14 (CPython, standard library first)

### Shape

- `asyncio.Runner` main loop for scheduler and RPC.
- inotify reader on a dedicated thread doing blocking `os.read` and handing events to the loop
   with `call_soon_threadsafe`,
   so CPU work on the loop thread cannot delay draining the kernel queue (X5).
- Hashing in a process pool;
   Python 3.14 changed the default start method on POSIX "from fork to forkserver"
   (<https://docs.python.org/3/library/multiprocessing.html>, verified).
- No third-party runtime packages.

### R1: cgroup placement, freeze, kill, limits

- `subprocess.Popen(preexec_fn=...)` is rejected:
   "The preexec_fn parameter is NOT SAFE to use in the presence of threads in your application. The child process could
   deadlock before exec is called."
   (<https://docs.python.org/3/library/subprocess.html>, verified;
   `Modules/_posixsubprocess.c:1001`),
   and this design has threads.
- `os.posix_spawn` has no cgroup parameter (os page, verified).
- **Chosen: X1 through `ctypes`**,
   verified on this host by the X1 probe and its control.
  `ctypes.CDLL` releases the GIL during the call
   ("The Python global interpreter lock is released before calling any function exported by these libraries",
   <https://docs.python.org/3/library/ctypes.html>, verified),
   and the buffers are `ctypes` C memory that CPython never moves.
  Pipes come from `os.pipe2(os.O_CLOEXEC)` with `adddup2` file actions.
- asyncio's own child handling waits only for the pids it started (`os.waitpid(pid, 0)` in `_PidfdChildWatcher`,
   `Lib/asyncio/unix_events.py:891-924`, verified),
   so it never reaps `ctypes`-spawned tasks.
- Fallback (X2):
   trampoline `python3 -I -S -m <daemon> __enter-cgroup ...` then `os.execvpe`.
  Measured start cost (`SCRATCH/others/startup.ts`, 30 runs each after one warm-up):
   `python3 -I -S -c pass` median 10.2 ms (9.6 to 11.5);
   `python3 -c pass` median 14.4 ms (13.5 to 15.8);
   `/usr/bin/true` median 1.4 ms.
- Freeze and kill:
   file writes;
   completion through `cgroup.events` (X6).

### R2: watching

- The standard library has no inotify module.
  Design:
   `ctypes` calls to `inotify_init1` and `inotify_add_watch`,
   `os.read`,
   `struct.unpack_from("iIII", ...)` per event,
   then X5 on the loop.
- Third-party options:
   `inotify_simple` 2.0.1 does the same thin `ctypes` wrapping;
   `asyncinotify` 4.4.4 is excluded by its R9 triggers.

### R3: RPC

- `asyncio.start_unix_server(handler, path, limit=...)`;
   the default `limit` "is set to 64 KiB"
   (<https://docs.python.org/3/library/asyncio-stream.html>, verified),
   so larger JSON-RPC lines fail unless the limit is raised.
- `json.loads`/`json.dumps`;
   JSON-RPC envelopes hand-written (the repository's TypeScript `@monochromatic-dev/mcp-stdio` is the model).
- `os.mkdir(dir, 0o700)`;
   peer uid via `getsockopt(SOL_SOCKET, SO_PEERCRED, 12)`
   (`socket.SO_PEERCRED` is 17 on this host, measured).
- Subscriptions:
   bounded `asyncio.Queue` per client with the same overflow-and-resume rule as P1.

### R4: btrfs

- `fcntl.ioctl(dst_fd, fcntl.FICLONE, src_fd)`:
   "On Linux >= 4.5, the fcntl module exposes the FICLONE and FICLONERANGE constants"
   (added in 3.12,
   <https://docs.python.org/3/library/fcntl.html>, verified;
   `Modules/fcntlmodule.c:675-681`).
- `os.statvfs` has no filesystem type field,
   so type detection uses `/proc/self/mountinfo` or a `ctypes` `statfs`.
- Subvolume and snapshot ioctls through `fcntl.ioctl(fd, request, bytearray)` with `struct.pack`ed argument blocks (X6).

### R5: hashing

- `hashlib.file_digest(f, "blake2b")`.
  "the Python GIL is released while computing a hash supplied more than 2047 bytes of data at once"
   (<https://docs.python.org/3/library/hashlib.html>, verified).
- Measured on this host
   (`SCRATCH/others/hash_probe.py`, warm page cache, GIL build confirmed by `sys._is_gil_enabled()` returning `True`):
   all 8,083 tracked files (151,343,931 bytes) hashed serially in 273 to 277 ms over three runs;
   with 16 threads,
   306 to 316 ms.
  Threads are slower than serial for this file-size mix,
   so parallel hashing needs processes.

### R6: supervision and Ctrl+C

- `asyncio.Runner` handles Ctrl+C by cancelling the main task
   ("Handling Keyboard Interruption", added in 3.11,
   <https://docs.python.org/3/library/asyncio-runner.html>, verified),
   or `loop.add_signal_handler`.
- Exit detection:
   `loop.add_reader(pidfd, ...)`,
   then `os.waitid(os.P_PIDFD, pidfd, os.WEXITED)`
   ("P_PIDFD - wait for the child identified by the file descriptor",
   <https://docs.python.org/3/library/os.html>, verified).
- Shutdown and startup sweep as in P1.

### R8: repository fit

- Present today:
   no Python files in the repository,
   no Python in CI (`rg`, measured).
  `doc/planning/load-bearing-code-languages.md:7-12` names Python among the unapproved languages the proposed rule
  covers.
- `mise registry` lists `core:python`,
   `aqua:astral-sh/uv`,
   and `aqua:astral-sh/ruff` (measured).
- Would have to be added:
   interpreter pin and `uv` lock,
   `pyproject.toml`,
   a formatter and linter configuration,
   a strict type checker,
   docstring enforcement,
   max-lines enforcement,
   tests,
   file-enforcer management,
   forbidden-strings coverage.
- Typing:
   `ctypes` byte buffers are invisible to type checkers,
   so the riskiest code is the least checked.
- Skill fit:
   decorators and comprehensions are avoidable;
   `ctypes` FFI and asyncio are "stop and ask" constructs (`SKILL.md:91-102`).

### R9: documentation

Standard library pages loaded with text present and no trigger found in the consumed sections read
(`subprocess` preexec and process group,
`os` pidfd and `waitid`,
`fcntl`,
`ctypes` library loading,
`hashlib` GIL note,
asyncio streams,
runner,
event loop,
subprocess,
`json`,
`multiprocessing` start methods).
Triggers in third-party documentation:

- **`asyncinotify`**
   (<https://asyncinotify.readthedocs.io/en/latest/asyncinotify.html>):
   "class asyncinotify.RecursiveInotify: A Recursive superclass of Inotify. Adds the add_recursive_watch() method,
   but otherwise works the same",
   which describes a subclass as a superclass;
   a second class,
   `RecursiveWatcher`,
   also watches "a folder recursively",
   with no guidance on how the two differ.
- **PyPI**
   (<https://pypi.org/project/blake3/>):
   plain `curl` receives a "Client Challenge" page requiring JavaScript (`probe-p2.tsv`).
- **`watchdog`**
   (<https://python-watchdog.readthedocs.io/en/stable/>):
   the "stable" site is titled "watchdog 2.1.5 documentation" while the latest GitHub release is v6.0.0 (2024-11-01).
- Shared X1 trigger for the primary cgroup placement;
   the trampoline fallback avoids it.
- Noted,
   not consumed:
   the `os` page defines `P_PGID` as waiting for a child "whose progress group ID is id".

### R10: maturity

- CPython:
   host 3.14.7;
   repository tag `v3.15.0rc2`,
   last commit 2026-09-16.
- The standard-library-first design needs no third-party runtime package.
- Optional:
   `chrisjbillington/inotify_simple` 2.0.1 (2025-08-25, 135 stars);
   `oconnor663/blake3-py` 1.0.9 (2026-06-22);
   `ifduyue/python-xxhash` v4.0.1 (2026-08-17);
   `ProCern/asyncinotify` v4.4.4 (2026-04-13, 47 stars, excluded by R9);
   `gorakhargosh/watchdog` v6.0.0 (2024-11-01, commits through 2026-09-06).
- Tooling:
   `astral-sh/uv` 0.12.15 (2026-09-15).

### Yikes

1. **All kernel work is untyped `ctypes` memory.**
    cgroup spawn,
     inotify event parsing,
     `statfs`,
     and btrfs argument blocks are byte buffers laid out by hand;
     a wrong offset corrupts memory or misreads events instead of failing type checks.
2. **Python is named as an unapproved language in the repository's own proposal**,
     and the repository's strict static typing culture has no equivalent over the FFI layer.
3. **GIL runtime.**
    Measured threads slower than serial for hashing;
     parallelism requires a forkserver process pool,
     and any CPU work on the loop thread delays RPC and risks inotify overflow unless the reader has its own thread.
4. **Primary placement depends on the NEWS-only glibc API (X1)**;
     the documented trampoline costs about 10 to 14 ms per task (measured).
5. **Third-party ecosystem fails the documentation gate** in the libraries a Python developer would reach for first,
     forcing the hand-written `ctypes` layer of yikes 1.
6. **Deployment of an interpreter plus environment** for a long-running daemon;
     importing `asyncio`, `json`, `ctypes`, and `hashlib` takes a median 54.8 ms (measured),
     which affects client command latency.

## Cross-option comparison

### By requirement

- **R1 placement.**
  Python has a probed in-language path (X1 via `ctypes`) plus a measured trampoline.
  .NET has the same X1 path through P/Invoke plus a `Process`-based trampoline,
   with a pid 1 reaping hazard.
  OCaml has no in-language path besides the trampoline or libffi;
   Eio documents that child actions must be C.
- **R2 watching.**
  All three hand-write X5 on raw inotify.
  .NET's built-in watcher cannot exclude directories;
   OCaml has a thin,
   idle binding with Dune as precedent;
   Python writes the binding itself.
- **R3 RPC.**
  All three are adequate.
  .NET has peer credentials built in;
   Python has them in `socket`;
   OCaml lacks them without FFI.
- **R4 btrfs.**
  Python documents `FICLONE`;
   .NET needs P/Invoke with a variadic `ioctl`;
   OCaml needs FFI or CLI subprocesses.
- **R5 hashing.**
  .NET and OCaml hash on all cores;
   Python measured 273 ms serial for the whole tracked tree and no gain from threads.
- **R6 supervision.**
  Python and OCaml have public fd-readiness APIs;
   .NET needs dedicated poll threads.
- **R8 fit.**
  All three need a full toolchain,
   lint,
   and test stack.
  .NET and Python are in the mise registry;
   OCaml's compiler is not.
  Python is explicitly listed as unapproved.
- **R9 documentation.**
  .NET's trigger is on the platform's own interop pages and hits the mechanism every kernel call uses.
  OCaml's and Python's triggers are in swappable libraries,
   which their chosen designs avoid.
  X1 affects any design using glibc's cgroup spawn.
- **R10 maturity.**
  .NET needs no third-party packages;
   Python's standard-library design needs none;
   OCaml depends on several small single-maintainer libraries.

### Ranking

Ranking under the user's recorded rules (documentation confusion culls; surface pros and cons do not rank):
Python > OCaml > .NET.

- **Python over OCaml**:
   Python's standard library plus `ctypes` covers R1 through R6 without repository C or CLI subprocesses,
   with the cgroup spawn path verified on this host,
   while OCaml's own Eio interface rules out in-process placement without C and its toolchain is outside mise.
  This flips if the user treats Python's explicit unapproved listing or the GIL measurement as disqualifying.
- **OCaml over .NET**:
   .NET's documentation contradiction sits on the platform's interop mechanism that every kernel call depends on,
   so the strict documentation rule removes the platform path,
   while OCaml's triggers sit in libraries its design can avoid.
  This flips if the user discounts that contradiction or accepts JIT deployment;
   then .NET's typed,
   multi-core runtime with no third-party packages ranks above OCaml's C-or-FFI constraint.

Neither ranking compares these options with TypeScript,
Rust,
Kotlin,
Go,
or Zig,
which other agents cover.
One comparison point applies across that boundary:
Rust's `CommandExt::pre_exec` does not depend on X1's NEWS-only glibc API.
