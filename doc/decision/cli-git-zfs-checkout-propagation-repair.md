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
the repair sets the mount private only when it is shared;
a mount that is slave,
unbindable or a combination is left as it is,
the route goes on,
and the state is reported.

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

## What this record does not decide

Nothing of the design's questions is left open by it.
The guest measurement that the provisional trigger waits for is owed by plan step 19.
