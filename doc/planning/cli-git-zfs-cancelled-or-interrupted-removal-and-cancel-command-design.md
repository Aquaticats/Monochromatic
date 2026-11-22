# What a cancelled, timed-out or killed removal leaves, and what the cancel command proves and does

Non-final,
unapproved design proposal by an agent.
The project has not reviewed it,
no other agent or model has reviewed it,
and nothing in it is final.
It designs three open items of the accepted ZFS worktree design that describe one thing,
the state a removal leaves when it does not finish and the way back from it:
"What a cancelled removal restores",
"What the kept cancel command proves and does",
and "What a timed-out or killed removal leaves besides its manual way back".
The plan lists all three for step 17 as blocking writer design work,
with the restored state put with the cancelled-removal question ([plan][plan],
"Open items and the step that needs each").

It writes one file and designs no code.
It edits no other document,
no file under `doc/handover/` or `doc/decision/`,
and nothing `file-enforcer` generates.

What it read,
all of it in this worktree unless a line names a source clone:
the accepted ZFS worktree design and its record,
the whole design proposal,
the staging decision and its accepted design,
the step-8 sweep choices,
the slot identity store format,
the implementation plan's steps 9 and 14 to 18,
its open-item list and its gap list,
`package/git-policy/cli/SPEC.md` in the sections it names below,
the code under `package/git-policy/cli/src/native/` that the route already has,
and three source clones it made in its scratch folder:
Git at `v2.56.1`,
OpenZFS at `zfs-2.4.4`,
and Linux at `v7.3`.

What it ran:
`gh repo clone` for those three trees,
each followed by `git remote set-url --push origin DISABLED` before anything else,
reads of their files by line number,
and the repository's linter over this file.
It ran no ZFS command,
changed no pool,
mounted or unmounted nothing,
started no guest and no container,
opened no `/dev/zfs`,
and ran no wrapper command.
Every fact below is read from source at a named tag or from a file in this repository,
and no sentence here reports a run of the ZFS route.

The design is [the decision record][decision] and the whole design is [the proposal][proposal].
The staging work it rests on is [the staging design][staging-design] and [its record][staging-record].
The build order is [the plan][plan],
whose step 17 builds this.

## The three open items, in the records' words

The proposal states each item and designs none of them ([proposal][proposal],
"Open items the decisions leave",
lines 1703 to 1724).
Quoted whole,
in its words:

> What a cancelled removal restores.
> The answer's words:
> "Kill or otherwise remove or destroy the holders whenever --force is specified,
> auto-cancel if not."
> Cancellation happens after the order "B:
> Destroy at once" keeps:
> the clone's own snapshots go first and the head destroy is the step a holder blocks,
> so a cancelled removal can find its own snapshots already destroyed.
> What the caller gets back is undesigned:
> whether the clone is reattached as it stands,
> whether the removal record is reversed or kept,
> and what `doctor` reports.
> The loss is plain:
> those snapshots do not come back,
> and the reattach the busy-unmount answer allows restores visibility,
> not history ([Cancelling a blocked removal][cancelling] and [Busy unmount after a staging move][busy-unmount]).

> What the kept cancel command proves and does.
> The answer's words:
> "Keep the command (Recommended)":
> a guarded `cancel-worktree-removal <slot>` stays as the manual way back,
> including for a removal whose wrapper was killed after it recorded the unmount or that ran past its deadline,
> reports any snapshots already destroyed,
> and cannot restore them.
> What "guarded" means here is undesigned:
> how the command proves the slot it names is the slot it may reverse under the per-slot claim,
> how it settles outstanding destroy requests,
> what it prints when an outcome is unknown or the head is gone,
> and how it relates to auto-cancellation,
> which acts with no person present.

> What a timed-out or killed removal leaves besides its manual way back.
> The answers' words:
> "Keep the command (Recommended)",
> whose command "stays as the manual way back,
> including for a removal whose wrapper was killed after it recorded the unmount or that ran past its deadline",
> and "Kill or otherwise remove or destroy the holders whenever --force is specified,
> auto-cancel if not."
> A deadline is not cancellation,
> and the recorded `unmount` intent stands across a restart ([Waiting and refusal][waiting-and-refusal]),
> so such a removal is neither cancelled nor finished:
> the durable request may still act.
> The kept command gives a person a way back,
> but nothing says what state the clone and any staged mount are in,
> who reconciles them,
> what the caller is told,
> or how the worktree is returned to use.
> The item for a removal that cancels itself is separate.

## What the project has decided that binds this design

Each passage below decides a point this design applies,
and each is quoted with its path and line.

### The removal and its order

The project answered "B:
Destroy at once",
which fixes the order this design's states follow ([record][decision],
lines 34 to 37):

> A normal authorized removal destroys the clone and its own snapshots in that invocation,
> answering "B:
> Destroy at once".

The proposal's removal runs the proof,
records the removal intent,
requests `unmount`,
persists the destruction plan after the unmount,
destroys the verified snapshot set,
destroys the head,
and only then settles the registration ([proposal][proposal],
"Removal and recovery",
steps 1 to 5,
lines 854 to 976).
Two sentences of it decide what this design may and may not reverse ([proposal][proposal],
lines 963 to 966):

> Settle recorded destroys only after proving the expected objects are gone,
> never from a missing name alone.
> While the head is present,
> require it to answer with its recorded live GUID and an empty live snapshot list.

### Cancellation, and the two arms

The answer's words,
quoted by the proposal and by the record ([proposal][proposal],
line 978,
and [record][decision],
line 39):

> Kill or otherwise remove or destroy the holders whenever --force is specified,
> auto-cancel if not.

The manual way back is the project's own answer ([record][decision],
lines 39 to 44):

> A guarded `cancel-worktree-removal <slot>` stays as the manual way back,
> including for a removal whose wrapper was killed after it recorded the unmount or that ran past its deadline.
> It reports any snapshots already destroyed and cannot restore them.

The draft this design inherits its recovery reasoning from said what any reversal must prove,
and the proposal keeps it for whatever reverses a removal,
the kept command included ([proposal][proposal],
lines 1017 to 1023):

> under the per-slot claim,
> settle all outstanding destroy requests and prove the clone,
> original lineage,
> Git registration,
> mount state and exact destruction outcomes before reversing intent.
> Report any confirmed history loss before doing so and retain that report.
> If a destroy outcome is unknown or the head is gone,
> do not issue `mount`;
> report the matching resume step or administrator recovery required.

### The reattach, and the one remount the defaults allow

The busy-unmount answer is `Reattach when safe (Recommended)` ([proposal][proposal],
"Busy unmount after a staging move",
lines 1436 to 1444),
and the sentence this design applies is:

> It moves the same mount back to the original slot only after revalidation,
> and retains it staged otherwise.

The default it qualifies,
with its one exception ([proposal][proposal],
"Decided defaults",
lines 1492 to 1498),
keeps `No implicit remount after a failure or restart`,
and its exception is:

> The exception is the checked reattach of a removal that cancels itself without `--force`,
> which the cancellation and busy-unmount decisions otherwise require,
> because an implicit remount would race the standing `unmount` intent and any staged mount.

### Waiting, deadlines and refusal

A deadline is not cancellation,
and a timeout report has a stated shape ([proposal][proposal],
"Waiting and refusal",
lines 782 to 795):

> A deadline is not cancellation:
> the durable request may act later.
> Do not retry against an unverified mount or submit a conflicting request without verifying and recording recovery.

The same passage requires a refusal to name the failed operation,
dataset,
slot,
actual state and reason,
to name a holder on `EBUSY` where one is known,
and to give the staged location and the administrator's recovery instructions when staging or rollback failed.

### Staging, the claims and the records this design reuses

The project accepted the staging design on 2026-11-20,
answering "Accept,
boundary (a) (Recommended)" ([staging-record][staging-record],
"Decision").
Its text is accepted and binding for what it states ([staging-design][staging-design]):
the wrapper creates one folder per removal under `~/.cache`,
the handler binds it,
makes the bind private and moves the clone onto it,
the handler records the folder's file handle and identity in a staged-folder record beside the folder,
a later sweep unmounts from the recorded handle,
and the wrapper's cleanup is `rmdir` only and stops at any mount.
Three parts of its settlement this design uses directly:

- A cancelled removal has its own receipt kind,
  naming the clone at its canonical slot and the bind gone from the host namespace,
  and it is the only receipt on which the wrapper removes a staging folder while the clone still stands at its slot
  ([staging-design][staging-design],
  A5).
- A record from another boot is void,
  and the wrapper settles the folder,
  the record and the claim on its own mount-table read ([staging-design][staging-design],
  "The staged-folder record").
- The staging claim is the wrapper's own claim of a new kind,
  holding the nonce of the `unmount` intent it waits for and the folder's identity,
  and its sweep runs from a removal and only for that repository's claims ([staging-design][staging-design],
  "The staging claim").

### A held slot claim, and the claim the root side takes

The project answered how a sweep behaves when another handler holds the slot ([step-8][step-8],
"A slot whose claim is held"):

> "A:
> holder re-reads after":
> a sweep that finds the slot's claim held refuses,
> and the holder reads the intent again after it releases the claim,
> so a request that arrives during a sweep is not lost.

### The threat model

The project's sentence for this machine,
and the record's reading of it ([staging-record][staging-record],
"The handle record's place",
and [staging-design][staging-design],
lines 221 to 237):

> We have to assume my machine isn't compromised.
> sudo is passwordless here anyway.

> The account is trusted,
> and root and the account are not a security boundary here.

The staging design followed that sentence "without dropping one on the strength of the sentence alone",
and this design does the same,
saying for each check whether an argument survives the sentence or the check exists only for a hostile account.

### No configuration, and no secret

The project answered that the route works with no configuration file ([step-8][step-8],
"With no configuration file"):

> With no file,
> the root handler finds the checkout as the one mounted ZFS dataset whose last name component is
> `monochromatic-git-checkout`,
> takes the container beside it,
> and the start probe finds its dataset by its design name too.

The wrapper never asks a person for a credential ([spec][spec],
line 7348,
in its auto-push section):

> Avoid ever asking for someone to type a username or password.
>
> Passphrases count too (Recommended).

Nothing in this design prompts,
and a place where authority it does not have would be needed is a refusal that names the next step.

The same rule reaches the host's service manager.
Nothing in this design asks `systemctl`,
the system bus or polkit for anything:
it writes records,
asks the root handler through one clone property,
and reads the files those two wrote.
The handler's unit installation,
its signals and the ending of holders are plan step 19's and the forced-arm item's,
because an unprivileged request to signal another account's process or a system unit's task does not simply fail here:
it puts a password dialog on the desktop of the person using the machine and waits on that person.
What such a request answers is read from systemd and polkit at this host's versions and from `kill(2)`,
and the run that would show it is owed in the guest,
as M9.

### The repository's own rules

- Concurrency,
  `AGENTS.md` rule `NLK`:
  "Concurrency is lock-free first (atomic rename,
  `O_EXCL`,
  one owner per item);
  an unavoidable lock covers the smallest unit,
  never a whole repository,
  with its reason written beside it."
- Shipped limits,
  `AGENTS.md` rule `LQM`:
  "Shipped resource limits (unit caps,
  timeouts,
  size bounds) cite a workload measurement or owner decision beside the value".
- Where worktrees live,
  `AGENTS.md` rule `WTU`:
  "Every worktree of this repository,
  Git's own or a Btrfs or ZFS clone,
  lives under `.claude/worktrees/` in the main checkout".
- Paths come from the injected home,
  never a hardcoded user name (`AGENTS.md` rule `HOM`).

## What the wrapper already has, and what this design reuses

Nothing here is proposed again.
Each entry names where it lives and what this design takes from it.

- The intent grammar and its reader:
  `package/git-policy/cli/src/native/worktree_clone_mount_protocol/clone_mount_intent.rs:176` to `:270`,
  with the property names and bounds in `clone_mount_property_names.rs:58`.
  It accepts exactly `v1:<nonce>:mount` or `v1:<nonce>:unmount`,
  it separates a local value from an inherited one and from a malformed one,
  and `read_intent` reports an absent or inherited value as asking for no work (`clone_mount_intent.rs:261`).
- The intent writer and its gate:
  `clone_mount_intent_write.rs`,
  whose refusal wording already says to inspect the clone's property before any retry.
- The receipts and their reader:
  `clone_mount_receipt.rs:47` to `:180`,
  with the keys in their written order,
  the outcome words `done`,
  `refused` and `failed`,
  and the derived file bound.
- The wait and its timeout report:
  `mount_request_wait.rs`,
  with `PROVISIONAL_MOUNT_REQUEST_DEADLINE` and the timeout text that already says a deadline is not cancellation
  (`mount_request_wait.rs:298`).
- The generation claim with an owner named by process birth:
  `package/git-policy/cli/src/native/lock/single_owner_claim.rs:351`,
  whose doc states that a killed claimant leaves nothing to clean by hand
  and that the next claimant takes the next generation.
  This design takes a claim of that kind for a removal,
  and no new claim mechanism.
- The root handler's per-slot claim:
  `package/git-policy/cli/src/native/worktree_clone_mount_handler/per_slot_root_claim.rs:58`,
  a root-owned folder under `/run` with one claim file per slot,
  which only root takes and which the wrapper cannot take.
- The slot name grammar:
  `package/git-policy/cli/src/native/worktree_clone_mount/slot_name.rs:24`,
  and the bounded no-follow pointer reader:
  `package/git-policy/cli/src/native/worktree_clone_mount/clone_slot_git_pointer.rs:231`.
- The slot identity store:
  `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities`,
  root-owned,
  one record per slot,
  published by one `link`,
  and `doctor` reporting what is left over ([slot-store][slot-store]).
  This design reads no record there and changes none.
- The staging claim,
  the staged-folder record and its settlement:
  [staging-design][staging-design],
  "The staging claim" and "The staged-folder record".
- The attach path and its boundary refusal:
  `package/git-policy/cli/src/native/worktree_clone_mount/attach_the_clone.rs:33` to `:34`,
  where the walk's refusal of a slot that already stands on a mount is what stands in for "not already mounted here".
- The removal reporting style and its commands:
  `package/git-policy/cli/src/native/worktree/worktree_remove_reports.rs`.
- The refusals' exit codes and streams:
  `0` success,
  `1` the operation failed,
  `2` the caller or the configuration is wrong ([exit codes][exit-codes]).

## Facts

Each fact is read at a named tag or in a named file,
and the line numbers are those of the tree it names.

### Git

- F1.
  Git's own `git worktree remove` deletes the registration even when deleting the working tree failed
  (`builtin/worktree.c:1463` to `:1473`,
  Git `v2.56.1`,
  whose comment says "continue on even if ret is non-zero,
  there's no going back from here").
  The registration's directory is removed recursively by `delete_git_dir` (`builtin/worktree.c:149` to `:162`).
- F2.
  `git worktree remove` refuses a locked worktree unless `--force` is given twice,
  and the lock is a file named `locked` inside the registration directory (`builtin/worktree.c:1450` to `:1457`,
  `worktree.c:305` to `:322`).
- F3.
  `git worktree prune` skips a worktree whose registration holds a `locked` file (`worktree.c:937`,
  and `:966` to `:969`).

### OpenZFS

- F4.
  Setting a user property,
  and inheriting it,
  both need the `userprop` permission for a user property (`module/zfs/zfs_ioctl.c:1193` to `:1203` for inherit,
  `:4954` to `:4966` for set,
  OpenZFS `zfs-2.4.4`).
- F5.
  `zfs inherit` of a user property reaches `dsl_prop_inherit`,
  which records the property as inherited rather than setting a value (`module/zfs/zfs_ioctl.c:3126` to `:3190`,
  `module/zfs/dsl_prop.c:931` to `:940`).
- F6.
  A head dataset's destroy refuses `EBUSY` while its longhold count is not the expected one
  (`module/zfs/dsl_destroy.c:775` to `:778`),
  and refuses `EBUSY` while it has snapshots of its own (`module/zfs/dsl_destroy.c:783` to `:790`).
- F7.
  A snapshot batch prechecks every name and destroys nothing when a check fails,
  and names each failing snapshot in its error list (`module/zfs/dsl_destroy.c:588` to `:600` and `:630` to `:661`).
  A program error returns its errno without a per-name list and can leave partial effects
  (`module/zfs/dsl_destroy.c:662` to `:671`).
- F8.
  A snapshot batch ignores names that no longer exist (`lib/libzfs_core/libzfs_core.c:433` to `:456`).
- F9.
  Destroying a head destroys its bookmark entries with it (`module/zfs/dsl_destroy.c:1122` to `:1159`).
- F10.
  A dataset's GUID is random at each creation (`module/zfs/dsl_dataset.c:1202` to `:1204` and `:1766` to `:1769`),
  so the GUID,
  not the name,
  is the identity a settlement compares.

### Linux

- F11.
  A non-lazy unmount returns `EBUSY` while anything holds the mount or a child of it,
  and only `MNT_DETACH` detaches instead (`fs/namespace.c:2003` to `:2015`,
  Linux `v7.3`,
  with the two flags at `include/linux/fs.h:1398` to `:1399`).
  The busy test spans the peer copies of the mount (`fs/pnode.c:501`).

### This repository

- F12.
  The intent grammar has exactly two operations,
  so no value can say "cancelled" without changing the grammar
  (`clone_mount_intent.rs:46` to `:62` and `clone_mount_property_names.rs:58`).
- F13.
  A receipt's outcome word is at most seven bytes,
  and its keys are fifteen in a fixed order (`clone_mount_receipt.rs:47`,
  `:78`,
  `:153`).
  The longest of the three outcome words is `refused`,
  so no longer word can be added without changing that bound and the derived file bound.
- F14.
  The attach path refuses a slot that already stands on a mount,
  because the slot walk is where "not already mounted here" is decided (`attach_the_clone.rs:33` to `:34`).
- F15.
  The root handler's per-slot claim lives under `/run`,
  is root-owned,
  and an unprivileged wrapper cannot create a file there (`per_slot_root_claim.rs:58` to `:67`).
- F16.
  The wrapper's claim names its owner by process birth,
  and a claim whose owner died is replaced by the next generation with nothing to clean by hand
  (`single_owner_claim.rs:1` to `:30` and `:351`).
- F17.
  The wait's timeout text already tells the caller that the durable request stays on the clone and that a later
  sweep may still act on it (`mount_request_wait.rs:298`).

## Assumptions

- A1.
  The add's durable binding record names,
  for one slot,
  the clone's dataset name and GUID,
  its origin's identity and the registration it belongs to,
  and the wrapper can read it with no mount and no root.
  That record's home and format are another writer's open item,
  "Where the add's durable binding record lives,
  and its format",
  and this design depends on it without deciding it.
- A2.
  The persisted destruction plan holds,
  for one clone,
  each verified snapshot tuple with its outcome,
  and it can hold an outcome meaning that the removal ended without finishing.
  Its home and format are another writer's open item,
  "Where a removal's persisted destruction plan lives,
  and its format".
- A3.
  The binding record and the plan record are reachable after a reboot and while the clone is unmounted,
  because both are read by a removal that has already unmounted its clone.
- A4.
  The account does not sabotage its own removal,
  which is the project's threat model for this machine read the way the staging design reads it.
- A5.
  The handler's attach path can answer a request for a clone that is already attached at its recorded slot
  without attaching a second time.
  This design requires that behaviour and names it as a build item,
  because the path refuses an occupied slot today (F14).
- A6.
  The trigger set the project controls is the one the proposal names:
  the boot unit's start reconcile,
  the checkout provider's post-key-load hook,
  the wrapper's sweep nudge on a clone,
  and ZED's delivery to the installed handler ([proposal][proposal],
  "Triggers only,
  and we control the triggers",
  lines 1393 to 1408).

## Limitations

- L1.
  Nothing in this design was run.
  No guest,
  no container and no pool was started,
  no mount was made,
  and no wrapper,
  handler or binding invocation ran.
  Every guest arm below is owed.
- L2.
  The busy head destroy's cause is inferred,
  not measured:
  the design separates a new own snapshot from a remaining holder by the errno plus the live snapshot list,
  and no run has shown the pair on this route.
- L3.
  The notification of a holder is incomplete by record:
  a receipt carries no holder identity,
  so a busy answer names a holder only where the wrapper's own scan or a receipt's failed step names one.
- L4.
  Whether `zfs inherit` of the intent property works for the account under the delegated `userprop` grant is
  read from source (F4,
  F5) and unmeasured.
- L5.
  The fresh attach of an unmounted clone and the reattach of a staged one are both unmeasured on this route,
  and the staging design records the same limit for the reattach ([staging-design][staging-design],
  L5).
- L6.
  The binding record and the plan record are assumed,
  not designed here (A1,
  A2,
  A3).
  Where either home changes,
  the proof and the terminal record of this design move with it.
- L7.
  The wrapper's mount namespace may differ from the handler's,
  which the staging design records as a limit of its own settlement ([staging-design][staging-design],
  L13);
  this design reads the handler's namespace pair in the staged-folder record and refuses on a mismatch,
  and no arm measures the mismatch.
- L8.
  No number here is measured.
  The wait this design uses is the removal deadline step 9 introduces and step 22 sizes,
  and this design ships no new constant.

## The stop points

A removal can stop at any of the points below.
Each is given with the state of the clone,
any staged mount,
the mount intent,
the removal's own records,
the destruction plan,
and the Git registration and its lock.
The removal's records are the wrapper's removal claim (named under "The records this design adds"),
the staging claim and the staged-folder record,
and the destruction plan.

The removal writes its durable records in this order,
which this design fixes because a state has to be readable from the outside:
the removal claim,
then the staging claim and the folder,
then the intent,
then the plan.
The folder comes before the intent so that no sweep ever meets an `unmount` intent whose clone is mounted and whose
folder does not yet exist,
which the handler would have to refuse.

1.  Before any durable write.
    The clone is mounted at its slot and its identity is not yet proven.
    No staged mount,
    no intent of the removal's own,
    no plan.
    The registration and its lock stand as the add left them.
    A kill or a deadline here leaves nothing that acts on the clone,
    and a removal that cancels here is impossible,
    because nothing recorded a removal.
2.  The removal claim taken,
    before anything else.
    The same state as point 1 describes,
    plus a claim file naming a live process or a dead one.
    A kill here leaves a claim whose owner is dead,
    which the next claimant replaces with the next generation and nothing to clean by hand (F16).
3.  The staging folder and its claim,
    before any intent.
    The clone is mounted at its slot;
    the folder is empty and has no staged-folder record yet,
    because the handler writes that record before its first bind;
    the intent still carries what the add left,
    which is `mount` or nothing;
    the removal claim is held.
    A kill here leaves an empty folder and a claim,
    and no request that can act.
4.  The intent written,
    `v1:<nonce>:unmount`,
    and the wrapper waiting.
    The clone is mounted at its slot;
    the folder is empty and unrecorded;
    the intent names the unmount;
    the plan does not exist.
    A kill here leaves the standing request the owner's answer names,
    and the handler's next trigger can act on it,
    because the folder exists.
5.  The handler bound the folder and moved the clone onto it,
    before the unmount.
    The clone is mounted at the folder instead of its slot;
    the bind and the clone are the only mounts there;
    the intent stands;
    the staged-folder record,
    the claims,
    and still no plan.
    A kill here leaves a staged clone a later sweep finishes from the recorded handle.
6.  The clone unmounted,
    its bind still standing.
    The clone is mounted nowhere;
    the folder holds the empty bind;
    the intent stands.
    A kill here leaves a bind a later sweep unbinds and then files a clear receipt for.
7.  The unmount and unbind done,
    the clear receipt filed,
    the wrapper not yet its reader.
    The clone is mounted nowhere;
    the folder is empty;
    the receipt names the clone absent in every namespace and the bind gone;
    the intent stands;
    no plan.
    A kill here leaves a state a later removal's staging sweep settles on the receipt.
8.  The wrapper read the receipt,
    removed the folder,
    unlinked the record and released the staging claim.
    The clone is mounted nowhere;
    no staged state;
    the intent stands;
    the removal claim is held;
    no plan.
    A kill here leaves an `unmount` intent with no folder,
    which the handler refuses rather than acting on,
    because the folder it must bind is the wrapper's (staging design,
    "The staging target and who creates it").
9.  The plan persisted,
    with every verified tuple and no outcome.
    The clone is mounted nowhere and its snapshots are all present;
    the intent stands;
    the removal claim is held.
    A kill here leaves a plan a later removal resumes from and a cancel may reverse,
    since nothing was destroyed.
10. The check before the first snapshot delete refused.
    Nothing was deleted,
    so the state is the one point 9 describes with a refusal recorded;
    what that refusal keeps is the deferred-destroy item's,
    another writer's open question.
11. The snapshot batch submitted,
    its answer not yet recorded.
    The clone is mounted nowhere and its head is present;
    the plan holds the submitted tuples with no outcome;
    the intent stands.
    A kill here leaves an outcome that only per-name presence settles (F7,
    F8).
12. The batch answered and each per-name outcome recorded.
    The clone is mounted nowhere,
    its head is present,
    and the destroyed and kept sets are both known;
    the intent stands.
13. The check before the head destroy refused.
    The head is present and the snapshots the batch destroyed are gone;
    the plan is kept with the destroyed set,
    and whether this refusal cancels is the deferred-destroy item's.
14. The head destroy submitted,
    its answer not recorded.
    The head may be gone and the clone with it;
    the plan's records are the only ones left.
    A cancellation must not attach here,
    and a settlement proves the head gone by its name's absence and by no child of the pinned parent carrying its
    recorded GUID (the proposal's sentence quoted under "The removal and its order").
15. The head destroy proven done.
    The clone is gone,
    and the intent went with it,
    because it was a property of that dataset.
    The registration and its lock stand until the settle step removes them (F1,
    F2);
    the underlay pointer stands;
    the plan's tuples are settled.
    Nothing here can be reversed,
    and the cancel command refuses at this point.
16. The registration settled and the underlay removed.
    Nothing of the removal is left but the released claim and the plan's terminal record.
    A cancel command finds no removal to reverse.

From points 1 to 14 the Git registration and its lock stand unchanged,
because settling them belongs to the step after the head's destroy,
and the lock is what keeps `git worktree prune` away from a registration whose clone may stand unmounted for a
while (F3).
The slot identity store's record and the underlay pointer stand unchanged for the same reason,
and the intent and the plan are the only records that change inside a removal.

Three of these points are where a removal stops by cancelling itself,
and they are the two holder-blocked steps:
the unmount of point 6,
where the handler's busy answer reattaches,
and the head destroy,
which refuses while the state is the one points 12 and 13 describe,
so that a cancelled removal can meet its own snapshots already destroyed.
Every other point refuses and keeps its records rather than cancelling,
which is the proposal's rule for a proof,
a dependency or a hold that fails ([proposal][proposal],
"Removal and recovery",
step 2).

## What a removal that cancels itself restores

Each arm ends with the clone standing at its slot and the removal ended.
They differ in what was destroyed before the block and therefore in whether the clone's mount is moved back or made
again.

### The arm blocked at the unmount

The holder keeps the staged clone busy and the handler's unmount refuses `EBUSY` (F11).
The handler's answer is the busy-unmount answer the project accepted:
it revalidates and moves the same mount back to the slot,
unbinds its bind where its own topmost check still finds the recorded bind,
and files the receipt the staging design calls the cancelled removal's ([staging-design][staging-design],
A5).
Nothing was destroyed,
so the loss is empty and the clone keeps every snapshot it had.

The wrapper then,
still holding the removal claim:

1.  writes the terminal plan record,
    with an empty destroyed set,
    the nonce of the request it is about to write,
    and the time.
    This is the report the draft said to retain before reversing.
2.  writes `v1:<fresh nonce>:mount` on the clone,
    which replaces the standing `unmount` request with a request for the state the clone is in.
3.  waits for the answer under that nonce,
    for the removal deadline.
    A clone already at its slot with its recorded identity is answered `done` without a second attach (A5).
4.  settles the staged state as the cancelled-removal receipt allows:
    where the receipt says the clone is at its slot and the bind is gone,
    the folder is removed by `rmdir`,
    the record is unlinked and the staging claim released;
    where the bind survives,
    the folder,
    record and claim are retained and reported.
5.  releases the removal claim and prints the report.

The handler needs no new mode for any of this.
It answers a `mount` request,
which is one of the two requests its grammar already carries.

### The arm blocked at the head destroy

The clone is mounted nowhere,
its own snapshots are gone and the head destroy refuses `EBUSY` (F6).
Two causes reach that errno and the design tells them apart by the live snapshot list:

- A snapshot appeared after the final enumeration.
  The live list is not empty.
  This is not a holder and not a cancellation.
  The removal's own amendment rule applies,
  which the proposal states as a rerun of every check with the amended plan
  recorded and no prompt ([proposal][proposal],
  "Removal and recovery",
  step 3).
- Holders keep the dataset busy.
  The live list is empty.
  Without `--force` the removal cancels itself here,
  which is the loss the open item names.

For the second cause the wrapper does what the first arm does,
with two differences.
The terminal plan carries the destroyed set,
taken from the plan's own settled outcomes,
before anything is written.
And the attach is a fresh one,
because there is no mount to move back:
the handler creates a mount of the clone at the slot under the same checks it uses for an add's attach.
The clone comes back at its slot,
with its files,
and without the snapshots the batch destroyed.
Those snapshots do not come back,
and the report names each one.

### What the removal record does, part by part

The open item asks whether the removal record is reversed or kept.
The design's answer is that the removal record is three records with three different fates:

- The mount intent is reversed.
  The clone's desired state becomes `mount` under a fresh nonce,
  so no later sweep acts on a removal that ended.
  Nothing is left asking for an unmount.
- The removal claim is released.
  It marked a running removal and there is none.
- The destruction plan is kept,
  with a terminal outcome.
  It holds what was destroyed,
  which cannot be restored,
  and it is the only durable record of that loss.
  A later removal of the same clone writes its own plan over it,
  which is the natural end of the record's life.
- The staging claim,
  folder and record are settled as the cancelled-removal receipt allows,
  by the staging design's own rule.
- The Git registration,
  its lock,
  the slot identity store's record and the underlay pointer are untouched.
  The clone still exists and is still registered,
  so nothing of the registration was due to be settled at any point
  before the head's destroy (points 1 to 14 under "The stop points").
  The lock matters here for a reason outside this design's own record:
  it is what keeps `git worktree prune` from deleting the registration of a clone that stands unmounted for a while
  (F3),
  while `git worktree remove` refuses a locked worktree without a double force (F2).
  Which lock is the wrapper's own and which is an operator's is the add-time-lock item's question.

### What the caller sees

The removal that cancels itself ends with exit `1`,
because the worktree was not removed,
and one report on standard error naming the slot,
the dataset and its GUID,
the operation that was blocked,
the reason,
any holder the wrapper or a receipt named,
what was destroyed,
that it cannot be restored,
the mount state,
and the two ways forward.
Two examples,
with the fixture's names:

```text
removal of `.claude/worktrees/one` did not finish: the clone's unmount refused `EBUSY`, because one mount of
`zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one` in namespace 4026532411 is held by process
41233. Nothing was destroyed and every snapshot survives. The clone is mounted at its slot again.
The removal cancelled itself, as it does without `--force`. Ways forward: leave the folder and run the command
again, run it with `--force` to end the holders other than your session, or run
`git monochromatic-git-policy-cli cancel-worktree-removal one` to cancel the removal by hand.
```

```text
removal of `.claude/worktrees/one` cancelled itself after destroying snapshots that cannot be restored:
`zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one@sanoid-hourly-2026-11-22_03`,
`zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one@sanoid-hourly-2026-11-22_04`.
The head destroy refused `EBUSY` with the clone's own snapshot list empty, which means holders keep the dataset
busy. The clone is mounted at its slot again without those two snapshots. Its files are unchanged.
The removal cancelled itself, as it does without `--force`. Ways forward: leave the folder and run the command
again, or run it with `--force` to end the holders other than your session.
`git monochromatic-git-policy-cli doctor` reports the same loss and the kept record.
```

### What `doctor` prints

The proposal asks `doctor` for the standing intent,
the last verified loss,
the remaining objects,
whether the removal will cancel itself,
and the name of the kept command ([proposal][proposal],
"What doctor checks and prints",
and the doctor paragraph quoted there).
This design fixes the fields and the wording those sentences need,
and plan step 18 builds them:

- the slot,
  its registration and the dataset with its GUID;
- the intent's operation and nonce,
  read through the same reader the wrapper uses,
  with an absent or inherited value said to be no request;
- the mount state at the slot,
  read from the mount table,
  and any staged folder with its dataset and its age;
- the removal claim's owner,
  live or dead,
  with the process identity the claim records;
- the plan's outcome:
  none,
  in progress,
  refused at a named step,
  cancelled with a destroyed set,
  or finished;
- for a cancelled removal,
  every destroyed name and the time of the cancellation;
- the next safe step,
  which is one of the two commands the cancellation report names,
  or the administrator's where a record cannot be proven.

`doctor` changes nothing,
so it neither reverses a removal nor settles a folder.
It reads what the removal and the cancel command wrote.
The lines for a cancelled removal read,
with the fixture's names:

```text
worktree clone slot `one`: dataset zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one,
GUID 12345678901234567890, registration .git/worktrees/one, lock held by the wrapper.
intent: `v1:9c41...:mount`, a local request for the clone's mount. mount: at its slot.
removal: cancelled 2026-11-22T16:41:07-05:00, and one snapshot was destroyed and cannot be restored:
`...@sanoid-hourly-2026-11-22_03`. The removal claim is released. No staged folder stands.
next safe step: none for the loss, which is recorded; run the removal again when the clone is to go.
```

## What a timed-out or killed removal leaves

### Whether the standing request can act

A removal that is killed or that runs past its deadline leaves the intent standing,
and whether the handler can act on it depends on one thing:
whether the wrapper's staging folder exists.

- With no folder (points 3 and 8),
  the handler cannot bind,
  so it refuses rather than unmounting,
  or,
  where the clone is already mounted nowhere,
  it answers that the clone is absent and nothing moves.
  The intent is then inert.
- With a folder (points 4 to 7),
  the handler's next trigger acts on the intent and finishes the unmount and the unbind,
  or refuses again with the holder named.

So the state is either inert or converging,
never half acting,
and that is a property of the wrapper's owning the folder rather than of any new rule.

A cancellation that is itself killed leaves one more state:
the intent says `mount`,
the clone stands at its slot,
and the folder,
its record and the staging claim still stand where the cancelled-removal receipt says they may be removed.
The next removal of that repository and the next cancel command both settle that state on the same receipt,
and `doctor` reports the folder with its age until one of them does.
Nothing acts on the clone's mount in that state,
because the intent asks for the state the clone is already in.

### Who reconciles, and on which event

The mount side reconciles itself,
on the triggers the project already controls (A6):

- the boot unit's start reconcile;
- the checkout provider's post-key-load hook;
- a later sweep nudge on that clone;
- ZED's delivery of an event to the installed handler;
- and a later removal of the same repository,
  whose staging sweep runs for its own claims and settles a folder keyed to a dead owner.

The wrapper side reconciles at the next wrapper operation for that repository:

- a later `git worktree remove` of the same slot takes the removal claim (the dead owner's claim is replaced),
  reads the plan and resumes at the point the plan records;
- the cancel command takes the claim and reverses;
- `doctor` reads and reports only.

No timer,
no daemon and no new trigger is added,
and no forwarded command in any repository pays for a scan.

### What the caller is told at the deadline

The wait's own timeout text already says the durable request stays on the clone and a later sweep may still act (F17).
The removal adds three sentences to it:

- whether a plan was written,
  and therefore whether anything was destroyed;
- that a deadline is not a cancellation,
  so the worktree is not back and the removal is not abandoned;
- the two commands that resolve it,
  which are the same two the cancellation report names,
  and the fact that `doctor` shows the state until then.

The removal exits `1`.
The report reads,
with the fixture's names and its own numbers:

```text
removal of `.claude/worktrees/one` did not finish within 30000 milliseconds. The last observation was: no result for
this request was filed. Nothing had been destroyed, because no destruction plan was written. The request is operation
`unmount` under nonce 3f2a... for dataset GUID 12345678901234567890, and it stays on the clone.
A deadline is not cancellation, so the clone stays unmounted and the removal is neither cancelled nor finished.
Ways forward: run `git worktree remove .claude/worktrees/one` again to resume it, run
`git monochromatic-git-policy-cli cancel-worktree-removal one` to reverse it, and run
`git monochromatic-git-policy-cli doctor` to see the state until one of them runs.
```

### How the worktree returns to use

There are four ways,
and each names the state it needs:

- Run the removal again.
  It takes the claim,
  reads the plan,
  and resumes:
  it finishes the unmount where the folder and the intent still stand,
  settles where the plan says the batch was submitted,
  and destroys the head where nothing blocks it.
- Run `cancel-worktree-removal <slot>`.
  It reverses where the head is present and every destroy outcome is settled,
  which covers every point up to the head's destroy.
- Wait for a trigger,
  then run one of the two commands above.
  A completed unmount changes the cancel command's work from moving a mount back to making one,
  and changes nothing else.
- Nothing returns the worktree where the head is gone.
  The removal must be run again to settle the registration and the origin cleanup,
  and the cancel command refuses and says so.

## The cancel command

### The grammar

The command is a management subcommand ([spec][spec],
"Management grammar"):

```text
git [<git-global-options>] monochromatic-git-policy-cli cancel-worktree-removal <slot>
```

It takes exactly one positional slot name and no option.
`--help` prints the subcommand's usage on standard output and exits `0`.
Any other token,
a second positional,
or an empty slot is a usage error with exit `2`,
which is the rule `doctor` already follows for its own arguments.
It runs inside the repository whose slot it names,
and the slot must be a direct child of that repository's pinned parent under `.claude/worktrees/` (rule `WTU`).

### The proof: which slot, which clone, which claim

The command acts only on what it can prove,
and it proves in this order.
Each step names its refusal.

1.  The slot is a plain slot name,
    by the same grammar the attach path uses (`slot_name.rs:24`).
    A name that is not one is refused with that grammar's reason,
    exit `1`.
2.  The slot's `.git` pointer is read with the bounded no-follow reader
    (`clone_slot_git_pointer.rs:231`) and must name `<common>/worktrees/<id>` of the caller's repository.
    A slot with no pointer,
    a pointer to another checkout,
    and a pointer whose bytes are not a single `gitdir:` line are each refused by name.
3.  The binding record (A1) must name this slot and the clone's dataset,
    GUID and origin.
    A missing record,
    a record naming another slot,
    and one whose clone has a different GUID from the plan's are each refused,
    and the refusal says the administrator's adoption is the step that repairs a damaged binding.
4.  At least one removal record must exist:
    a plan,
    a staged-folder record,
    or a local `unmount` intent.
    With none,
    the command refuses with exit `1` and says the slot is not being removed,
    because a person who typed another slot's name must not be told a removal was reversed.
5.  The removal claim for the clone's GUID is taken (F16).
    A live owner is refused,
    naming its process identity;
    a dead owner's claim is replaced by the next generation,
    which is the claim's ordinary takeover and needs no cleanup.
6.  Every read that decides is repeated under the claim,
    because the claim is what makes the reads one moment:
    the pointer,
    the binding record,
    the plan,
    and the staged-folder record where one stands.
    All of them must name one clone,
    one GUID and one slot,
    and the intent must read as a local request.
    A disagreement between any two records is a refusal,
    and never a choice between them.
7.  The clone must be live and its identity must match:
    the dataset name from the records exists,
    answers with the recorded GUID,
    and names the recorded origin.
    A missing dataset is the head-gone case under "Settling the destroy outcomes before anything moves",
    not this one.

### Settling the destroy outcomes before anything moves

The command settles every outstanding destroy request before it writes any intent,
and it settles from the live clone,
never from a name's absence alone.

- With no plan,
  nothing was submitted and nothing is settled.
- With a plan and the head present,
  the clone's live snapshot inventory,
  by name,
  GUID and `createtxg`,
  settles every recorded tuple:
  a tuple whose GUID is in the inventory was not destroyed,
  and a tuple whose GUID is absent was.
  A name reused for another GUID is a refusal,
  not a settlement,
  and F10 is why the GUID is the field compared.
- With a plan and the head absent,
  the head is proven gone only where its name is absent and no child of the pinned parent carries its recorded GUID.
  Where that proof passes,
  the command refuses and reports the removal's remaining settle step;
  where a child carries the GUID,
  the outcome is unknown and the command refuses for that reason.
- An inventory read that is incomplete or refused leaves every tuple unknown,
  and the command refuses.

The confirmed destroyed set is what the report names and what the terminal plan records.
Nothing in this part writes to the pool.

### The act: one mount intent and one answer

With the proof done and the outcomes settled,
the command first reads the intent it found:
where the intent already reads as a `mount` request,
the terminal plan already records a cancellation,
and the clone stands at its slot,
the command writes no second request,
reports that state and exits `0`,
which is the already-cancelled case the exit codes name.
In every other state it:

1.  writes the terminal plan,
    with the destroyed set and the nonce it is about to write;
2.  writes `v1:<nonce>:mount` on the clone through the existing intent writer,
    whose gate runs first;
3.  waits for a receipt that names that operation,
    that nonce and this boot,
    for the removal deadline;
    a receipt from another nonce,
    another operation or another boot is not the answer,
    which is the wait's own rule;
4.  on `done`,
    reads the receipt's slot,
    source and mount identity and requires them to be the clone's at its slot;
    where the receipt says the bind is gone,
    it removes the folder by `rmdir`,
    unlinks the record and releases the staging claim;
    where the bind stands,
    it retains and reports all three;
5.  on `refused` or `failed`,
    reports the receipt's failed step and errno,
    keeps every record,
    and exits `1`;
6.  on a timeout,
    reports what the wait last saw,
    keeps the terminal plan,
    and exits `1`,
    because the request stands and a later trigger answers it;
7.  releases the removal claim on every path out,
    including the paths that refuse.

The command never forces an unmount,
never detaches a mount,
never ends a holder,
never issues `zfs mount`,
never mounts by path,
never removes a registration and never destroys a snapshot.
It asks for one attach and reports what the handler answered.

### The report

The command's report goes to standard error and standard output stays empty,
which is what the repository's stream rule asks of a run that emits no data ([exit codes][exit-codes]).
It names the slot,
the dataset and GUID,
the destroyed set with its names,
that the set cannot be restored,
the mount state after the answer,
any retained staged state,
and the next step.
A successful report,
with the fixture's names:

```text
cancelled the removal of `.claude/worktrees/one` (`zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one`,
GUID 12345678901234567890). No snapshot was destroyed. The clone is mounted at its slot and the removal's staging
folder is gone. The kept cancel record is at the plan's place and `doctor` reads it.
```

### What it refuses

The refusals are the design's guard,
and each names the state it found and the step that repairs it:

- a name that is not a slot name,
  with exit `1`;
- a slot with no pointer,
  or a pointer to another checkout or slot;
- a missing or mismatched binding record;
- no removal record at all;
- a live removal claim,
  naming its owner;
- two records that disagree about the clone,
  the GUID or the slot;
- an intent that reads as malformed or refused;
- a clone whose live GUID or origin is not the recorded one;
- an inventory that cannot be read completely;
- an unknown destroy outcome;
- the head gone,
  which names the removal's settle step instead of an attach;
- a staged state the receipt says still holds a bind,
  or a staged folder the command cannot prove it may remove;
- a handler answer that refuses or fails,
  with the receipt's step and errno;
- a mount request whose receipt never arrives,
  at the deadline.

It refuses none of these with a prompt and it never asks for a credential,
because it needs no authority it does not have:
the account owns the clone,
the folder and the claims,
and the handler's work runs as root under its own per-slot claim.
Two of these refusals need their own words,
because their state is the one the open item asks about.
An unknown destroy outcome says what is unknown and names no attach:

```text
cannot cancel the removal of `.claude/worktrees/one`: the destruction plan records a snapshot whose outcome cannot be
settled. `...@sanoid-hourly-2026-11-22_03` is not in the clone's live snapshot list and no child of
`zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets` carries GUID 12345678901234567890, so the head's
state is unknown. Nothing was mounted and nothing was changed. Ways forward: run
`git worktree remove .claude/worktrees/one` again, which settles the plan before anything else, and read
`git monochromatic-git-policy-cli doctor`; where that removal also refuses, an administrator recovers the lineage.
```

A head that is gone says so and names the removal's own settle step:

```text
cannot cancel the removal of `.claude/worktrees/one`: its clone is gone. The dataset
`zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one` is absent and no child of the pinned parent
carries GUID 12345678901234567890, so the head was destroyed. A cancelled removal cannot bring it back and nothing is
mounted. Ways forward: run `git worktree remove .claude/worktrees/one` again, which settles the registration, the
underlay and the origin cleanup; `git monochromatic-git-policy-cli doctor` lists what is still owed.
```

### Exit codes and streams

`0` when the removal was reversed,
or when it had already been cancelled and the clone is at its slot.
`1` for every refusal in the list above and for a timeout.
`2` for a usage error.
Standard output carries nothing,
reports and refusals go to standard error,
and the log lines go through the wrapper's tagged loggers.

## Cancellation by itself and by the command

What the two share is the whole mechanism:

- one `mount` intent with a fresh nonce,
  written through the same writer and gate;
- one wait for one receipt under that nonce,
  on the same matching rules and the same deadline;
- one handler path,
  which is the ordinary attach answer plus the staged move-back where a staged mount stands;
- one terminal plan record,
  written before the intent and holding the destroyed set;
- one settlement of the staged folder,
  record and claim,
  on the cancelled-removal receipt;
- one report shape,
  with the same fields.

What differs is who is present:

- The removal holds the claim already,
  knows the clone from its own proof,
  and knows the destroyed set from the plan it wrote.
  Its outcome is exit `1`,
  because the worktree was not removed.
- The command takes the claim,
  proves the clone from the slot,
  settles the destroyed set from the live clone,
  and its outcome is exit `0` where it reversed.

When both act on one slot,
the removal claim decides:

- While a removal's claim has a live owner,
  the command refuses and names that owner.
  A person who wants to stop a running removal stops it,
  and then the claim's owner is dead.
- The automatic arm cannot run without its own process,
  so a killed removal never cancels itself.
  Its claim has a dead owner and the command takes it.
- Two command runs do not interleave,
  and the second finds the intent already `mount`,
  the terminal plan present and the clone at its slot,
  and it reports that and exits `0`.
- A `mount` intent written while a sweep is holding the root claim is not lost,
  because the sweep re-reads the intent after it releases the claim ([step-8][step-8],
  "A slot whose claim is held").
  That sweep's own answer to the earlier `unmount` request may still be a completed unmount,
  and then the `mount` request it reads next makes the clone again,
  so the state converges on the clone being at its slot with a `mount` request standing.

## The records this design adds

### The removal claim

The wrapper takes one claim per clone for the whole removal,
of the repository's own generation claim kind (F16):

- its place is `<common>/monochromatic-git-policy-cli-worktree-copy/removal/`,
  a private folder beside `v1` under the same private parent as the staging claim's folder,
  which no journal pass lists;
- its prefix is `<guid>.removal`,
  which is the clone's canonical dataset GUID,
  so a person and `doctor` can match the claim to a clone without reading its record,
  and the handler never reads it at all;
- its generations are the shared claim's,
  and its record is the shared record:
  the owner's process id,
  birth identity and PID namespace,
  then a released line.

It records no nonce and no phase,
because the shared claim's record has no field for one and because the plan carries both.
The alternative was to widen the staging claim's lifetime,
which the accepted staging design fixes as the folder's,
and that alternative is marked for the project's veto.
The claim is not the root handler's per-slot claim,
which only root can take (F15).

The claim's existence is also the only mark that a wrapper operation is working on that clone.
`doctor` reads it and prints its owner,
live or dead.
Nothing sweeps it:
a dead owner is replaced by the next claimant,
which is what the shared claim does,
and a released claim keeps its newest generation file as the shared claim requires.

### The terminal state of the destruction plan

A cancellation always writes one terminal plan record for the clone,
even where nothing was destroyed:

- the clone's dataset name and GUID,
  and the slot;
- the outcome word naming a cancelled removal;
- the destroyed tuples with their names and GUIDs,
  which may be none;
- the nonce of the `mount` request the cancellation wrote;
- the time,
  to the second with its offset.

It is written before the intent,
because the loss must be reported and retained before the state is reversed.
A later removal of the same clone writes its own plan over it.
How long the terminal record lives is one of the questions only the project can answer.

### The receipt's new answer field

A cancelled removal's receipt must say whether the bind is gone from the host namespace,
because the wrapper's namespace may differ from the handler's (L7),
and the receipt's own shape has no key for it today (F13).
This design asks step 9's reattach and step 17 to add one key to the receipt,
as its last key after `time`:

- `bind`,
  whose value is one of `none`,
  `gone` or `standing`;
- `none` on an answer to a request that had no staged state,
  which is every add's attach and every fresh attach;
- `gone` where the handler moved a staged mount back and its bind is no longer in the host namespace;
- `standing` where the bind survives.

The receipt's keys become sixteen,
and the file bound derived from them grows by that key's line,
which is the kind of change the receipt's own derivation exists to absorb (F13).
Nothing of the ZFS route is released,
so no written receipt needs migrating.
The alternatives are marked for the project's veto.

## Choices this design marks for the project's veto

Each entry is a choice the project's records do not decide,
made here so that the design is whole.
The project can veto any of them,
and each veto names the option that would replace it.

- The cancellation states a request instead of clearing the intent.
  (a) This design writes `v1:<fresh nonce>:mount`.
  After a busy unmount the property reads `v1:3f2a...:mount`,
  and the next reader of the clone sees a request for the state it is in.
  (b) The command inherits the property,
  as `zfs inherit` allows for a user property (F4,
  F5).
  The property then reads `-` with source `default`,
  and nothing asks for anything.
  (c) A third operation word,
  `cancel`.
  The property would read `v1:3f2a...:cancel`,
  which contradicts the accepted grammar's "exactly one of two values" and the intent's byte bound (F12).
  Pros of (a):
  every later reader,
  including `doctor` and a trigger,
  sees a request that explains the clone's state,
  and the intent is never left saying `unmount` about a clone nobody is removing.
  Cons of (a):
  one extra property write and one extra wait per cancellation.
  Pros of (b):
  one write fewer and no request that could be answered with a second attach.
  Cons of (b):
  a killed cancellation leaves no record of why the clone stands where it does,
  and a later sweep cannot tell a cancelled removal from an unfinished add that never mounted its clone.
  Pros of (c):
  one word says exactly what happened.
  Cons of (c):
  it changes an accepted grammar and its bound for a state (a) already carries.
  Ranking (a) > (b) > (c):
  (a) over (b) because the durable intent is the route's only authority to act and an absence explains nothing,
  while (b) buys one write with exactly that explanation;
  (b) over (c) because (b) needs no accepted sentence changed and (c) does,
  for no state that (b) lacks.
- The one-remount exception is read to cover a fresh attach of an unmounted clone a removal left behind.
  (a) This design's reading.
  The head-destroy cancellation creates a new mount of the clone at its slot,
  because the clone has no mount to move back.
  (b) The exception covers only the move-back of a staged mount.
  The same cancellation leaves the clone unmounted,
  and the report names the attach as the person's next step.
  Pros of (a):
  the worktree is returned to use by the mechanism the answer's own words describe,
  and the default's recorded reason is answered,
  because the design replaces the standing `unmount` intent before it asks for the attach,
  which is exactly the race that reason names.
  Cons of (a):
  the design attaches a clone in a state where history was already destroyed,
  so the report must be loud,
  and a person who never reads it works in a worktree with fewer snapshots.
  Pros of (b):
  no attach happens after a partial destruction,
  so nothing can look whole while history is missing.
  Cons of (b):
  the only state a head-destroy cancellation can reach is the one the cancellation cannot restore,
  which leaves the worktree unavailable after a cancellation the project asked to happen with nobody present.
  Ranking (a) > (b):
  (a) over (b) because the accepted answer exists to end a blocked removal with the worktree usable and reports the
  loss,
  while (b) ends it with the worktree unusable and the same loss.
- The handler answers a request for an already attached clone with `done`.
  (a) This design requires the attach path to check the mount table before its walk and answer `done` where the
  clone stands at its slot with its recorded identity (A5).
  A cancel of an already cancelled removal reads `done`,
  and no second mount is made.
  (b) The wrapper treats the attach's boundary refusal as success after reading its own mount table (F14).
  The same run sees a receipt that refuses with the slot's existing mount as its failed step,
  and the wrapper reports the clone attached.
  (c) A new outcome word,
  `attached`.
  The receipt would carry a fourth outcome,
  and the outcome bound would grow (F13).
  Pros of (a):
  the decision is taken in the namespace that holds the mounts,
  which is the handler's,
  and one answer covers an add's retry,
  a cancel and a race between two triggers.
  Cons of (a):
  the attach path gains a mount-table read and a case its walk never had.
  Pros of (b):
  no handler change.
  Cons of (b):
  the wrapper decides from its own view of the mount table,
  which the staging design records as able to differ from the handler's (L7),
  and a refusal's cause is read as an answer rather than as one.
  Pros of (c):
  the distinction is named in the record.
  Cons of (c):
  the mount identity the receipt already carries says whether the clone is at its slot,
  so the word spends a bound for a fact the design already reads.
  Ranking (a) > (b) > (c):
  (a) over (b) because the namespace that holds the mount is the only one that can answer about it safely;
  (b) over (c) because (b) at least changes no record,
  while (c) changes a record for a fact already present.
- The receipt gains one answer field.
  (a) This design adds the `bind` key with `none`,
  `gone` or `standing`.
  The reattach's receipt reads `bind=gone`,
  and the wrapper removes the folder on it.
  (b) A separate record per staged folder carries the bind's state.
  The wrapper would read a second file beside the record it already reads.
  (c) The wrapper infers the bind's state from its own mount table.
  The same reattach needs no new field,
  and the wrapper decides from the table it reads for its own cleanup.
  Pros of (a):
  the state and the answer come from the one invocation that moved the mount,
  matched by nonce,
  so they cannot be from different moments.
  Cons of (a):
  one key and a grown file bound,
  and a reader that must refuse an unknown value.
  Pros of (b):
  the receipt's shape stays as step 7 wrote it.
  Cons of (b):
  a second record with its own atomic write,
  lifetime and settlement for one word.
  Pros of (c):
  no record change at all.
  Cons of (c):
  it fails exactly where the wrapper's namespace is not the handler's (L7),
  which is the case the staging design wrote A5 for.
  Ranking (a) > (c) > (b):
  (a) over (c) because the handler's own observation is the only one made where the mounts are;
  (c) over (b) because (c) adds no record to settle,
  while (b) adds a whole one.
- The wrapper takes one removal claim per clone for the whole removal.
  (a) This design's new claim of the shared generation kind,
  beside the staging claim.
  A second `git worktree remove` of the same slot refuses while the first runs,
  and the cancel command refuses too.
  (b) The staging claim's lifetime widens to the whole removal.
  The same refusal happens,
  on a claim the staging design says lives as long as its folder.
  (c) No wrapper-side claim,
  and the root per-slot claim is the only serialization.
  The same removal runs without a wrapper-side claim,
  and the cancel command and a removal can interleave after the unmount.
  Pros of (a):
  the two wrapper operations that must not interleave are serialized,
  with the mechanism the repository already uses,
  and nothing accepted is amended.
  Cons of (a):
  one more claim folder,
  one more name in it,
  and one more thing `doctor` reports.
  Pros of (b):
  no new folder.
  Cons of (b):
  it amends an accepted settlement rule,
  and it still leaves the removal unserialized after the folder is removed,
  which is where the destroys happen.
  Pros of (c):
  nothing new.
  Cons of (c):
  it cannot serialize the dangerous window,
  because the wrapper cannot take a root claim (F15) and no root invocation runs while snapshots are destroyed.
  Ranking (a) > (b) > (c):
  (a) over (b) because (a) leaves the accepted staging settlement alone and covers the steps after the folder;
  (b) over (c) because (b) at least covers the staging step,
  while (c) covers nothing after the unmount.
- The cancel command runs the staging sweep for that clone's claim.
  (a) This design runs it,
  for the one clone the command proved.
  A cancelled removal whose folder still holds an empty bind is settled by the command that cancelled it.
  (b) The command refuses while a staged state stands and names the removal to run.
  The person runs `git worktree remove` again to sweep,
  then the cancel command.
  Pros of (a):
  one command settles what a cancellation produced,
  on the receipt the staging design already defines for it.
  Cons of (a):
  the staging design says its sweep runs from a removal,
  and a cancel command is a removal-side operation only by this design's reading.
  Pros of (b):
  the sweep stays a removal's alone.
  Cons of (b):
  the person pays two commands for a state one command made,
  and the removal they are told to run is the one they just cancelled.
  Ranking (a) > (b):
  (a) over (b) because the cancel command is the manual way back for a removal and the folder is that removal's own
  record,
  while (b) sends the person into the command they asked to undo.
- A slot with no removal record,
  and a removal already cancelled,
  are told apart.
  (a) This design refuses the first with exit `1` and reports the second with exit `0`.
  `cancel-worktree-removal typo` refuses,
  and `cancel-worktree-removal one` on an already cancelled removal prints its state and exits `0`.
  (b) Both refuse with exit `1`.
  The second run fails with a report saying nothing is being removed.
  (c) Both report and exit `0`.
  The typo's run reports success for a slot that was never removed.
  Pros of (a):
  a caller's retry is safe and a person's mistake is not reported as success.
  Cons of (a):
  two outcomes for two states a person may not distinguish without reading the report.
  Pros of (b):
  one rule.
  Cons of (b):
  a run that achieved the state the person asked for fails.
  Pros of (c):
  one rule and retries always succeed.
  Cons of (c):
  a mistyped slot is reported as reversed.
  Ranking (a) > (c) > (b):
  (a) over (c) because the states differ in whether anything was ever being removed;
  (c) over (b) because a retry that finds the state it wants is not a failure.
- The cancel command waits once and reports.
  (a) This design waits for one matching receipt,
  for the removal deadline,
  then reports.
  A silent handler ends the run with the last observation and exit `1`,
  and the intent stands.
  (b) It waits until the person interrupts it.
  The same silent handler holds the terminal until the person gives up.
  (c) It writes the request and reports at once.
  The same run exits `0` before anything is mounted.
  Pros of (a):
  the command's time is bounded and its report says what it saw,
  which is the shape of every other wait on this route (F17).
  Cons of (a):
  a slow handler is reported as a timeout even where it would have answered.
  Pros of (b):
  a slow handler is not reported as a failure.
  Cons of (b):
  a hung handler holds a person's terminal,
  and the wait has no report to give.
  Pros of (c):
  it never waits.
  Cons of (c):
  it reports a state it has not confirmed and exits before the work it asked for happens.
  Ranking (a) > (b) > (c):
  (a) over (b) because a bounded wait with a report is what the route's other waits do and what a person can act on;
  (b) over (c) because a wait that may end is better than a report that is never true.

## Questions only the project can answer

These are values,
authority and cost,
and no measurement settles them.

- How long the record of a cancelled removal's loss lives.
  (a) Until the clone's next removal writes its own plan,
  or the clone goes away.
  `doctor` prints the two destroyed snapshot names on every run until then,
  and a later `git worktree remove` of the same slot replaces the record.
  (b) Until the person acknowledges it,
  with a recorded acknowledgement.
  The same names show on every run until `doctor --acknowledge` or a like action,
  which the design does not have.
  (c) For a fixed period after the cancellation.
  The names show for that period and then stop,
  and the design would have to ship a number nobody measured.
  Pros of (a):
  no new mechanism and the loss survives as long as the clone it belongs to.
  Cons of (a):
  the record is read only by `doctor`,
  so a person who never runs it never sees the loss.
  Pros of (b):
  the loss is loud and its clearance is a person's decision.
  Cons of (b):
  a new acknowledgement action,
  a place for it,
  and a failure mode where nobody acknowledges.
  Pros of (c):
  the records do not accumulate.
  Cons of (c):
  a shipped retention bound with no measurement behind it,
  and a loss that silently stops being reportable.
  Ranking (a) > (b) > (c):
  (a) over (b) because (a) needs no mechanism the design lacks and `doctor` is the designed place for durable state;
  (b) over (c) because (b) keeps the report until somebody decides,
  while (c) decides on a number nobody measured.
- Whether a removal started with `--force` may be cancelled by the command.
  (a) Yes,
  on the same proof as any removal.
  A forced removal whose wrapper was killed is reversed by the same command,
  and no record says the flag mattered.
  (b) No,
  and a forced removal's record makes the command refuse.
  The same person is told to run the removal again,
  which continues the destruction they started.
  Pros of (a):
  a killed forced removal is recoverable without finishing the destruction,
  and the trust model treats the account as the person whose worktree it is.
  Cons of (a):
  a later run can reverse a decision that ended the holders of that clone.
  Pros of (b):
  the flag's decision is respected.
  Cons of (b):
  a killed forced removal can only be finished or left,
  so the command's purpose is lost exactly where the removal was most destructive.
  Ranking (a) > (b):
  (a) over (b) because the flag changes how a removal ends holders,
  not whether the person may stop a removal,
  and the project's threat model makes the account one person for this purpose.
- Whether a cancellation's loss also becomes a standing notice that later wrapper commands in that worktree show.
  (a) No:
  the report at the cancellation,
  the kept plan record,
  and `doctor`.
  A person who never runs `doctor` in that repository learns of the loss once,
  at the cancellation.
  (b) Yes:
  a standing notice the wrapper shows on later commands in that worktree until something clears it,
  of the kind the add already writes.
  The same person runs a commit in the worktree a week later and reads the two destroyed names.
  Pros of (a):
  no new notice,
  and the durable record is the one the design already keeps.
  Cons of (a):
  the loss can go unseen by the person it affects.
  Pros of (b):
  the loss reaches the person where they work.
  Cons of (b):
  a notice with its own lifetime and clearing rule,
  which no record here designs.
  Ranking (a) > (b):
  (a) over (b) because the design has a durable record and a place for it,
  while a standing notice needs a clearing rule that would be invented here;
  a veto that wants the notice is a new item rather than a change to this one.

## Which plan step builds which part

- Step 9 builds the busy unmount's reattach and the cancelled-removal receipt,
  and this design relies on both.
  It also builds the receipt's `bind` key with step 17,
  because the reattach is its first writer.
- Step 14 builds the removal claim and takes it before the plan,
  the folder and the intent,
  so that points 1 to 4 of "The stop points" are reachable in the order this design states.
  It also writes the plan's first state and accepts an unmounted clone with a standing intent as a resumable state.
- Step 15 decides whether its refusals keep the plan and whether such a removal cancels;
  this design's cancel command treats a kept refusal as a state with nothing destroyed and may reverse it.
- Step 16 builds the head destroy's two causes,
  the plan's per-name settlement and the plan's terminal outcomes,
  including the cancelled outcome this design writes.
- Step 17 builds the automatic arm for both blocked points,
  the intent and the plan write of a cancellation,
  the cancel command,
  and the attach path's idempotent answer (A5).
- Step 18 builds `doctor`'s lines for the intent,
  the claim,
  the plan's outcomes,
  the cancelled state and the loss.
- Step 22 measures the waits this design spends,
  which are the removal deadline and the attach's answer.

## Tests a build must have

### Device-free tests

These run with no pool,
no `/dev/zfs` and no guest,
using the stand-in device and the seams the route already has.

- The cancellation's writer sends exactly `v1:<32 hex>:mount` on the clone and nothing else,
  asserted against a stand-in device's recorded request,
  with a negative control that an `unmount` value is not sent.
- The removal claim's folder,
  prefix and generation names,
  with a live owner refusing,
  a dead owner's claim replaced by the next generation,
  and a released claim read as released.
  The seam after the listing (`single_owner_claim.rs:362`) forces the interleaving of two claimants without timing.
- The cancel command's grammar:
  no argument,
  two arguments,
  an option,
  an empty slot,
  and a bad slot name,
  each with its exit code and its stream.
- The pointer proof:
  a slot with no pointer,
  a pointer to another checkout,
  a pointer with two `gitdir:` lines and a pointer with trailing bytes,
  each refused with its own reason.
- The guard's refusals,
  driven by fixtures of each record:
  a missing binding record,
  a mismatched binding record,
  no removal record,
  two records naming different GUIDs,
  an intent that reads as inherited,
  an intent that reads as malformed,
  a plan tuple whose name now carries another GUID,
  and an incomplete inventory answer.
- The receipt answer's matching:
  a receipt under another nonce,
  another operation and another boot are each not the answer,
  and the wait's timeout report says what it last saw.
- The order of writes:
  a fault hook after the terminal plan and before the intent asserts the plan is already terminal,
  so a cancellation cannot reverse state before its loss is recorded.
- The report and refusal texts,
  each asserted whole,
  because each is text a person acts on.
- `doctor`'s lines over fixtures of every stop point 3 to 16,
  with each line's fields asserted.

### Guest tests, with the kill points named

These run in the throwaway guest with a file-backed pool,
root for the handler and the account with only the proposed grants for the wrapper,
as the plan's own arms do.

- The arm blocked at the unmount,
  which is stop point 6.
  Hold the staged clone from a process outside the caller's session,
  run the removal without `--force`,
  and assert the mount is back at the slot,
  the receipt says the clone is at its slot and the bind is gone,
  the wrapper's `mount` request is answered `done`,
  the folder,
  record and claims are gone,
  every snapshot survives,
  and the holder still runs.
- The arm blocked at the head destroy,
  which is the state stop point 12 describes with the live list empty.
  Hold the dataset busy while its snapshot list is empty,
  and assert the batch's destroyed names are gone,
  the head survives,
  the fresh attach puts the clone at its slot,
  the report names exactly those names,
  and the holder runs.
- The cancel command at each kill point,
  forced by a step-indexed fault hook that ends the process at that step and not by timing:
  stop points 3,
  4,
  5,
  7,
  8,
  9,
  11 and 14.
  Each arm asserts the command's proof,
  its settlement,
  its report and the end state,
  and each carries a control arm in the same run where the process is not ended,
  so an arm that reverses nothing fails.
- The deadline arm:
  a handler held past the removal deadline by a test-only delay seam,
  then released.
  Assert the removal's timeout report,
  exit `1`,
  that the intent still stands,
  and that a later trigger completes the unmount,
  with a control arm that is not delayed.
- The head-gone refusal:
  a removal taken past the head's destroy in the control arm,
  then the command run.
  Assert it refuses,
  mounts nothing,
  and names the removal's settle step;
  the control is the same command before the head's destroy,
  which attaches.
- The unknown-outcome arm:
  a plan with one tuple present and one absent after a killed batch.
  Assert both settle,
  that the report names the absent one as destroyed,
  and that the present one is not named;
  the control is a batch that answered,
  where both settle the same way.
- The already attached arm:
  a `mount` request for a clone at its slot,
  asserting `done` and one mount,
  with a control that the walk refuses a slot holding another mount.
- The staged bind survives arm:
  an account mount covering the folder,
  asserting the command retains and reports the folder,
  record and claim and removes nothing,
  with the undisturbed arm as its control.
- No holder ended without `--force`,
  asserted by the holder's process still running after a cancellation and after a cancel command.

## Measurements owed

Each is a probe a later round can run,
and none is run here.

- M1.
  The busy unmount's reattach,
  as a ratio.
  In one guest round,
  alternate a holder-blocked unmount that reattaches with an undisturbed unmount of a clone of the same shape,
  both in the same session,
  and report the interval from the intent's write to the receipt as a ratio of the blocked arm to the control,
  with its range.
  This sizes the cancellation's wait for the busy case;
  an absolute time is context and not a finding.
- M2.
  The attach of an unmounted clone.
  The same interleaved round with a fresh attach of an unmounted clone as the tested arm and a fresh attach of a
  clone at its slot after a clean unmount as the control,
  reported as a ratio with its range.
  This sizes the cancel command's wait.
- M3.
  The destroy settlement's read.
  On one clone,
  with its snapshot count recorded,
  read the inventory the settlement needs,
  and compare it with the same read on the same clone with no snapshots,
  interleaved,
  as a ratio with its range.
  This sizes the settlement the command runs before any attach.
- M4.
  The delegated inherit.
  As the account with only the proposed grants,
  run the property inherit on a clone and read the property back.
  Assert the request succeeds and that the read reports no local value.
  The control is a set of the same property by the same account,
  which must succeed,
  so a run where both fail shows a grant problem rather than an inherit problem.
- M5.
  The head destroy's two causes.
  With a clone whose snapshots are all destroyed,
  destroy the head while a holder keeps the dataset busy and assert `EBUSY` and an empty live snapshot list;
  then create one own snapshot and destroy the head again and assert `EBUSY` with a non-empty list.
  The control is the same destroy with no holder and no new snapshot,
  which succeeds.
- M6.
  The standing intent after a deadline.
  End the wrapper at the deadline,
  then fire a real trigger and assert the handler acts on the standing intent exactly as the design says:
  a completed unmount where the folder stands,
  and a refusal where it does not.
  The control is the same removal with no trigger fired,
  which changes nothing.
- M7.
  The forced arm.
  The plan's own owed measurement:
  end holders in each namespace kind with `--force`,
  record what each holder sees,
  and assert the caller and its session's other jobs live through it.
- M8.
  The reattach's receipt field.
  Against the built handler,
  run the blocked unmount and read the receipt:
  assert the `bind` field is `gone`,
  and on a fresh attach that it is `none`.
  The control is the account-mount arm,
  whose `bind` field must be `standing`.
- M9.
  What an unprivileged signal answers.
  In the throwaway guest,
  as the account with only the proposed grants,
  send a signal to another account's process and to a process of a system unit,
  and record each answer and each errno;
  then read systemd's and polkit's source at the version the guest runs to explain the answer.
  The control is the same signal to the same account's own process,
  which must succeed.
  This is never probed on the host,
  because an authorization request there puts a password dialog on the person's desktop and waits on that person.
  The design needs the fact only for a command it recommends,
  not for any step it performs.

## Dependencies on the sibling designs

Each dependency is stated with the assumption this design made,
so a later pass can reconcile the two.

- The add's binding record (A1):
  this design's slot-to-clone proof reads it,
  and it must be readable with no mount and no root.
  If that record's home cannot be read by an unprivileged wrapper,
  the proof needs another durable slot-to-clone record and this design's step 3 of the proof must name it instead.
- The destruction plan's home and format (A2):
  this design writes a terminal outcome and reads per-tuple outcomes,
  and it needs the record readable while the clone is unmounted and after a reboot.
  If that format has no terminal outcome,
  the cancellation's record becomes a second record beside it and its place must be named there.
- The add-time Git lock:
  this design's unmounted window between a kill and the next command relies on that lock
  keeping `git worktree prune` from deleting the registration (F3).
  If that lock is not written before the removal can run,
  this design's points 5 to 14 have an unprotected window and the cancel command must report it.
- The deferred-destroy checks of step 15:
  that design decides whether a refusal before the first delete or before the
  head destroy keeps the plan or cancels the removal.
  This design assumes it keeps the plan,
  and treats that state as one with nothing destroyed (before the first delete)
  or with the batch's destroyed set (before the head destroy).
- The forced arm:
  with `--force` the removal ends holders instead of cancelling,
  so the automatic arm of this design does not run for a forced removal,
  and the cancel command may still reverse one (the question on cancelling a forced removal).
  Who ends a holder,
  by what authority,
  and what an unprivileged signal to another account's process or to a system unit's task answers are that item's,
  and M9 owes the guest run that shows the answer.
- The lost-last-event triggers:
  the events this design's reconciliation relies on are that design's,
  and this design adds none.
- The origin-cleanup record:
  the head-gone refusal points at the removal's settle step,
  which writes that record.
  This design neither reads nor writes it.

## Hardening the stated threat model may make unnecessary

Each entry is a check this design keeps,
with what it costs and whether the project's sentence would let it go.
None is dropped here.

- The binding record's identity proof,
  the live GUID check and the live origin check survive the sentence,
  because they catch an accident:
  a reused clone name (VM item 12) and an administrator's lineage change are not attacks.
- The claim's owner-by-birth liveness test survives the sentence,
  because a reused process id would make a dead wrapper look alive and block the person's own next command.
- The pointer's bounded no-follow read and the receipt's ownership and mode checks exist only for a hostile account,
  and they also catch an ordinary surprise file in the slot.
- The staged-folder identity re-check before the folder's `rmdir` exists only for a hostile account,
  and it also catches a folder the account's own cleanup replaced.

## What this design does not cover

- Who ends a holder,
  how a holder is identified and by what authority,
  which is step 17's other open item and another writer's.
  This design neither ends a process nor asks the service manager for anything,
  so its reports only name the `--force` run as a way forward.
- How the wrapper tells its caller's session apart from other holders,
  which is the same item.
- The deferred-destroy checks' design,
  and whether their refusals cancel.
- The add's binding record,
  the destruction plan's home and format,
  and the origin-cleanup record.
- The add-time Git lock's format and the distinction between it and an operator's.
- The staging folder's own creation,
  bind,
  unmount and cleanup beyond what this design cites,
  which the accepted staging design owns.
- The start probe,
  the runtime layout checker,
  the propagation re-application and the trigger coalescing rule.
- The rungs below the ZFS clone,
  and every removal of a worktree that is not a managed clone.
- Any change to `package/git-policy/cli/SPEC.md`,
  which the build step makes with this design's sentences.

## Sources read

In this repository:

- `doc/decision/cli-git-zfs-worktree-design.md`,
  whole.
- `doc/decision/cli-git-zfs-staging-under-cache.md`,
  whole.
- `doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
  whole.
- `doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  whole.
- `doc/decision/cli-output-and-exit-codes.md`,
  whole.
- `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
  the sections this file quotes and the whole of "Removal and recovery",
  "Cancelling a blocked removal",
  "Waiting and refusal" and "Unmounted,
  pruned or moved worktrees".
- `doc/planning/cli-git-zfs-staging-under-cache-design.md`,
  its lead,
  "Answers of 2026-11-19",
  "The staging target and who creates it",
  "The private bind",
  "What the wrapper removes and when",
  "M6",
  "The staging claim",
  "The staged-folder record",
  "What a writer can settle without asking" and the veto section.
- `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
  steps 9 and 14 to 18,
  the open-item list and the gap list.
- `package/git-policy/cli/SPEC.md`,
  "Management grammar",
  "Doctor",
  "Records of a worktree that is removed",
  "The clone's mount intent,
  receipts and readiness",
  and the credential paragraph at line 7348.
  It has no removal section yet,
  because plan step 14 adds one with this design's sentences.
- The code named in "What the wrapper already has",
  read at the lines this file cites.

At a tag,
in the scratch folder:

- Git `v2.56.1`,
  `builtin/worktree.c` and `worktree.c`.
- OpenZFS `zfs-2.4.4`,
  `module/zfs/zfs_ioctl.c`,
  `module/zfs/dsl_destroy.c`,
  `module/zfs/dsl_prop.c`,
  `module/zfs/dsl_dataset.c` and `lib/libzfs_core/libzfs_core.c`.
- Linux `v7.3`,
  `fs/namespace.c`,
  `fs/pnode.c` and `include/linux/fs.h`.

## Links

[decision]: ../decision/cli-git-zfs-worktree-design.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[staging-design]: cli-git-zfs-staging-under-cache-design.md
[staging-record]: ../decision/cli-git-zfs-staging-under-cache.md
[step-8]: ../decision/cli-git-zfs-step-8-sweep-choices.md
[slot-store]: ../decision/cli-git-zfs-slot-identity-store-format.md
[exit-codes]: ../decision/cli-output-and-exit-codes.md
[spec]: ../../package/git-policy/cli/SPEC.md
[cancelling]: cli-git-zfs-worktree-design-proposal.md#cancelling-a-blocked-removal
[busy-unmount]: cli-git-zfs-worktree-design-proposal.md#project-decisions
[waiting-and-refusal]: cli-git-zfs-worktree-design-proposal.md#waiting-and-refusal

## Revisions

- The first revision,
  written 2026-11-22T17:23:19-05:00 and committed as `54d164aec4`,
  states every part of the three open items and owes every measurement it names.
