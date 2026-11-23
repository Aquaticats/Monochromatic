# Where the add's durable binding record lives, and its format

Non-final,
unapproved design proposal by an agent.
It is not approved by the project owner and nothing in it is final.
It designs the gap "Where the durable binding record lives",
which the implementation plan names and the accepted design's own open-item list does
not ([plan](cli-git-zfs-worktree-implementation-plan.md),
"Gaps the design's open-item list does not name"),
and it decides nothing the project has not decided except the writer's choices that "Choices this design marks for
the project's veto" lists for the project to settle.
It edits no code,
no decision record,
no handover and no other planning document.

This revision answers the gate review
of tip `1e9bc35f1e` (`/home/user/temp/agent/gate-review-zfs-add-time-git-lock-and-binding-record-designs-2026-11-22/keep/review.md`),
which read this document beside the sibling "The add-time Git lock of a ZFS clone worktree".
Its findings that belong here are H1 as far as the claim goes,
H2,
H3,
M6,
M7,
M8's second and third items,
M9,
M11,
M12,
M13,
L1,
L2,
L7,
L8 and L9,
the citation list it marks for this document,
the disagreements between the two documents,
and the clashes with the two other record designs.
"What this revision changed" says what each finding changed.

It read the accepted design,
the proposal that carries it,
the build plan,
the wrapper's specification,
the code whose records it reuses,
the three sibling designs on their branches,
and the OpenZFS `zfs-2.4.4` and Git `v2.56.1` sources it cites.
It ran plain `git` twice over:
once on this host,
where plain `git` is the wrapper `monochromatic-git-policy-cli 0.6.1-alpha.44` shadowing Git `2.56.1`,
and once as real Git `2.56.1` inside this repository's gate image,
with no folder of the home mounted.
Each result says which of the two produced it,
and what the wrapper adds to those subcommands is named beside it.
It also read read-only `zfs` commands on this host.
It started no guest and no service manager;
the one container was the gate image.
It ran no ZFS command that changes anything,
opened no `/dev/zfs`,
mounted nothing and touched no pool.
Every claim that rests on ZFS writes,
on a mount or on a kernel behaviour the repository has not measured is written as unmeasured,
and the probe that would measure it is under "Measurements owed".

The record this design places is the one the proposal's "Add and Git registration" calls the durable binding,
keys on repository and Git registration identity plus clone GUID,
and hands to whichever place a later design names ([proposal](cli-git-zfs-worktree-design-proposal.md):412 to :431).
Steps 10,
14 and 16 of the plan write,
read and settle it;
step 18 reports it.
Step 16 is this design's assignment of the record's end,
not the plan's:
the plan's "Gaps the design's open-item list does not name" names steps 10,
14 and 18 only,
while plan step 14 says "Steps 4 and 5,
origin cleanup and the Git settle,
need a destroyed clone and are built in step 16".

## What this revision changed

Each line names the review's finding and this document's answer.
The section names are the ones in this revision.

- H1,
  as far as the claim goes:
  the claim carries nothing new,
  the binding's `nonce=` is the one root of attribution,
  and "The add's writes in one order" is the single ordered list of every write the add makes,
  the managed lock's included,
  with what a kill at each point leaves.
  The lock's sentence is stated here as a pure function of the nonce,
  which is V11's answer B,
  and marked for the project's veto as V11.
- H2:
  the reader's rule was "the `claim=` generation is not newer than the newest generation present";
  this round replaced it with one rule,
  "`claim=` is informational and the claim's own liveness is the only refusal",
  and T18 takes two generations in a row with a binding read between them.
- H3:
  no identity in this record holds a device number.
  Each identity is the folder's inode and birth time,
  and `checkoutDatasetGuid=` anchors the dataset those inodes belong to.
  The boot hazard is L11,
  T17's second arm reads after the fixture's device number changes,
  and the answer waits on step 8's guest boot arm.
- M6:
  a seventh phase,
  `slot`,
  records the slot's identity after the slot exists,
                and "The add's writes in one order" says how a resume finds a registration Git made and the wrapper did
        not yet record.
  T20 and T21 force both kills.
- M7:
  F12 now says what the OpenZFS source says,
  L4 is withdrawn,
  and T27 replaces the old T20 with a real power cut.
- M8:
                the host's runs are named as the host's ZFS home dataset and the gate image's container filesystem are named
        as what they are;
  the opening and "Sources read and runs made" say that one container ran.
- M9:
  the Git common directory is named as the plan's suggestion,
  V18 weighs a home outside it,
  V1 is reordered,
  and the argument against a registration-internal home rests on F2 and the settle order rather than on prune arms.
- M11 and the route's convention:
                "The route's three record folders" states the convention this design recommends to the two siblings and
        where they differ from it.
- M12:
  "The claim" and "The readers" now say that every take and every liveness read lists the folder,
  and M2 measures its size.
- M13:
  "The account's claim and root's claim" keeps them apart,
  and the stage claim's citation is corrected.
- L1 and L2:
  the bound is computed from the terms in "The size bound",
  and the identity grammar allows `ObjectId`'s `-:-` form.
- L7:
  step 16 is named as this design's assignment.
- L8 and L9:
  F5 cites `git worktree list`'s own function and marks its universal claim as inferred,
  and the arm numbers in "What Git does to a file it does not know here" are corrected.
- X1 and X2 of the second gate review (`gate-review-zfs-removal-side-record-designs-2026-11-22`):
  the origin cleanup record's moment is that design's V4,
  so "The record's end" step 2 now confirms a record that already stands instead of publishing one;
  and the binding is removed as soon as that confirmation succeeds,
  blocked cleanup or not,
                because the proposal says "settling does not wait for the origin" and "settling frees
        the slot" ([proposal](cli-git-zfs-worktree-design-proposal.md):468 to :469 and :1460 to :1463).
  The earlier revision's "the binding stays while the origin is blocked" is withdrawn,
  and V10 states the corrected choice.
- The same review's fourteen disagreements:
  "The route's record convention" is the one place the other two rounds can cite,
  with the precedent each part follows,
  and V19 (its merged four) and Q7 (its merged five) put the two parts no record decides to the project.
- Every citation:
  "Citations" reports the count checked and corrected.

This round answers the two second passes,
the add-time lock review of `428040d45d` and the removal-side review of the three record designs.
Each line names the finding and the section that answers it.

- N3,
  the kill inside Git's add:
  "A resume after a kill between Git's add and the registered write" now runs the lock design's per-slot scan and
  gives every state Git's failure path can leave one action and one order,
  A6 owns the decision,
  A7 is retired,
  and T20 to T20b force each state.
- N2,
  the slot Git's failure path deletes:
  the same section's second half answers it,
  and T21's new arm asserts the rewrite.
- N5,
  the identity a mounted slot covers:
  "The identity fields and the device number" names each field's object and the two comparisons,
  the five resume states and "What no reader may conclude" repeat the rule,
  and T17 gains the mounted arm.
- N1,
  the claim generations:
  "What a reader may ask of the claim" states the rule once,
  the newer-than-present arm is deleted,
  `doctor` reports it instead,
  and T18 states both orders.
- N6,
  one order:
  "The add's writes in one order" names the lock design's moment at every step,
  A7 is retired without renumbering,
  and "Where the siblings disagree" no longer claims the two agree.
- N8,
  the request:
  `request=` is recorded at A2,
  its type and its bound term are stated,
  and T21a refuses a resume whose request differs.
- N10,
  N11 and N12:
  the inode sentence is corrected,
  the two stale sentences about the lock design's record and the copy journal's folder are rewritten,
  the fifth name form is removed from the grammar,
  and the claim about the origin cleanup record's flush is replaced by V19.
- X3:
  "The record's end" cites the origin cleanup design's list at `987a3ba886`,
  states only what this record needs from its merged list,
  V20 places this binding's removal before the plan's with the claim released last,
  and "What a record whose clone is gone means" no longer reads an ordinary kill as an outside change.
- X4:
  every statement about the two removal-side records is rewritten at `76849b8d9b` and `987a3ba886`,
  and the veto options that no longer differ are gone.
- The questions:
  Q1 to Q4 and Q9 are retired into V11 and V16 to V19,
  and Q7 and Q8 are the two that remain.

## The gap this design answers

The plan's own words,
at [plan](cli-git-zfs-worktree-implementation-plan.md):2864 to :2869:

> Where the durable binding record lives.
> "Add and Git registration" says "This design does not name where that binding and lineage record lives:
> only the per-add claim record has a named home,
> the account-owned Git common directory".
> Steps 10,
> 14 and 18 write or read it.

The plan's suggestion,
at [plan](cli-git-zfs-worktree-implementation-plan.md):2959 to :2963,
is a suggestion and not a decision:
"The plan suggests the account-owned Git common directory,
beside that claim,
for both".
This design placed its folder in the Git common directory because of that suggestion,
and the review's M9 is right that the document treated it as a starting point rather than as a choice.
"The home" now states the suggestion as one,
"When the home sits outside the repository" is the alternative the earlier revision never weighed,
and V18 puts the choice to the project.

## Decided sentences that bind this design

The record's content,
in the accepted design's words ([proposal](cli-git-zfs-worktree-design-proposal.md):412 to :416):

> The durable binding keys on repository and Git registration identity plus clone GUID,
> not spelling alone.
> Store its expected slot path and identity,
> pool GUID,
> origin dataset name and GUID,
> origin snapshot and clone names,
> GUIDs and `createtxg` values,
> pointer bytes,
> request nonce and progress.
> At creation,
> record the origin-side inventory of snapshots through the origin snapshot's transaction group and
> the origin dataset's bookmarks,
> by kind,
> name,
> GUID and `createtxg`.
> Record the clone's initial inventory separately;
> an empty new clone inventory cannot identify transferred history.

That the account writes it,
and that root therefore keeps its own identities ([proposal](cli-git-zfs-worktree-design-proposal.md):418 to :423):

> This design does not name where that binding and lineage record lives:
> only the per-add claim record has a named home,
> the account-owned Git common directory,
> and the account writes the binding record wherever it is placed (reasoning:
> the wrapped add writes it as the account).
> Root's conditional destruction therefore has to keep its own copy of the identities it checks,
> which matters only for the alternative the project did not select

When each identity is written ([proposal](cli-git-zfs-worktree-design-proposal.md):383 to :384 and :405 to :407):

> Record the actual snapshot and clone GUIDs and their `createtxg` values as each is confirmed

> Mark the binding usable only after the fill is verified

What a resume must tell apart,
and the one rule that governs every reader ([proposal](cli-git-zfs-worktree-design-proposal.md):424 to :426):

> On resume distinguish a claimed empty slot,
> registration without a dataset (including a possible origin only),
> an unmounted clone,
> a mounted but unfinished clone and a verified checkout.
> Recheck recorded identities,
> the mount and the visible pointer before resuming;
> never use names,
> properties or a matching result alone to prove ownership.

What a kill may leave unrecorded,
and how it is met ([proposal](cli-git-zfs-worktree-design-proposal.md):429 to :431):

> If mutation succeeded but its identity was not recorded before interruption,
> keep and report it unless identity can be independently recovered.
> No silent fallback over an unresolved managed state.

What a failed add leaves,
which this design keeps as the accepted text wants it ([proposal](cli-git-zfs-worktree-design-proposal.md):427
to :428):

> If snapshot or clone returns `ENOSPC`,
> report the operation,
> object and pool,
> keep the claim,
> and clean up only objects whose creation and identity are proven and whose destruction is allowed.

The claim's own sentence,
which this design keeps as written ([proposal](cli-git-zfs-worktree-design-proposal.md):373 to :376):

> Claim the destination as the account with an exclusive per-add record in the Git common directory.
> Write the proposed clone name and unique `git-worktree-clone-origin-<nonce>` snapshot name before any mutation beyond
> the claim.

The removal's rule that a managed slot is never handed
to Git's recursive removal ([proposal](cli-git-zfs-worktree-design-proposal.md):811 to :813):

> If an inspection cannot prove its result,
> refuse instead of falling through to Git's recursive removal for a managed slot.

The threat model,
in the project's own sentence,
recorded at [`../decision/cli-git-zfs-staging-under-cache.md`](../decision/cli-git-zfs-staging-under-cache.md):221
to :237:
"We have to assume my machine isn't compromised.
sudo is passwordless here anyway."
and,
in that record's words,
"the account is trusted,
and root and the account are not a security boundary here".
This design says,
for each check it keeps,
whether the argument survives that sentence.

No configuration,
at [`../decision/cli-git-zfs-step-8-sweep-choices.md`](../decision/cli-git-zfs-step-8-sweep-choices.md):17 to :68:
"The project first answered that everything must work by default with no configuration",
and "1:
no file needed (Recommended)".
The record this design places needs no configuration file and no setting a person has to make.

Rule `NLK` of [`../../../AGENTS.md`](../../../AGENTS.md),
the repository's concurrency rule:
"Concurrency is lock-free first (atomic rename,
`O_EXCL`,
one owner per item);
an unavoidable lock covers the smallest unit,
never a whole repository,
with its reason written beside it."
This design takes no lock.

Rule `LQM`,
the shipped-limits rule:
"Shipped resource limits (unit caps,
timeouts,
size bounds) cite a workload measurement or owner decision beside the value".
Every bound below is derived from a kernel constant named with it,
or left out with the measurement that would size it under "Measurements owed".

The wrapper never asks a person for a secret,
at `package/git-policy/cli/SPEC.md`:7348 to :7350:
"Avoid ever asking for someone to type a username or password."
and "Passphrases count too (Recommended)".
This record needs no authority it does not have,
so it never prompts.

## The wrapper's own records this design reuses

The wrapper already keeps several kinds of small durable record in the Git common directory,
and this design reuses their primitives and their shape before proposing anything new.

- The destination record `<id>.add`,
  published whole by a rename and flushed by nobody,
  with `key=value` lines and a `phase=` line,
  written before real Git runs and rewritten as the add learns more (`package/git-policy/cli/SPEC.md`:4400 to :4408,
  `src/native/worktree/worktree_add_binding.rs`:122 to :134).
  Its reader finds a field by its key and skips every line it does not ask for (`worktree_add_binding.rs`:77 to :87),
                and its identity fields are an `ObjectId` per directory and the pointer's bytes
        in lowercase hexadecimal (`worktree_add_binding.rs`:89 to :113).
- The per-add stage claim `<id>.stage`,
  whose name carries no owner and whose last line does,
  so a command killed between the claim and its bytes is a claim no owner owns (`SPEC.md`:4401 to :4405,
  `src/native/worktree_copying/worktree_copy_stage_claim.rs`:380 to :407).
- The slot identity store,
  one file per item,
  each published with an exclusive creation and one link,
  no lock,
  and with a bound computed from the kernel's name and access-list limits (`SPEC.md`:5525 to :5546,
  `src/native/worktree_clone_mount/slot_identity_store_publication.rs`:114 to :145,
                and [`cli-git-zfs-slot-identity-store-per-slot-files-design.md`](cli-git-zfs-slot-identity-store-per-slot-files-design.md)).
- The `worktree-added` record inside a registration,
  whose module says "The registration's directory is deleted with the worktree,
  so the record needs no removal of its own" (`src/native/worktree/worktree_added_by_the_wrapper.rs`:1 to :26),
  which is why this design's record cannot live there:
  see "What Git does to a file it does not know here" and F2.

The primitives it reuses rather than writes:

- `take_claim` and `live_owner_of` of `src/native/lock/single_owner_claim.rs`,
  the wrapper's lock-free claim:
  one numbered generation file published with one exclusive `link`,
  `EEXIST` deciding the winner,
  an owner named by process birth and PID namespace,
  a newer generation taken only when the older records a release or a dead owner,
  the newest generation never removed and every older generation removed by the command that takes the newer one,
  and both the file and the folder flushed (`single_owner_claim.rs`:1 to :40 and :362 to :404).
- `PRIVATE_DIRECTORY_MODE` and `PRIVATE_FILE_MODE` of `src/native/foundation/private_storage.rs`:82 to :88,
  and `replace_private_file` at `src/native/foundation/private_storage_replacing_a_record.rs`:50 to :59.
- `create_new_entry`,
  `fill_entry_and_sync`,
  `link_entry`,
  `remove_entry` and `sync_folder` of `src/native/foundation/publishing_entries_in_one_open_folder.rs`:1 to :188,
  which is how the store publishes a record that replaces nothing.
- `DatasetGuid` (`src/native/worktree_clone_mount_protocol/dataset_guid.rs`:94 to :128):
  a GUID's canonical spelling is its decimal digits with no sign and no leading zero.
- `RequestNonce` (`src/native/worktree_clone_mount_protocol/request_nonce.rs`:1 to :20):
  32 lowercase hexadecimal digits.
- `ObjectId` (`src/native/subvolume/fs_beneath.rs`:85 to :114):
  `device:inode:seconds:nanoseconds`,
  or `device:inode:-:-` when the filesystem reports no birth time.
  This design takes its inode,
  seconds and nanoseconds and drops its device,
  for the reason under "The identity fields,
  and the device number".
- `slot_name_refusal` (`src/native/worktree_clone_mount/slot_name.rs`:22 to :40):
  a slot name is one plain folder name,
  with no separator,
  no leading dot and no space or control character.

### The account's claim and root's claim

Two different records share one primitive,
and this design keeps them apart.

Root's per-slot claim,
at `SPEC.md`:6228 to :6272,
is "a root handler['s] exclusive claim on one slot before it rechecks the clone's desired state and identities
and acts":
"The claims live in `/run/git-worktree-clone-mount-slot-claims`,
a root-owned directory of mode `0755` ... one slot's claim is a numbered generation file
of mode `0600` named `<slot>.<generation>`",
and "The claims belong under `/run` rather than in persistent state:
a claim means nothing across a boot".
Step 8 of the plan is its builder.

The account's per-add claim,
which this design places under `<slot>.claim.<generation>` in the record's own folder,
is the account's hold on the destination before and during an add or a removal.
The proposal fixes its home:
"Claim the destination as the account with an exclusive per-add record in
the Git common directory" ([proposal](cli-git-zfs-worktree-design-proposal.md):373).
It must outlive a boot,
because a boot drops every mount and the removal that resumes afterwards must know that no command is working
the slot;
root's claim must not,
because it holds only a mount sweep's passage over a slot that this boot has already dropped.
Both are flushed on every take,
which the specification's own sentence about root's claim shows:
"every take writes and syncs a record".
The difference is the lifetime,
not the flush.
The two never share a folder,
a name or a meaning:
root's is `<slot>.<generation>` under `/run`,
the account's is `<slot>.claim.<generation>` under the common directory,
and no reader of one may read the other as a claim.

Earlier revisions of this document cited `SPEC.md`:6252 to :6272 for the stage claim `<id>.stage`;
that passage is root's claim.
The stage claim is at `SPEC.md`:4401 to :4405,
and it is the wrapper's third use of the same primitive,
for a copy's stage container.

## Facts

Each fact stands alone and names where it was read or measured.

F1.
Git names a worktree's administrative directory after the destination's last path component,
after sanitizing that component as a refname component,
and when `mkdir` reports `EEXIST` it appends a counter:
`worktrees/<name>`,
then `worktrees/<name>1`,
then `worktrees/<name>2` (Git `v2.56.1`,
`builtin/worktree.c`:492 to :511).
A registration's name is therefore neither the slot's name as the wrapper writes it nor unique by construction.

F2.
Git removes a registration by deleting its whole directory recursively:
`delete_git_dir` (`builtin/worktree.c`:149 to :160) calls `remove_dir_recursively`,
and `git worktree remove` calls `delete_git_dir` whatever the force level (`builtin/worktree.c`:1473).
No file inside a registration outlives the worktree's removal.

F3.
`git worktree prune` prunes with `expire = TIME_MAX` when no `--expire` is given (`builtin/worktree.c`:211 to :259),
and it prunes an entry whose `gitdir` file is missing,
or whose `gitdir` names a place that does not exist and whose registration holds no `index`,
or an `index` no newer than the expiry (`worktree.c`:960 to :1022).
A registration holding a `locked` file is kept (`worktree.c`:966 to :969).

F4.
`git gc` runs `git worktree prune --expire <gc.worktreePruneExpire>`,
whose default is `3.months.ago` (`builtin/gc.c`:144,
:181 to :183,
:338 to :347 and :718 to :719).

F5.
`git worktree list` lists the registrations of `<common>/worktrees` and nothing beside them (`builtin/worktree.c`:211
to :244 is the prune that walks the same folder;
the listing itself reads each registration's `gitdir` (`worktree.c`:140 to :151) and `locked` (`worktree.c`:305
to :326)).
No Git command in the arms of F6 read or wrote a folder of the common directory it did not create,
and none wrote an entry into one.
That universal sentence is inferred from those arms and from Git's source,
not measured for every command.

F6.
Measured,
this round,
on Git `2.56.1`,
in two runs that agree arm for arm.
A file in the record's folder beside the copy journal's folder and a file inside a registration were placed in
each arm,
and each command's effect recorded.
Nine arms now,
because the review's M9 showed the registration arms must also be measured with the managed lock in place.

Run A ran plain `git` on this host,
which is the wrapper `monochromatic-git-policy-cli 0.6.1-alpha.44` shadowing Git `2.56.1`,
in throwaway repositories on the host's own ZFS home dataset (`zpcachyos/ROOT/cos/home`,
a plain directory and not the route's checkout dataset),
each with its own `HOME`,
`XDG_RUNTIME_DIR`,
`XDG_CACHE_HOME` and `XDG_CONFIG_HOME` under this round's scratch folder.
The wrapper sits in front of these subcommands and adds its own steps,
so run A shows the wrapper's behaviour and Git's together:
its cheap-creation ladder serves a qualified `git worktree add` (`SPEC.md`:1152 to :1200),
and before `git worktree remove` or `git worktree prune` runs it copies unshown push failures
and unfinished push records into `<common>/monochromatic-git-policy-cli/removal-snapshots/` and keeps held commits
on `refs/cli-git/kept/` (`SPEC.md`:16471 to :16480 and :3915 to :3930);
the wrapper refuses a removal only in its own btrfs-conversion arm,
and `git worktree prune` is not refused at all (`SPEC.md`:3525 to :3530).
None of those steps reads or writes the folder or the registration file this probe placed,
and run A's `git worktree add` wrote its own records there too (`monochromatic-git-policy-cli-copy-owner`
and `monochromatic-git-policy-cli-worktree-added` in the registration,
`monochromatic-git-policy-cli-worktree-copy/` beside it).
The wrapper build this session runs was built on 2026-11-17,
before any ZFS step merged,
so its `worktree add` takes the old route and says nothing about the ZFS route.

Run B ran real Git `2.56.1` inside the gate image already present as `localhost/git-policy-native-test:development`,
in one throwaway container on that image's own container filesystem,
with `podman run --rm --interactive --network none --cpus 2 --memory 512m --pids-limit 128 --blkio-weight 10`,
no folder of the home mounted and every repository made inside the container.
There `git` answered `git version 2.56.1` at `/usr/sbin/git`,
so run B is Git itself.
The results:

- `git worktree list`,
  `git worktree prune`,
                `git worktree repair` and `git gc --prune=now` left both files as they were while
        the worktree's pointer existed (arm 1,
  both runs).
- `git worktree remove --force` deleted the registration and the file inside it,
  and left the folder beside the journal untouched (arm 2b,
  both runs).
  Without `--force` the same command refused,
  because a `--no-checkout` worktree has every tracked file missing (arm 2a,
  both runs).
- With the slot's `.git` pointer deleted,
  `git worktree prune` deleted the registration and the file inside it at once,
  and left the folder beside the journal untouched (arm 3,
  both runs).
- With the worktree's folder deleted and the default expiry in force,
  `git gc --prune=now` deleted both at once when the registration held no `index` (arm 4,
  both runs),
  kept both when the registration held a fresh `index` (arm 5,
  both runs),
  and deleted both when that `index` was four months old (arm 6,
  both runs).
  An explicit `git worktree prune` then deleted both whatever the `index` age (arm 5,
  both runs).
- `git worktree repair` with the registration's `gitdir` file deleted left the registration and the file inside it
  in place,
  did not recreate `gitdir`,
  and stopped listing the worktree (arm 7,
  both runs).
- With a registration of the destination's own name already present,
  `git worktree add` made `worktrees/topic1` beside it,
  never replacing the stale one (arm 8,
  run B).
- With the same add made by `git worktree add --lock --reason <text>` and the slot's `.git` pointer deleted,
  both `git worktree prune` and `git gc --prune=now` kept the registration and the file inside it (arm 9,
  both runs),
  because the `locked` file stops `should_prune_worktree` before it judges the pointer (`worktree.c`:966 to :969).
  After `git worktree unlock`,
  the explicit prune deleted both (arm 9,
  run B).

So every Git claim about these commands is decided by run B and by the source at `v2.56.1`,
and run A agrees with it.
The two claims that differ in arm 9 are the ones that decide M9:
an unlocked registration is deleted by prune and by gc,
and a locked one is not.
The transcripts and scripts are this round's `/home/user/temp/agent/zfs-add-binding-record-design-2026-11-22/keep/`.

F7.
The wrapper already writes a record inside a registration on every add it makes:
the arms above show `monochromatic-git-policy-cli-copy-owner` and `monochromatic-git-policy-cli-worktree-added` there,
and its module justifies that placement by the registration's own lifetime (see "The wrapper's own records
this design reuses").

F8.
A fresh clone of a checkout snapshot carries the checkout's `.git` directory until the add moves it aside,
as VM item 1 measured and the accepted design states ([proposal](cli-git-zfs-worktree-design-proposal.md):397
to :400).
So a clone's copied `.git` holds a copy of the common directory as it was at the snapshot,
and with it a copy of whatever folder this design places there.

F9.
ZFS accepts only these bytes in a dataset,
snapshot or bookmark name component:
letters,
digits,
`-`,
`_`,
`.`,
`:` and a space (`module/zcommon/zfs_namecheck.c`:57 to :65,
OpenZFS `zfs-2.4.4`),
and a whole dataset name including its `@snapshot` or `#bookmark` part is
at most 255 bytes (`include/sys/fs/zfs.h`:88,
`ZFS_MAX_DATASET_NAME_LEN` 256 including the terminating NUL).
No name can hold a newline or `=`,
so a `key=value` line whose value is a ZFS name cannot be forged by that name.

F10.
A bookmark carries the GUID and creation transaction group of the snapshot it was made from,
not its own:
`module/zfs/dsl_bookmark.c`:348 to :356 and :719 to :732 (OpenZFS `zfs-2.4.4`).

F11.
`createtxg` is "The transaction group (txg) in which the dataset was created",
and bookmarks "have the same `createtxg` as the snapshot they are initially tied to" (`man/man7/zfsprops.7`:126
to :132,
OpenZFS `zfs-2.4.4`).

F12.
A flush on a ZFS file commits the dataset's log,
not a whole filesystem:
`zfs_fsync` calls `zil_commit(zfsvfs->z_log, zp->z_id)` (`module/zfs/zfs_vnops.c`:110 to :122,
OpenZFS `zfs-2.4.4`).
That call builds one commit list from the sync lists of the open transaction groups and commits
it (`module/zfs/zil.c`:2908 to :2960),
so it commits every synchronous intent waiting in the log,
not only the flushed file's.
Which intents are synchronous:
the write,
truncate,
setattr,
xattr and ACL intents set `itx_sync = (zp->z_sync_cnt != 0)` (`module/zfs/zfs_log.c`:685,
:722,
:784,
:816,
:882 and :920),
while the create,
remove,
link,
symlink and rename intents never set it anywhere in their five functions (`zfs_log.c`:299 to :609),
so they keep `zil_itx_create`'s default `itx_sync = B_TRUE` (`zil.c`:2561).
By this source reading,
a `fsync` of a directory after a link or a rename makes that name durable at `sync=standard`;
the `os_sync == ZFS_SYNC_ALWAYS` guards at `module/os/linux/zfs/zfs_vnops_os.c`:865,
:1227,
:1422,
:1553,
:2667,
:3272 and :3472 add an immediate commit after those operations on a dataset whose `sync` property is `always`,
which is a different guarantee.
A transaction group is also committed at most `zfs_txg_timeout` seconds after its first change,
whose default is 5 (`module/zfs/txg.c`:116).
This corrects the earlier revision's reading,
which the review's M7 found wrong:
it read those guards as the only path that logs a directory operation.

F13.
Measured on this host today,
read-only:
the folder holding this repository's checkout lives on the dataset `zpcachyos/ROOT/cos/home` (`stat -f` says `zfs`;
`zfs list` shows the mountpoint `/home`),
which holds 25 snapshots,
all named `autosnap_<date>_hourly`,
with `com.sun:auto-snapshot` unset (`zfs get -H -o value`).
`/etc/sanoid/sanoid.conf` keeps 25 hourlies and no other kind for that dataset (`[template_home] hourly = 25`,
every other count zero),
and that file also says "zfs-space-guard deletes the oldest whenever the pool goes over 85% full".

F14.
Measured on this host:
`getconf PATH_MAX /` and `getconf PATH_MAX` inside the checkout both answer 4096,
and `getconf NAME_MAX /` answers 255.
The wrapper already spells the second constant as `NAME_MAX = 255` (`src/native/subvolume/fs_btrfs_ioctl.rs`:142
to :144).

F15.
The wrapper's own add records are published whole by a rename and flushed by nobody,
and the specification gives the reason:
a flush of any of them "is a commit of the whole btrfs filesystem,
which waits behind every other writer's data,
and it stands on the add's waiting path" (`SPEC.md`:4421 to :4423).
That reason names btrfs,
and this record lives on whichever filesystem holds the checkout,
which for this route is a ZFS dataset.

F16.
The device number the kernel assigns a ZFS mount is anonymous,
belongs to one mount,
and may differ after a boot or after an export and import;
the specification records it at `SPEC.md`:5589 to :5601:
"on ZFS that number is anonymous and per mount:
`stat --format=%d /home` answered `57` while `/proc/self/mountinfo` listed that dataset
as `0:57 ... - zfs zpcachyos/ROOT/cos/home`",
and "Whether the same number comes back after a reboot or after an export and import of the dataset
is unmeasured ... When it does not,
every recorded identity differs after a boot".
Any identity that must still match after a boot therefore cannot hold a device number.

F17.
`SPEC.md`:6228 to :6272 names root's per-slot claim under `/run/git-worktree-clone-mount-slot-claims`,
its `<slot>.<generation>` files,
and the reason it is not durable ("a claim means nothing across a boot"),
and `SPEC.md`:4401 to :4405 names the account's stage claim `<id>.stage` of a copy.
They are two records of one primitive,
and this design's claim is a third.

F18.
`take_claim` removes every older generation after it publishes a newer one (`single_owner_claim.rs`:392 to :396),
and the newest generation file is never removed (`single_owner_claim.rs`:28 to :33).
So after two operations on one slot,
the generation a record names may no longer exist,
which is why the record's `claim=` is informational and the claim's own liveness is the only refusal
("What a reader may ask of the claim").

F19.
`live_owner_of` and `take_claim` both list the whole folder:
`generations` is a `read_dir` of it (`single_owner_claim.rs`:218 to :233),
called from `take_claim_seamed` (`:365`) and from `live_owner_of` (`:420`).
No read of a claim is one file;
a read of the newest generation begins with the listing.

F20.
The wrapper's records move with the wrapper's own folder in the common directory:
the rename machinery's list of names kept there holds the wrapper's data folder and the copy journal's folder
as separate entries (`src/native/foundation/kept_names_in_the_git_directories.rs`:36 to :60,
and `src/native/foundation/wrapper_names_old_and_new.rs`:117).
A new top-level folder most likely needs its own entry there,
which is an inference from how the copy journal's folder is handled and not a reading of a rule.

## Assumptions

A1.
The checkout's Git common directory is on the checkout dataset,
because `.git` is in the checkout's own folder and the route gives the checkout a dataset of
its own ([proposal](cli-git-zfs-worktree-design-proposal.md),
"Dataset and path layout").
Nothing measured this round;
the route's step 19 migration creates that dataset and is not built.
Until it is,
the dataset holding the common directory is the home dataset,
whose GUID this record also names.

A2.
The add,
the removal and `doctor` all run as the account,
and under the delegated route root reads no lineage ([proposal](cli-git-zfs-worktree-design-proposal.md):418 to :423).
A root-side reader of this record is therefore out of scope,
and the record's modes below are the account's.

A3.
A removal runs for a slot whose add finished,
so a removal always meets a binding record written by that add.
A removal that meets no record refuses before its first delete,
which is the design's answer and not a measurement.

A4.
The wrapper's removal settles the registration by Git's own removal,
so F2's recursive delete is what removes the registration.
Which command performs that settle is not this design's,
and the sibling lock design's removal route ("unlink,
empty by descriptor,
remove by name") is a second route this record must survive:
it does,
because it is not inside the registration.

A5.
`zfs list -t snapshot,bookmark` answers an unprivileged account for the datasets the route uses.
The route's scoped delegation governs changing things,
not listing them,
and no measurement here read it as another account.

A6.
A snapshot at or below a recorded `createtxg` cannot appear later,
because transaction groups only increase.
So the origin-side inventory of a given origin snapshot can be rebuilt while that snapshot exists.
Nothing measured this round.

A7.
The account's own writing speed for a few hundred bytes plus a ZIL commit is small beside the add's snapshot
and clone.
F12 supports the shape of that claim;
its cost is measured by M1 and is not known now.

A8.
A record's file name and contents are compared as bytes,
and no two of this design's names can collide with a registration's name because they are in different folders (F1,
F5).

A9.
The sibling designs named under "Where the siblings disagree" place their records in the wrapper's own folder in
the common directory or in a registration,
and this design's folder holds only its own five name forms.

A10.
A folder's inode number and birth time are stable while its dataset is mounted and across a remount of that dataset,
because ZFS inode numbers are the objects' own numbers and the birth time is the object's creation time.
F16 shows the device number is not stable,
and nothing here measured inode and birth across a boot;
that is M7.

## Limits

L1.
Nothing in this design was run against ZFS.
The record's publication,
its flush,
the inventory it builds and the resume states it describes are unmeasured on any pool;
M1 to M8 say what would measure each.

L2.
The five resume states are told apart by reading the live system,
never by the record alone.
Where the live read cannot prove a fact,
the reader refuses;
a refusal can therefore stop an add whose object another hand made harmless,
and the design accepts that over guessing.

L3.
The record's size is bounded per line and per fixed field,
never in total,
because nothing bounds an origin's history.
A very large origin makes the record and every reader's pass proportional to that history (F9,
F11).

L4.
Withdrawn by the review's M7.
The earlier revision said a power cut could lose the newest name even after a flush,
because OpenZFS logs directory operations only at `sync=always`.
F12's source reading says the opposite:
the directory intents are synchronous by default.
What is unmeasured is whether the whole chain holds in practice,
and M6 measures it.
Until then the design flushes every write and does not claim more than "the write returned after a log commit",
which the filesystem guarantees and a power cut may still outrun in ways this document no longer predicts.

L5.
The record's bytes live on the checkout dataset,
so the checkout's hourly snapshots hold a copy of every record this route wrote into them (F13).
A person who restores such a snapshot of the checkout sees records for clones that no longer exist,
and `doctor` reports them as orphans rather than acting on them.

L6.
A clone carries a copy of the common directory as of its snapshot,
and with it a copy of this folder (F8).
A reader that resolved the record's home from inside a clone before
the pointer install would read another add's records.
Every reader resolves the home from the main checkout,
and nothing in the record can make a reader do otherwise.

L7.
The claim's liveness rules are the reused module's:
an owner in another PID namespace counts as alive until its claim is older than
that module's own patience (`single_owner_claim.rs`:32 to :38),
so an add in another namespace can block one here for that long.
Nothing measured a ZFS add in two namespaces.

L8.
F6's host arms ran on this host's ZFS home dataset as a plain directory,
not on the route's checkout dataset,
and its gate-image arms ran on that image's container filesystem.
Neither is the route's checkout dataset,
so the file operations the design makes there are unmeasured on it;
M5 measures them.

L9.
The record names the repository by its common directory's path and the folder's identity.
A restored or recreated `.git` answers another inode and another birth time,
so its records refuse.
That is intended and it is a limit:
a person who rebuilds `.git` from a backup must settle the records by hand or with `doctor`'s printed steps.

L10.
No configuration file,
environment variable or flag changes this design's home,
names,
modes or bounds.

L11.
Every identity this record keeps is an inode and a birth time,
and F16 shows the kernel's device number cannot be part of one.
Whether inode and birth time survive a boot,
a remount and an export and import is unmeasured;
step 8's guest boot arm is the measurement,
and M7 repeats it for this record.
If they do not,
every resumed add and removal refuses until a person settles the record,
which is the same exposure the accepted slot identity store already carries.

L12.
The resume removes a bare `worktrees/<name>` directory that holds no lock and no `gitdir` on the strength of its name
and of the claim it holds,
which is what makes it this add's.
Another account's Git add at the same slot,
started in the same instant and killed in the same window,
would leave a directory of that shape,
and the trusted-account sentence the project states is what makes this acceptable;
the name check is also the only place two half-made directories at once (the slot's and a slot whose name is the
slot's plus digits) could be confused,
and a complete registration is never removed under it.

## The home

The record lives in one folder of the Git common directory,
account-owned,
with no configuration:

```text
<common git dir>/monochromatic-git-policy-cli/worktree-clones/v1/
```

The parent is the wrapper's own folder in the common directory,
which the route's other records already use:
the removal's destruction plan is under `monochromatic-git-policy-cli/clone-destruction-plans/` and
the origin cleanup record under `monochromatic-git-policy-cli/origin-cleanups/`,
and the wrapper itself keeps `removal-snapshots/` and `leavings/` there (`SPEC.md`:3879 to :3881).
This design's earlier revision used a top-level `monochromatic-git-policy-cli-worktree-clone/v1/`,
copying the copy journal's folder (`worktree_copy_journal.rs`:130 to :136);
the review's M11 showed the route's three records should share the parent the rename machinery already moves (F20),
so this revision moves under it and recommends the same parent to both siblings.
V17 puts the choice to the project.

Inside the parent,
`worktree-clones` names what the folder holds,
and `v1` is the grammar's version,
so a later grammar cannot be read as this one.

The folder is the account's,
mode `0700`,
made with `PRIVATE_DIRECTORY_MODE` and created by the first command that needs it.
Every file in it is the account's,
mode `0600`,
made with `PRIVATE_FILE_MODE`.
The reason is not a security boundary,
which the threat model removes,
but ownership:
a record whose owner is another account is not this account's to act on,
and a reader says so instead of guessing.
Root may read the folder,
and no root-run mode does.

The folder holds four name forms and no others:

```text
<slot>.claim.<generation>      one per operation on the slot, numbered from 1
<slot>.binding                 the record itself
<slot>.origin-inventory        the origin-side inventory, written once
<slot>.clone-inventory         the clone's initial inventory, written once
```

A fifth form for a record a reader cannot read is deliberately not reserved:
no reader moves a record aside today (Q7's adopted A),
and a form the design does not use would be a name `doctor` must explain without a writer.
If Q7 is answered B,
the fifth form comes back with that answer,
named then and not before.

plus the staging names under "Publication,
and what a kill leaves",
which no reader reads as a record.

`<slot>` is a plain slot name by `slot_name_refusal` (`slot_name.rs`:22 to :40):
the destination's last path component,
which the route fixes at `.claude/worktrees/<name>`.
The parent and the version folder are made with mode `0700` and never replaced.

### What the key is made of at each moment

The file's name is keyed by the slot from the first byte written to the last,
and the clone's GUID is never part of a name.
That ordering is what the route's moments allow:

- Before anything is written there is nothing on disk.
- The claim is taken before any mutation,
  so the only fact in hand is the destination:
  the slot names the claim (`<slot>.claim.1`).
- The binding's first write follows the claim within the same command and names the same slot.
- The slot's own identity is written only once the slot exists,
  at phase `slot`;
  the earlier revision named it in the first write,
  which the review's M6 showed names a folder that may not exist yet.
- The origin snapshot's name is chosen by the add and written before the snapshot exists,
  so it is a value in the binding;
  the snapshot's GUID and `createtxg` are values added once `zfs snapshot` reports them.
- The clone's name is chosen by the add from the slot,
  so it too is a value;
  the clone's GUID does not exist until the clone does,
  and it becomes a value of `cloneGuid=` when `zfs clone` reports it,
  never part of a name.
- The Git registration's own name is read from Git,
  not derived (F1),
  and recorded as a path.

The key that proves anything is the triple the proposal names:
the repository's identity,
the registration's identity and the clone's GUID,
all three compared by every reader before it acts.
The file's name proves only which slot the file is filed under,
and a file whose `slot=` line names another slot is refused rather than read under the name it was found at,
as the slot identity store refuses a record filed under the wrong name (`slot_identity_store_records.rs`:170 to :179).

### Why the slot's name and not the nonce or the GUID

Every reader of this record knows the slot first:
a resumed add is an add at a slot,
a removal is a removal of a slot,
`doctor` walks the slots,
and a person types a slot's name.
A name keyed by the request nonce would make `doctor` list the folder to find one slot's record and would leave
one file per add forever;
a name keyed by the clone's GUID would need a rename at the moment the clone appears,
and a reader that found two names for one add would need a merge rule for names.
The clone's GUID is still the proof of the clone's identity;
it is a value,
not a name.
V2 lists the alternatives with their costs,
and V17 records that it is still open to the project.

### When the home sits outside the repository

The plan's suggestion is the Git common directory,
and the two sibling designs went there.
This design's limits say what that costs (L5,
L6,
L9):
the checkout's hourly snapshots hold copies of every record,
every clone carries a copy,
and a rebuilt `.git` orphans every record.
The alternative the earlier revision never weighed is an account state folder outside the dataset,
keyed by the repository's identity:
`~/.local/state/monochromatic-git-policy-cli/<key>/` with the same inner names.
It removes those three costs and adds two:
the record is not found when the home is restored separately from the repository,
and the key must be derived from something the repository keeps,
which is the repository's identity in a place that itself lives on the dataset.
This design keeps the common directory,
and V18 puts the choice to the project with both costs.

## Its two files, and which one is exclusive

The claim and the binding are two files,
and the claim is the exclusive one.

The claim is exclusive because of the kernel call that publishes it,
not because of a lock:
`take_claim` writes a private file and gives it the next generation's name with one `hard_link`,
so exactly one command creates `<slot>.claim.<n + 1>`,
and every other command's link answers `EEXIST` and sends it back to read the winner (`single_owner_claim.rs`:1
to :40,
:245 to :285 and :362 to :404).
A claim whose owner is alive stops the second command with the owner's process id;
a claim whose owner is dead,
or that records a release,
lets the next command take the next generation.
The newest generation is never removed,
so no two commands can create one number,
a command that took a number another command has overtaken withdraws it,
and every older generation is removed by the command that took the newer one (F18).

The binding is not exclusive,
and it does not need to be:
its only writer is the command that holds the claim,
and the writer's interface takes the held claim,
so no code path can write it without one.
It is replaced whole by one rename,
which is the wrapper's own replacing publication (`private_storage_replacing_a_record.rs`:50 to :59).

The inventories are written once each,
published with an exclusive creation and one link,
so a second publication of one inventory answers `EEXIST` and is refused rather
than merged (`publishing_entries_in_one_open_folder.rs`:1 to :188).

That split answers what the proposal asks of the claim without putting the identities in it:
the proposed clone name and the unique snapshot name are written into the binding immediately after the claim and
before any mutation,
which keeps the proposal's order ([proposal](cli-git-zfs-worktree-design-proposal.md):373 to :376) and keeps
the claim's grammar the shared one a fuzz sidecar already reads.
A kill between the claim and that first binding write leaves a claim,
no mutation and no binding,
and the next command takes the next generation and finds nothing to resume.

Both the add and the removal take this claim,
for the same slot.
That is what makes the claim an operation's hold on a slot rather than a record of an add:
a removal that runs while an add holds the claim is refused by the claim,
and an add that runs while a removal holds it is refused the same way.
The removal then reads the binding under the claim it holds,
which is why a concurrent add cannot change the record under a removal's proof.

### What a reader may ask of the claim

The rule is one sentence,
and this section is the only place it is stated:
the record's `claim=` generation is informational,
and the claim's own liveness is the only claim fact a reader refuses on.

`claim=` says which generation wrote the bytes the reader holds.
It is neither a proof nor a test.
`take_claim` deletes every older generation and never the newest (F18),
so after two operations on one slot the record's generation is older than the newest file,
which is the ordinary settled case.
A generation newer than every file present means the claim set was dismantled by something other than a take,
and the reader reports it (`doctor` does,
as "the record names a claim no generation file holds")
and does not refuse on it,
because authority over the slot is the live owner of the newest generation file,
never a number inside a record.
A record whose newest generation is held by a live owner refuses,
naming that owner's process id (V5,
T18).
Nothing else about the generation is a refusal,
and T18 forces both orders of it.

## The route's record convention

The route writes three durable record families.
This design is the first of them to be written by an add,
so this section is the one place the other two rounds can cite for the shape all three should follow.
Each part names the precedent it follows,
and each part no project record decides is marked with the question that puts it to the project.
Neither sibling is reviewed,
and nothing here decides its design:
the second gate review's "Disagreements between the three record designs" is the material,
and the other two rounds are told to line up with this section or list the difference for the project's veto.

- Parent folder:
  `<common>/monochromatic-git-policy-cli/`,
  the wrapper's own folder in the common Git directory (F20).
  Precedent:
  the wrapper's own `removal-snapshots/` and `leavings/` folders are there,
  and the rename machinery moves that parent as one unit (`kept_names_in_the_git_directories.rs`:36 to :60).
  This revision moved this record there,
  so all three now share the parent;
  the destruction plan's and the origin cleanup record's folders are already inside it.
- Inner folder and grammar version:
  one folder per record family named for what it holds,
  and a version the grammar itself compares,
  either a `v1/` segment or a first line such as `plan=1`.
  Precedent:
                the accepted slot identity store's version-2 header line (`SPEC.md`:5540 to :5546) and
        the copy journal's `v1/` folder (`worktree_copy_journal.rs`:130 to :136).
  A version inside the grammar is what lets a later release refuse rather than misread.
- Key:
  the thing the record's first reader knows first,
  and nothing else.
  This record's first reader knows the slot,
  the destruction plan's knows the clone's GUID,
  and the origin cleanup record's knows the origin snapshot's name.
  Precedent:
                the mount receipts are filed under the dataset GUID because the waiter knows
        the GUID (`clone_mount_receipts_folder.rs`),
  and the copy journal is filed under the destination's identifier.
  A uniform key is therefore not possible,
  and the set still reaches from any one key through this record's `cloneGuid=`,
  `originSnapshot=` and `slot=` fields.
- Encoding of names and numbers:
  ZFS names verbatim,
  because the kernel's charset holds neither a newline nor `=` (F9);
  paths in lowercase hexadecimal,
  because a path may hold either (`worktree_add_binding.rs`:89 to :113);
  GUIDs and transaction groups in canonical decimal (`dataset_guid.rs`:94 to :128);
  and no 64-bit GUID inside a JSON number,
  which the wrapper's own JSON reader says is unsafe.
  Precedent:
  the btrfs binding's two encodings and the slot store's token grammar.
  The destruction plan writes the same three encodings (read at `76849b8d9b`),
  so all three now agree here.
- Publication:
  one private file,
  filled whole,
  published by one call that replaces nothing,
  an exclusive creation with one `link` or `RENAME_NOREPLACE`,
  or by one rename for a record that is rewritten (`publishing_entries_in_one_open_folder.rs`:1 to :188,
  `private_storage_replacing_a_record.rs`:50 to :59).
  Precedent:
  the accepted slot identity store's publication,
  whose decision record chose exactly this over a lock.
- Flush:
  flush every write of a record whose facts a later command must not have to rediscover.
  This record flushes every write (V6).
  The destruction plan publishes unsynced by default (its Choice 5,
  read at `76849b8d9b`)
  and the origin cleanup record is unflushed too,
  so the route has two rules today;
  V19 puts one rule to the project,
  and M1 is the number that would settle it.
  No project record fixes one rule for the three,
  so it is V19,
  the second review's merged question four.
- Staging names and who removes them:
  every staging name is a dot name made by one known writer,
  and `doctor` at plan step 18 removes the known forms.
  Where a writer holds a claim across a long step,
  `doctor` removes a staging name only when the folder holds no live claim (V8).
  The destruction plan removes one only when a plan with that GUID stands and its `owner` is gone (its Choice 12),
  and the origin cleanup record removes none and reports them (read at `987a3ba886`),
  so the route has three rules here;
  the store's own decided rule covers the store alone
  ([`../decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md),
  "Who removes a killed writer's staging file").
  Precedent:
  the store's decision,
  which covers the store alone.
- A record a reader cannot read:
  refuse and change nothing,
  and name the file and the two ways forward.
  All three designs refuse a partial,
  foreign,
  oversized or wrongly owned record.
  Whether any reader may instead move an unreadable record aside,
  as the copy journal's pass does (`SPEC.md`:4873 to :4885),
  is Q7,
  the second review's merged question five.
- Bounds:
  each record's widest line and fixed part is derived from kernel constants named with it,
  and no cap is shipped on a count nothing bounds.
  Precedent:
                the accepted slot store's bound computed from named terms and
        its dropped whole-store cap (`slot_identity_store_reading.rs`:63 to :85,
  [`../decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md),
  "What was decided").
  Neither sibling rounds either:
  the destruction plan's bound is `10,128 + 340 × count` and the origin cleanup record's is 13,375,
  both computed from named terms (read at `76849b8d9b` and `987a3ba886`).

Where the two siblings differ from the convention today,
read from their branches:

- The destruction plan keeps no pointer inside a registration:
  the earlier revision's pointer is dropped,
  and the plan reads its folder by one `stat` before every forwarded command and lists it only when it exists
  (its "What finds a plan",
  read at `76849b8d9b`).
  Its finish pass is a second command that settles a registration,
  which this record's step 5 must survive (F2).
- The destruction plan's key is the clone's decimal GUID,
  which a registration and a clone dataset reach and a slot name alone does not;
  this design's key is the slot,
  which an add and a removal reach and a clone dataset alone does not.
  The second review's disagreement two says so,
  and no record decides it.
- The origin cleanup record's key is the origin snapshot's name,
  which carries the nonce,
  so that key needs no stored field.
- The origin cleanup record is unflushed (its option C premise and its F21),
  and so is the destruction plan's outcome writing;
  this design flushes every write,
  so the route has two rules and V19 puts one to the project.
- The origin cleanup record removes no staging entry and reports the ones it finds,
  while this design removes one only when no live claim stands (V8).
  Two rules for one folder family,
  each named in its own document.
- The origin cleanup record writes its record before the first snapshot delete and says a new add may take
  the freed slot at once;
  this design now agrees with both,
  which is X1 and X2 of the second review.

## The record family

### The claim

`<slot>.claim.<generation>` is `take_claim`'s own record and stays that:
a header line,
then either `released` or the owner's `pid=`,
`birth=` and `namespace=` lines (`single_owner_claim.rs`:167 to :199).
This design adds no field to it,
for the reason H1 and V11 give:
the attribution the lock design needs is a function of the binding's nonce,
not a field the shared claim could carry,
and adding one would change the primitive and the fuzz decoder that reads it.

Its generation is the only ordering evidence among operations on one slot,
and the binding's `claim=` line says which generation wrote the binding's current bytes.
Older generations disappear under a later take,
so no reader may ask that the named generation still exist (see "What a reader may ask of the claim").

Every take lists the folder (F19),
and every liveness read lists it too,
so the folder's size is a cost on the add's and the removal's path.
M2 measures it.
The folder holds one released claim per slot name ever used plus the live slot's four files,
and nothing sweeps a released claim.

### The binding

`<slot>.binding` is the record the proposal calls the durable binding.
Its shape is the wrapper's `key=value` shape (`worktree_add_binding.rs`:77 to :87),
one header line,
one field per line,
in this order:

```text
# monochromatic-git-policy-cli zfs worktree clone binding, version 1
claim=<generation>
nonce=<32 lowercase hexadecimal digits>
request=<branch|commit|detach> <lowercase hexadecimal of the argument, or ->
phase=<claimed|slot|registered|snapshotted|cloned|mounted|filled>
checkoutDataset=<dataset name>
checkoutDatasetGuid=<decimal>
repository=<lowercase hexadecimal of the common directory's path bytes>
repositoryIdentity=<inode>:<seconds>:<nanoseconds>
slot=<lowercase hexadecimal of the slot's path bytes>
slotIdentity=<inode>:<seconds>:<nanoseconds>
registration=<lowercase hexadecimal of the registration's path bytes>
registrationIdentity=<inode>:<seconds>:<nanoseconds>
pool=<pool's dataset name>
poolGuid=<decimal>
originSnapshot=<snapshot name>
originSnapshotGuid=<decimal>
originSnapshotCreatetxg=<decimal>
originDataset=<dataset name>
originDatasetGuid=<decimal>
cloneDataset=<dataset name>
cloneGuid=<decimal>
cloneCreatetxg=<decimal>
pointer=<lowercase hexadecimal of the `.git` file's bytes>
```

A field whose fact is not confirmed yet is absent,
and the phase says how far the add got.
A field present is a fact the writer confirmed before that write,
never a guess.

The wrapper's own reader finds a field by its key and skips every line it does not ask
for (`worktree_add_binding.rs`:77 to :87).
This record refuses a line whose key this release does not know,
because a key nobody wrote is either a later grammar (which raises the version line) or damage;
V12 marks that strictness as a writer's choice,
with the wrapper's skipping reader as the alternative.

### The identity fields, and the device number

Each identity is the folder's inode,
birth seconds and birth nanoseconds,
and the record names the dataset those inodes belong to once,
in `checkoutDatasetGuid=`.
The device number the earlier revision embedded is gone:
F16 shows it is anonymous,
belongs to one mount,
and may differ after a boot,
so a record that lives until a removal,
and therefore across boots,
could not match.

`ObjectId` writes `device:inode:seconds:nanoseconds`,
or `device:inode:-:-` when the filesystem reports no birth time (`fs_beneath.rs`:85 to :114).
This design takes its three last parts and drops the first,
and it allows the `-:-` form in both of its own fields:
an identity is either `<inode>:<seconds>:<nanoseconds>` or `<inode>:-:-`,
and a reader refuses any other shape.
A folder whose birth time is unknown still has an inode,
and the identity is weaker in exactly that way,
which the reader says when it refuses.

Each identity field names one object,
and which object a field names does not change:

- `repositoryIdentity=` names the common directory;
- `registrationIdentity=` names the registration directory `<common>/worktrees/<id>`;
- `slotIdentity=` names the underlay directory at `<slot>` on the checkout dataset,
  recorded at A4 while the slot is empty;
- `cloneGuid=` names the clone dataset.

The mount is what makes the distinction matter:
from A10 on,
the path `<slot>` names the clone dataset's root,
whose inode and birth time are its own,
and the underlay directory A4 recorded is no longer visible at that path.
So a reader compares `slotIdentity=` only while the clone is unmounted,
where the underlay is what `<slot>` shows and the comparison runs through the pinned `.claude/worktrees` parent
descriptor.
While the clone is mounted,
a reader compares the mount's source dataset GUID with `cloneGuid=` and the registration with
`registrationIdentity=` and the pointer,
and never compares `slotIdentity=`,
because the object it names is covered.
That is N5 of the second gate review,
whose first revision had the sentence and whose last revision dropped it while keeping the equality.

The anchor is checked as well as the inode:
a reader reads the identity of the live folder and the GUID of the dataset the record's home is on,
and it compares both.
The inode and birth time change when a folder is deleted and re-created,
and the GUID changes when the dataset is another one,
so both ways of losing the record's subject refuse.
A dataset restored from a snapshot keeps its objects' numbers,
and so does a clone or a `zfs send` replica,
so a rollback alone does not make the identity refuse.

L11 is the limit:
whether inode and birth survive a boot,
a remount and an export is unmeasured,
and step 8's guest boot arm answers it.
T19 remounts the checkout before a read.

### The two inventories

One grammar serves both,
and the file's name says which it is:

```text
# monochromatic-git-policy-cli zfs worktree clone inventory, version 1
kind=<origin|clone>
poolGuid=<decimal>
dataset=<dataset name>
datasetGuid=<decimal>
bounds=<dataset or snapshot name whose transaction group bounds the set>
boundsGuid=<decimal>
boundsCreatetxg=<decimal>
entries=<decimal count of entry lines>
entry=snapshot <guid> <createtxg> <name>
entry=bookmark <guid> <createtxg> <name>
```

In `<slot>.origin-inventory` the set is the origin dataset's snapshots whose `createtxg` is at or below
the origin snapshot's,
and every bookmark of the origin dataset,
which is the proposal's "inventory of snapshots through the origin snapshot's transaction group and
the origin dataset's bookmarks,
by kind,
name,
GUID and `createtxg`" ([proposal](cli-git-zfs-worktree-design-proposal.md):414 to :416).
`bounds` is the origin snapshot.

In `<slot>.clone-inventory` the set is the clone's own inventory as it stood immediately after the clone was made and
before any snapshot of it existed,
which is the second set the proposal asks be recorded separately ([proposal](cli-git-zfs-worktree-design-proposal.md):416 to :417).
`bounds` is the clone dataset itself,
and the file is published even when the set is empty:
`entries=0` with no entry line is a fact the proposal wants recorded
("an empty new clone inventory cannot identify transferred history"),
and a reader must not mistake an absent file for an empty set.

`entries=` is the completeness count:
a reader counts the `entry=` lines and refuses a file whose count disagrees,
so a truncated inventory is refused rather than mistaken for a short history.
Nothing else about the count is a bound:
the number of an origin's snapshots is no kernel constant (F9,
F11).

`kind=` must agree with the name the file was found under,
as the slot store requires of a record's key (`slot_identity_store_records.rs`:170 to :179),
so a copied file refuses under the wrong name.

A missing clone inventory refuses the record only when the clone holds history of its own:
if no snapshot of the clone answers a `createtxg` above the clone's own,
the live clone inventory is still the transferred set,
so the reader rebuilds the file from it and says it did.
V13 marks that refinement as a writer's choice;
the earlier revision refused every missing clone inventory,
which the review's M6 arm shows would refuse a clone that never got its file written and holds nothing of its own.

### The fields, with their types and their encodings

Every type below is already the wrapper's,
so a value cannot have two spellings:

- A path is its bytes in lowercase hexadecimal,
  which is how the btrfs binding writes a destination and a registration (`worktree_add_binding.rs`:89 to :113).
  The encoding exists because a path may hold a newline,
  and a newline would forge a line of this record.
- A ZFS name is written verbatim,
  because the kernel's own charset holds no newline and no `=` (F9),
  so the line's first `=` separates the key from the value and nothing in the value can add a line.
- A GUID is `DatasetGuid`'s canonical decimal spelling,
  with no sign and no leading zero (`dataset_guid.rs`:94 to :128).
- A `createtxg` is the same canonical decimal spelling,
  because the property is an unsigned 64-bit number printed as decimal (F11).
- A folder identity is `<inode>:<seconds>:<nanoseconds>` or `<inode>:-:-`,
  from `ObjectId`'s three last parts (`fs_beneath.rs`:85 to :114),
  with each number at most 20 decimal digits.
- The nonce is `RequestNonce`'s 32 lowercase hexadecimal digits (`request_nonce.rs`:1 to :20).
- The claim generation is decimal digits with no leading zero,
  which is the generation file's own name (`single_owner_claim.rs`:210 to :218).
- The request is one word and one value:
  `branch`,
  `commit` or `detach`,
  then the argument exactly as the command was given,
  in lowercase hexadecimal,
  or `-` when the word is `detach` and no commit-ish was given.
  Hexadecimal is what makes the value safe:
  a refname may hold `=`,
  and a commit-ish may hold `~`,
  `^` and `..`,
  none of which a `key=value` line can carry verbatim without a second grammar.
- The pointer is the `.git` file's bytes in lowercase hexadecimal,
  which is how the btrfs binding records it (`worktree_add_binding.rs`:122 to :134).
- The phase is one of the seven words above and nothing else.

### The size bound and its basis

No file of this family is bounded in total,
and each line and each fixed part is bounded from a kernel constant:

- The binding's fixed part is the sum of its lines,
  each counted as its key,
  its `=`,
  the widest value that field can hold,
  and one newline.
  The widest values come from these facts and from nothing else:
  - the three paths at `2 × 4096` bytes,
    because a path is at most `PATH_MAX` (F14) and its hexadecimal form doubles it;
  - the pointer at `2 × (8 + 4096 + 1)` bytes,
    because Git writes `gitdir: ` (8 bytes),
    a path of at most 4096 bytes,
    and one newline;
  - the four ZFS names at 255 bytes each,
    which is `ZFS_MAX_DATASET_NAME_LEN` minus its NUL and includes the `@snapshot` or `#bookmark` part (F9);
  - every decimal field at 20 digits,
    which is a 64-bit unsigned number's widest decimal form (F11);
  - a folder identity at `20 + 1 + 20 + 1 + 10` bytes:
    inode,
    birth seconds and birth nanoseconds,
    each at its widest,
    and its two colons,
    or `20 + 1 + 1 + 1 + 1` for the `-:-` form;
  - the nonce at 32 bytes,
    the phase's widest word `snapshotted` at 11,
    and the claim generation at 20;
  - the request at `6 + 1 + 2 × 4096` bytes,
    because the widest kind word is six letters and a refname or commit-ish is at most `PATH_MAX` (F14) before its
    hexadecimal form doubles it.
  The terms sum to 43,005 bytes,
    and a scratch script printed that sum from the wide strings this section names (`keep/bound.ts`,
    this round).
        The constant in code is computed from those terms the way the slot store computes
    its record's bound (`slot_identity_store_reading.rs`:63 to :85),
    so changing a term changes the bound.
    The earlier revision printed 34,502 from a set of terms that did not derive the identity lines;
    the review's L1 found the arithmetic and the missing derivations,
    and this is the corrected form.
- One inventory line is at most `entry=` (6) + the kind (8) + 3 separators + a GUID (20) + a `createtxg` (20) +
  a name (255) + the newline:
  313 bytes.
  The same 255 comes from `ZFS_MAX_DATASET_NAME_LEN` including the `@snapshot` or `#bookmark` part (F9).
- An inventory's total is the count in `entries=` times that line's widest form plus its eleven fixed lines.
  What an origin can reach is not bounded by a kernel fact,
  so this design ships no cap,
                and the accepted store made the same choice for the reason that its item count is bounded by nothing
        the kernel says ([`../decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md),
  "What was decided").

How large an inventory gets on a checkout the hourly tool also snapshots,
measured on this host (F13):
the dataset holding the checkout keeps 25 hourlies and no other kind,
so the origin-side inventory is 25 entries after a day and stays there;
at 313 bytes each that is 7,825 bytes.
Beside those stand the wrapper's own origin snapshots,
one per add and destroyed by that add's removal's origin cleanup ([proposal](cli-git-zfs-worktree-design-proposal.md):947 to :953),
and any snapshot a person takes.
The measurement that would size a cap,
if the project wants one,
is M3.

### An example

One add at `.claude/worktrees/add-cache-layer` in the checkout `tank/monochromatic-git-checkout`,
after its mount is verified and before the fill:

```text
# <common>/monochromatic-git-policy-cli/worktree-clones/v1/add-cache-layer.binding
# monochromatic-git-policy-cli zfs worktree clone binding, version 1
claim=2
nonce=9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f
request=branch 6d61696e
phase=mounted
checkoutDataset=tank/monochromatic-git-checkout
checkoutDatasetGuid=4815162342109876543
repository=2f686f6d652f757365722f6d6f6e6f6368726f6d617469632f2e676974
repositoryIdentity=131074:1763750400:123456789
slot=2f686f6d652f757365722f6d6f6e6f6368726f6d617469632f2e636c617564652f776f726b74726565732f6164642d63616368652d6c61796572
slotIdentity=131089:1763750420:135792468
registration=2f686f6d652f757365722f6d6f6e6f6368726f6d617469632f2e6769742f776f726b74726565732f6164642d63616368652d6c61796572
registrationIdentity=131081:1763750412:987654321
pool=tank
poolGuid=17293822569102704641
originSnapshot=tank/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f
originSnapshotGuid=16371409254608711923
originSnapshotCreatetxg=121293
originDataset=tank/monochromatic-git-checkout
originDatasetGuid=4815162342109876543
cloneDataset=tank/monochromatic-git-worktree-clone-datasets/add-cache-layer
cloneGuid=4821193048571903102
cloneCreatetxg=121294
pointer=6769746469723a202f686f6d652f757365722f6d6f6e6f6368726f6d617469632f2e6769742f776f726b74726565732f6164642d63616368652d6c617965720a
```

and its origin inventory,
whose 25 entries are the checkout's hourlies at or below that `createtxg`:

```text
# <common>/monochromatic-git-policy-cli/worktree-clones/v1/add-cache-layer.origin-inventory
# monochromatic-git-policy-cli zfs worktree clone inventory, version 1
kind=origin
poolGuid=17293822569102704641
dataset=tank/monochromatic-git-checkout
datasetGuid=4815162342109876543
bounds=tank/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f
boundsGuid=16371409254608711923
boundsCreatetxg=121293
entries=25
entry=snapshot 1122334455667788990 93823 autosnap_2026-11-21_22:00:00_hourly
entry=snapshot 1122334455667788991 95291 autosnap_2026-11-21_23:00:00_hourly
```

## The add's writes in one order

This is the list both this design and the lock design can cite.
The lock design owns how the lock itself is written,
told apart,
released and recovered;
this design owns what the claim and the binding carry,
the fields,
and the order below,
in which Git's invocation is one step and no lock record is written at all.
A step's owner is named beside it,
and so is the lock design's moment for that step,
read at `a7902e293b` ("The moments of an add",
D0 to D7).

The labels do not move again:
a later revision that adds a write appends a label,
and A7 keeps its place although this revision retires it.

A0.
Preflight,
no write.
The shared `doctor` preflight proves the supported Git form,
the checkout mount,
`/dev/zfs`,
the grants,
the slot parent and the handler's readiness,
as plan step 10 lists it.
Owner:
the route's preflight.
Lock moment:
D0 begins,
in which nothing in `worktrees` names this slot.
A kill here leaves nothing.

A1.
The claim.
The account takes `<slot>.claim.<generation>` with `take_claim`.
Owner:
this design.
Lock moment:
D0.
A kill here leaves the claim and nothing else;
the next command takes the next generation.

A2.
The binding's first write,
before any mutation.
`claim=`,
`nonce=`,
`phase=claimed`,
`checkoutDataset=`,
`checkoutDatasetGuid=`,
`repository=`,
`repositoryIdentity=`,
`slot=`,
`request=`,
the proposed clone name and the proposed `git-worktree-clone-origin-<nonce>` snapshot name.
Owner:
this design.
Lock moment:
D0,
whose lock attribution this write is the root of.
A kill here leaves a claim and a binding with no mutation;
the next command takes the next generation and resumes at A3.
This is the write the proposal's "Write the proposed clone name and unique ... snapshot name
before any mutation beyond the claim" asks for ([proposal](cli-git-zfs-worktree-design-proposal.md):373 to :376),
and `request=` is added this round so that a resumed add can prove it is the same request (N8 of the second gate
review).

A3.
The empty slot.
Create or verify the empty,
account-owned slot,
and prove it empty and owned by this account.
Owner:
this design,
with the slot's own checks from the route's preflight.
Lock moment:
D0.
A kill leaves an empty slot,
a claim and a binding at phase `claimed`;
the next command verifies the slot again.
Git's own failure path may delete this slot later (the resume's slot rule),
which is why A3 is a step a resume may run again.

A4.
The slot's identity.
`slotIdentity=` and `phase=slot`.
Owner:
this design.
Lock moment:
D0.
A kill leaves the slot named and nothing mutated in Git.
The identity is the underlay folder's,
recorded before any mount covers the path ("The identity fields",
and N5 of the second gate review).

A5.
Git's add,
with the managed lock.
`git worktree add --no-checkout --lock --reason <sentence> <slot> <commit-ish>`,
where the sentence is a pure function of the recorded `nonce=`,
and `request=` records the commit-ish and the form this step was given.
Owner:
the lock design for the invocation and the sentence's wording (its "The invocation",
read at `a7902e293b`);
this design for the order,
for the nonce being recorded before this step runs,
and for the request being recorded so a resume can compare it.
Lock moment:
D1,
D2,
D2a,
D2b,
D2c and D3,
the moments inside Git,
and its failure path,
which is outside D0 to D7.
A kill here leaves whatever Git leaves;
A6 and "A resume after a kill between Git's add and the registered write" decide it.

A6.
Finding and proving what Git made,
and clearing what Git's failure path left.
Run the scan of "A resume after a kill between Git's add and the registered write",
decide each entry by that section's table,
and either adopt the complete registration or remove the incomplete one in the settle's order before A5 is run
again.
For an adopted one,
read the slot's `.git` pointer,
the registration's `gitdir` back-link,
`commondir` and `HEAD`,
and refuse when they do not name each other;
refuse a registration whose `locked` holds words that are neither the sentence recomputed from the recorded nonce
nor a prefix of it.
`registration=`,
`registrationIdentity=`,
`pointer=`,
`phase=registered`.
Owner:
this design for the order,
the fields and what the resume does with what the scan finds;
the lock design for the scan,
the sentence and what the lock's bytes prove.
Lock moment:
D3,
and D4 is this step's write.
A kill between A5 and this write leaves one of the table's states;
the next command's scan meets it.

A7.
Retired.
The earlier revision put the lock design's own record here;
the lock design keeps none (its "The order of the add's writes" step 5,
read at `a7902e293b`),
so no write stands at this label.
The labels after it keep their numbers so that a citation from either document does not move.

A8.
The checkout snapshot,
then the origin inventory.
`zfs snapshot`,
then read back its name,
GUID and `createtxg`;
publish `<slot>.origin-inventory`,
then write `originSnapshot=`,
`originSnapshotGuid=`,
`originSnapshotCreatetxg=` and `phase=snapshotted`.
Owner:
this design.
Lock moment:
D5.
A kill leaves a snapshot the recorded name identifies,
and no inventory;
the resume rebuilds the inventory,
because its set is fixed by that snapshot's transaction group (A6).

A9.
The clone,
then the clone's initial inventory.
`zfs clone`,
read back its GUID and `createtxg`,
publish `<slot>.clone-inventory` even when empty,
then write `cloneGuid=`,
`cloneCreatetxg=` and `phase=cloned`.
Owner:
this design.
Lock moment:
D5.
A kill leaves a clone the recorded name and parent identify and possibly no inventory;
the resume rebuilds the inventory only while the clone holds no snapshot of its own (V13).

A10.
The mount,
verified in the caller's namespace,
then `phase=mounted`.
Owner:
this design with the route's attach checks.
Lock moment:
D6,
whose no-pointer window the lock protects.
A kill leaves a clone that may be mounted;
the resume reads the mount live,
and compares the identities "The identity fields" says are comparable while mounted.

A11.
The pointer inside the clone,
the verified checkout,
then `phase=filled`,
which the proposal's "Mark the binding usable only after the fill is verified" is ([proposal](cli-git-zfs-worktree-design-proposal.md):407).
Owner:
this design.
Lock moment:
D6 and D7.
A kill leaves a mounted clone whose pointer state the resume reads live.

A12.
The dependency repairs,
queued.
Owner:
the route's existing repair step.
Lock moment:
D7,
which the lock stands through.

What a failed add leaves,
at any step from A5 on,
is the accepted sentence's answer and not a rollback:
the claim,
the slot,
the locked registration and every object whose creation and identity are proven stand,
the failure is reported with the operation,
object and pool,
and the next add at that slot resumes ([proposal](cli-git-zfs-worktree-design-proposal.md):427 to :428).
This design writes no rollback,
and V16 puts the choice to the project with the lock design's rollback beside it.

### A resume after a kill between Git's add and the registered write

The claim is held and the binding's phase is `slot` (or `claimed`,
when A5 began but the `slot` write did not land),
so Git's add may have left anything from a bare directory to a complete registration.
The finder is the lock design's scan,
widened by one name check:
one listing of `<common>/worktrees/`,
one read of each entry's `locked` with the lock design's bound,
comparing it with the sentence recomposed from this add's `nonce=` (its S8 and S2,
read at `a7902e293b`),
one read of each entry's `gitdir`,
and the entry's name compared with the slot's.
The name check is what reaches the entry that holds no lock at all (D1 and Git's failure path),
which no lock scan can find,
and it is why a complete registration is never removed on a name alone.

A directory is this add's when one of these holds:

- its `locked` bytes equal the recomposed sentence;
- its `locked` bytes are a proper prefix of the sentence,
  which is what a killed single write leaves (the lock design's F8 and S2);
- it holds no `locked` and no `gitdir`,
  and its name is the slot's name or the slot's name followed by decimal digits,
  which is D1 of the lock design's moments and the directory Git's failure path leaves after it unlinks `locked`
  (its F11).

A directory whose `locked` holds other words,
or whose `gitdir` names another slot's `.git`,
is never this add's:
the resume refuses,
names the entry and the first line of its words,
and changes nothing.
It is a person's `git worktree unlock` and lock again,
or another command's add.

An entry that is this add's is complete when all five answers name each other:
the slot's `.git` pointer exists,
its bytes will be `pointer=`,
the `gitdir` file names exactly `<slot>/.git`,
`commondir` exists,
and `HEAD` exists.
That is the wrapper's own three-fact proof plus the `HEAD` the lock design's F14 names (read at `a7902e293b`).
Then:

- Complete:
  adopt it.
  Write `registration=`,
  `registrationIdentity=`,
  `pointer=` and `phase=registered`,
  and continue at A8.
- Incomplete:
  remove it in the settle's order and re-run A4 and A5.
  The states,
  each with what it holds:
  - D1,
    a bare directory,
    holds nothing.
    No recordable fact is lost:
    Git created it at `mkdir` (`builtin/worktree.c`:505 to :512) and wrote nothing in it.
  - D2,
    `locked` and nothing else,
    holds only Git's lock,
    whose bytes are recomposed from our nonce.
  - D2a,
    `gitdir` names `<slot>/.git` and the slot has no pointer:
    the pointer's bytes cannot be read,
    so `pointer=` cannot be written.
  - D2b,
    the pointer stands and `commondir` is missing:
    the wrapper's proof cannot pass.
  - D2c,
    `commondir` stands and `HEAD` is missing:
    the proof is short its last fact.
  In every incomplete case the removal is the lock design's own settle order,
  "How the wrapper's own removal releases the lock" (read at `a7902e293b`):
  1. unlink `locked`,
     so no lock stands in the way and no prune is stopped by one on a half-made directory;
  2. prove the directory empty through a descriptor opened on the pinned `worktrees` parent,
     so a name that pointed elsewhere is not removed;
  3. remove the entry by that descriptor's parent and name.
  The add prints the entry it removed and why.
- Nothing found:
  re-run A5,
  after the slot rule below.

The resume never runs `git worktree prune` instead:
a repository-wide prune would remove another slot's registration in the mounted,
no-pointer state the lock design's D6 names,
so this rule is per slot by construction.

A removal of a leftover that is itself killed is harmless:
the next resume finds what is left and repeats the same three steps,
because each step removes one fact (the lock,
then the directory) and the entry stays attributable until the last one.

The resume's other half is the slot Git's failure path may have deleted.
Git's own failure path removes the destination directory it was given,
even when that directory existed before Git ran and was empty (`remove_junk`,
`builtin/worktree.c`:273 to :288 and :536).
The second gate review measured it with Git alone in the gate image,
sending SIGTERM to the process group of twelve adds:
eight of the twelve removed the pre-existing empty slot,
and a Ctrl-C (SIGINT) is the common cause,
because Git catches it and runs that path.
A SIGKILL or a power loss there is unmeasured,
and M9 is the arm that would show it.
So at phase `slot` or `claimed` the resume may find no slot at all,
or an empty slot with another inode and birth time,
and `slotIdentity=` then answers nothing.

The rule is:

- The slot is absent,
  or an empty directory whose identity differs:
  re-run A3 and A4 and rewrite `slotIdentity=` and `phase=slot`.
  Nothing depends on the old identity yet:
  no registration is named,
  no dataset exists and no clone was made.
  The add prints that it re-made the slot.
- The slot holds a `.git` pointer or anything else:
  refuse,
  naming what stands there.
  A folder with a pointer is a registration's subject,
  not an empty slot.
- What identifies the slot across the deletion is the path (`slot=`) and the dataset
  (`checkoutDatasetGuid=`),
  never the inode.
  The inode is what A4 records after the folder is verified,
  and it is rewritten only in this window.
  From A6 on,
  a slot whose identity differs refuses,
  because the registration's `gitdir` names `<slot>/.git` and a re-created folder is another subject for it.

T20 forces every state of the table with a planted entry,
T20a the bare directory and the failure path's unlocked one,
T20b the two foreign cases,
and T21 the slot's deletion and the identity's rewrite.

## The phases, and a kill at each step

### The seven phases and the five resume states

The five states are the proposal's ([cli-git-zfs-worktree-design-proposal.md](cli-git-zfs-worktree-design-proposal.md):424 to :426).
Each line names the state,
the phases a kill can leave it at,
and the live facts that prove which state it is.
The phases may lag the live state by the step in flight,
and never lead it.

- A claimed empty slot:
  phases `claimed` and `slot`,
  with no registration at the recorded `slot` path and no dataset at `cloneDataset=`.
  Proven by the slot's folder being absent or empty and Git's registration folder absent,
  or by the only registration at the slot being one the table of "A resume after a kill between Git's add and the
  registered write" removes.
- Registration without a dataset,
  including a possible origin only:
  phases `claimed`,
  `slot`,
  `registered` and `snapshotted`.
  Proven by a registration whose `gitdir` names the slot's `.git` path and whose identity equals
  `registrationIdentity=`,
  with no clone dataset at `cloneDataset=`.
  With phase `snapshotted` the origin snapshot at `originSnapshot=` exists and its GUID equals `originSnapshotGuid=`;
  that is the "possible origin only" arm.
- An unmounted clone:
  phases `registered`,
  `snapshotted` and `cloned`.
  Proven by the clone dataset existing with `cloneGuid=` and no mount at the slot whose mount source is that dataset,
  and,
  because the underlay is what `<slot>` shows then,
  by the underlay's identity through the pinned parent equalling `slotIdentity=`.
- A mounted but unfinished clone:
  phases `cloned` and `mounted`.
  Proven by the mount at the slot whose source is the clone dataset `cloneGuid=`,
  and by the registration and the pointer,
  and not by `slotIdentity=`,
  whose object the mount covers.
- A verified checkout:
  phase `filled`.
  Proven by the registration's back-link,
  the pointer's bytes equalling `pointer=`,
  the mount's source being `cloneGuid=`,
  and the checkout's index and tracked content matching what the fill verified.

The phase vocabulary is this design's,
because the proposal says only "progress" ([proposal](cli-git-zfs-worktree-design-proposal.md):412 to :417).
V14 marks the seven words and their order as a writer's choice.

### What the writer confirms before each write

The writer's rule is one sentence:
it writes a phase only after the object that phase names answers with the values it is about to record,
and it writes that phase before it takes the next step.
So the record never claims a fact that did not answer,
and a kill at any instruction leaves a phase at or behind the truth.
"The add's writes in one order" lists each write with what it confirms;
this section gives the recovery each phase's gap needs when the fact is missing.

- The clone's identity is recoverable:
  the clone dataset's name was fixed by the add and recorded,
  and `zfs get guid` and `zfs get createtxg` answer it.
  The reader records both and goes on,
                provided the name holds a dataset whose `origin` names the recorded origin snapshot and whose parent is
        the configured clone parent.
  A name holding anything else refuses.
- The origin snapshot's identity is recoverable the same way,
  and its name carries the nonce,
  so a snapshot at that name whose `createtxg` is later than the add's claim is the add's.
- The origin inventory is rebuildable while the origin snapshot exists,
  because its set is fixed by that snapshot's transaction group (A6).
  A reader that finds no origin inventory rebuilds it and publishes it,
  and says it did.
- The clone's initial inventory is rebuildable only while the clone holds no snapshot of its own (V13).
  A reader that finds a clone,
  a snapshot of it newer than the clone,
  and no clone inventory refuses to use that clone and reports it:
  the clone's provenance can no longer be separated from history another hand put there.
- The registration is found by the scan of "A resume after a kill between Git's add and the registered write",
  not by a name.
- Nothing about the mount is recorded,
  because the mount's identity is read live on every resume.

### What no reader may conclude

A phase,
a name,
a property or a matching GUID alone proves nothing,
per the proposal's own rule ([proposal](cli-git-zfs-worktree-design-proposal.md):425 to :426).
A reader therefore never settles a fact from one observation:
the clone is the recorded clone when its name,
its GUID,
its `createtxg` and its origin agree;
the registration is the recorded one when its path,
its identity and its `gitdir` back-link agree with the pointer's bytes;
the slot is the recorded slot when,
unmounted,
the underlay's identity seen through the pinned parent equals `slotIdentity=` and the record's dataset GUID equals the
live one,
and,
mounted,
when the mount's source is the dataset whose GUID is `cloneGuid=`;
`slotIdentity=` is never compared against a mounted slot ("The identity fields");
and the origin snapshot is the recorded one when its name,
GUID and `createtxg` agree and `zfs get origin` of the clone names it.

A phase ahead of the live facts is never a lag:
it means the object was destroyed or another hand wrote the record.
The reader then refuses,
names the field that disagrees,
and changes nothing.

## Publication, and what a kill leaves

Every publication is one call that replaces nothing or one that replaces a name,
and both are the wrapper's:

- The claim:
  `take_claim`'s private file,
  one `hard_link` under the next generation,
  the private file removed,
  the folder flushed (`single_owner_claim.rs`:245 to :285).
  A kill before the link leaves the private file `.cli-git-claim.tmp.<pid>.<nanos>.<counter>`,
  which no reader reads and which `doctor` removes.
- The binding's first write and every rewrite:
  a private staging file made with `O_EXCL` and mode `0600`,
  filled whole,
  flushed,
                then renamed over `<slot>.binding` with one rename that replaces (`private_storage_replacing_a_record.rs`:50
        to :59).
  A kill before the rename leaves the staging name `.<slot>.binding.<random>.partial` and the binding as it was,
  which is the state before that write.
- Each inventory:
  a staging file made with `create_new_entry`,
  filled and flushed,
  then given its name by one `link_entry`,
  so a second publication answers `EEXIST`;
  the staging name is removed and the folder flushed (`publishing_entries_in_one_open_folder.rs`:1 to :188,
  and the store's own use of it at `slot_identity_store_publication.rs`:114 to :145).
  A kill before the link leaves `.<name>.<32 hex>.partial`.

So two staging forms exist in the folder,
`.cli-git-claim.tmp.*` and `.<record>.<random>.partial`,
and no reader reads either as a record.
`doctor` removes both forms only when the folder holds no live claim (F19 is how it knows),
and otherwise names them and leaves them,
because a writer holding the claim may own one.
That is a narrower rule than the store's,
whose doctor removes its two known forms unconditionally ([`../decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md),
"Who removes a killed writer's staging file"):
a ZFS add holds its claim for minutes across a snapshot and a clone,
where the store's writer holds nothing,
so the store's rule would break a running add.
V8 keeps it as a writer's choice.

### Durability, and why it differs from the wrapper's other add records

The wrapper's add records are flushed by nobody because the flush would be "a commit of the whole btrfs filesystem,
which waits behind every other writer's data,
and it stands on the add's waiting path" (`SPEC.md`:4421 to :4423).
On the filesystem this record lives on,
a flush is a log commit for one dataset and nothing else (F12),
so the reason does not carry over,
and this record is flushed at every publication.

The second reason is what each fact would cost if lost.
The identities are already durable in the pool when they are recorded,
so the record must not be the weaker half;
and the clone's initial inventory cannot be rebuilt once the clone holds history of its own.
What a lost record would leave is a clone dataset nobody can prove,
which is the orphan case under "The record's end".

What the flush buys against a power cut is what F12 says the source shows:
a directory `fsync` after the link commits the synchronous intents waiting in the log,
which include that link,
so the name is durable at `sync=standard` on that reading.
The earlier revision said the opposite and built an orphan case on it;
L4 withdraws that,
and M6 measures the chain in a guest with a real power cut.
Until M6 runs,
the design's answer is unchanged:
the record is flushed at every publication,
and the case where it is lost anyway is the orphan case,
reported and never guessed at.

## The readers, and what each does with a bad record

### A resumed add

A command run for a slot whose record exists takes the claim's next generation first,
then reads `<slot>.binding`.
It refuses,
changing nothing and naming the file,
when the record is:

- unreadable:
  the bytes are not UTF-8,
  the header line is not this version's,
  or a line is not one this release writes.
  The refusal names the line and offers the two ways forward:
  settle the add the record describes,
  or run `doctor`;
- foreign:
  `slot=`,
  `repository=` or `repositoryIdentity=` disagrees with the live common directory and slot,
  or `checkoutDatasetGuid=` disagrees with the dataset the record's home is on,
  or the file's name and its `slot=` disagree.
  A record copied from another repository,
  restored with a rebuilt `.git`,
  or left over from another dataset refuses here (L9);
- oversized:
  a line over the bound or a file over the binding's bound is refused by size,
  with both numbers printed;
- wrongly owned:
  the file's owner is not this account's,
  or its mode lets another account write it.
  Under this project's threat model that is a mistake rather than an attack,
  and the record still decides which dataset a destruction may target,
  so the refusal stays and prints the settling command's words;
- ahead of the live facts:
  a phase whose object does not answer,
  or answers with another identity.
  This is the one arm a reader meets when another hand destroyed an object (see "What no reader may conclude");
- a live owner of the newest claim generation:
  the only claim fact a reader refuses on,
  and the refusal names the owner's process id (see "What a reader may ask of the claim").

An add that finds no record takes the claim and writes one,
which is the normal path.

### The removal's lineage proof

The removal takes the claim,
then reads the binding and both inventories,
and refuses before its first delete when any of them is missing or unreadable,
except a clone inventory the live clone cannot contradict (V13).
It never rebuilds the origin inventory,
following the proposal's rule "never regenerate an authorized set from whatever names
are left" ([proposal](cli-git-zfs-worktree-design-proposal.md):968),
and never updates a recorded identity to match an external change ("Never update the recorded lineage to match
an external change",
[proposal](cli-git-zfs-worktree-design-proposal.md):873 to :874).
It uses the binding's fields exactly as the proposal's removal steps say:
the pool,
clone and origin GUIDs;
the origin snapshot's `createtxg`;
the clone's own `createtxg` to order its snapshots;
and the origin inventory for the third check ([proposal](cli-git-zfs-worktree-design-proposal.md):854 to :876).

### `doctor`

`doctor` lists the folder once per run (F19 says every take lists it anyway,
so a run's one listing is not a new cost),
and reports:

- the number of claims and of records,
  and each claim whose owner is alive,
  with its process id,
  which is `live_owner_of`'s answer (`single_owner_claim.rs`:416 to :441);
- each slot with a binding and no clone,
  each clone with a binding and no registration,
  and each registration with no binding,
  naming the fields that disagree;
- each record whose identity no longer matches the live folder,
  naming the identity and the dataset GUID it compared;
- each inventory whose `entries=` disagrees with its lines,
  each record filed under a name its `slot=` or `kind=` refuses,
  each file whose owner or mode is not this account's,
  and each entry whose name is none of the five forms;
- the two known staging forms,
  whose names it removes only when the folder holds no live claim;
- each released claim,
  which is one small file per slot name ever used and is not a finding.

It removes nothing else,
and it changes no record,
which keeps its promise of a read-only report except for the staging names the plan already gives
it ([plan](cli-git-zfs-worktree-implementation-plan.md),
step 18).

### Any other check that runs as the account

There is none,
and that is this design's decision rather than a record's:
no forwarded command reads this folder,
so an add's record costs nothing on any other command's path,
and the only account-run reader outside the three above is `live_owner_of`,
which an add or a removal calls and which lists the folder when it exists (F19).
V15 marks the decision,
with the wrapper's copy-journal pass as the alternative shape.

## The record's end

A removal writes nothing into this binding.
Its own plan is another record,
whose home is the destruction plan design's,
and the binding's phase stays at `filled` for the whole removal.
What the removal does with this binding is read it,
compare its origin fields with the origin cleanup record,
and remove it at the step the route's one removal order places.

That order is the origin cleanup design's list,
and this design states none of its own:
"The order,
and what a kill at each point leaves" at `987a3ba886`,
its steps 1 to 11.
Read at that commit,
the list ends at step 11,
the plan's removal,
and it does not yet hold this binding's removal or the claim's release;
that round is writing one merged list of every write and removal a removal makes across the three records and the
registration.
What this record needs from that merged list is three rules,
stated here as requirements and not as step numbers:

- this binding's three files are removed before the plan's removal,
  so the plan is the last witness of a removal that was killed at its end;
- the claim is released last,
  after the binding's removal and the plan's;
- the origin cleanup record's confirmation stays where that list puts it,
  before the first delete (its step 6),
  and this record's confirmation of it is the same step's other half.

V20 marks those three rules as the writer's choice they are until the merged list lands,
and the list's round owns the numbering.

What a removal does with this record is then,
in this record's terms:

1. It reads the binding and both inventories under the claim,
   and proves lineage (its step 1,
   and "The removal's lineage proof").
2. It confirms that the origin cleanup record stands,
   and that its origin fields equal this record's `originDataset=`,
   `originDatasetGuid=`,
   `originSnapshot=`,
   `originSnapshotGuid=` and `originSnapshotCreatetxg=`.
   It writes nothing there:
   that record's writer is the removal command,
   and its own design publishes it at its step 6 (its V4,
   read at `987a3ba886`).
   A removal that finds no such record,
   or one whose fields differ,
   refuses before its first delete,
   which is the same refusal the origin cleanup design gives a removal that cannot write its own record.
3. The destroys are the siblings' steps (its steps 7 to 9).
4. It settles the registration,
   which either Git removes with its folder (F2,
   A4) or the wrapper's own settle removes by descriptor,
   and releases the managed lock (its step 10).
   The binding is not inside the registration,
   so it survives either route.
5. It removes `<slot>.binding`,
   `<slot>.origin-inventory` and `<slot>.clone-inventory`,
   then releases its claim,
   which leaves the newest generation file as a released claim.
   It prints what it removed.
   This happens whether or not the origin's cleanup is still blocked,
   because the origin's identity now lives in that record and nothing about the pending snapshot depends on the
   slot:
   the proposal decides that "settling does not wait for the origin" and that "settling frees the slot
   but requires recovery without its Git entry" ([proposal](cli-git-zfs-worktree-design-proposal.md):468 to :469
   and :1460 to :1463).

So the binding is the carrier of the origin's identity only up to step 2,
and the slot is freed at the settle rather than at the cleanup's end.
The earlier revision kept the binding while the origin cleanup was blocked,
which the second gate review's X2 showed contradicts those two sentences.
The binding stays,
and the slot stays blocked,
only when step 2 cannot be confirmed:
no origin cleanup record,
one that cannot be read,
or one whose fields disagree.
Then the binding is the only record that names the origin,
and keeping it is the safe arm X2 allows.

### What a record whose clone is gone means

The clone's GUID is absent from the pinned clone parent,
and no child of that parent carries `cloneGuid=`.
Two readings follow from the order above,
and the order is what tells them apart.

- A destruction plan stands for `cloneGuid=`.
  A removal is in flight or was killed,
  wherever it was killed:
  because V20 puts the plan's removal after this binding's,
  a kill after the settle and before step 5 leaves the plan standing beside this binding.
  `doctor` prints the plan's standing intent and this record's origin names,
  and the next removal resumes from the plan and finishes step 5.
  If the plan reports the head's destroy as unresolved,
  nothing in the binding is regenerated and nothing is settled on a guess.
- No plan stands.
  Then either the clone was never destroyed by this wrapper (nothing else can remove a plan on the normal path),
  or a removal broke V20's order,
  and both are reported as an outside change:
  the reader requires "the name is absent and no child of the pinned parent carries the recorded GUID" before it
  settles anything,
  it destroys nothing,
  and it names which of the two it cannot rule out.
  The earlier revision's third reading,
  "no plan and an origin record stands",
  is now one arm of this case:
  the origin record standing without a plan does not prove a removal,
  because a removal that reached its origin record also wrote a plan and removes that plan last.

A binding whose clone is gone and whose absence cannot be proven,
or whose step 2 cannot be confirmed,
is kept,
and the slot it names refuses the next add until a person settles it.
That is Q8,
because it trades a blocked slot against a guessed settlement.

The command that finishes step 5 when the removal that owned it is dead is the next removal of that slot,
which resumes from the plan,
or,
when an add reaches the slot first,
the add's own settle under the claim before it writes its own binding.
The plan design's finish pass is not one of them:
it holds no claim,
so it must leave the plan standing (V20) rather than remove the last witness of a binding that still stands.

## What Git does to a file it does not know here

Measured,
this round,
exactly as F6 records it,
in run B through real Git `2.56.1` inside the gate image and repeated in run A through the wrapper,
arm for arm.
The candidate places were measured together in each arm,
and the result decides the home:

- Inside a registration (`<common>/worktrees/<id>/<slot>.binding`),
  the record is deleted by `git worktree remove` whatever the force level (F2,
  arm 2b),
  by an explicit `git worktree prune` the moment the slot's `.git` pointer is missing (F3,
  arm 3),
  and by `git gc --prune=now` once the registration's `index` is absent or older than `gc.worktreePruneExpire` (F3,
  F4,
  arms 4,
  5 and 6).
  Those arms ran on an unlocked registration;
  with the managed lock in place,
  arm 9 shows prune and gc keep the registration while the lock stands.
- Beside the wrapper's own folder,
  in this record's folder
  (`<common>/monochromatic-git-policy-cli/worktree-clones/v1/...`),
  every one of those commands left the folder's files exactly as they were,
  and `git worktree repair` left them too (arms 1,
  2b,
  3 and 7).

The argument against a registration-internal home is therefore F2 and the settle order,
not prune:
`git worktree remove` deletes the registration's whole directory at the settle,
and this record must outlive the settle until the origin cleanup record holds the origin's identity.
A record whose facts die with the registration may live there,
and the `worktree-added` record the wrapper already writes does (`worktree_added_by_the_wrapper.rs`:1 to :26);
this one may not,
because the origin's identity must outlive the settle.
A lost lock would also reopen the prune arm,
which arm 9 shows is otherwise closed.

Git never lists or reads the folder this design chooses (F5),
and for a worktree it creates `<common>/worktrees/` and the registration inside it,
and nothing else in the common directory.

The one Git behaviour the home must live with is F1's counter,
which arm 8 of run B measured:
a stale registration of the same name makes Git name the new one `<name>1`.
The design answers it by recording the registration's path and identity from Git's own answer,
never by deriving it from the slot,
by refusing a binding whose `registration=` no longer answers with `registrationIdentity=`,
and by the scan of "A resume after a kill between Git's add and the registered write" when there is
no recorded path yet.

## Which step of the plan builds which part

- Step 10 builds the home,
  the claim's use of `take_claim`,
  the binding's phases and fields,
  the two inventories,
  the add's writes in one order,
  and the resume rules:
  the folder and its modes,
  the four name forms,
  the publication and flush of each write,
  the finder of a registration,
  the table of what a resume does with each state Git's add can leave,
  the slot rule when Git's failure path deleted it,
  the checks a resumed add makes,
  and the refusals.
  It also builds the add's own settle of a standing binding whose clone is gone and whose removal is dead,
  which is the same three files under the same claim as step 16's step 5.
  Its "Design sections" gain this document,
  and `SPEC.md`'s "### The records an add writes for itself" gains the claim and binding records,
  as the plan already says ([plan](cli-git-zfs-worktree-implementation-plan.md):1439 to :1443).
- Step 14 builds the removal's reading of the binding and both inventories,
  the refusals of "The removal's lineage proof",
  and the claim the removal takes.
- Step 16 builds the record's end,
  which is this design's assignment rather than the plan's:
  plan step 14 sends origin cleanup and the Git settle to step 16,
  and the record's removal belongs with them.
  It builds absence proven by the recorded GUID,
  the hand-over to the origin cleanup record,
  the removal of the three files,
  and the released claim.
- Step 18 builds `doctor`'s listing,
  the four checks of "`doctor`" and the staging-name removal rule.
- Step 12 needs no part of it:
  a pruned or moved worktree is read from Git's own registrations,
  and the binding is not consulted for it.

## The tests a build must have

Each test names the race or the state it forces and how it forces it without timing luck.
The wrapper's own seams are the model:
a `thread_local` hook between two steps,
as the store's publication has (`slot_identity_store_publication.rs`:387 to :400 and :505 to :510) and
as `take_claim` has (`single_owner_claim.rs`:357 to :370).

Unit tests,
no guest needed:

T1.
`one_slot_has_one_claimer`:
two `take_claim` calls on one folder,
the second answering `Busy` with the first's process id;
then a claim planted for a dead birth,
the next call taking the next generation;
then a released claim,
the next call taking the next generation.
The races are forced by planting the claim files,
not by running two threads against the clock.

T2.
`a_binding_cannot_be_written_without_the_held_claim`:
the writer takes the held claim as its argument,
so the refusal is the type's and no runtime arm can reach it;
the test shows the other direction,
that a second command which cannot take the claim leaves the binding's bytes unchanged and names
the live owner's process id.

T3.
`a_killed_writer_between_the_binding_and_its_rename_leaves_the_previous_record_and_one_staging_name`:
the hook between fill and rename runs;
the record is read back as the previous one,
and `doctor`'s staging rule is shown removing the staging name only when no live claim stands.

T4.
`a_killed_writer_between_the_claim_and_the_first_binding_write_leaves_a_claim_and_no_mutation`:
the hook after the claim runs;
the slot,
the clone parent and the origin dataset are read as unchanged.

T5.
`a_kill_at_each_confirmation_leaves_no_phase_ahead_of_the_facts`:
one arm per phase,
each with a fake pool answer that returns the identity,
then a kill before the phase write;
the reader's phase-versus-live check is asserted at every arm.

T6.
`the_clone_identity_is_recovered_from_the_pool_and_nothing_else_is`:
the clone exists with the recorded name and GUID but no `cloneGuid=` line;
the reader records it.
Then a foreign dataset planted at the recorded name:
refused.

T7.
`a_clone_holding_no_snapshot_of_its_own_has_its_missing_initial_inventory_rebuilt`:
phase `cloned`,
no clone inventory,
no clone snapshot:
rebuilt and reported.
Then a clone snapshot newer than the clone:
refused.

T8.
`a_record_under_another_slots_name_refuses`,
`a_record_naming_another_repository_refuses`,
`a_record_naming_another_dataset_refuses`,
`a_record_naming_another_slot_identity_refuses`:
four planted records,
each refused by the field it contradicts.

T9.
`an_oversized_line_and_an_oversized_binding_are_refused_by_size`:
a line one byte over the widest form and a file one byte over the bound,
each refused with both numbers.

T10.
`an_inventory_whose_count_disagrees_with_its_lines_refuses`:
a truncated inventory and one with an extra line.

T11.
`a_record_owned_by_another_account_or_writable_by_others_refuses`:
the file's owner and mode are changed in the fixture,
and the refusal names the terms.

T12.
`a_phase_ahead_of_the_live_facts_refuses_and_changes_nothing`:
`phase=cloned` with no clone,
and `phase=filled` with a different pointer.

T13.
`the_home_is_resolved_from_the_main_checkout_only`:
a fixture where a clone's copied `.git` holds a record folder,
and the resolver is asked from the main checkout;
the clone's copy is never read (F8,
L6).

T14.
`doctor_names_each_orphan_and_removes_only_the_two_staging_forms_with_no_live_claim`:
a fixture with a slot record and no clone,
a clone and no record,
a registration and no record,
both staging forms,
a foreign dot entry and a released claim;
the count and the removals are asserted.

T15.
`the_removal_refuses_before_its_first_delete_without_a_binding`:
an arm whose binding is absent,
one whose origin inventory is absent,
one whose inventory is truncated,
and one whose clone GUID disagrees;
each refusal leaves every planned snapshot's identity readable in the fixture.

T16.
`the_record_is_removed_only_after_the_origin_cleanup_record_exists`:
an arm where the confirmation of step 2 fails,
and the three files stand;
an arm where it succeeds,
and they are gone with the released claim left;
and an arm where a removal was killed after the settle and before step 5,
which leaves the plan standing beside the binding,
and the next removal resumes from the plan and finishes step 5,
so no kill leaves the binding standing where "What a record whose clone is gone means" reads it as an outside
change.

T17.
`an_identity_without_a_device_matches_after_the_dataset_is_mounted_again`:
the fixture records an identity,
then the record is read after the fixture's own device number is changed in the stored line and the stored line
is otherwise equal:
the record still matches,
and the same line with another inode refuses.
A third arm asserts the table's mounted side:
a record whose `cloneGuid=` equals the mounted dataset's source is accepted while `slotIdentity=` names the
underlay,
and a record whose `cloneGuid=` differs refuses.
This is a unit test because it asserts the comparison's table,
not the kernel's number;
the guest arm that would measure the mount itself is the lock design's M2,
owed there.

T18.
`a_binding_read_after_two_takes_does_not_refuse`:
generation 1 writes `claim=1` and is released;
`take_claim` takes generation 2,
which removes file 1;
the binding is read under generation 2 and accepted.
The record's `claim=` is informational in both directions:
a binding whose `claim=` names 3 is accepted while generation 2 is the newest,
and `doctor` reports it,
and the only refusal is a newest generation whose owner is alive,
which refuses with that owner's process id.

T19.
`the_reader_compares_the_dataset_guid_and_the_inode`:
a record whose `checkoutDatasetGuid=` differs from the dataset the fixture's folder reports refuses,
and one whose inode differs refuses,
and one whose stored identity is the `-:-` form reads.

T20.
`a_resume_reads_each_state_gits_add_can_leave`:
one arm per state of the table,
each planted rather than raced:
D3,
a complete registration whose `locked` holds the recomposed sentence,
is adopted and written as `registered`;
D2,
`locked` and nothing else,
D2a,
`gitdir` naming `<slot>/.git` with no pointer,
D2b,
a pointer with no `commondir`,
and D2c,
`commondir` with no `HEAD`,
are each removed in the settle's order and the add re-runs A5;
and a kill during that removal leaves the entry attributable and the next resume repeats it.
Each arm asserts the three removal steps in order:
`locked` unlinked,
the directory proved empty through the descriptor,
the name removed.

T20a.
`a_bare_directory_and_a_failure_path_leftover_are_removed_before_the_add_reruns`:
D1,
a bare `worktrees/<slot>`,
and the directory Git's failure path leaves after unlinking `locked`,
both of which hold no `gitdir` and no lock,
are removed,
and the re-run's registration is at the slot's own name and not at `<slot>1`,
which F1's counter would otherwise make.

T20b.
`a_foreign_lock_and_a_foreign_gitdir_are_refused`:
a registration whose `locked` holds other words,
and one whose `gitdir` names another slot,
are both refused with the entry and the words' first line,
and nothing is removed.

T21.
`a_slot_identity_is_written_only_after_the_slot_exists`:
the phase `claimed` write carries no `slotIdentity=`,
and the phase `slot` write refuses when the slot is absent.
Then the arm the second gate review's N2 asks for:
a resume at phase `slot` whose slot Git's failure path deleted,
or whose empty slot answers another identity,
re-runs A3 and A4 and rewrites `slotIdentity=`;
a slot holding a `.git` pointer refuses;
and from phase `registered` on,
a slot with another identity refuses.

T21a.
`a_resumed_add_whose_request_differs_refuses`:
a record written by `request=branch 6d61696e`,
resumed by an add given `request=commit 4f1c...`,
refuses and names both,
and resumed by an add given the same request,
continues.

Guest tests,
because they need a real pool,
a real mount or the root handler:

T22.
`the_origin_inventory_records_the_hourlies_and_a_bookmark`:
a fixture pool whose origin dataset holds the fixture's hourlies and a bookmark;
the record's 11 fixed lines and its entry lines equal the `zfs list` answer,
`entries=` matching,
and a bookmark's GUID and `createtxg` equal the snapshot's (F10).

T23.
`a_kill_between_the_clone_and_its_record_is_resumed`:
the add is killed by the planned seam after `zfs clone` returns and before the phase write;
the resumed add records the recovered GUID and finishes;
the clone the fixture holds is the one the record names.

T24.
`a_kill_between_git_add_and_the_registered_write_is_resumed`:
the add is killed after `git worktree add` returns and before A6's write;
the resumed add finds the registration by the scan,
proves the lock from the nonce,
and finishes;
a second arm plants no registration and the resume re-runs Git.

T25.
`the_record_survives_prune_and_gc_while_the_clone_is_unmounted`:
a real `git worktree prune` after deleting the underlay pointer,
with the record in the place this design chooses;
the record and the clone dataset stand,
and the registration stands while the managed lock is there (arm 9) and goes when it is not.

T26.
`ENOSPC_at_the_snapshot_leaves_the_claim_the_slot_and_the_locked_registration`:
the fixture fills the pool,
the snapshot refuses,
and the claim,
the slot and the registration stand with nothing else made,
which is the accepted sentence ([proposal](cli-git-zfs-worktree-design-proposal.md):427 to :428).

T27.
`the_boot_arm_reads_a_record_written_before_the_guest_rebooted`:
record a binding whose `slotIdentity=` and `checkoutDatasetGuid=` are real,
reboot the guest,
and read the record;
it must match,
and the same arm repeated with an export and import of the dataset must match too.
This is M7's arm and step 8's boot arm,
run once for both.
T28.
`a_flushed_write_is_read_back_after_the_guest_loses_power`:
guest only,
in the M6 arm below.

## Measurements owed

M1.
The flush cost of one add's record writes.
Measure,
in the guest,
the wall time of the seven publications this design makes with a flush against the same seven without one,
both arms interleaved in one round,
and report the ratio with its spread.
This sizes the design's choice to flush every write (rule `LQM`,
and `TVC`).

M2.
The inventory's cost,
its widest real shape,
and the record folder's size.
Measure,
in the guest,
the time to build the origin inventory from `zfs list` for an origin holding the fixture's hourlies and one bookmark,
and the bytes it took;
then repeat with 500 snapshots and with 5,000,
and report both counts' bytes and times.
Measure too the size of the record folder after 100 adds and 100 removals,
because every take lists it (F19) and one released claim per slot name stays.

M3.
The count a cap would use,
if the project wants one.
From M2's arms,
report the largest count whose build and read stay under the add's own measured wait,
so a cap is derived from a measurement rather than chosen.

M4.
The claim's liveness across a reboot and across a second PID namespace.
Measure,
in the guest,
that a claim written before a reboot is taken by the next add as dead,
and that a claim whose owner is in another namespace blocks for exactly the reused module's patience and no longer.

M5.
Link,
rename and flush behaviour on the checkout dataset.
Measure,
in the guest,
that `hard_link`,
the replacing rename,
`create_new_entry` with `O_EXCL` and the store's fallback `renameat2` with `RENAME_NOREPLACE` all answer as
the store's design requires on the ZFS dataset that holds `.git`.
F6 measured them on the host's ZFS home dataset as a plain directory and on the gate image's container filesystem,
neither of which is the checkout dataset.

M6.
The name's durability against a real power cut.
Measure,
in the guest,
whether a directory `fsync` after a link or a rename makes the name survive a hard power cut of the guest,
at `sync=standard` and at `sync=always`.
F12 is a source reading that says it does at `sync=standard`;
this arm is the run that decides it,
and dropping the page cache is explicitly not the arm,
because `drop_caches` loses no unflushed data.

M7.
The identities across a boot,
a remount,
an export and an import.
Measure,
in the guest,
that a folder's inode and birth time still answer the recorded values after the guest reboots and after the dataset
is exported and imported,
which is step 8's boot arm run for this record's fields too (F16,
L11).

M8.
A failure taxonomy for the add's steps.
Measure,
in the guest,
what `ENOSPC` and a refused mount leave at A5 to A11,
and that the accepted sentence's "keep the claim,
the slot and the locked registration" is what a resume meets.

M9.
Git's failure path under a SIGKILL and under a power cut.
Measure,
in the guest,
whether a SIGKILL and a hard power cut inside `git worktree add` also remove the destination directory Git was
given,
which the second gate review measured only under SIGTERM (eight of twelve adds),
and what shape the registration is left in at each of D1 to D2c.
That number is what tells a reader whether this design's slot rule meets the common case or the rare one.

## Where the siblings disagree

Each line names this design's position,
the sibling's,
and which record decides it if any does.
The siblings are read at their current tips,
the lock design at `a7902e293b`,
the destruction plan at `76849b8d9b`,
and the origin cleanup record at `987a3ba886`,
and nothing here adopts their answer.
The second gate review's "Disagreements between the three record designs" is the full list,
and "The route's record convention" is where this design answers it.

- The lock design's "The record whose nonce the lock's proof rests on":
  the claim carries nothing new and the binding's `nonce=` is the one root of attribution,
  written at A2 before Git runs.
  This design agrees and states it as V11;
  the lock design reads `nonce=` and `registration=` and never `claim=`.
- The lock design's V2-A and "What the wrapper writes beside the registration":
  a record inside the registration holds the lock's bytes.
  This design needs no such record for attribution,
  and its own record cannot live there (F2).
  The two positions do not conflict,
  and V11's ranking puts the nonce first.
- The lock design's "A failed add":
  it keeps what the accepted sentence keeps and writes no rollback,
  in its own words "This design does not remove a registration after a failed add".
  This design keeps the same state (A12's paragraph),
  so the two agree and the accepted text decides ([proposal](cli-git-zfs-worktree-design-proposal.md):427 to :428);
  V16 keeps the rollback reachable for the project's veto.
- The lock design's D0 to D7 against this design's A0 to A12:
  each step of the order above names its moment,
  so the two documents no longer describe two orders.
  The mapping is A0 to A4 at D0,
  A5 across D1 to D3 and Git's failure path,
  A6 at D3 and D4,
  A8 and A9 at D5,
  A10 at D6,
  A11 and A12 at D6 and D7,
  and A7 retired because the lock design writes no record (its "The order of the add's writes" step 5).
  A kill inside Git's add,
  D1 to D2c,
  is this design's A6 and the table under "A resume after a kill between Git's add and the registered write":
  the lock design's S8 scan finds the entries and this design decides what happens to each.
  The next pass judges whether that is one order.
- The lock design's two-fact variant compares the slot's recorded identity at a path the mount may cover.
  This design's "The identity fields" states the same rule from this side:
  unmounted,
  the identity through the pinned parent;
  mounted,
  the mount's source and the registration,
  never the underlay's inode.
  The guest arm that would measure it is the lock design's M2,
  owed there.
- The destruction plan keys its record by the clone's GUID and keeps no pointer inside a registration
  (its "What finds a plan",
  read at `76849b8d9b`).
  This design keys by the slot and keeps its record outside any registration.
  Neither record can be read by the other's key alone,
  and the set still reaches from any one key through this record's fields (V17).
- The destruction plan publishes unsynced by default (its Choice 5),
  and this design flushes every write;
  the origin cleanup record is unflushed too (its option C premise and its F21).
  The route has two rules today,
  and V19 with M1 is where one rule would come from.
- The destruction plan's finish pass settles a registration and removes the plan,
  and it holds no claim.
  This design's V20 requires the plan to stay until this binding is removed,
  so the pass must leave a plan whose binding still stands.
  This is the second gate review's X3,
  and the placement is the plan design's.
- The origin cleanup record publishes at its step 6,
  after every refusal and before the first destroy,
  and says a new add may take the freed slot at once.
  This design agrees with both:
  step 2 of "The record's end" only confirms that record,
  and V20 puts this binding's removal before the plan's.
  The proposal decides the freeing ("settling does not wait for the origin",
  [proposal](cli-git-zfs-worktree-design-proposal.md):468 to :469).
- The origin cleanup record's moment is its own V4,
  still open to the project's veto.
  This design's step 2 depends on that veto:
  if the owner moves the origin record later than the first delete,
  this design's step 2 reopens with it.
- The origin cleanup record removes no staging entry and reports the ones it finds,
  where this design removes one only when no live claim stands (V8).
  Each rule is its own document's,
  and the store's decision covers the store alone.

## Choices this design marks for the project's veto

Each entry is a writer's choice adopted so the design is whole.
Each gives the options,
an example for each,
a pros and cons list,
and a ranking with the reason for each adjacent pair.
V1 to V15 stand as the last revision left them,
and V16 to V20 are the choices this revision moved here from its questions,
because the second gate review judged them writers' choices rather than the project's.

V1.
The folder's parent and its inner name.

- A,
  under the wrapper's own folder (adopted):
  `<common>/monochromatic-git-policy-cli/worktree-clones/v1/`,
  example `.git/monochromatic-git-policy-cli/worktree-clones/v1/add-cache-layer.binding`.
  Pros:
  two of the route's three records and the wrapper's own folders are already there (F20);
  one parent for a person to learn.
  Cons:
  a reader of this folder sees other records' folders beside it,
  so a listing of the parent is not a listing of this record.
- B,
  a top-level folder,
  as the earlier revision had it:
  `<common>/monochromatic-git-policy-cli-worktree-clone/v1/`,
  example `.git/monochromatic-git-policy-cli-worktree-clone/v1/add-cache-layer.binding`.
  Pros:
  one listing is exactly this record family,
  as the copy journal's folder is.
  Cons:
  a second parent convention in one route,
  and a new top-level name the rename machinery most likely needs its own entry for (F20,
  inferred).
- Ranking:
  A,
  then B.
  A beats B because the rename machinery and two sibling designs already point at the parent,
  and B's exact listing is bought with a naming rule the repository would have to learn twice.

V2.
The key:
the slot's name,
the request nonce,
the clone's GUID,
or a folder per add.

- A,
  the slot's name (adopted):
  example `add-cache-layer.binding` for the add at `.claude/worktrees/add-cache-layer`.
  Pros:
  every reader knows it first;
  no rename when the clone appears;
  `doctor` maps a slot to its record without naming anything else.
  Cons:
  a settled slot reused later needs the earlier record settled first,
  so a stale record blocks a slot until it is settled.
- B,
  the request nonce:
  example `9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f.binding`.
  Pros:
  unique per add for certain,
  so two adds never touch one file.
  Cons:
  `doctor` must list the folder and read every file to answer for one slot,
  and each slot name accumulates one file per add forever.
- C,
  the clone's GUID:
  example `4821193048571903102.binding`,
  as the mount receipts are filed under a GUID (`clone_mount_receipts_folder.rs`).
  Pros:
  the name is the clone's own identity,
  and a removal that knows only the GUID finds it.
  Cons:
  the GUID does not exist before the clone,
  so the claim and the earlier phases need another name,
  and the record is renamed the moment the clone appears,
  which needs a two-name read window.
- D,
  a folder per add,
  `<slot>/`,
  claimed by `mkdir`:
  example `.git/.../add-cache-layer/binding`.
  Pros:
  the folder's creation is the exclusive claim,
  one call with no link.
  Cons:
  a folder per add where the wrapper's records are one file per item,
  an empty folder when a kill lands between `mkdir` and its bytes,
  and a recursive removal at the end.
- Ranking:
  A,
  then B,
  then D,
  then C.
  A beats B because A costs no listing and leaves one file per slot name rather than one per add.
  B beats D because the wrapper's records are files,
  and a folder would be a second shape to publish,
  bound and remove.
  D beats C because a rename at the GUID's arrival reopens the read window that every other choice closes.

V3.
Two files (claim and binding) or one file that grows.

- A,
  two files (adopted):
  `<slot>.claim.<n>` for exclusivity,
  `<slot>.binding` for the facts.
  Pros:
  the claim's grammar stays the shared one a fuzz sidecar reads;
  the binding never changes owner under a reader;
  the takeover of a dead owner's claim is the kernel's `EEXIST` on a number,
  not a rewrite of the record.
  Cons:
  two names per operation,
  and every take lists the folder (F19).
- B,
  one file that grows:
  `<slot>.binding` created with `O_EXCL` and rewritten,
  taken over by `RENAME_NOREPLACE` to an abandoned name.
  Pros:
  one name,
  and the proposal's "exclusive per-add record" is literally one file.
  Cons:
  the takeover leaves an abandoned file,
  and a reader that trusts a field must first decide its owner's liveness from the same bytes it is reading.
- Ranking:
  A,
  then B.
  A wins because exclusivity and content change at different rates,
  and the wrapper already has a tested exclusive primitive to reuse,
  while B would need a new takeover rule and its litter.

V4.
The two inventories as separate immutable files or as sections of the binding.

- A,
  separate files (adopted):
  `<slot>.origin-inventory` and `<slot>.clone-inventory`.
  Pros:
  each is written once and never rewritten,
  so a reader needs no merge;
  the binding's rewrites stay small;
  a truncated inventory is refused by its own count.
  Cons:
  three files per add,
  and `doctor` and the removal read three names.
- B,
  sections inside the binding:
  one file whose inventory sections grow.
  Pros:
  one file to settle.
  Cons:
  every phase write rewrites up to the whole history,
  and a kill during a rewrite would have to be told from an append.
- Ranking:
  A,
  then B.
  A wins because the rewrite of a whole inventory is a cost the add would pay at each identity,
  and because B makes a single rename carry two facts of different lifetimes.

V5.
The claim's reuse of `take_claim`.

- A,
  reuse `take_claim` with prefix `<slot>.claim` (adopted).
  Pros:
  one tested lock-free claim with birth-identity liveness,
  generations,
  flush and release;
  the fuzz sidecar already reads its grammar.
  Cons:
  it lives under `src/native/lock/` with a subvolume error type the build must map,
  its temp names appear in this folder,
  and every take lists the folder (F19).
- B,
  a per-slot folder with `O_EXCL` and no generations.
  Pros:
  the module's generation machinery is not needed for a claim that is only ever taken by one add.
  Cons:
  taking over a dead owner's claim needs the record moved aside,
  which reintroduces the takeover B of V3 avoids.
- Ranking:
  A,
  then B.
  A wins because a dead owner's claim must be superseded without a move,
  and the number does that with one `link`.

V6.
Durability:
flush every write,
flush only what cannot be rebuilt,
or flush nothing.

- A,
  every write (adopted).
  Pros:
  the record is never weaker than the pool it describes,
  and the clone's initial inventory cannot be lost.
  Cons:
  seven flushes on the add's path,
  each a log commit (F12),
  and their cost is unmeasured (M1).
- B,
  only the clone's initial inventory.
  Pros:
  one flush.
  Cons:
  the identities a removal proves would rest on the page cache across a power cut,
  which is the record the removal trusts most.
- C,
  nothing,
  as the wrapper's other add records.
  Pros:
  no flush on the add's path,
  matching the measured btrfs reason.
  Cons:
  that reason names btrfs,
  and the clone's inventory cannot be rebuilt.
- Ranking:
  A,
  then B,
  then C.
  A beats B because the record's whole value is being the durable half of a clone the pool already committed.
  B beats C because C rests on a rule written for another filesystem.

V7.
Completeness by an `entries=` count,
or a total byte cap,
or neither.

- A,
  `entries=` (adopted).
  Pros:
  a truncated file is refused;
  no cap is shipped,
  so no unmeasured limit exists;
  the document can still print the measured count.
  Cons:
  nothing stops a 10,000,000-entry file from being read except its own size.
- B,
  a total cap with a refusal.
  Pros:
  a reader's cost is bounded.
  Cons:
  the cap would be a shipped limit with no measurement behind it (rule `LQM`),
  and it would refuse an add on an origin with a long history.
- Ranking:
  A,
  then B.
  A wins because the count gives the safety B was wanted for without a number nobody measured,
  and M2 and M3 are how B would earn its number.

V8.
What `doctor` removes.

- A,
  the two known staging forms when no live claim stands (adopted).
  Pros:
  a running add is never broken;
  a killed writer's litter goes at the next run.
  Cons:
  litter waits while any claim in the folder is live,
  and the folder is listed to find out (F19).
- B,
  the two forms unconditionally,
  as the store's doctor does.
  Pros:
  simpler,
  and the store's own decided rule.
  Cons:
  removing a live writer's staging file makes its publication fail,
  and a ZFS add holds a claim across a snapshot and a clone,
  where the store's writer holds nothing.
  The origin cleanup record removes none and reports them (read at `987a3ba886`),
  so neither sibling takes this option today,
  and it stays listed for the store's precedent alone.
- Ranking:
  A,
  then B.
  A wins because the store's rule was decided for a writer that holds nothing,
  and B would break an add that is merely slow.

V9.
The removal takes the same claim as the add.

- A,
  the same claim (adopted).
  Pros:
  one slot has one operation at a time;
  a concurrent add cannot change the record under a removal's proof.
  Cons:
  the removal's claim behaviour must be built into steps 14 and 16,
  and the sibling cancellation design must release it.
- B,
  a separate removal claim.
  Pros:
  the add's and the removal's concerns are separate files.
  Cons:
  two exclusivity mechanisms must agree about one slot,
  which is the failure this design's claim exists to prevent.
- Ranking:
  A,
  then B.
  A wins because one slot is one item,
  and rule `NLK`'s "one owner per item" is exactly that.

V10.
The record's end.

- A,
  removed once the origin cleanup record is confirmed,
  blocked or not (adopted).
  Example:
  the origin cleanup record stands with `owed` and this record's origin fields;
  the removal settles the registration,
  removes the three files and releases the claim,
  and a new add may take the slot at once.
  Pros:
  the proposal's "settling does not wait for the origin" and "settling frees the slot" hold as written,
  and the origin's identity is never homeless.
  Cons:
  the binding's origin fields must be compared with that record before anything is destroyed,
  so the removal depends on a record it does not own.
- B,
  never removed,
  with a `settled` phase.
  Pros:
  the removal's history survives in the slot's record.
  Cons:
  one file per slot forever,
  and a slot's reuse must distinguish a settled record from a standing one.
- Ranking:
  A,
  then B.
  A beats B because the proposal's own sentences decide the moment,
  and B turns every slot into permanent state that `doctor` must explain.
  The earlier revision's option "kept until the origin cleanup finishes" is gone:
  the proposal decides that settling does not wait for the origin,
  so it was an option no reader could adopt.

V11.
Where the managed lock's ownership is proved (the review's first question,
now this entry).

- A,
  a second record inside the registration holds the lock's exact bytes:
                example `<registration>/monochromatic-git-policy-cli-managed-lock` holding
        `monochromatic-git-policy-cli holds this registration ...`,
  with `locked` equal to it.
  Pros:
  a byte comparison with no recomputation.
  Cons:
  a second home and a second durability rule;
  a power loss can keep `locked` and drop the record,
  which then reads as a foreign lock.
- B,
  the sentence is a pure function of the binding's `nonce=`,
  and the proof recomputes it (adopted):
  example,
  the recorded nonce `9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f` recomputes the sentence,
  and a person's own lock reading `held` refuses.
  Pros:
  one record family,
  one durability rule,
  and attribution that survives the registration's loss.
  Cons:
  an unreadable binding makes every lock foreign,
  which the removal refuses on anyway.
- C,
  the claim file carries the nonce and the bytes:
  example `<slot>.claim.1` gains two lines.
  Pros:
  the evidence exists before Git runs by construction.
  Cons:
  changes the shared claim primitive and its fuzz decoder,
  and a released claim loses the fields.
- Ranking:
  B,
  then A,
  then C.
  B beats A because it removes a file,
  a durability mismatch and an unprovable power-loss state while keeping the proof a byte comparison.
  A beats C because it leaves the shared primitive alone.

V12.
A record with a key this release does not know.

- A,
  refuse the record (adopted):
  example a `binding` with `tariff=3` refuses and names the line.
  Pros:
  damage and a later grammar are both caught at once;
  a version line is the only way to change the grammar.
  Cons:
  it departs from the wrapper's own readers,
  which skip what they do not ask for (`worktree_add_binding.rs`:77 to :87).
- B,
  skip unknown lines,
  as the wrapper does:
  example the same record is read and the line ignored.
  Pros:
  one convention with the wrapper's other records,
  and a later grammar that only adds fields reads in an older release.
  Cons:
  a record whose writer added a field that changes a meaning would be read under the old meaning.
- Ranking:
  A,
  then B.
  A beats B because a field this release cannot read may decide a destruction,
  and the version line costs one line to change.

V13.
A clone whose initial inventory file is missing.

- A,
  rebuild it while the clone holds no snapshot of its own,
  and refuse otherwise (adopted):
  example a clone made a minute ago with no snapshot,
  whose file a kill lost,
  is rebuilt from the live inventory;
  a clone with its own hourly snapshot refuses.
  Pros:
  a lost file costs no clone,
  and the refusal is exactly where provenance becomes unprovable.
  Cons:
  a rebuild makes the recorded set depend on a live read.
- B,
  refuse every missing clone inventory,
  as the earlier revision did.
  Pros:
  one rule,
  no live rebuild.
  Cons:
  it refuses a clone that nothing can contradict,
  which M6's arm would meet often.
- Ranking:
  A,
  then B.
  A beats B because the set is fully determined until the clone's first snapshot,
  and B pays a blocked slot for a fact that is still provable.

V14.
The seven phase words and their order.

- A,
  seven phases as above (adopted):
  example `phase=slot` after the empty slot is verified,
  then `registered`.
  Pros:
  each write names the object it just proved,
  and the slot's identity is written after the slot exists.
  Cons:
  the words are this design's,
  and a later step that adds a write adds a word.
- B,
  the five resume states as the phases:
  example `phase=claimed-empty-slot`,
  `phase=registration-without-dataset`.
  Pros:
  the record's vocabulary is the proposal's.
  Cons:
  the states are read from live facts,
  so the record would claim a state it cannot prove alone.
- Ranking:
  A,
  then B.
  A beats B because a phase should name what the writer just confirmed,
  and the states are a reader's answer.

V15.
No forwarded command reads the record folder.

- A,
  three readers only (adopted):
  the add,
  the removal and `doctor`.
  Pros:
  no command pays for the folder,
  and no command can be broken by a record it does not understand.
  Cons:
  a person who runs an ordinary command learns nothing about an interrupted add until the next add or `doctor`.
- B,
  every forwarded command reads it,
  as the copy journal's pass reads the copy records (`SPEC.md`:4873 to :4885).
  Pros:
  an interrupted add is reported by the next command of any kind.
  Cons:
  every command lists the folder (F19) and every command must know this grammar.
- Ranking:
  A,
  then B.
  A wins because the folder's facts matter only to an operation on that slot,
  and the copy journal's pass exists to settle files a command's own run must not leave behind.

V16.
What a failed add that made a registration but no clone leaves.

- A,
  keep the claim,
  the slot,
  the locked registration,
  report and resume (adopted,
  and what the lock design's "A failed add" does too).
  Example:
  the snapshot returns "no space";
  `git worktree add .claude/worktrees/one` prints the operation,
  object and pool,
  and a later add at `one` resumes;
  `doctor` shows "registration without a dataset".
  Pros:
  matches the accepted words ([proposal](cli-git-zfs-worktree-design-proposal.md):427 to :428),
  destroys nothing,
  and keeps the registration's recorded identity.
  Cons:
  a locked registration Git cannot remove without two forces stays until a person resumes or discards it.
- B,
  roll back:
  release,
  empty and drop the claim.
  Example:
  the same failure ends with no registration and an empty slot.
  Pros:
  leaves no half state.
  Cons:
  contradicts "keep the claim",
  loses the evidence that explains the failure,
  and a kill in the rollback is a new state.
- C,
  keep on `ENOSPC` as the proposal names it,
  roll back on other failures.
  Example:
  `ENOSPC` keeps;
  a refused mount rolls back.
  Pros:
  follows the only failure the text names.
  Cons:
  needs a failure taxonomy no record gives,
  and M8 is the arm that would size it.
- Ranking:
  A,
  then C,
  then B.
  A is the accepted behaviour and needs no new rule;
  C is A for the named case plus an unwritten rule for the rest;
  B overrides an accepted sentence with no record behind it.

V17.
The route's folder,
key and durability convention,
as one choice for the three records.

- A,
  one parent and one shape:
  `<common>/monochromatic-git-policy-cli/<kind>/`,
  a version the grammar compares,
  `key=value` lines,
  names verbatim and paths in hex (adopted here,
  and what both siblings now do except for the origin record's JSON object).
  Example:
  `.git/monochromatic-git-policy-cli/worktree-clones/v1/add-cache-layer.binding` beside
  `clone-destruction-plans/` and `origin-cleanups/`.
  Pros:
  one parent the rename machinery already moves (F20),
  and a person who knows one record knows the others.
  Cons:
  a listing of the parent is not a listing of one family,
  and the keys still differ.
- B,
  a top-level folder per family,
  as the earlier revision had it.
  Example:
  `.git/monochromatic-git-policy-cli-worktree-clone/v1/add-cache-layer.binding`.
  Pros:
  one listing is exactly one family,
  as the copy journal's folder is.
  Cons:
  a second parent convention,
  and a new top-level name the rename machinery most likely needs an entry for (F20,
  inferred).
- C,
  leave each design as written and reconcile later.
  Example:
  two parents and three shapes.
  Pros:
  no rework now.
  Cons:
  three readers' conventions to document,
  and `doctor` lists each differently.
- Ranking:
  A,
  then B,
  then C.
  A beats B because two of three designs and the rename machinery already point at the parent.
  B beats C because a deliberate difference with a reason is better than an accidental one.
  The key cannot be uniform:
  this record is found from a slot,
  the plan from a clone's GUID and the cleanup from a snapshot's nonce.
  Durability is V19.

V18.
May this record live outside the Git common directory.

- A,
  the common directory (adopted):
  the plan's suggestion,
  both siblings' choice and this design's.
  Example:
  as V17-A.
  Pros:
  per repository,
  found without a machine-wide scan,
  beside the claim the proposal already places there.
  Cons:
  the checkout's hourly snapshots hold copies (L5),
  every clone carries a copy (L6),
  and a rebuilt `.git` orphans every record (L9).
- B,
  an account state folder outside the dataset,
  keyed by repository identity.
  Example:
  `~/.local/state/monochromatic-git-policy-cli/<repository-key>/add-cache-layer.binding`.
  Pros:
  no snapshot or clone copies,
  and survives a rebuilt `.git`.
  Cons:
  not found when the home is restored separately,
  a key for the repository is needed,
  and a home path is a new dependency (rule `HOM` applies).
- Ranking:
  A,
  then B.
  A is where the claim lives and where both siblings went;
  B solves limits the documents accept and adds two failure modes.

V19.
One flush rule for the route's records,
or one per record.

- A,
  flush every write of every route record.
  Example:
  the destruction plan's outcome line is flushed beside the destroy it reports.
  Pros:
  one rule to test and to explain,
  and no reader has to know which records a power cut can take.
  Cons:
  every outcome write waits for a log commit on the destroy's own path.
- B,
  flush nothing.
  Example:
  the same outcome line,
  and this binding's identity writes,
  published unsynced.
  Pros:
  no record pays a log commit.
  Cons:
  the clone's initial inventory and the identities cannot be rediscovered,
  so a power cut leaves a clone nothing can prove.
- C,
  flush by position (adopted):
  a record written after the object it names exists,
  where its loss would leave an object nothing can prove,
  is flushed (this binding's writes);
  a record written before an irreversible ZFS step,
  or to report an outcome,
  needs none,
  because the destroy waits for its transaction group and the record is on disk when it returns.
  Example:
  the binding's `registered` write is flushed;
  the plan's outcome line is not,
  and the origin cleanup record's `owed` write is not.
  Pros:
  every record pays only what its facts need,
  and all three designs already do this in effect.
  Cons:
  three durability stories,
  and `doctor` cannot say what a power cut took without knowing which record it was.
- Ranking:
  C,
  then A,
  then B.
  C beats A because the plan's and the origin record's writes lose only a repeated attempt,
  so a flush there buys no evidence.
  A beats B because a file `fsync` and its directory's make the name durable at `sync=standard` (F12),
  so flushing protects this record's orphan case,
  and B leaves it open.
  M1 and the plan's and the origin record's own measurements are the numbers that could change the ranking.

V20.
Where this binding's removal and the claim's release fall in the removal's one order.

- A,
  the binding before the plan,
  the claim last (adopted):
  exactly the three rules "The record's end" states,
  for the origin cleanup design's merged list to place.
  Example:
  a removal killed after the settle leaves the plan and the binding standing,
  which reads as a removal in flight,
  and the next removal finishes step 5 and then removes the plan.
  Pros:
  the plan is the last witness,
  so no kill leaves this binding standing where "What a record whose clone is gone means" would read it as an
  outside change;
  and the claim covers the whole tail.
  Cons:
  the plan's finish pass cannot remove a plan whose binding still stands,
  so it must leave it and report it.
- B,
  the plan before the binding.
  Example:
  the same kill leaves the binding alone,
  and the next add reads it as an outside destruction and may block the slot (Q8's default).
  Pros:
  the plan's finish pass needs no new rule.
  Cons:
  it creates exactly that misreading on the normal path.
- C,
  each record's own document decides,
  as the three stand today.
  Example:
  three placements for one tail.
  Pros:
  no coordination.
  Cons:
  no reader can tell a killed removal from an outside change.
- Ranking:
  A,
  then B,
  then C.
  A beats B because it removes the misreading X3 found without weakening the plan's finish pass.
  B beats C because one wrong placement is still one placement a reader can learn.

## Questions only the project can answer

Two questions are this design's,
and the second gate review's set puts them fourth and fifth.
Every other choice this design once asked is a writer's choice with a ranking,
and this revision moved it to the veto
list:

- where the proof that a lock is the wrapper's lives,
  once Q1,
  is V11;
- what a failed add leaves,
  once Q2,
  is V16;
- the route's folder,
  key and durability convention,
  once Q3,
  is V17;
- a home outside the Git common directory,
  once Q4,
  is V18;
- one flush rule for the route's records,
  once Q9,
  is V19.

The lock design's own questions (whether a lost managed lock is put back unasked,
and what a removal does when the
lock is already gone) are not repeated here.

Q7.
What a reader does with a record it cannot read,
in all three of the route's records.

Asked in plain words:
a record a later release cannot read stands in the way of a slot,
a clone or an origin snapshot;
does the reader stop,
or push the old file aside and go on?

- A.
  Refuse,
  name the file,
  and print the two ways forward:
  settle what the record describes,
  or run `doctor` (adopted here,
  and what all three designs do today).
  Example:
  `add-cache-layer.binding` holds a line this release cannot read;
  `git worktree add .claude/worktrees/add-cache-layer` stops before Git runs and prints the path,
  and a removal that meets the same file refuses before its first delete.
  Pros:
  no record is destroyed,
  so a record that may name a standing clone or a pending origin snapshot is never discarded.
  Cons:
  the slot,
  the clone or the cleanup waits for a person,
  and a record from a later release makes an operation wait where a skipping reader would not.
- B.
  Move the file aside and continue,
  as the copy journal's pass does (`SPEC.md`:4873 to :4885),
  under a fifth name form the answer would introduce.
  Example:
  the same file is renamed to `<slot>.unreadable` and the add runs.
  Pros:
  the route stays usable.
  Cons:
  if the record named a standing clone,
  its dataset and origin snapshot become unprovable,
  and the origin stays pinned with no record naming it.
- Ranking:
  A,
  then B,
  because each record's one job is proving an object,
  and B spends the only evidence that could do it for the sake of convenience.
  This is the second review's merged question five,
  and a real values question,
  not a measurement.

Q8.
A binding whose clone is gone and whose absence cannot be proven.

Asked in plain words:
the record names a clone that no longer answers,
but the wrapper cannot prove it is truly gone;
does the slot stay blocked,
or may a person force a settle?

- A.
  Keep the slot blocked and report (adopted).
  Example:
  `cloneGuid=4821193048571903102` matches no child of the clone parent and the clone's last snapshot cannot be listed;
  the slot refuses a new add and `doctor` prints the recorded identities and the two checks that failed.
  Pros:
  no destroy is authorized from a guess.
  Cons:
  the slot waits for an administrator.
- B.
  Print an explicit settling command that removes the binding after a person confirms.
  Example:
  the same,
  plus `git-policy-cli settle-worktree-clone-record add-cache-layer`.
  Pros:
  the slot can be recovered without an administrator.
  Cons:
  a command that removes evidence is one typo from a clone nobody then cleans up.
- Ranking:
  A,
  then B,
  because a blocked slot costs one slot while a wrong settle is unrecoverable.
  Ask it together with the destruction plan sibling,
  whose "unresolved" outcome decides when absence is unprovable.

The earlier revision's question about the released claim file,
is withdrawn from this list and is settled by the primitive itself:
the newest generation is never removed (F18),
so a released claim stays,
and nothing else in the design turns on it.

## What this design does not cover

- The destruction plan's home and format,
  the origin cleanup record's,
  and the add-time Git lock's:
  each is its own sibling design,
  named under "Where the siblings disagree".
- The content proof of the clone's fill (the frozen index and tree comparison) and where it lives across a kill.
  A resumed add whose proof is gone refuses rather than comparing against nothing,
  because a comparison that has lost its reference is not a proof;
  the proof's own home is the dirty-main restore design's,
  and this record carries none.
- The mount's own records:
  the intent property,
  the receipts and the readiness record are steps 7 and 8's.
- The claim's effect on the checkout's own conversion or migration,
  which uses the same primitive for a different item.
- Any root-side copy of the lineage identities,
  which the delegated route does not need (A2).

## Citations

The first revision's citations were checked by that round's two scratch scripts;
this round's two second passes listed the citations still wrong (the first pass's N10 to N15 and its
"Citations still wrong",
the second pass's "Citations still wrong"),
and this round corrected them and rechecked every citation it changed.
The errors this round fixed:

- the inode sentence that said a restored dataset answers another inode (now "The identity fields",
  after the first pass's N10);
- the two stale sentences about the lock design's record and the copy journal's folder (now "What Git does to a file
  it does not know here",
  after its N11);
- the claim that the origin cleanup record publishes with a flushed write (now the convention's flush bullet,
  after its N12);
- the `.unreadable` name form in the grammar and in Q7's option B (now four forms,
  after its N11);
- T18 and T23 as the claim rule's two readings (now T18 alone,
  after its N1);
- "the two orders agree step for step" (now the mapping in "Where the siblings disagree",
  after its N6);
- every statement about the destruction plan and the origin cleanup record that their new tips no longer say
  (after X4):
  the pointer inside a registration,
  the hex names,
  the flush,
  the staging rule,
  the owner rule,
  the bounds,
  and their Choice numbers;
- the earlier revision's own summary,
  which named the origin cleanup record's flush (now V19).

The earlier round's checking,
which this one reuses where nothing changed,
ran two scratch scripts:
one printed each of the document's one hundred and thirty-two cited ranges beside the lines it names,
the other searched each of its ten blockquotes in the sources it should come from.
Every blockquote matched its source,
and the ranges below were wrong and are corrected.

- The review's own items:
  the flush sentence's range (`SPEC.md`:4421 to :4423,
  where 4416 to :4420 is the measured parenthetical),
  `dataset_guid.rs`'s length (128 lines,
  so `:94 to :128`),
  F5's citation of `git worktree list`'s own reads (now `worktree.c`:140 to :151 and :305 to :326),
                the two host-statement slips (L8 and F6 now name the host's ZFS home dataset and
        the gate image's container filesystem),
  the "no container" sentence (the one container was the gate image),
  the size bound (recomputed from named terms),
  the "wrapper's own grammar" claim (now "shape",
  with V12 marking the departure),
  the claim-generation quote (H2),
  the `live_owner_of` listing (F19),
  the plan's step list (L7),
  and the stage claim's citation (now `SPEC.md`:4401 to :4405).
- Ranges this round's own checking found:
  `PRIVATE_FILE_MODE`'s line (`private_storage.rs`:82 to :88),
  the slot store's filed-under check (`slot_identity_store_records.rs`:170 to :179),
  the `os_sync` guards' own lines (`zfs_vnops_os.c`:865,
  :1227,
  :1422,
  :1553,
  :2667,
  :3272 and :3472),
  the extent of the five directory-log functions (`zfs_log.c`:299 to :609),
  the older-generation loop (`single_owner_claim.rs`:392 to :396),
  the rename machinery's sentence (now about the two names its list holds),
  and the `ENOSPC` sentence's end (proposal :427 to :428).

What the scripts printed is this round's `keep/citation-check.txt` and `keep/quote-check.txt`.

## Sources read and runs made

Read whole or in the parts cited:
`doc/decision/cli-git-zfs-worktree-design.md`;
`doc/decision/cli-git-zfs-slot-identity-store-format.md`;
`doc/decision/cli-git-zfs-staging-under-cache.md` (the threat model and the record's place);
`doc/decision/cli-git-zfs-step-8-sweep-choices.md` (no configuration,
and the per-slot claim);
`doc/planning/cli-git-zfs-worktree-design-proposal.md` ("Add and Git registration",
"Removal and recovery",
"What doctor checks and prints",
"Clone removal identity protection",
"Open items the decisions leave");
`doc/planning/cli-git-zfs-worktree-implementation-plan.md` (steps 10,
12,
14,
16 and 18,
and "Gaps the design's open-item list does not name");
`doc/planning/cli-git-zfs-staging-under-cache-design.md` (its opening and its veto list's shape);
`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md` (its bound and its publication);
`package/git-policy/cli/SPEC.md` at lines 38 to 60,
1152 to 1200,
3879 to 3881,
3915 to 3930,
4398 to 4425,
4873 to 4885,
5525 to 5601,
6228 to 6272,
7340 to 7355 and 16471 to 16480;
and the wrapper's code under `package/git-policy/cli/src/native/`:
`lock/`,
`foundation/`,
`worktree/`,
`worktree_copying/`,
`worktree_clone_mount/` and `worktree_clone_mount_protocol/`.

The three siblings,
read with `git show` from their branches this round and not adopted as decided:
"The add-time Git lock of a ZFS clone worktree"
at `a7902e293b` (branch `docs/planning-cli-git-zfs-add-time-git-lock-design`,
read for its "The order of the add's writes",
"The moments of an add" D0 to D7,
"What Git's failure path leaves",
"How doctor and removal tell the states apart" S1 to S8 and its scan,
"How the wrapper's own removal releases the lock",
its F8,
F11 and F14,
its M2 and its sibling section),
"Where a removal's persisted destruction plan lives,
and its format" at `76849b8d9b` (branch `docs/planning-cli-git-zfs-removal-destruction-plan-record-design`,
read for what finds a plan,
the finish pass,
its choices 5,
10,
11 and 12 and its sibling section)
and "The record that carries origin cleanup after a registration is settled"
at `987a3ba886` (branch `docs/planning-cli-git-zfs-origin-cleanup-record-design`,
read for "The order,
and what a kill at each point leaves",
its home,
its V2,
V4 and V8,
its question one and its F21).
At `987a3ba886` the origin cleanup design does not yet hold the one merged list of the removal's writes across the
three records:
its list ends at step 11,
the plan's removal,
and this design's V20 states what it needs from the merged list instead.

Read at a tag,
in clones under this round's scratch folder:
Git `v2.56.1` at `03b4152` (`builtin/worktree.c`,
`builtin/gc.c`,
`worktree.c`) and OpenZFS `zfs-2.4.4` at `71a9f95` (`module/zcommon/zfs_namecheck.c`,
`module/zfs/dsl_bookmark.c`,
`module/zfs/zfs_vnops.c`,
`module/os/linux/zfs/zfs_vnops_os.c`,
`module/zfs/zfs_log.c`,
`module/zfs/zil.c`,
`module/zfs/txg.c`,
`include/sys/fs/zfs.h`,
`man/man7/zfsprops.7`).

Ran,
twice over,
in the shape F6 records.
Run A:
the nine-arm probe with plain `git` `2.56.1` on this host,
where that name is the wrapper `monochromatic-git-policy-cli 0.6.1-alpha.44`,
each repository on the host's ZFS home dataset and each getting its own `HOME`,
`XDG_RUNTIME_DIR`,
`XDG_CACHE_HOME` and `XDG_CONFIG_HOME` under this round's scratch folder;
one commit per fixture made with `git write-tree`,
`git commit-tree` and `git update-ref`,
because the wrapper lands a commit through a resident service a scratch home does not run.
Run B:
the same arms with real Git `2.56.1` inside the gate image `localhost/git-policy-native-test:development` that
was already present,
in one `podman run --rm --interactive --network none --cpus 2 --memory 512m --pids-limit 128 --blkio-weight 10` container with no folder of the home mounted and every repository made inside it;
no image was built,
pulled or removed.
Also `zfs list`,
`zfs get` and `stat`/`df` read-only on this host,
`getconf PATH_MAX` and `getconf NAME_MAX`,
and a scratch script that printed the size bound's terms and the citation ranges.

Started no guest and no service manager;
the one container was the gate image.
Not run:
any ZFS command that changes anything,
any mount,
any guest,
any `pnpm install`,
and any test suite.

## Revisions

- 2026-11-22T17:30:26-05:00:
  the first draft,
  written on `docs/planning-cli-git-zfs-add-binding-record-design` from `main` at `5dd8883a60`.
- 2026-11-22T17:33:47-05:00:
  folded in the coordinator's note on the probes:
  both runs are named with their versions,
  the wrapper's added steps for `worktree add`,
  `git worktree remove` and `git worktree prune` are named beside the results,
  a run of real Git `2.56.1` in the gate image was added for every arm,
  the wrapper build's date is recorded,
  and the lines were normalized to the repository's Markdown style.
- 2026-11-22T17:33:57-05:00:
  named each revision's real commit time.
- 2026-11-22T17:35:10-05:00:
  named the probe container's real command.
- 2026-11-22T18:28:11-05:00:
  answered the first gate review:
  H1 (the claim carries nothing new and "The add's writes in one order" is the joint list),
  H2 (the `claim=` reader rule and T18),
  H3 (inode and birth with the dataset GUID,
  L11 and T19),
  M6 (the `slot` phase and the finder of "A resume after a kill between Git's add and the registered write"),
  M7 (F12 rewritten from the OpenZFS source,
  L4 withdrawn,
  T28 and M6's power cut),
  M8 (the host's ZFS home dataset and the gate image's filesystem,
  and the one container),
  M9 (the plan's suggestion named,
  V18,
  V1 reordered,
  the F2 argument),
  M11 and the route's convention,
  M12 (the listing cost and M2),
  M13 (root's claim kept apart),
  L1,
  L2,
  L7,
  L8 and L9,
  and every citation the review listed,
  plus the ranges this round's own checking found.
- 2026-11-22T19:06:07-05:00:
  answered the two second gate passes:
  N3 (the resume's table for every state Git's add can leave,
  the lock design's scan and the settle's order,
  A7 retired,
  T20 to T20b),
  N2 (the slot Git's failure path deletes,
  T21's arm),
  N5 (which identity field names which object and what a mounted slot compares,
  T17's arm),
  N1 (the claim rule once,
  T18),
  N6 (a moment at every step,
  the mapping instead of "agree step for step"),
  N8 (`request=` and T21a),
  N10 to N12 (the inode sentence,
  the stale sentences,
  the fifth name form,
  the origin record's flush),
  X3 (the record's end cites the origin cleanup design's list,
  V20,
  the two readings of a record whose clone is gone),
  X4 (every statement about the two removal-side records at their new tips),
  and the questions (Q1 to Q4 and Q9 retired into V11 and V16 to V19,
  Q7 and Q8 kept).
  M9 and L12 are added,
  and the sources name each sibling's tip read.
- 2026-11-22T19:06:17-05:00:
  recorded the previous commit's revision time in this list.
- 2026-11-22T19:06:42-05:00:
  named the resumed add's refusal when its frozen proof is gone,
  which the second pass's N8 observed had been dropped.
- The commits after this list's last entry record only a corrected path reference or a time,
  and are not listed,
  because a list cannot name a commit's time before that commit exists.
