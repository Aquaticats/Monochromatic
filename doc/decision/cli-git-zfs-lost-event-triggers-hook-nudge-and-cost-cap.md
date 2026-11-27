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
(`~/temp/agent/gate-review-zfs-lost-last-event-triggers-design-2026-11-22/keep/review-diff-check.md`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here),
section "The questions as they now stand,
and whether I would put the design"),
about the open item "Which triggers a lost last event waits for" of
`doc/planning/cli-git-zfs-worktree-design-proposal.md`.
That review put all three as the project's questions,
each with concrete options and a ranking it called sound:
Q-A ranked B > A > C > D against its trace of the hook action,
Q-B ranked A > B > C,
and Q-C ranked C > A > B,
adopted once measured.

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

## A further answer: how long a lost request keeps being nudged

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T20:49:43-05:00,
after the design was rewritten to the three answers of this record and reviewed twice more
(`~/temp/agent/gate-review-zfs-lost-last-event-triggers-design-2026-11-22/keep/review-diff-check-after-the-answers.md`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here),
section "What is left for the owner",
on the design's veto entry V14).
That review found V14 complete as a veto entry and asked for it as a question,
because its first option adds a consequence the owner was not shown:
a request nothing can answer is nudged for ever,
one handler run and one pass per `Gmax`,
by ordinary Git use,
with `Gmax` unmeasured.

Asked:
the design spaces a clone's nudges,
each wait twice the last up to a longest gap that a measurement sizes.
A request that nothing can ever answer,
an attach that keeps being refused for example,
is then nudged by ordinary Git use for as long as it stands,
once per longest gap,
and each nudge makes the root handler run one pass.
The project had not been shown that when it chose the nudge on Git commands.

The project chose "A:
forever,
at the capped gap (Recommended)":
a request is nudged as long as it stands,
never more often than the longest gap.

The cost the question stated with this option:
a request nothing can answer costs one root handler pass per longest gap,
for ever,
while Git is used in that repository.
The question's "once a minute" was an example and decides no number.

Not chosen:
a gap that doubles without a cap,
and stopping after a set number of tries.

## What this record does not decide

The size of the budget and its window,
the first gap and the longest gap,
which wait for the measurements that size them.

## The design is accepted

Accepted in the question tool on 2026-11-23,
recorded 2026-11-23T04:23:52-05:00,
at the design's commit `c8c22c3285`
(`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`).

The project chose "A:
accept as written (Recommended)":
"All seventeen choices stand.
Pros:
the design can be built in plan steps 19 and 22 as written.
Cons:
choice 5's file growth where helpers are killed stays uncapped until measured."

Not chosen:
"B:
accept,
with a veto";
and "C:
not yet".

The question said what accepting means:
"the document merges into main as an accepted design and the seventeen choices in the preview stand."
It named one cost first:
"where a background helper is killed before it records its result,
one small file is added per retry gap for as long as that request stays unanswered,
and no cap is proposed until a measurement can size one."
It also said that nothing in the design has been run,
that its measurements are owed when it is built,
and that one sentence in the accepted propagation design names this design at an older commit as a dated reading,
which stays as it is.

### The seventeen choices as the project was shown them

Each is given in the words of the question's preview,
with what was not taken in brackets.
The label that closes each is the design's veto entry,
added by the coordinator so that the entry can be found.

1.  The handler starts as an ordinary event script the event daemon does not wait for
    (not:
    one it waits for).
    V1.
2.  It looks only at events about the clone parent's datasets
    (not:
    every event of the pool;
    not:
    no filter).
    V2.
3.  Every Git command the wrapper forwards checks a small folder and may nudge,
    the gap between nudges doubling
    (not:
    only some commands;
    not:
    no gap).
    V3.
4.  A detached helper writes the nudge,
    so no command waits on the disk
    (not:
    the command itself with a deadline;
    not:
    the commit service).
    V17.
5.  The claim for the next nudge is not forced to disk.
    A crash can leave an empty file,
    which readers skip.
    A killed helper leaves one extra file per gap until the request is answered
    (not:
    force each claim to disk,
    which makes one Git command per gap wait on the disk).
    V18.
6.  An entry is written when the request is made,
    so a killed add still leaves a trigger,
    for one small synced write per add
    (not:
    only when the wait gives up).
    V15.
7.  A waiting command nudges once itself after a delay
    (not:
    never,
    so a lost event costs its whole deadline).
    V16.
8.  Event-driven passes run one at a time
    (not:
    side by side).
    V4.
9.  Passes started by boot readiness are never held back or counted
    (not:
    held like any other).
    V5.
10. The budget counts seconds of pass time per fixed window
    (not:
    number of passes;
    not:
    decide the unit later).
    V13.
11. The run claim,
    the mark and the budget live in one folder under `/run`
    (not:
    split across folders).
    V6.
12. A cost record that cannot be read is reported and the pass runs
    (not:
    refuse the pass).
    V7.
13. The handler's decisions go to the journal and standard error
    (not:
    a file under `/run`;
    not:
    standard error only).
    V8.
14. `doctor` prints the `systemctl` commands to run and asks the service manager nothing
    (not:
    read unit state itself).
    V9.
15. A lost last event is a finding in `doctor`'s report
    (not:
    an error exit).
    V10.
16. A wait that times out names what retries:
    the next Git command in this repository
    (not:
    say nothing;
    not:
    read the budget and blame it).
    V11.
17. The merge device is built in step 19,
    the budget's number set in step 22
    (not:
    both in 22;
    not:
    a provisional number in 19).
    V12.

### Where the design's entries say more than the project was shown

The coordinator's note,
not the project's words.
The preview shortened four entries,
and the design's fuller text is what gets built unless the project says otherwise:

- V2 also lets through an event whose parent the handler cannot resolve.
- V3's example gap values are placeholders for a measurement.
- V13 uses windows that do not overlap,
  and leaves the budget's threshold to its measurement M3.
- V11's sentence ships with the nudge's switch in plan step 22.

V14,
how long a lost request keeps being nudged,
is no veto entry:
it is the answer under "A further answer:
how long a lost request keeps being nudged".
