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
  Done (slice S1): `package/git-policy/cli/src/native/hook_presence.rs` decides per event, `write_hook_dispatch`
  writes only the entries whose event may have a hook, and `run_post_commit_hook` skips the author lookup, the lock
  and `git hook run` for `post-commit` when no entry of that name exists when it comes due (looked up then, not at
  planning, so a hook that `pre-commit` installs still runs). An error other than "not found" while looking keeps the
  entry. The replay's `pre-commit` re-run is skipped when its entry was omitted. Parity:
  `binary_hook_shim_tests.rs` runs a wrapped and a direct commit over fifteen shapes (no hooks, all executable, one
  event, not executable, links, configuration hooks, relative, absolute, empty, missing and `-c` hooks paths,
  disabled events, a linked worktree, a hook installed during the commit, a failing hook) and compares status,
  output, hook log, message and tree.
- `findmnt` (3 runs): the filesystem identity of `.git` and `index.lock`. To be replaced by `statx`, `/proc/self/mountinfo`
  and `/dev/disk/by-uuid` when the mapping is unambiguous; otherwise `findmnt` stays. Needs a check of how the
  identity is consumed before choosing (recorded below when read).
- One configuration snapshot (`git config --list -z --show-scope`) in place of the separate `config` runs
  (`core.hooksPath`, the four `hook.<event>.enabled` reads already batched, the maintenance keys, ref storage).
  Last value wins, `-c` and `GIT_CONFIG_*` values come last, bool canonicalized by Git's rules; `--type=path` values
  starting with `~` or `%(prefix)` fall back to a spawn.
  Done (slice S3): `config_snapshot.rs` reads `git config --list -z --show-scope` once, in `capture_invocation_layout`,
  and answers `core.hooksPath`, the four `hook.<event>.enabled` switches, the maintenance keys (bool-or-int text),
  whether any `hook.<name>.event` or `.command` key exists, the files ref storage, and the shadow's repository
  format. Any entry it does not model makes that lookup `Unhandled` and the caller starts Git for the key, as
  before: a valueless or empty path, a path starting with `~`, `%(` or `:(`, a boolean or integer with a unit suffix,
  a sign, a leading zero or hexadecimal, a bad boolean, text that is not UTF-8. `git config` formats every entry of
  a key, so one unmodelled entry makes the whole key unhandled even when it is not the last. Details found by probe
  on Git 2.56.0: listing order is precedence order (files, then `GIT_CONFIG_COUNT`, then `GIT_CONFIG_PARAMETERS`
  with `-c` appended last, all in the `command` scope); `--type=bool` of `1k` is `true` and of `08` is fatal;
  `--type=bool-or-int` of `007` prints `7`; `--type=path` of a valueless key is fatal.
  Differential tests: `config_snapshot_tests.rs` draws 60 deterministic configurations over files, includes,
  conditional includes, a global file, `GIT_CONFIG_COUNT`, `-c`, repeated and valueless keys and bad values, and
  compares each lookup with `git config --type=<kind> --get`; it counts answered and declined lookups so a run
  that declines everything fails.
  Which consumers share the read: the plan (hooks path, switches, named hooks) always, because it reads with the
  caller's global options as the snapshot does; the ref storage, the maintenance keys and the shadow format only
  when the read carried no global options, because their original reads ignored the options or ran elsewhere.
  The ref storage is proven `files` only when no entry names `extensions.refstorage`; any listing that names it
  asks Git. The shadow format comes from the `local` scope only when no `include.path` or `includeIf.*.path` key
  exists, because `git config --file config --list` reads that one file without includes.
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
regenerate the record by setting `CLI_GIT_TEARING_REPORT=<file>`. The record was made on the host's Git 2.55.0 and
the same 247 outcomes were reproduced unchanged on the gate's Git 2.56.0 in a capped container (two CPUs, 2 GiB), so
no regeneration was needed. The case keys must not depend on process IDs: a container numbers processes from 1, and
the policy scratch directory `cli-git-add-policy-<pid>-<nanoseconds>-<n>` is therefore keyed as
`cli-git-add-policy-<scratch>`. After a Git upgrade, rerun the controls on the gate's Git and regenerate if they
differ.

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
## Results of the hook-entry and configuration-snapshot slices

Method: `strace --follow-forks -e trace=execve,fsync,fdatasync,syncfs,sync_file_range` over one wrapped
`git commit --quiet --message=x -- a.txt` of the debug build in a fresh repository with the host's Git 2.56.0,
counting `execve` of `/usr/bin/git` (equal to the `| start ` lines of `GIT_TRACE2_PERF`, 52 in both) and sync calls;
scratch script `measure.ts <binary> <scenario>` beside the repository's `attribute-commit.ts`.
Before is the commit that preceded these slices; after is the branch tip.

- No hooks: 52 to 43 Git starts, 42 to 38 sync calls, three wrapper re-executions as dispatcher entries to none.
- One `pre-commit` hook: 52 to 44 Git starts, 42 to 39 sync calls.
- All four hooks: 52 to 48 Git starts, 42 to 42 sync calls (the configuration reads are the only saving).
- A configuration-defined hook: 52 to 48 Git starts, 42 to 42 sync calls.
- A custom `core.hooksPath` holding one `commit-msg` hook: 52 to 44 Git starts, 42 to 39 sync calls.
- `measure-fsync.ts` (an `--allow-empty` commit, no hooks): 41 to 37 sync calls.

Permanent budget: `binary_hook_shim_tests.rs` fails a commit with no hooks that starts any hook runner or more than
one `git config`, a commit with four hooks that does not start exactly four runners, and a commit with one hook that
does not start exactly one.

Known deviations, each accepted for the saving and each recorded here because it is not byte-for-byte what separate
reads gave:

- The snapshot is read when the commit begins, so the maintenance decision no longer sees a change that a hook or
  another process makes to `maintenance.auto`, `gc.auto` or the detach keys during the commit. Example: a
  `pre-commit` hook that runs `git config maintenance.auto false` used to stop that commit's maintenance run; it no
  longer does. The hooks path and the switches were read just before preparation and are now read earlier by the
  length of the policy pass.
- An entry is omitted from a plan-time look at the hooks directory, so a `pre-commit`, `prepare-commit-msg` or
  `commit-msg` hook created between planning and the native commit (milliseconds) does not run. `post-commit` is
  looked up when it comes due and has no such gap.
- A named hook (`hook.<name>.event`) is found from the real repository's configuration, while the dispatcher's
  `git hook run` sees the shadow's configuration, which includes the real one. They differ only if a conditional
  include matches the shadow's path and not the real one, for example a `gitdir:` pattern that names the shadow
  directory; that include would define a hook the wrapper no longer starts.

Found while probing and left unchanged: the dispatcher skips a user hook of an event with `hook.<event>.enabled=false`,
while Git 2.56.0 runs a traditional hook whatever that switch says (it disables only hooks defined by name).
`a_disabled_event_with_a_hook_is_still_skipped_by_the_wrapper` pins the wrapper's existing behavior.

Refused, with the counterexample:

- Using the snapshot for the ref storage whenever it names `extensions.refStorage`: with
  `core.repositoryformatversion = 0` Git fails (`repo version is 0, but v1-only extension found`), and with the key
  only in an included file Git prints `files`; the repository-format rules are Git's, so the key's presence asks Git.
- Modelling `1k`, `-1`, `007`, `0x10` and `08`: `--type=bool` gives `true`, `true`, `true`, `true` and a fatal error;
  `--type=bool-or-int` gives `1024`, `-1`, `7`, `16` and a fatal error; the numeric grammar is `strtoimax` base 0
  with unit suffixes, so only `0` and digit strings from `1` to `9` are decided in Rust.
- Reusing the snapshot for the consumers that ignore global options when the read carried a `-c`: the original reads
  would not have seen the option, and a `-C` or `--git-dir` changes which repository is listed.

Not done: the `git config --file config --list` of the shadow stays when the configuration includes another file;
the `config --get` reads of the auto-push (`remote.<name>.push`, `commit.gpgsign`) are outside the commit path
measured here; a commit made with global options (`git -c k=v commit`) still starts one read each for the ref
storage, the maintenance keys and the shadow format, because those reads ignore the options or run elsewhere.

## Budgets

The host and gate suites carry a spawn budget (from `GIT_TRACE2_PERF`) and the host suite a sync budget (from
`strace`); each slice lowers its budget to the measured count. Starting point: 45 processes, 41 syncs.

## Filesystem identity without `findmnt` (slice S2)

Scope: the three `findmnt` starts of a commit (two on `index.lock`, one on `.git`), all made by
`package/git-policy/cli/src/native/filesystem_identity.rs`. The `stat --file-system` fallback that follows an empty
`findmnt` answer is not part of this slice.

### What must stay identical

Read from `filesystem_identity.rs`, `real_index_lock.rs`, `recovery_files.rs` and the incumbent
`package/fs-id/core/src/platform-resolvers.ts`, `parsers.ts`.

- **The recorded string.** `filesystem_identity(path)` canonicalizes `path`, then on Linux returns
  `fs-uuid_<payload>` where the payload is the `findmnt --target <canonical> --output=UUID --noheadings` standard
  output, trimmed with JavaScript `.trim()` semantics, lowercased, and accepted only if it is 1 to 512 characters of
  `[a-z0-9.-]` and not a lone `-`. Example: a vfat UUID `44C8-C6BA` is recorded as `fs-uuid_44c8-c6ba`.
- **The empty case.** An empty, multi-line or unsafe payload (`findmnt` prints `\n` for `/`, `/tmp`, `/proc`, `/dev`,
  fuse mounts and `\n\n` for two mounts stacked on one mount point) is not an identity. The resolver then runs
  `stat --file-system --format=%i` and records `f-fsid_<hex>`. A string is therefore never empty; `None` only when both
  commands fail.
- **Where it is recorded and compared.** `created_identity` (`real_index_lock.rs`) records it for the directory
  holding the real index into the journal field `fsId`. `owned_lock_mismatch_with` and `release_owned_lock_with`
  (`recovery_files.rs`) compare it, for the lock file, with the recorded `fsId` together with device and inode.
- **How a mismatch is treated.** `None` from `filesystem_identity` is an error, never a mismatch: the landing stops
  ("the filesystem identity of the worktree is unavailable") and recovery returns a `RecoveryError`. A string that
  differs from the journal is "Index lock identity changed", the lock is reported `Foreign` and is left in place.
  A wrong string therefore leaves the owner's own lock in place as foreign and stops recovery from releasing it; the
  in-process path must never produce a string `findmnt` would not.
- **Journals outlive binaries.** The record is read by older builds of this wrapper and by the TypeScript incumbent
  (`fsId` in the journal JSON). The new code must produce the byte-identical text the old code and `fs-id-core`
  produce for the same filesystem: same prefix, same lowercase, same fallback to `f-fsid_`.
- **Mount namespaces and containers.** `findmnt` is a child of the wrapper, so it reads the wrapper's own
  `/proc/self/mountinfo`, root directory and `/run/udev`. The in-process code reads the same files in the same
  process, so every namespace question has the same answer. A container without `/run/udev` data for the device gets
  no UUID in-process, and the code spawns `findmnt` exactly as before; a host-recorded `fs-uuid_` and a
  container-computed `f-fsid_` already differ today, and that does not change.

### How `findmnt` 2.41 gets the text (read from util-linux `misc-utils/findmnt.c`, confirmed by `strace`)

- `--target X` first lists every `/proc/self/mountinfo` line whose mount point equals `X`; if none, it walks `X` up
  one component at a time (`mnt_table_find_mountpoint`, backward, so the last line wins) and then lists every line
  whose mount point equals the one found. Stacked mounts therefore print several lines.
- The UUID column resolves the source field with `mnt_resolve_spec`, canonicalizes it, and asks libudev for the block
  device by sysfs name (`/sys/class/block/<name>`), which reads `/run/udev/data/b<major>:<minor>` and returns
  `ID_FS_UUID_ENC` with `\xHH` sequences decoded. Only when that is absent does it call `libblkid`, which probes the
  device and needs read access. `strace` on this host shows `openat("/run/udev/data/b252:1")` and no access to
  `/dev/disk/by-uuid`, so `/dev/disk/by-uuid` is not consulted by the oracle (the brief named it; the oracle is the
  authority, and for a multi-device Btrfs the link names only one member while every member has the record).

### Exactness rules for the in-process path

- Answer only when exactly one mountinfo line has the winning mount point, the mount ID of that line equals the
  `stx_mnt_id` that `statx` reports for the path, and the source is a `/dev/` block device whose udev record holds
  `ID_FS_UUID_ENC`. Every other shape spawns `findmnt` unchanged.
- A source that is not a path (`overlay`, `tmpfs`, `proc`, `composefs`) has no UUID: `findmnt` prints an empty line.
  That conclusion is drawn without a spawn only if the source also names no existing path and no sysfs block device.
  The `stat` fallback still runs.

### Result

Code: `src/native/filesystem_identity_mount.rs` (pure decisions plus one `statx` call) and the `MountUuid` match in
`linux_fs_id` (`src/native/filesystem_identity.rs`). Assumption recorded: `findmnt` is linked with libudev, as the
Fedora and Debian builds are; the differential tests fail loudly on a host where it prints something else.

Process starts of one wrapped `git commit --allow-empty` (the repository's `attribute-commit.ts`, pointed at the debug
build of each commit through `BINARY`; `BASE` picks the fixture directory), run from `${HOME}/temp/agent/s2`:

- Fixture on Btrfs with a udev record, before (parent commit `e345bba143`): 49 starts, 3 of them `findmnt`.
  After: 46 starts, 0 `findmnt`. Command:
  `BINARY=<worktree>/package/git-policy/cli/target/debug/cli-git-native node attribute-commit.ts`.
- Fixture on tmpfs (`BASE=/tmp/s2-fixture`), before: 52 starts, 3 `findmnt` and 3 `stat`. After: 49 starts, 0
  `findmnt`, 3 `stat`. The `stat --file-system` runs remain; replacing them needs the `f_fsid` word order of GNU
  `stat` and is a separate slice.
- Cost of the removed starts: 5 to 14 ms each under `strace`; the in-process work is one `statx`, one read of
  `/proc/self/mountinfo`, two small sysfs and udev reads.

Differential evidence (`findmnt` is the oracle; every non-`Unknown` verdict is compared with its exact standard
output, and `Unknown` is printed with what `findmnt` printed):

- Host paths: `/`, `/tmp`, `/proc`, `/dev`, `/run`, `/sys`, `/usr` (empty line, `Absent`); `/etc`, `/var`, `/var/home`,
  the repository (Btrfs subvolumes of one device, `1bb3d23e-...`), `/boot` (ext4), `/boot/efi` (vfat, `44C8-C6BA`),
  `/var/mnt/encrypted` (device mapper source `/dev/mapper/crypt_sda`), a nested Btrfs subvolume whose `st_dev` differs
  from its mount's (matched through the mount ID, so subvolumes need no fallback), fuse mounts (`Absent`).
- A private user and mount namespace (`unshare --user --map-root-user --mount`, no host mount touched): bind mounts
  of a directory, of a directory whose name has a space, of a directory whose name has a backslash, of another
  device's directory, of a single file, of a `tmpfs` directory, a `tmpfs`, an unprivileged `overlay` (`Absent`), and
  two mounts stacked on one mount point. All equal `findmnt` or fall back.
- A rootless Podman container (`--memory=2g --cpus=2 --network=none`, Debian trixie, `findmnt` 2.41, no
  `/run/udev`): overlay root, `/proc`, `/dev`, `/etc/hosts` bind and `/dev/shm` are `Absent` and equal `findmnt`; the
  Btrfs bind whose source `/dev/mapper/crypt_sda` does not exist in the container is `Unknown` and spawns.

Classes that fall back to spawning `findmnt` (kept exact by construction, each with its reason):

- **Stacked mounts on one mount point** (`/proc/sys/fs/binfmt_misc` here: `autofs` under `binfmt_misc`): `findmnt`
  prints one line per mount, `\n\n`, which the payload check rejects, so the identity comes from `stat`. Counterexample
  for answering in-process with the last entry: it would claim `Absent` for a two-line output.
- **A mount covered by a later mount on an ancestor**: the name walk finds the covered mount, `statx` reports the
  covering one. `findmnt` prints the covered mount's line (namespace case `cover/y/z`); the mount ID check makes the
  in-process answer `Unknown`.
- **A device with no udev record** (containers, minimal images) or whose record lacks `ID_FS_UUID_ENC`: `findmnt` then
  asks `libblkid`, which probes the device and depends on read access.
- **Sources that are paths but not `/dev/` nodes, `host:/path` network sources, tag sources with `=`, names that
  exist in the working directory or in sysfs**: `libblkid` or libudev would look at them.
- **UUID text outside letters, digits, dot and dash** (spaces, `\xHH` escapes that decode to anything else, a NUL):
  libsmartcols may re-encode on output.
- Older kernels without `STATX_MNT_ID`: no mount ID, no in-process answer.

Rejected: `/dev/disk/by-uuid` as the source of the text (see "How `findmnt` 2.41 gets the text"). Reading
`ID_FS_UUID_SUB_ENC` (the Btrfs device UUID) instead of `ID_FS_UUID_ENC` is the natural mistake and the guard below
catches it: the repository path then records `87fa7a73-...` against `findmnt`'s `1bb3d23e-...`.

Guards shown failing before being kept (each break applied, tests run, restored):

- `ID_FS_UUID_SUB_ENC` in place of `ID_FS_UUID_ENC`: `udev_records_give_one_plain_uuid`,
  `in_process_uuids_equal_what_findmnt_prints` and `identities_equal_the_command_only_resolution` failed.
- Taking the last entry of stacked mounts instead of giving up: `the_decision_gives_up_on_unproven_shapes` and
  `in_process_uuids_equal_what_findmnt_prints` failed.
- Dropping the mount ID comparison: `the_decision_gives_up_on_unproven_shapes` failed (the differential tests cannot
  see it on this host, because the covered and the covering mount are on one filesystem).
- Dropping the working-directory existence check for pseudo sources: `pseudo_sources_need_no_lookup_to_succeed` failed.
- Forcing every path to `Unknown` (the old behavior): `a_pseudo_filesystem_starts_only_stat` and
  `a_device_with_a_udev_record_starts_nothing` failed. They count the commands through `linux_fs_id_with`.

Not done in this slice: replacing `stat --file-system --format=%i`; a permanent per-commit spawn budget test (no such
test exists in this tree yet; the lowered count is 3 for the `findmnt` row).
