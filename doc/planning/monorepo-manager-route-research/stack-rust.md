# Rust stack options for the from-scratch monorepo manager

Outcome on 2026-09-16:

- Option 2 is out:
   its TypeScript file-enforcer children need Node inside the single file,
   which the user ruled too big.
- Option 1 is the remaining route.
- This file's conclusions that task spawning must be repository-written `unsafe` code
   and that the watcher and JSON-RPC framing must be hand-written
   are corrected by `stack-rust-crates.md`.
- Configuration hosting for Option 1 is being redesigned;
   see "Current stack direction" in `doc/planning/monorepo-manager-from-scratch-design.md`.

Research date: 2026-09-16.
Scope: read-only design deep dive for two options from
`doc/planning/monorepo-manager-from-scratch-design.md`,
"Tech stack options":

- Option 1:
   all Rust,
   with file-enforcer rewritten in Rust.
- Option 2:
   a Rust daemon core,
   with the TypeScript file-enforcer run as Node child processes.

Nothing under `/var/home/user/Monochromatic` was modified.
No cgroup was created,
nothing was installed,
and no benchmark load was run.

## Conventions

- **Verified** means one of:
   a file and line range read in this session;
   a documentation page loaded with the user's curl command and the quoted text found in it;
   or a probe whose command and output are recorded in "Probes run".
- **Unverified** means inference,
   arithmetic on published figures,
   or recall.
- Path abbreviations:
  - `STD/` is
     `~/.rustup/toolchains/nightly-2026-09-16-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/`
     (`rust-src` of `rustc 1.100.0-nightly (215a8af4b 2026-09-15)`).
  - `REG/` is `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`
     (crates already downloaded on this host).
  - `SCR/` is this session's scratchpad directory
     `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad/`.
- Shallow clones in `~/temp/agent/`:
  - `cgroups-rs-2026-09-16`,
     kata-containers/cgroups-rs at `595c3da4` (2026-08-20).
  - `notify-2026-09-16`,
     notify-rs/notify at `141edc48` (2026-09-15),
     which is `notify 9.0.0-rc.5`.
  - `jsonrpsee-2026-09-16`,
     paritytech/jsonrpsee at `4260c35d` (2026-08-03).
  - `reflink-copy-2026-09-16`,
     cargo-bins/reflink-copy at `eb46449f` (2026-08-01).
- Single files saved under `SCR/rust-stack/src/`:
   Linux `kernel/cgroup/cgroup.c` and `include/uapi/linux/btrfs.h`
   from torvalds/linux `master`,
   which was commit `238650ef` when fetched.
- Documentation pages loaded with the user's curl command are saved under `SCR/rust-stack/docs/`.
- Documentation cull threshold applied:
   a page triggers a cull when it misstates behavior,
   contradicts its implementation or itself,
   or omits behavior that changes safety or correctness for this design.
  Typos in example strings are noted without culling.
  A cull removes the named API or crate from the design;
   it cannot remove an unavoidable transitive dependency such as `libc`.
  The user can veto this threshold.

## Probes run

Each item lists the command or script,
then its output.

- Toolchain:
   `mise exec rust -- rustc --version --verbose`
   printed `rustc 1.100.0-nightly (215a8af4b 2026-09-15)`,
   `LLVM version: 23.1.1`.
  Process note:
   mise printed "installing 1 tool" and rustup reported the toolchain "unchanged";
   later probes called the toolchain binary by path to avoid that check.
- inotify limits:
   `cat /proc/sys/fs/inotify/max_queued_events /proc/sys/fs/inotify/max_user_watches /proc/sys/fs/inotify/max_user_instances`
   printed `16384`,
   `524288`,
   `8192`.
- CPU:
   `rg --max-count 1 '^model name' /proc/cpuinfo` printed `AMD Ryzen 7 8700F 8-Core Processor`;
   the flags line contains `avx2`,
   `avx512f`,
   `avx512vl`,
   and `sha_ni`.
- Delegated cgroup file ownership:
  - `cat /proc/self/cgroup` printed
     `0::/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash`.
  - `stat --format='%U %a %n'` printed `user 755` for `user@1000.service`
     and `user 644` for `cgroup.procs` and `cgroup.subtree_control`
     in both `user@1000.service` and `user@1000.service/app.slice`.
    So the systemd-managed ancestors `user@1000.service` and `app.slice` are writable by this user.
- `libc` constant probe:
   `SCR/rust-stack/probe/clone_flag.rs` mirrors the `libc 0.2.189` declaration
   `#[allow(overflowing_literals)] const X: i32 = 0x200000000;`,
   compiled with the nightly `rustc` by path.
  Output:
   `mirrored i32 value = 0`,
   `widened to u64 = 0x0`,
   `kernel value = 0x200000000`.
- Node child startup floor,
   `SCR/rust-node-startup.ts`,
   10 sequential runs each:
  - `node --eval 0`:
     min 19.0 ms,
     median 20.7 ms,
     max 21.7 ms.
  - `node` importing `package/dev-script/file-enforcer/src/index.ts` without running a config:
     min 143.3 ms,
     median 146.8 ms,
     max 160.0 ms.
  - Afterward `git status --short` still showed only the pre-existing ` M mise.lock`,
     and `find ... node_modules/.cache/file-enforcer ... -mmin -20 -print` printed nothing.
- Content-hash input volume,
   `SCR/rust-input-volume.ts`
   (`rg --files --hidden --glob '!.git'` plus `lstat`):
   8,076 files,
   151,339,450 bytes (144.3 MiB);
   median 4,039 bytes;
   99th percentile 238,724 bytes;
   max 22,908,459 bytes;
   152 files at or above 128 KiB;
   12 files at or above 1 MiB holding 51.2 MiB.
  Cross-check:
   `git ls-files | wc --lines` printed `8083`.
- file-enforcer size,
   `SCR/rust-fe-count.ts`,
   described under "Option 1",
   "R7 file-enforcer interop".
- Root config logic,
   `SCR/rust-config-logic.ts`,
   described under "Option 1",
   "R7 file-enforcer interop".
- Crate registry metadata,
   `SCR/rust-crates-meta.ts`,
   and GitHub metadata,
   `SCR/rust-gh-meta.ts`,
   described under "Shared Rust core evidence",
   "R10 maturity and maintenance".
- Documentation loads,
   `SCR/rust-docs-curl.ts`,
   plus single curl loads,
   described under "Shared Rust core evidence",
   "R9 documentation quality".
- btrfs user-space pieces:
   `ls /usr/lib64/ | rg 'btrfsutil'` printed `libbtrfsutil.so.1`,
   `libbtrfsutil.so.1.4`,
   `libbtrfsutil.so.1.4.0`
   (no unversioned development symlink);
   `command -v btrfs` printed `/usr/bin/btrfs`.

Not run,
with reasons:

- Creating a cgroup,
   spawning into it,
   freezing,
   or killing:
   the caller forbade creating cgroups.
  Every cgroup behavior below is therefore from kernel docs and source,
   unverified by experiment on this host.
- A BLAKE3 throughput benchmark:
   AGENTS.md `RXI` and `BOX` route benchmark runs into a bounded container,
   and a container creates cgroups.
- A `cargo build` of a daemon prototype.

## Shared Rust core design

Both options use the same Rust daemon core.
They differ in file enforcement ("R7 file-enforcer interop" in each option) and what follows from it.

### Components

- `cgroup`:
   hand-written over `std::fs` against the kernel's cgroup v2 files;
   no cgroup crate.
- `supervise`:
   `std::process::Command` with an `unsafe` `pre_exec` closure that joins the task cgroup,
   `process_group(0)`,
   a pidfd from `rustix::process::pidfd_open`,
   and `tokio::io::unix::AsyncFd` for exit readiness.
- `watch`:
   the `inotify` crate directly,
   one non-recursive watch per non-ignored directory,
   walked with `ignore::WalkBuilder`.
- `rpc`:
   newline-delimited JSON-RPC 2.0 over a filesystem Unix socket,
   `serde_json` message types,
   and one sequence-numbered event log serving every subscription.
- `cache`:
   `blake3` content hashes without its `mmap` feature,
   plus reflink restore through `rustix::fs::ioctl_ficlone`.
- `btrfs`:
   `statfs` detection and optional subvolume ioctls.
- `schedule`:
   a priority queue with in-place priority changes,
   concurrency from `MONOCHROMATIC_JOBS` or a cached `std::thread::available_parallelism()`.
- `doctor`:
   the checks listed in the design doc's "Doctor" section.

AGENTS.md `MXR` limits each `.rs` file to 300 code lines,
so each component is a directory of sibling modules.

### Startup sequence

1. The user runs `systemd-run --user --scope -p Delegate=yes -- <tool> daemon` in a terminal.
2. The daemon reads `/proc/self/cgroup`,
    then checks that its cgroup directory is owned by its uid
    and that `cgroup.procs` and `cgroup.subtree_control` are writable;
    otherwise it exits with a `doctor` pointer,
    because sandboxing is required.
3. It creates `daemon/` under its delegated cgroup and writes its own PID to `daemon/cgroup.procs`.
4. It writes `+cpu +io +memory +pids` to the delegated cgroup's own `cgroup.subtree_control`,
    never to any ancestor.
5. It kills and removes leftover `tasks/*` cgroups from a previous crash.
6. It binds the control socket,
    walks and watches the repository,
    and starts the scheduler.

## Shared Rust core evidence

### R1 per-task cgroup v2

#### Spawn design

- Before spawn:
   `mkdir tasks/<id>`;
   write `cpu.max`,
   `memory.max`,
   `memory.oom.group` as `1`,
   and `pids.max`;
   open `tasks/<id>/cgroup.procs` for writing with close-on-exec in the parent.
- In `pre_exec`:
   read the child's PID with `getpid`,
   format it into a stack buffer,
   `write` it to the pre-opened descriptor,
   and return `io::Error::last_os_error()` on failure,
   so spawn fails closed in the parent.
  Writing an explicit PID uses the documented interface;
   writing `0` also means "the writer",
   but only the kernel source says so.
- After spawn:
   `pidfd_open(pid, NONBLOCK)`,
   register it with `AsyncFd`,
   and call `Child::try_wait` when it becomes readable.
- Pause:
   write `1` to `cgroup.freeze`,
   then treat the task as frozen only once `cgroup.events` shows `frozen 1`,
   watched through inotify.
  Resume writes `0`.
- End:
   write `1` to `cgroup.kill`,
   wait for `populated 0` in `cgroup.events`,
   then `rmdir` the cgroup.
- Graceful end variant:
   freeze,
   read `cgroup.procs`,
   send `SIGTERM` to each PID,
   thaw,
   wait a grace period,
   then `cgroup.kill`.
  Freezing first narrows PID reuse,
   because frozen processes run only to die from fatal signals
   (inference from the kernel freeze text;
   unverified by experiment).

#### Evidence

- `pre_exec` runs "in the context of the child process after a `fork`",
   forbids allocation-dependent operations,
   and notes that `Error::new` and `Error::other` allocate
   (`STD/os/unix/process.rs:60-110`, verified;
   the same text is on <https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html>, std 1.98.1,
   verified by curl).
- The child applies `setpgid` (`STD/sys/process/unix/unix.rs:346-348`) and `setsid` (`:350-352`)
   before running closures (`:388-390`),
   so the cgroup write happens after process-group setup (verified).
- Any `pre_exec` closure disables the `posix_spawn` fast path,
   `|| !self.get_closures().is_empty()` then `return Ok(None)`
   (`STD/sys/process/unix/unix.rs:469-477`, verified),
   so every task spawn uses `fork` of the daemon.
  The fork cost for the daemon's resident memory is unmeasured.
- Kernel `cgroup.procs` doc:
   "A PID can be written to migrate the process associated with the PID to the cgroup",
   the writer needs write access to `cgroup.procs` of the destination and of the common ancestor
   (<https://docs.kernel.org/admin-guide/cgroup-v2.html>, verified by curl).
  The page does not mention writing `0` (verified by the extracted section text).
- Kernel source:
   a `0` PID selects `current` (`SCR/rust-stack/src/linux-cgroup.c:3131-3139`, verified),
   and permission checks use "the credentials from file open to protect against inherited fd attacks"
   (`:5504-5510`, verified),
   so a parent-opened descriptor carries the daemon user's credentials into the child.
- Kernel doc for `cgroup.freeze`:
   completion updates `frozen` in `cgroup.events`,
   and frozen processes "can be killed by a fatal signal";
   `cgroup.kill` sends `SIGKILL` to the subtree,
   "deal[s] with concurrent forks appropriately and is protected against migrations"
   (<https://docs.kernel.org/admin-guide/cgroup-v2.html>, verified by curl).
- `clone3` with `CLONE_INTO_CGROUP` (Linux 5.7) creates the child directly in a cgroup given by a directory descriptor
   (<https://man7.org/linux/man-pages/man2/clone.2.html>, verified by curl;
   kernel `cgroup_css_set_fork`, `SCR/rust-stack/src/linux-cgroup.c:6863-6950`, verified).
  Rejected for 0.x:
  - std's spawn has no `clone3` path;
     `rg 'pidfd|PidFd|create_pidfd|clone3|CLONE_INTO_CGROUP'`
     over `STD/sys/process/unix/unix.rs` matched only pidfd lines (verified).
  - A `clone3` spawn needs hand-written exec,
     standard-descriptor setup,
     and error reporting that std already implements.
  - `libc`'s `CLONE_INTO_CGROUP` is a deprecated `c_int` holding `0x200000000`
     (`REG/libc-0.2.189/src/unix/linux_like/linux/gnu/mod.rs:871-876`, verified),
     which evaluates to `0` (libc constant probe),
     so passing it silently spawns into the parent's cgroup.
  - The `clone3` crate's last release is 0.2.3 on 2022-12-07 (crates.io API, verified).
- pidfd:
   std's `create_pidfd` is nightly-only (`linux_pidfd`, issue 82971)
   (`STD/os/linux/process.rs:5`, `:191-210`, verified;
   <https://doc.rust-lang.org/std/os/linux/process/trait.CommandExt.html> shows "This is a nightly-only experimental
   API", verified by curl).
  `rustix::process::pidfd_open` is stable
   (`REG/rustix-1.1.4/src/process/pidfd.rs:20-30`, verified).
- `std::thread::available_parallelism` reads cgroup quotas on Linux and is recomputed on every call
   (`STD/thread/functions.rs:633-666`, verified;
   repository note `doc/troubleshooting/rust-std-thread-defaults-for-compute.md`, verified),
   so the default concurrency should be read once and cached.

#### cgroup crates

- `cgroups-rs` 0.5.1 (kata-containers):
  - `Cgroup::create` for v2 calls `create_v2_cgroup`,
     which writes `+<controller>` to `cgroup.subtree_control` of the hierarchy root and of every ancestor on the path,
     discarding each write result with `let _rest = fs::write(...)`
     (`src/fs/cgroup.rs:83-92`, `:512-517`, `:545-569`, verified).
    With the standard root `/sys/fs/cgroup`,
     a task path under the delegated scope would also write to `user@1000.service` and `app.slice`,
     which systemd created and this user can write (delegated cgroup ownership probe);
     systemd's delegation doc forbids writing attributes of cgroups systemd created
     (`doc/planning/monorepo-manager-route-research/from-scratch-inputs.md:708-712`, verified).
    The resulting ancestor writes are inference from the code and ownership;
     not executed.
  - Controller availability is read from the global root `/sys/fs/cgroup/cgroup.controllers`,
     not from the delegated parent (`src/fs/cgroup.rs:520-527`, verified).
  - `FreezerController::state` for v2 reads `cgroup.freeze` and maps `1` to `Frozen`
     (`src/fs/freezer.rs:113-125`, verified),
     which reports the requested state,
     while the kernel doc puts completion in `cgroup.events`.
  - `FsManager::destroy` moves each process to the root cgroup and ignores errors
     (`src/manager/fs.rs:770-790`, verified),
     which an unprivileged delegated user cannot do (inference).
  - `zbus = "5.8"` is a non-optional dependency (`Cargo.toml:23`, verified).
  - Culled.
- `cgroups` 0.1.0 (2018-12-23) and `controlgroup` 0.3.0 (2020-03-29) are stale (crates.io API, verified);
   not examined further.

### R2 recursive watching

#### Watch design

- One `inotify::Inotify` instance converted with `into_event_stream`.
- Walk with `ignore::WalkBuilder`,
   whose `follow_links` defaults to `false` (`REG/ignore-0.4.33/src/walk.rs:567`, verified).
  Exclude `node_modules`,
   `.git`,
   `target`,
   and `dist`;
   5,480 directories remain against 99,416 when only `.git` is excluded
   (design doc "Watching and affected work", measured earlier).
- Mask:
   `CREATE | DELETE | CLOSE_WRITE | MOVED_FROM | MOVED_TO | ATTRIB | DELETE_SELF | MOVE_SELF | ONLYDIR | DONT_FOLLOW | EXCL_UNLINK`,
   without `OPEN` or `ACCESS`,
   so builds reading sources generate no events
   (flags exist in `REG/inotify-0.11.4/src/watches.rs:195-221`, verified).
- `CREATE` with `ISDIR` in a watched,
   non-ignored directory:
   add a watch,
   then scan the new directory and synthesize create events for what the scan finds,
   closing the window in which children were created before the watch existed.
- `Q_OVERFLOW`:
   mark every path dirty,
   rescan,
   and rehash.
  A full rehash reads 8,076 files and 144.3 MiB (content-hash input volume probe).
- `IGNORED`:
   drop the descriptor mapping.
  The same watch descriptor returns for a second path to the same inode
   (`REG/inotify-0.11.4/src/watches.rs:278-290`, verified),
   so the map is descriptor to a set of paths.
- `ENOSPC` when adding a watch:
   fail closed and point at `doctor`;
   `max_user_watches` is 524,288 (inotify limits probe).
- Bursts:
   a coalescing window plus an explicit hold state for `pnpm install` and branch switches.
  The kernel queue holds 16,384 events (inotify limits probe);
   beyond that the overflow path runs.

#### Evidence and the `notify` cull

- inotify(7) documents `IN_Q_OVERFLOW` and non-recursive directory monitoring
   (<https://man7.org/linux/man-pages/man7/inotify.7.html>, verified by curl for `IN_Q_OVERFLOW`).
- `notify` 8.2.0,
   the newest stable release (crates.io API, verified):
  - Always adds `WatchMask::OPEN` (`REG/notify-8.2.0/src/inotify.rs:425-432`, verified)
     and turns it into `Access(Open)` events (`:349-351`, verified).
    A daemon whose own builds open thousands of watched sources would receive those events
     and approach the 16,384-event queue (inference).
  - The event-kind filter `EventKindMask` exists only from `9.0.0-rc.1` (2026-01-25)
     (`~/temp/agent/notify-2026-09-16/notify/CHANGELOG.md:75`, `:83`, verified).
  - No path exclusion exists for recursive watches;
     8.2.0 `Config` offers only poll interval,
     manual polling,
     content comparison,
     and symlink following (`REG/notify-8.2.0/src/config.rs:58-106`, verified).
  - Recursive walks follow symbolic links "On by default" (`REG/notify-8.2.0/src/config.rs:100-106`, verified).
  - New subdirectories get watches only after the whole event batch,
     with no scan events for their contents (`REG/notify-8.2.0/src/inotify.rs:61-76`, `:383-397`, verified).
    Open issue #727 (2025-10-31) reports that `create_dir_all("1/2/.../10")` yields only one create event
     (`gh issue view 727 --repo notify-rs/notify`, verified).
  - Fixes for silently unwatched subtrees,
     "never abandon a recursive watch" (#970),
     and a watcher TOCTOU (#792) exist only in release candidates
     (`CHANGELOG.md:23-24`, `:64`, verified);
     the 9.0 line has been in release candidates since 2026-01-25.
  - Documentation triggers are listed under "R9 documentation quality".
  - Culled.

### R3 Unix socket JSON-RPC 2.0

#### RPC design

- Socket:
   `$XDG_RUNTIME_DIR/<tool>/daemon.sock` inside a `0700` directory.
  Bind with `std::os::unix::net::UnixListener::bind`,
   set non-blocking,
   then `tokio::net::UnixListener::from_std`.
- On accept,
   compare `UnixStream::peer_cred()` uid with the daemon's uid
   (`REG/tokio-1.53.1/src/net/unix/stream.rs:966-967`, verified).
- Framing:
   `(&mut reader).take(MAX_LINE)` plus `AsyncBufReadExt::read_until(b'\n', ...)`;
   a line reaching the cap closes the connection with an error.
- Messages:
   `serde_json` into typed request enums;
   responses and notifications serialize one object per line.
  JSON-RPC 2.0 batch handling was not re-read against the specification in this session (unverified).
- Subscriptions:
   one bounded ring buffer of events,
   each with a monotonically increasing sequence number.
  `subscribe { since }` replays from `since`;
   when `since` predates the oldest retained event,
   the daemon sends a fresh snapshot first,
   modeled on Watchman fresh-instance results.
- Backpressure:
   each connection's writer task holds only a cursor and waits on `tokio::sync::Notify`.
  A slow client falls behind in the ring and later receives a snapshot;
   producers never block and memory stays bounded by the ring size.
  Requests on one connection run with a small in-flight limit.

#### Evidence

- `std` rejects any NUL byte in a socket path,
   "paths must not contain interior null bytes"
   (`STD/os/unix/net/addr.rs:25-37`, verified).
- `tokio::net::UnixListener::bind` treats a path starting with NUL as an abstract socket
   (`REG/tokio-1.53.1/src/net/unix/listener.rs:76-89`, verified),
   and its doc comment does not say so (`:65-75`, verified);
   abstract sockets carry no file permissions (design doc "RPC").
  Hence binding with `std`.
- `jsonrpsee`:
  - Server transports are HTTP and WebSocket;
     custom transports are client-side only
     (`~/temp/agent/jsonrpsee-2026-09-16/README.md:15-19`, verified).
  - The generic `serve` runs hyper HTTP over any `AsyncRead + AsyncWrite`
     (`server/src/utils.rs:86-107`, verified),
     so it cannot speak newline-delimited JSON-RPC.
  - Newest feature release 0.26.0 on 2025-08-11;
     0.24.11 on 2026-05-27 is a backport (crates.io versions API, verified).
  - Documentation triggers are listed under "R9 documentation quality".
  - Culled.
- `tokio_util::codec::LinesCodec` is culled on documentation grounds ("R9 documentation quality").

### R4 btrfs

#### btrfs design

- Detection:
   `rustix::fs::statfs(repo)?.f_type == libc::BTRFS_SUPER_MAGIC`
   (`REG/rustix-1.1.4/src/fs/abs.rs:269`;
   `REG/libc-0.2.189/src/unix/linux_like/mod.rs:1531`, verified).
  `doctor` parses `/proc/self/mountinfo` super options for `user_subvol_rm_allowed`.
- Cache restore:
   open the cache blob read-only;
   create a temporary file next to the destination with exclusive create;
   `rustix::fs::ioctl_ficlone(&tmp, &blob)`;
   rename over the destination.
  On `EXDEV`,
   `EINVAL`,
   or `EOPNOTSUPP`,
   fall back to a copy into the same temporary file
   (errors per <https://man7.org/linux/man-pages/man2/ioctl_ficlone.2.html>, verified by curl for `EXDEV`).
- Subvolumes and snapshots stay optional in 0.x,
   because the checkout is not a subvolume (design doc "btrfs acceleration").
  When enabled:
   `#[repr(C)]` mirrors of `btrfs_ioctl_vol_args_v2` and `btrfs_ioctl_get_subvol_info_args`
   (`SCR/rust-stack/src/linux-uapi-btrfs.h:135`, `:867`, verified)
   and the request numbers for `SUBVOL_CREATE`,
   `SNAP_CREATE_V2`,
   `GET_SUBVOL_INFO`,
   and `SNAP_DESTROY_V2` (`:1170`, `:1187`, `:1247`, `:1253`, verified),
   called through `rustix`'s `unsafe` ioctl interface,
   with `BTRFS_SUBVOL_RDONLY` (`:49`, verified) set explicitly.
  Setting the flag explicitly sidesteps the btrfs-progs documentation contradiction about the default snapshot mode
   (`doc/planning/monorepo-manager-route-research/from-scratch-inputs.md:606-607`, verified).

#### Evidence

- `rustix::fs::ioctl_ficlone` is a safe function wrapping `ioctl(fd, FICLONE, src_fd)`
   (`REG/rustix-1.1.4/src/fs/ioctl.rs:45-58`, verified).
- `reflink-copy` 0.1.30:
  - Its Linux `reflink` creates the destination path itself with `create_new`,
     clones into it,
     and deletes it on failure (`src/sys/unix/linux.rs:7-16`; `src/sys/utility.rs:19-27`, `:55-67`, verified),
     then sets permissions afterward (`linux.rs:15`).
  - Documentation triggers are listed under "R9 documentation quality".
  - Culled.
- No `BTRFS_IOC` constant exists in `libc` 0.2.189,
   `rustix` 1.1.4,
   or `nix` 0.31.3
   (`rg --count-matches 'BTRFS_IOC'` over those sources exited 1 with no matches, verified).
- `libbtrfsutil` 0.8.0 generates bindings with bindgen and needs `libclang` plus `libbtrfsutil` headers
   (its README,
   `gh api repos/zhangyuannie/libbtrfsutil-rs/readme`, verified);
   this bootc host has only the runtime `libbtrfsutil.so.1*` (btrfs user-space probe),
   so building it would need layered packages or a container.
  Culled.
- `btrfsutil` 0.2.0 (2023-10-05) and `btrfs` 1.2.2 (2017-07-04) are stale (crates.io API, verified).

### R5 hashing

#### Hashing design

- `blake3` 1.8.7 with default features only.
- Read each file with buffered reads into `Hasher::update`;
   run files in parallel on a bounded blocking pool.
- Skip `update_rayon`:
   its docs say it is slower than `update` below 128 KiB on x86_64
   (`REG/blake3-1.8.7/src/lib.rs:1482-1486`, verified),
   and only 152 files reach that size (content-hash input volume probe).
- Never enable `mmap`:
   the crate's private comment says a concurrent change can make the hasher "hash nonsense bytes or crash with SIGBUS"
   (`REG/blake3-1.8.7/src/io.rs:39-44`, verified),
   and repository files change under the daemon by design.

#### Evidence

- Published figures:
   BLAKE3 6,866 MiB/s and OpenSSL SHA-256 484 MiB/s at 16 KiB inputs,
   single-threaded,
   on an AWS c5.metal Cascade Lake-SP 8275CL
   (`benchmarks/bar_chart.py:9-35`, `:72` in BLAKE3-team/BLAKE3-specs at `ac784c9f`, verified by fetch).
- This host has AVX-512 and SHA extensions (CPU probe),
   so neither published figure transfers directly (unverified).
- Arithmetic on the published figures,
   not a host measurement:
   144.3 MiB at 6,866 MiB/s is about 0.02 s of hashing,
   and at 484 MiB/s about 0.3 s,
   so file I/O rather than the hash function should dominate a full rehash (inference).
- Hash choice therefore does not separate the options.

### R6 process supervision

#### Supervision design

- Spawn with `std::process::Command`:
   `process_group(0)`,
   `stdin(Stdio::null())`,
   piped `stdout` and `stderr`,
   and the cgroup `pre_exec`.
- Convert each pipe with `tokio::net::unix::pipe::Receiver::from_owned_fd`
   (`REG/tokio-1.53.1/src/net/unix/pipe.rs:967`, verified).
- Exit:
   the pidfd becomes readable,
   then `Child::try_wait` reaps
   ("If the child has exited then on Unix the process ID is reaped",
   `STD/process.rs:2404-2411`, verified).
  The supervisor owns each `Child` until it is reaped,
   because "There is no implementation of `Drop` for child processes" (`STD/process.rs:180`, verified).
- No `SIGCHLD` handler,
   no `waitpid(-1)`,
   and no child subreaper,
   so nothing else reaps a supervised child (design choice).
- Signals:
   `tokio::signal::unix::signal(SignalKind::interrupt())` and `terminate()`.
  First `SIGINT`:
   stop scheduling,
   gracefully end every task,
   remove the socket and task cgroups,
   exit.
  Second `SIGINT`:
   `cgroup.kill` everything and exit.
  The handler replaces default `SIGINT` behavior for the whole process lifetime
   (`REG/tokio-1.53.1/src/signal/unix.rs:332-352`, verified),
   which is why the second-signal path must exist.
- Terminal Ctrl+C goes to the foreground process group;
   tasks in their own groups do not receive it
   (tokio's `process_group` doc example says so,
   `REG/tokio-1.53.1/src/process/mod.rs:757-793`, verified).

#### Evidence and culls

- `tokio::process`:
  - Module doc:
     "The asynchronous process support is provided through signal handling on Unix"
     (`REG/tokio-1.53.1/src/process/mod.rs:6-7`, verified;
     same text on <https://docs.rs/tokio/latest/tokio/process/index.html>, tokio 1.53.1, verified by curl).
  - On Linux the implementation first tries a pidfd reaper and falls back to `SIGCHLD` only if `pidfd_open` fails
     (`src/process/unix/mod.rs:106-133`;
     `src/process/unix/pidfd_reaper.rs:30-58`, `:150-162`, verified).
  - Reaping of dropped children is "best-effort" with "No additional guarantees"
     (`src/process/mod.rs:197-227`, `:640-667`, verified).
  - Culled on the documentation contradiction;
     the std plus pidfd design replaces it.
- `tokio::signal::ctrl_c` is culled on documentation grounds ("R9 documentation quality").
- `process-wrap` 10.0.0 manages process groups and `killpg`
   (`doc/planning/monorepo-manager-route-research/from-scratch-inputs.md:425-426`, verified);
   cgroups replace that role here.

### R8 repository fit

- Rust rules:
   `MXR` 300 code lines per `.rs` file (`AGENTS.md:1012-1020`, verified);
   `RDC` rustdoc on every documentable item,
   private included (`AGENTS.md:1022-1028`, verified).
- Clippy baseline enforced by file-enforcer:
   `disallowed_methods = "deny"`,
   `implicit_return = "deny"`,
   `needless_return = "allow"` (`doc/planning/cargo-toml-file-enforcer.md:86-91`, verified);
   the root `clippy.toml` bans `Result::unwrap` (verified).
- No Cargo workspace;
   every crate is its own root with an empty `[workspace]` table
   (`doc/planning/cargo-toml-file-enforcer.md:12-22`, `:92-98`, verified).
- Toolchain:
   floating `nightly` for every crate with no per-package override (`mise.no-env.toml:80-93`, verified);
   no `rust-toolchain*` file exists
   (`rg --files --hidden --glob 'rust-toolchain*'` exited 1, verified).
- Build and test tasks per package call Cargo from `mise.toml`:
   `build` as `cargo build --release`,
   `lint:clippy`,
   `lint:rust` via `monochromatic-rust-linter`,
   and `test` as `cargo nextest run --release`
   (`package/forbidden-strings/cli/mise.toml`, verified);
   `cargo-nextest` is a root tool (`mise.no-env.toml:105`, verified).
- Release precedent:
   `.github/workflows/cargo-publish.yml` has per-crate job sets.
  The publish jobs install the mise toolchain (`:113-114`, `:386-387`, verified),
   while the binary jobs run `rustup target add` and `cargo build --release --target` without mise
   (`:198-205`, `:467-474`, verified),
   so release binaries use the runner's default toolchain rather than the repository's nightly
   (which toolchain the runner provides is unverified).
  A Linux-only binary already has precedent in the `nws-*` jobs (x86_64 and aarch64 GNU only, verified).
  The same workflow's shell logic is cited as an embedded-code gap in
  `doc/planning/load-bearing-code-languages.md:29-33` (verified).
- Recorded Rust build durations:
  - forbidden-strings clean release build 50.86 s after the build-override fix
     (`doc/troubleshooting/cargo-build-override-opt-level.md:247`, verified).
  - A Tauri Android debug build including hyper,
     tokio,
     and reqwest,
     40.53 s (`doc/decision/kotlin-android-kopia-pcloud-vet-report/vet-tauri.md:330`, verified).
  - mise built from source in a bounded container,
     4 min 17 s (`doc/troubleshooting/mise-rust-components.md:546`, verified).
  - Nothing is recorded for a tokio daemon of this shape
     (absence in the `rg` over `doc` and `package` Markdown, verified).
- Existing tokio use:
   `rt`,
   `sync`,
   and `macros` features in the music-player crates
   (`package/music-player/desktop-app/Cargo.toml:116`, `package/music-player/android-app/rust/Cargo.toml:81`, verified);
   no repository crate uses tokio networking or process APIs (inference from that `rg`).
- Language approval:
   the proposal keeps "TypeScript as default for repository automation"
   (`doc/planning/load-bearing-code-languages.md:82`, verified)
   and says approval is explicit per scope,
   not inferred from existing files (`:108-114`, verified).
  Both options need Rust approved for this scope.

### R9 documentation quality

Every page below loaded with
`curl --location --silent --show-error --user-agent 'Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0'`,
exited 0,
and contained the checked text,
unless noted.

#### Triggers and culls

- `notify` 8.2.0,
   <https://docs.rs/notify/latest/notify/>:
  - The Installation section shows `notify = 8.1.0` on the 8.2.0 page (verified).
  - "Linux: Bad File Descriptor / No space left on device" says to run `sudo sysctl fs.inotify.max_user_watches=524288`
     and then `sudo sysctl -p`,
     which reloads the configuration file rather than persisting the value just set,
     so the remediation reads as persistent when it is not (verified text; `sysctl -p` semantics from recall).
  - The example comment "All files and directories at that path and below will be monitored"
     conflicts with the same page's "Watching large directories" warning and with issue #727 (verified).
  - Result:
     cull `notify`.
- `jsonrpsee-server`:
  - <https://docs.rs/jsonrpsee-server/latest/jsonrpsee_server/fn.serve.html>:
     "Serve a service over a TCP connection",
     while the signature accepts any `AsyncRead + AsyncWrite` (`server/src/utils.rs:86-99`, verified).
  - <https://docs.rs/jsonrpsee-server/latest/jsonrpsee_server/struct.ServerConfigBuilder.html>:
     "`n` messages can be buffered and if the client can't keep
     with up the server." is an incomplete sentence (`server/src/server.rs:448-450`, verified).
  - <https://docs.rs/jsonrpsee/latest/jsonrpsee/server/fn.serve.html> returns "This page does not exist" (verified).
  - Result:
     cull `jsonrpsee`.
- `cgroups-rs` 0.5.1,
   <https://docs.rs/cgroups-rs/latest/cgroups_rs/>:
  - The crate root has no crate-level prose;
     docs.rs reports "71.87% of the crate is documented" (verified).
  - `Cgroup::new` says "Create a new control group" and omits the ancestor `cgroup.subtree_control` writes;
     the saved page text has no "subtree" match (verified).
  - README:
     `.cpu().shares(85)` "gets 85% of the CPU time in relative to other control groups" (`README.md:29`),
     while shares are relative weights (recall),
     and "during the `0.0.*` phase" (`README.md:58`) on a 0.5.1 crate (verified).
  - Result:
     cull `cgroups-rs`.
- `reflink-copy` 0.1.30,
   <https://docs.rs/reflink-copy/latest/reflink_copy/fn.reflink.html>:
   "NOTE that it generates a temporary file and is not atomic" (`src/lib.rs:43`, verified),
   while the Linux code creates the destination path itself ("R4 btrfs").
  Result:
   cull `reflink-copy`.
- `libc` `CLONE_INTO_CGROUP`,
   <https://docs.rs/libc/latest/libc/constant.CLONE_INTO_CGROUP.html>:
   renders `pub const CLONE_INTO_CGROUP: c_int = 0x200000000;`
   with "Deprecated since 0.2.188: This constant overflows" (verified),
   while the value compiles to `0` (libc constant probe).
  Result:
   cull the constant;
   `libc` stays as a transitive dependency.
- `tokio::process`,
   <https://docs.rs/tokio/latest/tokio/process/index.html>:
   "provided through signal handling on Unix" versus the Linux pidfd reaper ("R6 process supervision").
  Result:
   cull the `tokio::process` module,
   not the `tokio` crate.
- `tokio::signal::ctrl_c`,
   <https://docs.rs/tokio/latest/tokio/signal/fn.ctrl_c.html>:
   the doc of a function returning a future says "Even if this `Signal` instance is dropped",
   and "the time it arrives, it will be translated to a stream event" is garbled
   (`REG/tokio-1.53.1/src/signal/ctrl_c.rs:18-31`, verified).
  Result:
   cull `ctrl_c`;
   use `tokio::signal::unix::signal`,
   whose caveat text is coherent (`src/signal/unix.rs:332-352`, verified).
- `tokio::net::UnixListener::bind`,
   <https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html>:
   omits the abstract-socket branch ("R3 Unix socket JSON-RPC 2.0").
  Result:
   bind with `std`,
   keep `UnixListener::from_std`.
- `tokio_util::codec::LinesCodec`,
   <https://docs.rs/tokio-util/latest/tokio_util/codec/struct.LinesCodec.html>:
   "Subsequent calls will discard up to `limit` bytes from that line until a newline character is reached"
   leaves unclear what happens to the rest of an overlong line,
   and names `limit` for a parameter called `max_length`
   (`REG/tokio-util-0.7.18/src/codec/lines_codec.rs:50-59`, verified).
  Result:
   cull;
   use `take` plus `read_until`.
- `blake3` `Hasher::update_mmap`,
   <https://docs.rs/blake3/latest/blake3/struct.Hasher.html>:
   the page text has no "SIGBUS" (verified),
   while the crate's private comment states the crash risk ("R5 hashing").
  Result:
   cull the `mmap` feature,
   keep the crate.
- `libbtrfsutil`:
   its README docs badge points at `img.shields.io/docsrs/btrfsutil`,
   a different crate (README line 4, verified).
  Combined with the build requirements in "R4 btrfs",
   cull.
- btrfs-progs documentation contradicts itself on the default snapshot mode
   (`doc/planning/monorepo-manager-route-research/from-scratch-inputs.md:606-607`, verified).
  Result:
   set `BTRFS_SUBVOL_RDONLY` explicitly through the ioctl instead of relying on CLI defaults.
- `rustix::process::set_child_subreaper(pid: Option<Pid>)` documents a `Pid` parameter
   for a `prctl` option that takes a flag,
   without saying what the `Pid` means (`REG/rustix-1.1.4/src/process/prctl.rs:858-866`, verified).
  The design does not use it.

#### Pages loaded without a trigger in the sections read

- <https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html> (`pre_exec`, `process_group`).
- <https://doc.rust-lang.org/std/os/linux/process/struct.PidFd.html> (loaded; unstable API not used).
- <https://docs.rs/tokio/latest/tokio/process/struct.Command.html> (loaded; module culled).
- <https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html>
   (lag semantics clear; not used, the event ring replaces it).
- <https://docs.rs/tokio/latest/tokio/io/trait.AsyncBufReadExt.html> (`read_until`).
- <https://docs.rs/tokio/latest/tokio/io/unix/struct.AsyncFd.html> (edge-triggered readiness text present).
- <https://docs.rs/tokio/latest/tokio/net/unix/pipe/struct.Receiver.html> (`from_owned_fd`).
- <https://docs.rs/inotify/latest/inotify/> and <https://docs.rs/inotify/latest/inotify/struct.WatchMask.html>;
   `Watches::add` hardlink note read in source (`REG/inotify-0.11.4/src/watches.rs:278-297`).
- <https://docs.rs/rustix/latest/rustix/fs/fn.ioctl_ficlone.html>,
   `fn.statfs.html`,
   `process/fn.pidfd_open.html`,
   `process/fn.waitid.html`:
   brief,
   each deferring to the man page,
   no contradiction found.
- <https://docs.rs/libc/latest/libc/struct.clone_args.html> (not used after the `clone3` rejection).
- <https://docs.rs/ignore/latest/ignore/struct.WalkBuilder.html> (`follow_links`).
- <https://docs.rs/serde_json/latest/serde_json/> (loaded; not read in depth).
- <https://docs.kernel.org/admin-guide/cgroup-v2.html> (`cgroup.procs`, `cgroup.freeze`, `cgroup.kill` sections read).
- <https://man7.org/linux/man-pages/man2/clone.2.html> (`CLONE_INTO_CGROUP` section read).
- <https://man7.org/linux/man-pages/man7/inotify.7.html>,
   <https://man7.org/linux/man-pages/man2/ioctl_ficlone.2.html>,
   <https://man7.org/linux/man-pages/man2/pidfd_open.2.html> (loaded; checked text present).
- Option-specific pages are listed in each option's R9.

Noted without culling:
the `std::os::unix::net::UnixListener::bind` example prints "Couldn't connect" on a bind failure
(`STD/os/unix/net/listener.rs:56-67`).

### R10 maturity and maintenance

crates.io API (`https://crates.io/api/v1/crates/<name>`) and GitHub API,
fetched 2026-09-16.
"Releases in the past year" counts non-yanked versions published since 2025-09-16.

- Kept:
  - `tokio` 1.53.1 (2026-07-20),
     21 releases in the past year,
     last commit 2026-09-16,
     446 open issues and PRs.
  - `inotify` 0.11.5 (2026-08-14),
     5 releases,
     last commit 2026-08-14,
     10 open.
  - `rustix` 1.1.5 (2026-09-16),
     3 releases,
     last commit 2026-09-16,
     143 open.
  - `serde_json` 1.0.151 (2026-07-20),
     6 releases,
     last commit 2026-08-08,
     232 open.
  - `blake3` 1.8.7 (2026-08-20),
     5 releases,
     last commit 2026-09-10,
     201 open.
  - `ignore` 0.4.33 (2026-08-04),
     10 releases.
  - `libc` 0.2.189 stable,
     17 releases including `1.0.0-alpha.4` (2026-07-21),
     last commit 2026-09-15.
- Culled or unused:
  - `notify` 8.2.0 stable (2025-08-03),
     newest `9.0.0-rc.5` (2026-08-30),
     last commit 2026-09-14,
     91 open.
  - `cgroups-rs` 0.5.1 (2026-07-14),
     2 releases,
     last commit 2026-08-20,
     23 open,
     156 stars.
  - `jsonrpsee` 0.26.0 (2025-08-11),
     last commit 2026-08-03,
     93 open.
  - `reflink-copy` 0.1.30 (2026-06-18),
     last commit 2026-07-31,
     5 open,
     47 stars.
  - `libbtrfsutil` 0.8.0 (2026-07-04),
     5 stars;
     `btrfsutil` 0.2.0 (2023-10-05);
     `btrfs` 1.2.2 (2017-07-04).
  - `clone3` 0.2.3 (2022-12-07),
     last commit 2022-12-07.
  - `nix` 0.31.3 (2026-05-11),
     last commit 2026-05-19,
     359 open;
     not needed beside `rustix`.
  - `tokio-util` 0.7.19 (2026-07-21);
     only `LinesCodec` was considered.

## Option 1: all Rust

### Design sketch

- The shared Rust core,
   plus a Rust `file-enforcer` library crate implementing FE01 to FE24
   (requirement codes in `doc/audit/tech-monorepo-manager-vet-2026-09-16.md:97-193`, verified).
- Library mapping:
  - FE11 TOML splice edits:
     `toml_edit` (0.25.15, 2026-09-11).
  - FE12 XML entries:
     `quick-xml` (0.42.0, 2026-08-22) with byte-offset splicing.
  - FE09 staleness manifest and lock:
     ported.
  - FE18 and FE19 package provisioning and the Repology index:
     ported,
     or delegated to Meta Package Manager as the vet allows
     (`doc/audit/tech-monorepo-manager-vet-2026-09-16.md:195-217`, per the research doc).
  - FE21 to FE23 watching,
     protection,
     and echo suppression:
     inside the daemon's watcher.
- Config host variants,
   because `file-enforcer.config.ts` is a program:
  - 1a:
     the config becomes a Rust binary crate depending on the library;
     each config edit triggers a `cargo build` task,
     and each evaluation runs the binary as a child process in a task cgroup.
  - 1b:
     the config is compiled into the daemon binary;
     each config edit rebuilds and restarts the daemon.
  - 1c:
     an embedded JavaScript engine such as `rquickjs` runs the config after type stripping,
     with host functions emulating the Node APIs the config uses.
  - 1d:
     declarative data (TOML or JSON) plus Rust generators.

### R1 per-task cgroup v2

As "Shared Rust core evidence",
"R1 per-task cgroup v2".
Variant 1a adds config-binary evaluations as cgroup tasks;
variant 1b runs enforcement inside the daemon's own `daemon/` cgroup,
so freezing or killing an evaluation is impossible without stopping the daemon (inference).

### R2 recursive watching

As "Shared Rust core evidence",
"R2 recursive watching".
In variants 1b and 1c the daemon knows each write before it happens,
so echo suppression can register expected hashes first.
In variant 1a,
writes arrive from a child,
with the same ordering problem described under "Option 2",
"R2 recursive watching".

### R3 Unix socket JSON-RPC 2.0

As "Shared Rust core evidence",
"R3 Unix socket JSON-RPC 2.0".

### R4 btrfs

As "Shared Rust core evidence",
"R4 btrfs".

### R5 hashing

As "Shared Rust core evidence",
"R5 hashing".
FE09's destination SHA-256 checks
(`package/dev-script/file-enforcer/src/io/staleness-destination-match.ts:30-76`, per the research doc)
can move to BLAKE3 in a rewrite,
which invalidates every existing manifest once (inference).

### R6 process supervision

As "Shared Rust core evidence",
"R6 process supervision".

### R7 file-enforcer interop

Rewrite scope,
measured by `SCR/rust-fe-count.ts` over `package/dev-script/file-enforcer/src`.
Code lines exclude blank lines and comment-only lines by a line heuristic,
so they are approximate.

- 79 non-test `.ts` modules
   (excluding `*.test.*`,
   `*.container-test.*`,
   and `staleness-lock-regression-fixture.ts`):
   36,213 lines,
   29,105 code lines.
- `data/packages.generated.ts` alone is 22,608 lines,
   9,647 Repology entries (its header, verified).
  Hand-written code is therefore 13,605 lines and 6,508 code lines.
- By directory,
   files and code lines:
   `io` 33 and 2,401;
   `watch` 9 and 1,027;
   `package` 8 and 867;
   `jetbrains` 5 and 711;
   `pipeline` 7 and 678;
   top level 9 and 485;
   `cargo` 4 and 241;
   `platform` 1 and 22;
   `data` 3 and 22,673.
- Tests to port:
   56 `*.test.ts` files,
   11,182 lines (`SCR/rust-config-logic.ts`).
- Public API:
   `src/index.ts` exports 119 names,
   values and types together,
   across FE03 to FE24
   (a `node` count of names inside its `export { }` and `export type { }` blocks, verified).
- Root config:
   `file-enforcer.config.ts` is 2,330 lines (1,113 code lines),
   with 68 function declarations,
   27 async functions,
   74 `await` expressions,
   25 `if` statements,
   8 loops,
   and 18 `.map(` calls (`SCR/rust-config-logic.ts`).
  It imports `node:crypto`,
   `node:fs/promises`,
   `nano-spawn`,
   the file-enforcer API,
   and `@monochromatic-dev/config-pnpr/ts` (`file-enforcer.config.ts:1-34`, verified).
- Other TypeScript consumers:
   `package/dev-script/vm-builder/src/import.ts:9` and `src/build-and-import.ts:20` import `exec`;
   `package/test-fixture/file-enforcer-perf/src/perf.config.ts:21` imports the API (verified).
- Configuration rules:
   `AD2` says "Switch from config-as-data to TypeScript when conf needs logic" (`AGENTS.md:1737-1740`, verified).
  The user relaxed FE01 so the configuration need not be TypeScript
   (`doc/audit/tech-monorepo-manager-vet-2026-09-16.md:97-101`, verified),
   but `AD2` itself is unchanged.
- Output parity:
   the TypeScript TOML path reformats a value it re-sets (`["derive"]` becomes `[ "derive", ]`),
   so the engine writes only on real differences (`doc/planning/cargo-toml-file-enforcer.md:57-64`, verified).
  Whether `toml_edit` and `quick-xml` splices reproduce today's bytes is unverified.
- Variant consequences (inference from the facts in this list):
  - 1a keeps a child-process boundary per evaluation and adds a compile per config edit;
     no compile duration is recorded for such a crate ("R8 repository fit").
  - 1b breaks the long-running process on every config edit,
     dropping subscriptions and running tasks.
  - 1c must strip types,
     resolve workspace ESM imports such as `@monochromatic-dev/config-pnpr/ts`,
     and emulate `node:fs/promises`,
     `node:crypto`,
     and `nano-spawn`.
  - 1d cannot express 68 functions of logic without inventing a DSL,
     and `AD2` asks for TypeScript once logic appears.
- Until the rewrite reaches parity,
   the daemon must run the TypeScript file-enforcer as a child,
   which is Option 2 (inference).

### R8 repository fit

- Everything in "Shared Rust core evidence",
   "R8 repository fit".
- About 6,508 hand-written code lines move to Rust,
   where `MXR` and `RDC` apply to every item.
- Moving repository automation to Rust departs from the proposal's TypeScript default for repository automation
   (`doc/planning/load-bearing-code-languages.md:82`, verified).
- The root `sync:files` tasks run the TypeScript CLI today
   (`mise.no-env.toml:1120-1127`, per `doc/planning/monorepo-manager-route-research/from-scratch-inputs.md:64-66`,
verified).

### R9 documentation quality

- Shared triggers apply ("Shared Rust core evidence",
   "R9 documentation quality").
- Loaded with the user's curl command,
   text present,
   not read in depth:
   <https://docs.rs/toml_edit/latest/toml_edit/> (`DocumentMut`),
   <https://docs.rs/quick-xml/latest/quick_xml/> (`Reader`),
   <https://docs.rs/rquickjs/latest/rquickjs/> (`QuickJS`),
   <https://docs.rs/rhai/latest/rhai/> (`Engine`).
  Their quality under the cull rule is unverified.

### R10 maturity and maintenance

- Shared crates as listed in "Shared Rust core evidence",
   "R10 maturity and maintenance".
- Added:
  - `toml_edit` 0.25.15 (2026-09-11),
     22 releases in the past year,
     toml-rs/toml last commit 2026-09-15,
     71 open.
  - `quick-xml` 0.42.0 (2026-08-22),
     10 releases,
     last commit 2026-09-02,
     91 open.
  - Variant 1c:
     `rquickjs` 0.13.0 (2026-09-08),
     6 releases,
     last commit 2026-09-16,
     63 open;
     or `rhai` 1.26.1 (2026-09-10),
     7 releases,
     last commit 2026-09-16,
     15 open.

### Yikes

Ranked by severity,
most severe first.

1.  **No acceptable home for the config.**
    The root config is a 2,330-line TypeScript program.
    Variant 1a keeps a per-evaluation child process and adds a compile per edit;
    1b restarts the daemon on every config edit;
    1c builds a Node-compatible JavaScript host inside the daemon;
    1d conflicts with `AD2` and needs an invented DSL.
    Each variant either recreates Option 2's process boundary with extra cost,
    breaks the daemon model,
    or conflicts with a repository rule.
2.  **The rewrite's interim state is Option 2.**
    79 modules (6,508 hand-written code lines),
    a 22,608-line generated index,
    and 56 test files (11,182 lines) must reach parity,
    while `vm-builder` and the perf fixture keep importing the TypeScript API.
    Until then,
    the daemon runs the TypeScript file-enforcer as a child,
    and afterward either both implementations stay or those consumers are ported.
3.  **Byte-for-byte output parity is unproven.**
    If `toml_edit` or `quick-xml` splices differ from `module-toml-edit` and `@lezer/xml` output,
    the first run rewrites managed files across the repository.
4.  **The sandbox rests on repository-written `unsafe` spawn code.**
    No cgroup crate fits (`cgroups-rs` culled);
    std has no `clone3`;
    `libc`'s `CLONE_INTO_CGROUP` is `0`;
    `pre_exec` forces `fork` (cost unmeasured).
    None of it could be exercised in this session.
5.  **Watch correctness is repository-written.**
    With `notify` culled,
    the new-directory race,
    overflow rescans,
    and descriptor reuse are the daemon's own code.
6.  **Supervision and RPC are hand-rolled.**
    `tokio::process`,
    `ctrl_c`,
    `jsonrpsee`,
    and `LinesCodec` are culled,
    so pidfd reaping,
    signal policy,
    and NDJSON framing with subscriptions are repository code.
7.  **Toolchain and release drift.**
    A long-running daemon builds on floating nightly locally,
    while the release workflow's binary jobs build without mise on the runner's toolchain.
8.  **btrfs subvolume operations need hand-written `#[repr(C)]` ioctls.**
    Optional in 0.x.

## Option 2: Rust core with TypeScript file-enforcer children

### Design sketch

- The shared Rust core runs everything,
   including watching for file-enforcer.
- Each file-enforcer configuration evaluation is a task:
   `node <file-enforcer daemon entry> --config <absolute path> --events-fd 3`,
   spawned in its own task cgroup with `stdin` from `/dev/null`.
- New TypeScript entry in `package/dev-script/file-enforcer`:
   it sets the active config,
   imports the config,
   then writes the tracker's results as events on descriptor 3 and exits.
- Events use the same JSON-RPC 2.0 notification envelope as the control socket,
   one object per line:
  - `enforcer/runStarted`:
     config path,
     Node version.
  - `enforcer/read`,
     `enforcer/glob`:
     tracked reads and globs with their static roots.
  - `enforcer/wrote`:
     destination,
     SHA-256 of written content,
     `mtimeMs`.
  - `enforcer/skipped`:
     destination unchanged or fresh.
  - `enforcer/moduleLoaded`:
     every resolved module URL,
     collected through `module.registerHooks` (see "R7 file-enforcer interop").
  - `enforcer/runFinished`:
     outcome,
     duration.
- `stderr` carries tagged logs,
   which the daemon relays with a task prefix;
   the logger already writes to `stderr` in process runtimes
   (`package/module/logger/src/sink/console.ts:226`, `:242`, verified).
- Descriptor 3 is created by `dup2` in the same `pre_exec` closure that joins the cgroup,
   avoiding `stdout` collisions with any library output.
- The daemon's watcher subscribes to the union of tracked reads,
   glob static roots,
   loaded modules,
   the config path,
   and destinations.
  A change schedules one new evaluation,
   never overlapping a running one for the same config.
- Protected destinations (FE22):
   an event on a destination whose content hash differs from the last `enforcer/wrote` hash schedules an evaluation,
   which rewrites it,
   and the daemon sends the desktop notification.
- Echo suppression (FE23):
   compare the rehashed destination with the reported hash,
   rather than relying on which channel's message arrives first.
- Variant 2b,
   a long-lived Node worker that re-imports the config with a cache-busting query:
   rejected,
   because module records accumulate,
   module-global singletons persist,
   and a running import cannot be cancelled except by killing the worker
   (`doc/planning/monorepo-manager-route-research/from-scratch-inputs.md:150-160`, verified).

### R1 per-task cgroup v2

As "Shared Rust core evidence",
"R1 per-task cgroup v2",
with each Node evaluation in its own task cgroup.

- `cgroup.kill` ends the Node child and anything it spawned through `nano-spawn` or `exec`.
- Killing a child that holds the staleness manifest lock leaves a lock owned by a dead PID,
   which the next run recovers
   (dead-owner recovery,
   `src/io/staleness-manifest-lock-recovery.ts:83-103`, per the research doc).
- Freezing a lock holder makes other writers time out after 5 s
   (`src/io/staleness-manifest-lock.ts:21-31`, `:165-172`, per the research doc).
  The daemon never runs two evaluations of one config,
   so only manual CLI runs outside the daemon hit that timeout (inference).

### R2 recursive watching

As "Shared Rust core evidence",
"R2 recursive watching",
plus:

- file-enforcer's own `--watch` mode is not used in daemon mode;
   it stays for standalone CLI use or is retired.
  Its `watch` directory holds 9 files and 1,027 code lines (`SCR/rust-fe-count.ts`).
- The watch-set rules move to Rust:
   parent directories of tracked reads,
   writes,
   and the config,
   plus glob static roots (`src/watch/watch-filter.ts:43-66`, per the research doc).
- The protected-versus-echo rule today compares `floor(mtimeMs)` with the recorded write time
   (`src/watch/watch-filter.ts:149-179`, per the research doc);
   the daemon replaces it with a content-hash comparison,
   because inotify events and descriptor-3 events arrive on separate channels with no ordering guarantee (inference).

### R3 Unix socket JSON-RPC 2.0

As "Shared Rust core evidence",
"R3 Unix socket JSON-RPC 2.0".
The child event protocol reuses the same envelope and schema.

### R4 btrfs

As "Shared Rust core evidence",
"R4 btrfs".

### R5 hashing

As "Shared Rust core evidence",
"R5 hashing".
file-enforcer keeps its SHA-256 destination checks for its own manifest,
while the daemon's cache uses BLAKE3;
the two never share keys.

### R6 process supervision

As "Shared Rust core evidence",
"R6 process supervision",
plus:

- A non-zero Node exit marks the evaluation failed;
   the daemon relays the last `stderr` lines.
- Cancelling an evaluation is `cgroup.kill`;
   destination writes are atomic renames (`src/io/write-atomic.ts`, per the research doc),
   so a killed child leaves old or new content,
   not torn files (inference).

### R7 file-enforcer interop

- Per-evaluation startup floor:
   median 146.8 ms (band 143.3 to 160.0 ms) for Node plus importing the library,
   before the config runs (Node child startup floor probe).
  Bare Node startup is 20.7 ms median,
   so the library import costs roughly 126 ms of the floor (arithmetic on the medians).
- TypeScript changes needed:
   the daemon entry and event emission.
  The tracker already exposes `reads`,
   `writes`,
   `globs`,
   and `writeTimestamps` (`src/index.ts` exports, verified by reading),
   and per-builder capture uses `AsyncLocalStorage` (`src/tracker-capture.ts:62`, per the research doc).
  A fresh child per evaluation isolates the module-global singletons
   (`src/tracker.ts:21-38`, per the research doc).
- Input tracking gaps at the boundary:
  - Modules imported by the config are not stamped today
     (`doc/planning/monorepo-manager-route-research/from-scratch-inputs.md:124-125`, verified).
  - `module.registerHooks` can observe resolution and loading;
     Node documents it as "Added in: v23.5.0, v22.15.0" and "Stability: 1.2 - Release candidate"
     (<https://nodejs.org/api/module.html>, verified by curl).
    Whether its hooks see every import in this config is unverified.
  - Direct `readFile`,
     `glob`,
     and `nano-spawn` calls in the root config bypass the tracker
     (`file-enforcer.config.ts:1-13`, per the research doc).
    This gap exists in Option 1 too,
     unless a rewrite forces all I/O through tracked APIs.
- Schema:
   one schema file for the notification envelope drives Rust `serde` types and a TypeScript contract test;
   nothing in the repository generates both today (inference from the absence of such a tool in the file lists read).
- Node documentation loaded with the user's curl command:
   <https://nodejs.org/api/child_process.html> (`stdio`),
   <https://nodejs.org/api/esm.html> (`import`),
   <https://nodejs.org/api/module.html> (`registerHooks`).

### R8 repository fit

- Everything in "Shared Rust core evidence",
   "R8 repository fit".
- file-enforcer stays TypeScript,
   matching the proposal's TypeScript default for repository automation
   (`doc/planning/load-bearing-code-languages.md:82`, verified).
- The root config keeps satisfying `AD2` (`AGENTS.md:1737-1740`, verified).
- `vm-builder` and the perf fixture keep importing the TypeScript API unchanged.
- The daemon needs Node 26 with type stripping on `PATH` for evaluations;
   the repository already requires it,
   and tool provisioning stays with its current owner (design doc "Migration from Mise").

### R9 documentation quality

- Shared triggers apply ("Shared Rust core evidence",
   "R9 documentation quality").
- No Rust crate is added beyond the shared core.
- Node pages loaded as listed in "R7 file-enforcer interop";
   `module.registerHooks` is a release-candidate API,
   documented as such.

### R10 maturity and maintenance

- Shared crates only ("Shared Rust core evidence",
   "R10 maturity and maintenance").
- file-enforcer's existing npm dependencies are unchanged;
   `chokidar` becomes unused in daemon mode.

### Yikes

Ranked by severity,
most severe first.

1.  **The sandbox rests on repository-written `unsafe` spawn code.**
    Same as Option 1's item 4.
    It is the most severe item here because sandboxing is a hard requirement,
    no crate fits,
    and nothing could be exercised in this session.
2.  **Watch correctness is repository-written.**
    Same as Option 1's item 5.
    Additionally,
    the file-enforcer watch-set and protection rules are re-derived in Rust,
    and write echoes cross two unordered channels,
    so classification must use content hashes.
3.  **Two-sided event protocol.**
    Rust `serde` types and the TypeScript emitter can drift;
    the repository has no generator for both,
    so a schema file with contract tests on both sides is required.
4.  **Input tracking depends on a release-candidate Node API.**
    Capturing config-imported modules needs `module.registerHooks` (Stability 1.2);
    without it,
    edits to imported workspace modules do not trigger reruns.
    Direct untracked I/O in the config remains a gap in both options.
5.  **Supervision and RPC are hand-rolled.**
    Same as Option 1's item 6.
6.  **Toolchain and release drift.**
    Same as Option 1's item 7.
7.  **Per-evaluation floor.**
    Every rerun pays about 147 ms (median) of Node startup and library import before the config runs,
    and freezing a lock-holding child stalls external CLI writers for 5 s.
8.  **btrfs subvolume operations need hand-written `#[repr(C)]` ioctls.**
    Same as Option 1's item 8.

## Cross-option comparison

Option 2 has the fewest and least severe yikes.

- **Option 1 contains every Option 2 core yike.**
   The cgroup spawn path,
   watcher,
   supervision,
   RPC,
   toolchain,
   and btrfs items come from the shared Rust core,
   so they appear in both lists.
- **Option 1's unique yikes are more severe than Option 2's.**
   Option 1 adds a config-host problem with no acceptable variant,
   a rewrite whose interim state is Option 2 (79 modules and 6,508 hand-written code lines,
   plus 56 test files and TypeScript consumers),
   and unproven output parity.
   Option 2's unique yikes are a two-sided protocol,
   a release-candidate Node API for module tracking,
   and a measured 147 ms per-evaluation floor.
   Each has a concrete mitigation:
   a schema with contract tests,
   an explicit fallback of watching the config's workspace dependencies,
   and serial evaluations.
- **Option 2's worst unique item also affects Option 1.**
   The re-derived watch and protection rules with unordered echoes exist in Option 1 variant 1a too,
   which also evaluates configs in a child process.
   Variants 1b and 1c avoid that ordering problem only by breaking the daemon model or building a JavaScript host.
- **Option 1 buys no sandbox or watcher advantage.**
   The disqualifying risks of a Rust daemon,
   spawn-into-cgroup correctness and watch correctness,
   are identical in both options.
   Rewriting file-enforcer changes only the enforcement layer,
   where it adds cost and risk.
- **Remaining unknowns cut the same way for both options.**
   The unverified cgroup behavior on this host,
   the `fork` cost of `pre_exec` spawns,
   and the CI runners' user systemd sessions apply equally.
   A disposable delegated-scope prototype that spawns,
   freezes,
   and kills a task is the next verification;
   it creates cgroups,
   which this session did not do.

Scope note:
AGENTS.md gained rule `YKZ` during this session (commit `8c76a8169`),
which asks for widening to plausible alternatives.
This report designed variants inside the two assigned options
(1a to 1d,
and 2b);
other stacks such as TypeScript on Node or Kotlin/Native were outside the assignment.
