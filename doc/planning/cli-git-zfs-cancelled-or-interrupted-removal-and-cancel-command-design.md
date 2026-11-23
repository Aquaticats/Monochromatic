# What a cancelled, timed-out or killed removal leaves, and what the cancel command proves and does

Non-final,
unapproved design proposal by an agent.
The project has not reviewed it,
no other agent or model has reviewed it,
and nothing in it is final.
A gate review by a reviewer from another vendor than the author read this text three times.
The first pass found a high,
the second gave "fit after the listed fixes,
with one high that must be fixed first (N1)",
and the third,
written after the project's four further answers of 2026-11-22,
gave "fit after the listed fixes" with nothing high:
its R2's second half,
R4,
R7 and the stale statements are what this revision applies.
That review is a finding,
not an approval,
and only the project declares work complete.
The project's answers of 2026-11-22 decide one thing this text once listed as a question:
a cancellation after part of the history is gone attaches the clone again and reports each snapshot it destroyed.
The record is [the forced-removal record][forced-record],
whose "Four further answers" hold it.
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
The forced arm,
who ends a holder and by what authority,
is a sibling design's;
this design owns what any removal leaves and restores when it does not finish,
forced or not,
and the two are reconciled under "Where this design and a sibling design disagree".

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
the sibling design documents at their branch tips,
read with `git show <branch>:<path>`,
the gate review's findings and the material it kept beside them,
and three source clones it made in its scratch folder:
Git at `v2.56.1`,
OpenZFS at `zfs-2.4.4`,
and Linux at `v7.3`.

What it ran:
`gh repo clone` for those three trees,
each followed by `git remote set-url --push origin DISABLED` before anything else,
reads of their files by line number,
a scratch script that prints every citation beside the lines it cites,
and the repository's linter over this file.
It ran no ZFS command,
changed no pool,
mounted or unmounted nothing,
started no guest and no container,
opened no `/dev/zfs`,
asked the system service manager for nothing,
signalled no process it did not start,
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
lines 36 to 37):

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
lines 964 to 965):

> Settle recorded destroys only after proving the expected objects are gone,
> never from a missing name alone.
> While the head is present,
> require it to answer with its recorded live GUID and an empty live snapshot list.

### The scans before each destructive step, and the split batch

A removal asks for a fresh all-namespace scan at each point where it opens a destructive step,
and asks for each one with a new `unmount` intent under its own nonce ([proposal][proposal],
"Requests,
results and ordering",
lines 587 to 595):

> Removal needs a fresh all-namespace scan at each point where the removal workflow opens a destructive step:
> the removal-ready unmount report,
> every snapshot-destroy request,
> whether it carries the first chunk or a later one,
> and the head destroy.
> The wrapper asks for each scan by writing a new
> `v1:<nonce>:unmount` intent on the clone it removes,
> and the handler answers under that same nonce,
> so a receipt always names the scan that produced it and no receipt is reused for a later one.

A holder a scan sees stops the destruction.
The project decided that on 2026-11-22,
fixing the reading of the earlier sentence ([record][forced-record],
"What a removal without `--force` does when its scan sees a holder",
lines 47 to 56):

> The project chose "A:
> cancel,
> delete nothing (Recommended)":
> the removal stops before any snapshot is deleted,
> attaches the clone at its slot again,
> and reports the holder.

> This fixes the reading of the earlier sentence:
> a holder the scan sees stops the destruction.

The earlier sentence,
that "the scan that refuses the holders it sees stays,
then the clone's snapshots and the head are destroyed",
is read with the record's own sentence ([proposal][proposal],
"Removal when the namespace exclusion cannot be shown",
lines 1306 to 1307):
the scan refuses the holders it sees,
so the destructive step it opens does not run,
and the destruction the sentence goes on to describe is what happens where the scan sees no holder.
That reading is the one the same paragraph's later sentences support,
where "a busy unmount still blocks destruction even when the slot is clear" and without `--force` "the blocked
removal cancels itself" ([proposal][proposal],
lines 1323 to 1328).
What the "Destroy anyway" decision still settles is the case its own paragraph calls unmeasured:
a holder the kept scan missed,
and one that attached after a clean scan,
whose head destroy then refuses `dataset is busy` after the batch is gone ([proposal][proposal],
lines 1319 to 1321,
and F18).
When the verified set exceeds the request budget the batch is split by the largest fitting prefix,
and each chunk carries a fresh proof,
a fresh all-namespace verification and a fresh deferred-destroy check ([plan][plan],
"Step 16",
lines 1872 to 1875).

### The four further answers of 2026-11-22

The record's second acceptance
([forced-record][forced-record],
"Four further answers")
decides four things.
Two shape what a forced removal's refusal leaves,
and the forced arm builds both:
a holder in a namespace whose mounts are shared is reached by designing an unmount there,
"for example by making its parent mount private first,
so that the service keeps running and the removal finishes" (its "A holder in a namespace whose mounts are shared"),
as a direction the forced arm's design now states ways for,
still to be measured;
and the signal is "`SIGTERM`,
a wait for the process to exit,
then `SIGKILL` if it has not" (its "The signal"),
with no number shipped until the project decides one or a measurement sizes it.
One bounds the force word this design's states assume (its "How long a forced request's authority lasts"):

> the handler ends processes only while the very command that asked is still running,
> checked by its process identity before every signal and every unmount.

And one is this design's (its "A cancellation after part of the history is gone"):

> a holder no scan saw can surface at the head destroy,
> after some of the clone's own snapshots were destroyed,
> and the removal then cancels itself.
> The project chose "A:
> mount again and report (Recommended)":
> the wrapper attaches the clone at its slot again,
> with nobody present,
> and reports each snapshot that was destroyed.

The record's own closing section says what is still open and what is this design's own choice ([forced-record][forced-record],
"What this record does not decide"):
"that the cancel command may reverse a forced removal,
and that a cancellation's loss is no standing notice,
are the designs' own choices inside the accepted answers,
listed for the project's veto".
It also names S8,
S11 and S13 there.
All three have since been decided:
S13 by the record's "One more answer:
a unit whose main process is a script",
which refuses `--force` also when the command's session was started directly by the service manager,
so a unit whose main process is a script is caught while commands that agents run stay allowed;
and S8 and S11 by the records record
([records-record][records-record],
"How long the plan a cancellation keeps stays",
lines 90 to 101,
and "Whether a forced removal records who it ended,
beyond the journal",
lines 103 to 110),
whose answer for S8 is that "the next removal of that clone reads the record,
proves it,
carries the loss forward and ends it",
and whose answer for S11 is that the handler's journal lines are the record.
The "Correction" paragraph of its "A holder in a namespace whose mounts are shared"
withdraws a sentence it first carried about what the shared-namespace answer authorises:
how far that change may reach is for the design round to propose and for the project to confirm after the measurement,
and this design does not quote the withdrawn text.

### Cancellation, and the answer's two halves

The answer's words,
quoted by the proposal and by the record ([proposal][proposal],
lines 980 to 981,
and [record][decision],
line 54,
which keeps the half sentence about the holders):

> Kill or otherwise remove or destroy the holders whenever --force is specified,
> auto-cancel if not.

The manual way back is the project's own answer ([record][decision],
lines 41 to 44):

> A guarded `cancel-worktree-removal <slot>` stays as the manual way back,
> including for a removal whose wrapper was killed after it recorded the unmount or that ran past its deadline.
> It reports any snapshots already destroyed and cannot restore them.

The draft this design inherits its recovery reasoning from said what any reversal must prove,
and the proposal keeps it for whatever reverses a removal,
the kept command included ([proposal][proposal],
lines 1018 to 1021):

> under the per-slot claim,
> settle all outstanding destroy requests and prove the clone,
> original lineage,
> Git registration,
> mount state and exact destruction outcomes before reversing intent.
> Report any confirmed history loss before doing so and retain that report.
> If a destroy outcome is unknown or the head is gone,
> do not issue `mount`;
> report the matching resume step or administrator recovery required.

### The reattach, and the remounts the defaults allow

The busy-unmount answer is `Reattach when safe (Recommended)` ([proposal][proposal],
"Busy unmount after a staging move",
lines 1436 to 1444,
whose deciding sentence is at 1439 to 1440),
and the sentence this design applies is:

> It moves the same mount back to the original slot only after revalidation,
> and retains it staged otherwise.

The default it qualifies ([proposal][proposal],
"Decided defaults",
lines 1492 to 1498)
keeps `No implicit remount after a failure or restart`,
and the exception the accepted sentence names is:

> The exception is the checked reattach of a removal that cancels itself without `--force`,
> which the cancellation and busy-unmount decisions otherwise require,
> because an implicit remount would race the standing `unmount` intent and any staged mount.

A later answer added a second case ([dirty-record][dirty-record],
"A further answer:
an add whose own unmount is refused as busy",
lines 109 to 139):
an add that fails its proof and whose own unmount is refused as busy withdraws its request,
writes it back to the mounted state,
and leaves the clone mounted and pending.
The record's words are that "the exception now covers an add that withdraws its own request,
beside a removal that cancels itself" (lines 136 to 139).
So the default has two cases,
and this design makes neither of them:
the removal's checked reattach is the one it applies,
and the add's withdrawal is the add design's.

### Waiting, deadlines and refusal

A deadline is not cancellation,
and a timeout report has a stated shape ([proposal][proposal],
"Waiting and refusal",
lines 782 to 795,
whose sentence on the deadline is at 792 to 793):

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
lines 222 to 232,
and [staging-design][staging-design],
line 95):

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
lines 7348 to 7350,
in the section "Merges and commits never fail"):

> Avoid ever asking for someone to type a username or password.
>
> Passphrases count too (Recommended).

Nothing in this design prompts,
and a place where authority it does not have would be needed is a refusal that names the next step.
This design adds no configuration key and reads none:
every record it writes is named from the clone's dataset GUID and the pinned parent,
and the cancel command takes one argument and no setting.
A person sets nothing up for it.

The same rule reaches the host's service manager.
Nothing in this design asks `systemctl`,
the system bus or polkit for anything:
it writes records,
asks the root handler through one clone property,
and reads the files those two wrote.
The handler's unit installation,
its signals and the ending of holders are plan step 19's and the forced-arm item's.
A request that goes through the system manager is what can raise a password dialog on the desktop of the person using
the machine and wait on that person;
a plain signal to another account's process does not,
and fails `EPERM` instead (F20).
The incident of 2026-11-22T17:24:06-05:00,
where one design run asked the system manager to kill a system unit,
is the record of the first case,
and what such a request answers on this host is unverified here because the local polkit rules are unread from this
account.
The run that would show it is owed in the guest,
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
- F18.
  The measured namespace-holder case.
  With a private checkout,
  a fresh process moved the clone mount to staging and unmounted it,
  the clone vanished from the first namespace,
  and both readers' mount tables still held a ZFS row for it in their own namespaces;
  a disposable own snapshot was then destroyed while they lived,
  and the clone's head destroy refused `dataset is busy` until both stopped ([measurements][measurements],
  item 15,
  lines 610 to 666).
  The scan saw both holders as live representative tasks.
  The probe ran its destroy with no gate in the way,
  which is what the removal's scan now is:
  the project answered that a holder the scan sees cancels the removal before the first delete
  ([forced-record][forced-record],
  lines 55 to 56),
  so the probe's destroyed snapshot is the behaviour that answer forbids and not what the route does.
  The case was not a race-free proof of absence,
  and it did not test a namespace reference with no visible task.
- F19.
  A receipt's done unmount carries no mount identity,
  because "an unmount that verified absence has no mount to identify" ([spec][spec],
  lines 6170 to 6176);
  what the handler answers for an unmount of a clone mounted nowhere is not stated as a handler behaviour anywhere,
  which is A7.
- F20.
  A signal to another account's process is refused by the kernel with `EPERM` unless the sender's credentials pass
  the test at `kernel/signal.c:789` to `:797`
  or it holds `CAP_KILL` in the target's user namespace,
  and the refusal is returned at `:821` to `:835` (Linux `v7.3`).
  Nothing in that path is a dialog.
- F21.
  A request that goes through the system manager is a different path:
  `systemctl` in system scope defaults to allowing interactive authorization,
  and with `--no-ask-password` the manager answers that the operation requires interactive authentication
  (systemd `v262`,
  `src/systemctl/systemctl.c:339` to `:340`,
  `src/systemctl/systemctl-util.c:68`,
  and `src/shared/bus-polkit.c:364` to `:367`).
  Those lines were read in the systemd tree this round cloned at that tag.
  What such a request answers on this host is unverified,
  because the local polkit rules are unreadable from this account.

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
  its head's state,
  and its owner's pid and birth identity,
  and it is rewritten whole at each step.
  Its home and format are the sibling design's,
  read here as that design states them,
  and this design reads it and never writes it for a cancellation.
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
- A7.
  The handler answers an `unmount` request for a clone mounted nowhere with a clear result rather than a refusal,
  and that result carries no mount identity (F19).
  No accepted record states this as handler behaviour,
  so a build must choose it there,
  and M10 is the arm that shows it.
- A8.
  The forced arm's request property carries the pid and start time of the process that wrote it,
  and its parent's,
  and the handler honours the force word only while that process is alive,
  checked before every signal and every unmount.
  The project decided that on 2026-11-22 ([forced-record][forced-record],
  "How long a forced request's authority lasts"),
  and this design's states assume it.
- A9.
  The deferred-destroy design keeps the intent and the plan on every refusal,
  and rewrites neither ([deferred][deferred],
  "What a refusal leaves",
  its V4).
  This design's states and the cancel command assume it.
- A10.
  The plan's owner and the removal claim's owner are the same process,
  because both are written by the removal's own run,
  and the plan design says its `owner` field exists so that "a later command tells a live removal from an abandoned
  one" ([plan-record][plan-record],
  "Field by field").
  A disagreement between the two is a refusal in this design,
  not a choice between them,
  and where no plan exists the claim's owner is the one that speaks.
- A11.
  The forced arm's read-only preflight runs over every holder namespace before its first structural act,
  its first signal and its first unmount,
  and it refuses before anything is ended or changed.
  Its design now has it ([forced][forced],
  "The preflight"),
  and this design's failure path assumes it and tells the refusals it foresees from the two that can follow an act.

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
  not designed here (A1 to A3).
  Where either home changes,
  the proof and the loss record of this design move with it.
- L7.
  The wrapper's mount namespace may differ from the handler's,
  which the staging design records as a limit of its own settlement ([staging-design][staging-design],
  L13);
  this design reads the handler's namespace pair in the staged-folder record and refuses on a mismatch,
  and no arm measures the mismatch.
- L8.
  The forced arm's authority facts,
  its signal choice and its record of who was ended are that design's,
  and this design states only what the failure path restores.
- L9.
  The trigger-driven repetition of a standing `unmount` request is read from the design's own states and from no run,
  and what a holder working in the clone sees while the mount moves out and back is unmeasured.
- L10.
  No number here is measured.
  The wait this design uses is the removal deadline step 9 introduces and step 22 sizes,
  and this design ships no new constant.
- L11.
  The pre-move scan at point 5 is the forced arm's V4,
  a writer's choice on that branch,
  and this design's earliest cancellation point depends on it.
  Where that choice is vetoed,
  the earliest scan is the one after the unmount,
  which is point 8.
- L12.
  The scan's coverage of a namespace with no visible task is unmeasured (F18's own limit),
  and the project's answer of 2026-11-22 makes a seen holder cancel.
  The routes that can still destroy history before a cancellation therefore rest on what the scan cannot see,
  a lazily detached copy,
  an unattached mount held by a descriptor,
  or an incomplete read,
  which M11 owes.
- L13.
  The forced arm's read-only preflight is assumed (A11) and its design has it ([forced][forced],
  "The preflight").
  The three example reports under "The forced arm's failure path" rest on its words and are this design's shapes
  for them.
- L14.
  The reach into a namespace whose mounts are shared is a direction the record accepted on 2026-11-22
  ([forced-record][forced-record],
  "A holder in a namespace whose mounts are shared").
  The forced arm's design states its ways ([forced][forced],
  "The ways out of a namespace whose mounts are shared"),
  and no guest has measured any of them,
  so what a forced removal reports for such a holder is not yet what the route will ship.

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
and the destruction plan a sibling design defines.
The plan's grammar and home are that design's,
and this design reads them under "Dependencies on the sibling designs".

The removal writes its durable records in this order,
which this design fixes because a state has to be readable from the outside:
the removal claim,
then the staging claim and the folder,
then the intent,
then the plan.
The folder comes before the intent so that no sweep meets an `unmount` intent whose clone is mounted and whose folder
does not yet exist.
The plan's first write comes after the verified unmount,
which is the destruction-plan design's own order ([plan-record][plan-record],
"The order").

The reads at point 1 are inspection only.
Every read that decides is repeated under the claim at point 2,
so a record that changed between the inspection and the claim is never acted on,
which is the same rule the cancel command's proof follows.

1.  Before any durable write.
    The clone is mounted at its slot.
    No staged mount,
    no intent of the removal's own,
    no plan.
    The registration and its lock stand as the add left them.
    A kill or a deadline here leaves nothing that acts on the clone.
    A `--force` removal whose command's parent is the service manager,
    or whose command's session was started directly by it,
    is refused here,
    before anything is written,
    and the words tell the person to run the removal as a child of the process that wants it
    ([forced-record][forced-record],
    lines 99 to 100,
    and "One more answer:
    a unit whose main process is a script"),
    which catches a unit whose main process is a script and leaves commands that agents run allowed.
    A protected task that holds the clone is refused at the scan before any move,
    which is point 5,
    so the clone,
    its snapshots and its registration are as they were when either refusal is read.
2.  The removal claim taken.
    A claim file names a live process or a dead one,
    and the proof that decides is read again under it.
    A kill here leaves a claim whose owner is dead,
    which the next claimant replaces with the next generation and nothing to clean by hand (F16).
3.  The staging folder and its claim,
    before any intent.
    The clone is mounted at its slot;
    the folder is empty and has no staged-folder record yet,
    because the handler writes that record before its first bind;
    the intent still carries what the add left,
    which is `mount` or nothing.
    A kill here leaves an empty folder and a claim,
    and no request that can act.
4.  The intent written,
    `v1:<nonce>:unmount`,
    and the wrapper waiting.
    The clone is mounted at its slot;
    the folder is empty and unrecorded;
    the plan does not exist.
    A kill here leaves the standing request the owner's answer names,
    and the handler acts on it at its next trigger,
    because the folder exists.
5.  The unmount request's own all-namespace scan finds a holder,
    before any move.
    The clone is still mounted at its slot;
    nothing was destroyed;
    the folder may hold the handler's bind.
    The scan before the move is the forced arm's own addition,
    adopted there as
    "every scan under the force word" ([forced][forced],
    "V4.
    When the forced arm acts"),
    so this point exists only while that choice stands.
    A holder in the protected set is a refusal and not a cancellation,
    which the never-end set decides,
    and it is the forced arm's refusal `caller-protected-holds` ([forced-record][forced-record],
    "Which processes a forced removal never ends",
    lines 92 to 100).
    With `--force`,
    a holder outside that set is met by the decided reach:
    the handler unmounts the clone inside the holder's namespace and ends only the tasks that block that unmount
    ([forced-record][forced-record],
    "How much a forced removal ends in another mount namespace",
    lines 66 to 76),
    and the request goes on;
    a namespace it cannot enter,
    an unmount it is refused,
    or a blocker it cannot end makes the forced removal refuse before the first delete,
    and the forced failure path restores that.
    Without `--force`,
    a holder the scan sees stops the destruction,
    so the removal cancels itself here with nothing destroyed,
    which is the first cancellation arm under "What a removal that cancels itself restores".
6.  The handler bound the folder and moved the clone onto it,
    and either its scan sees a holder or the unmount refused `EBUSY`.
    The clone was staged at the folder,
    and both answers move it back to its slot;
    nothing was destroyed;
    the handler files the cancelled-removal receipt.
    This is the second cancellation arm.
7.  The handler moved the clone and unmounted it,
    before the unbind.
    The clone is mounted nowhere;
    the folder holds the empty bind;
    the record names the folder's handle and identity.
    A kill here leaves a bind a later sweep unbinds and then files a clear receipt for.
8.  The unmount and unbind done,
    the all-namespace scan sees a holder,
    and no receipt is filed yet.
    The clone is mounted nowhere in this namespace,
    a copy stands in another,
    and nothing was destroyed.
    This is the scan the proposal calls the removal-ready unmount report,
    which runs after the host's unmount ([proposal][proposal],
    "Requests,
    results and ordering",
    lines 587 to 595),
    and [VM item 15][vm-15] is the case it measured (F18).
    Without `--force` the removal cancels here,
    with nothing destroyed,
    and the clone is attached again at its slot,
    which is the arm blocked by a scan under "What a removal that cancels itself restores".
    With `--force` the forced arm acts on the holder,
    and a namespace it cannot enter,
    an unmount it is refused,
    or a blocker it cannot end makes the removal refuse before the first delete.
9.  The unmount and unbind done,
    the clear receipt filed,
    the wrapper not yet its reader.
    The clone is mounted nowhere;
    the folder is empty;
    the receipt names the clone absent in every namespace and the bind gone;
    no plan.
    A kill here leaves a state a later removal's staging sweep settles on the receipt.
10. The wrapper read the receipt,
    removed the folder,
    unlinked the record and released the staging claim.
    The clone is mounted nowhere;
    no staged state;
    the removal claim is held;
    no plan.
    A kill here leaves an `unmount` intent with no folder,
    which the handler answers as a scan rather than acting on,
    because the clone is mounted nowhere (A7).
11. The plan's first write,
    with `head=present` and no tuples.
    The clone is mounted nowhere and its snapshots are all present;
    the removal claim is held.
    A kill here leaves a plan a later removal resumes from and a cancel may reverse,
    since nothing was destroyed.
12. The scan before the first snapshot delete finds a holder.
    The clone is mounted nowhere,
    the holder is in a namespace the host's own unmount did not reach,
    and nothing was destroyed yet.
    This is the case [VM item 15][vm-15] measured (F18),
    and the project's answer is that a holder the scan sees stops the destruction ([forced-record][forced-record],
    lines 55 to 56).
    Without `--force` the removal cancels here,
    with nothing destroyed and the clone attached again at its slot,
    which is the arm blocked by a scan under "What a removal that cancels itself restores".
    With `--force` the forced arm acts on the holder at this scan,
    and a namespace it cannot enter,
    an unmount it is refused,
    or a blocker it cannot end makes the removal refuse before this delete,
    which the forced failure path restores.
13. The deferred-origin check before the first delete refused.
    Nothing was deleted;
    the intent and the plan are kept,
    because the deferred-destroy design keeps them on every refusal ([deferred][deferred],
    "What a refusal leaves").
    The removal neither cancels nor finishes here,
    and the cancel command is the way back.
14. A chunk submitted,
    its per-name outcomes not recorded.
    The clone is mounted nowhere and its head is present;
    the plan holds that chunk's tuples as `requested`,
    which is the plan's unknown state,
    and every earlier chunk's outcomes.
    A kill here leaves an outcome only per-name presence settles (F7,
    F8).
15. A chunk's outcomes recorded,
    and the next chunk's scan,
    or the scan before the head destroy,
    about to run.
    The earlier tuples carry their outcomes;
    a scan that sees a holder is point 12 again,
    with that chunk's or the batch's destroyed tuples reported,
    and a deferred refusal is point 13 again.
    The record decides a scan that sees a holder before any delete ([forced-record][forced-record],
    lines 47 to 56),
    and it decides the restore for a cancellation after part of the history is gone (its "A cancellation after part of
    the history is gone").
    Applying that restore to a holder a later scan sees,
    rather than only to the head destroy the record names,
    is this design's extension,
    listed for the project's veto.
    The plan is rewritten before each chunk and after it,
    so a kill leaves either the request's tuples or their outcomes.
16. The check before the head destroy refused.
    The head is present and the batch's destroyed tuples are gone;
    the intent and the plan are kept,
    by the deferred-destroy design's rule.
    This is not a cancellation:
    the removal refuses and the cancel command is the way back.
17. The head destroy submitted,
    its answer not recorded.
    The head may be gone and the clone with it;
    the plan holds `head=requested`.
    Nothing may attach here,
    and a settlement proves the head by its name's absence and by no child of the pinned parent carrying its recorded
    GUID ([proposal][proposal],
    lines 964 to 965).
18. The head destroy refused `dataset is busy`,
    with the clone's own live snapshot list empty.
    The clone is mounted nowhere,
    the batch's snapshots are gone,
    and the head,
    the registration and the plan stand.
    This is the case a scan cannot catch:
    a holder the kept scan missed,
    or one that attached between the last scan and the destroy ([proposal][proposal],
    lines 1319 to 1321).
    Without `--force` the removal cancels itself,
    which is the fourth cancellation arm,
    and the restored state is a fresh attach with the destroyed set reported.
    With `--force` this is the forced failure path,
    because the answer's own promise was that the holders would be ended.
19. The head destroy proven gone.
    The clone is gone,
    and the intent went with it,
    because it was a property of that dataset.
    The registration and its lock stand until the settle removes them (F1,
    F2);
    the underlay pointer stands;
    the plan's tuples are settled.
    Nothing here can be reversed,
    and the cancel command refuses.
20. The registration settled and the underlay removed.
    Nothing of the removal is left but the released claim and the plan's record.
    A cancel command finds no removal to reverse.

From points 1 to 19 the Git registration and its lock stand unchanged,
because settling them belongs to the step after the head's destroy,
and the lock is what keeps `git worktree prune` away from a registration whose clone may stand unmounted for a while
(F3).
The slot identity store's record and the underlay pointer stand unchanged for the same reason,
and the intent and the plan are the only records that change inside a removal.

Five points are where a removal stops by cancelling itself,
and each is a scan that sees a holder,
a blocked unmount,
or a head destroy a holder outlasts:
the scan before the move (point 5,
where the forced arm's pre-move scan is adopted),
the blocked unmount in staging (point 6),
the scan after the unmount (point 8),
the scan before a delete (points 12 and 15),
and the head destroy a holder outlasts (point 18).
A holder in the protected set is a refusal at each of them,
not a cancellation,
because the removal never ends the caller,
its session or its ancestors;
its state is the same restore,
with different words.
A scan that sees a holder cancels because the project decided that it stops the destruction
([forced-record][forced-record],
lines 55 to 56),
and the deferred-destroy refusals do not cancel,
which is that design's rule.
Every other point refuses and keeps its records,
which is the proposal's rule for a proof,
a dependency or a hold that fails ([proposal][proposal],
"Removal and recovery",
step 2).

### The cancelling sequence's own states

A cancellation is itself a sequence,
and it can be killed at any of its steps.
Its states are these.

- C1.
  The handler moved the mount back and filed the cancelled-removal receipt,
  and the wrapper has not yet written the intent.
  The clone is at its slot,
  the intent still says `unmount`,
  and the folder,
  its record and its claim stand.
  The request can be acted on at each trigger until the intent changes,
  which is why the intent write is the wrapper's first act after the receipt.
  A kill here leaves the request live and the clone at its slot.
- C2.
  The wrapper wrote `v1:<fresh nonce>:mount` and no answer has arrived.
  The clone is at its slot,
  the intent asks for a mount,
  and the folder and its records stand.
  `doctor` reads the state as "cancelling".
  A kill here leaves nothing to act on,
  because the intent asks for the state the clone is in.
- C3.
  The answer `done` arrived and the folder is not yet removed.
  The clone is at its slot,
  the intent asks for a mount,
  and the folder,
  record and claim stand.
  A kill here leaves the same state,
  and the next removal or cancel command settles the folder.
- C4.
  The folder is removed,
  the record unlinked and the staging claim released.
  The removal is cancelled:
  the intent asks for a mount,
  the plan (where one exists) keeps its destroyed set and its dead owner,
  and the removal claim is released.
  `doctor` reads the state as "cancelled".

The state between C1 and C2 is the one a person sees when a cancellation's wrapper is killed at its first step,
and it is the reason the intent write comes before the folder's settlement.
A cancellation that is killed at C2 or C3 leaves a clone at its slot with a `mount` request standing,
which asks for the state it is in and therefore cannot move anything.
## What a removal that cancels itself restores

Each cancellation arm below ends with the clone usable at its slot and the removal ended.
They differ in what was destroyed before the block,
in what the clone's mount needs,
whether nothing at all,
a move back,
or a fresh attach,
and in what the report says was lost.

### The arm blocked before the move

The unmount request's own scan sees a holder before the handler moves the clone (point 5).
This arm exists while the forced arm's pre-move scan stands (its V4);
where that choice is vetoed,
the earliest scan is the one after the unmount,
which is the arm blocked by a scan.
Nothing was destroyed,
and the clone is still mounted at its slot,
so there is nothing to move back and nothing to attach.
The wrapper,
still holding the removal claim:

1.  writes `v1:<fresh nonce>:mount` on the clone,
    which replaces the standing `unmount` request with a request for the state the clone is in;
2.  waits for the answer under that nonce,
    for the removal deadline.
    A clone at its slot with its recorded identity is answered `done` without a second attach (A5);
3.  settles the staged state:
    where the handler bound the folder before its scan,
    the cancelled-removal receipt says so and the wrapper removes the folder on it;
    where no bind was made,
    the folder and its claim are settled by the staging rule for an empty bind;
4.  releases the removal claim and prints the report,
    which says that no snapshot was destroyed.

No plan exists at this point,
so nothing records a loss,
because there is none.

### The arm blocked in staging

The holder keeps the staged clone busy.
Two answers reach the same restore:
the scan before the unmount calls sees it and refuses,
and the unmount itself refuses `EBUSY` (F11).
Either way the handler revalidates and moves the same mount back to the slot,
unbinds its bind where its own topmost check still finds the recorded bind,
and files the receipt the staging design calls the cancelled removal's ([staging-design][staging-design],
A5).
Nothing was destroyed,
so the loss is empty and the clone keeps every snapshot it had.

The wrapper then does what the first arm does,
with one difference:
the receipt already names the clone at its slot and the bind gone,
so the intent write is the wrapper's first act and the folder's settlement follows the receipt.
It releases the removal claim and prints the report.

### The arm blocked by a scan

A scan that sees a holder is where the project's answer of 2026-11-22 bites,
and it can be met at three points,
all with the clone mounted nowhere in this namespace and a copy in another:

the scan after the unmount (point 8),
the scan before the first snapshot delete (point 12),
and the scan before a later chunk or before the head destroy (point 15).

The holder is a copy in a namespace the host's own unmount did not reach,
which is the case [VM item 15][vm-15] measured (F18),
and the handler files a refusal naming it rather than a clear receipt.
Nothing was destroyed at points 8 and 12;
at point 15 the earlier chunks' tuples are gone and the plan records them.
The wrapper,
still holding the removal claim:

1.  writes `v1:<fresh nonce>:mount` on the clone,
    which is its first act,
    because the refusal is already in hand and the standing `unmount` request must not be acted on again;
2.  waits for the answer under that nonce,
    for the removal deadline.
    There is no mount to move back,
    so the answer is a fresh attach of the clone at its slot (A5);
3.  settles the staged state on the receipt,
    which says the bind is gone because the handler unbinds before it scans;
    where the bind survives,
    the folder,
    record and claim are retained and reported;
4.  releases the removal claim and prints the report,
    which names the holder,
    says that the removal cancelled itself because a scan saw a holder,
    and says what was destroyed:
    nothing at points 8 and 12,
    and the earlier chunks' names at point 15.

No plan is written by this arm.
At point 15 the plan already records the earlier chunks,
and a cancellation writes nothing to it.

### The arm refused at the head destroy

The batch's snapshots are gone,
the clone is mounted nowhere,
and the head destroy refuses `EBUSY` while the clone's own live snapshot list is empty (F6).
Two causes reach that errno and the design tells them apart by the live list:

- A snapshot appeared after the final enumeration.
  The live list is not empty.
  This is not a holder and not a cancellation.
  The removal's own amendment rule applies,
  which the proposal states as a rerun of every check with the amended plan recorded and no prompt
  ([proposal][proposal],
  "Removal and recovery",
  step 3).
- Holders keep the dataset busy.
  The live list is empty.
  Without `--force` the removal cancels itself here,
  which is the case the record decides:
  the clone is attached again with nobody present,
  and each destroyed snapshot is reported ([forced-record][forced-record],
  "A cancellation after part of the history is gone").

For the second cause the wrapper writes the same fresh `mount` request and waits,
and the attach is a fresh one,
because there is no mount to move back:
the handler creates a mount of the clone at the slot under the same checks it uses for an add's attach (A5).
The clone comes back at its slot,
with its files,
and without the snapshots the batch destroyed.
Those snapshots do not come back,
and the report names each one.
The plan already records them as destroyed,
and a cancellation writes nothing to it.

### A cancellation after part of the history is gone

This is the case the record decides ([forced-record][forced-record],
"A cancellation after part of the history is gone"),
and it was this document's question S7 until 2026-11-22.
The wrapper writes the same fresh `mount` request under a new nonce and waits,
and the handler attaches the clone at its slot again with nobody present,
because there is no mount to move back (A5).
The report names each snapshot the batch destroyed,
and the plan keeps them,
which is all the loss record there is.
The cancellation writes nothing to the plan,
and the intent,
the claim,
the folder and the registration take the fates "What the removal record does,
part by part" gives.
The same restore is what the arm blocked by a scan makes at point 15 when a holder first appears after a chunk;
the record's trigger is the head destroy,
and reaching the same restore at a later scan is this design's extension,
listed for the project's veto.

A fresh attach can itself be refused,
for example when a holder now sits on the slot's folder.
The clone then stays unmounted with the history the batch destroyed,
and nothing is retried blind:
the intent keeps asking for a mount,
the plan keeps the destroyed tuples and its dead owner,
the report says the attach was refused and names the handler's failed step,
the wrapper exits `1`,
and the cancel command's next run refuses at the handler's words rather than attaching either.
`doctor` reads the state as cancelled with the loss,
and the ways back are the holder's release,
then the removal run again or the command.

### When a cancellation can report destroyed history

With a scan that sees a holder cancelling before any delete,
an automatic cancellation can report destroyed history only through a holder the scans did not see,
one that appeared after the scan that would have seen it,
or a scan whose own read was incomplete.
There are seven routes,
and each names the point it ends at:

- The holder attached after the scan before the first delete,
  so the first chunk ran,
  and the scan before a later chunk or before the head destroy sees it.
  The cancellation is point 15's,
  and the report names the earlier chunks' tuples.
- The holder is invisible to every scan:
  a pinned namespace with no visible task,
  a namespace the scan does not reach,
  or a task change racing the scan.
  The proposal calls these unmeasured for its own scan ([proposal][proposal],
  lines 1319 to 1321),
  and F18's item did not test them.
  Every chunk runs,
  the head destroy refuses `dataset is busy`,
  and the cancellation is point 18's with the whole batch reported.
- The holder attached between the scan before the head destroy and the destroy itself,
  which is the window the proposal keeps when it keeps the scan rather than making the destroy atomic with it.
  The same point 18 and the same loss.
- The removal died or ran past its deadline after a chunk,
  so no cancellation of its own ran at all.
  The whole batch's tuples are in the plan,
  and a person's cancel command,
  or a resumed removal,
  reports them (C4).
- The copy was lazily detached before the scan:
  a mount no mount table names,
  kept alive by a working directory or an open file.
  No scan that reads mount tables can see it ([staging-design][staging-design],
  L11),
  so every chunk runs and the head destroy refuses `dataset is busy`.
  The cancellation is point 18's,
  and the report says the holder is a detached copy no scan can name.
- The copy is an unattached mount held by a descriptor,
  made with `open_tree` and `OPEN_TREE_CLONE` or with `fsmount`.
  It stands in no namespace's mount table,
  so the scan does not name it and the dataset stays busy while its holder lives.
  The same point 18 and the same loss.
- The scan's own read was incomplete:
  a namespace it could not read,
  or the scan's 1,024-namespace bound,
  which the forced arm's design reports and never reads as clear.
  A reported incomplete read does not refuse by itself,
  so the removal goes on and the batch's loss can follow.

Two other endings report destroyed history without being cancellations:
the deferred-origin refusal after a batch (points 13 and 16),
which keeps the intent and the plan and is reversible by the command,
and the forced failure path at points 15 and 18,
which reports the batch's tuples as a cancellation does.

So the loss the open item names survives where a scan's view and the destroy's moment part,
where no mount table names the holder,
or where the scan itself was incomplete.
The attach after it is decided ([forced-record][forced-record],
its "A cancellation after part of the history is gone"),
and the plan's lifetime with it ([records-record][records-record],
lines 90 to 101),
so no question of this document's is left about the loss;
the scan's coverage is what M11 owes.

### The forced arm's failure path

A removal run with `--force` ends holders instead of cancelling.
Its reach is the project's own answer of 2026-11-22:

> the handler unmounts the clone inside the holder's namespace and ends only the tasks that block that unmount,
> and when the namespace cannot be entered or the unmount is refused,
> the forced removal refuses and names the holder.
> It never falls back to ending every task of the namespace.

([forced-record][forced-record],
"How much a forced removal ends in another mount namespace",
lines 69 to 72),
and the record's own conclusion is that "a forced removal can therefore still refuse" (line 76).
Its design has a read-only preflight over every holder namespace before its first signal or unmount (A11,
and [forced][forced],
"The preflight"):
it enters each namespace,
reads the table,
runs every structural check,
confirms a destination can be made,
and compares every holder with the protected set.
Two kinds of refusal follow from that,
and only the second can leave a process of this run ended.

Refused by the preflight,
before anything is changed:

1.  The holder's namespace cannot be entered.
    Nothing in that namespace is unmounted or ended.
    The report names the namespace,
    the call that refused and its errno,
    and says that nothing was ended.
    The forced arm files it as `holder-unendable` ([forced][forced],
    "The refusal,
    and what it leaves").
2.  A structural check fails:
    a mount under the copy,
    which no signal clears,
    a second mount of the dataset,
    a namespace whose root mount is the copy,
    a parent mount that is not the checkout's own,
    no destination or no room for one,
    or,
    on way 8's route,
    a receiver in the closure with a mount at the slot outside the namespaces the act handles.
    The report names the check that failed and the namespace,
    and says that nothing was ended and no signal clears the cause.
    `holder-unendable` is its word too.
3.  A holder in the protected set,
    which is the never-end set's refusal `caller-protected-holds`.
    Its state is point 5's or point 6's,
    according to where it is met,
    with nothing moved,
    ended or destroyed.
4.  A holder the preflight proves cannot be ended,
    which is `holder-unendable`,
    and a read that came back blind,
    which refuses too and which the forced arm's receipt names by its failed step rather than by one of the three
    words.
    Both refuse before any signal is sent,
    with nothing moved,
    ended or destroyed.

Refused during the structural stage,
which is the third state the forced arm names ([forced][forced],
"The refusal,
and what it leaves"):

5.  The kernel refuses a change the reading could not foresee,
    such as a propagation change or a destination the kernel will not take.
    Nothing was ended;
    the namespaces before it were changed (a parent made a slave,
    a directory made) and are moved back where possible
    (the forced arm's V6);
    and the report names the change,
    the move back and the directory.
    The word is `holder-unendable`.

Refused after an act,
which are the only refusals where a process of this run has already ended:

6.  A blocker the act could not end although the preflight found it endable:
    it survived the wait and the `SIGKILL`,
    so the unmount did not happen.
    The tasks this run ended in that namespace before the refusal stay ended,
    and the report names them beside the blocker,
    as `holder-unendable`.
7.  The unmount refused `EBUSY` after the matcher found no holder:
    a reference the design cannot see,
    which the forced arm files as `holder-unseen`.
    The tasks this run ended before the refusal stay ended,
    and the report names them.

Each is met at point 5,
8,
12,
15 or 18,
whichever ran first,
and what is left is:

- the clone where the removal met it:
  mounted at its slot before the move,
  staged or back at its slot after a blocked unmount,
  and mounted nowhere once a chunk ran;
- nothing destroyed where the failure is met before the first delete,
  and only the earlier chunks' tuples destroyed where it is met at point 15 or 18,
  with those names in the report and in the plan;
- nothing ended where the preflight or the structural stage refused,
  and the ended set named where a refusal came after an act;
- in the holders' namespaces,
  whatever the structural stage did before a refusal stands until it is undone:
  a parent made a slave is moved back where possible (the forced arm's V6),
  and a destination directory that was made and could not be removed is named by the report,
  printed by `doctor`,
  and removed by the next run or by this design's cancel command when it carries the reserved prefix and the clone's
  GUID,
  is empty,
  root-owned with mode `0700`,
  and sits directly under the parent mount's root (the forced arm's V7);
- the intent reversed to `mount` under a fresh nonce,
  exactly as a cancellation reverses it;
- the staging folder,
  record and claim settled on the receipt,
  or retained and reported where the bind survives;
- the plan left as the removal wrote it,
  the registration,
  its lock,
  the slot record and the underlay untouched;
- the removal claim released and the removal exiting `1`.

Three example reports,
with the fixture's names.
A namespace the arm could not enter:

```text
removal of `.claude/worktrees/one` was asked to end the holders other than your protected set and could not:
the namespace mnt:[4026532397] holds the clone, and entering it refused `EPERM`. Nothing was unmounted, nothing was
ended and nothing was destroyed. The clone is at its slot and the intent asks for a mount.
Ways forward: this needs an administrator, or the owner of that namespace ending its unit, then run the removal again.
```

An unmount that refused after an empty search:

```text
removal of `.claude/worktrees/one` was asked to end the holders other than your protected set and could not:
the preflight and the act's own reads found no holder in mnt:[4026532397], and the unmount there refused `EBUSY`: a
reference this removal cannot see remains, perhaps a child mount at `/tmp/build` made after the preflight. No process of
this removal was ended and nothing was destroyed. The clone is at its slot.
Ways forward: unmount or stop the job that made that mount, or name the reference to an administrator, then run the
removal again.
```

A blocker the arm could not end:

```text
removal of `.claude/worktrees/one` was asked to end the holders other than your protected set and could not:
the blocker is pid 51233 (build-worker) in mnt:[4026532397]. The preflight found it endable, `SIGTERM` expired, and
`SIGKILL` was sent, and the kernel has not ended it, so the unmount did not happen.
Two other holders were ended: pid 51240 and pid 51241. Nothing was destroyed. The clone is at its slot.
Ways forward: wait for pid 51233 to leave the state reading could not foresee and run the removal again, or ask an
administrator to end it.
```

One refusal is not a failure of that promise and restores the same state.
A `--force` removal whose command's parent is the service manager,
and one whose command's session was started directly by the service manager,
are refused outright ([forced-record][forced-record],
lines 99 to 100,
and "One more answer:
a unit whose main process is a script"),
and each refusal is decided before anything is touched:
it leaves the state point 1 describes,
with no claim,
no folder,
no intent and no plan written,
and the words tell the person to run the removal as a child of the process that wants it.
The session case catches a unit whose main process is a script,
which the parent alone misses;
commands that agents run stay allowed,
because their sessions are started by the agent,
and a console login's session is refused with the orphan sentence.

`caller-protected-holds` is the other word the forced arm uses,
for a holder in the protected set,
and it is the refusal the preflight makes at failure 3:
nothing moved,
nothing ended,
nothing destroyed.

`holder-unendable` is the forced arm's word for a blocker it cannot end,
which is failure 6,
and for the preflight's refusals at failures 1,
2 and 4,
and for a kernel refusal during the structural stage,
which is failure 5;
the report beside it names the blocker and the holders this run did end.

`holder-unseen` is its word for an `EBUSY` after an empty search,
which is failure 7,
and the report names the copy,
the namespace and the errno.

The reversal is this design's,
because it is the state a removal leaves when it does not finish.
The ending of holders,
the failures' refusals,
the preflight's refusals,
the service-manager refusal and the words they print are the forced arm's,
and this design needs five things from that design:
its read-only preflight over every holder namespace before its first signal or unmount (A11),
so a refusal its reading foresees leaves nothing ended;
the pid and start time of the writer in its force property,
so a request that outlives its wrapper ends nobody (see "What a timed-out or killed removal leaves");
its refusal kinds and their words,
`caller-protected-holds`,
`holder-unendable` and `holder-unseen` among them,
so the refusals are told apart in the report;
its record of which holders it ended,
so the report can name them;
and that it never rewrites the intent itself,
so one place writes the one reversing request.

### What the removal record does, part by part

The open item asks whether the removal record is reversed or kept.
The design's answer is that the records a removal leaves have four different fates:

- The mount intent is reversed.
  The clone's desired state becomes `mount` under a fresh nonce,
  so no later sweep unmounts a clone nobody is removing.
- The removal claim is released.
  It marked a running removal and there is none.
- The destruction plan is left exactly as the plan recorded it.
  It holds each tuple's outcome,
  the head's state and the dead owner,
  and the destroyed tuples are the only durable record of the loss.
  A cancellation writes nothing to it,
  which is the same rule the plan design states for its finish pass:
  "a settle is not an outcome"
  ([plan-record][plan-record],
  "The finish pass that settles a registration").
  A later removal of the same clone rewrites the plan under the same key,
  and that is the natural end of the loss record's life.
- The staging claim,
  folder and record are settled as the cancelled-removal receipt allows,
  by the staging design's own rule.
- The Git registration,
  its lock,
  the slot identity store's record and the underlay pointer are untouched.
  The clone still exists and is still registered,
  so nothing of the registration was due to be settled at any point before the head's destroy (points 1 to 18).
  The lock matters here for a reason outside this design's own record:
  it is what keeps `git worktree prune` from deleting the registration of a clone that stands unmounted for a while
  (F3),
  while `git worktree remove` refuses a locked worktree without a double force (F2).
  Which lock is the wrapper's own and which is an operator's is the add-time-lock item's question.

The plan design's own list of end states has no cancelled state,
and this design needs none:
the intent says `mount`,
no live removal claim names the clone,
and the clone is present,
and those three facts are what `doctor` reads as a cancellation.
Where a plan exists,
its dead owner corroborates them.
That also answers the deferred-destroy design's third option,
which asked for "a state and a removal rule that sibling does not have":
the state it wanted is the intent plus the kept plan,
and the rule is the cancellation this design writes.

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
and the way forward.
Four examples,
with the fixture's names.
The case the project's answer of 2026-11-22 created,
a scan that sees a holder before a delete:

```text
removal of `.claude/worktrees/one` cancelled itself: the all-namespace scan before the first snapshot delete saw a
holder in mnt:[4026532396], so nothing was destroyed and every snapshot survives. The clone is attached again at its
slot.
The scan answers the `unmount` request under nonce 3f2a...; the removal wrote a `mount` request under nonce 9c41...
and the handler answered it.
The holder is a process in another session reading through the clone; this removal ends no holder without `--force`.
Ways forward: end that job, or run the removal with `--force`, and run the removal again.
```

The unmount blocked in staging,
where a holder keeps the staged clone busy:

```text
removal of `.claude/worktrees/one` cancelled itself: the clone's unmount was blocked by a holder in another session,
so nothing was destroyed and every snapshot survives. The clone is at its slot.
The request that was blocked is `unmount` under nonce 3f2a...; the removal wrote a `mount` request under
nonce 9c41... and the clone is mounted at its slot.
The holder is a process in another session reading through the clone (mnt:[4026532396]); this removal ends no holder
without `--force`. Ways forward: end that job, or run the removal with `--force` to end the holders other than your
session, and run the removal again.
```

The loss the open item names,
which only a holder no scan saw reaches:

```text
removal of `.claude/worktrees/one` cancelled itself after destroying snapshots that cannot be restored:
`zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one@sanoid-hourly-2026-11-22_03`,
`zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/one@sanoid-hourly-2026-11-22_04`.
The head destroy refused `EBUSY` with the clone's own snapshot list empty: a holder the scan could not see, or one
that attached after its last scan, keeps the dataset busy. The clone is mounted at its slot again without those two
snapshots, and its files are unchanged. `git monochromatic-git-policy-cli doctor` reports the same loss and the kept
plan. Ways forward: identify and release the holder, then run the removal again.
```

A forced removal that could not do what it promised reports under "The forced arm's failure path",
with its own three examples.

A holder in the protected set is the case a person meets most,
and the decision's own words for it are a refusal,
"the removal refuses and says to leave the folder and run the command again" ([record][decision],
lines 49 to 54),
not a cancellation.
The set is the caller's session and every ancestor of the command up to the first process,
which the project decided on 2026-11-22 ([forced-record][forced-record],
"Which processes a forced removal never ends",
lines 92 to 100),
and the forced arm files that refusal as `caller-protected-holds`.
In state the refusal is the same restore as the arm blocked before the move:
if the clone was never moved,
its mount never left the slot,
and if a later stage refused,
the move back or the fresh attach is the same one the cancellation makes.
Only the words and the reason differ,
and the report names the holder and what releases it.
The decision's words cover only a holder whose working directory is the folder.
A holder that is a task in a mount namespace the caller's session created,
such as a unit with `PrivateMounts=yes` started after the clone was attached,
keeps a copy of the clone after the working directory is left,
and nothing but ending that namespace's last task or the unit releases it.
The report names that case in its own words rather than offering the working-directory advice:

```text
removal of `.claude/worktrees/one` refused: a process holds the clone, and this removal never ends the caller or the
processes of its session or its ancestors. The holder is a process of this account in this command's own session,
`zpcachyos/ROOT/cos/home/...` is held in mnt:[4026532299].
Nothing was destroyed. The clone is at its slot.
Ways forward: leave the folder and run the command again from another checkout; where the holder is a unit of your
session with its own mount namespace, end that unit, because leaving the folder releases nothing while the namespace's
last task lives. Run the removal with `--force` only where the holder is not in your protected set.
```

### What `doctor` prints

The proposal asks `doctor` for the standing intent,
the last verified loss,
the remaining objects,
whether the removal will cancel itself,
and the name of the kept command ([proposal][proposal],
"What doctor checks and prints",
and the doctor paragraph quoted there).
This design fixes the fields and the words those sentences need,
and plan step 18 builds them:

- the slot,
  its registration and the dataset with its GUID;
- the intent's operation and nonce,
  read through the same reader the wrapper uses,
  with an absent or inherited value said to be no request;
- the mount state at the slot,
  read from the mount table,
  and any staged folder with its dataset and its age;
- the removal claim's owner and the plan's owner,
  live or dead,
  and a line when the two disagree;
- the plan's state:
  none,
  in progress,
  a chunk's outcomes,
  the head's state,
  or gone;
- for a cancellation,
  whether it is "cancelling" or "cancelled" by the two facts that tell them apart:
  a `mount` request with no answer is cancelling,
  and a `mount` request answered,
  with no live removal claim and the clone present,
  is cancelled;
  a plan's dead owner corroborates it where a plan exists;
- every tuple the report marks destroyed,
  with the time from the plan's `started` field;
- the next safe step,
  which is one of the commands the reports above name,
  or the administrator's where a record cannot be proven.

`doctor` changes nothing,
so it neither reverses a removal nor settles a folder.
It reads what the removal and the cancel command wrote.

## What a timed-out or killed removal leaves

### Whether the standing request can act

A removal that is killed or runs past its deadline leaves the intent standing,
and the handler keeps acting on it at its next trigger.
Whether the action can move the clone depends on the wrapper's staging folder:

- Where no folder stands (point 10,
  and any request whose folder was never made),
  the handler cannot bind,
  so a clone that is mounted is refused rather than unmounted,
  and a clone that is mounted nowhere is answered as absent (A7).
  Nothing moves.
- Where a folder stands (points 3 to 9),
  the handler acts:
  it binds the folder,
  moves the clone onto it,
  and then either the scan sees a holder or the unmount does its work.
  A scan that sees a holder refuses and the handler moves the clone back,
  which is the cancellation point 6 records;
  otherwise the unmount either completes with the unbind or refuses `EBUSY` and the handler moves the clone back.

So a killed removal whose holder stays does not come to rest.
Each trigger repeats the move out and back,
against a holder that may be working in the clone,
and the first trigger after the holder leaves completes the unmount.
The clone is then mounted nowhere with nobody continuing the removal,
which is why this design does not call the state inert or converging.
Two things bound the repetition:
each attempt is bounded by the removal deadline,
and the attempts stop when the holder leaves,
when the wrapper rewrites the intent to `mount`,
which is a cancellation's first act,
or when the cancel command does.
No timer and no cap is added here;
the coalescing of nudges is the lost-last-event design's.
What a holder working in the clone sees while the slot's mount moves out and back is unmeasured (L9,
M10).

A forced request that outlives its wrapper is the same state with one more hazard:
the force word could end holders that attached after the person gave up.
The bound is this design's A8,
which the record decided ([forced-record][forced-record],
"How long a forced request's authority lasts"):
the request carries the writer's pid,
start time and recorded parent,
and the handler honours the force word only while that process lives,
checked before every signal and every unmount;
a trigger that finds a dead owner answers the request without the force word and says so in its receipt.
This design's states assume that bound:
an `unmount` intent a dead forced removal left is acted on as an ordinary request,
and its answer can still be a completed unmount where the holder has gone.
The removal claim's owner and the plan's owner name the same process (A10),
so a reader tells the abandoned case from a live removal without trusting the property alone.

While a forced act is running,
the handler and the wrapper each need to know the other lives,
and the forced arm states that rule under "The act's liveness" ([forced][forced],
read at `f7765aea32`):
the handler writes a root-owned progress file beside the scan file,
named with the clone's GUID and carrying the request's nonce and a counter it advances at each stage and once per
liveness interval while it waits;
the wrapper reads that counter while it waits,
prints that the request still stands when its own deadline passes,
and gives up only when the counter has not moved for the interval,
saying then that the handler is gone,
that nothing further will be ended,
and that the request stands as non-force work.
Both intervals are constants with no value until M12 sizes them.
What this design's two actors do when the wrapper dies during the act:
the handler sends no `SIGKILL`,
stops and files `caller-gone`,
so the holders the act had not yet signalled stay;
the intent and the plan stand as the removal left them,
and the wrapper's next command,
or the next trigger,
answers the standing request as non-force work,
which is the state this paragraph's first half describes.
What this design's cancel command does when it meets that state:
its proof reads the live state and the records,
and the handler's `caller-gone` answer tells it nothing was ended after that point;
it takes the removal claim (the dead wrapper's claim is replaced),
reports the destroyed set,
and says that who was ended is the handler's journal,
which is the forced arm's record of it;
then it attaches or settles as "The act" states,
or refuses at the handler's words.

A cancellation that is itself killed leaves the states C1 to C4 under "The cancelling sequence's own states".
The hazard there is C1:
the intent still says `unmount`,
and each trigger can act on it until the wrapper or the cancel command rewrites it.
That is why the intent write is the wrapper's first act after the cancelled-removal receipt.

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
The removal adds four sentences to it:

- what the wait last observed,
  and no more than that:
  the deadline sits inside the wait,
  which cannot see the mount,
  so the report never asserts what state the clone is in;
- whether a plan exists,
  and therefore whether anything was destroyed;
- that a deadline is not a cancellation,
  so the worktree is not back and the removal is not abandoned,
  and that the standing request may still act,
  including a completed unmount after a holder leaves or a repeated attempt before that;
- the ways forward,
  which are the same commands the cancellation report names,
  and the fact that `doctor` shows the state until one runs.

The removal exits `1`.
The report reads,
with the fixture's names and its own numbers:

```text
removal of `.claude/worktrees/one` did not finish within its deadline. The last observation was: no result for this
request was filed. No destruction plan was written, so nothing had been destroyed when the wait last looked. The
request is operation `unmount` under nonce 3f2a... for dataset GUID 12345678901234567890, and it stays on the clone.
A deadline is not cancellation: the request may still act. With a holder present it retries at each trigger and the
mount moves out and back; when the holder leaves, an attempt may complete the unmount, and nothing continues the
removal after that.
Ways forward: run `git worktree remove .claude/worktrees/one` again to resume it, run
`git monochromatic-git-policy-cli cancel-worktree-removal one` to end it, and run
`git monochromatic-git-policy-cli doctor` to read the state until one of them runs.
```

### How the worktree returns to use

There are five ways,
and each names the state it needs:

- Run the removal again.
  It takes the claim,
  reads the plan,
  and resumes:
  it finishes the unmount where the folder and the intent still stand,
  settles where the plan says a chunk was submitted,
  and destroys the head where nothing blocks it.
- Run `cancel-worktree-removal <slot>`.
  It reverses where the head is present and every destroy outcome is settled,
  which covers every point up to the head's destroy.
- Run `cancel-worktree-removal <slot>` after a deferred-destroy refusal.
  That refusal keeps the intent and the plan ([deferred][deferred],
  "What a refusal leaves"),
  so the state is one of points 13 and 16 and the command's proof accepts it:
  it rewrites the intent,
  attaches,
  and reports that nothing was destroyed or which tuples were.
- Wait for a trigger,
  then run the removal again or the cancel command.
  A completed unmount changes the cancel command's work from moving a mount back to making one,
  and changes nothing else.
- Nothing returns the worktree where the head is gone.
  The removal must be run again to settle the registration and the origin cleanup,
  and the cancel command refuses and says so.
## The cancel command

### The grammar

The project's words fix the command's name and its one argument:
"a guarded `cancel-worktree-removal <slot>`" ([record][decision],
lines 41 to 44).
Where the command lives,
and that its argument is a slot name rather than a path,
are this design's proposal,
marked for the project's veto.
It is proposed as a management subcommand:

```text
git [<git-global-options>] monochromatic-git-policy-cli cancel-worktree-removal <slot>
```

The accepted management grammar lists `check`,
`fix`,
`doctor`,
`kept-references` and the retired forms,
and does not hold this command ([spec][spec],
"Management grammar",
lines 608 to 618),
so a build adds it there with this design's sentences.
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
Each step names its refusal and its exit.

1.  The slot is a plain slot name,
    by the same grammar the attach path uses (`slot_name.rs:24`).
    A name that is not one is the caller's error and exits `2`.
2.  The slot's `.git` pointer is read with the bounded no-follow reader
    (`clone_slot_git_pointer.rs:231`) and must name `<common>/worktrees/<id>` of the caller's repository.
    A pointer to another checkout means the caller named a slot of another repository and exits `2`;
    a slot with no pointer,
    and a pointer whose bytes are not a single `gitdir:` line,
    are damaged states and exit `1`.
3.  The binding record (A1) must name this slot and the clone's dataset,
    GUID and origin.
    A missing record,
    a record naming another slot,
    and one whose clone has a different GUID from the plan's are each refused with exit `1`,
    and the refusal says the administrator's adoption is the step that repairs a damaged binding.
4.  At least one removal record must exist:
    a plan,
    a staged-folder record,
    or a local `unmount` intent.
    With none,
    the command refuses with exit `2` and says the slot is not being removed,
    because a person who typed another slot's name must not be told a removal was reversed,
    and the slot they named is simply not the one they meant.
    A pending add is that state and not a removal:
    its intent asks for a mount,
    it has no plan and no staged-folder record,
    and its claim is the add's own (the add design's markers).
    The command therefore refuses it with exit `2` before it takes the removal claim,
    so no claim is taken at all,
    and names the add's own removal command as the way forward;
    it never removes a folder or a clone a pending add holds.
    Where a slot's markers cannot be told from a pending add's,
    the refusal is the same either way,
    and only the sentence naming that command depends on the marker.
5.  The removal claim for the clone's GUID is taken (F16).
    A live owner is refused with exit `1`,
    naming its process identity,
    because a state prevents the command and waiting or stopping that removal is what a person must do;
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
    the plan's `owner`,
    where a plan exists,
    and the removal claim's owner must name one process (A10),
    and the intent must read as a local request.
    A disagreement between any two records is a refusal,
    and never a choice between them.
7.  The clone must be live and its identity must match:
    the dataset name from the records exists,
    answers with the recorded GUID,
    and names the recorded origin.
    A missing dataset is the head-gone case under "Settling the destroy outcomes before anything moves",
    not this one.

Two claims stand behind that proof,
and the accepted sentence's "under the per-slot claim" is read here as naming both.
The root handler's per-slot claim is taken by every handler invocation that acts on the slot,
so the mount request the command writes is decided under it,
and the wrapper neither takes it nor can (F15).
The wrapper's removal claim is taken first and held through the proof,
the settlement,
the request and the report,
so two wrapper operations on one clone cannot interleave.
A reading that made the root claim the wrapper's own would need a way into a root-owned folder that nothing here has,
and the reading is marked for the project's veto.

### Settling the destroy outcomes before anything moves

The command reads the plan through the destruction-plan design's own reader,
refuses whatever that reader refuses,
and settles the tuples by that design's absence rule ([plan-record][plan-record],
"The absence rule" and "The cancel command").
What this design adds is the decision that follows:
whether the state is one an attach may follow.
Its parts are:

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
- A tuple the plan marks `unknown` is refused for the same reason,
  because an unknown outcome is one the design may not attach over.

The confirmed destroyed set is what the report names.
Nothing in this part writes to the pool,
and nothing in this part writes to the plan either:
the plan design's rule is that "a settle is not an outcome",
and the plan keeps exactly what the removal wrote.

### The act: one mount intent and one answer

With the proof done and the outcomes settled,
the command first reads the intent it found.
Where the intent already reads as a `mount` request,
no live removal claim names the clone,
and the clone stands at its slot,
the command writes no second request,
reports that the removal was already cancelled,
and exits `0`.
In every other state it:

1.  writes `v1:<fresh nonce>:mount` on the clone through the existing intent writer,
    whose gate runs first.
    The nonce is fresh rather than the removal's,
    because the proposal's own rule is that "no receipt is reused for a later one" ([proposal][proposal],
    lines 587 to 595),
    and a receipt is matched by operation,
    nonce and boot;
2.  waits for a receipt that names that operation,
    that nonce and this boot,
    for the removal deadline;
    a receipt from another nonce,
    another operation or another boot is not the answer,
    which is the wait's own rule;
3.  on `done`,
    reads the receipt's slot,
    source and mount identity and requires them to be the clone's at its slot;
    where the receipt says the bind is gone,
    it removes the folder by `rmdir`,
    unlinks the record and releases the staging claim;
    where the bind stands,
    it retains and reports all three;
4.  on `refused` or `failed`,
    reports the receipt's failed step and errno,
    keeps every record,
    and exits `1`;
5.  on a timeout,
    reports what the wait last saw,
    keeps the plan as it stands,
    and exits `1`,
    because the request stands and a later trigger answers it;
6.  releases the removal claim on every path out,
    including the paths that refuse.

The plan design's one rule at `3cd520b6a0` is that both the removal that cancels itself and this command remove a
plan whose every tuple is `planned` and whose head is `present`,
and that neither removes a plan that records a destroyed snapshot
([plan-record][plan-record],
"A cancellation,
and the plan it leaves").
So that removal is part of this act:
the command proves the plan is in that state,
removes it before it releases the claim,
and removes no other plan.
Every automatic arm removes such a plan at the same point,
which the plan design's rule gives the removal that cancels itself.

After the answer,
the command also removes a destination directory a forced removal's way 7 left,
when it carries the reserved prefix and the clone's GUID,
is empty,
is root-owned with mode `0700`,
and sits directly under the parent mount's root;
otherwise it reports the directory and removes nothing ([forced][forced],
"V7.
Whether the report,
`doctor` and the next run name a changed propagation and a leftover directory").

The command never forces an unmount,
never detaches a mount,
never ends a holder,
never issues `zfs mount`,
never mounts by path,
never removes a registration and never destroys a snapshot.
It asks for one attach and reports what the handler answered.
A removal that was run with `--force` is reversible by the same command on the same proof,
which is the question on cancelling a forced removal.

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
folder is gone. The plan keeps the removal's own outcomes, the intent now asks for the mount, and
`git monochromatic-git-policy-cli doctor` reads both.
```

### What it refuses

The refusals are the design's guard,
and each names the state it found and the step that repairs it:

- a name that is not a slot name,
  with exit `2`;
- a pointer to another checkout or slot,
  with exit `2`;
- a slot with no pointer,
  or a pointer that is not one `gitdir:` line,
  with exit `1`;
- no removal record at all,
  with exit `2`;
- a directory a forced removal left that fails one of the checks above (its name,
  emptiness,
  owner,
  mode or place):
  it is reported and left,
  with exit `0` for a run that attached,
- a pending add,
  which is the same state and whose own removal command is the way forward,
  with exit `2`;
- a missing or mismatched binding record,
  with exit `1`;
- a live removal claim,
  naming its owner,
  with exit `1`;
- two records that disagree about the clone,
  the GUID,
  the slot or the owner,
  with exit `1`;
- an intent that reads as malformed or refused,
  with exit `1`;
- a clone whose live GUID or origin is not the recorded one,
  with exit `1`;
- an inventory that cannot be read completely,
  with exit `1`;
- an unknown destroy outcome,
  with exit `1`;
- the head gone,
  which names the removal's settle step instead of an attach,
  with exit `1`;
- a staged state the receipt says still holds a bind,
  or a staged folder the command cannot prove it may remove,
  with exit `1`;
- a handler answer that refuses or fails,
  with the receipt's step and errno,
  and exit `1`;
- a mount request whose receipt never arrives,
  at the deadline,
  with exit `1`.

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
settled. `...@sanoid-hourly-2026-11-22_03` is marked requested and is not in the clone's live snapshot list, and no
child of `zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets` carries GUID 12345678901234567890, so the
head's state is unknown. Nothing was mounted and nothing was changed. Ways forward: run
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
`1` for every state that prevents the command,
which is the operation failing:
a damaged pointer or binding,
a live removal,
two records that disagree,
a malformed intent,
a mismatched clone,
an unreadable inventory,
an unknown outcome,
the head gone,
a retained bind,
a handler refusal,
and a timeout.
`2` for the caller's mistakes:
a name that is not a slot name,
a slot of another repository,
and a slot that is not being removed.
Standard output carries nothing,
reports and refusals go to standard error,
and the log lines go through the wrapper's tagged loggers.

## Cancellation by itself and by the command

What the two share is the whole mechanism:

- one `mount` intent with a fresh nonce,
  written through the same writer and gate,
  by one place,
  because the forced arm never rewrites the intent itself;
- one wait for one receipt under that nonce,
  on the same matching rules and the same deadline;
- one handler path,
  which is the ordinary attach answer plus the staged move-back where a staged mount stands,
  and the fresh attach where the clone is mounted nowhere;
- no write to the destruction plan:
  both leave it as the removal wrote it,
  and its destroyed tuples are the loss record;
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
  settles the destroyed set through the plan design's reader,
  and its outcome is exit `0` where it reversed.

When both act on one slot,
the removal claim decides:

- While a removal's claim has a live owner,
  the command refuses and names that owner.
  A person who wants to stop a running removal stops it,
  and then the claim's owner is dead.
- The automatic arm cannot run without its own process,
  so a killed removal never cancels itself;
  its request keeps being acted on at triggers,
  which is the state "Whether the standing request can act" states.
  Its claim has a dead owner and the command takes it.
- Two command runs do not interleave,
  and the second finds the intent already `mount`,
  no live claim on the clone,
  and the clone at its slot,
  and reports that and exits `0`.
- A `mount` intent written while a sweep is holding the root claim is not lost,
  because the sweep re-reads the intent after it releases the claim ([step-8][step-8],
  "A slot whose claim is held").
  That sweep's own answer to the earlier `unmount` request may still be a completed unmount,
  and then the `mount` request it reads next makes the clone again,
  so the state converges on the clone being at its slot with a `mount` request standing.
- A removal run with `--force` does not cancel itself at a blocked unmount or a refused head destroy,
  because the answer's promise is that the holders are ended.
  Where the forced arm cannot end a holder,
  the failure path under "What a removal that cancels itself restores" is
  the state both designs refer to,
  and the command is the way back from it.

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
  and the handler never reads it at all,
  which is this design's own statement by analogy with the accepted staging claim,
  not an accepted record;
- its generations are the shared claim's,
  and its record is the shared record:
  the owner's process id,
  birth identity and PID namespace,
  then a released line.

It records no nonce and no phase,
because the shared claim's record has no field for one,
the plan carries the run's nonce and the intent carries the cancellation's,
and the plan's `owner` names the same process (A10).
The alternative was to widen the staging claim's lifetime,
which the accepted staging design fixes as the folder's,
and that alternative is marked for the project's veto.
The claim is not the root handler's per-slot claim,
which only root can take (F15).

The claim's existence is also the wrapper-side mark that an operation is working on that clone.
`doctor` reads it and prints its owner,
live or dead,
beside the plan's owner.

### What a cancellation leaves in the destruction plan

No field,
and at most one unlink.
The plan is the destruction-plan design's record,
and this design writes none of its fields:

- its outcome words are `requested`,
  `destroyed`,
  `refused`,
  `intact` and `unknown`,
  and its head's are `present`,
  `requested`,
  `gone`,
  `refused` and `unknown`,
  with no value for a cancellation ([plan-record][plan-record],
  "Field by field" and "The states");
- its own finish pass writes nothing to it,
  because "a settle is not an outcome" ([plan-record][plan-record],
  "The finish pass that settles a registration");
- the plan design's one rule for a cancellation is that both the removal that cancels itself and the cancel command
  remove a plan whose every tuple is `planned` and whose head is `present`,
  and that neither removes a plan that records a destroyed snapshot
  ([plan-record][plan-record],
  "A cancellation,
  and the plan it leaves",
  read at `3cd520b6a0`);
- a later removal of the same clone rewrites it whole,
  keeping every tuple and outcome
  ([plan-record][plan-record],
  "The removal intent").

So a cancellation leaves a plan that records a destroyed snapshot as the removal last wrote it,
and the loss is the tuples it marks `destroyed`,
with their names and GUIDs.
Where nothing was destroyed it removes the empty plan,
as that design states.
`doctor` reads the cancellation from three facts beside a kept plan:
the intent asks for a mount,
the plan's `owner` is dead,
and the clone is present.
That is the whole of the loss record,
and no field is added to the plan for it.

### The plan's lifetime, what each actor does in each state, and what a retry finds

The project decided the lifetime on 2026-11-22 ([records-record][records-record],
"How long the plan a cancellation keeps stays",
lines 90 to 101):

> The next removal of that clone reads the record,
> proves it,
> carries the loss forward and ends it.

So the plan lives until that clone's next removal,
and this design's actors leave it alone in every state but one.
By state:

- Nothing destroyed yet and no plan written (points 1 to 10):
  the self-cancelling removal writes no plan,
  and there is nothing to keep,
  so its next removal starts fresh.
- Nothing destroyed yet and a plan written with every tuple `planned` and `head=present` (point 11,
  and the deferred refusal of point 13):
  the plan design's one rule is that both the removal that cancels itself and the cancel command remove it,
  after proving the clone and the plan and before releasing the claim
  ([plan-record][plan-record],
  "A cancellation,
  and the plan it leaves",
  read at `3cd520b6a0`).
  Both actors here are this design's,
  so this design removes it in the automatic arm and in the command,
  writes no outcome,
  and never removes a plan that records a destroyed snapshot.
  A cancellation killed before that unlink leaves the plan to the next removal,
  which adopts it (the state below).
- Some destroyed (points 14 to 18):
  the plan keeps every outcome and the head's state,
  the self-cancelling removal writes nothing to it,
  and the cancel command writes nothing and removes nothing;
  it reports the destroyed set.
  The plan is the loss record,
  and nothing removes it at the cancellation.
- The clone's next removal finds the intent `mount`,
  a dead owner in the plan,
  and the clone present.
  It reads the plan,
  proves it against the live clone,
  adopts it as its own under its own nonce,
  keeps every tuple and outcome,
  carries the loss forward,
  and ends it at the plan design's step 13 after proving the remaining set and the head gone
  ([plan-record][plan-record],
  "A cancellation,
  and the plan it leaves",
  and the record's own words above).
  The plan design states the first half of that as its own rule:
  "An intent that reads `mount` while the head still stands is not an absence:
  it is the reattach of a removal that cancelled itself,
  so a plan beside it is adopted on the head-present path under the nonce rule above,
  and every tuple and outcome it holds is kept".

A retry after a cancel is one of four states,
not one,
and the deferred-destroy design's early look runs in only the first of them,
because the look "runs only when the clone carries no removal intent and no plan exists for it"
([deferred][deferred],
"The look before the unmount").
The plan design's text at `3cd520b6a0` gives each state's rule,
and where it does not,
this design says so and asks it.
State by state:

- No plan is kept.
  That is every cancellation before the plan's first write,
  and a cancellation that destroyed nothing,
  because the plan design's one rule has both actors remove the all-`planned` plan.
  The retry finds the intent `mount` and no plan for the clone.
  It is a first run:
  the look runs,
  and nothing about the cancellation refuses it.
  A mark on the origin is refused by the look as it would be on any first run,
  and the person is told what that first run says.
- A plan is kept with nothing destroyed,
  because the cancellation was killed before it removed the all-`planned` plan.
  The retry finds the intent `mount` and the plan beside it.
  The plan design's rule is that an intent reading `mount` while the head stands "is not an absence ... so a plan beside
  it is adopted on the head-present path under the nonce rule above,
  and every tuple and outcome it holds is kept"
  ([plan-record][plan-record],
  "The removal intent").
  No check refuses it,
  the look does not run because a plan exists,
  and the person is told nothing new:
  the run proceeds with a plan that witnessed nothing.
- A plan is kept with some destroyed.
  The same adoption:
  the retry re-proves everything,
  adopts the plan under its own nonce,
  keeps every tuple and outcome,
  carries the loss forward,
  and proceeds to the first deferred check.
  The look does not run.
  No check refuses it,
  and any later refusal carries the plan's history,
  which the plan design's words decide.
- A plan is kept with an `unknown` outcome.
  The plan design adopts a plan only "if the plan has no `unknown` outcome and its identities still match live";
  otherwise "it refuses,
  and the cancel command is the way forward" ([plan-record][plan-record],
  "The removal intent").
  So the retry refuses at the plan's adoption rule,
  before any destructive step,
  and the look does not run because a plan exists.
  The person is told that the plan records tuples whose outcome is unknown,
  that none of them may be assumed either way,
  and that the removal refuses until they are settled.
  The plan design's stated way forward is the cancel command,
  and for a plan a cancellation kept that command has already run;
  its settling command is stated only for a plan whose clone is gone:
  so this design asks the plan design,
  in words it can quote,
  "what settles a plan's `unknown` tuple while its head stands and its clone is present,
  and what the next removal prints until then".

The look's own answer,
where it runs,
needs no qualification:
a retry after a cancel that kept no plan is a first run.

That design's requirement has been reworded since this design first answered it,
so this design answers the wording at `d0ff7604b7`:
its "A retry after a cancel" now reads that "a retry after a cancel looks only when the cancellation removed its plan"
and that "a cancellation that kept a plan ... gives a retry that does not look".
In those words:
"a retry after a cancel that kept a plan presents it,
so the look does not run,
and the run's first refusal carries the history its plan holds;
a retry after a cancel that kept no plan is a first run and the look runs with no change".
This design confirms that the look does not run over a kept plan,
and states that its own earlier premise,
"a retry after a cancel always presents a plan",
is false for the states with no plan,
which the deferred design's section now also says.

What this design needs of the plan design,
and does not define:
the all-`planned` plan's fate at a cancellation must be one action in both of its sections,
and that action must name which actor does it after a self-cancellation,
where no command runs;
the kept plan's lifetime is the record's,
not this design's;
its reader must accept the state this design leaves,
which is a plan whose owner is dead beside an intent that asks for a mount;
and an `unknown` tuple of a plan whose head stands and whose clone is present needs a stated settling step,
because the adoption rule refuses it and the cancel command has already run.

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
The forced arm adds no key:
its refusals use the free `failed-step` word,
which already fits the receipt's 64-byte bound,
and a `caller-protected-holds` step is one such word.
The two changes are compatible,
because the free word and the new key are different fields,
and the step 7 receipt contract is what must accept both.
Nothing of the ZFS route is released,
so no written receipt needs migrating.
The alternatives are marked for the project's veto.

### One reversing request, written in one place

The intent is the account's property,
and one place must write it or two designs will disagree about the nonce.
This design writes it:
the automatic arm,
the forced failure path and the command all send `v1:<fresh nonce>:mount` through
`clone_mount_intent_write.rs`.
The forced arm never writes the intent itself;
it produces the refusal and the record of who was ended,
and this design produces the restored state.
That split is the ownership the two documents are given,
and the fresh nonce is the part of it the proposal's rule decides.

An add that withdraws its own unmount request writes the same property back to the mounted state
([dirty-record][dirty-record],
lines 109 to 139).
It follows this rule in shape and not in ownership:
the add writes its own request under its own fresh nonce,
and this rule is about one removal's reversal,
so no two writers ever write one removal's reversal.
This design asks the add design to confirm that its withdrawal uses a fresh nonce and that the request is the add's
to write.
## Choices this design marks for the project's veto

Each entry is a choice the project's records do not decide,
made here so that the design is whole.
The project can veto any of them,
and each veto names the option that would replace it.
The questions that decide what a person's processes may suffer,
or that widen an accepted sentence,
are not here;
they are under "Questions only the project can answer".

- The cancellation states a request instead of clearing the intent.
  (a) This design writes `v1:<fresh nonce>:mount`.
  After a busy unmount the property reads `v1:9c41...:mount`,
  and the next reader of the clone sees a request for the state it is in.
  (b) The command inherits the property,
  as `zfs inherit` allows for a user property (F4,
  F5).
  The property then reads `-` with source `default`,
  and nothing asks for anything.
  (c) A third operation word,
  `cancel`.
  The property would read `v1:9c41...:cancel`,
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
  (b) over (c) because (b) needs no accepted sentence changed and (c) does.
- The reversing request carries a fresh nonce.
  (a) This design writes a new nonce for the `mount` request.
  The removal's unmount ran under `3f2a...`,
  and the cancellation writes `v1:9c41...:mount`,
  so the receipt under `9c41...` can only be the answer to the reversal.
  (b) The request reuses the removal's nonce.
  The same run writes `v1:3f2a...:mount`,
  and the receipt is told from the earlier one only by its operation field.
  Pros of (a):
  the proposal's own rule is that each scan is asked for "by writing a new `v1:<nonce>:unmount` intent ... so a receipt
  always names the scan that produced it and no receipt is reused for a later one" ([proposal][proposal],
  lines 587 to 595),
  and the wait matches on nonce and operation,
  so a fresh nonce keeps one request and one answer.
  Cons of (a):
  a reader of the property sees two nonces in one removal.
  Pros of (b):
  one nonce per removal,
  which reads simply.
  Cons of (b):
  it breaks the proposal's no-reuse sentence and makes the operation the only field that tells two answers apart.
  Ranking (a) > (b):
  (a) over (b) because the proposal decides the one-request-one-nonce rule,
  and the forced arm's text that reads as the removal's nonce is the text to correct,
  not the rule.
- The command's place in the grammar,
  and its argument as a slot name.
  (a) A management subcommand taking one slot name,
  as this design proposes.
  `git monochromatic-git-policy-cli cancel-worktree-removal one`.
  (b) A new Git subcommand of the wrapper's own.
  `git worktree cancel-removal one`,
  which is a name real Git does not have and the wrapper would have to add to its dispatch.
  (c) An option on the existing removal.
  `git worktree remove --cancel one`,
  which real Git's own option table refuses before the wrapper sees it as a cancellation,
  and which cannot be run when the removal is not running.
  Pros of (a):
  it sits beside `doctor` and `kept-references`,
  takes the same repository context,
  and needs no Git grammar of its own.
  Cons of (a):
  the accepted grammar's list must grow,
  which a build does.
  Pros of (b):
  it reads as a Git command.
  Cons of (b):
  it invents a Git subcommand,
  which the wrapper's rules otherwise avoid.
  Pros of (c):
  nothing new at the namespace level.
  Cons of (c):
  it collides with real Git's option table and could only cancel a removal that a second command is running.
  Ranking (a) > (c) > (b):
  (a) over (c) because (c) is refused by real Git before any wrapper code sees it;
  (c) over (b) because (c) adds no command name,
  while (b) invents one.
- The attach path answers a request for an already attached clone with `done`.
  (a) This design requires the attach path to check the mount table before its walk and answer `done` where the clone
  stands at its slot with its recorded identity (A5).
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
  the mount identity the receipt already carries says whether the clone is at its slot.
  Ranking (a) > (b) > (c):
  (a) over (b) because the namespace that holds the mount is the only one that can answer about it safely;
  (b) over (c) because (b) changes no record,
  while (c) changes one for a fact already present.
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
  so they cannot be from different moments,
  and the forced arm's free `failed-step` word is a different field,
  so both can land.
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
  it fails exactly where the wrapper's namespace is not the handler's (L7).
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
  and an already cancelled removal,
  are told apart.
  (a) This design refuses the first with exit `2` and reports the second with exit `0`.
  `cancel-worktree-removal typo` refuses,
  and `cancel-worktree-removal one` on an already cancelled removal prints its state and exits `0`.
  (b) Both refuse,
  the first with `2` and the second with `1`.
  The second run fails with a report saying the removal is already ended.
  (c) Both report and exit `0`.
  The typo's run reports success for a slot that was never removed.
  Pros of (a):
  a caller's retry is safe and a person's mistake is not reported as success.
  Cons of (a):
  two outcomes for two states a person may not distinguish without reading the report.
  Pros of (b):
  one rule for "nothing to do".
  Cons of (b):
  a run that achieved the state the person asked for fails.
  Pros of (c):
  one rule and retries always succeed.
  Cons of (c):
  a mistyped slot is reported as reversed.
  Ranking (a) > (c) > (b):
  (a) over (c) because the states differ in whether anything was ever being removed;
  (c) over (b) because a retry that finds the state it wants is not a failure.
- The guard's "per-slot claim" is read as two claims.
  (a) This design:
  the wrapper holds the removal claim for its own decisions,
  and the handler takes the root per-slot claim for every action it takes on the slot (F15).
  The command proves and settles under its claim,
  and the one attach it asks for is decided under the handler's claim.
  (b) The root claim alone,
  with the cancel command as a new handler mode.
  The wrapper would write a request that a root invocation answers by taking the claim,
  proving the slot and attaching,
  and the wrapper would hold no claim.
  (c) The wrapper's removal claim alone,
  on a reading that the sentence's "per-slot claim" is the wrapper's.
  The same commands run,
  and the handler's own claim is then only what every handler invocation takes anyway.
  Pros of (a):
  both decisions are serialized by the mechanism each side already has,
  and no new root mode is added.
  Cons of (a):
  two claims to explain,
  and a reader must see which side each one covers.
  Pros of (b):
  one claim for the whole act.
  Cons of (b):
  a new root mode that duplicates the wrapper's proof in root,
  and the wrapper would still need its own serialization against a removal.
  Pros of (c):
  one claim named.
  Cons of (c):
  it leaves the handler's action unnamed in the design's own words.
  Ranking (a) > (c) > (b):
  (a) over (c) because (a) states what decides each side where (c) leaves the handler's half unnamed;
  (c) over (b) because (c) needs no new root mode,
  while (b) moves the wrapper's proof into root and still leaves the removal unsynchronized.
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
- The cancellation reaches a holder a later scan sees,
  after a chunk.
  (a) This design cancels there,
  with the earlier chunks' tuples reported and the clone attached again.
  A holder that attaches during the batch makes the removal stop at the next scan,
  and the worktree returns with the chunks that went.
  (b) The removal goes on and lets the head destroy refuse,
  so the whole batch is destroyed before the cancellation.
  (c) The removal refuses and keeps the intent and the plan,
  and the cancel command is the way back.
  Pros of (a):
  it stops spending history at the first evidence and applies the restore the record decides for a loss.
  Cons of (a):
  the record's own trigger is the head destroy,
  so reaching it at a later scan is this design's extension of the answer.
  Pros of (b):
  nothing is added.
  Cons of (b):
  it spends the rest of the batch to learn what the scan already said.
  Pros of (c):
  nothing is lost and nothing is attached.
  Cons of (c):
  the worktree stays unmounted until a person runs a command.
  Ranking (a) > (c) > (b):
  (a) over (c) because the record's answer for a loss is to attach again and report,
  and (c) leaves that to a person;
  (c) over (b) because (b) destroys history the scan already revealed.
- The cancel command reverses a forced removal.
  (a) This design lets it,
  on the same proof as any removal.
  `git worktree remove --force one` killed after the handler ended two holders is reversed by
  `cancel-worktree-removal one`,
  the clone is attached again,
  and the holders stay ended.
  (b) It refuses,
  and the person can only finish the removal.
  The same person is told to run the removal again,
  which continues the destruction they began.
  Pros of (a):
  a killed or refused forced removal is recoverable without finishing the destruction.
  Cons of (a):
  a later run reverses a decision that ended holders.
  Pros of (b):
  the flag's decision is respected.
  Cons of (b):
  the command's purpose is lost where the removal was most destructive.
  Ranking (a) > (b):
  (a) over (b) because the accepted answer keeps the command for a removal that was killed or ran past its deadline
  and excludes no kind,
  and the force word dies with its writer (A8),
  so what the command reverses is a state and not a live forced request.
  The record calls this one of the designs' own choices inside its answers,
  listed for the project's veto.
- A cancellation's loss is no standing notice.
  (a) This design's choice:
  the report at the cancellation,
  the kept plan and `doctor`.
  A person who runs a commit in that worktree a week later sees nothing about the two destroyed snapshots.
  (b) A standing notice later wrapper commands in that worktree show until something clears it.
  The same person reads the two names at that commit.
  Pros of (a):
  no new notice,
  and the durable record is the one already kept.
  Cons of (a):
  a person who never runs `doctor` learns of the loss once.
  Pros of (b):
  the loss reaches the person where they work.
  Cons of (b):
  a notice with its own lifetime and clearing rule,
  which no record here designs.
  Ranking (a) > (b):
  (a) over (b) because a durable record and a place for it exist,
  while a standing notice needs a clearing rule that would be invented here.
  The record calls this one of the designs' own choices inside its answers,
  listed for the project's veto.

## Questions only the project can answer

None of this document's questions remains.
The gate review numbered one set for this document and its sibling,
S1 to S13,
and every one is now decided:
S1 to S7 and S12 by [the forced-removal record][forced-record],
S13 by its "One more answer:
a unit whose main process is a script",
and S8 and S11 by [the records record][records-record]
("How long the plan a cancellation keeps stays",
lines 90 to 101,
and "Whether a forced removal records who it ended,
beyond the journal",
lines 103 to 110).
The plan's lifetime is stated under "The plan's lifetime,
what each actor does in each state,
and what a retry finds".
What remains put to the project is the veto list above,
including the two choices the record calls the designs' own
(the cancel command may reverse a forced removal,
and a cancellation's loss is no standing notice).

## Which plan step builds which part

- Step 9 builds the busy unmount's reattach and the cancelled-removal receipt,
  and this design relies on both.
  It also builds the receipt's `bind` key with step 17,
  because the reattach is its first writer.
- Step 14 builds the removal claim and takes it before the plan,
  the folder and the intent,
  so that points 1 to 4 of "The stop points" are reachable in the order this design states,
  and it repeats every deciding read under the claim.
- Step 15 keeps the intent and the plan on every refusal ([deferred][deferred],
  "What a refusal leaves"),
  which is what points 13 and 16 record,
  and this design's cancel command treats such a state as reversible,
  with nothing destroyed or with the batch's destroyed set.
- Step 16 builds the fresh all-namespace scan before each chunk and before the head destroy,
  the chunked batch,
  the plan's per-name outcomes and head states,
  and the head destroy's two causes.
  It writes no cancellation state,
  because a cancellation leaves the plan as the removal wrote it.
- Step 17 builds the four cancellation arms,
  the forced failure path's restored state,
  the intent write that reverses the removal,
  the cancel command,
  and the attach path's idempotent answer (A5).
  It also builds the cancel command's act on the all-`planned` plan,
  whichever action the plan design settles,
  and its refusal of a pending add.
- Step 18 builds `doctor`'s lines for the intent,
  the claim and the plan's owners,
  the plan's outcomes,
  and the three facts that read as a cancellation,
  with the kept plan named until that clone's next removal reads and ends it.
- Step 22 measures the waits this design spends,
  which are the removal deadline and the attach's answer.

## Tests a build must have

### Device-free tests

These run with no pool,
no `/dev/zfs` and no guest,
using the stand-in device and the seams the route already has.

- The cancellation's writer sends exactly `v1:<32 hex>:mount` on the clone and nothing else,
  asserted against a stand-in device's recorded request,
  with a negative control that an `unmount` value is not sent and that a reused nonce is refused.
- The removal claim's folder,
  prefix and generation names,
  with a live owner refusing,
  a dead owner's claim replaced by the next generation,
  and a released claim read as released.
  The seam after the listing (`single_owner_claim.rs:362` to `:366`) forces the interleaving of two claimants without
  timing.
- The cancel command's grammar:
  no argument,
  two arguments,
  an option,
  an empty slot,
  and a bad slot name,
  each with its exit code and its stream,
  and each caller mistake at `2` while a state that prevents exits `1`.
- The pointer proof:
  a slot with no pointer,
  a pointer to another checkout,
  a pointer with two `gitdir:` lines and a pointer with trailing bytes,
  each refused with its own reason and its own exit.
- The guard's refusals,
  driven by fixtures of each record:
  a missing binding record,
  a mismatched binding record,
  no removal record,
  two records naming different GUIDs,
  two records naming different owners,
  an intent that reads as inherited,
  an intent that reads as malformed,
  a plan tuple whose name now carries another GUID,
  and an incomplete inventory answer.
- The receipt answer's matching:
  a receipt under another nonce,
  another operation and another boot are each not the answer,
  and the wait's timeout report says what it last saw and asserts no mount state.
- The order of the cancelling acts:
  a fault hook between the cancelled-removal receipt and the intent write asserts that the intent is written before
  the folder is settled,
  so a kill at that seam leaves the state C1 the design names.
- The report and refusal texts,
  each asserted whole,
  because each is text a person acts on,
  including the two the reports above give for an unknown outcome and a gone head.
- `doctor`'s lines over fixtures of every stop point 3 to 20 and of C1 to C4,
  with each line's fields asserted.

### Guest tests, with the kill points named

These run in the throwaway guest with a file-backed pool,
root for the handler and the account with only the proposed grants for the wrapper,
as the plan's own arms do.

- The arm blocked before the move,
  which is stop point 5.
  Hold the clone from a process of another session and run the removal without `--force`;
  assert nothing was destroyed,
  the mount never left its slot,
  the intent is reversed under a fresh nonce,
  the folder and its claim are settled,
  and the holder still runs.
- The arm blocked in staging,
  which is stop point 6.
  Hold the staged clone from a process outside the caller's session;
  assert the mount is back at the slot,
  the receipt says the clone is at its slot and the bind is gone,
  the wrapper's `mount` request is answered `done`,
  the folder,
  record and claims are gone,
  every snapshot survives,
  and the holder still runs.
- The namespace-holder arm,
  which is the case [VM item 15][vm-15] measured and the state of points 8,
  12 and 18.
  Create a reader namespace and a `PrivateMounts=yes` unit after the clone is attached,
  move and unmount the clone from the host namespace,
  and run the removal without `--force` so that its scan before the first snapshot delete sees both readers.
  Assert the removal cancels at that scan,
  before any delete:
  every snapshot survives,
  the plan holds no destroyed tuple,
  the fresh attach puts the clone at its slot,
  the report names the holder,
  and both readers still run.
  Two controls.
  The first is the same removal with no reader,
  whose batch and head destroy succeed.
  The second is the missed-holder control,
  a reader the scan cannot see (a namespace pinned by a descriptor with no task) or one attached after the scan,
  where the batch runs and the head destroy refuses `dataset is busy`:
  that arm asserts the cancellation at point 18,
  the destroyed set reported exactly,
  and the fresh attach,
  so an arm that cancels before the delete and an arm that cannot reach it both fail if they do the other.
- The later-scan arm,
  which is this design's extension of the record's answer and the veto entry that lists it.
  Run a removal whose first scan is clear,
  let it destroy one chunk,
  then attach a reader namespace.
  Assert the scan before the next chunk sees the reader,
  the removal cancels at point 15,
  the report names exactly the first chunk's snapshot,
  the clone is attached again at its slot,
  and the reader still runs.
  The control is the same run with no reader,
  whose batch completes and whose head destroy succeeds.
- The invisible-holder arms,
  which are the last three routes under "When a cancellation can report destroyed history".
  Each holds the clone in a way no mount table names:
  a lazily detached copy kept alive by a task's working directory in it,
  an unattached mount made by a helper of the arm's own with `open_tree` and `OPEN_TREE_CLONE` and held open,
  and a namespace the scan cannot read.
  Run the removal without `--force` and assert the scan names nothing,
  the chunks run,
  the head destroy refuses `dataset is busy`,
  the cancellation is point 18's,
  and the report says the holder is one no scan can name.
  The control is the reader namespace of the arm above,
  which the scan names and which cancels before the delete.
- The arm blocked by the scan after the unmount (point 8),
  which the accepted order reaches before any plan exists.
  Hold a copy in a second namespace,
  unmount and unbind the clone in the host namespace,
  and assert the scan's refusal,
  the cancellation with no plan written,
  the fresh attach,
  the folder settled on the receipt,
  and every snapshot alive.
  The control is the same state with no holder,
  whose clear receipt lets the removal go on to its plan.
- The forced failure arms,
  which are the states "The forced arm's failure path" describes,
  and they are four rather than one,
  split by whether the preflight could know the refusal by reading.
  (a) The protected set:
  hold the clone from the caller's own session and run with `--force`;
  assert the refusal `caller-protected-holds`,
  nothing moved,
  ended or destroyed,
  and the caller's session and its jobs live.
  (b) A namespace the preflight cannot enter:
  an anonymous mount namespace is not added to the namespace tree,
  so a scan never meets one and the arm injects the refusal at a seam instead,
  as the claim controls do:
  the scan reports the namespace as unenterable and the preflight refuses.
  The reason a real `setns` would give is the explicit anonymous-namespace test (Linux `v7.3`,
  `fs/namespace.c:6717` to `:6718`),
  not a missing owning user namespace,
  which every namespace has.
  Assert the refusal before any act,
  nothing ended,
  nothing destroyed,
  the intent reversed under a fresh nonce,
  and the report naming the namespace and the errno.
  (c) An unmount that turns out busy after a clear preflight:
  make a mount appear under the clone after the preflight (a helper of the arm's own that mounts a tmpfs there at
  the injected seam),
  and assert the refusal,
  that no process of the run was ended,
  nothing destroyed,
  and the report naming the mount.
  (d) A blocker it cannot end:
  a task in uninterruptible sleep needs a file system that can hold one,
  so the arm uses a FUSE mount of the arm's own inside the guest,
  or an NFS mount whose server has stopped;
  where neither can be made in the guest,
  the arm is not reproducible there,
  and what is asserted instead is that the report names the blocker and the ended set separately,
  with a task that
  ignores `SIGKILL` replaced by a task of another account the handler's authority does not reach.
  Assert the refusal,
  the tasks it did end still ended and named,
  the blocker named,
  nothing destroyed,
  and the exit `1`.
  Each control is the same state under a task the arm can end,
  whose removal goes on to the batch.
- The service-manager parent arm:
  run `--force` from a command whose session was started directly by the service manager,
  with the command's own parent a script,
  which is the shape the record's later answer catches
  (a `systemd-run --user --unit` child whose main process is a shell script that runs the removal,
  with the unit the arm starts and stops itself);
  assert the refusal before the first durable write,
  with no claim,
  no folder,
  no intent and no plan written,
  and the words naming the session as what the manager started.
  Then the parent case alone:
  a command whose parent is the manager,
  refused with the same state.
  Then the controls:
  the same command run as a child of a shell,
  which proceeds;
  a console login's shell,
  refused with the orphan sentence;
  and an agent's command,
  which proceeds.
- The deferred refusal arm,
  which is points 13 and 16.
  Mark the origin and run the removal;
  assert it refuses before the first delete with the intent and the plan kept,
  that `cancel-worktree-removal` accepts that state,
  reverses it,
  and reports that nothing was destroyed;
  then the same arm at the check before the head destroy,
  where it reports the batch's destroyed names.
- The cancel command at each kill point,
  forced by a step-indexed fault hook that ends the process at that step and not by timing:
  stop points 3,
  4,
  5,
  7,
  8,
  9,
  10,
  12 and 17,
  and the cancelling states C1,
  C2 and C3.
  Each arm asserts the command's proof,
  its settlement,
  its report and the end state,
  and each carries a control arm in the same run where the process is not ended,
  so an arm that reverses nothing fails.
- The deadline arm:
  a handler held past the removal deadline by a test-only delay seam,
  then released,
  with the deadline itself injectable so the arm does not wait out the shipped value.
  Assert the removal's timeout report,
  exit `1`,
  that the intent still stands,
  that the report asserts no mount state,
  and that a later trigger acts on the request,
  with a control arm that is not delayed.
- The head-gone refusal:
  a removal taken past the head's destroy in the control arm,
  then the command run.
  Assert it refuses with exit `1`,
  mounts nothing,
  and names the removal's settle step;
  the control is the same command before the head's destroy,
  which attaches.
- The unknown-outcome arm:
  a plan with one tuple `requested` and absent and one `requested` and present after a killed chunk.
  Assert both settle,
  that the report names the absent one as destroyed,
  and that the present one is not named;
  the control is a chunk that answered,
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
- The clearing of the mounted clone answered as absent (A7):
  an `unmount` request for a clone mounted nowhere,
  asserting a `done` receipt with no mount identity and no bind move.
  This is the arm the review's unlisted assumption needs,
  and it is M10.

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
  This sizes the cancellation's wait for the busy case,
  and an absolute time is context and not a finding.
- M2.
  The fresh attach of an unmounted clone.
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
  Assert the request succeeds and that the read reports no local value;
  the control is a set of the same property by the same account,
  which must succeed.
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
  a repeated move out and back while a holder stays,
  and a refusal where no folder stands.
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
  because a request that goes through the system manager can put a password dialog
  on the person's desktop and wait on that person,
  and the local polkit rules are unreadable from this account.
  It is the one arm the forced arm's own M8 also owes,
  and the two design documents share it.
- M10.
  An `unmount` request for a clone mounted nowhere.
  Against the built handler,
  request the unmount of a clone that is mounted nowhere and assert the answer is `done`
  with no mount identity,
  no bind,
  and no move.
  The control is the same request for a mounted clone,
  which moves it.
  This is the arm A7 needs.
- M11.
  The scan's coverage of a mount namespace with no visible task.
  In the throwaway guest,
  pin a mount namespace with an open namespace file handle and no task in it,
  mount a copy of a mounted clone inside it,
  and run the removal's scan.
  Assert what the scan reports:
  the namespace,
  nothing,
  or a refusal.
  The control is a namespace with a live representative task,
  which F18's item saw named.
  Record the answer beside the code that reads the scan,
  because the project's answer makes a seen holder cancel,
  and the destroyed-set routes under "When a cancellation can report destroyed history" depend on what the scan cannot
  see.
  This is not probed on the host:
  it mounts inside a namespace of its own in the guest.
  Repeat it with the two invisibles that are not namespaces:
  a lazily detached copy kept alive by a working directory,
  and an unattached mount held by a descriptor made with `open_tree` and `OPEN_TREE_CLONE`,
  and record whether the scan names each one.
## Dependencies on the sibling designs

Each dependency is stated with the assumption this design made,
so a later pass can reconcile the two.
Every sibling document was read at its branch tip,
and each is unreviewed:
nothing below adopts a sibling's answer as decided.

- The add's binding record (A1):
  this design's slot-to-clone proof reads it,
  and it must be readable with no mount and no root.
  The sibling design now exists,
  and this design read its home and format;
  if that home cannot be read by an unprivileged wrapper,
  the proof needs another durable slot-to-clone record.
- The destruction plan (A2,
  A10):
  this design reads the plan's grammar,
  its absence rule,
  its tuple outcomes and its `owner`,
  and writes none of them.
  The project decided its lifetime ([records-record][records-record],
  lines 90 to 101):
  the next removal of that clone reads the record,
  proves it,
  carries the loss forward and ends it.
  What this design needs of that design,
  and states under "The plan's lifetime,
  what each actor does in each state,
  and what a retry finds":
  a plan that records a destroyed snapshot is never removed at a cancellation;
  the all-`planned` plan's fate at a cancellation is one action in its two sections,
  which disagree today;
  the record is readable while the clone is unmounted and after a reboot;
  and its `owner` names the same process the removal claim names.
  Its own list of end states has no cancelled value,
  and this design needs none.
  The deferred-destroy design's requirement on a retry after a cancel
  ([deferred][deferred],
  "A retry after a cancel") is answered there in the words it asked for.
- The add-time Git lock:
  this design's unmounted window between a kill and the next command relies on that lock keeping `git worktree prune`
  from deleting the registration (F3).
  If that lock is not written before the removal can run,
  this design's points 8 to 18 have an unprotected window and the cancel command must report it.
- The deferred-destroy checks (A9):
  that design keeps the intent and the plan on every refusal and rewrites neither,
  so points 13 and 16 are its states and this design's command is the way back from them.
  This design does not decide whether such a refusal should cancel instead;
  it states what its own states do with the kept refusal,
  which is to leave it reversible.
- The forced arm:
  with `--force` the removal ends holders instead of cancelling,
  and this design's automatic arms do not run for a forced removal.
  The project decided the reach,
  the ways out of a shared namespace,
  the signal,
  the protection and the session case on 2026-11-22 ([forced-record][forced-record],
  and its "One more answer"):
  the handler unmounts the clone inside the holder's namespace and ends only what blocks that unmount,
  including a namespace whose mounts are shared,
  through the ways its design now states ([forced][forced],
  "The ways out of a namespace whose mounts are shared"),
  still to be measured;
  a namespace it cannot enter,
  a structural check,
  an unendable holder or a blind read refuses at its preflight,
  and only a blocker that will not die and an `EBUSY` after an empty search follow an ended process;
  the signal is `SIGTERM`,
  a wait,
  then `SIGKILL`;
  the protected set is the caller's session plus its ancestors;
  and `--force` is refused for a command whose parent or whose session was started by the service manager.
  Its preflight is in its design ([forced][forced],
  "The preflight"),
  and it reports one line per refused namespace.
  The failure path under "What a removal that cancels itself restores" is this design's,
  and it needs five things from the forced arm:
  that preflight;
  the pid and start time of the writer in its force property (A8);
  its refusal kinds and their words,
  `caller-protected-holds`,
  `holder-unendable` and `holder-unseen`;
  its record of which holders it ended;
  and that it never rewrites the intent itself.
  The scan before the move at point 5 is that design's V4,
  a writer's choice there,
  and the earliest cancellation this design states depends on it.
- The lost-last-event triggers:
  the events this design's reconciliation relies on are that design's,
  and this design adds none.
  The repeated attempts a standing request makes at each trigger are bounded by that design's coalescing rule.
- The origin-cleanup record:
  the head-gone refusal points at the removal's settle step,
  which writes that record.
  This design neither reads nor writes it.

## Where this design and a sibling design disagree

The gate review found places where the two step 17 documents each assumed the other settled a state.
This revision makes the split explicit,
and this section is where a later pass reconciles them.

- The reversing request's nonce.
  The forced arm now leaves the nonce to this document,
  saying that the request that reverses the intent belongs to the sibling ([forced][forced],
  "The refusal,
  and what it leaves"),
  and this design writes it under a fresh one (the second veto entry).
  The proposal decides it:
  each scan is asked for "by writing a new `v1:<nonce>:unmount` intent ... so a receipt always names the scan that
  produced it and no receipt is reused for a later one" ([proposal][proposal],
  lines 587 to 595).
  The two documents agree.
- The forced removal's failure path.
  The forced arm's D2 leaves the restored state and the records to this document and asks it for nothing it lacks
  ([forced][forced],
  "Dependencies on sibling designs");
  it keeps only its own preflight,
  its three refusal words `caller-protected-holds`,
  `holder-unendable` and `holder-unseen`,
  and the fact that it never writes the intent.
  This design's "The forced arm's failure path" gives the restored state for the preflight's refusals,
  which leave nothing ended,
  for the structural stage's refusal,
  which leaves nothing ended and earlier namespaces changed,
  and for the two that follow an act,
  which name the ended set.
  The split is the ownership the two documents were given:
  this design owns the restored state and the record,
  and the forced arm owns the ending of holders,
  the refusals and their words.
- A forced request that outlives its wrapper.
  The forced arm now bounds the force word by the wrapper's pid,
  start time and recorded parent and by a held pidfd,
  checked before every signal and every unmount,
  and it does not use the removal's deadline ([forced][forced],
  "The time between the scan and the ending").
  While the act runs,
  its "The act's liveness" has the handler write a progress file and the wrapper read its counter,
  each with an interval that is a constant with no value until M12,
  and the wrapper gives up on a stalled counter rather than staying alive without limit.
  This design's states cite that rule under "What a timed-out or killed removal leaves",
  and its cancel command reads the `caller-gone` answer when it meets a dead wrapper's act.
  This design bounds the whole request by the claim's owner and the plan's owner,
  and by the intent that a cancellation rewrites.
  The two agree,
  and the project decided the bound itself ([forced-record][forced-record],
  "How long a forced request's authority lasts").
  This design's states assume it (A8),
  and without it a later trigger could end holders with nobody waiting.
- A holder in the protected set without `--force`.
  The forced arm's text calls that holder's stop a cancellation;
  this design calls it a refusal with `caller-protected-holds`,
  because the never-end set's own words for that holder are a refusal
  ("the removal refuses and says to leave the folder and run the command again",
  [record][decision],
  lines 49 to 54).
  The restored state is the same in both,
  and this design keeps the refusal's words.
- The receipt contract.
  The forced arm's refusal words `caller-protected-holds`,
  `holder-unendable` and `holder-unseen` ride in the receipt's free
  `failed-step` field,
  which fits its 64-byte bound;
  this design adds one `bind` key,
  last.
  The two are compatible,
  because the free step word and the new key are different fields,
  but both change the same record's contract and the step 7 owner must accept both together.
  This design's veto list carries its key;
  the forced arm's words are its own.
- The kept plan's one rule.
  The plan design at `3cd520b6a0` states one rule for a cancellation in both of its sections,
  so the disagreement this design reported between its "The cancel command" and "A cancellation,
  and the plan it leaves" is settled:
  both the cancelling removal and the cancel command remove an all-`planned`,
  `head=present` plan and remove nothing else.
  This design follows it in its automatic arms and in the command,
  and asks that design for two things its text does not yet give:
  what settles a plan's `unknown` tuple while its head stands and its clone is present,
  and what the next removal prints until then.
- The pre-move scan.
  The forced arm's V4 adds a scan before the handler's move,
  adopted there as "every scan under the force word";
  this design's point 5 is that state,
  so the two agree and this design needed no new transport for it.
- The duplicate guest arm.
  The forced arm's M8 and this design's M9 are the same arm,
  what an unprivileged signal answers.
  This design keeps M9 and states that the two documents owe it once.
- The in-flight step 9 work.
  This design builds on three things that round has not shipped:
  the cancelled-removal receipt kind,
  an attach path that answers an already attached clone with `done` (A5),
  and the `bind` key.
  It treats them as assumptions and build items,
  not as decided.
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
  The plan's `owner` beside it is the same test on the record the loss lives in.
- The pointer's bounded no-follow read and the receipt's ownership and mode checks exist only for a hostile account,
  and they also catch an ordinary surprise file in the slot.
- The staged-folder identity re-check before the folder's `rmdir` exists only for a hostile account,
  and it also catches a folder the account's own cleanup replaced.
- The requirement that the plan's owner and the removal claim's owner agree (A10) survives the sentence,
  because it is what lets a reader tell a cancellation from a live removal after a reboot.

## What this design does not cover

- Who ends a holder,
  how a holder is identified and by what authority,
  which is step 17's other open item and the forced arm's.
  This design neither ends a process nor asks the service manager for anything,
  so its reports only name the `--force` run as a way forward.
- How the wrapper tells its caller's session apart from other holders,
  which is the same item.
- The forced arm's failure path before the restore:
  its scan,
  its refusal,
  its signal and its record of who was ended.
- The forced reach's mechanics inside a holder's namespace:
  how the mount to unmount is named there,
  what is checked before and after the call,
  and what a peer group and a child mount do,
  which the forced arm's design and the measurement it owes carry ([forced-record][forced-record],
  "How much a forced removal ends in another mount namespace",
  lines 78 to 83).
- The deferred-destroy checks' design,
  and whether their refusals should cancel rather than keep the records.
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
- `doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
  whole,
  merged from `main` at `db028cb9be`.
- `doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
  whole,
  the same merge.
- `doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
  its "A further answer:
  an add whose own unmount is refused as busy",
  the same merge.
- The gate review of the two step 17 designs,
  all three passes,
  at `~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review.md`,
  `review-second-pass.md`,
  `review-after-the-owners-answers.md` and `review-after-the-further-answers.md`,
  with the material kept beside them.
- `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
  the sections this file quotes and the whole of "Removal and recovery",
  "Cancelling a blocked removal",
  "Waiting and refusal",
  "Requests,
  results and ordering" and "Unmounted,
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
  step 16's split and per-chunk checks,
  the open-item list and the gap list.
- `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
  item 15 and its limits.
- `package/git-policy/cli/SPEC.md`,
  "Management grammar",
  "Doctor",
  "Records of a worktree that is removed",
  "The clone's mount intent,
  receipts and readiness",
  and the credential paragraph at lines 7348 to 7350.
  It has no removal section yet,
  because plan step 14 adds one with this design's sentences.
- The code named in "What the wrapper already has",
  read at the lines this file cites.

At a branch tip,
with `git show <branch>:<path>`:

- `cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design`,
  read at `f7765aea32`,
  whole enough to state the split and the disagreements,
  with its "The act's liveness",
  "The preflight" and "The refusal,
  and what it leaves".
- `cli-git-zfs-deferred-destroy-checks-design`,
  read at `d0ff7604b7`,
  "The look before the unmount",
  "A retry after a cancel" and "What a refusal leaves".
- `cli-git-zfs-removal-destruction-plan-record-design`,
  read at `3cd520b6a0`,
  the grammar,
  the states,
  the order,
  the absence rule,
  the removal intent,
  "A cancellation,
  and the plan it leaves",
  the readers and the finish pass.
- `cli-git-zfs-add-binding-record-design`,
  its home,
  format and readability.
- `cli-git-zfs-add-time-git-lock-design`,
  what the lock is and when it is written.
- `cli-git-zfs-origin-cleanup-record-design`,
  what the head-gone refusal leaves owed.

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
  `fs/pnode.c`,
  `include/linux/fs.h` and `kernel/signal.c`.

- systemd `v262`,
  `src/systemctl/systemctl.c`,
  `src/systemctl/systemctl-util.c` and `src/shared/bus-polkit.c`,
  which F21 cites.
The polkit defaults are read from the gate review's account of them,
not from a policy tree of this round,
and the local rules under `/etc/polkit-1/rules.d` are unreadable from this account.

## Links

[decision]: ../decision/cli-git-zfs-worktree-design.md
[forced-record]: ../decision/cli-git-zfs-forced-removal-authority-and-reach.md
[records-record]: ../decision/cli-git-zfs-clone-worktree-records-seven-answers.md
[dirty-record]: ../decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md
[forced]: cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
[staging-design]: cli-git-zfs-staging-under-cache-design.md
[staging-record]: ../decision/cli-git-zfs-staging-under-cache.md
[step-8]: ../decision/cli-git-zfs-step-8-sweep-choices.md
[slot-store]: ../decision/cli-git-zfs-slot-identity-store-format.md
[exit-codes]: ../decision/cli-output-and-exit-codes.md
[spec]: ../../package/git-policy/cli/SPEC.md
[deferred]: cli-git-zfs-deferred-destroy-checks-design.md
[plan-record]: cli-git-zfs-removal-destruction-plan-record-design.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[cancelling]: cli-git-zfs-worktree-design-proposal.md#cancelling-a-blocked-removal
[busy-unmount]: cli-git-zfs-worktree-design-proposal.md#project-decisions
[waiting-and-refusal]: cli-git-zfs-worktree-design-proposal.md#waiting-and-refusal
[vm-15]: cli-git-zfs-measurements-owed-before-acceptance.md#15-mount-namespaces-created-after-attach

The two sibling designs linked above are on their own branches until they merge into `main`,
so they are read at their tips with `git show <branch>:<path>`.

## Revisions

Each entry is one commit of this document,
with the committer time `git log --format=%cI` reports for it,
oldest first.

- `54d164aec4`,
  2026-11-22T17:29:36-05:00:
  the first text.
  It states every part of the three open items,
  the stop points,
  the cancel command's proof and act,
  the vetoes and questions,
  the tests and the measurements it owes.
- `bfb64b11eb`,
  2026-11-22T17:32:06-05:00:
  corrects the stop points,
  the refusal wording and the receipt field,
  and states where each record of a removal stands at each point.
- `b53e0e081c`,
  2026-11-22T17:33:21-05:00:
  states the two claims the guard rests on,
  the deadline report and the two refusals whose state the open item asks about.
- `02caf810b6`,
  2026-11-22T17:33:31-05:00:
  replaces the revision list with one entry per commit.
- `797a7cf2d6`,
  2026-11-22T17:35:26-05:00:
  says the design needs no configuration and reads none.
- `d9d2e1af50`,
  2026-11-22T18:27:50-05:00:
  answers the gate review.
  It adds the pre-move scan and the per-chunk and per-destroy scans to the stop points,
  adds the namespace-holder arm VM item 15 measured,
  adds the forced failure path and the ownership split with the forced arm,
  leaves the destruction plan unwritten by a cancellation,
  moves the fresh-attach question to the project,
  tabulates the cancelling sequence's states,
  replaces "never half acting" with what a standing request does at each trigger,
  corrects the deadline and cancellation reports,
  marks the grammar and the fresh nonce as choices,
  maps caller mistakes to exit `2`,
  adds the merged questions Q7 to Q10,
  adds the sibling documents and the section where the two disagree,
  and corrects every citation the citation pass found wrong.
- `4d60e1ddfc`,
  2026-11-22T18:28:21-05:00:
  lists every commit the document had at that point
  and notes that the two sibling links live on their own branches.
- `a432d9250a`,
  2026-11-22T18:29:40-05:00:
  reads an already-cancelled removal from the intent,
  the claim and the clone rather than from a plan that may not
  exist,
  and corrects the last citation ranges.
- The commit that carries this entry,
  written 2026-11-22T18:29:43-05:00:
  adds the entries for `4d60e1ddfc` and `a432d9250a`.
- `2fb7caea17`,
  2026-11-22T19:03:10-05:00:
  merges `main` into this branch so that the forced-removal record of 2026-11-22 is read here.
  It changes nothing in this document.
- `710f386f17`,
  2026-11-22T19:13:44-05:00:
  answers the gate review's second pass.
  It reads a holder the scan sees as the project decided it on 2026-11-22:
  the removal cancels before any snapshot is deleted,
  attaches the clone at its slot again,
  and reports the holder,
  and the passage that glossed the earlier sentence as "does not stop the destruction" is replaced by that reading.
  It writes that reading into the stop points for the pre-move scan,
  the scan after the unmount (a new point 8,
  with the points after it renumbered),
  the scan before a delete,
  and the head destroy a missed holder outlasts,
  and adds the arm blocked by a scan with its two loss cases.
  It adds "The forced arm's failure path" with the three ways the decided reach refuses,
  the state each leaves and a report for each,
  and the two refusals decided before anything is touched.
  It adds "When a cancellation can report destroyed history",
  renumbers this document's questions to S7 to S10 with what the answers change about each,
  adds M11 and limits L11 and L12,
  reads the forced arm at `1931987673` and cites it by section with `caller-protected-holds`,
  and corrects the citation slips that pass lists.
- The commits that carry this entry,
  written from 2026-11-22T19:14:06-05:00:
  add the entries for `2fb7caea17` and `710f386f17`.
- `1fee10f2ce`,
  2026-11-22T19:53:51-05:00:
  merges `main` into this branch so that the record's four further answers are read here.
  It changes nothing in this document.
- `04dd250d24`,
  2026-11-22T19:58:34-05:00:
  answers the gate review of this revision and the record's four further answers.
  It makes the cancellation after a loss a decided section quoting the record,
  and removes its question (S7) from the list;
  moves the two questions the record calls the designs' own choices (S9 and S10) to the veto list;
  splits the forced arm's refusals by what its read-only preflight can see,
  so only an unmount that turns out busy and a blocker that cannot be ended can follow an ended process,
  and rewrites the examples and arms to match;
  reads the later-scan cancellation as this design's extension,
  listed for the veto;
  adds a lazily detached copy,
  an unattached mount held by a descriptor and an incomplete scan read to the routes,
  with their arms and the extended M11;
  rewords the premises that said the refusal comes before the first delete everywhere;
  updates the statements about the forced arm at `6871053b79` to its current sections and both refusal words;
  and adds limits L13 and L14.
- The commits that carry this entry,
  written from 2026-11-22T19:58:38-05:00:
  add the entries for `1fee10f2ce` and `04dd250d24`.
- `db028cb9be`,
  2026-11-22T21:20:16-05:00:
  merges `main` into this branch so that the two records of 2026-11-22 are read here.
  It changes nothing in this document.
- `7b8f23c746`,
  2026-11-22T21:26:19-05:00:
  answers the gate review after the further answers.
  It follows the forced arm at `80780ff9c2`:
  a shared copy or parent is no longer a refusal,
  the refusals are split by what its preflight can see (three words,
  `holder-unseen` included),
  and its session case and the add's withdrawal are stated.
  It makes the kept plan a decided section,
  with what the removal and the cancel command do to it in each state,
  what the clone's next removal does,
  the deferred design's answer in its words,
  and the plan design's two sections named for it to settle;
  removes S8 and S13 from the questions,
  so none of this document's questions remains;
  adds the dirty-main record's second case to the defaults' heading and text;
  says what the cancel command does when it meets a pending add,
  and asks the add design to confirm the nonce of its withdrawal;
  says what a refused fresh attach leaves;
  corrects the unenterable-namespace arm and the outdented line;
  and updates the record citations and the sources list.
- The commits that carry this entry,
  written from 2026-11-22T21:26:24-05:00:
  add the entries for `db028cb9be` and `7b8f23c746`.
