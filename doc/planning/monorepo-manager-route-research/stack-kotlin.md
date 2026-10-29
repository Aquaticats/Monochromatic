# Kotlin stacks for the from-scratch monorepo manager

Outcome on 2026-09-16:
the user judged every Kotlin option not worth further research.
This file is kept as evidence.

## Status and scope

- Date:
   2026-09-16.
- Scope:
   read-only design deep dive of three Kotlin stacks for the daemon in
   `doc/planning/monorepo-manager-from-scratch-design.md`.
  Nothing under `/var/home/user/Monochromatic` was modified.
- Options:
  - Option 1,
     Kotlin/Native `linuxX64`,
     with file-enforcer ported to Kotlin.
  - Option 2,
     Kotlin/JVM,
     with a GraalVM Native Image variant,
     with file-enforcer ported to Kotlin.
  - Option 3,
     a Kotlin daemon core with the TypeScript file-enforcer run as Node child processes.
- Labels:
  - **Verified**:
     a repository file and line,
     an external source file and line read from a clone or a single-file download,
     an official doc page fetched with plain `curl`,
     or a command run in this session with its output quoted.
  - **Measured**:
     a probe run in this session;
     the command and output are in "Probe log".
  - **Fetched page**:
     read through a summarizing fetcher because plain `curl` was blocked;
     quotes are less exact.
  - **Unverified**:
     inference or recall that no read source settles.
- Not done,
   because it would install software:
   no Kotlin/Native compile
   (the host has `~/.konan/kotlin-native-prebuilt-linux-x86_64-1.9.21` without its toolchain dependencies),
   no Kotlin compile,
   no GraalVM build,
   no cgroup creation,
   no `systemd-run` probe.
  Every Kotlin/Native build-time and runtime claim here is therefore unmeasured.

## Shared design every option must implement

These pieces don't depend on the language.
Each option section below says how its runtime reaches them.

- Process model:
   the user starts the daemon with `systemd-run --user --scope -p Delegate=yes`;
   the daemon moves itself into `daemon/` and creates `tasks/<id>/` leaf cgroups
   (design doc,
   "Process model").
- Race-free placement:
   a task must be in its cgroup before it runs anything.
  Three placement mechanisms exist on this host:
  - glibc `posix_spawnattr_setcgroup_np` with `POSIX_SPAWN_SETCGROUP`,
     plus `pidfd_spawn`.
    Added in glibc 2.39
     (glibc `NEWS` at tag `glibc-2.39`,
     lines 24-35,
     verified).
    glibc implements it as `clone3` with `CLONE_INTO_CGROUP | CLONE_PIDFD | CLONE_VM | CLONE_VFORK`
     and has no fallback when `clone3` is missing
     (`sysdeps/unix/sysv/linux/spawni.c` at `glibc-2.39`,
     lines 396-433,
     verified).
    The child resets signal handlers to `SIG_DFL` before exec
     (same file,
     lines 111-135,
     verified),
     so no runtime code runs in the child.
    The host glibc is 2.43
     (`ldd --version`,
     measured)
     and exports `pidfd_spawn@@GLIBC_2.39` and `posix_spawnattr_setcgroup_np@@GLIBC_2.39`
     (`nm --dynamic`,
     measured).
  - A launcher:
     a helper process writes its own PID to `cgroup.procs`,
     then calls `execve`.
  - `systemd-run --user --scope` for each task.
    In scope mode `systemd-run` makes a `StartTransientUnit` D-Bus call and then calls `execvpe` on the command itself
     (systemd `src/run/run.c`,
     main branch,
     `start_transient_scope` at line 2519,
     `execvpe` at line 2727,
     verified).
- Documentation gap shared by the first mechanism:
   `POSIX_SPAWN_SETCGROUP` and `pidfd_spawn` appear in glibc `NEWS` and the `spawn.h` header.
  They don't appear in the man7 `posix_spawn(3)` page
   (plain `curl`,
   0 matches,
   measured),
   in the local man pages
   (`man --where pidfd_spawn` reports "No manual entry",
   measured),
   or as a `@deftypefun` in the glibc manual source
   (mirror `bminor/glibc`,
   `manual/process.texi`,
   0 matches,
   verified).
  Under the HC7 test in `doc/audit/tech-monorepo-manager-vet-2026-09-16.md`,
   "a feature named only in a changelog ... counts as undocumented".
  The documented kernel primitive is `CLONE_INTO_CGROUP` in man7 `clone(2)`
   (10 matches,
   measured).
  No managed runtime can call it directly,
   because the child would continue running runtime frames
   (inference).
- Ctrl+C isolation:
   children spawned without a new session stay in the terminal's foreground process group,
   so a terminal Ctrl+C reaches every task directly
   (inference from `setpgid(2)` semantics).
  The spawn path must set `POSIX_SPAWN_SETSID`
   (`0x80`,
   glibc 2.39 `posix/spawn.h:62`,
   verified).
- Pause:
   write `1` to `cgroup.freeze` and wait for `frozen 1`.
  End:
   write `1` to `cgroup.kill`.
  Reaping goes through a pidfd.
- Watching:
   5,480 non-ignored directories,
   `max_user_watches` 524288,
   `max_queued_events` 16384
   (`/proc/sys/fs/inotify/max_queued_events`,
   measured).
- RPC:
   newline-delimited JSON-RPC 2.0 on a Unix socket in `$XDG_RUNTIME_DIR`,
   with subscriptions resumed from a sequence number.

## Cross-cutting findings for every Kotlin option

### X1: the Kotlin compile daemon survives `--no-daemon`

- The recorded decision says tasks run Gradle with its daemon disabled
   "so ending or freezing a task cannot kill or stall shared Gradle state"
   (`doc/planning/monorepo-manager-from-scratch-design.md:375-376`,
   verified).
- The Android tasks already pass `--no-daemon`
   (`package/music-player/android-app/mise.toml:137`,
   `:143`,
   `:149`,
   verified),
   yet `/tmp` holds 43 `kotlin-daemon.*.log` files
   (measured).
  The newest log,
   `/tmp/kotlin-daemon.2026-09-09.19-31-24-063.00.log`,
   records `Kotlin compiler daemon version 2.2.10-release-430`,
   `--daemon-autoshutdownIdleSeconds=7200`,
   `daemon is listening on port: 17864`,
   and `initiate elections`
   (measured).
  Its session file sits under `package/music-player/android-app/.kotlin/sessions/`.
- Kotlin docs:
   the daemon strategy is the default,
   and "The daemon process can be shared between different build system processes and multiple parallel compilations"
   (<https://kotlinlang.org/docs/compiler-execution-strategy.html>,
   verified).
  `kotlin.compiler.execution.strategy=in-process` turns it off
   (same page,
   verified).
- Neither Gradle project sets that property
   (`package/linter/kotlin/gradle.properties` and `package/music-player/android-app/gradle.properties`,
   verified).
- Consequence for every route, not only Kotlin:
   a Kotlin daemon started inside one task's cgroup can be shared by a concurrent task,
   so `cgroup.kill` or `cgroup.freeze` on the first task can kill or stall the second task's compile
   (inference from the doc quote plus the measured log).
  The Kotlin options add pressure,
   because the daemon's own development builds also run through Gradle.

### X2: the repository's non-TypeScript comment rule treats FFI as a stop-and-ask construct

- `.claude/skills/dum-dum-non-ts/SKILL.md` applies to Kotlin.
  For "FFI" it says "stop and ask the user"
   (lines 95-97,
   verified),
   and it requires a What,
   Why,
   and TypeScript-pseudocode block on every concept-introducing line,
   at about "15:1" comments to code
   (line 470,
   verified).
- The daemon core is mostly FFI in Option 1,
   partly FFI in Options 2 and 3.
  So the user has to decide under that skill before implementation starts.

### X3: documentation triggers shared by all Kotlin options

Each page was fetched once with the required `curl` command and user agent.
Every page loaded with text present
(status 200,
no block markers)
unless noted.

- kotlinx.serialization,
   <https://kotlinlang.org/docs/serialization-json-elements.html>:
   the page says "If you use Kotlin/JVM `BigDecimal`,
   the value stays precise,
   but `JsonPrimitive()` encodes the value as a string rather than as a number".
  The library source says `JsonPrimitive(value: Number?)` returns `JsonLiteral(value, isString = false)`
   (`formats/json/commonMain/src/kotlinx/serialization/json/JsonElement.kt:60-63`,
   verified),
   and `BigDecimal` is a `Number`.
  That is a contradiction on a JSON API the RPC layer would use.
- Kotlin Gradle plugin,
   <https://kotlinlang.org/docs/gradle-compilation-and-caches.html>:
   one paragraph says a new Kotlin daemon starts "only when ...
   existing Kotlin daemons do not have the same set of JVM arguments".
  The next paragraph says a running daemon "will be reused instead of starting a new one" "even if other requested JVM
  arguments are different".
  This contradiction sits on exactly the behavior X1 depends on.
- Kotlin scripting stability
   (matters only if a Kotlin port keeps configuration as scripts):
   <https://kotlinlang.org/docs/custom-script-deps-tutorial.html> says "Kotlin custom scripting is Experimental.
  It may be dropped or changed at any time."
  <https://kotlinlang.org/docs/components-stability.html> lists "Scripting embedding and extension API Beta 1.5.0"
  and "Scripting syntax and semantics Alpha 1.2.0".
- Gradle,
   <https://docs.gradle.org/current/userguide/gradle_daemon.html>:
   with `--no-daemon`,
   "the Gradle Daemon uses the JVM that launched the Gradle Client".
  The same page later says the daemon "will not be used at all since the build happens in the client JVM".
  The two statements frame the same case differently.
  The repository already uses Gradle,
   so this is not specific to Kotlin.
- systemd,
   used only by the `systemd-run` placement variant:
   <https://www.freedesktop.org/software/systemd/man/latest/systemd-run.html> and `systemctl.html` returned HTTP 418
   with "Checking you are not a bot" (go-away)
   (measured).
  The local `man systemd-run` works.
  It says that after `systemd-run` passes the command to the service manager,
   "the manager performs variable expansion".
  In scope mode,
   `systemd-run` itself expands `$VAR` in argv before `execvpe`
   (`src/run/run.c:2707-2714`,
   verified),
   so task arguments that contain `$` need `--expand-environment=no`.
  That is a syntax-boundary hazard under `SYB`.

Not run here:
the same strict probe was not run against Rust or TypeScript documentation,
so X3 does not show that Kotlin fares worse than those stacks.

## Option 1: Kotlin/Native `linuxX64`

### Design sketch

- One ELF executable built by `linkReleaseExecutableLinuxX64`.
- One event-loop thread on `epoll`,
   through `platform.linux`.
  It waits on the inotify fd,
   the listening Unix socket and its client sockets,
   per-task stdout and stderr pipes,
   per-task pidfds,
   an `eventfd` for scheduler wakeups,
   and inotify watches on each task's `cgroup.events`.
- The scheduler,
   cache,
   and graph run as coroutines on `Dispatchers.Default`.
  Hashing runs on a bounded worker pool.
- Spawn:
   `dlsym(RTLD_DEFAULT, "pidfd_spawn")` and `dlsym(..., "posix_spawnattr_setcgroup_np")`,
   cast to `CPointer<CFunction<...>>` and invoked.
  Flag values are declared by hand:
   `POSIX_SPAWN_SETCGROUP = 0x100`,
   `POSIX_SPAWN_SETSID = 0x80`,
   `P_PIDFD = 3`.
- Fallback spawn:
   `posix_spawn` of the daemon's own executable in a `--exec-in-cgroup <dir> -- argv` mode.
  That process writes its PID to `cgroup.procs` and calls `execve`.
  It uses only documented POSIX calls,
   at the cost of one extra executable start per task (unmeasured).
- RPC:
   a hand-written NDJSON framer over nonblocking sockets,
   with kotlinx.serialization `JsonElement` for the `id` and `params` unions.
- btrfs:
   a custom cinterop `.def` for `linux/fs.h` and `linux/btrfs.h`.
  Missing ioctl structs are declared by hand in the `.def` `---` section.
- file-enforcer:
   ported to Kotlin.
  Configuration becomes compiled Kotlin or data,
   because Kotlin/Native has no scripting host
   (unverified beyond the stability page,
   which lists scripting only for the JVM toolchain).

### R1 per-task cgroup v2

- Kotlin/Native compiles and links against a bundled sysroot:
   `toolchainDependency.linux_x64 = x86_64-unknown-linux-gnu-gcc-8.3.0-glibc-2.19-kernel-4.9-2`
   (`kotlin-native/konan/konan.properties:66` at tag `v2.4.20`,
   verified).
  The latest stable tag is `v2.4.20`
   (`gh api repos/JetBrains/kotlin/tags`,
   measured).
  That toolchain tarball is 101,793,914 bytes with `last-modified: Wed, 12 May 2021`
   (HTTP HEAD,
   measured).
- Symbols absent from the `platform.linux` and `platform.posix` bindings
   (byte search of the 1.9.21 platform klibs with identifier boundaries on both sides;
   present symbols such as `inotify_init1` and `epoll_create1` act as the positive control;
   measured):
  - `pidfd_spawn`
  - `posix_spawnattr_setcgroup_np`
  - `POSIX_SPAWN_SETSID`
  - `posix_spawn_file_actions_addchdir_np`
  - `pidfd_open`
  - `P_PIDFD`
  - `SYS_pidfd_open`
  - `pipe2`
  - `accept4`
  - `copy_file_range`
  - `statx`
  - `struct ucred`
- The 1.9.21 result carries over to 2.4.20 by inference from verified inputs:
   the same sysroot (`konan.properties:66`)
   and the same `linux.def` `compilerOpts = -D_ANSI_SOURCE -D_POSIX_C_SOURCE=199309 -D_BSD_SOURCE -D_XOPEN_SOURCE=700`
   without `_GNU_SOURCE`
   (`kotlin-native/platformLibs/src/platform/linux/linux.def:39` at `v2.4.20`,
   verified).
- Linking a host symbol that the sysroot lacks fails at link time.
  KT-67046 shows `ld.gold` errors such as "undefined reference to 'dlopen', version 'GLIBC_2.34'".
  A JetBrains engineer answered that the toolchain can't change without breaking the library ecosystem,
   that "the proper solution here is to rework the cinterop",
   and "I can't give any timeframe"
   (2024-04-16).
  On 2026-04-14 the suggestion was "this dirty hack":
   `--unresolved-symbols=ignore-all`.
  The issue state is "Answered",
   with affected versions up to 2.3.21
   (YouTrack REST API,
   verified).
- Precedent for the `dlsym` workaround:
   `kmp-process` loads `posix_spawn_file_actions_addchdir_np` with `dlsym(null, ...)` on Linux
   (`library/process/src/linuxMain/kotlin/io/matthewnelson/kmp/process/internal/spawn/LinuxPosixSpawn.kt:79`,
   verified).
- Kernel headers:
   Linux 4.9 `include/uapi/linux/sched.h` has no `CLONE_INTO_CGROUP` or `CLONE_PIDFD`
   (verified).
- Freeze,
   kill,
   and limits are file writes through `platform.posix` `open` and `write`
   (no gap).
- Verdict:
   possible only through `dlsym` function pointers and hand-declared constants,
   or through the launcher fallback.
  The race-free glibc API is also undocumented ("Shared design every option must implement").

### R2 recursive watching

- The `v2.4.20` `linux.def` includes `sys/inotify.h` (line 22),
   `sys/epoll.h` (line 19),
   `sys/eventfd.h` (line 20),
   and `sys/signalfd.h` (line 26)
   (verified).
  `inotify_init1`,
   `IN_Q_OVERFLOW`,
   and `IN_CLOEXEC` are present in the bindings
   (measured).
- The design walks the tree once and adds one watch per non-ignored directory.
  On `IN_CREATE` with `IN_ISDIR` it adds a watch and rescans that directory,
   to close the new-directory race.
  On `IN_Q_OVERFLOW` it marks all state dirty,
   rescans,
   and rehashes.
  It parses the variable-length `inotify_event` records with `reinterpret` and pointer arithmetic.
- Library alternative:
   `irgaly/kfswatch` 1.4.0 (2025-10-17) uses inotify on Linux native,
   "does not support recursive directory watching",
   and exposes an overflow flow
   (README lines 96, 274-276, 388,
   verified).
  Contributors:
   `irgaly` 398 commits,
   `renovate[bot]` 124
   (verified).

### R3 Unix socket JSON-RPC server

- Ktor `ktor-network` supports `UnixSocketAddress` on non-JVM targets
   (`ktor-network/nonJvm/src/io/ktor/network/sockets/SocketAddress.nonJvm.kt:64`,
   verified).
- Its Unix-like native selector uses `pselect`
   (`ktor-network/nix/src/io/ktor/network/selector/SelectUtilsNix.kt:87`,
   verified)
   and fails any descriptor at or above `FD_SETSIZE`:
   "File descriptor ... is larger or equal to FD_SETSIZE"
   (`:201-203`,
   verified).
  Every fd the daemon holds counts toward that limit:
   pipes,
   pidfds,
   watches on `cgroup.events`,
   and clients.
- The Ktor sockets doc says sockets "use an experimental API"
   (<https://ktor.io/docs/server-sockets.html>,
   verified)
   and doesn't mention Unix domain sockets.
- `kotlinx-io` has no sockets.
  Its README marks it Alpha and "experimental"
   (README lines 3, 38,
   verified).
- Design choice:
   skip Ktor and write the socket loop on `epoll`,
   with `sockaddr_un` and `SO_PEERCRED` from the bindings (measured present).
  `struct ucred` is absent (measured),
   so the 12-byte credential layout is declared by hand.
- kotlinx.serialization 1.11.0 is Stable
   (components-stability page;
   GitHub release `v1.11.0`,
   2026-04-09,
   verified)
   and supports Kotlin/Native.

### R4 btrfs

- `statfs` is present
   (`linux.def:27`;
   bindings,
   measured).
  The design compares `f_type` with `BTRFS_SUPER_MAGIC`.
- `ioctl` is present in the bindings (measured).
  A custom `.def` over the sysroot's Linux 4.9 headers gets:
  - `FICLONE`
     (`include/uapi/linux/fs.h` at `v4.9`,
     verified);
  - `BTRFS_IOC_SNAP_CREATE_V2`,
     `BTRFS_IOC_SUBVOL_CREATE_V2`,
     and `BTRFS_IOC_SNAP_DESTROY`
     (`include/uapi/linux/btrfs.h` at `v4.9`,
     verified).
- `BTRFS_IOC_GET_SUBVOL_INFO` (added in 4.18) and `BTRFS_IOC_SNAP_DESTROY_V2` are absent from the 4.9 header
   (verified).
  They need hand-declared request numbers and struct layouts.
- Whether cinterop exposes `FICLONE` is unclear from the docs.
  It is an object-like macro that expands to the function-like `_IOW`.
  The interop page says constant macros become properties,
   while function-like macros need a `static inline` wrapper
   (<https://kotlinlang.org/docs/native-c-interop.html>,
   verified).
  A wrapper is hand-written C inside the `.def` file (unverified which path applies).
- Fallback:
   run `btrfs subvolume snapshot` and `cp --reflink=always` as child tasks.

### R5 hashing

- No official Kotlin crypto library exists.
- `com.appmattus.crypto:cryptohash`:
   BLAKE3 and xxHash3 in `commonMain`,
   with a `linuxX64()` target
   (`cryptohash/build.gradle.kts:61`;
   `cryptohash/src/commonMain/kotlin/com/appmattus/crypto/internal/core/blake3/Blake3.kt`,
   verified).
  Latest release 1.0.2,
   "Kotlin 2.0 support",
   on 2024-06-06;
   108 stars
   (verified).
  Pure Kotlin,
   so no SIMD;
   throughput on Kotlin/Native is unmeasured.
- `KotlinCrypto/hash` 0.8.0 (2025-09-19) ships blake2,
   md,
   sha1,
   sha2,
   and sha3 only
   (verified).
- Linking a system C hash library hits R1's sysroot wall:
   `/usr/lib64/libcrypto.so.3` requires `GLIBC_2.38` symbols
   (`objdump --dynamic-syms`,
   measured).

### R6 process supervision

- No kotlinx process API exists.
- `kmp-process` 0.5.0 (2025-12-16):
   one contributor with 175 commits,
   52 stars
   (verified).
  It has no cgroup or session hook,
   and when `posix_spawn` isn't used it runs Kotlin code in the child after `fork()`
   (`unixForkMain/.../PlatformBuilder.kt:143-151`,
   verified).
  The daemon therefore writes its own spawn code.
- Reaping:
   the pidfd from `pidfd_spawn` becomes readable in `epoll`,
   then `waitid(P_PIDFD, ...)`.
  `waitid` is present;
   `P_PIDFD` is declared by hand
   (measured absent).
- Signals and Ctrl+C,
   an unresolved risk:
  - `signalfd` needs the signal blocked in every thread.
  - Kotlin/Native runtime threads are `std::thread` members
     (`kotlin-native/runtime/src/main/cpp/concurrent/ScopedThread.hpp:94`,
     verified),
     and `ScopedThread.cpp` contains no signal-mask code
     (verified by reading the whole 28-line file).
  - Whether those threads exist before Kotlin `main` runs is unverified.
  - Running a `staticCFunction` as a `sigaction` handler is undocumented.
     The interop and function-pointer pages don't mention signals (verified).
  - A YouTrack search for "Native signal handler" found no guidance (verified).

### R7 file-enforcer

- A port means rewriting 81 non-test source files.
  They total 36,381 lines,
   of which 22,607 are the generated `src/data/packages.generated.ts`
   (measured),
   leaving about 13,774 handwritten lines.
- `file-enforcer.config.ts` is 2,329 lines of TypeScript with arbitrary logic
   (measured).
  The user relaxed FE01 so configuration need not be TypeScript
   (`doc/planning/monorepo-manager-route-research/from-scratch-inputs.md:281`,
   verified).
  Kotlin/Native still has no script host,
   so configuration becomes part of the daemon build or a data format.
- Keeping TypeScript turns this into Option 3.

### R8 repository fit

- No Kotlin/Native or Kotlin Multiplatform exists in the repository.
  65 `.kt` files:
   3 in `package/linter/kotlin`,
   62 in `package/music-player/android-app`
   (measured).
- Builds go through Gradle wrappers pinned to 9.5.1,
   with Kotlin JVM 2.4.0 in the linter
   (`package/linter/kotlin/build.gradle.kts:23`)
   and the Compose plugin 2.2.10 in the Android app
   (`package/music-player/android-app/build.gradle.kts:6`)
   (verified).
- Kotlin/Native adds a toolchain download to `~/.konan` on first build
   (101,793,914-byte GCC/glibc tarball plus LLVM;
   the tarball size is measured,
   LLVM is unmeasured).
- Recorded friction on the Mac:
   duplicate konan distributions needed `-Xklib-duplicated-unique-name-strategy=allow-first-with-warning`
   (`doc/decision/ios-iphone-x-vet-report/vet-test-frameworks.md:208-213`,
   verified).
- Build time:
   "compilation of release binaries takes an order of magnitude more time than debug binaries"
   (<https://kotlinlang.org/docs/native-improving-compilation-time.html>,
   verified).
  Repository docs record only JVM and Android Gradle times,
   from 3 s to 1 m 5 s
   (for example `doc/decision/kotlin-android-kopia-pcloud-vet-report/vet-android-e2e.md:255`,
   `doc/troubleshooting/detekt-alpha5-test-fixtures.md:168`,
   verified).
  No Kotlin/Native time is recorded.
- Concurrent Gradle invocations on one project collide on the Kotlin incremental cache
   (`doc/troubleshooting/kotlin-2-2-10-parallel-gradle-cache.md`,
   verified).
- Packaging:
   one ELF executable.
  It links `-Bstatic -lstdc++ -Bdynamic -ldl -lm -lpthread` against `/lib64/ld-linux-x86-64.so.2`
   (1.9.21 `konan.properties:555`,
   `:571`,
   verified).
- Lint:
   detekt `2.0.0-alpha.5` plus `RequireKDoc`
   (`package/linter/kotlin/build.gradle.kts:38`,
   verified),
   run per package by the root `lint:detekt` fan-out
   (`mise.no-env.toml:910-919`,
   verified).
  detekt parses source files without target-specific type resolution in this setup (inference).
- Testing:
   no mutation tester exists for Kotlin/Native
   (`doc/decision/ios-iphone-x-vet-report/vet-test-frameworks.md:222-229`,
   verified).
  Jazzer is JVM-only,
   and `kotlinx.fuzz` is uninstallable
   (`doc/decision/kotlin-android-kopia-pcloud-stack.md:386-399`,
   verified).
- Debugging:
   "Expression evaluation in debugger tools is not supported,
   and currently there are no plans for implementing it"
   (<https://kotlinlang.org/docs/native-debugging.html>,
   verified).

### R9 documentation

All pages loaded under plain `curl` (measured).
Triggers:

- <https://kotlinlang.org/docs/native-c-interop.html>:
   "C function pointer-typed parameters are mapped to `CValuesRef<T>`".
  <https://kotlinlang.org/docs/mapping-function-pointers-from-c.html> shows
  `accept_fun(f: CPointer<CFunction<(Int) -> Int>>?)` for a function-pointer parameter.
  The two pages contradict each other.
- The same interop page says:
   "Every C macro that expands to a constant is represented as a Kotlin property".
  Its example `#define FOO foo(42)` expands to a function call.
- <https://kotlinlang.org/docs/native-memory-manager.html>,
   updated 29 May 2026:
   the "Unit tests in the background" workaround calls `args.freeze()`.
  `freeze` is `@DeprecatedSinceKotlin(errorSince = "2.1")`
   (`kotlin-native/runtime/src/main/kotlin/kotlin/native/concurrent/Freezing.kt:36-38` at `v2.4.20`,
   verified),
   so following the page fails to compile on current Kotlin.
  The snippet also imports Apple-only `platform.CoreFoundation` under a generic heading.
- <https://kotlinlang.org/docs/native-definition-file.html>:
   the prose says "use `staticLibrary` and `libraryPaths` properties".
  The property table and example use `staticLibraries`.
  Platform suffixes appear as `.linux` or `.osx` and as `.linux_x64` or `.macos_x64`,
   with no statement relating them.
- <https://kotlinlang.org/docs/native-improving-compilation-time.html>:
   "`kotlin.native.disableCompilerDaemon=true` disables the Gradle daemon".
  The property names the compiler daemon.
- Missing coverage:
   no page read documents which glibc and kernel headers the Linux bindings use.
  That came from `konan.properties` source and YouTrack.
  <https://kotlinlang.org/docs/native-platform-libs.html> only says the POSIX contents "differ across platforms".
- <https://ktor.io/docs/server-sockets.html>:
   sockets use "`java.nio` under the hood",
   which is false on native (`SelectUtilsNix.kt:87` uses `pselect`).
  The page says "read a line ... using `ByteReadChannel.readUTF8Line`",
   and the code beside it calls `readLine()`.
  Unix domain sockets appear only as API signatures
   (<https://api.ktor.io/ktor-network/io.ktor.network.sockets/-unix-socket-address/index.html>
   has no class description).
- <https://kotlinlang.org/api/kotlinx-io/> shows version 0.8.1,
   while the latest GitHub release is `v0.9.1` (2026-06-26)
   (verified).
- Status labels:
   "Kotlin/Native interop with C and Objective-C Beta" and "cinterop klib binaries Beta"
   (components-stability page).
  `linuxX64` is Tier 2:
   "tested on CI to be able to compile but may not be automatically tested to be able to run"
   (<https://kotlinlang.org/docs/native-target-support.html>).
- X3 applies as well.

### R10 maturity

- Kotlin/Native compiler and runtime:
   Stable since 1.9.0 and 1.9.20;
   cinterop Beta;
   `linuxX64` Tier 2
   (verified).
  The toolchain is frozen at a 2021 tarball with no upgrade timeframe (KT-67046).
- `kotlinx.coroutines` 1.11.0 (2026-05-08),
   Stable (verified).
- `kotlinx.serialization` 1.11.0,
   Stable;
   1.12.0-RC on 2026-09-04 (verified).
- `kotlinx-io` 0.9.1,
   Alpha (verified).
- Ktor 3.5.2 (2026-08-04).
  Its sockets API is described as experimental,
   and the native selector has an `FD_SETSIZE` ceiling (verified).
- `kmp-process` 0.5.0,
   single maintainer (verified).
- `kfswatch` 1.4.0,
   primarily one maintainer (verified).
- `cryptohash` 1.0.2,
   27 months without a release (verified).
- Okio 3.18.2,
   active,
   last push 2026-09-17 (verified).
  It offers a native file system API,
   but no watching,
   processes,
   or sockets.

### Yikes

Ranked by severity.
The first two are disqualifying.

1. **Disqualifying (user documentation gate):**
    the Kotlin/Native interop and memory-manager docs contradict themselves.
   One page contradicts the function-pointer tutorial,
    and the other ships an error-level `freeze()` snippet.
   Ktor's sockets page misdescribes native and says nothing about Unix sockets.
   The kotlinx.serialization contradiction in X3 also applies.
2. **Disqualifying (engineering):**
    the linker sysroot is frozen at glibc 2.19 and Linux 4.9,
    with JetBrains stating no timeframe for a fix.
   Every modern API the daemon needs is missing from the bindings:
    `pidfd_spawn`,
    `setcgroup_np`,
    `POSIX_SPAWN_SETSID`,
    `P_PIDFD`,
    `pipe2`,
    `accept4`,
    `copy_file_range`,
    `statx`,
    `ucred`,
    and newer btrfs ioctls.
   Each becomes a `dlsym` function pointer or a hand-declared constant or struct.
   Linking any modern host C library fails,
    for example `libcrypto` needs `GLIBC_2.38`.
3. **Major:**
    the ecosystem gaps force the daemon to hand-write its epoll loop,
    spawn and reap code,
    inotify parser,
    socket server,
    and ioctl structs.
   Ktor's native selector has an `FD_SETSIZE` ceiling.
   The only BLAKE3 option is pure Kotlin and hasn't been released since June 2024.
4. **Major:**
    Ctrl+C and signal handling have no documented safe path.
   Runtime threads start with no signal-mask handling,
    so `signalfd` may be unreliable,
    and signal handlers written as `staticCFunction` are undocumented.
5. **Major:**
    tooling.
   The debugger has no expression evaluation and "no plans".
   There is no mutation testing or fuzzing.
   `linuxX64` is Tier 2,
    and release builds are "an order of magnitude" slower than debug builds.
   Kotlin/Native would be new to the repository.
6. **Moderate:**
    the non-TypeScript skill's FFI stop-and-ask and 15:1 comment ratio fall on a core that is mostly FFI (X2).
7. **Moderate:**
    the file-enforcer port is about 13,774 handwritten lines,
    and the 2,329-line configuration has no scripting host.
8. **Moderate:**
    the X1 compile-daemon hazard.

## Option 2: Kotlin/JVM, with a GraalVM Native Image variant

### Design sketch

- Runtime:
   JDK 25 LTS,
   where the Foreign Function and Memory (FFM) API is final.
  On JDK 21 the `Linker` API is still `@PreviewFeature(feature=PreviewFeature.Feature.FOREIGN)`
   (`java.base/java/lang/foreign/Linker.java:487` in the Temurin 21.0.12 `src.zip`,
   verified).
  The root pins `java = "temurin-21"`
   (`mise.no-env.toml:185`,
   verified).
  Kotlin 2.4.20 has `JVM_25` and `JVM_26` targets
   (`libraries/tools/kotlin-gradle-compiler-types/src/generated/kotlin/org/jetbrains/kotlin/gradle/dsl/JvmTarget.kt:47-49`,
   verified).
  Gradle supports Java 25 from 9.1.0
   (<https://docs.gradle.org/current/userguide/compatibility.html>,
   verified).
- The spawn layer is written on FFM,
   because `ProcessBuilder` can't place a child in a cgroup:
  - `Linker.nativeLinker().defaultLookup()` resolves `pidfd_spawn` and `posix_spawnattr_setcgroup_np`
     from the host glibc at run time,
     so there is no sysroot problem.
  - `posix_spawn_file_actions_adddup2` connects the child to pipes made with `pipe2(O_CLOEXEC)`.
  - The flags are `POSIX_SPAWN_SETCGROUP | POSIX_SPAWN_SETSID`.
  - One platform thread runs `epoll_wait` over pipes and pidfds,
     then reads through FFM into a `MemorySegment`.
  - The daemon runs with `--enable-native-access=ALL-UNNAMED`.
- RPC:
   `ServerSocketChannel.open(StandardProtocolFamily.UNIX)`,
   one virtual thread per client,
   NDJSON framing,
   and `jdk.net.ExtendedSocketOptions.SO_PEERCRED` for a UID check.
- Watching:
   inotify through FFM on a dedicated platform thread,
   not `WatchService`
   (see R2).
- Hashing:
   JDK `MessageDigest` SHA-256.
- Cache restore:
   `Files.copy`,
   which reflinks on btrfs (see R4).
  Snapshots use FFM `ioctl` or the `btrfs` CLI.
- Ctrl+C:
   a `Runtime.addShutdownHook` writes `cgroup.kill` to every task cgroup and waits for `populated 0`.
- GraalVM variant:
   the same code compiled with `native-image`,
   with FFM downcall descriptors registered in `reachability-metadata.json`.

### R1 per-task cgroup v2

- `ProcessImpl` launches every child through `posix_spawn` of `jspawnhelper` by default
   (`java.base/java/lang/ProcessImpl.java:86-101`,
   `:126`,
   `:295`,
   verified).
  It has no hook between spawn and exec.
- The fetched `ProcessBuilder` javadoc has 0 matches for "cgroup",
   "process group",
   or "setsid".
  The positive control "redirect" matched (measured).
- The `java.base` Java sources have 0 lines mentioning `pidfd`,
   `clone3`,
   or `cgroup.procs`.
  The positive control "cgroup" matched 406 lines,
   all container metrics (measured).
- FFM can call `pidfd_spawn` directly.
  The glibc child path runs no JVM code (see "Shared design").
  The JDK's own default launch mechanism already calls `posix_spawn` from inside the JVM
   (`ProcessImpl.java:97-101`,
   verified).
- A spawned child's pipe fds can't be wrapped as JDK streams through public API.
  `FileDescriptor` exposes only the invalid `public FileDescriptor()` constructor;
   setting the int fd is internal through `SharedSecrets`
   (`java.base/java/io/FileDescriptor.java:72`,
   `:127`,
   verified).
  So I/O,
   reaping (`waitid(P_PIDFD)`),
   and signaling (`pidfd_send_signal`) are all FFM code.
- Reaping doesn't conflict with the JDK:
   JDK reaper threads wait per PID through `waitForProcessExit0(pid, reap)`,
   never `wait(-1)`
   (`java.base/java/lang/ProcessHandleImpl.java:140-190`,
   verified).
- Variant without FFM:
   `ProcessBuilder` launches `systemd-run --user --scope -p Delegate=... --expand-environment=no -- cmd`.
  Tasks then become systemd scope units under `app.slice`,
   not leaves of the daemon's delegated subtree.
  Freeze uses `systemctl --user freeze`,
   documented as "Added in version 246" in local `man systemctl`
   (verified),
   and the man page adds "Unit is automatically thawed just before we execute a job against the unit".
  The per-task D-Bus round trip is unmeasured.
  Children also stay in the JVM's process group,
   because `systemd-run --scope` execs in place (run.c:2727).
- Verdict:
   feasible with documented,
   final JDK 25 APIs.
  The glibc spawn-cgroup function remains undocumented ("Shared design every option must implement"),
   and the JDK's process machinery is reimplemented in FFM.

### R2 recursive watching

- `sun.nio.fs.LinuxWatchService` is inotify.
  `register` adds one watch per directory
   (`java.base/sun/nio/fs/LinuxWatchService.java:206-276`,
   verified).
  Modifiers other than sensitivity return `UnsupportedOperationException`,
   so there is no `FILE_TREE` recursion
   (`:228-236`,
   verified).
  `ENOSPC` becomes "User limit of inotify watches reached"
   (`:256-257`,
   verified).
- Overflow:
   `IN_Q_OVERFLOW` signals `OVERFLOW` to every key
   (`:396-401`,
   verified).
  `IN_IGNORED` invalidates the key
   (`:410-414`,
   verified).
- Each registration is a request handed to the single poller thread through a socketpair wakeup
   (`java.base/sun/nio/fs/AbstractPoller.java:216-230`,
   verified),
   so 5,480 registrations are 5,480 cross-thread round trips (cost unmeasured).
- Failure mode:
   on any `UnixException` other than `EAGAIN`,
   the poller loop ends with `x.printStackTrace()`
   (`LinuxWatchService.java:301-367`,
   verified).
  After that the service stops delivering events,
   and no key is cancelled
   (inference from the loop exit).
- The `WatchService` javadoc says detection details are "highly implementation specific"
   and names no Linux mechanism
   (<https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/nio/file/WatchService.html>,
   verified).
  The behavior above comes only from source.
- Design choice:
   inotify through FFM,
   with the same algorithm as Option 1 R2 but no binding gaps.
- `gmethvin/directory-watcher` last pushed 2025-06-23 (verified)
   and wraps `WatchService` on Linux (unverified),
   so it inherits the same issues.

### R3 Unix socket JSON-RPC server

- `UnixDomainSocketAddress` is documented since 16
   (<https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/net/UnixDomainSocketAddress.html>,
   verified).
  The JDK 25 `SocketChannel` javadoc lists Unix domain socket options
   (verified).
- `SO_PEERCRED` returns a `UnixDomainPrincipal`,
   `@since 16`
   (`jdk.net/jdk/net/ExtendedSocketOptions.java:186-200`,
   verified).
- No Ktor is needed.
  Ktor's JVM `UnixSocketAddress` detects the JDK class reflectively
   (`ktor-network/jvm/src/io/ktor/network/sockets/SocketAddressJvm.kt:140`,
   verified).
- JSON:
   kotlinx.serialization (X3 trigger) or a JDK-only codec.
  Jackson's documentation was not assessed.

### R4 btrfs

- Detection:
   `Files.getFileStore(path).type()` returned `btrfs` for the checkout and `cgroup2` for `/sys/fs/cgroup`
   (measured).
  It reads `/etc/mtab`
   (`java.base/sun/nio/fs/LinuxFileSystem.java:126-127`,
   verified),
   which is `../proc/self/mounts` here (measured).
- Reflink without FFM:
   on Linux,
   `Files.copy` calls `copy_file_range`,
   found at run time with `dlsym(RTLD_DEFAULT, "copy_file_range")`
   (`src/java.base/linux/native/libnio/fs/LinuxNativeDispatcher.c:81-82`,
   `:193-221`,
   verified).
  The kernel's `vfs_copy_file_range` uses `remap_file_range` for same-superblock copies
  when the file system lacks `->copy_file_range`
   (`fs/read_write.c:1553-1605`,
   torvalds master,
   verified).
  btrfs provides only `.remap_file_range = btrfs_remap_file_range`
   (`fs/btrfs/file.c:3817-3837`,
   verified).
  So `Files.copy` inside one btrfs file system clones
   (inference from source;
   not run,
   because the scratchpad is `tmpfs`).
  If the clone is rejected,
   the kernel silently falls back to splice copy,
   so a reflink can't be required or detected this way.
- Subvolumes and snapshots:
   FFM `ioctl`,
   variadic,
   with `Linker.Option.firstVariadicArg`
   (Linker javadoc,
   "Variadic functions",
   verified)
   and hand-built `MemoryLayout` structs.
  Alternatively,
   the `btrfs` CLI as a task.

### R5 hashing

- Measured on JDK 21.0.12,
   on a 256 MiB buffer,
   over 7 runs after a warm-up:
  - JDK `MessageDigest` SHA-256:
     median 2,062 MiB/s,
     band 2,016 to 2,069.
  - `commons-codec` 1.17.1 `Blake3`:
     median 312 MiB/s,
     band 306 to 314.
  The bands don't overlap,
   so SHA-256 through the JDK is the right choice.
- xxHash:
   `dynatrace-oss/hash4j` 0.30.0 (2026-03-09),
   pushed 2026-09-16;
   `OpenHFT/Zero-Allocation-Hashing`,
   pushed 2026-09-15
   (verified).
  Their throughput is unmeasured.

### R6 process supervision

- With the FFM spawn layer,
   `ProcessHandle` is only a convenience for listing descendants.
  Its `destroy` docs say termination "is implementation dependent"
   (<https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/ProcessHandle.html>,
   verified).
- Streaming uses FFM `epoll_wait` plus `read` on one platform thread.
  Blocking FFM reads from virtual threads would pin a carrier
   (unverified for JDK 25).
- Ctrl+C:
   "The Java Virtual Machine initiates the shutdown sequence ... when some external event occurs,
   such as an interrupt or a signal is received from the operating system".
  The same javadoc says hooks "should also finish their work quickly"
   (<https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Runtime.html>,
   verified).
  A long `cgroup.kill` wait in a hook conflicts with that advice.
- Parallelism default:
   `Runtime.availableProcessors()` returned 16 (measured).
  Its handling of cgroup CPU quota is recalled from HotSpot container support (unverified).

### R7 file-enforcer

- A port has the same size as Option 1 R7.
- A JVM script host is possible,
   but the scripting stability docs contradict each other (X3).

### R8 repository fit

- Closest to the existing Kotlin linter:
   Kotlin/JVM with Gradle 9.5.1.
- Requires JDK 25 alongside the repository-wide `temurin-21`.
  The linter notes "only JDK 21 is provisioned"
   (`package/linter/kotlin/build.gradle.kts:61`,
   verified).
  AGP 9.2.1 on JDK 25 is unverified;
   a per-package pin or a Gradle toolchain avoids touching the Android build (unverified).
- mise offers `temurin-25.0.4+7.0.LTS`,
   `oracle-graalvm-25.0.4.1.1`,
   and `mandrel-25.0.4.1-Final+java25`
   (`mise ls-remote java`,
   measured).
- Packaging:
   a jar plus a JRE (mise),
   or a `jlink` image.
  The GraalVM variant produces one executable.
- Startup on this host:
   a hello-world class started in a median 20.9 ms
   (band 20.2 to 24.9,
   15 runs).
  `node -e 0` took a median 19.5 ms
   (band 18.0 to 24.6)
   (measured).
  The bands overlap,
   so this probe resolves no startup difference.
  A real Kotlin client loading coroutines and serialization is unmeasured.
- Testing:
   JUnit 5,
   Kotest property tests,
   Jazzer,
   and Pitest are all verified in the repository's Android vet.
- Kotlin FFM calls use signature-polymorphic `MethodHandle.invokeExact`,
   supported since Kotlin 1.3.20 (KT-14416 "Fixed",
   verified).
  <https://kotlinlang.org/docs/java-interop.html> has 0 mentions of `invokeExact`,
   `PolymorphicSignature`,
   `MethodHandle`,
   or "foreign" (measured),
   so the Kotlin-side typing rules come from outside the Kotlin docs.

### R9 documentation

- JDK javadoc pages loaded with text present.
  The "JavaScript is disabled" banner is cosmetic.
  The `Linker` javadoc and the FFM guide index
   (<https://docs.oracle.com/en/java/javase/25/core/foreign-function-and-memory-api.html>)
   and the errno page
   (<https://docs.oracle.com/en/java/javase/25/core/checking-native-errors-using-errno.html>)
   read coherently in the sections consumed.
- Triggers:
  - `WatchService` Linux behavior is undocumented;
     only JDK source explains it.
     Avoidable by using FFM inotify.
  - X3 (kotlinx.serialization,
     Kotlin Gradle plugin,
     scripting,
     Gradle).
  - The glibc spawn-cgroup documentation gap ("Shared design every option must implement").
- GraalVM variant triggers:
  - <https://www.graalvm.org/latest/reference-manual/native-image/native-code-interoperability/foreign-interface/>
     returns a meta-refresh stub to `/reference-manual/.../ffm-api/`,
     which returns another meta-refresh stub to `/latest/.../ffm-api/`.
    Plain `curl` gets only "Redirecting…" twice (measured).
  - On the final page,
     the upcall example registers `"class": "org.example.QSortInvoke$Qsort"`,
     while the full example declares `public class InvokeQsort`
     (measured).
  - The same page says "The default lookup (`Linker.defaultLookup()`) is currently not supported in static executables".
  - The Oracle blog <https://blogs.oracle.com/java/detaching-graalvm-from-the-java-ecosystem-train> returned 403 to
     `curl`.
     It was read as a fetched page.

### R10 maturity

- JDK 25 LTS:
   FFM final since JDK 22.
- `commons-codec` pushed 2026-09-14,
   JNA pushed 2026-09-01
   (verified).
- GraalVM:
   Oracle wrote on 2025-09-15 that "GraalVM Early Adopter technology,
   including Native Image,
   is being discontinued for Java SE Product customers".
  It added that "The GraalVM team are transitioning to focus on non-Java Graal Languages"
   (fetched page).
  GraalVM Community still ships:
   "GraalVM Community 25 Innovation 3 (graal 25.3.4.1, jdk 25.0.4.1)",
   2026-08-25
   (verified).
  Every 2026 release is on a JDK 25 base,
   and `oracle/graal` was pushed 2026-09-16 (verified).
  Reachability metadata for `kotlinx-coroutines-core`
  and `kotlinx-serialization-json` exists in `oracle/graalvm-reachability-metadata` (verified).

### Yikes

Ranked by severity.

1. **Disqualifying (user documentation gate):**
    the X3 contradictions.
   The Kotlin Gradle plugin page can't be avoided when building Kotlin,
    and it contradicts itself on the exact daemon-reuse behavior X1 needs.
   kotlinx.serialization is avoidable only by giving up the Kotlin-native JSON library.
2. **Major:**
    the JVM's process API can't sandbox.
   Spawn,
    pipes,
    reaping,
    and signaling are rewritten in FFM,
    because raw fds can't become JDK streams through public API.
   This requires adding JDK 25 next to the repository's JDK 21.
   The race-free spawn function it relies on is documented only in glibc `NEWS`.
3. **Major:**
    `WatchService` isn't usable as specified.
   It has no recursion,
    one poller thread handles every registration,
    and it dies silently on an unexpected error.
   Replacing it adds FFM inotify code.
4. **Major (GraalVM variant only):**
    Oracle moved the GraalVM team away from Java and discontinued Native Image for Java SE customers.
   Community releases continue on JDK 25 only.
   The FFM Native Image page has a class-name mismatch and meta-refresh redirects.
   Each downcall shape needs build-time registration.
5. **Moderate:**
    the file-enforcer port,
    as in Option 1.
   The scripting route has contradictory stability labels.
6. **Moderate:**
    the non-TypeScript skill's FFI stop-and-ask (X2) over the spawn,
    inotify,
    and ioctl layers.
   This is smaller than Option 1,
    because sockets,
    files,
    hashing,
    threads,
    and JSON stay on the JDK.
7. **Moderate:**
    the X1 compile-daemon hazard.
8. **Minor:**
    pure-JVM BLAKE3 runs at 312 MiB/s,
    so the cache uses SHA-256.

## Option 3: Kotlin daemon core with TypeScript file-enforcer children

### Design sketch

- Runtime choice:
   the Option 2 JVM core on JDK 25 FFM.
  A Kotlin/Native core would carry every yike of Option 1
   except the file-enforcer port.
- file-enforcer runs as ordinary daemon tasks:
   `node package/file-enforcer/core/src/cli.ts --evaluate <config>` in a task cgroup,
   placed with the same `pidfd_spawn` path.
- The child writes typed NDJSON events on fd 3:
   `batchAccepted`,
   `runStarted`,
   `runFinished` with destinations written and protected reverts,
   `watcherFailure`.
  The daemon forwards them to RPC subscribers.
- The daemon owns watching.
  file-enforcer's own chokidar watch mode isn't started;
   the daemon reruns evaluation children for changed inputs.
- The protocol is a versioned JSON schema checked by tests on both sides,
   per the research's "Protocol documentation is a deliverable".

### R1 to R6 and R8 to R10

- Same as Option 2
   (or Option 1,
   if the core is Kotlin/Native).
- One addition under R6:
   Node children receive `POSIX_SPAWN_SETSID` like any other task.
  A frozen evaluation child keeps holding file-enforcer's staleness manifest lock,
   which is a `mkdir` lock with a 5 s timeout for other writers
   (`from-scratch-inputs.md:132-136`,
   `:176-178`,
   verified).
  A paused file-enforcer task therefore fails other file-enforcer tasks after 5 s,
   unless the scheduler serializes them or excludes them from pause (design decision needed).

### R7 file-enforcer interop

- Reuse is direct:
   no port.
- Required TypeScript changes:
  - `cli.ts` parses only `--watch` and one positional argument
     (`from-scratch-inputs.md:58-61`,
     verified).
  - A child mode with typed events is new work,
     because `LogRecord` has no structured fields (`:166-169`,
     verified).
  - `startWatching` has no stop handle (`:70-75`,
     verified).
- Per-evaluation cost:
   `node -e 0` median 19.5 ms (measured).
  Loading the configuration and running file-enforcer are unmeasured.
  The probe couldn't run them,
   because the root configuration writes repository files.
- The configuration keeps TypeScript,
   so the `AD2` config-as-code rule and the `TSD` and `TLG` rules keep applying unchanged
   (`AGENTS.md`).

### Yikes

Ranked by severity.

1. **Disqualifying (user documentation gate):**
    the X3 contradictions,
    inherited from the Kotlin core,
    plus Option 2's JVM-specific triggers.
2. **Major:**
    everything from Option 2's second and third yikes:
    the FFM process layer on JDK 25,
    glibc `NEWS`-only spawn documentation,
    and replacing `WatchService`.
3. **Moderate:**
    two languages and three runtimes in the operational path.
   The JVM daemon,
    Node children,
    and Gradle plus Kotlin compile daemons for Kotlin tasks all need a versioned protocol with tests on both sides.
   file-enforcer needs a new child mode and a typed event stream.
4. **Moderate:**
    pausing a file-enforcer child stalls other writers through the 5 s manifest lock timeout.
   This follows from the design,
    but it becomes concrete here.
5. **Moderate:**
    the X1 compile-daemon hazard and the X2 FFI comment burden on the JVM core.
6. **Minor:**
    each evaluation pays a Node process start (at least 19.5 ms median measured).

## Cross-option comparison

- **Fewest and least severe yikes:
   Option 3 on a JVM core.**
  It has one disqualifying-class item,
   the documentation gate,
   and every Kotlin option shares that item.
  It avoids the file-enforcer port that Options 1 and 2 carry.
  It avoids every Kotlin/Native-specific engineering blocker.
- Option 3 over Option 2:
   both carry the same JVM core yikes.
  Option 2 adds a port of about 13,774 handwritten lines,
   plus a 2,329-line configuration with no documented stable script host.
  Option 3 instead adds a documented child boundary and the pause-lock interaction.
  The boundary is new work,
   but it needs no port,
   and the research already recommends child-process evaluation.
- Option 2 over Option 1:
   the JVM resolves modern glibc symbols at run time through FFM.
  Unix sockets,
   peer credentials,
   SHA-256 at a measured 2,062 MiB/s,
   file store type,
   and btrfs cloning through `Files.copy` all use JDK APIs.
  The JVM testing stack (Jazzer, Pitest) is already verified in the repository.
  Kotlin/Native has two disqualifying-class items, not one:
   the documentation gate and the frozen glibc 2.19 sysroot with no fix timeframe.
  Its core is mostly hand-written FFI with unresolved signal handling,
   and its debugger can't evaluate expressions.
- Option 1 is last.
  Kotlin/Native is the option the user raised,
   and designing it through surfaced the sysroot problem.
  Every Linux API the daemon's sandbox and watcher need newer than 2014-era glibc
   must come in through `dlsym` or hand-declared layouts.
  JetBrains ties any fix to a cinterop rework with no timeframe (KT-67046).
- Under the user's strict documentation rule,
   all three Kotlin options are culled by X3.
  This report didn't probe Rust or TypeScript documentation the same way,
   so it can't say whether the rule separates Kotlin from those stacks.
  That comparison needs the same probe run against their docs.
- Findings that hold for any stack chosen:
  - X1:
     Kotlin compile daemons defeat the "Gradle daemon disabled" decision
     unless each Gradle project sets `kotlin.compiler.execution.strategy=in-process`.
  - The race-free glibc spawn-cgroup API is documented only in `NEWS` and a header.
  - `systemd-run --scope` expands `$` in argv by default.

## Probe log

Commands run in this session, with the load-bearing output.

- `mise which java --cd /var/home/user/Monochromatic`
   printed `.../java/temurin-21.0.12+8.0.LTS/bin/java`;
   `ls ~/.konan` printed `kotlin-native-prebuilt-linux-x86_64-1.9.21`.
- `ldd --version`:
   `ldd (GNU libc) 2.43`.
- `nm --dynamic --defined-only /lib64/libc.so.6 | rg 'pidfd|setcgroup|clone3|posix_spawn_file_actions_addchdir'`:
   `pidfd_spawn@@GLIBC_2.39`,
   `posix_spawnattr_setcgroup_np@@GLIBC_2.39`,
   `pidfd_open@@GLIBC_2.36`,
   `posix_spawn_file_actions_addchdir_np@@GLIBC_2.29`.
- `cat /proc/sys/fs/inotify/{max_user_watches,max_user_instances,max_queued_events}`:
   `524288`,
   `8192`,
   `16384`.
- `node scratchpad/probe/klib-symbols.ts ~/.konan/.../klib/platform/linux_x64` (133 files):
  - Present:
     `inotify_init1`,
     `inotify_add_watch`,
     `IN_Q_OVERFLOW`,
     `epoll_create1`,
     `signalfd`,
     `eventfd`,
     `posix_spawn`,
     `waitid`,
     `statfs`,
     `ioctl`,
     `syscall`,
     `dlsym`,
     `sockaddr_un`,
     `SO_PEERCRED`.
  - Absent:
     `pipe2`,
     `accept4`,
     `POSIX_SPAWN_SETSID`,
     `posix_spawn_file_actions_addchdir_np`,
     `posix_spawnattr_setcgroup_np`,
     `pidfd_spawn`,
     `pidfd_open`,
     `P_PIDFD`,
     `statx`,
     `copy_file_range`,
     `SYS_pidfd_open`,
     `ucred`.
- `curl --head --location https://download.jetbrains.com/kotlin/native/x86_64-unknown-linux-gnu-gcc-8.3.0-glibc-2.19-kernel-4.9-2.tar.gz`:
   `content-length: 101793914`,
   `last-modified: Wed, 12 May 2021 20:26:29 GMT`.
- `objdump --dynamic-syms /usr/lib64/libcrypto.so.3 | rg --only-matching 'GLIBC_2\.[0-9]+' | sort --version-sort --unique | tail`:
   highest `GLIBC_2.38`.
- `node scratchpad/probe/startup.ts` (15 runs each after one warm-up):
   `javaHello` median 20.9 ms (20.2 to 24.9);
   `javaHelloSerialGc` median 18.7 ms (17.8 to 20.9);
   `nodeNoop` median 19.5 ms (18.0 to 24.6).
- `java -Xmx1g -cp commons-codec-1.17.1.jar scratchpad/probe/HashBench.java`:
   `sha256-jdk MiB/s min=2016 median=2062 max=2069`;
   `blake3-commons-codec MiB/s min=306 median=312 max=314`.
- `java scratchpad/probe/StoreType.java /var/home/user/Monochromatic /sys/fs/cgroup`:
   `type=btrfs`,
   `type=cgroup2`,
   `availableProcessors=16`.
- `mise ls-remote java`:
   includes `temurin-25.0.4+7.0.LTS`,
   `oracle-graalvm-25.0.4.1.1`,
   `mandrel-25.0.4.1-Final+java25`.
- Kotlin daemon logs:
   43 files matching `/tmp/kotlin-daemon*`.
  The newest log shows version `2.2.10-release-430`,
   `--daemon-autoshutdownIdleSeconds=7200`,
   port `17864`,
   and `initiate elections`.
  `pgrep` found no Kotlin or Gradle daemon running at probe time.
- Documentation fetches:
   `node scratchpad/probe/fetch-docs.ts <batch.json> <outDir>`,
   one `curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0'` per page.
  Results:
  - Every kotlinlang.org,
     ktor.io,
     api.ktor.io,
     docs.oracle.com,
     and docs.gradle.org page returned 200 with text.
  - freedesktop.org systemd man pages returned 418 go-away challenges.
  - graalvm.org `foreign-interface` and the unversioned `ffm-api` URL returned meta-refresh stubs;
     `/latest/.../ffm-api/` returned 200 with text.
  - blogs.oracle.com returned 403.
- Source clones:
   `~/temp/agent/ktor-2026-09-16`,
   `~/temp/agent/kmp-process-2026-09-16`,
   `~/temp/agent/kotlinx-io-2026-09-16`
   (`gh repo clone ... -- --depth 1`).
  Single files came through `gh api .../contents/...` from JetBrains/kotlin at `v2.4.20`,
   torvalds/linux at `v4.9` and master,
   bminor/glibc at `glibc-2.39`,
   openjdk/jdk21u,
   systemd/systemd,
   and Kotlin/kotlinx.serialization.
  JDK Java sources came from the Temurin 21.0.12 `lib/src.zip`.

## Sources

- Kotlin/Native docs:
  - <https://kotlinlang.org/docs/native-c-interop.html>
  - <https://kotlinlang.org/docs/mapping-function-pointers-from-c.html>
  - <https://kotlinlang.org/docs/native-definition-file.html>
  - <https://kotlinlang.org/docs/native-platform-libs.html>
  - <https://kotlinlang.org/docs/native-target-support.html>
  - <https://kotlinlang.org/docs/native-memory-manager.html>
  - <https://kotlinlang.org/docs/native-improving-compilation-time.html>
  - <https://kotlinlang.org/docs/native-debugging.html>
- Kotlin build and stability docs:
  - <https://kotlinlang.org/docs/gradle-compilation-and-caches.html>
  - <https://kotlinlang.org/docs/compiler-execution-strategy.html>
  - <https://kotlinlang.org/docs/components-stability.html>
  - <https://kotlinlang.org/docs/custom-script-deps-tutorial.html>
  - <https://kotlinlang.org/docs/java-interop.html>
- kotlinx docs:
  - <https://kotlinlang.org/docs/serialization-json-elements.html>
  - <https://kotlinlang.org/api/kotlinx-io/>
  - <https://kotlinlang.org/api/kotlinx.coroutines/kotlinx-coroutines-core/kotlinx.coroutines/new-single-thread-context.html>
- Ktor:
  - <https://ktor.io/docs/server-sockets.html>
  - <https://api.ktor.io/ktor-network/io.ktor.network.sockets/-unix-socket-address/index.html>
- YouTrack:
  - KT-67046,
     KT-85157,
     KT-14416,
     through `https://youtrack.jetbrains.com/api/issues/<id>`.
- JDK:
  - <https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/nio/file/WatchService.html>
  - <https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/ProcessBuilder.html>
  - <https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/ProcessHandle.html>
  - <https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/net/UnixDomainSocketAddress.html>
  - <https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Runtime.html>
  - <https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/lang/foreign/Linker.html>
  - <https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/nio/channels/SocketChannel.html>
  - <https://docs.oracle.com/en/java/javase/25/core/foreign-function-and-memory-api.html>
  - <https://docs.oracle.com/en/java/javase/25/core/checking-native-errors-using-errno.html>
- GraalVM:
  - <https://www.graalvm.org/latest/reference-manual/native-image/native-code-interoperability/ffm-api/>
  - <https://www.graalvm.org/latest/reference-manual/native-image/>
  - <https://blogs.oracle.com/java/detaching-graalvm-from-the-java-ecosystem-train> (fetched page)
- Gradle:
  - <https://docs.gradle.org/current/userguide/gradle_daemon.html>
  - <https://docs.gradle.org/current/userguide/compatibility.html>
- Linux and glibc:
  - <https://man7.org/linux/man-pages/man3/posix_spawn.3.html>
  - <https://man7.org/linux/man-pages/man2/clone.2.html>
  - glibc 2.39 `NEWS`,
     `posix/spawn.h`,
     `sysdeps/unix/sysv/linux/spawni.c`,
     and `manual/process.texi`,
     from the `bminor/glibc` mirror.
     sourceware.org returned an Anubis bot challenge to `curl`.
- systemd:
  - local `man systemd-run` and `man systemctl`
  - `systemd/systemd` `src/run/run.c`
