# cli-git: fewer processes and fewer syncs per commit, and retry on process exhaustion

Status: accepted for implementation by the user (2026-10-29), scope below.
Owner package: `package/git-policy/cli`. Tracking issue: 1804 (follow-up work found while stabilizing it).

## Why

A wrapped `git commit` of one explicit path starts about 45 real Git processes and makes about 41 sync calls,
against one process and no syncs for direct Git. Measured on an idle host with the debug and release builds alike:

- wall time about 180 ms per commit, of which about 77 ms is inside sync calls (strace, same fixture);
- the container gate takes 12 to 32 seconds for the commit tests under host load, and the same suite takes about 67
  seconds when the host is quiet;
- a host-wide fork-failure episode (the user cgroup recorded 598 limit events) failed commits with
  `Resource temporarily unavailable (os error 11)`.

Every process start and every sync is multiplied by load, so the fix is to do less of both and to ride out a short
refusal to start a process.

## Decisions (user)

- Process-exhaustion safeguard: retry failed starts everywhere, and also retry read-only and shadow-repository Git
  commands that fail inside themselves (`unable to create thread`, `cannot fork`), with an allow-list.
- Spawn reduction: save as many processes as possible, engineering the replacement in the wrapper where the result is
  provably identical to Git's, in the manner of `package/forbidden-strings/core` (own engine, an
  `OPTIMIZATIONS.md`-style log of measured wins and rejections, oracle-checked parity).
- Journal syncs: option B of the earlier question (sync at the barriers before side effects), after a crash
  harness exists that tears, drops and reorders records.

## Method

- Parity first: every in-process replacement is tested against real Git (`git` is the oracle) over generated cases,
  and falls back to spawning Git for any shape it does not handle exactly. A fallback is never a failure.
- Every replacement shows its win with the same scripts: `GIT_TRACE2_PERF` process counts and `strace` sync counts.
  A permanent budget test per measure ratchets down after each slice (a regression fails the host and gate suites).
- A guard test is shown failing before the change is kept (the repo's rule).
- What is measured and rejected is recorded here, with the number or the counterexample.

## Process-start retry (slice R)

A start that fails with `EAGAIN` (`ErrorKind::WouldBlock`) started nothing, so retrying is always safe.

- One helper starts every child: 10 ms backoff doubling to 500 ms, a 5 s total budget, one warning per retry, then the
  existing error plus the attempt count. It covers the roughly 20 call sites (`run_git`, bounded children,
  forwarding including the `execve` path, hooks, signal relay, post-commit, probes).
- Git commands that start and then fail inside themselves are retried only from an allow-list of commands that are
  harmless to re-run (read-only queries, and the idempotent shadow-repository steps), matched on the exit status and
  the wording of the failure; mutating steps (reference updates, landing) are not retried, and recovery already
  completes or rolls back those.
- Tests: a unit test with a start that refuses K times; the fork-failure sweep gains burst cases
  (`--inject=clone3:error=EAGAIN:when=N..N+K`), which must now succeed for bursts shorter than the budget.

## Candidate replacements, with the exactness condition for each

Measured process lifetimes of one commit (strace): `findmnt` 3 runs about 25 ms, `rev-parse` 12 runs about 32 ms,
`config` 7 runs about 16 ms, `git hook` 4 runs about 14 ms plus the wrapper's own hook shim re-executions.

- Hook shims for events with no hook: the shadow commit points `core.hooksPath` at shims that re-enter the wrapper,
  which takes the hook lock (a synced lock candidate) and runs `git hook run --ignore-missing <event>`. An event with
  no user hook needs no shim. Exact when conservative: the shim is omitted only if the user's hooks directory has no
  entry of that name at all (any mode, because Git prints a hint for a non-executable file) and the configuration has
  no `hook.<name>.event` or `.command` key; any other shape keeps today's behavior. Saves up to four wrapper
  re-executions, four `git hook` runs and four synced lock candidates.
- `findmnt` (3 runs): the filesystem identity of `.git` and `index.lock`. To be replaced by `statx`, `/proc/self/mountinfo`
  and `/dev/disk/by-uuid` when the mapping is unambiguous; otherwise `findmnt` stays. Needs a check of how the
  identity is consumed before choosing (recorded below when read).
- One configuration snapshot (`git config --list -z --show-scope`) in place of the separate `config` runs
  (`core.hooksPath`, the four `hook.<event>.enabled` reads already batched, the maintenance keys, ref storage).
  Last value wins, `-c` and `GIT_CONFIG_*` values come last, bool canonicalized by Git's rules; `--type=path` values
  starting with `~` or `%(prefix)` fall back to a spawn.
- Empty-tree object name: the hash of `tree 0` under the repository's object format (a constant per format) in place of
  `git hash-object -t tree --stdin`.
- `symbolic-ref HEAD` (files ref storage): read `HEAD`; anything but `ref: <name>` or a full object name falls back.
  The landing re-read stays a read of the same file, so the compare-and-swap protection is unchanged.
- Sequencer markers (`MERGE_HEAD`, `CHERRY_PICK_HEAD`, `REVERT_HEAD`) on the files backend: when none of the
  loose files (including the six lookup spellings `rev-parse` tries) exists and `packed-refs` does not contain the
  name, they are absent without a spawn; any hit falls back to `rev-parse --verify`. A dangling marker therefore
  stays a conclusion (pinned by `a_marker_naming_a_missing_object_is_still_a_conclusion`).
- `git version`: cached per binary identity (path, size, mtime, inode) in the repository's private directory.

## Rejected, with evidence

- Reading the three sequencer markers with `cat-file --batch-check`: it reports a marker that names a missing object
  as missing, so a corrupt merge would become a plain commit that drops the merge parent.
- Merging `core.hooksPath` with the maintenance reads by hand: different phases and a `--type=path` value; covered
  instead by the snapshot above.
- Capping Git's threads (`pack.threads=1`, `core.preloadindex=false`): 101 ms against 105 and 100 ms baseline, and
  the container was never throttled (`nr_throttled 0`).
- Re-reading `HEAD` and the target ref once instead of twice: the second reads at landing are the compare-and-swap
  protection.

## Journal syncs (slice J, task 69)

Order from the independent review: skip directory syncs where nothing changed (three `ensure_*` syncs), skip the two
syncs after removing a landed transaction directory, defer directory syncs to barriers, add torn-record tolerance for
pre-landing records with a tearing, dropping and reordering harness (a kill is not a power loss), and only then drop
file syncs of pre-landing records. Lock-candidate syncs stay: a torn blocking `owner.json` would stick every
acquirer. The sync-site table is in `doc/handover/cli-git-rust-implementation.md`.

### J1 outcome: the `ensure_*` syncs were already conditional

The three `ensure_*` syncs in the handover table (`.git` twice, `cli-git-captures` once) are creation syncs, not
redundant ones. `ensure_transaction_root` syncs its parent only in the arm where it created the directory,
`ensure_worktree_id` only after it wrote the identity file, and `record_landed_capture` only after it wrote a new
record. The measurement ran a first commit in a fresh repository, where the registry, the capture store and the
landed-record directory really were created. A steady-state commit has none of them. No production code changed.

Measured on the debug binary (`strace -f` on `fsync` and `fdatasync`; the split is by the last `execve` of the calling
process, so a Git child counts as a child and a re-entered hook shim counts as the wrapper):

- fresh repository, `node sync-count.ts`: 42 calls, 36 by the wrapper and 6 by children. Three are `ensure_*`
  creation syncs (`.git` for the registry, `.git` for the capture store, `cli-git-captures` for the landed-record
  directory), and the new `worktree-id` file and its directory add two more;
- warm repository (one earlier wrapped commit), `node sync-count.ts warm`: 41 calls, 31 by the wrapper and 10 by
  children, none from an `ensure_*` helper. Per-call attribution comes from `node sync-sites.ts` (cold) and
  `WARM=1 node sync-sites.ts` (warm), which print the calling function of each sync in call order.

The deliverable is the pin: a test-only recorder in `sync_directory` (`take_directory_syncs` in
`src/native/private_storage.rs`, per thread) and unit tests that assert the exact directories synced when a helper
creates and the empty list when it finds. With the three guards broken (sync also on the already-exists and
already-written paths) all six tests fail; they pass restored.

### J2 outcome: one of the two post-removal syncs dropped

`remove_transaction_directory` (called by `dispose_landed_transaction` and by recovery) syncs the registry twice:
after renaming the directory to `<id>.retired`, and after deleting the retired tree. The second is dropped. A
retired directory that survives a power loss is removed by the next recovery (`recover_registered_transactions`
removes every `Retired` entry), and a landed directory that is still published is classified `AlreadyInstalled` and
cleaned; `a_completed_landing_that_is_still_published_is_cleaned_by_recovery` runs both together and fails when the
retired-entry removal is disabled.

The first is kept, and this departs from the instruction to drop both. It is not a durability sync of a removal but
an ordering barrier: the rename must reach the disk before the deletions inside the directory do. Without it a
filesystem that persists the unlinks but not the rename leaves the published name `<id>` with `owner.json` gone, and a
published directory without a valid owner record is a fail-closed exit 2 that keeps its contents (see the recovery
outcome list for the slice J3 harness, case "owner record deleted"). Dropping it too saves one more sync (about 2 ms
here) and is a one-line change if the user accepts that filesystem ordering assumption.

Measured with `node sync-count.ts` and `node sync-count.ts warm` before and after: 42 to 41 cold and 41 to 40 warm
(wrapper 31 to 30 warm). The pin is `removal_syncs_the_registry_only_for_the_rename` and
`disposal_syncs_the_registry_once`; both fail with the final `sync_parent` restored. No test in the repository counts
syncs of the built binary (`strace` appears only in `binary_fork_failure_tests.rs`), so the sync budget named under
Budgets does not exist yet.

## Budgets

The host and gate suites carry a spawn budget (from `GIT_TRACE2_PERF`) and the host suite a sync budget (from
`strace`); each slice lowers its budget to the measured count. Starting point: 45 processes, 41 syncs.
