# cli-git: fewer processes and fewer syncs per commit, and retry on process exhaustion

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Status:
accepted for implementation by the user (2026-10-29),
scope:
the process-start retry (slice R),
the candidate replacements,
the journal syncs (slice J,
task 69) and the in-process filesystem identity (slice S2).
Owner package:
`package/git-policy/cli`.
Tracking issue:
1804 (follow-up work found while stabilizing it).

## Why

A wrapped `git commit` of one explicit path starts about 45 real Git processes and makes about 41 sync calls,
against one process and no syncs for direct Git.
Measured on an idle host with the debug and release builds alike:

- wall time about 180 ms per commit,
  of which about 77 ms is inside sync calls (strace,
  same fixture);
- the container gate takes 12 to 32 seconds for the commit tests under host load,
  and the same suite takes about 67
  seconds when the host is quiet;
- a host-wide fork-failure episode (the user cgroup recorded 598 limit events) failed commits with
  `Resource temporarily unavailable (os error 11)`.

Every process start and every sync is multiplied by load,
so the fix is to do less of both and to ride out a short
refusal to start a process.

## Decisions (user)

- Process-exhaustion safeguard:
  retry failed starts everywhere,
  and also retry read-only and shadow-repository Git
  commands that fail inside themselves (`unable to create thread`,
  `cannot fork`),
  with an allow-list.
- Spawn reduction:
  save as many processes as possible,
  engineering the replacement in the wrapper where the result is
  provably identical to Git's,
  in the manner of `package/forbidden-strings/core` (own engine,
  an
  `OPTIMIZATIONS.md`-style log of measured wins and rejections,
  oracle-checked parity).
- Journal syncs:
  option B of the earlier question (sync at the barriers before side effects),
  after a crash
  harness exists that tears,
  drops and reorders records.

## Method

- Parity first:
  every in-process replacement is tested against real Git (`git` is the oracle) over generated cases,
  and falls back to spawning Git for any shape it does not handle exactly.
  A fallback is never a failure.
- Every replacement shows its win with the same scripts:
  `GIT_TRACE2_PERF` process counts and `strace` sync counts.
  A permanent budget test per measure ratchets down after each slice (a regression fails the host and gate suites).
- A guard test is shown failing before the change is kept (the repo's rule).
- What is measured and rejected is recorded here,
  with the number or the counterexample.

## Process-start retry (slice R)

A start that fails with `EAGAIN` (`ErrorKind::WouldBlock`) started nothing,
so retrying is always safe.

- One helper starts every child:
  10 ms backoff doubling to 500 ms,
  a 5 s total budget,
  one warning per retry,
  then the
  existing error plus the attempt count.
  It covers the roughly 20 call sites (`run_git`,
  bounded children,
  forwarding including the `execve` path,
  hooks,
  signal relay,
  post-commit,
  probes).
- Git commands that start and then fail inside themselves are retried only from an allow-list of commands that are
  harmless to re-run (read-only queries,
  and the idempotent shadow-repository steps),
  matched on the exit status and
  the wording of the failure;
  mutating steps (reference updates,
  landing) are not retried,
  and recovery already
  completes or rolls back those.
- Tests:
  a unit test with a start that refuses K times;
  the fork-failure sweep gains burst cases
  (`--inject=clone3:error=EAGAIN:when=N..N+K`),
  which must now succeed for bursts shorter than the budget.

## Candidate replacements, with the exactness condition for each

Measured process lifetimes of one commit (strace):
`findmnt` 3 runs about 25 ms,
`rev-parse` 12 runs about 32 ms,
`config` 7 runs about 16 ms,
`git hook` 4 runs about 14 ms plus the wrapper's own hook shim re-executions.

- Hook shims for events with no hook:
  the shadow commit points `core.hooksPath` at shims that re-enter the wrapper,
  which takes the hook lock (a synced lock candidate) and runs `git hook run --ignore-missing <event>`.
  An event with
  no user hook needs no shim.
  Exact when conservative:
  the shim is omitted only if the user's hooks directory has no
  entry of that name at all (any mode,
  because Git prints a hint for a non-executable file) and the configuration has
  no `hook.<name>.event` or `.command` key;
  any other shape keeps today's behavior.
  Saves up to four wrapper
  re-executions,
  four `git hook` runs and four synced lock candidates.
  Done (slice S1):
  `package/git-policy/cli/src/native/hook_presence.rs` decides per event,
  `write_hook_dispatch`
  writes only the entries whose event may have a hook,
  and `run_post_commit_hook` skips the author lookup,
  the lock
  and `git hook run` for `post-commit` when no entry of that name exists when it comes due (looked up then,
  not at
  planning,
  so a hook that `pre-commit` installs still runs).
  An error other than "not found" while looking keeps the
  entry.
  The replay's `pre-commit` re-run is skipped when its entry was omitted.
  Parity:
  `binary_hook_shim_tests.rs` runs a wrapped and a direct commit over fifteen shapes (no hooks,
  all executable,
  one
  event,
  not executable,
  links,
  configuration hooks,
  relative,
  absolute,
  empty,
  missing and `-c` hooks paths,
  disabled events,
  a linked worktree,
  a hook installed during the commit,
  a failing hook) and compares status,
  output,
  hook log,
  message and tree.
  Superseded (note added 2026-11-01):
  commit `3aa714eac0` dropped traditional hook support and deleted
  `hook_presence.rs` and its tests,
  and the hooks removal merged to main as `22dc5e82a6` (branch head `ed06e54a4f`)
  then dropped every external hook:
  each real-Git start carries `core.hooksPath` pointing at a directory that cannot
  exist plus `hook.<event>.enabled=false` for every event,
  and the wrapper starts no hook and no hook shim.
  The S1
  shim record,
  `write_hook_dispatch`,
  `run_post_commit_hook` and the fifteen-shape parity suite describe code that
  no longer exists;
  the current behavior is in `package/git-policy/cli/SPEC.md` and `src/native/hooks_off.rs`.
- `findmnt` (3 runs):
  the filesystem identity of `.git` and `index.lock`.
  To be replaced by `statx`,
  `/proc/self/mountinfo`
  and `/dev/disk/by-uuid` when the mapping is unambiguous;
  otherwise `findmnt` stays.
  Needs a check of how the
  identity is consumed before choosing (recorded under `What must stay identical` when read).
- One configuration snapshot (`git config --list -z --show-scope`) in place of the separate `config` runs
  (`core.hooksPath`,
  the four `hook.<event>.enabled` reads already batched,
  the maintenance keys,
  ref storage).
  Last value wins,
  `-c` and `GIT_CONFIG_*` values come last,
  bool canonicalized by Git's rules;
  `--type=path` values
  starting with `~` or `%(prefix)` fall back to a spawn.
  Done (slice S3):
  `config_snapshot.rs` reads `git config --list -z --show-scope` once,
  in `capture_invocation_layout`,
  and answers `core.hooksPath`,
  the four `hook.<event>.enabled` switches,
  the maintenance keys (bool-or-int text),
  whether any `hook.<name>.event` or `.command` key exists,
  the files ref storage,
  and the shadow's repository
  format.
  Any entry it does not model makes that lookup `Unhandled` and the caller starts Git for the key,
  as
  before:
  a valueless or empty path,
  a path starting with `~`,
  `%(` or `:(`,
  a boolean or integer with a unit suffix,
  a sign,
  a leading zero or hexadecimal,
  a bad boolean,
  text that is not UTF-8.
  `git config` formats every entry of
  a key,
  so one unmodelled entry makes the whole key unhandled even when it is not the last.
  Details found by probe
  on Git 2.56.0:
  listing order is precedence order (files,
  then `GIT_CONFIG_COUNT`,
  then `GIT_CONFIG_PARAMETERS`
  with `-c` appended last,
  all in the `command` scope);
  `--type=bool` of `1k` is `true` and of `08` is fatal;
  `--type=bool-or-int` of `007` prints `7`;
  `--type=path` of a valueless key is fatal.
  Differential tests:
  `config_snapshot_tests.rs` draws 60 deterministic configurations over files,
  includes,
  conditional includes,
  a global file,
  `GIT_CONFIG_COUNT`,
  `-c`,
  repeated and valueless keys and bad values,
  and
  compares each lookup with `git config --type=<kind> --get`;
  it counts answered and declined lookups so a run
  that declines everything fails.
  Which consumers share the read:
  the plan (hooks path,
  switches,
  named hooks) always,
  because it reads with the
  caller's global options as the snapshot does;
  the ref storage,
  the maintenance keys and the shadow format only
  when the read carried no global options,
  because their original reads ignored the options or ran elsewhere.
  The ref storage is proven `files` only when no entry names `extensions.refstorage`;
  any listing that names it
  asks Git.
  The shadow format comes from the `local` scope only when no `include.path` or `includeIf.*.path` key
  exists,
  because `git config --file config --list` reads that one file without includes.
- Empty-tree object name:
  the hash of `tree 0` under the repository's object format (a constant per format) in place of
  `git hash-object -t tree --stdin`.
- `symbolic-ref HEAD` (files ref storage):
  read `HEAD`;
  anything but `ref: <name>` or a full object name falls back.
  The landing re-read stays a read of the same file,
  so the compare-and-swap protection is unchanged.
- Sequencer markers (`MERGE_HEAD`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`) on the files backend:
  when none of the
  loose files (including the six lookup spellings `rev-parse` tries) exists and `packed-refs` does not contain the
  name,
  they are absent without a spawn;
  any hit falls back to `rev-parse --verify`.
  A dangling marker therefore
  stays a conclusion (pinned by `a_marker_naming_a_missing_object_is_still_a_conclusion`).
- `git version`:
  cached per binary identity (path,
  size,
  mtime,
  inode) in the repository's private directory.

## Rejected, with evidence

- Reading the three sequencer markers with `cat-file --batch-check`:
  it reports a marker that names a missing object
  as missing,
  so a corrupt merge would become a plain commit that drops the merge parent.
- Merging `core.hooksPath` with the maintenance reads by hand:
  different phases and a `--type=path` value;
  covered
  instead by `config_snapshot.rs`.
- Capping Git's threads (`pack.threads=1`,
  `core.preloadindex=false`):
  101 ms against 105 and 100 ms baseline,
  and
  the container was never throttled (`nr_throttled 0`).
- Re-reading `HEAD` and the target ref once instead of twice:
  the second reads at landing are the compare-and-swap
  protection.

## Journal syncs (slice J, task 69)

Order from the independent review:
skip directory syncs where nothing changed (three `ensure_*` syncs),
skip the two
syncs after removing a landed transaction directory,
defer directory syncs to barriers,
add torn-record tolerance for
pre-landing records with a tearing,
dropping and reordering harness (a kill is not a power loss),
and only then drop
file syncs of pre-landing records.
Lock-candidate syncs stay:
a torn blocking `owner.json` would stick every
acquirer.
The sync-site table described the transaction route that 4d part 2 deleted,
so it is no longer kept.

### J1 outcome: the `ensure_*` syncs were already conditional

The three `ensure_*` syncs in the handover table (`.git` twice,
`cli-git-captures` once) are creation syncs,
not
redundant ones.
`ensure_transaction_root` syncs its parent only in the arm where it created the directory,
`ensure_worktree_id` only after it wrote the identity file,
and `record_landed_capture` only after it wrote a new
record.
The measurement ran a first commit in a fresh repository,
where the registry,
the capture store and the
landed-record directory really were created.
A steady-state commit has none of them.
No production code changed.

Measured on the debug binary (`strace -f` on `fsync` and `fdatasync`;
the split is by the last `execve` of the calling
process,
so a Git child counts as a child and a re-entered hook shim counts as the wrapper):

- fresh repository,
  `node sync-count.ts`:
  42 calls,
  36 by the wrapper and 6 by children.
  Three are `ensure_*`
  creation syncs (`.git` for the registry,
  `.git` for the capture store,
  `cli-git-captures` for the landed-record
  directory),
  and the new `worktree-id` file and its directory add two more;
- warm repository (one earlier wrapped commit),
  `node sync-count.ts warm`:
  41 calls,
  31 by the wrapper and 10 by
  children,
  none from an `ensure_*` helper.
  Per-call attribution comes from `node sync-sites.ts` (cold) and
  `WARM=1 node sync-sites.ts` (warm),
  which print the calling function of each sync in call order.

The deliverable is the pin:
a test-only recorder in `sync_directory` (`take_directory_syncs` in
`src/native/private_storage.rs`,
per thread) and unit tests that assert the exact directories synced when a helper
creates and the empty list when it finds.
With the three guards broken (sync also on the already-exists and
already-written paths) all six tests fail;
they pass restored.

### J2 outcome: both post-removal syncs dropped

`remove_transaction_directory` (called by `dispose_landed_transaction` and by recovery) synced the registry twice:
after renaming the directory to `<id>.retired`,
and after deleting the retired tree.
Both are dropped;
the in-place
removal of a legacy directory,
which has no retired name,
keeps its one sync.
A retired directory that survives a
power loss is removed by the next recovery (`recover_registered_transactions` removes every `Retired` entry),
and a
landed directory that is still published is classified `AlreadyInstalled` and cleaned;
`a_completed_landing_that_is_still_published_is_cleaned_by_recovery` runs both together and fails when the
retired-entry removal is disabled.

The rename sync was an ordering barrier between the rename and the deletions inside the directory,
not a durability
sync of a removal.
Dropping it rests on an assumption that is not sourced here:
the filesystems this repository
targets (ext4,
XFS and btrfs,
per the handover) apply metadata operations in order,
so a power loss leaves the
published name,
the retired name with some contents,
or nothing.
If a filesystem persisted the unlinks but not the
rename,
the published name `<id>` would be left without `owner.json`,
and the `J3 outcome: what recovery does with emptied, halved and dropped records` section shows that this state
sticks (deleting `owner.json` fails closed at every phase,
naming the directory).
No such state was produced by any
test;
it is the risk accepted by the decision to drop the sync.

Measured with `node sync-count.ts` and `node sync-count.ts warm` before and after:
42 to 40 cold and 41 to 39 warm
(wrapper 31 to 29 warm).
The pins are `removing_a_registry_directory_syncs_nothing` and
`disposal_syncs_no_directory`;
both fail with a `sync_parent` restored after the rename (and the earlier form of the
pin failed with the post-deletion `sync_parent` restored).
No test in the repository counts syncs of the built binary
(`strace` appeared only in `binary_fork_failure_tests.rs` then);
the sync budget named under Budgets is now
`binary_budget_tests.rs`.

### J3 outcome: what recovery does with emptied, halved and dropped records

The harness is `src/native/binary_recovery_tearing_tests.rs` in `package/git-policy/cli`,
with the recorded outcomes in
`binary_recovery_tearing_expected.rs`.
A real wrapped `git commit --message=killed a.txt` is killed by
`CLI_GIT_TEST_ONLY_PHASE_SIGNAL=<phase>:kill:<dir>` at each of the six phases a commit reaches.
Then each regular file
the commit left is damaged on its own,
and the next wrapped commands run:
`git status` twice,
then a `git commit`.
The
files are everything under `cli-git-transactions` (journal records,
index snapshots,
the landing lock),
everything
under `cli-git-captures` (the capture lock,
`sequence`,
`worktree-id`,
landed records),
`cli-git/hook.lock`,
and the
real `index.lock` with its `index~pid.lock`.
The three damages are empty (zero bytes),
half (cut to half the length)
and delete (the file is gone while the records written after it stay,
a flush that reached the disk out of order).
A
damage that cannot change the file (emptying an empty file,
halving a file under two bytes) is skipped.
A kill leaves
the page cache intact,
so the kill alone shows nothing;
the damage step is the test,
and the harness fails when the
damage step is disabled (shown) or when any recorded outcome changes.

Vocabulary of the outcomes:
**completes** (exit 0,
the commit is on the branch and the index matches it),
**discards**
(exit 0,
nothing landed and the edit is intact),
**sticks** (exit 2 again on a repeat,
so every guarded command fails
until someone removes the file by hand),
**fails closed once** (exit 2,
then a later command works;
no case
did this).
A suffix marks a degraded state:
the following commit exits 2 although `git status` was fine,
or an
`index.lock` is left behind.
Run it with
`mise run //package/git-policy/cli:native:test:binary:host -- recovery_tearing` (six tests,
about 15 seconds);
regenerate the record by setting `CLI_GIT_TEARING_REPORT=<file>`.
The record was made on the host's Git 2.55.0 and
the same 247 outcomes were reproduced unchanged on the gate's Git 2.56.0 in a capped container (two CPUs,
2 GiB),
so
no regeneration was needed.
The case keys must not depend on process IDs:
a container numbers processes from 1,
and
the policy scratch directory `cli-git-add-policy-<pid>-<nanoseconds>-<n>` is therefore keyed as
`cli-git-add-policy-<scratch>`.
After a Git upgrade,
rerun the controls on the gate's Git and regenerate if they
differ.

Totals over 247 cases (two runs gave identical outcomes):
92 discard,
67 complete,
63 stick,
11 discard but the next
commit exits 2,
6 complete but the next commit exits 2,
4 discard and leave `index.lock` with the next commit failing,
2 leave a corrupt real index,
2 are inconsistent without any error.
No case hung,
and none failed closed and then
recovered.

#### Read first: the cases that matter

The first bullet (torn snapshots) and the silent-inconsistency bullet describe the state when the harness was written;
follow-ups 1 and 2 after the phase listings record how each was resolved,
and the listings were updated to match.

- Two index snapshots are not synced today and the table shows what tearing them does.
  `pre-landing-1.index` and
  `post-1.index` are written by a plain copy and by Git (the sync-site table in the handover has no sync for either,
  and Git's default `core.fsync=committed,-loose-object` excludes `index`,
  per `git help config`).
  The landing record
  names them only by device and inode,
  so recovery trusts their content.
  At `ref-updated`,
  an emptied or halved
  `post-1.index` is installed as the real index and every later command fails with
  `fatal: .git/index: index file smaller than expected` or `fatal: unable to read <object>` (`git status` exits 128,
  the next commit exits 1);
  an emptied or halved `pre-landing-1.index` sticks with "Real index no longer matches
  the pre-landing snapshot".
  A power loss that persists `landing-1.json` and the ref but not the snapshots' data
  can produce exactly this,
  depending on the filesystem (ext4 in `data=ordered` mode writes the data of files
  allocated in a committing transaction first;
  XFS and btrfs do not promise that).
  This is a possible exposure in the
  current code,
  not only a constraint on removing syncs.
  The
  remedies are a sync of both snapshots before `landing-1.json`,
  or recovery verifying their content.
  Neither is done
  here.
- Records that are synced today and must stay synced,
  because tearing them sticks every command:
  `owner.json` (all
  damages at every phase),
  `preparing.json` (empty and half),
  `index-lock-1.json`
  (empty and half before the ref moves),
  `landing-1.json` and `ref-updated.json` (empty and half),
  `store-held.json`
  after the ref moved
  (in a reftable repository,
  the only backend whose store can hold one,
  since 2026-11-09T03:26:44Z),
  and the `owner.json` of `landing.lock`.
  Their messages are fail-closed by design.
- Silent inconsistency:
  with `landing-1.json` or `preparing.json` missing at `ref-updated`,
  `git status` exits 0
  although the branch moved and the index and worktree were never reconciled (the observed outcome is neither
  completes nor discards;
  the likely cause is that a transaction without its landing record is treated as never
  landed).
  For
  `landing-1.json` the next commit then also succeeds;
  for `preparing.json` it exits 2.
  Only the sync of
  `landing-1.json` and its directory before the compare-and-swap prevents this;
  no later sync may replace it.
- A torn capture `sequence` or an emptied `worktree-id` leaves `git status` working while every later commit exits 2
  (the sequence is parsed only by a commit).
  A torn `capture.lock/owner.json` does the same.
  The sequence temporary
  file sync and the lock candidate syncs must stay.
- Deleted `index-lock-1.json` or `preparing.json` before the ref moves leaves the real `index.lock` in place,
  because
  recovery removes it only when a record names it;
  the next commit fails.
  That window also exists for a kill between
  creating `index.lock` and writing its record,
  so it is not caused by syncing.
- Deleting `index.lock` at `ref-updated` sticks ("opening the index lock ... No such file or directory"):
  the lock's
  directory entry is not synced before the compare-and-swap.
  This one is reachable only by losing the entry while the
  ref update persists.
- The J2 link:
  deleting `owner.json` sticks at every phase.
  That is the state a filesystem that persisted the deletions
  inside a removed transaction directory but not its rename would leave,
  which is the unsourced ordering assumption
  recorded under the J2 outcome.
- What the user is told (resolved by follow-up 3;
  the quoted message text is the state when the harness was written):
  only the
  owner-record messages name a path.
  The messages for `preparing.json`,
  the landing
  record,
  the index-lock record and the ref-updated record say that the record "is not a schema-version-2 ... record"
  without naming the file or the transaction,
  and every message carries the hint about fetching history,
  which does not
  apply.
  Nothing says how to recover (remove the named transaction directory by hand).
  That is a diagnostic gap
  to fix before relying on fail-closed as the user-facing outcome.

#### What the table allows

Benign at every phase and every damage,
so their syncs are candidates to drop (a user decision,
not done here):
`prepared.json` (file and directory sync),
`hooks/plan.json`,
`captured.json` (a torn one only costs a warning "could
not record the capture of landed ..." and subsumption in later replays),
`index~pid.lock`,
and the unsynced
`captured.index`,
`commit.index` and scratch policy indexes (already unsynced).
`store-held.json` is benign before the
ref moves and sticks after it,
so it can lose its own sync only if the landing barrier makes it durable.
Note 2026-11-09T03:26:44Z:
the writer now leaves the record out when its backend's ref store can hold no conclusion value,
which is every files-backend repository,
and its reader takes an absent record as naming nothing there.
The condition above therefore holds where values exist (a reftable repository) and has no subject elsewhere,
and the flush it removes is measured in
`doc/planning/cli-git-commit-latency-under-many-agents.md`.
Everything else listed in the torn snapshots bullet and in the records that are synced today bullet stays.

#### Outcomes by phase

Abbreviations:
`T` is `cli-git-transactions/<id>`,
`C` is `cli-git-captures`,
`L` is `cli-git-transactions`.
A line
lists the outcome of the next wrapped commands for each damage of one file.
`cli-git/hook.lock` never appears:
the hook
lock is released before the first phase marker.

#### capture-locked

Undamaged:
discards.

- `C/capture.lock/owner.json`:
  empty discards,
  next commit exits 2;
  half discards,
  next commit exits 2;
  delete discards
- `T/conclusion/store-held.json`:
  all applicable damages discards
- `T/owner.json`:
  all applicable damages sticks
- `T/preparing.json`:
  empty sticks;
  half sticks;
  delete discards

#### preparation-done

Undamaged:
discards.

- `C/sequence`:
  empty discards,
  next commit exits 2;
  half discards,
  next commit exits 2;
  delete discards
- `C/worktree-id`:
  empty discards,
  next commit exits 2;
  half discards;
  delete discards
- `T/captured.index`:
  all applicable damages discards
- `T/captured.json`:
  all applicable damages discards
- `T/cli-git-add-policy-<n>-<n>-0/index`:
  all applicable damages discards
- `T/commit.index`:
  all applicable damages discards
- `T/conclusion/store-held.json`:
  all applicable damages discards
- `T/hooks/plan.json`:
  all applicable damages discards
- `T/owner.json`:
  all applicable damages sticks
- `T/prepared.json`:
  all applicable damages discards
- `T/preparing.json`:
  empty sticks;
  half sticks;
  delete discards

#### landing-locked

Undamaged:
discards.

- `C/sequence`:
  empty discards,
  next commit exits 2;
  half discards,
  next commit exits 2;
  delete discards
- `C/worktree-id`:
  empty discards,
  next commit exits 2;
  half discards;
  delete discards
- `T/captured.index`:
  all applicable damages discards
- `T/captured.json`:
  all applicable damages discards
- `T/cli-git-add-policy-<n>-<n>-0/index`:
  all applicable damages discards
- `T/commit.index`:
  all applicable damages discards
- `T/conclusion/store-held.json`:
  all applicable damages discards
- `T/hooks/plan.json`:
  all applicable damages discards
- `T/index-lock-1.json`:
  empty sticks;
  half sticks;
  delete discards,
  leaves index.lock,
  next commit exits 2
- `T/owner.json`:
  all applicable damages sticks
- `T/prepared.json`:
  all applicable damages discards
- `T/preparing.json`:
  empty sticks;
  half sticks;
  delete discards,
  leaves index.lock,
  next commit exits 2
- `L/landing.lock/owner.json`:
  empty sticks;
  half sticks;
  delete discards
- `index.lock`:
  all applicable damages discards
- `index~pid.lock`:
  all applicable damages discards

#### objects-migrated

Undamaged:
discards.

- `C/sequence`:
  empty discards,
  next commit exits 2;
  half discards,
  next commit exits 2;
  delete discards
- `C/worktree-id`:
  empty discards,
  next commit exits 2;
  half discards;
  delete discards
- `T/captured.index`:
  all applicable damages discards
- `T/captured.json`:
  all applicable damages discards
- `T/cli-git-add-policy-<n>-<n>-0/index`:
  all applicable damages discards
- `T/commit.index`:
  all applicable damages discards
- `T/conclusion/store-held.json`:
  all applicable damages discards
- `T/hooks/plan.json`:
  all applicable damages discards
- `T/index-lock-1.json`:
  empty sticks;
  half sticks;
  delete discards,
  leaves index.lock,
  next commit exits 2
- `T/owner.json`:
  all applicable damages sticks
- `T/prepared.json`:
  all applicable damages discards
- `T/preparing.json`:
  empty sticks;
  half sticks;
  delete discards,
  leaves index.lock,
  next commit exits 2
- `L/landing.lock/owner.json`:
  empty sticks;
  half sticks;
  delete discards
- `index.lock`:
  all applicable damages discards
- `index~pid.lock`:
  all applicable damages discards

#### ref-updated

Undamaged:
completes.

- `C/sequence`:
  empty completes,
  next commit exits 2;
  half completes,
  next commit exits 2;
  delete completes
- `C/worktree-id`:
  empty completes,
  next commit exits 2;
  half completes;
  delete completes
- `T/captured.index`:
  all applicable damages completes
- `T/captured.json`:
  all applicable damages completes
- `T/cli-git-add-policy-<n>-<n>-0/index`:
  all applicable damages completes
- `T/commit.index`:
  all applicable damages completes
- `T/conclusion/store-held.json`:
  all applicable damages sticks
- `T/hooks/plan.json`:
  all applicable damages completes
- `T/index-lock-1.json`:
  all applicable damages completes
- `T/landing-1.json`:
  empty sticks;
  half sticks;
  delete sticks
- `T/owner.json`:
  all applicable damages sticks
- `T/post-1.index`:
  empty sticks;
  half sticks;
  delete sticks
- `T/pre-landing-1.index`:
  all applicable damages sticks
- `T/prepared.json`:
  all applicable damages completes
- `T/preparing.json`:
  empty sticks;
  half sticks;
  delete sticks
- `T/ref-updated.json`:
  empty sticks;
  half sticks;
  delete completes
- `L/landing.lock/owner.json`:
  empty sticks;
  half sticks;
  delete completes
- `index.lock`:
  all applicable damages sticks
- `index~pid.lock`:
  all applicable damages completes

#### index-installed

Undamaged:
completes.

- `C/landed/<commit>.json`:
  all applicable damages completes
- `C/sequence`:
  empty completes,
  next commit exits 2;
  half completes,
  next commit exits 2;
  delete completes
- `C/worktree-id`:
  empty completes,
  next commit exits 2;
  half completes;
  delete completes
- `T/captured.index`:
  all applicable damages completes
- `T/captured.json`:
  all applicable damages completes
- `T/cli-git-add-policy-<n>-<n>-0/index`:
  all applicable damages completes
- `T/commit.index`:
  all applicable damages completes
- `T/conclusion/store-held.json`:
  all applicable damages sticks
- `T/hooks/plan.json`:
  all applicable damages completes
- `T/index-installed`:
  all applicable damages completes
- `T/index-lock-1.json`:
  all applicable damages completes
- `T/landing-1.json`:
  empty sticks;
  half sticks;
  delete completes
- `T/owner.json`:
  all applicable damages sticks
- `T/post-1.index`:
  empty completes;
  half completes;
  delete sticks
- `T/pre-landing-1.index`:
  empty completes;
  half completes;
  delete sticks
- `T/prepared.json`:
  all applicable damages completes
- `T/preparing.json`:
  empty sticks;
  half sticks;
  delete completes
- `T/ref-updated.json`:
  empty sticks;
  half sticks;
  delete completes
- `L/landing.lock/owner.json`:
  empty sticks;
  half sticks;
  delete completes

### J3 follow-up 1: torn index snapshots

Two remedies were measured on the same machine (btrfs on `/var/home`,
host Git 2.55.0,
a repository of 5,000 tracked
files whose index is 521,631 bytes,
one wrapped `git commit --quiet --message=x -- a.txt` under
`strace --follow-forks --trace=fsync,fdatasync -y -T`;
scratch scripts `measure-landing.ts` and `raw-fsync.ts` in
`~/temp/agent/j3-snapshots`).

- Baseline:
  35,
  31,
  35 and 31 sync calls in four warm rounds (36 in the cold first round),
  each journal sync 1.8 to
  2.2 ms,
  82 to 97 ms inside syncs in total.
  A raw write and `fsync` of a 600,000-byte file took 1.1 ms (median,
  `/var/home`) to 3.0 ms (median,
  `/var/mnt/encrypted`).
- Remedy (a),
  sync both snapshots before `landing-1.json` (prototype,
  since reverted):
  38 calls in the cold first round,
  then 37,
  33,
  37,
  33 and 37,
  so exactly two more than the baseline round for round;
  the two snapshot syncs took 3.9
  (cold),
  4.0,
  4.3,
  4.0,
  4.2 and 5.5 ms per commit (about 2 ms each).
  The total inside syncs (67.6 to 85.4 ms) was below
  the baseline's,
  which shows the run-to-run noise exceeds the added cost.
- Remedy (b),
  record length and hash of each snapshot:
  FNV-1a 64 over the same 521,631 bytes took 0.38 ms in a release
  build (`rustc -O`,
  median of 200 reads;
  the read itself 0.02 ms),
  so about 0.8 ms per landing for both snapshots,
  growing linearly with the index.
  No syncs.

Chosen:
(b).
It costs about 3 ms less per landing than (a) on this index,
and it also catches damage that has nothing to
do with unwritten data.
FNV-1a is specified by two constants,
needs no dependency (`sha1` and `sha2` are in the lockfile
only through the scanner;
a direct dependency would go through the catalog and `file-enforcer`),
and does not change
across Rust versions as `std::hash::DefaultHasher` may.
It detects damage;
it does not resist someone who edits the file
on purpose,
which is outside what these records defend against.
The accepted cost:
after a power loss that persisted
`landing-1.json` and the moved branch but not a snapshot,
the landing is not completed (remedy (a) would have completed
it).
Recovery fails closed with a message that names the snapshot,
the landing record and the directory and lists the
steps that finish the commit by hand.
Regenerating the post-index from `landedTreeOid` was not attempted.

How it works:

- `landing-<n>.json` ends with `preLandingDigest` and `postDigest` (`{ "length", "fnv1a64" }`).
  A record without them
  (written by an older build) reads as having none and recovery trusts its identities as before;
  no such record is
  verified,
  which is the remaining exposure for a transaction left by an older binary across an upgrade.
- Recovery verifies both snapshots on the install path only,
  after the `index-installed` marker and the "real index
  already equals the post-index" checks,
  and before the comparison of the real index with the pre-landing snapshot.
  Verifying earlier would turn every `index-installed|post-1.index|*` and `index-installed|pre-landing-1.index|*` case
  from completes into sticks,
  because a landing that already installed needs neither snapshot.

Outcome table changes (247 cases regenerated;
exactly these two lines differ,
every other cell is identical):

- `ref-updated|cli-git-transactions/<id>/post-1.index|empty`:
  `exit-128!commit-exit-1` becomes `sticks`.
- `ref-updated|cli-git-transactions/<id>/post-1.index|half`:
  `exit-128!commit-exit-1` becomes `sticks`.

The `pre-landing-1.index` cases at `ref-updated` (empty,
half and delete,
all `sticks` before) stay `sticks`;
for empty
and half what changed is the message,
which now says the snapshot differs from the recorded length and hash instead
of claiming the real index changed.
The totals become:
corrupt real index 2 to 0,
sticks 63 to 65.

Guards shown failing:
with the comparison in `verify_snapshot` forced to accept,
`damage_at_ref_updated` fails at the
emptied `post-1.index` case with `fatal: .git/index: index file smaller than expected` (left `sticks`,
right
`exit-128!commit-exit-1`),
and three unit tests fail (`a_torn_snapshot_is_never_installed`,
`a_snapshot_that_differs_from_its_digest_is_refused_by_name`,
`a_normalization_names_the_branch_it_left_alone_and_a_missing_snapshot_names_the_file`).
With `postDigest` no longer
written,
`damage_at_ref_updated` fails the same way.
All passed again after restoring.

### J3 follow-up 2: the two silent-inconsistency cases

Cause,
read from `recover_dead_transaction` in `recovery_landing.rs`:
a directory without `preparing.json` was removed
as "stopped before capture",
and a directory with `preparing.json` but no `landing-<n>.json` was discarded as "never
landed",
whatever else it held.
At `ref-updated` both deletions left `ref-updated.json` (and,
for the second,
the
landing record) in place,
so recovery removed the directory of a commit that was already on the branch and exited 0
with the index never reconciled.

Detection is possible without any extra sync,
because the journal writes `preparing.json` first,
`landing-<n>.json`
before the compare-and-swap and `ref-updated.json` after it,
each synced before the next.
`recovery_contradiction.rs`
now refuses to discard,
and fails closed with the evidence,
the directory and the by-hand steps,
when:

- `preparing.json` is missing and the directory holds a landing record or `ref-updated.json`;
- `preparing.json` is present,
  no landing record exists and `ref-updated.json` exists;
- `preparing.json` is present,
  neither record exists,
  and the target's reflog holds this transaction's nonce entry
  (this also finds a move whose marker was lost together with the landing record).
  It costs one `git reflog show` in the
  recovery of a dead transaction that has no landing record,
  a crash residue and never a normal commit.
  A reflog that
  cannot be read is treated as no evidence,
  so a repository that does not keep reflogs for its branch is not covered.

An `index-installed` marker exempts the directory (the install finished,
nothing remains to reconcile),
which keeps
`index-installed|landing-1.json|delete` and `index-installed|preparing.json|delete` at `completes`.

Repair was judged not possible:
the landing record held the post-index identity and digest,
the pre-landing identity,
the real index lock identity and the expected old value,
and `preparing.json` held the target,
the real index path
and the shadow path.
Without the first,
recovery cannot tell which file is the post-index,
whether it is intact,
or
whether the `index.lock` on disk is the transaction's;
without the second it does not know the branch or the index.
Regenerating the post-index from the branch tip was not attempted.
What cannot be detected:
a directory that lost
`preparing.json` and every landing record and marker (nothing in it shows the move),
and a branch whose reflog is off
or unreadable.

Outcome table changes (247 cases regenerated;
exactly these two lines differ,
every other cell is identical):

- `ref-updated|cli-git-transactions/<id>/landing-1.json|delete`:
  `inconsistent!commit-exit-0` becomes `sticks`.
- `ref-updated|cli-git-transactions/<id>/preparing.json|delete`:
  `inconsistent` becomes `sticks`.

Totals after follow-ups 1 and 2:
silently inconsistent 2 to 0,
corrupt real index 2 to 0,
sticks 63 to 67.

Guards shown failing:
with the check made to return at once,
`damage_at_ref_updated` fails at
`ref-updated|cli-git-transactions/<id>/landing-1.json|delete` (left `sticks`,
the old cell right) and three of the four
`recovery_contradiction` tests fail;
with the install-marker exemption disabled,
`installed_and_unmoved_transactions_are_discarded_as_before` fails.
All passed again after restoring.

### J3 follow-up 3: the messages of unreadable records

Cause:
the strict parsers (`open_record` in `transaction_journal_parse.rs`) know only the bytes,
so their sentence named
the kind of record ("Transaction record landing record is not a schema-version-2 landing record") and no path.
The
fetch hint came from the gate:
`transaction_gate.rs` renders every recovery stop as the engine failure
`content-unavailable`,
whose generic hint (`event_hints.rs`) tells the person to fetch history.

What changed:

- `recovery_record_failure.rs` wraps the parser's failure at the four call sites (`preparing.json` in
  `recovery_landing.rs`,
  `landing-<n>.json` there,
  `ref-updated.json` in `recovery_completion.rs`,
  `index-lock-<n>.json` in `recovery_evidence.rs`).
  The message names the transaction,
  the file path and the directory,
  keeps the parser's sentence as the detail,
  and gives the way out per record:
  remove `preparing.json` (the wrapper then
  discards a transaction that holds no landing record or marker;
  an `index.lock` may remain and is removed by hand);
  restore `landing-<n>.json` from a copy,
  or finish by hand;
  remove `index-lock-<n>.json` (the wrapper then handles the
  transaction from its other records and leaves the lock file);
  remove `ref-updated.json` (the wrapper then reads the
  branch's reflog to decide whether the commit landed).
  The by-hand steps (`recovery_manual_steps.rs`) are shared with
  the snapshot and contradiction messages of follow-ups 1 and 2:
  check that no Git command runs,
  remove the lock,
  the
  shadow repository and the transaction directory,
  then `git status` and,
  if needed,
  `git reset` (which unstages other
  staged changes and leaves the worktree files alone).
- The wire code stays `content-unavailable` (its documentation covers transaction state,
  and `SPEC.md` ties that code to
  exit status 2),
  so a consumer matching on the code is unaffected.
  The gate now renders the failure with no `hint`
  field (`render_engine_failure_with_hint` in `diagnostics.rs`),
  because the message itself carries the ways out.
  This
  applies to every recovery stop,
  including the owner-record messages,
  which already named a path.
- The outcome table is unchanged (regenerated:
  247 cases,
  0 changed);
  only text differs.
- Declined:
  the same wording for `captured.json` (`capture_records.rs`),
  which is not in the named list and is read by
  replay,
  not by recovery.

Tests:
`recovery_record_failure_tests.rs` (every kind names file,
transaction and directory and a way out;
recovery
wraps each of the four reads),
`transaction_gate_tests.rs` (no hint field and no fetch text),
and
`the_messages_of_damaged_records_name_the_file_and_the_ways_out` in the tearing harness,
which kills real commits,
damages `preparing.json`,
`index-lock-1.json`,
`landing-1.json` and `ref-updated.json` (emptied and halved) and reads
the standard error of two repeated commands (exit 2 both times).
Shown failing by making `name_record_failure`
return the parser's failure unchanged (both unit tests and the harness test fail) and by giving the gate a hint again
(`gates_follow_recovery` fails);
restored,
all pass.

### J4 outcome: the benign syncs, the shadow `packed-refs` sync and four directory syncs dropped

Decision by the owner:
"Real gain real risk.
We're high rollers."
The three removals ranked first in
`doc/handover/wrapper-sync-attribution.md` are done,
each in its own commit,
and the exposure each one accepts is
recorded here.
A kill leaves the page cache intact,
so no test under "J4 outcome" is a power loss;
the harness damages files to
stand for one.

Nine syncs of records the J3 table shows to be benign (commit `664820231f`):
`captured.json`,
`hooks/plan.json`,
`prepared.json` (file and directory),
`index~pid.lock`,
the landed record (file and directory) and the `index-installed`
marker (file and directory).
They are still published whole by link,
so a kill leaves them whole or absent;
they are
no longer flushed.
The writers are `write_private_file_unsynced` and `write_exclusive_unsynced` in `private_storage.rs`,
and `journal_record.rs` names the level of a journal record (`RecordSync`).
The J3 table is unchanged (247 cases pass
against the recorded outcomes).

The shadow repository's `packed-refs` (commit `782e1cd2cd`):
the sweep in `binary_recovery_tearing_shadow_tests.rs`
empties,
halves,
zero-fills and deletes the file at all six phases and requires the recorded undamaged outcome each
time.
With recovery temporarily made to read the file (`remove_shadow_repository` refusing a file that does not start
with the pack-refs header) all six phase tests fail at the first damage,
`empty`,
with the engine failure
`content-unavailable` and the table word `sticks` against `discards` or `completes`;
restored,
all six pass.
The files
backend only:
on reftable the shadow's refs are read by `is_store_held` and the write stays a Git transaction.

Four directory syncs (commit `62a4642ec8`):
the transaction directory after `preparing.json`,
`index-lock-1.json` and
`ref-updated.json`,
and the `.pending` candidate directory.
The directory sync of `landing-1.json`,
which comes before
the compare-and-swap,
is now the first sync that makes the names in the transaction directory durable (the
`owner.json` name included);
the file syncs of those records stay.
The lost-name tests are in
`binary_recovery_tearing_names_tests.rs`.
Each deletes the record the way a lost directory entry would,
at every phase
before the barrier where the loss is reachable:

- `preparing.json` lost:
  discards at `capture-locked` and `preparation-done`,
  and "discards,
  leaves `index.lock`,
  the
  next commit exits 2" at `landing-locked` and `objects-migrated`.
  The outcome equals a state built as if
  `preparing.json` was never written (only `owner.json` left in the transaction directory,
  no shadow).
- `index-lock-1.json` lost:
  "discards,
  leaves `index.lock`,
  the next commit exits 2" at both phases that hold the lock.
  The outcome equals the state of a kill between creating the lock and writing its record (the record and the PID file
  both absent).
- `ref-updated.json` lost:
  completes at `ref-updated` and `index-installed`,
  because `landing-1.json` is durable.
- `owner.json` lost (the candidate directory sync dropped,
  and the sync after `preparing.json` that used to persist the
  same directory):
  does not converge.
  Every guarded command stops with exit 2 and
  "Transaction owner record is missing: <directory>" at every phase before the barrier,
  and the person has to remove the
  directory by hand.
  This is the row `T/owner.json: all applicable damages sticks` of the J3 table made reachable by
  the loss of one directory entry.
  It requires a filesystem that persists the rename of the published directory and the
  contents of `owner.json` but not the name of `owner.json` in it,
  which file sync does not promise by POSIX but ext4,
  XFS and btrfs do in practice;
  no such state was produced on a real filesystem,
  only by deleting the file.
  The recovery code was not changed:
  treating a published directory without an owner record as dead would make
  recovery remove a directory whose owner it cannot identify,
  and the fail-closed text is pinned by
  `published_owners_must_be_valid` in `recovery_inspect_tests.rs`.
  Open:
  whether to accept that,
  make recovery remove
  such a directory when it holds no landing record,
  or put the candidate directory sync back (one call per commit).

Superseded by the J4 follow-up "lost names converge":
the `preparing.json`,
`index-lock-1.json` and `owner.json`
bullets and the paragraph after them describe the state when the syncs were dropped,
before recovery learned to converge
on them.
The tests `a_lost_index_lock_name_equals_a_lock_record_never_written` and
`a_lost_owner_name_pins_the_known_gap_of_a_stuck_repository_naming_the_directory` no longer exist (replaced by the
convergence controls in the J4 follow-up),
and `a_lost_preparing_name_equals_a_preparing_record_never_written` was renamed.

The "leaves `index.lock`" outcome of the first two bullets does not converge either,
and the exposure grew.
The wrapper
cannot prove the lock is its own,
so `git status` works but every commit waits `indexLock.unprovenOwnerTimeoutMs`,
stops with exit 2 and `index-lock-unproven-owner` ("Remove the lock only after confirming no Git process is using this
repository"),
and does so again on every later commit (three rounds were run) until the lock is removed by hand.
Nothing is lost and the edit stays intact.
The state is not new:
a kill between creating the lock and writing its
record leaves it today.
What changed is the window.
The directory sync after `index-lock-1.json` closed the gap
between creating the lock and the entry of its record;
the gap now stays open until the directory sync of
`landing-1.json`,
and a power loss anywhere in it can take the record's name.

Guards shown failing,
then restored:
restoring any one of the nine syncs fails the unit pin of its writer and all four
budget scenarios by one or two calls (all six file syncs at once:
30 observed against 24 budgeted on the cold commit;
all three directory syncs at once:
27 against 24);
`the_files_snapshot_is_not_synced` and the budgets fail with the
shadow sync restored (24 against 23 cold);
the budgets fail with the `preparing.json` directory sync restored (20
against 19) and with the `ref-updated.json` directory sync restored (20 against 19),
and
`a_landing_lock_syncs_its_record_file_only` and
`publication_syncs_the_owner_file_and_the_registry_and_not_the_candidate` fail with the lock record's directory sync
and the candidate sync restored.
For the lost-name tests,
making recovery refuse a missing `preparing.json`,
a missing
`ref-updated.json` or a lock record missing while a PID file exists fails
`a_lost_preparing_name_equals_a_preparing_record_never_written`,
`a_lost_ref_updated_name_still_completes_the_landing` and
`a_lost_index_lock_name_equals_a_lock_record_never_written`.

Wrapper syncs of the budget scenarios,
before and after (the Git children's syncs are unchanged):

- Cold commit:
  33 to 19 wrapper syncs (36 to 22 with the 3 of `index-pack`).
- Warm commit (second wrapped commit,
  a scenario added before the removals):
  28 to 14 wrapper syncs (35 to 21 with the
  3 of `index-pack`,
  the 3 of `pack-objects` and the 1 of `multi-pack-index write`).
- Commit with one configuration hook (cold):
  37 to 23.
  Amend (cold):
  33 to 19.

Not changed:
`write_prepared_record` in `commit_preparation.rs` still syncs;
it is called only by
`prepare_native_commit`,
which no production path calls.
The migration `index-pack` syncs (3 per commit) are another
change.

### J4 follow-up: lost names converge

The owner accepted the durability risk and refused to put the syncs back;
the requirement that stays is that the wrapper
never leaves a repository stuck.
Recovery now converges on `preparing.json`,
`index-lock-1.json` and `owner.json` without a new sync,
a new spawn
or a new syscall on a commit that finds its state whole:
every new check runs in the branch that already found a dead
or ownerless transaction.
The budget scenarios pass at their recorded counts,
and 243 of the 247 cells of the J3 table
are unchanged.

Rule A,
a published directory without `owner.json` (`ownerless_directory.rs`).
The directory is abandoned,
and
recovered exactly like a dead owner's,
when two things hold:
the newest modification time of the directory and
everything in it is older than `FOREIGN_OWNER_PATIENCE` (3600 seconds,
the constant `pid_namespace.rs` already uses for
an owner whose PID cannot be probed),
and no other process has the directory or a file in it as a working directory or
an open descriptor (`process_use.rs`,
a pass over `/proc`).
Otherwise recovery fails closed as before,
and the message
names the directory,
the reason (the age of the newest entry,
or the process that uses it),
that the wrapper recovers
the directory by itself after sixty minutes of quiet,
and the one removal by hand that is safe:
`rm -r <directory>`
after confirming no cli-git or git command runs,
and only when the directory holds no `landing-<n>.json`,
because with
one the branch may have moved and removal skips the index installation that recovery performs.

The safety argument for rule A is an age bound,
not a proof about one process.
No record survives that names the
writer:
no other file in the directory holds a PID,
the scratch directory's name holds one only at some phases and
without the PID namespace,
and the registry locks are held by whoever works now.
A running wrapper cannot lose its own
`owner.json` name (the loss needs a crash,
and a crash ends every process that ran),
so the rule can free a live
wrapper's directory only if someone removed `owner.json` by hand and the wrapper then sat idle for the whole patience
with no file open.
That is the same trade the wrapper already makes for an owner of another PID namespace.
A recovered
directory is ordered after every attributed one (its creation time is unknown),
and no PID file is matched for it.
A
directory with a fresh file in it,
a directory a process uses,
and a directory dated in the future all stay refused.

Rule B1,
`preparing.json` lost with a lock record present (`recovery_unrecorded_lock.rs`).
Recovery used to read the
missing `preparing.json` as "stopped before capture" and remove the directory,
destroying the lock record that proves
which `index.lock` was the owner's.
It now releases the recorded locks first,
through the worktree's own `index.lock`
(the registry's parent is the Git directory;
a lock elsewhere does not match the recorded device and inode and stays),
and removes the transaction's pack keeps from the common object directory.
The proof is the existing one:
the lock's
device,
inode and filesystem identity equal the record,
and the landing lock is held because a lock record counts as
having entered the landing section.

Rule B2,
`index-lock-1.json` lost (`recovery_unrecorded_lock.rs`).
With no lock record,
the dead owner's PID file
(`index~pid.lock`) is the one fact that links the lock to the owner.
The lock is released only when all of these hold:
the PID file reads exactly `pid <owner>` for the dead owner of the transaction directory;
the owner record is readable
and its PID namespace is this one (a number from another namespace names nothing here);
no process with that number
exists,
so a reused number is refused;
the lock is a regular empty file (nothing was written into it);
the lock's
modification time is not before the owner record's and not after the PID file's,
which a lock another process took
after the owner released its own fails;
no process whose descriptors can be read holds the lock open (matched by
device and inode,
so a second name does not hide it);
and the landing lock is held,
because the PID file counts as
having entered the landing section.
The identity of the lock is read again just before the removal.
Every refusal logs
a warning that names the missing proof.

Named limits of rule B2,
none hidden:

- The open-file scan cannot read the descriptors of a process of this account that holds capabilities (`systemd --user`
  is one on this host:
  the first sweep after the rule was written refused the release for that reason,
  so an unreadable
  process no longer blocks it).
  Such a process is reported in
  the debug log and does not keep the lock.
  A Git or wrapper that holds the lock yet cannot be read is not a case the
  proofs leave open,
  because the other proofs show who created the lock and neither hands a lock on,
  but the scan is
  the weakest of them.
- The dates are file system times,
  with the granularity of the file system;
  an equal date passes.
- A lock of unknown origin (no PID file,
  or a PID file that names someone else,
  or a lock another process took) still
  stops every commit.
  A power loss that also loses the PID file's name,
  or the lock records and the owner record
  together,
  leaves that state.
  The message names the exact `rm` command,
  `lsof` as the check,
  and the condition under
  which removal is safe (no process holds the lock,
  and the PID file beside it,
  if any,
  names a process that no longer
  runs).
- Where `/proc` cannot be listed (macOS,
  Windows) the scan reports the process table as unavailable and rule B2 keeps
  the lock (the "no process table" unit case);
  rule A still rests on its age bound alone there,
  because the
  process scan is only a veto for it.
  This host is Linux;
  the non-Linux behavior is not exercised.
- A real file system may lose every name created since the last durable directory block at once.
  The tests lose one
  name at a time,
  as the sweep always did.

Changed cells of the J3 table,
four of 247,
all improvements,
none worsened:

- `landing-locked|T/preparing.json|delete`:
  from "discards+residue!commit-exit-2" to "discards".
- `objects-migrated|T/preparing.json|delete`:
  from "discards+residue!commit-exit-2" to "discards".
- `landing-locked|T/index-lock-1.json|delete`:
  from "discards+residue!commit-exit-2" to "discards".
- `objects-migrated|T/index-lock-1.json|delete`:
  from "discards+residue!commit-exit-2" to "discards".

The `owner.json` delete cells stay "sticks" in the sweep because the sweep damages a directory that is seconds old;
the
same loss in a directory aged two hours converges at all six phases (`a_lost_owner_name_in_an_aged_directory_converges`:
"discards" before the branch moves,
"completes" after).

Controls,
in `binary_recovery_tearing_converge_tests.rs`:
a fresh directory with a lost owner name stops with the exact
message;
an aged one with a sleeping process whose working directory is in it keeps the stop and names the process,
and
converges once the process is gone;
a lock of unknown origin keeps the stop and the message names `rm`,
`lsof` and the
safety condition;
a sleeper that holds the lock open,
a sleeper whose number the owner record and PID file name with a
different birth identity,
a lock dated after the PID file,
and a live creator (true birth identity) each leave the lock,
the PID file and the directory as they were.

Guards shown failing,
then restored (each breaks one rule,
runs the unit and binary controls,
restores the line):

- Rule A age check removed:
  three ownerless unit controls,
  `published_owners_must_be_valid` and the binary
  `a_lost_owner_name_stops_a_fresh_directory_and_names_the_way_out` fail.
- Rule A use scan removed:
  `a_directory_in_use_is_unproven_even_when_old`,
  `a_live_process_inside_an_old_directory_keeps_it` and the binary `a_live_process_in_an_aged_directory_keeps_the_stop_until_it_is_gone` fail.
- Rule A abandonment turned into a refusal:
  `a_lost_owner_name_in_an_aged_directory_converges` fails.
- Rule B1 disabled:
  `damage_at_landing_locked`,
  `damage_at_objects_migrated` and
  `a_lost_preparing_name_leaves_nothing_and_a_never_written_one_releases_the_announced_lock` fail.
- Rule B2 disabled:
  seven binary controls fail (`damage_at_landing_locked`,
  `damage_at_objects_migrated`,
  `a_lost_index_lock_name_converges`,
  `a_lost_preparing_name_leaves_nothing_and_a_never_written_one_releases_the_announced_lock` and the three
  keep-the-lock controls,
  which then find nothing to keep).
- Rule B2 without the process table check:
  the "no process table" case of
  `a_missing_proof_keeps_the_lock_and_names_the_proof` fails.
- Rule B2 without the process-number check:
  `a_reused_process_number_keeps_the_lock` and the unit case fail.
- Rule B2 without the open-file veto:
  `a_process_holding_the_lock_open_keeps_it` and the unit case fail.
- Rule B2 without the date order:
  `a_lock_newer_than_the_pid_file_keeps_it` and the unit case fail.
- Rule B2 without the PID file match:
  the unknown-origin binary control and two unit cases fail.

Open value call for the owner:
rule A waits sixty minutes of quiet (`FOREIGN_OWNER_PATIENCE`,
reused so one constant
serves both cases),
so a fresh ownerless directory stops every guarded command for up to an hour,
with the rule A message naming the manual way out.
A shorter bound (for example tied to `CLI_GIT_LOCK_WAIT_SECONDS`,
30 minutes by
default) converges sooner and widens the window in which a live wrapper whose `owner.json` was removed by hand and that
sat idle without a file open could lose its directory;
the choice trades convergence time against that window.

Remaining stuck states,
none hidden:

- Rule B2 refuses when any process carries the dead owner's number,
  because only the existence of the number is
  checked,
  not a start time (the PID file holds none).
  On a busy host the number can be reused soon;
  the lock then
  stays until it is removed by hand with the command the message names.
- A lock of unknown origin (see the unknown-origin limit under `Named limits of rule B2`).
- A `.pending` staging candidate whose `owner.json` name was lost is kept in place and reported as live
  (`StagingUnattributed`).
  This is not a stuck case:
  `Gate::Live` is not a refusal at any of its call sites
  (`wrapped_command.rs`,
  `index_writer_coordination.rs`,
  `management.rs`),
  which all continue on `Clear | Live`.
- The recovery order for an abandoned directory (after every attributed one,
  its creation time being unknown).

Not run:
the container gate,
the Windows clippy target,
and a real power loss.
In the full host suites the unit test
`markdown_linter::tests::the_real_linter_meets_the_contract` and the binary
`markdown::the_real_linter_rewrites_through_add_check_and_fix` fail because `GIT_POLICY_NATIVE_TEST_LINTER` is unset (the
gate image sets it),
and `fault_remote::an_auto_push_with_a_git_lfs_that_never_exits_is_stopped_with_everything_it_started`
failed once under load and passed on its own;
every other test passed (2375 of 2376 unit,
349 of 351 binary).

## Results of the hook-entry and configuration-snapshot slices

Method:
`strace --follow-forks -e trace=execve,fsync,fdatasync,syncfs,sync_file_range` over one wrapped
`git commit --quiet --message=x -- a.txt` of the debug build in a fresh repository with the host's Git 2.56.0,
counting `execve` of `/usr/bin/git` (equal to the `| start ` lines of `GIT_TRACE2_PERF`,
52 in both) and sync calls;
scratch script `measure.ts <binary> <scenario>` beside the repository's `attribute-commit.ts`.
Before is the commit that preceded these slices;
after is the branch tip.

- No hooks:
  52 to 43 Git starts,
  42 to 38 sync calls,
  three wrapper re-executions as dispatcher entries to none.
- One `pre-commit` hook:
  52 to 44 Git starts,
  42 to 39 sync calls.
- All four hooks:
  52 to 48 Git starts,
  42 to 42 sync calls (the configuration reads are the only saving).
- A configuration-defined hook:
  52 to 48 Git starts,
  42 to 42 sync calls.
- A custom `core.hooksPath` holding one `commit-msg` hook:
  52 to 44 Git starts,
  42 to 39 sync calls.
- `measure-fsync.ts` (an `--allow-empty` commit,
  no hooks):
  41 to 37 sync calls.

Permanent budget:
`binary_hook_shim_tests.rs` fails a commit with no hooks that starts any hook runner or more than
one `git config`,
a commit with four hooks that does not start exactly four runners,
and a commit with one hook that
does not start exactly one.

Known deviations,
each accepted for the saving and each recorded here because it is not byte-for-byte what separate
reads gave:

- The snapshot is read when the commit begins,
  so the maintenance decision no longer sees a change that a hook or
  another process makes to `maintenance.auto`,
  `gc.auto` or the detach keys during the commit.
  Example:
  a
  `pre-commit` hook that runs `git config maintenance.auto false` used to stop that commit's maintenance run;
  it no
  longer does.
  The hooks path and the switches were read just before preparation and are now read earlier by the
  length of the policy pass.
- An entry is omitted from a plan-time look at the hooks directory,
  so a `pre-commit`,
  `prepare-commit-msg` or
  `commit-msg` hook created between planning and the native commit (milliseconds) does not run.
  `post-commit` is
  looked up when it comes due and has no such gap.
- A named hook (`hook.<name>.event`) is found from the real repository's configuration,
  while the dispatcher's
  `git hook run` sees the shadow's configuration,
  which includes the real one.
  They differ only if a conditional
  include matches the shadow's path and not the real one,
  for example a `gitdir:` pattern that names the shadow
  directory;
  that include would define a hook the wrapper no longer starts.

Found while probing and left unchanged:
the dispatcher skips a user hook of an event with `hook.<event>.enabled=false`,
while Git 2.56.0 runs a traditional hook whatever that switch says (it disables only hooks defined by name).
`a_disabled_event_with_a_hook_is_still_skipped_by_the_wrapper` pins the wrapper's existing behavior.

Refused,
with the counterexample:

- Using the snapshot for the ref storage whenever it names `extensions.refStorage`:
  with
  `core.repositoryformatversion = 0` Git fails (`repo version is 0, but v1-only extension found`),
  and with the key
  only in an included file Git prints `files`;
  the repository-format rules are Git's,
  so the key's presence asks Git.
- Modelling `1k`,
  `-1`,
  `007`,
  `0x10` and `08`:
  `--type=bool` gives `true`,
  `true`,
  `true`,
  `true` and a fatal error;
  `--type=bool-or-int` gives `1024`,
  `-1`,
  `7`,
  `16` and a fatal error;
  the numeric grammar is `strtoimax` base 0
  with unit suffixes,
  so only `0` and digit strings from `1` to `9` are decided in Rust.
- Reusing the snapshot for the consumers that ignore global options when the read carried a `-c`:
  the original reads
  would not have seen the option,
  and a `-C` or `--git-dir` changes which repository is listed.

Not done:
the `git config --file config --list` of the shadow stays when the configuration includes another file;
the `config --get` reads of the auto-push (`remote.<name>.push`,
`commit.gpgsign`) are outside the commit path
measured here;
a commit made with global options (`git -c k=v commit`) still starts one read each for the ref
storage,
the maintenance keys and the shadow format,
because those reads ignore the options or run elsewhere.

## Budgets

The host and gate suites carry a spawn budget (from `GIT_TRACE2_PERF`) and the host suite a sync budget (from
`strace`);
each slice lowers its budget to the measured count.
Starting point:
45 processes,
41 syncs.

The ratchet tests are `src/native/binary_budget_tests.rs` (the constants and the four scenarios) and
`src/native/binary_budget_trace.rs` (the `strace` reader) in `package/git-policy/cli`.
Run them with
`mise run //package/git-policy/cli:native:test:binary:host -- budget::`.
A wrapped explicit-path commit
(`git commit --quiet --message=x -- a.txt`) runs under `strace --follow-forks` on a disposable repository in four
scenarios:
no hooks,
one hook defined in the configuration (`hook.<name>.event` and `hook.<name>.command`),
`--amend`,
and the second wrapped commit of a repository (warm,
where Git's maintenance repacks and the creation syncs of a first
commit do not happen).
After the J4 removals the wrapper makes 19 syncs in a cold commit and 14 in a warm one.
The tests count every `execve` of Git by subcommand,
every start of another program,
and every `fsync`,
`fdatasync`,
`syncfs` and `sync_file_range` call,
attributed to the wrapper process (threads included) or to the Git
subcommand that made it.
They compare exact counts with the constants,
so a rise and a saving both fail,
naming each
figure that moved,
and the constants are lowered in the same change that makes a commit cheaper;
a rise needs a recorded
reason.
Zero entries (`findmnt`,
`hash-object`) keep removed starts from returning.
Background maintenance is kept in
the foreground (`maintenance.autoDetach=false`,
as `keep_maintenance_in_the_foreground` does),
so in the cold scenarios
a landing's `maintenance run --auto` is one counted start with no repack child;
the warm scenario counts the `repack`,
the `multi-pack-index write` and the three `pack-objects` the maintenance of the second commit starts.
The `stat`
figure depends on the temporary directory's filesystem (three `stat --file-system` starts without a filesystem UUID,
such as tmpfs or overlayfs,
none
with one) and the test adjusts for it.
A host without `strace` skips the tests with a printed message.
Counts were
measured with the host's Git 2.56.0,
the version the gate builds;
the gate run is the authority.

Lowered after this section was written:
the `init` start of the shadow repository,
one start in each of the four
scenarios,
is now zero.
The wrapper creates the entries of an empty bare repository on the files ref backend itself
(`src/native/shadow_skeleton.rs`,
compared with real `git init --bare --template=` by its own test),
and starts
`git init` only for the reftable backend.
The `read-tree` count is unchanged (it now runs `--reset` over a copy of the
real index);
the reason and the timings are in `doc/planning/cli-git-commit-timeouts.md`.

Lowered again on 2026-11-03:
the `read-tree` start of a commit whose index already equals its base is zero in the warm,
monitor-on and amend scenarios (the cold scenarios commit on an unborn branch and keep their one start).
The private index is
the copy of the real index,
and the in-process `diff-index --cached` reader says the copy holds exactly the base tree;
a
question the readers leave to Git takes the old way,
with the start.
The reasons and the numbers are in
`doc/planning/cli-git-main-commit-within-native-plus-five.md`.

Lowered again on 2026-11-08:
the `update-index` start of the stat-data refresh is zero unless the real index no longer describes the worktree.
The refresh `cd1755d728` added to keep the scratch commit inside its ceiling ran
`git update-index --refresh` on the real index before the private copies were made,
which is one start in every scenario,
and Git takes `index.lock` on every such run even when it writes nothing;
the six constants were left at the counts measured before it,
so `budget::` failed six times on main's tree.
`src/native/commit/commit_index_stat_staleness.rs` answers the question in process from the index's bytes,
so an index that describes the worktree starts no Git and takes no lock,
and the counts under "Budgets" hold as they were measured before the refresh existed.
The measurements,
the reasoning and what is still to do are in
`doc/handover/cli-git-rust-implementation.md`,
"The refresh of a stale index asks for Git and the real index's lock only when it is stale".

Added and then removed on 2026-11-10:
`check-ignore`,
one start in every scenario,
was never in the constants,
so six controls failed the release gate at `483bbc6587`
(`a_commit_with_no_hooks_stays_within_its_budget`,
`a_commit_with_a_configured_hook_stays_within_its_budget`,
`a_commit_with_a_worktree_config_costs_what_one_without_costs`,
`a_commit_with_core_fsmonitor_true_costs_what_one_without_costs`,
`a_warm_commit_stays_within_its_budget` and
`an_amending_commit_stays_within_its_budget`,
all reading "starts of git check-ignore:
observed 1,
budget 0 (not in the budget)").
The question behind that start,
which a commit of a named ignored path asks before its private staging,
costs nothing now for a commit that cannot hit it:
`git add` refuses a named path as ignored only while the real index holds no entry at it,
so `src/native/commit/commit_named_ignored_files.rs` reads the real index in process
(`read_layout_in`,
then `read_regular_file` and `parse_index`)
and asks `git check-ignore -z --stdin` only about the names no entry holds,
starting nothing when there are none.
Every scenario here commits a tracked `a.txt`,
so the four constants carry `check-ignore` 0,
and a start of one is named by the failure.
A named path no entry holds still asks exactly one question,
which is what the refusal controls see;
`commit_named_ignored_files_tests.rs` proves both halves with a recording Git child,
and the reader that declines an index leaves every name to Git,
as before.
The reader declines one more case since 2026-11-10:
a command line that selects a repository (`--git-dir`,
`--work-tree`,
`--bare`)
names an index whose paths are spelled from that repository's own worktree,
while the layout reader answers only for the directory the command runs in,
so a prefix from the wrong worktree could match an unrelated entry and hide a name a person named.
There the reader leaves every name to Git,
as `capture_invocation_layout` already does for the prefix it reads.
The four constants name no repository,
so their `check-ignore` stays 0,
and a commit whose command line names one repository asks one question about its named paths.

Raised on 2026-11-10:
`wrapper_syncs` from 0 to 2 in all four constants,
for the verdict of a commit's own post-commit gate.
Since `77e8348a84` the wrapper writes what its gate decided into the push record the service held for the commit before
the branch moved
(`package/git-policy/cli/src/native/landing/service_handoff.rs`,
`promote_the_push`,
which calls `record_pass` in
`package/git-policy/cli/src/native/landing/work_item_holding.rs`),
and `private_storage::replace_private_file` makes that durable as one sync of the staged record and one of the folder
that holds it.
The write comes before the promotion because a command killed in that gap leaves a record whose own words say what it
decided,
which a later command settles from them
(`held_push_holder_and_verdict.rs`);
without the sync those words could be lost with the page cache.
Measured on 2026-11-10 with the harness of this section:
the no-hooks control,
with its constant still at 0,
failed with
`sync calls by the wrapper: observed 2, budget 0`,
and all six `budget::` controls pass with `wrapper_syncs` 2,
so every scenario reads exactly 2,
and the comparison reports a wrong number as well as a right one.
Those two calls are all the sync-class calls of a commit:
the kept `strace` of a cold commit holds them and no `fsync`,
`fdatasync`,
`syncfs` or `sync_file_range` of any Git child.
The counts this section states earlier,
19 syncs in a cold commit and 14 in a warm one,
are therefore historical;
a commit makes two now.
Leaving the verdict in the page cache instead would save both,
and it is a durability question for the owner:
a record whose verdict was lost keeps the commit local,
which is the fallback the change already names.

## Filesystem identity without `findmnt` (slice S2)

Scope:
the three `findmnt` starts of a commit (two on `index.lock`,
one on `.git`),
all made by
`package/git-policy/cli/src/native/filesystem_identity.rs`.
The `stat --file-system` fallback that follows an empty
`findmnt` answer is not part of this slice.

### What must stay identical

Read from `filesystem_identity.rs`,
`real_index_lock.rs`,
`recovery_files.rs` and the incumbent
`package/fs-id/core/src/platform-resolvers.ts`,
`parsers.ts`.

- **The recorded string.**
  `filesystem_identity(path)` canonicalizes `path`,
  then on Linux returns
  `fs-uuid_<payload>` where the payload is the `findmnt --target <canonical> --output=UUID --noheadings` standard
  output,
  trimmed with JavaScript `.trim()` semantics,
  lowercased,
  and accepted only if it is 1 to 512 characters of
  `[a-z0-9.-]` and not a lone `-`.
  Example:
  a vfat UUID `44C8-C6BA` is recorded as `fs-uuid_44c8-c6ba`.
- **The empty case.**
  An empty,
  multi-line or unsafe payload (`findmnt` prints `\n` for `/`,
  `/tmp`,
  `/proc`,
  `/dev`,
  fuse mounts and `\n\n` for two mounts stacked on one mount point) is not an identity.
  The resolver then runs
  `stat --file-system --format=%i` and records `f-fsid_<hex>`.
  A string is therefore never empty;
  `None` only when both
  commands fail.
- **Where it is recorded and compared.**
  `created_identity` (`real_index_lock.rs`) records it for the directory
  holding the real index into the journal field `fsId`.
  `owned_lock_mismatch_with` and `release_owned_lock_with`
  (`recovery_files.rs`) compare it,
  for the lock file,
  with the recorded `fsId` together with device and inode.
- **How a mismatch is treated.**
  `None` from `filesystem_identity` is an error,
  never a mismatch:
  the landing stops
  ("the filesystem identity of the worktree is unavailable") and recovery returns a `RecoveryError`.
  A string that
  differs from the journal is "Index lock identity changed",
  the lock is reported `Foreign` and is left in place.
  A wrong string therefore leaves the owner's own lock in place as foreign and stops recovery from releasing it;
  the
  in-process path must never produce a string `findmnt` would not.
- **Journals outlive binaries.**
  The record is read by older builds of this wrapper and by the TypeScript incumbent
  (`fsId` in the journal JSON).
  The new code must produce the byte-identical text the old code and `fs-id-core`
  produce for the same filesystem:
  same prefix,
  same lowercase,
  same fallback to `f-fsid_`.
- **Mount namespaces and containers.**
  `findmnt` is a child of the wrapper,
  so it reads the wrapper's own
  `/proc/self/mountinfo`,
  root directory and `/run/udev`.
  The in-process code reads the same files in the same
  process,
  so every namespace question has the same answer.
  A container without `/run/udev` data for the device gets
  no UUID in-process,
  and the code spawns `findmnt` exactly as before;
  a host-recorded `fs-uuid_` and a
  container-computed `f-fsid_` already differ today,
  and that does not change.

### How `findmnt` 2.41 gets the text (read from util-linux `misc-utils/findmnt.c`, confirmed by `strace`)

- `--target X` first lists every `/proc/self/mountinfo` line whose mount point equals `X`;
  if none,
  it walks `X` up
  one component at a time (`mnt_table_find_mountpoint`,
  backward,
  so the last line wins) and then lists every line
  whose mount point equals the one found.
  Stacked mounts therefore print several lines.
- The UUID column resolves the source field with `mnt_resolve_spec`,
  canonicalizes it,
  and asks libudev for the block
  device by sysfs name (`/sys/class/block/<name>`),
  which reads `/run/udev/data/b<major>:<minor>` and returns
  `ID_FS_UUID_ENC` with `\xHH` sequences decoded.
  Only when that is absent does it call `libblkid`,
  which probes the
  device and needs read access.
  `strace` on this host shows `openat("/run/udev/data/b252:1")` and no access to
  `/dev/disk/by-uuid`,
  so `/dev/disk/by-uuid` is not consulted by the oracle (the brief named it;
  the oracle is the
  authority,
  and for a multi-device Btrfs the link names only one member while every member has the record).

### Exactness rules for the in-process path

- Answer only when exactly one mountinfo line has the winning mount point,
  the mount ID of that line equals the
  `stx_mnt_id` that `statx` reports for the path,
  and the source is a `/dev/` block device whose udev record holds
  `ID_FS_UUID_ENC`.
  Every other shape spawns `findmnt` unchanged.
- A source that is not a path (`overlay`,
  `tmpfs`,
  `proc`,
  `composefs`) has no UUID:
  `findmnt` prints an empty line.
  That conclusion is drawn without a spawn only if the source also names no existing path and no sysfs block device.
  The `stat` fallback still runs.

### Result

Code:
`src/native/filesystem_identity_mount.rs` (pure decisions plus one `statx` call) and the `MountUuid` match in
`linux_fs_id` (`src/native/filesystem_identity.rs`).
Assumption recorded:
`findmnt` is linked with libudev,
as the
Fedora and Debian builds are;
the differential tests fail loudly on a host where it prints something else.

Process starts of one wrapped `git commit --allow-empty` (the repository's `attribute-commit.ts`,
pointed at the debug
build of each commit through `BINARY`;
`BASE` picks the fixture directory),
run from `${HOME}/temp/agent/s2`:

- Fixture on Btrfs with a udev record,
  before (parent commit `e345bba143`):
  49 starts,
  3 of them `findmnt`.
  After:
  46 starts,
  0 `findmnt`.
  Command:
  `BINARY=<worktree>/package/git-policy/cli/target/debug/cli-git-native node attribute-commit.ts`.
- Fixture on tmpfs (`BASE=/tmp/s2-fixture`),
  before:
  52 starts,
  3 `findmnt` and 3 `stat`.
  After:
  49 starts,
  0
  `findmnt`,
  3 `stat`.
  The `stat --file-system` runs remain;
  replacing them needs the `f_fsid` word order of GNU
  `stat` and is a separate slice.
- Cost of the removed starts:
  5 to 14 ms each under `strace`;
  the in-process work is one `statx`,
  one read of
  `/proc/self/mountinfo`,
  two small sysfs and udev reads.

Differential evidence (`findmnt` is the oracle;
every non-`Unknown` verdict is compared with its exact standard
output,
and `Unknown` is printed with what `findmnt` printed):

- Host paths:
  `/`,
  `/tmp`,
  `/proc`,
  `/dev`,
  `/run`,
  `/sys`,
  `/usr` (empty line,
  `Absent`);
  `/etc`,
  `/var`,
  `/var/home`,
  the repository (Btrfs subvolumes of one device,
  `1bb3d23e-...`),
  `/boot` (ext4),
  `/boot/efi` (vfat,
  `44C8-C6BA`),
  `/var/mnt/encrypted` (device mapper source `/dev/mapper/crypt_sda`),
  a nested Btrfs subvolume whose `st_dev` differs
  from its mount's (matched through the mount ID,
  so subvolumes need no fallback),
  fuse mounts (`Absent`).
- A private user and mount namespace (`unshare --user --map-root-user --mount`,
  no host mount touched):
  bind mounts
  of a directory,
  of a directory whose name has a space,
  of a directory whose name has a backslash,
  of another
  device's directory,
  of a single file,
  of a `tmpfs` directory,
  a `tmpfs`,
  an unprivileged `overlay` (`Absent`),
  and
  two mounts stacked on one mount point.
  All equal `findmnt` or fall back.
- A rootless Podman container (`--memory=2g --cpus=2 --network=none`,
  Debian trixie,
  `findmnt` 2.41,
  no
  `/run/udev`):
  overlay root,
  `/proc`,
  `/dev`,
  `/etc/hosts` bind and `/dev/shm` are `Absent` and equal `findmnt`;
  the
  Btrfs bind whose source `/dev/mapper/crypt_sda` does not exist in the container is `Unknown` and spawns.

Classes that fall back to spawning `findmnt` (kept exact by construction,
each with its reason):

- **Stacked mounts on one mount point** (`/proc/sys/fs/binfmt_misc` here:
  `autofs` under `binfmt_misc`):
  `findmnt`
  prints one line per mount,
  `\n\n`,
  which the payload check rejects,
  so the identity comes from `stat`.
  Counterexample
  for answering in-process with the last entry:
  it would claim `Absent` for a two-line output.
- **A mount covered by a later mount on an ancestor**:
  the name walk finds the covered mount,
  `statx` reports the
  covering one.
  `findmnt` prints the covered mount's line (namespace case `cover/y/z`);
  the mount ID check makes the
  in-process answer `Unknown`.
- **A device with no udev record** (containers,
  minimal images) or whose record lacks `ID_FS_UUID_ENC`:
  `findmnt` then
  asks `libblkid`,
  which probes the device and depends on read access.
- **Sources that are paths but not `/dev/` nodes,
  `host:/path` network sources,
  tag sources with `=`,
  names that
  exist in the working directory or in sysfs**:
  `libblkid` or libudev would look at them.
- **UUID text outside letters,
  digits,
  dot and dash** (spaces,
  `\xHH` escapes that decode to anything else,
  a NUL):
  libsmartcols may re-encode on output.
- Older kernels without `STATX_MNT_ID`:
  no mount ID,
  no in-process answer.

Rejected:
`/dev/disk/by-uuid` as the source of the text (see "How `findmnt` 2.41 gets the text").
Reading
`ID_FS_UUID_SUB_ENC` (the Btrfs device UUID) instead of `ID_FS_UUID_ENC` is the natural mistake,
and `udev_records_give_one_plain_uuid`,
`in_process_uuids_equal_what_findmnt_prints` and
`identities_equal_the_command_only_resolution` catch it:
the repository path then records `87fa7a73-...` against `findmnt`'s `1bb3d23e-...`.

Guards shown failing before being kept (each break applied,
tests run,
restored):

- `ID_FS_UUID_SUB_ENC` in place of `ID_FS_UUID_ENC`:
  `udev_records_give_one_plain_uuid`,
  `in_process_uuids_equal_what_findmnt_prints` and `identities_equal_the_command_only_resolution` failed.
- Taking the last entry of stacked mounts instead of giving up:
  `the_decision_gives_up_on_unproven_shapes` and
  `in_process_uuids_equal_what_findmnt_prints` failed.
- Dropping the mount ID comparison:
  `the_decision_gives_up_on_unproven_shapes` failed (the differential tests cannot
  see it on this host,
  because the covered and the covering mount are on one filesystem).
- Dropping the working-directory existence check for pseudo sources:
  `pseudo_sources_need_no_lookup_to_succeed` failed.
- Forcing every path to `Unknown` (the old behavior):
  `a_pseudo_filesystem_starts_only_stat` and
  `a_device_with_a_udev_record_starts_nothing` failed.
  They count the commands through `linux_fs_id_with`.

Not done in this slice:
replacing `stat --file-system --format=%i`;
a permanent per-commit spawn budget test (no such
test exists in this tree yet;
the lowered count is 3 for the `findmnt` row).
