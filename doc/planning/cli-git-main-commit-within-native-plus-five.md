# A main-checkout commit within native plus five seconds

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

The owner's bound:
"commits vs native git should not take 5s longer."
And on a report that main's commits took up to 184 s
because the disk was saturated:
"that's not an excuse."
And:
"Commits should never fail."
This note splits the slow commit of 2026-11-03T10:33:28.419Z,
compares a wrapped commit with a native one step by step,
and
records what was changed,
with the numbers before and after.

## The split of 184 s

The commit is the line of `.git/cli-git-commit-timing.jsonl` with `requestId` `af35c85f-8aaa-4962-8b2b-75a9bf09c1a1`
(main,
release 0.5.23,
`git commit --message=... -- doc/handover/cli-git-rust-implementation.md`):

- `totalMs` 184201,
  of which `prepareMs` 61384,
  `buildMs` 25043,
  `submitMs` 70977 (`answerWaitMs` 70976),
  `afterLandingMs` 26532,
  `gateMs` 194,
  `handoffMs` 34,
  `otherMs` 34.
  The parts add up to the total.
- The service's `answered` line for the same request (journal of `cli-git@fc85fdf7a58f866b7f7a84b44fe0b69a.service`,
  05:33:01
  local):
  `queued_ms=0 lock_wait_ms=0 lock_place_ms=0 landing_ms=70976`,
  and inside the landing `reads_ms=4 objects_ms=12
  journal_ms=1 ref_ms=2 sync_ms=70955 lock_hold_ms=70958`.
  So the 71 s were not a queue and not a lock held by another program:
  the request started at once,
  and 99.97 percent of its
  landing was `sync_ms`,
  the follow-up the service runs while it holds Git's index lock (`follow_up` in
  `src/native/service_land.rs`):
  `git diff-tree -r --name-only -z --no-renames <old> <new>`,
  then `git reset -q <new>
  --pathspec-from-file=-` over a copy of the index.
  The journal shows both as running steps the whole time:
  the `diff-tree`
  past 5,
  15 and 25 s,
  then the `reset` past 10,
  20 and 30 s,
  and the answer at 05:33:01.
  Other sessions' requests waited behind this batch (for example `bf26e603-a76b-4e31-886a-e1d087a76110`,
  a request answered
  `branch-moved` after 32.6 s in the queue),
  not the other way round.
- The client side,
  from the lines the client printed:
  `prepareMs` is `git read-tree --reset <HEAD>` (past 20 s) and then
  `git add --all -- <the file>` (past 20 to 30 s) in the private index,
  plus the in-process policy work;
  `buildMs` is the
  one `git commit` of the scratch repository (signing is on in the person's global configuration,
  so every commit is built
  by Git,
  never by the service);
  `afterLandingMs` in 0.5.23 was the summary `git diff-tree --shortstat --summary -M <old>
  <new>` (past 25 s).
  The current source prints Git's own summary and starts no such step (`landed_summary.rs`).
- The same pattern holds for the other main commits of the window:
  216363 ms (`sync_ms` 85344 of a `landing_ms` of 85427),
  168794 ms,
  and later 299988 ms with `prepareMs` 120013,
  `buildMs` 59790 and `submitMs` 120009 (`sync_ms` 119429).
  The linked worktrees of the same minutes took 2.7 to 4.4 s each,
  of which `buildMs` was 2.3 to 3.3 s,
  where the releases
  before 0.5.23 had 50 to 150 ms (the timing file's medians per release).

What every slow step has in common is measurable.
Of the Git starts of one wrapped explicit-path commit,
these read the index
and connect to the file-system monitor socket of the repository (`strace -f -e trace=execve,connect` of a wrapped commit on
a throwaway repository with the person's global configuration):
`read-tree --reset`,
`add --all`,
the scratch repository's
`commit`,
and in the service `diff-tree`,
`reset` and the review's `diff-tree --no-commit-id`.
Six of them for the commit
and its landing,
seven with the review;
native `git commit` makes three connections,
all from one process.
The Git starts
that read only refs and objects (`rev-list`,
`rev-parse`,
`update-ref`,
`cat-file`) make none.
On the throwaway repository,
with the repository's monitor not listening,
`git diff-tree HEAD~1 HEAD` took 1.60 s and
`git rev-list --parents --max-count=1 HEAD` 0.00 s (a repository with about 110 000 files and directories;
the permitted
command stopped that repository's own monitor,
the only one stopped).
On the real host at 2026-11-03T11:12Z the main checkout had no listening monitor,
three `git fsmonitor--daemon start`
processes of 12,
34 and 38 s of age at 99 percent of a processor each (their parent chain:
a Git started with the wrapper's
hook-off settings by `jbcontext watch run`),
244 monitor processes in all,
and a load average above 40.
This is evidence,
not a proof:
the 184 s run was never traced,
and Git's monitor is left exactly as the person configured it
(`README.md`,
"Git's own fsmonitor"):
nothing here sets,
stops or avoids it.
What the change below does about it is only that
fewer Git processes start per commit,
and so fewer of them read the index.

## A wrapped commit against a native one, step by step

On a throwaway repository of main's shape (20 670 tracked files,
a 2.8 MB index,
53 000 ignored files,
the person's global
configuration with a throwaway signing key,
`core.fsmonitor = true` and `commit.gpgsign = true`),
release 0.5.23 source at
the branch point,
an explicit-path commit:

- Native `git commit --message=x -- <file>`:
  one process (plus `ssh-keygen`),
  6.1 MB written (the real index and the temporary
  index,
  2.9 MB each,
  and the objects),
  three connections to the monitor from that one process.
- Wrapped,
  before:
  14 Git starts in the client and 10 in the service (the service's landing and its work after the
  command),
  six processes connecting to the monitor,
  20.5 MB written by the client (three index rewrites in the private index
  and copies) and 5.7 MB by the service.
- Full-tree and full-index operations the wrapper added:
  `read-tree --reset` of the whole base tree into the private index;
  `ls-tree -r --full-tree` twice;
  the service's `diff-tree`;
  a copy of the 2.9 MB index three times (reflinks on this file
  system;
  real copies on one without them).

## Changes

Commits,
oldest first,
with their timestamps,
are in the final report of the work;
each has a red control first.

1.  Tree-to-tree name lists in process (`repo_read_query.rs`,
    `repo_read_tree.rs`):
    `diff-tree -r --name-only -z
    [--no-renames] <old> <new>` and `diff-tree --no-commit-id --name-only -r -z <commit>` for a commit with exactly one parent.
    The walk compares two trees together and reads only the subtrees whose names differ,
    so it costs in proportion to
    what changed.
    A root commit,
    a merge commit,
    a file that became a directory and a gitlink are declined (Git answers).
    The service's follow-up and the review then start no `diff-tree`,
    and neither connects to the monitor.
2.  A commit whose real index already equals its base starts no `read-tree` (`commit_index.rs`):
    the private index is the copy of
    the real index,
    the in-process `diff-index --cached` reader says the copy holds exactly the base tree,
    and the question the
    readers leave to Git takes the old way.
    Budgets `WARM` and `AMEND` in `binary_budget_tests.rs` lowered from one start to zero.
3.  The in-process object reader keeps what it decoded (`repo_read_pack.rs`,
    32 MiB per pack) and a process keeps the lists of
    its last three trees (`flatten_tree_shared`).
    Measured on the throwaway repository,
    an in-process commit made 302 920
    `pread64` calls and was 270 ms slower than the same binary with `CLI_GIT_IN_PROCESS_READS=off`;
    the dozen questions
    of one commit each opened the pack again and read the whole base tree through delta chains without sharing a base.
    With the keeps,
    12 alternating pairs gave 434 ms against 399 ms.
4.  `ls-tree -r` is left to Git:
    listing a whole tree in process was 30 ms slower than the start and touches no index.

## Measured

Method:
two byte-identical throwaway copies of the repository measured in
`A wrapped commit against a native one, step by step` (each with its own commit service,
started by the
measurement,
and its own runtime directory,
so the service units are not rewritten between samples).
Native Git ran in both
copies (the control for the spread),
the base binary in one and the changed binary in the other,
one commit of each in turn,
twenty rounds,
an explicit-path commit of one file,
no wait for a quiet host:
the load average was 17 to 22 and the
input and output pressure `some avg60` 3.9 percent during the runs.

- Native,
  copy of the base:
  median 63 ms,
  61 to 115 ms. Native,
  copy of the change:
  median 61 ms,
  58 to 103 ms.
  The spread of the same binary on unchanged input is under 5 ms around the median.
- Wrapped,
  base (the branch point of this work):
  median 651 ms,
  632 to 711 ms,
  so 588 ms more than native.
- Wrapped,
  changed:
  median 437 ms,
  423 to 818 ms (one sample at 818 ms),
  so 376 ms more than native.
  Phases from the timing file (medians):
  prepare 358 to 194 ms,
  gate 174 to 130 ms,
  build 47 ms,
  submit 66 to 62 ms.
- Git starts of one commit:
  client 13 and service 12 before,
  client 12 and service 10 after.
  Processes connecting to the
  monitor:
  six before (seven with the review) and three after,
  native one.
- Bytes written,
  client plus service:
  26.2 MB before,
  22.6 MB after,
  native 6.1 MB (the difference after the change is
  the private index written twice and the copies of the index,
  which reflink on this file system).
- Same binary,
  in-process reads on and off,
  twelve alternating pairs:
  839 against 525 ms before the object keeps,
  690
  against 424 ms with the delta keep,
  and 434 against 399 ms with the keep of the tree lists.

These are the quiet-load figures.
They do not reproduce the 184 s;
they show the work the wrapper adds when nothing stalls,
and
the count of processes that can stall.
The stall itself needs the real checkout and its monitor,
which no measurement here may
touch.
The bound is met on this host by a wide margin when nothing stalls (376 ms against 5000 ms) and is not shown to be met
when the processes that read the index wait for the monitor,
which is the `Git's monitor and the three-to-one gap` item under the `Left to the owner` heading.

## The landing's index update and the index check in process

The follow-up `git reset <commit> --pathspec-from-file=- --pathspec-file-nul` that the service ran under Git's index lock took
59 to 60 s on main on 2026-11-03 (request `54490c0c`:
`lock_hold_ms=59122 sync_ms=59118`),
and the wrapper's own
`git diff-index --cached --quiet HEAD --` took more than 40 s before a merge.
Native Git ran the same commands in 12 ms on the
same checkout,
so no change to what the commands do could explain it,
and none is attempted:
the Git children are removed.

What is ported,
from the Git 2.56.0 source:

- `builtin/reset.c` `read_from_tree` and `update_index_from_diff`,
  with the comparison of `diff-lib.c` (`diff_cache`,
  `oneway_diff`,
  `do_oneway_diff`):
  a path-limited mixed reset (`src/native/index_reset.rs`).
  An entry that differs is removed
  (`remove_file_from_index`) or replaced by an entry made from the tree's mode and object name with all-zero stat data
  (`make_cache_entry` with no refresh,
  `add_index_entry` with the add and replace options),
  so the next status hashes the file.
  Amended 2026-11-08:
  a landing records the worktree's stat data for a path whose file provably holds the tree's bytes
  (`index/index_reset_worktree_stat.rs`),
  and the all-zero value stands where no proof exists
  (`doc/handover/cli-git-rust-implementation.md`,
  "The landing records the stat data it can prove,
  and the staleness question walks its entries in parallel").
  The skip-worktree bit of a replaced entry is kept;
  the assume-valid bit and the stage are not.
- The extensions Git keeps (`read-cache.c` `do_write_index`):
  the cache-tree is invalidated along each path
  (`cache-tree.c` `do_invalidate_path`) and read in Git's order,
  shorter subtree name first (`subtree_name_cmp`);
  an unmerged
  entry that leaves the index is recorded in resolve-undo (`resolve-undo.c`);
  the untracked cache is cleared for each added or
  removed path,
  and,
  only when fsmonitor is on,
  for each replaced one (`dir.c` `invalidate_one_component`,
  `fsmonitor.h`
  `mark_fsmonitor_invalid`);
  the fsmonitor extension is kept with its token and a bitmap rebuilt from the entries,
  the new ones
  dirty,
  when fsmonitor is on,
  and dropped otherwise (`fsmonitor.c` `tweak_fsmonitor`,
  `fill_fsmonitor_bitmap`).
  The two
  bitmaps use Git's EWAH encoding,
  ported with its `ewah_set` corner cases (`src/native/index_ewah.rs`).
- The file:
  the version rule (a version 3 file without an extended flag is written as version 2),
  version 4 name compression,
  the entry padding and the checksum (`src/native/index_full.rs`,
  `index_full_write.rs`).
- `diff-index --cached --quiet <commit or HEAD> [-- <paths>]` (`src/native/repo_read_query_quiet.rs`),
  reached by the existing
  `run_git` and `run_metadata_git` entry points,
  so the wrapper's checks in `rule_commit_index.rs`,
  `merge_isolation.rs` and
  `worktree_dirty_restore.rs` start no Git.
  An unmerged entry is always a difference;
  an intent-to-add entry is an ordinary
  entry,
  because `diff-index` never sets `ita_invisible_in_index`;
  a submodule difference is answered unless a `.gitmodules`
  exists anywhere,
  because it could say `ignore = all` (`diff.c` `is_submodule_ignored`).

The entries come from the commit's tree,
not from stat data the capture took:
the capture holds none,
the journaled intent holds
only the paths (so a restarted service finishes the same update),
and zero stat data is what the old command wrote and what
makes Git compare the file's bytes.
The paths of other sessions' commits need no update here:
each landing updates its own,
and
a commit built elsewhere lands only while the branch is still at its base,
so the derived list is that commit's own paths.

Where the port stops and Git's own command runs (now `git reset --no-refresh`,
which skips the whole-index refresh of
`builtin/reset.c` that the earlier `-q` form kept):
a split index,
a sparse index or sparse checkout,
a required extension it
does not know,
a file whose checksum or layout is off,
`index.skipHash`,
`feature.manyFiles`,
the offset-table settings,
a case
folding worktree,
`core.hooksPath` or a `post-index-change` hook,
an untracked cache Git would add,
a monitor setting without
the extension (or the reverse),
a file where the index has a directory,
a path Git's name checks could refuse,
an entry recorded
in the second the index file was written (Git would hash it from the worktree while writing),
and the in-process switch off.
The index update that is not part of a landing (`update_index`,
used when corrections remove every change) still starts the
`--no-refresh` command.

What the cross-vendor review of the branch at `48d6abb28c` changed:

- The update runs only under the landing's lock.
  `service_land.rs` takes the live `index.lock` (`acquire_noting_wait`)
  before the branch moves and passes it to `follow_up`;
  `build_index` copies the live index while it is held;
  `IndexLock::release_with` (`service_index_hold.rs`) renames the copy over the live index and removes the lock.
  The
  recovery path takes the lock the same way (`try_acquire` in `recover_selected`).
  A foreign `git add` finds the lock
  taken from the copy to the installation,
  so nothing it staged before is lost and nothing after starts.
  The helper
  now takes a borrow of the held `IndexLock`,
  so only the holder can call it,
  and works on
  `private_index_path(lock.index(), seq)` alone;
  its only caller is `follow_up`,
  which owns the lock on both paths.
  It
  writes through a temporary file it creates exclusively (no run shares a name,
  and a run removes only its own),
  sets
  Git's index mode on that file before the first byte (created as Git creates its lock file,
  then the mode `calc_shared_perm` makes of
  the mode the new file really has,
  default ACL included),
  flushes it,
  and renames it.
- A bitmap in the fsmonitor or untracked-cache extension is read only if its runs,
  literals and last run-length word
  are consistent with its bit size and the bit size is at most the entries or the directories;
  a 20-byte bitmap that
  described terabytes is refused before anything is listed.
- Names are declined for every spelling the NTFS and HFS protections of Git refuse (colon and backslash included,
  and
  the hashed short name of `.gitmodules`,
  ported from `is_ntfs_dotgitmodules` in `path.c`),
  whatever
  `core.protectNTFS` says;
  this is a superset of Git's refusals,
  so Git answers all of them.
  Real Git 2.56.0 refuses
  each one the control lists.
- The review expected an intent-to-add entry equal to the tree's to be replaced.
  Git 2.56.0 leaves it with its flag
  (the differential control `an_intent_to_add_entry_equal_to_the_tree_is_left_as_git_leaves_it`),
  and so does the
  update.

What the third cross-vendor review changed (the first three items are defects of the landing's outer protocol that
shipped in 0.5.27,
and are fixed here for the same reason):

- A batch whose index was installed and whose lock was given up could be replayed by a restart that found its intent still open,
  overwriting a change another session staged in between.
  The installed index is now recorded in the journal while the lock is
  held (`<batch>.indexed`),
  a failed record keeps the lock,
  and a restart that finds the record retries only the conclusion
  state,
  which is recorded apart.
- The worktree's checked-out branch was read before the index lock was awaited.
  It is read again with the lock held and before
  the ref moves,
  and a restart that finds another branch checked out finishes the batch without touching the index.
- The private copy could be the live index (a client's `GIT_INDEX_FILE` named like one) or another client's index.
  It is now a
  `PrivateCopy`,
  created exclusively with a name built from the whole live name,
  and the in-process update takes that value.
- The mode of the published file starts from the mode the file system gave the new file,
  so a default ACL of the Git directory
  is honoured as it is for Git's own lock file.
- An interrupted write is tried again;
  the status and diff controls a truncated rewrite had deleted are back;
  the racy-entry
  fallback has an end-to-end control through `build_index`.
- Was not changed here:
  `remove_stale` reads a lock's marker and then removes the lock,
  which is not atomic.
  Two services
  reclaiming the same physical index at once could remove a lock a third program took in between.
  Closed afterwards:
  the
  reclaim holds an exclusive `flock` on the stale lock file and unlinks the name only while it still refers to that file,
  at
  all three reclaim sites,
  with controls for two competing reclaimers (`doc/planning/cli-git-one-reclaimer-per-index.md`).

What the fourth cross-vendor review changed:

- A replay of an index update that never installed could overwrite what another session staged since.
  The landing now records
  the entries of its paths under the lock before the branch moves (`<batch>.start`),
  a replay resets only the paths still at
  them,
  and the others keep their staging with a notice.
  This is the coordinator's pick,
  open to the owner's veto;
  the owner's
  rule "never discard user data" decides it.
  Not closed:
  the unlocked gap between `remove_stale` and `try_acquire` in a restart
  (the replay is now non-destructive,
  so a program that stages in the gap loses nothing),
  a batch with no record replays every
  path (a commit Git built lists its paths only when it is finished,
  so it has none),
  and a path staged to exactly the entry
  the landing started from is reset (the staging equals what the landing found,
  so nothing differs to keep).
  A panic that
  unwinds through the record leaves the lock.
- `is_indexed` read every error as no record.
  The record is now three-valued (absent,
  installed,
  unreadable) and a restart that
  cannot read it leaves the batch and the index alone.
- The installed index's directory is synced before the completion record,
  and the journal directory before the record is removed.
- The worktree's head is read from the Git directory the request recorded.
  The ref transaction cannot carry
  `symref-verify HEAD`:
  Git 2.56.0 refuses it beside an update of the branch `HEAD` names ("multiple updates for 'HEAD'",
  checked by hand on a throwaway repository),
  so the head is read again after the move,
  under the lock,
  and a worktree that left
  the branch keeps its index.
  This differs from the review's proposal.
- The private copy is a random-named owner-only directory with a marker (`service_land_copy.rs`),
  which replaces the loose
  file:
  no default permissions before the first byte,
  no shared name,
  no exhaustible names,
  no length that grows with the
  index's name (the lock's own staging name overflows a file name long before that matters,
  `service_index_staging.rs`).
- A restart after a branch switch still finishes an index kept in another file,
  and the batch stays open until the conclusion
  state is removed.
- The default ACL control sets the ACL through its extended attribute,
  so the gate image needs no `setfacl`;
  a file system
  that refuses it skips with the reason.

What the fifth cross-vendor review changed (the owner's rule "never discard user data" decides each of the first items):

- A replay is made only over the index file the landing saw.
  The entries of the batch's paths cannot tell a path nobody staged
  from a path staged back to the starting entry (stage another version,
  then the old one),
  cover only the exact names of a
  reset (which also rewrites the entries under and above its path),
  and were absent for a commit Git built and for a landing
  that could not write the record.
  The landing now records a witness of the live index file under its lock for every batch
  (`<batch>.start`:
  inode,
  size,
  modification and change times,
  and the checksum tail,
  `service_land_start.rs`),
  a restart takes
  the lock and replays the whole batch,
  carried conflict stages included,
  only when the file is still that file,
  and any other
  case (a file written since,
  no record,
  a record in damaged text) leaves every path of the batch as its writers left it,
  says
  so in the log,
  and finishes the conclusion state.
  A record that cannot be read now keeps the batch open.
  Every program that
  changes an index renames a new file over it,
  so an unchanged file has been written by nobody;
  the record is not synced,
  since
  a record lost to a power failure only withholds a replay.
  The per-path comparison,
  `read_entries` and `split_by_start` are gone.
- A restart reads the worktree's head again under the lock it takes,
  and a head Git cannot read stops it with the batch open:
  `git symbolic-ref --quiet HEAD` ends with status 1 for a detached head and with other statuses for a question it could not
  answer,
  and only the first is a detached head (checked by hand on Git 2.56.0:
  status 128 outside a repository).
- A detached head that was attached to a branch at the same commit after the last look was replaced by a detached head:
  the
  update with `option no-deref` accepts the symbolic head when the old value matches (measured by hand on a throwaway repository:
  `ref: refs/heads/B` became the new commit's name).
  The update now follows the head as native Git's commit does,
  and the head is read again
  after the move (`follows_move`):
  the index is updated for a head that shows the commit,
  detached at it or on the branch that
  took it.
  Still open and the same as native Git:
  a switch after the last look.
  `git checkout` installs its index and releases
  the index lock before it updates `HEAD`,
  and `git symbolic-ref` takes no index lock,
  so no hold of the index lock orders the
  landing with a head update;
  a landing that wins that race leaves the state of `git commit` followed by `git symbolic-ref`.
  Holding `HEAD.lock` would fail the other program's head update after its index was installed,
  so it is not done.
- The live exit after a switch settles the same obligations as the restart (`finish_after_move`):
  the worktree's own default
  index is left to its new branch,
  an index kept in another file is finished,
  and the batch is done only when the conclusion
  state is removed.
- The release flushes a changed index just before the rename (`IndexLock::release_flushed_with_record`) unless the in-process
  update already flushed that file,
  and installs nothing when the copy holds the live index's bytes:
  the live file keeps its
  access list,
  which a copy made in another directory does not inherit.
  The flush of the copy of the live index in
  `PrivateCopy::make` is dropped,
  since every writer replaces that copy.
  Measured on a copy of this repository's index (20 741
  entries,
  15 landings per arm,
  `strace -f -e trace=fsync,fdatasync`):
  9 fsync calls per landing against 12 at the branch
  point (the index copy,
  and the start record's file and directory),
  and the wall time of a landing is the same within a spread
  larger than the difference (medians of the same binary between 489 and 810 ms in a debug build on a loaded host).
- The sweep of a dead service's private copy no longer reads a process number.
  The owner holds an exclusive `flock` on its
  directory for as long as the value lives;
  the kernel drops it on death.
  A recycled number,
  another PID namespace,
  and a service
  killed before it wrote a marker no longer decide anything.
  `flock` is a hold of this kernel:
  a directory shared by hosts over a
  file system without one is never swept.
- A stale index lock is taken aside by one rename and looked at again before it is removed,
  and put back by a hard link that
  fails rather than replace a lock placed meanwhile.
  A second instance serving one physical index no longer removes a lock a
  third program placed after the marker was read.
  Left then:
  a lock taken aside that proves live is put back by a hard link,
  and a
  second lock placed at the name in the microseconds between the rename and the hard link keeps the name;
  the taken-aside lock
  then stays at its aside name and the log says where.
  Closed afterwards by one reclaimer per physical index:
  nothing is taken
  aside any more,
  the reclaim holds a `flock` on the stale file and unlinks the name after checking it still refers to that file.

Not changed:
the long alternate-index name still overflows the lock's own staging name (`service_index_staging.rs`) before the
name Git supports,
so the control for it searches for a name the service accepts.

Measured on 2026-11-03 on throwaway copies of main's shape (20 672 tracked files,
53 000 ignored,
a signed explicit-path commit,
`core.untrackedCache = true`,
no fsmonitor,
release builds,
20 rounds after 2 warm-up rounds,
native Git and both wrappers in
turn,
host not quiet):
native 38 and 39 ms (37 to 41);
the base wrapper (the branch point) 137 ms (132 to 213,
median absolute
deviation 2);
the changed wrapper 139 ms (134 to 146,
deviation 2).
With nothing stalling the two are the same within their
spread,
as expected:
in release the in-process update takes about 20 ms (read and parse 6,
serialize 3.5,
write and flush 10),
where the process it replaces takes about as long.
The measured services ran in a transient scope capped at 16 GiB of memory,
with none of the unit's limits of that day;
the scope's memory-limit events did not rise (none) and its major faults rose by
81 over the run.
The effect is in what can stall:
strace of the commit service over six
commits shows the base starting 6 `reset` processes of 54 Git starts,
the change none of 48.
The slow case needs the real
checkout and its monitor,
which no measurement here may touch (`README.md`,
"Git's own fsmonitor"),
so the fsmonitor-on branch
of the port is checked against the source,
not against a running Git;
the fixtures cannot turn it on.

## Left to the owner

- Git's monitor and the three-to-one gap:
  a wrapped commit still starts three Git processes that read the index (`add`,
  the
  scratch `commit`,
  the service's `reset`) where native starts one.
  Each is a start that waits when the monitor of the
  repository is not listening.
  Fewer would need the private index built without `add` (a hash in process,
  which would skip
  the person's clean filters,
  so it is not offered) or the scratch commit built without a worktree.
- Commit failure paths that load or a slow machine can reach:
  see the final report.
- The benchmark `native:perf` runs its fixtures with `GIT_CONFIG_GLOBAL=/nonexistent-global-config`,
  so it never sees signing
  or the monitor;
  its 5 s bound cannot catch what the person's commits pay.
