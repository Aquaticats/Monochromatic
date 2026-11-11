# The ignored-state copy an interrupted add owes is the worktree's own work

Written 2026-11-11T01:55:00-05:00 on
`fix/git-policy-cli-killed-add-leftover-copy-is-not-another-commands-wait`,
from `84b737d1cc` on `integrate/git-policy-cli-jam-fixes-after-alpha-9`.

## What was found

An audit of the integration tip (`/var/home/user/temp/agent/sonnet-audits/integration-tip-never-stuck`,
finding 2) measured an ordinary command in one linked worktree paying for another worktree's killed
`git worktree add`:

- after the add's process was `SIGKILL`ed while it copied ignored state,
  `git merge` (already up to date) in a second linked worktree took 63.5 s,
  `git worktree add` there took 64.0 s,
  and `git push` there took 119.8 s
  (release build, 15,151 ignored entries, about 150 MB,
  `drive_s11_scale.sh` and `s11_killed_add_then_in_scope_command.sh` beside that report);
- nothing was printed until the copy ended,
  then one line: `finished the ignored-state copy an interrupted git worktree add owed ...`;
- a commit through the commit service in a linked worktree did not pay it.

The owner's rules that this breaks are
"Agents should never wait on each other",
"nothing should ever wait forever or indefinitely",
and "commits vs native git should not take 5s longer",
because the same pre-command check stands in front of `commit`, `merge`, `push` and `add`.

## Why it happened

`recover_pending_worktree_copies`
(`package/git-policy/cli/src/native/worktree_copying/worktree_copy_recovery.rs`,
the function the pending check calls before every forwarded command in a linked worktree or a bare
repository) finished the copy **in the command that found it**:
it passed every dead owner's claim to
`finish_owed_worktree_copies`
(`worktree_copy_owed_copy.rs`), which staged the source's whole ignored state and installed it before
the command's own Git started.
A journaled copy was resumed the same way by `recover_transaction`.
The same file's own doc comment said "It never waits", and that was true of locks and of live owners,
but not of this work: the copy is the size of the source's ignored tree, and the command that starts
the pass does all of it.

## What changed

1.  A copy is finished only by a command run in the worktree it fills.
    `RecoveryDestinations`
    (`worktree_copy_recovery.rs`) carries the worktree the command runs in,
    taken from the invocation's own identity (`worktree_copy.rs`), and both the owed copy and a
    journal's installation refuse a destination that is another worktree's.
    A command in another worktree writes one line naming the worktree whose files are missing,
    that the first command run in that worktree makes the copy,
    and that this command does not wait for it.
    The discards and the keeps (an unregistered destination, a missing stage, a pointer that proves
    nothing, a stage a dead owner left) stay open to every command: they read a few records and
    remove the files of a transaction nothing may resume, and a worktree that is gone has no command
    left to run in it.
2.  The command that does make the copy says so before the first entry is read,
    naming the worktree, the source, and that it stops at a ceiling.
3.  That copy is bounded.
    `CopyCeiling` (`worktree_copy_ceiling.rs`, `OWED_COPY_CEILING`, six minutes) is asked by the walk
    of the source, by every entry the copy writes, by every installation batch and between the
    phases.
    At the ceiling the copy ends as one that could not be finished:
    the installation rolls back what it wrote,
    the worktree is named by the record of an unfilled worktree, with the sentence that names the
    ceiling, and `git worktree remove --force` plus the same `git worktree add` line copies the same
    state with no ceiling, which is the remedy every unfilled worktree already names.
    A creating command's own copy keeps no ceiling: the owner allows an add its own time.

## Measurements

The audit's numbers stand for the state before this branch: with the release build of the integration tip and
15,151 ignored entries (about 150 MB), after a `git worktree add` was `SIGKILL`ed during its ignored-state copy,
a `git merge` (already up to date) in a second linked worktree took 63.5 s,
a `git worktree add` there 64.0 s,
and a `git push` there 119.8 s,
summarised by one line printed after the work was done
(`reproductions/s11_killed_add_then_other_command.sh` and `drive_s11_scale.sh` in
`/var/home/user/temp/agent/sonnet-audits/integration-tip-never-stuck`).
A commit in a linked worktree, which runs through the commit service, paid none of it.

The measurement was not repeated at that scale in this session:
the shared build tool (`cargo-shadow`) spent an hour of it in an uninterruptible wait behind other sessions' builds,
and the volume the build trees live on read 99 to 100 per cent full.
What stands in for it is the control that decides the behaviour the audit timed:
`a_command_in_another_worktree_leaves_the_copy_to_the_worktree_it_fills`
(`src/native/worktree_copying/worktree_copy_owed_copy_tests.rs`)
holds a killed add's state, runs a command in another linked worktree,
and requires that it writes nothing into the unfilled worktree,
starts no copy,
and prints one line naming the worktree whose files are missing.
It fails on the code the audit measured and passes here.

## What the kill points leave, and what finishes each

The 14 `a_worktree_copy_killed_at_each_*` controls in `src/native/binary/binary_fault_trace_tests.rs`
kill the wrapper at every counted call of one kind and then require the worktree to hold its source's ignored state,
with no record, claim or stage left.
Three of them failed on the integration tip (`chmod`, `openat2`, `write`) and a different set on each run,
because a kill lands at the Nth call of its kind and timing decides which step that is.
What each leftover now settles to:

- a stage container the copy created and never filled:
  the copy clears it, because it records the registration it fills as its first step,
  and a container that holds nothing at all is the step before that record and loses nothing when it is removed;
- a record that was rewritten with the add's binding:
  the record keeps the source and the selection of the copy the add owes,
  so a command run in that worktree still has everything it needs to make the copy;
  before this, the rewrite dropped both lines and no later command could ever make that copy;
- a checkout the add never finished, with no index or with the lock of the index publish still standing:
  a command in that worktree finishes it with Git's own `reset --hard`,
  and only when every entry standing there holds the very bytes the source holds at that name,
  so no file of the person's is replaced;
  a lock goes only when it is the published index's own object,
  or when the add that made it is proven dead and no process holds it open;
- a journal a kill left beside the add's claim:
  the journal's own recovery installs the copy and settles that claim,
  which nothing used to remove.

## Left to the owner

- The alternative to point 1 is to hand the copy to the commit service as a durable work item, the way
  a trash, a prewarm, a relink and a subvolume delete are already handed over. Nothing would wait
  anywhere, a machine with no reachable service would leave the worktree named instead of filling it,
  and the work would be one more work kind with its own handler. It was not taken here because the
  worktree's own first command is the command that needs the files, and because a new work kind is a
  change to the service protocol rather than to one recovery pass.
- `OWED_COPY_CEILING` is derived from the audit's measurement, not from one of this branch: the owner may
  prefer a different margin, or no ceiling at all for a worktree's own work.
- The stage the snapshot rung creates is recorded with its object as soon as it exists, so a kill between
  that creation and the record leaves a folder nothing proves the wrapper made. The clone rung, which is
  the rung these controls take, leaves no such folder. A repository that is a btrfs subvolume takes the
  snapshot rung, and that window is still open there: its stage can hold a copy of the person's
  uncommitted work, so nothing in this branch removes it.
