# Worktree removal and conversion-route findings on `main`

Status:
audit,
2026-11-08T22:25:01-05:00.
Audited revision:
`main` at `20acf6f904`,
the base of the branch `docs/cli-git-worktree-removal-route-findings-on-main`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-worktree-removal-route-findings-on-main`.
`main` has since moved to `cf63cacad8`;
`git diff --name-only 20acf6f904..main` changes forty-nine paths and
none of them is under
`package/git-policy/cli/src/native/worktree/`,
`package/git-policy/cli/src/native/subvolume/`,
`package/git-policy/cli/src/native/lock/`,
`package/git-policy/cli/src/native/landing/` or
`package/git-policy/cli/src/native/process/`,
so every `path:line` below holds on both revisions.
No production code was changed and no test was committed.
The throwaway controls are in
`package/git-policy/cli/src/native/subvolume/audit_scratch_tests.rs`
in that worktree only,
and their output is quoted under "Reproductions".
Linux only;
the owner's pause covers Windows and macOS.

Scope:
what removes,
renames or replaces whole directories:

- `git worktree remove` and `git worktree prune` through the wrapper
  (`worktree_remove*.rs`,
  `worktree_removal_rescue.rs`,
  `worktree_removal_stops_units.rs`,
  and the plan and forward files that reach them);
- the btrfs ladder's renames and replacements
  (`worktree_ladder_run.rs`,
  `worktree_ladder_finish.rs`,
  `worktree_snapshot.rs`,
  `worktree_subvolume.rs`),
  not its decision to add a worktree,
  which `doc/planning/cli-git-copy-route-findings-on-main.md` covers;
- converting existing worktrees
  (`worktree_convert*.rs`);
- converting and undoing a main checkout
  (`subvolume_migrate.rs`,
  `subvolume_rollback.rs`,
  `subvolume_preflight.rs`,
  `subvolume_lock.rs`);
- the trash the snapshot leave-out fills,
  and every sweep of it
  (`worktree_trash*.rs`).

The brief called the trash "the trash the removal moves folders into".
No removal route moves anything into it:
the trash is made by the snapshot rung of `git worktree add`
(`worktree_trash.rs:321`,
`prune_snapshot`),
and it holds the directories that rung leaves out of a new worktree.
Its disposal deletes whole trees,
so it is audited here;
whether the leave-out chose the right directories is the add route's question.

Out of scope:
the copy of ignored state into a new worktree
(the audited page above),
the landing and push routes,
and the token conversion of call sites on other branches.

## What the handover already records

`doc/handover/cli-git-rust-implementation.md`,
"Open review findings",
records nothing about this route.
Its open items are the copy route's nine findings and eight off-`main` fixes,
the audit-rotation generations,
the accepted downgrade limit,
the push route's twelve findings,
and the four Critical findings of the commits-never-fail review.
Its worktree material that could be mistaken for a review of this route is:
the lock-free settlement's supersession
(`doc/decision/cli-git-concurrent-commits.md:299-307`),
the remaining conversions and their two summary counters
(handover,
"Conversion of the remaining worktrees and its small fixes"),
the registration comparison fixed in `1486f5c112`,
and the temporary journal files swept in `d4c213edf0`.
The seven findings below are not among those,
and each of the four fixed items above was re-read and is present in the code
this page cites,
so none of them is reported again.

## The owner's decisions this page judges by

Verbatim,
from the brief and from the handover:

- "never discard user data".
- "nothing should ever wait forever or indefinitely"
  (`doc/handover/cli-git-rust-implementation.md`,
  "Nothing waits forever",
  2026-11-03 about 12:37-05:00),
  and what happens at a bound:
  "Return with an error",
  with the saved work kept.
- "Use as few locks as possible."
- "We should never invoke ourselves recursively..."
- Races with other programs,
  `doc/handover/cli-git-rust-implementation.md`,
  "Worktree add and package managers",
  2026-11-03T00:15:33-05:00:
  "That is not an our problem.
  By using this command or this program,
  make the user understand and accept the risk."
  The adopted reading there:
  races caused by other processes changing files or names while the wrapper works
  are outside what it defends against,
  and it states that risk plainly,
  and it keeps fixing its own deterministic bugs.
  This page uses that line to separate the findings that are ours,
  which are deterministic,
  from the ones the accepted-risk notice covers.
- "cli-git never deletes a lock"
  (`doc/decision/cli-git-concurrent-commits.md:166`).
  Both routes that hold Git's `index.lock` delete only a lock whose bytes are
  their own token
  (`subvolume_lock.rs:128`,
  `is_ours`,
  `worktree_convert_phases.rs:187`,
  `release_index_lock`),
  which honors that decision.
- `doc/decision/cli-git-rust-rewrite.md`:
  "Preserve the existing required policy,
  transaction,
  concurrency,
  recovery,
  worktree,
  and push behavior."
  Nothing in this route was preserved from the TS cli-git's worktree removal,
  so this page has no parity question to answer;
  the decisions that bind are the four rules above.

## The route as it runs on this host

`git worktree remove <path>` with no option,
in a repository whose worktree directory is a btrfs subvolume,
on a mount with `user_subvol_rm_allowed`:

1.  Before anything else,
    every wrapped command settles and takes removal snapshots
    (`invocation/wrapped_command_notices_snapshots_and_manual_push.rs:118`,
    `:203`).
    A snapshot copies the unshown push-failure notices and the unfinished push
    records of every linked worktree that has any into
    `<common>/monochromatic-git-policy-cli/removal-snapshots/<instance>`
    (`worktree_removal_rescue.rs:58`,
    `:343`),
    and records the taking process in an `attempts` file
    (`:218`).
2.  The plan accepts only the plain form
    (`worktree_remove_plan.rs:53`,
    `parse_remove_plan`);
    `--force` and every other form go to real Git.
3.  `remove_with` (`worktree_remove.rs:492`)
    reads the registration of the repository the command runs in
    (`worktree_remove_registry.rs:150`,
    `registration_of`),
    refuses a locked worktree,
    a non-subvolume,
    a mount without the option,
    and a worktree it cannot open without following a link,
    then proves the tracked files clean in process plus one `git ls-files`
    (`:527`),
    looks at the lock and at the identities of the directory,
    its parent and the registration once more
    (`:531`,
    `confirm`),
    and destroys the subvolume by id,
    or by name below the held parent
    (`:535`).
4.  `finish` (`:436`)
    empties the administrative directory through the held descriptor
    (`:452`),
    removes its name from `worktrees`
    (`:458`),
    and removes `worktrees` itself when it is empty.
5.  `ForwardedRun::Ran(0)` from the served path prints the destroy notice
    (`worktree_copy_forward.rs:332-341`)
    and stops the removed worktree's commit service units
    (`worktree_removal_stops_units.rs:110`).
6.  On the next command,
    `settle_removals`
    (`worktree_removal_rescue.rs:515`)
    decides each snapshot:
    the Git directory is gone,
    so the removal happened,
    and the records are promoted to the orphan directory;

    the Git directory is there and the taker has ended and the checkout is
    there,
    so the removal was refused,
    and the snapshot is discarded;

    anything else is undecided and the snapshot is kept.

`git worktree prune` is not served at all:
it takes the snapshots in step 1 and then real Git runs,
which deletes the administrative directories of registrations whose directory
is gone.

`git monochromatic-git-policy-cli convert-worktree-to-subvolume <path>`
(`worktree_convert.rs:147`,
`convert_worktree`):

1.  `inspect` reads the `.git` file,
    the administrative directory and the
    back-pointer (`worktree_convert_inspection.rs:83`).
2.  A lock-free numbered claim gives one command the worktree
    (`single_owner_claim.rs:348`,
    `take_claim`),
    and a probe retires the flock file an older release left
    (`worktree_convert_journal.rs:541`,
    `legacy_lock_is_held`).
3.  The journal is created exclusively
    (`worktree_convert.rs:255`,
    `create_new`).
4.  `copy_phase` creates the sibling `<path>.cli-git-converting` as a subvolume,
    records its identity in the journal,
    copies with clones,
    then takes Git's `index.lock` with its token
    (`worktree_convert_phases.rs:227`)
    and copies a second time with deletions to prove the copy equal
    (`:229`).
5.  `swap_phase` exchanges the two directories with `RENAME_EXCHANGE`
    (`:319`),
    so the original tree is at the converting name from then on.
6.  `verify_swap` checks the installed identity and the `.git` bytes.
7.  `index_phase` rewrites the index stat fields under the held lock and then
    releases it (`:394`).
8.  `removal_phase` scans for processes inside the original tree
    (`:423`),
    compares the original tree against the installed one
    (`worktree_convert_compare.rs:40-70`,
    `first_difference`,
    which compares type,
    size and modification time,
    and a link's target),
    and deletes the original only when nothing differs.
9.  `finish` removes the journal
    (`worktree_convert_phases.rs:453`).

`git monochromatic-git-policy-cli migrate-to-subvolume` and its
`--rollback`,
`--remove-old` and `--dry-run` modes
(`subvolume_command.rs:144`,
`execute`):

- a claim gives one command the migration
  (`subvolume_preflight.rs:185`,
  `claim_prefix`);
- preflight refuses a non-btrfs mount,
  a mount point,
  a checkout that is a subvolume already,
  leftovers nothing records,
  any Git operation in progress,
  a live conversion claim in any linked worktree
  (`:319`,
  `conversion_refusal`),
  unfinished service work,
  and too little free space
  (`:228`,
  `migrate_refusals`);
- the copy runs in passes into `<root>.subvol-new`,
  then `index.lock` of the checkout and of every linked worktree is created
  with a token
  (`subvolume_lock.rs:146`,
  `acquire`,
  never waited for),
  the last delta runs,
  the two directories are exchanged,
  the original becomes `<root>.before-subvolume`,
  newer files are carried over,
  the indexes are rewritten,
  and the locks are released
  (`subvolume_migrate.rs:189`,
  `exchange_trees`,
  `:275`,
  `finish`);
- the original tree is never deleted by the migration itself;
  `--remove-old` does that on request
  (`subvolume_rollback.rs:299`,
  `remove_old_tree`),
  after carrying files newer than the catch-up into the checkout
  (`:273`,
  `carry_late_writes`),
  and `--rollback` swaps the trees back,
  carries the new root's newer files into the old tree,
  and deletes the new subvolume deepest first
  (`:196`,
  `rollback`).

The trash
(`worktree_trash.rs:321`,
`prune_snapshot`):

- the snapshot rung renames the directories the main-checkout rule leaves out
  into `.claude/cli-git-trash/<40 hex digits>` inside the new worktree
  (`:62`,
  `:209`,
  `make_trash`),
  and records the identity of every moved directory in a manifest;
- the add hands the manifest to a thread that runs beside the package repair
  (`worktree_copy_after_lock.rs:196-210`),
  which writes a trash work item first
  (`worktree_trash_dispose.rs:209`,
  `enqueue`),
  then either wakes the commit service or empties the trash itself
  (`:165`);
- the service's handler
  (`service_work_drain_pending_claims.rs:191`)
  empties it and settles the item;
- `delete_trash` verifies the worktree's `.git` file points at the recorded Git
  directory (`worktree_trash_dispose.rs:301`,
  `:255`),
  the trash's identity against the manifest,
  and every entry's identity,
  and refuses when the trash holds anything the manifest does not name
  (`:339-344`).

## Findings ranked by what they risk losing

### 1. The deletion of the old tree deletes a file the carry left out

Reproduced;
the first block under "Reproductions".

Sequence:

1.  `git monochromatic-git-policy-cli migrate-to-subvolume` finishes.
    The checkout is a subvolume and the original tree is kept at
    `<root>.before-subvolume`
    (`subvolume_migrate.rs:320`,
    `closing`).
2.  The person puts a file back into the old tree,
    for example by unpacking an archive or a `cp --preserve=timestamps` of a
    backup.
    The file's modification time is older than the migration's catch-up.
3.  `git monochromatic-git-policy-cli migrate-to-subvolume --remove-old` runs.
4.  `carry_late_writes` copies the old tree into the checkout in one
    update-only pass whose rule is
    `missing from the checkout and older than the catch-up, so leave it out`
    (`subvolume_run.rs:169`,
    `created_since: Some(since)`;
    `subvolume_sync.rs:145-152`,
    `self.stats.left_out += 1`).
5.  `remove_old_tree` deletes the whole old tree
    (`subvolume_rollback.rs:324`,
    `remove_tree(&old_path)`),
    and the command exits 0
    (`subvolume_command.rs:198-203`).

The person sees one line with a count and no name,
printed between the pass and the deletion:

```text
late writes (files written to the old tree after the catch-up): 1 entr(ies) absent from the destination were left out because they predate the catch-up
deleting /.../checkout.before-subvolume (every file is unlinked; this can take minutes)
```

(`subvolume_run.rs:211`).
There is no pause and no flag to hold the deletion,
so the file is gone before anything can be done about it,
and `remove_old_refusals`
(`subvolume_preflight.rs:414-429`)
asks for the journal's `verify` step and for the old tree's existence,
never for what the carry left out.

Why the count exists:
the same rule keeps the pass from resurrecting a file the person deleted from
the checkout after the catch-up,
so `left_out` is not by itself a refusal.

What it risks:
a person's file,
with no name ever printed and no second copy.

What notices it later:
nothing.
The line names a count,
the tree is deleted,
and the journal goes with it.

Suggestion,
not a decision:
collect the paths the pass leaves out the way the unreadable list is collected
(`subvolume_rollback.rs:314-319`),
print them,
and either refuse the deletion until the person passes an explicit flag
beside `--discard-unreadable`,
or keep the old tree when the list is not empty.
Either way the rule that keeps a deliberate deletion deleted stays,
because the list would name both cases.

### 2. A removal snapshot for a registered worktree whose checkout is gone is never settled

Reproduced;
the rescue block under "Reproductions",
with both decided states run as controls.

Sequence:

1.  A linked worktree W has an unshown push-failure notice or an unfinished push
    record in its Git directory.
2.  W's checkout directory is gone while its registration remains.
    Two ordinary ways to reach that:
    `git worktree remove W` was killed after the destroy and before the
    registration went (finding 4),
    or the person deleted the directory by hand,
    which is the state `git worktree prune` exists for.
3.  Any wrapped command runs `git worktree remove` or `git worktree prune`,
    so `snapshot_linked_worktrees` copies W's records
    (`worktree_removal_rescue.rs:343`).
4.  The taker ends without removing W.
5.  Every later command's `settle_removals` decides W as follows:
    the Git directory is present,
    no taker runs,
    and `checkout_present` is false because the registration's `gitdir` file
    names a directory that is not there
    (`:282`,
    `:308-317`),
    so `removal_of` answers `Undecided` and `settle_removals` skips it
    (`:526`).

The snapshot stays in
`<common>/monochromatic-git-policy-cli/removal-snapshots/<instance>` for good,
and the records in it are never promoted to the orphan directory,
so no command ever shows them.
Nothing sweeps the snapshot directory by age;
`rg 'SNAPSHOT_DIRECTORY'` finds the copy,
the settle and the removal of a
settled snapshot,
and nothing else.

What it risks:
a notice the person was promised.
The bytes stay,
and the same bytes also stay in W's Git directory,
but no command can read them:
`take_push_failures_for` is shown for the Git directory of the worktree a
command runs in,
and a worktree without a checkout cannot run one.
So a push failure that was never shown is never shown,
which is exactly the loss the rescue route exists to prevent,
and the snapshot holds disk until the registration is pruned.

Why the third state exists at all:
`checkout_present` false was read as "the removal may be under way or was cut
short",
which is right for the taker's own removal and wrong for a registration whose
directory was deleted long ago.

Suggestion,
not a decision:
a snapshot whose Git directory is present,
whose taker has ended,
and whose checkout has been gone longer than the taker's patience is a
removal that will never finish;
promote its records and leave the registration to the person,
or report the snapshot each time with the path and the command that settles it.

### 3. A conversion interrupted past its swap leaves a full copy that nothing can clean

Read from the code.

Sequence:

1.  `convert-worktree-to-subvolume W` is killed between the exchange and the end,
    or fails in the index phase,
    which keeps the original tree at `W.cli-git-converting` on purpose
    (`worktree_convert_phases.rs:361`,
    `:394`,
    `:434-446`).
    So after every interruption past the swap,
    `W` is the new subvolume and the original tree sits at the converting name.
2.  The person then removes W,
    with `git worktree remove W` or with `git worktree remove --force W`.
3.  The removal route removes W's directory and W's registration.
    Nothing consults the conversion:
    `worktree_remove.rs` reads no claim,
    and `rg 'live_owner_of'` over the removal modules finds none.
    When the conversion's lock is still held the removal is refused
    (`check_clean` at `:527` sees `index.lock`),
    and when the lock is released (after the index rewrite,
    `worktree_convert_phases.rs:394`) it is served and destroys W.
4.  The conversion's journal lived in the registration that step 3 deleted,
    so no `--abandon` can act:
    `inspect` refuses first,
    because `W` has no `.git` entry
    (`worktree_convert_inspection.rs:87`),
    and the fallback message about the converting name
    (`:276`)
    is only reached for a worktree that still exists.

What is left:
`W.cli-git-converting`,
a complete copy of the worktree as it stood at the swap,
including its ignored state,
with no journal,
no claim,
and no record anywhere that it is a leftover.

What it risks:
disk,
and a copy of the worktree beside where it used to be.
For a person who removed a worktree because of what it held,
the copy outlives the removal,
under a name only the wrapper uses.
No person's file is lost.

What notices it later:
nothing,
unless the same path is registered and converted again,
which cannot happen while W is unregistered.

Suggestion,
not a decision:
sweep the directory that holds the registered worktrees for a
`<name>.cli-git-converting` whose worktree has no registration and whose journal
is gone,
the way `sweep_temporary_files`
(`worktree_convert_journal.rs:332`)
sweeps its own leftovers on every journal write;
and let the removal route refuse while
`<path>.cli-git-converting` exists,
which is one `symlink_metadata` in the same steps as the lock check.

### 4. A kill between the destroy and the removal of the registration leaves a stale registration

Read from the code;
the Git half of it measured,
the fourth block under "Reproductions".

Sequence:

1.  `git worktree remove W` is served,
    and the destroy succeeds
    (`worktree_remove.rs:535`).
2.  The process is killed before `finish` removes the registration
    (`:436-461`),
    or `finish` fails and reports `Incomplete`
    (`:397`,
    `incomplete_report`).
3.  W's files are gone and `<common>/worktrees/<id>` remains with its `gitdir`
    file naming W.

What notices it later:
`git worktree list` marks the entry prunable,
and the next `git worktree add` at the same path fails:

```text
fatal: '../wt' is a missing but already registered worktree;
use 'add -f' to override, or 'prune' or 'remove' to clear
```

which is a complete diagnosis for a person.
Nothing inside the wrapper sweeps it:
the settle of finding 2 keeps the snapshot,
and no command runs `prune` by itself.

What it risks:
a command the person must repair with the command Git names,
and a removal snapshot that never settles,
which is finding 2's loss attached to a second cause.

The window exists in native Git too,
in the same order,
measured with `strace` on a fixture
(output under "Reproductions"):
`git worktree remove --force` unlinks `<wt>/.git` and removes `<wt>` before it
unlinks anything under `.git/worktrees/wt`,
so a kill after the directory goes leaves the same stale registration.
Neither order loses data the person did not ask to remove;
the wrapper's window is the wider one,
because `finish` empties and removes the registration through its own proof
and can fail where Git's unlinks would not.

Suggestion,
not a decision:
record the destroy in the add's own claim style before the ioctl,
so a later command can finish the registration,
which is the shape `worktree_removal_rescue.rs` already uses for the records.

### 5. A trash whose emptying fails is dropped, and nothing sweeps the trash parent

Read from the code.

Sequence:

1.  A snapshot rung leaves a directory out of a new worktree,
    for example a cache that itself holds a btrfs subvolume or a mount,
    and moves it into `.claude/cli-git-trash/<40 hex digits>`.
2.  The disposal compares every directory's device with the trash's
    (`worktree_trash_remove.rs:152-158`),
    so that one entry is refused with
    `is on another device, so it is left alone`,
    and `delete_trash` returns the reason
    (`worktree_trash_dispose.rs:301`).
3.  The commit service's handler gets `Verdict::Failed`,
    writes one line to the service's own standard error,
    and settles the work item as failed
    (`service_work_drain_pending_claims.rs:191-197`),
    so it is never tried again.

The inline path promises the opposite in words:
`the work item ... stays, so the commit service empties it when it next runs`
(`worktree_trash_dispose.rs:170`),
which is true only for the first service try.

What it risks:
disk in a worktree that never ends,
and a directory of full copies that no command names.
No unique data:
everything in the trash is a copy of a directory the add chose to leave out,
and its original is still in the source.

What notices it later:
a line in the service's journal,
which a person at a terminal does not read,
and nothing else.
There is no sweep of `.claude/cli-git-trash`:
`rg 'TRASH_PARENT|cli-git-trash'` over `src/` finds the leave-out,
the disposal and the paths that skip it,
and no walk of the parent.

The same silence covers a trash given up on:
`requeue_expired_reporting` gives up after `MAX_TRIES`
(`service_work_handlers_replanning_and_acceptance.rs:36`),
and `tell_about_given_up` reports pushes only
(`:226-230`,
`if item.kind != WorkKind::Push { continue; }`),
so a trash item the service was killed on three times disappears without a word.

Suggestion,
not a decision:
count a failed disposal as a person-facing notice once per worktree,
as the push route does,
and let the next add sweep `.claude/cli-git-trash` for a trash that no work item and no manifest
names.

### 6. The deletion of the old tree never re-verifies which directory the root is now

Read from the code,
and the least likely to happen.

Sequence:

1.  A migration finishes and keeps the old tree.
2.  The checkout at `root` is replaced by hand,
    for example `mv root root.kept` then `mv another root`,
    while the journal and `<root>.before-subvolume` stay where they are.
3.  `--remove-old` runs.
    `remove_old_refusals` asks only that the journal records `verify` and that
    `<root>.before-subvolume` exists
    (`subvolume_preflight.rs:414-429`),
    and nothing compares the directory at `root` with the identity the migration
    installed.
4.  `carry_late_writes` copies files that are newer in the old tree into
    whatever is at `root` now
    (`subvolume_rollback.rs:276`),
    and `remove_tree` deletes the old tree
    (`:324`).

What it risks:
files written into a directory the migration never recorded,
which is the wrong-target shape the brief asks about,
and the old tree's loss,
which is finding 1's loss with a smaller cause.

What notices it later:
nothing.

Suggestion,
not a decision:
record the identity of the installed subvolume in the journal at `verify`
(the journal already records two identities,
`worktree_convert_journal.rs:140-148`),
and refuse `--remove-old` and `--rollback` when the root's identity is not that.

### 7. `--abandon` destroys a stage that a process may be working in

Read from the code.

Sequence:

1.  A conversion is interrupted before its swap,
    so `<path>.cli-git-converting` holds a full copy and the journal records its
    identity.
2.  A person or a tool opens a shell or an editor inside that copy,
    which is a directory with the worktree's own name plus a suffix.
3.  `convert-worktree-to-subvolume --abandon` runs and destroys the subvolume
    (`worktree_convert.rs:121`),
    with no scan of the process table.

The removal phase of the same conversion refuses in the same position
(`worktree_convert_phases.rs:423`,
`processes_using`),
and so do the two places that refuse a conversion
(`worktree_convert_inspection.rs:298`,
`worktree_convert_phases.rs:341`),
so the asymmetry is in this one verb.

What it risks:
the working directory of a process,
and a write made into a destroyed directory,
which the accepted-risk notice covers only where that notice is printed:
`RISK_NOTICE` is said for a converting run
(`worktree_convert_command.rs:363`)
and `MIGRATE_RISK_NOTICE` for a migration
(`subvolume_arguments.rs:64`),
and `--abandon` says neither.

Suggestion,
not a decision:
run the same `processes_using` scan in `abandon` and refuse while a process is
inside the stage,
or print the risk notice before the destroy.

## Locks in this route

Two kernel locks exist,
both of them Git's own index protocol,
and both are taken for the smallest unit there is,
one index,
never a repository.

- The conversion's `index.lock` of the worktree
  (`worktree_convert_phases.rs:227`),
  taken after the long first copy and released after the index rewrite
  (`:394`),
  because a Git command that wrote the index across the swap would lose that
  write and could install a stale stat cache.
  A rerun accepts the lock when it holds the conversion's token
  (`subvolume_lock.rs:146`),
  so a killed run does not block its own resumption.
  It cannot be removed:
  nothing else keeps a Git writer out of an index whose checkout is being
  exchanged.
- The migration's `index.lock` of the checkout and of every linked worktree
  (`subvolume_migrate.rs:232`,
  `acquire_all`),
  held from the last delta to the end of the index rewrite,
  never waited for,
  and released from the old tree (`:251-255`) and from the new one (`:298`).
  Same reason,
  same minimum unit.
- One probe takes a kernel flock and gives it straight back:
  `legacy_lock_is_held`
  (`worktree_convert_journal.rs:541`)
  probes the file release 0.5.20 held,
  with `LOCK_EX | LOCK_NB`,
  and removes the file when nobody holds it.
  It never waits and it never deletes a held lock.
  It could be removed once no installation older than the claim protocol is
  supported;
  until then it is what keeps two releases from converting one worktree at once.

Everything else is lock-free:

- the conversion's claim and the migration's claim are numbered files published
  by one exclusive `link`
  (`single_owner_claim.rs:273`,
  `publish_new`),
  and a live owner is refused rather than waited for
  (`:348`,
  `:338`);
- the conversion's journal is published by an exclusive create and replaced by
  a rename of a private file
  (`worktree_convert_journal.rs:400-448`);
- the removal route holds no lock between the proof and the destroy,
  which the module's own doc describes as the milliseconds between the last
  look and the destroy
  (`worktree_remove.rs:27-42`);
- the trash uses a work item and identity checks,
  and no lock.

## Waits and their ceilings

Every wait in this route,
with its bound:

- Every Git process the removal and the conversion start runs through the
  bounded step runner:
  a plumbing step is ended at two minutes
  (`service/wait_ceilings.rs:378`),
  an object step at ten
  (`:395`),
  and the ceiling is chosen by the subcommand's class
  (`transaction/transaction_git_step_bounds.rs:52`).
  The one `git ls-files` the removal's proof runs is plumbing
  (`worktree_clean.rs:254-272`),
  so no removal waits on Git forever.
- The trash's hand-over waits for the service with
  `START_WAIT` for the start,
  `ANSWER_CEILING` (twenty minutes) for the answer,
  `WITHDRAW_CEILING` (ten) and `START_CEILING` (a minute)
  (`worktree_trash_dispose.rs:218-230`).
- The claim is taken with at most 64 rounds and never sleeps
  (`single_owner_claim.rs:70`,
  `:338`).
- The process table's own scan and the index refresh are local file work with
  no timer;
  a blocked disk is not a wait this code can bound,
  which the owner's outside-our-control list covers.
- The migration's index lock is refused,
  not waited for
  (`subvolume_lock.rs:146-160`).
- The add's disposal thread is joined before the command ends,
  and that join is
  a step of its own
  (`worktree_copy_after_lock.rs:204`).

Waits with no ceiling:

- none that a timer could end.
  The only unbounded ones are inside the kernel:
  a `sync_all` on the journal and the lock files
  (`worktree_convert_journal.rs:410`,
  `subvolume_file_write_steps.rs`),
  a delete of a large tree,
  and a file read on a filesystem that never answers.
  The owner's "the final disk save is never cut short"
  (`doc/handover/cli-git-rust-implementation.md`,
  "The final disk save is never cut short")
  makes the first one deliberate.
- Deliberately not a wait:
  a live claim of another command is a refusal with that process's id,
  never a queue.

## Kill points, one by one

For each state change:
what a kill just before and just after it leaves,
what
notices that later,
and whether work can be lost,
hidden or applied to the
wrong place.
Every one of these was read from the code;
the ones with a control in the repository are named.
The conversion's phases are the ones `worktree_convert_tests.rs:226`,
`a_kill_at_every_phase_is_finished_by_a_rerun`,
already walks with the `Interrupt` seam,
and the migration's are walked by
`subvolume_migrate_tests.rs:161`.

The removal,
served by the destroy:

1.  Before the registration is proven:
    nothing changed;
    real Git decides.
2.  After the proof and before the destroy:
    nothing changed;
    a rerun redoes the proof.
    The guard tests move a directory,
    a registration and a parent link into this
    window and each stops the removal
    (`worktree_remove_guard_tests.rs:186`,
    `:205`,
    `:291`,
    `:328`).
3.  After the destroy and before the registration is emptied:
    the subvolume is gone,
    the registration remains,
    which is finding 4.
4.  After the administrative directory is emptied and before its name goes:
    the same shape,
    with an empty registration.
5.  After the name is removed and before `worktrees` is removed:
    a stray empty `worktrees` directory,
    which Git recreates and which no command needs removed.
6.  After the removal:
    nothing left,
    and the units are stopped
    (`worktree_removal_stops_units.rs:110`).
    A kill between the destroy and the stop leaves units listening for a
    worktree whose files are gone;
    `service stop-units-of-removed-worktrees` sweeps them
    (`:20`).

The removal,
forwarded to real Git:
the wrapper changes nothing but the snapshot,
so every kill point is Git's.

The rescue snapshot,
either way:

1.  After the snapshot directory is made and before its `attempts` file:
    a directory with no `gitdir` file,
    which every settle skips
    (`worktree_removal_rescue.rs:520-524`);
    disk only.
2.  After the `attempts` file and before the `gitdir` file:
    the same,
    and the snapshot is never decided.
3.  After the `gitdir` file:
    decidable,
    which is the state findings 2 and 4 are about.

The conversion:

1.  After the claim and before the journal:
    nothing on disk;
    a rerun takes the next generation of the claim.
2.  After the journal's exclusive create and before the stage's subvolume:
    `ensure_stage` creates it on the rerun
    (`worktree_convert_phases.rs:130`).
3.  After the subvolume is created and before its identity is recorded:
    the name holds a directory the journal cannot prove,
    and every later run refuses with the message that says so and asks for
    `--abandon`
    (`:131-145`,
    `worktree_convert_inspection.rs:276`).
    Nothing is lost (the stage is empty at that point) and a person is asked
    once.
4.  During the first copy,
    before `index.lock`:
    a rerun copies again from the start
    (`:215`,
    `copy_phase` runs on every resume before the swap).
    A Git writer may act in this window,
    which is why the proving pass follows.
5.  After the proving pass and before the exchange:
    the journal says `Synced`,
    and a rerun proves the copy again before it
    swaps,
    so a write made in this window is carried or refused.
    A kill here leaves the stage and the journal,
    and a later run finishes.
6.  Between the exchange and its journal line:
    the journal is one phase behind and the identity in it says where the old
    tree is
    (`worktree_convert.rs:190-206`,
    `old_tree_is_at`),
    so a rerun neither exchanges back nor loses the tree.
    The existing controls stop a run at exactly these points
    (`worktree_convert_safety_tests.rs:131`,
    `:374`).
7.  After the exchange,
    before the index rewrite:
    the checkout holds the copy with an index that describes the old inodes,
    and the old tree is at the converting name.
    A rerun finishes the index and then deletes the old tree.
8.  Between the index rewrite and the removal of the old tree:
    the lock is released,
    so a removal of the same worktree by another command is served from here
    (finding 3).
9.  After the old tree is deleted and before the journal goes:
    a rerun finds the name empty,
    counts it absent,
    and removes the journal
    (`worktree_convert_remove.rs:87`).

The migration:

1.  Before the journal:
    nothing.
2.  After the journal and before the new subvolume:
    `create` recorded as started;
    a rerun skips creating what exists.
3.  During any copy pass:
    every pass is idempotent and a rerun repeats the unfinished one.
4.  After `index.lock` is created and before the exchange:
    a rerun accepts its own locks
    (`subvolume_lock.rs:146`),
    and a person's Git commands are blocked until the rerun or the rollback;
    the error names the journal and both commands
    (`subvolume_command.rs:148-160`).
5.  Between the exchange and the rename:
    `exchange` recorded as started with the checkout a subvolume is finished by
    a rerun that renames and records
    (`subvolume_migrate.rs:196-205`).
6.  After the rename and before the index rewrite:
    the old tree exists at its final name and the catch-up and index steps run
    again.
7.  After `verify`:
    the migration is complete and the old tree stays until asked.

The `--remove-old` deletion itself:
a kill part way through leaves a partly deleted old tree,
and the journal still records `verify`,
so a rerun deletes the rest.
Nothing else in the old tree is read or needed,
which is why finding 1 is the only loss in that step.

The trash:

1.  After the trash directory is made and before an entry is moved into it:
    an empty trash directory inside the new worktree,
    which `prune_snapshot` removes again when nothing was moved
    (`worktree_trash.rs:403-410`).
2.  During the moves:
    the manifest lives in the command's memory until the hand-over,
    so a kill here leaves moved directories in a trash whose manifest is gone.
    They are copies of directories the source still holds,
    so nothing unique is lost,
    and no sweep can name them (finding 5's second half).
3.  After the work item is written and before the emptying:
    the service empties it later,
    which is what the record is for.
4.  During the emptying,
    with or without a work item:
    the manifest that names the entries is in the record
    (`worktree_trash_dispose.rs:200-215`,
    `parent: manifest.encode()`),
    so a rerun removes the rest,
    and the strangers check refuses a trash that gained anything.

## Races between two concurrent commands

Pairs that were read,
with the outcome claimed.

Two `git worktree remove` of the same worktree:
both may pass every proof;
only one wins the destroy,
because the second finds no subvolume with that id and falls to Git,
which reports that the path is not a working tree.
Nothing is deleted twice and nothing is left empty that held data;
the loser's exit status is Git's refusal while the worktree is in fact gone.
Suggestion,
not a decision:
after a failed destroy,
look whether the registration and the directory both
went,
and report that as success with the destroy notice.

A removal of a worktree a conversion is working on:
nothing serializes them.
The conversion consults the removal only through `index.lock`,
and only after its first copy
(`worktree_convert_phases.rs:227`).
The removal consults no claim at all.
So the removal can destroy a worktree between the start of the copy and the
proving pass,
or after the index lock is released and before the old tree goes
(finding 3).
The conversion then fails with `changed while it was copied`
(`:236-240`)
or `neither ... is the directory this conversion recorded`
(`worktree_convert_phases.rs:322-326`)
and keeps its stage,
so both commands report and no person's file is lost.
Suggestion,
not a decision:
have the removal route read the conversion claim through
`live_owner_of`
(`single_owner_claim.rs:413`)
in the same step as the lock check,
which is one directory listing.

A conversion of one worktree and a conversion of another:
independent in every respect except the filesystem,
each with its own administrative directory,
index,
claim and journal
(`worktree_convert_parallel_tests.rs`).
Two conversions of the same worktree:
the second is refused with the first's process id in under a millisecond,
measured (the claim block under "Reproductions").

A conversion and a migration of the main checkout that holds it:
each refuses while the other's claim is live
(`worktree_convert_inspection.rs:221`,
`subvolume_preflight.rs:319`).
The two checks read the other's directory before taking their own claim,
so a window a few file reads wide exists in which both proceed;
the conversion then finds the identity it recorded at neither name and stops
without swapping
(`worktree_convert_phases.rs:322-326`),
so the cost is an orphan stage,
not data.
Suggestion,
not a decision:
re-read the other claim after taking the claim,
which is the retry
`take_claim` already performs for its own generations.

Two commands taking removal snapshots and settling them:
the snapshot is named by the worktree's instance and written under it by
`save_once` and `register_attempt`
(`worktree_removal_rescue.rs:218`,
`:343`),
so two commands add to one snapshot rather than replace it;
the settle promotes with a rename,
which another command's promotion makes
`NotFound` and which is tolerated
(`:410-414`),
and removing a settled snapshot twice is tolerated
(`:528`,
`:532`).
The one write that must not be raced is protected by order:
the `attempts` file is written first and the `gitdir` file last,
so a snapshot the settle can decide is one whose taker is recorded.
Measured as sound by the snapshot block's two decided arms.

A `git worktree add` and a `git worktree remove` at the same path:
the removal needs the path to be a directory and to be registered,
and the add registers the path before its rung fills it,
so the removal is either refused by `registration_of` or refused by the proof
of a worktree that holds no tracked file yet.
Both end in real Git's own answer.

A `git worktree prune` and a conversion:
prune removes only registrations whose directory is gone,
and a conversion keeps the directory at its path at every instant
(`worktree_convert_phases.rs:311-336`),
so prune's own rule cannot touch a worktree being converted.
The snapshots prune takes are the ones finding 2 is about when the pruned
worktree's records cannot be promoted.

## Reachability measured on this host

Host:
`/var/mnt/encrypted` on `/dev/mapper/crypt_sda`,
btrfs,
`subvolid=5`,
`compress=zstd:3`,
with `user_subvol_rm_allowed` in the mount options,
which is the option the served destroy needs
(`worktree_remove.rs:70`).

Census of the repository's own registrations,
2026-11-08T22:27:39-05:00,
by reading every `<common>/worktrees/*/gitdir` and `stat`ting each named
directory,
with a subvolume root recognized as inode 256
(`fs_kind.rs:157-166`):

- 356 registered worktrees,
  none locked,
  none whose directory is missing;
- 315 are subvolume roots,
  so a plain `git worktree remove` of one of them,
  when its tracked files are proven clean,
  is served by the one-call destroy rather than by Git's file-by-file removal;
- 41 are ordinary directories,
  so they are conversion targets;
- 4 of those 41 lie inside the main checkout,
  the only place `convert-worktree-to-subvolume --all` looks
  (`worktree_convert_command.rs:78-100`);
  the other 37 have to be named on the command line;
- no registration had a `<path>.cli-git-converting` beside it,
  so finding 3's leftover is not present today.

So the destroy is the ordinary path for a removal on this host,
not a rare one,
and finding 4's window sits in it.

## Reproductions

One throwaway file,
`package/git-policy/cli/src/native/subvolume/audit_scratch_tests.rs`,
registered in that module only for these runs and not committed.
It runs the real command on the stand-in platform of the migration controls
(`subvolume_test_support.rs:55`,
`PlainPlatform`),
which is a plain directory tree whose "subvolume" is a registered device and
inode,
so the migration's own steps run without btrfs,
and the real `settle_removals` and `snapshot_linked_worktrees` on a real Git
fixture.

Command:
`mise run //package/git-policy/cli:native:test:host -- audit_scratch_tests`
in a `systemd-run --user --scope` with `CPUQuota=300%`,
`MemoryMax=16G`,
`TasksMax=256`,
`IOWeight=10`.

### The old tree's left-out file, with a newer file as the control

Both files are written into the old tree after the migration;
`age` gives one a modification time two minutes in the past,
which every
mtime-preserving restore does.

```text
restored.txt mtime 1794194253, late.txt mtime 1794194373
remove-old exit 0
  late writes (files written to the old tree after the catch-up): 102 entries, 1 files copied (0 cloned), 0 hard-linked, 0 symlinks, 60 unchanged, 0 deleted, 0 vanished, 0.01 s
  late writes (files written to the old tree after the catch-up): 1 entr(ies) absent from the destination were left out because they predate the catch-up
  late writes (files written to the old tree after the catch-up): carried late.txt
  1 file(s) written to the old tree after the catch-up were carried into /.../checkout
  deleting /.../checkout.before-subvolume (every file is unlinked; this can take minutes)
  the old tree is deleted and the migration journal is removed.
the old tree is gone: true
restored.txt reached the checkout: false
late.txt reached the checkout: true
any line names the left-out file: false
```

The positive control is the file written now,
which the same pass carried and named.
The file with the older time was counted and then deleted,
and no line names it.

### The old tree deleted under a process whose working directory is inside it

The process starts in the checkout before the migration,
which is the ordinary way a person's shell ends up in the old tree.

```text
migrate exit 0
  WARNING: 1 process(es) have a working directory inside the checkout and will stay in the OLD tree after the exchange; restart them (or `cd "$PWD"` in a shell) afterwards:
    pid 2718305 (sleep)
processes whose working directory or a file is inside the old tree: [ProcessUser { pid: 2718305, command: Some("sleep") }]
the scan found the process that was in the checkout: true
remove-old exit 0
  monochromatic-git-policy-cli: this command replaces or deletes files of the checkout while other programs may be changing them at the same moment. A change made between a check and the step that relies on it is lost, and that risk is yours; stop the programs working inside the checkout first.
  late writes (files written to the old tree after the catch-up): 100 entries, 0 files copied (0 cloned), 0 hard-linked, 0 symlinks, 60 unchanged, 0 deleted, 0 vanished, 0.00 s
  deleting /.../checkout.before-subvolume (every file is unlinked; this can take minutes)
  the old tree is deleted and the migration journal is removed.
the holder is still running: true
its working directory now reads: Ok("/.../checkout.before-subvolume (deleted)")
the old tree is gone: true
```

The scan that would have caught it is the one the conversion route runs
(`worktree_convert_phases.rs:423`);
here it was run only to show that it sees the process.
The removal went ahead and exited 0.
What this measures is the asymmetry,
not an unstated risk:
the risk notice above is printed,
and the owner's accepted-risk reading covers the race it describes.

### A rescue snapshot in each of the three states

```text
the registration is still there: true
snapshot problems: []
round 1: snapshots 1, notices any worktree would show 0, problems []
round 2: snapshots 1, notices any worktree would show 0, problems []
round 3: snapshots 1, notices any worktree would show 0, problems []
```

That is finding 2:
three settles in a row,
no notice promoted,
no problem reported,
and the snapshot still there.
The two decided states settle as designed,
which is the control that the test can tell them apart:

```text
snapshot problems: []
done: snapshots 0, notices promoted 1
the worktree is intact: true
snapshot problems: []
refused: snapshots 0, notices promoted 0
```

The registration gone promotes the notice;
the worktree intact discards the snapshot.

### The claim that never waits, and Git's answer to a stale registration

```text
the first claim is held: true
the second claim was refused in 852.347µs: true
after the release the next claim is held: true
```

The second command is refused rather than queued,
which is what "nothing should ever wait" asks of this lock.

Native Git's own order,
measured with
`strace --quiet --follow-forks --trace=unlinkat,rmdir,unlink --decode-fds=path git worktree remove --force ../wt`
on a fixture under `${HOME}/temp/agent/pi-removal-audit/order`:

```text
unlink("/.../order/wt/.git") = 0
rmdir("/.../order/wt") = 0
unlink(".git/worktrees/wt/gitdir")      = 0
unlink(".git/worktrees/wt/commondir")   = 0
rmdir(".git/worktrees/wt/refs")         = 0
unlink(".git/worktrees/wt/logs/HEAD")   = 0
rmdir(".git/worktrees/wt/logs")         = 0
unlink(".git/worktrees/wt/HEAD")        = 0
unlink(".git/worktrees/wt/index")       = 0
rmdir(".git/worktrees/wt")              = 0
rmdir(".git/worktrees")                 = 0
```

The directory goes before the registration,
which is the order the wrapper's served path also takes.

Git's answer to the state finding 4 leaves was measured outside the test file,
in a throwaway repository under `${HOME}/temp/agent/pi-removal-audit/fix`:

```text
--- add at the same path while the registration is there:
fatal: '../wt' is a missing but already registered worktree;
use 'add -f' to override, or 'prune' or 'remove' to clear
--- list:
worktree /.../fix/wt
prunable gitdir file points to non-existent location
--- prune dry run:
Removing worktrees/wt: gitdir file points to non-existent location
```

## What I checked and found sound

So that nobody repeats it:

- The destroy's target:
  the registration must belong to the repository the command runs in and be the
  one Git's last-component match would pick
  (`worktree_remove_registry.rs:162`),
  a lock entry that cannot be looked up is not an absent lock
  (`worktree_remove.rs:88`),
  and the last look before the ioctl compares the directory,
  its parent,
  the registration and the exact `gitdir` bytes
  (`:336-372`).
  The guard tests move each of those into the window and each stops the destroy.
- The proof of cleanliness is conservative in the direction that matters:
  it refuses split and sparse indexes,
  racy entries,
  submodules,
  assume-unchanged entries
  (`worktree_clean.rs:404-460`)
  and any untracked file Git does not ignore
  (`:514-530`),
  and every refusal falls to Git,
  which words its own answer.
- The recursive deletions never follow a link and never cross a mount or a
  nested subvolume:
  every directory is opened below its parent's descriptor with
  `RESOLVE_NO_SYMLINKS` and `RESOLVE_NO_XDEV`
  (`fs_beneath.rs:158-190`),
  the device baseline is compared at each level
  (`subvolume_remove_walk.rs:147-149`),
  and a boundary refuses the whole walk before anything is deleted.
  The trash's walk re-checks the identity of the top directory against the
  recorded one on the opened object (`worktree_trash_remove.rs:190-202`).
- Both `index.lock` holders delete only their own lock
  (`subvolume_lock.rs:128`,
  `worktree_convert_phases.rs:187`),
  which is the decision "cli-git never deletes a lock" kept.
- The conversion's copy is proven equal before the swap and proven again before
  the old tree is deleted,
  and the comparison refuses on a size or modification-time difference rather
  than deleting on a guess
  (`worktree_convert_compare.rs:40-70`).
- The conversion's two exclusions from its own refusals are by identity,
  never
  by name:
  its own `index.lock` by token and its own stage by the identity the journal
  recorded
  (`worktree_convert_inspection.rs:265-275`).
- The trash's disposal removes only the trash the manifest names,
  each entry only when it is the recorded object,
  and refuses a trash that holds a stranger
  (`worktree_trash_dispose.rs:301-352`).
- The migration never deletes the original tree by itself,
  and `--rollback` carries everything the restored tree lacks before it deletes
  the new subvolume,
  with an explicit refusal for files this user cannot read
  (`subvolume_rollback.rs:156-200`,
  `:299-320`).
- The ladder's swap never leaves the destination missing or partial:
  one `RENAME_EXCHANGE`,
  the identity of the directory Git made is checked
  before and after,
  a wrong one is swapped back,
  and anything that appeared at
  the destination meanwhile is moved back into the worktree rather than deleted
  (`worktree_snapshot.rs:394-443`,
  `:445-490`).
  The ladder's only deletions are of objects it made and can still prove
  (`worktree_ladder_run.rs:296-320`).
- A conversion's journal write and a claim's publication are the same shape:
  a private file,
  an fsync,
  and one exclusive link or rename,
  with the leftovers swept by the next write
  (`worktree_convert_journal.rs:332`,
  `:400-448`,
  `single_owner_claim.rs:273`).
- Waits:
  no sleep,
  poll or retry loop exists in any module of this route except
  the bounded one inside the trash hand-over,
  and every Git process is bounded by the step runner.

## What I could not determine

- Whether the removal route and a conversion ever meet in the window of
  finding 3 in practice.
  The window was read,
  not raced:
  a live two-command race would need a seam inside `index_phase`,
  and the conversion offers none between the lock release and the removal
  phase.
- How often a person restores a file into a finished migration's old tree.
  Finding 1 is deterministic when it happens and its frequency was not
  measured,
  so its rank rests on the loss,
  not on how common it is.
- Whether any host actually keeps a `.cli-git-converting` directory today.
  None of the 356 registrations had one,
  and a leftover beside an unregistered
  path is outside that census.
- Whether the trash's device baseline refuses a real left-out directory on this
  repository's host.
  No trash existed to inspect;
  the refusal is read from `worktree_trash_remove.rs:152-158` and was not
  provoked on btrfs.
- What a kill during the rollback's own exchange leaves beyond what
  `subvolume_rollback_tests.rs:223-250` already walks;
  the tests stop the rollback after its exchange and after the rename,
  and both
  resume,
  so no new state was found,
  but the swap back mid-delete was not
  exercised.
- The installed release's behavior:
  every measurement ran the test binary built from this revision,
  not the installed wrapper,
  and the owner's pause covers Windows and macOS.

## What to inspect and how to respond

Inspect,
in this order:

1.  finding 1 against `subvolume_rollback.rs:273-326` and
    `subvolume_sync.rs:145-152`,
    with the first block's output above;
2.  finding 2 against `worktree_removal_rescue.rs:282-320` and `:515-534`,
    with the rescue block's three rounds;
3.  findings 3,
    4,
    5 and 6,
    each of which is a sequence and a `path:line`
    rather than a measurement;
4.  the kill-point walk and the races section,
    for the states they call sound.

Respond with a decision on which findings to fix,
in which order,
and whether the accepted-risk reading covers a deletion under a live process
(finding 7 and the block on the deletion under a live process) or whether the conversion's refusal is the
shape the migration should take too.

No production code was changed,
so nothing else is pending from this audit.
