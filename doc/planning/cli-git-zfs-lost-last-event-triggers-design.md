# Which triggers a lost last event waits for, and how nudges are merged

## Status

Non-final and unapproved design proposal by an agent.
The project answered its three questions on 2026-11-22
([the decision record][answers])
and has not read this whole design;
only the project declares a design accepted.
"Revisions" lists the commits and what each changed,
including the findings each answered.

It designs the open item "Which triggers a lost last event waits for" of the accepted ZFS worktree route
([decision][decision],
"Decision";
[proposal][proposal],
"Open items the decisions leave").
The plan puts that item on
steps 7 and 22,
"not blocking either,
writer design work;
the rate rule waits for step 22's measurement"
([plan][plan],
"Open items and the step that needs each",
lines 2850 to 2854).
This design ships no rate:
it designs
the rule's shape and names the measurement that sizes it.

What it read:
the records named in "The records that bind this design" and in "Sources read",
the project's three answers
([the decision record][answers]),
the built handler code
the facts name,
the accepted start-probe design,
the sibling design documents on their branches,
and OpenZFS at
`zfs-2.4.4` and systemd at `v262`.

What it ran:
read-only probes of this host,
listed in "Facts";
two shallow clones of the outside sources under this
round's scratch folder;
a script that printed every cited line range beside the words the document quotes from it.
What it did not run:
no guest,
no virtual machine,
no container,
no ZFS or pool command that changes anything,
no
mount or namespace of its own,
no handler,
no wrapper command that makes a request,
no test suite,
and no service
manager action of any kind.
Nothing here was measured on this machine,
and every measurement it owes sits under
"Measurements owed".

It edits no code,
no specification,
no decision record,
no handover,
no sibling design and no other planning
document.

## The open item, and the answer it carries

The item,
quoted whole from [proposal][proposal],
"Open items the decisions leave",
lines 1780 to 1787:

> Which triggers a lost last event waits for.
> The answer's words:
> "Triggers only,
> and we control the triggers,
> which
> makes the con of "unavailable until next trigger" moot."
> The triggers this design installs or writes are the boot
> unit's start reconcile in `reconcile-git-worktree-clone-mounts.service`,
> the verified checkout provider's
> post-key-load readiness hook,
> and the clone-local sweep nudge the wrapper writes on a caller's behalf.
> ZED's
> delivery of events to the installed handler is installed but not controlled by the design,
> and its loss is the one
> that reaches the no-waiter case,
> where no caller waits and no nudge is written.
> The provider hook is proposed and
> unrun,
> and the coalescing and rate rule for nudges is unwritten
> ([Lost last event without a waiter](#project-decisions));
> none of that is designed here.

The answer itself,
in the project's words,
from [proposal][proposal],
"Project decisions",
under "Lost last event
without a waiter",
lines 1393 to 1395:
"Triggers only,
and we control the triggers,
which makes the con of
"unavailable until next trigger" moot."
The same entry,
lines 1404 to 1407,
keeps the gap this design closes:

> how the nudges pending for one clone merge into a single sweep,
> and how many sweeps may run in a period across
> every clone,
> is unspecified,
> and nothing measured sizes either

and requires that the project "Measure the cost of a sweep and the lost-event behaviour before any bound on how many
sweeps may run in a period is shipped."

Four things were undesigned when this design started.
The triggers are named but not described:
who fires each,
when,
as which user,
what each guarantees and which lost event each recovers.
The no-waiter case has no statement of what happens between the loss and the next trigger.
The provider hook is "proposed and unrun" and has no shape.
The merge and rate rule have no shape at all.
This design answers those four;
the project's three answers settle the hook,
the lost request with nobody waiting and what the cost rule may hold,
and "The project's three answers" states them.

## The project's three answers

The project answered all three questions in
[the decision record][answers],
accepted 2026-11-22 in the question tool and recorded 2026-11-22T19:20:28-05:00.
This design is written to those answers:
each answer below quotes the record under the heading it answers,
the sections it decides cite it,
and "The options not chosen" lists the rest once.

### The readiness hook asks the boot unit

The record's words,
under "Which trigger runs after the checkout becomes ready":

> The project chose "B:
> ask the boot unit (Recommended)":
> a drop-in on the provider whose `ExecStartPost=` is prefixed with a dash and runs
> `systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service`,
> with the boot unit gaining an `ExecReload=` that runs the handler's `boot-reconcile`.
> The same command is the lever a person runs.

So T3 and T5 are what the project chose,
the hook's two files are fixed,
and the hook section states them once for a sibling to cite.
The record also records the costs it accepted with this option:
a `systemctl` call from a unit's start is untested until a guest runs it,
a hand-run `zfs mount` fires nothing,
and the `ExecReload=` line is what keeps the printed lever from restarting the unit.

### A Git command recovers a lost request with nobody waiting

The record's words,
under "Whether a lost request with nobody waiting may wait for the next trigger":

> The project chose "B:
> nudge on Git commands":
> whenever a wrapped Git command runs in a repository that holds a clone with an unanswered request,
> it writes that clone's nudge.
> This is no timer.
> The recommended option was to accept the wait with no bound;
> the project did not take it.

and what it owes before shipping:

> The design therefore owes a way for a command to learn cheaply that a repository has no unanswered request,
> and a measurement of what a command pays in each case,
> before this ships.

So the wrapper's command path carries a trigger (T4),
"## The nudge on Git commands" designs the record and the command's check,
and measurement M8's numbers ship before the nudge does.
"Periodic reconciliation without a waiter is not part of this route" stands,
and this design adds no timer.

### The cost rule holds only passes with no unanswered request

The record's words,
under "Whether the cost rule may hold a pass while a caller waits":

> The project chose "C:
> hold only idle events (Recommended)":
> the rule holds only passes whose event names a clone with no unanswered request,
> and an event for a clone that has one always runs.

and the condition the record keeps:

> so the rule is adopted once that is measured

So "What is bounded" states the rule with its exception,
a caller's own pass is never held,
and measurement M3 sizes the one clone read the rule takes before its decision.
The cost the record accepted with this answer is restated there.

### The options not chosen

The record's list,
once,
with what each would have done.

- Running the handler directly from the provider,
  a mount unit of its own for the checkout,
  and no hook.
  The direct hook is designed as
  "Why the direct hook cannot work,
  and what happens if it is written anyway",
  and the `.mount` shape as
  "The shape the project did not take",
  because the tests they would need and the provisioning they would change are why the project
  did not take them.
- No time bound,
  and a timer,
  for a lost request with nobody waiting.
  No time bound is what L1 states;
  a timer is refused by the proposal's own sentence ([proposal][proposal],
  lines 537 to 539),
  which a nudge on commands replaces without a timer.
- Holding every pass over the budget,
  and no bound,
  for the cost rule.
  "What is bounded" states why the exception the project chose is the one a waiting caller needs,
  and V13 and its window choice are the shape the bound ships in.

## The records that bind this design

Each passage is quoted with its path and line,
and the sentence after it states what it settles here.

The route is accepted with its open items open,
and the root-installed ZED handler and its lifecycle unit own clone
mount and unmount,
"not Git or automatic deletion" ([decision][decision],
"Decision",
line 32).

The intent is a clone-local property,
the receipt a root-owned file,
readiness a root-owned record,
and the nudge a
property a command writes (T4):
"A waiter may change the local
`org.monochromatic:git-worktree-clone-mount-sweep-nonce` as often as it needs on the verified managed clone it is
waiting for,
not on the clone parent.
The nudge uses the same hex format and is not a mount instruction."
([proposal][proposal],
"Requests,
results and ordering",
lines 573 to 576).
The same passage states the limit and the
shape:
"The handler cannot limit the nudge to one write:
the account owns that property and may change it repeatedly,
and each change may start a sweep that scans every task,
so bound the work of one sweep instead of counting nudges."
(lines 575 to 578).
So a burst of nudges is expected,
the handler cannot refuse the write,
and a rule must bound work
rather than count writes.

A handler's refusal is never a success,
and only work reaches zero:
"A refusal is never a success:
a boot unit's `ExecStart` that could not reconcile must fail,
so a status of zero is
written only after a mode did its work" (`handler_mode_dispatch.rs`,
lines 11 to 14),
with `EXIT_REFUSED` 2 (lines 35
to 40).
So a trigger that passes at nothing,
held or filtered,
exits 2 and says why.

A receipt carries the request's nonce,
and a stale receipt is never the answer ([proposal][proposal],
"Requests,
results and ordering",
lines 564 to 572;
[spec][spec],
"The receipts",
lines 6131 to 6183).
So a caller's
confirmation is its own nonce.

Waits are bounded and monotonic,
and a deadline is not cancellation:
"Use a monotonic deadline and bounded waits;
a
matching receipt must still be confirmed by the caller's mount view.
Report a hidden mount separately from a
timeout."
([proposal][proposal],
"Waiting and refusal",
lines 782 to 785),
and "A deadline is not cancellation:
the
durable request may act later."
(lines 792 to 794).

A ready checkout is a condition of every attach:
"An attach is admitted only when the record names this boot,
the
caller's pool and the caller's checkout,
and both the pool and the checkout are ready;
`unknown` admits nothing."
([spec][spec],
"The readiness record",
lines 6184 to 6228).
The builder of that record publishes it only from facts
that passed,
so a boot whose checkout is not ready has no record for this boot rather than a `not-ready` one (fact
F16).

A failed prerequisite publishes nothing and authorizes no attachment ([spec][spec],
lines 6224 to 6228;
`sweep_preconditions.rs`,
lines 535 to 551;
`boot_reconcile_mode.rs`,
lines 22 to 26).

A held slot is refused rather than queued,
and the holder reads the intent again after it releases,
"a sweep that
finds the slot's claim held refuses ... so a request that arrives during a sweep is not lost"
([decision 8][step8],
"A slot whose claim is held",
lines 78 to 86;
[spec][spec],
"The per-slot claim",
lines 6229 to
6271).
The accepted text also keeps parallel passes:
"parallel handlers can sweep at once but cannot take conflicting
actions on the same mount" ([proposal][proposal],
lines 561 to 562).

A refusal files no receipt except an unavailable key's ([decision 8][step8],
"Receipts for refusals",
lines 70 to 76;
[spec][spec],
"The sweep,
and the boot it reconciles",
lines 6272 to 6387).

The boot unit is `Type=oneshot` with `RemainAfterExit=yes`,
`Requires=` and `After=` the checkout provider,
`ExecStart` running `boot-reconcile` and `ExecStopPost` running `shutdown-unmount`
([proposal][proposal],
"Boot and shutdown unit",
lines 499 to 540).
Its `Requires=` carries the dependency's other
face:
"with or without specifying `After=`,
this unit will be stopped (or restarted) if one of the other units is
explicitly stopped (or restarted)" (`systemd.unit(5)`,
`Requires=`,
lines 681 to 694 at tag `v262`).

The start probe runs as `ExecStartPre=-` of that same boot unit,
in a private mount namespace of its own,
and writes
this boot's probe record before the unit's `ExecStart` reads it
([start probe design][probedesign],
"Who runs it,
and where",
lines 1390 to 1401;
[start probe decision][probdecision],
"Decision").
So the probe record exists only after the provider has finished
starting,
because the boot unit is ordered after the provider.

The sweep reads that record first,
and a record that is absent,
names another boot or carries any verdict but
`passed` refuses the run ([spec][spec],
"The sweep,
and the boot it reconciles",
lines 6272 to 6300).

On this host the checkout provider is `zfs-mount.service`,
a oneshot with `RemainAfterExit=yes` whose `ExecStart` is
`/usr/bin/zfs mount -a` and which is ordered `Before=local-fs.target` (fact F3).

The mount-request deadline is provisional and unsized ([spec][spec],
"The wrapper's side and its deadline",
lines
6388 to 6416),
the constant is `PROVISIONAL_MOUNT_REQUEST_DEADLINE` at 30,000 milliseconds with a 50 millisecond poll
(`mount_request_wait.rs`,
lines 59 and 68),
and step 22 sizes it ([plan][plan],
"Step 22",
lines 2340 to 2354).

The wait reads before it sleeps,
and a fresh request's first read cannot match its own nonce
(`mount_request_wait.rs`,
lines 326 to 359).
So any nudge written by a wait is written after at least one read,
and a
nudge written on the first read is a nudge at the start.

No configuration is needed and nobody makes a setting:
"The project first answered that everything must work by
default with no configuration",
and "1:
no file needed (Recommended)" for a host with no file
([decision 8][step8],
"The datasets the sweep acts on" and "With no configuration file",
lines 17 to 68).

The wrapper never asks for a secret and refuses where it lacks authority ([spec][spec],
lines 7347 to 7352).

A request to a system unit is answered by the caller session's authorization agent,
and the command waits while that
agent asks,
so a printed command is printed and never run by a step,
and it carries `--no-ask-password` so that no shell of a script waits on the agent
([sibling propagation][sibprop],
its fact F22,
lines 649 to 662 at `30274435f3`).
That is why this design prints levers and runs none.

Concurrency is lock-free first,
"one owner per item",
and an unavoidable lock covers the smallest unit with its reason
written beside it ([AGENTS.md](../../AGENTS.md),
rule `NLK`).
A shipped resource limit cites a workload measurement or
an owner decision beside the value (the same file,
rule `LQM`).
The precedent for a small durable record without a lock
is one file per item,
published whole,
with no listing on a waiting path ([decision store][store],
whole file).

### Where the records need amending

Five statements do not hold after this design,
and none is edited here.

1.  The shipped refusals say `doctor` reports a receipt,
    a readiness record and a claim "once plan step 10 adds it"
    (`mount_request_wait.rs`,
    line 298;
    `clone_mount_readiness_admission.rs`,
    line 100;
    `clone_mount_receipts_folder_refusals.rs`,
    line 185;
    `per_slot_root_claim.rs`,
    line 151),
    while the plan builds
    `doctor` in step 18 ([plan][plan],
    "Step 18",
    lines 2017 to 2120).
2.  The plan's step 7 and step 22 "Open items" sentences say the coalescing and rate rule stay unwritten
    ([plan][plan],
    lines 1148 to 1153 and 2410 to 2416).
    This document writes them.
3.  The accepted text says parallel passes may run at once ([proposal][proposal],
    lines 561 to 562).
    This design's route-wide claim makes
    event-driven passes exclusive,
    which narrows that sentence;
    "Choices this design marks for the project's veto",
    V4,
    offers the alternative and the project decides.
4.  The plan's step 22 says "no existing constant changes" ([plan][plan],
    lines 2340 to 2354),
    while this design puts
    code there if the project keeps the bound's number for that step;
    "Dependencies on other designs and on the plan"
    names the plan edits owed.
5.  The specification edits this design implies,
    the three cost records,
    the record of unanswered requests and the timeout sentence,
    are listed in the
    same dependencies section,
    because this design edits no specification.

## ZED's delivery, read at zfs-2.4.4

Every statement here is read from OpenZFS at tag `zfs-2.4.4`,
cloned in this round's scratch folder,
and none is
measured on this host.

A user property change reaches ZED as a `history_event`.
The kernel posts that class from the history record's sync
task and only for an internal history record (`module/zfs/spa_history.c`,
lines 296 to 331,
with the call at line 329),
and the class string is `history_event` (`include/sys/sysevent/eventdefs.h`,
line 121).
A `zfs set` of a user property
is such a record:
the step 7 measurement records that "the property and its value are in `history_internal_str` and
`ZEVENT_HISTORY_CMD` is empty"
(`package/git-policy/cli/doc/zfs-mount-receipt-deadline-measurement-2026-11-21.md`,
"Where",
lines 64 to 66).

The kernel holds events in one in-memory list,
512 by default (`module/zfs/fm.c`,
line 72).
A new event goes in at the
head,
and at the bound the kernel drains the tail,
which is the oldest (`module/zfs/fm.c`,
lines 180 to 192).
A drain
credits a drop only to a reader that holds a cursor on the drained event (lines 143 to 156);
a fresh reader starts at
the tail with no count (lines 316 to 323 and 353 to 359).
So a drain that happens while ZED holds no cursor,
which is
every drain while ZED is stopped or restarting,
is silent:
no `Missed N events` line is ever written for it.

When ZED reads,
the count reaches it and it logs `Missed N events` and doubles the bound up to a ceiling
(`cmd/zed/zed_event.c`,
lines 969 to 978;
lines 111 to 152).
Nothing re-posts a drained event.

A post can fail before any event exists:
an event larger than the report payload bound,
or an allocation failure,
is
counted and dropped (`module/zfs/fm.c`,
lines 224 to 255).

ZED records the last event it wrote state for and seeks to it at start (`cmd/zed/zed.c`,
lines 264 to 299;
`cmd/zed/zed_conf.c`,
lines 543 to 665).
The seek has two outcomes:
if the saved event is still there,
ZED services
what follows it;
if it is not there and a saved id existed,
or if no state file exists,
ZED seeks to the start and
services the whole queued stream from its oldest event (`cmd/zed/zed_event.c`,
lines 168 to 221).
So a ZED start
replays the queue,
and an event posted while ZED was down is delivered when ZED starts,
unless the queue drained it.
The state file is `RUNSTATEDIR "/zed.state"` (`cmd/zed/zed.h`,
line 27),
which this host's installed binary embeds as
`/var/run/zed.state`,
a per-boot path (fact F11).

ZED forks each matching zedlet and,
for an asynchronous one,
does not wait for it:
the state for the event is written
right after the fork (`cmd/zed/zed_event.c`,
lines 1020 to 1030).
A zedlet whose name puts `-sync-` right after its
class prefix makes ZED wait for all its children around that fork (`cmd/zed/zed_exec.c`,
lines 339 to 357 and 422 to
441).
The installed handler's name,
`history_event-mount_git_worktree_clone_datasets` ([spec][spec],
lines 262 to 268),
has no `-sync-`,
so the handler is asynchronous today.

ZED runs at most 16 zedlet processes at once (`cmd/zed/zed_conf.c`,
line 51) and blocks before forking while that many
are alive (`cmd/zed/zed_exec.c`,
lines 130 to 140).
Blocked that way it is not reading the queue,
so a burst can drain
events only if ZED holds a cursor,
which it does between reads.

A `fork` that fails is logged and the event's other zedlets still run (`cmd/zed/zed_exec.c`,
lines 149 to 156),
so one
zedlet can miss an event while ZED reports success for it.

ZED scans its zedlet directory at start and again on `SIGHUP` (`cmd/zed/zed.c`,
lines 243 and 291 to 293),
so a zedlet
installed under a running ZED is not run until that scan.

The host's unit runs `/usr/bin/zed -F` (`/usr/lib/systemd/system/zfs-zed.service`),
and in the foreground ZED points
each zedlet's standard input,
output and error at `/dev/null` (`cmd/zed/zed_exec.c`,
lines 161 to 166).
So a
ZED-started handler's own lines reach nobody.
What the journal does hold is ZED's `Invoking` line and its `Finished`
line with the exit status or signal (`cmd/zed/zed_exec.c`,
lines 189 and 240 to 261).

The ways the last event for a clone is lost,
not an exhaustive list:

1.  The queue drained it because events arrived while ZED held a cursor and did not read fast enough,
    or because more
    events arrived than the queue holds while ZED was stopped,
    which is silent (see above).
2.  The module unloaded or the machine rebooted,
    which empties the list.
    Nothing on disk is replayed as an event.
3.  ZED forked the handler and wrote the event's state,
    and the handler then failed or was killed.
    The event is not
    delivered again.
    A handler killed by a ZED restart is this case,
    because systemd kills the whole control group of a
    stopped service by default (`KillMode=` defaults to `control-group`,
    `src/core/kill.h`,
    line 8,
    and
    `systemd.kill(5)`,
    lines 99 to 103 at tag `v262`).
4.  The event reached ZED and the handler ran,
    but the pass refused or was held back,
    so no receipt answers the
    request.
5.  The kernel dropped the post,
    the fork failed,
    or the zedlet was not in ZED's scanned set.
    Each leaves no event for
    the clone and each is consistent with the trigger set here,
    because a pass reads live state.

A sixth case is not a loss:
a request whose event was never written,
because the account's `zfs set` failed or its
outcome is unknown.
The wrapper inspects an unknown outcome before any retry (`clone_mount_intent_write.rs`,
lines 68
to 82),
and this design's triggers do not cover it.

## The triggers, one by one

Six entries follow.
Five are triggers;
the sixth is a state of the trigger that is easiest to mistake for one.
Each
entry names who fires it,
when,
as which user,
what it guarantees,
which lost event it recovers,
and where its
decisions are recorded.
T3 is the hook the project's answer chose,
and T4 is the trigger it chose for a request with nobody waiting
([the decision record][answers]).

### T1: a ZED `history_event` delivery, or a ZED start that replays the queue

- Fired by:
  the ZED daemon (`zfs-zed.service`),
  for each event it reads from the kernel queue whose class matches the
  installed zedlet,
  and at its own start,
  which replays the queue from the saved event or from the oldest.
- When:
  whenever the kernel posts a `history_event` for any pool,
  which includes the account's `zfs set` of an intent
  or a nudge,
  and whenever ZED starts,
  which delivers what the queue still holds.
- As which user:
  ZED runs as root and the zedlet inherits that;
  the handler runs in the host mount namespace
  ([proposal][proposal],
  lines 483 to 484).
- Guarantees:
  one delivery attempt per queued event that ZED holds a cursor on,
  at most once after the state write,
  and
  a replay of everything still queued at a start.
  It guarantees nothing about a drained event,
  a failed fork,
  or a
  zedlet installed after the last scan.
- Recovers:
  a request whose event is delivered,
  a request whose event was posted while ZED was down but not drained,
  and therefore most lost last events of a ZED stop or crash.
  It does not recover a drained event.
- Recorded where:
  ZED's `Invoking` and `Finished` lines with the exit status,
  under `zfs-zed.service`.
  The handler's
  own line reaches nobody (fact F13),
  which "Choices this design marks for the project's veto",
  V8,
  fixes.
- Filter:
  the handler filters the event before it opens `/dev/zfs` (V2),
  because the zedlet runs for every pool.

### T2: the boot unit's start reconcile

- Fired by:
  `reconcile-git-worktree-clone-mounts.service`,
  whose `ExecStartPre=-` runs the start probe and whose
  `ExecStart` runs `boot-reconcile` ([plan][plan],
  "Step 19",
  lines 2121 to 2150;
  [start probe design][probedesign],
  lines 1390 to 1401).
- When:
  once per boot,
  ordered after the checkout provider,
  the key providers and `zfs-import.target`
  ([proposal][proposal],
  lines 499 to 514).
  It is also re-fired by `start` when the unit is failed or inactive,
  and by
  `reload-or-restart` when the sibling's `ExecReload=` exists (T5).
- As which user:
  root,
  in the host mount namespace ([proposal][proposal],
  line 513).
- Guarantees:
  one probe record and one pass per start after the route's prerequisites,
  and a readiness record published
  `running` before the pass and `finished` or `failed` after it,
  or nothing at all when a prerequisite refuses
  ([spec][spec],
  lines 6272 to 6387;
  fact F16).
- Recovers:
  every lost event of the previous boot and of this boot so far,
  because the pass reads live intents.
  It
  recovers nothing until it runs.
- Recorded where:
  the unit's journal,
  under `reconcile-git-worktree-clone-mounts.service`.

### T3: the checkout provider's start asks the boot unit to reconcile

- Fired by:
  `zfs-mount.service`'s `ExecStartPost=`,
  once step 19 installs the drop-in (the project's answer,
  [the decision record][answers]).
- When:
  whenever the provider runs a start job,
  which on this host is the boot,
  a `restart`,
  or a `start` after the unit was stopped.
  A `start` of the already active unit does nothing (fact F15).
  In the late-key flow the provider is active,
  because `zfs mount -a` skips an encrypted dataset whose key is unavailable and still exits zero
  (`lib/libzfs/libzfs_mount.c`,
  lines 1163 to 1179 at `zfs-2.4.4`;
  fact F37),
  so there the commands that fire the hook are a provider `restart`,
  or a stop and a start;
  a provider `start` alone runs no job.
  What a provider `restart` does to the boot unit depends on that unit's state (fact F17):
  an active unit is restarted,
  with its `ExecStopPost=` `shutdown-unmount` first,
  and a stopped or failed one is left alone,
  because a restart closes to a try-restart on the dependents and a try-restart collapses to nothing for a
  unit that is not active.
  So the hook alone covers the failed and stopped states,
  which are the late-key state;
  the active state reconciles without it,
  at the price of the stop pass.
- Runs:
  the one line "The hook,
  in one form a sibling can cite" gives,
  a `--no-block` request to `reload-or-restart` the boot unit.
  The hook itself runs no handler and makes no pass.
  Its reload arm is never reached on its own:
  a provider `restart` stops the boot unit first,
  because the unit orders after the provider,
  and by the time the provider's start-post runs the unit is stopped or failed and holds a pending start,
  so the request collapses to a start and merges into that pending start
  (`src/core/job.c`,
  the merging table at lines 416 to 424 at `v262`).
  The reload is the project's lever,
  not the hook's.
- As which user:
  root,
  through the service manager,
  which runs the unit in its own namespace and journal.
- Guarantees:
  one enqueued job for the boot unit,
  and never a failed provider:
  the hook is prefixed `-` and the
  command it runs is a job request,
  not a reconcile (the record lists the costs it accepted with this answer).
- Recovers:
  a boot whose reconcile refused because the checkout was not ready,
  once the provider is restarted,
  or stopped and started,
  which in the late-key flow is also the command that mounts the checkout now that the key is loaded;
  and any request written between the two.
- It cannot see a checkout mounted by a hand-run `zfs mount`,
  because no unit runs then.
  T5 is that case's lever.
- This is the project's answer ([the decision record][answers]),
  which chooses it and the boot unit's `ExecReload=` together,
  and step 19 installs both.

### T4: the wrapper's clone-local nudge on Git commands

- Fired by:
  an invocation of the wrapper that forwards its command to Git,
  once per invocation,
  when the repository it runs in holds the record of unanswered requests.
  The project chose this trigger
  ([the decision record][answers],
  "A Git command recovers a lost request with nobody waiting").
- When:
  on the forwarded path,
  where the wrapper already resolves the repository,
  and before the child Git starts (fact F38).
  A command writes at most one nudge per listed clone,
  and only when that clone's entry is older than `G`,
  the gap M8 sizes.
- As which user:
  the account writes the property,
  through the same `mount,userprop` grant as before
  ([proposal][proposal],
  "Scoped delegation",
  line 338);
  the delivery and the pass are root's,
  as T1.
- Guarantees:
  one further history event per nudged clone,
  hence one delivery attempt of T1,
  and nothing for the command to wait on or to fail from:
  the check is one `stat` when no record exists,
  the write is best-effort,
  and a refused device,
  a refused property write or a record that cannot be rewritten prints one line and leaves the
  command's exit status alone.
  A pass the nudge's event starts is never held by the budget,
  because the event names a clone with an unanswered request
  ([the decision record][answers],
  "The cost rule holds only passes with no unanswered request").
- Recovers:
  a request whose own event was lost while ZED is alive,
  at the next wrapped command in that repository,
  and a request written after a running pass had read that clone.
  With ZED stopped it recovers nothing,
  which is a likely cause of the first loss.
- Recorded where:
  the wrapper's one line for the nudge,
  the record itself,
  and V8's answer for the handler's line.

### T5: a trigger the project fires

- Fired by:
  the project,
  with the commands `doctor` prints,
  and run by nobody the design starts.
- The root lever,
  the record's answer:
  `systemctl --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service`,
  which is one verb for every unit state:
  on an active unit it runs the sibling's `ExecReload=`,
  on a failed or inactive one it starts the unit,
  and a plain `reload` is refused on a failed or inactive unit,
  which is the state a boot whose checkout was not ready leaves
  (`systemctl(1)`,
  `reload-or-restart`,
  lines 530 to 537;
  `src/core/unit.c`,
  lines 2154 to 2156 at `v262`;
  fact F15).
  Its one condition is the sibling's `ExecReload=`:
  with that line an active unit reloads and nothing unmounts;
  without it,
  the same command on an active unit is a restart,
  which runs `ExecStopPost=shutdown-unmount` and unmounts every clone,
  so the printed lever would be a stop pass over the clones.
  If the sibling's choice 3 is vetoed,
  the printed lever is `start` for a failed or inactive unit and none for an active one.
  `start` alone re-runs a failed or inactive unit and does nothing on an active one (fact F15);
  `restart` always runs the stop pass first and unmounts every clone (fact F17).
  Running the handler binary from a person's shell is not recommended,
  because that
  shell's mount namespace is not the host's (fact F18).
- The account lever:
  a `zfs set` of a fresh nudge nonce on the clone,
  which is T1's delivery and therefore ZED-bound:
  while ZED is stopped it recovers nothing.
- As which user:
  root for the root lever,
  the account for the nudge.
- Guarantees:
  one pass in the unit's own context for the root lever,
  and exactly T1's guarantees for the nudge.
- Recovers:
  any lost event when ZED is alive (the account lever),
  and any lost event including a stopped ZED (the root
  lever).
- Recorded where:
  the unit's journal for the root lever,
  and V8's answer for the nudge.
- The choice of what `doctor` prints is V9,
  and the same answer makes the provider hook share this command.

### What is not a trigger

A ZED restart is not a separate trigger,
but a ZED *start* is a delivery:
it replays the queue (T1),
so its effect is
T1's guarantee and not a silence.
A systemd mount noticed after the fact is not a trigger:
systemd shows a mount made
outside a unit as a mount unit,
but no unit file ran and no `ExecStartPost=` fires (`systemd.mount(5)`,
"Mount points
created at runtime",
lines 72 to 78 at tag `v262`).
A wrapper's read of a receipt is not a trigger:
the wrapper only waits (fact F20).
The wrapper's forwarded path is a trigger,
and it writes a nudge only when the record names a clone (T4).
`doctor` is not a trigger;
it reports and prints T5's commands without running them
([plan][plan],
"Step 18",
line 2064).

### The event filter, in brief

The built handler reads no event variable,
so T1 would start a pass for every history event of every pool,
including
every sanoid snapshot of the checkout and the home datasets and every `zfs set` by another tool.
V2 puts the filter's
options to the project;
the route this design describes accepts an event whose pool is the parent's pool and whose
dataset,
with any `@snapshot` suffix removed,
is the parent or below it,
and drops every other event with one line and
no device open.
The filter keeps the bound for real requests,
and its cost is the accidental recovery it gives up:
a
pass it drops could have attached a clone whose own event was lost (section "The no-waiter case").

## The nudge on Git commands

The project's second answer makes the wrapper's command path a trigger,
and the record says what it owes before it ships:
a way for a command to learn cheaply that a repository has no unanswered request,
and a measurement of what a command pays in each case.
This section designs both,
with the record that carries them.

### The record an unanswered request leaves

The wrapper leaves one small record per repository,
in the Git common directory of the invocation,
whose canonical path the wrapper already has from the identity it asks Git for on every command
(fact F38).
The file is `zfs-unanswered-request`,
mode `0600`,
in that directory:
the account owns the directory and writes the file,
the wrapper reads it,
and no root process writes it.
The per-add claim record already lives in the same account-owned directory
(fact F40),
so the place is the route's own and not a new one.

One line per clone with an unanswered request,
after a first line `v1`,
with the constant-width fields first and the name last:

    <nonce> <operation> <ended-at> <nudged-at> <dataset>

- `nonce` is the request's 32 lowercase hexadecimal digits,
  the grammar the intent property carries
  ([spec][spec],
  "The durable intent").
- `operation` is `mount` or `unmount`.
- `ended-at` is the moment the wait ended with no answer,
  as an ISO 8601 timestamp with its offset.
- `nudged-at` is the moment of the last nudge written for it,
  or `never`.
- `dataset` is the clone's canonical dataset name,
  last on the line,
  so no field before it can be broken by a space in the name.

The file is replaced whole by one staging write and one rename,
the way the route's other records are published (fact F23),
and it is read with one bounded read of one page,
the bound the start probe already gives each small file it reads (fact F41).
A writer whose entry would not fit the page refuses the entry,
prints one line and leaves the file as it was;
`doctor` reports the refusal and the entries it could read.

An entry is written by the wrapper's wait at the point it gives up.
A wait that ends with no receipt answering its own nonce writes the clone's entry
before it prints the deadline report,
and a wait the person interrupts writes it too,
because the request still stands.
A wait that receives its answer writes no entry,
and removes the clone's entry if an earlier invocation left one.

An entry is removed,
in the order that usually happens,
by:

- the next wait on that clone that receives its answer,
  because that command knows its clone and its nonce;
- the removal,
  when it settles the clone,
  which is step 14's (D8);
- an invocation that finds an entry whose `ended-at` is older than `A`,
  which removes it without opening a device,
  where `A` is the stale age measurement M8 sizes;
- `doctor`,
  which reports an entry whose clone no longer exists and prints the one line that removes it,
  and runs nothing.

What a stale entry costs:
until one of those removes it,
each invocation that finds it opens the device once,
writes one property that fails,
prints one line and continues.
So the cost is one failed write per command per stale entry,
bounded in time by `A`,
and no command fails because of it.

What happens when the record is missing though a request is unanswered:
the record is a cache and never authority.
The request's durable intent still stands,
the route's other triggers (T1,
T2,
T3,
T5) still recover it,
the next wait on that clone writes a new entry,
and `doctor` reports a clone whose intent has no answering receipt whether or not a record names it.
Nothing about the clone is decided by the record's presence:
the record only decides whether a command looks further.

### Which commands, and where

Every invocation of the wrapper that forwards its command to Git checks the record,
once,
on the forwarded path,
before the child Git starts,
at `run_wrapped_command` (`src/native/invocation/wrapped_command.rs`,
fact F38).
The same path already carries a cheap check of one Git pointer file (fact F39),
which is the precedent for a check here.
The check is one `stat` of the record's path:
a repository with no record opens nothing,
reads nothing and writes nothing,
which is the cheap case the record exists for.
A check that cannot stat the path for any other reason prints nothing and treats the repository as having no record,
so a broken path never makes a command fail.

A command writes one nudge per listed clone when that clone's `nudged-at` is older than `G`,
where `G` is measurement M8's,
and rewrites the record with the new `nudged-at` values.
The write is best-effort:
a device that cannot be opened,
a refused property write and a record that cannot be rewritten each print one line,
and none of them waits for a receipt or a pass,
and none changes the command's exit status.
The syscall itself is all the command pays beyond the check.

The wrapper's own management commands do not take this path:
the add and the removal have their own waits and write their own entries,
and `doctor` reads the record to report it.
A child Git the wrapper starts is the real Git and not this wrapper,
so a command cannot reach the check twice.

### How often, and the burst

One entry per clone,
one nudge per entry per invocation,
and only when the entry's `nudged-at` is older than `G`.
A burst of commands inside `G` therefore writes one nudge for that clone,
and the first command after `G` writes one more.
`G` is a shipped limit sized by M8:
the measured time from a nudge's write to the receipt that answers it,
so that a second command during a served request does not write again,
plus the measurement's spread.
Two things bound the rest of the burst:
the merge device,
which lets one pass run and makes the contenders mark,
so N nudges inside one pass cost one pass and at most
one extra pass;
and the project's third answer,
which lets the nudge's pass run even over the budget,
so the budget itself never bounds a nudge.
`G` and the merge device are what bound a command burst,
and V3 offers the alternatives.

### The cost rule with a pending request

The third answer changes the taker's budget step.
Before it decides,
the taker reads the one clone the event names:
its intent and the receipt under its clone's GUID (facts F24 and F25).
A request without an answering receipt passes without the budget being read further,
and an event that names no clone,
a parent event the filter accepts or a ZED-start replay,
is decided by the budget alone.
The read is the cost the record's answer waits for,
and M3 measures it.
A nudge's event always names a clone,
so a nudge is never held;
what bounds nudges is `G`,
the merge device and the single writer of `pass-budget`.

### What a person sees, and what doctor prints

A command in a repository with no record prints nothing new.
A command that writes a nudge prints one line under the wrapper's diagnostics,
naming the clone and saying that the route was asked to reconcile it,
and a command that finds a refused device or write prints one line saying the nudge was not written.
Neither line changes the command's own output or status.

`doctor` reads the record and prints,
for each entry,
the clone,
the nonce and operation,
when the wait ended,
when it was last nudged,
whether the clone still exists,
and,
for an entry past `A` or a clone that is gone,
the one line that removes the entry.
Its lost-event block gains a `nudge` line naming the record and its entries,
and its list of next triggers gains "the next Git command in this repository".
A record it cannot read is reported as unknown,
not as absence.

### The measurement owed

M8 measures what a command pays as ratios against the same command on `main`'s build,
interleaved in one round on this machine and repeated for the spread (rule `QNB`),
in three cases:

- no record:
  one `stat` and nothing else;
- a record with a request still unanswered:
  the read,
  the device open,
  one property write per clone and the record's rewrite;
- a stale record:
  the same,
  with the property write failing.

Each case alternates with the same command on `main`'s build on the same host,
so the ratio is against the wrapper's existing per-command cost and not against a stopwatch.
The positive control is a planted record,
which must make the record path run:
a case that shows no difference is read only after the control moved.
M8 also sizes `G`,
the receipt latency from the nudge's write,
and `A`,
the age after which an entry is stale.

### What builds it

Step 10 builds the record's writer,
its removal on an answer and the command-path check,
with the nudge writing behind a compiled switch that is off;
step 14 writes and removes entries for a removal's waits;
step 18 reads the record in `doctor`;
and step 22 sets `G` and `A` from M8 and turns the nudge on,
which is the record's own condition for shipping it at all.

## The no-waiter case

A clone carries a durable intent,
its last event was lost by one of the ways in "ZED's delivery",
and no caller waits:
the add that wrote the intent timed out and reported,
or the last caller exited long ago.
The project's second answer gives the first trigger to look for,
and the others are unchanged.
Which trigger fires next,
and when:

- T4 at the next invocation of the wrapper that forwards a Git command in that repository,
  once the wait that ended unanswered has left its entry,
  which the project chose as the trigger and M8 measures.
  With a repository in use this is the next recovery,
  and with no command it never fires.
- T1 at the next history event of any dataset under the pinned parent,
  or at the next ZED start,
  which replays what the queue holds.
  With ZED running this is the other early recovery;
  with ZED stopped the next start delivers what was not drained,
  and a drain while ZED stopped is silent (fact F12).
- T3 at the next start of the checkout provider as a job,
  which is rare on this host because the unit stays active.
- T2 at the next start of the boot unit,
  which the next boot always causes,
  and a failed boot unit can be re-fired with `start` (fact F15).
- T5 whenever the project fires one of its two levers.
  The root lever works with ZED stopped;
  the account lever does not.

No trigger here is a timer,
and the project's answer keeps it that way:
"Periodic reconciliation without a waiter is not part of this route"
([proposal][proposal],
lines 537 to 539).
So how long the no-waiter case lasts is "until one of those occurs":
with a repository in use,
the next command,
and with no command,
the next boot.
The project's answer to the word "moot" is that the command trigger is the one it wants,
and it accepted the wait the other triggers leave for a repository nobody touches
([the decision record][answers],
"A Git command recovers a lost request with nobody waiting"):
the answer also requires M8's measurement before the command trigger ships.

The clone's state meanwhile is not lost and not silently changed:

- Its durable intent stands,
  because nothing but its owner or a removal clears it,
  and a timeout is not cancellation
  ([proposal][proposal],
  lines 792 to 794).
- Its slot,
  if the add registered it,
  holds the Git registration and the pointer the add wrote,
  and the clone's dataset is not attached there,
  so the folder shows what the underlay holds.
- The receipt for the clone,
  if any,
  answers an earlier request,
  so a reader sees a stale result rather than a wrong answer
  ([spec][spec],
  lines 6131 to 6183).
- This boot's readiness record names whatever the last readiness trigger published,
  or does not exist;
  a caller is either admitted or refused by the rule that reads it,
  never admitted on a guess (facts F14 and F16).
- Nothing removes the clone,
  mounts it elsewhere,
  or changes its intent while it waits.
  The next trigger's pass is the only writer of its mount state.
- The record of unanswered requests,
  if one was written,
  names the clone and when it was last nudged;
  a record that is missing changes which trigger looks,
  not what the clone needs,
  and the next wait on that clone writes a new one.
- The filter's trade applies:
  an event outside the parent that would have started a pass is dropped (V2),
  so a recovery that would have come from such an event is later.
- The cost rule never holds the pass that this clone's event starts,
  because the event names a clone with an unanswered request
  ([the decision record][answers],
  "The cost rule holds only passes with no unanswered request").

`doctor` reports that state as a clone whose live intent has no receipt answering it (section "What doctor reports for
a clone whose last event was lost") and prints the two levers and the record's entries,
so a project that wants the worktree now has a trigger the design wrote down,
and the next command it runs in the repository has one too.

## The provider's readiness hook

This section answers what the hook is on this machine,
what has to be installed or decided outside the wrapper for it
to exist,
by which command,
where that is recorded,
what it runs,
and why a direct hook cannot work.

### What exists on this host today

`zfs-mount.service` is enabled and active,
a oneshot with `RemainAfterExit=yes`,
`Before=local-fs.target`,
whose
`ExecStart` is `/usr/bin/zfs mount -a` (fact F3).
There is no `.mount` unit for the checkout,
because the checkout is
not a dataset:
`/home/user/Monochromatic` resolves to `zpcachyos/ROOT/cos/home`,
an inherited dataset (fact F6).
`zfs-load-key.service` is masked and `/etc/default/zfs` sets `ZFS_LOAD_KEY='yes'`,
while the pool's keystatus is
`available`;
the root datasets are encrypted,
so keys are available before the boot units run here (fact F31).
The
handler is installed nowhere,
`zfs-zed.service` is disabled and inactive,
and none of the `/run` folders exists (fact
F16).
So today no trigger of this design exists on this machine,
and nothing here claims one runs.

### Why the direct hook cannot work, and what happens if it is written anyway

Five things,
each read from a source or from the project's records.

1.  The probe record does not exist yet.
    The probe is `ExecStartPre=-` of the boot unit ([start probe
    design][probedesign],
    lines 1390 to 1401),
    and the boot unit is ordered after the provider
    ([proposal][proposal],
    lines 499 to 505).
    The provider's `ExecStartPost=` therefore runs before the probe writes
    anything for this boot,
    and `boot-reconcile` refuses a missing record (fact F19) with exit 2 (fact F21).
    At every
    boot the direct hook is a refusal,
    not a pass.
2.  An unprefixed refusal fails the provider:
    "If any of those commands (not prefixed with `-`) fail,
    the rest are not
    executed and the unit is considered failed" (`systemd.service(5)`,
    `ExecStartPost=`,
    lines 427 to 437 at tag
    `v262`).
    The boot unit has `Requires=` and `After=` on the provider,
    so a failed provider means the boot unit "will
    not be started" (`systemd.unit(5)`,
    `Requires=`,
    lines 681 to 694).
    An unprefixed direct hook at boot would fail
    the host's mount provider and keep the reconcile,
    its probe and its readiness from ever starting.
3.  With the dash prefix the failure is recorded and has no further effect (`systemd.service(5)`,
    the special
    executable prefixes),
    so the direct hook is harmless at boot and does nothing there.
4.  `start` cannot re-fire it:
    the unit is active,
    and a start job on an active unit returns `-EALREADY`
    (`src/core/unit.c`,
    lines 1955 to 1962 at tag `v262`).
    The verbs that run it again are the provider's `restart`,
    which restarts
    the boot unit through `Requires=` and so runs its `ExecStopPost=` `shutdown-unmount` first,
    unmounting every
    attached clone before the hook's pass puts them back (fact F17),
    and a stop followed by a start.
    A `reload` of the provider runs nothing,
    because the provider has no `ExecReload=`.
5.  The hook runs inside the provider's start,
    on the boot's critical path:
    the provider is `Before=local-fs.target`
    (fact F3),
    `ExecStartPost=` counts for ordering (`systemd.service(5)`,
    lines 454 to 465),
    a oneshot's start timeout
    is disabled by default (`systemd.service(5)`,
    `TimeoutStartSec=`,
    lines 682 to 692),
    and the hook only runs after
    `zfs mount -a` exits zero,
    so a boot where that fails for any dataset never reaches it.
    The delay it adds is a
    delay to `local-fs.target`,
    which measurement M5 measures.

Two smaller consequences:
a direct hook's output is read under `zfs-mount.service`,
not under the boot unit,
because
that is the unit whose start ran it;
and a boot with the direct hook and the boot unit spends two passes only in the
sense that the first is a refusal.

### The hook, in one form a sibling can cite

One statement,
and every other place in this document refers here.
The provider hook is two files.
The drop-in is shown whole,
so a copy of it is a working file;
the boot unit's line is shown with the section header that carries it,
and the rest of that unit is plan step 19's.

```ini
# /etc/systemd/system/zfs-mount.service.d/reconcile-git-worktree-clone-mounts-after-checkout.conf
[Service]
ExecStartPost=-/usr/bin/systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service
```

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
[Service]
ExecReload=/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets boot-reconcile
```

The `[Service]` header is not decoration:
systemd ignores an assignment outside a section ("Assignment outside of section.
Ignoring.",
`src/shared/conf-parser.c`,
line 252 at tag `v262`),
so a copy without it is an inert file.

What the drop-in's line is,
stated here so that a sibling can cite this section alone:

- The request.
  It asks the boot unit,
  `reconcile-git-worktree-clone-mounts.service`,
  to reconcile:
  `--no-block` queues the job and returns,
  and the hook runs no handler and makes no pass itself.
- The verb,
  per state of the boot unit.
  `reload-or-restart` reloads an active unit through the `ExecReload=` above and starts an inactive or failed one;
  it never takes `restart`'s stop pass,
  a plain `reload` is refused while the unit is inactive,
  and `start` alone does nothing while it is active
  (`systemctl(1)`,
  `reload-or-restart`,
  lines 530 to 537;
  `src/core/unit.c`,
  lines 2154 to 2156 at `v262`;
  fact F15).
- The provider is never failed.
  The `-` prefix makes a non-zero request a logged failure instead of a failed unit
  (`systemd.service(5)`,
  `ExecStartPost=`,
  lines 434 to 436 and 456 to 459 at `v262`),
  so `zfs-mount.service` stays `active (exited)` and the boot unit's `Requires=` on it stays satisfied (fact F17).
  The request's own lines are in `zfs-mount.service`'s journal;
  the pass it starts is in the boot unit's.
- The root command.
  The same verb with `--no-ask-password` is the project's lever and the line `doctor` prints:
  `systemctl --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service` (T5).
  Its one condition is the `ExecReload=` above:
  without it,
  an active unit's `reload-or-restart` is a restart whose stop pass unmounts every clone,
  and the printed command becomes `start` for a failed or inactive unit and nothing for an active one.

The `ExecReload=` is the propagation sibling's veto choice 3
([sibling propagation][sibprop],
lines 2676 to 2699 at `30274435f3`,
2026-11-22T18:50:22-05:00),
and the same line serves its printed lever.
A sibling cites this section by name.

### The shape the project chose

The project's answer
([the decision record][answers],
"The readiness hook asks the boot unit"),
chosen as the only shape that avoids all five of the direct hook's problems:
the two files "The hook,
in one form a sibling can cite" gives.
The hook asks the unit that already owns the ordering,
the namespace,
the probe,
the journal and the stop pass to reconcile;
the provider itself is never failed,
never carries the pass on its critical path,
and never needs a second verb to be re-fired.

What it runs is a job request:
`reload-or-restart` reloads an active unit through `ExecReload=`,
and starts one that is inactive or failed (`systemctl(1)`,
lines 530 to 537 at tag `v262`;
fact F15).
Each of the three unit states needs a
guest arm (test 12).

Sharing one mechanism with the sibling:
the propagation sibling proposes the same `ExecReload=` line as its veto choice 3 and
names `reload-or-restart` as its manual lever
([sibling propagation][sibprop],
lines 2676 to 2699,
read at `30274435f3`,
2026-11-22T18:50:22-05:00).
The project's answer takes exactly that line:
one `ExecReload=`,
two consumers (the hook's job request and the project's printed lever).
The sibling's answer is still unapproved as a whole,
but the line is now part of this design's accepted answer,
so the lever needs no fallback sentence here.

The sibling adds one more trigger that asks the boot unit in the same way:
its question 2 option 1 starts a `checkout-propagation.service` from a drop-in on the checkout's mount unit
with `Upholds=` and `BindsTo=`,
and that service's `ExecStart=` is the same `systemctl --no-block --no-ask-password reload-or-restart` request
([sibling propagation][sibprop],
"The propagation service and its drop-in",
lines 2229 to 2236 and 2464 to 2500 at `30274435f3`;
fact F42).
So one verb has three senders:
this design's provider hook,
the sibling's upheld service,
and the project's printed lever.
Neither the service nor its drop-in is this design's to install,
and the project's answer to the sibling's question is not made here.

What it needs from the project,
outside the wrapper,
and where each part is recorded:

- The provider integration the record settles,
  recorded in this design's answer and in the record.
- The handler binary at `/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets`,
  installed by
  plan step 19 and recorded in `SPEC.md`'s handler paragraph ([spec][spec],
  lines 262 to 270).
  The name is the crate's `[[bin]]` (`package/git-policy/cli/Cargo.toml`,
  line 47),
  and the `ExecReload=` line must carry it and not a variant.
- The zedlet at `/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets`,
  also step 19's and recorded there.
- The checkout dataset `monochromatic-git-checkout`,
  mounted at the checkout path,
  created by step 19's migration and
  recorded in the plan's step 19 ([plan][plan],
  lines 2121 to 2145).
  Until it exists,
  this host's provider has nothing
  to mount for the hook to follow.
- The drop-in and the boot unit's `ExecReload=`,
  the two files "The hook,
  in one form a sibling can cite" gives.
  Both are step 19's interface and are recorded in its runbook;
  the commands that apply them are step 19's printed
  setup commands,
  run on the project's authorisation,
  because none of this runs on this host and writing under
  `/etc/systemd` is the project's act ([plan][plan],
  lines 2150 to 2162).
- The unit states the hook must cover,
  active,
  failed and inactive,
  and the guest arms that cover them (test 12).

It needs nothing from the key:
the wrapper never prompts for a passphrase,
and no part of this asks for one.

### The shape the project did not take

A `.mount` unit for the checkout with a companion unit bound to it fires on every activation of the checkout
mount and gives the checkout a unit state `doctor` can read,
but it changes how the checkout is provisioned
and needs the checkout excluded from `zfs mount -a`,
which is a larger design than this open item.
The record lists it,
with the direct hook and no hook,
among the options it did not take.

## Merging the nudges pending for one clone

### The principle

A pass derives its work from live state,
not from a queue (fact F24).
Two nudges for one clone therefore need no two
work items;
they need one pass that reads the clone after both writes.
The merge rule is about how many passes run and
who runs them.
A nudge carries no state a pass reads,
and a pass that runs after a nudge sees what the nudge's writer
asked for.

What one waiting caller needs is a pass that reads its clone after its intent write.
Its own T1 delivery is one
attempt;
the nudge is a second;
this device makes either enough when a pass is already running.

### The records

One root-owned folder,
`/run/git-worktree-clone-mount-sweeps`,
made by the handler when a fresh boot has none,
mode
`0755`,
beside the receipts and readiness folders this route already publishes (fact F16):

- `run.<generation>`,
  the run claim's numbered generations.
  The repository's numbered claim keeps only names of the
  form `<prefix>.<digits>` and ignores every other name in its folder (`single_owner_claim.rs`,
  lines 208 to 234),
  and
  the per-slot directory already holds every slot's claim files in one folder,
  so the other two names sit beside the
  generations and no subfolder is needed (fact F22).
- `another-pass-is-due`,
  a root-owned regular file of mode `0644` whose presence means a trigger asked for a pass no
  pass has covered.
  Its content is the single line `v1`;
  its modification time is the time it was written,
  because a
  handler has no nonce of its own to record and a non-property event carries none.
- `pass-budget`,
  a root-owned regular file of mode `0644` holding one line of four fields:
  the version word `v1`,
  the window's start in seconds from `CLOCK_BOOTTIME`,
  the milliseconds spent in that window,
  and the same moment as an ISO 8601 timestamp with its offset.
  It is replaced whole,
  never appended,
  and it is the record V13's first option needs:
  a pass's cost is charged to the window when the pass ends,
  because a pass's duration is known only then.
  When the next taker finds the window older than `P`,
  it starts a new window with nothing spent,
  and a charge that lands after the window expired opens a new window at the charge,
  so a pass longer than `P` is charged to a live window instead of a window the next taker would discard.
  The window is tumbling,
  V13's second choice inside its first option.

All three live under `/run`,
so a boot clears them,
and "a claim means nothing across a boot" ([spec][spec],
line
6264).
The shape is V6.

### Order inside a trigger

The dispatch runs the filter first,
because the filter opens nothing and can drop an event without the gate;
then the
compatibility gate,
which is the dispatch's own rule for any mode that will make a request
(`handler_mode_dispatch.rs`,
lines 131 to 146);
then the mode.
The order inside either mode is the propagation sibling's and not this design's
([sibling propagation][sibprop],
"Where it runs,
as whom,
and in what order",
lines 1334 to 1409 at `30274435f3`):
read the configuration and resolve the checkout,
its propagation step,
the start probe's record,
the pool and the checkout checks,
and then the sweep.
This design adds one thing to that order and states nothing else about it:
the cost decision below is taken when the sweep is entered,
after the sibling's checks,
and the run claim is taken there too,
so a refused prerequisite needs no claim to release.
A run held at the claim has paid the filter,
the gate,
the sibling's entry order and one failed claim attempt,
and nothing else;
a run held by the budget has paid those,
the claim and the budget read.

### What a taker does

A handler that may pass,
which is T1's or T4's delivery,
does this in order:

1.  Run the mode's entry order,
    which is the sibling's own:
    read the configuration and resolve the checkout,
    its propagation step,
    the start probe's record,
    and the pool and the checkout checks
    ([sibling propagation][sibprop],
    "Where it runs,
    as whom,
    and in what order",
    lines 1334 to 1409 at `30274435f3`).
    A prerequisite that refuses exits 2,
    and it records no start and removes no mark,
    because nothing will be enumerated and no claim was taken.
2.  Take the run claim with the repository's numbered claim (fact F22).
    If it is held,
    go to step 8.
3.  Read `pass-budget`,
    unless the event names a clone with an unanswered request.
    When the event names a clone,
    read that clone's intent and the receipt under its GUID first;
    a request without an answering receipt skips the rest of this step and passes,
    which is the project's third answer,
    and the read is what M3 measures.
    If the window it names is older than `P`,
    the window is reset to now with nothing spent.
    If what it records as spent has reached the budget,
    write `another-pass-is-due`,
    release the claim,
    print the held line naming the budget,
    the window and the time spent,
    and exit 2,
    because it did no work.
4.  Remove `another-pass-is-due` if it is there:
    this pass will enumerate.
5.  Enumerate:
    each clone's turn under its slot claim,
    and the receipts.
    When the pass ends,
    charge its own duration to `pass-budget`:
    if the window the file names expired while the pass ran,
    open a new window at the charge and put the duration in it,
    so a pass longer than `P` is charged to a live window;
    then replace the file whole,
    because only the ended pass knows its cost.
6.  Release the claim.
7.  Look for `another-pass-is-due`.
    If it is absent,
    report and exit 0 or 2 as the pass's own outcome.
    If it is there,
    take the claim once more and do steps 3 to 6 once more,
    without step 7;
    a mark that survives the second pass is
    left standing for the next taker,
    and the line says so.
8.  When step 2 found the claim held:
    write `another-pass-is-due`,
    then try the claim once more.
    If that second try
    succeeds,
    continue at step 3,
    and step 4 removes the mark the handler wrote.
    If it fails,
    print the held line and exit 2.
    The second try never waits (fact F22).

A readiness trigger,
T2 or T3's unit run,
does steps 1,
4 and 5:
it runs the entry order,
removes the mark and enumerates.
It never takes the run claim and never reads or writes `pass-budget`,
so step 5's charge is skipped,
and it is never held (V5's first option).
Its pass may overlap an event-driven pass,
which the per-slot
claim makes safe for each clone (fact F24),
at the cost of one extra pass per readiness transition.

### What the marks guarantee, and what they do not

The sequence holds in the interleavings traced for two and three handlers:
a mark written before the
running pass's release is seen by that pass's look,
and a mark written after the look finds the claim free for its own
second attempt.
Two things are weaker than an absolute rule would be:

- A mark written while the second pass runs is left standing for the next taker,
  and that is the cap in step 7,
  not a
  bug:
  with one nudge per command per gap,
  an event and its nudge can both be marked and left,
  and the wait's deadline is the only
  report.
  L3 states it.
- A mark written by a rate-refused handler,
  step 3,
  is followed by its release and nothing else;
  a later taker removes
  it at step 4,
  or it stands for the next trigger,
  which is the deferral the bound intends.

So the invariant this design claims is:
while a pass holds the claim,
a mark written before its release is seen by
that pass or by the next taker;
a mark written after the last look stands for the next trigger.
No nudge is lost in
the record;
only the pass that would answer it can wait.

### Why this needs no lock

No process waits on the run claim.
The claim is the repository's numbered one:
the kernel lets exactly one process
create the next generation,
a holder is named by process birth,
and a loser returns at once with the owner's identity
(fact F22).
The mark is one `O_EXCL` creation and one `unlink`;
presence is the whole meaning.
`pass-budget` is
replaced by one staging write and one rename (fact F23).
The device is lock-free in the repository's sense,
and its
unit is one pass of the route's single parent.
It does narrow the accepted "parallel handlers can sweep at once"
([proposal][proposal],
lines 561 to 562) for event-driven passes,
which V4 puts to the project.

### The device is cost, not safety

Every interleaving here is safe without it:
two passes take different slots' claims,
a clone whose slot is held is
left to its holder,
and a clone already attached keeps its receipt (facts F24 and F26).
The device decides how many
passes run.
Therefore an unreadable,
corrupt or missing record never refuses a pass:
a taker that cannot take the claim
passes anyway and prints that the cost device could not be read,
and one that cannot read `pass-budget` passes and
prints that the budget was not applied (V7's first option).
A pass that could not take the claim writes nothing to `pass-budget`,
so the record keeps one writer at a time (assumption A10).

### The wrapper's nudge policy

A waiting command writes no nudge of its own:
its first read is at its start (fact F20),
so a nudge then would be a second event inside the intent's own loss window,
and the project's chosen trigger does not need it.
When a wait ends unanswered it writes the record's entry,
and every later invocation that forwards a command writes the nudge
("The nudge on Git commands").
The nudge uses a fresh nonce,
so the write is a change
([spec][spec],
"The durable intent"),
and V3's options are which commands and how often.

## The rate rule's shape, with no number

### What is bounded

Bounded:
the passes that T1's deliveries and T4's nudges start,
in the unit V13 adopts,
a budget of pass time in a window,
with every number owed to step 22,
except the passes the project's third answer exempts.

The exception is the project's
([the decision record][answers],
"The cost rule holds only passes with no unanswered request"):
a pass whose event names a clone with no receipt answering its intent always runs,
and only a pass whose event names no such clone is held.
The taker reads the one clone the event names before it reads the budget
("The cost rule with a pending request"),
and an event that names no clone is decided by the budget alone.

Not bounded,
each for a stated reason:

- The handler invocations ZED forks.
  ZED forks before any of this route's code runs,
  so the bound cannot reach it;
  the 16-process ceiling and the queue bound it instead (facts F10 and F12).
- The account's ability to write the property,
  which the proposal says the handler cannot limit ([proposal][proposal],
  lines 575 to 578).
- Readiness passes,
  V5's first option,
  because a cost rule must not decide whether callers are admitted.
- A loop of nudges on a clone with an unanswered request,
  which the project's answer leaves unbounded and its threat model accepts
  ([the decision record][answers],
  the cost it states with this answer).

### What a waiter sees when no pass runs for it

A caller whose own request is unanswered is never held,
because its event names the clone and the exception runs it.
So the deadline report has no hold to explain,
and a wait ends at its deadline only when no pass has run for it at all.
What it prints is the request,
the last observation,
`doctor` and the journal (fact F20),
the entry it has just written,
and the trigger that entry feeds:
the next Git command in this repository.
V11 offers the alternatives for that sentence,
and the waiting path still gains no read.

### The measurement that sizes it

Step 22 owns the numbers ([plan][plan],
"Step 22",
lines 2340 to 2400).
This design fixes what the measurement must produce:

1.  Measure a pass's cost at two populations,
    the acceptance workload's clone count and one more than the child bound,
    interleaved in the same round with a control arm of the same binary that pays the handler's start,
    the filter and the gate and runs no pass,
    which is a parent with one child or a dropped event,
    and report the spread of repeated rounds before any difference is called one (rule `QNB`).
2.  Measure the passes one add starts and the history events one add produces,
    with the device on and off,
    because the add also snapshots and clones and those are in-scope events.
3.  Measure the one clone read the exception needs before the decision:
    the intent read and the receipt read for the event's clone,
    in the build that has the exception,
    against the same event handled by a build whose exception is compiled out.
    Count the acceptance workload's waits that end at the deadline in both builds.
    The positive control is one pass per hour over an event that names no unanswered clone,
    which must be held,
    while an event naming an unanswered clone must not be;
    a zero is not read as none without that control.
4.  Report what each candidate threshold would allow under the measured cost;
    the threshold is not asked here (V13's last line),
    and the project asks it after this measurement and after the cost branch merges.
5.  Measure lost-event behaviour and trigger recovery separately (M1 to M6,
    M8),
    because no bound on passes bounds recovery.

### Where this design stands on the threat model

The project's sentence is "We have to assume my machine isn't compromised.
sudo is passwordless here anyway.",
read as
"the account is trusted,
and root and the account are not a security boundary here" ([decision staging][staging],
lines 221 to 237).
Applying it:

- The rate rule is cost,
  not security:
  it keeps a buggy caller or a stray shell loop from spending root's time,
  and it
  does not imagine an attacker.
- The event filter is cost too,
  and not a boundary.
- The records' ownership and mode checks are accident protection,
  not a boundary between root and the account.
- Nothing here stops a trusted account from making root pass as often as the rule allows;
  only the rule does,
  and its
  number ships with a measurement.

A further boundary this design keeps:
nothing here asks the system service manager to change or signal anything.
The
only acting service-manager commands it names are the printed levers of T5 and the hook's two lines,
and no step or agent
runs them on this host.
Nothing here signals a process:
the killed-handler case is a guest arm that runs as root inside
the guest (test 10),
and the design needs no fact about what an unprivileged signal answers;
that question belongs to
the removal's holder designs.
The drop-in takes effect when the manager next reloads its configuration,
which step 19's
runbook owns and no agent performs here.

## What doctor reports for a clone whose last event was lost

`doctor` runs as the account,
reads the records read-only,
and runs nothing it prints ([plan][plan],
"Step 18",
line
2064).
For each managed clone it reports these readings,
each with its source,
and for a clone whose live intent has no
receipt answering it,
it says so in one line and prints the levers of T5.

- The clone's dataset name,
  slot and live intent,
  with the nonce and operation,
  read from the clone's own property and
  only when the property's source is the clone (fact F17).
  A user property has no write time an account can read,
  so
  the report prints no "written at" for it.
- The clone's mount state at its slot,
  present or absent,
  read from the live mount table the way the sweep reads it.
- The receipt under the clone's canonical GUID,
  if any:
  its nonce,
  operation,
  outcome,
  boot,
  failed step and time,
  and
  one of three verdicts,
  that it answers the live intent,
  that it answers an earlier request and is stale,
  or that
  there is no receipt (fact F25).
- This boot's readiness record,
  if any:
  its boot,
  pool and checkout states,
  its sweep state,
  and its time.
  A record
  that names another boot is reported as another boot's;
  an absent record is reported as absent,
  which is what a
  refused boot leaves (fact F16);
  what the wrapper says when no record exists is step 10's,
  and this design does not show it.
- The run claim's newest generation,
  and that its owner needs root to read,
  because claim records are mode `0600`
  (`single_owner_claim.rs`,
  line 252).
  The account can see that a pass is running,
  not who runs it.
- Whether `another-pass-is-due` is present,
  and its time.
- The pass budget:
  the window's start,
  the time spent in it,
  the budget and the window,
  or,
  before step 22 ships them,
  the sentence that the rule is not in force because its measurement
  is owed.
- Where each of the three cost records exists and where its creating step has not landed:
  step 18 builds these readings before step 19 creates the claim and the mark and before step 22 creates `pass-budget`,
  so the report prints `not built yet` for a record whose step has not run,
  rather than an error.
  The report cannot tell that sentence from a record that is absent in this boot,
  so it prints the same words for both;
  step 19 and step 22 replace the sentence with the real reading when they land,
  each reading the record its own step creates,
  and neither makes step 18 depend on it.
- The handler's last decisions,
  from V8's answer;
  a ZED-started handler's own lines are not in any journal today (fact
  F13),
  so today the report points at ZED's `Finished` lines for exit statuses and at nothing for the handler's lines.
- The service-manager facts:
  V9 decides whether the report prints the `systemctl` commands or reads their answers.
- The record of unanswered requests ("The record an unanswered request leaves"):
  its path from the repository's common Git directory,
  and each entry's clone,
  nonce and operation,
  the moment its wait ended,
  the moment it was last nudged,
  whether the clone still exists,
  and,
  for an entry past `A` or a clone that is gone,
  the one line that removes the entry.
  A record it cannot read is reported as unknown,
  not as absence.

The lost-event block,
with concrete values:

```text
clone zpcachyos/nosnap/monochromatic-git-worktree-clone-datasets/one
  intent   v1:6f1c0a9d8b7e5f4a3c2b1d0e9f8a7b6c:mount, source the clone
  mount    absent at /home/user/Monochromatic/.claude/worktrees/one
  receipt  1234567890123456789: none, so no pass has answered this request
  ready    boot 5d0f..., pool ready, checkout ready, sweep finished at 2026-11-22T17:29:58-05:00
  runs     no pass is running, as far as the account can read; a pass is due since 2026-11-22T17:30:05-05:00
  events   the last event for this clone may be lost; the next triggers are a ZED event or start, a provider start, the next Git command in this repository, or a lever you fire
  nudge    the repository's record names this clone, last nudged 2026-11-22T17:30:05-05:00; the next Git command here nudges it again after G
  fix      as root: systemctl --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service
  fix      as you: zfs set org.monochromatic:git-worktree-clone-mount-sweep-nonce=<32 fresh lowercase hex digits> <clone>
  journal  journalctl -u reconcile-git-worktree-clone-mounts.service ; journalctl -u zfs-zed.service
```

The lines reuse the route's wording for a stale result,
a claim and a readiness refusal.
The two `fix` lines are T5's
levers,
and `doctor` never runs them;
the account lever works only while ZED is alive.
The `nudge` line names the wrapper's own trigger,
which runs with the next Git command in this repository
and needs no person.

A record that cannot be read,
or
a check that cannot be proved,
is reported as unknown,
which is the plan's rule ([plan][plan],
"Step 18",
line 2090).

Whether a lost event is a finding or an error status is V10;
this design ranks it a finding,
and the exit-status rule
V10 quotes is this design's proposal,
because `SPEC.md`'s Doctor section describes only the hooks listing today
([spec][spec],
`### Doctor`,
lines 683 to 760).

## Which build step builds which part

Plan steps 1 to 8 are merged,
so nothing here reopens them;
the parts land on steps still ahead.

- Step 10,
  the add route:
  the record of unanswered requests,
  its entry written when a wait ends with no answer and removed when an answer arrives,
  the command-path check of "Which commands,
  and where",
  the nudge behind a compiled switch that is off,
  and the wait's cost text of V11.
  Step 22 sets `G` and `A` from M8 and turns the nudge on,
  which is the record's own condition for shipping it ("The measurement owed").
- Step 14,
  removal:
  the same record's entries for a removal's waits,
  and the entry's removal when a removal settles the clone,
  so a lost `unmount` event is recovered by the same triggers;
  the unmount stays step 9's ([plan][plan],
  "Step 14",
  lines 1738 to 1814).
- Step 18,
  `doctor`:
  the readings,
  the lost-event block,
  the printed levers,
  V8's decisions record,
  V9's service
  reading,
  V10's status,
  the three cost records reported as `not built yet` until their steps land,
  the record of unanswered requests and its entries,
  the correction of the four shipped sentences that name step 10 ([plan][plan],
  lines 2017
  to 2120).
- Step 19,
  the units and the zedlet:
  the event filter,
  the provider drop-in and `ExecReload=`,
  enabling
  `zfs-zed.service`,
  the boot unit as the plan describes it,
  and the merge device:
  the run claim,
  the mark,
  the one-extra-pass cap and the second claim attempt,
  as V4 and V12 adopt them with the project's veto open (V12's first option).
- Step 22,
  the measurements:
  `pass-budget` and the exception's one clone read with M3,
  the record's `G` and `A` with M8 and the switch turning the nudge on,
  the budget and its window in V13's unit,
  and their numbers with the measurement beside them.

The device and the mark land in step 19 because step 19 enables ZED,
and without them every event starts its own pass;
the record's writer and the command-path check land in step 10 because that is where the wrapper's first wait
and its forwarded path are built,
and the gap,
the stale age and the numbers land in step 22 because the record itself requires the nudge's
measurement before the nudge ships,
and rule `LQM` requires the budget's measurement before it ships (V12).

## The tests a build must have

Each test names the race or the case it forces and how it forces it without timing luck.
Tests marked guest run only in
the throwaway guest;
the repository's guest gate is `CLI_GIT_ZFS_SWEEP_GUEST`
(`binary_zfs_sweep_guest_tests.rs`,
lines 1 to 40) and its driver is
`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`.
Tests marked host can run here with fixtures,
the injected gate (`run_behind`) and the claim's existing seam (`take_claim_seamed`,
fact F22).

1.  Filter accepts and drops,
    host:
    the default invocation with the event's pool and dataset set for the parent,
    a
    clone,
    a clone's snapshot,
    a dataset outside the parent,
    another pool,
    both absent,
    and no resolvable parent.
    Each
    asserts the decision and,
    for a drop,
    that `/dev/zfs` was never opened and no claim was taken.
2.  A nudge during a pass,
    guest:
    start a pass on a parent with enough children that it lasts,
    wait for the run claim's
    newest generation to appear,
    write a nudge,
    and assert the mark was removed by the second pass.
    The forcing is the
    record's appearance;
    the clone population is the measurement's (M4),
    not a bet,
    and the arm asserts the nudge was written while the claim was held,
    so a pass too short to be caught fails rather than passes.
3.  The device's contended paths,
    host:
    the test holds the run claim with its own owner,
    starts two default
    invocations,
    and asserts both printed the held line,
    both wrote the mark,
    neither read a clone,
    and both exited
    2,
    with the gate injected as compatible;
    it then releases the claim and starts one more invocation,
    which must remove the mark and pass.
    This forces the
    contention with the test's own claim (race R3).
4.  A mark written after the last look,
    host:
    the device's module gains a seam between the release and the look,
    called
    as `take_claim_seamed` is,
    and the test writes the mark there;
    it asserts the second attempt passed,
    or the next
    invocation removed the mark on entry.
    No window inside the handler is targeted from outside (race R4).
5.  The rule holds a pass,
    host with a fixture record and an injected clock:
    a `pass-budget` whose window is open and whose spent time has reached the budget must produce the held line,
    no receipt and the mark;
    one whose spent time is below the budget must pass;
    and one whose window is older than `P` must reset and pass.
6.  The rule's positive control repeats test 5 with the smallest admissible budget and asserts a wait's deadline report
    carries V11's sentence,
    so the arm is proved able to show a hold (M3's control).
7.  Unreadable cost records,
    host:
    the claim folder unreadable,
    `pass-budget` corrupt,
    and the mark a directory.
    Each
    must still pass,
    print which record could not be read,
    and leave the clones' state alone (V7's first option).
8.  A lost event and its recovery per trigger,
    guest,
    in separate arms,
    each naming the state it starts from and the verb it runs:
    stop ZED,
    write an intent,
    assert no receipt;
    then fire exactly one thing per arm:
    a ZED start (`systemctl restart zfs-zed.service`),
    a Git command in the repository with ZED running,
    which writes the record's nudge,
    the boot unit made failed by a boot with the key unloaded and then `systemctl start`,
    the provider by `systemctl restart zfs-mount.service`,
    and the project's lever on an active unit (`systemctl reload-or-restart reconcile-...`),
    and assert a receipt under the intent's nonce appears in each arm.
    A `start` of an active unit is not an arm,
    because it runs no job (fact F15).
    The ZED-start arm proves the replay;
    the nudge arm proves the delivery (race R6).
9.  A drained queue,
    guest:
    with ZED stopped,
    write more property changes than the kernel queue holds,
    start ZED,
    and
    assert that ZED logged no `Missed` line,
    that the first intents have no receipt,
    and that `zpool events` shows the
    property change's event id no longer queued.
    The evidence is the event id,
    not a `Missed` line,
    because a drain with
    no reader is silent (fact F12).
10. A handler killed mid-pass,
    guest:
    the arm runs as root in the guest and kills the handler's process group during a
    pass,
    so no unprivileged signal is attempted anywhere;
    it then fires one trigger and asserts the next taker took
    the next generation,
    recognised a standing mount,
    and answered the receipt under the live intent's nonce
    (race R8).
11. No feedback,
    guest:
    a logging zedlet beside the handler,
    one pass,
    and an assertion that the pass produced no
    `history_event`,
    which proves the loop the design forbids cannot start (assumption A5).
12. The provider hook's unit states,
    guest,
    each arm naming its starting state and its verb:
    a boot with the hook and the boot unit;
    the boot unit active and the provider restarted,
    which asserts the restart's stop pass ran and the hook's request did not add a second stop;
    the boot unit failed and the provider restarted,
    which asserts the hook started the unit;
    the boot unit stopped and the provider started,
    which asserts the hook started the unit;
    and the project's lever on an active unit (`systemctl reload-or-restart`),
    which is the lever's arm and not the hook's,
    then the same with `ExecReload=` absent,
    which must show a restart and its stop pass.
    Each asserts the provider stays active,
    the reconcile ran in the boot unit's journal,
    and no clone was unmounted by a reload (the project's answer;
    facts F17 and F15).
13. The provider's late readiness,
    guest:
    boot with the checkout's key unloaded so the boot's reconcile refuses and
    publishes no record,
    assert a caller is refused and that no readiness record for this boot exists,
    load the key,
    `systemctl restart zfs-mount.service`,
    and assert the hook's job published readiness and attached the clone without another add (the
    proposal's unrun item,
    [proposal][proposal],
    lines 2024 to 2030).
    A `start` would run no job,
    because the provider is active:
    `zfs mount -a` skipped the encrypted checkout
    and exited zero (fact F37).
14. `doctor`'s lost-event report,
    host over planted records:
    one clone with a matching receipt,
    one with a stale
    receipt,
    one with none,
    one whose record belongs to another boot,
    and no record at all,
    each with and without an entry in the unanswered-request record.
    Assert the exact lines,
    the `nudge` line,
    the two printed levers,
    and that nothing was run.
15. The command's record check,
    host with a fixture repository and an injected device:
    a repository with no record opens nothing and prints nothing;
    a record with one entry opens the device once and writes one nudge;
    a refused device prints one line and leaves the command's exit status unchanged;
    a malformed or over-page record is reported and no write happens.
    The positive control is the same command with a well-formed record,
    which must open the device,
    so a zero from the first arm is not read as none (V3's first option).
16. The command burst,
    host with an injected clock and a record:
    three commands inside `G` write one nudge and move `nudged-at` once;
    the first command after `G` writes another.
    Forcing is the injected clock,
    not a sleep (race R9).
17. The record's write and removal on a wait,
    host:
    a wait that ends unanswered writes its entry under the repository's common directory;
    a wait answered removes it;
    an interrupted wait writes it,
    and none of the three changes the command's exit status.
18. A stale entry and a missing record,
    host:
    an entry past `A` is removed without a device open;
    an entry whose clone is gone is reported by `doctor` and the line it prints removes it;
    and a repository with no record but an unanswered intent is reported by `doctor`,
    which names the other triggers.
19. The rule's exception,
    host with a fixture budget and a planted clone:
    with the budget spent,
    an event naming a clone with no answering receipt passes and files its receipt,
    while an event naming no clone is held with the held line;
    the positive control repeats with the smallest admissible budget (M3's control).
20. The Git command recovers a request with nobody waiting,
    guest:
    stop ZED,
    write an intent,
    let its wait end unanswered so the record has the entry,
    then run one forwarded command in the repository and assert a receipt under the intent's nonce
    and one history event for the nudge.
    Control:
    the same run with the record removed,
    which writes nothing and leaves the clone waiting (race R6).

Tests 1,
3,
4,
5,
6,
7,
14,
15,
16,
17,
18 and 19 run on the host.
Tests 2,
8,
9,
10,
11,
12,
13 and 20 need the guest,
because each needs a
real pool,
a real ZED,
real mounts or a real systemd.

## Measurements owed

Each names its probe,
its control and what it decides.
None ran.

- M1,
  lost-event behaviour per trigger.
  Probe:
  in the guest,
  stop `zfs-zed.service`,
  write one intent,
  then fire one
  trigger per arm (a ZED start,
  a nudge with ZED running,
  a `start` of the boot unit after a refused boot left it failed,
  a provider `restart`,
  and the project's `reload-or-restart` lever on the active unit) and record
  the time from the trigger to the receipt.
  Each arm starts from the state its command needs,
  as test 8 lists.
  Control:
  the same intent with ZED running and no nudge,
  which gives the
  delivery's own latency.
  Decides:
  each trigger's recovery and its delay.
- M2,
  a drained queue.
  Probe:
  with ZED stopped,
  write more property changes than the queue holds,
  start ZED,
  and record
  whether ZED logged `Missed`,
  which intents still have no receipt,
  and whether `zpool events` still lists the property
  change's event id.
  Control:
  the same writes with ZED running,
  where a `Missed` line can appear.
  Decides:
  that a
  stopped-ZED drain is silent and that the event id is the evidence.
- M3,
  the rule's caller cost and its exception.
  Probe:
  the acceptance workload with a candidate threshold in V13's unit,
  counting waits that end at the deadline;
  the one clone read the exception needs before the decision
  (the intent read and the receipt read for the event's clone),
  in the build with the exception against the build with it compiled out;
  positive control the smallest admissible budget,
  which must hold an event that names no unanswered clone and must not hold one that does;
  control with the rule absent.
  Decides:
  the numbers and whether the exception is adopted,
  as the record requires.
- M4,
  the merge device's saving.
  Probe:
  passes started and history events produced per add,
  with the device on and
  off,
  and the pass's wall time at the acceptance population.
  Decides:
  whether the device earns its code,
  and the
  counts in "Counts and numbers this design does not ship".
- M5,
  the boot's critical path.
  Probe:
  the time from the provider's `ExecStart` to `local-fs.target` reaching active,
  with the hook and with it absent,
  interleaved over repeated boots;
  and,
  for the direct hook the project did not take,
  the time to the boot
  unit's exit.
  Decides:
  whether any provider hook is acceptable on the critical path.
- M6,
  the event filter's saving.
  Probe:
  on the guest's fixture,
  with the test's own property writes on the checkout and
  the home datasets at sanoid's cadence,
  count handler starts and passes with the filter and with it compiled out.
  Decides:
  whether the filter's cost is worth a number in the specification.
- M8,
  what a Git command pays for the record.
  Probe:
  the same command on this machine in three cases:
  a repository with no record,
  one with a record and a request still unanswered,
  and one with a stale record,
  each alternated with the same command on `main`'s build in the same round,
  repeated for the spread (rule `QNB`),
  and each reported as a ratio to `main`'s build.
  Positive control a planted record,
  which must make the record path run.
  Decides:
  that the command trigger ships at all,
  and the gap `G` and the stale age `A`,
  from the receipt latency the record's nudge is answered with.

## Counts and numbers this design does not ship

For each count or delay:
the step that builds the code,
what that step ships for the value before step 22 measures it,
and what a person sees meanwhile.
No number is invented to fill a gap (rule `LQM`).

- The pass's child bound of 4096.
  Already shipped with the merged measurement named in fact F28,
  and unchanged here.
- One extra pass per admitted run,
  and one second claim attempt after a mark.
  Built in step 19.
  It ships as the shape V4 and V12 adopt,
  a count of one each,
  with the project's veto open:
  the decision beside those values is the project's answer to V4 and V12,
  not a measurement.
  A person sees a second pass only when a mark was pending,
  and the line says so.
  M4 reports whether the device earns that cost,
  and an answer that it does not reopens V12.
- The nudge on Git commands:
  its record's one-page bound,
  the gap `G` and the stale age `A`.
  Step 10 builds the record's writer,
  its removal and the command-path check,
  with the nudge behind a compiled switch that is off;
  step 22 ships `G` and `A` with M8 beside them and turns the nudge on,
  which the record itself requires before the nudge ships.
  Until then a person sees one event per add (the intent),
  no nudge,
  and the timeout text and the levers when a wait times out;
  T4 is the trigger that arrives with the measurement.
- The budget of pass time and its window,
  in V13's unit.
  Built in step 22,
  which ships both with M3 beside them.
  Until then the rule is not in force and `doctor` says so.

## Facts

Each fact is one thing read or measured,
with its source.
Nothing here is new measurement on this host.

- F1.
  This host runs systemd 262 (262.2-1-arch),
  kernel `7.3.5-1-cachyos`,
  OpenZFS `zfs-2.4.4-1` and Git `2.56.1`,
  and
  the installed wrapper reports `monochromatic-git-policy-cli 0.6.1-alpha.44` (`systemctl --no-ask-password --version`,
  `uname -r`,
  `zfs version`,
  `git --version`,
  2026-11-22).
- F2.
  `zfs-zed.service` is loaded,
  disabled and inactive,
  preset `enabled`;
  its unit runs `/usr/bin/zed -F` with
  `Restart=always` (`systemctl --no-ask-password status zfs-zed.service`,
  `/usr/lib/systemd/system/zfs-zed.service`).
- F3.
  `zfs-mount.service` is enabled and active (exited),
  `Type=oneshot`,
  `RemainAfterExit=yes`,
  `ExecStart=/usr/bin/zfs mount -a`,
  `After=zfs-import.target`,
  `Before=local-fs.target`,
  `WantedBy=zfs.target`
  (`systemctl --no-ask-password status zfs-mount.service`,
  `/usr/lib/systemd/system/zfs-mount.service`).
- F4.
  `zfs-load-key.service` is masked,
  `/etc/default/zfs` sets `ZFS_LOAD_KEY='yes'`,
  and the pool and the checkout's
  ancestor report `keystatus available` with `encryption aes-256-gcm` (`systemctl --no-ask-password is-enabled`,
  `/etc/default/zfs`,
  `zfs get encryption,keystatus`).
- F5.
  No managed dataset exists:
  `zfs list -d 2` shows the pool's `nosnap` children and no `monochromatic-git-*` name
  (`zfs list -o name,mountpoint,mounted,canmount -d 2`).
- F6.
  `/home/user/Monochromatic` is on `zpcachyos/ROOT/cos/home`,
  an inherited dataset,
  not a mount of its own
  (`findmnt --target /home/user/Monochromatic`,
  `findmnt --target /home/user`).
- F7.
  None of `/usr/libexec/monochromatic-git-policy-cli`,
  `/etc/monochromatic-git-policy-cli`,
  `/var/lib/monochromatic-git-policy-cli`,
  `/run/git-worktree-clone-mount-results`,
  `/run/git-worktree-clone-mount-readiness`,
  `/run/git-worktree-clone-mount-slot-claims` or `/run/zed.state` exists
  (`ls -d` on each,
  2026-11-22).
- F8.
  `/etc/zfs/zed.d` holds the distribution's zedlets,
  including `all-syslog.sh` and
  `history_event-zfs-list-cacher.sh`,
  and no handler zedlet (`ls /etc/zfs/zed.d`).
- F9.
  This host's kernel queue bound is 512 (`cat /sys/module/zfs/parameters/zfs_zevent_len_max`).
- F10.
  ZED runs at most 16 zedlet processes at once and blocks before forking while that many are alive
  (`cmd/zed/zed_conf.c`,
  line 51;
  `cmd/zed/zed_exec.c`,
  lines 130 to 140 at `zfs-2.4.4`).
- F11.
  ZED's state file is `RUNSTATEDIR "/zed.state"` (`cmd/zed/zed.h`,
  line 27),
  and this host's installed binary embeds
  `/var/run/zed.state` (`strings /usr/bin/zed`),
  which links to `/run` and so is per boot.
- F12.
  The kernel's event list holds 512 events by default,
  inserts at the head,
  drains the tail (the oldest),
  credits a
  drop only to a reader holding a cursor on the drained event,
  and starts a fresh reader at the tail with no count
  (`module/zfs/fm.c`,
  lines 72,
  143 to 156,
  180 to 192 and 316 to 359 at `zfs-2.4.4`).
  So a drain while ZED holds no
  cursor is silent.
- F13.
  In the foreground,
  ZED points each zedlet's standard input,
  output and error at `/dev/null`,
  so a ZED-started
  handler's own lines reach nobody,
  and the journal holds only ZED's `Invoking` and `Finished` lines with the exit
  status (`cmd/zed/zed_exec.c`,
  lines 161 to 166,
  189 and 240 to 261 at `zfs-2.4.4`).
- F14.
  An attach is admitted only on a readiness record naming this boot,
  pool and checkout with both facts ready,
  and
  `unknown` admits nothing (`clone_mount_readiness_admission.rs`,
  lines 100 to 150;
  [spec][spec],
  lines 6184 to 6228).
- F15.
  A start job on an active unit returns `-EALREADY` (`src/core/unit.c`,
  lines 1955 to 1962 at `v262`);
  a oneshot that
  failed can be started again and re-runs its `ExecStartPre=` and `ExecStart=`;
  a reload needs an `ExecReload=` and is admitted only while the unit is active,
  and is refused otherwise with "Unit cannot be reloaded because it is inactive"
  (`src/core/unit.c`,
  lines 2154 to 2156;
  `src/core/service.c`,
  lines 3777 to 3785 at `v262`);
  and `reload-or-restart`
  reloads an active unit,
  starts one that is not active,
  and becomes a restart for an active unit that cannot reload
  (`systemctl(1)`,
  lines 530 to 537;
  `src/core/job.c`,
  lines 500 to 510;
  `src/core/unit.c`,
  lines 7179 to 7197 at `v262`).
- F16.
  `boot-reconcile` publishes readiness only after `complete()` returns facts that passed `admit()`,
  and its record
  carries `sweep_nonce: None`
  (`boot_reconcile_mode.rs`,
  lines 122 to 181;
  `sweep_preconditions.rs`,
  lines 535 to 551).
  So a boot whose checkout is
  not ready has no record for this boot,
  not a `not-ready` one.
- F17.
  `Requires=` with `After=` means the boot unit "will be stopped (or restarted) if one of the other units is explicitly
  stopped (or restarted)" (`systemd.unit(5)`,
  `Requires=`,
  lines 681 to 694 at `v262`;
  [proposal][proposal],
  line 525),
  and the propagation is exact:
  a restart closes to a try-restart on the dependents
  (`src/core/transaction.c`,
  lines 1199 to 1214),
  and a try-restart collapses to nothing for a unit that is not active
  (`src/core/job.c`,
  lines 486 to 500 at `v262`).
  So a provider restart restarts an active boot unit,
  running its `ExecStopPost=` `shutdown-unmount` first,
  and leaves a stopped or failed one alone.
- F18.
  The handler runs in the host mount namespace ([proposal][proposal],
  lines 483 to 484 and 513),
  and the probe is
  `ExecStartPre=` of the boot unit,
  which is the service manager's own child
  ([start probe design][probedesign],
  lines 1390 to 1401).
  A person's shell is not that namespace.
- F19.
  The sweep reads the probe record first,
  and a record that is absent,
  unreadable,
  names another boot or carries any
  verdict but `passed` refuses the run (`sweep_preconditions.rs`,
  lines 281 to 296;
  [spec][spec],
  lines 6272 to 6300).
- F20.
  The wrapper waits for a matching receipt for a provisional 30,000 milliseconds,
  polling every 50 milliseconds,
  and
  reads before it sleeps,
  so a fresh request's first read is at its start;
  its timeout text names the request,
  the last
  observation,
  `doctor` and the journal (`mount_request_wait.rs`,
  lines 59,
  68,
  326 to 359 and 290 to 300).
- F21.
  A mode's refusal exits 2 and only completed work exits 0 (`handler_mode_dispatch.rs`,
  lines 11 to 14 and 35 to 40).
- F22.
  The repository's numbered claim keeps only names of the form `<prefix>.<digits>` in its folder,
  writes its records
  mode `0600`,
  names an owner by process birth,
  never waits,
  and has a seam for tests
  (`single_owner_claim.rs`,
  lines 208 to 234,
  252 and 355 to 370).
- F23.
  A receipt and the readiness record are published by a staging write,
  a sync,
  a rename and a directory sync
  ([spec][spec],
  lines 6131 to 6183 and 6184 to 6228).
- F24.
  A pass enumerates the parent's immediate children,
  takes each slot's claim,
  re-reads the intent under it,
  leaves a
  held slot to its holder,
  and the holder reads the intent again after releasing
  (`sweep_one_clone.rs`,
  lines 1 to 55;
  [spec][spec],
  lines 6272 to 6387).
- F25.
  A receipt carries the request's nonce and a stale result is never the answer ([spec][spec],
  lines 6131 to 6183;
  `mount_request_wait.rs`,
  lines 280 to 300).
- F26.
  A clone whose mount already stands at its slot keeps its `done` receipt rather than having a refusal overwrite it,
  and
  a turn files its answer before reading the intent once more (`sweep_one_clone.rs`,
  lines 44 to 50 and 440 to 450).
- F27.
  The shipped readiness refusal already names the trigger set:
  "let the next trigger (a ZED event,
  the checkout
  provider's post-key-load hook,
  or the caller's own sweep nudge) run the sweep"
  (`clone_mount_readiness_admission.rs`,
  line 100).
- F28.
  `main`'s child bound is 4096,
  with the merged measurement beside it:
  one sweep over 4,097 children visited the first
  4,096 in 14,183 milliseconds,
  and one handler run over a single clone took 99 milliseconds
  (`sweep_dataset_reader.rs`,
  lines 26 to 58).
  The branch
  `fix/git-policy-cli-zfs-step-8-recheck-findings-and-sweep-cost` at `d97a683346` replaces those figures and is unmerged
  and unreviewed (F34).
- F29.
  Four shipped refusals say `doctor` reports a receipt,
  a readiness record and a claim "once plan step 10 adds it"
  (`mount_request_wait.rs`,
  line 298;
  `clone_mount_readiness_admission.rs`,
  line 100;
  `clone_mount_receipts_folder_refusals.rs`,
  line 185;
  `per_slot_root_claim.rs`,
  line 151),
  and the plan builds
  `doctor` in step 18 ([plan][plan],
  lines 2017 to 2120).
- F30.
  The step 7 measurement puts a test zedlet's arrival 10,218 to 10,581 microseconds after the property change and the
  product wait's answer 36 to 64 microseconds after a receipt that was already on disk
  (`zfs-mount-receipt-deadline-measurement-2026-11-21.md`,
  "The rounds",
  lines 105 to 121).
- F31.
  `zfs-load-key.service` is masked on this host while the pool's keys are available (F4).
- F32.
  `ExecStartPost=` runs after a oneshot's `ExecStart=` succeeds,
  an unprefixed failure fails the unit,
  its execution
  counts for ordering,
  and a oneshot's start timeout is disabled by default;
  `OnSuccess=` activates on entering
  `inactive`;
  and a mount unit carries no `Exec` list
  (`systemd.service(5)`,
  lines 427 to 437,
  454 to 465 and 682 to 692;
  `systemd.unit(5)`,
  lines 878 to 882;
  `systemd.mount(5)`,
  lines 40 to 66,
  all at `v262`).
- F33.
  This design's host probes are reads only:
  `systemctl --no-ask-password --version`,
  `status` and `is-enabled`,
  `journalctl` was not used,
  `strings /usr/bin/zed`,
  and file reads under `/usr/lib/systemd/system`,
  `/etc/zfs`,
  `/etc/default/zfs` and `/sys/module/zfs/parameters`.
  No acting verb,
  no D-Bus call,
  no `pkexec`,
  and no signal to any
  process was issued.
- F34.
  The branch `fix/git-policy-cli-zfs-step-8-recheck-findings-and-sweep-cost` at `d97a683346`
  (2026-11-22T17:32:12-05:00) replaces the cost comment in `sweep_dataset_reader.rs` with cold and steady sweeps per
  population (64 children 1,254 milliseconds cold and 303 to 402 steady;
  4,096 children 174,754 cold and 20,092 to
  22,288 steady) and says the earlier figure counted children the origin check refuses.
  It is unmerged,
  unreviewed,
  and
  cited here only as existing.
- F35.
  The step 7 measurement observed `ZEVENT_POOL`,
  `ZEVENT_HISTORY_DSNAME` and the property and value in
  `ZEVENT_HISTORY_INTERNAL_STR` for a `zfs set`
  (`zfs-mount-receipt-deadline-measurement-2026-11-21.md`,
  "Where",
  lines 60 to 66).
- F36.
  `SPEC.md`'s `### Doctor` section describes the traditional-hooks listing only ([spec][spec],
  lines 683 to 760).
- F37.
  `zfs mount -a` does not fail on an encrypted dataset whose key is unavailable:
  the mount callback returns zero for such a dataset without attempting the mount
  (`lib/libzfs/libzfs_mount.c`,
  lines 1163 to 1179 at `zfs-2.4.4`),
  so `zfs-mount.service` reaches `active (exited)` in the late-key flow.
- F38.
  The wrapper's forwarded path is `run_wrapped_command`
  (`src/native/invocation/wrapped_command.rs`,
  line 428),
  and it already asks real Git for the invocation's identity there,
  whose `WorktreeIdentity` carries the canonical `common_dir` for every shape that is a repository
  (`src/native/worktree/worktree_identity.rs`,
  `WorktreeIdentity`).
  So the record's path is derived from a value the command already holds,
  with no extra Git call.
- F39.
  A cheap filesystem check already runs before every forwarded command:
  the worktree pending check reads one Git pointer file with a bound and follows no link,
  and the specification says why it may not block
  ([spec][spec],
  "The records an add writes for itself"):
  "the pending check runs before every forwarded command".
- F40.
  The per-add claim record has a named home in the account-owned Git common directory
  ([proposal][proposal],
  "Add and Git registration",
  step 2,
  line 373,
  and "Open items the decisions leave",
  line 1742),
  so the unanswered-request record sits beside a record the route already owns.
- F41.
  The start-time gate reads each small file it depends on with one bound per file
  ([spec][spec],
  "The start-time ZFS compatibility gate",
  lines 20490 to 20500),
  which is the bound the unanswered-request record's one-page read takes.
- F42.
  The propagation sibling's question 2 option 1 is a `checkout-propagation.service` upheld by a drop-in on the
  checkout's mount unit,
  whose `ExecStart=` is the same `systemctl --no-block --no-ask-password reload-or-restart` request this
  design's hook sends
  ([sibling propagation][sibprop],
  "The propagation service and its drop-in",
  lines 2464 to 2500,
  and its question 2's option 1,
  lines 2866 to 2886,
  read at `30274435f3`).

## Assumptions

- A1.
  The parent the pass sweeps and the parent the filter accepts are the same dataset,
  because both come from one
  configuration read with one fallback.
- A2.
  The handler is started by ZED only as root,
  so the filter's environment cannot be chosen by another account.
- A3.
  ZED sets the event variables a filter reads,
  as the installed list-cacher zedlet relies on for the same class (F35).
- A4.
  A pass's own work writes no property and mounts nothing that posts a ZFS history event.
  That is an inference and is an assumption here.
  Test 11 checks it.
- A5.
  A hooked boot's readiness triggers are at most the hook's job request and the boot unit's own start,
  because each of
  those units acts once in that boot.
- A6.
  The wrapper can write a nudge on a clone it waits for under the agreed `mount,userprop` grant,
  and not on the parent
  ([proposal][proposal],
  lines 573 to 580).
- A7.
  A person's `sudo` needs no passphrase on this machine,
  as the project's threat model states;
  if it does,
  the printed
  lever asks that person's shell,
  and the wrapper never does.
- A8.
  The guest's kernel and OpenZFS behave like the host's for the queue and ZED,
  because both are `zfs-2.4.4` and the
  guest's kernel is one the module supports;
  the guest's kernel is not this host's `7.3.5-1`,
  which is why M2 belongs
  to the guest.
- A9.
  A `systemctl --no-block` job request from a unit's `ExecStartPost=` is sound;
  it is unprobed here ([the decision record][answers],
  the cost it states with the hook's answer).
- A10.
  `pass-budget` has one writer at a time,
  because only an event-driven pass writes it,
  every such pass holds the run claim,
  and a pass that cannot take the claim writes nothing there (V7's first option).
- A11.
  The wrapper can write and rewrite the unanswered-request record in the account-owned Git common directory,
  and read it back,
  because it runs as the account and the directory is the account's.
- A12.
  The identity the forwarded path resolves carries the canonical common directory for every repository shape
  it forwards in (fact F38);
  where it does not,
  the command finds no record and writes no nudge,
  and the other triggers recover (L18).

## Limitations

- L1.
  The no-waiter case has no time bound of its own:
  the next trigger is the next Git command in the repository,
  and with no command it is the next boot;
  no timer exists by the project's decision,
  and the project accepted the wait a repository nobody touches leaves
  ([the decision record][answers]).
- L2.
  A caller's own pass is never held (the project's third answer),
  so a caller times out when no pass runs for it at all,
  and its request then waits for its record's next command or another trigger.
- L3.
  A mark written after the last look,
  including one written while the second pass runs,
  stands for the next trigger;
  the
  record never loses it,
  but the wait is not answered sooner.
- L4.
  The rule bounds passes and not handler invocations:
  a loop of property writes still costs one fork and one claim
  attempt per write,
  and ZED's 16-process ceiling is what bounds that.
- L5.
  The event filter gives up the accidental recovery an out-of-scope event would have provided,
  and the no-waiter case's
  next trigger can be later because of it.
- L6.
  The provider hook cannot see a checkout mounted outside a unit,
  so that case waits for T2 or T5.
- L7.
  A direct hook at boot is a refusal,
  not a pass,
  and a hooked boot spends two readiness actions only in the sense that
  the first does nothing;
  the hook spends one action and one unit job.
- L8.
  A readiness pass can overlap an event-driven pass,
  so a boot or provider transition can coincide with a second pass
  over the same clones;
  the per-slot claim keeps it safe,
  at one extra pass.
- L9.
  A drain while ZED holds no cursor is invisible:
  no `Missed` line exists,
  and only the unanswered intent and the event
  ids `zpool events` still lists show it.
- L10.
  `doctor` reads no service manager today,
  so a disabled ZED is visible only through the commands it prints unless the
  project reads them;
  V9 offers the alternative.
- L11.
  This design's records are per boot,
  so no report can name a loss from a previous boot beyond the live intent and its
  unanswered state.
- L12.
  Every number here is either the merged measurement F28 names,
  which an unmerged branch is changing,
  or is owed.
  No
  number ships by this document.
- L13.
  The account lever works only while ZED is alive,
  and the root lever's `reload-or-restart` needs the `ExecReload=` the project's answer takes:
  with it an active unit reloads and nothing unmounts,
  and the record now fixes that line for step 19 to install.
- L14.
  The route-wide claim narrows the accepted parallel-pass sentence ([proposal][proposal],
  lines 561 to 562),
  and the project decides in V4.
- L15.
  The pass budget is charged when a pass ends,
  so one pass can overshoot it,
  and a pass killed before it ends is charged nothing (V13's cons).
  A charge that lands after the window expired opens a new window at the charge,
  so the overshoot is not discarded with the old window and `P` need not exceed the longest pass;
  the price is that such a pass starts the next window with its whole duration spent.
  A tumbling window can also admit a burst at the end of one window and the start of the next,
  up to twice the budget inside a span shorter than `P`.
- L16.
  Every invocation that forwards to Git pays one `stat` of the record's path,
  even a repository that will never hold a clone;
  M8 measures it,
  and it is the price the project accepted for the command trigger.
- L17.
  A stale entry costs one device open and one failed property write per command until `A` passes,
  the next wait on that clone answers,
  the removal settles the clone,
  or `doctor` prints the line that removes it.
- L18.
  Where the forwarded path does not know the repository's common directory,
  the command writes no nudge,
  and the request waits for T1,
  T2,
  T3 or T5;
  the same is true when the record is missing though a request is unanswered (fact F38,
  A12).
- L19.
  The record lives in the account's tree and the account is trusted,
  so it can be removed or edited;
  removing it costs the trigger and never the request,
  which its durable intent keeps.
- L20.
  The nudge is best-effort:
  a refused device,
  a refused write or a record that cannot be rewritten prints one line and never changes the
  command's exit status,
  and no nudge waits for a receipt.

## Choices this design marks for the project's veto

Each entry gives the options,
a concrete example with an input and the result a person would see,
pros and cons,
and a
ranking with the reason for each adjacent pair.
The first option is what the rest of the document describes.

### V1. The ZED handler stays an asynchronous zedlet

- A,
  asynchronous,
  adopted:
  the installed name keeps no `-sync-` segment.
  Example:
  a sanoid snapshot of a clone writes a history event;
  ZED forks the handler and reads the next event at once,
  and the handler prints nothing a journal holds,
  and exits after its pass.
  Pro:
  ZED's event loop keeps reading,
  so a pass cannot lengthen the queue's exposure through ZED's own loop,
  and two
  triggers become one pass through the mark.
  Con:
  up to 16 handlers can be alive at once,
  and ZED's 16-process ceiling
  is shared with the distribution's zedlets,
  so a pass can wait for a slot there (fact F10).
- B,
  `history_event-sync-mount_git_worktree_clone_datasets`:
  ZED waits for all its children around that fork.
  Example:
  the same snapshot event;
  ZED waits for the pass to finish before it reads the next event,
  so a long pass is a
  long pause in ZED's reading.
  Pro:
  at most one handler at a time,
  so no concurrent passes.
  Con:
  the pass's duration is added to the time ZED is not
  reading,
  which makes a drain more likely under a burst,
  and `wait_for_children` waits for every child,
  including
  unrelated notification zedlets.
- Ranking:
  A > B,
  because the loss this design is about is a drain,
  and B lengthens ZED's exposure to make cheaper a
  problem the merge device already solves.

### V2. The event filter's scope, and what an unresolved parent does

- A,
  adopted:
  accept an event whose pool is the parent's and whose dataset,
  with any `@snapshot` suffix removed,
  is the parent or below it;
  drop everything else with one line and no device open.
  An unresolved parent passes the event on,
  so the handler's own checks run and refuse or pass on live facts;
  it never fails closed.
  Example:
  a sanoid snapshot of `zpcachyos/ROOT/cos/home` while clone `one` has an unanswered request is dropped;
  a `zfs set` on `one` is accepted.
  Pro:
  the bound stays for real requests,
  and a pass is not spent on another dataset's event.
  Con:
  it gives up the accidental recovery an out-of-scope event would have made (L5),
  and an unresolved parent costs one gate run.
- B,
  the same,
  and the checkout dataset too:
  every history event of `monochromatic-git-checkout` passes as well.
  Example:
  a sanoid snapshot of the checkout after a replacement mount passes,
  the pass runs the propagation sibling's step and repairs the checkout,
  and a clone with an unanswered request is attached by the same pass.
  Pro:
  it gives the propagation sibling's trigger a channel through the route's own zedlet,
  which is what its question 2 option 2 asks for;
  and under the project's third answer a checkout event names no clone,
  so it is the first kind the budget holds and spends only spare budget.
  Con:
  every checkout event,
  at sanoid's cadence,
  forks a handler and can hold a pass;
  the widening is the sibling's decision to ask for,
  and this entry states the cost so its question can refer to it.
- C,
  filter by pool only:
  every `history_event` of the parent's pool passes.
  Example:
  the same sanoid snapshot passes and the pass attaches `one`;
  an event on another pool is dropped.
  Pro:
  keeps that accidental recovery.
  Con:
  same-pool events spend the bound and can hold a real request's pass.
- D,
  no filter:
  every event of every pool starts a handler and a pass.
  Example:
  every event on every pool starts a pass.
  Pro:
  the most recovery.
  Con:
  a full pass per event of unrelated datasets and pools.
- Ranking:
  A > B > C > D,
  because A keeps the bound for this design's requests,
  B adds another design's channel at one event class,
  C lets a snapshot cadence spend the bound,
  and D makes every unrelated event a pass.

### V3. Which commands nudge, and how often

- A,
  adopted:
  every invocation that forwards to Git checks the record once,
  and writes one nudge per listed clone whose entry is older than `G`.
  Example:
  two `git status` runs ten seconds apart with `G` at one minute and clone `one` unanswered:
  the first writes one nudge and moves the entry's `nudged-at`,
  the second writes nothing,
  and the first command after the minute writes one more.
  Pro:
  any command recovers the clone,
  and `G` bounds a burst on its own,
  because the project's third answer keeps the budget from holding a pending clone's pass.
  Con:
  every command pays one `stat`,
  which M8 measures,
  and `G` is unmeasured until then.
- B,
  only the wrapper's management commands and a command that selects the clone's worktree.
  Example:
  the same clone unanswered;
  a `git status` in the repository writes nothing,
  and the recovery waits for an add or a removal.
  Pro:
  the forwarded path pays nothing.
  Con:
  the person's ordinary commands,
  which are the ones that run,
  do not recover the clone,
  and the project's answer asks for the wrapped command.
- C,
  every forwarding invocation,
  one nudge per clone per invocation and no gap.
  Example:
  forty `git status` runs in a loop write forty nudges for the same clone and forty history events.
  Pro:
  the simplest rule,
  and no `G` to size.
  Con:
  a burst writes a burst of nudges,
  and the budget cannot hold them under the project's third answer,
  so only the merge device bounds the passes.
- Ranking:
  A > B > C,
  because A recovers on any command while `G` keeps a burst to one nudge,
  B leaves the commands that run inert,
  and C leaves the burst to the merge device alone.

### V4. Whether event-driven passes are exclusive route-wide

- A,
  adopted:
  one run claim for the route,
  so one event-driven pass at a time,
  and a contender marks instead of running.
  Example:
  two nudges a second apart:
  the second marks and exits 2,
  and the first pass runs a second pass.
  Pro:
  bounded passes and coalescing,
  which is this item's task.
  Con:
  it narrows the accepted "parallel handlers can
  sweep at once" ([proposal][proposal],
  lines 561 to 562),
  and a long pass makes every other event wait for the next trigger.
- B,
  per slot only:
  no run claim,
  so every trigger runs a pass and only the per-slot claim serializes them.
  Example:
  two nudges a second apart run two passes at once,
  each attaching different clones.
  Pro:
  the accepted behaviour stands,
  and no pass waits for another.
  Con:
  no coalescing and no bound:
  N triggers are N
  passes,
  which is the cost this item exists to answer.
- Ranking:
  A > B,
  because the item's task is the coalescing B does not give,
  and A's narrowing is offered for the
  project's veto rather than assumed.

### V5. Whether readiness passes are outside the bound

- A,
  adopted:
  `boot-reconcile` from the boot unit or its reload never takes the run claim,
  never reads or writes
  `pass-budget`,
  and is never held.
  Example:
  a late key load,
  with the budget spent a second earlier:
  the started unit publishes readiness and attaches
  the clone.
  Pro:
  a cost rule never decides whether callers are admitted,
  and a boot always publishes readiness.
  Con:
  two passes
  can overlap,
  and a hooked boot runs one extra action.
- B,
  readiness passes inside the bound and holdable.
  Example:
  the same late key with the bound spent:
  readiness stays unpublished and every caller is refused until an
  admitted trigger runs.
  Pro:
  one number covers all root work.
  Con:
  a cost rule becomes an availability rule.
- Ranking:
  A > B,
  because availability must not depend on a limit whose number is not yet measured.

### V6. The cost records' shape

- A,
  adopted:
  one folder,
  `/run/git-worktree-clone-mount-sweeps`,
  holding `run.<generation>`,
  `another-pass-is-due`
  (content `v1`) and `pass-budget`.
  Example:
  `ls -l` shows the generations,
  the mark and the budget file,
  all root-owned,
  the two files `0644` so the
  account's `doctor` can read them.
  Pro:
  per boot,
  no migration,
  no configuration,
  and the numbered claim ignores the other names by its own rule (F22).
  Con:
  names a person may wonder about,
  which `doctor` answers.
- B,
  a subfolder for the generations and the mark beside it.
  Example:
  the same names,
  with `run/1` inside a folder of its own.
  Pro:
  the claim's folder holds only its names.
  Con:
  an extra level the claim's own name filter does not require,
  because
  the claim already ignores other names.
- C,
  the mark inside the readiness record and the budget beside the receipts.
  Example:
  a `pass_due` field in readiness and a budget field in the receipts folder.
  Pro:
  two folders instead of one.
  Con:
  a published record changes format for a cost device.
- Ranking:
  A > B > C,
  because A needs no subfolder for a reason the code does not require,
  and C changes a format and
  adds a listing.

### V7. A cost record that cannot be read never refuses a pass

- A,
  adopted:
  an unreadable claim folder,
  mark or budget record is reported and the pass continues.
  Example:
  `pass-budget` corrupt:
  the handler prints that the budget was not applied,
  passes,
  attaches,
  and files its
  receipt.
  Pro:
  the device cannot become an availability bug,
  and safety stays with the per-slot claim.
  Con:
  a corrupt record
  lets passes run unbounded until it is fixed.
- B,
  refuse the pass when a cost record is unreadable.
  Example:
  the same corrupt file:
  the caller is refused and the clone stays unmounted.
  Pro:
  the bound is never silently unenforced.
  Con:
  a file in `/run` decides whether a worktree is available.
- Ranking:
  A > B,
  because the device is cost and the per-slot claim is safety.

### V8. Where a ZED-started handler's decisions are recorded

- A,
  adopted:
  the handler writes its own lines to journald as well as to standard error,
  through the crate's journald
  writer,
  so a ZED-started pass's filter,
  held and refusal lines are readable.
  Example:
  a held event leaves `held: the pass budget is spent for this window` in
  `journalctl -u zfs-zed.service`,
  beside ZED's `Finished ... exit=2` line.
  Pro:
  the decisions are visible where the trigger is.
  Con:
  a journal write per decision,
  and the fields must not leak
  beyond the clone and the record names.
- B,
  a root-owned decisions record under `/run` that the handler appends to and `doctor` reads.
  Example:
  the same held event leaves a line in `/run/git-worktree-clone-mount-decisions`,
  which `doctor` prints.
  Pro:
  the account can read it.
  Con:
  a new record family with its own format,
  bound and `doctor` line.
- C,
  stderr only,
  as today.
  Example:
  the same held event leaves nothing;
  only ZED's `exit=2` line exists.
  Pro:
  no code.
  Con:
  the pass's own words are unreachable,
  which leaves an operator with an exit status and no reason.
- Ranking:
  A > B > C,
  because A puts the line where the trigger already is,
  B needs a new format for the same fact,
  and
  C leaves an operator with an exit status and no reason.

### V9. Whether `doctor` reads the service manager

- A,
  adopted:
  `doctor` prints the `systemctl --no-ask-password` commands and reads no manager.
  Example:
  the lost-event block prints `systemctl is-enabled zfs-zed.service` and `systemctl is-active zfs-zed.service`.
  Pro:
  no bus,
  no authorization,
  no prompt.
  Con:
  the report cannot say whether ZED is enabled without the project
  running the command.
- B,
  `doctor` reads the unit properties over the system bus,
  read-only.
  Example:
  the same report says `zfs-zed.service: disabled, inactive`.
  Pro:
  one report,
  no command.
  Con:
  a bus call and its failure modes in a report that today needs none.
- Ranking:
  A > B,
  because the plan's step 18 checks ZED and service wiring and a printed command is the smallest way to
  keep that check without a new authority.

### V10. Whether a lost event is a finding or an error status

- A,
  adopted:
  a finding,
  with the exit status the command already uses for a report.
  Example:
  `doctor` prints the lost-event block and exits as it does for a hook it lists.
  Pro:
  a state that is expected and recoverable does not read as a tool failure.
  Con:
  a script cannot tell it from a
  report with no findings by status alone.
- B,
  the error status 2.
  Example:
  the same report exits 2.
  Pro:
  a script sees it.
  Con:
  the route's own recoverable state becomes an error.
- Ranking:
  A > B,
  because the route expects this state and prints its levers.

### V11. What the timeout report says about the next trigger

- A,
  adopted:
  the timeout's fixed sentence names the record's entry it has just written and the trigger that entry feeds,
  the next Git command in this repository,
  and the wait reads no cost record.
  Example:
  a timed-out add's report ends with "the request is recorded;
  the next Git command in this repository will nudge it".
  Pro:
  no read on the waiting path,
  whose first read the step 7 measurement timed at 36 to 64 microseconds (F30),
  and the sentence names the trigger the project chose.
  Con:
  the report cannot say why no pass ran.
- B,
  nothing beyond the existing text.
  Example:
  the same report ends with the request,
  the last observation,
  `doctor` and the journal.
  Pro:
  one sentence fewer to test.
  Con:
  it hides the trigger the design just armed.
- C,
  read `pass-budget` on the timeout and name the rule as the cause.
  Example:
  the same report says "the pass budget for this window is spent,
  and this trigger was held",
  which the project's third answer makes impossible for this request.
  Pro:
  the shape would fit a rule that held it.
  Con:
  a root-owned record gains a reader on every timeout to explain a case that cannot happen.
- Ranking:
  A > B > C,
  because A names the armed trigger at no read,
  B hides it,
  and C explains a hold the rule no longer takes.

### V12. Which step builds the device and the rule

- A,
  adopted:
  the merge device and the mark in step 19,
  with the zedlet and the units;
  `pass-budget` and the number in
  step 22.
  Example:
  after step 19 a held event prints `held`;
  after step 22 `doctor` prints the window,
  the time spent and the budget.
  Pro:
  the coalescing exists as soon as ZED does,
  and the number still waits for its measurement (rule `LQM`).
  Con:
  the
  plan's step 19 gains code the plan did not assign it.
- B,
  both in step 22.
  Example:
  from step 19 to step 22 every event runs its own pass.
  Pro:
  one step owns the device and its record.
  Con:
  the cost the item exists for stands until step 22.
- C,
  the device in step 19 with a provisional budget,
  as step 7 shipped a provisional deadline.
  Example:
  from step 19 a provisional budget holds passes.
  Pro:
  a bound exists early.
  Con:
  a shipped limit with no measurement,
  which the decision forbids.
- Ranking:
  A > B > C,
  because A gets the coalescing when ZED arrives and keeps the number measured,
  B leaves the cost
  unbound until step 22,
  and C ships an unmeasured limit.
- The nudge on Git commands is not part of this entry:
  the record itself requires measurement M8 before the nudge ships,
  so its switch is the record's condition and not a step choice.
  The record's writer and the command-path check land in step 10,
  and `G` and `A` in step 22.

### V13. The unit the bound counts in

- A,
  adopted:
  a budget of pass time per window,
  in seconds,
  charged from the recorded duration of each completed pass.
  Example:
  with the small clone population a pass costs a fraction of a second,
  so an illustrative budget of a few seconds admits several passes in a window;
  with the largest population one pass can spend the whole budget,
  so the next trigger waits for the next window.
  Pro:
  bounds the resource the passes spend.
  Con:
  a pass's duration is known only when it ends,
  so the budget is charged after the pass and one pass can overshoot it;
  a pass killed before it ends is charged nothing.
- B,
  a count of passes per window.
  Example:
  the same illustrative count admits back-to-back passes over the largest population,
  and holds passes over the small one that cost almost nothing.
  Pro:
  simple to count and to read in `doctor`.
  Con:
  cost and count diverge with the clone count.
- C,
  leave the unit to step 22's measurement.
  Example:
  `doctor` says the rule is not in force until the measurement exists.
  Pro:
  no shape decided here.
  Con:
  this design's task is the shape,
  and C defers it.
- Ranking:
  A > B > C,
  because A bounds what is spent where B bounds a poor proxy for it,
  and B still gives step 22 a shape where C gives none.
- The threshold is not asked here:
  step 22's M3 reports what each candidate threshold would allow,
  and the project asks the threshold after the cost branch merges and M3 runs,
  with the measured numbers beside it.
- The window kind is a second choice inside A,
  and A takes the tumbling one:
  - A tumbling window,
    adopted:
    `P` starts at the first charge in it and the spent time resets once `P` has passed.
    Example:
    with an illustrative window of ten minutes,
    a burst at 17:29 and another at 17:31 share the window started at 17:29,
    and the window that starts at 17:39 begins with nothing spent.
    Pro:
    one line,
    one start time and one spent time,
    replaced whole by the single claim holder (A10).
    Con:
    a burst at the end of one window and the start of the next can pass twice the budget inside a span shorter
    than `P`;
    L15 states it.
  - A sliding window,
    one start time per pass kept in the record.
    Example:
    the same two bursts,
    with the 17:31 pass admitted only if the 17:29 one has fallen outside the last `P`.
    Pro:
    no boundary burst.
    Con:
    a row per pass to write and read,
    and the record's size depends on how many passes fit in `P`.
  - Ranking:
    tumbling > sliding,
    because the record stays one line the single holder replaces whole,
    and the boundary burst is a bounded overshoot L15 states,
    where sliding buys that bound with a record sized by the pass rate.

## Dependencies on other designs and on the plan

- D1.
  The propagation sibling ([sibling propagation][sibprop]),
  read at `30274435f3`
  (2026-11-22T18:50:22-05:00).
  Its veto choice 3 is the manual re-run:
  `ExecReload=` on the boot unit plus `reload-or-restart` as the printed command
  (its lines 2676 to 2699),
  and the project's answer takes exactly that line and that command,
  so T5,
  the hook section and `doctor` state them without a fallback.
  The order inside both of its entry points
  ("Where it runs,
  as whom,
  and in what order",
  lines 1334 to 1409)
  is the order this design cites and states no other:
  configuration and checkout,
  its propagation step,
  the probe record,
  the pool and checkout checks,
  the sweep;
  this design's taker runs those prerequisites first and takes its claim only after them.
  At this tip its A3 (lines 892 to 903),
  its hook bullet in "Dependencies on sibling designs" (lines 3063 to 3079)
  and its L11 (lines 1009 to 1018)
  read this design's hook as this design states it:
  a `-`-prefixed `ExecStartPost=` that asks the boot unit to act,
  the boot unit's `ExecReload=`,
  and `reload-or-restart` as the printed root command.
  Its own note (lines 3106 to 3115) records that its round rewrote the four sentences that assumed this
  design's earlier direct hook,
  and that its two pointers to this design's V3 and order are accurate at their commit `5e6a2ff9dc`;
  this design has since moved those lines,
  so a later agreement check should re-read them.
  Its question 2 option 1 adds a propagation service upheld by the checkout's mount unit,
  whose `ExecStart=` is the same job request this design's hook sends (F42,
  its lines 2464 to 2500 and 2866 to 2886);
  the sibling installs that service and the project answers its question,
  and this design only records that the verb now has three senders.
  Its F31 (lines 752 to 766) relies on this design's filter keeping the checkout dataset's events out,
  which V2's first option does;
  V2's option B widens it,
  which the sibling's question 2 option 2 asks for,
  and V2 ranks that below the adopted scope with its cost stated.
  Its F22 (lines 649 to 662) is the authorization-agent fact the records section cites.
- D2.
  The forced-removal sibling ([sibling forced][sibforced]) assumes "each removal request is one nudge-shaped property
  write" and that the handler may "coalesce sweeps without merging two nonces" (its D5,
  lines 1789 to 1790).
  This design
  writes the intent and,
  under V3,
  one nudge per forwarding command per gap,
  and the project's third answer exempts a pending clone's pass from the budget,
  so a removal's waiting pass is never held.
  The sibling's assumption is narrower than this design;
  no record decides whether a destructive step's pass is exempt,
  and the exception the project chose is the exemption that exists.
- D3.
  The cancelled-removal sibling ([sibling cancelled][sibcancelled]) lists "a later sweep nudge on that clone" among its
  triggers (line 974).
  This design writes a nudge from any forwarding command whose repository record names the clone,
  or by T5,
  so that trigger exists whenever a later command runs in that repository,
  and the sibling's sentence should say so.
- D4.
  The add route's record entry,
  the command path's check and the nudge are step 10's,
  the removal's entry and its removal on settle are step 14's;
  this design assumes both follow V3 and "The nudge on Git commands" and nothing else.
- D5.
  The remedy for a registered but unmounted slot after a timeout is step 10's,
  and this design only reports the state
  `doctor` shows.
- D6.
  The unmount half of an `unmount` intent is step 9's,
  and this design's triggers recover a lost `unmount` event the same
  way as a `mount` event;
  what the unmount does is that step's.
- D7.
  The cost branch `fix/git-policy-cli-zfs-step-8-recheck-findings-and-sweep-cost` at `d97a683346` is unmerged and
  unreviewed and changes the figures F28 cites (F34).
  No argument here needs either set to be final:
  L12 says so,
  and F28
  and F34 are cited as existence,
  not as a settled cost.
- D8.
  The plan edits owed after acceptance:
  the four shipped sentences naming step 10 for `doctor` (F29);
  the step 7 and step
  22 "Open items" sentences and the open-item entry's "steps 7 and 22" line;
  the "Gaps" list,
  which gains the cost
  record family;
  step 19's build list,
  which gains the filter,
  the device and the mark (V12's option A);
  step 10's,
  which gains the record of unanswered requests,
  the command-path check and the nudge's switch;
  step 14's,
  which gains the removal's entry and its removal on settle;
  step 22's
  "no existing constant changes" line,
  which gains `pass-budget`,
  the rule,
  and the record's gap `G` and stale age `A`;
  step 18's
  "ZED and service wiring" check,
  which V9 narrows to the printed commands and the plan's text should say so.
  Steps 19 and 22 update the readings they create,
  so step 18 gains no dependency on either;
  the `not built yet` sentence is the report's own rule until they land.
- D9.
  The `SPEC.md` edits owed:
  a section for the three cost records,
  their names,
  modes and bounds;
  a section for the record of unanswered requests:
  its name in the Git common directory,
  its owner,
  mode,
  fields and one-page read,
  the command path's one-stat check and its one line,
  and what happens when it is stale or missing;
  the timeout sentence of V11;
  the filter's line in the default invocation;
  and the handler paragraph's note that a held or filtered run exits
  2.
  The readiness refusal's trigger list (fact F27) needs no change:
  the nudge is now a designed trigger,
  and the text was written for its return.
- D10.
  Siblings that state no dependency on this design and gain none from it:
  the add-time Git lock,
  the add binding record (which says the two "share no fact"),
  the dirty-main restore (which "adds no trigger of its own"),
  the deferred-destroy checks (no mention),
  and the removal plan record.
  The add-time lock at `a7902e293b` says nothing about triggers at all;
  its earlier note that the trigger design would "gain one action,
  not one trigger" if that design's Q1 is A
  no longer stands at that tip,
  and it is the condition to reconcile if the sentence returns.
  The origin cleanup record at `4daa6c66ce` is not only a quotation:
  it says its carrier "is a trigger of its own" (line 2669),
  and it rejects its root-side option partly because
  "a nudge needs a channel and a rate rule the project has not designed" (lines 2560 to 2571).
  This design now writes that rule's shape,
  with its number owed,
  so that con is weaker than the sibling states;
  whether its carrier needs this design's records is its own answer.

## What this design does not cover

The pass's own checks and order (step 8's),
the unmount and staging (step 9's),
the add route's registration and fill
(step 10's),
the dirty-main restore (step 11's),
the managed lock (step 13's),
removal (steps 14 to 17),
the rest of
`doctor` (step 18's),
the migration and the units' full bodies (step 19's),
sanoid (step 20's),
dependency repair (step
21's),
the numbers (step 22's) and the host-kernel qualification (step 23's).
It does not design a timer,
a periodic
reconciliation,
a queue of requests,
or a second checkout.
It does not design the start probe's record;
it reads an
absent one as a refusal,
which is that design's answer.

## Sources read

- `doc/decision/cli-git-zfs-worktree-design.md`,
  whole file,
  and `doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
  whole file.
- `doc/decision/cli-git-zfs-start-probe.md`,
  whole file,
  and `doc/planning/cli-git-zfs-start-probe-design.md`,
  "Who runs
  it,
  and where".
- `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
  "ZED reconciliation and mount safety",
  "Boot and shutdown
  unit",
  "Requests,
  results and ordering",
  "Waiting and refusal",
  "Scoped delegation",
  "Project decisions" (the
  lost-last-event entry),
  "Open items the decisions leave",
  "Acceptance tests the build still owes".
- `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
  steps 7,
  8,
  10,
  14,
  18,
  19,
  22,
  the open-item list and the
  gaps list.
- `doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
  `doc/decision/cli-git-zfs-slot-identity-store-format.md` and
  `doc/decision/cli-git-zfs-staging-under-cache.md` ("The handle record's place"),
  whole sections named.
- `package/git-policy/cli/SPEC.md`,
  "Management grammar",
  "Doctor",
  "The clone's mount intent,
  receipts and
  readiness" and the credential paragraph.
- `package/git-policy/cli/doc/zfs-mount-receipt-deadline-measurement-2026-11-21.md`,
  whole file.
- `package/git-policy/cli/src/native/` files the facts name,
  under `worktree_clone_mount_protocol/`,
  `worktree_clone_mount_handler/`,
  `lock/` and `binary/`.
- `package/git-policy/cli/Cargo.toml`,
  the two `[[bin]]` entries and the handler's installed name.
- Sibling design documents,
  with `git show <branch>:<path>`:
  the private propagation document at `30274435f3`,
  then the forced removal,
  cancelled or
  interrupted removal,
  add-time lock,
  add binding record and dirty-main restore documents.
- OpenZFS at `zfs-2.4.4`:
  `cmd/zed/zed.c`,
  `cmd/zed/zed_event.c`,
  `cmd/zed/zed_exec.c`,
  `cmd/zed/zed_exec.h`,
  `cmd/zed/zed_conf.c`,
  `cmd/zed/zed.h`,
  `cmd/zed/zed.d/history_event-zfs-list-cacher.sh.in`,
  `module/zfs/fm.c`,
  `module/zfs/spa_history.c`,
  `include/sys/sysevent/eventdefs.h`.
- systemd at `v262`:
  `man/systemd.unit.xml`,
  `man/systemd.service.xml`,
  `man/systemd.mount.xml`,
  `man/systemd.kill.xml`,
  `man/systemctl.xml`,
  `src/core/unit.c`,
  `src/core/service.c`,
  `src/core/kill.h`.
- This host's unit files,
  `/etc/default/zfs`,
  and the read-only probes listed in "Facts".

## Revisions

- 2026-11-22T17:29:03-05:00:
  the first version,
  unapproved and non-final,
  with the brief's two coordinator corrections folded in.
- 2026-11-22T17:29:34-05:00:
  corrected the revision record and the provider section's wording.
- 2026-11-22T17:30:36-05:00:
  said which taker clears a rate-refused mark.
- 2026-11-22T18:15:57-05:00:
  the first gate review's answer,
  a rewrite that fixed the provider hook (H1),
  the nudge policy (H2),
  the eleven medium findings,
  the eight low findings,
  the wrong citations,
  the eight silently decided choices and the sibling readings.
- 2026-11-22T18:16:16-05:00:
  recorded that commit's time in this section.
- 2026-11-22T18:16:38-05:00:
  pointed T1's sentence about the handler's lost line at V8 rather than V7.
- 2026-11-22T18:17:06-05:00 and 2026-11-22T18:17:11-05:00:
  the nudge-during-pass arm,
  doctor's step 18 narrowing,
  the full revision list and the revision tail's wording.
- 2026-11-22T18:41:39-05:00:
  the second gate pass's answer.
  The printed root lever is `reload-or-restart` with its `ExecReload=` condition (T5,
  the `doctor` block,
  Q-A option B,
  L13).
  The hook's reach is stated per unit state and its reload arm is marked unreachable (T3,
  F17,
  F37,
  tests 12 and 13).
  Tests 8,
  12 and 13 name their starting state and their verb.
  The sibling statements cite the propagation design at `5e6a2ff9dc`,
  and the hook is stated once for a sibling to cite
  (D1,
  D10,
  "The hook,
  in one form a sibling can cite").
  The counts list and the build steps now agree:
  step 10 ships the nudge off,
  step 19 ships the device's two counts as a
  veto-open adoption,
  step 22 ships the budget and the delay.
  Q-D became V13 and its threshold is asked after M3.
  A refused prerequisite spends no start and clears no mark (the taker's steps 3 and 4).
  The first measurement's control is the no-pass arm.
  Step 18 reports the three cost records as `not built yet` until their steps land.
  The review history and the scratch paths left the body.
  One change goes beyond the findings:
  `pass-budget` replaced `recent-starts`,
  because V13's first option is a pass-time budget and a duration is known only when a pass ends;
  the second pass had not flagged V6,
  and this change is forced by the budget's kind.
- 2026-11-22T19:03:19-05:00:
  the diff check's answer.
  The hook is stated as two whole files with their section headers,
  and the section itself states the request,
  the verb per unit state,
  the provider's `-` prefix and the root command.
  The taker runs the sibling's entry order before it takes the claim,
  so the steps are renumbered and step 8's reference follows the mark's new step.
  The sibling statements cite the propagation design at `30274435f3` and its current coordinates,
  and D1 no longer says its sentences assume the earlier direct hook.
  D8 drops the step 18 dependency edit and says steps 19 and 22 update the readings they create.
  The lows:
  a charge after an expired window opens a new one (V13,
  L15 and the taker's step 5),
  the tumbling window as V13's second choice,
  the reload arm's reason,
  the nudge's switch as V12's second choice with the refusal text and the D9 edit it needs,
  doctor's `start` command without `ExecReload=` and the `not built yet` reading,
  and the blank line before "What this design does not cover".
- 2026-11-22T19:03:39-05:00:
  recorded the answer's commit time above.
- 2026-11-22T19:33:18-05:00:
  the project's three answers are written in;
  `main` was merged first,
  at 2026-11-22T19:22:22-05:00,
  bringing the decision record `cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`.
  The questions left the question list and became "The project's three answers",
  and the options not chosen are listed once.
  The hook is the record's answer and is stated without a fallback;
  the sibling's `Upholds=` service is named as a third sender of the same job request,
  and the `ExecReload=` sample now names `mount-git-worktree-clone-datasets`.
  T4 is the wrapper's nudge on Git commands;
  "The nudge on Git commands" designs the record in the Git common directory,
  the command-path check,
  the gap `G`,
  the stale age `A`,
  the burst and the measurement M8.
  The no-waiter case is rewritten to the command trigger.
  The cost rule holds only passes with no unanswered request,
  with the taker's one clone read.
  V2 gained the checkout-dataset option,
  V3 became the command-nudge choice,
  V11 the timeout sentence,
  and V12 lost the switch sub-choice.
  The build steps,
  tests 15 to 20,
  measurement M8,
  the counts,
  facts F38 to F42,
  assumptions A11 and A12,
  limits L16 to L20,
  and dependencies D1 to D4,
  D8 and D9 follow.
- 2026-11-22T19:33:36-05:00:
  recorded the answer's commit time above,
  and the last read corrected two citations it left:
  the one-page bound now points at the start-time gate,
  and the pending-check sentence at the add-records section that carries it,
  with `WorktreeIdentity` narrowed to the shapes that are repositories.
- Any commit touching this file after the entries above is newer than all of them,
  and `git log --format=%cI -- doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`
  lists it with its committer time.

[decision]: ../decision/cli-git-zfs-worktree-design.md
[answers]: ../decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[spec]: ../../package/git-policy/cli/SPEC.md
[step8]: ../decision/cli-git-zfs-step-8-sweep-choices.md
[store]: ../decision/cli-git-zfs-slot-identity-store-format.md
[staging]: ../decision/cli-git-zfs-staging-under-cache.md
[probedesign]: cli-git-zfs-start-probe-design.md
[probdecision]: ../decision/cli-git-zfs-start-probe.md
[sibprop]: cli-git-zfs-private-propagation-after-replacement-mount-design.md
[sibforced]: cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md
[sibcancelled]: cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md
