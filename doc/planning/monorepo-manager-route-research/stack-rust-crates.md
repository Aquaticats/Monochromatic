# Rust crate survey for the from-scratch monorepo manager

Outcome on 2026-09-16:
the option this survey served,
a Rust core with TypeScript file-enforcer children,
is out over single-file size.
Its crate findings for the daemon core carry over to the remaining all-Rust route;
see "Current stack direction" in `doc/planning/monorepo-manager-from-scratch-design.md`.

Research date: 2026-09-16.
Scope: library-level redo of the Rust daemon core research in
`doc/planning/monorepo-manager-route-research/stack-rust.md`,
for the option "Rust daemon core with the TypeScript file-enforcer run as Node child processes"
in `doc/planning/monorepo-manager-from-scratch-design.md`.
Nothing under `/var/home/user/Monochromatic` was modified.
No cgroup, systemd unit, or scope was created.

## Conventions

- **Verified** means one of:
   a source file and line range read in this session;
   a documentation page loaded with curl in this session and the quoted text found in it;
   or a probe whose command and output are recorded in "Probes run".
- **Unverified** means inference, recall, or a behavior that needs a cgroup or unit this session could not create.
- Documentation problems are recorded and never cull a crate, per the user's decision.
  A crate leaves the design only for requirement, correctness, or maintenance reasons.
- Path abbreviations:
  - `SRC/` is `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad/rust-crates/src/`,
     published `.crate` sources downloaded from `static.crates.io` and extracted
     (what a Cargo user compiles, not a Git checkout).
  - `SD/` is `SCR/rust-crates/gh/systemd/systemd/v259/`,
     single files fetched with `gh api repos/systemd/systemd/contents/<path>?ref=v259`.
     The host runs systemd `259.8-1.fc44`.
  - `KB/` is `SCR/rust-crates/gh/torvalds/linux/master/fs/btrfs/ioctl.c`,
     fetched from `torvalds/linux` `master` today.
  - `STD/` is `~/.rustup/toolchains/nightly-2026-09-16-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/`.
  - `PROBE/` is `SCR/rust-crates/probe/`, the throwaway Cargo project.
  - `DOCS/` is `SCR/rust-crates/docs/`, pages saved by `SCR/rust-crates/docs.ts`.
  - `SCR/` is `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad/`.
- Dependency counts are unique packages including the crate itself,
   `cargo tree --offline --edges normal --target x86_64-unknown-linux-gnu`,
   default features unless the feature set is named
   (`SCR/rust-crates/deptree2.ts`, results in `deptree2.txt` and `deptree3.txt`).
  Transitive versions come from the local registry cache,
   so they can be older than the newest release;
   counts for crates marked "direct" are crates.io direct dependency counts only.
- Maintenance metadata comes from the crates.io API and `gh api repos/<owner>/<repo>`,
   fetched today (`SCR/rust-crates/meta.ts`, results in `meta.txt`).
  "Releases in the past year" counts non-yanked versions published since 2025-09-16.
- docs.rs coverage is the "N% of the crate is documented" text on the loaded page.

## Repository incumbents

Measured with `rg --glob 'Cargo.toml'` and the lockfiles of the two `zbus` users (verified):

- `zbus`:
   `package/cli/nested-wayland-session/Cargo.toml:72` (`5.19.0`, blocking API, default features)
   and `package/music-player/desktop-app/Cargo.toml:155` (`5`, default features, locked `5.16.0`).
- `tokio`:
   `rt`, `sync`, `macros` in `package/music-player/truepeak-core/Cargo.toml:57`,
   `package/music-player/android-app/rust/Cargo.toml:81`,
   and `package/music-player/desktop-app/Cargo.toml:116`.
- `ignore = "0.4"`:
   `package/linter/rust/Cargo.toml:69`,
   `package/forbidden-strings/cli/Cargo.toml:76`,
   `package/forbidden-strings/core.bench/Cargo.toml:38`.
- `serde_json = "1"`:
   `package/linter/rust`, `package/desktop-app/file-manager-gtk-sticky`,
   `package/music-player/truepeak-core.bench`, `package/rust-module/rust-linter-core`,
   `package/music-player/desktop-app`.
- Not listed by the caller but present:
  - `notify-debouncer-mini = "0.7"` in `package/music-player/desktop-app/Cargo.toml:81`,
     used with `RecursiveMode::Recursive` in `src/watch.rs:188`;
     its lockfile pins `notify 8.2.0` and `inotify 0.11.2`.
  - `libc = "0.2"` in `package/wg/cli-quicker-exempt/Cargo.toml:33`
     and `package/music-player/desktop-app/Cargo.toml:185`.
  - The desktop app lockfile also carries `nix 0.31.3` and `rustix 1.1.4` transitively.
- Repository `unsafe` precedent:
   `rg --count-matches '\bunsafe\s*(\{|fn|impl)'` found `unsafe` in 25 files,
   for example `package/wg/cli-quicker-exempt/src/keeper_process.rs:120`, `:166`, `:175`
   (`libc::waitpid` and `libc::kill`).
  No lint forbids `unsafe_code`
   (`rg 'unsafe_code|undocumented_unsafe_blocks'` over `*.toml`, `*.ts`, `*.rs` exited 1),
   and the repository Rust linter sources contain no `unsafe` rule
   (`rg --files-with-matches --ignore-case unsafe` over the linter packages exited 1).
- Rules that apply to every design:
   `MXR` (300 code lines per `.rs` file) and `RDC` (rustdoc on every documentable item, private included).
  A local `zbus` `#[proxy]` trait needs `///` on every method;
   depending on a generated proxy crate moves that code out of the repository.

## Probes run

### Process note on fetching crates

`cargo` could not reach `index.crates.io`:
`CARGO_HTTP_DEBUG=true` showed every IPv4 and IPv6 connect failing with "Software caused connection abort",
while `curl` and Node `fetch` to the same host succeeded,
which points at a per-application network policy (unverified which one).
The policy was not bypassed.
Instead, `.crate` files were downloaded with Node from `static.crates.io`,
and probes were built with `cargo build --offline`
against the local registry cache plus `[patch.crates-io]` path entries for uncached crates
(`SCR/rust-crates/resolve-loop.ts`).
The probe therefore used cached `inotify 0.11.4`, `rustix 1.1.4`, and `tokio-util 0.7.18`
where the newest releases are `0.11.5`, `1.1.5`, and `0.7.19`.

### Probe list

Every probe binary starts with `#![forbid(unsafe_code)]` and compiled,
so none of the repository-side code below needs `unsafe` (verified).
Temporary files went to the scratchpad except the first `watch` run,
which used `/tmp/watch-probe-*`;
those directories were removed afterward.

- `PROBE/src/bin/launcher.rs`, release build:
  - Positive control:
     the probe spawns itself as `launch <own cgroup>/cgroup.procs -- /usr/bin/cat /proc/self/cgroup`;
     the launcher writes its PID with `std::fs::write`, then calls `CommandExt::exec`.
    Output:
     `status=exit status: 0 task stdout="0::/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash"`.
    The target was the probe's own cgroup,
     so this proves ordering and fail-closed flow,
     not a migration into a different cgroup.
  - Negative control:
     `launch /sys/fs/cgroup/cgroup.procs -- /usr/bin/touch <marker>` printed
     `status=exit status: 125 stderr="launcher: placement into /sys/fs/cgroup/cgroup.procs failed: Permission denied (os error 13)" marker_exists=false`.
  - Cost, three alternating series of 30 sequential spawns each, `tokio::process` with `process_group(0)`:
    - direct `/usr/bin/true`: medians 0.550, 0.529, 0.523 ms; minimums 0.454 to 0.466 ms.
    - launcher then `/usr/bin/true`: medians 1.433, 1.421, 1.351 ms; minimums 1.167 to 1.184 ms;
       one maximum of 16.563 ms in series 1, other maximums under 2 ms.
    - The launcher adds about 0.8 to 0.9 ms median per spawn;
       the difference exceeds the direct-spawn run band (0.523 to 0.550 ms).
- `PROBE/src/bin/watch.rs`, scratch directories on tmpfs:
  - `notify 8.2.0`, non-recursive watch, read-only `std::fs::read`:
     `["Ok(Access(Open(Any))) Ok(1)"]`.
    Write (positive control):
     `Access(Open)`, `Modify(Data)`, `Access(Close(Write))`.
  - `notify 8.2.0`, recursive watch, `create_dir_all` of 10 levels plus a file in the deepest:
     11 events, one `Create(Folder)` and ten `Access(Open(Any))`; no event for the deep file.
  - `notify 9.0.0-rc.5` with `Config::with_event_kinds(EventKindMask::CORE)`, read-only open: `[]`.
    Write (positive control): `["Ok(Modify(Data(Any))) Ok(1)"]`.
  - `notify 9.0.0-rc.5`, recursive, same nested creation: 1 event, `Create(Folder)`; no event for the deep file.
  - `inotify 0.11.4`, mask without `OPEN`, read-only open: `no events: WouldBlock`;
     write: `EventMask(CLOSE_WRITE) Some("f.txt")`.
  - `filesentry 0.2.2` with a `Filter` ignoring directories named `ignored`:
     root crawl `Ok(true)`;
     read-only open `[]`;
     a write under `ignored/` plus the 10-level nested creation produced exactly one event,
     `Create .../n1/.../n10/deep.txt`.
- `PROBE/src/bin/repo_watch.rs`, release build, read-only over the repository,
   excluding directory names `.git`, `node_modules`, `target`, `dist`, three runs:
  - `ignore::WalkBuilder` walk: 5,481 directories in 104.3, 127.8, 102.5 ms.
  - `inotify 0.11.4`, one `Watches::add` per directory: 32.2, 39.6, 33.7 ms, 0 failures.
  - `notify 8.2.0`, one `watch(dir, NonRecursive)` per directory: 123.7, 144.1, 120.7 ms, 0 failures.
  - `filesentry 0.2.2` recursive root with a name filter, until the crawl callback: 378.7, 418.8, 376.9 ms.
- `PROBE/src/bin/rpc.rs`, two `tokio::net::UnixStream::pair()` connections,
   newline-delimited JSON with `tokio-util` `LinesCodec`,
   every line passed to `jsonrpsee` `RpcModule::raw_json_request`:
  - `A ping -> {"jsonrpc":"2.0","id":1,"result":"pong"}`.
  - `A subscribe -> {"jsonrpc":"2.0","id":2,"result":7162361724085009}`,
     then `{"jsonrpc":"2.0","method":"tick","params":{"subscription":7162361724085009,"result":0}}`.
  - A JSON-RPC batch array on connection B:
     `{"error":{"code":-32700,"message":"invalid type: map, expected a string \"2.0\" at line 1 column 1"},...}`.
  - Connection B sent `unsubscribeTicks` with A's subscription ID:
     `{"jsonrpc":"2.0","id":5,"result":true}`;
     A then received one final line,
     `{"jsonrpc":"2.0","method":"tick","params":{"subscription":7162361724085009,"error":"The connection channel is closed"}}`.
  - `LinesCodec::new_with_max_length(16)` over a 40-byte line then `short`:
     `["Err(MaxLineLengthExceeded)", "None", "Ok(short)", "None"]`.
- `PROBE/src/bin/dbus.rs`, read-only calls on the user bus through `zbus_systemd`:
  - `connect+proxy: 1.189ms` (one sample).
  - `user manager Version: Ok("259.8-1.fc44")`.
  - `GetUnitByPID(self): Ok(OwnedObjectPath(ObjectPath("/org/freedesktop/systemd1/unit/app_2eslice")))`.
  - Built but did not send transient-unit values:
     `ExecStartEx value signature: a(sasas)`, `StandardOutputFileDescriptor value signature: h`.
- `busctl --user introspect org.freedesktop.systemd1 /org/freedesktop/systemd1 org.freedesktop.systemd1.Manager`
   listed `AttachProcessesToUnit ssau`, `FreezeUnit s`, `KillUnit ssi`, `KillUnitSubgroup sssi`,
   `QueueSignalUnit ssii`, `StartTransientUnit ssa(sv)a(sa(sv)) o`, `Subscribe`, `ThawUnit s`,
   `JobRemoved uoss`, `UnitRemoved so`.
- `PROBE/src/bin/fsops.rs`:
  - On `~/temp/agent/reflink-probe-2026-09-16` (btrfs, removed afterward):
     `statfs f_type=0x9123683e`;
     `rustix ioctl_ficlone into temp: Ok(())` then rename, destination length 1,048,576;
     `reflink_copy::reflink onto existing dest: Err(AlreadyExists)`;
     `reflink_copy::reflink onto fresh path: Ok(())`;
     directory afterward `["blob.bin", "dest.bin", "fresh.bin"]`;
     negative control `rustix ioctl_ficlone btrfs -> tmpfs: Err(Os { code: 18, kind: CrossesDevices, ... })`.
  - Unprivileged fanotify through `nix 0.31.3`:
     `fanotify_init(NOTIF|REPORT_DFID_NAME) unprivileged: Ok("ok")`;
     inode mark (positive control) `Ok(())`;
     filesystem mark `Err(EPERM)`;
     mount mark `Err(EPERM)`.
- `btrfs-uapi 0.13.0` offline debug build in `SCR/rust-crates/deptree2/dt-34`:
   finished, and its build script wrote a 152,443-byte `bindings.rs`,
   so `bindgen` found `libclang` (`/usr/lib64/libclang.so.22.1` exists) on this host.
- Host checks:
   `command -v watchman` printed nothing;
   `/usr/include/btrfsutil.h` does not exist.

Not run, with reasons:
creating a cgroup, a scope, or a service
(needed to exercise migration into a new task cgroup, `cgroup.freeze`, `cgroup.kill`, `FreezeUnit`, and `KillUnit`)
was forbidden by the caller.

## C1 per-task cgroup placement, freeze, and kill

### Kernel and systemd facts every design relies on

- `cgroup.events` changes generate file-modified events
   ("a value change in this file generates a file modified event",
   `SCR/rust-stack/docs/kernel-cgroup-v2.txt`, verified),
   so freeze completion (`frozen 1`) and emptiness (`populated 0`) can be awaited with inotify.
- systemd `KillUnit` with whom `all` or `cgroup` and `SIGKILL` writes the unit's `cgroup.kill`,
   falling back to enumeration only on `EOPNOTSUPP`
   (`SD/src/core/unit.c:4111-4146`; `SD/src/basic/cgroup-util.c:435-453`, verified).
- systemd `FreezeUnit` writes `cgroup.freeze` (`SD/src/core/cgroup.c:4108-4112`)
   and defers the D-Bus reply until the kernel reports the frozen state
   (`SD/src/core/dbus-unit.c:836-848`, `:1878-1901`; `SD/src/core/unit.c:6474-6488`, verified).
  It refuses while the unit has a pending job, "Unit has a pending job"
   (`SD/src/core/unit.c:6500-6501`; `SD/src/core/dbus-unit.c:824`, verified).
- For transient services, the service manager expands `${VARIABLE}` in `ExecStart` arguments;
   `systemd-run` switches to `ExecStartEx` with the `no-env-expand` flag to prevent it
   (`SD/src/run/run.c:1426`, `:1625-1650`; `SD/man/systemd-run.xml:99-102`, `:189-196`, verified).
- `systemd-run --scope` registers its own PID (or pidfd) as the scope's `PIDs`/`PIDFDs`,
   waits for the job, then `execvpe`s the command
   (`SD/src/run/run.c:1671-1697`, `:2674-2740`, `:2869`, verified).
  `PIDs` value `0` means the D-Bus sender (`SD/src/core/dbus-scope.c:88-141`, verified).
- Transient units accept dependency properties such as `BindsTo`
   (`SD/src/core/dbus-unit.c:2557`, verified),
   and `systemd-run` sets `AddRef` to "Pin the object as least as long as we are around"
   when connected through the bus broker (`SD/src/run/run.c:1342-1350`, verified).

### Candidates

- `zbus` 5.19.0 (2026-08-09), incumbent:
  - Maintenance: 10 releases in the past year; `z-galaxy/zbus` pushed 2026-09-17; 74 open issues.
  - License MIT.
  - Dependencies: 66 packages with default features; 55 with only `tokio`.
  - Documentation: not re-audited here; the probe used it through `zbus_systemd` without surprises.
- `zbus_systemd` 0.26100.0 (2026-06-26):
  - Maintenance: 5 releases in the past year; `lucab/zbus_systemd` pushed 2026-06-26; 2 open issues;
     the release regenerated proxies from systemd 261 definitions (commit "zbus_systemd: release 0.26100.0").
  - License `MIT/Apache-2.0` (slash form, not an SPDX expression).
  - Dependencies: 56 packages with `systemd1` and `zbus-async-tokio`.
  - API: async-only proxies (`gen_blocking = false`),
     `kill_unit(name, whom, signal)` (`SRC/zbus_systemd-0.26100.0/src/systemd1/generated.rs:144`),
     `freeze_unit` (`:172`), `thaw_unit` (`:176`),
     `start_transient_unit(name, mode, Vec<(String, OwnedValue)>, aux)` (`:224-230`),
     `attach_processes_to_unit` (`:238-243`), `subscribe` (`:407`), `JobRemoved` signal (`:690-691`), verified.
  - Documentation problems:
     docs.rs reports 71.63% documented (verified);
     every method's rustdoc is only a link to `systemd.directives.html#<Method>()` plus "Call interface method `X`"
     (verified),
     so semantics live entirely in systemd's pages;
     whether those anchors resolve is unverified,
     because `freedesktop.org` answered HTTP 418 "Checking you are not a bot".
     Without either async feature the crate "will likely get a build error" (README, verified).
- `systemd-zbus` 5.3.2 (2025-05-20):
  - Maintenance: 0 releases in the past year; repository on GitLab (`flukejones/systemd-zbus`), activity not fetched.
  - License `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`.
  - Dependencies: 71 packages with `zbus` `tokio`.
  - API covers `freeze_unit`, `kill_unit`, `start_transient_unit` with borrowed `Value`s
     (`SRC/systemd-zbus-5.3.2/src/generated/manager.rs:118`, `:172`, `:339-345`, verified).
  - Documentation problems: docs.rs 42.12% documented; method docs restate the name, for example "FreezeUnit method"
     (verified).
- `unitbus` 0.1.7 (2026-01-23):
   system bus only (`SRC/unitbus-0.1.7/src/bus.rs:48-55`; README "over the **system D-Bus**", verified),
   no freeze API found (`rg freeze` over `src` matched nothing, verified),
   transient tasks set `ExecStart` without the no-expand flag (`src/units.rs:841`, verified).
  MIT; 0 GitHub stars.
  Excluded: the daemon talks to the user manager.
- `systemdzbus` 0.1.7 (2026-03-05, license `Beerware`, 1 star) and `systemd_client` 0.2.1 (2022-01-23):
   not examined beyond metadata.
- `libcgroups` 0.7.0 (2026-07-25), from youki:
  - Maintenance: 4 releases in the past year; `youki-dev/youki` pushed 2026-09-14; 140 open issues; 7,602 stars.
  - License Apache-2.0.
  - Dependencies: 66 packages with only `v2`; depends on `oci-spec`, `procfs`, `pathrs`, and `nix 0.29.0`
     (the desktop app already locks `nix 0.31.3`).
  - Behavior:
    - `v2::manager::Manager::new(root_path, cgroup_path)` joins paths, so the root can be the delegated cgroup;
       controller discovery reads `root_path/cgroup.controllers`
       (`SRC/libcgroups-0.7.0/src/v2/manager.rs:85-97`; `src/v2/util.rs:56-79`, verified).
    - `add_task(pid)` creates the cgroup and then writes the PID, so placement happens after the process exists
       (`src/v2/manager.rs:118-168`, `:208-214`, verified).
    - `with_rootless(true)` turns `EACCES`/`EROFS` into a debug log and "leaving process in its parent cgroup"
       (`:104-108`, `:148`, `:174`, verified): with it set, the sandbox can be silently off.
    - `enable_controllers` ignores every write error (`:187-201`, verified).
    - `remove` writes `cgroup.kill`, then retries `rmdir` 4 times with 100 ms sleeps (`:244-263`, verified).
    - Freezing polls `cgroup.events` with `thread::sleep(10 ms)` up to 1,000 times
       (`src/v2/freezer.rs:105-136`, verified), so it blocks a thread for up to 10 s.
    - Unknown controllers log `tracing::warn!("Controller {} is not yet implemented.")` (`src/v2/util.rs:74`);
       this host delegates `dmem`, so every discovery would warn (inference).
    - Its systemd manager is its own D-Bus client describing units as "youki container" and attaching `PIDs` after start
       (`src/systemd/dbus_native/dbus.rs:485`, `:503`, verified).
  - Documentation problems:
     docs.rs 26.06% documented;
     `README.md` is one line, `# libcgroups` (verified);
     `Manager::new` says the root is "the mount point of a cgroup v2 fs" while any cgroup v2 directory works (verified);
     `with_rootless` renders an unresolved `[libcontainer]` link (verified in `DOCS/libcgroups-v2-manager.txt`);
     a missing root path returns "non default cgroup root not supported" (`src/common.rs:275`, `:317`, verified).
- `cgroups-rs` 0.5.1 (2026-07-14), re-verified against the published crate:
  - Maintenance: 2 releases in the past year; `kata-containers/cgroups-rs` pushed 2026-08-20; 23 open issues.
  - License MIT OR Apache-2.0.
  - Dependencies: 73 packages; `zbus = "5.8"` non-optional; `nix = "0.25.0"`.
  - Prior claims hold at the same lines:
     ancestor `cgroup.subtree_control` writes with discarded results
     (`SRC/cgroups-rs-0.5.1/src/fs/cgroup.rs:83-92`, `:512-517`, `:545-569`),
     controller list from the global root (`:519-527`),
     freezer state from `cgroup.freeze` (`src/fs/freezer.rs:113-125`),
     and `destroy` moving processes to the root with errors ignored (`src/manager/fs.rs:770-790`), verified.
  - Newly found:
     `hierarchies::V2::new()` hard-codes `/sys/fs/cgroup` and has no root parameter (`src/fs/hierarchies.rs:323-330`,
     `:339`),
     and the systemd manager uses `zbus::blocking::Connection::system()` only (`src/systemd/dbus/proxy.rs:6`, `:12`),
     verified.
  - Documentation problems:
     docs.rs 71.87% documented;
     `V2::new` says it "Finds where control groups are mounted" while returning a constant (verified);
     the prior page problems stand.
- `processkit` 3.3.4 (2026-08-22):
  - Maintenance: 52 releases in the past year; `ZelAnton/ProcessKit-rs` pushed 2026-09-15; 3 open issues; 41 stars.
  - License MIT.
  - Dependencies: 25 packages.
  - Behavior:
    - Per-spawn leaf cgroup joined in a `pre_exec` closure inside the crate
       (`SRC/processkit-3.3.4/src/sys/linux.rs:195-213`, verified).
    - The group cgroup is a child of the process's own cgroup read from `/proc/self/cgroup` (`:916-932`, verified).
    - Falls back to POSIX process groups "when no writable cgroup is available" (`src/sys/linux.rs:1-3`, verified);
       `ProcessGroup::mechanism()` reports which one is active (`src/group.rs:1437`).
    - `suspend` writes `cgroup.freeze` and returns without waiting for `frozen 1` (`src/sys/linux.rs:2171-2185`,
       verified).
    - Resource limits need "this process at the *real* cgroup root", "not under any systemd session/scope/service"
       (`src/limits.rs:33-52`, verified), because the crate enables controllers in its own occupied cgroup.
  - Documentation: docs.rs 100% documented; no contradiction found in the sections read.
- `rustix` 1.1.5 (2026-09-16):
   Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT;
   3 releases; pushed 2026-09-16; 143 open issues;
   3 packages with `process` and `fs`;
   safe `pidfd_open` (`SRC/rustix-1.1.5/src/process/pidfd.rs:29`), `pidfd_send_signal` (`:41`),
   `kill_process_group` (`src/process/kill.rs:35`), `waitid` (`src/process/wait.rs:488`);
   no `clone3`, no spawn with cgroup (verified by `rg` over `src/process`).
  docs.rs 100% documented.
- `nix` 0.31.3 (2026-05-11):
   MIT; 4 releases; pushed 2026-05-19; 359 open issues;
   5 packages with `fanotify`, `inotify`, `process`, `signal`;
   `nix::spawn` wraps `posix_spawn` (`SRC/nix-0.31.3/src/lib.rs:212`) without a cgroup attribute;
   `sched::clone` and `unistd::fork` are `unsafe` (recall; not re-read).
  docs.rs 99.93% documented; fanotify documentation problems are under C2.
- `libc` 0.2.189 (2026-07-21) and `1.0.0-alpha.4` (2026-07-21):
   MIT OR Apache-2.0; 17 releases; pushed 2026-09-15.
  `CLONE_INTO_CGROUP` is still the deprecated overflowing `c_int` in 0.2.189
   (`SRC/libc-0.2.189/src/unix/linux_like/linux/gnu/mod.rs:871-876`, verified),
   but `1.0.0-alpha.4` declares `c_ulonglong = 0x200000000`
   (`SRC/libc-1.0.0-alpha.4/src/unix/linux_like/linux/mod.rs:3273`, verified),
   and `linux-raw-sys` 0.12.1 declares `u64 = 8589934592` (`SRC/linux-raw-sys-0.12.1/src/x86_64/general.rs:1906`,
   verified).
  Neither `libc` version, `nix`, nor `rustix` binds `pidfd_spawn` or `posix_spawnattr_setcgroup_np`
   (`rg --count-matches 'setcgroup|pidfd_spawn'` over all four exited with no output, verified).
- `clone3` 0.2.3 (2022-12-07):
   MIT; last commit 2022-12-07;
   has `flag_into_cgroup` (`SRC/clone3-0.2.3/src/wrapper.rs:61-63`) but `pub unsafe fn call` (`:212`), verified.
- `command-fds` 0.3.3 (2026-04-10):
   Apache-2.0; 1 release in the past year; `google/command-fds` pushed 2026-08-18;
   17 packages with `tokio`;
   safe `CommandFdExt::fd_mappings` that installs a `pre_exec` closure internally
   (`SRC/command-fds-0.3.3/src/lib.rs:129-151`, verified), which disables `std`'s `posix_spawn` path (see below).
  docs.rs 86.67% documented.
- `pidfd` 0.2.4 (2019-12-20), `pidfd-util` 0.1.0 (2026-03-03, 4 stars), `async-pidfd` 0.1.5 (2025-05-21),
   `mio-pidfd` 0.4.0 (2024-10-25):
   pidfd wrappers only, no placement; `tokio::process` already uses pidfds on Linux (C4).
- `process-wrap` 10.0.0 (2026-08-24):
   process groups and sessions; no cgroup support (prior research); not re-read.
- `hakoniwa` 1.7.2 (2026-07-05, LGPL-3.0-only WITH LGPL-3.0-linking-exception):
   namespace and cgroup sandbox for running commands; license and container scope were judged out of fit;
   not examined further.
- `std` on the repository's nightly:
  - `CommandExt::exec` is a safe, stable function (`STD/os/unix/process.rs:134-163`, verified).
  - `pre_exec` is `unsafe` (`:110`), and any closure disables the `posix_spawn` fast path
     (`STD/sys/process/unix/unix.rs:469-476`, verified).
  - `std::io::pipe` is stable since 1.87.0 (`STD/io/pipe.rs:81`, verified).
  - No placement API; `linux_pidfd` is still unstable (`STD/os/linux/process.rs:5`, verified).
- `systemd-run --user --scope --expand-environment=no` CLI per task: see design C1-D.

### Designs

#### C1-A: cgroupfs under a delegated cgroup plus a safe re-exec launcher

- Spawn:
   `tokio::process::Command::new("/proc/self/exe")` with
   `["__launch", "<task cgroup>/cgroup.procs", "--", argv...]`,
   `process_group(0)`, `stdin(Stdio::null())`, piped `stdout` and `stderr`.
  The launcher mode writes `std::process::id()` to the path with `std::fs::write`;
   on error it prints a prefixed message and exits 125, so the task never runs;
   on success it calls `Command::new(argv0).args(rest).exec()`.
- The task is the daemon's direct child after `exec`,
   so `tokio::process` reaps it through its pidfd reaper and the pipes are ordinary.
- Setup, freeze, kill:
   `mkdir tasks/<id>`, write limits (`memory.max`, `cpu.max`, `pids.max`, `memory.oom.group`),
   freeze with `cgroup.freeze` then await `frozen 1` through an `inotify` watch on `cgroup.events`,
   kill with `cgroup.kill` then await `populated 0` and `rmdir`.
- Delegation:
   either the user starts the daemon with `systemd-run --user --scope -p Delegate=yes`,
   or the daemon calls `StartTransientUnit` once at startup with `PIDs=[0]` and `Delegate=yes`
   through `zbus_systemd`, as `systemd-run --scope` does for itself.
  The self-delegation path is unverified by experiment.
- Evidence:
   launcher probe (fail-closed and ordering verified, migration into a new cgroup unverified);
   0.8 to 0.9 ms median overhead per spawn (verified);
   no `pre_exec` closure, so `std` keeps `posix_spawn` (verified).
- `unsafe` in repository code: none (verified by `#![forbid(unsafe_code)]` compile).
- Crates: `tokio` (incumbent), `inotify` (transitive incumbent), optionally `zbus_systemd` for self-delegation.
- Disqualifying problems found: none.
  Risks:
   the cgroup file protocol (controller enabling, no-internal-process rule, cleanup after crash) is repository code;
   exec of `/proc/self/exe` running the original binary after an on-disk rebuild is unverified
   (inference from Linux `exe` link semantics; the probe used `std::env::current_exe()`);
   a launcher failure (exit 125) is indistinguishable from a task that exits 125
   unless the daemon also matches the launcher's `stderr` prefix or adds a status channel.

#### C1-B: systemd transient scope plus a launcher handshake

- Spawn the same launcher, but with `stdin` as a pipe from the daemon;
   the launcher blocks reading one byte.
- The daemon opens a pidfd for the child (`rustix::process::pidfd_open`),
   calls `StartTransientUnit` for `<tool>-task-<id>.scope` with mode `fail` and properties
   `PIDFDs=[pidfd]`,
   `MemoryMax`,
   `CPUQuotaPerSecUSec`,
   `TasksMax`,
   `AddRef=true`,
   and `BindsTo=` the daemon's unit,
   waits for `JobRemoved` for that job, then writes the byte;
   the launcher calls `Command::new(argv0).stdin(Stdio::null()).exec()`.
- Freeze and resume: `FreezeUnit` and `ThawUnit` (reply deferred until frozen, verified in source).
  Kill: `KillUnit(name, "cgroup", SIGKILL)` (uses `cgroup.kill`, verified in source).
- `unsafe` in repository code: none (the D-Bus values type-check in the `dbus` probe; no unit was started).
- Crates: `zbus` with `tokio`, `zbus_systemd` (56 packages), `rustix`.
- Disqualifying problems found: none.
  Problems:
   `FreezeUnit`, `ThawUnit`, `Freeze`, `Thaw`, `FreezerState`, and `CanFreeze` are marked
   `<!--method FreezeUnit is not documented!-->` and similar in `SD/man/org.freedesktop.systemd1.xml:580-582`,
   `:2181-2183`, `:2213`, `:2225` (verified),
   so freezer semantics come only from source;
   every task costs a D-Bus round trip plus a job (unmeasured);
   `FreezeUnit` fails while the scope's start job is pending;
   it needs a user systemd session, which CI runners may lack (unverified).

#### C1-C: systemd transient service spawned by systemd

- `StartTransientUnit("<tool>-task-<id>.service", ...)` with `Type=exec`,
   `ExecStartEx=[(path, argv, ["no-env-expand"])]`,
   `Environment=[...]` (the full task environment, because services inherit the manager's),
   `WorkingDirectory`,
   `StandardInputFileDescriptor`,
   `StandardOutputFileDescriptor`,
   and `StandardErrorFileDescriptor`
   passed as file descriptors from `std::io::pipe`,
   limits, `AddRef=true`, `BindsTo=` the daemon's unit.
- Exit: `JobRemoved` plus the service's `ExecMainStatus`, `ExecMainCode`, and `Result` properties before release.
- `unsafe` in repository code: none (signatures `a(sasas)` and `h` verified; nothing sent).
- Problems:
   placement is guaranteed by systemd but the task is not the daemon's child, so there is no pidfd or wait;
   forgetting `ExecStartEx` silently expands `$VAR` in arguments (verified in source and man page);
   unit garbage collection races the exit-status read unless `AddRef` is used;
   the undocumented freezer API and D-Bus round trips of C1-B apply;
   fd passing through `zbus` was not exercised.

#### C1-D: `systemd-run --user --scope --expand-environment=no` per task

- Spawn `systemd-run` as the child; it registers itself and `exec`s the task, so the task stays the daemon's child.
- `unsafe`: none.
- Problems:
   one extra process, D-Bus connection, and job per task (unmeasured);
   freeze and kill still need D-Bus or cgroupfs on the scope;
   error output from `systemd-run` mixes with task `stderr`.

#### C1-E: `processkit` groups

- One `ProcessGroup` per task; spawn through the group; `suspend`/`resume`; kill on drop or shutdown.
- `unsafe` in repository code: none (the crate owns its `pre_exec`).
- Disqualifying problems:
   per-task limits cannot work under the delegated scope this design uses (`src/limits.rs:33-52`, verified);
   the silent fallback to process groups must be checked with `mechanism()` on every group to fail closed;
   `suspend` does not wait for `frozen 1`;
   52 releases in a year means frequent upgrades.

#### C1-F: `libcgroups` v2 manager rooted at the delegated cgroup

- `Manager::new(<delegated root>, "tasks/<id>")`, `apply` limits via `oci_spec::runtime::LinuxResources`,
   placement still through the C1-A launcher (writing the PID itself) or `add_task(launcher_pid)` with a handshake,
   `freeze` and `remove` on `spawn_blocking`.
- `unsafe` in repository code: none.
- Problems:
   blocking sleeps up to 10 s in freeze;
   66 packages and OCI types for a few file writes;
   silent sandbox-off with `with_rootless(true)`;
   a warning per call for `dmem` on this host (inference);
   26.06% documented.
  It removes little repository code relative to C1-A, because placement and event waiting remain.

#### C1-G: `pre_exec` join (the prior design)

- `unsafe` in repository code: yes, the closure.
- Problems: forks the daemon for every task (no `posix_spawn`), allocation-free closure discipline.

#### C1-H: `clone3` with `CLONE_INTO_CGROUP`, or glibc `pidfd_spawn` with `posix_spawnattr_setcgroup_np`

- `unsafe` in repository code: yes (`clone3::Clone3::call` is `unsafe`;
   glibc functions need repository `extern "C"` declarations).
- Problems:
   `clone3` requires hand-written exec and descriptor setup in the child;
   the `clone3` crate is unmaintained since 2022;
   `libc` 0.2.189's constant still overflows unless the alpha or `linux-raw-sys` is used.

#### C1-I: `cgroups-rs`

- Excluded on requirements:
   writes systemd-owned ancestors, root hard-coded, systemd path on the system bus only.

### Ranking

C1-A > C1-B > C1-C > C1-D > C1-E > C1-F > C1-G > C1-H > C1-I.

- C1-A over C1-B:
   both are safe and place before `exec`,
   but C1-A keeps the hot path free of D-Bus round trips and the undocumented freezer API,
   and B's advantage of not needing prior delegation is available to A through one startup call.
- C1-B over C1-C:
   B keeps the task as the daemon's child with ordinary pipes and a pidfd,
   while C adds the `ExecStartEx` flag trap, fd passing, and exit status only through D-Bus.
- C1-C over C1-D:
   C needs one D-Bus call per task, while D adds a whole `systemd-run` process and connection per task.
- C1-D over C1-E:
   D enforces limits through unit properties, while E cannot enforce limits under a scope
   and can silently lose the cgroup.
- C1-E over C1-F:
   E is async and complete for freeze and kill, while F blocks threads, still needs a launcher, and carries 66 packages.
- C1-F over C1-G:
   F keeps repository code safe, while G needs an `unsafe` closure and forks the daemon.
- C1-G over C1-H:
   G reuses `std`'s exec and descriptor setup, while H hand-writes them around a stale crate or raw FFI.
- C1-H over C1-I:
   H at least places the child correctly, while I writes cgroups that systemd owns.

## C2 recursive watching of about 5,480 directories

### Candidates

- `notify` 8.2.0 (2025-08-03), and `9.0.0-rc.5` (2026-08-30):
  - Maintenance:
     5 releases in the past year, all 9.0 release candidates after 8.2.0;
     rc.1 was 2026-01-25, so 9.0 has been a candidate for about eight months;
     `notify-rs/notify` pushed 2026-09-14; 91 open issues.
  - License CC0-1.0.
  - Dependencies: 10 packages (8.2.0); 11 (9.0.0-rc.5).
  - Re-verified claims:
    - 8.2.0 always adds `WatchMask::OPEN` (`SRC/notify-8.2.0/src/inotify.rs:427`)
       and emits `Access(Open)` (`:349-351`); the probe saw an event for a read-only open (verified).
    - Neither version has path exclusion for recursive watches:
       9.0.0-rc.5's walk skips only "barriers", which are user watches on non-dereferenced symlinks
       (`SRC/notify-9.0.0-rc.5/src/inotify.rs:700-708`), verified.
    - Recursive follows symlinks by default in 9.0.0-rc.5 (`src/config.rs:241-246`, verified).
    - Nested creation loses the deep file in both versions (probe, verified; issue #727 still `OPEN`, `gh issue view`).
    - 8.2.0 reports only `MaxFilesWatch` errors when adding watches for new subdirectories; other errors are dropped
       (`src/inotify.rs:383-397`, verified).
  - Newly found:
    - 9.0.0-rc.5 builds the kernel mask from `EventKindMask` (`src/inotify.rs:42-86`);
       `EventKindMask::CORE` excludes access events
       (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/notify-types-2.1.0/src/event.rs:338-341`),
       and the probe saw no event for a read-only open (verified).
      The default is still `EventKindMask::ALL` (`src/config.rs:246`).
    - Both versions surface queue overflow as `EventKind::Other` with `Flag::Rescan`
       (`SRC/notify-8.2.0/src/inotify.rs:212-214`; `SRC/notify-9.0.0-rc.5/src/inotify.rs:339-342`, verified).
    - 8.2.0 `watch` does a channel round trip with `rx.recv().unwrap()` (`src/inotify.rs:554-560`, verified);
       5,481 non-recursive `watch` calls took 121 to 144 ms against 32 to 40 ms for direct `inotify` (verified).
  - Documentation problems (recorded, not culled):
     docs.rs 100% documented for both;
     the 8.2.0 page shows `notify = "8.1.0"` (verified);
     both pages give `sudo sysctl -p` after setting a value (verified; persistence semantics from recall);
     the example comment "All files and directories at that path and // below will be monitored" remains on 8.2.0
     (verified)
     while nested creations are missed.
- `notify-debouncer-mini` 0.7.0 (2025-08-03), incumbent:
   MIT OR Apache-2.0; 0 releases in the past year; 18 packages.
  `add_event` iterates `event.paths` only (`SRC/notify-debouncer-mini-0.7.0/src/lib.rs:299-312`, verified),
   and the overflow event has no paths,
   so the incumbent silently drops queue overflow.
  Documentation problem: docs.rs 100% documented, but neither "rescan" nor "Rescan" appears on the root page (verified).
- `notify-debouncer-full` 0.7.0 (2026-01-23), newest `0.8.0-rc.2`:
   MIT OR Apache-2.0; 3 releases; 12 packages; built on `notify 8.2.0`, so it inherits `OPEN` events.
  Forwards `Rescan` as a debounced event (`SRC/notify-debouncer-full-0.7.0/src/lib.rs:282-286`, verified);
   uses `NoCache` on Linux (`src/cache.rs:60-65`, verified).
  Documentation problem: docs.rs 73.91% documented; the root page does not mention rescan (verified).
- `rolldown-notify` 10.5.1 (2026-08-21):
   a fork published from `rolldown/notify` (GitHub fork of `notify-rs/notify`, 4 stars),
   16 releases in the past year, still masks `OPEN` (`SRC/rolldown-notify-10.5.1/src/inotify.rs:701`, verified).
  Documentation problem: its manifest `repository` and README still point at `notify-rs/notify` (verified).
- `inotify` 0.11.5 (2026-08-14):
  - Maintenance: 5 releases; `hannobraun/inotify-rs` pushed 2026-08-14; 10 open issues.
  - License ISC.
  - Dependencies: 17 packages with the default `stream` feature (tokio `net`, `futures-util`).
  - Public API is safe except `from_bits_unchecked` constructors
     (`SRC/inotify-0.11.5/src/events.rs:344`, `src/watches.rs:233`, verified).
  - docs.rs 100% documented; no problem found in the pages loaded.
- `rustix::fs::inotify` and `nix::sys::inotify`:
   safe lower-level alternatives (`SRC/rustix-1.1.5/src/fs/inotify.rs:60-88`, verified for rustix).
- `watchexec` 8.4.2 (2026-09-15):
  - Maintenance: 8 releases; `watchexec/watchexec` pushed 2026-09-15; 35 open issues; 7,187 stars.
  - License Apache-2.0.
  - Dependencies: 58 packages; depends on `notify 8.2.0`, so `OPEN` events arrive.
  - Behavior:
     since 8.4.0 (2026-08-24) it walks recursively itself with non-recursive watches and asks `Filterer::check_dir`
     before watching a directory (`SRC/watchexec-8.4.2/CHANGELOG.md:6-8`; `src/sources/fs.rs:1-17`;
     `src/filter.rs:16-28`, verified);
     `sources::fs::worker` is public (`src/sources/fs.rs:644`);
     overflow maps to a topology rescan (`:796-797`, verified).
  - Disqualifying problem:
     when a channel is full it drops events with only a `debug!` log
     (`src/sources/fs.rs:240-242`, `:901-906`, verified).
  - Documentation problems:
     docs.rs 100% documented;
     `event_channel_size` says adjusting "may help" and omits that a full channel drops events
     (`src/config.rs:170-176`, verified).
- `filesentry` 0.2.2 (2025-12-19), from the Helix editor organization:
  - Maintenance: 3 releases; `helix-editor/filesentry` last commit 2026-06-22; 1 open issue; 56 stars.
  - License MPL-2.0.
  - Dependencies: 26 packages, including `ignore`, `mio`, `papaya`, `rustix`.
  - Behavior:
     kernel mask without `OPEN` or `ACCESS` (`SRC/filesentry-0.2.2/src/inotify/sys.rs:47-56`, verified);
     `Filter::ignore_path` excludes paths (`src/config.rs:29-42`);
     crawls into an in-memory tree, compares `stat` results, and recrawls on overflow (README; `src/inotify.rs:94`,
     `:104`);
     reported the deep file of a nested creation and nothing under the ignored directory (probe, verified);
     reports files only, not directories (README, verified).
  - Disqualifying problem:
     watch failures are only `log::error!` (`src/worker.rs:65`, `:73`, `:140`, `:154`, verified),
     and "parent wasn't yet in the tree! Ignoring..." paths also only log (`src/tree.rs:315`, `:393`),
     so the daemon cannot fail closed on `ENOSPC` through the API.
  - Documentation problems:
     docs.rs 7.41% documented (verified);
     `Watcher::add_root`, `set_filter`, `start`, and `add_handler` have no rustdoc (`src/lib.rs:125-215`, verified);
     test hook `new_impl(_slow)` is public (`src/lib.rs:187`).
- Fanotify: `nix::sys::fanotify` 0.31.3, `fanotify-rs` 0.3.1 (2024-02-09), `naughtyfy` 0.2.1 (2023-11-29, archived,
   AGPL-3.0-only),
   `fanotify-fid` 0.7.1 (2026-09-16, 0 stars, 14 releases):
  - Unprivileged `FAN_MARK_FILESYSTEM` and `FAN_MARK_MOUNT` return `EPERM` (probe, verified),
     matching "Use of this flag requires the CAP_SYS_ADMIN capability" (`DOCS/man-fanotify-mark.txt`, verified).
  - Inode marks remain per directory, so fanotify offers no recursion advantage here.
  - `nix` has no `FAN_REPORT_FID`, `FAN_REPORT_DIR_FID`, or `FAN_REPORT_NAME` flags
     (`nix-0.31.3/src/sys/fanotify.rs:96-121`) and parses only `fanotify_event_metadata` (`:201-207`, `:357`), verified.
  - Documentation problem: `nix` links `man7/fanotify_init.2.html`, which returns HTTP 404 (verified).
- Watchman through `watchman_client` 0.9.0 (2024-06-18):
   MIT; 0 releases in the past year; 45 packages; docs.rs 66.08% documented.
  Needs a running Watchman server; `watchman` is not installed on this host (verified).
- `wtr-watcher` 0.14.5 (2026-02-23): not examined beyond metadata.
- `ignore` 0.4.33 (2026-08-04), incumbent:
   Unlicense OR MIT; 10 releases; 13 packages;
   the 5,481-directory walk took 102 to 128 ms (verified).

### Designs

#### C2-A: `inotify` directly, per-directory watches from an `ignore` walk

- As the prior design, with the probe-verified mask.
  Add filesentry's model to the repository code:
   an in-memory tree of `stat` results, rescan and diff on `Q_OVERFLOW` and on each new directory.
- `unsafe`: none.
- Measured setup: 32 to 40 ms for 5,481 watches.
- Problems: new-directory scans, overflow rescans, descriptor-to-path sets, and failure policy are repository code.

#### C2-B: `filesentry` with a gitignore-style `Filter`

- `unsafe`: none.
- Measured crawl: 377 to 419 ms.
- Disqualifying problem: watch failures are not observable through the API.
- A local patch that returns errors would remove it (unverified effort; not estimated).

#### C2-C: `notify` 9.0.0-rc.5 with `EventKindMask::CORE`, non-recursive per directory

- `unsafe`: none.
- Problems:
   depends on a release candidate;
   new-directory scans remain repository code because recursive mode misses nested files (verified);
   non-`MaxFilesWatch` failures during notify's own recursion are not a concern here because recursion is not used.

#### C2-D: `notify` 8.2.0 non-recursive per directory

- `unsafe`: none.
- Disqualifying problem:
   every build that opens sources produces `OPEN` events in the kernel queue of 16,384 (prior probe),
   so large builds can overflow and force full rescans (inference from the verified `OPEN` events).

#### C2-E: `watchexec::sources::fs::worker` with a `check_dir` filterer

- `unsafe`: none.
- Disqualifying problems: silent event drops on a full channel, plus `OPEN` events from `notify 8.2.0`.

#### C2-F: Watchman

- Disqualifying problem: an external daemon that is not installed and is not repository-provisioned.

#### C2-G: fanotify

- Disqualifying problem: no unprivileged recursion (verified).

### Ranking

C2-A > C2-B > C2-C > C2-D > C2-E > C2-F > C2-G.

- C2-A over C2-B:
   the daemon must fail closed when a watch cannot be added, and only A exposes every `add` result.
- C2-B over C2-C:
   B already handles nested creation and overflow recrawls on a stable release (verified),
   while C misses nested files (verified) and is a release candidate.
- C2-C over C2-D:
   C removes `OPEN` events at the kernel mask (verified), while D floods the queue during builds.
- C2-D over C2-E:
   D reports overflow, while E can drop events silently and still receives D's `OPEN` events.
- C2-E over C2-F:
   E is an in-process library, while F needs an external daemon the host lacks.
- C2-F over C2-G:
   F does recursive watching, while G cannot without `CAP_SYS_ADMIN`.

## C3 JSON-RPC 2.0 over a Unix socket with notifications and subscriptions

### Candidates

- `tokio-util` 0.7.19 (2026-07-21) with `serde_json` 1.0.151 (2026-07-20):
  - `tokio-util`: MIT; 3 releases; 6 packages with `codec`; docs.rs 100% documented.
  - `serde_json`: MIT OR Apache-2.0; 6 releases; 5 packages; incumbent.
  - `LinesCodec` behavior (probe on 0.7.18):
     an overlong line yields `Err(MaxLineLengthExceeded)`, then `None` once, then later lines decode.
    A `while let Some(line) = framed.next().await` loop treats that `None` as end of stream,
     so the connection must be closed on the error.
    `FramedRead` returns `None` after a decoder error by design
    (`SRC/tokio-util-0.7.19/src/codec/framed_impl.rs:161-168`, verified).
  - Documentation problems:
     "Subsequent calls will discard up to `limit` bytes" names `limit` for `max_length`
     and leaves the rest of the line unclear
     (`src/codec/lines_codec.rs:50-59`; same text on docs.rs, verified);
     the page does not say the stream yields `None` after the error.
- `jsonrpsee` 0.26.0 (2025-08-11); newest publish `0.24.11` (2026-05-27, backport):
  - Maintenance:
     2 releases in the past year, no feature release since 2025-08-11;
     `master` has unreleased work such as "feat: add support for connection_timeout (#1643)" (2026-07-31);
     93 open issues.
  - License MIT.
  - Dependencies: 50 packages with `server-core`; 89 with `server`.
  - Re-verified: server transports are HTTP and WebSocket; `jsonrpsee-server` says so in its crate doc
     (`SRC/jsonrpsee-server-0.26.0/src/lib.rs:28-30`), and `RpcService::new` is `pub(crate)`
     (`src/middleware/rpc.rs:67`), verified.
  - Newly found:
    - `Methods::raw_json_request(&str, buf_size)` returns the response and a notification receiver for subscriptions
       (`SRC/jsonrpsee-core-0.26.0/src/server/rpc_module.rs:351-361`, verified),
       and the probe served methods and subscriptions over NDJSON Unix streams with it (verified).
    - Every call uses `ConnectionId(0)` (`:376`) and a one-permit subscription limit (`:1054-1056`), verified;
       one connection unsubscribed another connection's subscription in the probe (verified).
    - Batch arrays fail to parse (probe, verified), so batch handling is repository code.
    - A per-connection dispatcher is possible with public items:
       `Methods::method_with_name` (`:276`), `MethodCallback` (`:146-156`), `MethodSink::new`
       (`src/server/helpers.rs:46`),
       `SubscriptionState` public fields (`src/server/subscription.rs:491-498`), `BoundedSubscriptions::new` (`:472`),
       verified;
       that re-implements the server's dispatch loop.
  - Documentation problems (prior findings stand, recorded not culled):
     `jsonrpsee-server` `serve` "over a TCP connection" for a generic stream;
     the incomplete `ServerConfigBuilder` sentence;
     `jsonrpsee::server::serve` page 404.
    docs.rs 100% documented; `Methods` page does not mention batch (verified).
- `jsonrpc-core` 18.0.0 and `jsonrpc-ipc-server` 18.0.0 (2021-07-20):
   MIT; 0 releases; repository README: "This crate is no longer actively developed" (verified via `gh api .../readme`);
   55 packages for the IPC server; docs.rs 94.81% documented, and the docs.rs page does not repeat the deprecation
   (verified).
  Excluded: unmaintained.
- `lsp-server` 0.10.0 (2026-07-16):
   MIT OR Apache-2.0; 3 releases; rust-analyzer repository; 15 packages; docs.rs 35% documented.
  `Content-Length` framing (`SRC/lsp-server-0.10.0/src/msg.rs:284-299`), blocking `Message::read`/`write` (`:173`,
  `:191`),
   transports stdio, TCP, and memory (`src/lib.rs:40-69`), no batch variant (`src/msg.rs:13`), verified.
  Using it means changing the design's framing decision from newline-delimited JSON.
- `tower-lsp` 0.20.0 (2023-08-11, 0 releases, last push 2024-08-15) and `tower-lsp-server` 0.23.0 (2025-12-07;
   `0.24.0-rc.1` 2026-09-11):
   `tower-lsp-server` is 51 packages, docs.rs 100% documented, `Content-Length` framing (`src/codec.rs`),
   and an LSP initialize lifecycle (`src/service/state.rs:10-12`), verified.
  Excluded: LSP-specific protocol state.
- `yerpc` 0.7.0 (2026-09-03), Delta Chat and chatmail:
  - Apache-2.0/MIT; 1 release; `chatmail/yerpc` pushed 2026-09-03; 10 open issues; 49 packages.
  - Transport-agnostic `RpcSession::handle_incoming(&str)` (`SRC/yerpc-0.7.0/src/requests.rs:112-117`),
     `RpcClient::send_notification` (`:162-178`), outbound channel `bounded(10)` (`:135-140`), verified.
  - No batch: `Message` is `Request` or `Response` (`src/lib.rs:71-74`, verified); no subscription primitive.
  - Generates TypeScript types through `typescript-type-def` (`src/lib.rs:14`, `:20`, verified),
     which addresses the Rust and TypeScript schema drift for file-enforcer events (unverified output).
  - Documentation problems:
     docs.rs 30.77% documented;
     README example imports no `Extension`, lacks a semicolon after `let api = Api {}`,
     and calls `axum::Server::bind` (`README.md:12-14`, `:33`, `:39`)
     while the crate depends on `axum 0.8.1`, which exports `serve` and no `Server` (`SRC/axum-0.8.1/src/lib.rs:447`,
     `:476`), verified.
- `jsonrpc-v2` 0.13.0 (2023-09-02): MIT; HTTP integrations for actix-web and hyper only (`src/lib.rs:4-7`,
   verified). Excluded.
- `ajj` 0.7.1 (2026-04-17): MIT OR Apache-2.0; 8 releases; 55 packages with only `ipc`;
   IPC pub/sub over `interprocess` with streaming JSON framing (`src/pubsub/ipc_inner.rs:132-150`);
   `opentelemetry`, `tracing-opentelemetry`, and `metrics` are non-optional dependencies (`Cargo.toml`, verified).
- `karyon_jsonrpc` 1.0.1 (2026-08-23): MIT; 11 releases; Unix sockets and pub/sub; 85 packages with `unix`
   and `tokio`. Not examined further.
- `jsonrpc-fdpass` 0.1.1 (2026-04-17), `bootc-dev`:
   MIT OR Apache-2.0; 0 stars; self-delimiting JSON framing plus fd passing (README section 2.2);
   depends on `jsonrpsee 0.24` and `tracing-subscriber` (`Cargo.toml`, verified); 97 packages.
- `ndjson-rpc` 0.0.2, `jasonrpc`, `json-rpc-types` 1.3.4 (2023),
   `jsonrpcmsg` 0.1.2: message types or early-stage crates, not examined.

### Designs

#### C3-A: `LinesCodec` plus `serde_json` and repository types

- Per connection: `FramedRead::new(read, LinesCodec::new_with_max_length(N))`; close on any error;
   decode into a request enum; handle arrays as batches.
- Subscriptions: the sequence-numbered ring buffer and snapshot-on-gap design from the prior research.
- `unsafe`: none.
- Problems: dispatch, batch, and subscription code are repository-owned
   (the subscription part is app logic in every option).

#### C3-B: `jsonrpsee` `RpcModule` behind a repository NDJSON loop

- As the probe, or with a per-connection dispatcher over the public `MethodCallback` API.
- `unsafe`: none.
- Problems:
   shared connection ID on the `raw_json_request` path (verified);
   batch still repository code (verified);
   the resume-from-sequence model is not `jsonrpsee`'s subscription model and remains repository code;
   44 more packages than C3-A for method dispatch;
   13 months without a feature release.

#### C3-C: `yerpc` session behind a repository NDJSON loop

- `unsafe`: none.
- Problems: no batch, no subscriptions, 30.77% documented; gains TypeScript type generation.

#### C3-D: `lsp-server` messages with LSP framing

- `unsafe`: none.
- Problems: blocking I/O on threads, no batch, and a framing change for every client.

### Ranking

C3-A > C3-B > C3-C > C3-D > `tower-lsp-server` > `ajj` > `karyon_jsonrpc` >
`jsonrpc-fdpass` > `jsonrpc-core` > `jsonrpc-v2`.

- C3-A over C3-B:
   batch and resumable subscriptions are repository code in both, and B adds a shared connection ID
   and 44 packages to save only method dispatch.
- C3-B over C3-C:
   B has typed subscriptions and complete docs, while C has neither subscriptions nor batch.
- C3-C over C3-D:
   C keeps newline-delimited framing, while D changes framing and blocks threads.
- C3-D over `tower-lsp-server`:
   D is generic JSON-RPC messages, while `tower-lsp-server` imposes the LSP lifecycle.
- `tower-lsp-server` over `ajj`:
   `tower-lsp-server` is fully documented, while `ajj` forces telemetry dependencies and unbounded streaming framing
   (unverified bound).
- `ajj` over `karyon_jsonrpc`: 55 packages against 85 for similar pub/sub.
- `karyon_jsonrpc` over `jsonrpc-fdpass`: fdpass depends on an older `jsonrpsee` line plus `tracing-subscriber`
   and has no adoption signal.
- `jsonrpc-fdpass` over `jsonrpc-core`: `jsonrpc-core` is unmaintained by its authors.
- `jsonrpc-core` over `jsonrpc-v2`: `jsonrpc-v2` has no non-HTTP transport.

## C4 process supervision and Ctrl+C

### Candidates

- `tokio` 1.53.1 (2026-07-20), incumbent with other features:
  - MIT; 21 releases; 13 packages with `rt-multi-thread`, `process`, `signal`, `net`, `io-util`, `sync`, `time`,
     `macros`.
  - `tokio::process` on Linux first builds a `PidfdReaper` and falls back to `SIGCHLD` only if that fails
     (`SRC/tokio-1.53.1/src/process/unix/mod.rs:118-137`, verified);
     `Command::process_group` exists (`src/process/mod.rs:790`).
  - Documentation problems (recorded, not culled):
     the module doc says support "is provided through signal handling on Unix",
     and the docs.rs page never mentions pidfd (verified);
     `ctrl_c` documents "Even if this `Signal` instance is dropped"
     and "translated to a stream event" for a function returning a future
     (`src/signal/ctrl_c.rs:26-31`; docs.rs, verified);
     `UnixListener::bind` treats a leading NUL as an abstract socket without documenting it
     (`src/net/unix/listener.rs:65-89`, verified).
- `signal-hook` 0.4.4 (2026-04-04) and `signal-hook-tokio` 0.4.0 (2026-01-01):
   MIT OR Apache-2.0; 5 and 1 releases; 4 and 9 packages;
   safe `flag::register_conditional_shutdown` and `register_conditional_default`, documented for "double CTRL+C"
   (`SRC/signal-hook-0.4.4/src/flag.rs:166-230`, verified);
   docs.rs 98.92% documented.
- `async-signal` 0.2.14 (2026-04-07, smol): not needed with tokio.
- `nix::sys::wait` and `rustix::process::waitid`: redundant with the tokio pidfd reaper.
- `watchexec-supervisor` 5.4.0 and `process-wrap` 10.0.0: supervision around process groups; cgroups replace that role.

### Design

- Spawn through C1-A's launcher with `tokio::process::Command`; hold every `Child` until it is reaped.
- Ctrl+C:
   `tokio::signal::unix::signal(SignalKind::interrupt())` in the runtime for the graceful path
   (stop scheduling, freeze, `SIGTERM` members, thaw, grace period, `cgroup.kill`).
  For a second Ctrl+C that must work even if the runtime is stuck,
   a `std::thread` with `signal_hook::iterator::Signals` writes `1` to the delegated root's `cgroup.kill`,
   which kills the daemon and every task at once
   (inference from the kernel `cgroup.kill` semantics; unexercised).
  `register_conditional_shutdown` alone would exit the daemon but leave tasks running,
   because tasks are in their own process groups.
- `unsafe`: none.

## C5 btrfs

### Candidates

- `rustix::fs::ioctl_ficlone` (1.1.5): safe (`SRC/rustix-1.1.5/src/fs/ioctl.rs:56`);
   probe cloned into an exclusive temporary file and renamed over the destination; cross-filesystem returns `EXDEV`
   (verified).
- `reflink-copy` 0.1.30 (2026-06-18):
  - MIT/Apache-2.0; 2 releases; `cargo-bins/reflink-copy` pushed 2026-09-15; 5 open issues; 6 packages.
  - Linux `reflink` creates the destination with `create_new`, clones, and deletes it on failure
     (`SRC/reflink-copy-0.1.30/src/sys/unix/linux.rs:7-16`; `src/sys/utility.rs:19-27`, `:53-67`, verified);
     probe: `AlreadyExists` for an existing destination, success for a fresh path, no temporary file left (verified).
  - Documentation problem: "NOTE that it generates a temporary file and is not atomic" (`src/lib.rs:43`, `:59`;
     docs.rs, verified) does not match Linux.
- `btrfs-uapi` 0.13.0 (2026-05-14), `rustutils/btrfsutils`:
  - MIT OR Apache-2.0; 13 releases in the past year; pushed 2026-05-14; 20 open issues; 14 stars.
  - Dependencies: 20 packages normal, 45 with build dependencies (`bindgen`).
  - "All `unsafe` code is confined to the [`raw`] module" with a safe public API (`src/lib.rs:10-19`, verified);
     `subvolume_create`, `snapshot_create`, `subvolume_delete`, `subvolume_info` (`src/subvolume.rs:209-327`).
  - Builds on this host with `bindgen` and `libclang.so.22.1` (verified); CI availability of `libclang` unverified.
  - Documentation problems:
     `subvolume_create` says "Requires `CAP_SYS_ADMIN`" (`src/subvolume.rs:200-204`),
     while the kernel says "Subvolume creation is not restricted, but snapshots are limited to own subvolumes only"
     (`KB:1203-1207`);
     `subvolume_delete` says the same (`src/subvolume.rs:232`),
     while unprivileged deletion is allowed with `user_subvol_rm_allowed` (`KB:2383-2398`), verified.
    docs.rs 100% documented.
- `libbtrfsutil` 0.8.0 (2026-07-04):
   MIT; 5 stars; needs `libbtrfsutil` headers, absent on this host (verified); docs.rs 66.67% documented.
- `btrfsutil` 0.2.0 (2023-10-05): repository archived (`gh api`, verified).
- `btrfs` 1.2.2 (2017), `libbtrfs` 0.0.40 (0 stars), `kache-fs` 0.23.0, `clone-file` 0.1.0 (2023), `reflink` 0.1.3
   (2019): not examined beyond metadata.
- `btrfs` CLI from `btrfs-progs` through `tokio::process`: available at `/usr/bin/btrfs` (prior research).

### Design

- Restore: `rustix::fs::ioctl_ficlone` into an exclusive temporary file next to the destination, then rename;
   copy on `EXDEV`, `EINVAL`, or `EOPNOTSUPP`.
- Optional subvolumes and snapshots: `btrfs-uapi`, with `BTRFS_SUBVOL_RDONLY` set explicitly.
- `unsafe`: none (the prior hand-written `#[repr(C)]` ioctls are unnecessary).

## Corrections to prior research

1.  **"Task spawning into cgroups is repository-written `unsafe` code" is wrong.**
    A re-executed launcher that writes its own PID with `std::fs::write` and then calls the safe `CommandExt::exec`
    places the task before its code runs, fails closed, and compiled under `#![forbid(unsafe_code)]` (launcher probe,
    verified).
    systemd scopes and services through `zbus_systemd` are also `unsafe`-free (C1-B, C1-C).
2.  **The `pre_exec` design forced `fork` of the daemon; the launcher keeps `posix_spawn`
    and costs 0.8 to 0.9 ms median per spawn** (verified).
3.  **The systemd D-Bus route and the `zbus` incumbent were not surveyed.**
    The host's user manager exposes `StartTransientUnit`, `FreezeUnit`, `ThawUnit`, `KillUnit`,
    `AttachProcessesToUnit` (busctl, verified);
    `KillUnit` with `SIGKILL` uses `cgroup.kill`, and `FreezeUnit` replies only after freezing (systemd source,
    verified).
4.  **The `cgroups-rs` cull stands but for requirement reasons**, with two more findings:
    a hard-coded `/sys/fs/cgroup` root and a system-bus-only systemd manager (verified).
5.  **`libcgroups`, `processkit`, and `command-fds` were not considered.**
    Each keeps repository code free of `unsafe`,
    with the trade-offs recorded under C1 (blocking APIs, limits that fail under a scope, or a hidden `pre_exec`).
6.  **The `libc` constant finding is version-specific.**
    `libc 1.0.0-alpha.4` and `linux-raw-sys 0.12.1` declare `CLONE_INTO_CGROUP` as a 64-bit value (verified);
    only `0.2.189` overflows.
7.  **The watcher survey missed the incumbent `notify-debouncer-mini`**, which drops queue-overflow events (verified),
    and missed `filesentry` and `watchexec` 8.4's filtered non-recursive recursion.
8.  **`notify` 9.0.0-rc.5 removes the `OPEN` flood with `EventKindMask::CORE`** (verified);
    the prior research noted that `EventKindMask` exists only in release candidates
    but did not test whether it changes the kernel mask, which it does.
    The nested-directory bug and missing recursive exclusion are confirmed in rc.5 (verified).
9.  **"`jsonrpsee` cannot speak newline-delimited JSON-RPC" is incomplete.**
    `RpcModule::raw_json_request` serves methods and subscriptions over any framing (verified);
    its real problems are the shared connection ID and missing batch support (verified).
10. **Documentation-only culls are reversed under the user's decision:**
    `tokio::process`, `tokio::signal::ctrl_c`, `tokio_util::codec::LinesCodec`, `reflink-copy`,
    and `jsonrpsee` return as candidates;
    their documentation problems are recorded above.
    `LinesCodec`'s behavior after an overlong line was measured (`Err`, `None`, resume).
11. **"btrfs subvolume operations need hand-written `#[repr(C)]` ioctls" is wrong.**
    `btrfs-uapi` 0.13.0 provides safe wrappers and builds on this host (verified).
12. **Unprivileged fanotify was not evaluated**; it cannot mark a filesystem or mount without `CAP_SYS_ADMIN`
    (verified).
13. **"Repository-written `unsafe`" is not new to the repository**:
    25 Rust files already contain `unsafe`, for example `package/wg/cli-quicker-exempt/src/keeper_process.rs:120`
    (verified).
14. **The Rust and TypeScript schema yike has a generator candidate**:
    `yerpc` generates TypeScript types from Rust through `typescript-type-def` (verified in source; output unverified).
15. The design doc's summary, "`notify`, `jsonrpsee`, and `cgroups-rs` fail the documentation rule or the requirements,
    so the watcher and JSON-RPC framing are hand-written",
    should now read that framing and watch correctness remain repository code by choice after comparison,
    not by necessity.

## Revised Rust design

- C1 placement, freeze, kill:
   C1-A, cgroupfs under a delegated cgroup with a re-executed launcher;
   `std` and `tokio` (incumbent) for spawn and files;
   `inotify` for `cgroup.events`;
   `zbus` plus `zbus_systemd` only if the daemon self-delegates at startup.
  Repository `unsafe`: none.
- C2 watching:
   C2-A, `inotify` directly with per-directory watches from an `ignore` (incumbent) walk,
   plus a filesentry-style `stat` tree for rescans.
  `filesentry` becomes the choice if a change that surfaces watch errors lands upstream.
  Repository `unsafe`: none.
- C3 JSON-RPC:
   C3-A, `tokio-util` `LinesCodec` with a line cap and close-on-error, `serde_json` (incumbent) message types,
   repository batch handling and sequence-numbered subscriptions.
  Repository `unsafe`: none.
- C4 supervision and Ctrl+C:
   `tokio::process` (pidfd reaper) and `tokio::signal::unix`,
   plus a `signal-hook` thread for the second Ctrl+C that writes `cgroup.kill`.
  Repository `unsafe`: none.
- C5 btrfs:
   `rustix::fs::ioctl_ficlone` for restores, `btrfs-uapi` for optional subvolumes and snapshots.
  Repository `unsafe`: none.
- Every design above compiles its repository-side probe code under `#![forbid(unsafe_code)]`,
   so the revised Rust core can adopt that attribute (verified for the probed paths).

## Remaining yikes

Ranked by severity, most severe first.

1.  **Cgroup migration into a new task cgroup, freezing, and killing are unexercised on this host.**
    The launcher probe proved ordering and fail-closed behavior only against the probe's own cgroup,
    and systemd units were never started.
2.  **Delegation is still a prerequisite.**
    Self-delegation through `StartTransientUnit` with `Delegate=yes` is unexercised,
    and whether CI runners provide a user systemd session is unverified.
3.  **Watch correctness remains repository code.**
    No surveyed crate combines directory exclusion, observable watch failures, no `OPEN` flood, and a stable release.
4.  **Subscriptions, resume, backpressure, and batch remain repository code** in every JSON-RPC option.
5.  **The Rust and TypeScript event schema can drift**; `yerpc`'s TypeScript generation is a candidate but unevaluated.
6.  **The launcher re-executes the daemon binary.**
    Running `/proc/self/exe` after a rebuild replaces the file on disk is expected to run the original image
    (inference, unverified).
7.  **systemd's freezer D-Bus API is undocumented**, which matters only if C1-B or C1-C is chosen.
8.  **Each file-enforcer evaluation still pays about 147 ms of Node startup** (prior measurement, unchanged).
9.  **`btrfs-uapi` needs `libclang` at build time**, present on this host and unverified on CI.
10. **Documentation problems remain in chosen dependencies**:
    `tokio::process` and `ctrl_c` pages, `LinesCodec`, `reflink-copy` if used, `btrfs-uapi` permission notes,
    and systemd's freezer pages.
