# cli-git ZFS staging under the cache folder

## Status

Accepted 2026-11-20 in the question tool,
by the answer "Accept,
boundary (a) (Recommended)".
The design is
[`doc/planning/cli-git-zfs-staging-under-cache-design.md`](../planning/cli-git-zfs-staging-under-cache-design.md),
read and accepted at `c685cabebe`,
and its body stands as accepted:
the only change the acceptance made to that file is one note at its top naming this record.
The evidence,
the probes,
the arms the design still owes,
the rejected options and the reviews that shaped it stay there,
in its "Revisions" entries and the review folders they name,
and in the documents it rests on:
the whole design in
[`cli-git-zfs-worktree-design-proposal.md`](../planning/cli-git-zfs-worktree-design-proposal.md),
the measurements in
[`cli-git-zfs-measurements-owed-before-acceptance.md`](../planning/cli-git-zfs-measurements-owed-before-acceptance.md),
and the build order in
[`cli-git-zfs-worktree-implementation-plan.md`](../planning/cli-git-zfs-worktree-implementation-plan.md).
The place the implementation interface will live is `package/git-policy/cli/SPEC.md`.
The design's lead names six reviews,
and one more was run on the sixth revision:
the diff-only gate check kept at
`~/temp/agent/zfs-staging-under-cache-design-sixth-revision-diff-only-gate-check-2026-11-20/keep/review.md`,
whose lows this record carries.

This record was written at `2026-11-20T05:54:01-05:00`,
by an agent,
from the answers the project gave.
The project has not reviewed it,
and only the project can amend it.
The nine lows of the last diff-only gate check are corrected where this record restates them,
under "The last check's lows,
and what this record says instead",
because the design's body stays as accepted.

## Decision

The project accepted the staging design,
answering "Accept,
boundary (a) (Recommended)".
Staging happens in one folder per removal under the account's `~/.cache`,
and both sides derive the folder's name from the clone's canonical numeric dataset GUID,
as `monochromatic-git-policy-cli-staging-<guid>`.
The wrapper creates the folder,
which is the answer to the first question of 2026-11-19.
The root handler binds the folder,
makes the bind private before the clone is moved onto it,
so the clone's move onto that verified private bind propagates nowhere,
and writes the staged-folder record beside the folder,
at `<home>/.cache/monochromatic-git-policy-cli-staging-<guid>.json`,
before its first bind.
The record holds the folder's identity,
its file handle,
the nonce of the intent it serves,
and the handler's mount-namespace pair and boot id.
A later sweep opens the folder from that handle and unmounts from the held descriptor,
which is the interface the open item owed.
The wrapper's cleanup is `rmdir`-only,
never recursive,
stops at any mount,
and runs from the staging sweep keyed to a dead owner.

When the move lands on a mount the account stacked on the folder,
the handler rolls the exact mount back to its slot by its own descriptor at once,
and keeps it staged only if the rollback fails,
which is the third answer of 2026-11-19.
Where the rollback succeeds,
the bind,
the folder,
the record and the claim stay for the sweep,
the removal writes a refusal receipt naming the outcome,
and its next attempt starts at the slot,
where the clone's mount standing in the host namespace refuses it again.
The design also refuses when a mount stands on the clone alone,
and that refusal is the design's own rule,
not the answer's:
the answer's limit covers the scene where the clone landed on a mount the account stacked.

The pin of the `user-cache` mount records the dataset's source and the inode of the mount root under it,
and every check compares those two fields:
at boot reconcile,
before every bind,
and in the wrapper before it creates a folder.
The source carries the ordering check,
because a bind made before `user-cache` mounts lands on the home underlay and the dataset's later mount hides it.
The inode catches a mount at `~/.cache` whose root is not the dataset's root,
such as a bind of a subdirectory of the same dataset,
and it cannot refuse falsely after a reboot,
because a ZFS mount root's inode is its on-disk root object number,
read from disk at every mount.
On this host every ZFS mount root the last check measured read inode 34,
so the inode separates no dataset from another here,
and the source alone does that work.
The pin also records the device the boot that wrote it read at the root,
for `doctor` to print,
and no check ever compares it:
the device is ZFS's anonymous per-mount minor,
which a reboot may change,
so the pin cannot gate a durable key on it (L14).

`doctor` prints the staging folder,
its live bind's propagation,
any staged mount with its dataset and age,
and the next safe step,
and it changes nothing.
The record's place under `~/.cache`,
the two `/proc`-path readings,
and the `~/.cache` lookup from the pinned UID's passwd home stand as "Veto none" left them.

## The acceptance question, 2026-11-20

The question was where the accepted answer's own limit reaches,
and its answer is "Accept,
boundary (a) (Recommended)".
The design is accepted,
and the owner's third answer stays bounded to the scene it names:
"move it back at once ... keep and report it only if the move back fails",
which is how the acceptance round put that limit in words.
The proposal records the same limit as
"retaining and reporting it only if rollback fails"
([proposal](../planning/cli-git-zfs-worktree-design-proposal.md),
"Project decisions",
"Misplaced attach").
The limit therefore covers the scene where the clone landed on a mount the account stacked.
Refusing when a mount sits on the clone alone is the design's own rule,
not the owner's,
and the design states it that way.
The seven veto-open choices the design lists stand as it wrote them,
and none was vetoed:
see "The proposals left open to veto".

## Where a clone's mount waits, and who creates the folder

The answer is "Private bind per removal (Recommended)".
The wrapper creates one folder per removal,
the handler binds it and makes the bind private for that removal,
and the clone is moved onto the bind,
so two removals of one clone never share a folder and the move onto the verified bind propagates nowhere.
A mount the account stacks on the folder between the bind and the move still receives the clone,
which is the scene the rollback answers.
The alternatives stay described under "Ways to meet it" in the design:
a private `~/.cache` made once at boot,
root creating the folder,
a boot-made bind,
a root-owned empty source,
and a staging root two levels deep.
The answer fixed Way A,
and the design records those five as ways a writer could build but did not take,
not as ways any decision refused.
The sixth alternative,
the namespace teardown,
is refused by the kernel rather than by the answer:
`do_move_mount` requires the moved mount to be the caller's own (F25).

## How a later sweep finishes an unmount

The answer is "Record the file handle (Recommended)".
The handler writes the folder's file handle into the staged-folder record before the first mount,
and a later sweep opens the folder from that record and unmounts from the held descriptor.
That field is the one part of the record no live state rebuilds once the handler's invocation has ended,
and the record's read rules,
the boot-id test before the handle,
and the `ESTALE` pair of tests stand as the design states them.
The other two routes stay recorded as not taken:
a boot-long supervisor holding every staged folder open,
and waiting for the administrator.

## What the handler does when the move lands on a stacked mount

The answer is "Move it back at once (Recommended)",
given after "What?"
to the same question and a second asking in plain words.
The handler rolls the exact mount back to its slot by its own descriptor,
and keeps it staged only if the rollback fails.
The design reads a rollback whose proof cannot pass as a failed rollback,
which is its own reading,
and it stops and reports there.

## The proposals left open to veto

The answer of 2026-11-19 is "Veto none (Recommended)",
and it was given to the four proposals that round left open:
the `~/.cache` lookup from the pinned UID's passwd home,
the `user-cache` pin,
and the two `/proc`-path readings.
The design's "Choices this revision marks for the project's veto" holds six more choices the project had not seen,
and its pin entry holds a seventh,
the pin's field set.
The acceptance of 2026-11-20 vetoed none of the seven,
so all seven stand as the design has them:

- The rollback's destination and its proof.
- The scene where a mount is stacked on the clone,
  and where that scene ends.
- The end state and the receipts after a successful rollback,
  after a reattach and after a cancel.
- A record whose folder is gone.
- A record from another boot.
- How the accepted no-implicit-remount default covers the rollback.
- The pin's field set,
  which the pin entry marks as the writer's.

Two things in that section are not vetoes of a writer's addition.
The question inside it,
where the accepted answer's own limit reaches,
was answered on 2026-11-20,
under "The acceptance question,
2026-11-20",
and it is the only choice there with no default.
The rollback entry's option (c) would relax the accepted "prove both" in the scene where the halves part,
so it was put to the project as a decision and not as a veto,
and the project did not take it:
the rollback keeps the parent-row clause.

## The handle record's place

The answer is
"In ~/.cache .
We have to assume my machine isn't compromised.
sudo is passwordless here anyway."
The design had placed the record in `/var/lib/monochromatic-git-policy-cli/`,
and the answer moves it beside the folder,
as one cache file rather than root's state.
The threat model that answer states is the project's own for this machine:
the account is trusted,
and root and the account are not a security boundary here.
The design follows it,
says where an argument survives it,
and lists the checks a hostile account motivated under
"Hardening the stated threat model may make unnecessary",
without dropping one on the strength of the sentence alone.

## No implicit remount, and its one exception

The project keeps "Keep,
one exception (Recommended)",
and the design reads the rollback as the accepted "Misplaced attach" rule applied within the invocation
that made the move.
The rollback is therefore no second exception,
and the checked reattach of a removal that cancels itself without `--force` stays the one exception.
The sentences that record one exception stay true as recorded,
which is why none of them is amended:
see "Amendments to other documents".

## The last check's lows, and what this record says instead

The last check is the diff-only gate check of the sixth revision,
kept at
`~/temp/agent/zfs-staging-under-cache-design-sixth-revision-diff-only-gate-check-2026-11-20/keep/review.md`.
It is an agent's review,
non-final and unapproved,
and it found no high finding,
no medium finding and nine lows.
The corrections below are the statements this record carries;
the design's body keeps the accepted wording,
and its top note sends a reader here.

### Low 1: the pin's field set, marked for the project's veto

The check found the field set marked as the writer's inside the pin entry of "Proposals the owner settled",
outside the veto section and without the options a veto entry gives.
The design marked it in that pin entry for the project's veto,
as the seventh of its seven veto-open choices,
and the acceptance vetoed none of the seven,
so the field set stands as the design took it:
the dataset's source and the mount root's inode,
which every check compares,
and the device,
which is recorded for `doctor` and never compared.
The other two field sets are ones the project did not take:
(a) the dataset's source alone,
which is the accepted refusal sentence's literal text,
and (c) the source,
the inode and the device,
which refuses after any reboot that changes the anonymous minor (L14) until the pin is rewritten.
The design recorded a ranking of the three,
b over a over c:
b over a because a leaves a mount at `~/.cache` whose root is not the dataset's root,
such as a bind of a subdirectory,
undetected,
while b catches it;
and a over c because c banks the pin on the anonymous per-mount minor,
which a reboot may change and which nothing here measured,
so c refuses at a boot where the pin still holds and needs the pin rewritten.
That ranking is the design's record of the choice it made,
and no option of the three is still open.

### Low 2: what the source and the inode each catch

The check measured that every ZFS mount root on this host reads inode 34,
so the inode separates no dataset from another here,
and it found the design silent about what the field adds.
This record says what each field does,
under "Decision":
the source carries the ordering check that a bind made before `user-cache` mounts needs,
and the inode catches a mount at `~/.cache` whose root is not the dataset's root,
such as a bind of a subdirectory of the same dataset.
The host observation is that check's,
made on 2026-11-20,
and it is labelled as measured here rather than as a ZFS constant.
What removing the pin would simplify is a pin comparison,
not a per-removal source comparison.

### Low 3: the device-across-a-reboot arm's staging step

The check found that the arm's added staging step cannot run as written,
because after the reboot the settled clone is unmounted and the accepted default does not remount it.
The statement this record carries:
the arm settles the record's device field,
and it runs the pin's staging step for a fresh clone,
staged after the reboot,
so the pin's comparison is seen not to be what fails at the settlement and does not depend on the settled clone.
The settled clone's two branches each give their own outcome:
the folder,
record and claim removed where the number held,
and kept,
reported and printed by `doctor` where it changed.

### Low 4: the boundary answer and the stacking options

The check found the question naming one conflicting pair of answers,
its option (c) reaching past the scene the answer covers,
and the design not saying what it builds while the question stood open.
This record says the three things:
an answer of (a) or (b) leaves the stacking entry's options (b) and (c) contradicting it in the combined scene,
and under (b) in both scenes;
option (c) here stands the answer's literal "Move it back at once" wherever a rollback could be attempted,
which extends the answer's text to the clone-alone scene the question never covered;
and while the question stayed open the design builds both refusals,
because (a) and (b) refuse identically.
The project answered (a),
so both refusals stand,
the combined-scene refusal as the answer's own limit,
and the clone-alone refusal as the design's own rule.

### Low 5: why the rollback is not a second exception

The check found the rollback entry's answer to the default's race reason overstated,
because "no new managed attach" does not separate the rollback from the checked reattach.
This record gives the reason the design gives elsewhere:
the rollback repairs a placement inside the invocation that made the move,
before that invocation ends.
The rollback therefore accepts the proposal's own cost line for a move back,
that "an implicit remount would race the standing `unmount` intent and any staged mount",
and that cost is reasoning,
not a measurement.

### Low 6: text the last revision left stale

The check found four places:
"M5's three arms",
where M5 now has four,
and those are the negative arm,
the positive arm,
the mismatched pin inode arm and the boot-order arm;
"no per-removal source comparison",
corrected to a pin comparison,
under "Low 2";
the lead's list of reviews,
which omits that check and its folder;
and the "Revisions" lead,
which does not say a revision may be labelled by its write time.
The first two statements this record carries as corrected.
The other two are the design's own lead and revision history,
which the acceptance leaves as accepted,
and this record carries what those two places leave out:
the lead leaves out a seventh review,
the diff-only gate check of the sixth revision,
kept at
`~/temp/agent/zfs-staging-under-cache-design-sixth-revision-diff-only-gate-check-2026-11-20/keep/review.md`;
and the sixth revision's label is the time its entry was written,
`2026-11-20T00:14:25-05:00`,
because a commit cannot name its own commit,
so a revision may be labelled by its write time.

### Low 7: wording in the pin entry and in L14's arms

The check found five overstatements.
The pin keeps the dataset's source and the mount root's inode and records the device,
and it compares the source and the inode and never compares the device.
The device is what the pin cannot gate on,
not what no durable root key can gate on,
since the accepted proposal records a device in durable root records (L14).
The pin never compares a device,
rather than comparing no device across a boot.
The arm's premise is the device the reboot may change,
not the device the reboot changed.
And the identity the other-boot settlement compares is the one L14 says a reboot may change,
not one it may not leave in place.
The statements this record makes under "Decision" and "What the handler does" follow the corrected wording.

### Low 8: M5's mismatched-inode arm

The check found one assertion in the arm that cannot fail:
"No entry is created on the home underlay" carries over from the negative arm,
where the dataset is not mounted,
while in this arm a wrapper that wrongly proceeded would create its folder under the mounted `.cache`.
The assertion this record carries is the corrected one,
that no folder or claim is created under the mounted `.cache`.
The arm's other assertions can fail,
so the arm tests the right thing.

### Low 9: rewrapped paragraphs

The check found five ragged tails left in the paragraphs the last revision touched,
and the revision entry's "Four paragraphs are rewrapped" inexact.
The five tails remain in the accepted body,
which the acceptance leaves as it stands,
and this record restates no claim of theirs.
The entry's rewrap claim is inexact in both count and effect:
five places have tails and it counts four,
and it says the paragraphs are rewrapped when they are not.

## Amendments to other documents

### The design named two conditional amendment sets, and neither is due

The design names amendments to other documents only under options the project did not take:

- The no-implicit-remount entry's option (b),
  the second-exception reading,
  amends the proposal's "Decided defaults" bullet,
  its "the one remount the no-implicit-remount decision allows",
  and the plan's entries on what a cancelled removal restores,
  all in the design's "Choices this revision marks for the project's veto".
- The rollback entry's option (c) would amend an accepted sentence,
  the "prove both" proof the design applies to the rollback.

The project answered "Keep,
one exception (Recommended)" and took neither option,
so no sentence of the proposal,
the plan,
the accepted worktree design or `package/git-policy/cli/SPEC.md` changes for them.
`package/git-policy/cli/SPEC.md` holds no sentence that changes for this design:
its boot-arm list already asks for
"the replacement of the staging and checkout mounts until a private setup is verified",
which the design meets by verifying the pin at boot reconcile
and each removal's bind and propagation,
and its other uses of the word name a copy's staging name,
a lock's staging file,
or the act of staging a copy.

### The notes and sentence changes this acceptance puts on the documents that waited on it

Three documents said in words that this design did not exist yet.
The proposal and the accepted ZFS worktree design's record keep their text
and gain an in-place note dated 2026-11-20,
while the plan is live work and states the accepted design in its own sentences instead.
None of the statements changes a decision,
and each of the notes names this record.

- The proposal,
  [`cli-git-zfs-worktree-design-proposal.md`](../planning/cli-git-zfs-worktree-design-proposal.md),
  in four places.
  The staging requirements of "Descriptor mount and unmount" carry one note
  after the paragraph that ends "the placement was one of the defaults the project answered";
  it covers that paragraph's "have no design yet" sentence for the folder's creation,
  its private bind and the wrapper's cleanup,
  and the next paragraph's "has no design yet" sentence for the interface that unmounts from staging.
  The staging-folder check of "What doctor checks and prints" carries a note
  after its "have no design yet" sentence for the folder's creation,
  its private bind and the wrapper's cleanup.
  The open item "Staging under `~/.cache` and the wrapper's cleanup" carries a note
  after its "are undesigned" sentence,
  covering that sentence and its "is not designed either" sentence.
  The acceptance test "The wrapper's staging cleanup" under "Decided behaviours with no test yet"
  carries a note after its "has no design yet" sentence.
  The notes name the accepted design and this record,
  and they keep the point that nothing is built yet:
  no unmount runs from staging until step 9 ships that interface.
- The plan,
  [`cli-git-zfs-worktree-implementation-plan.md`](../planning/cli-git-zfs-worktree-implementation-plan.md),
  in eight places,
  each changed in place rather than noted:
  the paragraph on the designs that block the route early,
  which now names two designs and says the staging work is accepted;
  step 19's "The staging design blocks its boot arm" sentence,
  which now names step 9's staging build;
  step 9's "Open items",
  where the blockage and the wait of steps 9,
  14,
  16 and 17 stand for the build alone;
  the list of steps that cannot run their VM arm,
  where the staging steps move to a list of their own that waits on the build;
  the "What can start now" bullet,
  which now names two blocking designs;
  the paragraph beginning "While all three designs are unwritten",
  which now counts two and names step 9's staging interface;
  the staging item of "Open items and the step that needs each",
  which says the design is accepted and the step blocks on the build;
  and the "Where the design is silent" entry,
  whose title and text now order two blocking designs.
- The accepted ZFS worktree design's record,
  [`cli-git-zfs-worktree-design.md`](cli-git-zfs-worktree-design.md),
  in "Status",
  where the sentence "with its open items still open" gains a note that this open item is designed and accepted,
  and that the other open items stay open.

## What the acceptance does not do

- It measures nothing.
  Every measurement arm the design states stays owed,
  M1 to M9 included.
  Step 9 repeats VM items 3,
  4 and 15 with the built code
  ([plan](../planning/cli-git-zfs-worktree-implementation-plan.md),
  step 9,
  "VM measurements"),
  and items 3 and 4 measured one path on the guest's root file system,
  not this location and not a folder per removal (F19),
  so their repeat is the one that covers this location and fixture.
- It builds nothing.
  Step 9 and the steps behind it wait on the build,
  and no unmount runs from staging before step 9 ships.
- It changes none of the design's sizes,
  deadlines or instructions,
  and it decides none of the hardening entries,
  which stay suggestions for the project.
- It does not touch `doc/handover/`,
  and it does not edit the design's body.
