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
the project's answers
([the decision record][answers]),
the built handler code
the facts name,
the accepted start-probe design,
the project's propagation-repair answer,
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
accepted 2026-11-22 in the question tool and recorded 2026-11-22T19:20:28-05:00,
and answered a further one on how long a lost request is nudged,
recorded 2026-11-22T20:49:43-05:00.
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

The record labels the requirement that follows from that cost as the coordinator's,
not the project's:

> What follows from that cost is the coordinator's requirement on the design,
> not a part of the option the project chose:
> the design owes a way for a command to learn cheaply that a repository has no unanswered request,
> and a measurement of what a command pays in each case,
> before this ships.

So the wrapper's command path carries a trigger (T4),
the section "The nudge on Git commands" designs the record and the command's check,
and measurement M8's numbers ship before the nudge does.
"Periodic reconciliation without a waiter is not part of this route" stands,
and this design adds no timer.
The further answer below states the consequence the project accepted for a request nothing can answer:
it keeps being nudged for as long as it stands,
at one nudge per longest gap,
and each nudge is one root handler pass.

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

### A lost request keeps being nudged for as long as it stands

The record's words,
under "A further answer:
how long a lost request keeps being nudged":

> The project chose "A:
> forever,
> at the capped gap (Recommended)":
> a request is nudged as long as it stands,
> never more often than the longest gap.

The cost the record states with this answer:

> a request nothing can answer costs one root handler pass per longest gap,
> for ever,
> while Git is used in that repository.
> The question's "once a minute" was an example and decides no number.

and what it leaves to the measurements:

> The size of the budget and its window,
> the first gap and the longest gap,
> which wait for the measurements that size them.

So "How long a lost request keeps being nudged" states the decided rule,
V14 is no longer a veto entry,
and the first gap and the longest gap are M8's.

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
  because the tests they would need and the provisioning they would change are this design's own reasons for
  agreeing with the record's choice.
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
lines 817 to 830 at `037bcee2e5`).
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
    the folder of unanswered requests and the timeout sentence,
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
  when the repository it runs in holds the folder of unanswered requests
  ("The record:
  one file per clone").
  The project chose this trigger
  ([the decision record][answers],
  "A Git command recovers a lost request with nobody waiting").
- When:
  on the forwarded path,
  where the wrapper already finds the repository's common directory without starting Git
  (fact F38),
  and before the child Git starts.
  The command reads the receipt for each listed clone first,
  and asks for a nudge only for an entry the receipt does not answer and whose gap has passed,
  the backoff "How often,
  and the burst" states.
- As which user:
  the account writes the property through the helper,
  under the same `mount,userprop` grant as before
  ([proposal][proposal],
  "Scoped delegation",
  line 338);
  the delivery and the pass are root's,
  as T1.
- Guarantees:
  one further history event per nudged clone,
  hence one delivery attempt of T1,
  and nothing for the command to wait on or to fail from:
  the command reads a receipt and at most spawns a detached helper,
  and the gate,
  the device,
  the property write and the record's publish are the helper's (fact F43).
  A refused device or a failed write is recorded in the next attempt as its outcome,
  prints nothing on the common path,
  and leaves the command's exit status alone.
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
  the entry's next attempt and its outcome,
  the helper's own line when it writes one,
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
  Its one condition is the `ExecReload=` the record fixes:
  with that line an active unit reloads and nothing unmounts;
  without it,
  the same command on an active unit is a restart,
  which runs `ExecStopPost=shutdown-unmount` and unmounts every clone,
  so the printed lever would be a stop pass over the clones.
  `start` alone re-runs a failed or inactive unit and does nothing on an active one (fact F15);
  `restart` always runs the stop pass first and unmounts every clone (fact F17).
  Running the handler binary from a person's shell is not recommended,
  because that
  shell's mount namespace is not the host's (fact F18).
- The account lever:
  the same `zfs set` of a fresh nudge nonce on the clone that T4's command nudge writes,
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
and a measurement of what a command pays in each case
([the decision record][answers],
"A Git command recovers a lost request with nobody waiting").
This section designs both,
with the record that carries them.

### The record: one file per clone

The wrapper leaves one record per repository,
a folder named `zfs-unanswered-requests` in the Git common directory,
which the wrapper finds without starting Git through the same call the shadow sweep uses before a command
(`known_common_dir()`,
`src/native/repo_read/repository_facts.rs` lines 176 and 517 to 525;
`src/native/invocation/wrapped_command_notices_snapshots_and_manual_push.rs` line 503;
fact F38).
The folder is the account's,
made by the wrapper on the first entry.

Inside it,
one file per clone with an unanswered request,
named `<guid>.<attempt>`:
the clone's canonical numeric dataset GUID,
which is also the receipt's name,
and the attempt number,
from 1.
One file per item is the shape the project chose for the slot identity store
([store decision][store],
"What was decided"),
and it removes the shared-file lost update that store's question names:
two waits ending together write two files rather than one rewrite that loses the other,
and two commands started together cannot both rewrite one entry.
The file's content is one line:

    v1 <guid> <nonce> <operation> <written-at> <outcome> <dataset>

- `nonce`,
  `operation` and `dataset` are the request's,
  as the intent carries them.
- `written-at` is the moment the entry was written,
  as an ISO 8601 timestamp with its offset.
- `outcome` is `claimed` when a writer has just taken the attempt,
  `nudged` when the helper wrote the property,
  `refused-gate` or `refused-device` when the gate or the device refused,
  and `clone-gone` when the write found no dataset,
  so a host without the device records its refusal once instead of printing a line per command.

A new attempt is published the way the slot store publishes a record:
written whole under a staging name made with `O_EXCL`,
synced,
and `link`ed to `<guid>.<attempt>`,
and the writer removes its staging name after the link,
leaving it to `doctor` only when it was killed.
The staging name is unique per writer,
`.` and 32 lowercase hexadecimal digits as the slot store's own staging names are
(`slot_identity_store_records.rs`,
`STAGING_NAME_DIGITS` and `is_known_staging_name`),
so a killed writer's leftover blocks no later writer:
the next writer's own name is free,
and its `link` is what decides the race.
`doctor` at step 18 removes the known staging names,
as the slot store's answer has `doctor` remove its own
([store decision][store],
"Who removes a killed writer's staging file"),
and a command ignores a staging name it did not make.
The file's modification time is the moment of its attempt,
so the gap that "How often,
and the burst" states is read from it.

The entry is read the way the pending check reads a Git pointer file:
opened as a plain file without following a link,
refused if it is not a regular file,
bounded to one page,
and refused if its owner or mode is not the account's
([spec][spec],
lines 4548 to 4556,
the passage that says a named pipe would stop a command in an unrelated worktree;
fact F41).
One page is the gate's own per-file bound,
and no measurement is owed for it:
an entry's fields are bounded (the GUID,
a 32-digit nonce,
two fixed timestamps,
one operation word and a dataset
name under ZFS's own name limit),
so a page holds any entry this design writes,
and a longer file is one the reader refuses as corrupt.

An entry is written with the request,
before the intent property,
so a wait killed before its deadline leaves it behind,
and it is removed when the wait that asked is answered (V15's first option).
A wait that writes a new entry first removes that clone's older entries,
proving only that they name the same clone's GUID,
because the clone's durable intent carries one nonce and an older one can never be answered.
An entry is removed:

- by the command that reads the receipt and finds it answering the entry's nonce;
- by the helper,
  after it reads the clone's intent with the device open,
  when the intent is absent or carries another nonce than the entry's,
  which proves the entry can never be answered;
- by the helper whose nudge write fails because the clone is gone;
- by the wait that is finally answered,
  before it reports;
- by the removal,
  when it settles the clone (step 14's,
  D8);
- by `doctor`'s printed line,
  which runs nothing.

No age removes an entry:
the project's sentence is "whenever a wrapped Git command runs in a repository that holds a clone with an unanswered request",
and an age that deleted a pending entry would narrow it.
The folder is removed with its last entry:
the remover tries `rmdir` and ignores a failure,
so a repository whose request was answered returns to the one-`stat` case,
and a folder left behind by a racing creator costs one `read_dir` that finds nothing
(M8 has that case).

### What a command pays, in each case

The command reads the receipt before it spends anything else.
The entry carries the clone's GUID,
receipts are named by GUID,
and one bounded read of `/run/git-worktree-clone-mount-results/<guid>`
tells the command whether the request was answered,
with no device and no gate.
This is what keeps the cheap negative the owner required for a repository whose request was answered.

- No folder:
  one `stat` of the folder,
  the cost class of the shadow sweep's two `read_dir`s that find nothing (fact F38).
- A folder whose entry the receipt answers:
  the `stat`,
  one `read_dir`,
  one bounded read of the root-owned `0644` receipt,
  and one `unlink`.
  The receipt read is the wait's own read,
  36 to 64 microseconds in the step 7 measurement (F30),
  and nothing opens the device.
- A folder whose entry is unanswered and not yet due:
  the same,
  and nothing else.
- A folder whose entry is due:
  the same,
  and the command writes a staging file and `link`s `<guid>.<n + 1>` with the outcome `claimed`;
  the link decides the one owner,
  a command that lost it writes nothing more,
  and the one that won spawns a detached helper for that attempt.
  A `.git` the command cannot write in stops it there:
  nothing is spawned,
  nothing is printed,
  and the trigger is unavailable for that repository until it can be written.

The helper pays what the command must not:
the compatibility gate,
which reads `/sys/kernel/btf/vmlinux` and the module's BTF and walks them
([spec][spec],
lines 20441 to 20520;
its start cost is unmeasured,
`doc/planning/cli-git-zfs-runtime-layout-checker-design.md` line 1185),
the device open,
the property write,
and the outcome's record:
the helper replaces the attempt's own file with `nudged`,
`refused-gate`,
`refused-device` or `clone-gone` by writing a staging file and renaming it over that attempt,
which only this helper owns.
The property write does not return until its transaction group commits:
`dsl_props_set` calls `dsl_sync_task` (`module/zfs/dsl_prop.c` line 1053 at `zfs-2.4.4`),
which ends in `txg_wait_synced` (`module/zfs/dsl_synctask.c` lines 94 to 105).
The command waits for a spawn,
never for the route,
which is the rule this design takes from the cost the record states for the second answer:
a nudge must not make a command wait or fail.
A nudge that the helper writes,
and the pass it starts,
are the root work M8's root-side arm counts.

A host without `/dev/zfs` pays the first three cases and no more:
the gate refuses at its device step before it reads any BTF
([spec][spec],
lines 20492 to 20493),
the helper publishes the next attempt with `refused-device`,
and nothing on the common path prints a line.
A `.git` copied to a machine without the datasets behaves the same way,
and `doctor` reports the refusals.

### Which commands, and where

Every invocation of the wrapper that forwards its command to Git checks the folder,
once,
on the forwarded path,
before the child Git starts,
at the same point the shadow sweep runs,
after configuration and before the commit path
(`run_wrapped_command`,
`src/native/invocation/wrapped_command.rs` line 428;
fact F38).
A command refused in `prepare_guarded_command` returns before that point and takes no part in the trigger,
which is accepted.

A command asks for a nudge only for an entry the receipt does not answer and whose attempt is due,
and the link decides which command of a burst is the one owner of the next attempt.
Any failure on the command's read path (the folder,
an entry,
the receipt,
the staging write,
the link or the spawn) is ignored without a line and without a change to the command's status;
a `.git` it cannot write in leaves the trigger unavailable for that command,
silently.
The helper is detached:
the command starts it and returns,
the helper takes its own session,
and its standard streams go to no terminal,
so no nudge makes the command wait for the gate,
the device,
the transaction group or the record's sync,
and none changes the command's exit status.
A spawn that fails prints one line and leaves the command's own outcome alone.

The wrapper's own management commands do not take this path:
the add and the removal write and remove their own entries,
and `doctor` reads the folder to report it.
A child Git the wrapper starts is the real Git and not this wrapper,
so a command cannot reach the check twice.

### How often, and the burst

An entry's attempt number `n` sets the gap before the next attempt:
`G(n) = G0 * 2^(n - 1)`,
capped at `Gmax`.
The first nudge comes `G0` after the entry was written,
the second twice as far,
and from the attempt that reaches `Gmax` on,
one nudge per `Gmax` for as long as the request is unanswered.
`G0` and `Gmax` are shipped limits,
and M8's arms size them:
the receipt latency gives `G0`,
the smallest delay that leaves a fast wait alone,
and the root cost of a nudge whose request nothing can answer gives `Gmax`.

A burst inside a gap asks for nothing,
and the link of `<guid>.<n + 1>` makes exactly one command of a burst the one that writes.
Two more things bound the rest:
the merge device,
which lets one pass run and makes the contenders mark,
and the project's third answer,
which lets the nudge's pass run even over the budget.

### How long a lost request keeps being nudged

The project decided this as its further answer
([the decision record][answers],
"A lost request keeps being nudged for as long as it stands"):
a request is nudged as long as it stands,
never more often than the longest gap.
So the gap stops at `Gmax`,
and nothing stops the nudging while the request stands:
a request nothing can answer costs one root handler pass per `Gmax`,
for ever,
while Git is used in that repository,
which is the cost the record states and the project accepted.
A request stops being nudged only by being answered or by its clone going,
which "The record:
one file per clone" lists,
and the record's "once a minute" was an example that decides no number.
`G0` and `Gmax` are M8's,
and V14 is no longer a veto entry.

### The cost rule with a pending request

The third answer changes the taker's budget step.
Before it decides,
the taker reads the one clone the event names:
its intent and the receipt under its clone's GUID (facts F24 and F25).
A request without an answering receipt passes without the budget being read further,
and an event that names no clone,
which is a parent event the filter accepts,
is decided by the budget alone.
The read is the cost the record's answer waits for,
and M3 measures it.
A nudge's event always names a clone,
so a nudge is never held;
what bounds nudges is the backoff,
the merge device and the single writer of `pass-budget`.

### The waiting command's own nudge

A wait writes no entry of its own:
the entry is written with the intent,
so any wait that ends unanswered,
or is killed,
leaves the entry behind (V15's first option).
At a delay `D` after its start with no receipt answering its nonce,
the wait claims its nudge through the same link a command uses:
it writes a staging file and `link`s `<guid>.<n + 1>`,
and spawns the helper only if it won,
so a wait and a command that find the entry due at the same moment write one nudge.
`D` is a shipped limit that M8 sizes,
the same receipt latency that gives `G0`:
the smallest delay that leaves a wait answered promptly alone.
So a lost event costs a caller one extra event only when its wait has produced no receipt by `D`,
and the next command recovers the clone if that event is lost too.
V16 lists keeping it and not keeping it,
with their costs.

### What a person sees, and what doctor prints

A command in a repository with no folder prints nothing new.
A command that finds an entry answered removes it and prints nothing.
A command that asks for a nudge prints one line naming the clone,
and the helper prints nothing on the command's streams;
the attempt's recorded outcome shows whether the nudge was written or refused.
A host without the device prints nothing per command,
which is what a sandbox needs,
and `doctor` reports the recorded refusals once.

`doctor` reads the folder and prints,
for each entry,
the clone,
the attempt number,
the nonce and operation,
when the entry was written,
when its attempt ran and what its outcome was,
whether the receipt answers it,
and,
for an entry whose clone is gone or whose staging name was left,
the one line that removes it.
Its lost-event block gains a `nudge` line naming the folder and the entries,
and its list of next triggers gains "the next Git command in this repository".
A folder it cannot read is reported as unknown,
not as absence.

### The measurement owed

M8 measures what a command pays as ratios against the same command on `main`'s build,
interleaved in one round on this machine and repeated for the spread (rule `QNB`),
in these cases:

- no folder:
  one `stat`;
- a folder whose entry the receipt answers:
  the `stat`,
  the `read_dir`,
  the receipt read and the `unlink`,
  with no device and no gate;
- a folder whose entry is unanswered and inside its gap:
  the `stat`,
  the `read_dir` and the receipt read;
- a folder whose entry is due:
  the same,
  plus the spawn,
  with the helper's own run measured separately
  (the gate,
  the device open,
  the property write and the publish);
- a host with the device refused:
  the recorded refusal and its next attempt;
- the root side:
  handler runs,
  pass time and events per command for an answerable request and for one nothing answers,
  with the backoff's `G0` and `Gmax` read from it.

Each case alternates with the same command on `main`'s build on the same host,
and the gate's cost gets its own control,
a build whose command path skips the gate,
so the gate's share is not hidden in the total.
The positive control is a planted entry whose request is unanswered and due,
which must make the record path and the helper run;
a case that shows no difference is read only after the control moved.

### What builds it

Step 10 builds the folder,
the entry written with the intent,
its removal when a wait is answered,
the command-path check,
the helper's mode and the nudge behind a compiled switch that is off;
step 14 writes and removes entries for a removal's waits;
step 18 reads the folder in `doctor` and removes the staging names;
and step 22 sets `D`,
`G0` and `Gmax` from M8 and turns the nudge on,
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
  whose entry its intent wrote,
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
- The folder of unanswered requests,
  whose entry its intent wrote,
  names the clone,
  its attempt and when it was written;
  a folder that is missing changes which trigger looks,
  not what the clone needs,
  and the next wait on that clone writes a new entry.
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

The `ExecReload=` is the line the project's lost-event answer accepts
([the decision record][answers],
"Which trigger runs after the checkout becomes ready"),
and the propagation sibling states it as accepted too,
so the same line serves its printed lever.
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
the `ExecReload=` line is the one the project's lost-event answer accepts
([the decision record][answers],
"Which trigger runs after the checkout becomes ready"),
and the propagation sibling states it as accepted in the same words,
keeping one short note for a machine that lacks it
([sibling propagation][sibprop],
"The project's four answers,
and what this design does with each",
its fifth bullet,
read at `037bcee2e5`,
2026-11-22T20:48:10-05:00).
So one `ExecReload=` serves two consumers here:
the hook's job request and the project's printed lever,
and neither needs a fallback sentence.

The sibling adds one more trigger that asks the boot unit in the same way,
and the project has since decided it:
a `checkout-propagation.service` upheld by a drop-in on the checkout's mount unit,
with `Upholds=` and `BindsTo=`,
whose `ExecStart=` is the same `systemctl --no-block --no-ask-password reload-or-restart` request
([repair record][repair],
"Which triggers the route adds";
[sibling propagation][sibprop],
"The trigger the route adds,
and what it must show",
lines 1400 to 1503,
and "The propagation service and its drop-in",
lines 2941 to 3067,
read at `037bcee2e5`;
fact F42).
The project took it provisionally:
it is dropped if the guest measurement shows that it misses a replacement typed by hand or that it loops.
So one verb has three senders:
this design's provider hook,
the sibling's upheld service,
and the project's printed lever.
The service and its drop-in are step 19's for the propagation design to install,
and this design does not install them.

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
The propagation repair's trigger is a different shape,
a drop-in beside the checkout's existing mount unit and an upheld service,
and the repair record has taken it provisionally.

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
lines 1566 to 1644 at `037bcee2e5`):
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
    lines 1566 to 1644 at `037bcee2e5`).
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

A wait writes no entry of its own:
its entry is written with the intent,
so a wait killed before its deadline leaves the trigger behind (V15's first option).
At a delay `D` after its start with no receipt answering its nonce,
the wait asks for one nudge through the same detached helper the command path uses,
which is the accepted proposal's nudge the wrapper writes on a caller's behalf.
`D` is owed to M8,
and V16 lists keeping that nudge and dropping it.
The nudge uses a fresh nonce,
so the write is a change
([spec][spec],
"The durable intent"),
and every later forwarding invocation nudges the entry again under the decided backoff.

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
- A request that nothing can answer,
  which is nudged for ever at the capped gap `Gmax`,
  one handler run and one pass per `Gmax`.
  The record's threat-model sentence was written for a hostile loop of property writes;
  under the second and third answers together the loop is ordinary Git use,
  so this design states the cost rather than leaning on that sentence,
  and the project accepted it in its further answer ("How long a lost request keeps being nudged").
- The number of commands that read the folder and ask for nothing,
  which is one `stat` each (V3's first option).

### What a waiter sees when no pass runs for it

A caller whose own request is unanswered is never held,
because its event names the clone and the exception runs it.
So the deadline report has no hold to explain,
and a wait ends at its deadline only when no pass has run for it at all.
What it prints is the request,
the last observation,
`doctor` and the journal (fact F20),
the entry its intent wrote,
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
- The folder of unanswered requests ("The record:
  one file per clone"):
  its path from the repository's common Git directory,
  and each entry's clone and attempt number,
  its nonce and operation,
  when it was written,
  when its attempt ran and what its outcome was,
  whether the receipt answers it,
  and,
  for an entry whose clone is gone or whose staging name was left,
  the one line that removes it.
  A folder it cannot read is reported as unknown,
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
  nudge    the folder names this clone (attempt 3, outcome pending, written 2026-11-22T17:30:04-05:00); the receipt does not answer it; the next command after the current gap nudges again
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
  the folder of unanswered requests,
  its entry written with the intent and removed when a wait is answered,
  the command-path check of "Which commands,
  and where",
  the nudge behind a compiled switch that is off,
  and the wait's cost text of V11.
  Step 22 sets `G0`,
  `Gmax` and `D` from M8 and turns the nudge on,
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
  the folder of unanswered requests,
  its entries and the known staging names it removes,
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
  the record's `G0`,
  `Gmax` and `D` with M8 and the switch turning the nudge on,
  the budget and its window in V13's unit,
  and their numbers with the measurement beside them.

The device and the mark land in step 19 because step 19 enables ZED,
and without them every event starts its own pass;
the folder,
the entry written with the intent,
the helper and the command-path check land in step 10 because that is
where the wrapper's first wait and its forwarded path are built,
and the first gap,
the cap and the delay land in step 22 because the record itself requires the nudge's
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
    a Git command in the repository after the intent's event was lost by the zedlet not being scanned
    (test 20's technique),
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
    a folder absent opens nothing and prints nothing;
    a folder whose entry the receipt answers reads the receipt and unlinks the entry with no device open;
    a folder whose entry is unanswered and due spawns the helper once,
    and the helper's write is the only device use;
    a folder whose entry is inside its gap spawns nothing;
    a refused device makes the helper record `refused-device` and prints nothing on the command's streams;
    a malformed entry,
    a record that is a pipe or a link,
    and an entry whose clone is gone are refused or removed as "The record:
    one file per clone" says.
    The positive control is a planted entry the receipt answers,
    which must be removed,
    so a zero from the absent-folder arm is not read as none.
16. The command burst and the backoff,
    host with an injected clock:
    three commands inside the entry's gap ask for nothing,
    and the first after it asks once;
    two commands started together on one due entry leave one link winner and exactly one spawned helper,
    and the other prints nothing and spawns nothing;
    the attempt number and the doubling gap are the file's,
    and the cap applies from the attempt that reaches it.
    Forcing is the injected clock and the link's `EEXIST`,
    not a sleep (race R9).
    The same arm with a read-only common directory asserts no staging write,
    no link,
    no spawn and no output,
    and a leftover staging name of a killed writer does not block a later command.
    A wait and a command that find the entry due together write one nudge (V16's first option).
17. The entry's life on a wait,
    host:
    an add writes its entry before its intent,
    an answered wait removes it,
    and a killed wait leaves it (V15's first option);
    a second request on the same clone removes the older entries it supersedes;
    an add that fails before its intent write leaves an entry the helper drops,
    because the clone's intent is absent;
    a command killed after the link leaves the attempt counted,
    and its staging name is one `doctor` removes;
    a helper killed before it records its outcome leaves the attempt `claimed`,
    which the next command counts;
    the mid-wait nudge at `D` asks once,
    and a receipt before `D` asks nothing (V16's first option).
18. A missing folder and a refused device,
    host:
    a folder the account cannot read is reported as unknown;
    a repository with no folder but an unanswered intent is reported by `doctor`,
    which names the other triggers;
    a host without the device records `refused-device`,
    prints nothing per command,
    and the next attempt's gap has grown (M5's case).
19. The rule's exception,
    host with a fixture budget and a planted clone:
    with the budget spent,
    an event naming a clone with no answering receipt passes and files its receipt,
    while an event naming no clone is held with the held line;
    the positive control repeats with the smallest admissible budget (M3's control).
20. The Git command recovers a request whose event was lost,
    guest:
    start ZED with the zedlet absent from its scanned set,
    write the intent,
    so its event is consumed with no matching zedlet,
    then install the zedlet,
    send `SIGHUP`,
    and run one forwarded command;
    assert a receipt under the intent's nonce and one history event for the nudge.
    Control:
    the same run with the entry removed,
    which writes nothing and leaves the clone waiting.
    A ZED start is not the arm,
    because a start replays the intent's event too (race R6).

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
  what a Git command pays for the folder of unanswered requests,
  and what one nudge costs root.
  Probe:
  the same command in these cases,
  each alternated with the same command on `main`'s build in the same round,
  repeated for the spread (rule `QNB`),
  and each reported as a ratio to `main`'s build:
  - no folder,
    which is one `stat`;
  - a folder whose entry the receipt answers,
    which is the `stat`,
    the `read_dir`,
    the receipt read and the `unlink`,
    with no device and no gate;
  - a folder whose entry is unanswered and inside its gap;
  - a folder whose entry is due,
    with the spawn measured on the command and the helper's gate,
    device open,
    property write and publish measured on the helper;
  - a folder left behind by a racing creator,
    which pays one `read_dir` that finds nothing;
  - an entry the helper drops because its intent is absent or names another nonce;
  - a host whose device is refused,
    whose helper records `refused-device`;
  - the root side:
    handler runs,
    events and pass time per command,
    for a request the route can answer and for one nothing answers,
    which is what sizes the cap `Gmax`.
  The gate's share gets its own control,
  a build whose helper skips the gate,
  and the positive control is a planted entry the receipt answers,
  which must be removed and must not touch the device,
  so a zero from the absent-folder case is not read as none.
  Decides:
  that the command trigger ships at all,
  the first gap `G0`,
  the cap `Gmax`,
  and the wait's delay `D`,
  from the receipt latency the wait measures (F30).

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
  each entry file's one-page bound,
  the first gap `G0`,
  the capped gap `Gmax` and the wait's delay `D`.
  Step 10 builds the folder,
  the entry written with the intent,
  its removal when a wait is answered,
  the command-path check,
  the helper and the nudge behind a compiled switch that is off;
  step 22 ships `G0`,
  `Gmax` and `D` with M8 beside them and turns the nudge on,
  which the record itself requires before the nudge ships.
  Until then a person sees one event per add (the intent),
  no nudge,
  and the timeout text and the levers when a wait times out;
  T4 is the trigger that arrives with the measurement.
  The one-page bound of an entry file is the bound the start-time gate gives each file it reads (F41),
  and no measurement is owed for it because an entry's fields are bounded;
  an entry that would not fit is refused by its writer with one line,
  and `doctor` reports it.
  With the switch off,
  the command path's folder check is compiled out too,
  so nothing of the folder is read until step 22.
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
  and the repository's common directory is found there without starting Git by `known_common_dir()`,
  from the location already asked or from the file-system layout
  (`src/native/repo_read/repository_facts.rs`,
  lines 176 and 517 to 525),
  which is the call the shadow sweep makes before every command
  (`src/native/invocation/wrapped_command_notices_snapshots_and_manual_push.rs`,
  line 503).
  An inspection command does not ask Git for the location,
  so the layout read is its source,
  and a layout the reader declines answers nothing (A12).
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
  The propagation record took "Unit on the checkout's mount (Recommended)" as the trigger,
  provisionally,
  and did not widen the event filter
  ([repair record][repair],
  "Which triggers the route adds").
  The sibling's `checkout-propagation.service`,
  upheld by a drop-in on the checkout's mount unit,
  is that trigger,
  and its `ExecStart=` is the same job request this design's hook sends
  ([sibling propagation][sibprop],
  "The trigger the route adds,
  and what it must show",
  lines 1400 to 1503,
  and "The propagation service and its drop-in",
  lines 2941 to 3067,
  read at `037bcee2e5`).
- F43.
  The wrapper's intent writer runs the compatibility gate before it opens the device:
  `CloneIntentWriter::open` calls `gate.verdict()` and then `ZfsDevice::open`
  (`clone_mount_intent_write.rs`,
  lines 135 to 142),
  and the gate reads `/sys/kernel/btf/vmlinux` and the module's BTF and walks them
  ([spec][spec],
  lines 20441 to 20520).
  Its start cost is recorded as unmeasured
  (`doc/planning/cli-git-zfs-runtime-layout-checker-design.md`,
  line 1185).
- F44.
  A property write does not return until its transaction group commits:
  `dsl_props_set` calls `dsl_sync_task` (`module/zfs/dsl_prop.c`,
  line 1053 at `zfs-2.4.4`),
  and `dsl_sync_task` ends in `txg_wait_synced` (`module/zfs/dsl_synctask.c`,
  lines 94 to 105).
- F45.
  The project chose one file per item for the slot identity store,
  created with `O_EXCL` under a staging name and linked into place,
  and put the removal of a killed writer's staging file in `doctor`
  ([store decision][store],
  "What was decided" and "Who removes a killed writer's staging file").
  Its staging names are unique per writer:
  a `.` and 32 lowercase hexadecimal digits,
  or the parent's `..parent.<random>.partial`,
  and `is_known_staging_name` is what `doctor` reads
  (`slot_identity_store_records.rs`,
  `STAGING_NAME_DIGITS` and `is_known_staging_name`).
- F46.
  The release writes the user units `monochromatic-git-policy-cli@.socket` and `monochromatic-git-policy-cli@.service`
  (`package/git-policy/cli/README.md`,
  line 91;
  the templates under `src/native/service/service_units/`).
  The service is the account's existing place to run work outside a command's process.

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
  The wrapper can make the folder,
  publish an entry's attempt with a staging name and a link,
  and unlink an entry in the account-owned Git common directory,
  because it runs as the account and the directory is the account's.
- A12.
  `known_common_dir()` answers for the repository shapes the wrapper forwards in,
  from the location it asked or from the layout it reads (fact F38);
  where it declines the layout,
  the command finds no folder and writes no nudge,
  and the other triggers recover (L18).
- A13.
  The wrapper can start the nudge helper detached as the account,
  and the helper can open the device under the agreed `mount,userprop` grant (F43).

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
  Every invocation that forwards to Git pays one `stat` of the folder,
  even a repository that will never hold a clone,
  and a repository with a folder pays a `read_dir` and a receipt read per entry;
  M8 measures both,
  and they are the price the project accepted for the command trigger.
- L17.
  An entry whose request is answered is removed by the next command's receipt read,
  before anything else is spent.
  No age removes a pending entry ("How long a lost request keeps being nudged").
  An entry whose clone is gone is removed by the helper whose write fails.
- L18.
  Where `known_common_dir()` declines the layout,
  the command writes no nudge,
  and the request waits for T1,
  T2,
  T3 or T5;
  the same is true when the folder is missing though a request is unanswered (fact F38,
  A12).
- L19.
  The record lives in the account's tree and the account is trusted,
  so it can be removed or edited;
  removing it costs the trigger and never the request,
  which its durable intent keeps.
- L20.
  The nudge never makes a command wait for the route:
  the command reads a receipt and at most spawns a detached helper,
  and the helper alone pays the gate,
  the device open,
  the property write that waits for a transaction group and the publish (F43,
  F44).
  A refusal the helper meets is recorded as the next attempt's outcome,
  so a host without the device prints nothing on the common path.
- L21.
  A request that nothing can answer is nudged for ever at one nudge per `Gmax`,
  and each nudge is one handler run and,
  under the project's third answer,
  one full pass,
  which the project accepted in its further answer ("How long a lost request keeps being nudged").
- L22.
  An add writes an entry before its intent and removes it on the answer,
  so it pays a staging write with its sync,
  a link and an unlink even when nothing goes wrong (V15's first option).
  A command killed after it links the next attempt leaves the attempt counted and a staging name that `doctor` removes;
  a helper killed before it records its outcome leaves the attempt `claimed`,
  which the next command counts.

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
  The project declined the wider scope for the propagation repair:
  "The project did not choose the wider event filter or a watcher on the kernel's mount events.
  So the event filter's scope does not gain the checkout's dataset for this purpose"
  ([repair record][repair],
  "Which triggers the route adds").
- B,
  filter by pool only:
  every `history_event` of the parent's pool passes.
  Example:
  the same sanoid snapshot passes and the pass attaches `one`;
  an event on another pool is dropped.
  Pro:
  keeps that accidental recovery.
  Con:
  same-pool events spend the bound and can hold a real request's pass.
- C,
  no filter:
  every event of every pool starts a handler and a pass.
  Example:
  every event on every pool starts a pass.
  Pro:
  the most recovery.
  Con:
  a full pass per event of unrelated datasets and pools.
- Ranking:
  A > B > C,
  because A keeps the bound for this design's requests,
  B lets a snapshot cadence spend the bound,
  and C makes every unrelated event a pass.

### V3. Which commands nudge, and how often

- A,
  adopted:
  every invocation that forwards to Git checks the folder once,
  reads the receipt for each entry,
  and asks for one nudge for an entry the receipt does not answer and whose attempt is due,
  under the decided backoff ("How often,
  and the burst").
  Example:
  with an illustrative `G0` of one second and an illustrative `Gmax` of one minute,
  both placeholders and no value shipped here,
  a loop of `git status` in a repository with one unanswered entry writes one nudge at the start,
  reads the receipt and writes nothing for the next second,
  then one nudge after the second,
  then one after two more,
  and from the cap on one per minute.
  Pro:
  any command recovers the clone,
  the negative for a repository with no folder is one `stat`,
  and the backoff keeps a loop to a rate the project is shown.
  Con:
  a command with a folder pays a `read_dir` and a receipt read,
  and `G0` and `Gmax` are unmeasured until M8.
- B,
  only the wrapper's management commands and a command that selects the clone's worktree.
  Example:
  the same unanswered entry;
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
  one nudge per entry per invocation and no gap.
  Example:
  forty `git status` runs in a loop write forty nudges for the same clone,
  and forty history events.
  Pro:
  the simplest rule,
  and no `G0` or `Gmax` to size.
  Con:
  a burst writes a burst of nudges,
  and the budget cannot hold them under the project's third answer,
  so only the merge device bounds the passes.
- Ranking:
  A > B > C,
  because A recovers on any command while the backoff keeps a burst to a rate that can be stated,
  B leaves the commands that run inert,
  and C leaves the burst to the merge device alone.

### V17. How the nudge is written without the command waiting

- A,
  adopted:
  a detached helper process,
  a hidden mode of the wrapper's own binary,
  spawned by the command that wins the entry's next attempt.
  Example:
  a command finds a due entry,
  links the next attempt and spawns the helper,
  and returns to run Git;
  the helper pays the gate,
  the device open,
  the property write and the record,
  in its own session with its streams closed.
  Pro:
  nothing of the route is on the command's path,
  and the command's status and output are untouched.
  Con:
  a new process model for the wrapper (a hidden mode,
  a spawn on the command path,
  a process a sandbox may kill),
  and the mode's name must guard against a person typing it.
- B,
  in the command's own process with a deadline.
  Example:
  the same command runs the gate and the write itself,
  racing the write against a timer and abandoning it when the timer fires.
  Pro:
  no new process and no mode name.
  Con:
  the gate's BTF walk and the transaction-group wait are unbounded syscalls a timer cannot cancel,
  so the command can still wait as long as the transaction group takes,
  which the record's cost forbids.
- C,
  handed to the account's existing service.
  Example:
  the same command writes a request for the clone to the user service's runtime folder,
  and `monochromatic-git-policy-cli@.service` runs the nudge.
  Pro:
  no new process model,
  and the service's lifecycle is known.
  Con:
  the service's own readiness and restart story becomes the nudge's,
  and a request file is one more record to publish and clean.
- Ranking:
  A > B > C,
  because A is the only option whose path holds no unbounded syscall,
  B cannot bound the two waits the record forbids,
  and C trades them for a service dependency and another record.

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
  the timeout's fixed sentence names the entry its intent wrote and the trigger that entry feeds,
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
- The sentence ships with the nudge's switch in step 22 and not in step 10:
  until the switch is on it would name a trigger that writes nothing.

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
  The folder,
  the entry written with the intent,
  the helper and the command-path check land in step 10,
  and `G0`,
  `Gmax` and `D` in step 22.

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

### V15. When an entry is written

- A,
  adopted:
  with the intent,
  before the property is written,
  and removed when the wait is answered.
  Example:
  an add writes its entry,
  then its intent,
  and is killed before its deadline:
  the entry stays,
  and the next command in that repository nudges the clone.
  Pro:
  no kill loses the trigger,
  and the waiting command's own nudge and the command trigger read one entry.
  Con:
  every add writes a staging file,
  syncs it (a ZFS log commit on every add),
  links it into place and removes it on the answer,
  with no device opened,
  even when nothing goes wrong.
- B,
  when the wait gives up,
  as the earlier revision wrote it.
  Example:
  the same killed add leaves no entry;
  the next command does nothing,
  and the clone waits for T1,
  T2,
  T3 or T5.
  Pro:
  no file for an add that succeeds.
  Con:
  a kill before the deadline leaves the missing-record state,
  and the waiting command's own nudge needs a second path.
- Ranking:
  A > B,
  because A leaves the trigger after any kill at the price of two file operations per add,
  and B leaves a killed request with no command trigger at all.

### V16. The waiting command's own nudge

- A,
  adopted:
  kept,
  one nudge after a delay `D` with no receipt answering the wait's nonce.
  Example:
  an add whose intent event was lost writes its entry with the intent,
  gets no receipt by `D`,
  and asks for one nudge;
  the clone attaches during the add,
  before its deadline.
  Pro:
  the caller's own trigger is real,
  which is the accepted proposal's nudge the wrapper writes on a caller's behalf,
  and an add pays the extra event only when its wait has no receipt by `D`.
  Con:
  one extra handler run and pass for that add,
  and `D` is unmeasured until M8.
- B,
  not kept:
  a waiting command writes no nudge of its own,
  which is what the previous revision said without listing it.
  Example:
  the same add waits its whole deadline and reports the timeout;
  the next command in the repository recovers the clone.
  Pro:
  one event per add.
  Con:
  a lost intent event costs the caller its whole deadline,
  and a repository with no later command leaves the clone to another trigger.
- Ranking:
  A > B,
  because A gives a waiting caller its own recovery at one event on the waits that need it,
  and B leaves that caller to its deadline.

## Dependencies on other designs and on the plan

- D1.
  The propagation sibling ([sibling propagation][sibprop]),
  read at `037bcee2e5`
  (2026-11-22T20:48:10-05:00),
  and its record ([repair record][repair],
  accepted 2026-11-22T19:23:30-05:00,
  with its further answer on a shared and slave mount recorded 2026-11-22T20:37:32-05:00).
  The two records settle what this design cites the sibling for:
  the lost-event record accepts the boot unit's `ExecReload=` line and the
  `systemctl --no-ask-password reload-or-restart` command,
  and the repair record takes the checkout mount unit as the trigger,
  provisionally,
  and declines the wider event filter
  ([repair record][repair],
  "Which triggers the route adds").
  The repair record's further answer that a checkout which is shared and slave counts as shared
  touches no statement here,
  because this design only cites the record for the trigger and the filter.
  The sibling states all of this as accepted
  ("The project's four answers,
  and what this design does with each",
  lines 249 to 378)
  and its two pointers to this design are by commit,
  accurate at its tip `b6855dce15`,
  which this design has since moved.
  The order inside both of its entry points
  ("Where it runs,
  as whom,
  and in what order",
  lines 1566 to 1644)
  is the order this design cites and states no other:
  configuration and checkout,
  its propagation step,
  the probe record,
  the pool and checkout checks,
  the sweep;
  this design's taker runs those prerequisites first and takes its claim only after them.
  The sibling's service and drop-in are that trigger's files
  ("The trigger the route adds,
  and what it must show",
  lines 1400 to 1503,
  and "The propagation service and its drop-in",
  lines 2941 to 3067;
  F42),
  and its `ExecStart=` is the same job request this design's hook sends.
  Its A3 (lines 1113 to 1134) and its dependency bullet (lines 3303 to 3380) read this design's hook as two
  files asking the boot unit,
  which is what this design states;
  its L11 (lines 1250 to 1263) keeps one note for a machine whose boot unit lacks the line.
  Its F31 (lines 920 to 934) relies on this design's filter keeping the checkout dataset's events out,
  which V2's first option does and the repair record confirms.
  Its F22 (lines 817 to 830) is the authorization-agent fact the records section cites.
- D2.
  The forced-removal sibling ([sibling forced][sibforced]) at `2dda3d993e`
  assumes "each removal request is one nudge-shaped property write"
  and that the handler may "coalesce sweeps without merging two nonces" (its D5,
  lines 1789 to 1790 there).
  At its current tip `6871053b79` the quoted assumption is no longer carried,
  and the design says it "adds no nudge",
  which is the state this design's entry written with the intent changes.
  This design
  writes the intent and,
  under V3,
  one nudge per forwarding command per gap,
  and the project's third answer exempts a pending clone's pass from the budget,
  so a removal's waiting pass is never held.
- D3.
  The cancelled-removal sibling ([sibling cancelled][sibcancelled]) at `797a7cf2d6` lists
  "a later sweep nudge on that clone" among its triggers (line 974 there;
  the line is line 1795 at its current tip `2c70b9203c`).
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
  which gains the folder of unanswered requests,
  the entry written with the intent,
  the command-path check,
  the helper and the nudge's switch;
  step 14's,
  which gains the removal's entry and its removal on settle;
  step 18's,
  which gains the folder's readings and the removal of the known staging names;
  step 22's
  "no existing constant changes" line,
  which gains `pass-budget`,
  the rule,
  the record's first gap `G0`,
  its cap `Gmax` and the wait's delay `D`;
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
  a section for the folder of unanswered requests:
  its name in the Git common directory,
  its owner and mode,
  the entry's format,
  the staging name,
  the one-page read and the no-follow open,
  the receipt read that removes an answered entry,
  the command path's check,
  the helper's mode and the outcome words of a refused attempt;
  the timeout sentence of V11,
  which ships with the nudge's switch in step 22;
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
  `doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
  whole file,
  and `doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
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
  `lock/`,
  `binary/`,
  `invocation/` and
  `repo_read/`.
- `package/git-policy/cli/Cargo.toml`,
  the two `[[bin]]` entries and the handler's installed name.
- Sibling design documents,
  with `git show <branch>:<path>`:
  the private propagation document at `037bcee2e5`,
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
  `module/zfs/dsl_prop.c`,
  `module/zfs/dsl_synctask.c`,
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
- 2026-11-22T20:32:51-05:00:
  the review of the revision after the answers.
  `main` was merged first,
  at 2026-11-22T20:25:49-05:00,
  bringing the propagation repair record.
  The nudge on Git commands is designed again:
  the record is a folder of one file per clone in the Git common directory,
  named by the clone's GUID and attempt,
  published by a staging name and a link,
  with the staging name removed by `doctor`;
  a command reads the receipt by GUID and removes an answered entry before it spends anything else;
  the entry carries an attempt number and the gap doubles from `G0` to `Gmax`;
  no age removes a pending entry;
  the gate,
  the device and the transaction-group wait are the detached helper's,
  and the command waits only for a spawn;
  a refused device is recorded in the next attempt and prints nothing on the common path.
  The waiting command's own nudge returns,
  after a delay `D`,
  which the previous revision had dropped without listing it,
  and V16 lists both options.
  V14 (how long a lost request is nudged) and V15 (when an entry is written) are new,
  V3 now carries the backoff,
  and V2 lost the checkout-dataset option the repair record declined.
  The hook section,
  F38,
  F42,
  D1,
  D2 and D3 cite the sibling at `c7708953c0` and the repair record,
  D8 and D9 name the folder,
  the helper,
  the attempt words and the switch,
  tests 15 to 20 are rewritten and test 8's Git arm loses its confound,
  and M8 carries the answered-entry,
  gate-control and root-side arms.
- 2026-11-22T20:32:59-05:00:
  recorded the answer's commit time above.
- 2026-11-22T21:00:23-05:00:
  the diff check after the answers,
  and the project's further answer.
  `main` was merged first,
  at 2026-11-22T20:56:33-05:00,
  bringing the further answer in the lost-event record and the repair record's answer on a shared and slave mount.
  How long a lost request keeps being nudged is decided:
  V14 left the veto list,
  "The project's three answers" carries the fourth answer,
  and "How long a lost request keeps being nudged" states it,
  with the record's "once a minute" as an example and the first and longest gaps left to M8.
  The record's shape is fixed along the slot store's own:
  a unique staging name of a dot and 32 hexadecimal digits per writer,
  removed after the link or by `doctor` when a writer was killed;
  the command links the next attempt and only the winner spawns the helper,
  which replaces its attempt's file with the outcome;
  a wait claims its nudge through the same link.
  A second request on a clone removes its older entries,
  and the helper drops an entry whose intent is absent or names another nonce,
  each stating what it proves.
  An empty folder is removed with its last entry,
  any failure on the command's read path is silent,
  the command path is compiled out with the switch,
  and the one-page bound and V15's sync are stated.
  V17 lists how the nudge is written without the command waiting,
  and V3's example numbers are marked placeholders.
  The sibling statements cite the propagation design at `037bcee2e5`,
  the repair record's further answer touches nothing here,
  and facts F45 and F46 carry the staging names and the user service.
- 2026-11-22T21:00:30-05:00:
  recorded the answer's commit time above.
- Any commit touching this file after the entries above is newer than all of them,
  and `git log --format=%cI -- doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`
  lists it with its committer time.

[decision]: ../decision/cli-git-zfs-worktree-design.md
[answers]: ../decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md
[repair]: ../decision/cli-git-zfs-checkout-propagation-repair.md
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
