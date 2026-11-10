# Worktree add-route findings on `main`

Status:
audit,
2026-11-09T01:01:10-05:00.
Audited revision:
`main` at `f24cca91a9`,
in the worktree
`.claude/worktrees/pi-worktree-add-route-findings-on-main`
(the branch's base is `9d4e5a9970`;
`git diff --name-only 9d4e5a9970..f24cca91a9 -- package/git-policy/cli/src package/git-policy/cli/SPEC.md`
lists nothing,
so every `path:line` in this page holds on both).
No production code was changed and no control was committed.
The throwaway controls are in
`package/git-policy/cli/src/native/worktree/audit_scratch_tests.rs`
in that worktree only,
registered by one `#[cfg(test)]` line in
`package/git-policy/cli/src/native/worktree/mod.rs`,
and their output is quoted under "Reproductions".
Linux only;
the owner's pause covers Windows and macOS.

Scope:
how a worktree gets made,
which is what neither of the two audited pages covers:

- the choice of rung before real Git runs
  (`worktree_ladder_run.rs`,
  `worktree_snapshot_source.rs`,
  `worktree_clean*.rs`),
- the destination the wrapper makes ready for Git
  (`worktree_subvolume.rs`),
- the claim and the record written before Git runs
  (`worktree_copy_stage_claim.rs`),
- the binding of the add to the directories Git made
  (`worktree_add_binding.rs`),
- the snapshot rung's stage,
  its swap and the settling of what it displaced
  (`worktree_snapshot.rs`,
  `worktree_snapshot_index_step.rs`,
  `worktree_snapshot_placeholders.rs`),
- the clone rung and the index both filling rungs install
  (`worktree_clone.rs`,
  `subvolume/subvolume_index_install_into_new_worktree.rs`),
- the index step,
  the checkout and the fallback
  (`worktree_ladder_target.rs`,
  `worktree_dirty_restore.rs`,
  `worktree_ladder_finish.rs`),
- `git worktree move`,
  which the wrapper forwards like any other command
  (`package/git-policy/cli/SPEC.md`,
  "Linked-worktree ignored-state synchronization").

Out of scope,
already audited elsewhere:

- the ignored-state copy into a worktree Git checked out,
  and its recovery:
  `doc/planning/cli-git-copy-route-findings-on-main.md`
  (five findings,
  two reproduced);
- removal and conversion,
  and the trash's disposal:
  `doc/planning/cli-git-worktree-removal-and-conversion-route-findings-on-main.md`
  (seven findings,
  four reproduced).

## What the handover and the two audits already record

`doc/handover/cli-git-rust-implementation.md`,
"Open review findings",
records nothing about how a worktree is made.
Its worktree material is the copy route's nine findings and eight off-`main`
fixes,
the accepted downgrade limit,
the push route's twelve findings,
the commits-never-fail review's four Critical findings,
and the lock-free settlement's supersession.
The two audited pages record:

- a kill between real Git returning and the copy's journal,
  a copy into a registration the command did not create,
  a recovery that discards the obligation when the stage is missing,
  a source whose own copy is running,
  and a failed installation nobody retries
  (copy route,
  findings one to five);
- the old tree's deleted left-out file,
  a removal snapshot that never settles,
  a conversion copy nothing can clean,
  a kill between the destroy and the registration's removal,
  a trash whose emptying is dropped,
  a deletion that never re-verifies the root,
  and `--abandon` destroying a stage a process may be in
  (removal route,
  findings one to seven).

None of the five findings in this page is one of those twelve,
and the four states the removal audit fixed were re-read in this code and hold.

## The owner's decisions this page judges by

Verbatim:

- "never discard user data".
- "nothing should ever wait forever or indefinitely"
  (`doc/handover/cli-git-rust-implementation.md`,
  "Nothing waits forever",
  2026-11-03 about 12:37-05:00),
  and what happens at a bound:
  "Return with an error",
  with the saved work kept.
- "Use as few locks as possible."
- "60s is fine for adding worktrees"
  (`doc/handover/cli-git-rust-implementation.md`,
  "Races with other programs",
  the speed answer of 2026-11-03T04:03:31-05:00).
- `doc/decision/cli-git-concurrent-commits.md:299-307`,
  owner 2026-11-03T02:57:19-05:00:
  "no settlement lock exists any more.
  Concurrent adds never wait for one another;
  the destination comes from the command line,
  and journals and stage claims name their owner process."
- The accepted risk of another process changing names or files while the
  wrapper works
  (`doc/handover/cli-git-rust-implementation.md:7136-7140`,
  owner 2026-11-03T00:15:33-05:00):
  "That is not an our problem.
  By using this command or this program,
  make the user understand and accept the risk."
  Its adopted reading,
  veto open:
  races caused by other processes are outside what the wrapper defends
  against,
  it states that risk plainly,
  and it keeps fixing its own deterministic bugs.
  This page separates its findings by that line:
  every sequence in this page is one the wrapper reaches by itself,
  and the races the notice covers are named as such.
- `package/git-policy/cli/SPEC.md`,
  "Cheap worktree creation and removal on btrfs,
  XFS and overlayfs",
  and "Linked-worktree ignored-state synchronization",
  are the specification this page checks the code against.
- `AGENTS.md` rule `NHP` makes a process killed at any instruction,
  a slow fsync,
  and an unusual filesystem normal here,
  so every kill point in this page is one to expect rather than an accident.

## The route as it runs on this host

`git worktree add -b topic .claude/worktrees/topic`
from the main checkout,
which is a btrfs subvolume root and clean:

1.  Before anything else,
    every wrapped command settles and takes removal snapshots
    (`invocation/wrapped_command_notices_snapshots_and_manual_push.rs:118`).
    For a main checkout that does not create a worktree this is all that
    happens:
    a main checkout's other commands take no lifecycle
    (`worktree_copying/worktree_copy_forward.rs:96`,
    `copy_applies_with`,
    which answers `serves_ladder || creates` for a main worktree).
2.  The plan reads the destination from the command line
    (`worktree_add_plan.rs:229`,
    `parse_add_plan`),
    with the real path of its parent joined with its last component
    (`:158`,
    `recorded_destination`).
    A form outside the table,
    a missing parent,
    or a destination that is already a symbolic link leaves the whole
    invocation to plain Git.
3.  `begin_ladder` chooses the rung
    (`worktree_ladder_run.rs:179`,
    `choose_rung`):
    `snapshot_refusal` and `clone_refusal` first
    (`worktree_snapshot_source.rs:230`,
    `:296`),
    then one `git status` and one `ls-files` for the source's cleanliness
    (`worktree_clean.rs`),
    then the claim
    (`worktree_copy_stage_claim.rs:278`,
    `claim_stage_for_add`),
    which writes two files under
    `<common>/monochromatic-git-policy-cli/`:
    a `.stage` claim naming the stage container and the owning process,
    and an `.add` record naming the destination with `phase=planned`.
4.  Real Git runs,
    with `--no-checkout` inserted directly after `add` for the snapshot and
    clone rungs
    (`worktree_ladder_run.rs:331`,
    `first_step`).
    Git validates the request,
    makes the destination directory,
    writes `gitdir: <common>/worktrees/<id>` into its `.git` file,
    writes the registration's `gitdir` back-pointer,
    `HEAD`,
    `commondir`,
    and `locked` when asked.
5.  `bind_registration`
    (`worktree_add_binding.rs:211`)
    proves the destination and the registration name each other,
    captures the identity of both directories and the pointer's bytes,
    and `record_binding` writes them into the `.add` record as
    `phase=registered`.
6.  The rung:
    for a snapshot,
    `prepare_target` reads the new `HEAD`,
    the tree difference and the
    configuration
    (`worktree_ladder_target.rs:417`);
    `take_snapshot`
    (`worktree_snapshot.rs:283`)
    runs `BTRFS_IOC_SNAP_CREATE_V2` from the source root to a stage name
    beside the destination
    (`.cli-git-worktree-copy-<uuid>`,
    `worktree_copying/worktree_copy_stage.rs:172`,
    `new_stage_container`),
    makes it private,
    reads the snapshot's own index when the source has changes,
    prunes the main checkout's leave-out,
    removes the stub of every nested worktree and of the destination,
    installs the index into the new worktree's administrative directory,
    and replaces the snapshot's `.git` with Git's pointer bytes;
    then the tree difference is applied,
    or a dirty main checkout is restored
    (`worktree_dirty_restore.rs:205`,
    `restore_dirty`);
    a kill anywhere here leaves the destination exactly as step 4 made it.
7.  `put_in_place`
    (`worktree_snapshot.rs:399`)
    checks that the destination is still the directory Git made,
    exchanges it with the stage by `RENAME_EXCHANGE`,
    and checks the displaced identity again;
    `settle_displaced`
    (`:445`)
    moves anything another process added to Git's directory into the new
    worktree,
    removes Git's pointer when its bytes are the ones written,
    and removes the empty directory.
8.  The add is marked complete
    (`worktree_ladder_finish.rs:338`,
    `mark_complete`,
    unsynced after a swap),
    Git's checkout line is printed,
    and the claim is released
    (`:498`,
    `release_claim`,
    record first,
    then marker,
    then claim).
9.  The lifecycle then empties the snapshot's trash beside the package repair
    (`worktree_copying/worktree_copy_after_lock.rs:191`) and repairs the
    package managers of the new worktree.

The clone rung fills the destination itself
(`worktree_clone.rs:494`,
`clone_tracked_files`),
installs the same index,
and applies the same tree difference;
Git's own checkout is the third rung,
and the fallback after a refused step
(`worktree_ladder_finish.rs:281`,
`checkout_like_git`,
`git reset --hard --no-recurse-submodules`).

## What happens off btrfs

Both cheap refusals fire
(`worktree_snapshot_source.rs:230`,
`:296`),
`prepare_subvolume_destination` makes nothing because its parent is no btrfs
(`worktree_subvolume.rs:81`),
and the add is Git's own checkout:
no stage,
no claim,
no rung.
The ignored state is then the copy route's,
whose reach on tmpfs and overlayfs the copy audit measured,
and the "not made as a snapshot" sentence is suppressed,
because nothing the person does makes such an add a snapshot
(`worktree_slow_route.rs:257`,
`explain_unsupported`).
Off btrfs is not the same as Git's checkout,
though:
on XFS and inside an overlay the clone rung still serves,
because a reflink needs no subvolume
(`worktree_snapshot_source.rs:275`,
`may_clone`),
and only a filesystem that never shares data blocks leaves the whole add to
Git.
No wait of the ladder's own exists there,
and none of the five findings is new there:
the waits are the copy's and the repair's,
which the two audited pages measure.

## Findings ranked by what they risk losing

### 1. An interrupted add is named once, then forgotten, and the line may never be printed

Read from the code,
with the state reproduced;
the third control under "Reproductions".

Sequence:

1.  `git worktree add .claude/worktrees/topic` from the main checkout takes the
    snapshot rung,
    so the claim and the `.add` record exist before Git runs
    (`worktree_ladder_run.rs:220`).
2.  Real Git returns zero:
    the destination is registered and holds nothing but
    its `.git` file
    (`worktree_ladder_finish.rs:453`,
    `finish_snapshot`).
3.  The process is killed anywhere between there and the swap:
    in `prepare_target`,
    during the snapshot ioctl,
    during the leave-out,
    during the index walk,
    or in the tree difference.
    For a large repository that window is seconds to minutes,
    and it leaves a registered worktree with no checkout plus a full snapshot
    subvolume at the stage name.
4.  The next command that takes the lifecycle
    (`worktree_copying/worktree_copy.rs:549`,
    `recover_pending_worktree_copies`)
    judges the claim's owner dead,
    prints one line about the stage
    (`worktree_copying/worktree_copy_stage_claim_reaping.rs:384-391`)
    and one about the add
    (`:143-147`,
    `settle_text`,
    `Recorded::Registered`),
    and then removes the `.add` record,
    the marker and the claim
    (`:404-406`).
5.  Nothing prints anything about that worktree again:
    the second control run of the same recovery prints an empty line
    (quoted under "Reproductions").

For a main checkout,
step 4 happens only for a later command that creates a worktree
(`worktree_copying/worktree_copy_forward.rs:96`,
`copy_applies_with`,
which gives a main checkout the lifecycle only when it creates one or the
ladder serves it),
or for any command run inside a linked worktree.
A person's `git status`,
`git commit` or `git log` from the main checkout never reaches the recovery,
so the one line can be missed entirely,
and the same is true for an agent that reads only its own command's output.

What is left:
a registration whose worktree holds only `.git`,
which `git worktree list` shows as an ordinary entry,
and a snapshot subvolume named
`.cli-git-worktree-copy-<uuid>` in the destination's parent,
which only the one line ever names.

What it risks:
disk from the stage,
and a worktree that looks like the add's but holds nothing,
which a later `git worktree add` at the same path refuses
("already registered").
No person's file is lost:
the source is untouched and the stage is a copy of it.

What notices it later:
nothing.
The recovery's obligation is one string in one process's standard error,
and the record that carried it is deleted in the same step
(`worktree_copy_stage_claim_reaping.rs:404-406`).

Suggestion,
not a decision:
keep the `.add` record while the destination is not proven whole,
and report it once per command that takes the lifecycle until the worktree is
finished or removed,
which is the shape the push route uses for an unshown notice;
alternatively write the marker only when the destination's `git status` is
empty and the registration resolves,
and leave a record for everything else.

### 2. A snapshot worktree can carry another add's stage container and its whole payload

Reproduced;
the first control under "Reproductions".

Sequence:

1.  A worktree is created from a main checkout,
    so the destination lies inside the source:
    `.claude/worktrees/topic`,
    as this repository makes them.
2.  At the same time another command copies ignored state into a worktree whose
    destination is in the same folder,
    so `stage_ignored_snapshot`
    (`worktree_copying/worktree_copy_stage.rs:191`)
    has made `.claude/worktrees/.cli-git-worktree-copy-<uuid>` and is filling
    its `payload` with that worktree's ignored files.
    That container is an ordinary directory with mode 0700,
    and the copy takes minutes for a real `node_modules`.
3.  `git worktree add .claude/worktrees/topic` from the main checkout takes the
    snapshot rung,
    which copies the whole subvolume.
4.  `prune_snapshot`
    (`worktree_trash.rs:321`)
    leaves out a directory only when `Prune::skips_path` names it as a cache or
    a run record
    (`worktree_copying/worktree_copy_prune.rs:157`),
    or when `skips_checkout` finds a `.git` in it
    (`:195`).
    `.cli-git-worktree-copy-<uuid>` is neither a cache name,
    a run record,
    nor a checkout,
    and `.claude` is not a tracked path,
    so the walk descends into it and keeps it.
5.  `remove_snapshot_placeholders`
    (`worktree_snapshot_placeholders.rs:180`)
    removes only the empty stand-in directories btrfs shows for nested
    subvolumes (inode 2,
    `:112`),
    which is the shape a **snapshot** rung's stage has;
    an ordinary stage directory is not one.

The wrong result:
the new worktree holds
`.claude/worktrees/.cli-git-worktree-copy-<uuid>/payload/...`,
a full copy of another worktree's ignored state,
with nothing that names it and nothing that empties it.
The bytes can include the other worktree's credentials,
`node_modules`,
and build state,
and they appear in a worktree on another branch,
where a `git add --all` would commit them.

What notices it later:
nothing.
The copy route removes its own container when it finishes,
but by then the snapshot carries an independent copy,
which the new worktree's trash does not name and no sweep looks for.

Suggestion,
not a decision:
treat `STAGE_PREFIX` like the trash parent in the leave-out walk,
by name,
so no stage of any rung is carried,
and let the stand-in removal stay for what btrfs leaves of a subvolume.

### 3. A snapshot of a dirty main checkout deletes the ignored files the new commit does not ignore

Reproduced;
the second control under "Reproductions",
with the clean source as the control.

Sequence:

1.  The main checkout ignores `cache/` and holds `cache/data.bin`.
    The commit the new worktree will check out ignores nothing:
    a branch that predates the rule,
    or a rule that a later commit removed,
    which is ordinary on any repository.
2.  The main checkout has one uncommitted change to a tracked file,
    so `source_or_dirty`
    (`worktree_snapshot_source.rs:391`)
    snapshots it and `restore_dirty` runs.
3.  `git read-tree --reset -u <new commit>`
    (`worktree_dirty_restore.rs:211`)
    brings the tracked `.gitignore` to the new commit,
    so the stage's ignore rules are now the new commit's.
4.  `git clean --force -d`
    (`:219`)
    removes every untracked file the stage's rules do not cover,
    which now includes `cache/data.bin`.
5.  `prove_restored`
    (`:171`)
    asks only for an empty `ls-files --others --exclude-standard`,
    a clean `diff-files`,
    and an index equal to the commit,
    so the deleted ignored file is exactly what the proof expects.
6.  The add succeeds and prints that the uncommitted changes were left out.

The wrong result:
the same repository,
with the same ignored state,
hands `cache/data.bin` to a new worktree when the source is clean and does not
when the source has one unrelated edit,
and the same command on a filesystem without snapshots hands it over in both
cases,
because the copy route selects the source's ignored paths by the source's own
exclusion rules
(`package/git-policy/cli/SPEC.md`,
"Linked-worktree ignored-state synchronization").
For a worktree added to build,
that is a missing dependency tree,
a missing local configuration,
or a missing environment file,
silently,
with a line that says something else.

What notices it later:
nothing.
The note names the number of **uncommitted changed paths**
(`worktree_dirty_restore.rs:55`,
`left_out_note`),
never a deleted ignored path,
and the journal entry records the same count.

Suggestion,
not a decision:
run the restore's `clean` with an exclude list taken from the source's ignore
rules,
or stage the source's ignored set aside before the clean and put it back,
and tell the person how many ignored paths the clean removed.

### 4. A kill while the rung installs the index leaves a lock that the printed remedy cannot get past

Read from the code,
with Git's half measured;
the fourth control under "Reproductions".

Sequence:

1.  The snapshot rung reaches `install_index`
    (`worktree_snapshot_index_step.rs:100`),
    which calls `install_index_in_with`
    (`subvolume/subvolume_index_install_into_new_worktree.rs:66`).
2.  `install_index_in_with` creates
    `<common>/worktrees/<id>/index.lock` in the new worktree's administrative
    directory
    (`:70-73`),
    writes the index into it,
    links it under the name `index`,
    and removes the lock.
3.  The process is killed between the create and the removal.
    `NHP` makes that normal here.
4.  The destination is registered and holds only `.git`,
    and the administrative directory holds `index.lock` with no `index`.
5.  The next recovery prints
    "`git -C <destination> reset --hard` completes its checkout"
    (`worktree_copy_stage_claim_reaping.rs:144`).
6.  That command fails:

```text
fatal: Unable to create '.../worktrees/<id>/index.lock': File exists.
```

The wrong result:
the one remedy the person was given cannot run,
and nothing names the lock.
`git status` in that worktree still works,
so the worktree looks usable while every write to it fails,
and the stage is reported separately.

What notices it later:
nothing in the wrapper:
`rg 'index.lock'` over `worktree/` and `worktree_copying/` finds the clean
check's refusal
(`worktree_clean.rs:57`)
and nothing that sweeps an administrative directory.

Suggestion,
not a decision:
name the lock in the interrupted-add line when it exists,
or have the recovery's notice offer `git worktree remove --force` first and
`reset --hard` only when no lock stands in the way.

### 5. The ladder's own Git steps wait with no ceiling

Read from the code.

Sequence:

1.  The snapshot rung's tree difference runs
    `git read-tree -m -u <source> <new>`
    (`worktree_ladder_target.rs:496`,
    `apply_difference`),
    the dirty restore runs `git read-tree --reset -u`
    (`worktree_dirty_restore.rs:211`),
    and the fallback runs `git reset --hard`
    (`worktree_ladder_finish.rs:281`,
    `checkout_like_git`).
2.  Each goes through `run_step`
    (`worktree_ladder_target.rs:379`)
    to the lifecycle's runner,
    which starts real Git as a child and waits for it
    (`invocation/forwarding.rs:187`,
    `run_real_git`,
    `status()`).
3.  Nothing in that path sets a deadline,
    and `read-tree -m -u` and `read-tree --reset -u` run the checkout's
    conversion filters,
    so a smudge filter,
    a Git LFS process,
    or a network filesystem that never answers holds the add forever.

The wrong result:
a command that never returns,
which is what the owner's rule forbids,
reached by the wrapper's own steps and not only by the person's command.
The wrapper's **queries** are bounded by the step runner
(`service/wait_ceilings.rs:378`,
`PLUMBING_STEP_CEILING`),
and the dirty restore's `clean` is one of them
(`worktree_dirty_restore.rs:222`,
`run_git`,
two minutes),
so the same restore bounds one step and not the other.

What notices it later:
nothing.
A killed wrapper leaves 1's records,
which finding 1 covers.

Suggestion,
not a decision:
run the ladder's own steps through the same bounded step runner as its
queries,
with the object ceiling for `read-tree` and a ceiling of its own for
`reset --hard`,
and leave the person's own forwarded command uncapped as native Git is.

## Locks in this route

One lock exists,
and it is Git's own index protocol:

- `<common>/worktrees/<id>/index.lock`
  (`subvolume/subvolume_index_install_into_new_worktree.rs:70`),
  created exclusively,
  never waited for,
  held from before the index is written to the moment the index has its name,
  and removed only when the name still holds the file this call made.
  It covers one index of one new worktree,
  never a repository.
  It is what makes a concurrent Git writer fail cleanly instead of losing its
  own write to a link that cannot replace an index.
  It could be removed,
  because publication is a `link` that refuses an existing name anyway,
  and the price would be that a Git command which wrote an index in that
  instant reports `File exists` instead of the wrapper's step refusing first.
  Suggestion,
  not a decision:
  keep it.

Nothing else in the route locks:

- no settlement lock exists
  (`doc/decision/cli-git-concurrent-commits.md:299-307`);
- the stage claim is a file whose name carries the owning process,
  written before the stage exists
  (`worktree_copy_stage_claim.rs:293`);
- the stage name is a random name beside the destination,
  so no two adds choose the same one
  (`worktree_copy_stage.rs:172`);
- the index install refuses rather than waits when an index or a lock is
  already there
  (`subvolume_index_install_into_new_worktree.rs:72`,
  `:83`);
- `RENAME_EXCHANGE` is the only mutual exclusion the swap needs,
  because the destination's identity is checked first
  (`worktree_snapshot.rs:399-440`);
- the leave-out's moves are renames beneath held directory handles,
  each verified by identity afterwards
  (`worktree_trash.rs:183-205`).

The migration's `index.lock` tokens
(`subvolume/subvolume_lock.rs`)
are not in this route:
`git worktree add` never calls the migration.

## Waits and their ceilings

Every wait the route can reach,
with its bound:

- The package-manager repair of the new worktree,
  one ceiling per tool:
  `REPAIR_TOOL_CEILING`,
  eighteen minutes
  (`service/wait_ceilings.rs:454`,
  `repair/package_repair_request.rs:206`).
- The trash's hand-over to the commit service:
  `START_WAIT` for the start,
  `ANSWER_CEILING` at twenty minutes
  (`service/wait_ceilings.rs:34`),
  `WITHDRAW_CEILING` at ten
  (`:345`),
  and `START_CEILING` at a minute
  (`:60`),
  configured at
  `worktree/worktree_trash_dispose.rs:228`.
- The disposal thread's join before the command ends
  (`worktree_copying/worktree_copy_after_lock.rs:166`),
  whose own work is the removal of the trash's directories,
  which is local file work with no timer.
- The bounded queries:
  `git status` and `ls-files` of the clean check,
  `ls-files --others` of the proof,
  and the dirty restore's `clean`,
  all through the two-minute plumbing ceiling
  (`service/wait_ceilings.rs:378`).
- The claim,
  the checkpoint writes and the releases never wait:
  a live owner's item is skipped,
  not queued
  (`worktree_copy_owner.rs:246-262`).

Waits with no ceiling:

- The ladder's own `read-tree` and `reset --hard` children,
  which is finding 5.
- The `fsync` of the claim's writes and the journal root
  (`worktree_copy_stage_claim.rs:98-118`,
  `private_storage::sync_directory`),
  which the owner's "Assume the final disk save would never be cut short"
  makes deliberate
  (`doc/handover/cli-git-rust-implementation.md`,
  "The final disk save is never cut short").
- The btrfs ioctls themselves:
  a snapshot call on a busy filesystem is a kernel wait this code cannot
  bound,
  which the owner's outside-our-control list covers.

## Kill points, one by one

For each state change of the route:
what a kill just before and just after it leaves,
what notices that later,
and whether work can be lost,
hidden or applied to the wrong place.
All of these were read from the code;
the ones with a control are named.

1.  Before `prepare_subvolume_destination`
    (`worktree_subvolume.rs:81`):
    nothing changed.
    After it,
    for the clone and Git rungs only:
    an empty subvolume at the destination that no record names.
    A later add at that path finds an empty directory Git accepts,
    so it is harmless;
    the destination's parent keeps an empty directory if the person never adds
    again.
    No data,
    no notice needed.
2.  Before the claim
    (`worktree_copy_stage_claim.rs:293`):
    nothing.
    After the `.stage` claim and before the `.add` record:
    a claim naming a stage that does not exist,
    which the next recovery removes without a word
    (`worktree_copy_stage_claim_reaping.rs:384-396`,
    the `present` branch is not taken).
3.  After the `.add` record and before real Git:
    the same,
    plus a `planned` record whose destination has no `.git`,
    so `settle_text` says nothing
    (`:131-141`).
4.  After real Git returns:
    the registered worktree with only `.git`,
    the `planned` record,
    and one line at the next recovery naming the destination and
    `git worktree remove --force`
    (finding 1).
5.  After `bind` and `record_binding`
    (`worktree_ladder_finish.rs:382`):
    the record names both identities,
    and the recovery's line offers `reset --hard` and `remove --force`
    (finding 1,
    finding 4 when an index lock is in the way).
6.  After the snapshot ioctl,
    the `chmod`,
    the own-index read,
    the leave-out or the stub removal
    (`worktree_snapshot.rs:283-330`):
    a stage subvolume holding a copy of the source,
    a claim whose owner is dead,
    and a destination that is Git's directory.
    The recovery reports the stage and never removes it
    (`worktree_copy_stage_claim_reaping.rs:384-396`),
    because a claim names a path and a path proves nothing.
    Nothing of the person's is lost:
    the stage is a copy,
    and the source is never written.
7.  During `install_index`
    (`worktree_snapshot_index_step.rs:100`):
    the administrative directory can keep `index.lock`
    (finding 4),
    and either no index or the finished index.
8.  Between removing the stage's `.git` and writing the pointer
    (`worktree_snapshot.rs:263`,
    `replace_git_pointer`):
    the stage has no `.git`,
    which the recovery reports as a stage and never removes.
    The destination is untouched.
9.  During `restore_dirty` or `apply_difference`:
    the stage's index and files are partly updated.
    The stage is reported and destroyed by nothing;
    the destination is untouched.
10. Before the exchange,
    after `put_in_place`'s checks:
    nothing changed yet.
    After the exchange and before the identity check that follows it:
    the destination is the snapshot and the stage holds Git's empty directory.
    If the process dies there,
    the next recovery says the add is "possibly incomplete" for a worktree that
    is whole,
    and changes nothing.
    A false alarm,
    which is the safer error.
11. During `settle_displaced`:
    the stage name holds Git's directory,
    with its pointer or without it,
    and is reported as one more stage.
    Nothing of the person's is moved or deleted:
    entries another process added are renamed into the new worktree or left
    where they are.
12. After `mark_complete` and before `release_claim`:
    a marker and a claim,
    so the recovery drops all three records silently
    (`worktree_copy_stage_claim_reaping.rs:64-72`,
    `settle_marker`).
    After part of the release:
    whichever of the record,
    the marker and the claim is left is removed without a notice,
    except that a record left by a reordered unsynced removal is never swept
    (`release_unsynced`,
    `worktree_copy_stage_claim.rs:152`).
    A few unread bytes,
    nothing else.
13. After the release:
    nothing of the route's is left.
    The trash and the package repair run beside the command and are the
    removal audit's and the copy audit's.

The clone rung adds no state of its own before the swap,
because it fills the destination in place:
a kill during the clones leaves the entries it created in the destination,
which Git's own checkout then overwrites,
and the person is told nothing about them until the fallback's line
(`worktree_clone.rs:233`,
`leftover_report`).

## Races between two concurrent commands

- Two adds name the same destination.
  Measured,
  five runs of each shape,
  the fourth control under "Reproductions":
  real Git refuses the loser with `fatal: '../dest' already exists`,
  whether or not the destination was created first as an empty subvolume,
  because the winner has written its `.git` file before the loser reads the
  directory.
  The wrapper's Git step then fails,
  no rung runs,
  and the claim of the loser is released
  (`worktree_ladder_finish.rs:533-538`).
  Nothing of the loser's is left.
  The residual window is Git's own:
  both processes would have to pass the "is this directory empty" test before
  either writes its `.git` file,
  which the wrapper widens not at all in these five runs.
- Two adds in one repository whose destinations differ:
  each claims its own random stage name,
  writes its own records,
  and exchanges only its own destination.
  The one shared name is the folder that holds the worktrees,
  which the placeholders step reads
  (`worktree_snapshot_placeholders.rs:180`);
  it removes only empty inode 2 stand-ins,
  so a directory that the other add has meanwhile filled is refused by the
  empty-directory removal and stays.
- A destination inside another worktree of the same repository,
  for example `repo/.claude/worktrees/a/nested`:
  read from the code and not measured.
  The plan accepts it because its parent exists,
  the source is still the worktree the command runs in,
  and the stage is made beside the destination,
  so inside `a`.
  The snapshot of the source holds only a stand-in for `a`,
  so `remove_stubs` removes that stand-in and then finds the destination's path
  already gone,
  which it treats as removed
  (`worktree_snapshot.rs:242`,
  `:55`).
  The exchange is between two names inside `a`,
  so the destination becomes the snapshot of the source,
  which is what the command asked for,
  and nothing of `a` is moved or removed:
  the only names the rung touches inside it are the stage's and the
  destination's.
- An add from the main checkout and a copy into a worktree beside it:
  finding 2.
  The two never contend on a name,
  and the copy's container is carried because the leave-out rules do not know
  its name.
- An add whose source is another worktree of the repository:
  `snapshot_refusal` and `clone_refusal` ask only about the source and the
  destination's filesystems,
  and the source's cleanliness,
  so an add from worktree A does not read or write worktree B's state.
  Its snapshot carries A only,
  and the ignored-state copy is the copy audit's route.
- An add and the removal of a nested worktree of its source:
  the registered roots are read before Git runs
  (`worktree_snapshot_source.rs:76`),
  so a root removed meanwhile is still named;
  the leave-out then moves its copy inside the stage,
  and never the original,
  because every move names a directory beneath the stage's own handle
  (`worktree_trash.rs:183-205`).
- An add and a person's Git command in the source:
  a held `index.lock` or an unfinished operation refuses the cheap rungs
  (`worktree_clean.rs:57`),
  so the add falls to Git's own checkout:
  no snapshot of a moving index is ever taken.
  A write to a tracked file between the clean check and the snapshot is caught
  by the content walk
  (`worktree_snapshot_index_step.rs:100-160`)
  and a main checkout is restored instead,
  which is the accepted-risk line,
  not a finding.
- An add and a `git worktree move` of its own destination:
  every write into the worktree is preceded by `verify_association`
  (`worktree_add_binding.rs:353`),
  which compares the identities and the pointer's bytes,
  and the swap re-checks the destination's identity before and after the
  exchange,
  so a moved or replaced destination is never written into and never reset.
  The two controls in
  `worktree_ladder_finish_tests.rs`
  move and replace the destination at exactly those points.
- An add and another add's **snapshot** stage in the same folder:
  the stand-in that btrfs shows for it is removed as an empty directory,
  and a stand-in that gained anything is refused by the removal,
  so nothing of the other add's stage can be destroyed from this one's
  snapshot
  (`worktree_snapshot_placeholders.rs:112-140`).

## What I checked and found sound

- The source is never written by the route.
  Every Git run of a filling rung names the stage and the new worktree's Git
  directory,
  `restore_dirty` refuses to start when the stage is the source's own
  directory or the Git directory is the source's
  (`worktree_dirty_restore.rs:206`),
  and every read of the source uses `--no-optional-locks`
  (`worktree_snapshot_source_changed_paths.rs:161`).
- The source's `.git` cannot reach the new worktree.
  `replace_git_pointer` removes the snapshot's `.git` whole before writing
  Git's pointer
  (`worktree_snapshot.rs:263-268`),
  so a held `index.lock`,
  a stale `MERGE_HEAD`,
  or a half-written index of the source is not carried anywhere;
  and a source that held `index.lock` was never eligible for the snapshot in
  the first place
  (`worktree_clean.rs:57`,
  `worktree_clean_stat_verdicts.rs:86`,
  where the operation-in-progress kind is not restorable).
- The swap cannot leave a half-written destination.
  `RENAME_EXCHANGE` is one call,
  the destination is checked to be the directory Git made by identity before
  it,
  and the displaced identity is checked after it
  (`worktree_snapshot.rs:399-440`).
  A destination that changed is swapped back and reported,
  and the stage is destroyed only when the name still holds this process's
  object
  (`worktree_ladder_run.rs:296`,
  `discard_stage`).
- Files another process added to the destination are not deleted.
  `settle_displaced` renames them into the new worktree without replacing
  anything,
  removes only a `.git` whose bytes are the ones Git wrote,
  and refuses the empty-directory removal when anything is left
  (`worktree_snapshot.rs:445-500`).
- A destination that is not the add's is never written into.
  `bind_registration` proves the pointer and the registration name each other,
  and `verify_association` repeats the proof before every write and after every
  checkout
  (`worktree_add_binding.rs:211`,
  `:353`).
- The index the new worktree starts with is published without replacing
  anything:
  the install refuses an existing index and an existing lock,
  and it links rather than renames over the name
  (`subvolume_index_install_into_new_worktree.rs:76-100`).
- A `.git` pointer that Git wrote relative (`--relative-paths`,
  `worktree.useRelativePaths`) is resolved lexically against the registration,
  and a pointer that reaches the registration through a link is refused with
  a message that names the link
  (`worktree_add_binding.rs:282-330`).
- `git worktree move` of a worktree whose root is a btrfs subvolume,
  which is the shape the snapshot rung leaves at the destination:
  measured,
  the fifth control under "Reproductions".
  It is a rename,
  the root stays inode 256,
  the `.git` pointer keeps its bytes,
  the registration is rewritten,
  and `git status` in the moved worktree is clean.
  `move` is forwarded like any other command
  (`package/git-policy/cli/SPEC.md`,
  "Linked-worktree ignored-state synchronization"),
  and the ladder never names a worktree by a path it recorded in an earlier step.
- A dirty main checkout's uncommitted edits cannot reach the new worktree.
  The restore ends with a proof that the index equals the new commit,
  that no untracked file is left,
  and that no tracked file differs
  (`worktree_dirty_restore.rs:171`),
  and a proof that fails destroys the stage and lets Git's own checkout make
  the worktree
  (`worktree_ladder_finish.rs:453`),
  which is why the second control's dirty worktree has an empty `git status`.
- A running build in the source is three cases,
  none of them a stale lock:
  a build that writes only ignored files is carried as of the snapshot's
  instant,
  which the accepted-risk line covers;
  a build that writes a tracked file is caught by the content walk and the
  restore;
  a build that holds `index.lock` refuses both cheap rungs,
  so no snapshot of a moving index is taken.
- No wait in the route is unbounded except finding 5 and the deliberate
  disk sync.
- One lock,
  Git's own,
  never waited for.
- `git worktree add --lock` and `--reason` are in the plan's table
  (`worktree_add_plan.rs:282`),
  and the swapping and the settling touch only the destination and the stage,
  so a locked worktree is made whole like any other.

## Reproductions

All the controls ran on this host,
`/var/mnt/encrypted` on `/dev/mapper/crypt_sda`,
btrfs with `user_subvol_rm_allowed`,
six CPUs,
with real Git 2.56.0 at `/usr/bin/git`.
The three Rust controls are in
`package/git-policy/cli/src/native/worktree/audit_scratch_tests.rs`,
run as
`mise exec -- cargo nextest run --locked --offline --lib audit_scratch --no-capture`
with `CLI_GIT_TEST_ROOT` under
`${HOME}/temp/agent/pi-add-route-audit/fixtures`.
Each drives the seam:
`begin_ladder` chooses the rung,
real Git runs the `--no-checkout` add,
and the rung is finished through the ladder's own `finish`.
The two shell controls are
`${HOME}/temp/agent/pi-add-route-audit/same-destination.ts` and
`${HOME}/temp/agent/pi-add-route-audit/lock-and-move.ts`,
which start `/usr/bin/git` directly,
never the installed wrapper.

### A stage of another add carried into a snapshot

The first Rust control,
finding 2.
The stage container an add in flight makes was put in the source by hand,
because a real one exists only while its copy runs,
and the snapshot was taken through the ladder.

```text
CONTROL 1 steps: []
CONTROL 1 the ladder served the add: Some("/var/mnt/encrypted/temp/agent/pi-add-route-audit/fixtures/native-unit-3599826-audit-stage-carried/repo/.claude/worktrees/topic")
CONTROL 1 the stage is still in the source: true
CONTROL 1 the stage's payload is in the new worktree: true
CONTROL 1 the new worktree's .claude/worktrees lists: Ok([".cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543"])
```

`steps: []` is the control that the snapshot served the add with no Git step of
its own,
and the payload of the other add's stage is in the new worktree.

### The ignored files a restored snapshot drops

The second Rust control,
finding 3.
Both adds check out the branch `other`,
whose commit's `.gitignore` ignores nothing;
the only difference is one uncommitted edit to a tracked file in the second.

```text
CONTROL 2 the clean check of the fixture: Ok(2)
CONTROL 2 the clean add's steps: [["-C", ".../.cli-git-worktree-copy-845a9716-cf07-4840-91c8-b3cd0fbcabf1", "read-tree", "-m", "-u", "74b262918577eb405e6f986c50e460b0a67c25cd", "95563cc0c59b5cbd5aa4c434b49bd2fd084e0b44"]]
CONTROL 2 the clean add carried: Some(".../audit-ignore-rules/clean")
CONTROL 2 clean: cache/data.bin in the new worktree: true
CONTROL 2 clean: git status in the new worktree: "?? cache/"
CONTROL 2 the dirty add's steps: [["-C", ".../.cli-git-worktree-copy-913f04a2-b66f-4300-ad22-5772c68e7e0b", "read-tree", "--reset", "-u", "95563cc0c59b5cbd5aa4c434b49bd2fd084e0b44"]]
CONTROL 2 the dirty add carried: Some(".../audit-ignore-rules/dirty")
CONTROL 2 dirty: cache/data.bin in the new worktree: false
CONTROL 2 dirty: git status in the new worktree: ""
```

The clean source is the control:
its rung applies the tree difference (`read-tree -m -u`) and the ignored file
reaches the new worktree.
The dirty source's rung restores (`read-tree --reset -u`,
then the captured `clean` the runner does not see),
and the ignored file is gone,
with the add's own line saying only that the uncommitted changes were left out.

### An interrupted add reported once

The third Rust control,
finding 1.
The claim is written on behalf of a process that no longer exists,
real Git registered the destination,
the add bound itself to it,
and the snapshot was there.
Recovery was then run twice,
as the next two commands would run it.

```text
CONTROL 3 what the first later command printed:
monochromatic-git-policy-cli: the stage ".../.cli-git-worktree-copy-f9f21b81-3e31-461d-a2d6-05f8fdf4c6ea" was left by an interrupted `git worktree add` and was not changed; remove it yourself once nothing needs it (`btrfs subvolume delete` for a snapshot, `rm -r` otherwise).
monochromatic-git-policy-cli: an interrupted `git worktree add` left ".../.claude/worktrees/topic" registered and possibly incomplete; it was not changed. `git -C ".../.claude/worktrees/topic" reset --hard` completes its checkout, and `git worktree remove --force ".../.claude/worktrees/topic"` removes it.

CONTROL 3 records left after that command: []
CONTROL 3 the stage is still there: true
CONTROL 3 the worktree holds: Ok([".git"])
CONTROL 3 git worktree list:
.../repo                         3e9fe7c [main]
.../repo/.claude/worktrees/topic 3e9fe7c [topic]
CONTROL 3 git status in that worktree:
D  .gitignore
D  tracked.txt
CONTROL 3 what the second later command printed: ""
CONTROL 3 the worktree is still registered and empty: true / Ok(1)
```

The records are gone after the one line,
the stage and the empty registered worktree stay,
and the second command says nothing.

### Two adds naming one destination

`same-destination.ts`,
five runs of each shape.
The empty subvolume of the second column is what
`prepare_subvolume_destination` makes.

```text
(a) destination absent: statuses [ 0, 128 ]
(a) registrations: 2
(a) the failure said: "fatal: '../dest' already exists"
(b1) empty destination present: statuses [0,128]; registrations [".../repo",".../dest"]
(b1) the failure said: "fatal: '../dest' already exists"
(b1) git status in the destination: ""
(b2) empty destination present: statuses [0,128]; registrations [".../repo",".../dest"]
(b2) the failure said: "fatal: '../dest' already exists"
(b3) empty destination present: statuses [128,0]; registrations [".../repo",".../dest"]
(b3) the failure said: "fatal: '../dest' already exists"
(b4) empty destination present: statuses [0,128]; registrations [".../repo",".../dest"]
(b5) empty destination present: statuses [0,128]; registrations [".../repo",".../dest"]
```

One add wins in every run and the other is refused,
with the same refusal whether or not the wrapper had made the destination
first.

### A stale index lock, and a worktree moved

`lock-and-move.ts` and `move-subvolume.ts`.
The lock is the one `install_index_in_with` creates,
placed by hand as a kill inside it leaves it.

```text
(1) a Git command in that worktree:
    git status -> 0 ""
    git reset --hard -> 128 "fatal: Unable to create '.../repo/.git/worktrees/empty/index.lock': File exists.\n\nAnother git process seems to be running in this repository, or the lock file may be stale"
```

```text
before the move: inode 256 files [".git","tracked.txt"]
after the move: inode 256
the .git file is the same text: true
git status in the moved worktree: ""
registrations: [".../repo",".../wt-moved"]
```

A subvolume root is inode 256 on btrfs,
and `btrfs subvolume show` needs root on this host,
so the inode is what the control reads.

## What I could not determine

- Whether a kill between `mark_complete_unsynced` and the release can leave a
  marker whose disk copy did not survive,
  which the next recovery reads as an interrupted add.
  The unreleased records would produce one false alarm,
  and I did not make the kernel lose the marker.
- Whether Git ever removes a destination directory that it did not create,
  which decides how long an empty subvolume the wrapper made can stay behind
  after a late Git failure.
  I could not make Git 2.56.0 fail after filling a pre-created destination.
- Whether a worktree made by the snapshot rung can be moved onto another
  filesystem by `git worktree move`:
  Git's own `rename` decides,
  and I measured only the same-filesystem rename.
- Whether a repository whose main checkout is a `.git` file
  (`--separate-git-dir`) reaches the dirty path at all:
  `restorable_here` requires `git_dir == worktree_root/.git`
  (`worktree_snapshot_source.rs:545`),
  which such a checkout fails,
  so it takes the clone rung or Git's checkout and I did not measure which.
- Whether finding 2 can happen when the destination lies outside the source:
  it cannot,
  because a stage container is made beside its destination and is then outside
  the snapshot too,
  which I read from `new_stage_container` and did not need to measure.
- The height of finding 3's cost in a real repository:
  it depends on whether the target branch's `.gitignore` covers the source's
  ignored names,
  which differs per repository.

## What to inspect and how to respond

Inspect,
in this order:

1.  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_claim_reaping.rs:114-160`
    and the record's release,
    for finding 1,
    with the third control's output beside them.
2.  `package/git-policy/cli/src/native/worktree/worktree_trash.rs:321-440`
    for finding 2,
    and the `STAGE_PREFIX` check that the walk would need.
3.  `package/git-policy/cli/src/native/worktree/worktree_dirty_restore.rs:205-245`
    for finding 3,
    with the second control's two worktrees.
4.  `package/git-policy/cli/src/native/subvolume/subvolume_index_install_into_new_worktree.rs:66-110`
    for finding 4.
5.  `package/git-policy/cli/src/native/invocation/forwarding.rs:187`
    and every `run_step` caller for finding 5.

To respond,
each finding is a suggestion and none is a decision:
say which ones to fix,
and whether the recovery record of finding 1 should outlive the line,
which is the one change here that alters a behavior the SPEC states
(`package/git-policy/cli/SPEC.md`,
"Linked-worktree ignored-state synchronization").
