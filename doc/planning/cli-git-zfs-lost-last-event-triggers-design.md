# Which triggers a lost last event waits for, and how nudges are merged

## Status

Non-final,
unapproved design proposal by an agent.
The project owner has not read it,
no reviewer has read it,
and
nothing in it is final or approved;
only the owner declares a design accepted.

It designs the open item "Which triggers a lost last event waits for" of the accepted ZFS worktree route
([decision][decision],
"Decision";
[proposal][proposal],
"Open items the decisions leave",
the item beginning
"Which triggers a lost last event waits for").
The plan puts that item on steps 7 and 22,
"not blocking either,
writer
design work;
the rate rule waits for step 22's measurement"
([plan][plan],
"Open items and the step that needs each").
This design therefore ships no rate:
it designs the rule's
shape and names the measurement that sizes it.

What it read:
the ZFS decision,
the accepted proposal's sections named in "The records that bind this design",
its
open-item list,
the implementation plan's steps 7,
8,
10,
14,
18,
19 and 22,
`package/git-policy/cli/SPEC.md`'s
sections on the intent,
receipts,
readiness and the sweep,
the step 8 choices record,
the slot identity store record,
the step 7 measurement record,
the code under `package/git-policy/cli/src/native/` named in the facts,
and the source
of OpenZFS at `zfs-2.4.4` and systemd at `v262`.

What it ran:
read-only probes of this host,
listed in "Facts" with the result each gave;
two shallow clones of the
outside sources under this round's scratch folder.
What it did not run:
no guest,
no virtual machine,
no container,
no ZFS command that changes anything,
no mount or namespace of its own,
no handler,
no wrapper command that makes a
request,
no test suite,
and no systemd unit.
Nothing here was measured.
Every measurement this design owes sits under
"Measurements owed" and is still owed.

It edits no code,
no specification,
no decision record,
no handover and no other planning document.
It reads no
sibling round's unfinished branch as decided;
"Dependencies on other designs" states each dependency and the
assumption made for it.

## The open item, and the answer it carries

The item,
quoted whole from [proposal][proposal],
"Open items the decisions leave":

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

The answer it carries,
in the owner's words,
is quoted whole in [proposal][proposal],
"Project decisions",
under "Lost
last event without a waiter":
"Triggers only,
and we control the triggers,
which makes the con of "unavailable until
next trigger" moot."
The same entry keeps the gap this design closes:

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
The no-waiter case has no statement of what happens between the loss and
the next trigger.
The provider hook is "proposed and unrun" and has no shape.
The merge and rate rule have no shape at
all.
This design answers those four,
and nothing else.

## What this design did not take from the brief

The design reads the answer's "moot" as a claim about a caller that waits,
not a claim that a lost event is recovered
promptly with no caller.
A caller that waits installs its own trigger by writing a nudge,
so for that caller the
answer's words hold.
A clone whose last event was lost with nobody waiting is recovered by the next trigger the
project controls,
and that next trigger can be the next boot,
so the recovery time has no upper bound that this design
promises (section "The no-waiter case").
Stating that plainly is part of the answer,
not a departure from it.

## The records that bind this design

Each passage is quoted with its path and line,
and the sentence after it states what it settles here.

The route is accepted with its open items open,
and it decides that "The root-installed ZED handler and its lifecycle
unit own clone mount and unmount,
not Git or automatic deletion"
([decision][decision],
"Decision",
line 32).
So the triggers are root's units and ZED,
and never a Git
invocation's side effect.

The intent is a clone-local property,
the receipt is a root-owned file,
readiness is a root-owned record,
and the
nudge is a property written by a waiter:
"A waiter may change the local
`org.monochromatic:git-worktree-clone-mount-sweep-nonce` as often as it needs on the verified managed clone it is
waiting for,
not on the clone parent.
The nudge uses the same hex format and is not a mount instruction."
([proposal][proposal],
"Requests,
results and ordering",
lines 573 to 576).
The same passage states the handler's
limit and the plan's shape:
"The handler cannot limit the nudge to one write:
the account owns that property and may
change it repeatedly,
and each change may start a sweep that scans every task,
so bound the work of one sweep instead
of counting nudges."
(lines 576 to 578).
So a burst of nudges is expected,
the handler cannot refuse the write,
and a
rule must bound work rather than count writes.

Requests are answered by a receipt that carries the request's nonce,
and a stale receipt is never the answer
([proposal][proposal],
"Requests,
results and ordering",
lines 561 to 572;
[spec][spec],
"The receipts",
lines 6132 to 6170).
So a caller's confirmation is its own nonce,
and a sweep that replaces a receipt under the same
nonce robs nobody.

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
(lines 793 to 795).
So a held-back sweep lengthens a caller's wait but never cancels
its request,
and the timeout text must say so.

The built protocol states the three triggers in its own refusals:
a refused attach tells the caller to "wait for the
boot unit's `boot-reconcile` to verify the provider and publish this boot's readiness,
then let the next trigger (a
ZED event,
the checkout provider's post-key-load hook,
or the caller's own sweep nudge) run the sweep"
(`package/git-policy/cli/src/native/worktree_clone_mount_protocol/clone_mount_readiness_admission.rs`,
line 100).
So
the shipped texts already name the trigger set this design has to make true.

A ready checkout is a condition of every attach,
and a record that says `not-ready` refuses regardless of live state
(the same file,
lines 102 to 150;
[spec][spec],
"The readiness record",
lines 6210 to 6257).
So a trigger that
reconciles after a late checkout readiness must republish readiness,
not only sweep.

A held slot is refused rather than queued,
and the holder reads the intent again after it releases so that a request
arriving during a turn is not lost ([decision 8][step8],
"A slot whose claim is held";
[spec][spec],
"The per-slot
claim",
lines 6307 to 6330).
So the per-slot device already answers the request-during-a-turn race for one clone,
and
this design's merge device answers the cost of many turns,
not their safety.

A refusal files no receipt except an unavailable key's ([decision 8][step8],
"Receipts for refusals";
[spec][spec],
"The sweep,
and the boot it reconciles",
lines 6360 to 6390).
So a held-back or filtered trigger files
no receipt and reaches a waiting caller only as its deadline.

The sweep visits at most 4096 children,
and the number is measured:
one sweep over 4097 children visited the first
4096 and stopped in 14183 milliseconds,
and one handler run over a single clone took 99 milliseconds
(`package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_dataset_reader.rs`,
lines 26 to 58).
A round
is changing that cost now on the branch `fix/git-policy-cli-zfs-step-8-recheck-findings-and-sweep-cost`,
so the two
numbers are the merged ones as they stand and may move;
this design cites them and nothing here depends on their
exact value.

The mount-request deadline is provisional and unsized:
"The mount-request deadline is this route's first ZFS wait
constant.
It ships as a provisional 30 seconds" ([spec][spec],
lines 6413 to 6416),
the constant is
`PROVISIONAL_MOUNT_REQUEST_DEADLINE` at 30,000 milliseconds with a 50 millisecond poll
(`package/git-policy/cli/src/native/worktree_clone_mount_protocol/mount_request_wait.rs`,
lines 59 and 68),
and step
22 sizes it ([plan][plan],
"Step 22",
lines 2340 to 2360).
So this design's rate rule must be expressible in terms
step 22 can size,
and no number in it may ship before that measurement.

The plan's step 7 says the coalescing and rate rule stay unwritten there ("Open items",
lines 1148 to 1153),
and step
22 repeats that it "Proceeds without "Which triggers a lost last event waits for" and measures before any rate rule
ships" (lines 2410 to 2416).
So this document is the writing the plan anticipated,
and after acceptance the plan's two
sentences and its open-item entry need amending (section "Dependencies on other designs and on the plan").

No configuration is needed and no person makes a setting:
"The project first answered that everything must work by
default with no configuration",
and for a host with no file at all the project answered "1:
no file needed
(Recommended)" ([decision 8][step8],
"The datasets the sweep acts on" and "With no configuration file",
lines 17 to 68).
So every trigger and record here works with no file,
and the parent name the event filter needs comes from the same
configuration the handler already reads,
with the design's own names as its fallback.

The wrapper never asks a person for a secret,
and where it lacks authority it refuses and says what to do:
"Avoid ever asking for someone to type a username or password."
with "Passphrases count too (Recommended)"
([spec][spec],
lines 7348 to 7352).
So no trigger here prompts,
and a trigger a person must fire is printed as a
command for that person's own shell.

Concurrency is lock-free first,
"one owner per item",
and an unavoidable lock covers the smallest unit with its reason
written beside it ([AGENTS.md](../../AGENTS.md),
rule `NLK`).
So the merge device reuses the repository's numbered
claim,
no one waits on it,
and its reason sits in this document.

A shipped resource limit cites a workload measurement or an owner decision beside the value
([AGENTS.md](../../AGENTS.md),
rule `LQM`).
So the rate rule's two numbers ship only with step 22's measurement,
and
the one-extra-pass cap cites the measured sweep cost named in the facts.

The precedent for a small durable record without a lock is one file per item,
made with `O_EXCL` and linked into place,
with no listing on the waiting path and `doctor` reporting leftovers ([decision store][store],
whole file).
So this
design's records are one file per meaning,
published whole,
never appended under a lock,
and never listed on a path
that must not list.

Every worktree of this repository lives under `.claude/worktrees/` in the main checkout,
and paths come from the
injected home ([AGENTS.md](../../AGENTS.md),
rules `WTU` and `HOM`).
So no path here is hardcoded and no trigger writes
inside a checkout's own tree.

### Where the records need amending

Three statements in the project's records do not hold after this design,
and none of them is edited here.

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
    The plan's assignment is the one this design
    follows,
    so those four sentences name the wrong step and should say step 18.
2.  The plan's step 7 "Open items" and step 22 "Open items" say the coalescing and rate rule stay unwritten.
    This
    document writes them,
    so after acceptance both sentences,
    the open-item entry at lines 2846 to 2854,
    and the plan's
    "Gaps the design's open-item list does not name" gain the one record family named in "Merging the nudges pending
    for one clone".
3.  The proposal says a rule should "bound the work of one sweep instead of counting nudges"
    ([proposal][proposal],
    lines 576 to 578),
    and the decision requires a measurement "before any bound on how many
    sweeps may run in a period is shipped" ([proposal][proposal],
    lines 1404 to 1407).
    Both stand:
    the child bound
    bounds one pass's work and stays;
    the rate rule bounds passes per period and ships with step 22's number.
    No
    sentence here replaces the child bound.

## ZED's delivery, read at zfs-2.4.4

Every statement in this section is read from OpenZFS at tag `zfs-2.4.4`,
cloned at
`/home/user/temp/agent/zfs-lost-last-event-triggers-design-2026-11-22/scratch/openzfs`,
and none of it is measured on
this host.

A user property change reaches ZED as a `history_event`.
The kernel posts that class from the history record's sync
task,
and only for an internal history record:
the code branch that calls the notify is
`nvlist_exists(nvl, ZPOOL_HIST_INT_NAME)` (`module/zfs/spa_history.c`,
lines 296 to 331,
with the call at line 329),
and the class string is `history_event` (`include/sys/sysevent/eventdefs.h`,
line 121).
A `zfs set` of a user
property is such an internal record:
the step 7 measurement records that "the property and its value are in
`history_internal_str` and `ZEVENT_HISTORY_CMD` is empty"
(`package/git-policy/cli/doc/zfs-mount-receipt-deadline-measurement-2026-11-21.md`,
"Where",
lines 60 to 66).

The kernel holds events in one in-memory list,
by default 512 of them (`module/zfs/fm.c`,
line 72).
A new event is
inserted at the head,
and when the list is at the bound the kernel drains the tail,
which is the oldest event
(`module/zfs/fm.c`,
lines 180 to 192),
and counts a drop against any reader that had not yet consumed it (lines 148
to 156).
The next read reports the count (lines 344 to 359),
ZED logs it as `Missed N events` and doubles the bound up
to a ceiling (`cmd/zed/zed_event.c`,
lines 190 to 198 and 111 to 152).
Nothing re-posts a drained event.

ZED reads events one at a time and never replays:
it opens and reads a state file that records the last event it wrote
state for,
seeks past it,
then services the queue (`cmd/zed/zed.c`,
lines 264 to 299;
`cmd/zed/zed_conf.c`,
lines 543
to 665).
On this install the state file is the build's `RUNSTATEDIR "/zed.state"` (`cmd/zed/zed.h`,
line 27),
and the installed binary embeds `/var/run/zed.state` (`strings /usr/bin/zed`,
2026-11-22),
which `/var/run`'s link to `/run` makes a per-boot path that does not cross a boot.

ZED forks each matching zedlet and,
for an asynchronous one,
does not wait for it:
the state for the event is written
immediately after the fork (`cmd/zed/zed_event.c`,
lines 1020 to 1030),
and the loop services the next event.
A
zedlet whose name puts `-sync-` right after its class prefix makes ZED wait for all its children before and after that
fork (`cmd/zed/zed_exec.c`,
lines 339 to 357 and 422 to 441).
The built handler's installed name,
`history_event-mount_git_worktree_clone_datasets` ([spec][spec],
lines 265 to 268),
has no `-sync-`,
so the handler is
asynchronous today.

ZED runs at most 16 zedlet processes at once (`cmd/zed/zed_conf.c`,
line 51),
and while that many are alive it blocks
before forking the next one,
sleeping in 200 millisecond steps (`cmd/zed/zed_exec.c`,
lines 130 to 140).
Blocked that
way it is not reading the queue,
so the kernel's 512 slots can fill and drain the oldest events,
which is the loss ZED
then reports.

ZED's unit restarts it on any exit (`/usr/lib/systemd/system/zfs-zed.service`,
`Restart=always`),
and a restart seeks
past the last state write,
so events still in the kernel queue are delivered to the restarted daemon,
and events that
the last daemon wrote state for are not delivered again.

A zedlet matches by class prefix and runs for every pool:
the loop compares each zedlet's name against the event's
class,
its subclass and the string `all` (`cmd/zed/zed_exec.c`,
lines 409 to 421).
The installed zedlet therefore runs
for every `history_event` of every pool on the machine,
and an installed history zedlet does its own filtering,
as
`history_event-zfs-list-cacher.sh.in` does with `ZEVENT_SUBCLASS` and `ZEVENT_POOL` (`cmd/zed/zed.d/`,
lines 12 to
82).
The handler built today reads no `ZEVENT_` variable at all (`rg 'ZEVENT' package/git-policy/cli/src` returns
nothing),
so today every history event on every pool starts a full pass.

The four ways the last event for a clone is lost follow,
and each is a different recovery problem:

1.  The queue drained it because 512 newer events arrived while ZED was not reading.
    ZED logs a count and never
    re-posts it.
2.  The module unloaded or the machine rebooted,
    which empties the list.
    Nothing on disk is replayed as an event.
3.  ZED forked the handler and then wrote the event's state,
    and the handler then failed or was killed.
    The event is
    never delivered again.
    A handler killed by a ZED restart is this case,
    because systemd kills the whole control group of a stopped service by default
    (`KillMode=` defaults to `control-group`,
    `src/core/load-fragment.c`,
    lines 855 to 861 at tag `v262`),
    children included.
4.  The event reached ZED and the handler ran,
    but the handler's own pass refused or was held back,
    so no receipt
    answers the request even though the event was delivered.

A fifth case is not a loss:
a request whose event was never written,
because the account's `zfs set` failed or its
outcome is unknown.
The wrapper treats an unknown write outcome as unknown and inspects it before any retry
(`clone_mount_intent_write.rs`,
lines 68 to 82),
and this design's triggers do not cover it.

## The triggers, one by one

Five triggers exist after this design,
and one device that looks like a trigger is not one.
Each entry names who fires
it,
when,
as which user,
what it guarantees and which lost event it recovers.
The names `T1` to `T5` are this
document's handles and appear nowhere in code.

### T1: a ZED `history_event` delivery

- Fired by:
  the ZED daemon (`zfs-zed.service`),
  for each event it reads from the kernel queue whose class matches the
  installed zedlet's name.
- When:
  whenever the kernel posts a `history_event`,
  which for this route means any internal history record on any
  pool,
  including the account's `zfs set` of an intent or a nudge on a managed clone (fact F9).
- As which user:
  ZED runs as root,
  and the zedlet it forks inherits that;
  the handler runs in the host mount
  namespace ([proposal][proposal],
  "Boot and shutdown unit",
  line 512).
- Guarantees:
  one delivery attempt per event that is still in the kernel queue when ZED reads it,
  at most once after
  the state write,
  and no retry.
  It guarantees nothing about an event the queue drained (fact F10).
- Recovers:
  a request whose event is delivered,
  which includes a request written while a pass ran and a request
  written after a lost event when any later history event arrives.
  It does not recover its own loss.
- This design's addition:
  the handler filters on the event's pool and dataset before it opens `/dev/zfs`,
  because the
  installed zedlet runs for every pool (section "The event filter").

### T2: the boot unit's start reconcile

- Fired by:
  `reconcile-git-worktree-clone-mounts.service`'s `ExecStart`,
  installed by plan step 19
  ([plan][plan],
  "Step 19",
  lines 2121 to 2150).
- When:
  once per boot,
  ordered after the checkout provider,
  the key providers and `zfs-import.target`
  ([proposal][proposal],
  "Boot and shutdown unit",
  lines 499 to 514).
- As which user:
  root,
  in the host mount namespace
  ([proposal][proposal],
  "Boot and shutdown unit",
  line 513).
- Guarantees:
  one pass per boot after the route's prerequisites,
  and a readiness record for the boot whose `running`
  publish precedes the pass and whose `finished` or `failed` follows it
  ([spec][spec],
  "The sweep,
  and the boot it reconciles",
  lines 6332 to 6360).
  A failed prerequisite publishes
  nothing ([spec][spec],
  lines 6350 to 6360).
- Recovers:
  every lost event of the previous boot and of this boot so far,
  because the pass reads live intents rather
  than a queue.
  It recovers nothing until the next boot.
- It is a oneshot with `RemainAfterExit=yes`,
  so `systemctl start` on an active unit does nothing and
  `Restart=` never applies to a oneshot (`systemd.service(5)`,
  `Restart=`,
  and the note that oneshot services are
  never restarted on a clean exit).
  A person cannot re-fire it with `start`;
  the person's lever is T5.

### T3: the checkout provider's readiness hook

- Fired by:
  the checkout provider's own start,
  through a drop-in on the unit that mounts the checkout (section "The
  provider's readiness hook").
- When:
  whenever that provider runs under systemd,
  which on this host is at boot and whenever a person or a unit
  starts `zfs-mount.service` again.
- As which user:
  root,
  in the host mount namespace.
- Guarantees:
  one pass after the checkout is mounted and its key is available,
  and a readiness record that replaces a
  `not-ready` one,
  which is what admits callers again (fact F14).
- Recovers:
  a boot whose boot unit refused because the checkout was not ready,
  including the key-load-after-boot case
  the proposal names ([proposal][proposal],
  lines 546 to 550),
  and any request written between the two.
- It cannot see a checkout mounted by a person's own `zfs mount`,
  because no unit runs then.
  That case's lever is T5.

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
  on the first receipt read that finds no receipt answering the wait's own nonce (section "The
  wrapper's nudge policy").
- As which user:
  the account writes the property,
  which its `mount,userprop` grant on the parent's descendants allows
  ([proposal][proposal],
  "Scoped delegation",
  lines 336 to 345);
  the delivery and the pass are root's,
  as T1.
- Guarantees:
  one further history event,
  hence one more delivery attempt of T1,
  and nothing more.
  A nudge is not an
  instruction and carries no state the handler reads ([proposal][proposal],
  lines 573 to 576).
- Recovers:
  a request whose own event was lost,
  and a request written after a running pass had already read that
  clone,
  when the nudge's event is delivered.
  It does not survive its own event's loss.

### T5: a trigger a person fires

- Fired by:
  a person,
  with the two commands `doctor` prints:
  `/usr/libexec/monochromatic-git-policy-cli/
  mount-git-worktree-clone-datasets boot-reconcile` as root,
  and a `zfs set` of a fresh nudge nonce on the clone as the
  account.
- When:
  when a person reads a timeout,
  a refusal or `doctor`'s report and chooses to.
- As which user:
  root for the first,
  the account for the second.
  Neither prompts:
  this machine's `sudo` is
  passwordless ([decision staging][staging],
  lines 221 to 237),
  and the wrapper itself asks for no credential
  (fact F20).
- Guarantees:
  one pass,
  with the readiness publish for the root command,
  and exactly the guarantees of T1 for the
  account command.
- Recovers:
  any lost event,
  including the no-waiter case,
  because it is a trigger the person can always fire.
- It is documented,
  not installed.
  This design's answer to "which triggers exist" therefore includes the person,
  and
  the project's answer that "we control the triggers" holds:
  the two commands are the design's own,
  not an event
  source nobody controls (section "Questions only the project can answer",
  Q3).

### What is not a trigger

A ZED restart is not a trigger and not a loss of its own:
the daemon seeks past its last state write and delivers what
is still queued (fact F11),
so a `systemctl restart zfs-zed.service` re-delivers the tail and skips what it already
handled.
A systemd mount being noticed after the fact is not a trigger:
systemd shows a mount made outside a unit as a mount unit,
but no unit file ran for it and no `ExecStartPost=` fires
(`systemd.mount(5)`,
"Mount points created at runtime",
lines 84 to 90 of the v262 manual source).
A wrapper's read of a receipt is not a
trigger:
the wrapper only waits (fact F25).
And `doctor` is not a trigger:
it reports,
and prints T5's two commands
without running them ([plan][plan],
"Step 18",
lines 2030 to 2045).

### The event filter

Today's handler reads no event variable,
so T1 starts a full pass for every history event of every pool on the machine,
including every sanoid snapshot of the checkout and the home datasets,
every `zfs set` by another tool,
and every
event of another pool.
This design adds one filter at the start of the default invocation,
before the compatibility
gate and before any `/dev/zfs` open:

- Read the event's pool name and dataset name from the environment (`ZEVENT_POOL` and `ZEVENT_HISTORY_DSNAME`),
  strip
  any `@snapshot` suffix from the dataset name,
  and read the configured parent dataset from the same configuration the
  pass reads,
  with the design's own names as its fallback ([decision 8][step8],
  "With no configuration file").
- Accept the event when the pool equals the parent's first path component and the dataset equals the parent or begins
  with `<parent>/`.
- Refuse nothing:
  an event outside that scope prints one line naming the event's dataset and says no pass was run,
  and
  exits 0.
  An absent variable means the mode was not started by ZED,
  so it passes.
- The filter reads no ZFS state,
  opens no device,
  and takes no claim,
  so it cannot fail into a refused pass.

The filter is cost work and not security work:
the account cannot start a handler by any means the filter changes,
and
the events it drops are events the pass could not have acted on.
It is part of T1's guarantee,
and step 19 installs
the zedlet,
so step 19 owns the filter (section "Which build step builds which part").

## The no-waiter case

A clone carries a durable intent,
its event was lost by one of the four mechanisms in "ZED's delivery",
and no caller
waits:
the add that wrote the intent timed out and reported,
or the last caller exited long ago.
Which trigger fires
next,
and when:

- T1 fires at the next history event of any dataset under the pinned parent,
  which any caller's intent or nudge
  produces.
  That is the earliest possible recovery and has no bound of its own.
- T3 fires at the next run of the checkout provider under systemd,
  which at boot is the same order as T2 and later
  only when a person or a unit starts that provider.
- T2 fires at the next boot,
  which is the only trigger whose maximum delay is one boot.
- T5 fires whenever a person chooses.
  Before a person does,
  `doctor` is the only thing that names the state.
- T4 fires only if some wrapper wait happens,
  and after this design every such wait writes one.

No trigger here is a timer,
and the decision keeps it that way:
"Periodic reconciliation without a waiter is not part
of this route" ([proposal][proposal],
"Boot and shutdown unit",
lines 537 to 539).
So the honest answer to "how long
can that be" is:
as long as it takes for one of those triggers to occur,
which in the worst case is the next boot,
and
no sentence here promises less.

Meanwhile the clone's state is not lost and not silently changed:

- Its durable intent stands on the dataset,
  because nothing but its owner or a removal clears it,
  and a timeout is not
  cancellation ([proposal][proposal],
  lines 793 to 795).
- Its slot,
  if the add registered it,
  holds the Git registration and the pointer the add wrote,
  and the clone's
  dataset is not attached there,
  so the folder shows what the underlay holds and Git reports the registration without
  the worktree's files.
- The receipt for the clone,
  if any,
  answers an earlier request,
  so a reader sees a stale result and never a false
  answer ([spec][spec],
  "The receipts",
  lines 6160 to 6170).
- This boot's readiness record names whatever the last readiness trigger published,
  so a caller is either admitted or
  refused with the field that is not ready,
  and never admitted on a guess (fact F14).
- Nothing removes the clone,
  mounts it elsewhere,
  or changes its intent while it waits.
  The next trigger's pass is the
  only writer of its mount state.

`doctor` reports that state as a clone whose live intent has no receipt answering it (section "What doctor reports for
a clone whose last event was lost"),
and prints T5's two commands,
so a person who wants the worktree now has a lever
that is a trigger the design wrote down.

## The provider's readiness hook

This section answers what the hook is on this machine,
what has to be installed or decided outside the wrapper for it to exist,
by which command,
where that is recorded,
and what it runs.
The host facts it builds on were read on 2026-11-22 and are listed with their commands in "Facts".

On this machine today:
`zfs-mount.service` is enabled and active,
and its `ExecStart` is `/usr/bin/zfs mount -a`
(`/usr/lib/systemd/system/zfs-mount.service`,
`[Service]`).
There is no `.mount` unit for the checkout,
because the
checkout is not a dataset yet:
`/home/user/Monochromatic` resolves to `zpcachyos/ROOT/cos/home`,
an inherited dataset
(`findmnt --target /home/user/Monochromatic`),
and no `monochromatic-git-checkout` dataset exists (`zfs list`).
`zfs-load-key.service` is masked and `/etc/default/zfs` sets `ZFS_LOAD_KEY='yes'`,
while the pool's keystatus is
`available`;
the root datasets are encrypted,
so the keys are available before the boot units run on this host.
The
handler is installed nowhere:
`/usr/libexec/monochromatic-git-policy-cli` and `/etc/monochromatic-git-policy-cli` do
not exist,
and `zfs-zed.service` is disabled and inactive.
So today no trigger of this design exists on this machine,
and nothing here claims one runs.

What step 19 installs,
in the interface this design gives it:
a drop-in
`/etc/systemd/system/zfs-mount.service.d/reconcile-git-worktree-clone-mounts-after-checkout.conf` whose body is
`[Service]` and one line,
`ExecStartPost=/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets
boot-reconcile`.
The directive is the provider service's own:
`ExecStartPost=` runs after every `ExecStart=` command
exits successfully for a oneshot,
and its execution counts for `Before=` and `After=` ordering
(`systemd.service(5)`,
`ExecStartPost=`,
lines 427 to 465 of the v262 manual source).
A drop-in adds this list without
touching `ExecStart=`,
so `zfs mount -a` still mounts everything and this host's unit file is not edited.
A `.mount`
unit could not carry it:
mount units take `[Unit]` and `[Mount]` settings and run `mount(8)` themselves,
with no
`Exec` list (`systemd.mount(5)`,
lines 40 to 66 of the v262 manual source),
and `OnSuccess=` cannot serve either,
because it activates on entering `inactive`,
which a `RemainAfterExit=yes` service or a live mount does not do while it
is up (`systemd.unit(5)`,
`OnSuccess=`,
lines 878 to 882 of the v262 manual source).

What it runs,
and why the same mode as the boot unit:
`boot-reconcile`,
the mode step 8 built,
because a late checkout
readiness needs a republished readiness record and not only a pass (fact F14).
Its order therefore repeats the boot
unit's:
the start probe record,
the key and checkout checks,
the private propagation,
`running`,
the pass,
then
`finished` or `failed` ([spec][spec],
lines 6332 to 6400).
At a boot where both fire,
the hook runs first,
inside the
provider's start,
and the boot unit runs after it,
so the boot spends two passes;
the second is idempotent and the
merge device does not join them,
because readiness passes are outside the bound (choice V3).
Step 22 owes that boot
cost a measurement arm,
and this design names it (measurement M5).

The bootstrap refusal a caller can see is not changed by the hook:
a `not-ready` record still refuses,
and its text
already names this hook as the next trigger (`clone_mount_readiness_admission.rs`,
line 100).
The hook's own journal
line names the checkout state it found,
and `journalctl -u reconcile-git-worktree-clone-mounts.service` reads it,
because the drop-in runs the handler inside that service's cgroup,
and the handler's diagnostics go to standard error,
which the journal collects for the unit that started it.

What the hook needs from the project,
outside the wrapper,
and where each part is recorded:

- The provider integration question Q1 settles.
  It is recorded in this design's answer to Q1
  and in the decision record the acceptance writes,
  and until then the first-ranked route is the one described here.
- The handler binary,
  installed at
  `/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets`.
  Its installation is plan step 19's,
  the release archive is step 5's,
  and the path is recorded in `SPEC.md`'s handler paragraph ([spec][spec],
  lines 262 to 270).
- The zedlet,
  installed at `/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets`,
  also step 19's and also recorded there.
- The checkout dataset,
  `monochromatic-git-checkout`,
  mounted at the checkout path with a mountpoint,
  which step 19's migration creates and records in the plan's step 19
  ([plan][plan],
  "Step 19",
  lines 2121 to 2145).
  Until it exists,
  this host's provider has nothing to mount for the hook to follow.
- The drop-in file itself,
  at the path above,
  with the one `ExecStartPost=` line.
  It is step 19's interface and is recorded in its runbook;
  the command that applies it is step 19's printed setup command,
  run on the project's authorisation,
  because writing under `/etc/systemd` is never a step's act on this host
  ([plan][plan],
  "Step 19",
  lines 2126 to 2136).
- The command that fires the hook,
  once installed,
  is the provider's own start or restart,
  `systemctl start zfs-mount.service`,
  and nothing in the wrapper calls it.

It needs nothing from the key:
the wrapper never prompts for a passphrase,
and the hook never asks for one.

The one trigger it cannot provide is a checkout mounted outside a unit.
`systemd.mount(5)` says systemd monitors
runtime mount points,
but no unit file ran,
so no `ExecStartPost=` exists to fire;
that case waits for T2 or T5,
and
`doctor` prints both.

## Merging the nudges pending for one clone

### The principle

A pass derives its work from live state,
not from a queue:
it enumerates the parent's children,
reads each clone's
intent,
and acts under the slot's claim (fact F24).
So two nudges for one clone need no two work items;
they need one
pass that reads the clone after both writes.
The merge rule is therefore about how many passes run and who runs them,
not about accumulating requests.
A nudge carries no state any pass reads,
and a pass that runs after a nudge sees
exactly what the nudge's writer asked for.

What one waiting caller needs is a pass that reads its clone after its intent write.
Its own T1 delivery is one
attempt at that;
the nudge is a second;
this device is what makes either of them enough even when a pass is already
running.

### The records

Three names under one root-owned folder,
`/run/git-worktree-clone-mount-sweeps`,
made by the handler when a fresh boot
has none,
mode `0755`,
beside the receipts and readiness folders this route already publishes (fact F16):

- `run/`,
  the run claim's own folder.
  It holds only the numbered generations `take_claim` writes,
  because that call
  lists its folder and must see nothing else (fact F22).
  One pass of the whole route runs at a time.
- `another-pass-is-due`,
  a root-owned regular file of mode `0644` whose presence means a trigger has asked for a pass
  that no pass has yet covered.
  Its whole content is one line,
  `v1:<32 lowercase hex digits>`,
  so a writer's nonce and
  the file's modification time let `doctor` say who asked and when.
- `recent-starts`,
  a root-owned regular file of mode `0644` holding the last `B` passes' start times,
  one per line,
  each a count of seconds since boot from `CLOCK_MONOTONIC` followed by the same moment as an ISO 8601 timestamp with
  its offset.
  It is replaced whole,
  never appended.

All three live under `/run`,
so a boot clears them,
a claim means nothing across a boot as the route's other claim
does (fact F23),
and nothing here needs a migration or a durable format.

The folder and names are the writer's choice (choice V6).
The reasons are the ones the route already gives for
`/run`:
a pass and its claim mean nothing across a boot,
every take writes and syncs a record,
and a durable folder
would pay that on every trigger ([spec][spec],
"The per-slot claim",
lines 6310 to 6325).

### What a taker does

A handler that may pass,
which is T1's or T4's delivery,
does this in order:

1.  Take `run/`'s claim with the repository's numbered claim (fact F22).
    If it is held,
    go to step 8.
2.  Read `recent-starts`.
    If the last `B` starts are all within the last `P`,
    write `another-pass-is-due`,
    release the
    claim,
    print the held line naming the bound and the last start,
    and exit 0.
    Nothing was read,
    mounted,
    unmounted or
    recorded on a clone.
3.  Rewrite `recent-starts` with this pass's start appended and the oldest entry dropped when the list is full.
4.  Remove `another-pass-is-due` if it is there,
    because this pass covers everything marked before it began.
5.  Run one pass,
    exactly the pass step 8 built:
    the start probe record,
    the pool and checkout checks,
    the
    enumeration,
    each clone's turn under its slot claim,
    and the receipts.
6.  Release the claim.
7.  Look for `another-pass-is-due`.
    If it is absent,
    report the outcome and exit.
    If it is there,
    take the claim once
    more and do steps 2 to 6 once more,
    without step 7;
    a mark that survives that second pass is left standing for the
    next taker,
    and the line says so.
8.  When step 1 found the claim held:
    write `another-pass-is-due`,
    then try the claim once more.
    If that second try
    succeeds,
    continue at step 2,
    which removes the mark the handler itself wrote.
    If it fails,
    print the held line and
    exit 0.
    The second try never waits:
    the claim call returns at once with the owner's identity (fact F22).

A readiness trigger,
which is T2's or T3's run,
does steps 4 and 5 only:
it never takes `run/`,
never reads or writes
`recent-starts`,
and is never held (choice V3).
Its pass may therefore overlap an event-driven pass,
which the per-slot
claim already makes safe for each clone (fact F24);
the cost of that overlap is one extra pass per readiness
transition,
and readiness transitions are one per boot and one per provider run.

### Why no mark waits for ever

The device's invariant is that a mark always has a live attempt behind it.
A mark written at step 8 is followed by a
claim attempt;
if that attempt fails,
some other process holds the claim,
and every holder releases before its own
look for the mark at step 7,
so that holder's look sees a mark written before its release.
A mark written at step 2 is
followed by the release;
either a taker comes and removes it at step 4,
or the holder's own second pass at step 7
removes it,
or it stands for the next trigger,
which is the deferral the bound intends.
So:

- A nudge written while a pass runs is covered by that pass's second pass when its mark is written before the pass's
  release,
  and by the next taker otherwise,
  and a handler that fails to take the claim tries again after marking,
  so
  the window in which a mark stands with nobody attempting a pass is the moment between a pass's release and a
  handler's first claim attempt,
  which no order can close without a waiting queue.
- A nudge written while no pass runs is covered by its own delivery:
  its handler takes the claim,
  removes the mark it
  or an earlier handler wrote,
  and passes.

The residual window is a limit and not a defect this design hides:
a trigger whose mark lands after the last taker
released,
and whose own second attempt fails because a third taker arrived first,
waits for the next trigger,
and the
waiting caller's own deadline reports it.
No nudges are lost in the record;
only the pass that would answer them waits.

### Why this needs no lock

No process waits on `run/`.
The claim is the repository's numbered one:
the kernel lets exactly one process create the
next generation,
a holder is named by process birth so a killed taker leaves nothing to clean by hand,
and a loser
returns at once with the owner's identity (fact F22).
The mark is one `O_EXCL` creation,
and its removal is one
`unlink`;
a writer that loses the creation race is in the same state as a writer that won it,
because presence is the
whole meaning.
`recent-starts` is replaced by one staging write and one rename (fact F21).
So the device is lock-free
in the repository's sense,
its unit is one pass of the route's single parent,
and no smaller unit exists:
the per-slot
claim is the smaller unit and stays where it is for safety ([AGENTS.md](../../AGENTS.md),
rule `NLK`).

### The device is cost, not safety

Every interleaving here is safe without it.
Two passes at once take different slots' claims,
a clone whose slot is held
is left to that holder,
and a clone already attached is recognised and keeps its receipt (facts F24 and F26).
The
merge device only decides how many passes run.
Therefore an unreadable,
corrupt or missing record never refuses a
pass:
a taker that cannot take the claim because `run/` is unreadable passes anyway,
prints that the cost device could
not be read,
and a taker that cannot read `recent-starts` passes and prints that the bound was not applied (choice V5).
The same rule keeps a bug in this device from becoming an availability bug in the route.

### The wrapper's nudge policy

The wrapper writes at most one nudge per wait,
and it writes it on the first receipt read that finds no receipt
answering the wait's own nonce;
a wait answered on its first read writes none.
The reason is cost:
an add's intent
write already produces T1's event,
so a nudge written at the start would double every add's events for nothing.
The
wait's poll interval is 50 milliseconds today (fact F25),
so the nudge lands about that soon after the intent,
well
inside the provisional deadline,
and it carries a fresh nonce so the write is a change even when the previous value was
its own (fact F18).
The writer is the same gated writer the intent uses,
so no raw request leaves the wrapper before the
compatibility gate passes (fact F20).
This is choice V7.

## The rate rule's shape, with no number

### What is bounded, and what is not

Bounded:
the number of passes that T1's deliveries and T4's nudges start,
per period,
across every clone of the route's
one pinned parent.
The bound is expressed as `B` starts in any window of `P` seconds,
both measured in step 22,
and it
is counted per pass rather than per nudge or per event,
so one nudge,
one event and one second pass of an existing run
each cost one start.

Not bounded,
and each for a stated reason:

- The number of handler invocations ZED forks.
  Every `history_event` of the machine forked one before this design's
  filter existed;
  after the filter,
  an event outside the parent forks one that opens no device and exits.
  The bound
  cannot reach this,
  because ZED forks before any of this route's code runs (fact F9).
  ZED's own 16-process ceiling
  and the kernel's 512-slot queue bound it instead (facts F12 and F10).
- The account's ability to write the property.
  The proposal says the handler cannot limit it (fact F4),
  and the honest
  consequence is that a loop of writes still costs one fork and one claim attempt per write.
- Readiness passes,
  which are T2's and T3's (choice V3).
  A rate rule must not decide whether callers are admitted.

### What a caller sees when a pass is held back

The caller's intent write stands,
its nudge stands,
and no receipt answers it.
Its wait therefore ends at the
provisional deadline with the text built today,
which already names the request,
the last observation,
`doctor` and the
journal,
and already says that a deadline is not cancellation (fact F25).
This design adds one sentence to that text:
that a pass can be held back by the route's cost rule,
that `doctor` reports the last starts and whether a pass is due,
and that the next trigger passes when the bound allows.
The text does not read the records itself,
so the waiting path
gains no read (choice V8).

The reason to keep the read off the waiting path is measured in the route's own record:
the wait's answer came 36 to 64
microseconds after a receipt that was already on disk,
and every read on that path is paid on every poll
(`zfs-mount-receipt-deadline-measurement-2026-11-21.md`,
"The rounds").
A held pass is rare,
and the report that
explains it is `doctor`'s.

### The measurement that sizes it

Step 22 owns the number ([plan][plan],
"Step 22",
lines 2340 to 2400).
This design fixes what the measurement must
produce,
and it ships nothing until then:

1.  Measure the pass's cost on the acceptance workload at two populations:
    the clone count the acceptance workload
    uses,
    and one more than the child bound,
    which is the already measured worst case.
    Report each pass's wall time and
    the per-clone increment,
    interleaved with a control arm of the same binary in the same round,
    and report the
    spread of repeated rounds before any difference is called a difference (rule `QNB`).
2.  Measure how many passes one add starts,
    with this design's merge device on and with it off.
    The control is the same
    binary with the device's two decisions compiled out;
    it shows what the device saves and that the device has a
    measurable effect at all.
3.  Measure the caller cost of a bound:
    with a candidate `(B, P)`,
    run the acceptance workload and count the waits that
    end at the deadline.
    The positive control is a deliberately tiny bound,
    `B` of 1 in a window of one hour,
    which
    must produce a held pass and a timed-out wait with the sentence above;
    if it does not,
    the arm cannot detect a
    held pass and its zero must not be read as none.
4.  Choose `(B, P)` so that `B` passes fit inside the spare capacity the control arm measured on the host during the
    workload,
    and so that the acceptance workload's waits time out zero times in the interleaved control.
    Report both
    numbers with the measurement beside them,
    as rule `LQM` requires.
5.  Measure the lost-event behaviour and the trigger recovery separately (measurements M1 to M4),
    because no bound on
    passes may be read as a bound on recovery.

### Where this design stands on the threat model

The project's sentence is "We have to assume my machine isn't compromised.
sudo is passwordless here anyway.",
and the
record reads it as "the account is trusted,
and root and the account are not a security boundary here"
([decision staging][staging],
lines 221 to 237).
Applying it here:

- The rate rule exists for cost and not for security.
  An account that writes the property in a loop is trusted not to
  do it,
  and the rule's real work is to keep a buggy caller,
  a confused editor integration or a person's stray
  `zfs set` in a shell loop from spending root's time.
  The rule's reason survives that sentence because it is not about
  an attacker.
- The event filter also exists for cost.
  It removes passes for events the pass could not act on,
  and it is not a
  boundary:
  the account cannot reach the handler's dispatch anyway.
- The records' ownership and mode checks survive the threat model as accident protection (a stray write by another
  root process) and are not claimed as a boundary between root and the account.
  Where a check exists only because a
  hostile account was imagined,
  this design says so in the entry:
  none of the three records adds one.
- One thing this design does not do,
  and says it does not:
  it does not stop a trusted account from making root pass as
  often as the bound allows.
  Only the bound does that,
  and the bound ships with a measured number.

A further boundary this design keeps,
and states because this host runs a desktop session:
nothing here asks the system service manager to change or signal anything.
The only acting service-manager commands the design names are the two it prints for the project's own act,
`systemctl start zfs-mount.service` and the handler's `boot-reconcile`,
and no step or agent runs either on this host.
Nothing here signals a process at all:
the killed-handler case is a guest arm that runs as root inside the guest (test 10),
and the design needs no fact about what an unprivileged signal to another account's process or a system unit's task answers.
That question belongs to the removal's holder designs,
not here.
The drop-in takes effect when the service manager next reloads its configuration,
which step 19's runbook owns and no agent performs on this host.

## What doctor reports for a clone whose last event was lost

`doctor` runs as the account,
reads the records read-only,
and runs nothing it prints ([plan][plan],
"Step 18",
lines 2030 to 2045).
For each managed clone it reports these readings,
and for a clone whose live intent has no
receipt answering it,
it says so in one line and then prints the two triggers of T5.

The readings,
each with the source it comes from:

- The clone's dataset name,
  slot and live intent,
  with its nonce and operation,
  read from the clone's own property and
  only when that property's source is the clone itself (fact F17).
- The clone's mount state at its slot,
  as present or absent,
  read from the live mount table the way the sweep reads it.
- The receipt under the clone's canonical GUID,
  if any:
  its nonce,
  operation,
  outcome,
  boot,
  failed step and time,
  and
  one of three verdicts:
  it answers the live intent,
  it answers an earlier request and is therefore stale,
  or there is
  no receipt (fact F19).
- This boot's readiness record:
  its boot,
  pool and checkout states,
  its sweep state and nonce,
  and its time.
  A record
  that names another boot is reported as another boot's,
  because it says nothing about this one (fact F14).
- The run claim's newest generation and its owner birth,
  so a pass that is running or whose owner died is visible
  (fact F22).
- Whether `another-pass-is-due` is present,
  and its time,
  so a trigger that asked for a pass that has not run is
  visible.
- The last starts in `recent-starts`,
  the `B` and `P` in force,
  or,
  before step 22 ships them,
  the sentence that the
  rate rule is not in force because its measurement is owed (choice V7's consequence).
- The ZED installation facts it can read without a bus:
  the zedlet's path and mode under `/etc/zfs/zed.d`,
  and this boot's ZED state file when it exists,
  whose path this host's binary embeds as `/var/run/zed.state` (fact F11).
  The unit's enabled and active state is printed as the two commands to run,
  because
  `doctor` asks no service manager.

The lost-event line,
with concrete values:

```text
clone zpcachyos/nosnap/monochromatic-git-worktree-clone-datasets/one
  intent   v1:6f1c0a9d8b7e5f4a3c2b1d0e9f8a7b6c:mount, source the clone, written 2026-11-22T17:30:04-05:00
  mount    absent at /home/user/Monochromatic/.claude/worktrees/one
  receipt  1234567890123456789: none, so no pass has answered this request
  ready    boot 5d0f..., pool ready, checkout ready, sweep finished under nonce 0a9b... at 2026-11-22T17:29:58-05:00
  runs     no pass is running; a pass is due since 2026-11-22T17:30:05-05:00
  zed      /etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets present; run `systemctl is-enabled zfs-zed.service` and `systemctl is-active zfs-zed.service`
  events   the last event for this clone may be lost; the next trigger is a ZED event, the provider hook, or a pass you fire
  fix      as root: /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets boot-reconcile
  fix      as you: zfs set org.monochromatic:git-worktree-clone-mount-sweep-nonce=<32 fresh lowercase hex digits> <clone>
  journal  journalctl -u reconcile-git-worktree-clone-mounts.service ; journalctl -u zfs-zed.service
```

The lines reuse the route's wording for a stale result,
for a claim,
and for a readiness refusal,
so a person reads
one vocabulary in the refusal,
the timeout and `doctor`.
The two `fix` lines are the design's answer to the no-waiter
case for a person,
and `doctor` never runs them.
A record that cannot be read,
or a check that cannot be proved,
is
reported as unknown rather than as a pass,
which is the rule the plan already gives `doctor`
([plan][plan],
"Step 18",
lines 2060 to 2070).

The exit status follows the command's existing rule:
a finding that a check could not be read is the error status the
hooks report as 2 ([spec][spec],
`### Doctor`,
lines 683 to 760),
and a clone with a lost event is a finding,
not an
error,
so it does not by itself choose that status.
Which of the two a lost event is,
is the wording `doctor` already uses for a stale result and a refusal,
and this design ranks it a finding.

## Which build step builds which part

Plan steps 1 to 8 are merged,
so nothing here reopens them;
the parts land on steps still ahead,
and the plan needs
the edits named in "Dependencies on other designs and on the plan".

- Step 10,
  the add route:
  the wrapper's nudge policy (one nudge per wait,
  on the first unanswered read),
  and the
  call to the existing nudge writer ([plan][plan],
  "Step 10",
  lines 1399 to 1587).
- Step 14,
  removal:
  the same nudge policy for a removal's waits,
  so a lost event for an `unmount` intent is recovered
  by the same triggers;
  the unmount itself stays step 9's and the removal's own state reporting stays step 17's
  ([plan][plan],
  "Step 14",
  lines 1738 to 1814).
- Step 18,
  `doctor`:
  the readings and the lost-event line above,
  the two printed commands,
  and the correction of the
  four shipped sentences that name step 10 ([plan][plan],
  "Step 18",
  lines 2017 to 2120).
- Step 19,
  the units and the zedlet:
  the event filter,
  the provider drop-in,
  enabling `zfs-zed.service`,
  and the boot
  unit exactly as the plan already describes it ([plan][plan],
  "Step 19",
  lines 2121 to 2240).
- Step 22,
  the measurements:
  the run claim,
  the mark,
  `recent-starts`,
  the taker's sequence,
  and the bound's `B` and
  `P` with the measurement beside them ([plan][plan],
  "Step 22",
  lines 2340 to 2420).

The merge device and the rate rule land in one step because the rule's record is the same record the merge reads;
a
provisional bound would ship an unmeasured limit,
which rule `LQM` and the decision both forbid (choice V7).

## The tests a build must have

Each test names the race or the case it forces and how it forces it without timing luck.
Tests marked guest run only in
the throwaway guest,
which this round does not start;
the repository's guest gate is
`CLI_GIT_ZFS_SWEEP_GUEST` (`package/git-policy/cli/src/native/binary/binary_zfs_sweep_guest_tests.rs`,
lines 1 to 40),
and the driver is `package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`.

1.  Filter accepts and drops,
    host-side unit test:
    the default invocation with `ZEVENT_POOL` and
    `ZEVENT_HISTORY_DSNAME` set for the parent,
    a clone,
    a clone's snapshot,
    a dataset outside the parent,
    another pool,
    and with both absent.
    Each case asserts the decision and,
    for the dropping cases,
    that `/dev/zfs` was never opened
    and no claim was taken.
    The forced race is none:
    the inputs are the whole decision.
2.  A nudge during a pass,
    guest:
    start a pass over a parent with enough children that its enumeration is observable,
    wait for `run/`'s newest generation to appear,
    write a nudge on one clone,
    and assert that the pass's second pass
    ran and removed the mark.
    Forcing is by the record's appearance,
    not by a sleep (race R2).
3.  Two triggers at once,
    guest:
    the test takes the run claim itself with a live owner,
    starts two default invocations,
    and asserts both printed the held line,
    both wrote the mark,
    and neither read a clone;
    it then releases the claim
    and starts one more invocation,
    which must remove the mark and pass.
    This forces the contention with the test's own
    claim (race R3).
4.  A mark written after a pass releases,
    guest:
    the test writes the mark between the pass's release and its look,
    by
    taking the claim in that window through the test's own claim file;
    the handler's second attempt must pass,
    or the
    next invocation must remove the mark on entry.
    The test asserts no nudge is left waiting without an attempt
    (race R4).
5.  The bound holds a pass,
    host-side unit test with a fixture record:
    `recent-starts` holding `B` starts inside `P`
    must produce the held line,
    no receipt and the mark;
    holding `B` starts with the oldest outside `P` must pass.
    Time
    is written into the fixture,
    so no test waits (race R5).
6.  The bound's positive control repeats test 5 with `B` of 1 and `P` of one hour,
    and then asserts a caller's wait
    ends with the new sentence,
    so the arm is proved able to show a hold (measurement M3's control).
7.  Unreadable cost records,
    host-side unit test:
    `run/` unreadable,
    `recent-starts` corrupt,
    and the mark a directory
    instead of a file.
    Each must still pass,
    print which record could not be read,
    and leave the clones' state alone
    (choice V5).
8.  A lost event and its recovery per trigger,
    guest:
    stop `zfs-zed.service`,
    write an intent on a clone,
    assert no
    receipt after a bounded wait;
    then fire exactly one trigger per arm (start `zfs-zed.service` and write one nudge;
    run `boot-reconcile`;
    start `zfs-mount.service`;
    run the pass by hand) and assert a receipt under the intent's nonce
    appears in each arm.
    Each arm proves one trigger alone recovers what the queue lost (race R6).
9.  A drained queue,
    guest:
    with ZED stopped,
    write more property changes than `zfs_zevent_len_max` on the pool,
    then
    start ZED and assert its journal contains `Missed` and that no clone was attached by the drained events;
    the arm
    then fires one trigger and asserts recovery.
    This proves the loss is real in the fixture before any recovery is
    claimed (race R7).
10. A handler killed mid-pass,
    guest:
    the arm runs as root inside the guest and kills the handler's process group during a pass,
    so no unprivileged signal is attempted anywhere,
    then fire one trigger and assert
    the next taker took the next claim generation,
    that the killed pass's mount,
    if any,
    is recognised as already
    standing,
    and that its receipt is answered under the live intent's nonce (race R8).
11. No feedback,
    guest:
    install a logging zedlet beside the handler,
    run one pass,
    and assert the handler's own work
    produced no `history_event`.
    This proves the loop the design forbids cannot start (fact F27).
12. Boot with both readiness triggers,
    guest:
    enable the boot unit,
    the drop-in and the zedlet,
    reboot,
    and assert one
    readiness record for the boot,
    two passes in the journal,
    and every pending clone attached without a caller
    (measurement M5).
13. The provider's late readiness,
    guest:
    boot with the checkout's key unloaded so the boot unit refuses,
    assert a
    caller is refused naming the checkout,
    load the key and start `zfs-mount.service`,
    and assert the drop-in's pass
    published readiness and attached the clone without another add (the proposal's unrun acceptance item,
    [proposal][proposal],
    lines 2024 to 2030).
14. `doctor`'s lost-event report,
    host-side unit test over planted records:
    one clone with a matching receipt,
    one with
    a stale receipt,
    one with none,
    one whose record belongs to another boot.
    Assert the exact lines,
    the two printed
    commands,
    and that nothing was run.

Tests 1,
5,
6,
7 and 14 run on the host with no ZFS access.
Tests 2,
3,
4,
8,
9,
10,
11,
12 and 13 need the guest,
because each needs a real pool,
a real ZED,
real mounts or a real systemd.

## Measurements owed

Each measurement names its exact probe,
its control arm,
and what it decides.
None is run in this round,
and no
sentence above rests on one.

- M1,
  lost-event behaviour per trigger.
  Probe:
  in the guest,
  stop `zfs-zed.service`,
  write one intent,
  then fire one
  trigger at a time in separate arms and record the time from the trigger to the receipt.
  Control:
  the same intent
  with ZED running and no nudge,
  which gives the delivery's own latency.
  Decides:
  how long each trigger takes,
  and that
  each one alone recovers.
- M2,
  a drained queue.
  Probe:
  with ZED stopped,
  write more property changes than the kernel's queue holds,
  start ZED,
  and record its `Missed` line and which intents still have no receipt.
  Control:
  the same writes with ZED running.
  Decides:
  that the loss mechanism the design names is the one that occurs,
  and that the count is the kernel's.
- M3,
  the bound's caller cost.
  Probe:
  the acceptance workload with a candidate `(B, P)`,
  counting waits that end at
  the deadline;
  positive control `B` of 1 and `P` of one hour;
  control with the bound absent.
  Decides:
  `(B, P)`.
- M4,
  the merge device's saving.
  Probe:
  passes started per add with the device on and off,
  and the pass's wall time at
  the acceptance clone count.
  Decides:
  whether the device earns its code,
  and the one-extra-pass cap's cost.
- M5,
  the boot's cost with both readiness triggers.
  Probe:
  boot wall time to the boot unit's exit with the drop-in
  installed and with it absent,
  interleaved over repeated boots.
  Decides:
  whether the double pass at boot is
  acceptable,
  or whether step 19 needs the guard the plan should then carry.
- M6,
  the event filter's saving.
  Probe:
  on the guest's fixture,
  with sanoid's snapshot cadence emulated by the test's
  own property writes on the checkout and the home datasets,
  count passes with the filter and with it compiled out.
  Decides:
  whether the filter's cost is worth stating as a number in the specification.

## Facts

Each fact is one thing read or measured,
with its source,
so a reviewer can check it alone.
Nothing here is new
measurement.

- F1.
  This host runs systemd 262 (262.2-1-arch),
  kernel `7.3.5-1-cachyos`,
  OpenZFS `zfs-2.4.4-1` and Git `2.56.1`
  (`systemctl --version`,
  `uname -r`,
  `zfs version`,
  `git --version`,
  2026-11-22;
  `git --version` reached the
  installed wrapper,
  which reported `monochromatic-git-policy-cli 0.6.1-alpha.44`).
- F2.
  `zfs-zed.service` is loaded,
  disabled and inactive on this host,
  with preset `enabled`
  (`systemctl status zfs-zed.service`).
  Its unit runs `/usr/bin/zed -F` with `Restart=always`
  (`/usr/lib/systemd/system/zfs-zed.service`).
- F3.
  `zfs-mount.service` is enabled and active (exited),
  with `Type=oneshot`,
  `RemainAfterExit=yes`,
  `ExecStart=/usr/bin/zfs mount -a`,
  `After=zfs-import.target`,
  `Before=local-fs.target`,
  `WantedBy=zfs.target`
  (`systemctl status zfs-mount.service`,
  `/usr/lib/systemd/system/zfs-mount.service`).
- F4.
  The project says the handler cannot limit the nudge writes ([proposal][proposal],
  lines 575 to 578).
- F5.
  The acceptance route installs the handler at
  `/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets` and at
  `/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets` ([spec][spec],
  lines 262 to 270).
- F6.
  The handler's modes are the default,
  `boot-reconcile`,
  `shutdown-unmount` and `adopt-worktree-parent`,
  and the
  first two run a pass ([spec][spec],
  lines 270 to 280;
  lines 6332 to 6360).
- F7.
  The installed zedlet's name has no `-sync-` segment,
  so ZED forks it asynchronously
  ([spec][spec],
  lines 262 to 268;
  `cmd/zed/zed_exec.c`,
  lines 339 to 357 at `zfs-2.4.4`).
- F8.
  A user property change reaches ZED as a `history_event` carrying the property and value in the internal string,
  observed in the guest on 2026-11-21
  (`package/git-policy/cli/doc/zfs-mount-receipt-deadline-measurement-2026-11-21.md`,
  "Where",
  lines 60 to 66;
  [proposal][proposal],
  "Lost last event without a waiter",
  line 1400).
- F9.
  The kernel posts the class only for an internal history record (`module/zfs/spa_history.c`,
  lines 296 to 331),
  and the class string is `history_event` (`include/sys/sysevent/eventdefs.h`,
  line 121),
  both at `zfs-2.4.4`.
- F10.
  The kernel's event list holds 512 events by default,
  inserts at the head and drains the tail,
  which is the
  oldest,
  counting a drop against unconsuming readers (`module/zfs/fm.c`,
  lines 72,
  148 to 156,
  180 to 192 and 344 to
  359 at `zfs-2.4.4`).
- F11.
  ZED reads a state file,
  seeks past the last event it wrote state for,
  then services the queue
  (`cmd/zed/zed.c`,
  lines 264 to 299;
  `cmd/zed/zed_conf.c`,
  lines 543 to 665 at `zfs-2.4.4`),
  and the state file is
  `RUNSTATEDIR "/zed.state"` (`cmd/zed/zed.h`,
  line 27),
  which this host's installed binary embeds as `/var/run/zed.state`
  (`strings /usr/bin/zed`,
  2026-11-22),
  and `/var/run` links to `/run` on this Linux userspace,
  so the path is per boot.
- F12.
  ZED runs at most 16 zedlet processes at once (`cmd/zed/zed_conf.c`,
  line 51) and blocks before forking while
  that many are alive (`cmd/zed/zed_exec.c`,
  lines 130 to 140).
- F13.
  ZED writes an event's state right after forking its zedlets,
  before they finish
  (`cmd/zed/zed_event.c`,
  lines 1020 to 1030).
- F14.
  An attach is admitted only on a readiness record naming this boot,
  pool and checkout with both provider facts
  ready,
  and a `not-ready` field refuses
  (`package/git-policy/cli/src/native/worktree_clone_mount_protocol/clone_mount_readiness_admission.rs`,
  lines 100 to
  150;
  [spec][spec],
  lines 6210 to 6300).
- F15.
  The handler reads no `ZEVENT_` variable (`rg 'ZEVENT' package/git-policy/cli/src` returns no match,
  2026-11-22),
  and a zedlet matches by class prefix only,
  so it runs for every pool (`cmd/zed/zed_exec.c`,
  lines 409 to 421).
- F16.
  On 2026-11-22 this host has no `/usr/libexec/monochromatic-git-policy-cli`,
  no
  `/etc/monochromatic-git-policy-cli`,
  no `/var/lib/monochromatic-git-policy-cli`,
  no
  `/run/git-worktree-clone-mount-results`,
  no `/run/git-worktree-clone-mount-readiness` and no `/run/zed.state`,
  and
  the checkout is `zpcachyos/ROOT/cos/home` (`ls -d` on each path,
  `findmnt --target /home/user/Monochromatic`).
- F17.
  An intent value is read only from a local source;
  an inherited value authorizes nothing ([spec][spec],
  "The
  durable intent",
  lines 6100 to 6130).
- F18.
  The nudge shares the nonce's bound and alphabet:
  32 lowercase hexadecimal digits,
  no field separator
  (`clone_mount_sweep_nonce.rs`,
  lines 100 to 150;
  [spec][spec],
  lines 6123 to 6130).
- F19.
  A receipt is named by the clone's GUID,
  carries the nonce and outcome,
  is replaced by a live clone's next
  result,
  and a stale result is never the answer ([spec][spec],
  "The receipts",
  lines 6132 to 6170;
  `mount_request_wait.rs`,
  lines 280 to 300).
- F20.
  The wrapper's intent and nudge writes pass the compatibility gate first and never prompt for a credential
  (`clone_mount_intent_write.rs`,
  lines 34 to 82;
  [spec][spec],
  lines 7348 to 7352).
- F21.
  The route publishes a receipt and the readiness record by a staging write,
  a sync,
  a rename and a directory
  sync ([spec][spec],
  "The receipts",
  lines 6150 to 6165;
  "The readiness record",
  lines 6175 to 6190).
- F22.
  The repository's numbered claim is lock-free,
  published by one exclusive `link` under the next generation,
  names its owner by process birth,
  never waits,
  and a killed owner leaves nothing to clean by hand
  (`package/git-policy/cli/src/native/lock/single_owner_claim.rs`,
  lines 1 to 40).
- F23.
  `/run` is per boot for this route's records,
  and "a claim means nothing across a boot"
  ([spec][spec],
  "The per-slot claim",
  lines 6310 to 6325).
- F24.
  A pass enumerates the parent's immediate children,
  takes each slot's claim,
  re-reads the intent under it,
  leaves
  a held slot to its holder,
  and the holder reads the intent again after releasing
  ([spec][spec],
  "The sweep,
  and the boot it reconciles",
  lines 6332 to 6420;
  `package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_one_clone.rs`,
  lines 1 to 55).
- F25.
  The wrapper waits for a matching receipt for a provisional 30,000 milliseconds,
  polls every 50 milliseconds,
  and its timeout text names the request,
  the last observation,
  `doctor` and the journal
  (`mount_request_wait.rs`,
  lines 59,
  68 and 290 to 300).
- F26.
  A clone whose mount already stands at its slot keeps the `done` receipt rather than having a refusal overwrite
  it,
  and a turn files its answer before reading the intent once more
  (`sweep_one_clone.rs`,
  lines 44 to 50 and 440 to 450).
- F27.
  No step of the route writes an acknowledgement property from root ([proposal][proposal],
  lines 571 to 573),
  so
  a pass creates no property change and no event of its own.
- F28.
  The pass visits at most 4096 children,
  measured at 14,183 milliseconds for 4097 children and 99 milliseconds for
  one clone (`sweep_dataset_reader.rs`,
  lines 26 to 58).
- F29.
  This host's kernel queue bound is 512 (`/sys/module/zfs/parameters/zfs_zevent_len_max`,
  2026-11-22).
- F30.
  The four shipped refusals that name `doctor` say "once plan step 10 adds it"
  (`mount_request_wait.rs` line 298,
  `clone_mount_readiness_admission.rs` line 100,
  `clone_mount_receipts_folder_refusals.rs` line 185,
  `per_slot_root_claim.rs` line 151),
  and the plan builds
  `doctor` in step 18 ([plan][plan],
  lines 2017 to 2120).
- F31.
  `zfs-load-key.service` is masked on this host,
  `/etc/default/zfs` sets `ZFS_LOAD_KEY='yes'`,
  and the pool and
  checkout ancestor report `keystatus available` with `encryption aes-256-gcm`
  (`systemctl is-enabled zfs-load-key.service`,
  `sed -n '1,60p' /etc/default/zfs`,
  `zfs get encryption,keystatus`).
- F32.
  systemd's `ExecStartPost=` runs after a oneshot's `ExecStart=` exits successfully and counts for ordering,
  and
  `OnSuccess=` activates on entering `inactive`
  (`systemd.service(5)`,
  lines 427 to 465;
  `systemd.unit(5)`,
  lines 878 to 882,
  at tag `v262`),
  and mount units carry
  no `Exec` list (`systemd.mount(5)`,
  lines 40 to 66).
- F33.
  This design's host probes on 2026-11-22 were reads only:
  `systemctl --version`,
  `systemctl status`,
  `systemctl is-enabled`,
  and file reads under `/usr/lib/systemd/system` and `/etc/default/zfs`.
  No acting `systemctl` verb,
  no D-Bus call,
  no `pkexec`,
  no `journalctl` and no signal to any process was issued,
  so no service manager of this host was asked to authorize anything.

## Assumptions

- A1.
  The parent the pass sweeps and the parent the filter accepts are the same dataset,
  because both come from one
  configuration read with one fallback.
- A2.
  ZED sets `ZEVENT_POOL` and `ZEVENT_HISTORY_DSNAME` for a `history_event`,
  as the installed list-cacher zedlet
  relies on for the same class (fact F15's source file).
- A3.
  The handler is started by ZED only as root,
  so the filter's environment cannot be chosen by another account.
- A4.
  The readiness passes at a hooked boot are at most two,
  one from the drop-in and one from the boot unit,
  because
  each of those units starts once in that boot.
- A5.
  A pass's own work writes no property and mounts nothing that posts a ZFS history event (fact F27).
- A6.
  The wrapper can write a nudge on a clone it is waiting for under the agreed `mount,userprop` grant,
  and not on
  the parent ([proposal][proposal],
  lines 573 to 580).
- A7.
  A person's `sudo` needs no passphrase on this machine,
  as the project's threat model states;
  if it does,
  T5's
  root command asks that person's shell for it,
  and the wrapper never does.
- A8.
  The guest's kernel and OpenZFS behave like the host's for the queue and ZED,
  because both are `zfs-2.4.4` and a
  kernel the module supports;
  the guest's kernel is `6.8.0-147-generic` in the recipe,
  not this host's `7.3.5-1`,
  which is why measurement M2 belongs to the guest and not to a host claim.

## Limitations

- L1.
  The no-waiter case has no time bound:
  the recovery happens at the next trigger,
  which in the worst case is the
  next boot,
  and no timer exists by the project's decision.
- L2.
  A caller can time out because a pass was held back by the cost rule,
  and its request then waits for a trigger
  that the bound admits.
- L3.
  A trigger's mark can land in the window after the last taker released,
  so its pass waits for the next trigger;
  the mark itself is never lost.
- L4.
  The rate rule bounds passes and not handler invocations:
  a loop of property writes still costs one fork and one
  claim attempt per write,
  and ZED's 16-process ceiling is what bounds that.
- L5.
  The event filter can drop an event whose dataset is outside the parent even if a future feature puts an intent
  on such a dataset;
  the filter would then need that scope.
- L6.
  The provider hook cannot see a checkout mounted outside a unit,
  so that case waits for T2 or T5.
- L7.
  A hooked boot runs two passes,
  and step 22 has not measured whether that is acceptable.
- L8.
  Readiness passes can overlap an event-driven pass,
  so a boot or provider transition can coincide with a second
  pass over the same clones;
  the per-slot claim keeps it safe,
  and the cost is one extra pass.
- L9.
  The handler's `Missed N events` line is the only evidence a queue drain happened;
  if ZED's journal is rotated
  away,
  the loss is invisible and only the unanswered intent remains.
- L10.
  `doctor` reads no service manager,
  so a disabled ZED is visible only through the commands it prints unless the
  person runs them.
- L11.
  This design's records are per boot,
  so nothing here can report a loss that happened in a previous boot beyond
  the live intent and its unanswered state.
- L12.
  Every number named here is either the merged measurement named in the facts,
  which a round is changing,
  or is
  owed.
  No number is shipped by this document.

## Choices this design marks for the project's veto

Each entry gives the options,
a concrete example of what a person would see,
the pros and cons,
and a ranking with the
reason for each adjacent pair.
The adopted option is first and is what the rest of this document describes.

### V1. The ZED handler stays an asynchronous zedlet

- A,
  asynchronous,
  adopted:
  the installed name keeps no `-sync-` segment.
  Example:
  a sanoid snapshot of a clone writes a history event;
  ZED forks the handler and reads the next event
  immediately,
  and the handler prints `held` or runs its pass.
  Pro:
  ZED's event loop keeps draining the queue,
  so a pass can never fill the queue on its own,
  and two triggers
  become one pass through the mark.
  Con:
  up to 16 handlers can be alive at once,
  so concurrent passes exist and the
  merge device is needed.
- B,
  `history_event-sync-mount_git_worktree_clone_datasets`:
  ZED waits for all its children around that fork.
  Example:
  the same snapshot event;
  ZED waits for the pass to finish before it reads the next event,
  and a 14-second
  pass means 14 seconds of no event reading.
  Pro:
  at most one handler at a time,
  so no concurrent passes.
  Con:
  the pass's duration is added to the queue's
  drain time,
  which makes the oldest-event loss more likely under a burst,
  and `wait_for_children` waits for every
  child,
  including unrelated notification zedlets.
- Ranking:
  A > B,
  because the loss the design is about is a queue drain,
  and B lengthens the queue's exposure to make
  a problem the merge device already solves cheaper.

### V2. One set of cost records for the route's single parent

- A,
  adopted:
  one `run/`,
  one mark and one `recent-starts` for the route,
  because the route has one pinned parent per
  boot.
  Example:
  the checkout's parent is `zpcachyos/nosnap/monochromatic-git-worktree-clone-datasets`;
  its clones'
  triggers share one claim,
  and a second checkout would report a refused configuration rather than get its own claim.
  Pro:
  one record,
  one name,
  no dataset name to escape,
  and the bound is the decision's "across every clone" in its
  simplest reading.
  Con:
  a second checkout is unsupported by this design.
- B,
  per-parent records named by the parent's escaped dataset name.
  Example:
  a second checkout at a second path would get a second `run/` folder whose name encodes
  `pool/container`,
  with an escaping grammar of its own.
  Pro:
  two checkouts could each pass.
  Con:
  a new name grammar,
  a new refusal path for a name that does not fit,
  and a
  second parent nothing in the route supports today.
- Ranking:
  A > B,
  because the route supports one checkout,
  and a name grammar for a configuration nobody can use yet
  is a cost with no user.

### V3. Readiness passes are outside the bound and never held

- A,
  adopted:
  `boot-reconcile` from the boot unit and from the provider hook never takes `run/`,
  never reads or writes
  `recent-starts`,
  and is never held.
  Example:
  a caller whose checkout key was loaded late sees readiness published by the provider's pass,
  and is admitted
  on its next attach,
  even if the bound was exhausted a second earlier.
  Pro:
  a cost rule can never decide whether callers are admitted,
  and a boot always publishes readiness.
  Con:
  two
  passes can overlap,
  and a hooked boot runs two passes.
- B,
  readiness passes inside the bound and holdable.
  Example:
  the same late key,
  with the bound exhausted:
  readiness stays `not-ready`,
  and every caller is refused until
  a trigger the bound admits runs.
  Pro:
  one number covers all root work.
  Con:
  a cost rule becomes an availability rule,
  and a boot can end with no
  readiness record at all.
- Ranking:
  A > B,
  because availability must not depend on a limit whose number is not yet measured,
  and the cost of A
  is one measured pass per transition.

### V4. The mark, the second attempt and the one-extra-pass cap

- A,
  adopted:
  the taker removes the mark on entry,
  runs one pass,
  releases,
  and runs at most one more pass if a mark
  appeared;
  a handler that fails the claim marks and tries the claim once more;
  a mark left after the second pass
  stands for the next trigger.
  Example:
  two nudges during a pass produce one extra pass,
  and a third nudge during that extra pass waits for the next
  trigger.
  Pro:
  no waiting,
  a bounded number of passes per trigger,
  and a proof that a mark always has an attempt behind it.
  Con:
  the residual window in L3,
  and one more claim call on the contended path.
- B,
  a queue:
  a handler that fails the claim waits for the claim and then passes.
  Example:
  ten nudges during a pass produce ten queued passes,
  run one after another after the first.
  Pro:
  every nudge is answered without waiting for a new trigger.
  Con:
  processes wait,
  which the repository's
  concurrency rule forbids without a recorded reason,
  and ten passes cost ten times the measured pass.
- Ranking:
  A > B,
  because B reintroduces waiting and multiplies the cost the rule exists to bound.

### V5. A cost record that cannot be read never refuses a pass

- A,
  adopted:
  an unreadable claim folder,
  mark or start record is reported and the pass continues.
  Example:
  `run/` replaced by a file:
  the handler prints that the cost device could not be read,
  passes,
  attaches the
  clone,
  and files its receipt.
  Pro:
  the device cannot become an availability bug,
  and the route's safety still rests on the per-slot claim.
  Con:
  a
  corrupt record can let passes run unbounded until it is fixed.
- B,
  refuse the pass when a cost record is unreadable.
  Example:
  the same replacement:
  the caller is refused,
  the clone stays unmounted,
  and `doctor` reports the record.
  Pro:
  the bound is never silently unenforced.
  Con:
  a file in `/run` decides whether a worktree is available,
  which is
  a bigger failure than the cost the file bounds.
- Ranking:
  A > B,
  because the device is cost and the per-slot claim is safety,
  so the failure of the cost device must
  not take the safety path's availability with it.

### V6. The three cost records' names and place

- A,
  adopted:
  `/run/git-worktree-clone-mount-sweeps` holding `run/`,
  `another-pass-is-due` and `recent-starts`.
  Example:
  `ls -l` on the folder shows the three names,
  all root-owned,
  the two files `0644` so `doctor` can read them.
  Pro:
  per boot,
  cleared with the boot,
  beside the receipts and readiness folders,
  no migration,
  no configuration,
  and
  names that say what each holds.
  Con:
  three new names in `/run` that a person may read and wonder about,
  which
  `doctor` answers.
- B,
  the mark inside the readiness record and the starts inside the receipt folder.
  Example:
  a `pass_due` field in the readiness record and a file per start under the receipts folder,
  named by a
  timestamp.
  Pro:
  two folders instead of three.
  Con:
  a readiness record that changes format for a cost device,
  and a folder of
  start files that must be listed to count,
  which is the listing the slot store's answer avoids.
- Ranking:
  A > B,
  because B changes a published format and adds a listing,
  and A adds names only.

### V7. The merge device and the rate rule land together in step 22

- A,
  adopted:
  both are built in step 22,
  with the measured `B` and `P`.
  Example:
  after step 22,
  `doctor` prints `2 of 4 starts used in the last 600 seconds`;
  before it,
  it prints that the
  rule is not in force because its measurement is owed.
  Pro:
  no provisional limit ships,
  rule `LQM` and the decision are satisfied,
  and the record's shape is built with the
  number it holds.
  Con:
  until step 22,
  a nudge burst is unthrottled and the merge device's second pass is the only
  bound.
- B,
  build the merge device earlier with a provisional bound,
  as step 7 shipped a provisional deadline.
  Example:
  before step 22,
  `doctor` prints `4 of 4 starts used in the last 600 seconds` from a provisional `B`.
  Pro:
  the bound exists as soon as step 19 enables ZED.
  Con:
  the shipped number would rest on no measurement,
  which the
  decision's own sentence forbids and the deadline's precedent does not cover,
  because a wait needs some deadline and a
  bound needs a measured cost.
- Ranking:
  A > B,
  because the deadline had to exist for the wait to compile,
  and the bound does not.

### V8. The timeout text points at `doctor` instead of reading the cost records

- A,
  adopted:
  the timeout's fixed sentence names the possibility of a held pass and the
  `doctor` report that explains it,
  and the wait reads no cost record.
  Example:
  a held add's report ends with
  "a pass can be held back by the route's cost rule;
  `doctor` reports the last starts and whether a pass is due".
  Pro:
  the waiting path gains no read,
  so the latency the step 7 measurement recorded stays what it is,
  and the sentence cannot go stale against a number that has not shipped.
  Con:
  the report cannot say that the bound is why this particular wait ended.
- B,
  read `recent-starts` on the timeout and name the bound as the cause.
  Example:
  the same report says
  "the cost rule allowed 4 starts in the last 600 seconds,
  and this request's trigger was held".
  Pro:
  the report names the actual cause of that wait.
  Con:
  a root-owned record gains a trusting reader on a path every wait takes,
  and the sentence changes shape when step 22's numbers change.
- Ranking:
  A > B,
  because the held case is rare and `doctor` exists for the explanation,
  while every normal wait would pay B's read.

## Questions only the project can answer

### Q1. Which trigger runs after the checkout becomes ready

- A,
  recommended:
  the drop-in `ExecStartPost=` on `zfs-mount.service`,
  running `boot-reconcile`.
  Example:
  a person loads the pool's key and runs `systemctl start zfs-mount.service`;
  the drop-in runs the handler,
  readiness turns `ready`,
  and a waiting add's clone is attached with no further caller.
  Pro:
  uses the unit this host already mounts with,
  adds one drop-in file,
  changes no mount path,
  and fires at boot and
  on every later run of that unit.
  Con:
  a checkout mounted by a person's `zfs mount` fires nothing,
  a hooked boot runs
  two passes,
  and a `.mount` unit cannot carry the hook.
- B,
  a `.mount` unit for the checkout,
  with a companion `reconcile-…-on-checkout-ready.service` bound to it.
  Example:
  the same key load,
  but the checkout is mounted by its own unit;
  systemd starts the companion,
  which runs
  `boot-reconcile` and attaches the clone.
  Pro:
  fires on every activation of the checkout mount,
  whichever root or unit mounts it through systemd,
  and gives the
  checkout a unit state `doctor` can read.
  Con:
  it changes how the checkout is provided,
  needs the checkout excluded
  from `zfs mount -a` so two mounters do not race,
  and touches the accepted boot-protection decisions.
- C,
  no hook;
  the person's command only.
  Example:
  the same key load,
  with `doctor` printing the root `boot-reconcile` command and the clone staying unmounted
  until somebody runs it.
  Pro:
  no new unit,
  no new install step,
  and the fewest moving parts.
  Con:
  the proposal's obligation for a
  post-key-load reconciliation is unmet,
  and the route depends on a person's memory.
- Ranking:
  A > B > C,
  because A meets the obligation with the unit this host actually uses and no change to how the
  checkout mounts,
  B meets it too but changes the checkout's provisioning and so belongs to a larger design,
  and C
  leaves the obligation to a person.

### Q2. May the cost rule hold back a pass while a caller waits

- A,
  recommended:
  yes,
  the bound applies to every event- and nudge-driven pass,
  and the caller's deadline reports the
  possibility.
  Example:
  with a candidate bound of three passes in ten minutes already spent,
  four callers add at once;
  the fourth waits,
  its clone is attached at the
  next admitted trigger,
  and its report says a pass was held back and names `doctor`.
  Pro:
  the bound is simple and applies to the case it is for,
  a burst;
  the caller's request is durable and its mount
  happens later.
  Con:
  a caller can time out because of the bound rather than because of a slow mount.
- B,
  no bound until it can be proved never to hold a requested pass.
  Example:
  the same four callers,
  all served at once,
  with four passes running.
  Pro:
  no caller ever times out because of a limit.
  Con:
  no bound can distinguish a waiting caller's nudge from any
  other write,
  so B is in practice "no bound",
  which is the state the decision asks to end.
- C,
  no bound at all;
  the trigger set is the whole answer.
  Example:
  the same four callers,
  plus a shell loop writing the property,
  which runs a pass per write.
  Pro:
  the fewest parts.
  Con:
  the decision's sentence that a bound must ship after measurement is not carried out,
  and
  a loop spends root's time with nothing to stop it.
- Ranking:
  A > C > B,
  because A bounds the cost while keeping every request durable,
  C leaves the measured cost
  unbounded where the decision asked for a bound,
  and B cannot be implemented as stated and would be C with more words.

### Q3. Who may fire a trigger by hand, and what `doctor` prints

- A,
  recommended:
  `doctor` prints both commands,
  the root `boot-reconcile` and the account's `zfs set` nudge,
  and runs
  neither.
  Example:
  the lost-event report above,
  whose last lines are the two commands;
  the account copy-pastes the `zfs set`
  line and its clone is attached by the next pass.
  Pro:
  the no-waiter case has a lever a person can always use,
  and the account already holds `userprop`,
  so printing
  the line adds no capability.
  Con:
  a printed `zfs set` line invites a loop,
  which is the cost the bound bounds.
- B,
  only the root command is printed.
  Example:
  the report prints the `boot-reconcile` line and says nothing about a nudge.
  Pro:
  one lever,
  and no invitation to write the property by hand.
  Con:
  an account that cannot use `sudo` at that
  moment has no lever,
  and the nudge stays an undocumented capability.
- C,
  neither is printed;
  the report names the triggers and the journal.
  Example:
  the report says the next trigger is a ZED event and gives `journalctl` lines only.
  Pro:
  `doctor` stays a report.
  Con:
  the no-waiter case has no person's lever,
  and the answer's "we control the
  triggers" reaches only the units.
- Ranking:
  A > B > C,
  because A gives the person the lever the decision's premise assumes while adding no authority,
  B gives one lever at the price of hiding an existing capability,
  and C leaves the no-waiter case to time.

## Dependencies on other designs and on the plan

- D1.
  The provider hook's installation,
  the checkout dataset and the propagation it needs are plan step 19's and the
  sibling design "What re-applies private propagation after a replacement mount".
  This design assumes the propagation
  is applied by whatever mechanism that design names before a readiness pass sweeps,
  and the hook's pass refuses with
  the checkout field when it is not,
  as the readiness check does today.
- D2.
  The add route's nudge call is step 10's,
  and the removal's is step 14's.
  This design assumes both write one nudge
  per wait with a fresh nonce,
  and nothing else.
- D3.
  The remedy for a registered but unmounted slot after a timeout is the add route's:
  whether a re-run add adopts
  the slot or refuses it is step 10's design,
  and this design only reports the state `doctor` shows.
- D4.
  The unmount half of an `unmount` intent is step 9's,
  and this design's triggers recover a lost `unmount` event
  the same way they recover a `mount` event;
  what the unmount then does is that step's.
- D5.
  A cancelled or interrupted removal's state,
  and what its cancel command proves,
  are the sibling designs of those
  items.
  This design reports a standing `unmount` intent as `doctor` sees it and does not interpret it.
- D6.
  The slot identity store's per-slot records and their `doctor` findings are already accepted,
  and this design
  reuses the claim and the publish pattern without changing either.
- D7.
  The plan needs three edits after acceptance:
  the four shipped sentences that name step 10 for `doctor` (F30),
  the step 7 and step 22 "Open items" sentences and the open-item entry,
  and the "Gaps" list,
  which gains the cost
  record family of V6.
  This design makes none of them.

## What this design does not cover

The pass's own checks and its order (step 8's),
the unmount and staging (step 9's),
the add route's registration and
fill (step 10's),
the dirty-main restore (step 11's),
the managed lock (step 13's),
removal (steps 14 to 17),
the rest
of `doctor` (step 18's),
the migration,
the units' full bodies and the zfs delegation (step 19's),
sanoid (step 20's),
dependency repair (step 21's),
the numbers (step 22's) and the host-kernel qualification (step 23's).
It does not design
a timer,
a periodic reconciliation,
a queue of requests,
or a second checkout.
It does not design the start probe's
record,
and it reads that record's absence as a refusal by the pass,
which is that design's answer.

## Sources read

- `doc/decision/cli-git-zfs-worktree-design.md`,
  whole file.
- `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
  "ZED reconciliation and mount safety",
  "Boot and shutdown
  unit",
  "Requests,
  results and ordering",
  "Descriptor mount and unmount",
  "Waiting and refusal",
  "Scoped delegation",
  "Project decisions" (the lost-last-event entry),
  "Open items the decisions leave",
  "Acceptance tests the build still
  owes" (the boot and ZED bullets).
- `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
  steps 7,
  8,
  10,
  14,
  18,
  19,
  22,
  the open-item list and
  the gaps list.
- `doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
  whole file.
- `doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  whole file.
- `doc/decision/cli-git-zfs-staging-under-cache.md`,
  "The handle record's place".
- `package/git-policy/cli/SPEC.md`,
  "Management grammar",
  "Doctor",
  "The clone's mount intent,
  receipts and
  readiness",
  and the credential paragraph.
- `package/git-policy/cli/doc/zfs-mount-receipt-deadline-measurement-2026-11-21.md`,
  whole file.
- `package/git-policy/cli/src/native/worktree_clone_mount_protocol/`,
  `worktree_clone_mount_handler/`,
  `lock/single_owner_claim.rs` and `binary/binary_zfs_sweep_guest_tests.rs`,
  the files the facts name.
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
  `man/systemd.mount.xml`.
- This host's `/usr/lib/systemd/system/zfs-mount.service`,
  `/usr/lib/systemd/system/zfs-zed.service`,
  `/etc/default/zfs`,
  and the read-only probes listed in "Facts".

## Revisions

- 2026-11-22T17:29:03-05:00,
  written by an agent:
  the first version,
  unapproved and non-final,
  carrying this session's two coordinator corrections
  (the provider section names what the project installs or decides and where each part is recorded,
  and the acting-manager and signalling boundaries are stated).

[decision]: ../decision/cli-git-zfs-worktree-design.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[spec]: ../../package/git-policy/cli/SPEC.md
[step8]: ../decision/cli-git-zfs-step-8-sweep-choices.md
[store]: ../decision/cli-git-zfs-slot-identity-store-format.md
[staging]: ../decision/cli-git-zfs-staging-under-cache.md
