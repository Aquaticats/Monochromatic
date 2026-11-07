# Git-free jobs of the Git wrapper and the published crates that could do them

## Research status

- Status:
  complete for the questions it answers;
  not a vet report (see section "Which gates ran and which did not")
- Written:
  2026-11-07T03:30:00-05:00
- Wrapper measured:
  `package/git-policy/cli`, crate `monochromatic-git-policy-cli` 0.5.34,
  source at `05a23ad09f0bf318d59b2fb06c011f7f24fd84a9`
- Scope:
  every Git-free job named in `doc/planning/cli-git-source-layout-target-tree.md`,
  sections "The crate split" and "Jobs found buried in the program"
- Product changes:
  none;
  every build, edit and test ran in a throwaway worktree or under `${HOME}/temp/agent/published-crates/`

## Question

The owner asked on 2026-11-07:
"Why are we not using already established crates for some functions?"
For each Git-free job of the wrapper, does an established published crate already do it well enough to replace our code?
A job that a vetted crate covers needs no crate of ours, and that changes the split into crates.

## The owner's words that bind this research

- "commits vs native git should not take 5s longer."
- "Disk pressure being high is not an excuse." "load is not an excuse."
- "(debuginfo level, incremental off for dependencies - no messing with these."
  and "other profile settings (split-debuginfo, strip, codegen-units) should be off-limits too"
- "Ignore Win and mac until I say so."
- "never discard user data"; "nothing should ever wait forever or indefinitely".
- New Rust dependencies go through `rust-catalog.toml`
  (`doc/decision/rust-dependency-catalog.md`, `doc/runbook/rust-dependencies.md`, minimum release age 1440 minutes).
- Ruling of 2026-11-07, which changes the brief:
  "\" text identical to JavaScript output\" - that's not needed."
  So byte identity with JavaScript or with the old TypeScript tools is not a requirement for any job below,
  above all for time stamps, JSON records, log lines and shell quoting.
  A candidate there is judged by correctness and by standard formats (RFC 3339 for time).
  Where this document says a test pins JavaScript-identical text only for its own sake, that test may change.

## The owner's answers of 2026-11-07, and how they change this document

The owner answered the open choices of the first version, verbatim:

- Replace with published crates:
  "Time stamps: jiff, Digests: sha1 + sha2, Linux calls: rustix, Memory headroom: sysinfo" (all four).
- How many of our library crates remain:
  "I'm not ready to answer this specific question yet."
- D-Bus:
  "zbus 6, and I suppose the lack of a blocking api can be worked around."
  The words "I suppose" make it an assumption;
  the section "Tokio as the runtime" below tests it.
- Shell quoting:
  "Keep ours (readable)."
  So `shell_quote.rs` stays, and `shell-quote` of open choice 10 is not adopted.

A second ruling the same day changed the D-Bus check:
"Adopt tokio, do not adopt ureq, use the `hyper` set of https libs."
The wrapper gets `tokio` as its async runtime, the standards-review call moves from `ureq` to `hyper` 1.x with
`hyper-util`, `hyper-rustls` and `http-body-util`, and the search for a minimal executor and a `block_on` setup was
dropped (what was learned before the second ruling is kept in the section below).
Where an earlier line of this document recommends against `zbus` or for `ureq`, the owner's ruling governs;
the earlier text is kept as the record of what was measured.

## Answer in short

One line per job.
"Replace" means the crate does the job and our code goes;
"partly" means a crate does a measured piece and the rest stays;
"keep" means no crate meets a behavior the wrapper's tests pin,
with the evidence;
"no candidate" means the registry queries found nothing to test.

- Digests (`sha-1-and-sha-256-digests`):
  replace with `sha1` and `sha2`.
  Already linked at 0.10.x, 4.2 times (SHA-1) and 7.0 times (SHA-256) faster on an index-sized input,
  and the wrapper's index tests pass with them, including a SHA-256 repository that real Git accepts.
- Time stamps (`iso_time`, and the second calendar in `discarded_prune_stamps`):
  replace with `jiff`, already linked.
  Equal to our text on 2933 of 2933 instants.
- JavaScript text rules (`js_text`) and JSON quoting (`json_record`):
  `serde_json` already does the quoting;
  `js_text` could become `std` after its readers are compared (the ruling frees output text, not accepted input).
- Linux file and disk calls (`linux-file-and-disk-calls`):
  partly.
  `rustix`, already linked, does every call except the btrfs ioctls,
  and its behavior on the probe equals what the wrapper needs.
  The btrfs ioctls stay ours for now:
  the one crate for them (`btrfs-uapi`) needs `bindgen` and `libclang` at build time,
  which is not recommended before the build environments are checked (not done).
- Safe file saving (`safe-file-saving`):
  partly.
  `tempfile`, already linked, takes the staging name and the no-replace publish;
  synchronizing, private modes, no-follow reads and the scratch-location rule stay.
- Starting and watching programs (`starting-and-watching-programs`):
  keep the no-progress watcher, the signal relay, the output capture and the start retry
  (no crate found for the first; `signal-hook` never restores signal dispositions);
  `procfs` and a socket-diagnostics crate are candidates for the `/proc` and netlink parsers,
  not yet validated against the wrapper's tests.
- Parallel work within a budget (`parallel-work-within-a-budget`):
  partly.
  `sysinfo` reproduced our memory headroom figure under two cgroup caps and could replace the host reader;
  the ordered runner and the start-or-run-here fallback stay,
  because the global `rayon` pool panics where our code runs the job in turn
  (other `rayon` setups were not evaluated).
- Text, time and log line formats (`text-time-and-log-line-formats`):
  replace the time part with `jiff`, the JSON quoting is already `serde_json`,
  and `shell-quote` 0.8.0 with its Bash quoter is a candidate for `shell_quote`:
  it printed control-free text that bash read back byte for byte on every input but NUL, which bash cannot hold
  (the other three crates tried did not);
  the cost is a less readable spelling.
  Keep the small `diagnostic_line`.
  Little is left to be a crate of ours.
- Talking to the systemd user manager (`talking-to-the-systemd-user-manager`):
  the owner chose `zbus` 6 on `tokio` (2026-11-07); the prototype in section "Tokio as the runtime" works on a
  toolchain older than the one the repository pins and does not compile on the pinned one.
  The first-version finding follows, kept as the record:
  keep the connection.
  `zbus` 5.19.0 adds 52 packages and 2.47 MB to the wrapper,
  and its blocking client has no deadline for connect and handshake;
  zbus 6 has no blocking client at all.
  Its wire-format-only build (6 packages) could replace only the 316-line encoder and reader.
- Fix packages after copying a folder (`fix-packages-after-copying-a-folder`):
  no candidate found.
- Dependent version bump check (`dependent-version-bump-check`):
  candidates found (release tools), library suitability not evaluated.
- Disposable test folders (`disposable-test-folders`):
  partly.
  `tempfile` makes the folders;
  the watchdog, process-group ownership and core-dump helpers have no crate counterpart.
- Journal writer:
  keep (blocking socket in both crates read; a full-queue probe was not run).
  The two journal crates send entries with a blocking `send_to`,
  and the wrapper must never wait for the journal.
- Three-way text merge (`word_merge`):
  keep.
  Ours equals Jujutsu's stored results on 41 of 41 corpus inputs;
  `diffy` and `flickzeug` on 31 of 41;
  `reconcile-text` never reports a conflict.
- Crash points for tests:
  keep, with a limit:
  `fail` has no kill action and no marker file, and its pause is released only in process;
  its callback hook was not evaluated for partial reuse.
- Btrfs subvolume conversion, Markdown autofix runner, saved-copies store:
  no candidate found.
- Durable work items and audit log rotation:
  unresolved; candidates were named and not tested against the wrapper's contract.
- Unified-diff reader and reverse-apply check:
  keep (not differentially tested against Git; reason in its section).
- Inotify wait:
  replace the read loop with `rustix`'s `inotify` module when `rustix` is adopted for the calls above.

## Which gates ran and which did not

This document is a research note written to the brief.
It is not a completed vet report under `.claude/skills/choosing-technology/SKILL.md`,
and no `doc/audit/` file exists for it.

Gates that ran:

- Category fit, licence and dependency surface from the crates.io API for every named candidate
  (facts collected 2026-11-07, `crate-facts.ts`, saved in the scratch folder).
- Repository precedent:
  the recorded build-versus-buy passages and the sibling packages (section "What the repository already records").
- Production source of the deciding behavior for the candidates that decide a verdict:
  `tempfile`, `atomic-write-file`, `rustix`, `signal-hook-registry`, `rayon-core`, `fail`, `systemd-journal-logger`,
  `tracing-journald`, `zbus`, `cap-primitives`, `sysinfo`, `procfs`, `btrfs-uapi`, `kill_tree`,
  with the crate version and the path read.
- Behavior probes on throwaway code for the pinned behaviors:
  hashing, shell quoting, three-way merge, time text, `rustix` calls, memory headroom, `procfs` parsing, `zbus`.
  The journal crates, `fail`, `kill_tree`, `cap-primitives`, `atomic-write-file`
  and the sock-diag crate were read, not run.
- Maintenance sample for the finalists:
  last commit date from the repository host,
  not the full sample of issues and pull requests the skill asks for.

Gates that did not run:

- The check-in before broad discovery.
  The skill asks the owner through a question tool who runs the enumeration;
  that tool is not available to this agent.
  What was run is the narrower form:
  repository precedent, the candidates the brief named, and one frozen round of crates.io queries (section "Queries").
  The owner decides whether a full vet of any adopted crate runs (open choice 9).
- Saturation of the discovery pages, the execution manifest, upstream CI and fuzz runs of any candidate,
  the full maintenance audit, and the weighted scoring with its sensitivity runs.
  Rankings in this document are plain reasoned orderings,
  and none of them is a scored recommendation.
- The package's own `mise` tasks.
  The wrapper tests that ran used `cargo test --lib` with name filters in the throwaway worktree.
- `native:perf`.
  The commit latency harness was not run;
  the startup and size measurements below stand in for it,
  and none of the candidates sits on a path that costs seconds.

## How it was measured

- Host:
  28 hardware threads, about 61 GiB of memory, shared with other sessions,
  scratch folder on btrfs.
  The host was loaded by other sessions during every run.
- Every command ran in its own `systemd-run --user` unit or scope with `CPUQuota`, `MemoryMax`, `TasksMax` and
  `IOWeight=10`.
  No synthetic load was made.
- Worktree:
  `.claude/worktrees/published-crates-check` at `05a23ad09f`, removed at the end.
  The release profile of the wrapper was not touched.
- Baseline binary:
  `cargo build --release --locked --offline --bins`, 16,735,752 bytes, sha256 starting `4c550e63ed21`.
  A second build from the same source gave the same sha256.
- Lockfile deltas:
  `cargo add <crate>` in the worktree, then a comparison of the registry packages of `Cargo.lock`
  (281 before).
  A package for a platform the wrapper does not build (Windows, wasm) still counts in a lockfile;
  the owner said to ignore Windows and macOS,
  and the counts below say when that matters.
- Startup:
  `monochromatic-git-policy-cli rev-parse HEAD` in a scratch repository,
  300 runs per binary in rotating order,
  with a byte-identical copy of the baseline as the same-binary control for the spread.
  Cold runs dropped the binary's cached pages first (`dd iflag=nocache`).
- Probes:
  small crates under `probes/` of the scratch folder built with the same release settings,
  each with a differential or behavior check.

## Measured baseline

- The wrapper's normal dependency tree prints 295 unique lines (`cargo tree --edges normal --prefix none`, sorted).
  Already linked into the binary and named in this document:
  `jiff` 0.2.24, `rayon` 1.12.0, `rustix` 1.1.4 (features `std`, `alloc`, `default`, `fs`), `tempfile` 3.27.0,
  `fastrand`, `getrandom`, `ring`, `sha1` 0.10.6, `sha2` 0.10.9, `sha1-checked` 0.10.0, `gix-hash` 0.25.0,
  `serde_json`, `libc`.
- Warm start of the baseline: median 3.85 to 4.04 ms over three runs of 300;
  the same-binary control differed by 0.04 to 0.06 ms.
- Cold start: baseline medians of 21.5 ms in one session and 33.9 ms in another,
  because other files were cached differently;
  the control differed by 0.07 to 0.2 ms inside one run.
  Compare binaries only within one run.
- Major page faults of a cold start, five runs each, no spread:
  baseline 5, SHA variant 7, zbus-linked variant 9.
  Minor faults: about 482, 480 and 502.
  This is the stable cold-start metric;
  the cold wall times do not agree across runs (below).

## Jobs one by one

### Safe file saving

What our code does:
files totalling 1,623 lines
(`private_storage.rs` 733, `scratch_location.rs` 335, `random_id.rs` 130, `recovery_files.rs` 114,
`worktree_add_plan_path_questions.rs` 78, `recovery_error.rs` 72, `path_bytes.rs` 64, `lexical_path.rs` 50,
`rename_that_never_replaces.rs` 47).
The write path of `private_storage.rs`: exclusive staging file `.<name>.<uuid>.partial`,
write, `sync_all`, `chmod`, publish by hard link with a checked rename when the filesystem refuses links,
remove the staging name, synchronize the directory (a filesystem that returns `EINVAL`, `ENOTSUP` or `ENOSYS` counts
as synchronized).
Tests pin it: `a_refused_link_falls_back_to_the_rename`, `a_taken_name_is_refused_with_the_operating_systems_error`,
`a_write_cut_half_way_leaves_nothing_behind`, `writers_refuse_a_planted_link`,
`a_directory_that_cannot_be_synced_is_not_an_error`.

Candidates:

- `tempfile` 3.27.0 (MIT or Apache-2.0, last commit 2026-10-30, already linked, 0 added packages).
  `persist_noclobber` tries `renameat_with(NOREPLACE)` and falls back to `hard_link` then `unlink`
  (`src/file/imp/unix.rs` lines 94 to 133 of the 3.27.0 crate).
  Names come from `getrandom` (the `getrandom` feature is on in the tree).
  Differences from our code:
  it does not synchronize the file or the directory,
  its order is rename-no-replace first and hard link second, ours is hard link first and a checked rename second,
  so a filesystem that refuses links but supports `renameat2` works with both, and a filesystem that supports neither
  fails with `tempfile` where ours renames after checking the name (this combination was not tested),
  and it ignores an `unlink` error after the link (whether that breaks the wrapper's cleanup rule was not checked).
- `atomic-write-file` 0.3.1 (BSD-3-Clause, last commit 2026-08-11, 6 added packages: `nix`, `rand`, `chacha20`,
  `cpufeatures`, `rand_core`).
  It always replaces the target through `renameat` and synchronizes the file and the directory
  (`src/imp/unix/mod.rs` lines 184 to 190).
  It has no option to refuse an existing name.
  It cannot do our exclusive publish.
- `atomicwrites` 0.4.4 (MIT, last release 2024-09-19):
  has `DisallowOverwrite` implemented as a hard link and a delete (`src/lib.rs` line 272),
  with `rustix`, `tempfile` and `windows-sys`.
  Same shape as `tempfile`, older, no rename fallback found by reading.
- `cap-tempfile` 4.0.3 and `cap-std` 4.0.3:
  directory-handle based (11 added packages);
  see "Linux file and disk calls" for why `cap-std` does not cover the no-follow rules.
- Random names:
  `uuid` 1.29.1 with `--no-default-features --features v4` adds 1 package (its default features add 11).
  Our `random_id.rs` is 130 lines over the already linked `getrandom`.

The scratch-location rule (`doc/planning/cli-git-scratch-locations.md`, section "Rule") bans
`tempfile::tempdir`, `tempfile::tempfile`, `TempDir::new` and `NamedTempFile::new` because they use the system
temporary directory.
It does not ban `Builder::tempfile_in` with a chosen folder,
so it is no reason against `tempfile`.

Verdict: partly.
`tempfile::Builder` with `persist_noclobber` can take the staging-name and no-replace-publish code (tens of lines),
once the existing publish controls (`private_storage_publish_tests.rs`) are run against it with the combinations above;
whatever fallback they show necessary stays beside the call.
The synchronizing order, the private modes, the no-follow reads with a size limit and the directory-sync tolerance
have no crate counterpart and stay.
What remains is smaller than the library proposed and is still a Git-free job,
but its case as a crate of ours rests on the same weak evidence as before.

### Linux file and disk calls

What our code does:
files totalling 1,964 lines
(`fs_btrfs_ioctl.rs` 455, `fs_beneath.rs` 408, `fs_mount_options.rs` 340, `worktree_trash_remove.rs` 207,
`subvolume_remove_walk.rs` 162, `fs_kind.rs` 147, `fs_primitive_error.rs` 109, `fs_clone_exchange.rs` 71,
`subvolume_remove.rs` 65).
Counts of `libc::` uses and of `unsafe` in the syscall files: 30 and 14 in `fs_beneath.rs`,
19 and 11 in `fs_btrfs_ioctl.rs`, 7 and 4 in `fs_kind.rs`, 3 and 2 in `fs_clone_exchange.rs`.
Behavior pinned: `openat2` with `RESOLVE_BENEATH`, `NO_SYMLINKS`, `NO_XDEV` and `NO_MAGICLINKS`,
`renameat2` `RENAME_EXCHANGE`, `RENAME_NOREPLACE` (also in `rename_that_never_replaces.rs`), `FICLONE`,
`statfs` magic numbers, the btrfs `SNAP_CREATE_V2`, `SUBVOL_CREATE_V2` and `SNAP_DESTROY` ioctls,
and a mount-option read from `/proc/self/mountinfo` (`user_subvol_rm_allowed`).

Candidates:

- `rustix` 1.1.4 (Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT, last commit 2026-10-27, already linked with the
  `fs` feature, 0 added packages).
  The probe `probes/rustixprobe` (rustix 1.1.4, feature `fs`) ran on a btrfs scratch folder and printed:
  `renameat_with` `EXCHANGE` swapped two files;
  `NOREPLACE` on a taken name failed with `EEXIST`;
  `statfs` returned `f_type` 0x9123683e, the btrfs magic;
  `ioctl_ficlone` returned `Ok`;
  `openat2` with the four resolve flags opened a plain file, refused a symbolic link with `ELOOP`
  and refused `../x` with `EXDEV`.
  `rustix` also has `fs::inotify` with `init`, `add_watch`, `remove_watch` and a `Reader`
  (`src/fs/inotify.rs` lines 60 to 176).
  It has no btrfs constants or ioctl wrappers (a search of the 1.1.4 source for `btrfs` found nothing);
  its generic `ioctl` module takes custom opcodes.
  The earlier reason "not needed" (`doc/planning/cli-git-btrfs-worktree.md` line 479) was written before the
  wrapper had 30 `unsafe` call sites in these files.
- `cap-std` 4.0.3 and `cap-primitives` 4.0.3 (11 added packages):
  its Linux open uses `openat2` with `RESOLVE_BENEATH` only
  (`src/rustix/linux/fs/open_impl.rs` line 52);
  a search of the crate for `NO_XDEV` and `RESOLVE_NO_MAGICLINKS` found nothing,
  and it has no `RENAME_EXCHANGE`.
  Our code refuses to cross a mount, so `cap-std` does not cover the pinned behavior.
- `btrfs-uapi` 0.13.0 (MIT or Apache-2.0, 28 added packages):
  has `subvolume_create`, `snapshot_create`, `subvolume_delete` and `subvolume_delete_by_id`
  over a held directory descriptor.
  Its `Cargo.toml` lists `bindgen` 0.72.1 as a build dependency that is not optional,
  so a build needs `libclang`.
  It built on this host in 21.1 s;
  whether the container images of the wrapper carry `libclang` was not checked,
  and no repository rule against a native build prerequisite was looked up.
  `btrfs-transaction` 0.13.0 (same author, appeared in the search) was not evaluated.
- `procfs-core` 0.18.0 has a `MountInfo` parser; see the next section for `procfs`.
  The wrapper has two mountinfo parsers (`fs_mount_options.rs` and `filesystem_identity_mount.rs`),
  one that skips unreadable lines and one that declines the whole table.
  Whether `MountInfo` keeps the super-options and unescapes `\ooo` was not tested.
- `reflink-copy` 0.1.32 (11 added packages, mostly Windows): clones files;
  the wrapper needs `FICLONE` on open descriptors, which `rustix` already has.

Verdict: partly.
Replace the syscall layer with `rustix` (no new package):
the `unsafe` sites in `fs_beneath` (14), `fs_clone_exchange` (2), `fs_kind` (4) and `rename_that_never_replaces` (1),
20 or 21 in all by the counts above (the 11 in `fs_btrfs_ioctl` mostly stay),
and the inotify read loop in `service_index_watch.rs` (248 lines).
Keep the btrfs ioctl definitions (`fs_btrfs_ioctl.rs`),
the mount-option reader unless `procfs-core` is shown to match,
and `fs_primitive_error.rs`.
The tests of the five files pin errno values (`EXDEV`, `ELOOP`, `EINVAL` mapping),
so the swap needs the same controls to run against `rustix` errors;
this was not run.
What remains is `fs_btrfs_ioctl.rs` (455 lines), `fs_mount_options.rs` (340) and `fs_primitive_error.rs` (109),
904 lines, of which only the ioctl layer has no published counterpart that builds without `bindgen`.

### Starting and watching programs

What our code does:
files totalling 5,382 lines
(`signal_relay.rs` 494, `push_progress.rs` 464, `child_watch*.rs` 844, `process_identity.rs` 467,
`index_lock_holders.rs` 449, `process_start.rs` 390, `wait_progress.rs` 333, `tcp_sockets.rs` 318, `process_use.rs` 292,
`thread_quiet.rs` 279, `start_retry.rs` 229, `index_lock_holder_parsers.rs` 203, `process_tree.rs` 200,
`output_capture.rs` 175, `quiet_ledger.rs` 155, `stale_ref_lock_nested_numbers.rs` 90).
The deciding behavior is the owner's "Stop only on no progress":
a child is ended only when it has shown no sign of work for a quiet window,
where a sign is bytes moved (`rchar` plus `wchar` of `/proc/<pid>/io`, summed over the tree),
processor time growing, a task in state `R` or `D`,
or a connected socket with growing `tcpi_bytes_acked` or `tcpi_bytes_received` read from `NETLINK_SOCK_DIAG`.
The signal relay relays a signal to the child only when a process, not the terminal, sent it (`si_code`),
and restores the previous dispositions when the guard drops.
Start retry repeats a start that failed with `EAGAIN` for a bounded time.
Output capture keeps child output in anonymous files so a child can outlive the command.

Candidates:

- No-progress deadline.
  The frozen queries "inactivity timeout child process", "idle timeout subprocess", "stall detection process",
  "process no progress timeout", "child process watchdog" returned crates that are agents or unrelated products;
  none was a library for a child process.
  `wait-timeout` 0.2.1 (269 million downloads, 1 added package) waits for a child for an elapsed time;
  the owner ruled elapsed time out as a reason to end a child.
  `process-wrap` 10.0.1 (11 added packages, mostly Windows) wraps process groups and a kill; no stall logic.
  `kill_tree` 0.2.4 (last release 2024-02-12, 19 added packages including `bindgen`, `nix` 0.27, `rustix` 0.38)
  has no start-time field in its source (a search for `start_time`, `starttime` and `reuse` found none),
  so it cannot do "signalled only while still the process that was found".
  Verdict for the watcher: no candidate; keep.
- Signals.
  `signal-hook` 0.4.6 with `signal-hook-registry` 1.4.8 (2 added packages) can read `siginfo` and has
  `WithOrigin`.
  Its `unregister` documents that it does not restore the previous handler and that the program "will effectively
  ignore" the signal afterwards (`signal-hook-registry-1.4.8/src/lib.rs` lines 695 to 702).
  `SignalGuard` restores on drop, so `signal-hook` cannot replace it.
  This adds a source-backed reason to the earlier recorded ones (`rustix` hides its handler interface;
  `ctrlc` gives the closure no signal number; `doc/planning/cli-git-rust-open-decisions.md` around line 730).
- `/proc` parsing.
  `procfs` 0.18.0 (MIT or Apache-2.0, last commit 2026-06-26, last release 2025-08-30; without default features
  3 added packages: `hex`, `procfs`, `procfs-core`; with default features 27 because of `chrono` and `flate2`),
  built on `rustix`.
  Its `Process` holds a directory descriptor opened once (`src/process/mod.rs` lines 208 to 232),
  with `stat()`, `io()`, `fd()`, `tasks()`, `Task::stat()`, `Task::io()`, `boot_time_secs()` and
  `ticks_per_second()`.
  The probe `probes/procfs` swept every process of this host three times:
  745 processes, all 745 `stat` files parsed,
  `io` readable for 171 and `PermissionDenied` (a distinct error) for 574,
  a sweep of 84 to 90 ms, a binary 157 KB larger than an empty one.
  The wrapper's own sweep was not timed, so no speed claim is made.
  Gap:
  `procfs` opens with plain `rustix::fs::openat` and not `openat2` with `RESOLVE_NO_XDEV`,
  while the handover for `cargo-shadow` records pinned `/proc` reads with `openat2` as a kept guard
  (`doc/handover/cli-git-rust-implementation.md` around line 4603).
  The wrapper's identity strings, the time-namespace offset of field 22 and the lock-holder matching were not
  compared.
- Socket diagnostics.
  `netlink-packet-sock-diag` 0.5.0 (MIT, 4 added packages) has `TcpInfo` with both `bytes_acked` and `bytes_received`
  (0.5.0 source, `src/inet/nlas.rs` lines 599 and 600), with `netlink-sys` 0.9.0 (1 more).
  It was not run against `tcp_sockets.rs` (318 lines of netlink written by hand) and is unvalidated.
  `procfs` offers `/proc/net/tcp` with queue lengths and no byte counters,
  which the wrapper's own documentation gives as the reason for sock_diag.
- Process listing, `sysinfo` and `listeners`: not tested for these jobs.

Verdict: keep the deadline, the signal relay, the output capture and the start retry.
Treat `procfs` (for `process_identity`, `process_start`, `process_tree`, `process_use`, the readers in
`push_progress` and `thread_quiet`) and `netlink-packet-sock-diag` (for the netlink in `tcp_sockets`) as candidates
that could remove parser code, each to be validated against the existing tests before a decision.
The library of ours would then hold the watcher, relay and capture,
which have no published counterpart.

### Parallel work within a budget

What our code does:
`parallel_budget.rs` (248 lines), `parallel_runner.rs` (268), `scoped_start.rs` (169), `side_by_side_threads.rs` (178).
The budget is the allowed CPUs minus a margin, bounded by the memory still free (`/proc/meminfo` `MemAvailable` and
the least headroom up the cgroup hierarchy), a measured ceiling and the person's cap.
The runner runs items on scoped threads and prints their lines in input order, live for the first unfinished item.
`start_or_run_with` runs the job on the calling thread when a thread cannot start:
the owner's rule "Commits should never fail." (`scoped_start.rs`).
Tests pin: `a_starved_host_still_gets_one_worker`, `the_least_headroom_up_the_hierarchy_wins`,
`jobs_run_in_turn_where_no_thread_starts`, `an_ordered_run_where_no_thread_can_start_reports_every_item_in_order`,
`the_first_items_lines_are_live`.

Candidates:

- `std::thread::available_parallelism`:
  already used by `HostResources::read`;
  it follows the cgroup quota (`doc/planning/cli-git-main-worktree-ignored-copy.md` line 279).
- `rayon` 1.12.0 and `rayon-core` 1.13.0 (already linked, 0 added packages).
  `global_registry()` ends in `.expect("The global thread pool has not been initialized.")`
  (`rayon-core-1.13.0/src/registry.rs` line 171), and the single-thread fallback in `default_global_registry`
  runs only when the error kind is `Unsupported` (`registry.rs` lines 218 to 225; `lib.rs` line 745),
  not when the system returns `EAGAIN` for a thread.
  A thread that cannot start therefore panics through `rayon`, where our code runs the job in turn.
  `rayon` also has no ordered live line printing.
- `sysinfo` 0.40.1 (MIT, last commit 2026-10-30; without default features and with `system`, 1 added package).
  Its cgroup code takes the minimum of limit and free memory over the cgroup and its ancestors
  (`src/unix/linux/cgroup.rs` function `memory_limits`).
  The probe `probes/budget` ran our `HostResources::read` and `sysinfo` together under a `MemoryMax` cap:
  at 2 GiB ours gave 2,146,734,080 bytes and `sysinfo` 2,146,734,080;
  at 512 MiB ours gave 536,117,248 and `sysinfo` 536,121,344 (one page of drift);
  in the default cgroup (32,000,000,000 bytes) ours gave 29,369,634,816 and `sysinfo` 29,369,909,248.
  Both read 1 for `CPUQuota=100%`.
  The probe binary was 180 KB larger than an empty one, which includes our copied module.
- `num_cpus` 1.17.0 (2 added packages), `jobserver` 0.1.35, `threadpool` 1.8.1:
  no ordered output, no inline fallback; not tested further.

Verdict: partly.
`sysinfo` could replace `HostResources::read` and its text parsers (`memory_available_from`, `control_group_path`,
`headroom_from` and the group walk), keeping `worker_budget` and the owner's margins.
The ordered runner and the start-or-run-here fallback have no crate that behaves the same.
The parallel library of ours stays, smaller.

### Text, time and log line formats

What our code does:
`iso_time.rs` 97, `js_text.rs` 120, `json_record.rs` 346, `diagnostic_log.rs` 161, `shell_quote.rs` 103.

Per the owner's ruling of 2026-11-07, JavaScript identity is dropped.

Time (`iso_time.rs`, and `days_from_civil` in `conflict/discarded_prune_stamps.rs`):

- `jiff` 0.2.24 is already linked (0 added packages; 0.2.39 is current).
  `Timestamp::from_millisecond(ms).strftime("%Y-%m-%dT%H:%M:%S%.3fZ")` equals our text on 2933 of 2933 instants
  spread over 1970 to 9999, and on -1 ms.
  Its `Display` drops the fraction at zero (`1970-01-01T00:00:00Z`), as does `time`'s RFC 3339 writer
  (`time` 0.3.55, 5 added packages),
  so a plain `Display` loses the fixed width that string ordering would need.
Whether any code orders these strings is the untraced claim below; fixed width is kept as a precaution, not as a
demonstrated requirement.
  `humantime::format_rfc3339_millis` (2.4.0, 1 added package) keeps the three digits.
  Above year 9999 `jiff` returns an error where ours prints a JavaScript-style `275760-09-13`;
  nothing in the wrapper needs it.
- The second hand-written calendar (`days_from_civil`, `days_in_month`) in `discarded_prune_stamps.rs` is the inverse
  of the first; `jiff` covers both.

JavaScript text rules (`js_text.rs`): `trim_javascript`, `is_javascript_whitespace` and `decimal_attempt` emulate
JavaScript.
The ruling frees the output text, not necessarily what the readers accept.
Replacing them with `str::trim` and `str::parse` changes the whitespace set (JavaScript includes U+FEFF and excludes
U+0085); a difference in accepted inputs, overflow and sign handling of `decimal_attempt` was not compared,
and its consumers were not listed.
That comparison comes before any change.

JSON (`json_record.rs`): `quote` already calls `serde_json::to_string`.
`serde_json` does not keep insertion order without its `preserve_order` feature (an extra package),
so whether the small ordered object writer can go depends on whether any reader needs key order;
none was found (the TypeScript reader uses `JSON.parse`), and the question is unresolved.
The reader rules "safe integers follow JavaScript" (accepted number range, duplicate keys) are input semantics that the
ruling does not decide; they stay until the readers of those records are checked.

Shell quoting (`shell_quote.rs`): the probe `probes/shq` ran our function and four crates on eleven inputs
(an empty value, a bare path, a space, a single quote, `$(rm -rf x)`, a newline, an escape sequence, a NUL,
an invalid byte, two accented letters, a tab).
Calls: `shlex::bytes::try_quote(&[u8])`, `shell_words::quote(&str)` and `shell_escape::unix::escape` on the lossy
`String` of the bytes (so the last two never saw an invalid byte as a byte), and
`shell_quote::Bash::quote_into_vec(&[u8])` and `shell_quote::Sh::quote_into_vec`.
Our output contains no literal control character for any input:
`it's` gives `'it'\''s'`, a newline gives `$'a\nb'`, an escape sequence gives `$'\x1b[31mred'`,
a NUL gives `$'a\x00b'`, an invalid byte gives `$'\xffa'`.
- `shlex` 2.0.1, `shell-words` 1.1.1 and `shell-escape` 0.1.5 printed the raw control character for a newline, a tab and
  an escape sequence, so a person who copies the line from a terminal copies it;
  `shlex` returned an error for NUL, `shell-words` printed it raw, and `shlex` wrote `it's` as `"it's"`.
  The invalid byte came out as U+FFFD from `shlex` (bytes in, so the crate or its lossy output converted it)
  and from the other two (they were given the lossy string).
  `shlex` records the missing style in its source (`src/bytes.rs` line 241: "TODO: add $'xxx'").
- `shell-quote` 0.8.0 (Apache-2.0, last release 2026-08-07, 3.5 million downloads, 1 added package with only the
  `bash` feature) is the exception.
  Its Bash quoter printed no control character on any input and wrote the invalid byte as `$'\xFFa'` and NUL as
  `$'a\x00b'`.
  It spells more bytes as escapes than ours does: a space gives `$'a b'` (ours `'a b'`),
  `it's` gives `$'it\'s'`, and the accented letters give `$'\xC3\xA9 \xC3\xBC'` (ours `'é ü'`).
  Its Sh quoter printed raw control characters for a newline, a tab, an escape sequence and NUL.
  Round trip: the Bash-quoted text and ours were each given to bash 5 (`printf %s <quoted> | od`) and gave the input
  bytes back on every input without a NUL; the NUL case cannot be given to a bash argument at all, so the value is a
  diagnostic spelling there.
  The empty value in the script failed for both because of a harness slip (the script shifts its arguments), not a
  quoting fault, and was not rerun.
- `snailquote` 0.3.1 is GPL-3.0-only and was not evaluated pending a licensing review
  (the wrapper declares LGPL-3.0-or-later; no repository policy on copyleft dependencies was read).
Verdict: `shell-quote`'s Bash quoter is a replacement candidate,
the only tested crate API that meets the module's own contract (printed text holds no control character and no line
break, `shell_quote.rs` top doc),
with a readability cost the owner decides (open choice 10).
Three other tested APIs fail that contract.
The module's top doc names a TypeScript twin (`package/wg/cli-quicker/src/shell-quote.ts`) as the source of its
rules; the ruling frees us from that twin.
The control-free rule is the module's own stated contract for text printed to a terminal and is kept separate from
shell grammar (rule `SYB` in `AGENTS.md` covers the grammar of the destination, not terminal display).
The module's tests pin exact strings (`shell_quote_tests.rs`) and would change with the spelling.

Log lines (`diagnostic_log.rs`): the line is
`[level] [time] [monochromatic-git-policy-cli] [tag] message\n` with control characters turned into spaces.
`log` or `tracing` would add a facade and a subscriber for 161 lines;
`tracing` 0.1.44 is already in the lockfile (0 added) and `tracing-journald` 0.3.2 would add 1, but nothing was built.
Verdict: keep, small.

Readers of these formats that exist in the repository, and what a change would break
(the search used `rg` over `package/git-policy` for `createdAt`, `invokedAt`, `[warn]`, `[debug]`, the time shape and
the timing file names):

- `transaction_owner.rs` (lines 37 to 180) writes and reads `createdAt` as an opaque string with `string_field`;
  a search of non-test code for `created_at` found no comparison site.
  The doc comment of `iso_time.rs` says recovery orders dead transactions and the reservation grants turns by
  comparing these strings;
  that claim was not traced to code.
  Any RFC 3339 UTC form with a fixed width still sorts correctly;
  a form that drops `.000` does not.
- `capture_invocation.rs` writes `invokedAt`; `commit_timing.rs`, `add_timing.rs`, `push_trigger_log.rs`,
  `work_item.rs`, `service_handoff.rs`, `subvolume_journal.rs` write the string into JSON-lines or JSON records.
- `conflict_sides.rs` (line 65) takes the first 19 characters of the text to build a stamp,
  and `discarded_prune_stamps.rs` parses that stamp back with `days_from_civil`:
  these two depend on the `YYYY-MM-DDTHH:MM:SS` prefix and on nothing else.
- `package/git-policy/cli.fuzz` (`seed/owner_records/owner`, `dictionary/owner_records.dict`, `src/owners_tests.rs`)
  holds owner records with the `createdAt` text as seeds.
- `package/git-policy/cli/bin/native-perf-evidence.ts` reads the timing file
  `monochromatic-git-policy-cli-commit-timing.jsonl` with `JSON.parse` per line;
  any valid JSON works.
  `bin/native-perf.ts` writes `new Date().toISOString()` into its own result files and reads nothing back in this shape.
- The archived result files under `package/git-policy/cli/perf/*.json` quote old `[warn] [<time>] [cli-git]` lines
  as history; nothing reads them in code.
- `binary_service_pruned_tests.rs` (line 113) and `binary_service_replay_tests.rs` (line 255) drop every stderr line
  that starts with `"[warn] "` before comparing the rest, so the `[warn] ` prefix is a contract;
  the time and tag inside may change.
- No journal reader, `journalctl` script or log parser in the repository reads these lines;
  the journal path writes fields (`CLI_GIT_REQUEST_ID`, and others) not this line shape.

Tests that pin JavaScript-identical text only for its own sake, and may change:
`iso_time_tests.rs::known_instants_match_to_iso_string`,
`json_record_tests.rs::the_writer_matches_json_stringify` and `quoting_matches_json_stringify`,
`safe_integers_follow_javascript`, `the_fatal_decoder_matches_text_decoder`,
and the whole of `js_text_tests.rs`.
Tests that pin a shape other code reads and so should stay or move with the reader:
`diagnostic_log_tests.rs` (the `[warn] ` prefix and control-character rule),
`transaction_owner_tests.rs`, `discarded_prune_tests.rs` and `discard_audit_tests.rs` (the stamp prefix).

Verdict for the job:
replace `iso_time` with `jiff` (and the second calendar with it), keep `quote` on `serde_json`,
treat `js_text` as replaceable by `std` only after its readers are compared (see the next paragraph),
take `shell-quote` for `shell_quote` if the owner accepts its spelling, keep `diagnostic_line`.
After that, `text-time-and-log-line-formats` has no job that is general and not covered by a crate;
it no longer justifies a crate of ours.

### Talking to the systemd user manager

What our code does:
`systemd_dbus.rs` (377 lines) and `systemd_dbus_wire_format.rs` (316), tests `systemd_dbus_tests.rs` (571 lines)
and a fuzz target.
It speaks D-Bus to the session bus socket for `Hello`, `Reload`, `ResetFailedUnit`, `StartUnit`, `StopUnit`,
`GetUnit` and an `ActiveState` read.
One deadline of 20 s (`EXCHANGE_WAIT`) bounds the whole exchange: connect, authentication, `Hello` and the calls.
The reader is total over arbitrary bytes.
Tests pin: `a_call_is_bounded_by_the_time_it_was_given_and_a_timeout_is_told_apart`,
`refusals_missing_sockets_and_silence_are_told_apart`, `the_reader_is_total_over_damaged_input`.

Candidates, with the recorded reason ("roughly fifty crates and an executor",
`doc/planning/cli-git-resident-service.md` line 847) checked:

- `zbus` 5.19.0 with default features (`async-io` and `blocking-api`):
  52 added registry packages in the lockfile (some are for platforms the wrapper does not build, such as `js-sys`,
  `wasm-bindgen` and `uds_windows`), so "roughly fifty" is right as a lockfile count.
  Linked into the wrapper behind an environment switch and run against the real user manager (read only:
  `GetUnit` then the `ActiveState` of `dbus.socket`, which printed `active`):
  the release binary grew from 16,735,752 to 19,207,912 bytes (+2,472,160, +14.8%),
  the call took under 10 ms with 3 `clone3` calls (the executor threads), and a cold start took 9 major faults
  against 5.
  Warm start was unchanged (median 4.10 ms against 4.04 and a control at 4.08).
  The only timeout in its connection builder is `method_timeout`, which wraps a method call
  (`src/connection/mod.rs` lines 269 to 270);
  the handshake has none in the files read, so the single 20 s deadline would need a thread and a channel around it.
- `zbus` 6.2.1 (last release 2026-11-01): the default build has 30 added packages and also raised `rustix` from
  1.1.4 to 1.1.6 in the lock.
  The source has no `blocking` module (a search for `mod blocking` found nothing), so there is no blocking client;
  the crate moved to an async runtime layer (`zruntime`).
  The catalog entry for `zbus` is `>=5.19.0` (`rust-catalog.toml` line 128), so a new dependency would resolve 6.x,
  not the 5.x that the nested-wayland-session sibling locks.
  With `default-features = false` it compiles only the wire format and names: 6 added packages, an encoded
  `("cli.socket", "replace")` of 28 bytes round-tripped,
  and the standalone probe was 285 KB larger than an empty one (not measured inside the wrapper).
- `rustbus` 0.19.3 (MIT, last release 2023-08-29, near-zero recent downloads): the recorded reason "not in the offline
  registry cache" was an environment limit;
  the crate is on crates.io, but it has not been released for more than three years.
  It was not built.
- `dbus` 0.9.12 links `libdbus` (the recorded reason stands).
  `systemd` 0.11.1 links `libsystemd` and is LGPL-2.1-or-later; `libsystemd` 0.7.2 adds 15 packages.
- A search for "systemd dbus StartUnit" returned 0 crates.

Verdict: keep the connection, with the recorded reason now measured:
+52 lockfile packages, +2.47 MB, 4 more major faults on a cold start,
no deadline over the handshake, and no blocking API in the current major version.
Partly, if the owner wants it:
the 316-line encoder and reader could use the wire-format-only build of `zbus` 6
(6 added packages, +285 KB in a standalone probe, not measured inside the wrapper);
this was not run against the fuzz target or the `damaged_replies_are_refused` controls.

### Fix packages after copying a folder

What our code does:
the files of `repair/`, 2,321 lines (`package_repair_cargo.rs` 476, `package_repair_command.rs` 395,
`package_repair_tree_clone.rs` 270, `package_repair_request.rs` 201, `package_repair_report.rs` 196,
`package_repair_toolchain.rs` 191, `package_repair_cargo_workspaces.rs` 182, and others):
relink `node_modules`, rewrite Cargo build state, repair toolchain launchers after a worktree copy.

Candidates: the queries "package.json dependent version bump", "pnpm workspace", "node_modules relink",
"repair node_modules", "cargo target directory relocate" returned crates of 4 kinds:
this repository's own (`monochromatic-pnpm-shadow`, `monochromatic-catalog-specifier`, the wrapper itself),
release tools, agent products, and unrelated names.
Nothing repairs a copied `node_modules` or a Cargo `target`.

Verdict: no candidate found.

### Dependent version bump check

What our code does:
about 2,600 lines of non-test files in `dependent_version/`
(`lifecycle` 343, `manifest` 286, `content` 269, `plan` 231, `release` 205, `policy` 203, `states` 182, `text` 179,
`graph` 170, `imports` 155, `bundled` 138, `paths` 134, `publishable` 110).

Candidates: queries "package.json dependent version bump" (186 results), "monorepo version bump dependents"
(180), "workspace dependents bump npm" (296), "semver bump package.json" (64) returned release tools:
`changesette`, `changepacks`, `cargo-monorepo`, `sampo`, `monochange`, `ferrflow`, `vup-rs`, `bver`.
These are mostly command-line tools that decide a release; none was read as a library.
`semver` 1.0.28 is locked and adds 0 packages; whether the planner already parses versions by hand was not checked.

Verdict: candidates found, library suitability not evaluated.
Nothing read takes the planner's input (a changed package's version, a workspace of `package.json` files) and returns
the dependents to bump,
but `changesette` and `changepacks-core` were not read against `dependent_version_plan_tests.rs`,
and that reading comes before any statement that the job is unique.

### Disposable test folders

`tempfile` 3.27.0 makes folders (`TempDir`, `Builder::tempdir_in`) and files,
and has `keep` for the owner's keep-fixtures switch.
It does not provide the watchdog thread (`arm_watchdog`), the owned process group (`own_group`, `spawn_owned`),
the core-dump ban (`forbid_core_dumps`) or `run_child_under_file_size_limit`.
`assert_fs` and `rusty-fork` were not read.
Verdict: partly.
The folder helpers (`fixture`, `fixture_with_prefix`, `fixture_root`, `settle_fixture`, `remove`) can sit on `tempfile`;
the rest stay.
A test-only crate of ours would then hold the process helpers, not the folders.

### Jobs folded into the program

Each is named in the proposal's section "Jobs found buried in the program".

- System journal writer (`journald_native.rs` 557 lines with `journald_native_socket_target.rs` 46 and
  `client_journal.rs` 116).
  `systemd-journal-logger` 2.2.2 (MIT or Apache-2.0, 1 added package) sends with `UnixDatagram::send_to`
  (`src/client.rs` line 51);
  its socket is `UnixDatagram::unbound()` (line 37) and the send is `send_to(payload, JOURNALD_PATH)` with no
  `set_nonblocking`, so the send blocks when journald's queue is full (a search for `nonblock` and `WouldBlock` found
  nothing).
  `tracing-journald` 0.3.2: `UnixDatagram::unbound()` at `src/lib.rs` line 106 and `send_to` at line 245, same search
  result.
  A full-queue probe of either crate was not run.
  Both use the `log` or `tracing` facade, not a call with fields.
  `libsystemd` 0.7.2 adds 15 packages and `systemd-journal-sdk-*` is GPL-3.0-or-later.
  The wrapper pins `a_full_journal_queue_falls_back_instead_of_waiting` and
  `a_full_journal_stream_drops_the_line_instead_of_waiting`:
  a commit never waits for the journal.
  The recorded reason (`doc/handover/cli-git-logging.md` lines 333 to 337: "No crate was added.
  Ranking: no crate (now) > a journal crate") rests on sealed `memfd` support alone;
  the blocking send is a second, source-backed reason.
  Verdict: keep.
- Three-way text merge (`word_merge.rs`, 366 lines, on `similar` 3.2.1 already).
  The probe `probes/merge` ran the stored corpus of 41 inputs, where Jujutsu 0.46.0 resolves 21 and leaves 20:
  ours matched the verdict and the bytes on 41;
  `diffy` 0.5.2 (1 added package) on 31 and resolved 11;
  `flickzeug` 0.6.0 on 31 and resolved 11;
  `reconcile-text` 0.14.1 (word tokenizer) matched 18 of the 21 resolved bytes and resolved all 41,
  so it would resolve 20 inputs that Jujutsu leaves for a person.
  `gix-merge` 0.23.0 adds 49 packages and works on lines.
  `doc/planning/cli-git-merge-auto-resolution.md` had already compared `jj-lib`, Jujutsu's code and `weave`.
  Verdict: keep.
- Crash points for tests (`transaction_phase.rs` 413 lines, `service_fault.rs` 139).
  `fail` 0.5.1 (last release 2022-10-08, 7 added packages) has the actions `off`, `return`, `sleep`, `panic`, `print`,
  `pause`, `yield`, `delay` and a callback (`src/lib.rs` lines 268 to 284, 416 to 423).
  It has no `SIGKILL`, no `.reached` marker file, and its `pause` ends when the configuration changes in process, not
  when a file appears.
  The wrapper's `CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill:<dir>` needs all three.
  A callback could do the kill and the marker, but the variable parsing, the refusal of a malformed value and the
  release-file wait would stay ours; how much code that saves was not measured.
  Verdict: keep.
- Durable work items (`work_item.rs` 540, `work_item_recovery.rs` 271, claim names 116).
  Queries "durable queue files", "file based work queue", "claim work item rename" returned 1,175, 2,521 and 475
  results, none a library for claim-by-rename directories.
  `yaque` 0.6.6 (last release 2023-11-04, 49 added packages with `notify` and `futures`), `queue-file` 2.0.0
  (2 added packages), `fjall` 3.1.13 (18) and `redb` 4.5.0 (1) are single-writer queues or key-value stores;
  the wrapper's design needs several processes claiming by atomic rename
  and recovering stale claims by process identity.
  Their semantics were read from their descriptions, not tested.
  Whether claim-by-rename directories are binding, or only the durability, concurrency and recovery they give,
  was not separated.
  Verdict: unresolved.
- Btrfs subvolume conversion core (about ten files of `converting_the_main_checkout`).
  Queries "btrfs subvolume" (56), "btrfs snapshot create ioctl" (7), "convert directory to subvolume" (4):
  the hits are `btrfs-uapi`/`btrfs-transaction` (ioctl layer, see above), backup tools and the wrapper itself.
  Verdict: no candidate found.
- Markdown autofix runner:
  it runs the repository's own linter (`monochromatic-lint`) on a guarded thread;
  `rumdl` 0.3.2 is a different linter with its own rules.
  Verdict: not applicable.
- SHA-1 and SHA-256 digests: see the next section.
- Saved copies with age and size limits:
  `trash` 5.3.0 is the desktop trash;
  verdict: no candidate found for an age and size pruned store.
- Rotating audit log (`discard_audit_generations`):
  `file-rotate` 0.8.0 adds 25 packages (`chrono`, `flate2`) and rotates by size or lines.
  The audit log's contract was not read against it.
  Verdict: unresolved.
- Unified-diff reader and reverse-apply check (`replay_patch.rs` 379, `replay_reverse_apply.rs` 255):
  its header says it is "the exact emulation of `git apply --reverse --check`" and a seeded differential run compares
  it with real Git.
  `diffy` 0.5.2, `flickzeug` 0.6.0 (fuzzy application, which is the opposite of an exact check), `patch` 0.7.0 (25),
  `unidiff` 0.4.1 (6) implement their own matching.
  No candidate was run against the Git oracle, so this verdict is a reasoned keep, not a measured one.
- Inotify wait (`service_index_watch.rs`, 248 lines): `rustix::fs::inotify` is already linked (above);
  the `inotify` crate 0.11.5 adds 10 packages with default features and `notify` 8.2.1 adds 9.
  Verdict: replace the read loop with `rustix` when `rustix` is adopted.

### Digests

What our code does:
`subvolume_sha.rs`, 172 lines of SHA-1 and SHA-256 written by hand for the index checksum;
its header says "a new dependency was not shown to be needed for two short functions".
It is used by `subvolume_index.rs`, `index_reset_file.rs`, `worktree_clean_content.rs` and `service_index_claim.rs`.
A commit `842830c2ee` (2026-11-03T09:48:24-05:00) already optimized the hand SHA-1 for speed.

Candidates:

- `sha1` 0.10.6 and `sha2` 0.10.9 (RustCrypto, MIT or Apache-2.0, repository active 2026-11-03):
  already in the wrapper's lockfile and normal dependency tree through `gix-hash` and `sha1-checked`,
  so a direct dependency at `=0.10.x` adds 0 packages.
  The catalog holds `sha2 = ">=0.11.1"` (`rust-catalog.toml` line 105) for the siblings,
  so `workspace = true` resolves 0.11.1, which adds 7 packages (`digest` 0.11.3, `block-buffer`, `crypto-common`,
  `cpufeatures` 0.3, `hybrid-array`, `const-oid`) beside the 0.10 set;
  `sha1` 0.11.0 adds the same six supporting packages and itself, so both together add 8 (the six are shared).
- Speed (`probes/sha`, 15 batches, ours copied unchanged, median in microseconds per digest):
  index-sized input of 521,631 bytes: ours SHA-1 772.9, `sha1` 0.10.6 182.1 (4.2 times), 0.11.0 181.5;
  ours SHA-256 1,359.3, `sha2` 0.10.9 194.8 (7.0 times), 0.11.1 194.9.
  The same-function control (ours again) differed by 1.5 and 0.9 microseconds.
  A 64-byte input: ours SHA-1 0.28, `sha1` 0.07 and 0.05; ours SHA-256 0.39, `sha2` 0.06 and 0.07.
  All four agreed on 305 inputs (every length to 300 and four larger).
  So one digest of an index-sized input saves about 0.6 ms of SHA-1 on this host;
  how many such digests a commit makes was not counted, and the commit path was not measured.
- The wrapper with the crates: `subvolume_sha.rs` replaced by 12 lines calling `sha1::Sha1::digest` and
  `sha2::Sha256::digest`, `sha1 =0.10.6` and `sha2 =0.10.9` added, lockfile delta 0 packages.
  `cargo test --lib -- subvolume_sha index_full_write index_reset_file index_full` ended with
  `43 passed; 0 failed; 1 ignored` (the ignored one is the 512 MiB control),
  including `index_reset_file::tests::a_sha256_repository_equals_git`,
  `every_index_version_equals_git` and `subvolume_sha::tests::sha1_matches_the_published_vectors`.
  Real Git accepts the checksums written through the crates in those tests.
  Release binary: 16,955,248 bytes, 219,496 more (+1.3%) than the baseline, because the crates' block code was not
  reached before.
  Cold major faults 7 against 5; warm start unchanged.
- `ring` is linked through `ureq`'s TLS and has SHA-256 and a legacy SHA-1; not tested, no reason to prefer it over
  the crates every sibling already uses.

Verdict: replace.
The recorded reason in `doc/planning/cli-git-source-layout-target-tree.md` ("replace by the `sha2` crate for SHA-256
and a SHA-1 crate for SHA-1") is right and is now measured.
The only open question is the version (open choice 2).

## What the repository already records

- D-Bus: `doc/planning/cli-git-resident-service.md` line 847; measured above.
  The part "rustbus is not in the offline registry cache" was an environment limit;
  the replacing fact is rustbus's 2023 last release.
- Signals: `doc/planning/cli-git-rust-open-decisions.md` lines 695 to 790.
  Checked: `rustix` hides `kernel_sigaction`; `ctrlc` hands no signal number.
  Not recorded there and found now: `signal-hook-registry` does not restore the previous handler.
- Btrfs ioctls: `doc/planning/cli-git-btrfs-worktree.md` line 479: "`rustix` 1.1.x ... is not needed".
  That judged three ioctls; the wrapper now has `openat2`, `renameat2`, `statx` and `FICLONE` in the same files,
  which `rustix` covers.
- Journal: `doc/handover/cli-git-logging.md` lines 333 to 337 (see above).
- Hashing: `doc/planning/cli-git-spawn-and-sync-reduction.md` around line 420 chose FNV-1a for its own record digests
  because `sha1` and `sha2` were "in the lockfile only through the scanner" and a direct dependency "would go
  through the catalog"; the index checksum in `subvolume_sha.rs` is a different digest with a Git-defined algorithm.
- Merge: `doc/planning/cli-git-merge-auto-resolution.md` lines 860 to 910 compared Jujutsu's code, `jj-lib` and weave.
- Scratch locations: `doc/planning/cli-git-scratch-locations.md`, section "Rule" (above).
- Siblings that chose crates: `sha2` in `package/browserslist/core-rust`, `package/file-enforcer/core-rust`,
  `package/mpm-shadow/cli`, `package/cargo-shadow/cli`; `rayon` in `package/forbidden-strings`;
  `zbus` in `package/nested-wayland-session/cli` and `package/music-player/desktop-app`.
- No recorded reason was found, after searching `doc/planning`, `doc/handover`, `doc/decision`, `doc/troubleshooting`,
  the README, SPEC and DESIGN of the wrapper and `git log --grep` of its folder, for:
  atomic file saving, the watcher and the `/proc` readers, the parallel budget, the time and text formats,
  the version planner, the package repair and the work-item queue.
  No rationale was found in the searched records.

## What changes in the crate split

- `text-time-and-log-line-formats` leaves the list:
  time goes to `jiff`, JSON quoting is already `serde_json`, `shell_quote` may go to `shell-quote`;
  what remains (`diagnostic_line`, the ordered object writer, `js_text` until its readers are compared) folds into the
  program, and the crate was the one the proposal graded as tier two.
- `linux-file-and-disk-calls` shrinks to the btrfs ioctl layer and the mount reader,
  or becomes folders of the program if the owner prefers.
- `safe-file-saving` shrinks by the staging and publish code;
  it stays a candidate for a crate of ours with the synchronizing and no-follow record primitives.
- `starting-and-watching-programs` stays, with a smaller `/proc` part if `procfs` is adopted after validation.
- `parallel-work-within-a-budget` stays with the runner and the fallback; its host reader may become `sysinfo`.
- `talking-to-the-systemd-user-manager`, `fix-packages-after-copying-a-folder`, `dependent-version-bump-check`
  stay: no published crate covers them.
- `disposable-test-folders` stays with fewer helpers.
- Folded jobs: the digests are replaced; everything else folded stays folded.
- Of the eight production libraries of the proposal (plus the test-only ninth), option (a) of open choice 1 keeps six
  production libraries (`safe-file-saving` smaller, `starting-and-watching-programs`,
  `parallel-work-within-a-budget`, `talking-to-the-systemd-user-manager`, `fix-packages-after-copying-a-folder`,
  `dependent-version-bump-check`), option (b) keeps eight, option (c) keeps five or six.

## Open choices for the owner

Status after the owner's answers of 2026-11-07 (section "The owner's answers of 2026-11-07, and how they change this
document"): choice 1 is not yet answered ("not ready"); the crates of choices 2 and 3 and the time stamps and memory
headroom are chosen (the `sha1` and `sha2` version of choice 2 and the catalog pin are not stated);
choice 4 is answered as `zbus` 6 on `tokio`, with the toolchain failure raised in choice 11;
choice 10 is answered "Keep ours (readable)".
Choices 5 to 9 are not answered. The text of each choice is kept as it was written.

Each has options, pros and cons, and a ranking with a reason for each adjacent pair.

### 1. What to do with the crates this research shrinks

Options:
(a) keep the production libraries that still hold a job no crate does (`starting-and-watching-programs`,
`parallel-work-within-a-budget`, `talking-to-the-systemd-user-manager`, `fix-packages-after-copying-a-folder`,
`dependent-version-bump-check`, and a smaller `safe-file-saving`), keep `disposable-test-folders` smaller, and fold
`text-time-and-log-line-formats` and `linux-file-and-disk-calls`;
(b) keep all eight as proposed and let each swap crates inside;
(c) as (a), and also fold `safe-file-saving` and `disposable-test-folders`.

Pros and cons:
(a) matches the "reuse before build" rule and keeps crates that have a demonstrated job, but it changes the count the
owner already saw.
(b) keeps the plan stable, but keeps crates whose job is mostly someone else's crate.
(c) is the least to maintain, but folds `safe-file-saving`,
whose write-and-sync primitives the siblings `cargo-shadow` and `file-enforcer` copy today
(the proposal's tier-one evidence).

Ranking: (a) > (b) > (c).
(a) beats (b) because a crate of ours that wraps `tempfile`, `rustix` and `jiff` mostly adds a name to remember.
(b) beats (c) because (c) also folds `safe-file-saving`, the place where the proposal found two siblings carrying their
own copy; (b) keeps it, at the price of keeping `text-time-and-log-line-formats` as a thin crate.

### 2. The version of `sha1` and `sha2`

Options: (a) a named catalog pin at 0.10.x (`[catalogs.pinned]`, runbook section "Pin or bound a version");
(b) follow the catalog floor, `sha2` 0.11.1 and `sha1` 0.11.0.

Pros and cons:
(a) adds 0 packages and no second `digest` in the wrapper's lock, but the wrapper then differs from four siblings
and the pin needs a comment explaining why (`PIN` in `AGENTS.md`).
(b) matches the siblings and moves with the update tool,
but adds 8 packages with both crates (7 with one) and a second copy of `digest` next to
the 0.10 one that the scanner still locks, and the measured speed is the same.

Ranking: (b) > (a).
(b) beats (a) only provisionally: the speed is equal and the siblings already follow the floor,
but the 0.11 variant of the wrapper was not built, so its binary and lock cost beyond the package count is unknown.
The owner may value a smaller lock more.

### 3. How far to move the Linux calls to `rustix`

Options: (a) the syscall files and the inotify loop; (b) leave `libc`.
The signal relay stays on `libc` either way, because `rustix` hides its handler interface.

Pros and cons:
(a) removes 20 or 21 `unsafe` sites at 0 new packages, but every errno mapping control must run again against `rustix`
errors, and reachable code from more `rustix` features is not measured.
(b) keeps tested code, but keeps the `unsafe`.

Ranking: (a) > (b), provisional.
(a) beats (b) because the probe showed `rustix` behaving as the wrapper needs on a btrfs folder for the calls tried
and no package is added;
the ranking stands only if the errno controls and the inotify replacement pass, which was not run.

### 4. The D-Bus client

Options: (a) keep the hand client; (b) keep the connection and use the wire-format-only `zbus` 6 for the encoder and
reader; (c) `zbus` 5.19 blocking; (d) `zbus` 6 with its async runtime.

Pros and cons:
(a) is measured at 693 lines, fuzzed, with one deadline over everything.
(b) removes 316 lines for 6 packages and about 285 KB (standalone probe), but the fuzz target and the damaged-reply
controls must be re-pointed at a reader the crate owns.
(c) removes both files but adds 52 packages, 2.47 MB, an executor thread and an unbounded handshake.
(d) needs an async runtime in a binary that starts on every command.

Ranking: (a) > (b) > (c) > (d).
(a) beats (b) because the reader is already fuzzed and bounded, and (b) saves 316 lines at a cost not measured inside
the wrapper; if a later measurement shows the crate's reader is total over arbitrary bytes, (b) could rank first.
(b) beats (c) because (c) was measured at +52 lockfile packages and +2.47 MB inside the wrapper while (b) was only
measured standalone, and (c) has no handshake deadline in the files read, so the single 20 s deadline would need a
helper thread.
(c) and (d) are not ranked against each other on evidence: (d) was not built, so its startup, cancellation and
deadline behavior are unknown; (c) is listed first only because it was measured.

### 5. Whether to validate `procfs` and `netlink-packet-sock-diag`

Options: (a) validate `procfs` against `process_identity`, `process_start`, `process_tree`, `process_use` and the
progress readers, and `netlink-packet-sock-diag` against `tcp_sockets`, then decide; (b) keep the hand parsers.

Pros and cons:
(a) could remove parser code (about 1,700 lines of process facts) for 3 plus 5 lockfile packages, but `procfs` lacks the
`openat2` guard the repository kept elsewhere and a validation costs real work.
(b) is zero work and keeps tested guards.

Ranking: (a) > (b).
(a) beats (b) only on expected information: the parse of every process of this host succeeded (745 of 745) and the
crate holds a directory descriptor per process, but the decisive gaps (the `openat2` guard, identity races,
time-namespace offsets, socket counters) are untested.
A bounded plan: run the existing identity, start-time, tree and holder controls against `procfs`, and the socket
controls against the sock-diag crate, in a worktree, and stop at the first control that cannot be met.
If the owner counts the `openat2` guard as binding, (b) wins.

### 6. The time stamp shape

Options: (a) fixed-width `YYYY-MM-DDTHH:MM:SS.mmmZ` through `jiff`'s `strftime`; (b) `jiff` `Display`
(variable fraction).

Ranking: (a) > (b).
(a) beats (b) because the 19-character stamp prefix of `conflict_sides.rs` and any string ordering keep working;
(b) drops `.000`, and whether anything orders these strings is untraced.

### 7. What happens to the JavaScript-parity tests

Options: (a) classify each assertion of the tests named in "Text, time and log line formats" as obsolete byte
identity, retained format contract or retained input safety (decoding, number range), then rewrite only the first;
(b) keep them as they are.

Ranking: (a) > (b).
(a) beats (b) because the owner ruled output parity unnecessary on 2026-11-07, which makes the first kind of assertion
obsolete; the other two kinds stay, and (b) keeps tests whose only reason is gone.
Keeping a test does not require keeping its implementation.

### 8. Documents that name the twin tools

`private_storage.rs`, `iso_time.rs`, `process_identity.rs` and `shell_quote.rs` cite files of the old TypeScript wrapper
as the source of their formats;
under the ruling those citations become history.
Options: (a) reword the top docs when each module is touched; (b) one pass now.
Ranking: (a) > (b), because (a) costs no extra churn.

### 9. Who runs the full vet

The skill asks the owner who enumerates candidates.
Options: (a) this session class again, extended to saturation and scoring for the crates the owner adopts (`sha1`,
`sha2`, `jiff`, `rustix`, `tempfile`, `sysinfo`, and `procfs` if chosen); (b) a cheaper model the owner names;
(c) the owner.

Ranking: (a) > (b) > (c), on a stated preference rather than evidence.
(a) beats (b) because the probes, scripts and read sources are in this session's scratch folder, which saves a
handoff (another reviewer could reuse them at the cost of reading them);
(b) beats (c) because the owner spends no time on enumeration, which assumes the owner would rather not;
if the owner prefers to run it, (c) first.

### 10. The shell quoting spelling

Options: (a) keep `shell_quote.rs`; (b) adopt `shell-quote` 0.8.0's Bash quoter (1 added package) and accept its
spelling, which escapes spaces, quotes and non-ASCII letters inside `$'...'`; (c) adopt it and wrap it so bare words
and plain single-quoted text keep their present look.

Pros and cons:
(a) keeps readable output, 103 lines of ours and tests pinning exact text.
(b) removes the module for 1 package with byte-exact, control-free output, but a person reading
`$'\xC3\xA9 \xC3\xBC'` instead of `'é ü'` loses readability.
(c) keeps readability, but keeps a wrapper that is most of the module.

Ranking: (a) > (b) > (c), provisional.
(a) beats (b) only because readability of the printed command is a person-facing feature the owner has not ruled on;
if the owner takes byte-exact escapes as fine, (b) first.
(b) beats (c) because (c) keeps most of the code (b) would remove.

## Tokio as the runtime

Prototype of 2026-11-07, after the owner's rulings above.
Nothing on `main` changed except documents.
The code is a patch against `main` at `c3d037b3c8` in a throwaway worktree, kept at
`${HOME}/temp/agent/published-crates/prototype.patch` (new file
`lazy_tokio_runtime.rs` beside it); it is not for merging.

### What does not hold: the repository's pinned toolchain cannot compile zbus 6

- `toolchain.lock.json` pins `rust@nightly` as `nightly-2026-11-01`, whose compiler is
  `rustc 1.101.0-nightly (f9ceb4a4d 2026-10-31)`.
- With it, every build of `zbus` 6 that enables a connection fails in `zbus` itself:
  `error[E0391]: cycle detected when computing when zbus_utils::signature::child::Child needs async drop`
  (from `zbus_utils-6.1.0/src/signature/child.rs` line 7, reached through `zbus_utils::signature::Signature`).
  Checked with `zbus` 6.2.1 and 6.0.0, with the feature `default-rt` and with the feature `tokio`, in a scratch crate
  outside the repository (so the repository's own settings are not the cause).
- The same crates compile on `nightly-2026-10-28` (`rustc 41089d07a 2026-10-27`), on `nightly-2026-10-17`, on the
  installed `nightly` (`rustc 19833af72 2026-10-20`) and on stable 1.99.0.
  The first toolchain that fails is the one with `f9ceb4a4d`.
- Searches of the `zbus` issues for "async drop" and of `rust-lang/rust` issues for the cycle message found nothing,
  so whether this is a compiler regression or a `zbus` defect, and whether a fix is coming, is not known.
- The requirement it breaks: the wrapper must build with the repository's pinned toolchain.
- `zbus` 5.19.0 fails the same way on the pinned nightly
  (`cycle detected when computing when zvariant_utils::signature::child::Child needs async drop`,
  `zvariant_utils-4.2.0/src/signature/child.rs` line 7), in a scratch crate;
  it built on `nightly-2026-10-27` earlier in this study.
  `package/nested-wayland-session/cli` locks `zbus` 5.19.0 (its `Cargo.lock`), so it very likely does not build on the
  pinned toolchain either; that package was not built, and the music player (which also names `zbus`) was not looked at.
  Going back to 5.x therefore does not avoid the problem.
- Every number below comes from builds on `nightly-2026-10-28`, with the baseline rebuilt on it from the same source
  (the baseline binary on that toolchain is 16,786,200 bytes; it includes the wired `ureq` call).
  The owner's options: wait for a toolchain or `zbus` fix, pin an older nightly for the wrapper, or build the wrapper
  on stable 1.99.0 (not tried for the whole wrapper).

### How a caller without async code drives zbus 6, and the smallest setup (before the tokio ruling)

- `zbus` 6 has no blocking module (a search of the 6.2.1 source for `mod blocking` found nothing).
  It offers `zbus::block_on` (`src/utils.rs` lines 63 to 88): with `default-rt` it runs a future on the calling thread
  and on `zruntime` 0.2, a single-thread runtime that `zbus` brings; "if the call returns while a connection is still
  alive, a helper thread takes over that connection's work until the next call".
  The smallest setup is `zbus` with `default-features = false, features = ["default-rt"]` and calls inside one
  `zbus::block_on`.
  In the wrapper's lock that setup added 27 packages and removed 2 (a `rustix` and `bitflags` bump);
  it was not built inside the wrapper after the owner chose `tokio`.
- With the feature `tokio`, `zbus` picks the runtime that is current on the thread
  (`src/runtime/mod.rs`, `default_for_build`: "Tokio when the `tokio` feature is on and a runtime is current on this
  thread"), needs `tokio` features `rt`, `net`, `time`, `sync` (`Cargo.toml` of 6.2.1), and its own `block_on` then
  creates a process-global runtime, current-thread, or multi-thread with the feature `tokio-multithread`.
  The prototype owns the runtime instead and enters it with `Runtime::block_on`, which is what the owner's ruling asks.

### The prototype

- `foundation/lazy_tokio_runtime.rs` (35 lines): one `OnceLock<io::Result<tokio::runtime::Runtime>>`,
  built on the first call of `run(future)`; nothing runs at process start.
  A cargo feature picks the flavour for the measurement: `new_current_thread`, `new_multi_thread` (default workers),
  or `new_multi_thread` with 2 workers.
- `service/systemd_dbus.rs`: the hand client `Bus` (the file was 377 lines) replaced by 106 lines of `zbus` calls:
  `zbus::connection::Builder::address("unix:path=...").build()` (connect, authenticate, `Hello`),
  `call_method` for `Reload`, `ResetFailedUnit`, `StartUnit`, `StopUnit` and `GetUnit`, and
  `org.freedesktop.DBus.Properties.Get` for `ActiveState`.
  Errors map to the old `CallError`: a method error to `Manager { name, text }`, a handshake error to
  `Authentication`, an unreachable socket to `Connection`, an elapsed deadline to `TimedOut`.
- One deadline over the whole exchange: `bounded(deadline, future)` runs the future under
  `tokio::time::timeout(remaining, ...)`, for the connect (handshake and `Hello` are inside `build()`)
  and for each call;
  `within(wait)` replaces the deadline as before.
- `standards_review/standards_review_client.rs`: `ureq` replaced (below).

### The deadline test

`a_bus_that_accepts_and_never_answers_ends_at_the_deadline`: a listener accepts the connection and holds it for 4 s
without a byte; `Bus::connect(path, 1 s)` must return `CallError::TimedOut` after at least 1 s and less than 1.5 s.
It passes (it ends inside the window).
The existing `a_call_is_bounded_by_the_time_it_was_given_and_a_timeout_is_told_apart` and
`a_new_time_replaces_what_was_left_of_the_connections` also pass on the new code.

### The existing D-Bus tests against the new code

Result: 18 passed, 0 failed, 1 ignored (`the_real_user_manager_loads_and_starts_a_runtime_only_probe_unit`, which
writes unit files to the real user manager and was not run), after these changes to the tests' fake bus, none to the
code:

- The fake bus must answer `NEGOTIATE_UNIX_FD` with `AGREE_UNIX_FD`.
  `zbus` sends `AUTH EXTERNAL`, `NEGOTIATE_UNIX_FD` and `BEGIN` pipelined, and its handshake has no public option to
  skip the negotiation (a search of `connection/builder.rs` found none).
- The fake `NameAcquired` signal had only a member field; `zbus` refuses a signal without path and interface
  ("A required field is missing from message headers"), as a real bus never sends one.
  The hand client skipped any signal.
- The fake bus sent `NameAcquired` before the reply to `Hello`; `zbus` reports `Unexpected message Signal` and the
  exchange fails as an authentication error. A real bus replies first.
- The fake bus must echo the serial of each call: `zbus` counts serials for the whole process, so a test cannot
  assume 1, 2, 3 (it hung until the deadline otherwise).
- Assertions that compare the exact bytes of each call with the hand encoder became checks of the member names, and
  a missing socket needed an error mapping (`zbus` reports it as `failed to connect to ...` in a general failure).

The wire-format controls (`damaged_replies_are_refused`, `the_reader_is_total_over_damaged_input`, the layout tests)
still pass, but they test `systemd_dbus_wire_format.rs`, which the new `Bus` no longer calls;
they prove nothing about the code in use, and would go with that file.

### What the `dbus_reply` fuzz target becomes

It calls `decode_reply` on arbitrary bytes and asserts totality and bounded names and texts
(`package/git-policy/cli.fuzz/src/dbus_reply.rs`).
With `zbus` there is no such function.
Two replacements, neither built:
a target on `zbus::Message::from_bytes` (an `unsafe fn` whose doc says "bytes may have an invalid encoding",
`src/message/mod.rs` line 143) followed by the header accessors and the three body shapes the client reads
(string, object path, variant holding a string), with the same totality assertion;
or a socket-level target in which a fake bus finishes a valid handshake and then writes the input, asserting that
`Bus::call` returns inside the deadline without a panic.
The first checks `zbus`'s parser, which the owner now relies on; the second checks the wrapper's use of it, including
the deadline.

### Measured inside the wrapper

Three release builds of the same prototype, differing in the runtime flavour, against today's binary rebuilt on the
same toolchain.

- Lockfile: 40 registry packages added and 5 removed against today's lock
  (removed: `ureq`, `ureq-proto`, `utf8-zero`, `base64`, and an older `bitflags`).
  Added: `tokio`, `mio`, `socket2`, `slab`, `hyper`, `hyper-util`, `hyper-rustls`, `tokio-rustls`, `http-body`,
  `http-body-util`, `tower-service`, `want`, `try-lock`, `atomic-waker`, `zbus`, `zbus_macros`, `zbus_utils`,
  `zcheapstr`, `zruntime`, `endi`, `winnow`, `hex`, `ordered-stream`, `async-trait`, `uuid`, `serde_repr`,
  `signal-hook-registry`, `futures-*`, and, for platforms the owner said to ignore, `js-sys`, four `wasm-bindgen-*`
  and `uds_windows`.
- Binary size: today 16,786,200 bytes;
  current-thread 18,023,096 (+1,236,896, +7.4%);
  multi-thread with default workers 18,134,168 (+1,347,968, +8.0%);
  multi-thread with 2 workers 18,134,080.
  Dropping `ureq` is already counted.
- Threads, read from `/proc/self/status` inside the binary around one real call:
  D-Bus `GetUnit` plus `ActiveState` of `dbus.socket` on this host's session bus (read only), 2.2 ms with the
  current-thread runtime: 1 thread before, during and 300 ms after dropping the `Bus`;
  multi-thread default: 29 threads after the first call, still 29 after the `Bus` is dropped (the runtime is a process
  static and is never dropped); multi-thread with 2 workers: 3.
  The HTTP call to a loopback server by address: 1, 29 and 3 the same way.
  By host name (`localhost`), the current-thread runtime shows 2 threads after the call, because
  `hyper-util`'s resolver runs `getaddrinfo` on tokio's blocking pool; that thread stays idle for tokio's default
  keep-alive (10 s) and was seen at 300 ms.
- A command that never touches D-Bus or HTTPS does not pay for the runtime:
  `strace -f -e trace=clone,clone3,execve` of `rev-parse HEAD` shows only the `execve` of Git for today's binary and for
  the multi-thread build, and no `clone` or `clone3`.
- Warm `rev-parse HEAD`, 300 runs each in rotating order, median: today 3.87 ms, current-thread 3.87, multi-thread
  3.85, multi-thread with 2 workers 3.78, same-binary control (a byte copy of today's) 3.80;
  p95 4.39, 4.34, 4.42, 4.28 and 4.34 ms.
  The spread of the control against today's binary is 0.07 ms, so none of the differences is a signal.
- Cold start, major page faults of `rev-parse HEAD` after dropping the binary's cached pages, five runs each:
  today 5, current-thread 5, multi-thread 7, multi-thread with 2 workers 7 (one run 9).
  Minor faults: about 480 for today, 487 to 497 for current-thread, 491 to 500 for multi-thread,
  467 to 498 with 2 workers.
  Fault counts moved by 2 for a change that adds no code to the start path in an earlier test of this study,
  so a difference of 2 is within what layout alone does.
- The HTTP call against a loopback server took about 23 to 25 ms in all flavours.
  The same measurement on today's `ureq` build was not made.

### Recommendation on the runtime

Current-thread.
Reasons:
no thread at all in the wrapper's process for D-Bus or for an HTTP call by address,
the smallest binary of the three (about 111 KB under the multi-thread builds),
the same cold major faults as today, and the same warm median;
the multi-thread flavour leaves 29 threads (3 with two workers) alive for the rest of the process after the first
use, and the exchanges are sequential request and answer, where workers help nothing.
One property to design around: with a current-thread runtime, `zbus` connection tasks (reading incoming messages) run
only while a thread is inside `block_on`, so a `Bus` kept between calls reads nothing in the background and a signal
that arrives between calls is read at the next call (the calls of the wrapper do not need signals).
If the owner later wants concurrent exchanges, multi-thread with a small fixed `worker_threads` is the step up.
The name resolution thread by host name stays in either flavour unless the resolver is replaced.

### The standards-review call on `hyper`

- Code: `standards_review_client.rs` rewritten on `hyper` 1.12 with `hyper-util`'s legacy client,
  `hyper-rustls` 0.27.10 (features `http1`, `ring`, `webpki-tokio`, `tls12`; no `rustls-native-certs`, so nothing is
  read from the system), `http-body-util`, `rustls` with the `ring` provider, and `webpki-roots` for the embedded roots;
  `trusted_pem` still replaces them, parsed with `rustls`'s PEM reader.
  The catalog entries exist (`rust-catalog.toml` lines 70 to 73, 112); `rustls`, `webpki-roots`, `bytes` and
  `tower-service` are not in it and were added as literals in the throwaway manifest.
- Deadline and errors: the ceiling is `tokio::time::timeout(ceiling, ...)` and ends with the same sentence as before.
  The quiet window is a watchdog: a wrapper around the connection (`Marking`) records every read or write that
  completes, and the call ends with "The operation was aborted due to timeout" when nothing has moved for the window,
  connect and handshake included.
  `ureq` set one timeout per phase and per read or write; the watchdog is one clock over "no byte moved",
  which is the rule the module's doc states, and it also covers the time before a connection exists.
  No redirect is followed (the legacy client follows none), no proxy variable is read, plain `http` is still accepted as
  `ureq` accepted it, and the error text of a failed call now carries the whole cause chain.
- Tests: the 9 existing client tests pass (loopback mock: request line, headers and body, error status as a reply,
  chunked body, redirect not followed, silent server ends at the quiet window, refused connection, unreadable authority,
  reply that keeps arriving is never cut off and is cut at the ceiling).
  The mock is plain `http`, so none of them reaches TLS;
  one added control starts a local TLS server with a throwaway CA and a leaf signed by it:
  the trusted authority is reached and the request carries the bearer header, and the same server is refused with the
  embedded roots.
  (A first attempt with a self-signed certificate marked as a CA failed with `CaUsedAsEndEntity`, which is a rule of
  the verifier and a fixture error, not a client defect.)
- The structural guard `source_guard/standards_review_boundary_tests.rs` looks for the literal `ureq` in the client
  file: `the_scan_sees_the_client_and_flags_a_planted_import` fails until its literal becomes the new library names.
  A second guard test failed only because the prototype's measurement probe in `main.rs` names the client module,
  which the guard forbids; the probe is not part of any implementation.
- Selection run (`systemd_dbus`, `standards_review`, `source_guard`): 123 passed, 3 failed, 1 ignored on the prototype;
  the two guard failures above, and `the_runner_waits_for_a_slow_job_and_contains_a_panic`
  (`Busy` instead of `Finished(7)`).
  The same selection on today's unchanged code in a second throwaway worktree: 123 passed, 1 failed, the same runner
  test, with the same assertion.
  It passes alone (run twice on the prototype).
  So it is an ordering or shared-state defect of the existing suite, not an effect of this change.
- Not done: `ureq`'s per-phase timeouts were not reproduced one by one; the HTTP call was not measured against `ureq`;
  `hyper`'s `Client` pool was not studied (a client is built per call here, as the `ureq` agent was).

### Summary of what holds and what does not

- A deadline over the whole exchange, connection and handshake included: holds, with a test that ends inside the window.
- No thread left behind: holds with the current-thread runtime for D-Bus and for HTTP by address (1 thread throughout);
  does not hold for the multi-thread runtime (29 or 3 stay), and an HTTP call by host name leaves one blocking-pool
  thread for about 10 s.
- Startup: holds. No clone before the `execve` of Git, warm median equal within 0.07 ms of the control.
- Size: +1.24 MB (+7.4%) net of dropping `ureq`, current-thread.
- Does not hold: the repository's pinned toolchain `nightly-2026-11-01` cannot compile `zbus` 6.
- Costs of the move that are not requirements: the fake-bus tests needed five changes because `zbus` is stricter than
  the hand reader; the hand wire-format file and its fuzz target lose their purpose; 35 more packages in the lockfile
  net of `ureq`'s.

## Cross-vendor review of this document

Reviewed by `openai/gpt-6.1-sol` (no tools, reading only this document, run 2026-11-07, output in
`${HOME}/temp/agent/published-crates/sol-review.txt`).
It is a different vendor from the author.
Findings taken (the text was changed): input semantics are not freed by the output ruling; timestamp ordering is
precautionary; startup and digest figures are not commit-path figures; the five-second sentence is not an allowance;
the shell-quoting conclusion was too broad (a further crate, `shell-quote`, was then tested and passes);
the sock-diag version was checked at 0.5.0 for both counters; the journal sockets were read, not only searched;
the `rayon` claim is limited to the global pool; the `bindgen` point is a pending build-policy question;
version planning, work items and audit rotation are unresolved rather than "no candidate";
the `fail` callback is noted; unsafe counts, retained line totals, the library count, the combined SHA delta (8,
measured), the number of shell crates, the test summary line and the standalone-probe qualifier were corrected;
rankings were restated with their conditions; omitted crates (`nix`, `duct`, `subprocess`, `command-group`, `chrono`,
`shell-quote`) were added.
Findings not taken as written: none rejected;
the request to probe a full journal queue and to run the tempfile publish combinations was not done and is listed as
not done.

### 11. How to get past the toolchain failure of `zbus`

The owner chose `zbus` 6 on `tokio`; the pinned `nightly-2026-11-01` cannot compile `zbus` 5 or 6
(section "Tokio as the runtime").
Options:
(a) pin the wrapper to an older nightly that compiles it (`nightly-2026-10-28` was used here), with a `PIN` comment
and the removal condition "the pinned toolchain compiles `zbus`";
(b) build the wrapper on stable 1.99.0 (compiles `zbus` 6 in a scratch crate; the whole wrapper was not tried);
(c) wait for a compiler or `zbus` fix and keep the hand client until then;
(d) report it upstream first and decide after the answer.

Pros and cons:
(a) unblocks at once, but the wrapper then differs from the repository's toolchain, which other packages share, and
may meet the sibling `zbus` users' same problem.
(b) removes the nightly dependence for this package but changes how it is built and tested.
(c) costs nothing now and delays the move.
(d) is cheap and does not unblock by itself.

Ranking: (d) > (c) > (a) > (b), as a plan of order rather than exclusive options:
(d) first because it costs one report and tells whether a fix is near; (c) next because it keeps the working client
meanwhile; (a) before (b) because a pinned older nightly changes one setting, where stable changes the build of the
whole package.
Which of them the owner takes is the owner's; nothing here was built on the pinned toolchain.

## Queries

Frozen round of crates.io searches on 2026-11-07 (sort by relevance, 12 results per page, first page only;
a provider cap was not reached but pagination to saturation was not done).
The literal queries and result totals are in `${HOME}/temp/agent/published-crates/searches.txt`.
Totals found: "inactivity timeout child process" 72, "idle timeout subprocess" 148, "stall detection process" 494,
"process no progress timeout" 1738, "kill process tree" 1083, "package.json dependent version bump" 186,
"monorepo version bump dependents" 180, "btrfs subvolume" 56, "btrfs snapshot create ioctl" 7, "renameat2 exchange" 5,
"atomic file write" 4986, "three-way merge text" 105, "diff3 merge" 24, "word-level merge" 86,
"systemd dbus StartUnit" 0, "journald native protocol" 18, "sock_diag netlink tcp" 6, "mountinfo parse" 9,
"shell quote ansi-c" 18, "RFC 3339 timestamp millis" 16, "crash point fault injection" 156, "failpoint kill" 3.
Crates the review of this document named that were checked only by metadata and the lockfile delta, not run:
`nix` 0.31.3 (1 added package; used by `atomic-write-file`, `fs_at`, `kill_tree`), `duct` 1.1.2 (7),
`subprocess` 1.2.3 (1), `command-group` 5.0.1 (last release 2023-11-18, superseded by `process-wrap`) and `chrono`
0.4.45 (20).
None of these has a no-progress deadline in its description, and `chrono` is dominated by `jiff` for the time job on
the wrapper (already linked); a closer look at `duct` and `subprocess` for output capture is not done.
The registry ranks by text match and returns many generated or agent products,
so a first page is a weak sample;
for each job the decisive candidates were the ones the brief or the repository named,
and the sample only adds the crates listed above.

## Limits

- The tokio prototype: every number is from `nightly-2026-10-28`; the repository's pinned nightly cannot build it.
  The comparisons against `ureq` are for size and threads, not for time (the `ureq` HTTP call was not timed).
  The suite run covered `systemd_dbus`, `standards_review` and `source_guard`, not the whole package; the real-manager
  D-Bus test was not run; the fuzz target and the TLS path against a real host were not run.
  Both throwaway worktrees were removed at the end.

- Cold start wall times do not agree across runs.
  Run 1 (n=60): baseline 33.96 ms, control 33.73, zbus-linked 33.60.
  Run 2 (n=60): 21.51, 21.58, SHA variant 22.55, zbus-linked 23.46.
  Runs 3 and 4 (n=100 each): baseline 33.87 and 33.86, control 33.91 and 34.09, SHA variant 31.01 and 31.01,
  zbus-linked 38.84 and 38.78.
  A run with three physical copies of each binary under heavier host load (medians 56 to 81 ms, p95 up to 690 ms)
  is discarded: identical copies of the baseline differed by 4 ms, more than the effect sought.
  Within a run the control differed by at most 0.2 ms,
  but the sign of the SHA variant's difference flipped between sessions, so no cold wall-time claim is made.
  The stable cold metric is major page faults: 5, 7 and 9.
  The owner's sentence is "commits vs native git should not take 5s longer";
  the effects measured here are milliseconds,
  and no allowance below that figure is read into the sentence.
- The SHA tests ran with `cargo test --lib` (debug profile) and name filters, not the package's `mise` tasks;
  the full suite of the package was not run for any candidate.
- No candidate's own CI, fuzz or mutation suites were run.
- The sysinfo, `rustix`, `procfs`, merge, time, shell and SHA probes ran on one host (Linux x86-64, btrfs scratch).
  The SHA-NI speed-up depends on the CPU;
  `cpufeatures` falls back to portable code elsewhere, which was not measured.
- Lockfile counts include packages for platforms the owner said to ignore;
  `zbus` includes `js-sys`, `wasm-bindgen` and `uds_windows`, and `process-wrap`, `reflink-copy` and `listeners` add
  mostly `windows-*` packages.
- `cargo add` of a crate resolves its optional dependencies according to default features;
  rows say when `--no-default-features` was used.
- Scores and a weighted ranking were not computed;
  the orderings are reasoned and are not a scored recommendation.
- A smoke run of the wrapper against the worktree's shared Git directory printed migration notices
  and wrote no file (the Git directory's newest files kept their earlier times);
  later runs used a scratch repository.
