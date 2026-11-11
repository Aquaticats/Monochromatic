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

REPLACED BELOW when measured.

## Left to the owner

- The alternative to point 1 is to hand the copy to the commit service as a durable work item, the way
  a trash, a prewarm, a relink and a subvolume delete are already handed over. Nothing would wait
  anywhere, a machine with no reachable service would leave the worktree named instead of filling it,
  and the work would be one more work kind with its own handler. It was not taken here because the
  worktree's own first command is the command that needs the files, and because a new work kind is a
  change to the service protocol rather than to one recovery pass.
- `OWED_COPY_CEILING` is derived from the measurements below; the owner may prefer a different
  margin or no ceiling at all for a worktree's own work.
