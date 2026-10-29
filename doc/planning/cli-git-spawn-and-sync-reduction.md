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

### J2 outcome: both post-removal syncs dropped

`remove_transaction_directory` (called by `dispose_landed_transaction` and by recovery) synced the registry twice:
after renaming the directory to `<id>.retired`, and after deleting the retired tree. Both are dropped; the in-place
removal of a legacy directory, which has no retired name, keeps its one sync. A retired directory that survives a
power loss is removed by the next recovery (`recover_registered_transactions` removes every `Retired` entry), and a
landed directory that is still published is classified `AlreadyInstalled` and cleaned;
`a_completed_landing_that_is_still_published_is_cleaned_by_recovery` runs both together and fails when the
retired-entry removal is disabled.

The rename sync was an ordering barrier between the rename and the deletions inside the directory, not a durability
sync of a removal. Dropping it rests on an assumption that is not sourced here: the filesystems this repository
targets (ext4, XFS and btrfs, per the handover) apply metadata operations in order, so a power loss leaves the
published name, the retired name with some contents, or nothing. If a filesystem persisted the unlinks but not the
rename, the published name `<id>` would be left without `owner.json`, and the J3 outcome below shows that this state
sticks (deleting `owner.json` fails closed at every phase, naming the directory). No such state was produced by any
test; it is the risk accepted by the decision to drop the sync.

Measured with `node sync-count.ts` and `node sync-count.ts warm` before and after: 42 to 40 cold and 41 to 39 warm
(wrapper 31 to 29 warm). The pins are `removing_a_registry_directory_syncs_nothing` and
`disposal_syncs_no_directory`; both fail with a `sync_parent` restored after the rename (and the earlier form of the
pin failed with the post-deletion `sync_parent` restored). No test in the repository counts syncs of the built binary
(`strace` appears only in `binary_fork_failure_tests.rs`), so the sync budget named under Budgets does not exist yet.

### J3 outcome: what recovery does with emptied, halved and dropped records

The harness is `src/native/binary_recovery_tearing_tests.rs` in `package/git-policy/cli`, with the recorded outcomes in
`binary_recovery_tearing_expected.rs`. A real wrapped `git commit --message=killed a.txt` is killed by
`CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill:<dir>` at each of the six phases a commit reaches. Then each regular file
the commit left is damaged on its own, and the next wrapped commands run: `git status` twice, then a `git commit`. The
files are everything under `cli-git-transactions` (journal records, index snapshots, the landing lock), everything
under `cli-git-captures` (the capture lock, `sequence`, `worktree-id`, landed records), `cli-git/hook.lock`, and the
real `index.lock` with its `index~pid.lock`. The three damages are empty (zero bytes), half (cut to half the length)
and delete (the file is gone while the records written after it stay, a flush that reached the disk out of order). A
damage that cannot change the file (emptying an empty file, halving a file under two bytes) is skipped. A kill leaves
the page cache intact, so the kill alone shows nothing; the damage step is the test, and the harness fails when the
damage step is disabled (shown) or when any recorded outcome changes.

Vocabulary of the outcomes: **completes** (exit 0, the commit is on the branch and the index matches it), **discards**
(exit 0, nothing landed and the edit is intact), **sticks** (exit 2 again on a repeat, so every guarded command fails
until someone removes the file by hand), **fails closed once** (exit 2, then a later command works; no case
did this). A suffix marks a degraded state: the following commit exits 2 although `git status` was fine, or an
`index.lock` is left behind. Run it with
`mise run //package/git-policy/cli:native:test:binary:host -- recovery_tearing` (six tests, about 15 seconds);
regenerate the record by setting `CLI_GIT_TEARING_REPORT=<file>`.

Totals over 247 cases (two runs gave identical outcomes): 92 discard, 67 complete, 63 stick, 11 discard but the next
commit exits 2, 6 complete but the next commit exits 2, 4 discard and leave `index.lock` with the next commit failing,
2 leave a corrupt real index, 2 are inconsistent without any error. No case hung, and none failed closed and then
recovered.

#### Read first: the cases that matter

- Two index snapshots are not synced today and the table shows what tearing them does. `pre-landing-1.index` and
  `post-1.index` are written by a plain copy and by Git (the sync-site table in the handover has no sync for either,
  and Git's default `core.fsync=committed,-loose-object` excludes `index`, per `git help config`). The landing record
  names them only by device and inode, so recovery trusts their content. At `ref-updated`, an emptied or halved
  `post-1.index` is installed as the real index and every later command fails with
  `fatal: .git/index: index file smaller than expected` or `fatal: unable to read <object>` (`git status` exits 128,
  the next commit exits 1); an emptied or halved `pre-landing-1.index` sticks with "Real index no longer matches
  the pre-landing snapshot". A power loss that persists `landing-1.json` and the ref but not the snapshots' data
  can produce exactly this, depending on the filesystem (ext4 in `data=ordered` mode writes the data of files
  allocated in a committing transaction first; XFS and btrfs do not promise that). This is a possible exposure in the
  current code, not only a constraint on removing syncs. The
  remedies are a sync of both snapshots before `landing-1.json`, or recovery verifying their content. Neither is done
  here.
- Records that are synced today and must stay synced, because tearing them sticks every command: `owner.json` (all
  damages at every phase), `preparing.json` (empty and half), `index-lock-1.json`
  (empty and half before the ref moves), `landing-1.json` and `ref-updated.json` (empty and half), `store-held.json`
  after the ref moved, and the `owner.json` of `landing.lock`. Their messages are fail-closed by design.
- Silent inconsistency: with `landing-1.json` or `preparing.json` missing at `ref-updated`, `git status` exits 0
  although the branch moved and the index and worktree were never reconciled (the observed outcome is neither
  completes nor discards; the likely cause is that a transaction without its landing record is treated as never
  landed). For
  `landing-1.json` the next commit then also succeeds; for `preparing.json` it exits 2. Only the sync of
  `landing-1.json` and its directory before the compare-and-swap prevents this; no later sync may replace it.
- A torn capture `sequence` or an emptied `worktree-id` leaves `git status` working while every later commit exits 2
  (the sequence is parsed only by a commit). A torn `capture.lock/owner.json` does the same. The sequence temporary
  file sync and the lock candidate syncs must stay.
- Deleted `index-lock-1.json` or `preparing.json` before the ref moves leaves the real `index.lock` in place, because
  recovery removes it only when a record names it; the next commit fails. That window also exists for a kill between
  creating `index.lock` and writing its record, so it is not caused by syncing.
- Deleting `index.lock` at `ref-updated` sticks ("opening the index lock ... No such file or directory"): the lock's
  directory entry is not synced before the compare-and-swap. This one is reachable only by losing the entry while the
  ref update persists.
- The J2 link: deleting `owner.json` sticks at every phase. That is the state a filesystem that persisted the deletions
  inside a removed transaction directory but not its rename would leave, which is the unsourced ordering assumption
  recorded under the J2 outcome.
- What the user is told: only the owner-record messages name a path. The messages for `preparing.json`, the landing
  record, the index-lock record and the ref-updated record say that the record "is not a schema-version-2 ... record"
  without naming the file or the transaction, and every message carries the hint about fetching history, which does not
  apply. Nothing says how to recover (remove the named transaction directory by hand). That is a diagnostic gap
  to fix before relying on fail-closed as the user-facing outcome.

#### What the table allows

Benign at every phase and every damage, so their syncs are candidates to drop (a user decision, not done here):
`prepared.json` (file and directory sync), `hooks/plan.json`, `captured.json` (a torn one only costs a warning "could
not record the capture of landed ..." and subsumption in later replays), `index~pid.lock`, and the unsynced
`captured.index`, `commit.index` and scratch policy indexes (already unsynced). `store-held.json` is benign before the
ref moves and sticks after it, so it can lose its own sync only if the landing barrier makes it durable.
Everything else listed under the first two bullets above stays.

#### Outcomes by phase

Abbreviations: `T` is `cli-git-transactions/<id>`, `C` is `cli-git-captures`, `L` is `cli-git-transactions`. A line
lists the outcome of the next wrapped commands for each damage of one file. `cli-git/hook.lock` never appears: the hook
lock is released before the first phase marker.

#### capture-locked

Undamaged: discards.

- `C/capture.lock/owner.json`: empty discards, next commit exits 2; half discards, next commit exits 2; delete discards
- `T/conclusion/store-held.json`: all applicable damages discards
- `T/owner.json`: all applicable damages sticks
- `T/preparing.json`: empty sticks; half sticks; delete discards

#### preparation-done

Undamaged: discards.

- `C/sequence`: empty discards, next commit exits 2; half discards, next commit exits 2; delete discards
- `C/worktree-id`: empty discards, next commit exits 2; half discards; delete discards
- `T/captured.index`: all applicable damages discards
- `T/captured.json`: all applicable damages discards
- `T/cli-git-add-policy-<n>-<n>-0/index`: all applicable damages discards
- `T/commit.index`: all applicable damages discards
- `T/conclusion/store-held.json`: all applicable damages discards
- `T/hooks/plan.json`: all applicable damages discards
- `T/owner.json`: all applicable damages sticks
- `T/prepared.json`: all applicable damages discards
- `T/preparing.json`: empty sticks; half sticks; delete discards

#### landing-locked

Undamaged: discards.

- `C/sequence`: empty discards, next commit exits 2; half discards, next commit exits 2; delete discards
- `C/worktree-id`: empty discards, next commit exits 2; half discards; delete discards
- `T/captured.index`: all applicable damages discards
- `T/captured.json`: all applicable damages discards
- `T/cli-git-add-policy-<n>-<n>-0/index`: all applicable damages discards
- `T/commit.index`: all applicable damages discards
- `T/conclusion/store-held.json`: all applicable damages discards
- `T/hooks/plan.json`: all applicable damages discards
- `T/index-lock-1.json`: empty sticks; half sticks; delete discards, leaves index.lock, next commit exits 2
- `T/owner.json`: all applicable damages sticks
- `T/prepared.json`: all applicable damages discards
- `T/preparing.json`: empty sticks; half sticks; delete discards, leaves index.lock, next commit exits 2
- `L/landing.lock/owner.json`: empty sticks; half sticks; delete discards
- `index.lock`: all applicable damages discards
- `index~pid.lock`: all applicable damages discards

#### objects-migrated

Undamaged: discards.

- `C/sequence`: empty discards, next commit exits 2; half discards, next commit exits 2; delete discards
- `C/worktree-id`: empty discards, next commit exits 2; half discards; delete discards
- `T/captured.index`: all applicable damages discards
- `T/captured.json`: all applicable damages discards
- `T/cli-git-add-policy-<n>-<n>-0/index`: all applicable damages discards
- `T/commit.index`: all applicable damages discards
- `T/conclusion/store-held.json`: all applicable damages discards
- `T/hooks/plan.json`: all applicable damages discards
- `T/index-lock-1.json`: empty sticks; half sticks; delete discards, leaves index.lock, next commit exits 2
- `T/owner.json`: all applicable damages sticks
- `T/prepared.json`: all applicable damages discards
- `T/preparing.json`: empty sticks; half sticks; delete discards, leaves index.lock, next commit exits 2
- `L/landing.lock/owner.json`: empty sticks; half sticks; delete discards
- `index.lock`: all applicable damages discards
- `index~pid.lock`: all applicable damages discards

#### ref-updated

Undamaged: completes.

- `C/sequence`: empty completes, next commit exits 2; half completes, next commit exits 2; delete completes
- `C/worktree-id`: empty completes, next commit exits 2; half completes; delete completes
- `T/captured.index`: all applicable damages completes
- `T/captured.json`: all applicable damages completes
- `T/cli-git-add-policy-<n>-<n>-0/index`: all applicable damages completes
- `T/commit.index`: all applicable damages completes
- `T/conclusion/store-held.json`: all applicable damages sticks
- `T/hooks/plan.json`: all applicable damages completes
- `T/index-lock-1.json`: all applicable damages completes
- `T/landing-1.json`: empty sticks; half sticks; delete inconsistent (exit 0)
- `T/owner.json`: all applicable damages sticks
- `T/post-1.index`: empty corrupt index (exit 128); half corrupt index (exit 128); delete sticks
- `T/pre-landing-1.index`: all applicable damages sticks
- `T/prepared.json`: all applicable damages completes
- `T/preparing.json`: empty sticks; half sticks; delete inconsistent (exit 0)
- `T/ref-updated.json`: empty sticks; half sticks; delete completes
- `L/landing.lock/owner.json`: empty sticks; half sticks; delete completes
- `index.lock`: all applicable damages sticks
- `index~pid.lock`: all applicable damages completes

#### index-installed

Undamaged: completes.

- `C/landed/<commit>.json`: all applicable damages completes
- `C/sequence`: empty completes, next commit exits 2; half completes, next commit exits 2; delete completes
- `C/worktree-id`: empty completes, next commit exits 2; half completes; delete completes
- `T/captured.index`: all applicable damages completes
- `T/captured.json`: all applicable damages completes
- `T/cli-git-add-policy-<n>-<n>-0/index`: all applicable damages completes
- `T/commit.index`: all applicable damages completes
- `T/conclusion/store-held.json`: all applicable damages sticks
- `T/hooks/plan.json`: all applicable damages completes
- `T/index-installed`: all applicable damages completes
- `T/index-lock-1.json`: all applicable damages completes
- `T/landing-1.json`: empty sticks; half sticks; delete completes
- `T/owner.json`: all applicable damages sticks
- `T/post-1.index`: empty completes; half completes; delete sticks
- `T/pre-landing-1.index`: empty completes; half completes; delete sticks
- `T/prepared.json`: all applicable damages completes
- `T/preparing.json`: empty sticks; half sticks; delete completes
- `T/ref-updated.json`: empty sticks; half sticks; delete completes
- `L/landing.lock/owner.json`: empty sticks; half sticks; delete completes

## Budgets

The host and gate suites carry a spawn budget (from `GIT_TRACE2_PERF`) and the host suite a sync budget (from
`strace`); each slice lowers its budget to the measured count. Starting point: 45 processes, 41 syncs.
