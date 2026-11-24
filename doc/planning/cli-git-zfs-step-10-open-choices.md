# Step 10's three open choices

## Status

A proposal.
Not a decision,
not approved,
and not final:
only the project declares otherwise.
Written by one round on 2026-11-24 in the worktree
`.claude/worktrees/zfs-step-10-open-choices-proposal`,
branch `docs/planning-cli-git-zfs-step-10-open-choices`.
No wrapper code changed,
nothing under `doc/decision/` or `doc/handover/` was touched,
and no container gate,
guest or ZFS-changing command ran.

Where this document proposes a change to code,
the change is described and not built.
Where it reports what a person sees,
it says whether the sentence comes from live code,
from a control this round ran,
or from a design that is not built yet.

The round that this one follows is
`~/temp/agent/zfs-step-10-build-matches-its-accepted-designs-2026-11-24/keep/report-pi-zfs-step-10-build-matches-its-accepted-designs.md`
("the build round's report" in this document).
One of its sentences is corrected in the finding headed
"the file a root side owns is readable by the account",
and the correction changes which option its ranking should have put first.

## What this round was given

The build round made step 10's build match its accepted designs
(merged as `2d9fa02ea5`) and left three choices to the project.
The accepted designs are
`doc/planning/cli-git-zfs-add-binding-record-design.md` and
`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
accepted by `doc/decision/cli-git-zfs-step-10-add-time-lock-and-binding-record.md`,
and the plan is `doc/planning/cli-git-zfs-worktree-implementation-plan.md`.

1.  How far the add route reaches.
    The gate serves only a checkout whose dataset leaf is
    `monochromatic-git-checkout`,
    which only step 19's migration makes,
    while the accepted question says step 10
    "builds the first working `git worktree add` for a repository on ZFS".
2.  What the add prints beside a standing removal plan.
    The command for a state lives in `command_for`,
    and the add's own sentence
    (`plan_folder_read.rs`,
    `names_the_clone_sentence`)
    names the removal for every state.
    The project's choice 18 (V24) asks for the head word's command.
3.  Which grants the preflight judges.
    It judges the whole decided set of nine,
    including the permissions steps 14 and 16 use,
    so an add on a machine missing only a destruction permission refuses
    before the claim.

The build round's ranking of choice 1 was `C > E > A > D > B`,
and its reason for the first pair was that C
"needs no new root command,
record or step".
That is a build cost,
and `AGENTS.md`'s `RBE` forbids ranking by one:
an option is ranked by what it delivers,
and effort is a cost to state beside it.
The same sentence also gives E the one-checkout-per-machine limit,
which is C's:
C's file names one checkout,
and E's records are per repository.
This document ranks by delivery only,
and states each cost as a cost.

## The example, and what is measured about it

Every option is measured on the example the build round used:

- the repository is at `/home/u/work/Monochromatic`,
  and its checkout is the root of dataset `tank/home/u/work/Monochromatic` in pool `tank`;
- the account is user 1000;
- the person runs `git worktree add .claude/worktrees/fix-flaky-test`,
  so the destination is
  `/home/u/work/Monochromatic/.claude/worktrees/fix-flaky-test`;
- the clone container the code names today is the checkout dataset's sibling,
  `tank/home/u/work/monochromatic-git-worktree-clone-datasets`
  (`the_designs_datasets_in_the_mount_table.rs`:255,
  which replaces only the last component);
- the origin snapshot the route would take is
  `tank/home/u/work/Monochromatic@git-worktree-clone-origin-<nonce>`,
  and the clone is
  `tank/home/u/work/monochromatic-git-worktree-clone-datasets/fix-flaky-test`.

Two facts about this machine decide what could be probed here,
and what the person in the example would meet on it today.

`/home/user/Monochromatic` is not its own dataset here:
`/proc/self/mountinfo` holds `zpcachyos/ROOT/cos/home` at `/home`,
so the checkout's dataset leaf is `home`,
and the checkout is a subtree of `zpcachyos/ROOT/cos/home`.
The gate would answer `OtherDataset` for this repository too,
so no run on this machine reaches the route,
under any option this document designs.

This repository's tree is 526,283,607 tracked bytes in 25,241 files,
a mean of 20,850 bytes a file,
from `git ls-tree -r -l HEAD` at `37143a1daf`.
That matters because it decides which rung of the ladder an add takes today:
on ZFS a clean source is cloned only when its mean tracked file is at least
2,600 bytes (`worktree_clone_rung_limits.rs`:199),
and a source with changes only when the clones would carry at least
762,000,000 bytes (`worktree_clone_rung_limits.rs`:118).
A clean add of this repository therefore takes the same-dataset reflink rung,
which asks the kernel to clone each tracked file with `FICLONE` inside the one
dataset,
and a dirty add of it falls to Git's own checkout.

So "the route does not reach this repository" does not mean the person writes
files by hand today:
it means the add is a block-sharing copy in the source's own dataset,
with no separate dataset,
no mount at the slot,
no record and no lock,
instead of the snapshot and clone the route makes.

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

The gate is not the only bound on reach.
Three more facts decide whether a person meets the route,
and none of them is visible in the gate:

- `preflight_the_add` reads a current handler readiness report
  for this boot,
  this pool and this checkout path
  (`add_preflight.rs`:98 and :125 to :137),
  so a checkout the root side never reconciled is refused;
- the root side finds the checkout itself,
  by the same dataset leaf name when no configuration file names one
  (`sweep_configuration.rs`:197 to :200,
  through `the_one_mounted_checkout`,
  `the_designs_datasets_in_the_mount_table.rs`:157),
  and publishes nothing at all when that fails
  (`boot_reconcile_mode.rs`:131 returns before any publish);
- the handler that mounts the clone is the second executable this build ships
  (plan step 5),
  which root installs and starts from a systemd unit and a ZED history event
  (plan step 19),
  so no attach happens on a machine without that install.

The load-bearing consequence is this:
**the root side must be told which checkout to reconcile,
and it must be told in a place only root writes.**
The gate is the visible half of reach;
`admit_the_handler` and the handler install are the invisible half,
and every option here is judged on both.

A person can also read the root side's own advice today.
`read_configuration` refuses a checkout of another leaf name with
(`mounted_dataset_refusal.rs`:214 to :216):

```text
the checkout /home/u/work/Monochromatic is on the dataset
`tank/home/u/work/Monochromatic`, and the dataset this route finds by its own name is
the one whose last component is `monochromatic-git-checkout`; a checkout of another
dataset is another checkout's, which one probe's pass says nothing about. Nothing was
enumerated, attached or recorded, and every clone's mount intent stands untouched.
Ways forward: mount the design's checkout dataset at that path, or write
`cloneMounts.checkoutDataset` in
/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc naming
`tank/home/u/work/Monochromatic`.
```

That sentence is the built code's own answer,
and it is the second half of this
choice:
the sweep already reads `cloneMounts.checkoutDataset`,
and with the path and the dataset both present it asks the mount table nothing
(`sweep_configuration.rs`:187 to :188).
Only the add and the start probe still apply the leaf name.

### A. Keep the gate until step 19's migration

Nothing changes in the wrapper.
The route serves a checkout only after root moves it onto its own dataset.

What the person sees for the example,
measured in a throwaway repository on this host
(the heading "What was probed,
and what was not" names the run):
no line about the route,
because the gate's answer is dropped silently (`add_route_gate.rs`:202);
the add succeeds,
and the wrapper prints the ladder's own two lines,
`copied 0 ignored filesystem entries ... into 1 new worktree`
and,
for a source the clone rung served,
`this worktree was not made as a snapshot, so it took longer, because
<the source> is not a btrfs subvolume`;
and no record folder appears,
so the route wrote no claim,
no binding and no plan folder.
For a clean tree of this size the add is the clone rung's,
and for a dirty one it is Git's own checkout.
If they also run the boot unit,
`boot-reconcile` fails with the sentence quoted at "What decides reach today",
and `doctor` reports nothing about the route,
because `route_checks` has no live caller
(`doctor_zfs_route/route_checks.rs`:112 is called only from controls and the guest arm).

What stops it:
nothing.
It is the shape the plan drew,
and the accepted decision's own text says the route's dataset
"exists only after plan step 19's one-time migration"
(`cli-git-zfs-add-binding-record-design.md`,
the heading `What step 10 takes, and from which document`).

What it costs:
the route is code no person meets.
Its first live run is the migration's own road test,
which is the least forgiving moment for a first run of a path that takes
snapshots,
clones,
a claim,
a lock and a record.
The accepted question's sentence stays unmet until then.

### A2. Keep the gate and print why it declined

The same gate,
plus one line from the add when the gate answers `OtherDataset`,
naming the dataset found and the way forward.

What the person sees:
the sentence quoted at "What decides reach today",
before an add that then runs as it does today.

What stops it:
the remedy the code has is `cloneMounts.checkoutDataset` in a root-owned file,
and with the add's gate unchanged that entry changes nothing for the add,
so the line names an edit that does not settle the person's case.
A line whose way forward is known not to work is worse than silence
(`AGENTS.md`,
`DGT`).

What it costs:
one field on `CheckoutMount` to carry the refusal,
one sentence,
and its control.

### B. Serve any checkout that is its own dataset's root

The leaf comparison goes,
and the mount at the path must be the dataset's own
mounted root.

What the person sees for the example,
on a machine with the container made and
the grants granted:
the gate passes;
the preflight reads the plan folder (nothing),
the slot parent,
then opens the device;
the checkout,
the pool and the container must answer;
the delegation is judged;
and then `admit_the_handler` refuses,
because the root side published no record for this checkout.
The sentence is the preflight's own (`add_preflight.rs`:128 to :133):

```text
no handler readiness report stands for this boot, so no attach is admitted
(/run/…). Ways forward: start the boot unit's `boot-reconcile` and run the add again
```

and `boot-reconcile` on that machine fails with the leaf-name sentence quoted at
"What decides reach today".
So the person is told to run a command that cannot succeed,
which is a closed loop rather than a refusal.

What stops it:
the root side has no name to find the checkout by.
The build round named the only candidate the code carries,
each clone's origin,
and that candidate hands the account the choice of which dataset root
reconciles and mounts,
which is exactly the bound the fixed leaf name provides
(design proposal,
"Scoped delegation",
"Do not assume those snapshots protect against that account or put unrelated ZFS
descendants there").
Every other discovery rule with no root-written name
(a mount under `/home` that holds both a `.git` folder and a `.claude/worktrees`
folder,
every dataset at depth two or more under the pool)
is steered by an account in the same way.
So B cannot be completed as designed:
it opens the add and leaves the root side unable to follow.

Two further problems in B,
each enough on its own:
the condition it adds,
that the mount at the path is the dataset's own root,
is a condition today's gate does not have,
so B would be stricter than A in one place and looser in another,
and a person whose checkout path is deeper than its dataset's mountpoint would
keep the ladder instead of the route;
and the readiness record carries no dataset name,
so a gate built on a name the record does not prove could act on a dataset the
root side never reconciled
(see "the readiness record proves no dataset").

What it costs:
one deleted comparison,
one added comparison,
a sweep discovery rule that does not exist and cannot be made safe,
and the probe's own look-up.

### C. Serve the checkout the root-owned configuration file names

The add reads the same file the sweep and the probe read,
`/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc`,
and serves an add only when the repository's root is its
`cloneMounts.checkout`.
The dataset and the container come from `cloneMounts.checkoutDataset` and
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
without it the probe's `found_source` calls `the_checkout_dataset_at` on the
named path and refuses with the leaf-name sentence
(`probe_configuration.rs`:277 to :288).
`startProbe.checkout` must equal `cloneMounts.checkout`,
or `parse_configuration`
refuses with `CheckoutsDiffer` (`sweep_configuration.rs`:266 to :270).

What the person sees for the example,
once the file is in place and the grants
are granted:
the route runs,
the snapshot lands on `tank/home/u/work/Monochromatic`,
the clone lands in the container,
the sweep publishes a readiness record for
`/home/u/work/Monochromatic`,
`admit_the_handler` compares it with the record's
own fields and admits,
and the add prints the filled note (`FILLED_NOTE`,
`add_route_run.rs`).
A container that does not answer under the name the file gives is refused
before the claim,
with the preflight's sentence that the route's layout is half-made
(`add_preflight.rs`:91 to :96),
and a wrong dataset name is caught only at the frozen-content comparison,
after the snapshot and the clone,
and only because the two datasets' bytes differ.

What stops it:
no disqualifying problem this round found,
with two limits to state:
one checkout per machine under the current grammar;
the file's names are text,
so a typo costs a snapshot and a clone before the
frozen proof refuses it;
and the checkout path is compared as text,
both by the add's own gate and by
`admit_attach` against the record,
so the file must hold the very string Git reports for the top level,
and an add whose checkout is reached through a symlinked home should be measured
before a build relies on that.
A second limit belongs to the record rather than to the file:
the record proves the checkout path and the pool,
and a file rewritten between the boot's read and the add's read can name another
dataset at the same path and in the same pool
("the readiness record proves no dataset").

What it costs:
the add's own read of the file,
which the code already has and which needs the owner argument `0`
rather than the caller's own account
(`read_file_only_root_may_write_owned_by`,
`file_only_root_may_write.rs`:162,
and
"the file a root side owns is readable by the account");
one refusal and its control for a file that exists and fails those terms;
the sweep,
the probe and the boot unit already read the same three keys,
so nothing else changes;
and the setup that writes the file,
which is step 19's and must land before the route reaches anyone,
whichever option is chosen.

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
after they have run the adoption command once instead of editing a file.

What stops it from being a separate design:
the sweep runs at boot with no repository to name,
so the per-repository records must sit in one root-owned folder it can
enumerate,
and the command must write there.
The one other way to give the boot a per-repository name is a unit instance per
repository,
and whatever generates those instances reads a list,
which is G with a writer.
E is therefore C's place of record with a validating writer,
not a different place:
the names root acts on are still names,
and the writer is the part that adds anything.

What it costs:
that command,
its proof of the dataset and container it records,
its grammar and refusals,
its controls,
a setup step,
and a reader in every later step that names a checkout.
The record must also be trusted by the removal and by `doctor`,
which the plan builds at steps 14 and 18.

### G. Serve the checkouts a root-owned file lists

C's file,
with `cloneMounts` holding a list of checkouts rather than one.

What the person sees for the example:
the same as under C.

What stops it:
one record file serves the whole machine
(`READINESS_DIRECTORY`,
`clone_mount_readiness.rs`:43,
and `record()`,
`clone_mount_readiness_file.rs`:227),
and one record names one checkout (`ReadinessRecord`,
`clone_mount_readiness.rs`:227 to :247),
so a machine with two checkouts needs one record per checkout,
which is a change to a record the project decided,
or a second file,
which is a change to the root side's own layout.

What it costs:
the file's grammar,
the sweep's loop over checkouts,
the probe's refusal of two checkouts,
and the readiness record's shape.

### D. Make the delegation the gate

The gate widens,
and `check_delegation` becomes what refuses an unmanaged
checkout.

What the person sees for the example:
the same as under B,
with a different sentence at the same place:
the add opens the device,
reads the delegation,
reports it as a finding on a dataset that carries none,
and then meets `admit_the_handler`'s refusal after that.

What stops it:
the root side again.
D widens the add's gate and leaves the sweep's discovery as it is,
so no readiness record exists for the checkout D admits,
and D's own sentence sends the person to the delegation while the thing missing
is the root side's installation.
Measured on this machine the same way as B:
`/home/user/Monochromatic` is a subtree of `zpcachyos/ROOT/cos/home`,
so D would open the device and read a delegation on every add that reaches the
shape check here,
including every add that today is refused before the device is opened.

What it costs:
one more device read on every ZFS add,
and the same missing discovery rule as B.

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
Choosing H is choosing to pull steps 12,
18 and 19 forward,
which is a different decision from this one.
It also cannot be done by an agent at all:
the design puts it in the owner's hands,
with every session stopped.

What it costs:
steps 12,
18 and 19 as they stand,
plus a root-run move of a live checkout.

### The ranking for choice 1

`E > G > C > A > A2`.

B,
D and H are not ranked,
for the reasons each states:
B and D cannot be completed because the root side has no name it can trust,
and H is not a change to the gate.

- E beats G:
  both deliver the route to as many checkouts as a machine holds,
  and E delivers the names as identities a command proved
  when it wrote them,
  so a dataset that was renamed or replaced since is refused
  at the gate,
  where G's names are text a person typed and are checked only
  where the work has already been done.
- G beats C:
  C delivers the route to one checkout,
  and G to as many as the file lists,
  which is the same delivery for the example and more for a person who has two
  repositories.
- C beats A:
  C delivers the route's snapshot and clone,
  its own dataset,
  the mount at the
  slot,
  the record and the lock;
  A delivers the ladder's block-sharing copy inside the source's dataset,
  with no dataset of its own and nothing for steps 14 and 16 to read.
- A beats A2:
  A2's only delivery is the knowledge that a feature the person cannot install
  exists,
  and its way forward is an edit that changes nothing for the add.

## Choice 2: the add's printed sentence beside a standing removal plan

### What the add reads today, and what it prints

`refuse_while_a_plan_names_the_clone` (`plan_folder_read.rs`:160)
lists the plan folder,
takes only entries whose name is a clone's canonical decimal GUID,
refuses when a plan's `cloneGuid=` or `cloneDataset=` line names this clone
(:127 to :141),
and refuses a file it cannot read.
Its sentence is one constant,
whatever the plan says
(:214 to :219):

```text
a removal's plan at <path> names the clone this add would fill, so the add stops
before its first mutation and changes nothing. Ways forward: run the removal this
plan belongs to, which resumes from the plan and ends it; then run the add again at
the slot.
```

The two readings that sentence needs are both absent:

- the plan's head word,
  written as `head=` in the plan's grammar
  (`cli-git-zfs-removal-destruction-plan-record-design.md`,
  "The grammar",
  whose example record carries `head=present`);
  the add's reader never looks at that line;
- whether the recorded clone still answers under the pinned parent,
  which the plan design's "Which resume path a removal takes" makes the other
  half of the decision and which the add never reads.

`command_for` (`add_state_commands.rs`:175) already holds the decided answer for
every combination of the two facts,
including the arm its own doc calls "a reader that did not look for the clone,
which is the add's smallest plan read",
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

and the design's own plan row is by "the plan's head word and the live pinned
parent".

### 2A. Print the removal for every state, as today

What the person sees,
in V24's own example
(a removal refused at its second deferred-origin check leaves a plan at
`head=present` beside the live clone):
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
where the finish pass is the route and no command
accepts the state.
So in those cells the person runs two commands where one was meant,
and the first one told them to run it.

What stops it:
nothing stops it working;
it is simply wrong wherever the clone does not stand,
which is seven of the fifteen cells the five head words and three presence
answers make,
and getting them wrong is what the build round refused to build.

What it costs:
nothing to change,
and every cell where the clone does not stand.

### 2B. Read the head word only

The add parses `head=` and calls `command_for` with `clone: None`.

What the person sees,
in V24's own example:
the settling command,
because that is what
`command_beside_a_standing_plan` answers for `present` when no clone was looked
for (`add_state_commands.rs`:238 to :252).
The settling command refuses a binding whose clone stands,
and names `git worktree remove`.
So the add sends the person to a command that refuses,
in the one scene V24 uses to justify the rule.

What stops it:
V24's own example.
Disqualified.

What it costs:
one parsed line and its refusals.

### 2C. Read the head word and the pinned parent, and print `command_for`

The add parses `head=`,
enumerates the container's immediate children through the device the preflight
already opens (`each_child`,
`sweep_dataset_reader.rs`:302;
`OpenedDevice` holds a public `ZfsDevice`,
`add_zfs.rs`:58),
and builds the `ClonePresence` `command_for` reads:
`Stands` when a child of the container carries the recorded GUID,
whatever its
name,
which is the plan design's first bullet;
`GoneProved` when the recorded name is absent and no child carries the recorded
GUID;
and `GoneUnprovable` when the listing refused or came back partial.

What the person sees for the example,
state by state:

- plan at `head=present`,
  clone standing under the container:
  the removal,
  as V24's example asks;
- plan at `head=present`,
  clone gone and proved gone:
  the settling command,
  which accepts it,
  and no removal that refuses it;
- plan at `head=requested`,
  `unknown` or `gone`,
  clone gone and proved gone:
  the removal,
  which takes its head-gone path;
- plan at `head=gone`,
  clone gone and the pool cannot prove it:
  the finish pass,
  whose line names no command and says to wait for the pool
  (`NextCommand::TheFinishPass`,
  `add_state_commands.rs`:257 to :267);
- the name carrying another clone's GUID:
  no command settles it,
  and today no `ClonePresence` arm says so.

What stops it:
one thing,
and it is a gap to close in the build rather than a reason to
refuse the option:
the reused-name cell has no arm in `ClonePresence` today,
so `command_for` cannot answer "no command settles this" for it
(the build round reported the same gap).
A second,
smaller thing:
a plan whose `head=` line is absent or carries a word the five do not hold
must refuse with the plan named and no command,
which is the plan design's own rule for a record that does not read,
and the add's reader must say so rather than default to a word.

What it costs:
a `head=` line in the add's reader with its refusals,
the container enumeration through the device the preflight opens
(the plan read moves after `OpenedDevice::open`,
`add_preflight.rs`:85 and :88,
because the presence read needs the device),
a fourth `ClonePresence` arm for the reused name,
and both plan reads carrying the same answer
(`add_preflight.rs`:85 and `add_route_run.rs`:230),
or the same state prints two different sentences.

### 2D. Print the facts and no command

The add prints the plan,
the head word,
and whether the clone answers,
and says the command is the removal's or the settling command's.

What stops it:
V24 asks for the command,
and `DGT` asks a diagnostic to name what a person
runs.
Disqualified.

### 2E. Print both commands with the condition that chooses

The add prints the head word's two candidate commands,
with the fact that decides between them,
and the person looks at the container.

What the person sees for the example:
two commands and one question,
instead of one command.

What stops it:
nothing stops it working,
and it needs no device read,
but the wrapper is the side that can read the pinned parent and the person is
not,
so it hands the person a judgement it can make itself.

What it costs:
the `head=` line and its refusals,
and a two-command sentence.

### The ranking for choice 2

`2C > 2E > 2A`.

2B and 2D are not ranked:
2B sends the person to a command that refuses in V24's own example,
and 2D names no command at all.

- 2C beats 2E:
  both name a command that will be accepted,
  and 2C names exactly one,
  chosen by the reader that can see the pinned parent,
  where 2E makes the person answer a question the wrapper had already answered.
- 2E beats 2A:
  2E never sends the person to a command that refuses,
  and 2A does in the two
  states a person can produce without any help from the wrapper.

## Choice 3: which grants the preflight judges

### What the preflight judges today

`refuse_without_the_decided_grants` reads the account's effective delegation over
the checkout dataset and the container with two `ZFS_IOC_GET_FSACL` requests
(`add_grants.rs`:60 and :84),
builds `DelegationFacts`,
and refuses unless `check_delegation` passes
(`doctor_zfs_route/delegation_checks.rs`:483).
That judgement is the whole decided set of nine
(`REQUIRED_GRANTS`,
:64 to :74),
the absence of nine lineage-changing permissions (:90),
no route permission granted to everyone,
and no route permission reached
through a named set.

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
the second and third groups
are these four commands
(`allow_command`,
`delegation_checks.rs`:431 to :438):

```text
zfs allow -d -u 1000 destroy tank/home/u/work/monochromatic-git-worktree-clone-datasets
zfs allow -d -u 1000 mount   tank/home/u/work/Monochromatic
zfs allow -d -u 1000 destroy tank/home/u/work/Monochromatic
```

and the three together are the two `destroy` grants the build round's report
names,
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
`cli-git-zfs-add-binding-record-design.md`:5591 to :5607).
The first destroy in the whole route is step 11's fallback,
then steps 14 and 16.

### 3A. Judge all nine, as today

What the person sees for the example on a machine that has the six and is
missing only the clone removal's `destroy`:
the add refuses before the claim with
(`add_grants.rs`'s wrapper sentence,
through `CheckReport::line`):

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
nine steps later.

What stops it:
nothing.
It is deliberate,
and its ground is that one machine's delegation is either the decided shape or
it is not,
so the add and `doctor` judge the same bytes.

What it costs:
a root command and a second add,
for a permission the add's own commands do not use,
in a state where the person's first add would otherwise have finished.

### 3B. Judge only the six the add's own steps use

The preflight refuses on the six,
and the other three are left to the step that uses them
or to `doctor` at step 18.

What the person sees for the example:
the add finishes,
and the missing `destroy` meets them at the first removal:
the removal's own steps (plan step 14) make no delegation read today,
so the kernel refuses the first snapshot destroy with `EPERM`
after the plan is written and the intent recorded,
and the plan then stands with its head word at `refused`
(`PlanHeadWord::Refused`,
`add_state_commands.rs`:83).
The add at that slot then refuses naming the settling command,
if choice 2 is built as 2C.

What stops it:
nothing,
but the refusal arrives after the work and in the presence of a plan,
which is the most expensive place this route can put a refusal.

What it costs:
the six as a second name for one judgement,
and a decision about which ground decides a later step's refusal.

### 3C. Judge the nine and split the verdict

One judgement,
unchanged:
`check_delegation` still answers for the nine.
When it finds a missing permission,
the preflight asks which places that permission belongs to:
a missing permission in the six refuses as today,
and a missing permission in the removal's or the origin cleanup's group
is printed as one line and the add goes on.

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

What stops it:
nothing,
with one thing to state:
the split is a rule about which step uses which permission,
and the proposal's three groups are its ground,
not a measurement.
An arm in the guest,
with the six granted and one of the three withheld,
should confirm that the add's own requests never fail for want of the withheld
one,
before this is built as more than a proposal.

What it costs:
a way to name the missing permissions in a form the caller can read,
which means `check_delegation` or a sibling exposing them
rather than only formatting them into a sentence;
the group table;
the warning line and its two controls (one where it stays quiet,
one where it
prints).

### 3D. Judge only the forbidden nine

Missing required permissions are left to the kernel's own `EPERM`
at the first request that needs them.

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
nothing,
but it moves a refusal the wrapper can make for free
to a point where it has already written four durable objects,
which is the opposite of A0's reason for existing.

What it costs:
nothing to build beyond deleting a call,
and the state a person then reads.

### The ranking for choice 3

`3C > 3B > 3A > 3D`.

- 3C beats 3B:
  both let the add finish where only a later step's permission is missing,
  and 3C also names that permission and its command before the person needs it,
  where 3B leaves them to meet the kernel's refusal at the removal.
- 3B beats 3A:
  3B finishes the add the person asked for,
  and 3A stops it for a permission the
  add never uses.
- 3A beats 3D:
  both refuse where a permission the add needs is missing,
  and 3A refuses before the claim with the command that settles it,
  where 3D refuses after the claim,
  the binding,
  the registration and the lock,
  with a sentence about a refused write.

## Findings this round did not fix

### Finding 1: the gate does not prove the checkout is its dataset's own root

`the_checkout_dataset_at` takes the mount holding the path
(`the_designs_datasets_in_the_mount_table.rs`:229 to :235)
and compares the dataset's leaf name (:236 to :238).
It never compares the mount's own mount point with the path,
and `MountHoldingAPath` carries no root field at all
(`mounted_dataset_instance.rs`:182 drops it).
So a checkout at `/u/repo`,
on a dataset named `monochromatic-git-checkout`
mounted at `/u`,
passes the gate,
and a bind mount of a subtree at the path passes it too,
because the row's own root is dropped before the comparison.
The route would then snapshot the whole dataset the checkout is a subtree
of and clone it at the slot.
The design puts the checkout at the dataset's own mountpoint
("Dataset and path layout",
`cli-git-zfs-worktree-design-proposal.md`:270),
so this is a gap between the design and the gate.
`AGENTS.md`'s `WDQ` says a defect we control is to be fixed;
this round changes no wrapper code,
so the fix belongs to whichever round takes step 10's gate next,
and option B is the only option here that would have closed it.

### Finding 2: the readiness record proves no dataset

`ReadinessRecord` carries the boot,
the pool's name and state,
the checkout's path and state,
the sweep's nonce and the time
(`clone_mount_readiness.rs`),
and `admit_attach` compares the pool and the checkout path
(`clone_mount_readiness_admission.rs`:128 to :131).
No field names the checkout dataset or the container.
Today the leaf rule makes the dataset name nearly a constant,
so the gap is masked;
every option that widens the gate unmasks it,
because the add then takes a dataset name from somewhere the root side's record
does not compare.
Closing it means a field in a record the project decided,
which is a design change and not a build's to make.

### Finding 3: the file a root side owns is readable by the account

The build round's report says,
of option C:

> `read_configuration` calls `read_file_only_root_may_write`,
> which refuses a
> file whose owner is not the account running the program
> (`FileControlRefusal::Owner`).
> That rule is written for the root caller;
> the
> add runs as account 1000 and could not read the file through it.
> The add needs
> its own read of that path with the rule "owned by uid 0 and no group or other
> write bit" (a new function,
> its refusals and a control).

The first half is right and the conclusion does not follow.
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
What the file does need is a mode the account can read,
which `0644` is and which `doctor` already accepts.
This does not change the ranking at "The ranking for choice 1",
which is by delivery;
it changes C's cost,
which the build round overstated,
and it removes the reason that report gave for ranking C first.

### Finding 4: the sweep already serves a named checkout, and the add does not

`checkout_of` takes the path and the dataset from the file when the file holds
both and asks the mount table nothing
(`sweep_configuration.rs`:187 to :188),
and `LeafNameDiffers`'s own remedy tells the person to write
`cloneMounts.checkoutDataset`
(`mounted_dataset_refusal.rs`:214 to :216).
So the root side is already capable of reconciling the example's checkout,
and the only parts that are not are the add's gate and the probe's fallback.
That is worth knowing before the project weighs C:
the choice is narrower than it looks.

### Finding 5: the fallback's line names btrfs to a ZFS checkout

The line an add prints when the snapshot route did not serve it is
`this worktree was not made as a snapshot, so it took longer, because
<the source> is not a btrfs subvolume`,
and the reason behind it is `snapshot_refusal`
(`worktree_snapshot_source.rs`:231 to :243),
which knows two causes:
the source is no subvolume root,
and the source and the destination are not on one btrfs filesystem.
On a ZFS checkout both answers are beside the point:
the design's snapshot route is step 10's,
and what kept it away there is the dataset leaf name or the dataset names a
root-owned file would carry.
So the one line a person on ZFS reads about the missing snapshot names a
filesystem they are not on,
and `DGT` asks a diagnostic to name the cause the person can act on.
This round changes no wrapper code,
so the fix belongs to the round that takes this line,
and option A2's idea does not settle it either:
a second line saying the ZFS route declined would still have to name a remedy
that works.

## What was probed, and what was not

Run on this host,
each in its own `systemd-run --user --scope` unit with `CPUQuota`,
`MemoryMax`,
`TasksMax`,
`IOWeight=1` and `RuntimeMaxSec`:

- `cargo test --locked --offline --lib -- add_route_gate
  the_designs_datasets_in_the_mount_table delegation_checks add_state_commands
  plan_folder_read clone_mount_readiness_admission add_grants add_preflight
  sweep_configuration probe_configuration boot_reconcile_mode
  worktree_clone_rung_limits sweep_dataset_reader`:
  `test result: ok. 125 passed; 0 failed; 0 ignored; 0 measured; 9426 filtered out`,
  kept whole in `keep/probe-controls-run.txt` beside this round's folder.
- `cargo test --locked --offline --lib -- clone_mount_readiness add_route_run
  add_route_steps worktree_clone_add_route`:
  `test result: ok. 83 passed; 0 failed; 0 ignored; 0 measured; 9468 filtered out`,
  kept in `keep/probe-controls-run-admission-and-run.txt`.

The controls that carry this document's claims about the example:

- `every_other_filesystem_or_dataset_keeps_the_ordinary_add` and
  `the_host_reads_reach_the_same_decision`
  (`add_route_gate_tests.rs`),
  which run `decide_the_routes_add` and the real
  host reads for a checkout that is not the design's;
- `the_checkout_dataset_at_a_path_of_another_leaf_name_is_refused`
  (`the_designs_datasets_in_the_mount_table_tests.rs`),
  which is the example's
  shape exactly:
  the path is the dataset's mountpoint and the leaf differs;
- `a_missing_grant_is_a_finding_with_the_command` and
  `the_nine_decided_grants_pass` (`delegation_checks_tests.rs`),
  which show the
  finding's text and the command for one withheld permission;
- `the_decided_grants_are_the_ones_the_design_names`,
  which pins the nine;
- the `add_state_commands` and `plan_folder_read` controls,
  which are the only
  places `command_for` and the constant sentence run.

One add of this worktree's own build also ran,
in a throwaway repository under this round's `scratch/`:

- the build was `mise run //package/git-policy/cli:native:build`,
  whose `Finished dev profile` line is at `keep/native-build.txt`;
- `git worktree add .claude/worktrees/probe-one` exited zero and printed the
  ladder's lines (`keep/probe-add-one.txt`);
- afterwards `.git/monochromatic-git-policy-cli/` held `work` and no
  `worktree-clones` folder,
  so the route wrote no claim,
  no binding and no plan folder,
  and the slot was an ordinary directory on `zpcachyos/ROOT/cos/home`;
- a second add with `MONOCHROMATIC_VERBOSE=true` after a `git status` printed
  `clones served` for the slot and the snapshot line quoted in the option that
  keeps the gate (`keep/probe-add-three-verbose.txt`),
  and one before that `git status` printed the slow route's full reason
  (`keep/probe-add-two-verbose.txt`).

Not probed,
and why:

- no live run of the route anywhere.
  This machine has no dataset named the design's,
  and this round ran no ZFS-changing command and no guest,
  so the route's own steps have still never run on any machine,
  and every claim about them here is from the code and the designs;
- no guest arm.
  Choice 3's split would be settled by one,
  with the six granted and one of the other three withheld,
  and that arm is the first thing a build of 3C should run;
- no measurement of what an add costs under each option.
  The claim that the route copies nothing where the ladder copies every file is
  a mechanism from the code,
  not a timing;
- no container gate,
  as the round's note says,
  and no `mise run toolchain:install`.

## What the next round would do

If the project chooses C,
E or G:
the add's own read of the configuration file,
with `read_file_only_root_may_write_owned_by(path, 0)`,
and the gate rebuilt on the names it holds,
plus the probe's `startProbe.dataset` fallback.
Finding 1 belongs in the same round,
because that gate is the one being opened.

If the project chooses 2C:
the `head=` line in the add's reader,
the fourth `ClonePresence` arm for a reused name,
the container enumeration in the preflight after the device opens,
and both plan reads answering the same way.

If the project chooses 3C:
a way to name the missing permissions as values,
the group table from the proposal's three sentences,
the warning line,
and the guest arm that withholds one of the three.
