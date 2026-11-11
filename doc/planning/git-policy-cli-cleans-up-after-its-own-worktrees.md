# What the wrapper leaves behind, and the cleanup it now does itself

Work on `package/git-policy/cli` (crate `monochromatic-git-policy-cli`),
 branch `fix/git-policy-cli-cleans-up-after-its-own-worktrees`,
 worktree `.claude/worktrees/pi-wrapper-cleans-up-after-its-own-worktrees`.
Started 2026-11-10T19:02:11-05:00.

The owner's words,
in order,
on 2026-11-10:
"I believe I just found another wrapper defect.
It seems to be keeping all the snapshots,
 to a point where my 4TB drive is nearly full.",
"You're the coordinator and you're supposed to remove finished worktrees automatically.",
"If you're wondering about something in something we control,
 that something is already proven bad.
Potentially confusing behaviors,
 when intentional,
 should still be documented in a way that would make end users not be confused.",
"And in this specific case it's a more buggy bug fix.
The wrapper should have cleaned up after itself automatically deterministically."

The two decisions that bear on this work,
quoted with their paths:

- [`doc/decision/cargo-build-dir-host-cache.md`](../decision/cargo-build-dir-host-cache.md),
   section "The build-dir layout is internal":
   "Do not write repository tooling that parses these paths."
   The one exception is `cargo-shadow lean`'s reader,
   granted by the amendment of 2026-11-07T22:19:43-05:00.
   So the wrapper must not decide where a worktree's Cargo build folder is,
   and the removal of that folder belongs to `package/cargo-shadow/cli`.
- The same record,
   sections "Throwaway worktree builds outlive their worktree" and
   "Nothing prunes `~/.cargo/build/` automatically",
   records that a removed worktree's build folder stays in the host cache.

## What was measured on the live repository, read-only

Every command below only reads:
 `ls`,
`stat`,
`du`,
`btrfs filesystem du`,
`git worktree list`,
`git rev-parse`.

Measurements of 2026-11-10T19:12:55-05:00,
in `<common git dir>/monochromatic-git-policy-cli/`:

- `discarded/` 11 entries,
  464K,
   newest 2026-11-04T20-14-58,
   so nothing is due under the 90-day age limit.
- `notices/` 0 entries.
- `push/` 342 entries,
  1.4M,
   oldest file 2026-10-14T23:31:32-04:00,
   directory last written 2026-11-01T15:23:16-05:00.
   These are `<ref>.last-pushed.json` records of a release before the 2026-11-04 rename.
- `recorded-destroys/` 2 entries.
- `removal-snapshots/` 0 entries.
- `review-entries/` 19 entries,
  664K,
  newest 2026-11-01T17:24:34-05:00,
   16-hex-digit names of the same earlier release.
- `shadow/` 2 empty directories from 2026-10-11.
- `standards-review-shadow.md` 3.3M,
  and its `.from-earlier-release` copy 76K.
- `work/` 6058 entries in `done/` (27M) and 173 in `failed/` (80 review,
  79 push,
  10 maintenance,
  4 other),
   the newest written 2026-11-10T19:11:53-05:00.
   `done/` holds 1916 `maintenance`,
  2173 `push`,
  1846 `review` and 123 `trash` records.

Measurements of 2026-11-10T19:13:03-05:00,
at the repository top level:

- `.claude/worktrees/` 368 entries and `git worktree list` 400 registrations,
   so 32 registrations name a directory that is already gone.
   The coordinator was removing finished worktrees while this was measured,
   so both numbers fell during the reading:
   the same reading at 2026-11-10T18:47:43-05:00 by the coordinator gave 460 entries.
- Of the 368 entries,
  365 are btrfs subvolume roots (inode 256) and 2 are not.
- `<common git dir>/cli-git-add-policy-*` 40 directories,
  41M in total,
   left by a release before the rename.
- `cli-git-captures` 3 entries,
  `cli-git-transactions` 3 entries,
  `cli-git-scratch` 0 entries,
   all from a release before the rename.
- `monochromatic-git-policy-cli-scratch` 1 entry;
  `monochromatic-git-policy-cli-worktree-copy` 1 entry;
   `monochromatic-git-policy-cli-merges` 0 entries.
- `df` of the repository volume:
  3.4T used,
  229G free of 3.7T (94 percent).

Measurements of 2026-11-10T19:13:17-05:00,
on one live finished worktree
 (`.claude/worktrees/add-explains-slow-route/`):

```txt
     Total   Exclusive  Set shared  Filename
  26.63GiB   591.12MiB    13.52GiB  .claude/worktrees/add-explains-slow-route/
```

This is the coordinator's reading confirmed:
 one finished worktree holds 26.63 GiB of blocks it can reach,
 owns only 591 MiB of them,
 and shares 13.52 GiB with the checkout it was snapshotted from.
No single snapshot owns the space,
 and `du` on the source does not show it either,
 which is why it stayed invisible until the volume filled.

## The inventory: what the wrapper creates and what removes it

Every item below was read from the source,
 with the file and function that creates it,
 the one that removes it,
 and the event that triggers that removal.
"Nothing" means no code removes it,
 which is the defect this work closes where it can.

### The worktrees themselves

- The linked worktree,
  a btrfs snapshot on this filesystem.
   Created by `worktree_copy_forward.rs` `run_forwarded_git_with_unit_stop` through
   `worktree_copy.rs` `run_with_worktree_copy`,
   which reaches `worktree_snapshot.rs` for rung 1.
   Removed before this change by `worktree_remove.rs` `remove_subvolume_worktree`,
   reached only from `worktree_copy_forward.rs` `remove_by_subvolume`,
   which is reached only by a person running `git worktree remove`.
   The event was therefore a person remembering.
   Not guaranteed,
  and this is the space the volume lost to.
   After this change:
   `worktree_landed_removal.rs` `sweep_landed_worktrees`,
   on the merge that lands the worktree's branch.
- The ignored-state copy's stage,
  `.cli-git-worktree-copy-*` and `.monochromatic-git-policy-cli-partial-*`
   beside a new worktree.
   Created by `worktree_copy_stage.rs`.
   Removed by `worktree_copy_stage_claim_reaping.rs` and `worktree_copy_journal_removal.rs`,
   and by the next command in the repository,
  which finds the record the stage claim wrote.
   Event:
  the completion of the copy,
  or the next command.
   Guaranteed once any command runs in the repository.
- The copy journal under `monochromatic-git-policy-cli-worktree-copy/`.
   Created by `worktree_copy_journal.rs`,
   removed by `worktree_copy_journal_removal.rs` when the copy completes,
   and resumed and removed by the next command otherwise.
   Event:
  the completion of the copy,
  or the next command.
- The add's own record beside the worktree.
   Written before real Git runs by `worktree_copy.rs`,
   removed when the copy completes or when the worktree is removed.
   Event:
  either.
- The removal snapshot under `monochromatic-git-policy-cli/removal-snapshots/<instance>`.
   Created by `worktree_removal_rescue.rs` `snapshot_one` when a worktree being removed holds
   unshown push failures or unfinished pushes.
   Removed by `worktree_removal_snapshot_promotion.rs` `promote_snapshot` once every record in it is promoted.
   Event:
  the promotion,
  which the next command runs.
   Kept,
  and named by every command,
  while promotion keeps failing,
   because it is then the last copy of that record.

### The stores beside the repository's Git data

- `monochromatic-git-policy-cli/recorded-destroys/<name>`.
   Created by `worktree_removal_interrupted_destroy.rs` `record_destroy`,
  before a subvolume destroy.
   Removed by `forget_destroy` when the destroy failed or the removal finished,
   and by `finish` in the next command otherwise.
   Event:
  the removal ends,
  or the next command.
   Guaranteed.
- `monochromatic-git-policy-cli/discarded/<UTC time>/<path>`.
   Created by `discarded_store.rs` when a person discards a conflict side.
   Removed by `discarded_prune.rs` `prune`,
   reached from `policy_linked_worktree.rs` `judge_conflict_sides`:
   a `git checkout -- <paths>` or `git restore` of conflicted paths is the event that prunes,
   within `discardedCopies.maxAgeDays` and `maxSize`.
   Guaranteed for any repository where a person discards a side again;
   not a timer,
  and the limits are configuration.
- `monochromatic-git-policy-cli/notices/<id>` and `orphaned-notices/<id>`.
   Created by `push_failure_notice.rs`,
   removed by `push_failure_notice.rs` once the notice is shown to the person.
   Event:
  the next command shows it.
   Guaranteed.
- `monochromatic-git-policy-cli/work/{pending,claimed,recovering,blocked,held,owed}`.
   Created and moved by `work_item.rs` `enqueue`,
  `claim_next`,
  `settle`,
  `hold`.
   Removed by the same module's renames while the item is being worked on.
   Event:
  the consumer's round.
   Guaranteed.
- `monochromatic-git-policy-cli/work/{done,failed}`.
   Written by `work_item.rs` `settle` when a consumer finishes or abandons an item.
   **Nothing removes them.**
   Measured 6058 done and 173 failed records.
   Gap,
  and the smallest honest addition is a retention bound over them:
   a `done` record exists so a replayed landing does not queue the same work twice,
   so the bound must be a policy number the owner or the coordinator picks,
   not something this change should invent.
- `monochromatic-git-policy-cli/push/` and `review-entries/`,
  342 and 19 entries.
   Written by a release before the 2026-11-04 rename;
  this release neither reads nor writes them.
   **Nothing removes them.**
   Gap:
  2.1M in total,
  harmless in bytes,
  and proof that a store whose owner went away is never reclaimed.
- `cli-git-add-policy-*` in the common Git directory,
  40 directories,
  41M.
   The same earlier release's add records.
   **Nothing removes them.**
   Gap.
- `cli-git-captures`,
  `cli-git-transactions`,
  `cli-git-scratch`.
   The same earlier release's.
   `README.md`,
  section "State on disk",
  says a command that finds commit records in
   `cli-git-transactions` names the directory and prints the `rm --recursive --force` line.
   The other two are named by nothing.
   Gap.
- `monochromatic-git-policy-cli/standards-review-shadow.md`.
   Appended once per landed commit by `standards_review_run.rs`.
   Nothing trims it.
   Gap,
  3.3M and growing one entry per landing.
- `monochromatic-git-policy-cli/scene` was not found;
  `monochromatic-git-policy-cli/shadow/` holds two empty
   directories from 2026-10-11,
  which this release no longer writes there
   (`landing/service_native.rs` puts a shadow inside the commit's own scratch directory).
   Nothing removes them.
   Gap,
  empty,
  and proof of the same kind as `push/`.
- `monochromatic-git-policy-cli-scratch/`.
   The repository scratch.
   Removed by `scratch_entry_removal.rs` as commands run:
   an entry nothing has written for two hours goes,
   a commit's own tree goes when its attempt ends,
   other entries after a day,
   at most eight per command.
   Event:
  any command.
   Guaranteed.
- `monochromatic-git-policy-cli-merges/<id>`.
   Written before a merge starts by `merge_isolation_journal.rs`,
   removed when the branch moved;
   `merge_isolation_recovery.rs` drops one whose merge already landed.
   Event:
  the end of the merge,
  or the next command.
   Guaranteed.
- `monochromatic-git-policy-cli-shadow-pool/<worktree key>/`.
   Created by `shadow_pool.rs`,
  at most `POOL_LIMIT` (2) members per worktree,
   with `sweep_abandoned` removing what a killed fill left.
   Event:
  a commit in that worktree.
   Guaranteed by use,
  bounded by the limit.
- `monochromatic-git-policy-cli-service-<id>-<attempt>` in the scratch directory.
   A commit's scratch tree,
  created by the commit,
   removed by the service once that attempt's landing is over,
  within
   `SCRATCH_REMOVAL_CEILING` (thirty seconds),
   or by the command when no service took it.
   Event:
  the end of the landing.
   Guaranteed in the normal case;
  a service that never runs leaves it to the age sweep.
- `refs/cli-git/carried/<n>`,
  a merge's carried edits.
   Created by `merge_isolation_carry.rs` before the checkout changes,
   removed when the merge concludes or is made again.
   Event:
  the end of the merge.
   Guaranteed by the same recovery that finishes a merge.
- `monochromatic-git-policy-cli-audit.jsonl`,
  `-commit-timing.jsonl`,
   `-worktree-add-timing.jsonl`,
  `-push-trigger.jsonl`.
   Appended by the audit,
  commit timing,
  add timing and push trigger recorders.
   Nothing rotates them.
   Gap,
  small and append-only.
- The service's own journals under the per-user state directory,
   `%S/monochromatic-git-policy-cli/service/<instance>`.
   Created by `service_provision.rs`,
   removed by `service_journal.rs` when the instance retires,
   and by `service_units_of_removed_worktrees.rs` when its worktree is removed.
   Event:
  retirement or removal.
   Guaranteed for a removed worktree;
  not measured here,
  because it is not on the repository volume.

### Outside the repository

- The host Cargo build folder of each worktree,
  `<CARGO_HOME>/build/<shard>/<hash>/`.
   Created by Cargo in that worktree.
   Removed by `cargo-shadow lean` mechanism C,
   `lean_idle_trees.rs` `collect_idle_trees`,
  when a tree's workspace is gone.
   Event:
  a `lean run`,
  which the timer `cargo-shadow-lean.timer` fires once a day,
   or a person running it.
   There is **no per-workspace interface**:
   the CLI offers `lean run` (machine-wide,
  measured at 398 s on 2026-11-08),
   `lean records <directory>...`,
  and `lean timer-install`.
   So a removed worktree's build folder waits for the machine-wide pass.
   The smallest addition is `cargo-shadow lean trees <workspace root>...`,
   running mechanism C's per-tree proof over the trees whose ledger names one of those roots,
   so the wrapper and any other remover can name the folder it just orphaned.
   This change does not build it:
  it is another package's owner,
  its own tests,
  and the
   decision quoted above forbids the wrapper from deciding those paths.

## What this change does

### The record that says the wrapper made a worktree

`src/native/worktree/worktree_added_by_the_wrapper.rs`.
The add that Git accepted writes one small file beside the worktree's registration,
 `monochromatic-git-policy-cli-worktree-added`,
 holding the worktree's path.
The registration's directory is deleted with the worktree,
so the record needs no removal of its own.
Why it is needed at all:
 every automatic removal must be able to prove the wrapper made that worktree,
 because `git worktree remove` deletes what Git ignores,
 and a person's own worktree can hold ignored work of theirs.
Nothing on disk proved it before:
 `worktree_add_binding.rs` writes its copy owner only when Git refused an existing destination,
 and the ignored-state copy removes its journal when it finishes.
The record is written without a flush:
 a record a power loss takes back only keeps one worktree that nobody then removes,
 and a record that cannot be written is a warning,
never a failed add.

### The removal on the merge that lands a branch

`src/native/worktree/worktree_landed_removal.rs` (the sweep,
the conditions and the removal),
 `src/native/worktree/worktree_landed_registrations.rs` (the registrations and their branches,
from file reads only),
 `src/native/worktree/worktree_landed_after_a_merge.rs` (the two heads and the printing),
 `src/native/worktree/worktree_landed_sweep_notice.rs` (the words),
 `src/native/worktree/worktree_process_inside.rs` (the process table read).

The trigger is the event that ends a worktree's use:
 a `git merge` run through this wrapper that ended with status 0 and moved the branch.
`lock/index_writer_forward.rs` `run_coordinated_forward` reads the branch before the merge,
 sweeps after it,
 and does so from all three ways a merge can end:
 the isolated merge,
 native Git's own,
 and the word-level second pass of a resolved conflict.
A merge that moved nothing sweeps nothing and costs one string comparison.

The set of candidates is the branches this merge brought in:
 every branch contained in the new head and not contained in the head before the merge,
 from one `git for-each-ref --merged` per head.
That keeps a worktree whose branch merely sat on an older commit of the landed branch,
 which is not work this merge brought in,
 and it never considers a branch with no commits of its own.
The candidates are also limited to the worktrees the repository registered before the merge ran,
 which `window` reads in one directory read beside the branch it already reads before Git starts.
Another cross-vendor review found the hole:
the registrations were read after the merge commit existed,
 so a worktree a command added in that window was in the candidate set
 and a merge that landed before it existed removed it.
A registration that was not there when the merge started is no work this merge ended,
 so the sweep does not examine it at all.

A candidate is removed only when all of these hold,
each one a control:

- the record above exists and names this worktree;
- it is not the worktree the merge runs in;
- it is not locked;
- no `MERGE_HEAD`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`,
  `rebase-merge`,
  `rebase-apply` or `sequencer`
   stands in its administrative directory;
- `worktree_process_inside.rs` `process_working_inside` finds no process whose working folder is it or below it,
  and no process of this account that hides its working folder and started after the worktree was made;
- every ignored file and link it holds is also in the worktree this merge ran in,
  by kind,
  mode and bytes (the paragraph below).

A process can hide its working folder from its own account:
`PR_SET_DUMPABLE` 0 makes `/proc/<pid>/cwd` unreadable to the user that owns the process,
and another cross-vendor review found that the sweep removed the worktree under such a process.
The kernel answers nothing else about one,
and the programs that hide their folders on this desktop are the services that start at boot or login
(measured 2026-11-11: `systemd --user`,
`(sd-pam)`,
`kwin_wayland`,
`polkit-kde-authentication-agent-1`,
`ssh-agent` and three more),
so keeping every worktree they might be in would remove none at all.
A working folder can only lead into a directory that already existed,
so a process older than the worktree directory cannot be inside it,
and the directory's birth time settles that.
A hidden process that is younger,
or whose age or the worktree's cannot be read,
keeps the worktree,
and the reason names the process so a person can look at it.
The cost is a worktree kept while a young hidden program runs elsewhere on the machine:
safe,
and readable in the notice rather than silent.
The landed sweep's controls write their own process table,
so no test depends on which programs this machine happens to run.

Cleanliness,
that Git would remove the worktree without `--force`,
is Git's own to decide.
The sweep does not pre-judge cleanliness:
an earlier draft called `worktree_clean.rs` `check_clean` first and refused more than Git does,
 because a freshly checked-out worktree has files as new as its index until something refreshes it,
 which is the state every worktree this program makes is in.
Instead the removal runs and Git's refusal is what the person reads.

The removal is `worktree_remove.rs` `remove_subvolume_worktree` first,
 which is one ioctl on btrfs and what actually gives the shared blocks back,
 and real `git worktree remove` when that answers `LeftToGit`:
 a filesystem that is not btrfs,
 a mount without `user_subvol_rm_allowed`,
 or a case Git must word itself.
Git's refusal words are what a kept worktree reports,
 so the wrapper keeps exactly Git's line:
 it deletes what Git ignores and refuses what Git would keep.
The branch is kept in every case;
 only the checked-out copy goes.

Those ignored files are the one hole in Git's own line,
 and a cross-vendor review found it:
`git worktree remove` deletes the files Git ignores without a word,
 and the btrfs destroy takes the whole directory,
 so a file the person wrote there after the add,
 a `.env` covered by a committed ignore rule,
 went with the worktree and the merge exited 0.
The owner's rule is that no command of this program discards a person's data,
 so the sweep now asks Git for the worktree's ignored entries (`worktree_landed_ignored_files.rs`)
 and compares every file and link among them with the worktree the merge ran in,
 by kind, mode and bytes (`worktree_copy_compare.rs` `entry_matches`).
A file this repository's own worktree holds identically came from the add or from the repository,
 and removing this worktree takes a copy and nothing else;
 a file it does not hold,
 or holds with other bytes,
 may be the person's,
 so the worktree stays and the file is named in the reason.
Ignored directories are never read:
 a build writes them into the worktree alone,
 which is the state this sweep exists to give back,
 and reading every file below them would cost the tree on every merge;
 the removal line says the ignored files went because the merge's own worktree holds every one of them.
The comparison is a stat and a bounded byte read per ignored file Git lists,
 and this repository's own worktree measured 136 such files against 468 ignored directories,
 so the sweep keeps the space it was written for.
A query or a comparison that cannot be made keeps the worktree and says why,
 because nothing proves the files are not the person's.

A commit a reference lock holds in a landed worktree lives in that worktree's own Git directory
 and on the per-worktree reference `refs/worktree/cli-git-service/<batch>`,
 and the removal deletes both,
 so the sweep keeps it on `refs/cli-git/kept/<batch>` of the common Git directory before it examines a candidate,
 through the same `worktree_removal_held_commits.rs` `keep_held_commits` the ordinary removal uses.
Another cross-vendor review found that the sweep skipped this
 and that `git gc --prune=now` then took the commit.
The held lines are printed before the removal lines,
 so a person reads where the commit is before reading that the worktree went.

Every merge that lands a branch prints one line per worktree it removed,
 left,
 or could not prove it made,
 naming the path,
 the branch,
 the branch that received the commits,
 the reason,
 and the `git worktree remove` line that removes what the wrapper left.
Nothing is removed in silence.

Nothing in the sweep can fail the merge:
 every failure is a line on standard error,
 and standard error only,
 because standard output carries the merge's own report.

## What a worktree added before this change looks like

The 368 worktrees measured at 2026-11-10T19:13:03-05:00 carry no record.
They are never removed by the wrapper,
 and the new sweep names each one whose branch lands,
 with the `git worktree remove` line that removes it,
 so a person can act on it.
Writing a record for them by inference would defeat the only proof that keeps a person's own
 worktree safe,
so this change does not.

## Tests

`src/native/worktree/worktree_landed_removal_tests.rs`,
 `worktree_added_by_the_wrapper_tests.rs`,
 `worktree_process_inside_tests.rs`,
 `worktree_landed_sweep_notice_tests.rs`.

The suite's summary line,
run 2026-11-10T19:37:00-05:00:
`20 tests run: 20 passed, 6289 skipped`
 (the exact line is in the commit message).

The three the brief asks for,
each on a real repository and a real merge:

- `a_landed_clean_worktree_is_removed_and_its_branch_stays`:
   the worktree directory is gone,
  the branch `refs/heads/feature` still resolves,
   and the repository stays.
- `a_landed_worktree_with_uncommitted_work_stays`:
   an untracked file the person wrote is still there afterwards.
- `a_landed_worktree_with_a_process_inside_stays`:
   a `sleep` started with the worktree as its working folder keeps it,
   and the line names the reason with that process id.

Plus `a_landed_worktree_this_program_cannot_prove_it_made_is_left` and
 `a_branch_that_merely_sat_on_an_older_commit_is_no_candidate`.

The suites the brief names,
run on the host at 2026-11-10T19:46:00-05:00:

- `test(/worktree_add|worktree_remove/)`:
  `67 tests run: 67 passed, 6242 skipped`.
- `test(/worktree_copying/)`:
  `384 tests run: 384 passed, 5925 skipped`.
- `test(/landing::/)`:
  `850 tests run: 850 passed, 5459 skipped`.

Each condition is proved by planting its removal and watching the test that names it fail,
 then restoring the file.
The four runs below were made against the committed code with
 `node ${HOME}/temp/agent/planted-controls-20261110/planted-controls.ts`,
 whose last line is `restored: true`:

- the authorship record ignored:
   `1 test run: 0 passed, 1 failed, 6308 skipped` for
   `a_landed_worktree_this_program_cannot_prove_it_made_is_left`.
- the process table not read:
   `1 test run: 0 passed, 1 failed, 6308 skipped` for
   `a_landed_worktree_with_a_process_inside_stays`.
- the sweep examining nothing:
   `1 test run: 0 passed, 1 failed, 6308 skipped` for
   `a_landed_clean_worktree_is_removed_and_its_branch_stays`.
- Git asked to remove with `--force`:
   `1 test run: 0 passed, 1 failed, 6308 skipped` for
   `a_landed_worktree_with_uncommitted_work_stays`,
   which is the control that proves the test would catch a `--force` creeping in and taking a person's untracked file.

## What this change does not do

- It does not remove a worktree on a fetch that shows its branch on the remote's default branch.
   The merge is the event this repository's flow has,
   and the fetch path needs the remote's default branch named without a new Git start per candidate.
   Next step if it is wanted.
- It does not remove a removed worktree's Cargo build folder,
   for the reason quoted above,
  and the smallest addition is named in the inventory.
- It does not touch `work/done`,
  `work/failed`,
  the pre-rename stores,
  or the append-only logs.
   Those need retention numbers nobody has picked yet.
- It does not remove any worktree that already existed when it landed,
   because none of them carries the record.

## The merge of main, and the final run

`main` was merged into this branch at 2026-11-10T19:58:00-05:00 in merge commit `b4e1da36ac`,
 which brought `AGENTS.md`,
 `CLAUDE.md`,
 the wrapper handover,
 `package/crate-release/cli/bin/native-release-host-controls.ts` and its unit test,
 and a `pnpm-lock.yaml` catalog refresh.
Nothing in the crate's Rust changed in that merge,
 so the only risk to this change was a semantic one,
 and the suites were run again after it:

- `native:clippy` (the gate's Clippy flags, warnings denied): clean.
- `test(/worktree_landed|worktree_added_by_the_wrapper|worktree_process_inside/)`:
   `20 tests run: 20 passed, 6289 skipped`.
- `test(/worktree_add|worktree_remove/)`: `67 tests run: 67 passed, 6242 skipped`.
- `test(/worktree_copying/)`: `384 tests run: 384 passed, 5925 skipped`.
- `test(/landing::/)`: `850 tests run: 850 passed, 5459 skipped`.

The package's `lint` task is red on this branch for findings this work did not add.
Every one of them is in a file whose diff against `main` is empty,
 which was checked file by file:
 `package/git-policy/cli/SPEC.md`,
 `src/native/service/service_server_connection_handling_tests.rs`,
 `src/native/worktree/idle_io_priority.rs`,
 `src/native/worktree/worktree_trash_hand_over.rs`,
 `src/native/worktree/worktree_trash_dispose.rs` and
 `src/native/worktree/worktree_trash_entries.rs`.
The `lint:oxlint` and `lint:types` children of the same task also fail;
 this change holds no TypeScript and no generated file,
 so neither failure can be this branch's.
