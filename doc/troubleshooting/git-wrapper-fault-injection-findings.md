# Fault injection into the git wrapper (cli-git-native 0.4.2, Git 2.56.0) found five defects and one Git quirk

`package/git-policy/cli` must keep working under the conditions listed in its README section `Operating conditions`.
This note records how each condition was injected without privilege, what the wrapper did before the controls existed,
and what was changed.
The inventory of which test pins which condition is in that README section.

## Symptom

Before the controls, with one fault injected into an explicit-path commit (`git commit --message=x -- a.txt` through the
built wrapper), these outcomes were observed:

- `write` failing with `ENOSPC` at the landing record or the ref record, or a `SIGKILL` just before those writes, left
  an empty `landing-1.json`, `ref-updated.json`, `preparing.json` or `cli-git-captures/worktree-id`, and every later
  guarded command stopped with exit status 2 (`Cannot recover transaction ... cannot be read`, or
  `Capture store worktree identity is empty`) until a person removed the file.
  With `ref-updated.json`, the commit had already landed.
- After that was fixed, a `SIGKILL` between the creation of `.git/index.lock` and the write of `index-lock-1.json`
  left a bare lock that no later command may remove (`fatal: Unable to create '.git/index.lock': File exists`).
- A failed write of a lock owner record left `landing.lock.<uuid>.pending` beside the lock for good.
- `fsync` on a directory failing with `EINVAL` (procfs; Git itself does not need it) failed every commit with
  `syncing the registry parent ... Invalid argument`.
- A hook entry (`.git/cli-git-transactions/<id>/hooks/pre-commit`) started with a standard error that has no reader
  exited with status 101 (a panic of `eprintln!`) instead of 1.
- A stopped auto-push (`CLI_GIT_PUSH_DEADLINE_SECONDS=1`) reported "was stopped" and returned, while
  `git remote-http` and `git-remote-http` stayed alive, orphaned, until Git's own low-speed abort 60 seconds later; a
  `git-lfs` that never exits stayed alive for good.

## Root cause

- `write_private_file` and `write_exclusive_synced` in `src/native/private_storage.rs` opened the record with
  `O_CREAT | O_EXCL` at its final name and then wrote it.
  Their documentation promised "either no file or a complete one", which holds for a power loss only if the file
  system orders data before names, and not for a kill or a failed write between the two steps.
  Recovery reads an empty record as damaged and fails closed, which is recorded as the outcome `sticks` in
  `src/native/binary_recovery_tearing_expected.rs` for every journal file.
- `create_identified_lock` in `src/native/real_index_lock.rs` created `index.lock` by `O_EXCL` and journaled its
  identity (device and inode) afterwards, so no order of the two writes closes the window by itself.
- `write_candidate` in `src/native/owner_lock.rs` removed its candidate directory only when the publishing rename
  failed, not when the record write failed.
- `sync_directory` in `src/native/private_storage.rs` returned the error of `File::sync_all` for every errno.
- `dispatch_failure` in `src/native/hook_dispatch.rs` used `eprintln!`, which panics when the write fails with anything
  but `EBADF`.
- `stop_push` in `src/native/bounded_push.rs` sent `SIGTERM` and then `SIGKILL` to the `git push` process only.
  Its comment said Git's own signal handling ends the helpers; the measurement below shows it does not.

## Verification

Method: `strace --inject=<call>:error=<ERRNO>:when=<n>` (also `signal=KILL`, and `delay_exit=<time>` for a slow
flush) on the wrapper alone, so the `n`th call is the same call on every run; `n` runs from 1 to the number of calls
an uninjected commit makes (24 `write`, 33 `fsync`, 9 `rename`, 27 `mkdir`, 199 `openat`, 72 `pipe2` on this build).
With `--follow-forks` the counter is per process, so the sweep reaches as far as the busiest process.
Resource limits (`RLIMIT_NOFILE`, `RLIMIT_FSIZE` with `SIGXFSZ` ignored) are set between `fork` and `exec`.
Everything is in `package/git-policy/cli/src/native/binary_fault_*_tests.rs`.

Failing first (host, `mise run :native:test:binary:host -- --no-fail-fast fault_`, commit da03b11dfb):

- `fault_enospc::a_full_disk_at_any_write_of_the_wrapper_leaves_a_recoverable_repository`: 3 of 24 calls, for example
  `enospc-write-19: the next command must succeed, exit Some(2): ... landing-1.json cannot be read (Transaction record
  landing record is not a schema-version-2 landing record)`.
- `fault_kill::a_kill_at_any_write_leaves_a_recoverable_repository`: 5 of 24 calls (preparing, index lock, landing,
  ref-updated records and `worktree-id`).
- `fault_stream::a_hook_entry_that_cannot_report_still_exits_with_its_status`: `left: Some(101) right: Some(1)`.
- `fault_remote::an_auto_push_with_a_git_lfs_that_never_exits_is_stopped_with_everything_it_started`: "a process the
  push started was still running after the command returned"; the silent-server control passed only after 60 seconds.
- Unit controls (commit 1162fab840): `private_storage::tests::a_write_cut_half_way_leaves_nothing_behind`,
  `owner_lock::tests::a_lock_candidate_whose_record_fails_to_write_is_removed`,
  `private_storage::tests::a_directory_that_cannot_be_synced_is_not_an_error`.
- After the records were published whole, `fault_kill::a_kill_at_any_write...` and `a_kill_at_any_sync...` still failed
  on `git failed: fatal: Unable to create '.../index.lock': File exists.` (the bare lock above).

After the fixes every control passes (the full unit and binary suites pass on the host except
`markdown_linter::tests::the_real_linter_meets_the_contract` and
`markdown::the_real_linter_rewrites_through_add_check_and_fix`, which fail the same way without these changes).

Observed Git behavior behind the last defect, from a scratch run of a stopped auto-push to a local server that accepts
and never answers: the wrapper returned after about 2 seconds, and `ps` then showed `git remote-http origin ...` and
`git-remote-http origin ...` with elapsed time 1 second and the init process as parent.
After the fix the descriptor tripwire of `fault_remote` (a pipe end every descendant inherits, polled for hang-up)
reports that none is left, and the same control takes 2 seconds instead of 60.

## Verified workarounds (the changes)

- Records are written to a hidden `.<name>.<uuid>.partial` file, synced, and published by hard link (by rename after a
  check, where the filesystem refuses links), then the staging name is removed.
  Tradeoff: two more calls per record (`linkat`, `unlink`), no more syncs; a kill between the link and the removal
  leaves a staging name inside a transaction directory, which recovery removes with the directory.
- The landing's index lock is made as a candidate file in the transaction directory, its identity journaled, and then
  published as `index.lock` by hard link (as exclusive as `O_EXCL`).
  Tradeoff: needs a hard link between the transaction directory and the Git directory; a filesystem that refuses it
  gets the old order, with the old window.
- A lock candidate is removed when its record cannot be written.
- `sync_directory` skips `EINVAL`, `ENOTSUP` and `ENOSYS`; `EIO`, `ENOSPC` and `EDQUOT` stay errors.
  Tradeoff: on such a filesystem an entry cannot be made durable by any means.
- `dispatch_failure` ignores a failed print.
- `stop_push` reads the processes below the push from `/proc` before signalling (`src/native/process_tree.rs`),
  asks them to end with it, and kills those still running after the grace, each only while its start time is the one read.
  Tradeoff: Linux only; elsewhere only the push is stopped.

## What does not work

- `RLIMIT_FSIZE` alone: it cannot reach `landing-1.json` or `ref-updated.json`, because `preparing.json` (892 bytes) is
  larger and is written first, so a cap that cuts the later records cuts the first one before them.
  It is kept for the half writes of Git's pack and index files.
- `RLIMIT_NPROC`: it compares against the user's processes on the whole host, so it is neither deterministic nor
  sweepable; `clone3` failing with `EAGAIN` from the `n`th start on is the control.
- A hook that never exits under `CLI_GIT_CHILD_WAIT_SECONDS=1`: the commit ran for the whole 120 second bound.
  Hooks run as long as they run, as the README documents, so this is not a defect and has no control.
- Treating a `.pending` or `.stale` entry of a killed process as a leak: recovery keeps staging candidates of a dead
  owner on purpose (`src/native/recovery_scan.rs`, "staging candidate kept"), so the controls ignore them.

## Upstream filing decision

Nothing is filed.
The five defects are in this repository.
The Git behavior (a `git push` stopped by `SIGTERM` leaves its transport helper running until the low-speed abort) was
seen once on Git 2.56.0 with a silent server, its cause in Git was not traced to source, and `.out-of-scope/` and the
upstream tracker were not searched, so constraints 1 to 6 of the filing check are not established and no draft exists.
Filing would put the poster's account in front of strangers and is the poster's decision.
