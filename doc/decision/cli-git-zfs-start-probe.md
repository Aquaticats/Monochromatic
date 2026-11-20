# cli-git ZFS start probe decision

## Status

Accepted 2026-11-20 in the question tool,
under the answer "Accept:
skip goes ahead,
claim early (Recommended)".
The design is [`doc/planning/cli-git-zfs-start-probe-design.md`][design] at commit `05d1102848`,
on the branch `docs/planning-cli-git-zfs-start-probe-design`.
That commit is the design as accepted,
and this record adds no sentence to its body:
the only addition is one note at its top,
which names this record and the acceptance date.
The evidence,
the probes,
the rejected options and every owed measurement stay in the design,
and this record restates each rejected option in one sentence,
leaving the evidence to the design.
The seventh check of the design,
read on the diff to that commit,
left seven lows and no medium finding;
none of them blocks,
and each is fixed in this record's own copy of the statement it concerns,
as [Corrections carried from the seventh check](#corrections-carried-from-the-seventh-check) says.
That check's entry in the design says the nine lows of the check before it were not touched.
That is false in part:
the previous LOW-2 and LOW-4 were answered in part,
in the capability bullet's naming of `mount` and `mount_setattr` and in the Answers record's Q6 sentence.
This record was written by an agent;
the acceptance it records is the project's.

## Decision

The project accepted the start probe design,
answering "Accept:
skip goes ahead,
claim early (Recommended)".
In plain words:
the root probe at boot,
the account companion before each managed add,
and the first real attach's record are accepted as the design states them.
The companion's printed skip lets the add go ahead,
rather than refusing the managed add.
The first-attach claim is taken on entry,
before the walk to the slot,
rather than at `fsopen`.
Neither deadline veto was taken,
so the design's own choices stand:
no probe deadline value ships before step 6's measurement,
and no companion deadline ships before step 10's.
The answers of 2026-11-19 stay as they are,
and each one has its own section,
which quotes it.
The design's two limits stay limits:
its Ubuntu reading is mainline kernel source and not a run,
and its new guest controls have not run ([Limits this record keeps](#limits-this-record-keeps)).

## The acceptance question of 2026-11-20

The question was put in the question tool,
and its answer reads:
"Accept:
skip goes ahead,
claim early (Recommended)".
It accepted the design with the two choices under it,
and it took neither of the two deadline vetoes that [Questions the owner answered][answered] names.

### Q-a: what the companion does when it cannot run its check

The answer to Q-a,
option a:
"when the account helper cannot run its check (a host that refuses user namespaces,
denies their permissions,
or blocks a setup mount),
it prints a line naming the refused call and the add goes ahead."

This record states the skip's reach in one phrase,
because the design words its third case two ways:
a host that refuses the companion a user namespace,
one that denies the capabilities inside it,
and one that refuses a mount the setup makes inside it,
the private propagation and the scratch tmpfs included.
The skip covers no other failure,
bar the two residues the design names:
a first mapping write whose line or opener is not the one it builds,
and a host whose `/proc` is not mounted.
The alternative,
refusing the managed add instead,
is not taken;
it would turn the managed route off on those same three reaches.

The design's setup calls are the two `unshare` calls,
the `setgroups` write,
the two mapping writes,
the private propagation,
and the `mount` that puts the scratch tmpfs in place.
The propagation is made with `mount_setattr` and `AT_RECURSIVE` with `attr.propagation = MS_PRIVATE`,
or with `mount` and `MS_REC | MS_PRIVATE`.
The setup ends when the scratch tmpfs is in place.
Every answer from any call after that point,
`fsopen`,
`fsmount` and `move_mount` included,
and every answer from any call the skip list does not name,
is an arm failure that refuses the add.

#### Q6's own example call

Q6 as asked gave the companion's case as "a kernel update changes what a user namespace may pass to `mount_setattr`".
That call is the propagation when the companion makes the propagation with `mount_setattr`,
so it sits on the skip side when it answers `EPERM` during the setup,
and that is the one case where Q6's own example call still skips.
A `mount_setattr` after the scratch tmpfs is in place,
and every `EPERM` from `fsmount`,
is an arm failure and refuses the add.

#### The boundary of the `EACCES` bullet

The skip also covers a `mount` call answering `EACCES` from mount mediation during the setup:
the `mount` that puts the scratch tmpfs in place,
and the propagation when the companion makes that with `mount`.
That bullet ends when the scratch tmpfs is in place,
as the capability bullet does.
The refusal control's bind mount is planted after the setup,
and the design names no call for it,
so an `EACCES` there is an arm failure and not a skip.

#### The capability checks the bullet covers

The bullet covers any capability check the setup's `mount` and `mount_setattr` make,
`may_mount` among them.
The other two checks of that kind in the setup are `may_change_propagation` and `mount_capable`,
and no other `EPERM` source is reachable there,
on the seventh check's reading of v7.3 rather than a proof that no site is missed:
`mount_too_revealing` answers false for tmpfs,
`can_change_locked_flags` returns the mount's own flags unchanged for a propagation-only call,
and the idmap `EPERM` sites need an idmapped mount.
So the call-and-errno rule is safe for the setup's calls,
and a `mount` refusal after the setup stays the fail-closed side.

#### What M11 must show for each side of the boundary

M11 owes a control for the skip side,
beside its `fsmount` fixture that must refuse:
a filter for `mount` answering `EPERM` during the setup,
and one for `mount_setattr` when the propagation uses it,
each of which must print the skip line naming that call.
Its fixture whose child answers `EPERM` from `fsmount` covers the arm-failure side,
and its `mount` filter answering `EACCES` and its `unshare` filters cover further skip lines.

### Q-b: where the first-attach claim sits

The answer to Q-b,
option a:
"the first-attach claim starts on entry,
before the walk to the slot,
so a refused walk or a drifted slot becomes that boot's record."

The claim is taken by the first attempt that passes the key check,
on entry and before the walk that opens the slot.
A refusal at the walk,
at the slot's first-observation record or at the identity compare therefore becomes that boot's record,
with its step name and its errno,
or `errno none` where the refusal is a comparison.
A refusal at the key check takes no claim and writes nothing.
The alternative,
claiming at `fsopen`,
is not taken:
it would keep only the attach calls' outcomes and lose a drifted slot's record.

## Q1: whether the plan bars the probe's unmount arm

Chosen:
"Product mounts only",
which the design records with the option's "(Recommended)" marker.
The plan's sentence covers the product's mounts,
so the probe's own move and unmount arms are built in step 6,
inside the namespace and the tmpfs the probe makes for itself.
Not taken:
the literal reading,
which would build the arm in step 9 and ship a gate whose unmount half never ran.

## Q2: what a failed arm disables

Chosen:
"Refuse new attaches only",
which the design records with the option's "(Recommended)" marker.
An arm that ran and failed refuses new managed attaches,
and every other operation keeps its route:
an unused slot takes the reflink or the ordinary Git route,
`boot-reconcile` attaches nothing and keeps every intent,
and removal of a mounted clone is refused only when the failed arm is the move,
the unmount or the scan.
Not taken:
mirroring the module gate,
which would strand the removal of an unmounted clone,
and warn-and-continue,
which would proceed into the attach the probe had just seen fail.

## Q3: whether a setup-time probe dataset exists

Chosen:
"Dedicated dataset",
which the design records with the option's "(Recommended)" marker.
The source is the dataset `zpcachyos/nosnap/git-worktree-probe` with the empty snapshot `ready`,
and plan steps 18 to 20 carry its creation,
its configuration field and its exclusion.
Not taken:
a snapshot of the checkout,
because the source can be absent right after a migration,
and arms can silently not run.

## Q4: whether an unexercised arm may enable the route

Chosen:
"Block until it has run",
which the design records with the option's "(Recommended)" marker.
An arm that could not run keeps the managed route blocked,
with the per-operation effects the failed-arm answer states.
Not taken:
reporting the gap and keeping the route enabled,
which would let a broken kernel pass with the gap visible only in `doctor`.

## Q5: whether a missed deadline fails closed

Chosen:
"Fail closed",
which the design records with the option's "(Recommended)" marker.
A probe that misses its deadline writes a timed-out result,
the record is not current,
an add refuses and names the reason,
an unused slot takes the reflink or ordinary Git route,
and existing bindings and intent stay.
Not taken:
failing open with a warning,
which would attempt an attach on the strength of a record that says the probe could not finish.

## Q6: whether the account runs a companion probe

Chosen:
"Root plus account companion",
which the design records as the project's choice,
its own first rank having been the other option.
The account runs the small user-namespace companion before each managed add,
and the root probe stays the only evidence about the dataset mount path.
The project took the companion knowing that it never touches ZFS and skips on a host that refuses user namespaces;
the wider reach of that skip was this design's own when Q6 was asked,
and the acceptance took it as [Q-a](#q-a-what-the-companion-does-when-it-cannot-run-its-check) states.
Not taken:
root only,
which would leave an account-side regression of the walk or the mount surface,
and that regression would be seen only where something mounts as the account.

## Q7: whether the first real attach records its outcome

Chosen:
"Write first-attach",
which the design records as the project's choice,
its own first rank having been the other option.
The handler process that made the first real attach after boot writes its outcome to `first-attach`,
which sits beside the probe's record,
and `doctor` shows it beside the probe's result.
Not taken:
the probe record only,
which keeps no outcome from a real clone,
a real slot and the host namespace.

## Limits this record keeps

The design's two limits stay limits,
and the acceptance rests on neither of them being measured:

- The Ubuntu behaviour is a reading of mainline kernel source,
  not a run on that guest's patched 6.8 kernel;
  `M6` and `M11` record which call it named,
  and a run on that guest is still owed.
- The new guest controls have not run:
  the no-`CONFIG_USER_NS` guest,
  the mount `EACCES` filter,
  the `mount` and `mount_setattr` `EPERM` filters this record's Q-a section adds,
  the SELinux first-write reading,
  the `fsmount` fixture and the restricted-namespace arm.

Every item under [Owed measurements][owed] is unrun,
and nothing here authorizes work on this host's pool.

## Amendments made with this record

Four sentences outside the design needed an amendment note beside them,
and each note gives the new statement and the date:

- The plan's preface,
  "no unmount runs before the third",
  in [`doc/planning/cli-git-zfs-worktree-implementation-plan.md`][plan]:
  the note says the sentence covers the product's mounts only,
  and that the probe's own move and unmount arms are built in step 6.
- The proposal's floor sentence,
  "The syscall floor is Linux 5.6 for `openat2`;
  the other mount APIs used here start at 5.2",
  in [`doc/planning/cli-git-zfs-worktree-design-proposal.md`][proposal]:
  the note says the managed route's effective floor is Linux 6.6,
  because a kernel that refuses `FSCONFIG_CMD_CREATE_EXCL` leaves the attach arm unexercised,
  and an unexercised arm blocks the route.
- The proposal's exclusion count,
  "Use the eight exclusions named in the planning decision",
  in the same file:
  the note says the probe dataset's own section is a ninth exclusion,
  written in step 20.
- Plan step 22,
  "The measurements that size waits and retention",
  in the plan:
  the note narrows "every shipped limit cited beside its measurement",
  and the acceptance text "to size any shipped deadline or optional scan interval",
  to step 7's mount-request deadline and step 9's unmount deadline,
  and says the probe's deadline and the companion's are sized by steps 6 and 10 beside their own runs.

Two other documents need no amendment:
[`doc/decision/cli-git-zfs-worktree-design.md`][worktree-decision],
and [`package/git-policy/cli/SPEC.md`][spec].
This design changes no decision of the accepted design,
and the SPEC's passages it cites are read,
not changed.

## Corrections carried from the seventh check

Each of the check's seven lows is answered in this record's own words,
and the design's body keeps the text it was accepted with:

- LOW-A,
  the third reach worded two ways and the alternative naming only the first:
  the reach is stated once,
  in one phrase,
  and the alternative carries the same three reaches,
  under [Q-a](#q-a-what-the-companion-does-when-it-cannot-run-its-check).
- LOW-B,
  Q6's literal example call and the unnamed propagation call:
  the setup's calls name the propagation,
  and [Q6's own example call](#q6s-own-example-call) says which side that call falls on.
- LOW-C,
  "the nine lows were not touched":
  [Status](#status) states the correction.
- LOW-D,
  the setup's calls named in one bullet only:
  the setup's calls are listed once,
  and every other answer is the arm-failure side,
  under [Q-a](#q-a-what-the-companion-does-when-it-cannot-run-its-check).
- LOW-E,
  no control for the skip side of the narrowed bullet:
  [What M11 must show for each side of the boundary](#what-m11-must-show-for-each-side-of-the-boundary).
- LOW-F,
  the `EACCES` bullet's missing boundary:
  [The boundary of the `EACCES` bullet](#the-boundary-of-the-eacces-bullet).
- LOW-G,
  the bullet's title citing one capability check of three:
  [The capability checks the bullet covers](#the-capability-checks-the-bullet-covers).

The check's cosmetic note,
that a rewrap left short lines in the design,
changes no statement,
so this record carries nothing for it.

[answered]: ../planning/cli-git-zfs-start-probe-design.md#questions-the-owner-answered
[design]: ../planning/cli-git-zfs-start-probe-design.md
[owed]: ../planning/cli-git-zfs-start-probe-design.md#owed-measurements
[plan]: ../planning/cli-git-zfs-worktree-implementation-plan.md
[proposal]: ../planning/cli-git-zfs-worktree-design-proposal.md
[spec]: ../../package/git-policy/cli/SPEC.md
[worktree-decision]: cli-git-zfs-worktree-design.md
