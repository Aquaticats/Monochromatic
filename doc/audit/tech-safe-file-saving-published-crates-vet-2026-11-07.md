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
  `tempfile::persist_noclobber` tries `renameat_with(NOREPLACE)`, ignores `EINVAL` and `ENOSYS`, then `hard_link`, and returns the
  link error (`src/file/imp/unix.rs`, function `persist`, lines 94 to 133 of 3.27.0).
  The incumbent's test `a_refused_link_falls_back_to_the_rename` pins the fallback.
  The facts the coordinator asked for are in the section "Facts for the owner's answer on G1" below.
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

## Facts for the owner's answer on G1

Written 2026-11-07T08:20:00-05:00.
The owner's two rules in conflict: "Commits should never fail" (choice 10: "Strong: what Git completes, we complete") and
"safe-file-saving shouldn't be something we own at all" (choice 5).
This section gives facts only; the incumbent's checked rename stays in `foundation/private_storage.rs` until the owner answers.

### Which Linux filesystems lack both

- Kernel source read: `torvalds/linux` at `d48723711bae9b8cdc56719c448ba42fef28cbee` (master on 2026-11-07), fetched to
  `${HOME}/temp/agent/tokio/kernel/`.
- The VFS makes the `RENAME_NOREPLACE` existence check itself (`fs/namei.c` line 4110, `-EEXIST` when the target exists); a
  filesystem only has to accept the flag in its `.rename`.
- exFAT accepts it: `fs/exfat/namei.c` line 1283, `if (flags & ~RENAME_NOREPLACE) return -EINVAL;` (so the flag passes).
  exFAT has no hard links: its `exfat_dir_inode_operations` (lines 1342 to 1352) has no `.link`.
- FAT (`vfat`) has no `.link` either (`fs/fat/namei_vfat.c` lines 1190 to 1201), and its `.rename` is `vfat_rename2`;
  the `rename(2)` manual page of this host says "Support for many other filesystems was added in Linux 4.9, including ext2, minix,
  reiserfs, jfs, vfat" (`/usr/share/man/man2/rename.2.gz`).
- NFS rejects every flag (`fs/nfs/dir.c` lines 2781 and 2782, `if (flags) return -EINVAL;`) and has hard links (`nfs_link`, line 2702).
- ntfs3 has `.link` (`fs/ntfs3/namei.c` line 537).
- FUSE passes the flag only when the server implements `FUSE_RENAME2`, otherwise `-EINVAL` (`fs/fuse/dir.c` lines 1310 to 1337), and
  returns `-ENOSYS` from `link` when the server has none (`fuse_link`, lines 1344 to 1353).
- So no in-kernel filesystem read lacks both.
  The filesystems that lack both are FUSE servers that implement neither.
  Source evidence of one: `s3fs-fuse` at `015af0dd73c89880380e6fcbf8997eb6c40cc664`, `src/s3fs.cpp`:
  `s3fs_rename` rejects every flag with `EINVAL` (lines 2016 to 2030, the comment says S3 cannot honor `RENAME_NOREPLACE`) and
  `s3fs_link` returns `-ENOTSUP` (lines 2098 to 2108).
  `sshfs` at `9b68e5f1b978c1d3c6ef7632b68d580a52ec0232` honors `RENAME_NOREPLACE` (`sshfs.c` line 2719), so it is not one.
- Measured as the user: `fuse2fs` mounting an ext4 image under `${HOME}/temp/agent/tokio/fsm/` (a FUSE server of libfuse):
  `renameat2(NOREPLACE)` onto a free name gave `EINVAL`, onto a taken name `EEXIST`, and `link` succeeded; tmpfs (control) gave 0,
  `EEXIST` and success.
  That is the case `tempfile` handles (it falls through to the hard link), not G1.
  A server that lacks both could not be mounted here (`ntfs-3g` refuses an unprivileged mount; vfat and exFAT need a privileged mount,
  and their in-kernel code supports the flag anyway), so G1 itself was emulated (next section), not mounted.

### What a person sees with `tempfile` only

Emulation: a scratch control (patch kept at `${HOME}/temp/agent/tokio/g1-scratch-private-storage.patch`, test at
`${HOME}/temp/agent/tokio/g1-scratch-test.rs`, both reverted) made the link in `publish_exclusive` fail with error 95
(`ENOTSUP`, what `s3fs` returns) for paths under one repository, and in the second arm also removed the fallback, which is what
`persist_noclobber` does when `renameat2` answers `EINVAL`.
Each run is a real wrapped commit through the resident service on the built binary.
With today's fallback every run below lands.
Without it:

- A plain explicit-path commit (`git commit --message=x -- a.txt`):
  the commit lands (exit 0) and the person sees
  "Your commit is saved. Its backup push and upkeep could not be handed to the commit service: writing the work item
  push-<commit id> failed: Operation not supported (os error 95). Run `git push` when ready."
  Only the wrapper's record of work to do after the commit is lost; the push has to be run by hand.
- A partial commit while another session's merge is unfinished: the commit fails (exit 1) with
  "writing packed-refs failed: Operation not supported (os error 95)".
- The commit that concludes the merge: the commit fails (exit 1) with
  "copying a conclusion entry into the shadow failed: Operation not supported (os error 95)".
- Read from the code, not run (the same error text follows the colon):
  `push/push_failure_notice.rs` ("writing the notice failed: ..."), `push/push_retry_store.rs` ("writing the plan failed: ..."):
  wrapper records after a push, the commit has landed;
  `merge/merge_isolation_journal.rs`, `worktree/worktree_removal_rescue.rs`, `worktree_copying/worktree_copy_journal.rs`,
  `landing/service_prepare_changes_config_and_pathspec_steps.rs` (the pathspec read from standard input) and
  `landing/service_claim_watch.rs` return the error to their caller; whether each fails a command was not exercised.
- Independent of G1 and measured with `strace` injecting `EOPNOTSUPP` into the service's `link` and `linkat` calls: the service's
  journal publishes its decision file by `hard_link` with no fallback (`service/service_busy.rs` line 149), so on a state directory
  that refuses links a commit through the service already fails today:
  "The commit service could not finish this commit, so it was not made. Run the commit again; ...".
  The writers above matter when the repository is on the filesystem that refuses links and the state directory is not.

### The owner's answer on G1

Recorded 2026-11-07, in the question tool.
The question said: "Commits keep working on those disks. We own 10 lines plus a 15-line test. On those disks only, two programs
saving the same file at the same instant could overwrite one another."
The owner picked, verbatim: "Keep the fallback (Recommended)".
Effect: the replacement is built on `tempfile::persist_noclobber`, and the checked rename is kept for the one case where neither
`renameat2` with `RENAME_NOREPLACE` nor a hard link works.

### How much of our own code the fallback is

- In `publish_exclusive` (`foundation/private_storage.rs` lines 191 to 206) ten code lines are the fallback:
  the `Err(error) =>` arm with its `debug` call (six lines), the `symlink_metadata` check (three lines) and the `rename` (one line).
  The paragraph of the doc comment that explains it is four lines, and its control `a_refused_link_falls_back_to_the_rename`
  is fifteen lines (`foundation/private_storage_publish_tests.rs`).
- The fallback is weaker than `RENAME_NOREPLACE`: it checks that the name is free, then renames, and a rename onto a name taken in
  between replaces it without error; the writers of one record are one process, which the incumbent's comment relies on.

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
