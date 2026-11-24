# Step 10's three choices, as the owner answered them

## Status

The owner answered the three choices on 2026-11-24T14:10:00-05:00,
through the question tool,
and the answers are recorded at
`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`.

This is the third revision of this document.
The first was merged in `dce1e893a3`,
the second in `dc0635971e`,
and the questions the second carried were built from a same-vendor read of it,
whose full text is in that round's
`keep/read-of-the-step-10-open-choices-revision.md`,
under `~/temp/agent/gate-review-zfs-step-10-open-choices-revision-2026-11-24/`.
That read is a second pass by a Claude model,
not an independent review.

This revision is no longer a proposal:
it states the design the owner chose.
It is not final,
and nothing here is built:
only the project declares a design complete,
and no wrapper code changed for this round.

It was written in the worktree
`.claude/worktrees/zfs-step-10-answers-into-proposal-and-plan`,
branch `docs/planning-cli-git-zfs-step-10-answers-and-plan-order`,
made from `main` at `1ef4afefa8` and merged with `main` at `28267d769d`.

Citations were re-read against `main` at `28267d769d`.
Between the revision's base `06d3e542b8` and that commit,
the only change in this package is issue 6450's fix,
`79d644b9c8` (2026-11-24T13:52:28-05:00),
which is why the defect the second revision counted as a cost of every
option is gone (the heading "What the add reads today, and what it prints").

Nothing under `doc/decision/` or `doc/handover/` was touched.
No container gate,
guest,
or ZFS-changing command ran,
and no clock was waited on.

Where this document proposes a change to code,
the change is described and not built.
Where it reports what a person sees,
it names the file and line the sentence comes from,
or the probe that printed it,
or says that the sentence belongs to a design that is not built.

## The answers, in the owner's words

The record quotes each answer as the question tool gave it back.
Both the question texts and the answers were shaped by the read named above.

- Choice 1,
  which checkouts the add route serves:
  "E: a root command records it (Recommended)".
- Choice 2,
  what the add prints beside a standing removal plan:
  "2C: read the plan and the pool (Recommended)".
- Choice 3,
  which grants the add checks before it claims:
  "3C: refuse on six, warn on three (Recommended)".
- When the checkout migrates:
  "After removal exists (Recommended)".
- Who runs the migration,
  the owner's second reply:
  "You run it whenever you feel convenient."

The record also adopts one earlier answer without a new question:
a dirty main checkout takes rung 3 until step 11 lands,
as the owner answered on 2026-11-19
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`:1858 to :1862).
The code refuses such a source instead,
which is issue 6452.

## Where the read's findings landed

The read made two high findings,
seven medium ones and six lows.
Each is folded where it bears on the chosen answers,
and this heading says where,
so a reader can check the folding without reading the document twice.

### The high findings

- H1,
  the dirty checkout.
  Its correction is right and it is now the owner's own answer:
  rung 3 stands until step 11,
  so the refusal on `main` is the defect (issue 6452),
  not the design.
  It is the shared fact "A checkout with uncommitted changes takes the ladder
  until step 11",
  and the build of these answers carries the fall-through with its narrow
  condition.
- H2,
  reach before a removal exists.
  The owner answered it:
  the migration runs after steps 14 to 16,
  so no route-made worktree stands on a machine whose wrapper cannot remove it.
  It is the shared fact "What the route makes can be removed only once steps
  14 to 16 land",
  and the plan's step 19 records the order.

### The medium findings

- M1,
  the `root` field that `MountHoldingAPath` does not have,
  is corrected in the shared fact
  "The checkout path must be its dataset's own mount point"
  and again in Finding 1.
  The second revision's "correction" was the error.
- M2,
  2F's `gone` arm,
  is corrected in that option's own entry under
  "The options not chosen",
  and the correction is what puts it below 2C.
- M3,
  that a plan cannot stand before step 14,
  is the first paragraph of choice 2,
  true of the chosen answer as it was of every option.
- M4,
  that the step 11 expiry also ends 3B's premise,
  is the last heading of choice 3.
  The chosen 3C survives the expiry as a rewrite;
  3B does not.
- M5,
  choice 1's pair reasons,
  is folded into the chosen answer's grounds:
  the measured baseline is cited,
  and E over C is stated as the re-created dataset, not as the renamed one.
- M6,
  choice 2's pair reasons,
  is corrected in the chosen answer's own text:
  2C does not name one command in every state,
  and the cancel line is separable from the presence read.
- M7,
  B's regression sentence,
  is corrected in B's entry:
  the regression is an own-root checkout under another leaf name.

### The lows

- L1 is corrected:
  `doctor` judges the configuration file's facts
  (`handler_checks.rs`:329 to :342)
  and does not call the owner-argument reader,
  whose release callers are the sweep and the probe only.
- L2's citation is fixed:
  the design's sentence is at
  `cli-git-zfs-worktree-design-proposal.md`:1557 to :1558.
- L3 is corrected in the shared fact about the source refusals:
  `source_refusal` also refuses a sparse checkout and a worktree-scope Git
  configuration,
  and `snapshot_blocker` is a refusal of its own.
- L4 is the heading "What the add reads today, and what it prints":
  both call sites print the stop's sentence,
  because issue 6450 was filed and fixed on `main`,
  and issue 6451 is filed.
  The second revision's "no issue was filed" is out of date.
- L5 is cited in the shared fact about the shared mount:
  step 19's gate makes the checkout private first,
  by the owner's answer V5 of 2026-11-24.
- L6 is 3C's cost:
  the warning prints on every add while the grant stays missing.

## The example, and what is measured about it

Every option is measured on the example the build round used:

- the repository is at `/home/u/work/Monochromatic`,
  and its checkout is the root of dataset `tank/home/u/work/Monochromatic` in pool `tank`;
- the account is user 1000;
- the person runs `git worktree add .claude/worktrees/fix-flaky-test`,
  so the destination is
  `/home/u/work/Monochromatic/.claude/worktrees/fix-flaky-test`;
- the clone container the code names today is the checkout dataset's sibling,
  `tank/home/u/work/monochromatic-git-worktree-clone-datasets`,
  which `the_designs_container_beside` makes by replacing only the last component
  (`the_designs_datasets_in_the_mount_table.rs`:255 to :260);
- the origin snapshot the route would take is
  `tank/home/u/work/Monochromatic@git-worktree-clone-origin-<nonce>`,
  and the clone is
  `tank/home/u/work/monochromatic-git-worktree-clone-datasets/fix-flaky-test`.

Three facts about this machine decide what could be probed here,
and what the person in the example would meet on it today.

`/home/user/Monochromatic` is not its own dataset here.
`/proc/self/mountinfo` holds `zpcachyos/ROOT/cos/home` at `/home` with the
`shared:216` tag,
re-read this round,
so the checkout's dataset leaf is `home`,
and the checkout is a subtree of `zpcachyos/ROOT/cos/home`.
The gate answers `OtherDataset` for this repository,
so no run on this machine reaches the route under A,
B,
C,
D,
E or G.

The home dataset's mount is shared,
and it was already shared when the first revision measured it.
The same table holds `zpcachyos/nosnap/vm-builder-output` at
`/home/user/Monochromatic/package/vm-builder/cli/output` with `shared:246`,
which is the nested dataset the owner's answer of 2026-11-24 sets
`canmount=noauto` and orders a unit after the gate
(`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
"V4 and V9, nested datasets under the checkout").
That second fact decides whether naming this checkout in a root-owned file
would make the route run here,
and it would not:
the sweep records a shared checkout mount as `not-ready`
(`sweep_preconditions.rs`:401 to :430),
and the add refuses a `not-ready` record
(`clone_mount_readiness_admission.rs`:144 to :148).
So on this host the route needs step 19's private propagation before it needs
anything else.

This repository's tree at `06d3e542b8` is 526,754,210 tracked bytes in 25,276 files,
a mean of 20,840 bytes a file,
from `git ls-tree -r -l`, re-measured this round.
That decides which rung of the ladder an add takes today:
on ZFS a clean source is cloned only when its mean tracked file is at least 2,600 bytes
(`worktree_clone_rung_limits.rs`:199),
and a source with changes only when the clones would carry at least 762,000,000 bytes
(`worktree_clone_rung_limits.rs`:118).
A clean add of this repository takes the same-dataset reflink rung,
which asks the kernel to clone each tracked file with `FICLONE` inside the one dataset,
while the index's recorded file times still match the files;
when they do not,
the rung ends and Git's own checkout writes the worktree,
and the wrapper prints the whole reason and the remedy
(`probe-add-two-verbose.txt` under the first revision's `keep/`).
A dirty add of this repository falls to Git's own checkout.

So "the route does not reach this repository" does not mean the person writes files by hand today:
it means the add is a block-sharing copy in the source's own dataset,
with no separate dataset,
no mount at the slot,
no record and no lock,
instead of the snapshot and clone the route makes.

## What the chosen answers meet

Seven facts shape all three answers.
Each is a property of the code on `main` or of a record the project decided,
so no answer designs it away,
and each is charged to the answers that widen reach rather than repeated in each.

### A checkout with uncommitted changes takes the ladder until step 11

`preflight_the_add` proves the source with `source_refusal`
(`add_preflight.rs`:76),
which is the clean-only check (`worktree_snapshot_source.rs`:392 to :394 and :599).
The ladder's own call is `source_or_dirty` (:412),
and it is the only caller that accepts a source with ordinary changes.

So wherever the route serves,
an add whose main checkout has an uncommitted change stops before the claim with

```text
monochromatic-git-policy-cli: the main checkout cannot be the source of a clone: <the clean check's reason>
```

and every preflight refusal becomes `Some(ForwardedRun::Failed)`
(`add_route_run.rs`:77 to :80),
so the add does not fall back to the ladder.
The refusal names no way forward:
the sentence is the clean check's reason and nothing else,
which `DGT` asks a diagnostic to remedy.

The owner's answer is not that refusal.
On 2026-11-19 the owner answered "Ship step 10 first (Recommended)":
while step 11 is unlanded,
an add from a dirty main checkout takes rung 3,
as today
(plan `:1858 to :1862`).
The record of 2026-11-24 adopts that answer without a new question.
The merge bar the refusal would cross is recorded too:
"a branch may make nothing `main` does well worse"
(`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`:229).

The code deviates from that answer,
and issue 6452 records the deviation.
So the obligation these answers carry is the fall-through,
narrowed to exactly this refusal,
with a control that shows no other preflight refusal falls through.
It applies before and after the migration,
because the route refuses a dirty source wherever it reaches:
a person with uncommitted changes gets the ladder's worktree today and after
step 19 moves the checkout.

### The source refusals the preflight already makes

The refusal list a person can meet before the claim is wider than "not clean",
and each sentence is its own:

- `source_refusal` (`add_preflight.rs`:76) refuses a sparse checkout
  (`worktree_snapshot_source.rs`:601 to :602),
  a worktree-scope Git configuration (`configuration_refusal`,
  :129 to :139),
  a source whose Git configuration cannot be read (:631),
  and every reason the cleanliness check gives;
- `snapshot_blocker` (`add_preflight.rs`:77) refuses a nested worktree reached
  through a symbolic link and an ordinary nested worktree that a snapshot
  would turn into an empty stub (`worktree_snapshot_source.rs`:741 to :750);
- the preflight itself refuses a repository whose Git does not ignore
  `.claude/cli-git-trash` (`add_preflight.rs`:80 to :84).

The second revision listed the clean check and the trash rule only.
That is why the dirty case is stated first here:
it is the one refusal of that group the owner has already answered.

### The root side needs a name only root writes

`boot-reconcile` finds the checkout itself.
With no configuration file it takes the one mounted dataset whose last name component is the design's
(`sweep_configuration.rs`:197 to :200,
through `the_one_mounted_checkout`,
`the_designs_datasets_in_the_mount_table.rs`:157 to :178).
With a file it takes the names the file holds (`sweep_configuration.rs`:187 to :188),
and it publishes one readiness record per run (`boot_reconcile_mode.rs`:131 to :136 and :162).

The three cases a file can be in are not one sentence,
and which one a person reads decides whether the remedy the route prints is one that works.
The read ran the sweep's own parser on the example's mount table,
and this round re-read the three arms:

- no file,
  or a file without `cloneMounts`:
  `NoneMounted`
  (`mounted_dataset_refusal.rs`:147 to :150 and :202 to :204),
  whose remedy reads:

  ```text
  Ways forward: write `cloneMounts.checkout` (the checkout path) or
  `cloneMounts.checkoutDataset` (the checkout dataset) in <the file>, naming
  the checkout this run acts on; leave both keys out and the route finds the one
  mounted dataset whose last name component is `monochromatic-git-checkout`,
  which this release carries no setup that creates or mounts.
  ```

- a file naming `cloneMounts.checkout` alone:
  `LeafNameDiffers`
  (:170 to :174 and :214 to :216),
  whose remedy is one that works,
  because it names `cloneMounts.checkoutDataset` and that entry alone parses;
- a file naming `cloneMounts.checkoutDataset` alone:
  it parses,
  and the path comes from that dataset's one own-root mount
  (`sweep_configuration.rs`:193 to :195).

So a file that names one checkout is the only form of that name the code on
`main` reads,
and the root side acts on a name only root writes.
The answer E builds exactly that writer as a command, with the identity proved.

### The checkout's own mount must not be shared

The sweep's checkout state is `not-ready` when the mount holding the checkout carries a `shared:` tag,
and the sentence names it:

```text
the checkout's own mount is `shared:216`, so it is shared and a mount made on it is
delivered to another namespace; this route requires a not-shared immediate parent
```

(`sweep_preconditions.rs`:420 to :427).
Two controls cover it
(`sweep_preconditions_tests.rs`,
`a_shared_checkout_is_refused_with_its_own_reason` and
`a_shared_checkout_is_not_ready`).

Nothing in `worktree_clone_mount_handler/` sets propagation:
the folder reads the tag and refuses on it,
and no file there makes a mount private.
Plan step 19 builds the private propagation,
and the owner has since answered where it runs:
"V5: the gate makes the checkout private first"
(`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
"V1, V3, V5 and V7, accepted as the design ranks them").

So every answer that widens the gate needs that part of step 19 before a person
on a host like this one meets the route.
That is not the migration,
and it is the piece the route cannot work without.

### The checkout path must be its dataset's own mount point

The route's origin snapshot is a snapshot of the checkout dataset,
so the checkout must be that dataset's root,
and the design puts it there
(`cli-git-zfs-worktree-design-proposal.md`,
"Dataset and path layout").
The gate does not check it:
it compares the dataset's last name component and nothing else
(`the_designs_datasets_in_the_mount_table.rs`:236 to :238).

The struct the gate receives carries no root:
`MountHoldingAPath` has three fields,
the dataset, the filesystem type and the mount point
(`mounted_dataset_instance.rs`:50 to :58).
`MountedDatasetInstance.root` (:85) belongs to the other struct,
and `holding_of` (:182 to :185) builds the three-field one.
The root the table already read is dropped on the way:
`MountFacts.root` exists (`subvolume/fs_mount_options.rs`:61),
and `holding_of` does not carry it.
So the second revision's "the struct has a `root` field" was false,
and the first revision's "it carries no root field at all" was right.

The read's probe showed a checkout at `/u/repo` under a dataset named `monochromatic-git-checkout` mounted at `/u`
passing the gate,
and a bind mount of a subtree passing it too
(`review-probes-output.txt` in the read's `keep/`).
On `main` the add is then refused at `admit_the_handler`,
because the record's checkout path is the dataset's own mount point and the add's source root is not
(`add_preflight.rs`:134 to :136).

So the defect is real and latent:
it is not reachable through the gate alone,
and it becomes reachable through the chosen answer,
because a root command records a path beside a dataset,
and the sweep accepts the pair as text.
On this host a record naming the home dataset would make the route snapshot
and clone all of `/home`.
It is therefore a precondition of E,
and the fix takes one of the two shapes the code already has:
carry `MountFacts.root` through `holding_of`,
or take the row from `instances_of`
(`the_designs_datasets_in_the_mount_table.rs`:137)
and ask `shows_the_datasets_own_root`
(`mounted_dataset_instance.rs`:91).

### The handler must be installed, and this boot's record must have been published

Two root acts are outside the add and outside the gate:

- the handler is the second executable this build ships,
  which root installs and starts from a systemd unit and a ZED history event (plan step 19),
  and a clone that no handler attaches is a clone the add's mount wait ends without
  (`add_mount.rs`,
  the wait and its deadline);
- the readiness record is published by `boot-reconcile`,
  one per run,
  so a boot whose unit never ran has no record and the add refuses
  (`add_preflight.rs`:125 to :137).

Both are step 19's,
and neither is the migration.

### What the route makes can be removed only once steps 14 to 16 land

A worktree the route makes is locked,
and the lock's sentence sends the person to Git's own removal:
"remove the worktree with git worktree remove"
(`lock_words.rs`:34).
The wrapper leaves a locked worktree to Git
(`worktree_remove.rs`:343 to :348),
and Git refuses it "plain and with one force"
(`cli-git-zfs-add-time-git-lock-design.md`:344).
No removal of a route-made clone is built:
the plan's own status says step 1 alone is built (plan `:18` to `:19`),
and removal is steps 14 to 16.

So reach without removal leaves a person holding a worktree the wrapper cannot
take back,
and until the owner's answer below, step 19's own dependency list named none
of them,
because the migration's own code reads no removal record.
The owner's answer is the order:
"After removal exists",
steps 14 to 16 land before the migration runs,
so every worktree the route makes on this machine can be removed with the
wrapper from the first day.
The plan now records that order in step 19.

## Choice 1: a root command records the checkout (E)

### What the owner answered

"E: a root command records it (Recommended)".
The answer and the sentence below it are the record's,
section "Choice 1, which checkouts the route serves"
(`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`).

The record's sentence:
a root command records the checkout's path,
dataset and GUID,
container and pool,
and the add serves the checkout that record names,
refusing a dataset swapped in under the same name.

What the route is worth to a person is measured on this machine,
and the measurement is of today's ladder:
ten adds through the wrapper build of 2026-11-17T18:33:13-05:00 took
13.0 to 58.4 s each (median 24.6 s),
and one new worktree is 5,820 MiB by `du`
(`doc/decision/heavy-worktree-use-waits-for-the-zfs-route.md`:25 and :28).
That is the baseline the route exists to beat,
and no round has measured the route against it;
what the chosen answer adds is the reach that makes the comparison possible on
a checkout other than the design's own.

### What E builds

A root command,
run once per repository,
that proves each name it is given against the pool and then writes them to a
root-owned record:

- the checkout path and its dataset;
- that dataset's GUID;
- the container dataset and the pool.

The add,
the sweep and the probe read that record.
The add serves the checkout the record names and only that one,
and it compares the dataset's GUID with the recorded one before it claims anything,
so a dataset swapped in under the same name is refused at the gate.

This is a design for the next round and not built.
Two things it closes over do not exist yet in the shape it needs:
the checkout name today is the fixed leaf `monochromatic-git-checkout`
(`the_designs_datasets_in_the_mount_table.rs`:236 to :238),
and the readiness record carries "no field that names the checkout dataset or
the container" (Finding 2).

### What a person sees

For the example,
once root has run the command,
the checkout's mount is private,
the handler is installed and `boot-reconcile` has published this boot's record:
the route runs,
the snapshot lands on `tank/home/u/work/Monochromatic`,
the clone lands in the container,
and the add prints the filled note (`FILLED_NOTE`,
`add_route_run.rs`:45).

A container that does not answer under the recorded name is refused before the claim,
with the preflight's sentence that the route's layout is half-made
(`add_preflight.rs`:91 to :96).
A wrong dataset name is refused before the claim too,
at the dataset identity read (`add_preflight.rs`:89).
A dataset re-created under the recorded name is refused at the GUID comparison,
before the snapshot.

With no record,
or a record that does not name this repository,
the add keeps the ladder.
The accepted answer of 2026-11-22 is "no file needed"
(`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
"With no configuration file"),
so a missing record is not an error and the route simply does not serve the add.

### What it delivers over the option the read preferred

E and C reach the same one checkout with the same route and the same lines.
They differ in what the name can prove,
and the difference is narrower than the second revision's ranking said.

A dataset renamed after the name was written is refused under both:
the record proves the path,
and the add's identity read (`add_preflight.rs`:89) refuses a dataset name that
does not answer,
which is the sentence the second revision already charged to C.
What C cannot refuse is a dataset destroyed and re-created under the same name at
the same path,
because C's name is text and the record it is checked against proves the path and
the pool and nothing else.
E refuses that case at the GUID comparison.

The price is the mirror image:
a legitimate restore of the checkout dataset gives it a new GUID,
and E refuses the add until root re-adopts the checkout with the command.
C does not refuse that restore.

### What E costs

- the command,
  its proof of the dataset and the container it records,
  its grammar and refusals,
  and its controls;
- a setup step in plan step 19 that runs it,
  which the plan now names;
- a reader in every later step that names a checkout:
  the removal at step 14 and `doctor` at step 18;
- the own-root check on the pair the record holds,
  which is the gate fix the heading
  "The checkout path must be its dataset's own mount point" describes;
- the record's shape,
  which names one checkout per machine until the readiness record can name
  more (the second axis of the second revision's choice 1,
  which is a design change to a record the project decided);
- the fall-through for a dirty checkout, until step 11;
- the root-owned file's read,
  with `read_file_only_root_may_write_owned_by(path, 0)`
  (`file_only_root_may_write.rs`:162,
  and the wrapper at :140);
- and one sentence that is wrong today and will be read on this path:
  the `Owner` refusal says "its owner is user {found} and this program runs as
  user {wanted}" (`file_only_root_may_write.rs`:107 to :108),
  which for an add run as account 1000 with `wanted = 0` reads "this program runs
  as user 0" and is false (Finding 3).

The file's mode needs a word of its own:
`doctor` accepts `0600` and the account cannot read it
(`handler_checks.rs`:329 to :342),
so a root-owned `0644` is the mode this needs,
and the add's read must not assume `doctor` would have reported a mode it cannot use.

`doctor` judges that file's facts and does not call the reader itself;
the reader's release callers are the sweep
(`sweep_configuration.rs`:304) and the probe
(`probe_configuration.rs`:122).
The second revision said `doctor` already called it, which is false.

### The bounds that decide whether an add meets the route at all

The gate is the visible half of reach;
the readiness record,
the handler install and the checkout's propagation are the invisible half.
Several bounds stand between a person and the route,
and each answers in one of two shapes,
which decides whether a person keeps the ladder's worktree:

- silent,
  the ladder runs:
  the gate (not ZFS,
  or another dataset leaf,
  `add_route_gate.rs`:197 to :203),
  a destination that is not a direct child of `<checkout>/.claude/worktrees`
  (`add_route_gate.rs`:132 to :134),
  an add that is not the main worktree (:149 and :193),
  and an account companion that refuses (`add_route_run.rs`:66 to :68).
- a refusal,
  nothing else runs:
  the preflight's own refusals,
  which begin with the person's `--lock`,
  `--reason` or `--checkout` (`add_preflight.rs`:69 to :74),
  then the source refusals of the shared fact above,
  a plan that names the clone (:85 to :86),
  a slot parent that is not the account's (:87),
  a device,
  a dataset or a container that does not answer (:88 to :96),
  a delegation that is not the decided one (:97),
  and no readiness record,
  or one of another boot,
  pool or checkout (:98 and :128 to :133).

E changes the first bound's answer for the checkout the record names,
and leaves every other one as it is.

## Choice 2: the add reads the plan and the pool (2C)

### What the owner answered

"2C: read the plan and the pool (Recommended)".
The answer and the sentence below it are the record's,
section "Choice 2, what the add prints beside a standing removal plan"
(`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`).

The record's sentence:
once removal plans exist,
the add reads the plan's head word and whether a child of the container still
carries the clone,
and prints the one command that ends the plan,
the finish-pass wait line where the pool cannot prove the clone gone,
and the cancel command when nothing has started.

### What the add reads today, and what it prints

`refuse_while_a_plan_names_the_clone` (`plan_folder_read.rs`:160)
lists the plan folder,
takes only entries whose name is a clone's canonical decimal GUID,
refuses when a plan's `cloneGuid=` or `cloneDataset=` line names this clone
(:127 to :141),
and refuses a file it cannot read.
Its sentence is one constant,
whatever the plan says
(:213 to :219):

```text
a removal's plan at <path> names the clone this add would fill, so the add stops
before its first mutation and changes nothing. Ways forward: run the removal this
plan belongs to, which resumes from the plan and ends it; then run the add again at
the slot.
```

A person reads that sentence today.
Both call sites print `stop.sentence()`
(`add_preflight.rs`:85 to :86 and `add_route_run.rs`:230),
which is what issue 6450's fix, `79d644b9c8`, made true;
before it,
both formatted the stop with `{stop:?}`.
So the sentence exists and a person reads it,
and what 2C changes is which sentence it is,
not how it is printed.

The two readings that sentence needs are both absent:

- the plan's head word,
  written as `head=` in the plan's grammar
  (`cli-git-zfs-removal-destruction-plan-record-design.md`,
  "The grammar":
  `:1057 to :1060`),
  of which the add's reader never looks at that line;
- whether the recorded clone still answers under the pinned parent,
  which the plan design's "Which resume path a removal takes" makes the other half of the decision
  and which the add never reads.

`command_for` (`add_state_commands.rs`:175)
already holds the decided answer for every combination of the two facts,
and it has no release caller:
`rg` finds it in `mod.rs`'s re-export and in controls,
and nowhere else.

The design is explicit that V24 wants the decided command and not a constant.
`cli-git-zfs-add-binding-record-design.md`,
"Which command is true for a state":

> For any binding for which a plan stands,
> ... the destruction plan design's
> "Which resume path a removal takes" decides,
> and this design cites it and does
> not restate it.
> ... The add names the plan and the command the heading gives.

and the design's own plan row is by "the plan's head word and the live pinned parent".

One more clause is part of V24,
and the owner's answer of 2026-11-24 names it:
where the plan's tuples are all `planned`,
so that the plan witnesses nothing destroyed,
the refusal also names the cancel command,
"so a person who wants the worktree kept is not told only how to finish removing it"
(`cli-git-zfs-add-binding-record-design.md`:6024 to :6028).
That clause came in with `57d1e966f6` (2026-11-23T19:20:03-05:00),
the next commit to that design after the `aa6a5f18f0` the decision record
accepts.

### When a person can meet it

No plan can stand before step 14
(`plan_folder_read.rs`:10 to :12,
and the same in `add_route_run.rs`:226 to :228),
the settling command is step 16
(`doc/decision/cli-git-zfs-step-10-add-time-lock-and-binding-record.md`:40),
and the cancel command is step 17 (plan `:2373`).

So the add's read is built at step 10 and finds nothing until step 14,
and two of the commands 2C prints do not exist until steps 16 and 17.
That timing is true of the chosen answer as it was of every option,
and it is why this choice adds nothing to the plan's step order.

### The decided command for every state

Two facts decide the command,
and together they make fifteen states.
The read tabulated them by running `command_for` (`review-probes-output.txt` in the read's `keep/`);
this round re-read the function and got the same answers
(`add_state_commands.rs`:238 to :252):

- the clone stands (a child of the container carries the recorded GUID,
  under any name):
  the removal,
  for every head word (:239 to :241);
- the clone is proved gone,
  no child carrying the GUID:
  the settling command for `present` and `refused`,
  the removal for `requested`,
  `gone` and `unknown` (:248 to :251);
- the absence cannot be proved (the listing refused or came back partial):
  the settling command for every head word but `gone`,
  and for `gone` the finish pass,
  whose line names no command and says to wait for the pool (:242 to :246).

That is the removal in eight of the fifteen states,
the settling command in six,
and the finish pass in one.
The single constant sentence is right in eight and wrong in seven.

### What a person sees, state by state

2C parses `head=`,
enumerates the container's immediate children,
and builds the `ClonePresence` `command_for` reads:
`Stands` when a child of the container carries the recorded GUID,
whatever its name;
`GoneProved` when the recorded name is absent and no child carries the recorded GUID;
and `GoneUnprovable` when the listing refused or came back partial
(`add_state_commands.rs`:44 to :55).
It then prints the one command for that pair of facts,
and,
where the all-planned clause applies,
the cancel command beside it.

For the example,
a removal refused at its second deferred-origin check leaves a plan at
`head=present` beside the live clone,
and the person reads the removal,
which is the right command.
The rest of the states print the one command the pair allows:

- `present` and the clone stands:
  the removal;
- `present` and the clone is proved gone:
  the settling command,
  which accepts it;
- `present` and the absence is unprovable:
  the settling command,
  which refuses a proved-destroyed clone and names the read that failed;
- `requested`,
  `unknown` or `gone`,
  proved gone:
  the removal,
  which takes its head-gone path;
- `requested` or `unknown`,
  unprovable:
  the settling command;
- `gone`,
  unprovable:
  the finish pass,
  whose line names no command and says to wait for the pool;
- any head word with the clone standing:
  the removal;
- a plan whose every tuple is `planned`:
  the same command,
  and the cancel command beside it;
- the recorded name carrying another clone's GUID:
  no command settles it,
  and today no `ClonePresence` arm says so.

The second revision claimed 2C "names exactly one command in every state".
It does not,
and the record's own sentence is narrower on purpose:
the finish-pass cell names no command,
an all-planned plan names two,
and a reused name names none.
What 2C does deliver is a right command wherever a command exists,
and the wait line where waiting is the route.

### What 2C costs

- the `head=` line in the add's reader,
  with its refusals for an absent line and for a word outside the five;
- the plan read moved after `OpenedDevice::open`
  (`add_preflight.rs`:85 and :88),
  because presence needs the device,
  which also means a plan that stops the add no longer stops it before the device opens;
- an adapter from the preflight's opened device to the `DatasetReader` seam
  `each_child` takes
  (`sweep_dataset_reader.rs`:302 to :320),
  because `ZfsDatasetReader` owns its own device (:196 to :205);
- one stats read per child for the GUID match,
  up to the 4,096-child bound (`MAXIMUM_CHILDREN`,
  :90),
  and a statement of that bound beside the read,
  because the step 8 record treats growth on a waiting path as a defect;
- a fourth `ClonePresence` arm for a name reused by another GUID;
- the `tuple=` read and the cancel command for the all-planned clause,
  which widens a read the design calls "the smallest one that can refuse";
- and both plan reads carrying the same answer
  (`add_preflight.rs`:85 to :86 and `add_route_run.rs`:230),
  or the same state prints two different sentences.

### What 2C does not settle

The add changes nothing:
the read refuses,
and the plan,
the clone and the binding stay as they were.
So the person's next step is the command,
never the add.

## Choice 3: refuse on six, warn on three (3C)

### What the owner answered

"3C: refuse on six, warn on three (Recommended)".
The answer and the sentence below it are the record's,
section "Choice 3, which grants the add checks before it claims"
(`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`).

The record's sentence:
the add refuses when one of the six permissions it uses is missing,
and warns,
naming the `zfs allow` command,
when one of the other three is missing.
The split moves at plan step 11,
when the add's own failure path starts using both `destroy` grants.

### What the preflight judges today, and the five outcomes

`refuse_without_the_decided_grants` reads the account's effective delegation over the checkout dataset and the
container with two `ZFS_IOC_GET_FSACL` requests
(`add_grants.rs`:85 and :86),
builds `DelegationFacts`,
and refuses unless `check_delegation` passes
(`delegation_checks.rs`:483).
That judgement is the whole decided set of nine
(`REQUIRED_GRANTS`,
:64 to :74),
the absence of nine lineage-changing permissions (:90),
no route permission granted to everyone,
and no route permission reached through a named set.

`check_delegation` answers one of five kinds,
in one order,
and stops at the first:

- `permission`:
  the account or everyone holds a lineage-changing grant,
  and the line names the `zfs unallow` for each (:491);
- `everyone`:
  a route permission is granted to everyone,
  and the line lists them (:500);
- unknown:
  a route permission reaches the account through a named set,
  and the members are not in the reply (:509);
- `grant`:
  a decided permission is missing,
  and the line names the `zfs allow` for each (:517);
- `scope`:
  a decided permission is effective but wider than the design scopes it,
  and the line says "the narrowed command is printed for each" and prints none
  (:527),
  which is Finding 6.

`refuse_unless_the_decided_ones` refuses for all five
(`add_grants.rs`:105 to :114),
because it asks only whether the report passed.

### The nine permissions and the three groups

The proposal splits the nine itself,
in "Scoped delegation"
(`cli-git-zfs-worktree-design-proposal.md`:336 to :343):

- the six the unprivileged lifecycle needs:
  `snapshot` local to the checkout,
  `clone` on the checkout's descendants,
  `create` and `mount` local to the container,
  `mount` and `userprop` on the container's descendants;
- the clone removal's `destroy`,
  added on the container's descendants;
- the checkout-origin grant the project decided,
  `mount` and `destroy` on the checkout's descendants,
  which the origin cleanup uses and which the proposal warns "lets the account
  destroy **any checkout snapshot**".

For the example,
account 1000 and the two datasets,
the second and third groups are these three commands
(`allow_command`,
`delegation_checks.rs`:431 to :438):

```text
zfs allow -d -u 1000 destroy tank/home/u/work/monochromatic-git-worktree-clone-datasets
zfs allow -d -u 1000 mount   tank/home/u/work/Monochromatic
zfs allow -d -u 1000 destroy tank/home/u/work/Monochromatic
```

and the three together are the two `destroy` grants the build round's report names,
and the checkout-origin `mount` it does not.

Step 10's add issues no destroy.
Its own pool calls are the two identity reads,
the snapshot,
the clone,
the two inventories,
the origin read,
the grant read and the mount request's property write
(`add_zfs.rs`:81 to :186),
and on a failure it keeps what it made rather than destroying it
(V16's option A,
`cli-git-zfs-add-binding-record-design.md`:5588 to :5612).
The first destroy in the whole route is step 11's fallback,
then steps 14 and 16.

### What a person sees

For the example on a machine missing only the clone removal's `destroy`:

```text
monochromatic-git-policy-cli: this add will finish, and the wrapper cannot destroy
this clone until you run, as root:
zfs allow -d -u 1000 destroy tank/home/u/work/monochromatic-git-worktree-clone-datasets
The add changed nothing about that yet.
```

and then the clone,
the mount and the filled worktree.

A machine missing one of the six is refused before the claim,
with the sentence that names the permission and its `zfs allow`
(`add_grants.rs`:105 to :114,
and the read's probe line for a missing clone `destroy` in
`review-probes-output-three.txt`):

```text
the ZFS clone route cannot run with this account's delegation: zfs: delegation:
finding: grant: 1 decided permission(s) are not granted to the account: `destroy` on
`tank/home/u/work/monochromatic-git-worktree-clone-datasets` (zfs allow -d -u 1000
destroy tank/home/u/work/monochromatic-git-worktree-clone-datasets). Ways forward: run
the command the finding names, then run the add again; the add changed nothing.
```

### What it does for each outcome kind

- `permission`,
  `everyone` and unknown refuse,
  always,
  whatever group the permission belongs to;
- `scope` refuses,
  always:
  a removal-group `destroy` granted on an ancestor defeats the breadth the design's own option bought,
  and the read's probe shows that finding refusing today
  (`review-probes-output-three.txt`,
  the `broad-removal-destroy` line);
- a missing permission in the six refuses,
  as today;
- a missing permission in the removal's group and in the origin group is printed as one line,
  and the add goes on.

### What 3C costs

- a way to name the missing permissions in a form the caller can read,
  which means `check_delegation` or a sibling giving back the values rather than
  only formatting them into a sentence;
- the group table,
  which needs `check_delegation`'s own order to survive it,
  because the check stops at the first kind it finds;
- the `scope` finding's missing command,
  which a refusal for a removal-group `destroy` now depends on (Finding 6);
- the warning line and its two controls,
  one where it stays quiet and one where it prints;
- and the warning's own cost while the grant stays missing:
  it prints on every add,
  and a person who reads it and does nothing reads it again.

What stops it is nothing that argues against it,
with one thing to state:
the split is a rule about which step uses which permission,
and the proposal's three groups are its ground,
not a measurement.
An arm in the guest,
with the six granted and one of the three withheld,
should confirm that the add's own requests never fail for want of the withheld one,
before this is built as more than a design.

### The expiry at step 11

The add's own fallback destroys the clone and the origin snapshot
(A22 and A23,
`cli-git-zfs-add-binding-record-design.md`:2284 to :2318),
and "the route's grants carry the destroy" is the design's own words in A22.
No `destroy` appears in any non-test file of `worktree_clone_add_route/` on `main`,
so the table holds today.

After step 11 it does not:
the add itself then uses the removal group's `destroy` and the origin group's
`destroy` on its own failure path,
so the six become eight,
and 3C's warning would tell a person the wrapper cannot destroy a clone or an
origin the wrapper is about to try to destroy.
So the group table is a constant step 11 rewrites,
the warning's wording must come from the table and not be a fixed sentence,
and the two builds have to meet.

The expiry is not 3C's alone.
3B's premise, "the six the add's own steps use", ends at the same step and for
the same reason,
so the six-and-three split buys one step's worth of difference.
That is why the owner's answer is a rewrite owed at step 11 and not a rule that
lasts.

## The options not chosen

Each option below was priced in the second revision and put to the owner.
The evidence that decided against it is kept here,
so the next round does not re-derive it.

### Choice 1

#### A: keep the gate until step 19's migration

Nothing changes in the wrapper,
and the route serves a checkout only after root moves it onto its own dataset.

A person sees no line about the route during an add,
because the gate's answer is dropped silently (`add_route_gate.rs`:202),
and reads the ladder's own lines:
`copied 0 ignored filesystem entries ... into 1 new worktree`,
and,
for a source the clone rung served,
the line that says the worktree was not made as a snapshot.
No record folder appears,
and `doctor` reports nothing about the route,
because `route_checks` has no live caller
(`route_checks.rs`:112).

A delivers a worktree for every add,
and the ladder's block-sharing copy inside the source's dataset.
What decided against it is that the route then has no first live run before the
migration,
and the migration is the least forgiving moment for a path that takes a
snapshot,
a clone,
a claim,
a lock and a record.

#### C: serve the checkout the root-owned configuration file names

The add reads the same file the sweep and the probe read,
`/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc`,
and serves an add only when the repository's root is its `cloneMounts.checkout`,
with the dataset and container from `cloneMounts.checkoutDataset` and
`cloneMounts.parent`.

For the example the file holds:

```jsonc
// /etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc
{
  "cloneMounts": {
    "checkout": "/home/u/work/Monochromatic",
    "checkoutDataset": "tank/home/u/work/Monochromatic",
    "parent": "tank/home/u/work/monochromatic-git-worktree-clone-datasets"
  },
  "startProbe": {
    "checkout": "/home/u/work/Monochromatic",
    "dataset": "tank/nosnap/git-worktree-probe"
  }
}
```

`startProbe.dataset` is not decoration:
without it the probe's `found_source` calls `the_checkout_dataset_at` on the named path and refuses with the
leaf-name sentence
(`probe_configuration.rs`:277 to :288).
`startProbe.checkout` must equal `cloneMounts.checkout`,
or `parse_configuration` refuses with `CheckoutsDiffer`
(`sweep_configuration.rs`:266 to :270).

What decided against it is the identity:
a dataset re-created under the same name at the same path passes C,
because the record C is checked against proves the path and the pool and
nothing else,
and E refuses that case.
C's own four limits stand as evidence:

- one checkout per machine under the current record;
- the file's names are text;
- no file names this repository's checkout unless setup writes one:
  the step 8 record says the setup writes both names and that
  "no person edits that file by hand"
  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
  "The datasets the sweep acts on"),
  and the example's dataset is not one step 19 creates,
  so setup needs an adopt mode,
  which is the writer E describes;
- with no file,
  or a file that does not name this repository,
  the add keeps the ladder,
  which the step 8 record's "no file needed" answer permits.

#### G: serve the checkouts a root-owned file lists

C's file with a list of checkouts.
What decided against it is that the list delivers nothing until the readiness
record can name a second checkout:
one record file serves the whole machine
(`READINESS_DIRECTORY`,
`clone_mount_readiness.rs`:43,
and `record()`,
`clone_mount_readiness_file.rs`:227),
and one record names one checkout (`ReadinessRecord`,
:227 to :246).
So G buys the file's grammar,
the sweep's loop over checkouts,
the probe's refusal of two checkouts and the record's shape,
for a delivery that waits on the same design change E's stated reach waits on.

#### B: serve any checkout that is its own dataset's root

The leaf comparison goes,
and the mount at the path must be the dataset's own mounted root.

What decided against it is that it delivers nothing E does not,
and takes the ladder's worktree away from some checkouts.
Every checkout the root side can name is either the design's dataset,
which the route already serves after the migration,
or one a root-written name points at,
which E serves.

The regression is narrower than the second revision said:
an own-root checkout under a leaf name other than the design's is refused
under B,
because the gate stops answering `Ok(None)` and no record names it,
while a subtree checkout such as the example's keeps the ladder exactly as
today,
because it fails B's own own-root test.
The read's M7 is this correction.

`boot-reconcile` also cannot name the checkout for such a machine,
so the person is sent to a command that cannot succeed:

```text
no handler readiness report stands for this boot, so no attach is admitted
(/run/...). Ways forward: start the boot unit's `boot-reconcile` and run the add again
```

(`add_preflight.rs`:128 to :133),
while `boot-reconcile` answers the `NoneMounted` sentence of the shared fact.

#### D: make the delegation the gate

The gate widens,
and `check_delegation` becomes what refuses an unmanaged checkout.

What decided against it is that it is the weaker half of B's mistake:
it opens the add and leaves the root side unable to name the checkout,
and it adds a device read and a delegation judgement to every ZFS add in a
repository of any dataset,
including the ones the ladder serves today.
It delivers nothing B does not.

#### H: do the migration now, as its own question

Not a change to the gate:
root moves the checkout onto its own dataset with the design's names,
and the gate then serves it.

H delivers what E delivers plus the design's own layout,
the immutable covered entry,
the private propagation,
`snapdir=disabled`,
and a checkout that no later step has to migrate.
What stopped it from being an alternative was never its cost but its order:
the migration's own recovery depends on steps 12 and 18,
the plan keeps those in step 19's dependency list for that reason,
and an interrupted migration has nothing a person can read until those land.

The owner answered the order question,
and the answer is not "now in the plan's old order" and not "keep the route
unreachable":
the migration happens after steps 14 to 16,
and the agent runs it with sudo when no process has the checkout open.
So H is no longer an alternative to A.
It is the route's arrival,
ordered behind removal,
and the plan's step 19 records that.

### What the dirty checkout comes to

The second revision left this open as an add-on with three arms.
The owner's earlier answer settles it:

- W2, fall through to the ladder for exactly this refusal,
  is the owner's answer of 2026-11-19,
  and the code's instead refusing is issue 6452.
  It delivers the worktree the person asked for,
  from the route's own decision point.
  It costs the narrow condition and its control,
  and it must not fall through for any other preflight refusal.
- W3, refuse and name the way forward,
  remains worth its sentence either way:
  the current refusal breaches `DGT` on its own terms,
  because it names no way forward.
- W1, refuse as today,
  is the arm the code implements and the record does not choose.

### Choice 2

#### 2A: print the removal for every state

The constant sentence,
which a person now reads as written,
is right in eight states and wrong in seven.
Where the clone is proved gone and the plan's head is `present` or `refused`,
the removal refuses and names the settling command;
where the pool could not prove the absence it refuses and names the settling
command,
except at a head of `gone`,
where the finish pass is the route and no command accepts the state.
What decided against it is the seventh state:
a refusal that names no command.

#### 2F: read the head word only, and print both where it cannot decide

The option the second revision invented,
and the read corrected its `gone` arm:

- `requested` and `unknown` print the removal;
- `present` and `refused` print the removal and the settling command with the
  fact that chooses;
- `gone` prints the removal **plus the finish-pass line**:
  where the pool cannot prove the absence,
  the removal refuses naming no command,
  and the finish pass is the route once the pool answers
  (`cli-git-zfs-removal-destruction-plan-record-design.md`,
  "Which resume path a removal takes",
  `:1754 to :1765`).

That corrected arm is right in thirteen of the fifteen states,
and its two wrong ones are `requested` and `unknown` beside an unprovable
absence,
where the removal refuses and names the settling command.
The correction also takes away the two reasons the second revision used for it:
it never names a wrong command that has no next step only because of the
finish-pass line,
and "thirteen of fifteen" is at most twelve without it.

What decided against it is that it asks the person a question in the nine states
whose head word is `present`,
`refused` or `gone`,
and V24's own example is one of them.
It needs no device read,
so it keeps the plan read where it is.

#### 2E: print both commands with the condition that chooses

The add prints the head word's two candidate commands with the fact that decides
between them,
and the person looks at the container.

What decided against it is that it hands the person a judgement the wrapper can
make with one read it already has the device for,
in fifteen states,
and it is undefined for `gone`.
It never names a wrong command,
which is not enough against a right one.

#### 2B: read the head word only and print `command_for`'s answer

The add parses `head=` and calls `command_for` with `clone: None`.
In V24's own example that is the settling command,
because that is what `command_beside_a_standing_plan` answers for `present` when
no clone was looked for
(`add_state_commands.rs`:238 to :251,
and the read's probe line
`head=present clone=not-looked-for => TheSettlingCommand`).
The settling command refuses a binding whose clone stands and names
`git worktree remove`,
so the add sends the person to a command that refuses in the scene V24 uses to
justify the rule.

#### 2D: print the facts and no command

V24 asks for the command,
and `DGT` asks a diagnostic to name what a person runs.

### Choice 3

#### 3A: judge all nine, as today

The person with only the clone removal's `destroy` missing sees the refusal
quoted in choice 3,
runs one `zfs allow` as root,
and adds again.
The permission they granted is one the add never uses;
it is the clone removal's,
four steps later for the removal,
and six for the destroy that step 16 makes.

What decided against it is that it blocks an add for a permission the add does
not use,
and it costs a root command and a second add in a state where the first add
would otherwise have finished.
Its compensating value stands:
the add and `doctor` judge the same bytes.

#### 3B: judge only the six the add's own steps use

The add finishes and the missing permission meets the person at the removal.
The design requires that removal's preflight to read the same grants:
"keep descendant `destroy` and require `doctor` and removal preflight to verify
the scoped effective grants"
(`cli-git-zfs-worktree-design-proposal.md`:1557 to :1558).
With that read built,
the first removal refuses before any plan is written,
naming the `zfs allow` command.

Plan step 14's own build list does not name that read,
although step 14 builds the plan,
the intent and the refusal before the first delete
(plan `:2153 to :2227`).
So on today's plan text the refusal arrives from the kernel instead,
at step 16's first destroy,
after the plan is written and the intent recorded:
a snapshot destroy in the batch,
or the head destroy when the clone has no snapshot of its own.
What the plan holds then depends on which request was first,
and neither answer makes the add's sentence wrong:
a clone with its own snapshots has them refused first,
and the plan stands with its attempt lines and `head=present`;
a clone with none has the head destroy refused,
and the plan stands at `head=refused`.
In both the clone still stands,
and `command_beside_a_standing_plan` answers the removal for every head word
beside a standing clone (`add_state_commands.rs`:239 to :241),
so the add at that slot names the removal,
which refuses again on the same missing grant.

What decided against it is that it leaves the person to meet the refusal at the
removal,
with a plan already written,
where 3C names the permission before it is needed.
Its premise also ends at step 11,
as the last heading of choice 3 sets out.

#### 3D: judge only the forbidden nine

Missing required permissions are left to the kernel's own `EPERM` at the first
request that needs them.
A machine missing `snapshot` takes the claim,
writes the binding,
registers the worktree,
takes the lock,
and then the snapshot is refused.

What decided against it is that it moves a refusal the wrapper can make for free
to a point where it has already written four durable objects,
which is the opposite of A0's reason for existing.
It also leaves a too-wide removal `destroy` uncaught and an unproved
delegation unjudged.

## What these answers add to the plan's steps

The plan,
`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
now records the answers where they are work,
and the record of 2026-11-24 is the authority for each.

- Step 10 gains three things.
  The add's gate reads the record root's command writes and serves the checkout
  it names,
  refusing a dataset swapped in under the same name (choice 1).
  The preflight reads a standing removal plan's head word and the container's
  children,
  and prints the one command that pair allows (choice 2).
  The grant judgement splits: the six the add uses refuse,
  and the other three warn (choice 3).
- Step 11 gains the split's move.
  Its own failure path then uses both `destroy` grants,
  so the six become eight and the group table is rewritten (choice 3).
- Step 19 gains the root recording command choice 1 needs,
  and records the order:
  its migration runs after steps 14 to 16,
  and the agent runs the migration of steps 12,
  18 and 19 and the `canmount=noauto` change on
  `zpcachyos/nosnap/vm-builder-output` with sudo at a time no process has the
  checkout open.

No other step's content changes,
and the chosen answers add no step.

## Findings this round did not fix

This round changes no wrapper code,
so each of these is described and left for the round that takes the code it lives in.

### Finding 1: the gate does not prove the checkout is its dataset's own root

`the_checkout_dataset_at` takes the mount holding the path
(`the_designs_datasets_in_the_mount_table.rs`:229 to :235)
and compares the dataset's leaf name (:236 to :238).
It never compares the mount's own mount point with the path,
and the struct it receives carries no root
(`mounted_dataset_instance.rs`:50 to :58),
because `holding_of` (:182 to :185) drops `MountFacts.root`
(`subvolume/fs_mount_options.rs`:61).
So a checkout at `/u/repo`,
on a dataset named `monochromatic-git-checkout` mounted at `/u`,
passes the gate,
and a bind mount of a subtree at the path passes it too.
On `main` the add is then refused at the readiness record,
so the defect is latent;
it becomes reachable through the chosen answer's record,
which makes it a precondition of E.
On this host a record naming the home dataset would make the route snapshot and
clone all of `/home`.
The fix is to carry `MountFacts.root` through `holding_of`,
or to use `instances_of` (`the_designs_datasets_in_the_mount_table.rs`:137) and
`shows_the_datasets_own_root` (`mounted_dataset_instance.rs`:91).

### Finding 2: the readiness record proves no dataset

`ReadinessRecord` carries the boot,
the pool's name and state,
the checkout's path and state,
the sweep's nonce and the time
(`clone_mount_readiness.rs`:227 to :246),
and `admit_attach` compares the pool and the checkout path
(`clone_mount_readiness_admission.rs`:132 and :138).
No field names the checkout dataset or the container.
Today the leaf rule makes the dataset name nearly a constant,
so the gap is masked;
the chosen answer unmasks it,
because the add then takes a dataset name from a record the root side's
readiness record does not compare.
Closing it means a field in a record the project decided,
which is a design change and not a build's to make.
The same record's one-checkout shape is what stops E from serving two
repositories.

### Finding 3: the file a root side owns is readable by the account, and two edges of that

The build round's report said the add could not read the root-owned configuration file,
because `read_file_only_root_may_write` refuses a file whose owner is not the account running the program
(`FileControlRefusal::Owner`).
The reader takes its owner as an argument,
`read_file_only_root_may_write_owned_by(path, owner)`
(`file_only_root_may_write.rs`:162),
the public wrapper passes `geteuid()` (:140),
and `doctor`'s own handler-configuration check demands exactly
"owner is 0 and no group or other write bit"
(`handler_checks.rs`:329 to :342).
So the add calls the existing function with `0`,
which is the same two terms `doctor` judges,
and adds no reader of its own.

Two edges stand:

- `doctor` accepts a `0600` file the account cannot read,
  so `doctor` can pass where the add's read would fail with `EACCES`.
  The mode this feature needs is a root-owned `0644`,
  and no check in the tree says so;
- the `Owner` refusal's sentence says
  "its owner is user {found} and this program runs as user {wanted}"
  (`file_only_root_may_write.rs`:107 to :108),
  which for an add run as account 1000 with `wanted = 0` reads
  "this program runs as user 0" and is false.
  The chosen answer meets this refusal path,
  so the sentence needs a form that names the caller and the wanted owner.

### Finding 4: the sweep already serves a named checkout, and the sentences differ by case

`checkout_of` takes the path and the dataset from the file when the file holds both and asks the mount table nothing
(`sweep_configuration.rs`:187 to :188),
and a file naming only `cloneMounts.checkoutDataset` parses,
taking the path from that dataset's one own-root mount
(:193 to :195).
The three cases print three different sentences,
which the shared fact "The root side needs a name only root writes" sets out.
The root side is therefore already capable of reconciling the example's checkout,
and the parts that are not are the add's gate and the probe's fallback.

### Finding 5: the fallback's line names btrfs to a ZFS checkout

The line an add prints when the snapshot route did not serve it is
`this worktree was not made as a snapshot, so it took longer, because <the source> is not a btrfs subvolume`,
and the reason behind it is `snapshot_refusal`
(`worktree_snapshot_source.rs`:235 to :248),
which knows two causes:
the source is no subvolume root,
and the source and the destination are not on one btrfs filesystem.
On a ZFS checkout both answers are beside the point:
the design's snapshot route is step 10's,
and what kept it away there is the dataset leaf name,
the record's names,
the shared mount or the missing handler.
So the one line a person on ZFS reads about the missing snapshot names a filesystem they are not on,
and `DGT` asks a diagnostic to name the cause the person can act on.

### Finding 6: the too-wide delegation finding promises a command and prints none

`finding("scope", ...)` says
"the narrowed command is printed for each"
and joins no command into the line
(`delegation_checks.rs`:519 to :528).
Its control asserts the field name and the two dataset names and nothing else
(`delegation_checks_tests.rs`:115 to :124).
A person who meets a removal-group `destroy` granted on an ancestor reads a promise and no command,
and 3C makes that finding the refusal for exactly that permission.
The fix is either the command or the sentence,
and the control should assert whichever the project wants.

### Two stale statements in code docs

Not defects in behaviour,
and not this round's to change:

- `plan_folder_read.rs`:13 to :15 says "The plan grammar is not accepted yet",
  and the destruction plan design was accepted on 2026-11-23
  (`doc/decision/cli-git-zfs-removal-side-records-and-the-adds-record-accepted.md`:5 to :9);
- the same module's doc says the read asks only for `cloneGuid=` and `cloneDataset=`,
  which is true,
  and 2C is what makes it read more.

## What was probed, and what was not

This round ran no build,
no test and no probe of its own.
Its evidence is the code on `main` at `28267d769d`,
the records,
and the probe outputs the read kept in its `keep/`:

- the fifteen-state command table,
  the example's three file cases,
  and the gate on a subtree and a bind mount (`review-probes-output.txt`);
- the three delegation sentences,
  including the too-wide one (`review-probes-output-three.txt`);
- the package's two control runs the read repeated,
  `125 passed` and `83 passed` (`rerun-controls-first-command.txt` and
  `rerun-controls-second-command.txt`).

What this round did itself:

- re-read every call site and record the read and the second revision cite,
  and corrected the ones that had drifted:
  `MountHoldingAPath`'s fields,
  the design's `:1557 to :1558`,
  the `scope` control's range,
  `instances_of` and `the_one_mounted_checkout`'s ranges in the mount table,
  and the second revision's "`doctor` already calls" the owner-argument reader;
- measured this host's mount table for the shared tag on `/home` and for the
  nested dataset under the checkout;
- measured this repository's tree at `06d3e542b8`;
- read plan steps 10,
  11,
  12,
  14,
  15,
  16,
  18 and 19,
  the two accepted designs,
  the destruction plan design's grammar and resume headings,
  and the step 8 and step 19 records.

Not probed,
and why:

- no live run of the route anywhere.
  This machine has no dataset named the design's,
  its checkout mount is shared,
  and this round ran no ZFS-changing command and no guest,
  so the route's own steps have still never run on any machine,
  and every claim about them here is from the code and the designs;
- no guest arm.
  Choice 3's split would be settled by one,
  with the six granted and one of the other three withheld,
  and that arm is the first thing a build of 3C should run;
- no measurement of what an add costs under each option.
  The claim that the route copies nothing where the ladder copies every file is a mechanism from the code,
  not a timing;
- no container gate,
  as the round's rules say,
  and no `mise run toolchain:install`;
- no count of how often a checkout has uncommitted changes.

## What the next round would do

For choice 1's answer:
the root recording command and its record,
the add's gate rebuilt on the names it holds,
Finding 1's own-root comparison in the same round because that gate is the one
being opened,
the record's shape for one checkout,
and the statement that the add keeps the ladder with no record.
The readiness record's per-checkout shape is a design question first,
and E's stated reach of one checkout waits on it.

For the dirty checkout:
the narrow fall-through in `run_a_zfs_add`,
its control showing that no other preflight refusal falls through,
and the sentence fix either way.

For choice 2's answer:
the `head=` line,
the fourth `ClonePresence` arm,
the adapter and the child bound,
the plan read moved after the device opens,
the all-planned clause with its `tuple=` read and cancel command,
and both plan reads answering the same way.

For choice 3's answer:
a way to name the missing permissions as values,
the group table as a constant step 11 rewrites,
the `scope` finding's command or its sentence,
the warning line,
and the guest arm that withholds one of the three.

For the order:
nothing in code,
because the plan's step 19 now records it,
and the migration is run by the agent with sudo when no process has the
checkout open.
