# cli-git ZFS lost events: the readiness hook, a nudge on Git commands, and what the cost cap may hold

## Status

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T19:20:28-05:00.
It answers the three questions of the design
`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`
(on its own branch,
still unapproved;
section "Questions only the project can answer",
Q-A,
Q-B and Q-C),
as its gate review would put them
(`~/temp/agent/gate-review-zfs-lost-last-event-triggers-design-2026-11-22/keep/review-diff-check.md`,
section "The questions as they now stand,
and whether I would put the design"),
about the open item "Which triggers a lost last event waits for" of
`doc/planning/cli-git-zfs-worktree-design-proposal.md`.

## What was decided

### Which trigger runs after the checkout becomes ready

Asked:
the accepted design obliges the checkout's provider,
on this machine `zfs-mount.service`,
to trigger reconciliation once the checkout is ready.
The project chose "B:
ask the boot unit (Recommended)":
a drop-in on the provider whose `ExecStartPost=` is prefixed with a dash and runs
`systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service`,
with the boot unit gaining an `ExecReload=` that runs the handler's `boot-reconcile`.
The same command is the lever a person runs.

The costs the question stated with this option:
a `systemctl` call from a unit's start is untested until a guest runs it;
it fires only when the provider runs as a job,
so a hand-run `zfs mount` fires nothing;
and it depends on the boot unit having the `ExecReload=` line,
without which the command restarts the unit and unmounts every clone.

Applying the drop-in and the unit change to a machine is its owner's act,
never an agent's.

### Whether a lost request with nobody waiting may wait for the next trigger

Asked:
the project had called periodic reconciliation with nobody waiting "moot",
and the design asked whether that accepts an unbounded wait.
The project chose "B:
nudge on Git commands":
whenever a wrapped Git command runs in a repository that holds a clone with an unanswered request,
it writes that clone's nudge.
This is no timer.
The recommended option was to accept the wait with no bound;
the project did not take it.

The cost the question stated with this option:
every Git command pays a device open and a property read,
unmeasured,
and every Git call becomes coupled to the ZFS route.
What follows from that cost is the coordinator's requirement on the design,
not a part of the option the project chose:
the design owes a way for a command to learn cheaply that a repository has no unanswered request,
and a measurement of what a command pays in each case,
before this ships.

A timer was not chosen:
"Periodic reconciliation without a waiter is not part of this route"
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
lines 537 to 539)
stands.

### Whether the cost rule may hold a pass while a caller waits

Asked:
the route bounds how much root work events and nudges may cause.
The project chose "C:
hold only idle events (Recommended)":
the rule holds only passes whose event names a clone with no unanswered request,
and an event for a clone that has one always runs.

The cost the question stated with this option:
one clone read before the decision,
unmeasured,
so the rule is adopted once that is measured;
and a loop on a clone that has a pending request is not bounded,
which the project's threat model accepts.

## What was not chosen

- Running the handler directly from the provider,
  a mount unit of its own for the checkout,
  and no hook,
  for the trigger.
- No time bound,
  and a timer,
  for a lost request with nobody waiting.
- Holding every pass over the budget,
  and no bound,
  for the cost rule.

## What this record does not decide

The size of the budget and its window,
which wait for the measurement that sizes them.
