# Vet report: published crates for the wrapper's safe file saving

## Report status

- Status:
  complete for the candidates named below;
  a scoped vet (see "What this report did not do").
- Lifecycle phase:
  recommended, not adopted.
  The replacement lands in the steps of `doc/planning/git-policy-cli-full-tokio.md` where the callers become asynchronous.
- Subject:
  published crates that replace the wrapper's hand-written safe file saving
  (`package/git-policy/cli/src/native/foundation/private_storage.rs`, 733 lines, and the files beside it).
- Started and last updated:
  2026-11-07.
- Governing skill:
  `.agents/skills/choosing-technology/SKILL.md`,
  last commit `45d42d9b23d2f2752f9c512683b45905e13afe99`,
  SHA-256 `f4cfa6bc5d1d71f4b136f92ff9fbdab68575c920bd334f824d253336c8782e1b`.
- Compatibility fingerprint (SHA-256 of the canonical input; key order sorted, ASCII strings only):
  `d7c4c43f7cac7cb8361b523ed2f805c150df910f7039acf8428d207cab926e21`.
- Active audit owner:
  the agent of the branch `feat/git-policy-cli-on-tokio`.
- Prior compatible report:
  none.
  The nearest earlier work is the first-version research in
  `doc/research/wrapper-git-free-jobs-and-published-crates.md`, section "Safe file saving", which this report extends with
  measured behavior.

## The owner's words that decide this

- Choice 5 of `doc/planning/git-policy-cli-full-tokio.md`, picked 2026-11-07:
  "Everything async, and safe-file-saving shouldn't be something we own at all."
- "Replace with published crates" for time stamps, digests, Linux calls and memory headroom (same day).
- Standing: "never discard user data"; "nothing should ever wait forever or indefinitely";
  "Ignore Win and mac until I say so."

## Hard constraints

- Linux only until the owner says otherwise.
- Async on tokio, with the blocking pool as the way to reach a blocking call (`foundation/blocking_job.rs`).
- A write that started is published whole or not at all, and a taken name is never overwritten.
- Private modes 0600 (files) and 0700 (folders).
- A link is never followed when the wrapper reads state or creates a file (the current semantics are `O_NOFOLLOW`, which
  refuses a link as the last component only; the probe below shows the difference from `RESOLVE_NO_SYMLINKS`).
- Every dependency goes through `rust-catalog.toml` and its release-age and trust policy.

## Base components and overlays

- Base:
  inspectable open-source local technology (libraries).
- Overlay:
  incumbent dependency replacement
  (the incumbent is our own code, so "replace" means retiring code; the consumed responsibilities are ledgered below).
- Overlays not active:
  high-trust execution (no credentials, no network),
  native or prebuilt artifacts (pure Rust crates from crates.io),
  sensitive data (local records the wrapper itself writes),
  multi-platform (Linux only by ruling).

## Candidate ledger

The candidates come from the agent's knowledge, the owner's picks and repository precedent,
the three sources that need no check-in with the user under the skill's "Check in before broad discovery".
No registry or repository-host search was run, so the discovery protocol of the skill (frozen query schedule, saturation)
was not applied.
That is the first limit of this report.

### `tempfile` 3.27.0

- Source:
  already in the lockfile (the scanner links it); named by the owner's brief.
- Registry copy read:
  `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tempfile-3.27.0/`.
- Screening:
  MIT or Apache-2.0; pure Rust; dependencies `fastrand`, `once_cell`, `rustix` (unix), `getrandom`; zero added packages.
- Result:
  finalist.

### `rustix` 1.1.6

- Source:
  owner's pick ("Linux calls: rustix"); already linked.
- Screening:
  MIT, Apache-2.0 or Apache-2.0 with LLVM exception; pure Rust over raw system calls (the `linux_raw` backend).
- Result:
  finalist.

### `tokio::fs` (tokio 1.54.1)

- Source:
  the runtime the owner ruled for; `fs` feature is on in the manifest.
- Screening:
  MIT; runs "ordinary blocking file operations behind the scenes" on the blocking pool (`src/fs/mod.rs`).
- Result:
  finalist.

### `atomic-write-file` 0.3.1

- Source:
  the earlier research (6 added packages).
- Screening:
  it always replaces the target through `renameat` (`src/imp/unix/mod.rs`) and has no option to refuse an existing name;
  the wrapper's journals and lock candidates must never replace.
- Result:
  exits at the hard constraint "a taken name is never overwritten".

### `atomicwrites` 0.4.4

- Source:
  the earlier research.
- Screening:
  `DisallowOverwrite` is a hard link and a delete (`src/lib.rs`); same shape as `tempfile`'s fallback, older (last release
  2024-09-19), and adds `windows-sys` and its own copy of the same logic.
  It brings nothing `tempfile` lacks.
- Result:
  not promoted; no behavior beyond `tempfile` to validate.

### `cap-std` and `cap-tempfile` 4.0.3

- Source:
  the earlier research (11 added packages).
- Screening:
  directory-handle based, with its own no-follow rules that differ from the wrapper's semantics;
  it would change what a read refuses.
- Result:
  not promoted.

### Keeping our own code

- Kept in the ledger as the incumbent.
  The owner's ruling is against it ("shouldn't be something we own at all"), so it is the fallback only for a behavior that
  no crate gives, and every such behavior is listed in "What no crate gives".

## What the incumbent does (consumed responsibilities)

Read from `foundation/private_storage.rs` (lines cited are of the tree at this report).

- R1, `write_private_file`, `write_private_file_unsynced`, `write_exclusive_synced`:
  write under an exclusive staging name `.<name>.<uuid>.partial`, optionally sync the file, set mode 0600, publish by hard
  link with a checked rename when links are refused, remove the staging name, sync the directory (a filesystem that returns
  `EINVAL`, `ENOTSUP` or `ENOSYS` counts as synchronized).
- R2, `write_exclusive_scratch`:
  create-new with mode 0600 and `O_NOFOLLOW`, no sync.
- R3, `create_private_directory` and `create_private_directory_all`:
  mode 0700 on the folders it creates only.
- R4, `read_regular_file`:
  `O_NOFOLLOW` open, handle metadata must say regular file, size limit checked and enforced again by `take(limit + 1)`,
  `ENOENT` and `ENOTDIR` are "missing", `ELOOP` is "refused link".
- R5, `sync_directory`, `take_file_syncs`, `take_directory_syncs`:
  the directory sync and the recorders tests use.
- R6, `remove_tree`:
  remove a tree without following links.
- Users:
  38 production files call this module.

## Behavior measured with the published calls alone

The probe is `${HOME}/temp/agent/tokio/safe-save-probe/` (scratch, outside the repository),
run on Linux x64, `nightly-2026-11-07`, rustc `1.101.0-nightly`, tempfile 3.27.0, rustix 1.1.6, tokio 1.54.1,
`cargo run --offline` in a `systemd-run --user --scope` with `MemoryMax=6G`, `CPUQuota=400%`, `TasksMax=512`, `IOWeight=10`.
The output is `${HOME}/temp/agent/tokio/safe-save-probe.out`; all 15 lines read PASS.

- R1, publish without overwriting, mode 0600:
  `tempfile::Builder::new().prefix(..).suffix(".partial").permissions(0600).tempfile_in(dir)`,
  write, `as_file().sync_all()`, `persist_noclobber(target)`.
  A second publish onto the same name returned `AlreadyExists` and the first content stayed.
  A refused publish left no `.partial` file (`tempfile` removes the staging file when the value drops).
- R1, directory sync:
  `rustix::fs::fsync(&File::open(dir)?)` with the three tolerated errors matched at the call;
  the call is composed in the caller (about ten lines), `tempfile` does not sync.
- R2, create-new:
  `tokio::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).custom_flags(O_NOFOLLOW)` refused a taken name
  and a planted link (`O_EXCL` refuses a link as the last component and creates nothing through it).
- R3:
  `tokio::fs::DirBuilder::new().recursive(true).mode(0o700)` gave 0700 on every folder it created.
- R4, read:
  `tokio::fs::OpenOptions` with `custom_flags(O_NOFOLLOW)` refused a planted link as the last component and followed a link in
  the middle of the path, which is exactly the incumbent's behavior;
  the handle's `metadata()` refused a folder; the size limit and the `take(limit + 1)` re-check work through
  `tokio::io::AsyncReadExt`.
- R4, the stricter call:
  `rustix::fs::openat2(.., ResolveFlags::NO_SYMLINKS)` refused even the plain file here, because an ancestor of the scratch
  path is a link (`${HOME}/temp/agent` points at another disk).
  Adopting it would change what the wrapper refuses and would fail on any host whose home folder sits behind a link,
  so it is not adopted for R4.
- R6:
  `tokio::fs::remove_dir_all` (the standard library's hardened removal) removed a link inside the tree and left the folder it
  pointed at.

## What no crate gives

Each item names the evidence and what the replacement does instead.

- G1, the fallback to a checked rename on a filesystem that supports neither `renameat2` with `RENAME_NOREPLACE` nor hard links.
  `tempfile::persist_noclobber` tries `renameat_with(NOREPLACE)`, then `hard_link`, and returns the link error
  (`src/file/imp/unix.rs`, function `persist`, lines 94 to 133 of 3.27.0).
  The incumbent's test `a_refused_link_falls_back_to_the_rename` pins the fallback.
  Not triggered here: no such filesystem was available (a FAT or some FUSE mounts would be).
  Effect of dropping it:
  on such a filesystem the publish fails with the operating system's error and the staged file is returned inside
  `PersistError` (nothing is lost, the caller reports the path), where the incumbent publishes.
  Open for the owner: accept, or keep a checked rename beside the call.
- G2, killed between the staging file and the publish:
  a `SIGKILL` leaves `.<prefix>.<random>.partial` (the incumbent does the same, and recovery ignores such names).
  No crate removes it; `tempfile` deletes on drop only.
- G3, the order file sync, link, directory sync as one call:
  no crate offers it; the caller composes three calls.
  This is glue of about fifteen lines in one helper, not a library of ours, and it stays in the program next to the
  asynchronous call (`foundation/blocking_job.rs` runs the composition on the blocking pool).
- G4, the recorders of file and directory syncs that tests read (`take_file_syncs`, `take_directory_syncs`):
  test seams, retired with the module; the budget tests that count sync calls use `strace` instead
  (`binary/binary_budget_tests.rs`).
- G5, asynchronous `tempfile`:
  `tempfile` is synchronous.
  The composition in G3 runs on the blocking pool through `run_blocking_job`, which returns "the system refused a thread" as a
  plain error (the owner's choice 11).

## Parity ledger (RCO)

- R1 owner:
  `tempfile` + `rustix::fs::fsync` + `tokio` blocking pool;
  parity test: `private_storage_publish_tests.rs` ported to the new helper, except G1;
  retired behavior: the link-then-rename order and G1.
- R2 owner:
  `tokio::fs::OpenOptions`;
  parity test: the create-new and planted-link controls of the probe, ported into the crate.
- R3 owner:
  `tokio::fs::DirBuilder`; parity test: the mode control of the probe.
- R4 owner:
  `tokio::fs::OpenOptions` + `O_NOFOLLOW` (`rustix::fs::OFlags::NOFOLLOW`); parity test: `ReadRefusal` cases of
  `private_storage_tests.rs`.
- R5 owner:
  `rustix::fs::fsync` with the tolerated errors at the call; the recorders retire.
- R6 owner:
  `tokio::fs::remove_dir_all`.
- Removal of `private_storage.rs` is recommended only after every row has a ported parity test green.

## Scoring

Not applicable.
The candidates are not alternatives to each other: `tempfile`, `rustix` and `tokio::fs` cover different rows of the ledger and
all three are needed.
The one real alternative for R1 (`atomic-write-file`) fails a hard constraint.
No soft criterion orders the finalists, so no score is reported.

## Ranking

Not a ranking of alternatives; the composition is `tempfile` for R1 publication, `tokio::fs` for R2 to R4 and R6,
`rustix` for the directory sync and the constants.
`atomic-write-file` exits on the replace-only rule; `atomicwrites` and `cap-std` were not promoted (reasons above).

## Recommendation

Adopt the composition above, subject to the owner's answer on G1.
This is a recommendation, not an adoption: no product code changed in this report's commit, and no decision record was written.

## What this report did not do

- No registry or repository-host discovery with a frozen query schedule; no saturation claim.
- No clone of `tempfile`, `rustix` or `tokio`; their registry copies were read, which is source, but not a repository-history,
  issue-tracker or CI audit (the maintenance audit of the skill was not run).
- No upstream test suites were run (`tempfile`, `rustix`, `tokio` suites).
  The consumer boundary was exercised by the probe only; it is not the wrapper's own test suite on the new code.
- No fuzzing or mutation search of the candidates.
- The probe ran on one filesystem (the scratch disk); a filesystem that refuses hard links, one that cannot sync a directory,
  and a full disk were not exercised.
