# cli-git ZFS forced removal: the authority, a seen holder, the reach, and who is spared

## Status

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T19:00:56-05:00.
It answers four questions that the gate review of the two step 17 designs put to the project
(`~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review-second-pass.md`,
section "The single set of questions for the owner",
questions S1,
S2,
S3 and S5),
about the open items "Forced removal:
holders ended,
caller's session spared" and "Cancelled or interrupted removals and the cancel command"
of `doc/planning/cli-git-zfs-worktree-design-proposal.md`.
The designs it binds are still unapproved:
`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`
and `doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
each on its own branch.

## What was decided

### By what channel a holder of another account is ended

Asked:
when `git worktree remove --force` has to end a process of another account or a unit's task,
which privileged way may the wrapper use.
The project chose "B:
root handler (Recommended)":
the wrapper writes a request that the installed root handler reads,
and root ends the holders,
through the property and receipt protocol the route already installs.
No second privileged channel is added.

The cost the question stated with this option:
a property the account can write makes root end processes,
so the handler's own checks of who asked,
which holders,
and who is spared carry the weight.

### What a removal without `--force` does when its scan sees a holder

Asked:
the project's earlier answer says "the scan that refuses the holders it sees stays",
and two designs read that sentence in opposite ways.
The project chose "A:
cancel,
delete nothing (Recommended)":
the removal stops before any snapshot is deleted,
attaches the clone at its slot again,
and reports the holder.
A removal that would have gone through had the holder left a moment later is cancelled and is run again by the person.

This fixes the reading of the earlier sentence:
a holder the scan sees stops the destruction.

### How much a forced removal ends in another mount namespace

Asked as a direction and not as a final acceptance,
because the narrower options need root to enter another mount namespace and unmount there,
which the accepted design does not authorise
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
line 732)
and which no guest has measured.
The project chose "B:
unmount,
end blockers only":
the handler unmounts the clone inside the holder's namespace and ends only the tasks that block that unmount,
and when the namespace cannot be entered or the unmount is refused,
the forced removal refuses and names the holder.
It never falls back to ending every task of the namespace.

The recommended option was the same unmount with that fallback;
the project did not take the fallback.
A forced removal can therefore still refuse.

Still owed before this is final:
the design states how the mount to unmount is named inside the holder's namespace,
what is checked before and after the call,
what happens to a peer group and to a child mount,
and a guest measures it;
then the project confirms.

### Which processes a forced removal never ends

Asked:
the project's decision spares "the caller and the processes of its session",
and the review measured on this machine that a command run by a harness's shell tool has its shell in one session
and the harness in another,
and that a command started as a `systemd-run` unit has the service manager as its parent.
The project chose "D:
ancestors,
refuse as unit (Recommended)":
a forced removal never ends the caller's session or any ancestor of the command up to the first process,
and refuses,
naming it,
when one of them holds the clone;
and it refuses `--force` outright when the command's parent is the service manager,
telling the person to run the removal as a child of the process that wants it.

The cost the question stated with this option:
a forced removal launched as a service unit is refused.

## What was not chosen

- A root helper started through `sudo -n`,
  and the account acting alone,
  for the channel.
- Refusing and keeping the removal pending,
  and going on to delete the clone's snapshots,
  for a holder a scan sees without `--force`.
- Ending every task of each holder namespace,
  alone or as a fallback,
  for the reach.
- The caller's session alone,
  and the session with its ancestors but no refusal for a service unit,
  for who is spared.

## What this record does not decide

The signal a forced ending sends and any grace before it,
how long a forced request's authority lasts,
whether a cancelled removal's reattach covers a fresh attach after history was destroyed,
how long the record of a cancelled removal's loss lives,
whether a forced removal may be cancelled by the command,
and whether a forced ending leaves a durable record beyond the journal
are the same review's questions S4 and S6 to S11 and are still open.
