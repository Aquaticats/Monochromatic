# cli-git ZFS checkout propagation: who repairs it, by which trigger, in which states, and how far

## Status

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T19:23:30-05:00.
It answers the four questions of the design
`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`
(on its own branch,
still unapproved;
section "Questions only the project can answer"),
as its gate review would put them
(`~/temp/agent/gate-review-zfs-private-propagation-after-replacement-mount-design-2026-11-22/keep/review-diff-check.md`),
about the open item "Re-applying private propagation after a replacement mount" of
`doc/planning/cli-git-zfs-worktree-design-proposal.md`.

The project had already decided that the boot unit sets the checkout's mount private.
A mount that replaces the checkout's mount later comes up shared again,
and clones mounted beneath a shared checkout are copied into every other mount namespace.

## What was decided

### Who may set the propagation when no person is present

The project chose "A:
every root run (Recommended)":
whenever the root handler runs,
for any event,
it first checks the checkout and sets it private when it finds it shared,
and records that it did.

The costs the question stated with this option:
root changes a mount in response to an event nobody watched,
with the journal and the record as the only trace;
and it repairs nothing while the ZFS event daemon is not running.

### Which triggers the route adds

Asked as a set to choose from,
since the triggers combine,
with the root command a person runs present whatever is chosen.
The project chose one:
"Unit on the checkout's mount (Recommended)",
a drop-in on the checkout's mount unit and one one-shot unit,
which asks the boot unit to reconcile when the mount comes back,
with the same request the readiness hook uses.
It is taken provisionally:
it is dropped if the guest measurement shows that it misses a replacement typed by hand or that it loops.

The project did not choose the wider event filter or a watcher on the kernel's mount events.
So the event filter's scope does not gain the checkout's dataset for this purpose.

Applying the unit and the drop-in to a machine is its owner's act,
never an agent's.

### What the repair does with a state that is neither private nor shared

The project chose "A:
act on shared,
report the rest (Recommended)":
in the option's own words,
"Set the mount private only when it is shared;
for any other state,
leave it,
go on,
and report what was found."

Correction of 2026-11-22T20:21:19-05:00:
this section first said that "a combination is left as it is",
which the option did not say and the project was not asked.
The design's review (`review-after-the-owners-answers.md` beside the review this record cites,
finding R5) named the difference.

### A mount that is shared and slave at once

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T20:37:32-05:00.
Asked:
a mount can pass mounts on to its peers and also receive them from a master,
as on a machine that itself runs inside a container;
does such a checkout count as shared here.
The project chose "A:
treat it as shared (Recommended)":
the repair sets a checkout that is shared and slave private,
like any shared one.

The cost the question stated with this option:
the checkout also stops receiving mounts its master makes beneath it.

Not chosen:
making such a mount a plain slave,
which would keep the master's mounts arriving and was offered as not yet designed or measured;
and leaving it as it is and reporting.

### How far the setting reaches

The project chose "A:
the checkout's mount alone (Recommended)":
the repair changes the checkout's own mount and no mount beneath it,
and reports a shared mount it finds beneath.

## What was not chosen

- Only the boot unit as setter.
- The wider event filter,
  the mount-event watcher,
  and no trigger beyond the root command.
- Refusing every state that is not private.
- Setting the propagation recursively.

## The design is accepted

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T22:15:20-05:00.
The design is `doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`
as it stood at commit `467d05e307` on its own branch,
for plan step 19 of `doc/planning/cli-git-zfs-worktree-implementation-plan.md`.

The project chose "A:
accept as written (Recommended)":
"All eight choices stand."
The eight choices,
as the project was shown them,
each with the alternative not taken,
under the design's own numbers V1 to V8 in the same order:

1.  The step's result goes in a small root-owned file of its own under `/run`.
    Not taken:
    put it in the existing readiness record.
2.  That file uses its own outcome words:
    applied,
    already-private,
    left,
    refused.
    Not taken:
    reuse the readiness record's state words.
3.  A refusal files no receipt,
    as step 8 already decided.
    Not taken:
    a "refused" receipt naming the step.
4.  The setting is changed only through the handle that was verified;
    no fallback to changing it by path.
    Not taken:
    fall back.
5.  The step runs once the configuration and checkout are resolved,
    before the start probe's record.
    Not taken:
    after the start probe,
    before the sweep.
6.  The record holds both of the kernel's numbers for the mount:
    the unique one decides identity,
    the reused one finds its row in the mount table.
    Not taken:
    the unique one alone.
7.  At most four shared mounts beneath the checkout are named in the record;
    the count stays exact.
    Four is the writer's own pick.
    Not taken:
    name every one.
8.  The rule that turns the guest's readings into "the systemd trigger stays" or "is dropped"
    is spelled out as two rules,
    numbers left to the measurement.
    Not taken:
    take the record's words literally.

Also stated with the question,
and not a choice:
the trigger costs one systemd unit and one drop-in to install and to report on.

What the project was told with the question:
the design's last three reviews found nothing high;
the coordinator,
and no reviewer,
checked the last review's fixes against the diff;
nothing in the design has been run,
its guest arms being specifications;
whether the systemd trigger stays is decided by a guest measurement still owed;
and it cites the lost-event triggers design,
which was not yet ready for acceptance.

Not chosen:
accepting with a veto,
and not accepting yet.

## What this record does not decide

Nothing of the design's questions is left open by it.
The guest measurement that the provisional trigger waits for is owed by plan step 19.
