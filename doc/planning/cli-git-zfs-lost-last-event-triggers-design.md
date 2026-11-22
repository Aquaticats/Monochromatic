# Which triggers a lost last event waits for, and how nudges are merged

## Status

Non-final and unapproved.
No reviewer has read this revision,
and the project has not read it;
only the project declares a design accepted.
A cross-vendor gate review read the version at `641fbc1ad4` and returned "fit after the listed fixes,
not as it
stands" (`/home/user/temp/agent/gate-review-zfs-lost-last-event-triggers-design-2026-11-22/keep/review.md`).
This revision answers every finding in it;
"Revisions" lists the commits and what each changed.

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

Four things are undesigned.
The triggers are named but not described:
who fires each,
when,
as which user,
what each
guarantees and which lost event each recovers.
The no-waiter case has no statement of what happens between the loss
and the next trigger.
The provider hook is "proposed and unrun" and has no shape.
The merge and rate rule have no
shape at all.
This design answers those four,
and nothing else.

## The project's word "moot" is put to the project

The answer's word "moot" can carry at least two readings:
that a waiting caller always has a trigger of its own,
so
the old objection to triggers-only does not apply;
or that a lost event with nobody waiting is recovered soon enough
not to matter.
The first reading is true of a caller that waits,
because its own nudge is a trigger (T4).
The second
is not something this design can show:
with no timer,
the next trigger can be the next boot.

This design does not choose between them.
"Questions only the project can answer",
question Q-B,
puts the reading to
the project with its options and a ranking,
and nothing in this document depends on the answer beyond the statement
that no timer exists,
which is the project's recorded decision.

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
property a waiter writes:
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
"Where it runs,
as whom,
and in what order" and the paragraph after its refusal
quote).
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
    the three cost records and the timeout sentence,
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
T3 is the option this design recommends for the provider hook;
the project's answer to Q-A
decides whether it exists.

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
  `reload` once the sibling's `ExecReload=` exists (T5).
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
  once step 19 installs the drop-in (Q-A's option B).
- When:
  whenever the provider starts as a job,
  which on this host is the boot and any later start after the unit was
  stopped.
  A `start` of the already active unit does nothing (fact F15),
  and a `restart` restarts the boot unit
  through `Requires=` and so reconciles even without the hook (fact F17).
- Runs:
  `/usr/bin/systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service`,
  which reloads
  the active unit through the sibling's `ExecReload=`,
  starts it when it is inactive,
  and restarts it when it is failed
  (`systemctl(1)`,
  `reload-or-restart`,
  lines 530 to 537 at tag `v262`).
  The hook itself runs no handler and makes no
  pass.
- As which user:
  root,
  through the service manager,
  which runs the unit in its own namespace and journal.
- Guarantees:
  one enqueued job for the boot unit,
  and never a failed provider:
  the hook is prefixed `-` and the
  command it runs is a job request,
  not a reconcile (Q-A's cons list the rest).
- Recovers:
  a boot whose reconcile refused because the checkout was not ready,
  once the provider is started again,
  and
  any request written between the two.
- It cannot see a checkout mounted by a hand-run `zfs mount`,
  because no unit runs then.
  T5 is that case's lever.
- This is Q-A's option B and does not exist until the project answers.

### T4: the wrapper's clone-local nudge

- Fired by:
  the wrapper,
  on the account's behalf,
  by writing the clone's
  `org.monochromatic:git-worktree-clone-mount-sweep-nonce` property with a fresh nonce.
  The writer exists
  (`clone_mount_intent_write.rs`,
  `nudge_property`),
  and no command calls it yet ([spec][spec],
  lines 20580 to 20595);
  plan step 10's add route and step 14's removal are the callers.
- When:
  once per wait,
  after the wait's measured delay `D` has passed with no receipt answering its own nonce,
  and
  after the wait's first read,
  which is at its start (fact F20).
  The nudge policy is V3,
  and `D` is owed to step 22
  (measurement M7).
- As which user:
  the account writes the property,
  which its `mount,userprop` grant on the parent's descendants allows
  ([proposal][proposal],
  "Scoped delegation",
  line 338);
  the delivery and the pass are root's,
  as T1.
- Guarantees:
  one further history event,
  hence one more delivery attempt of T1,
  and nothing more.
  It carries no state
  the handler reads.
- Recovers:
  a request whose own event was lost but ZED is alive and the queue did not drain the nudge too,
  and a
  request written after a running pass had read that clone.
  It recovers nothing while ZED is stopped,
  which is a
  likely cause of the first loss.
- Recorded where:
  the handler's line,
  which needs V8's answer.

### T5: a trigger the project fires

- Fired by:
  the project,
  with the commands `doctor` prints,
  and run by nobody the design starts.
- The root lever,
  recommended:
  `systemctl --no-ask-password reload reconcile-git-worktree-clone-mounts.service`,
  which runs the sibling's
  `ExecReload=` in the unit's own namespace and journal
  ([sibling propagation][sibprop],
  "What becomes of `Type=oneshot` with `RemainAfterExit=yes`",
  lines 1269 to 1290).
  `start` re-runs the unit when it is failed or inactive;
  `restart` runs the stop pass first and
  unmounts every clone (fact F17).
  Running the handler binary from a person's shell is not recommended,
  because that
  shell's mount namespace is not the host's (fact F18).
- The account lever:
  a `zfs set` of a fresh nudge nonce on the clone,
  which is T1's delivery and therefore ZED-bound:
  while ZED is stopped it recovers nothing.
- As which user:
  root for the reload,
  the account for the nudge.
- Guarantees:
  one pass in the unit's own context for the reload,
  and exactly T1's guarantees for the nudge.
- Recovers:
  any lost event when ZED is alive (the account lever),
  and any lost event including a stopped ZED (the root
  lever).
- Recorded where:
  the unit's journal for the reload,
  and V8's answer for the nudge.
- Q3 of the earlier revision is replaced by Q-A and by the levers printed in "What doctor reports";
  the choice of what
  `doctor` prints is V9.

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
the wrapper only
waits (fact F20).
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

## The no-waiter case

A clone carries a durable intent,
its last event was lost by one of the ways in "ZED's delivery",
and no caller waits:
the add that wrote the intent timed out and reported,
or the last caller exited long ago.
Which trigger fires next,
and when:

- T1 at the next history event of any dataset under the pinned parent,
  or at the next ZED start,
  which replays what the
  queue holds.
  With ZED running this is the earliest recovery;
  with ZED stopped the next start delivers what was not
  drained,
  and a drain while ZED was stopped is silent (fact F12).
- T3 at the next start of the checkout provider as a job,
  which is rare on this host because the unit stays active.
- T2 at the next start of the boot unit,
  which the next boot always causes,
  and a failed boot unit can be re-fired with
  `start` (fact F15).
- T5 whenever the project fires one of its two levers.
  The root lever works with ZED stopped;
  the account lever does
  not.
- T4 only if some wrapper wait happens on that clone,
  and after this design every such wait writes one.

No trigger here is a timer,
and the decision keeps it that way:
"Periodic reconciliation without a waiter is not part
of this route" ([proposal][proposal],
lines 537 to 539).
So how long the no-waiter case lasts is "until one of those
occurs",
which in the worst case is the next boot;
whether that is what the answer's "moot" accepted is Q-B,
and this
design states no bound.

The clone's state meanwhile is not lost and not silently changed:

- Its durable intent stands,
  because nothing but its owner or a removal clears it,
  and a timeout is not cancellation
  ([proposal][proposal],
  lines 792 to 794).
- Its slot,
  if the add registered it,
  holds the Git registration and the pointer the add wrote,
  and the clone's dataset
  is not attached there,
  so the folder shows what the underlay holds.
- The receipt for the clone,
  if any,
  answers an earlier request,
  so a reader sees a stale result rather than a wrong
  answer ([spec][spec],
  lines 6131 to 6183).
- This boot's readiness record names whatever the last readiness trigger published,
  or does not exist;
  a caller is
  either admitted or refused by the rule that reads it,
  never admitted on a guess (fact F14,
  F16).
- Nothing removes the clone,
  mounts it elsewhere,
  or changes its intent while it waits.
  The next trigger's pass is the
  only writer of its mount state.
- The filter's trade applies:
  an event outside the parent that would have started a pass is dropped (V2),
  so the next
  trigger might be later than it would be without the filter.

`doctor` reports that state as a clone whose live intent has no receipt answering it (section "What doctor reports for
a clone whose last event was lost") and prints the two levers,
so a project that wants the worktree now has a trigger
the design wrote down.

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
    The verbs that run it again are `restart`,
    which restarts
    the boot unit through `Requires=` and so runs its `ExecStopPost=` `shutdown-unmount` first,
    unmounting every
    attached clone before the hook's pass puts them back (fact F17),
    and `reload`,
    which needs the sibling's
    `ExecReload=`.
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

### The shape this design recommends

Q-A's option B,
and the only shape that avoids all five:
the drop-in's line is
`ExecStartPost=-/usr/bin/systemctl --no-block reload-or-restart reconcile-git-worktree-clone-mounts.service`.
The hook
asks the unit that already owns the ordering,
the namespace,
the probe,
the journal and the stop pass to reconcile;
the provider itself is never failed,
never carries the pass on its critical path,
and never needs a second verb to be
re-fired.

What it runs is a job request:
`reload-or-restart` reloads an active unit through `ExecReload=`,
starts an inactive
one,
and restarts a failed one (`systemctl(1)`,
lines 530 to 537 at tag `v262`).
Each of the three unit states needs a
guest arm (test 12).

Sharing one mechanism with the sibling:
the propagation sibling already proposes `ExecReload=<the same handler>
boot-reconcile` on the boot unit as its veto choice V4,
and names `systemctl reload` as its manual lever
([sibling propagation][sibprop],
lines 1269 to 1290 and 1955 to 1965).
This design's Q-A option B uses exactly that
line:
one `ExecReload=`,
two consumers (the hook's job request and the project's printed lever).
The sibling's answer
is unreviewed and not adopted here;
if its V4 is vetoed,
Q-A option B loses its reload arm and only its
start-when-inactive arm remains,
and Q-A's ranking changes.

What it needs from the project,
outside the wrapper,
and where each part is recorded:

- The provider integration question Q-A settles,
  recorded in this design's answer to it and in the decision record the
  acceptance writes;
  until then the first-ranked route is the one above.
- The handler binary at `/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets`,
  installed by
  plan step 19 and recorded in `SPEC.md`'s handler paragraph ([spec][spec],
  lines 262 to 270).
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
- The drop-in at `/etc/systemd/system/zfs-mount.service.d/reconcile-git-worktree-clone-mounts-after-checkout.conf`,
  with the one `ExecStartPost=-` line,
  and the unit's `ExecReload=` line.
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

### The shape this design does not take, and why Q-A still lists it

A `.mount` unit for the checkout with a companion unit bound to it (Q-A's option C) fires on every activation of the
checkout mount and gives the checkout a unit state `doctor` can read,
but it changes how the checkout is provisioned
and needs the checkout excluded from `zfs mount -a`,
which is a larger design than this open item.

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
- `recent-starts`,
  a root-owned regular file of mode `0644` holding the last `B` passes' start times,
  one per line,
  each
  a count of seconds from `CLOCK_BOOTTIME`,
  which includes suspend,
  followed by the same moment as an ISO 8601
  timestamp with its offset.
  It is replaced whole,
  never appended.

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
lines 103 to 145);
then the default mode,
which reads the configuration,
takes the run
claim,
reads or writes `recent-starts`,
then reads the probe record,
then the propagation step
([sibling propagation][sibprop],
"Where it runs,
as whom,
and in what order"),
then the pool and the checkout,
and then
the pass.
A held run has paid the gate and nothing else.

### What a taker does

A handler that may pass,
which is T1's or T4's delivery,
does this in order:

1.  Take the run claim with the repository's numbered claim (fact F22).
    If it is held,
    go to step 8.
2.  Read `recent-starts`.
    If the last `B` starts are all inside the last `P` seconds,
    write `another-pass-is-due`,
    release the claim,
    print the held line naming the bound and the last start,
    and exit 2,
    because it did no work.
3.  Rewrite `recent-starts` with this pass's start appended and the oldest entry dropped when the list is full.
4.  Remove `another-pass-is-due` if it is there:
    this pass covers everything marked before it began.
5.  Run one pass:
    the probe record,
    the propagation step,
    the pool and checkout checks,
    the enumeration,
    each clone's
    turn under its slot claim,
    and the receipts.
6.  Release the claim.
7.  Look for `another-pass-is-due`.
    If it is absent,
    report and exit 0 or 2 as the pass's own outcome.
    If it is there,
    take the claim once more and do steps 2 to 6 once more,
    without step 7;
    a mark that survives the second pass is
    left standing for the next taker,
    and the line says so.
8.  When step 1 found the claim held:
    write `another-pass-is-due`,
    then try the claim once more.
    If that second try
    succeeds,
    continue at step 2,
    which removes the mark the handler wrote.
    If it fails,
    print the held line and exit 2.
    The second try never waits (fact F22).

A readiness trigger,
T2 or T3's unit run,
does steps 4 and 5 only:
it never takes the run claim,
never reads or writes
`recent-starts`,
and is never held (V5's first option).
Its pass may overlap an event-driven pass,
which the per-slot
claim makes safe for each clone (fact F24),
at the cost of one extra pass per readiness transition.

### What the marks guarantee, and what they do not

The sequence holds in the interleavings traced by the review for two and three handlers:
a mark written before the
running pass's release is seen by that pass's look,
and a mark written after the look finds the claim free for its own
second attempt.
Two things are weaker than an earlier revision claimed:

- A mark written while the second pass runs is left standing for the next taker,
  and that is the cap in step 7,
  not a
  bug:
  with one nudge per wait,
  an event and its nudge can both be marked and left,
  and the wait's deadline is the only
  report.
  L3 states it.
- A mark written by a rate-refused handler,
  step 2,
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
`recent-starts` is
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
and one that cannot read `recent-starts` passes and
prints that the bound was not applied (V7's first option).

### The wrapper's nudge policy

The wait reads,
then checks the deadline,
then sleeps (`mount_request_wait.rs`,
lines 326 to 359),
so a fresh request's
first read is at its start and cannot match.
A nudge written then is a nudge at the start:
two events per add within
milliseconds,
sharing the intent's loss window,
which is the case the policy's own cost argument avoids.
V3's options
are,
in this design's ranking,
C a single nudge after a delay `D` set from the measured wait,
A at the start,
E none,
D at the deadline.
`D` is owed to step 22 (measurement M7),
the wrapper writes at most one nudge per wait,
and the
nudge uses a fresh nonce so the write is a change (fact F18).

## The rate rule's shape, with no number

### What is bounded

Bounded:
the passes that T1's deliveries and T4's nudges start,
in the unit Q-D chooses,
a count `B` in a window `P`
or a budget of pass time in a window,
with every number owed to step 22.

Not bounded,
each for a stated reason:

- The handler invocations ZED forks.
  ZED forks before any of this route's code runs,
  so the bound cannot reach it;
  the
  16-process ceiling and the queue bound it instead (facts F10 and F12).
- The account's ability to write the property,
  which the proposal says the handler cannot limit ([proposal][proposal],
  lines 575 to 578).
- Readiness passes,
  V5's first option,
  because a cost rule must not decide whether callers are admitted.

### What a caller sees when a pass is held back

The intent and the nudge stand and no receipt answers the wait,
so the wait ends at its deadline with the built text,
which names the request,
the last observation,
`doctor` and the journal (fact F20).
V11 adds one sentence about the
cost rule and the `doctor` report,
and the waiting path gains no read.

### The measurement that sizes it

Step 22 owns the numbers ([plan][plan],
"Step 22",
lines 2340 to 2400).
This design fixes what the measurement must
produce:

1.  Measure a pass's cost at two populations,
    the acceptance workload's clone count and one more than the child bound,
    interleaved in the same round with a control arm of the same binary running the same workload with the device's
    decisions compiled out,
    and report the spread of repeated rounds before any difference is called one (rule
    `QNB`).
2.  Measure the passes one add starts and the history events one add produces,
    with the device on and off,
    because the
    add also snapshots and clones and those are in-scope events (low finding L-2).
3.  Measure the caller cost of a candidate rule:
    the acceptance workload's waits that end at the deadline,
    with the
    rule absent,
    with the candidate,
    with Q-C's spare-the-answered-clone rule,
    and with a positive control of one pass
    per hour,
    which must produce a held pass and a timed-out wait with V11's sentence.
    If the positive control does not,
    a zero is not read as none.
4.  Hand the resource and the threshold to the project (Q-D);
    the measurement reports what each candidate would allow,
    not which is acceptable.
5.  Measure lost-event behaviour and trigger recovery separately (M1 to M6),
    because no bound on passes bounds
    recovery.

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
only acting service-manager commands it names are the printed levers of T5 and Q-A's hook line,
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
- The last starts in `recent-starts`,
  and the rule in force,
  or,
  before step 22 ships it,
  the sentence that the rule is
  not in force because its measurement is owed.
- The handler's last decisions,
  from V8's answer;
  a ZED-started handler's own lines are not in any journal today (fact
  F13),
  so today the report points at ZED's `Finished` lines for exit statuses and at nothing for the handler's lines.
- The service-manager facts:
  V9 decides whether the report prints the `systemctl` commands or reads their answers.

The lost-event block,
with concrete values:

```text
clone zpcachyos/nosnap/monochromatic-git-worktree-clone-datasets/one
  intent   v1:6f1c0a9d8b7e5f4a3c2b1d0e9f8a7b6c:mount, source the clone
  mount    absent at /home/user/Monochromatic/.claude/worktrees/one
  receipt  1234567890123456789: none, so no pass has answered this request
  ready    boot 5d0f..., pool ready, checkout ready, sweep finished at 2026-11-22T17:29:58-05:00
  runs     no pass is running, as far as the account can read; a pass is due since 2026-11-22T17:30:05-05:00
  events   the last event for this clone may be lost; the next triggers are a ZED event or start, a provider start, or a lever you fire
  fix      as root: systemctl --no-ask-password reload reconcile-git-worktree-clone-mounts.service
  fix      as you: zfs set org.monochromatic:git-worktree-clone-mount-sweep-nonce=<32 fresh lowercase hex digits> <clone>
  journal  journalctl -u reconcile-git-worktree-clone-mounts.service ; journalctl -u zfs-zed.service
```

The lines reuse the route's wording for a stale result,
a claim and a readiness refusal.
The two `fix` lines are T5's
levers,
and `doctor` never runs them;
the account lever works only while ZED is alive.
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
  the nudge call and V3's policy ([plan][plan],
  "Step 10",
  lines 1399 to 1587).
- Step 14,
  removal:
  the same policy for a removal's waits,
  so a lost `unmount` event is recovered by the same
  triggers;
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
  and the correction of the four shipped sentences that name step 10 ([plan][plan],
  lines 2017
  to 2120).
- Step 19,
  the units and the zedlet:
  the event filter,
  the provider drop-in and `ExecReload=`,
  enabling
  `zfs-zed.service`,
  the boot unit as the plan describes it,
  and the merge device with its mark (V12's first option).
- Step 22,
  the measurements:
  `recent-starts`,
  the rule in Q-D's unit,
  and its numbers with the measurement beside them.

The device and the mark land in step 19 because step 19 enables ZED,
and without them every event starts its own pass;
the record and the number land in step 22 because the number is a limit that rule `LQM` lets no step ship unmeasured
(V12).

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
    `recent-starts` holding `B` starts inside
    `P` must produce the held line,
    no receipt and the mark;
    holding `B` starts with the oldest outside `P` must pass.
6.  The rule's positive control repeats test 5 with the smallest admissible value and asserts a wait's deadline report
    carries V11's sentence,
    so the arm is proved able to show a hold (M3's control).
7.  Unreadable cost records,
    host:
    the claim folder unreadable,
    `recent-starts` corrupt,
    and the mark a directory.
    Each
    must still pass,
    print which record could not be read,
    and leave the clones' state alone (V7's first option).
8.  A lost event and its recovery per trigger,
    guest,
    in separate arms:
    stop ZED,
    write an intent,
    assert no receipt;
    then fire exactly one thing per arm,
    which are a ZED start,
    a nudge with ZED running,
    a boot-unit `start`,
    a
    provider `start`,
    and a reload,
    and assert a receipt under the intent's nonce appears in each arm.
    The ZED-start arm
    proves the replay;
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
    guest:
    a boot with the hook and the boot unit;
    the unit active and reloaded;
    the
    unit failed and `reload-or-restart`;
    the unit inactive and started.
    Each asserts the provider stays active,
    the
    reconcile ran in the boot unit's journal,
    and no clone was unmounted by a reload (Q-A's option B;
    fact F17).
13. The provider's late readiness,
    guest:
    boot with the checkout's key unloaded so the boot's reconcile refuses and
    publishes no record,
    assert a caller is refused and that no readiness record for this boot exists,
    load the key,
    start the provider,
    and assert the hook's job published readiness and attached the clone without another add (the
    proposal's unrun item,
    [proposal][proposal],
    lines 2024 to 2030).
14. `doctor`'s lost-event report,
    host over planted records:
    one clone with a matching receipt,
    one with a stale
    receipt,
    one with none,
    one whose record belongs to another boot,
    and no record at all.
    Assert the exact lines,
    the
    two printed levers,
    and that nothing was run.

Tests 1,
3,
4,
5,
6,
7 and 14 run on the host.
Tests 2,
8,
9,
10,
11,
12 and 13 need the guest,
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
  a boot-unit `start`,
  a provider `start`,
  a reload) and record
  the time from the trigger to the receipt.
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
  the rule's caller cost.
  Probe:
  the acceptance workload with a candidate in Q-D's unit,
  counting waits that end
  at the deadline;
  positive control the smallest admissible value;
  control with the rule absent;
  and one arm with
  Q-C's spare-the-answered-clone rule,
  which needs one clone read per event.
  Decides:
  the numbers and Q-C.
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
  for Q-A's option A,
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
- M7,
  the nudge's delay.
  Probe:
  the acceptance workload's waits,
  recording each wait's receipt latency and the point at
  which a nudge after delay `D` would have been written,
  for several candidate `D`;
  control the same workload with no
  nudge.
  Decides:
  `D`,
  as the smallest delay that leaves a fast wait alone while still catching a slow one.

## Counts and numbers this design does not ship

Each is a count the design uses and cannot size,
with what would size it.
None ships before its measurement or the
project's decision.

- One extra pass per admitted run.
  Sized by M4's pass cost and by whether the second pass is worth its cost at the
  acceptance population;
  the shape is "What a taker does" steps 7 and 8.
- One second claim attempt after a mark.
  Sized by the same measurement,
  because it is one `take_claim` call on the
  contended path;
  the shape is "What a taker does" step 8.
- One nudge per wait.
  Sized by M7,
  which also gives its delay.
- `B` in `P`,
  or the budget of pass time,
  in Q-D's unit.
  Sized by M3 and decided by Q-D.
- The pass's child bound of 4096 is already shipped with the merged measurement named in fact F28,
  and nothing here
  changes it.

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
  a reload needs an `ExecReload=` and is
  admitted only in the running or exited states (`src/core/service.c`,
  lines 3762 to 3790);
  and `reload-or-restart`
  reloads,
  starts or restarts as the state needs (`systemctl(1)`,
  lines 530 to 537 at `v262`).
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
  stopped (or restarted)",
  so a restart of the provider restarts the boot unit and runs its `ExecStopPost=`
  `shutdown-unmount` first (`systemd.unit(5)`,
  `Requires=`,
  lines 681 to 694 at `v262`;
  [proposal][proposal],
  line 525).
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
  The earlier revision carried this as a fact;
  it is an inference and is an assumption here.
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
  it is unprobed here (Q-A's option B) and
  the review did not run it either.
- A10.
  `recent-starts` has one writer at a time,
  because only an event-driven pass writes it and every such pass holds the run
  claim.

## Limitations

- L1.
  The no-waiter case has no time bound:
  the next trigger can be the next boot,
  and no timer exists by the project's
  decision.
- L2.
  A caller can time out because a pass was held back by the cost rule,
  and its request then waits for a trigger the rule
  admits.
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
  Q-A's option B spends one action and one unit job.
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
  and the root lever depends on the sibling's `ExecReload=` if it is
  the reload;
  without it,
  `start` after a failed unit and `restart` with its stop pass remain.
- L14.
  The route-wide claim narrows the accepted parallel-pass sentence ([proposal][proposal],
  lines 561 to 562),
  and the project decides in V4.

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
  so a pass can wait for a slot there (fact F10),
  which the earlier revision
  denied.
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
  is the
  parent or below it;
  drop everything else with one line and no device open.
  An unresolved parent passes the event on,
  so the handler's own checks run and refuse or pass on live facts;
  it never fails closed.
  Example:
  a sanoid snapshot of `zpcachyos/ROOT/cos/home` while clone `one` has an unanswered request is dropped;
  a
  `zfs set` on `one` is accepted.
  Pro:
  the bound stays for real requests,
  and a pass is not spent on another dataset's event.
  Con:
  it gives up the
  accidental recovery an out-of-scope event would have made (L5),
  and an unresolved parent costs one gate run.
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
  because A keeps the bound for real requests while B lets a snapshot cadence spend it,
  and B keeps
  some recovery where C makes every unrelated event a pass.

### V3. Where a waiting command writes its nudge

- C,
  adopted:
  one nudge after a delay `D` has passed with no answering receipt,
  `D` owed to M7.
  Example:
  an add writes its intent at 17:30:04,
  its receipt does not appear,
  and at 17:30:04 plus `D` the wrapper writes
  one nudge;
  a fast add writes none.
  Pro:
  one extra event only on a slow wait,
  and a loss window independent of the intent's own first milliseconds;
  a fast
  add pays nothing.
  Con:
  `D` is unmeasured,
  and a wait answered just before `D` still writes one nudge.
- A,
  at the start,
  after the first read:
  the policy the earlier revision described by mistake,
  and the one the wait's
  code does if a nudge is written on its first read.
  Example:
  every add writes two events within milliseconds.
  Pro:
  simplest,
  and no timer to size.
  Con:
  doubles every add's events and shares the intent's loss window,
  so it barely
  widens recovery.
- E,
  none:
  T4 does not exist.
  Example:
  a lost intent event is recovered by T1's next event,
  T2,
  T3 or T5 only.
  Pro:
  one event per add,
  nothing to bound.
  Con:
  the answer's own trigger set loses one of its three,
  and a caller has
  only its deadline.
- D,
  at the deadline:
  one nudge when the wait gives up.
  Example:
  an add times out at 17:30:34 and writes its nudge as it reports,
  which may attach the clone after the add has
  failed.
  Pro:
  no cost on any wait that succeeds.
  Con:
  it arrives when the caller has already been told the add failed.
- Ranking:
  C > A > E > D,
  because C keeps recovery while sparing fast waits,
  A recovers little for twice the events,
  E
  drops a recorded trigger where A at least keeps it,
  and D helps only after the failure it exists to prevent.

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
  `recent-starts`,
  and is never held.
  Example:
  a late key load,
  with the bound spent a second earlier:
  the reloaded unit publishes readiness and attaches
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
  (content `v1`) and `recent-starts`.
  Example:
  `ls -l` shows the generations,
  the mark and the starts file,
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
  an extra level whose reason an earlier revision got wrong,
  because
  the claim already ignores other names.
- C,
  the mark inside the readiness record and the starts inside the receipts folder.
  Example:
  a `pass_due` field in readiness and a file per start under the receipts folder.
  Pro:
  two folders instead of one.
  Con:
  a published record changes format for a cost device,
  and counting starts needs
  a listing.
- Ranking:
  A > B > C,
  because A needs no subfolder for a reason the code does not require,
  and C changes a format and
  adds a listing.

### V7. A cost record that cannot be read never refuses a pass

- A,
  adopted:
  an unreadable claim folder,
  mark or starts record is reported and the pass continues.
  Example:
  `recent-starts` corrupt:
  the handler prints that the bound was not applied,
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
  a held event leaves `held: the cost rule allowed 4 starts in 600 seconds` in
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
  which is what the review found.
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

### V11. The timeout text points at `doctor` instead of reading the cost records

- A,
  adopted:
  the timeout's fixed sentence names the possibility of a held pass and the `doctor` report,
  and the wait
  reads no cost record.
  Example:
  a held add's report ends with "a pass can be held back by the route's cost rule;
  `doctor` reports the last
  starts and whether a pass is due".
  Pro:
  no read on the waiting path,
  whose first read the step 7 measurement timed at 36 to 64 microseconds (F30).
  Con:
  the report cannot say the rule is why this wait ended.
- B,
  read `recent-starts` on the timeout and name the rule as the cause.
  Example:
  the same report says "the cost rule allowed 4 starts in 600 seconds,
  and this trigger was held".
  Pro:
  the actual cause.
  Con:
  a root-owned record gains a reader on a path every timeout takes,
  and the sentence changes
  when the numbers do.
- Ranking:
  A > B,
  because the held case is rare and `doctor` exists for the explanation.

### V12. Which step builds the device and the rule

- A,
  adopted:
  the merge device and the mark in step 19,
  with the zedlet and the units;
  `recent-starts` and the number in
  step 22.
  Example:
  after step 19 a held event prints `held`;
  after step 22 `doctor` prints how many of `B` starts remain.
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
  the device in step 19 with a provisional bound,
  as step 7 shipped a provisional deadline.
  Example:
  from step 19 a provisional `B` holds passes.
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

## Questions only the project can answer

### Q-A. Which trigger runs after the checkout becomes ready, and what may a refusal of it do to the mount provider?

Context:
the proposal obliges the checkout provider to trigger reconciliation after post-key-load readiness
([proposal][proposal],
lines 546 to 550).
On this host the provider is `zfs-mount.service`,
a oneshot that stays active;
its start on an active unit does nothing (F15);
the boot unit requires it (F17);
and `boot-reconcile` refuses with exit
2 whenever the key,
the checkout or this boot's probe record is not ready (F19,
F16).

- A.
  A direct drop-in `ExecStartPost=-.../mount-git-worktree-clone-datasets boot-reconcile` on `zfs-mount.service`,
  with the dash prefix.
  Example:
  at boot the hook refuses because no probe record exists for this boot;
  the journal under `zfs-mount.service`
  shows exit 2 and the provider stays active;
  `zfs mount -a` also runs the pass only if it exits zero,
  and the hook lands
  before `local-fs.target`.
  At 09:00 a `systemctl restart zfs-mount.service` re-fires it,
  restarting the boot unit too,
  whose stop pass unmounts every clone before the hook's pass re-attaches them.
  Pro:
  one drop-in file,
  no `systemctl` call from a unit,
  no dependency on another design.
  Con:
  it refuses at every boot;
  it is skipped when `zfs mount -a` fails;
  it adds the pass to the boot's critical path;
  `start` does nothing and
  `restart` unmounts every clone first.
- B.
  A drop-in `ExecStartPost=-/usr/bin/systemctl --no-block --no-ask-password reload-or-restart
  reconcile-git-worktree-clone-mounts.service`,
  with the boot unit gaining `ExecReload=<handler> boot-reconcile`.
  Example:
  the same 09:00 lever is `systemctl --no-ask-password reload reconcile-git-worktree-clone-mounts.service`:
  the pass runs in the
  unit's namespace and journal,
  nothing unmounts.
  At boot the hook only enqueues the unit the boot starts anyway.
  Pro:
  the provider is never failed;
  nothing on the critical path;
  one owner for ordering,
  probe,
  namespace and journal;
  a supported lever for the project.
  Con:
  a `systemctl` call from a unit's start is unprobed (A9);
  each unit state needs
  a guest arm;
  it depends on the sibling's `ExecReload=` choice.
- C.
  A `.mount` unit for the checkout with a companion unit bound to it.
  Example:
  the same key load,
  with the checkout mounted by its own unit,
  whose companion runs `boot-reconcile`.
  Pro:
  fires on every activation of the checkout mount and gives the checkout a unit state.
  Con:
  changes how the checkout
  is provisioned,
  needs the checkout excluded from `zfs mount -a`,
  and belongs to the boot-protection decision the plan
  leaves conditional.
- D.
  No hook;
  the printed levers only.
  Example:
  the same key load,
  with no trigger;
  the project reads `doctor` and runs the reload.
  Pro:
  nothing new.
  Con:
  the proposal's obligation is unmet.
- Ranking:
  B > A > C > D,
  because B meets the obligation with the unit that already owns the order and never fails the
  provider,
  A meets it at a boot refusal and a critical-path pass,
  C meets it by changing the checkout's provisioning,
  and D does not meet it.
- The review's own ranking is the same;
  its option C and D are this question's C and D.

### Q-B. Does "moot" accept that a lost last event with nobody waiting may stay unrecovered until the next trigger, up to the next boot?

- A.
  Yes:
  no time bound;
  `doctor` names the state and prints the levers.
  Example:
  an add at 17:30:04 with ZED stopped times out at 17:30:34;
  the machine stays up nine days;
  the clone stays
  unmounted until a boot,
  a provider start or a lever the project fires.
  Pro:
  matches "Periodic reconciliation without a waiter is not part of this route" ([proposal][proposal],
  lines 537 to 539) and costs
  nothing.
  Con:
  a request the account made can wait for days.
- B.
  No:
  the wrapper adds a trigger it controls.
  Whenever a managed command runs in a repository that holds a clone with
  an unanswered request,
  it writes that clone's nudge,
  which is not a timer.
  Example:
  the next `git status` in the repository at 17:41 writes the nudge and the clone is attached a second later.
  Pro:
  "we control the triggers" taken literally,
  and recovery follows the project's own activity.
  Con:
  every Git
  command pays a device open and a property read,
  unmeasured,
  and couples every Git call to the ZFS gate.
- C.
  No:
  a timer.
  Example:
  a unit every ten minutes runs the pass,
  so the clone is attached within ten minutes.
  Pro:
  bounded recovery.
  Con:
  reverses the recorded decision and needs the coalescing and rate rule to keep its own cost.
- Ranking:
  A > B > C,
  because A matches the record and costs nothing,
  B adds a trigger the project controls without
  reversing the decision but at an unmeasured price on every Git call,
  and C reverses a recorded sentence.
- This question is the review's Q-B,
  kept in its words.

### Q-C. May the cost rule hold back a pass while a caller waits?

- A.
  Yes,
  every event- or nudge-driven pass over the bound is held,
  and the deadline names the possibility.
  Example:
  the window is spent and four adds start at once;
  the fourth times out and its clone attaches at the next
  admitted trigger.
  Pro:
  simple and applies to the burst it is for;
  the request is durable.
  Con:
  noise can time out a real caller.
- B.
  No bound at all.
  Example:
  the same four adds are all served,
  and a shell loop writing the nudge property runs a pass per write.
  Pro:
  no caller is ever held.
  Con:
  the decision's cap is not shipped.
- C.
  Bound only passes whose event names a clone with no unanswered request.
  Example:
  the window is spent;
  an add of clone `one` writes its intent,
  the handler reads `one`,
  finds it unanswered and
  runs;
  a shell loop on clone `two`,
  with nothing pending,
  is held after the first passes.
  Pro:
  a waiting caller is not timed out by noise,
  and it fits "bound the work of one sweep" ([proposal][proposal],
  lines 575 to 578).
  Con:
  one clone read before the decision,
  unmeasured;
  a loop on a clone with a pending request is not bounded,
  which the
  threat model accepts;
  and an event that names no clone needs a rule.
- Ranking:
  C > A > B,
  because C keeps a real caller moving where A can time it out on noise,
  and A ships the decision's
  cap where B drops it.
- This is the review's Q-C,
  with its added option C.

### Q-D. What may nudges cost root, in which unit?

Context:
the in-flight branch measures a steady sweep at about 0.3 to 0.4 seconds for 64 children and about 20 seconds for
4,096 (F34,
unmerged),
so a pass's cost varies by orders of magnitude with the clone count.

- A.
  A budget of pass time per window,
  in seconds,
  sized from the measured pass duration.
  Example:
  with 64 clones a budget of a few seconds per window admits many passes;
  with 4,096 clones the same budget
  admits one.
  Pro:
  bounds the resource the passes actually spend.
  Con:
  a pass's duration must be known before it runs,
  or the budget
  is charged after the fact.
- B.
  A count of passes per window.
  Example:
  the same count admits 4,096-clone passes back to back and holds 64-clone passes that cost almost nothing.
  Pro:
  simple to count and to read in `doctor`.
  Con:
  cost and count diverge by orders of magnitude.
- C.
  Leave the unit to step 22's measurement.
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
  and B still gives step 22 a
  shape where C gives none.
- This is the review's Q-D,
  kept in its words.

## Dependencies on other designs and on the plan

- D1.
  The propagation sibling ([sibling propagation][sibprop]).
  It proposes `ExecReload=<the same handler> boot-reconcile` on the boot unit as its veto choice V4 and names
  `systemctl reload` as its manual lever (lines 1269 to 1290 and 1955 to 1965),
  and this design's Q-A option B and T5 use
  that line;
  if its V4 is vetoed,
  Q-A's ranking changes and T5 loses the reload.
  It also assumes the provider hook runs `boot-reconcile` (its A3 and dependency list,
  lines 2151 to 2155),
  which this
  design's Q-A option B does not do directly:
  it asks the unit whose `ExecStart` runs it.
  No record decides which shape
  the proposal's obligation takes,
  and the two documents differ there rather than contradicting each other.
  Its propagation step at the head of both entry points replaces this design's earlier assumption that a readiness pass
  refuses with the checkout field when the propagation is not set;
  a pass that sets it does not refuse for that reason.
- D2.
  The forced-removal sibling ([sibling forced][sibforced]) assumes "each removal request is one nudge-shaped property
  write" and that the handler may "coalesce sweeps without merging two nonces" (its D5,
  lines 1789 to 1790).
  This design
  writes the intent and,
  under V3,
  one nudge per wait,
  and its rule can hold a removal's pass as it holds any other.
  The
  sibling's assumption is narrower than this design;
  no record decides whether a destructive step's pass is exempt,
  and
  Q-C's option C is the nearest thing to an exemption this design offers.
- D3.
  The cancelled-removal sibling ([sibling cancelled][sibcancelled]) lists "a later sweep nudge on that clone" among its
  triggers (line 974).
  This design writes a nudge only inside a wrapper wait or by T5,
  so that trigger exists only if a
  later command waits on that clone,
  and the sibling's sentence should say so.
- D4.
  The add route's nudge call is step 10's,
  and the removal's is step 14's;
  this design assumes both follow V3 and nothing
  else.
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
  step 22's
  "no existing constant changes" line,
  which gains `recent-starts` and the rule;
  and step 18's
  "ZED and service wiring" check,
  which V9 narrows to the printed commands and the plan's text should say so.
- D9.
  The `SPEC.md` edits owed:
  a section for the three cost records and their modes and bounds;
  the timeout sentence of
  V11;
  the filter's line in the default invocation;
  and the handler paragraph's note that a held or filtered run exits
  2.
- D10.
  Siblings that state no dependency on this design and gain none from it:
  the add-time Git lock (whose own note says
  the trigger design "gains one action,
  not one trigger" if that design's Q1 is A),
  the add binding record (which says the two
  "share no fact"),
  the origin cleanup record (which only quotes the answer),
  the dirty-main restore (which "adds no
  trigger of its own"),
  the deferred-destroy checks (no mention),
  and the removal plan record (whose sweep-trigger
  sentence is T1's).

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
- Sibling design documents,
  with `git show <branch>:<path>`:
  the private propagation,
  forced removal,
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
  with the brief's two coordinator
  corrections folded in.
- 2026-11-22T17:29:34-05:00:
  corrected the revision record and the provider section's wording.
- 2026-11-22T17:30:36-05:00:
  said which taker clears a rate-refused mark.
- 2026-11-22T18:15:57-05:00:
  the gate-review answer,
  a rewrite that fixes every finding in
  `~/temp/agent/gate-review-zfs-lost-last-event-triggers-design-2026-11-22/keep/review.md`:
  the provider hook (H1),
  the nudge policy (H2),
  the eleven medium findings,
  the eight low findings,
  the wrong citations,
  the eight silently decided choices,
  the sibling readings and this section.
- 2026-11-22T18:16:16-05:00:
  recorded that commit's time in this section.
- 2026-11-22T18:16:38-05:00:
  pointed T1's sentence about the handler's lost line at V8 rather than V7.
- Any later commit touching this file corrects this section's own record,
  and `git log --format=%cI -- doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`
  lists each of them with its committer time.

[decision]: ../decision/cli-git-zfs-worktree-design.md
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
