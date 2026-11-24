# Step 10's three open choices, revised after the read

## Status

A proposal.
Not a decision,
not approved,
and not final:
only the project declares otherwise.

Second revision.
The first was
`doc/planning/cli-git-zfs-step-10-open-choices.md` as merged in `dce1e893a3`,
written by the round
`~/temp/agent/zfs-step-10-open-choices-proposal-2026-11-24/`.
This revision answers the read of it,
`~/temp/agent/gate-review-zfs-step-10-open-choices-2026-11-24/keep/read-of-the-step-10-open-choices.md`,
and was written in the worktree
`.claude/worktrees/zfs-step-10-open-choices-revision`,
branch `docs/planning-cli-git-zfs-step-10-open-choices-revision`,
made from `main` at `06d3e542b8`.

The read is a same-vendor second pass,
not an independent review.
Nothing here is approved,
and the project has not been asked anything by this round.

No wrapper code changed.
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

## What this revision changed

The read made three high findings,
eleven medium ones and eight lows.
Every one is folded below,
and this heading says where each landed,
so a reader can check the folding without reading the whole document twice.

### The high findings

- **The dirty checkout.**
  Wherever the route reaches,
  an add from a checkout with uncommitted changes is refused until step 11,
  and the first revision ranked reach without saying so.
  That is now a fact every option meets
  (the heading "A checkout with uncommitted changes is refused"),
  a cost in every option that widens reach,
  and the ground of a ranked add-on
  (the heading "Add-on W:
  what an add does on a checkout with uncommitted changes").
- **What the add prints.**
  Both plan reads format the stop with `{stop:?}`,
  so a person reads a Rust value and not the sentence the design quotes.
  That is now a defect stated once
  (the heading "What the add prints today is a Rust `Debug` value"),
  a cost in every option of choice 2,
  and a finding this round did not fix.
  No issue was filed:
  the coordinator files it,
  as the round's brief says.
- **Choice 3,
  option 3B.**
  Its example had the wrong head word,
  the wrong command,
  and a cost that belongs to the design's removal preflight rather than to 3B.
  All three are corrected in 3B's own heading,
  and the 3C-over-3B reason is rebuilt on the correction.

### The medium findings, each where it landed

- M1,
  the private checkout mount,
  is a shared fact
  ("The checkout's own mount must not be shared")
  and a precondition named in every option that widens the gate.
- M2,
  the sentence each of the sweep's three file cases prints,
  is corrected in the shared facts.
  The consequence it draws for B and D is stronger than the read's
  and is the ground on which B and D are no longer ranked at all.
- M3,
  the step 8 record's "no person edits that file by hand" and its rejected "setup writes whole file",
  is folded into C's cost,
  which now says that naming a dataset step 19 did not create is E's writer.
- M4,
  the two axes and the one-record-per-machine bound,
  is the heading "The two axes this choice turns on",
  and E's cost now carries the readiness-record change G was alone charged with.
- M5,
  A2 as a complement,
  moved A2 out of the ranking
  and into "Two add-ons,
  not alternatives".
- M6,
  the own-root precondition and "only B would have closed it",
  is the shared fact
  ("The checkout path must be its dataset's own mount point"),
  and Finding 1 no longer names B.
- M7,
  V24's cancel arm,
  is in the shared facts of choice 2 and in 2C's cost,
  and the two texts a person would read are shown.
- M8,
  the fifteen states,
  is the heading "The decided command for every state",
  is what puts the new 2F above 2E,
  and is stated as a cost in every option of choice 2.
- M9,
  the five outcome kinds,
  is the heading "What the preflight judges today,
  and the five outcomes",
  and each option of choice 3 now says what it does for each kind.
- M10,
  the expiry at step 11,
  is in 3C's heading and cost,
  and in the last paragraph of choice 3's ranking.
- M11,
  H left out of the ranking,
  is answered by putting H to the owner as its own question,
  which is the choice the brief's "ranked or stated as its own question" allows.

### The lows

- L1's three arithmetic slips are corrected where they stood.
- L2's citation drift is corrected:
  `clone_mount_readiness_admission.rs` `:132` and `:138`,
  `add_grants.rs` `:85` and `:86`,
  and `add_zfs.rs` `:56`.
- L3's four more bounds are listed with the gate and the preflight,
  each with the shape of its answer.
- L4's three points are in the shared fact about the root-owned file and in Finding 3.
- L5 is C's cost:
  a typo is refused before the claim in every case this round could find but one.
- L6 is 2C's cost:
  an adapter,
  one stats read per child,
  and the 4,096-child bound.
- L7 and L8 are findings this round did not fix.

### What the read found that did not hold

Nothing.
This round re-read the code for every finding and every citation it could check,
and each one holds.
Two needed a small correction,
given here so a reader does not have to find them:

- The read's "the add cannot reach the snapshot that way" is exact for the code on `main`,
  and the reason is one step later than the read places it:
  the gate passes a subtree of a design-leaf dataset,
  the record's checkout path is that dataset's own mount point,
  and `admit_the_handler` compares the two
  (`add_preflight.rs`:134 to :136).
  The refusal is the readiness record's,
  not the gate's.
- The read's M2 says the ladder is lost "when the account companion passes".
  The companion is a weaker escape than that:
  the ladder is reached only when the companion refuses
  (`add_route_run.rs`:66 to :68,
  and `the_managed_attach_goes_ahead`,
  `companion_lines.rs`:154 to :156),
  so on a machine where the companion skips for want of user namespaces the add is still refused.

### What this round found beyond the read

The "too wide" delegation finding says
"the narrowed command is printed for each"
and prints no command
(`delegation_checks.rs`:527),
and its control asserts the field name and the two dataset names and nothing else
(`delegation_checks_tests.rs`:115 to :125).
A person who meets that finding is promised a command that is not in the sentence.
It is the sixth of the findings this round did not fix,
and it decides one of 3C's five outcome kinds.

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

`/home/user/Monochromatic` is not its own dataset here:
`/proc/self/mountinfo` holds `zpcachyos/ROOT/cos/home` at `/home` with the `shared:216` tag,
so the checkout's dataset leaf is `home`,
and the checkout is a subtree of `zpcachyos/ROOT/cos/home`.
The gate answers `OtherDataset` for this repository,
so no run on this machine reaches the route under A,
B,
C,
D,
E or G.

The home dataset's mount is shared,
and every nested dataset under it is too:
`.cache` is `shared:234` and `.cargo/build` is `shared:240`.
That second fact is the one that decides whether naming this checkout in a root-owned file would make the route
run here,
and it would not:
the sweep records a shared checkout mount as `not-ready`
(`sweep_preconditions.rs`:401 to :428),
and the add refuses a `not-ready` record
(`clone_mount_readiness_admission.rs`:144 to :148).
So on this host the route needs step 19's private propagation before it needs anything else.

This repository's tree at `06d3e542b8` is 526,754,210 tracked bytes in 25,276 files,
a mean of 20,840 bytes a file,
from `git ls-tree -r -l HEAD`.
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

## What every option here meets

Six facts shape every option below.
Each is a property of the code on `main`,
so no option can design it away,
and each is charged to the options that widen reach rather than repeated in each.

### A checkout with uncommitted changes is refused

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

This is by design and not a defect of reach:
the accepted design's A0 says the preflight
"refuses a source that is not clean,
so a dirty main checkout stops the add before the claim instead of falling to the ladder's rung 3"
(`cli-git-zfs-add-binding-record-design.md`:1857 to :1859),
and the decision record gives the dirty half of the restore to step 11
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 11:
The dirty-main restore and the frozen proof").

The consequence for this document is narrow and it is the reason it is stated first:
**an option that widens reach lowers what a person with uncommitted changes gets,
until step 11.**
Under A they get a worktree today and after the migration;
under C,
E,
G or H they get a refusal the day the names are written.
The add-on W is the question that can change this,
and it is separable from every option here.

### What the add prints today is a Rust `Debug` value

Both plan reads answer with a `PlanStop`,
and both call sites format it with `{stop:?}`:

- `add_preflight.rs`:85 to :86,
  in the preflight;
- `add_route_run.rs`:230,
  at A2.

`PlanStop` derives `Debug` and has no `Display`
(`plan_folder_read.rs`:99 to :112),
so what a person reads is

```text
NamesTheClone(PlanNamesTheClone { plan: "<path>", sentence: "a removal's plan at <path> names the clone this add would fill, ..." })
```

and not the sentence the design quotes.
The sentence exists as a value,
`PlanStop::sentence()` (`plan_folder_read.rs`:109 to :115),
and no caller uses it.

No control asserts the add's printed text:
the `plan_folder_read` controls run the reader,
and `add_preflight_tests.rs` and `add_route_run_tests.rs` assert the stop's own fields.
So this is a defect on `main` that the tests could not catch,
and the fix is two substitutions plus one control that prints the refusal and asserts the sentence.
Every option of choice 2 carries that fix as a cost,
because every one of them changes what this sentence says.

The read's probe printed the `Debug` form and the sentence side by side
(`review-probes-output-debug-format.txt` in the read's `keep/`),
and this round re-read both call sites.

### The root side needs a name only root writes

`boot-reconcile` finds the checkout itself.
With no configuration file it takes the one mounted dataset whose last name component is the design's
(`sweep_configuration.rs`:197 to :200,
through `the_one_mounted_checkout`,
`the_designs_datasets_in_the_mount_table.rs`:157 to :174).
With a file it takes the names the file holds (`sweep_configuration.rs`:187 to :188),
and it publishes one readiness record per run (`boot_reconcile_mode.rs`:131 to :136 and :162).

The three cases a file can be in are not one sentence,
and which one a person reads decides whether the remedy the route prints is one that works.
The read ran the sweep's own parser on the example's mount table,
and this round re-read the three arms:

- **no file,
  or a file without `cloneMounts`**:
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

- **a file naming `cloneMounts.checkout` alone**:
  `LeafNameDiffers`
  (:170 to :174 and :214 to :216),
  whose sentence is the one the first revision quoted,
  and whose remedy is one that works,
  because it names `cloneMounts.checkoutDataset` and that entry alone parses;
- **a file naming `cloneMounts.checkoutDataset` alone**:
  it parses,
  and the path comes from that dataset's one own-root mount.

So the first revision quoted the second case's sentence for the first case,
and it is the second case's sentence that names a working edit.
Both facts stand in the shared record:

**the root side acts on a name that only root writes,
and a file that names one checkout is the only form of that name the code on `main` reads.**

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
`a_shared_checkout_is_refused_with_its_own_reason` and `a_shared_checkout_is_not_ready`).

Nothing in `worktree_clone_mount_handler/` sets propagation:
the folder reads the tag and refuses on it,
and no file there makes a mount private.
Plan step 19 builds "the private propagation the unit sets on the verified checkout,
and what re-applies it after a
replacement mount"
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 19:
The one-time migration,
the immutable entry and the boot and shutdown units").

So every option that widens the gate needs a part of step 19 before a person on a host like this one meets the route:
the private propagation,
applied to the checkout's own mount.
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
(`the_designs_datasets_in_the_mount_table.rs`:236 to :238),
and the `MountHoldingAPath` the gate receives has a `root` field
(`mounted_dataset_instance.rs`:85) that `holding_of` never fills (:182 to :184).

The read's probe showed a checkout at `/u/repo` under a dataset named `monochromatic-git-checkout` mounted at `/u`
passing the gate,
and a bind mount of a subtree passing it too
(`review-probes-output.txt` in the read's `keep/`).
On `main` the add is then refused at `admit_the_handler`,
because the record's checkout path is the dataset's own mount point and the add's source root is not
(`add_preflight.rs`:134 to :136).

So the defect is real and latent:
it is not reachable through the gate alone,
and it becomes reachable through any option that lets a root-written name pair a path with a dataset,
because the sweep accepts the pair as text.
On this host a file naming the home dataset would make the route snapshot and clone all of `/home`.
It is therefore a precondition of C,
E and G,
and the fix is the one the code already has the parts for:
`instances_of` and `shows_the_datasets_own_root`.

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
  (`add_preflight.rs`:120 to :133).

Both are step 19's,
and neither is the migration.

## Choice 1: how far the add route reaches

### What decides reach today

`the_routes_add` makes two host reads
(`add_route_gate.rs`:192):
`filesystem_kind(worktree_root)`,
and then `the_checkout_dataset_at`
(`the_designs_datasets_in_the_mount_table.rs`:228).
The second finds the mount holding the path,
takes its dataset name,
and refuses unless the dataset's last component is
`monochromatic-git-checkout`
(:236 to :238).
Any refusal becomes `CheckoutMount::OtherDataset`
(`add_route_gate.rs`:202),
`decide_the_routes_add` answers `Ok(None)` (:152),
`run_a_zfs_add` returns `None` (`add_route_run.rs`:58),
and `run_served` falls through to the ladder
(`worktree_copy_forward.rs`:378).

Several bounds stand between a person and the route.
The gate is the one a reader meets first,
and each bound answers in one of two shapes,
which decides whether a person keeps the ladder's worktree:

- **silent,
  the ladder runs**:
  the gate (not ZFS,
  or another dataset leaf),
  a destination that is not a direct child of `<checkout>/.claude/worktrees`
  (`add_route_gate.rs`:132 to :134),
  an add that is not the main worktree (:149 and :193),
  and an account companion that refuses (`add_route_run.rs`:66 to :68).
- **a refusal,
  nothing else runs**:
  the preflight's own refusals,
  which are the person's `--lock`,
  `--reason` or `--checkout`
  (`add_preflight.rs`:69 to :74),
  a repository whose Git does not ignore `.claude/cli-git-trash` (:80 to :84),
  a source that is not clean (:76),
  a plan that names the clone (:85 to :86),
  a slot parent that is not the account's (:87 and the function below it),
  a device,
  a dataset or a container that does not answer (:88 to :96),
  a delegation that is not the decided one (:97),
  and no readiness record,
  or one of another boot,
  pool or checkout (:98 and :125 to :137).

The load-bearing consequence is this:
**the root side must be told which checkout to reconcile,
and it must be told in a place only root writes.**

The gate is the visible half of reach;
the readiness record,
the handler install and the checkout's propagation are the invisible half,
and every option here is judged on both.

### The two axes this choice turns on

The round's options and the first revision's mix two questions,
and separating them is what makes the ranking below short.

- **How many checkouts one machine can serve.**
  Today:
  one.
  The readiness record names one checkout
  (`ReadinessRecord`,
  `clone_mount_readiness.rs`:227 to :246,
  one `checkout_path`),
  `boot_reconcile_mode::run_at` reads one `SweepConfiguration` and publishes one record
  (:131,
  :136 and :162),
  and `admit_attach` compares that one path (`clone_mount_readiness_admission.rs`:138).
  So a second repository named in a file is not served until the record,
  the boot unit and the add's admission
  all change.
  That change is a design change to a record the project decided,
  and it is the same change for C,
  for G and for E.
- **What the name is.**
  The design's own leaf name (A),
  text a root writes and the add re-reads (C,
  G),
  or an identity a root-run command proved when it wrote it (E).
  The design's own ground is the third:
  "Root configuration pins the account UID,
  the checkout path,
  pool and dataset names and GUIDs"
  (`cli-git-zfs-worktree-design-proposal.md`:336).

A choice that answers the first axis "many" pays the record change;
a choice that answers the second "proved" pays the writer.
C,
E and G answer the first axis the same way until that change is made,
so they differ on delivery only along the second,
and that is where the ranking's adjacent pairs come from.

### A. Keep the gate until step 19's migration

Nothing changes in the wrapper.
The route serves a checkout only after root moves it onto its own dataset.

What the person sees for the example,
measured in a throwaway repository on this host whose checkout sits on this machine's dataset rather than on
`tank/home/u/work/Monochromatic`
(the heading "What was probed,
and what was not" names the run):
no line about the route,
because the gate's answer is dropped silently (`add_route_gate.rs`:202);
the add succeeds,
and the wrapper prints the ladder's own lines,
`copied 0 ignored filesystem entries ... into 1 new worktree`,
and,
for a source the clone rung served,
`this worktree was not made as a snapshot, so it took longer, because <the source> is not a btrfs subvolume`;
and no record folder appears,
so the route wrote no claim,
no binding and no plan folder.
For a clean tree of this size whose recorded file times match,
the add is the clone rung's;
for a dirty one it is Git's own checkout.
If they also run the boot unit,
`boot-reconcile` fails with the `NoneMounted` sentence quoted at
"The root side needs a name only root writes",
and `doctor` reports nothing about the route,
because `route_checks` has no live caller
(`doctor_zfs_route/route_checks.rs`:112 is called only from controls and the guest arm).

What it delivers:
a worktree for every add,
including every add from a checkout with uncommitted changes,
and the ladder's block-sharing copy inside the source's dataset.

What stops it:
nothing.
It is the shape the plan drew,
and the accepted decision's own text says the route's dataset
"exists only after plan step 19's one-time migration"
(`cli-git-zfs-add-binding-record-design.md`,
"What step 10 takes,
and from which document").

What it costs:
the route is code no person meets.
Its first live run is the migration's own road test,
which is the least forgiving moment for a first run of a path that takes a snapshot,
a clone,
a claim,
a lock and a record.
The accepted question's sentence,
that step 10
"builds the first working `git worktree add` for a repository on ZFS",
stays unmet until then.

### B. Serve any checkout that is its own dataset's root

The leaf comparison goes,
and the mount at the path must be the dataset's own mounted root.

What it delivers:
nothing that A or C does not already deliver.
Every checkout the sweep reconciles is either the design's dataset,
which A already serves,
or a dataset a root-owned file named,
which C serves.
B changes the add's half of the gate and leaves the root side's half,
so its extra reach is empty.

What the person meets for the example on `main`'s code with only B changed:
the add opens the device and reads the checkout,
the pool and the container,
judges the delegation,
and then `admit_the_handler` refuses,
because no record names this checkout:

```text
no handler readiness report stands for this boot, so no attach is admitted
(/run/...). Ways forward: start the boot unit's `boot-reconcile` and run the add again
```

(`add_preflight.rs`:128 to :133).
`boot-reconcile` on that machine answers the `NoneMounted` sentence,
so the way forward cannot be followed.
The person is sent to a command that cannot succeed,
which is a closed loop rather than a refusal,
and they have lost the worktree A would have given them.

What stops it:
the root side has no name to find the checkout by.
The build round named the only candidate the code carries,
each clone's origin,
and that candidate hands the account the choice of which dataset root reconciles and mounts,
which is exactly the bound the fixed leaf name provides
(`cli-git-zfs-worktree-design-proposal.md`,
"Scoped delegation":
`:335 to :345`).
Every other discovery rule with no root-written name
(a mount under `/home` that holds both a `.git` folder and a `.claude/worktrees` folder,
every dataset at depth two or more under the pool)
is steered by an account in the same way.

What it costs:
one deleted comparison,
one added comparison,
and a regression:
every add on this machine's shape,
a ZFS checkout that is a subtree of its dataset,
falls through today and is refused under B,
because the gate stops answering `Ok(None)`.
`DGT` would also ask B's refusal for a way forward,
and the only one available is C's file.

### C. Serve the checkout the root-owned configuration file names

The add reads the same file the sweep and the probe read,
`/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc`,
and serves an add only when the repository's root is its `cloneMounts.checkout`.
The dataset and the container come from `cloneMounts.checkoutDataset` and `cloneMounts.parent`.

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

What the person sees for the example,
once the file is in place,
the checkout's mount is private,
the handler is installed and `boot-reconcile` has published this boot's record:
the route runs,
the snapshot lands on `tank/home/u/work/Monochromatic`,
the clone lands in the container,
and the add prints the filled note (`FILLED_NOTE`,
`add_route_run.rs`:45).
A container that does not answer under the name the file gives is refused before the claim,
with the preflight's sentence that the route's layout is half-made
(`add_preflight.rs`:91 to :96).
A wrong dataset name is refused before the claim too,
at the dataset identity read (`add_preflight.rs`:89),
unless it names another existing dataset the account is delegated over,
in which case it is caught at the frozen-content comparison after the snapshot and the clone.

What stops it:
no disqualifying problem this round found,
with four limits to state.

**One checkout per machine** under the current record,
as the heading "The two axes this choice turns on" sets out.

**The file's names are text**,
so a dataset renamed or replaced since the file was written is not refused at the gate,
and a file rewritten between the boot's read and the add's read can name another dataset at the same path and in the
same pool,
because the record proves the path and the pool and nothing else
(Finding 2).

**No file names this repository's checkout unless setup writes one.**
The step 8 record says the setup writes both names and that
"no person edits that file by hand"
(`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
"The datasets the sweep acts on").
The example's dataset is not one step 19 creates,
so setup needs an adopt mode for it,
which is the writer E describes.
C and E therefore differ more in where the names live than in whether a root-run writer exists.

**With no file,
or a file that does not name this repository,
the add keeps the ladder.**
The accepted answer of 2026-11-22 is "no file needed"
(`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
"With no configuration file"),
so a missing file is not an error and the route simply does not serve the add.

What it costs:
the add's own read of the file,
with `read_file_only_root_may_write_owned_by(path, 0)`
(`file_only_root_may_write.rs`:162,
and the wrapper at :140),
which already takes the owner and which `doctor` already calls with the same two terms
(`handler_checks.rs`:329 to :342);
one refusal and its control for a file that exists and fails those terms;
the setup that writes the file,
which is step 19's and must land before the route reaches anyone,
plus the adopt mode above for a dataset step 19 did not create;
and the own-root check on the pair the file holds,
which is the gate fix the heading
"The checkout path must be its dataset's own mount point" describes;
and the refusal of a checkout with uncommitted changes until step 11,
which the heading "A checkout with uncommitted changes is refused" states.

The file's mode needs a word of its own:
`doctor` accepts `0600` and the account cannot read it,
so a root-owned `0644` is the mode this needs,
and the add's read must not assume `doctor` would have reported a mode it cannot use.

### D. Make the delegation the gate

The gate widens,
and `check_delegation` becomes what refuses an unmanaged checkout.

What it delivers:
nothing B does not,
and it is the weaker half of the same mistake:
D opens the add and leaves the root side unable to name the checkout,
then adds a device read and a delegation judgement to every ZFS add in a repository of any dataset,
including the ones the ladder serves today.

What the person meets for the example:
B's sentence in the end,
with the delegation finding printed before it on a dataset that carries none.

What stops it:
the root side,
exactly as for B,
and the fact that the delegation cannot stand in for a name:
a checkout whose dataset carries the design's nine grants is still a checkout the sweep never reconciled.

What it costs:
one more device read on every ZFS add,
the same empty delivery as B,
the same regression of the ladder,
and the refusal of a checkout with uncommitted changes until step 11.

### E. Serve a checkout a root-run adoption command records

A root command records,
per repository,
the checkout path,
its dataset and GUID,
the container and the pool,
in a root-owned record the add,
the sweep and the probe read.

What the person sees for the example:
the same as under C,
after they or root has run the adoption command once instead of setup writing a file.

What it delivers:
C's route for one checkout,
and one thing C does not:
the names are identities a command proved when it wrote them,
so a dataset renamed,
replaced or re-created since is refused at the gate,
where C's text is checked only at the points the work has already been done.
Its stated reach of one checkout per repository needs the readiness-record change
the heading "The two axes this choice turns on" describes,
so E does not deliver "many checkouts" until then either.

What stops it from being a separate design:
nothing about the add.
Its difference from C is the writer and the proof,
and the sweep at boot reads the same folder either way.

What it costs:
that command,
its proof of the dataset and container it records,
its grammar and refusals,
its controls,
a setup step,
a reader in every later step that names a checkout,
the readiness-record change for its own stated reach,
and the refusal of a checkout with uncommitted changes until step 11.
The record must also be trusted by the removal and by `doctor`,
which the plan builds at steps 14 and 18.

### G. Serve the checkouts a root-owned file lists

C's file,
with `cloneMounts` holding a list of checkouts rather than one.

What the person sees for the example:
the same as under C.

What it delivers:
C's route for one checkout,
and for two checkouts it delivers nothing more until the readiness record can name two,
which is the same design change E needs.
The list is a future delivery bought now.

What stops it:
the one-record-per-machine bound:
one record file serves the whole machine
(`READINESS_DIRECTORY`,
`clone_mount_readiness.rs`:43,
and `record()`,
`clone_mount_readiness_file.rs`:227),
and one record names one checkout (`ReadinessRecord`,
`clone_mount_readiness.rs`:227 to :246),
so a machine with two checkouts needs one record per checkout,
which is a change to a record the project decided,
or a second file,
which is a change to the root side's own layout.

What it costs:
the file's grammar,
the sweep's loop over checkouts,
the probe's refusal of two checkouts,
the readiness record's shape,
and the refusal of a checkout with uncommitted changes until step 11.

### H. Do the migration now

Not a change to the gate:
root moves the checkout onto its own dataset with the design's names,
and A's gate then serves it.

What the person sees:
everything C delivers,
plus the design's own layout,
the immutable covered entry,
the private propagation,
`snapdir=disabled`,
and a checkout that no later step has to migrate.

What stops it from being this choice's answer:
the migration's own recovery depends on steps 12 and 18,
which are later in the plan:
unmounted and pruned worktrees have no reader,
and `doctor` has no live caller,
so a migration interrupted in the middle has nothing a person can read.
It also cannot be done by an agent at all:
the design puts it in the owner's hands,
with every session stopped.
It is therefore stated as its own question at
"H,
put to the owner as its own question".

What it costs:
steps 12,
18 and 19 as they stand,
a root-run move of a live checkout,
and the refusal of a checkout with uncommitted changes until step 11.

On this host H is the only option that reaches the route:
the checkout is a subtree of the shared home dataset,
so the private propagation H brings is a precondition of every other route option here as well.

### Two add-ons, not alternatives

Two questions sit beside the six options and compose with any of them.
Neither belongs in the ranking,
because neither is a way of reaching the route:
one changes what the route says when it does not serve an add,
the other changes what it does when it serves one it cannot finish.

#### Add-on A2: say why the route declined

The same gate as A or C or E or G,
plus one line from the add when the gate answers `OtherDataset` or `NotZfs`,
naming the dataset found and the way forward.

What it delivers:
the person learns that a route exists and why it did not serve this add.
Under A alone the remedy has to be the migration;
under C,
E or G it is the file entry,
which works.

What stops it as an alternative to A:
the remedy the code has today is a file entry the add's own gate would still ignore under A,
and a line whose way forward is known not to work is worse than silence (`DGT`).
That is why A2 composes with C,
E and G and not with A.

What it costs:
one field on `CheckoutMount` to carry the refusal,
one sentence,
and its control.

#### Add-on W: what an add does on a checkout with uncommitted changes

This is the question the high finding leaves open,
and it is separable from every option above.
Three arms:

- **W1,
  refuse as today.**
  The accepted A0 stands.
  What it delivers:
  nothing to the person;
  the refusal is a fact.
  What it costs:
  the worktree A would have given them,
  and a sentence that names no way forward.
- **W2,
  fall through to the ladder for exactly this refusal.**
  What it delivers:
  the worktree the person asked for,
  from the route's own decision point,
  and it composes with C,
  E,
  G and H.
  What it costs:
  a re-decision of A0's own sentence,
  which chose the refusal over the ladder's rung 3,
  and the slow full copy that choice was made to avoid;
  and it must not fall through for any other preflight refusal,
  so the arm needs its own narrow condition and its own control.
- **W3,
  refuse and name the way forward.**
  What it delivers:
  the one thing the person needs to act,
  which is that the route takes a clean source and that committing or stashing is what makes this checkout one,
  and that step 11 removes the condition.
  What it costs:
  one sentence and its control,
  and it leaves the worktree unsaved.

They compose:
W3 belongs whichever arm the project picks,
because the current sentence breaches `DGT` on its own terms.
So the ranked question is the first two,
and W3 is a defect fix either way.

Ranking:
**W2 > W1**,
because W2 delivers the worktree the person asked for and W1 delivers none.
W2 is ranked first with its cost stated,
not because the cheaper arm should win:
A0's ground was a slow copy,
and a slow copy is what the person asked for.

### The ranking for choice 1

`E > C > G > A`.

B and D are not ranked.
They deliver nothing A and C do not,
because every checkout the root side can name is either the design's dataset,
which A serves,
or one a root-written name points at,
which C serves;
and they take away the ladder's worktree on every other ZFS checkout.
This is stronger than the first revision's ground,
which said they could not be completed:
they can be completed,
and completing them buys nothing.

- **E beats C**:
  both reach one checkout,
  and E's name is an identity a root command proved when it wrote it,
  so a dataset renamed or replaced after the record was written is refused at the gate;
  C's name is text re-read on every add,
  and the record it is checked against proves the path and the pool and nothing else.
- **C beats G**:
  they deliver the same reach today,
  one checkout named by text,
  and G's list buys nothing until the readiness record can name a second checkout,
  which is the same design change E needs for its own stated reach.
  G's extra delivery is a future one,
  and the price is the record's shape and the sweep's loop.
- **G beats A**:
  G delivers the route's snapshot and clone,
  the checkout left where it is,
  the clone's own dataset,
  the mount at the slot,
  the record and the lock;
  A delivers the ladder's block-sharing copy inside the source's dataset,
  with no dataset of its own and nothing for steps 14 and 16 to read.

One limit on that order,
stated as the cost it is:
until step 11,
A delivers a worktree to a person whose checkout has uncommitted changes,
and E,
C and G deliver a refusal.
How much that matters is a fact about how people work,
which this round did not measure and cannot:
if the owner weighs the period before step 11 equally,
A moves up,
and the add-on W is the lever.
After step 11 the order above is unaffected.

### Whether choice 1 is separable from the other two

Yes.
Choice 1 decides which adds meet the route;
choices 2 and 3 change what the route does on the adds it already serves.
A project that answers choice 1 with A can still build 2C and 3C,
and the two would run on the first add after the migration,
which is a testable state and not a dead one.
Nothing in choices 2 or 3 needs to know which option of choice 1 was chosen,
with one exception worth stating:
2C's device read and 3C's grant judgement both live in `preflight_the_add`,
so they meet in one function and the order of the two builds matters to a merge,
not to the design.

### H, put to the owner as its own question

H is not a fourth answer to this choice,
and it is too large to leave unasked:

> Step 10's route reaches nobody until step 19's migration moves the checkout onto its own dataset,
> and step 10 is the step whose question says it builds the first working `git worktree add` for a repository on ZFS.
> Should step 10 land with the route unreachable (A),
> or should the project pull steps 12,
> 18 and 19 forward so the
> migration is the first thing that runs (H)?
> H is the only option that reaches the route on this machine,
> because this checkout is a subtree of the shared home dataset,
> and it needs the owner's own hands for the move and for a session-free window.

The two arms are priced above at A and H.
H's cost is steps 12 and 18 built before their turn,
which the plan's own order puts after step 10 for reasons of recovery.

## Choice 2: what the add prints beside a standing removal plan

### What the add reads today, and what it prints

`refuse_while_a_plan_names_the_clone` (`plan_folder_read.rs`:160)
lists the plan folder,
takes only entries whose name is a clone's canonical decimal GUID,
refuses when a plan's `cloneGuid=` or `cloneDataset=` line names this clone
(:127 to :141),
and refuses a file it cannot read.
Its sentence is one constant,
whatever the plan says
(:213 to :220):

```text
a removal's plan at <path> names the clone this add would fill, so the add stops
before its first mutation and changes nothing. Ways forward: run the removal this
plan belongs to, which resumes from the plan and ends it; then run the add again at
the slot.
```

What a person reads today is not that sentence.
It is the `Debug` value of the stop,
as the shared fact "What the add prints today is a Rust `Debug` value" sets out,
and that is a defect every option here pays for.

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

One more clause is part of V24 and absent from every option the first revision designed.
Where the plan's tuples are all `planned`,
so that the plan witnesses nothing destroyed,
the refusal also names the cancel command,
"so a person who wants the worktree kept is not told only how to finish removing it"
(`cli-git-zfs-add-binding-record-design.md`:6024 to :6028).
That clause came in with `57d1e966f6` (2026-11-23T19:20:03-05:00),
the next commit after the `aa6a5f18f0` the decision record accepts,
and the decision record's own words are "at `aa6a5f18f0` or its next commit"
(`doc/decision/cli-git-zfs-step-10-add-time-lock-and-binding-record.md`:6).
So the clause is accepted,
and the sentence the owner saw when accepting says only
"names the command the plan design's 'Which resume path a removal takes' gives for the plan's head word".

### The decided command for every state

Two facts decide the command,
and together they make fifteen states.
The read tabulated them by running `command_for` (`review-probes-output.txt` in the read's `keep/`);
this round re-read the function and got the same answers
(`add_state_commands.rs`:238 to :252):

- **the clone stands** (a child of the container carries the recorded GUID,
  under any name):
  the removal,
  for every head word;
- **the clone is proved gone**,
  no child carrying the GUID:
  the settling command for `present` and `refused`,
  the removal for `requested`,
  `gone` and `unknown`;
- **the absence cannot be proved** (the listing refused or came back partial):
  the settling command for every head word but `gone`,
  and for `gone` the finish pass,
  whose line names no command and says to wait for the pool.

That is the removal in eight of the fifteen states,
the settling command in six,
and the finish pass in one.
The single constant sentence is right in eight and wrong in seven.

### 2A. Print the removal for every state, as today

The constant sentence,
with the defect above fixed so the sentence is what a person reads.

What the person sees,
in V24's own example
(a removal refused at its second deferred-origin check leaves a plan at `head=present` beside the live clone):
the removal,
which is the right command.

What the person meets whenever the clone does not stand,
which this option cannot tell:
the same sentence,
and then the removal refuses.
Where the clone is proved gone and the plan's head is `present` or `refused`,
the removal refuses with the outside destroy and names the settling command;
where the pool could not prove the absence,
it refuses with the failed read and names the settling command,
except at a head of `gone`,
where the finish pass is the route and no command accepts the state.
So in six of the seven wrong states the person runs one command that refuses and tells them the next,
and in the seventh,
`gone` beside an absence the pool cannot prove,
they read a refusal that names no command.

What it delivers:
one right command in the two ordinary states,
and a wrong one that corrects itself in six more.

What it costs:
the printed-text fix from the shared fact,
with its control,
the knee of the fifteen-state table above,
and the one cell where the correction names no next step.

### 2B. Read the head word only

The add parses `head=` and calls `command_for` with `clone: None`.

What the person sees,
in V24's own example:
the settling command,
because that is what `command_beside_a_standing_plan` answers for `present` when no clone was looked for
(`add_state_commands.rs`:238 to :251,
and the read's probe line
`head=present clone=not-looked-for => TheSettlingCommand`).
The settling command refuses a binding whose clone stands,
and names `git worktree remove`.
So the add sends the person to a command that refuses,
in the one scene V24 uses to justify the rule.

What stops it:
V24's own example.
Disqualified.

What it costs:
the printed-text fix from the shared fact,
with its control,
and one parsed line and its refusals.

### 2C. Read the head word and the pinned parent, and print `command_for`

The add parses `head=`,
enumerates the container's immediate children,
and builds the `ClonePresence` `command_for` reads:
`Stands` when a child of the container carries the recorded GUID,
whatever its name;
`GoneProved` when the recorded name is absent and no child carries the recorded GUID;
and `GoneUnprovable` when the listing refused or came back partial.
It then prints the one command for that pair of facts,
and,
where the all-planned clause applies,
the cancel command beside it.

What the person sees for the example,
state by state,
is the one command the pair of facts allows:

- `present` and the clone stands:
  the removal,
  as V24's example asks;
- `present` and the clone is proved gone:
  the settling command,
  which accepts it;
- `present` and the absence is unprovable:
  the settling command,
  which refuses a proved-destroyed clone and names
  the read that failed;
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
- the name carrying another clone's GUID:
  no command settles it,
  and today no `ClonePresence` arm says so;
- a `head=` line absent or carrying a word outside the five:
  a refusal naming the plan and no command.

What it delivers:
one right command and never a wrong one,
in every one of the fifteen states,
plus the way out for a person who wants the worktree kept.

What stops it:
nothing that argues against the option;
three gaps to close in the build:
the reused-name arm `ClonePresence` does not have,
the `head=` refusals,
and the all-planned clause,
which needs the `tuple=` lines and so widens a read V24 calls "the smallest one that can refuse".

What it costs:
the printed-text fix from the shared fact,
with its control;
the `head=` line in the add's reader with its refusals;
the plan read moved after `OpenedDevice::open`
(`add_preflight.rs`:85 and :88),
because presence needs the device,
which also means a plan that stops the add no longer stops it before the device opens;
an adapter from the preflight's opened device to the `DatasetReader` seam `each_child` takes
(`sweep_dataset_reader.rs`:302 to :320),
because `ZfsDatasetReader` owns its own device (:191 to :205);
one stats read per child for the GUID match,
up to the 4,096-child bound (`MAXIMUM_CHILDREN`,
:90),
and a statement of that bound beside the read,
because the step 8 record treats growth on a waiting path as a defect;
a fourth `ClonePresence` arm for the reused name;
the `tuple=` read and the cancel command for the all-planned clause;
and both plan reads carrying the same answer
(`add_preflight.rs`:85 to :86 and `add_route_run.rs`:230),
or the same state prints two different sentences.

### 2D. Print the facts and no command

The add prints the plan,
the head word,
and whether the clone answers,
and says the command is the removal's or the settling command's.

What stops it:
V24 asks for the command,
and `DGT` asks a diagnostic to name what a person runs.
Disqualified.

### 2E. Print both commands with the condition that chooses

The add prints the head word's two candidate commands,
with the fact that decides between them,
and the person looks at the container.

What the person sees for the example:
two commands and one question,
instead of one command.

What it delivers:
never a wrong command,
in any of the fifteen states,
and no device read.

What stops it:
nothing that argues against the option,
and it hands the person a judgement the wrapper can make with one read it already has the device for.

What it costs:
the printed-text fix from the shared fact,
with its control,
the `head=` line and its refusals,
and a two-command sentence whose second half is the question the wrapper could answer.

### 2F. Print one command by the head word, and both only where the head word cannot decide

The option the first revision did not consider.
The add parses `head=` and nothing else.
For `requested` and `unknown` it prints the removal.
For `present`,
`refused` and `gone` it prints both the removal and the settling command with the fact that chooses,
because those three head words are exactly the ones whose answer turns on the clone.

Why `gone` is in the second group and not the first:
a `gone` plan beside a clone the pool cannot prove gone is the finish pass,
and the removal names no command there,
so a head-word-only rule that printed the removal for `gone` would leave that person with a refusal and no next step.
The read's variant printed the removal for `gone`;
this is the correction,
and it is one head word.

What the person sees for the example:
one command in the six states whose head word is `requested` or `unknown`,
and two commands and a question in the nine whose head word is `present`,
`refused` or `gone`.

What it delivers:
a right command in thirteen of the fifteen states and never a wrong one that has no next step;
the two wrong states are `requested` and `unknown` beside an unprovable absence,
where the removal refuses with the failed read and names the settling command.
It needs no device read,
so it keeps the plan read where it is,
before the device opens.

What stops it:
nothing that argues against the option;
it asks the person a question in the nine states where the head word cannot answer,
and one of those nine is V24's own example.

What it costs:
the printed-text fix from the shared fact,
with its control,
the `head=` line and its refusals,
two sentence forms instead of one,
and the two states above.

### The ranking for choice 2

`2C > 2F > 2E > 2A`.

2B and 2D are not ranked:
2B sends the person to a command that refuses in V24's own example,
and 2D names no command at all.

- **2C beats 2F**:
  both never name a wrong command that has no next step,
  and 2C names exactly one command in every state,
  where 2F asks the person a question in the nine states whose head word is `present`,
  `refused` or `gone`,
  including the ordinary V24 scene.
- **2F beats 2E**:
  both never name a wrong command,
  and 2F names exactly one in the six states whose head word decides alone,
  where 2E names two in all fifteen.
- **2E beats 2A**:
  2E never names a command that refuses,
  and 2A names a removal that refuses in seven states,
  one of which names no next command.

Every option here also carries the printed-text fix described in
"What the add prints today is a Rust `Debug` value",
so the first thing any of them delivers is the sentence the design thinks it already prints.

### Whether choice 2 is separable

Yes.
Its change is the plan read and the two call sites,
and it needs nothing from choices 1 or 3.
Its value depends on choice 1 only in the sense that a person meets it only on an add the route serves.
One overlap with choice 3 is worth stating:
2C's device read and the grant judgement of choice 3 both change `preflight_the_add`,
and 2C's move of the plan read after `OpenedDevice::open` decides which order the two changes merge in.

## Choice 3: which grants the preflight judges

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
  and the line names the `zfs unallow` for each;
- `everyone`:
  a route permission is granted to everyone,
  and the line lists them;
- unknown:
  a route permission reaches the account through a named set,
  and the members are not in the reply;
- `grant`:
  a decided permission is missing,
  and the line names the `zfs allow` for each;
- `scope`:
  a decided permission is effective but wider than the design scopes it,
  and the line says "the narrowed command is printed for each" and prints none,
  which is the sixth finding this round did not fix.

`refuse_unless_the_decided_ones` refuses for all five
(`add_grants.rs`:105 to :114),
because it asks only whether the report passed.

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
`delegation_checks.rs`:431 to :439):

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

### 3A. Judge all nine, as today

What the person sees for the example on a machine that has the six and is missing only the clone removal's
`destroy`:
the add refuses before the claim with
(`add_grants.rs`:110 to :113,
and the read's probe line in `review-probes-output-three.txt`):

```text
the ZFS clone route cannot run with this account's delegation: zfs: delegation:
finding: grant: 1 decided permission(s) are not granted to the account: `destroy` on
`tank/home/u/work/monochromatic-git-worktree-clone-datasets` (zfs allow -d -u 1000
destroy tank/home/u/work/monochromatic-git-worktree-clone-datasets). Ways forward: run
the command the finding names, then run the add again; the add changed nothing.
```

They run one `zfs allow` as root,
and the add then runs.
The permission they granted is one the add never uses;
it is the clone removal's,
four steps later for the removal,
and six for the destroy that step 16 makes.

What it delivers:
one machine's delegation is either the decided shape or it is not,
so the add and `doctor` judge the same bytes.

What it costs:
a root command and a second add,
for a permission the add's own commands do not use,
in a state where the person's first add would otherwise have finished.

What it does for each outcome kind:
refuses all five,
as today,
including the three that say nothing about a missing permission.

### 3B. Judge only the six the add's own steps use

The preflight refuses on the six,
and the other three are left to the step that uses them
or to `doctor` at step 18.

What the person sees for the example,
missing only the clone removal's `destroy`:
the add finishes,
and the missing `destroy` meets them at the first removal.
The design requires that removal's preflight to read the same grants:
"keep descendant `destroy` and require `doctor` and removal preflight to verify the scoped effective grants"
(`cli-git-zfs-worktree-design-proposal.md`:1556 to :1557).
With that read built,
the first removal refuses before any plan is written,
naming the `zfs allow` command.
Plan step 14's own build list does not name that read,
although step 14 builds the plan,
the intent and the refusal before the first delete
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 14:
Removal:
the lineage proof,
the plan and the removal intent").
So on today's plan the refusal arrives from the kernel instead,
at step 16's first destroy,
after the plan is written and the intent recorded:
a snapshot destroy in the batch,
or the head destroy when the clone has no snapshot of its own.

What the plan holds then depends on which request was first,
and neither answer makes the add's sentence wrong:
a clone with its own snapshots has them refused first,
in the snapshot batch,
and the plan stands with its attempt lines and `head=present`;
a clone with none has the head destroy refused,
and the plan stands at `head=refused`.
In both the clone still stands,
and `command_beside_a_standing_plan` answers the removal for every head word beside a standing clone
(`add_state_commands.rs`:239 to :241),
so the add at that slot names the removal,
which refuses again on the same missing grant.
The first revision's 3B example had all three of those wrong.

What it delivers:
the add the person asked for finishes,
and the refusal it defers lands where the missing permission is used.

What it costs:
the six as a second name for one judgement,
a decision about which ground decides a later step's refusal,
and the correction to plan step 14 that the design already requires and the plan's list omits.
Without that correction,
3B's deferred refusal arrives with a plan standing and a person who must read two commands to learn one grant is
missing.

What it does for each outcome kind:
refuses `permission`,
`everyone` and unknown as today
(none of the three is about which six),
refuses a missing permission in the six,
and leaves a missing permission in the removal's or the origin group to the step that uses it.
It refuses `scope` as today,
which is the kind a breadth defect arrives as,
and the design's option scopes descendant `destroy` precisely to catch one.

### 3C. Judge the nine and split the verdict

One judgement,
unchanged:
`check_delegation` still answers for the nine.
The difference is what the preflight does with the answer,
by group:

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
- a missing permission in the removal's group or the origin group is printed as one line,
  and the add goes on.

What the person sees for the example,
missing only the clone removal's `destroy`:

```text
monochromatic-git-policy-cli: this add will finish, and the wrapper cannot destroy
this clone until you run, as root:
zfs allow -d -u 1000 destroy tank/home/u/work/monochromatic-git-worktree-clone-datasets
The add changed nothing about that yet.
```

and then the clone,
the mount and the filled worktree.

What it delivers:
the add finishes,
and the person learns which permission is missing and what command settles it,
before the removal asks them for it.

What stops it:
nothing that argues against the option,
with one thing to state:
the split is a rule about which step uses which permission,
and the proposal's three groups are its ground,
not a measurement.
An arm in the guest,
with the six granted and one of the three withheld,
should confirm that the add's own requests never fail for want of the withheld one,
before this is built as more than a proposal.

What it costs:
a way to name the missing permissions in a form the caller can read,
which means `check_delegation` or a sibling giving back the values rather than only formatting them into a sentence;
the group table,
which needs `check_delegation`'s own order to survive it,
because the check stops at the first kind it finds;
the `scope` finding's missing command,
which a refusal for a removal-group `destroy` now depends on;
the warning line and its two controls (one where it stays quiet,
one where it prints);
and the table's own expiry,
which is the next paragraph.

The expiry is the sharpest cost.
The add's own fallback destroys the clone and the origin snapshot
(A22 and A23,
`cli-git-zfs-add-binding-record-design.md`:2284 to :2318),
and "the route's grants carry the destroy" is the design's own words in A22.
No `destroy` appears in any non-test file of `worktree_clone_add_route/` on `main`,
so the table holds today.
After step 11 it does not:
the add itself then uses the removal group's `destroy` on its own failure path,
and 3C's warning would tell a person the wrapper cannot destroy a clone the wrapper is about to try to destroy.
So the group table is a constant step 11 rewrites,
the warning's wording must come from the table and not be a fixed sentence,
and the two builds have to meet.

### 3D. Judge only the forbidden nine

Missing required permissions are left to the kernel's own `EPERM` at the first request that needs them.

What the person sees for the example on a machine missing `snapshot`:
the add takes the claim,
writes the binding,
registers the worktree,
takes the lock,
and then the snapshot is refused
(`add_zfs.rs`,
`snapshot_the_checkout`'s sentence).
The claim,
the slot,
the registration and the lock stand,
and the next add resumes on them.

What stops it:
nothing that argues against the option,
but it moves a refusal the wrapper can make for free to a point where it has already written four durable objects,
which is the opposite of A0's reason for existing.

What it costs:
nothing to build beyond deleting a call,
and the state a person then reads.

What it does for each outcome kind:
refuses `permission` and `everyone`,
and leaves `grant`,
`scope` and unknown unjudged,
so a missing permission waits for the kernel,
a too-wide removal `destroy` is not caught before the add at all,
and an account whose delegation is unproved through a named set goes on.

### The ranking for choice 3

`3C > 3B > 3A > 3D`.

- **3C beats 3B**:
  both let the add finish when only a later step's permission is missing,
  and 3C also names that permission and its command before the person needs it,
  where 3B leaves them to meet the refusal at the removal,
  with the plan already written on today's plan text.
- **3B beats 3A**:
  3B finishes the add the person asked for;
  3A stops it before the claim for a permission the add's own path never uses.
- **3A beats 3D**:
  both refuse where a permission the add needs is missing,
  3A refuses before the claim with the command that settles it,
  and 3D refuses after the claim,
  the binding,
  the registration and the lock.

Three limits on that order,
each a reason to ask rather than to answer:

- 3C's advantage over 3B expires in part at step 11,
  so a project that cares mainly about the build before step 11 should weigh 3C as a stopgap with a rewrite owed;
- 3C needs the `scope` finding's command to be a person's next step,
  and today that finding promises one and prints none;
- 3B is firmer than the first revision said,
  because the design's removal preflight would refuse cleanly before any plan,
  and it needs plan step 14 to gain the read the design already requires.

If the owner wants the smaller promise,
3B is the safe pick until step 11,
and it needs the step 14 correction stated above.

### Whether choice 3 is separable

Yes,
with one dependency that is not on the other choices:
its table expires at step 11.
Nothing in it needs choice 1's answer or choice 2's,
and a project that answers choice 1 with A can still build it.
It shares `preflight_the_add` with 2C,
which is a merge matter.

## Findings this round did not fix

This round changes no wrapper code,
so each of these is described and left for the round that takes the code it lives in.

### Finding 1: the gate does not prove the checkout is its dataset's own root

`the_checkout_dataset_at` takes the mount holding the path
(`the_designs_datasets_in_the_mount_table.rs`:229 to :235)
and compares the dataset's leaf name (:236 to :238).
It never compares the mount's own mount point with the path,
and the `MountHoldingAPath` the gate receives has a `root` field
(`mounted_dataset_instance.rs`:85) that `holding_of` never fills (:182 to :184).
So a checkout at `/u/repo`,
on a dataset named `monochromatic-git-checkout` mounted at `/u`,
passes the gate,
and a bind mount of a subtree at the path passes it too.
On `main` the add is then refused at the readiness record,
so the defect is latent;
it becomes reachable through any root-written name that pairs a path with a dataset,
which makes it a precondition of C,
E and G and not of B alone.
On this host a file naming the home dataset would make the route snapshot and clone all of `/home`.
The code already has the parts for the fix:
`instances_of` (`the_designs_datasets_in_the_mount_table.rs`:137) and `shows_the_datasets_own_root`
(`mounted_dataset_instance.rs`:91).

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
every option that widens the gate unmasks it,
because the add then takes a dataset name from somewhere the root side's record does not compare.
Closing it means a field in a record the project decided,
which is a design change and not a build's to make.
The same record's one-checkout shape is what stops C,
E and G from serving two repositories,
which is the second axis of choice 1.

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
This removes that report's reason for ranking C first.

Two edges the read found and this round confirms:

- `doctor` accepts a `0600` file the account cannot read,
  so `doctor` can pass where the add's read would fail with `EACCES`.
  The mode this feature needs is a root-owned `0644`,
  and no check in the tree says so;
- the `Owner` refusal's sentence says
  "its owner is user {found} and this program runs as user {wanted}"
  (`file_only_root_may_write.rs`:107 to :108),
  which for an add run as account 1000 with `wanted = 0` reads
  "this program runs as user 0" and is false.
  C,
  E and G all meet this refusal path,
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
and the only parts that are not are the add's gate and the probe's fallback.

### Finding 5: the fallback's line names btrfs to a ZFS checkout

The line an add prints when the snapshot route did not serve it is
`this worktree was not made as a snapshot, so it took longer, because <the source> is not a btrfs subvolume`,
and the reason behind it is `snapshot_refusal`
(`worktree_snapshot_source.rs`:234 to :247),
which knows two causes:
the source is no subvolume root,
and the source and the destination are not on one btrfs filesystem.
On a ZFS checkout both answers are beside the point:
the design's snapshot route is step 10's,
and what kept it away there is the dataset leaf name,
the file's names,
the shared mount or the missing handler.
So the one line a person on ZFS reads about the missing snapshot names a filesystem they are not on,
and `DGT` asks a diagnostic to name the cause the person can act on.
The fix belongs to the round that takes this line,
and the add-on A2 does not settle it either:
a second line saying the ZFS route declined would still have to name a remedy that works.

### Finding 6: the too-wide delegation finding promises a command and prints none

`finding("scope", ...)` says
"the narrowed command is printed for each"
and joins no command into the line
(`delegation_checks.rs`:519 to :528).
Its control asserts the field name and the two dataset names and nothing else
(`delegation_checks_tests.rs`:115 to :125).
A person who meets a removal-group `destroy` granted on an ancestor reads a promise and no command,
and choice 3's 3C makes that finding the refusal for exactly that permission.
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
  and choice 2's 2C is what makes it read more.

## What was probed, and what was not

This round ran no build,
no test and no probe of its own.
Its evidence is the code on `main` at `06d3e542b8`,
the records,
and the probe outputs the read kept in its `keep/`:

- the add's printed form of a plan stop (`review-probes-output-debug-format.txt`);
- the fifteen-state command table,
  the example's three file cases,
  and the gate on a subtree and a bind mount (`review-probes-output.txt`);
- the three delegation sentences,
  including the too-wide one (`review-probes-output-three.txt`);
- the package's two control runs the read repeated,
  `125 passed` and `83 passed` (`rerun-controls-first-command.txt` and
  `rerun-controls-second-command.txt`).

What this round did itself:

- re-read every call site and record the read cites,
  and corrected the citations the read reported as drifted
  (`clone_mount_readiness_admission.rs`:132 and :138,
  `add_grants.rs`:85 and :86,
  `add_zfs.rs`:56),
  adding the four refusal sentences' lines,
  which the first revision cited by range and no more;
- measured this host's mount table for the shared tag on `/home` and its nested datasets;
- measured this repository's tree at `06d3e542b8`;
- read plan steps 10,
  11,
  14,
  16,
  18 and 19,
  the two accepted designs,
  the destruction plan design's grammar and resume headings,
  and the step 8 record.

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
- no count of how often a checkout has uncommitted changes,
  which is the one number that could move A above the widening options in the ranking.

## What the next round would do

If the project chooses C,
E or G:
the add's own read of the file or the record,
the gate rebuilt on the names it holds,
the probe's `startProbe.dataset` fallback,
Finding 1's own-root comparison in the same round because that gate is the one being opened,
and a statement of what the add does with no file.
If the project chooses E or G,
the readiness record's per-checkout shape is a design question first.

If the project chooses W2:
the narrow fall-through in `run_a_zfs_add`,
its control showing that no other preflight refusal falls through,
and the sentence fix W3 either way.

If the project chooses 2C:
the `stop.sentence()` at both call sites,
the `head=` line,
the fourth `ClonePresence` arm,
the adapter and the child bound,
the plan read moved after the device opens,
the all-planned clause with its `tuple=` read and cancel command,
and both plan reads answering the same way.
If the project chooses 2F instead:
the `head=` line,
two sentence forms,
and the same printed-text fix.

If the project chooses 3C:
a way to name the missing permissions as values,
the group table as a constant step 11 rewrites,
the `scope` finding's command or its sentence,
the warning line,
and the guest arm that withholds one of the three.
