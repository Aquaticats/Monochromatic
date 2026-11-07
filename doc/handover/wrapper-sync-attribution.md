# Wrapper sync attribution for one explicit-path commit (task 78)

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

This answers task 78 in `doc/handover/cli-git-rust-implementation.md` [section removed on 2026-11-03 under rule H0P
and kept in commit `e10b8186bc`]:
in one wrapped `git commit -m x -- file`,
which sync calls come from the wrapper's own code and which from the child Git processes it starts.
Nothing in the product was changed;
this is a measurement and a reading of the code.

## Method

- Binary:
  the release wrapper `cli-git-native` (crate version 0.4.5) built from this worktree with
  `mise run //package/git-policy/cli:build`.
  A debug build (`mise run //package/git-policy/cli:native:build`) was used only for stack traces (`strace -k`),
  which name the Rust function behind each sync;
  its counts equal the release counts.
- Real Git:
  `/usr/bin/git` 2.56.0,
  the version the budget tests in `src/native/binary/binary_budget_tests.rs` name.
- Filesystem under test:
  btrfs (`stat -f` on the scratch directory,
  `/var/mnt/encrypted`).
  Sync call counts do not depend on the filesystem;
  the durability reasoning in the section on which syncs are
  needed does.
- Environment:
  the hermetic one of the budget tests (`env_clear`,
  `HOME` in the fixture,
  no global or system Git
  configuration),
  `maintenance.autoDetach=false` so a repack that Git starts after a commit belongs to the traced
  tree,
  a disposable repository per run under `~/temp/agent/m78`.
- Trace:
  `strace -f -tt -y -qq --no-abbrev -e trace=fsync,fdatasync,sync_file_range,syncfs,sync,msync,openat,execve,
  clone,clone3,fork,vfork`.
  Each sync line was attributed to the process that issued it by following `execve` and `clone` lines:
  a process or
  thread with no `execve` of its own takes the program of its parent.
  The wrapper is any program under the fixture;
  Git children are named by subcommand (`index-pack`,
  `pack-objects`).
  This is the attribution method of `src/native/binary/binary_budget_trace.rs`,
  reimplemented in a scratch script.
- Scripts (scratch,
  not committed):
  `~/temp/agent/m78/sync-attribution.ts` (one traced run),
  `run-all.ts` (three runs per scenario),
  `summarize.ts`,
  `index-pack-fsync-probe.ts`.
  Raw traces and per-run reports are in the same directory.
- Three runs per scenario,
  each in a fresh repository.
  The host was heavily loaded (load average 70 on 28 cores at the start);
  counts are unaffected by load,
  and no timing is reported or claimed.

## Counts per run

Only `fsync` appeared.
No `fdatasync`,
`sync_file_range`,
`syncfs`,
`sync` or `msync` call was made by any process in any run,
and no file was opened with `O_SYNC` or `O_DSYNC`.

Cold commit (first wrapped commit in a fresh repository,
`commit --quiet --message=x -- a.txt`):

- run 1:
  36 syncs,
  33 wrapper,
  3 `git index-pack`
- run 2:
  36 syncs,
  33 wrapper,
  3 `git index-pack`
- run 3:
  36 syncs,
  33 wrapper,
  3 `git index-pack`
- This equals the budget constants `wrapper_syncs: 33` and `git_syncs: index-pack 3` in `binary_budget_tests.rs`.

Warm commit (second wrapped commit in the repository):

- runs 1 to 3:
  35 syncs each,
  28 wrapper,
  3 `git index-pack`,
  3 `git pack-objects`,
  1 `git multi-pack-index`

Steady state (seventh commit in the repository,
after five untraced wrapped commits):

- runs 1 to 3:
  35 syncs each,
  28 wrapper,
  3 `git index-pack`,
  3 `git pack-objects`,
  1 `git multi-pack-index`

Amend (`commit --amend --quiet --message=x -- a.txt` after a base commit):

- runs 1 to 3:
  36 syncs each,
  33 wrapper,
  3 `git index-pack` (cold repository,
  as for the cold commit)

Baselines:

- `git status --short` through the wrapper:
  0 syncs in all three runs (the trace saw the processes and 60 `openat`
  calls,
  so zero is a measured zero,
  not an empty trace).
- The same explicit-path commit by `/usr/bin/git` directly:
  0 syncs in all three runs.
  Direct Git writes loose objects,
  the ref and the index without any sync under its default `core.fsync`.

The run-to-run spread is zero in every scenario.

### Existing opt-outs

The wrapper accepts `--no-worktree-copy` (skip ignored-state synchronization) and `--no-enforce-only` (do not inject
`--only` into a commit);
both are listed in `src/native/invocation/wrapper_controls.rs` and `command_commit.rs`.
Neither changes the commit path's syncs:

- `commit --no-worktree-copy -- a.txt`:
  36 syncs in all three runs (33 wrapper,
  3 `git index-pack`).
- `commit --no-enforce-only -- a.txt`:
  36 syncs in all three runs.
- `commit --no-enforce-only` with a staged file and no path:
  36 syncs in all three runs,
  and the same journal records,
  so the transaction is not skipped by that option either.

No existing option removes any sync of a wrapped commit.

## Syncs by origin

Cold commit,
36 syncs:

- Wrapper:
  33 (18 file syncs and 15 directory syncs;
  the warm 28 are 17 and 11,
  and the cold-only extras are listed
  under Cold-only syncs).
- `git index-pack`:
  3 (the temporary index,
  reverse index and pack files of the migrated commit objects).
- Every other child Git process makes none:
  `init`,
  `update-ref` (shadow and real),
  `symbolic-ref`,
  `read-tree`,
  `add`,
  `diff`,
  `write-tree`,
  `commit` in the shadow repository,
  `pack-objects --stdout`,
  `reset`,
  `show`,
  `diff-tree`,
  `remote`,
  `rev-parse`,
  `ls-files`,
  `cat-file`,
  `config`,
  `for-each-ref` and `maintenance run`.
  In particular the real `index.lock` and the ref update by Git itself are not synced,
  which fits Git's default
  `core.fsync` (`committed,-loose-object`,
  as `git help config` states for the host's Git 2.56.0).
- Warm and steady commits add 4 syncs from Git's automatic maintenance:
  `git maintenance run --auto --no-detach` starts
  `git repack -d -l -q --cruft --write-midx`,
  which starts `pack-objects` twice (3 syncs on its output pack,
  index and
  reverse index) and `multi-pack-index write` (1 sync on `multi-pack-index.lock`).
  The first commit in a repository triggers none;
  a repack ran in every later wrapped commit I traced.
  Each wrapped commit adds one pack (see the `index-pack` item),
  and direct Git (loose objects only) had no
  maintenance syncs in its run,
  but the trigger was not isolated:
  a rerun of `maintenance run --auto` afterwards,
  on
  a repository already repacked to two packs,
  did nothing,
  and two packs is far below `gc.autoPackLimit`.
  Whether the pack count,
  the multi-pack-index or something else fires the repack is open.
  With the default `maintenance.autoDetach=true` the same repack runs detached after the commit returns.

Wrapper syncs of the warm commit (28),
by protected object,
with the function that issues each
(`strace -k` on the debug binary,
files under `package/git-policy/cli/src/native`):

1.  Transaction owner record,
    `owner.json` in the `.pending` candidate:
    `fill_candidate` in `transaction_registry.rs`,
    through `write_private_file` and `write_then_publish` in `private_storage.rs`.
    1 file sync.
2.  Directory of that candidate:
    `fill_candidate`,
    `sync_directory`.
    1 directory sync.
3.  Registry `cli-git-transactions` after the publishing rename:
    `publish_transaction_directory`,
    `transaction_registry.rs`.
    1 directory sync.
4.  `preparing.json` (target,
    real index path,
    shadow path;
    the first journal record):
    `write_journal_record` in
    `commit_transaction.rs`.
    1 file and 1 directory sync (the directory is the transaction directory).
5.  `conclusion/store-held.json` (pseudorefs held in the ref store,
    written for every commit):
    `copy_conclusion_state`
    in `conclusion_copy.rs`.
    1 file sync.
6.  Shadow repository `packed-refs` (the snapshot of the real refs inside the throwaway shadow):
    `write_shadow_refs`
    in `shadow_refs.rs`.
    1 file sync.
7.  Capture lock `owner.json`:
    `write_candidate` in `owner_lock.rs`,
    called with `write_exclusive_synced`.
    1 file sync.
8.  Capture order `sequence` (temporary file,
    then rename):
    `write_sequence` in `capture_order.rs`.
    1 file sync and 1 directory sync (`cli-git-captures`).
9.  `captured.json` (the paths and sequence of this capture):
    `write_captured_record` in `capture_order.rs`.
    1 file sync.
10. `hooks/plan.json` (the hook dispatch plan):
    `write_hook_dispatch` in `hook_dispatch_entries.rs`.
    1 file sync.
11. `prepared.json`:
    `write_journal_record` in `commit_transaction.rs`.
    1 file and 1 directory sync.
12. Landing lock `owner.json`:
    `write_candidate` in `owner_lock.rs`.
    1 file sync.
13. `index-lock-1.json` (identity of the real `index.lock` the landing holds):
    `write_lock_record` in
    `real_index_lock.rs`.
    1 file and 1 directory sync.
14. `index~pid.lock` (the PID file announcing the lock):
    `announce_held_lock` in `real_index_lock.rs`.
    1 file sync.
15. `landing-1.json` (expected old value,
    new commit,
    snapshot identities and digests,
    lock identity,
    pack name):
    `write_journal` in `landing_attempt.rs`,
    called by `record_landing`.
    1 file and 1 directory sync.
16. `ref-updated.json`:
    `write_journal` in `landing_attempt.rs`,
    called by `advance_target`.
    1 file and 1 directory
    sync.
17. Landed-capture record `captures/landed/<commit>.json`:
    `record_landed_capture` in `capture_records.rs`.
    1 file and 1 directory sync.
18. Real index installation:
    `write_lock_contents` in `recovery_files.rs` syncs the lock file holding the post-index
    before the rename over `.git/index`,
    then `install_recovered_index` syncs `.git`.
    1 file and 1 directory sync.
19. `index-installed` marker:
    `write_journal` in `landing_attempt.rs`,
    called by `complete_landing`.
    1 file and 1 directory sync.

By kind,
the 28 are 17 file syncs and 11 directory syncs.
By protected object:
owner records 3 (items 1,
7,
12),
recovery journal records 5 files (items 4,
13,
15,
16 and the
`store-held.json` of item 5),
capture ordering 2 (item 8),
records that the tearing harness shows to be benign 6 files
(items 9,
10,
11,
14,
17, 19) plus their directories,
the shadow repository 1 (item 6),
the real index 2 (item 18),
and directory syncs for the remaining entries.

### Cold-only syncs

The cold commit has 5 more wrapper syncs than the warm one,
all creation syncs on a fresh repository:
`.git` after creating `cli-git-transactions` (`ensure_transaction_root`,
`transaction_registry.rs`,
reached from
`create_commit_workspace` in `commit_workspace.rs`),
`.git` after creating `cli-git-captures` (the same function,
reached from `ensure_capture_store`),
the `worktree-id` file and `cli-git-captures` after it
(`ensure_worktree_id` in `capture_order.rs`),
and `cli-git-captures` after creating the `landed` directory
(`ensure_transaction_root` again,
reached from `record_landed_capture`).
The `worktree-id` pair guards a record whose tearing breaks later commits (see the needed list);
the three
directory-creation syncs protect only the existence of a directory that the next command recreates.

### Child syncs by protected object

- Migrated commit objects (`git index-pack --stdin --keep=...`,
  issued by `migrate_shadow_objects` in
  `landing_objects.rs`):
  3 per commit,
  on the temporary pack,
  index and reverse index.
  Git issues them because its default `core.fsync` includes packs;
  the wrapper passes no `core.fsync` setting.
  A probe of the same invocation on a tiny pack (`index-pack-fsync-probe.ts`) made 3 syncs by default,
  and 0 with
  `-c core.fsync=none` and 0 with `-c core.fsync=-pack,-pack-metadata`.
- Maintenance repack and multi-pack-index:
  4 per warm commit,
  as described.
- Nothing synced for `index.lock`,
  the ref update,
  or the shadow repository by any Git child.

## Which wrapper syncs are needed

Evidence used:
what recovery reads (`recovery_landing.rs`,
`conclusion_cleanup.rs`,
`recovery_contradiction.rs`)
and the outcome table of the tearing harness in `doc/planning/cli-git-spawn-and-sync-reduction.md`
("J3 outcome") over 247 cases.
I did not rerun the harness;
I rely on its recorded table,
whose damage steps are empty,
half and deleted records.
The filesystem claims (which syncs a later sync covers) are assumptions about journaling behavior,
not tested here,
and this host's filesystem is btrfs,
for which the ordering promises of ext4 `data=ordered` do not carry over.

Needed (removal loses crash safety;
the harness shows a torn record sticks every guarded command,
or the sync is the
only barrier before a side effect):

- Items 1,
  7 and 12,
  the three owner records:
  a torn `owner.json` sticks at every phase.
- Item 4,
  `preparing.json` file sync:
  emptied or halved sticks.
- Item 13,
  `index-lock-1.json` file sync:
  emptied or halved sticks before the ref moves.
- Item 15,
  `landing-1.json` file and directory:
  empty,
  half or deleted at the ref-updated phase sticks (or,
  before the
  contradiction check existed,
  silently left the index unreconciled);
  this sync is the barrier before the
  compare-and-swap.
- Item 16,
  `ref-updated.json` file sync:
  emptied or halved sticks.
  Its directory sync is not needed (see the redundant list).
- Item 5,
  `store-held.json` file sync:
  benign before the ref moves,
  sticks after.
  It can lose its own sync only if a later barrier makes it durable.
- Item 8,
  `sequence`:
  emptied or halved sticks later commits (they exit 2);
  the rename's directory sync keeps the number
  from repeating.
- Cold-only `worktree-id` file sync:
  emptied sticks later commits.
- Item 18,
  the post-index in `index.lock` and the `.git` sync:
  protects the real index contents behind a renamed file.
  Direct Git does not sync its index,
  but here the ref has already moved,
  and the snapshot recovery would install is
  itself unsynced (digest-verified only).
  The harness damages journal files,
  not the installed index,
  so what a torn installed index does is not shown.
- Item 3,
  the registry sync after the publishing rename:
  the later directory syncs are of the transaction directory,
  not of the registry,
  so a lost rename would lose the whole directory with its landing record.
  On ext4 and XFS a later fsync's journal commit would carry the rename;
  on btrfs this is not promised.

Redundant or benign (a later sync covers it,
or the harness shows no outcome change at any phase or damage):

- Items 9 (`captured.json`),
  10 (`hooks/plan.json`),
  11 (`prepared.json`,
  file and directory),
  14 (`index~pid.lock`),
  17 (landed record,
  file and directory),
  19 (`index-installed`,
  file and directory):
  the harness shows every damage
  of these files completes or discards,
  at every phase.
  These 9 syncs protect nothing recovery needs.
  A torn `captured.json` or landed record costs a warning and subsumption in later replays.
- Item 6,
  shadow `packed-refs`:
  the shadow is thrown away;
  recovery reads shadow objects,
  conclusion files,
  `MERGE_RR`
  and the sequencer copy,
  and resolves shadow refs only for store-held pseudorefs,
  which exist only on the reftable
  backend (`is_store_held` in `conclusion_cleanup.rs`).
  On the files backend nothing reads this file after a crash.
  This reading is from code,
  not a test.
- Item 2,
  the candidate directory sync:
  the `owner.json` it follows is already fsynced.
  The directory sync matters only where a file fsync does not persist the file's own name,
  which POSIX does not
  promise but ext4,
  XFS and btrfs do in practice (assumed,
  not tested).
- The transaction-directory syncs that follow `preparing.json` and `index-lock-1.json` (items 4 and 13):
  each file is
  already fsynced,
  and the directory sync at `landing-1.json` covers every entry of that same directory before the
  compare-and-swap;
  the same file-fsync assumption applies.
- Item 16's directory sync:
  deleting `ref-updated.json` completes in the harness (and so does deleting
  `index-installed`,
  item 19).

Children:

- The 3 `index-pack` syncs:
  no wrapper code reads the pack name for verification (`pack_name` is only written into
  `landing-1.json` and parsed back;
  recovery removes `.keep` files by message).
  If the pack data is lost while the ref update persists,
  recovery does not notice.
  Direct Git has the same exposure for its loose objects (not synced by default),
  so these syncs exceed direct Git's
  durability.
  They are Git honoring `core.fsync`;
  forcing them off would override a user's own `core.fsync` choice.
- The 4 maintenance syncs:
  Git's own,
  after the commit;
  not crash safety of the wrapper at all.

## Ranked candidate removals

Savings are in sync calls per warm commit (cold commits save the same,
because none of these are cold-only).
Status:
removals 1,
2,
3 and 4 are implemented (owner decision "Real gain real risk.
We're high rollers.");
5 was
measured and not kept (breaks three tests and the pack keep).
Item 4:
cold 36 to 33 and warm 35 to 32 syncs,
measured with `strace` on a changed build;
see `landing_objects.rs` and
`src/native/binary_index_pack_fsync_tests.rs`.
The measured result of a changed build matches the arithmetic below exactly:
wrapper syncs 33 to 19 cold and 28 to 14
warm,
in `binary_budget_tests.rs`.
The tests,
guards and findings are in
`doc/planning/cli-git-spawn-and-sync-reduction.md`,
section "J4 outcome".
Two cases do not converge:
a lost `owner.json`
name stops every guarded command,
and a lost `preparing.json` or `index-lock-1.json` name next to a held `index.lock`
makes every commit exit 2 until the lock is removed by hand.
The text below is the ranking as it stood before the changes.

1.  Drop the syncs of records the tearing harness shows to be benign:
    `captured.json`,
    `hooks/plan.json`,
    `prepared.json` (file and directory),
    `index~pid.lock`,
    the landed record (file and directory),
    and the
    `index-installed` marker (file and directory).
    Saves 9 (6 file syncs and 3 directory syncs).
    Reason for rank one:
    the harness shows no outcome change at any phase or damage,
    and the planning document already
    lists them as allowed.
    A ratchet in `binary_budget_tests.rs` would drop `wrapper_syncs` from 28 to 19 warm
    (33 to 24 cold).
2.  Write the shadow `packed-refs` without a sync.
    Saves 1.
    Reason for rank two:
    no code path reads it after a crash on the files backend,
    so it is safe on the code reading,
    but the harness has no case for it;
    add a damage case for the shadow's refs first.
3.  Stop syncing three transaction-directory entries and the candidate directory:
    the directory after `preparing.json`
    and after `index-lock-1.json` (the barrier at `landing-1.json` covers them),
    the directory after `ref-updated.json`
    (a deleted record completes),
    and the `.pending` candidate directory (item 2).
    Saves 4.
    Reason for rank three:
    it relies on a file fsync persisting its own name,
    which no filesystem promise in this
    repository backs;
    a lost `preparing.json` or `index-lock-1.json` entry before the landing leaves a stale
    `index.lock` (the harness shows "discards,
    leaves index.lock,
    next commit exits 2"),
    the failure that already
    exists for a kill between creating the lock and writing its record.
4.  Pass `-c core.fsync=-pack,-pack-metadata` to the migration `index-pack` only.
    Saves 3 (probe:
    3 to 0).
    Reason for rank four:
    the saving is measured,
    but it overrides a user's own `core.fsync`,
    and after a power loss
    on a filesystem that does not order data before the later `landing-1.json` sync (XFS and btrfs make no such
    promise;
    ext4 `data=ordered` does) the branch could point at a missing pack,
    which recovery would not notice.
    Direct Git has the same exposure for loose objects,
    so it is a policy choice,
    not a bug.
5.  Avoid the pack per commit by migrating small object sets as loose objects.
    In steady state this would remove the
    3 `index-pack` syncs and,
    only if the repack trigger turns out to be the added pack (not isolated here),
    the 4
    maintenance syncs as well,
    7 per commit at most,
    provided the pack `.keep` protection can be replaced.
    It is a design change of the
    landing,
    not a sync removal,
    so it is ranked last and its saving is a claim,
    not a measurement.
    Direct Git,
    which writes loose objects,
    made no maintenance syncs in its run.

Not recommended:
the real index sync and its directory sync (item 18),
the owner records,
`landing-1.json`,
`ref-updated.json`,
`preparing.json`,
`index-lock-1.json`,
`sequence` and `worktree-id`.
The first three candidates together save 14 of 28 wrapper syncs,
leaving 14 warm (cold:
19 left of 33).

## Could not measure

- Time spent in syncs per commit:
  the host was loaded,
  and the question asked for counts.
- Behavior after a power loss:
  a SIGKILL leaves the page cache,
  so only the recorded harness table and code reading
  speak to durability.
  No sync was actually removed.
- Why the maintenance repack runs on every commit after the first (pack count,
  multi-pack-index or another rule).
- The same counts on Git versions other than 2.56.0,
  or with a user `core.fsync` setting (the child counts would change,
  the wrapper's would not).
- Counts for commits with hooks,
  merges,
  or an empty message editor path;
  only the no-hook explicit-path commit and an
  amend were traced.
