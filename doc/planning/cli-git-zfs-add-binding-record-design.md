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

The second revision answered the first two passes as follows,
at `c144499eb9`;
the next list is this round's.

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
  the resume states and "What no reader may conclude" repeat the rule,
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
  the origin design's V13,
  at `8cc31f59a4` where it first stands,
  places this binding's removal before the plan's with the claim released last,
  and "What a record whose clone is gone means" no longer reads an ordinary kill as an outside change.
- X4:
  every statement about the two removal-side records is rewritten at `76849b8d9b` and `987a3ba886`,
  and the veto options that no longer differ are gone.
- The questions:
  Q1 to Q4 and Q9 are retired into V11 and V16 to V19,
  and Q7 and Q8 are the two that remain.

This round answers two third checks,
the add side's and the removal side's,
and the project's answers recorded while it ran.

- X1:
  the recipe empties `refs/` and `logs/` file by file,
  `logs/HEAD` first among them,
  before either directory,
  and row 6 and T20's fixture hold `logs/HEAD`,
  which the third check measured in a finished add.
- X2:
  the refusal is scoped to entries at this slot's candidate names,
  "owns" is defined as the names a binding's own slot name derives,
  and the `topic`/`topic1` walk is written out.
- X3:
  A10b keeps the copied `.git` and A11 exchanges it with the pointer in one exchanging rename,
  under the accepted decision of 2026-11-22;
  D6 is A10 alone and the exchange ends it,
  and the lock design is told what to name.
- X5,
  X7 to X10,
  X13 to X16:
  the trailing dot is gone from the sanitiser's summary,
  the fixtures carry Git's measured bytes (`<slot>/.git` plus a newline,
  `../..` plus a newline,
  `logs/HEAD`),
  rows 9 and 10 and the re-run's name are planted,
  the empty-file windows and the conditional copies are named,
  the citations are corrected,
  the finder's owner and the removal's one mechanism are stated,
  the recipe's parents and `ENOENT` are stated,
  and the two-owner case is a refusal with an arm.
- Y1:
  A20 to A27 are the add's own fallback and "The add's own fallback" states the states it passes through,
  the order,
  the close,
  the claim,
  the refused snapshot destroy and the kill points,
  with T29 to T33.
- Y2:
  the origin cleanup list is cited at `ea271e9e1d` under its own headings and fourteen numbers.
- Y5:
  "Who may finish that step" is one sentence:
  the pass,
  or the next removal while a registration stands,
  never an add except at `phase=settling`.
- Y6:
  the key rule is one paragraph the plan design can cite,
  with `<slot>` derived from the last component of `targetHex`'s path.
- Y12:
  the condition's second arm has no finisher and is named as a removal's own confirmation.
- The project's answers:
  a record that cannot be read is refused by name and waited on ("A record that cannot be read"),
  and a slot whose clone's absence cannot be proved is settled by "The settling command",
  whose name and confirmation are V21.

This round answers the two diff checks,
the add-side check of `c144499eb9` and the removal-side check of the three texts.
Each line names the finding and the section that answers it.

- R1:
  "The states Git's source can leave" replaces the three-step recipe with one row per state,
  each walked from `builtin/worktree.c`:495 to :604 and `worktree.c`:1105 to :1134,
  with the recipe that removes the files before the lock and the directory after it,
  the slot's pointer among the names it removes,
  the failure path's leftover as R7,
  and T20 to T20c as one fixture per row.
- R2:
  the same section's finder takes Git's own sanitiser and counter for a candidate name,
  a prefix only when it covers the fixed part and the nonce,
  the newline Git completes the lock with,
  the relative `gitdir` resolved against the registration,
  and a second read under the descriptor before the first removal;
  T20c is the name case.
- R3:
  "A resumed add" refuses a `filled` binding whose clone is gone,
  "The record's end" and "Which step" name the plan design's pass or the next removal as the finisher,
  and step 10 settles nothing.
- R4:
  A10a and A10b are appended for the frozen-content comparison and the discard,
  D6 has one boundary owner,
  the lock design is cited at `7d0dcdc875`,
  the `HEAD` fact comes from its scan and not its F14,
  the assumption labels say "assumption",
  and the count is four name forms.
- Z1:
  "The condition a finisher meets" is the plan's arm or the origin record's,
  the origin record's absence is not a failure when a plan stands,
  and the pass that takes the claim is the finisher.
- Z3:
  step 5 removes the two inventories first and the binding last,
  V20 weighs only that,
  and the resume refuses an orphan inventory with no binding.
- Z4:
  every statement about the two removal-side records is rewritten at `b4dc758a76` and `0973f2fc2a`,
  the tail cites the merged list's steps 10 to 13,
  and the three records' one flush rule and one staging convention are named as such.

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

The fallback is the add's own finish,
in the project's answer of 2026-11-22,
whose words this design's A20 to A27 carry
(`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`):
"at every step,
the add destroys only what this add made and recorded (the clone,
then its snapshot),
runs Git's own checkout at the same slot,
and says which route made the worktree and why the first was abandoned".
The same record decides what the slot holds while it is filled,
"A2:
keep copy,
swap at end":
"the copied `.git` stays until the last step,
and the pointer and the copy then change places in one exchanging rename,
so that no moment and no kill leaves the folder without a `.git`",
which is A10b and A11's rule.

A lost managed lock is written again by any wrapped command that holds the slot and has proved the registration,
and one line says so,
in the project's answer of 2026-11-22
(`doc/decision/cli-git-zfs-add-time-git-lock-put-back-when-lost.md`).
A removal is such a command:
it writes the lock again,
says so,
and then releases it.
This design's A6 refuses an unlocked,
complete registration and names it until step 13 builds the installer,
then adopts it and lets that installer write the lock again (V22),
and "The record's end" step 4 writes a lost lock before it releases it,
which is the removal's own act.

Two decisions of 2026-11-22 settle this record's own questions:
a record a later release cannot read is refused by name and waited on by a person,
which is "A record that cannot be read";
and a slot whose recorded clone is gone and cannot be proved gone is settled by an explicit command a person
confirms,
which is "The settling command"
(`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
"A record that cannot be read" and "A slot whose recorded clone is gone and cannot be proved gone").

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
and this design's folder holds only its own four name forms.

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
The resume states are told apart by reading the live system,
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
the project decided on 2026-11-22 that a reader refuses such a file and waits for a person,
so no reader moves a record aside,
and a form the design does not use would be a name `doctor` must explain without a writer.
The fifth form would come back only if the project reversed that answer,
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

The rule a sibling can cite,
stated once:
the key is `<slot>`,
the destination's last path component,
refused by `slot_name_refusal` when it is empty,
`.`,
`..`,
holds a `/`,
starts with a dot,
or holds whitespace or a control character;
`slot=` in the record is the lowercase hexadecimal of the whole slot path,
which is what a plan design's `targetHex` holds;
and a record is filed as `<slot>.binding`.
A clone's dataset name is not the key and holds `/`,
so a design that keeps the clone's dataset name in a field
named for the slot must derive `<slot>` from the last component of the path its `targetHex` holds and compare
`slot=` with that whole path.
The destruction plan design's field `slotName` holds the clone's dataset name and is not this key:
at `3cd520b6a0` that design renames it `cloneDataset` and derives `<slot>` from `targetHex`,
which is the rule below,
so both documents state one rule (Y6).

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
Neither sibling is unreviewed:
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
  The destruction plan writes the same three encodings (read at `1921adf393`),
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
  The three records now share one rule by position (V19):
  a write made before an irreversible ZFS step,
  or one that reports an outcome,
  needs no flush,
  and a write made after an object exists that nothing else can prove is flushed.
  This record's writes all fall on the flushed side (V6);
  the destruction plan's outcome writes and the origin cleanup record's `owed` write fall on the other
  (the plan's Choice 5 and the origin design's V14,
  read at `cf51d140af` and `ea271e9e1d`).
- Staging names and who removes them:
  every staging name is a dot name made by one known writer,
  and `doctor` at plan step 18 removes the known forms.
  Where a writer holds a claim across a long step,
  `doctor` removes a staging name only when the folder holds no live claim (V8).
  The destruction plan removes one only when a plan with that GUID stands and its `owner` is gone,
  which its own entry for staging names states (cited by its subject,
  not its number,
  because its entries were
  renumbered),
  and the origin cleanup record removes none and reports them (its V8,
  read at `ea271e9e1d`),
  which is one convention with each record's own evidence;
  the store's own decided rule covers the store alone
  ([`../decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md),
  "Who removes a killed writer's staging file").
  Precedent:
  the store's decision,
  which covers the store alone.
- A record a reader cannot read:
  refuse and change nothing,
  name the file,
  and say what a person does (restore it from a copy,
  or read it and delete it,
  and run the command again).
  All three designs refuse a partial,
  foreign,
  oversized or wrongly owned record,
  and an unreadable one,
  which the project decided on 2026-11-22 is refused by name and waited on by a person.
  The copy journal's pass moves such a file aside (`SPEC.md`:4873 to :4885);
  this route does not.
- Bounds:
  each record's widest line and fixed part is derived from kernel constants named with it,
  and no cap is shipped on a count nothing bounds.
  Precedent:
    the accepted slot store's bound computed from named terms and
  its dropped whole-store cap (`slot_identity_store_reading.rs`:63 to :85,
  [`../decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md),
  "What was decided").
  Neither sibling rounds either:
  the destruction plan's bound is `10,128 + 340 × count` and the origin cleanup record's is 13,368,
  both computed from named terms (read at `cf51d140af` and `ea271e9e1d`).

Where the two siblings differ from the convention today,
read from their branches:

- The destruction plan keeps no pointer inside a registration:
  the earlier revision's pointer is dropped,
  and the plan reads its folder by one `stat` before every forwarded command and lists it only when it exists
  (its "What finds a plan",
  read at `1921adf393`).
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
phase=<claimed|slot|registered|snapshotted|cloned|mounted|filled|settling>
settleReason=<proof|enospc|mount|clone|snapshot|destroy|checkout>
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
  An argument longer than `PATH_MAX` refuses the add at A2 rather than being truncated,
  which is what the bound's 4096-byte term assumes.
  Hexadecimal is what makes the value safe:
  a refname may hold `=`,
  and a commit-ish may hold `~`,
  `^` and `..`,
  none of which a `key=value` line can carry verbatim without a second grammar.
- The pointer is the `.git` file's bytes in lowercase hexadecimal,
  which is how the btrfs binding records it (`worktree_add_binding.rs`:122 to :134).
- The phase is one of the eight words above and nothing else.
- `settleReason` is one of the seven kind words above and nothing else:
  which step's object stopped answering,
  and it is written only beside `phase=settling`.
  The restoring design's reason line holds the words a person reads;
  this field holds only the kind,
  so that a resumed add knows which of A20 to A27 to skip.

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
  - the settle reason at `13 + 8 + 1` bytes:
    `settleReason=`,
    the widest kind word `checkout` and the newline,
    written only beside `phase=settling`.
  The two removal-side designs quoted 34,797,
  which was this record's sum before `request=` (8,208 bytes) and `settleReason=` (22 bytes) were added;
  43,027 is the current sum,
  and the origin cleanup record now quotes it too (13,368 is its own bound,
    and it names this record's 43,027).
  The terms sum to 43,027 bytes,
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
read at `7d0dcdc875` ("The moments of an add",
D0 to D7).

The labels do not move again:
a later revision that adds a write appends a label,
and A7 keeps its place although this revision retires it.
These A-labels are the order's;
the Assumptions section's A1 to A10 are another list,
and a citation that means an assumption says "assumption" beside its label.

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
read at `7d0dcdc875`);
this design for the order,
for the nonce being recorded before this step runs,
and for the request being recorded so a resume can compare it.
The reason the plan's step 10 says the registration records is this sentence:
the binding's `nonce=` and the sentence recomputed from it stand for the words,
and no second copy of them is written anywhere.
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
decide each entry by that section's rows,
and either adopt the complete registration or clear the incomplete one with that section's recipe before A5 is run
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
the fields,
the finder and what the resume does with what the finder finds;
the lock design for the sentence and for what the lock's bytes prove.
Lock moment:
D3,
and D4 is this step's write.
A kill between A5 and this write leaves one of the table's states;
the next command's scan meets it.

A7.
Retired.
The earlier revision put the lock design's own record here;
the lock design keeps none,
and its "The record whose nonce the lock's proof rests on" says so (read at `7d0dcdc875`),
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
because its set is fixed by that snapshot's transaction group (assumption A6).

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
This step ends at that write.
Owner:
this design with the route's attach checks.
Lock moment:
D6 is this step's window and the two below,
and A11's exchange ends it (the lock design reads D6 as A10,
its A10a and A10b,
and A11,
at `9e3ceff079`).
A kill leaves a clone that may be mounted;
the resume reads the mount live,
and compares the identities "The identity fields" says are comparable while mounted.

A10a.
The frozen-content comparison inside the clone,
before any Git runs against the copied administration:
the index and tracked content against the proof the add captured just before the snapshot,
as the proposal's add step 4 asks ([proposal](cli-git-zfs-worktree-design-proposal.md):397 to :400)
and as the dirty-main decision's window states it (that record's "Which writes the frozen proof refuses").
Owner:
the dirty-main restore design for the proof;
this design for its place in the order.
Lock moment:
inside D6.
A kill leaves a mounted clone with the comparison unmade;
the resume redoes it or,
with the proof gone,
refuses ("What this design does not cover").

A10b.
The restore,
with the copied administration left in place:
the clone-local leave-outs removed,
the captured index written under a private name and published at the end of the restore,
the dirty restore run against that private index,
the fence's paths written through Git,
and the paths it wrote checked once more.
The copied `.git` is **not** discarded here.
Owner:
the dirty-main restore design for the restore's steps;
this design for one rule it depends on,
which is the copied `.git`'s fate at A11.
Lock moment:
inside D6.
A kill leaves a mounted clone whose slot still shows the copied `.git` and whose index may be the private name or
the real one;
the resume reads which,
as the restoring design's resume states do.

A11.
The exchange,
then `phase=filled`.
The saved pointer bytes are written inside the clone at a trash entry whose identity the trash's own manifest
records,
and that entry changes places with `<clone>/.git` in one `renameat2(RENAME_EXCHANGE)`:
the pointer takes `.git`'s name and the copied administration lands at the trash entry's name in the same instant,
so the slot's `.git` is the copy until that call and the pointer after it,
and no moment and no kill leaves the folder without a `.git`
(the dirty-main decision's "A2:
keep copy,
swap at end",
recorded 2026-11-22T19:11:12-05:00).
Then the worktree root,
the common directory,
the reciprocal registration,
the mount identity and the checked-out state are verified,
and `phase=filled` is written,
which is the proposal's "Mark the binding usable only after the fill is verified"
([proposal](cli-git-zfs-worktree-design-proposal.md):407).
Owner:
the dirty-main restore design for the exchange's mechanics and its trash entry;
this design for the order's labels and for the rule that the phase follows the exchange and the verification.
Lock moment:
D6 ends at the exchange and D7 begins;
that is the one statement of D6's boundary between the two documents.
A kill leaves a mounted clone whose slot shows either the copied directory or the pointer,
never neither,
and the resume reads which and continues.
The exchange's own guest run is still owed (the decision's "Still owed"),
and it is the dirty-main design's measurement,
not this record's.

A12.
The dependency repairs,
queued.
Owner:
the route's existing repair step.
Lock moment:
D7,
which the lock stands through.

A20 to A27 are the add's own fallback.
They are a branch and not a continuation:
from the step whose object stopped answering they run instead of the rest of the route,
and they carry labels appended after A12 so that every earlier label keeps its meaning.
The project decided on 2026-11-22 that a failed add finishes itself with Git's own checkout,
destroying only what it made
(`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
"always finish itself");
the dirty-main restore design owns the settle marker,
the reason's words,
the unmount request and the destroy mechanics,
and this record owns the phase the binding carries through it,
the close and the end.

A20.
The phase `settling` and `settleReason=`,
written before the first destroy.
Owner:
this design for the phase and the field;
the restoring design for the reason's words and for the settle marker it publishes beside its own record.
Lock moment:
the moment the entry phase had.
A kill leaves a binding that says an add is finishing with Git's checkout,
which is the one binding state a resumed add continues instead of refusing.
Writing it before the first destroy is what removes the window the lock design calls its honest cost,
in which a kill after a destroy leaves a binding naming a clone that no longer answers.

A21.
The unmount requested and its receipt verified,
when the entry phase has a mounted clone,
or the request withdrawn on every outcome but success,
which is the accepted design's wider rule and its choices 14 and 15:
a clone the add finds already unmounted continues the settle.
The further answer of 2026-11-22 decides the busy arm:
the add writes its request back to the mounted state,
nothing having been unmounted,
leaves `phase=settling` with the clone mounted,
prints that the worktree is pending and the command that removes it,
with the clause "do not run Git in this folder until it is removed",
and changes nothing else
(`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
"A further answer:
an add whose own unmount is refused as busy",
recorded 2026-11-22T20:21:19-05:00).
Owner:
the restoring design for the withdrawal and the text,
with the staging design's add-side use of the handler.
Lock moment:
D6's successor while the clone stands.
A kill leaves the request standing,
or the withdrawn state and a mounted clone,
and the resume reads the receipt and the mount live.

A22.
The clone destroyed from the identities A9 recorded,
with one delegated `zfs destroy` and no `-R`.
Owner:
the restoring design;
the route's grants carry the destroy.
A kill leaves the clone's GUID present or absent,
and the resume proves which before acting.
Skipped when the entry phase is `registered`,
or `snapshotted` with no clone at the proposed name,
and run for a `snapshotted` clone that the fallback's read recovers at the proposed name,
as the fallback's entry list in "The add's own fallback" states (U27 of the eighth and K12 of the eleventh
reads).

A23.
The origin snapshot destroyed from the identities A8 recorded,
with `ENOENT` counted as done,
or,
for an entry at `registered`,
from one live read at the proposed snapshot name.
From `registered` the snapshot may stand although A8's write did not land,
so A23 reads it at the name A2 recorded,
proves it as A8's own resume proves one (the name and its `createtxg` later than the add's claim),
publishes the origin cleanup record from that read,
and writes `originSnapshot=`,
its GUID and its `createtxg` into the binding;
when no snapshot stands at that name it writes no origin record and skips A23.
The origin cleanup record stands before A22,
or before A23 when A22 is skipped because the add made no clone,
written by the restoring design's call on the origin cleanup design's writer,
and it carries the snapshot if this destroy is refused (W9 of the fourth and N5 of the fifth
removal-side checks).
Owner:
the origin cleanup design for the record and its attempt;
the restoring design for the add's call.
A kill leaves the snapshot present or absent and the record `owed` or gone.

A24.
The managed lock released,
the registration kept:
one `unlinkat` of `locked` through the held registration descriptor,
and one sync of the registration's directory.
Owner:
the lock design's release step.
A kill leaves the lock standing or gone,
and the next command that holds the slot and proves the registration writes it again if it is gone
(the decision of 2026-11-22).

A25.
Git's own checkout at the slot,
against the kept registration and the underlay pointer the mount had covered.
Owner:
the restoring design.
A kill leaves the checkout partial,
and the resume repeats it.

A26.
The binding closed:
`<slot>.origin-inventory` and `<slot>.clone-inventory` first and `<slot>.binding` last,
the same order the removal uses.
Owner:
this design.
A kill leaves the binding,
or the binding and one inventory,
and the resume finishes the close.

A27.
The claim released last,
and nothing printed.
The add's sentence is the restoring design's,
and that design prints it before the records go,
after A25 and before A26,
as its "Dependencies on the sibling designs" asks of this label (S1 of its own text).
Owner:
this design for the claim alone.
A kill leaves the claim naming a dead owner,
and the claim module's own liveness rule lets the next generation go on.

### The add's own fallback: the states it passes through, and its end

Which failures enter the fallback,
once,
by the phase the binding stands at and the failure that stopped the add
(the accepted dirty-main restore design's "one route from every failure after the claim",
in its "What happens when the proof fails or the restore cannot finish",
`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`):

- `registered`,
  `snapshotted` or `cloned`,
  with any failure of its own step:
  no mount exists,
  so A21 is skipped and the fallback runs the steps its objects need.
  A clone may still stand unrecorded from `snapshotted`:
  A9 writes `cloneGuid=` only after `zfs clone`,
  the read-back and the clone inventory,
  so a failure of the read-back or of the inventory write enters here with the clone already made.
  The fallback therefore reads the clone at the proposed name A2 recorded,
  as A23 reads an unrecorded snapshot at its name,
  and a dataset there whose origin is the recorded snapshot and whose `createtxg` is later than this add's claim
  is the add's clone:
  its GUID and `createtxg` are written,
  and A22 runs for it.
  Where that read cannot answer,
  the fallback stops,
  keeps the binding at `settling`,
  and says an unrecorded clone may stand,
  rather than closing a record over a clone nothing names (U27 of the eighth add-side read);
- `mounted`,
  with a failure of the comparison,
  of the restore,
  of the exchange or of the fill:
  A21 to A27;
- `filled`,
  never:
  a filled binding is a verified checkout,
  and no failure after it enters this path.

The entry reads the mount live and not the phase:
a binding at `cloned` whose clone is in fact mounted runs A21,
because the phases may lag the live state,
and a `settling` resume proves the mount,
the clone,
the snapshot and the lock before it acts.

So the list is:

- from `registered`,
  no clone and no mount:
  A20,
  A23,
  A24,
  A25,
  A26,
  A27;
- from `snapshotted`,
  no mount,
  and the same six steps,
  with A22 added for a clone the fallback's own read recovers at the proposed name,
  which the fallback's entry list in "The add's own fallback" states (U35 of the ninth add-side read);
- from `cloned`,
  the clone exists and is not mounted:
  A20,
  A22,
  A23,
  A24,
  A25,
  A26,
  A27;
- from `mounted`,
  and from `cloned` whose clone a live read shows mounted:
  A20 to A27,
  in order.

A resumed add that finds `phase=settling` continues at A20 with what it can prove:
the clone's GUID present or absent,
the snapshot's present or absent,
the lock standing or gone,
the checkout done or partial,
and no plan standing for its clone,
which the add reads the plan folder to check and for nothing else ("Where the siblings disagree",
K9 of the tenth removal-side read).
When it cannot prove the clone's absence it stops,
changes nothing,
and names the command "Which command is true for a state" gives that state:
the settling command for a binding that holds a `cloneGuid=`,
and its own re-run for one that holds none,
which the command refuses (U14 of the sixth add-side read and J4 of the eighth).
This is the one case in which an add finishes a binding,
and it is not R3's case:
R3 refuses a binding at `filled` or later whose clone is gone and which carries no `settling` phase,
because that is a removal's tail and not the add's own path.
A `settling` binding is the add's,
its claim was taken at A1 and never released,
and the resumes prove every fact before acting.

A refused snapshot destroy at A23 does not stop the fallback:
A23 rests on the project's answer of 2026-11-22,
"Before destroying anything,
the add records the snapshot as owed cleanup (the same record a removal uses)"
(`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
"Who records what when the add undoes what it made",
recorded 2026-11-22T21:37:01-05:00),
so the origin cleanup record stands before A22,
or before A23 when A22 is skipped,
and says `owed`,
the add goes on at A24 to A27,
one line says the snapshot stays for now,
and the next `git worktree` command destroys it once the hold is gone.
That is also the origin cleanup design's rule for an add's own destroy ("The add that fails and destroys what it
made",
read at `22be706b09`),
and the accepted restoring design agrees:
a marked or pinned snapshot is not a failed settle (its "What happens when the proof fails or the restore cannot
finish").
A failure of A23 for any other reason than a refused destroy is treated by a person:
the binding stays at `settling` with the record `owed`,
`doctor` prints both,
and the origin look or the person settles it.

Who removes the binding,
the inventories and the claim after a successful fallback:
the add itself,
at A26 and A27.
A fallback that fails anywhere else keeps the binding at `settling` and changes nothing else.
Which command is true for each state after A20 is stated once,
in "Which command is true for a state",
and the places that name a command cite it.

"Cannot be proved" means one thing wherever this design uses the phrase:
the pinned parent's children were read and either the pool refused the listing or it answered it only in part.
A reused name is not this state:
it is a refusal,
below,
because a live dataset holds the recorded clone's name (G9).

#### When a checkout has begun

One test,
stated once,
and every place that needs it cites this heading:
a checkout has begun when nothing is mounted at the slot and the slot holds an entry besides `.git`.

Why that is the whole test.
A3 made the slot an empty directory and A5 wrote Git's pointer into it,
so the underlay the mount covers holds nothing but the pointer.
A10b's restore writes inside the clone,
under the mount,
so its files leave with the mount at A21.
The registration's `index` says nothing about A25:
the accepted dirty-main restore design publishes the captured index into the registration before the exchange
(its "The restore",
step 8),
so an `index` can stand before A25 begins,
and it can be missing while A25 writes the tree,
because `git reset --hard` writes the working-tree files first and commits the index at its end
(`builtin/reset.c`:86,
:117 and :530 at `v2.56.1`).
The lock says nothing either:
A24 released it on purpose,
and a later put-back writes it again.
So the slot's own contents,
read while nothing is mounted there,
are the only observation true exactly when A25 has begun writing a person's files
(H1 of the seventh removal-side read,
U15 of the seventh add-side read and J8 of the eighth).
The command already refuses a mount,
so its own read of the slot is that one.

#### Which command is true for a state

One list,
stated once,
and the printing rule,
the sentence in "The add's own fallback" that names this section,
"A resumed add",
"`doctor`",
R3,
"What a record whose clone is gone means" and the removal's rule cite it and say nothing of their own.
"Cannot be proved" is defined above,
at "Cannot be proved" in "The add's own fallback";
and a reused name is a refusal and not a state (G9).
A plan may stand beside a binding of any phase,
`settling` included,
because a removal accepts a `settling` binding whose clone stands and writes its plan before it deletes anything:
a kill of that removal,
or a refusal at its second deferred-origin check,
leaves the plan at `head=present` beside the standing clone,
and the plan row below is that state's (K5 of the ninth removal-side read).

For a binding at `phase=settling` that holds a `cloneGuid=`,
and for which no plan stands:

- the clone stands,
  mounted or not,
  and no plan stands:
  two commands are true.
  `git worktree remove <slot>` removes it,
  and the add again at the slot continues at A20 and closes at A26 and A27.
  In the busy state only the removal is true,
  because the add meets the refused unmount again,
  and the removal prints the pending worktree's line,
  "this worktree is pending and the clone that would have made it is still mounted;
  do not run Git in this folder until it is removed;
  `git worktree remove <slot>` removes it once nothing holds it".
- the clone is gone,
  its absence is proved (the pinned parent's children carry neither the GUID nor the name),
  and its registration stands:
  the add again at the slot,
  which finishes the checkout and closes the binding and the claim at A26 and A27.
- the clone is gone,
  its absence is proved,
  and its registration is gone (a kill inside the settling command,
  run again once the pool answers):
  the settling command,
  which takes the binding for either listing answer and removes the records.
- the clone is gone,
  its absence cannot be proved,
  and no checkout has begun:
  the settling command,
  whose full recipe removes the abandoned registration and then the records.
- the clone is gone,
  its absence cannot be proved,
  and a checkout has begun:
  the settling command,
  which removes only the records (the plan,
  the two inventories,
  the binding and the claim),
  leaves the registration and the slot,
  and names the worktree with `git worktree remove <slot>`,
  and with `--force` if Git refuses it,
  which Git answers for a worktree holding files it did not index.

For a binding at `phase=settling` that holds no `cloneGuid=`:
the add again at the slot,
which continues at A20 and needs no clone's identity;
the settling command refuses that binding and names the add.

For any binding for which a plan stands,
of any phase,
with the clone standing or gone:

- the destruction plan design's "Which resume path a removal takes" decides,
  and this design cites it and does not restate it.
  That heading is what the removal reads,
  by the plan's head word and the live pinned parent:
  where it takes its head-present path the removal is the command and its own order ends the plan;
  where it takes its head-gone path the removal is the command and its order ends the plan too;
  where it refuses,
  it names the settling command,
  which prints what the plan held and removes it,
  except where the heading names no command at all:
  a reused name and a plan that cannot be read are refusals that name the file or the reuse and no command
  (K10 of the tenth).
  So the command that accepts a state with a plan standing is the removal's or the settling command's,
  as that heading places them,
  and a `settling` binding is no exception:
  the settling command's refusal of a `settling` binding whose absence is proved applies only where no plan
  stands,
  and with one standing it takes the binding,
  prints what the plan held and removes it with the records (K7).
  The plan design's finish pass acts only on a plan whose head is `gone`,
  and its "The finish pass that settles a registration" says so.
- the add refuses while such a plan stands,
  which is this design's addition to that heading:
  an add that closed its binding over a standing plan would leave the plan with no clone,
  no binding and no command,
  and the plan folder is not the add's to act on (K5 of the ninth removal-side read).
  The add names the plan and the command the heading gives.
- the failing arm of that heading,
  a pool that cannot prove the clone's absence,
  names the plan and the failed read,
  and the settling command when the head word is not `gone`,
  in the heading's own words.

For a binding at `filled` whose clone is gone,
whether its absence is proved or cannot be proved,
and for which no plan stands,
which a person's own destroy leaves before any removal ran:
the settling command,
which is the one command that can act,
because a removal resumes from a plan and the finish pass is driven by plans (K1 of the ninth).

For a binding at `cloned` or `mounted` whose clone is gone,
whether its absence is proved or cannot be proved,
and for which no plan stands:
the settling command.
The unprovable absence is the project's own case for a printed command,
and without this clause no row named it there while the add refused it and named the section's command,
so nothing was named (K14 of the eleventh removal-side read).
The add refuses a phase whose object no longer answers,
and `settling` is the one phase it continues ("A resumed add"),
so a `cloned` or `mounted` binding with no clone is the command's (K8 of the tenth removal-side read and U39 of
the tenth add-side read).
Whether a gone clone enters the add's fallback is stated here:
a clone found gone on a resume is not a failure of the add's own step,
so the fallback is entered by a failing step and not by a resume,
and the fallback's entry list admits only "any failure of its own step" (K12 of the eleventh removal-side read).

For a binding at `registered` or `snapshotted`:
a gone clone has no meaning there,
because neither phase records a clone,
and an ordinary resume continues the route in both:
"registered" is the resume-state list's registration without a dataset,
and a `snapshotted` resume that finds a clone at the proposed name records its GUID and `createtxg` and goes on,
which is "What the writer confirms before each write" and T23,
not this list's (K8 of the tenth removal-side read and N2 of the eleventh add-side read).
What the fallback does from `snapshotted` is its own entry list's matter,
and that is not a resume.

The five readers of the plan folder are named here,
because the plan design's "Readers that do not exist" says an add reads none:
the removal,
in the plan design's "Which resume path a removal takes";
the finish pass,
in its "The finish pass that settles a registration";
the settling command,
in its sixth read;
`doctor`,
in the plan design's own `doctor` section;
and the add,
which reads the folder to refuse and for nothing else,
as "Where the siblings disagree" records (K9 of the tenth removal-side read and U41 of the tenth add-side read).
The add writes no plan,
and a plan of a clone that stood at the same path before names another GUID and is not this binding's plan.

A fallback killed before A20 leaves no `settling` phase:
the binding names the clone it made,
the clone stands because nothing was destroyed yet,
and the ordinary resume applies,
re-entering the fill or the fallback by what the live facts answer.

The order of A20's two writes,
stated once:
the phase and `settleReason=` are written first,
then the restoring design's settle marker,
both before A21 and before the first destroy.
The phase is what a resume keys on;
a marker whose binding does not say `settling` is treated as absent,
because the binding's phase is the record this design owns.
The accepted restoring design writes them in this order too:
it appends `phase=settling` and `settleReason=` to the binding first,
then publishes its settle marker (its "What happens when the proof fails or the restore cannot finish").

What a failed add leaves,
at any step from A5 on,
during step 10 and until the fallback is built,
is the accepted sentence's answer and not a rollback:
the claim,
the slot,
the locked registration and every object whose creation and identity are proven stand,
the failure is reported with the operation,
object and pool,
and the next add at that slot resumes ([proposal](cli-git-zfs-worktree-design-proposal.md):427 to :428).
This design writes no rollback there,
the fallback at A20 to A27 is what replaces it once step 11 builds it,
and V16 puts both to the project with the lock design's rollback beside them.

### A resume after a kill between Git's add and the registered write

The claim is held and the binding's phase is `slot` (or `claimed`,
when A5 began but the `slot` write did not land),
so Git's add may have left anything from a bare directory to a complete registration.
Everything below runs under the claim,
through one descriptor opened on the pinned `<common>/worktrees` parent,
and it reads each entry twice:
once to recognise it and once,
immediately before the first removal and again before the lock's own removal,
to prove it still answers the row the first read chose.
A second read that differs refuses and changes nothing.

This record owns the finder.
The lock design owns the sentence that attributes a lock,
its recomputation from the nonce,
and the doctor line that reports a lock-only directory;
its scan named those matches and this record's rows decide what happens to each,
so a reader of the lock design should follow this section's finder rather than a second copy,
and that design's round is told to cite it by name.
The finder is:
one listing of `<common>/worktrees/`,
one read of each entry's `locked` and `gitdir`,
and the name compared with the ones Git could derive from this slot.

- The sentence is recomposed from this add's `nonce=` and compared with `locked`'s bytes as Git wrote them,
  **including the newline `write_file` completes it with** (`wrapper.c`:760 to :783).
  A proper prefix counts only when it covers the sentence's fixed part and the whole nonce;
  a shorter prefix,
  including an empty file,
  is what any just-opened lock looks like and attributes nothing.
- A `gitdir`'s text is trimmed of its ending newline and,
  when it is relative,
  resolved against the registration's directory,
  which is what Git writes under `worktree.useRelativePaths` or `--relative-paths`
  (`worktree.c`:1120 to :1134),
  and what the wrapper's own reader does with a pointer (`recorded_destination` at
  `worktree_add_binding.rs`:415,
  and `pointer_target` at :466).
- A name is a candidate when Git could derive it from this slot:
  the slot's name through Git's own sanitiser (`check_refname_component` and `check_or_sanitize_refname`,
  `refs.c`:191 to :321,
  and `sanitize_refname_component` at :376,
  which collapses `..`,
  replaces a forbidden byte and a leading dot with `-`,
  turns the brace of `@{` into `-` so `a@{b` becomes `a@-b`,
  and strips a trailing `.lock` repeatedly,
  `k.lock.lock` becoming `k`),
  then no suffix or a decimal counter (`builtin/worktree.c`:495 to :512).
  It does not strip a trailing dot:
  `x.` stays `x.` (`refs.c`:308 to :312),
  which the third check measured as well as read.
  A name is excluded when another slot's binding in this record folder *owns* it,
  and a binding owns the names its own slot name derives:
  `topic1.binding` owns `topic1`,
  `topic11` and so on,
  whether or not its `registration=` is written,
  because the binding's first write at A2 precedes Git.
  That is what makes the exclusion work for the instant in which slot `topic1`'s own live add has made its bare
  directory and written nothing else.

Walk slot `topic` against slot `topic1`,
by hand:
slot `topic`'s candidates are `topic`,
`topic1`,
`topic2` and so on;
slot `topic1`'s are `topic1`,
`topic11` and so on,
and never `topic`.
So the overlap runs one way only:
slot `topic`'s finder can meet slot `topic1`'s entry,
and never the reverse.
A complete,
locked `topic1` is met with its lock holding other words and its `gitdir` naming another slot;
it is not at one of slot `topic`'s candidate names,
because `topic1.binding` owns that name,
so the finder leaves it alone.
A bare `topic1` that slot `topic1`'s live add has just made is excluded by `topic1.binding`'s ownership,
so slot `topic` neither clears it nor spends that add.

An entry is this add's when one of these holds:

- `locked` equals the recomposed sentence,
  newline included;
- `locked` is a proper prefix that covers the fixed part and the nonce (the lock design's F8 and S2);
- there is no `locked`,
  and `gitdir` resolves to `<slot>/.git`;
- there is neither `locked` nor `gitdir`,
  the name is a candidate as above,
  and the slot stands empty.

An entry at one of this slot's candidate names whose `gitdir` names another slot is not this add's at all:
it belongs to that slot,
the finder leaves it,
and no resume refuses on it.
An entry whose `gitdir` names this slot and whose `locked` holds other words is this add's registration with a
person's lock on it:
the resume refuses,
names the entry and the first line of its words,
and changes nothing,
because only a person's `git worktree unlock` decides what that lock means.
The scope matters:
an entry at another slot's name is not this slot's business at all and is not a refusal,
so a repository with a second clone slot does not make every resume refuse,
and row 9's "re-run A5" stays reachable.

An entry that is this add's is complete when these five answers name each other:
the slot's `.git` exists and its bytes will be `pointer=`,
`gitdir` names exactly `<slot>/.git`,
`commondir` exists,
and `HEAD` exists.
That is the wrapper's own three-fact proof (`same_file` over the pointer,
the back-link and `commondir`) plus the
`HEAD`,
which the five facts above name and which no lock list supplies.
Complete:
adopt it.
A lock cut short is completed in place first (the lock design's S2),
and an entry with no lock at all is the case step 10 has no installer for.
The decision of 2026-11-22 makes any command that holds the slot and has proved the registration write a lost
lock again
(`doc/decision/cli-git-zfs-add-time-git-lock-put-back-when-lost.md`),
and the installer is the lock design's,
built at step 13.
So until step 13 lands,
step 10 refuses such an entry,
names it,
prints the sentence the lock must hold,
and leaves the entry locked by nobody (W12 of the fourth check);
V22 puts the alternative,
moving the installer into step 10,
to the project.
From step 13 on,
A6 adopts the entry and the lock design's installer writes the lock again with one line.
Then,
in either case once the entry is adopted,
`registration=`,
`registrationIdentity=`,
`pointer=` and `phase=registered` are written and the add continues at A8.

Incomplete:
clear it with the recipe below.

#### The states Git's source can leave

One row per state,
walked from `add_worktree` at `v2.56.1`
(`builtin/worktree.c`:495 to :604,
`worktree.c`:1105 to :1134),
and checked against the shapes a finished add left when the third check measured one in the gate image
(its `probe-registration-shapes-output.txt`):
the registration's `gitdir` holds `<slot>/.git` and one newline,
the slot's `.git` holds `gitdir: <registration>` and one newline,
`commondir` holds `../..` and one newline,
and a finished registration holds `HEAD`,
`commondir`,
`gitdir`,
`locked`,
`logs/`,
`logs/HEAD` and `refs/`.
Our route passes `--lock`,
so `keep_locked` is set,
and Git's `done:` label unlinks `locked` only on a failure (`builtin/worktree.c`:601 to :603);
a `die` or a caught signal runs `remove_junk` instead and never unlinks it (`:273 to :300`).
Git creates each of `locked`,
`gitdir`,
the slot's `.git` and `commondir` by opening it with `O_CREAT | O_TRUNC` and then writing
(`wrapper.c`:760 to :766),
and nothing here is an exclusive create,
so a kill between the two calls leaves that file empty or short;
an empty `locked` is treated as no lock for attribution,
an empty `gitdir` or pointer names nothing,
and each such entry is refused or treated as the row its other names pick,
as the rows say.

- row 1,
  before `locked` (`mkdir`,
  `:505`).
  Registration:
  empty.
  Slot:
  empty,
  as A3 made it.
  Recognised by:
  no `locked`,
  no `gitdir`,
  and a candidate name.
  Cleared by:
  the directory alone;
  there is no lock and no file to remove.
- row 2,
  after `locked` (`:526 to :530`).
  Registration:
  `locked`,
  holding the sentence or a prefix of it,
  or empty in the instant before its write.
  Slot:
  empty.
  Recognised by:
  the lock,
  and an empty `locked` is recognised by the name alone,
  as row 1's shape with one file.
  Cleared by:
  `locked`;
  the directory.
- row 3,
  after `gitdir` (`worktree.c`:1129 or :1132,
  the first of the two writes).
  Registration:
  `locked`,
  `gitdir`.
  Slot:
  empty.
  Recognised by:
  the lock,
  or,
  when the lock is gone,
  `gitdir` resolving to `<slot>/.git`.
  Cleared by:
  `gitdir`;
  `locked`;
  the directory.
- row 4,
  after the slot's `.git` (`worktree.c`:1130 or :1133,
  the second write).
  Registration:
  `locked`,
  `gitdir`.
  Slot:
  `.git`,
  whose bytes name this registration.
  Recognised by:
  the lock,
  or `gitdir`.
  Cleared by:
  the slot's `.git`;
  `gitdir`;
  `locked`;
  the directory.
  A slot `.git` that is empty or short names nothing,
  so step 1 leaves it and the resume refuses;
  a person's `git worktree repair` or its removal finishes that case.
- row 5,
  after `commondir` (`:543 to :544`).
  Registration:
  `locked`,
  `gitdir`,
  `commondir`.
  Slot:
  `.git`.
  Recognised by:
  the lock,
  or `gitdir`.
  Cleared by:
  the slot's `.git`;
  `commondir`;
  `gitdir`;
  `locked`;
  the directory.
- row 6,
  after `HEAD` and the ref store (`:555 to :566`).
  Registration:
  `locked`,
  `gitdir`,
  `commondir`,
  `HEAD`,
  and the ref store's `refs/`,
  `logs/` and `logs/HEAD`.
  Slot:
  `.git`.
  Recognised by:
  the lock,
  and complete by the five facts.
  Adopted,
  not cleared.
  Git copies sparse-checkout patterns and per-worktree configuration after `HEAD` (`:573 to :582`) when those
  features are on,
  and a kill there leaves a registration the five facts call complete without those copies;
  this route's preflight refuses a sparse-checkout source and a worktree-scope configuration source,
  so neither copy runs for an add this design serves,
  and a registration that holds one is not this add's.
- row 7,
  Git's failure path killed after the unlink (`:601 to :603`,
  then `remove_junk` at exit).
  Registration:
  `gitdir`,
  `commondir`,
  `HEAD`,
  `refs/`,
  `logs/`,
  `logs/HEAD`,
  and no `locked`
  (`HEAD` stands only in the rare arm where the unlink follows a successful one).
  Slot:
  `.git`.
  Recognised by:
  no lock and a `gitdir` naming `<slot>/.git`;
  its contents then pick the row it is cleared as,
  with one exception:
  an entry complete by the five facts is refused and named until step 13 builds the lock design's installer,
  and is then adopted with its lock written again and one printed line (V22).
- row 8,
  Git's own removal of a failed add killed inside it (`remove_dir_recursively`,
  `:273 to :289`).
  Registration:
  whatever Git had removed so far,
  with `locked` still there unless the unlock ran.
  Slot:
  `.git`,
  or the slot itself,
  may be gone.
  Recognised by:
  the lock while it stands,
  then `gitdir`,
  then a candidate name.
  Cleared by the recipe for the names that stand.
  A kill that has taken both `locked` and `gitdir` and left the slot's `.git` is row 10's refusal,
  and one that has taken the slot's `.git` too is row 9's re-run.
- row 9,
  nothing found and the slot has no `.git`.
  Action:
  re-run A5,
  after the slot rule below.
- row 10,
  nothing found and the slot holds a `.git`.
  Action:
  refuse and name it.
  Nothing proves this add wrote a pointer whose registration is gone.

#### The recipe that clears rows 1 to 5, 7 and 8

Each step removes one name the row proved stands,
step 1 through the pinned `.claude/worktrees` parent and every later step through the pinned `<common>/worktrees`
parent,
and the lock goes last so the entry stays attributable at every earlier kill:

1. the slot's `.git`,
   when the row has one and its bytes name this registration;
   a `.git` naming anything else is left and the resume refuses;
2. `gitdir`;
3. `commondir`,
   when it stands;
4. `HEAD`,
   when it stands;
5. each file the listing shows inside `refs/` and `logs/`,
   `logs/HEAD` first among them,
   and then those two directories,
   which are empty once their files are gone;
6. any other entry the row's listing showed,
   and nothing else;
7. `locked` last,
   so every kill before this point leaves a locked entry and every kill after it leaves a
   directory of nothing but its own emptiness;
8. the registration's directory by name from the pinned parent (a plain `rmdir`,
   which answers `ENOTEMPTY` rather
   than removing anything it did not account for).
   The `worktrees` parent itself may be gone:
   Git's own prune removes it when its last registration goes (`builtin/worktree.c`:164 to :168),
   and a repository's first registration has none before Git makes it.
   `ENOENT` at any step means the name is already gone,
   which is what the step wanted;
   the recipe must not treat it as a failure,
   and the resume continues at the next step.

The re-read under the descriptor treats a name an earlier step of the same recipe already removed as done,
as read 4 of "The settling command" does for the registration,
and it happens twice:
once before step 1,
and once before step 7,
because the lock is the last thing that attributes the entry and a change to it under the recipe is a refusal.

Then re-run A4 and A5 and print what was removed.
Every kill inside the recipe leaves one of the rows above:
rows 2 to 7 while `locked` stands,
and row 1's bare directory after it,
because the recipe removes the files before the lock and the directory after it.
The earlier revision's recipe unlinked the lock first and then asked for an empty directory,
which fails for rows 3,
4 and 5,
whose registration holds files,
and it removed `logs/` before the file inside it,
which fails for row 6's shape,
which is the shape every settled registration has (X1 of the third check).
This recipe is the lock design's settle order with two steps its own list lacks:
the slot's pointer,
and the emptying of a registration's files before its directories.
The lock design's round is where both belong too.

The resume never runs `git worktree prune` instead:
a repository-wide prune would remove another slot's registration in the unmounted,
pointer-deleted state the lock design's D6 names,
so this rule is per slot by construction.

A standing inventory with no binding refuses the add at A2 as well:
the inventories are published with an exclusive link,
so a second publication answers `EEXIST`,
and an orphan inventory is the tail's leftover,
not the add's to clear (Z3 and "The record's end").
The add names it and points at the finisher.

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
- The slot holds a `.git` pointer or anything else,
  after the recipe has removed the row's pointer:
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

T20 plants one fixture per row,
rows 1 to 10,
each built from the source line that leaves it,
and asserts the recipe's order name by name,
that `locked` goes last,
and that every kill between two steps leaves a
row the table still recognises.
T20a plants rows 7 and 8 and the two failure-path kills and a relative `gitdir`,
T20b a foreign lock and a short prefix on this slot,
T20c another slot's entry and the names this slot does not own,
and T21 the slot's deletion and the identity's rewrite.

## The phases, and a kill at each step

### The eight phases and the resume states

The first five states are the proposal's ([cli-git-zfs-worktree-design-proposal.md](cli-git-zfs-worktree-design-proposal.md):424 to :426).
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
- The add's own fallback in progress:
  phase `settling`,
  with `settleReason=` naming the step that stopped.
  Proven by a clone that may stand or be gone,
  a registration that stands,
  an origin cleanup record that may stand with `owed`,
  and a managed lock that may stand or be gone;
  the fallback's own steps decide which,
  and the add continues at A20 rather than refusing.
- A verified checkout:
  phase `filled`.
  Proven by the registration's back-link,
  the pointer's bytes equalling `pointer=`,
  the mount's source being `cloneGuid=`,
  and the checkout's index and tracked content matching what the fill verified.

The phase vocabulary is this design's,
because the proposal says only "progress" ([proposal](cli-git-zfs-worktree-design-proposal.md):412 to :417).
V14 marks the eight words and their order as a writer's choice.

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
  because its set is fixed by that snapshot's transaction group (assumption A6).
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
  The refusal names the line,
  and says what a person does,
  because no command acts on such a record:
  restore the file from a copy,
  or read it and delete it,
  and then run the command again;
  `doctor` prints the same two ways (the project's answer of 2026-11-22,
  "Change nothing,
  name the file,
  and wait for a person");
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
  This is the one arm a reader meets when another hand destroyed an object (see "What no reader may conclude").
  A killed removal leaves the same arm,
  and the add refuses it too:
  a binding at `filled` whose clone dataset is gone is not the add's to settle.
  The one exception is the add's own fallback:
  a binding at `phase=settling` is the add's,
  its claim was taken at A1 and never released,
  and,
  while no plan for its clone stands,
  the add continues at A20 ("The add's own fallback").
  A plan standing refuses the add whichever phase its binding is at,
  `settling` included,
  which is the plan row's rule for a binding of any phase (K12 of the eleventh).
  A plan standing makes the add refuse and name what the plan design's "Which resume path a removal takes" gives,
  which is this design's addition to that heading (K9 of the tenth removal-side read).
  The add names the record and the one command "Which command is true for a state" gives the state it found.
  For a binding at `filled` whose clone is gone that is the settling command when no plan stands,
  and the plan design's "Which resume path a removal takes" when one does,
  which is the state U26 and J1 of the eighth name and K1 of the ninth.
  The tail belongs to the removal's order,
  and the plan design's finish pass,
  which runs before every forwarded command,
  or the next removal of the slot finishes it without the add (R3);
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
  naming the fields that disagree,
  with one exception for the registration line (W7):
  a registration whose slot holds no clone mount and whose `.git` names that registration is the add's own
  fallback's finished worktree,
  so `doctor` reports it as Git's own worktree and not as a missing binding,
  and a later removal finds no binding and no clone dataset under the pinned parent,
  so it takes the wrapper's ordinary worktree route and not the clone route,
  whose selection this design leaves to the wrapper's existing code and does not vouch for;
- for a binding at `phase=settling` that holds a `cloneGuid=`:
  the command "Which command is true for a state" gives its state,
  spelled for a person and one command per state,
  as the plan row has it:
  "run the add again at `<slot>`" when its clone's absence is proved,
  a registration stands,
  and no plan stands;
  `git worktree remove <slot>` for a plan whose head is `requested` or `unknown` beside a gone clone,
  which is the plan design's "Which resume path a removal takes" head-gone path;
  the settling command for a plan whose head is `present` or `refused`,
  which the same heading refuses and sends to this command,
  and for an absence that cannot be proved,
  or a clone gone with no registration standing;
  `git worktree remove <slot>` when its clone still stands and no plan stands,
  and,
  with no plan standing,
  the add again at the slot while nothing is busy;
  with a plan standing beside a standing clone it names the removal alone,
  because the add refuses over a plan (N1 of the eleventh add-side read);
- for a binding at `cloned` or `mounted` whose clone is gone,
  whether its absence is proved or cannot be proved,
  and for which no plan stands:
  the settling command (K14 of the eleventh removal-side read);
- for a binding at `phase=settling` with no `cloneGuid=`:
  "run the add again at `<slot>`";
- for a binding at `filled` whose clone is gone and no `settling` phase:
  the same list's command for that state,
  the settling command when no plan stands,
  and the plan design's "Which resume path a removal takes" when one does;
- each record whose identity no longer matches the live folder,
  naming the identity and the dataset GUID it compared;
- each inventory whose `entries=` disagrees with its lines,
  each record filed under a name its `slot=` or `kind=` refuses,
  each file whose owner or mode is not this account's,
  and each entry whose name is none of the four forms;
- the two known staging forms,
  whose names it removes only when the folder holds no live claim;
- each released claim,
  which is one small file per slot name ever used and is not a finding;
- for any state "Which command is true for a state" gives to the settling command,
  the two lines "The settling command" states,
  with the command's two spelled names.

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
and the binding's phase is no precondition for the removal:
a removal accepts a binding at any phase whose clone still stands,
`phase=settling` included,
read the live clone's identity from the binding and the pool,
and proves lineage from it as from any clone;
the phase stays where the add or the fallback left it for the whole removal.
A removal that finds a clone already gone and a `settling` binding,
and for which no plan stands to resume,
refuses,
and names the one command the section "Which command is true for a state" gives that state,
which is the add's own re-run or the settling command,
and never both (G5 of the sixth removal-side read,
U16 of the seventh,
J4 of the eighth,
and K2 of the ninth,
which keeps the sentence's scope).
Where a plan stands,
the removal is not this sentence's subject at all:
the plan design's "Which resume path a removal takes" decides,
as that section's plan row says.
What the removal does with this binding is read it,
confirm its origin fields,
and remove it where the route's one removal order places it.

That order is the origin cleanup design's list,
and this design states none of its own:
"The removal's one order of writes and removals" and "What a kill at each point leaves,
and who finishes it"
at `8cc31f59a4`,
its steps 1 to 14 under its own numbers:
the snapshot requests and the second deferred-destroy check are its steps 7 to 9,
the first check being its step 5,
its step 10 attempts the origin cleanup,
its step 11 settles the registration,
its step 12 removes this binding's files,
its step 13 removes the plan,
and its step 14 releases the claim.
That list is one step higher than the thirteen-step one an earlier revision of this document cited,
from its step 8 on,
because the deferred-destroy decision gave the second check a step of its own;
the correspondence between the two is by what each step does and not by the number (the list's own sentence).
Its rule is that the plan is the last witness and the claim is released last,
and its kill list gives each point's state and finisher.
What this record adds to that list is one sentence about its own three files (Z3 of the third removal-side check):
the two inventories are removed first and `<slot>.binding` last,
so the binding stands while its step 12 is unfinished and every kill inside the step leaves a binding a finisher
can recognise:
a kill after both inventories and before the binding leaves the binding alone,
which is that step's state before its first removal;
one after a single inventory leaves the binding and the other inventory,
and the finisher removes the rest.
A finisher that finds a standing inventory with no binding treats it as the same step's leftover and finishes it.

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
   the removal command writes that record at its step 6 (its V4,
   read at `22be706b09`),
   and the add's own fallback is its second writer (A23),
   so the record has two writers and one grammar.
   A removal that finds no such record,
   or one whose fields differ,
   refuses before its first delete,
   which is the same refusal the origin cleanup design gives a removal that cannot write its own record.
3. The destroys are the siblings' steps (its steps 7 to 10).
4. It settles the registration and releases the managed lock (its step 11).
   The mechanism is one,
   and it is the wrapper's own:
   the settle empties the registration through the held descriptor and removes the directory by name,
   which is the lock design's settle step (its "How the wrapper's own removal releases the lock",
   read at `7d0dcdc875`);
   it never hands the registration to `git worktree remove`.
   Git's own removal is the other caller of the same effect,
   a person's or a command's outside this route,
   and it takes the registration with its folder (F2,
   step A4),
   which this binding survives because it is not inside.
   A removal that finds the managed lock gone writes it again and says so,
   then releases it,
   as the decision of 2026-11-22 requires of every command that holds the slot and has proved
   the registration.
5. It removes the two inventories and then `<slot>.binding`,
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

### The condition a finisher meets

A removal meets step 5's condition itself,
because it made the confirmation at step 2.
A finisher meets it after a kill,
and its condition cannot be "the origin cleanup record stands":
on the ordinary path that record is gone at its step 10,
before the settle,
so a finisher after a kill between its steps 11 and 12 would keep every binding of an ordinary removal and block
the slot
(Z1 of the third removal-side check).
The condition is therefore either of these:

- a plan stands for `cloneGuid=` with `head=gone`,
  and the plan's `originDataset=`,
  `originDatasetGuid=`,
  `originSnapshot=`,
  `originSnapshotGuid=` and `originSnapshotCreatetxg=` equal this binding's
  (the plan carries all five,
  read at `1921adf393`);
- no plan stands,
  and the origin cleanup record stands with those five fields equal.

The origin cleanup record's absence is not a failure when a plan stands.
When neither holds,
the binding stays and the slot stays blocked,
which is Q8's case.

Who may finish that step,
in the words a sibling can quote:
the plan design's finish pass,
which runs before every forwarded command,
or the next removal of the slot while a registration still stands;
never an add,
except the add's own fallback at A20 to A27,
and only where `phase=settling` says the add itself stopped mid-fallback.
The pass takes the claim,
proves the clone's absence,
settles the registration,
finishes its step 12 for the binding that names the plan's clone and removes the plan
(its "The finish pass that settles a registration",
read at `1921adf393`);
the next removal resumes from the plan.
An add that meets a `filled` binding whose clone is gone and which is not `settling` refuses,
and names the command "Which command is true for a state" gives it:
the settling command when no plan stands,
and the plan design's "Which resume path a removal takes" when one does (R3 of the third add-side check,
"A resumed add" and K1 of the ninth removal-side read).

The condition's second arm,
no plan and the origin cleanup record standing with equal fields,
has no finisher among the pass and a removal:
the pass is driven by plans,
and a removal resumes from one,
so with no plan neither acts.
A person's command stands there:
the settling command,
which its printed rule names for exactly this state (H2).
The arm is still the same confirmation a removal makes at its own step 2 for a plan it wrote;
this design keeps it
for a reader that finds the pair standing alone,
and the settling command is what finishes it.

### What a record whose clone is gone means

The clone's GUID is absent from the pinned clone parent,
and no child of that parent carries `cloneGuid=`.
Two readings follow from the order above,
and the order is what tells them apart.

- A destruction plan stands for `cloneGuid=`.
  A removal is in flight or was killed,
  wherever it was killed:
  because the origin design's V13 puts the plan's removal after this binding's,
  a kill after the settle and before step 5 leaves the plan standing beside this binding.
  `doctor` prints the plan's standing intent and this record's origin names,
  and the finisher above finishes its step 12:
  the pass before the next forwarded command,
  or the next removal,
  meeting the plan's arm of the condition.
  If the plan reports the head's destroy as unresolved,
  nothing in the binding is regenerated and nothing is settled on a guess.
- No plan stands.
  Then either the clone was never destroyed by this wrapper (nothing else can remove a plan on the normal path),
  or a removal broke that order,
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
The command each such state's reader names is "Which command is true for a state"'s,
and `doctor` and the refused add print it there.
The project decided on 2026-11-22 that a person may settle it,
and the command below is the way
(`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
"A slot whose recorded clone is gone and cannot be proved gone"):
"Print an explicit command that removes the record after you confirm."
Taken as the question stated it,
the cost is that "a command that removes evidence is one typo from a clone nobody then cleans up",
and the guard below is written against exactly that.

### The settling command

Its name,
listed for the project's veto (V21):
`git monochromatic-git-policy-cli settle-worktree-clone-record <slot> --confirm-removal-of <slot>
--confirm-clone-guid <the last four decimal digits of cloneGuid=>`.

The program's name is the wrapper's own,
the one its subcommands run as (`SPEC.md`:616 and :507);
the earlier revision's `git-policy-cli` names no executable and a pasted line would fail.

The guard against "one typo from a clone nobody then cleans up" is three data the record supplies and the
person must copy:

- the slot is named twice,
  once as the argument and once as the flag's value,
  and the two must be identical,
  so a mistyped flag refuses;
- both must equal the slot the record derives,
  which is the last path component of the slot path that `slot=` holds in hexadecimal;
- the GUID flag must equal the last four decimal digits of the binding's own `cloneGuid=`,
  and the command the refusal and `doctor` print shows that flag's value as a placeholder,
  not as the digits.

What that catches and what it cannot,
said plainly:

- it catches a mistyped slot,
  a slot typed for another record,
  and a blind paste of the printed line,
  because the line a person copies does not carry the two values;
- a person who finds another slot's record by a typo and then types that record's own printed GUID has confirmed
  that record,
  which is exactly what they asked for;
- a scripted caller that supplies all three cannot be told from a person,
  and the command does not claim to tell them apart;
- a person who reads every printed line and still means the wrong thing is not caught,
  and the project chose this command knowing that cost.

What it reads,
in order:

1. The slot's claim,
   by `take_claim`,
   and a live owner of the newest generation refuses,
   naming that owner's process id.
2. The binding at `<slot>.binding`,
   by the same reader the add and the removal use.
   An unreadable,
   foreign,
   oversized or wrongly owned record refuses by name and changes nothing,
   which is the project's answer on an unreadable record
   (the decision record,
   "A record that cannot be read":
   "Change nothing,
   name the file,
   and wait for a person");
   this command is not a way around that refusal.
3. The pinned clone parent's children,
   for `cloneGuid=`,
   by GUID and by name.
   A successful listing that finds either refuses;
   a listing the pool refuses or returns only part of is not a refusal,
   because the state this command exists for is precisely the one where the absence cannot be proved,
   and the refusal or the partial answer is kept and printed,
   and is named in the text the person confirms (W3 of the fourth removal-side check).
4. The slot:
   a `.git` at the slot,
   the registration it names,
   that registration's identity against `registrationIdentity=`,
   the registration's own entries (an `index` among them),
   and the mount table for a mount at the slot.
   A `.git` that names another registration refuses;
   one that names this registration is the ordinary abandoned slot's,
   and the recipe's first step removes it.
   A registration the command's own recipe already removed counts as that step done,
   as the recipe treats its own `ENOENT`,
   so a rerun after a kill goes on to the plan and the inventories (G2 of the sixth removal-side read).
   Whether a checkout has begun is "When a checkout has begun"'s one test,
   and this read supplies it:
   the slot's own entry count besides `.git`,
   read while nothing is mounted there.
   The registration's `index` and the lock are read and are not the test,
   for the reasons that heading gives (G1).
5. The origin cleanup record,
   when one stands,
   compared with this binding's five origin fields.
   It is compared and never removed or rewritten:
   the command prints it,
   refuses only when its fields disagree with the binding's,
   and the origin's cleanup stays the origin look's business.
6. The plan folder,
   for a plan whose `cloneGuid=` is this clone's.
   A plan whose `head` is `gone` refuses:
   that is the finish pass's,
   and it removes the plan itself.
   Any other plan is printed and removed after the confirmation;
   it is a cancellation's kept plan whose clone was destroyed outside the wrapper,
   and no next removal will ever come to end it (W2 of the fourth removal-side check).

What it refuses,
changing nothing:

- a live owner of the slot's claim;
- the binding unreadable,
  foreign,
  oversized or wrongly owned,
  or disagreeing with the slot's `.git` and the registration while that registration stands;
  a registration this command's own recipe already removed is read 4's case,
  which counts it as that step done;
- a dataset under the pinned parent that carries `cloneGuid=`,
  or one whose name is `cloneDataset=` while another GUID stands there.
  A reused name is a refusal and never a state to settle:
  a live dataset holds the recorded clone's name,
  so nothing proves the recorded clone's absence,
  and a removal and this command both refuse and name the reuse (N3 of the fifth removal-side check);
- a mount at the slot;
- a `.git` at the slot that names another registration;
- a standing origin cleanup record whose five origin fields disagree with the binding's;
- a plan for `cloneGuid=` whose `head` is `gone`;
- a plan it cannot read,
  or one whose recorded identities disagree with the live clone,
  the binding or the pool,
  which is the refusal the plan design asks of this command at "A clone gone whose absence cannot be proved"
  (its heading at `28c590b4c4`;
  it was "What a later run does" at `1921adf393`) (N3);
- a binding at `phase=settling` whose clone's absence is proved,
  whose registration still stands,
  and for which no plan stands,
  which the add's own re-run finishes at A20 instead.
  With a plan standing the command takes the binding,
  whatever its head word but `gone`,
  and its read 6 prints what the plan held and removes it with the records:
  a plan beside a `settling` binding whose clone a person destroyed outside the wrapper is otherwise a ring,
  the removal naming this command,
  this command naming the add,
  and the add naming this command (K7 of the tenth removal-side read and U40 of the tenth add-side read).
  With the registration gone,
  as a kill inside this command leaves it,
  the command takes the binding for either listing answer and removes the records,
  because no add can run the checkout without a registration (U16 of the seventh add-side read);
- a binding at `phase=settling` that holds no `cloneGuid=`,
  which entered from `registered` or `snapshotted`:
  the fallback recovers a clone the add made before its record landed,
  by its proposed name,
  before it reaches A26,
  so a binding that still holds none at that point names no clone of this add's,
  and the command refuses it and names the add's re-run,
  which continues at A20 without a clone's identity;
  a `settling` binding whose absence cannot be proved,
  which holds a `cloneGuid=`,
  and whose registration stands or is gone,
  is this command's case,
  and the command reads the slot and the registration as A20's resume does before it acts;
- two records that both answer the two names.

What it asks the person to confirm,
and how:
it prints the slot path and identity,
the clone's dataset name,
GUID and `createtxg`,
the origin snapshot's name,
GUID and `createtxg`,
what each read answered,
which of them could not prove the clone's absence and why (a refused listing),
the exact removals below in order,
and one sentence saying that after them the wrapper can no longer prove which clone the slot held.
The confirmation is the two flags' values,
never a prompt,
because the wrapper's commands are non-interactive.

What it removes,
in this record's order,
which is the recipe's,
when no checkout has begun ("When a checkout has begun"):

1. a lost managed lock is written again,
   with one line,
   because the command holds the slot and has proved the registration,
   as the decision of 2026-11-22 requires of every such command;
2. the recipe's eight steps on the recorded registration:
   the slot's `.git`,
   `gitdir`,
   `commondir`,
   `HEAD`,
   the files inside `refs/` and `logs/`,
   any other entry the listing showed,
   `locked` (releasing the lock it wrote or found),
   and the directory by `rmdir`;
3. the plan it printed,
   whose head is not `gone`,
   removed after the confirmation and before the two inventories,
   printing what the plan held (the snapshot names it recorded as destroyed);
4. `<slot>.origin-inventory`,
   then `<slot>.clone-inventory`;
5. `<slot>.binding` last;
6. its claim,
   released after the removal,
   which leaves the newest generation file as a released claim.

For a `settling` binding whose slot holds a checkout begun,
which "Which command is true for a state" names with its command and the removal's spelling,
the command removes only the records and leaves the worktree:
the plan,
the two inventories,
the binding,
and the claim,
in that order.
It writes no lost lock there:
A24 released the lock on purpose,
an ordinary worktree is never locked,
and a lock left on it would make the removal this command names refuse (U19 of the seventh add-side read).
It names the registration and the slot it leaves,
with that list's words (U20).
The full recipe would strip the registration of a worktree Git made at A25 and leave its files behind,
so it never runs there (G1).
V23's other option,
B,
refuses every `settling` binding instead,
for the project to choose.

That order is one sentence a sibling can quote,
and the plan design cites it:
a lost lock written again,
then the registration,
then the plan,
then the two inventories,
then the binding,
then the claim.
The plan goes before the inventories so the binding stays the witness a replayed run reads;
the project decided this on 2026-11-22,
"A further answer:
the settling command and a plan a cancellation kept",
in its words "after the person confirms,
the command prints the snapshot names the plan recorded as destroyed,
then removes the plan with the other records and frees the slot"
(`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`).

What a kill leaves:

- after the claim and before the lost-lock write:
  the claim naming a dead owner,
  and the next run takes the next generation;
- inside the lost-lock write:
  a lock that is a prefix of the sentence,
  which the next run completes (the lock design's S2),
  or a person's reasonless lock if it is shorter,
  which a removal refuses until `git worktree unlock`;
- inside the recipe:
  one of the rows,
  with the binding still standing,
  and the next run repeats the recipe;
- after the recipe and before the plan:
  the registration is gone and the plan stands,
  and the next run reads the binding,
  not the plan,
  and removes the rest;
- after the plan and before the first inventory:
  the binding and both inventories stand and the plan is gone,
  and the next run removes the rest;
- between the two inventories,
  or after them and before the binding:
  the binding and whatever inventories remain,
  and the next run removes the rest;
- after the binding and before the claim's release:
  the claim alone;
- after the claim's release:
  nothing of this operation stands,
  and the origin snapshot,
  if it still stood,
  is left to the origin look and `doctor`'s unaccounted-snapshot line,
  which the command's own lines say.

What `doctor` prints,
and what the refused add prints:
the evidence lines,
and the command itself with both flags shown as placeholders
(`--confirm-removal-of <slot>` and `--confirm-clone-guid <the last four digits printed above>`),
spelled with the slot from the record's own `slot=`,
so the line a person copies carries neither value and each must be read and typed.
The command is printed whichever of "Which command is true for a state" names it for the state that stands,
and the add's refusal,
the removal and `doctor` print it there,
not only when the pool refused the listing (G2 of the sixth removal-side read).
That list names it in five states,
and one of them is the state a person makes by destroying the clone by hand before any removal ran:
a binding at `filled`,
its clone gone,
and no plan at all,
where nothing can act but this command (H2 of the seventh removal-side read,
U26 and J1 of the eighth).
A rerun of the command is let through in the states a kill inside it leaves,
because read 4 counts a registration the recipe already removed as that step done,
which is what the kill list below says.

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
by refusing a binding whose `registration=` no longer answers with `registrationIdentity=` (a registration the
settling command's recipe already removed is that command's read 4,
which counts it as done),
and by the scan of "A resume after a kill between Git's add and the registered write" when there is
no recorded path yet.

## Which step of the plan builds which part

### What step 10 takes, and from which document

One list,
in the order a builder needs it.
Each line names the part and the document that specifies it,
and the last list says what step 10 cannot take yet.

- The add's preflight,
  before any write:
  the accepted proposal's pre-flight list,
  and the route's own checks;
  this document's A0 names it and adds no rule.
- The account companion's check code,
  before the claim:
  the start-probe design's "The account companion" and its step 10 entry,
  under the accepted decision of 2026-11-21 that it is built in step 10.
  Step 6 shipped without it,
  so step 10 also builds the reaper rule the design says step 6 does,
  because the only reaper in the tree still takes a `Child`.
- The claim,
  the home,
  the binding's fields and phases,
  the two inventories,
  the flush:
  this document,
  from "The home" to "The size bound".
- The add's writes in one order,
  A0 to A12:
  this document,
  "The add's writes in one order".
  Step 10 builds on a failure what V16's A says:
  it keeps the claim,
  the slot and the locked registration,
  reports,
  and lets the next add resume (W5 and W6 of the fourth check).
  A20 to A27 are reserved labels and a phase word that step 10 writes only if the project moves their step;
  their build is step 11's,
  as the next bullet says.
- Git's invocation and its sentence:
  the lock design's "The invocation" and "The reason text",
  read at `f45d05a96d`;
  this document records the nonce the sentence is recomputed from.
- The finder,
  the rows,
  the recipe and the slot rule after a kill inside Git's add:
  this document,
  "A resume after a kill between Git's add and the registered write".
- The frozen-content comparison:
  the accepted dirty-main restore design's "The frozen proof",
  placed at this document's A10a and inside the accepted decision's window
  (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
  "Which writes the frozen proof refuses").
- The restore and the exchange of the copied `.git` with the pointer:
  the accepted dirty-main restore design's "The restore",
  placed at this document's A10b and A11 under the accepted decision's "A2:
  keep copy,
  swap at end".
  What step 10 builds of it is the accepted design's own allocation
  (its "Which build step builds which part"):
  the leave-outs of step 4,
  the private index of step 5,
  and the check,
  the publication and the exchange of step 8,
  with the frozen proof.
  Steps 6 and 7 are the dirty half,
  which is step 11's,
  and they run empty for a clean source,
  which that design says (N8 of the eleventh add-side read).
- The mount's own mechanics and the attach:
  steps 7 to 9's designs;
  this document's A10 records the phase and the identities a reader compares.
- The add's fallback,
  A20 to A27,
  built at step 11 with the rest of the accepted dirty-main restore design's settle
  (its "Which build step builds which part"):
  that design for the marker,
  the reason,
  the unmount and the destroys,
  the origin cleanup design for the record its add publishes,
  the lock design for the release,
  and this document for the phase,
  the close and the claim.
  The dirty-main design and the lock design both allocate these to step 11,
  and this list follows them (W6).
- The removal's lineage proof and the claim it takes:
  this document's "The removal's lineage proof",
  built at step 14.
- The record's end and the tail:
  this document's "The record's end" and "The condition a finisher meets",
  the origin cleanup design's list and the plan design's finish pass,
  built at step 16.
- The settling command:
  this document's "The settling command",
  built at step 16 with the record's end,
  because it needs step 10's claim,
  step 14's reader and step 16's recipe and removal order;
  `doctor`'s two evidence lines for it are step 18's (W6).
- `doctor`'s listing of this folder:
  this document's "`doctor`",
  built at step 18.

### What step 10 cannot take yet

- The frozen proof and the exchange are specified in the accepted dirty-main restore design
  (its "The frozen proof" and "The restore"),
  and its own allocation gives both to step 10
  (its "Which build step builds which part").
  What is still the owner's is one plan sentence:
  the plan's step 10 text still lists "the discarded copied Git administration",
  which the accepted decision's "A2:
  keep copy,
  swap at end" makes wrong.
  A10a,
  A10b and A11 take their mechanics from that design and their order from this one.
- The account companion's per-add deadline is measured in the start-probe design's owed guest run (its M11).
  The build can start on the design's words;
  the number ships only with that measurement (rule `LQM`).
- The fallback's unmount request and receipt are the handler's,
  and the staging design must state that an add may use them;
  the lock design's release step is named at its current tip `4625537ae5`,
  and the installer that step 10 lacks until step 13 is its too (V22).

### What each later step builds

- Step 14 builds the removal's reading of the binding and both inventories,
  the refusals of "The removal's lineage proof",
  and the claim the removal takes.
  It also builds the add's read of the plan folder,
  because step 14 is the first step that writes a plan,
  and until then the folder holds nothing for the add to find (N4 of the eleventh add-side read;
  the add's list in "What step 10 takes" is not the place for it).
- Step 16 builds the record's end,
  which is this design's assignment rather than the plan's,
  together with the plan design's finish pass,
  which finishes the tail after a kill:
  plan step 14 sends origin cleanup and the Git settle to step 16,
  and the record's removal belongs with them.
  It builds absence proven by the recorded GUID,
  the hand-over to the origin cleanup record,
  the removal of the three files,
  and the released claim.
- Step 18 builds `doctor`'s listing,
  the checks of "`doctor`" and the staging-name removal rule.
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
one arm per row of "The states Git's source can leave",
each fixture planted as the source line and the third check's measured shapes leave it,
never raced.
Row 1,
an empty `worktrees/<name>` whose name is the slot's through Git's sanitiser;
row 2,
the same with `locked` holding the recomposed sentence and its newline,
and a second arm with an empty
`locked`;
row 3,
row 2 plus `gitdir` holding `<slot>/.git` and one newline (not `gitdir: <path>`,
which is the slot's
pointer file);
row 4,
row 3 plus `<slot>/.git` holding `gitdir: <registration>` and one newline;
row 5,
row 4 plus `commondir` holding `../..` and one newline;
row 6,
row 5 plus `HEAD`,
`refs/` and `logs/` with `logs/HEAD`.
Row 6 is adopted and written as `registered`;
rows 2 to 5 are cleared by the recipe and the add re-runs A5 and A6.
Every arm asserts the recipe's order name by name,
that `logs/HEAD` is removed before `logs/` and that `logs/` before the `rmdir`,
that `locked` is the last file removed,
that the directory goes by `rmdir` from the pinned parent,
and that a kill before each step leaves a row the table still recognises.
The re-run then makes the registration at the slot's own name and not at `<slot>1`,
which F1's counter would otherwise make.

T20d.
`the_two_nothing_found_rows`:
row 9,
no registration at any candidate name and no `.git` in the slot,
re-runs A5 and makes the registration at the slot's own name;
row 10,
no registration and a `.git` in the slot that names no live one,
refuses and names it.

T20a.
`the_failure_paths_leftover_and_a_kill_inside_gits_removal_are_cleared`:
row 7,
no `locked` with `gitdir`,
the slot's pointer,
`commondir` and `HEAD`,
recognised by the `gitdir` and cleared as row 5's shape would be,
and a second arm with the five facts complete,
which step 10 refuses and names until step 13's installer exists,
and which is then adopted and its lock written again with one printed line;
row 8,
`locked`,
`gitdir` and the pointer with the slot partly gone,
cleared by the recipe for the names that stand,
and a second arm in which both `locked` and `gitdir` are gone and the slot's `.git` stands,
which is row 10's refusal;
a `gitdir` written relative (`../../../../slots/<slot>/.git`-style) resolves against the registration's
directory and is cleared the same way;
and a `refs/` holding a file the listing shows is emptied before `refs/` goes.

T20b.
`a_foreign_lock_and_a_short_prefix_on_this_slot_are_refused_and_another_slot_is_left`:
an entry whose `gitdir` names this slot and whose `locked` holds other words,
and one whose `gitdir` names this slot and whose `locked` holds a prefix shorter than the sentence's fixed part
and nonce,
are each refused with the entry and the words' first line,
and nothing is removed;
an entry whose `gitdir` names another slot is left and never refuses,
which T20c carries.
An empty `locked` is not among them:
with a `gitdir` naming this slot it is row 3's shape and is cleared,
and with a pointer in the slot it is row 4's,
which is the one rule for an empty lock.
Two entries that both attribute to this add are reported and neither is removed,
which is the lock design's own assumption for that case.

T20c.
`a_name_git_could_not_derive_and_a_name_another_slot_owns_are_left`:
a bare directory named after another slot,
one whose name this slot's sanitiser cannot produce,
and one that is a candidate of this slot but that another slot's binding owns,
are left and named,
so a resume for one slot never removes another slot's directory.
The `topic` and `topic1` walk is the fixture:
`topic1.binding` stands with no `registration=`,
and slot `topic`'s resume leaves `worktrees/topic1` alone.
A complete registration at a candidate name no binding owns,
whose `gitdir` names another path,
is left too,
which is the finder's third attribution bullet and the arm T20b no longer carries.

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

T29.
`a_fallback_that_succeeds_closes_its_record_and_releases_its_claim`:
one arm per entry phase,
`registered`,
`snapshotted`,
`cloned`,
`mounted`,
and one `cloned` binding whose clone a live read shows mounted,
each with the objects that phase leaves:
the run writes `phase=settling` and `settleReason=` before the first destroy,
and the `registered` arm proves A23 recovers the snapshot's identities by its proposed name,
runs the steps that phase needs,
removes the two inventories and then `<slot>.binding`,
releases the claim,
and leaves an ordinary worktree at the slot with no binding,
no inventory and no claim generation owned by it.
The `snapshotted` arm proves the recovery reads the clone at the proposed name,
and that with no dataset there no clone is named,
and the `mounted` arm proves the unmount comes before the destroy.

T30.
`a_kill_at_each_fallback_step_leaves_a_settling_binding_the_next_add_finishes`:
a kill before A20,
after A20 and before A22,
after A22,
after A23 with a refused destroy,
after A24,
after A25,
after A26,
and after A25 with the pool refusing the listing;
the first leaves no `settling` phase and the ordinary resume;
the arms from A20 to A24 leave `phase=settling` with the reason,
and those whose clone is gone and whose absence the fixture proves assert that the next add continues at A20 and
closes at A26 and A27;
the arm killed between A20 and A22 has the clone standing,
so it asserts that the person's removal accepts the `settling` binding and removes the clone,
and that the add again at the slot is true there too while nothing is busy;
the arm after A25 with the pool answering asserts that the add's next run continues at A20 and closes the binding,
because the absence is proved and a registration stands;
the arm after A25 with the pool refusing the listing asserts that the add's next run stops and names the
settling command,
which removes the records only,
because a checkout has begun;
the arm after A26 asserts that no binding stands,
so only the claim's liveness rule has work to do;
an arm for a plan standing beside a `settling` binding whose clone stands asserts that the next add refuses and
names the plan and the plan design's command for its head word (K5 of the ninth);
and an arm for a plan at `head=present` beside a `settling` binding whose clone is gone and whose registration
stands asserts that the settling command accepts it,
prints what the plan held and removes it with the records (K7 of the tenth).

T31.
`a_refused_snapshot_destroy_leaves_the_origin_record_owed_and_the_add_finishes`:
a hold on the origin snapshot at A23,
the destroy refused,
the origin cleanup record standing with `owed`,
the add going on to A26 and A27,
and the look finishing the snapshot later;
the binding and the claim are gone and the worktree is Git's own checkout.

T32.
`a_settling_binding_is_the_one_state_an_add_continues`:
a binding at `filled` with no clone and no `settling` phase refuses (R3),
and the same binding with `settling` and its reason is continued at A20 while no plan stands;
a `settling` binding with a plan standing refuses and names the command the plan design's "Which resume path a
removal takes" gives,
and acts on nothing;
a binding whose settle reason names a step this release does not know refuses and prints both.

T33.
`the_settling_command_removes_only_what_it_proved`:
one arm per refusal:
a slot spelling that differs between the argument and the flag,
a GUID flag that is not the record's last four digits,
a misspelled slot that finds no record,
a live claim owner,
an unreadable record,
a `settling` binding whose clone's absence is proved while its registration stands and no plan stands,
a `settling` binding whose absence cannot be proved but whose clone answers under the pinned parent,
a dataset under the pinned parent that carries `cloneGuid=`,
a reused `cloneDataset=` with another GUID,
a mount at the slot,
a slot `.git` that names another registration,
a plan for `cloneGuid=` whose `head` is `gone`,
an unreadable plan,
and two records that both answer the two names;
each refuses,
names what it found,
and changes nothing.
Then the arm that succeeds,
planted as the ordinary abandoned slot leaves it:
the clone's GUID absent (or the pool's listing refused or partial and printed as the evidence),
the slot's `.git` naming this registration and therefore removed by the recipe's first step,
the registration standing and emptied by the recipe,
`locked` removed last of the recipe's files,
the lost lock written again and one line printed,
a kept plan at `head=present` printed with the snapshot names it recorded as destroyed,
and removed before the two inventories,
the two inventories removed,
the binding removed,
and the claim released;
and a kill before each removal leaves the state the next run repeats,
including a kill after the plan and before the first inventory.
A second success arm plants the same state with the lock already gone,
and asserts the lock is written again before it is released;
a third plants a `settling` binding whose absence cannot be proved,
and asserts the command accepts it and removes its records;
a further arm plants a binding at `cloned` or `mounted` whose clone is gone,
whose absence cannot be proved,
and for which no plan stands,
and asserts the command accepts it and removes its records,
which is the state K14 of the eleventh removal-side read names;
a fourth plants a disagreeing origin record,
which refuses;
a fifth plants a `settling` binding after A25 whose slot holds a checkout begun,
and asserts the command removes only the records,
leaves the registration and the slot,
and names the worktree;
a sixth plants a kill inside A25 (files in the slot and no new `index` in the registration),
which asserts the same records-only mode;
a seventh plants a kill after the restore and before A25 (an `index` in the registration and only the pointer in
the slot),
which asserts the full recipe,
and together with the sixth that is the arm that tells the two modes apart ("When a checkout has begun");
an eighth plants a `settling` binding whose slot's `.git` and registration the recipe already removed,
which asserts the command accepts it for either listing answer;
a ninth plants a `settling` binding with no `cloneGuid=`,
which refuses and names the add's re-run;
a tenth reruns the command after a kill between the recipe and the plan,
with the registration already gone,
and asserts the rerun goes on to the plan and the inventories;
an eleventh plants a binding at `filled` whose clone is gone with no plan,
which asserts the command is the one printed for it (J1 and J3 of the eighth reads);
a twelfth plants a `settling` binding whose clone is gone,
whose registration stands and whose plan is at
`head=present`,
and asserts that the command accepts it,
prints what the plan held,
and removes the plan with the records,
which is the state K7 of the tenth names;
and a thirteenth plants a `settling` binding whose clone still stands beside a plan at `head=present`,
and asserts that the add refuses and names the plan and the command the plan design's "Which resume path a removal
takes" gives,
so that no plan is left with nothing to act on it (K5 of the ninth).

T34.
`the_removal_accepts_a_settling_binding`:
a removal at a slot whose binding says `settling` and whose clone stands,
mounted and unmounted,
proves lineage from the live clone and removes it;
a busy unmount at A21's arm withdraws the request,
leaves `settling` with the clone mounted,
and prints the pending worktree's line with its clause and `git worktree remove <slot>`;
and a removal whose binding says `settling` and whose clone is gone,
with no plan standing,
refuses and names the one command "Which command is true for a state" gives that state:
the add's re-run when the absence is proved and a registration stands,
and the settling command when the absence cannot be proved or the registration is gone;
and a removal that finds a plan standing takes the plan design's "Which resume path a removal takes" instead,
which is not this test's subject.

T35.
`doctor_names_the_right_command_for_a_settling_binding`:
a `settling` binding whose absence is proved prints "run the add again at `<slot>`" while a registration stands,
one whose absence cannot be proved prints the settling command,
one whose absence is proved and whose registration is gone prints the settling command,
one whose checkout has begun prints the settling command,
whose own line carries `git worktree remove <slot>` and `--force` as well "if Git refuses it",
one that holds no `cloneGuid=` prints "run the add again at `<slot>`",
one whose clone stands prints `git worktree remove <slot>`,
a binding at `filled` whose clone is gone with no plan prints the settling command,
a binding at `cloned` or `mounted` whose clone is gone with no plan prints the settling command,
and the same with the pool refusing the listing prints it too,
a registration whose slot holds no clone mount and whose `.git` names it prints the worktree line and not a missing
binding,
and a `filled` binding whose clone is gone with a plan standing prints the plan design's command for the head word
the plan holds (K3 of the ninth removal-side read).
A last arm plants a `settling` binding beside a plan at `head=present` and one at `head=requested`,
and asserts that `doctor` names the settling command for the first and `git worktree remove <slot>` for the second,
which is one command per state and the same rule as the plan row (N1 and N5 of the eleventh add-side read).

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
The siblings' current tips,
as this round read them,
are the lock design at `4625537ae5`,
the destruction plan at `cf51d140af`,
and the origin cleanup record at `ea271e9e1d`,
and nothing here adopts their answer.
Some lines below and above name an older commit with "read at";
that is where the fact they state was read,
and the tips here are the ones this design is checked against now
(U33 of the eighth add-side read and U38 of the ninth).
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
  A10 and its A10a and A10b at D6,
  A11's exchange as D6's end and D7's start,
  A12 at D7,
  and A7 retired because the lock design writes no record
  (its "The record whose nonce the lock's proof rests on",
  read at `7d0dcdc875`).
  What the lock design must then name,
  because this document owns the order:
  D6 as A10 with the copied administration standing and A11's exchange ending it,
  which its tip already says at `7d0dcdc875`,
  and the fallback's record at A20,
  its release following A20 rather than "the end of A11".
  A kill inside Git's add,
  D1 to D2c,
  is this design's A6 and the table under "A resume after a kill between Git's add and the registered write":
  the finder is this design's (X13),
  and the lock design's own finder section cites it.
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
- The destruction plan's "Readers that do not exist" now records,
  in its own words,
  that the add "reads the plan folder to refuse and for nothing else",
  writes no plan,
  stops where a plan names the clone of the binding the add meets,
  names the plan and the command "Which resume path a removal takes" gives,
  and acts on nothing.
  This design's add does that,
  and it writes none.
  That is one rule in two documents and not a conflict a record settles:
  the read is V24,
  and the paragraph at K9 of the tenth removal-side read is in that section,
  written at `c54601afd8`.
  Its heading there no longer fits,
  since the section records a reader rather than a reader that does not exist,
  and that design's round is to change it (N7 of the eleventh add-side read and K12 of the eleventh removal-side
  read).
  The five readers of the folder are named in "Which command is true for a state".
- The destruction plan keys its record by the clone's GUID and keeps no pointer inside a registration
  (its "What finds a plan",
  read at `1921adf393`).
  This design keys by the slot and keeps its record outside any registration.
  Neither record can be read by the other's key alone,
  and the set still reaches from any one key through this record's fields (V17).
  The plan's field called `slotName` holds the clone's dataset name and is not this key:
  "What the key is made of at each moment" states the rule the plan can cite,
  and the third removal-side check's Y6 is the plan's to apply.
- The destruction plan publishes unsynced by default (its Choice 5),
  and this design flushes every write;
  the origin cleanup record is unflushed too (its V14's option C and its L8).
  All three now rank the same rule by position first,
  and V19 is that rule:
  this record's writes all fall on the flushed side.
- The destruction plan's finish pass takes the claim,
  proves the clone's absence,
  settles the registration,
  finishes its step 12 for the binding that names the plan's clone and removes the plan
  (its "The finish pass that settles a registration",
  read at `1921adf393`).
  This design's V20 keeps that order among its own three files,
  and "The condition a finisher meets" is the pass's,
  so the pass is the finisher and the add is not;
  the second gate review's X3 and the third's Z1 are the two findings that made both sentences necessary.
- The accepted dirty-main restore design quotes a sentence of this design that is no longer its text,
  in its "Dependencies on the sibling designs":
  "and a removal that finds a clone already gone and a `settling` binding refuses and names the add's own re-run
  instead".
  The words that should stand there,
  word for word as "The record's end" reads,
  are:
  "and a removal that finds a clone already gone and a `settling` binding,
  and for which no plan stands to resume,
  refuses,
  and names the one command the section "Which command is true for a state" gives that state,
  which is the add's own re-run or the settling command,
  and never both".
  That is an edit to an accepted design and belongs to the project (U23,
  and item 7 of the eighth reads).
- The origin cleanup record publishes at its step 6,
  after every refusal and before the first destroy,
  and says a new add may take the freed slot at once.
  This design agrees with both:
  step 2 of "The record's end" only confirms that record,
  and the origin design's V13 keeps this binding's removal before the plan's and the claim's release last,
  which its list numbers 12,
  13 and 14.
  The proposal decides the freeing ("settling does not wait for the origin",
  [proposal](cli-git-zfs-worktree-design-proposal.md):468 to :469).
- The origin cleanup record's add subsection makes an add that fails publish the same record before it destroys
  the clone it made,
  and its own invocation attempts the cleanup
  (its "The add that fails and destroys what it made",
  read at `8cc31f59a4`).
  This design's A20 to A27 call that writer and that attempt,
  and a refused snapshot destroy leaves the record `owed` there,
  which is the rule this design follows at A23 and the reason is in "The add's own fallback".
- The origin cleanup record's moment is its own V4,
  still open to the project's veto.
  This design's step 2 depends on that veto:
  if the owner moves the origin record later than the first delete,
  this design's step 2 reopens with it.
- The origin cleanup record removes no staging entry and reports the ones it finds (its V8,
  read at `8cc31f59a4`),
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
and V16 to V20 are the choices the second gate review moved here from this design's questions,
because it judged them writers' choices rather than the project's;
V21 to V24 are this design's own additions,
and each entry names its alternatives and ranks them.

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
  The origin cleanup record removes none and reports them (read at `8cc31f59a4`),
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
The eight phase words and their order.

- A,
  eight phases as above (adopted):
  example `phase=slot` after the empty slot is verified,
  then `registered`.
  Pros:
  each write names the object it just proved,
  and the slot's identity is written after the slot exists.
  Cons:
  the words are this design's,
  and a later step that adds a write adds a word.
- B,
  the states as the phases:
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
  and what the lock design's "A failed add" does too),
  and,
  once step 11 builds it,
  let the add's own fallback close the binding and release the claim at A26 and A27:
  the project's answer of 2026-11-22 gives the add that finish,
  and the restoring design names this record's allowance as its requirement.
  Step 10,
  which builds none of A20 to A27,
  does exactly this and no more:
  it keeps,
  reports,
  and lets the next add resume (W6 and W12 of the fourth check).
  Example:
  the snapshot returns "no space";
  `git worktree add .claude/worktrees/one` prints the operation,
  object and pool,
  and a later add at `one` resumes;
  `doctor` shows "registration without a dataset".
  Once step 11 lands,
  a failure at any phase from `registered` up enters the fallback instead (whatever the source's dirtiness):
  it destroys what it made,
  runs Git's own checkout,
  and its record and claim are gone when it returns.
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
Which of this record's three files goes first,
and what condition a finisher meets.

- A,
  the inventories first and the binding last,
  so the binding stands while its step 12 is unfinished (adopted).
  Example:
  a kill between the origin inventory's removal and the clone inventory's leaves the binding and one inventory,
  and a finisher repeats the step and then removes the plan.
  Pros:
  every kill inside the step leaves a binding the finisher recognises;
  an orphan inventory with no binding cannot arise from an ordinary kill;
  the plan stays the last witness the origin design's list makes it.
  Cons:
  a finisher must treat a standing inventory with no binding as the same step's leftover.
- B,
  the binding first,
  then the inventories.
  Example:
  the same kill leaves two inventories and no binding,
  and the next add refuses at A2 because an inventory's second publication answers `EEXIST`,
  with nothing naming the dead removal.
  Pros:
  the binding is gone as early as possible.
  Cons:
  it creates exactly the orphan-inventory refusal Z3 names.
- Ranking:
  A,
  then B.
  A beats B because it keeps the binding as the step's witness and leaves no state nothing names.

The tail's order across the three records is not this entry's to weigh:
the origin cleanup design's V13 decides it (the binding's files,
then the plan,
then the claim),
its list places the steps,
and this design cites that list and states no order of its own.
The finisher's condition is not a choice either;
it is the two arms "The condition a finisher meets" states,
whose first arm is the plan's:
a standing plan whose `head` is `gone` with origin fields equal to this binding's,
or,
with no plan,
the origin record with equal fields.
The origin record's absence is not a failure when a plan stands.

V21.
The settling command's name and how it asks for confirmation.

- A,
  the adopted form:
  `git monochromatic-git-policy-cli settle-worktree-clone-record <slot> --confirm-removal-of <slot>
  --confirm-clone-guid <the last four decimal digits of cloneGuid=>`,
  and the command a refusal or `doctor` prints shows both values as placeholders.
  Example:
  `git monochromatic-git-policy-cli settle-worktree-clone-record add-cache-layer --confirm-removal-of
  add-cache-layer --confirm-clone-guid 3102`,
  where `3102` was read from the printed `cloneGuid=` line.
  Pros:
  a mistyped slot,
  a slot typed for another record and a blind paste each refuse;
  no interactive prompt,
  so it works in a script and without a terminal.
  Cons:
  two flags to read and type,
  and a scripted caller that supplies all three cannot be told from a person.
- B,
  the same slot twice and no GUID:
  the option the earlier revision adopted.
  Example:
  as A without its last flag.
  Pros:
  one fewer value to type.
  Cons:
  the slot's own name is the only datum,
  and a paste of the printed line confirms without reading anything.
- C,
  an interactive confirmation that types the GUID.
  Example:
  the command prints `cloneGuid=4821193048571903102` and waits for it.
  Pros:
  the strongest single confirmation.
  Cons:
  not scriptable,
  and a 19-digit number is easy to mistype.
- D,
  a `--settle` flag on `doctor`.
  Example:
  `git monochromatic-git-policy-cli doctor --settle add-cache-layer`.
  Pros:
  one command a person already runs.
  Cons:
  plan step 18 makes `doctor` read-only,
  and a read-only report that deletes is the one thing its users trust least.
- Ranking:
  A,
  then B,
  then C,
  then D.
  A beats B because the GUID is a datum the slot name does not supply,
  so a typo that finds another record still refuses.
  B beats C because a person's terminal may not be there,
  and the wrapper's commands are non-interactive.
  C beats D because a typed confirmation beats a write hidden in a report.

V22.
Where the lost lock's installer is built,
and what step 10 does before it exists.

- A,
  the installer stays in the lock design's step 13 (adopted,
  and what that design and the decision of 2026-11-22 say),
  and step 10 refuses an unlocked complete entry,
  naming it and printing the sentence the lock must hold,
  until step 13 lands.
  Example:
  a resume meets a complete registration whose lock a person removed:
  the add stops,
  prints the entry and the sentence,
  and a later add after step 13 adopts it and the installer writes the lock again with one line.
  Pros:
  one owner for the install,
  and step 10 does not wait for step 13.
  Cons:
  an add at that slot waits for a person or for step 13.
- B,
  move the installer into step 10.
  Example:
  the same resume adopts the entry and writes the lock itself.
  Pros:
  no waiting.
  Cons:
  it splits the install from the classification and the doctor lines that step 13 builds,
  against the decision's own "step 10 does not wait for it".
- Ranking:
  A,
  then B.
  A beats B because the decision names step 13's build as the installer's and step 10 as not waiting,
  and B would give one write two owners.

V23.
How far the settling command reaches `settling` bindings.

- A,
  the adopted reach,
  in two modes and one refusal,
  which "Which command is true for a state" states for every state and this entry does not restate:
  it takes a `settling` binding that holds a `cloneGuid=` and whose slot shows no checkout begun
  ("When a checkout has begun" is the one test),
  which is the state the project chose the command for;
  when a checkout has begun it removes the records only (the plan,
  the two inventories,
  the binding and the claim),
  leaves the registration and the slot,
  and names the worktree with the wrapper's ordinary removal command for it;
  and a `settling` binding with no `cloneGuid=` refuses and names the add's re-run.
  Example:
  a fallback was killed after A22,
  leaving `phase=settling` and the clone's GUID absent,
  and the pool refuses the listing;
  `git monochromatic-git-policy-cli settle-worktree-clone-record add-cache-layer --confirm-removal-of
  add-cache-layer --confirm-clone-guid <digits>` prints the refused listing as its evidence and removes the records
  after the person confirms.
  A second example:
  a fallback was killed after A25,
  so Git's checkout stands and the lock is gone;
  the same command prints the refused listing,
  removes only the records,
  and names the worktree and `git worktree remove <slot>`.
  In one sentence about the person's files:
  with no checkout begun their worktree's files were never written,
  so nothing of theirs is removed;
  with a checkout begun their files stay where they are and only the wrapper's records go.
  Pros:
  every state has one true command but a reused name,
  which is a refusal by design,
  and no worktree Git made is stripped of the registration that makes it Git's.
  Cons:
  with a pool that cannot be asked,
  the command can remove the records of a `settling` binding whose clone in fact stands;
  and it erases a fallback's evidence while leaving its worktree.
- B,
  refuse every `settling` binding and delete the unprovable state's bullet:
  a `settling` slot whose clone cannot be proved gone waits for the pool,
  and no command fits it.
  Example:
  the same killed fallback and refused pool,
  and the person is told the pool must answer.
  In one sentence about the person's files:
  nothing is removed and the slot stays blocked.
  Pros:
  the command never touches an add's own path.
  Cons:
  it leaves a blocked slot with no command,
  which is the state the project declined in its answer on such a slot.
- Ranking:
  A,
  then B.
  A beats B because the project's answer names the command for exactly the slot B leaves blocked,
  and the checkout test keeps A from stripping a worktree.
  A third option was weighed and is not offered:
  for a begun checkout,
  refuse and name `git worktree remove <slot>`.
  A removal refuses a `settling` binding whose clone is gone and names the settling command,
  so that option would send the person from one command to the other and back,
  and the slot would stay blocked until the removal's own rule changed (U17 of the seventh add-side read).

V24.
Whether an add refuses while a destruction plan stands for its clone.

- A,
  it refuses (adopted):
  it reads the plan folder,
  stops while a plan names its clone,
  names the plan and the command the plan design's "Which resume path a removal takes" gives for the plan's head
  word,
  and acts on nothing.
  Example:
  a person removes a pending worktree;
  the removal refuses at a deferred-origin check and leaves its plan beside the live clone;
  the person runs the add again at the slot.
  With this rule the add stops,
  prints the plan and the removal that ends it,
  and changes nothing;
  the person's next removal takes the plan design's head-present path and ends the plan.
  Where the plan's tuples are all `planned`,
  so that it witnesses nothing destroyed,
  the add's refusal names the proposal's cancel command as well,
  which the plan design's section 7 removes in that one state and returns the worktree to use,
  so a person who wants the worktree kept is not told only how to finish removing it (N3 of the eleventh).
  Pros:
  no plan is left with no clone,
  no binding and no command,
  which is what a kill or a second-check refusal would otherwise produce once the add finishes;
  and the plan is the record of what a refused removal had already destroyed.
  Cons:
  an add that could have finished stops for a plan,
  and the add becomes a reader of another design's folder,
  which that design's section records rather than denies,
  as its heading there no longer fits (K12 of the eleventh).
- B,
  it continues and closes its binding,
  as an earlier revision did.
  Example:
  the same scene;
  the add finishes its checkout,
  removes its binding and its claim,
  and the plan stands with no clone,
  no binding and no command,
  named on every `doctor` run and ended by nothing.
  Pros:
  the add never reads another design's folder,
  and no plan can block an add.
  Cons:
  it leaves a plan nothing can act on,
  which is the trace the ninth removal-side read's K5 found.
- Ranking:
  A,
  then B.
  A beats B because B leaves a record nobody can use and a folder every command then lists,
  while A costs one refusal that names the way forward.

## Questions only the project can answer

None is left.
The project answered this design's two while this round ran,
in `doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`
(recorded 2026-11-22T21:16:30-05:00),
and every other choice this design once asked is a writer's choice with a ranking in the veto list:

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
  is V19;
- a record a later release cannot read,
  once Q7,
  is decided,
  and "A record that cannot be read" states it;
- a slot whose recorded clone is gone and cannot be proved gone,
  once Q8,
  is decided,
  and "The settling command" designs it;
- the settling command's name and confirmation are V21,
  which the project's answer leaves to this design;
- whether an add refuses while a destruction plan stands for its clone is V24,
  which is this design's addition to the plan design's "Which resume path a removal takes"
  (K9 of the tenth removal-side read).

The lock design's two questions are answered by
`doc/decision/cli-git-zfs-add-time-git-lock-put-back-when-lost.md`,
and this design's A6 and "The record's end" step 4 follow that answer.

### A record that cannot be read

Decided 2026-11-22 in the same record,
"A record that cannot be read":
"Change nothing,
name the file,
and wait for a person."
The refusal this design already gave is the decision,
and it stays as "A resumed add" and "`doctor`" state it:
the file is named,
the line that stopped the read is named,
and what a person does is printed (restore the file from a copy,
or read it and delete it,
and run the command again),
because no command acts on such a record.
No reader moves such a file aside,
no reader drops it after one line,
and the settling command is not a way around it:
an unreadable record refuses there too.

## What this design does not cover

- The destruction plan's home and format,
  the origin cleanup record's,
  and the add-time Git lock's:
  each is its own sibling design,
  named under "Where the siblings disagree".
- The content proof of the clone's fill (the frozen index and tree comparison) and where it lives across a kill.
  A resumed add whose proof is gone refuses rather than comparing against nothing,
  because a comparison that has lost its reference is not a proof,
  and that refusal stands for a clone whose identity cannot be proven;
  where the clone's identity can be proven,
  the fallback at A20 settles and finishes instead,
  which is the accepted restoring design's own requirement on this record (its "Dependencies on the sibling
  designs");
  the proof's own home is the dirty-main restore design's,
  and this record carries none.
- The mount's own records:
  the intent property,
  the receipts and the readiness record are steps 7 and 8's.
- The claim's effect on the checkout's own conversion or migration,
  which uses the same primitive for a different item.
- Any root-side copy of the lineage identities,
  which the delegated route does not need (assumption A2).

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

This round's two diff checks listed four things still wrong here,
and this round fixed them,
each rechecked against the source or the sibling at the commit now cited:

- the lock design's quotations,
  refreshed from `a7902e293b` to `86f122c2b1`,
  and the statement that its "The order of the add's writes" has a step 5,
  which it no longer has;
- the `HEAD` fact,
  which is the scan's completeness list and not the lock design's F14;
- the "(A6)" and "(F2,
  A4)" collisions,
  now written as "assumption A6" and "step A4";
- the count of this record's name forms in the home section,
  which said five beside four.

The rows of "The states Git's source can leave" were read this round from one clone at `v2.56.1`:
`builtin/worktree.c`:273 to :300 and :495 to :604,
`worktree.c`:1105 to :1134,
`refs.c`:191 to :321 and :376,
`wrapper.c`:760 to :783,
and the two tips of the removal-side pair were read at `b4dc758a76` and `0973f2fc2a`.

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
read with `git show` from their branches and not adopted as decided:
"The add-time Git lock of a ZFS clone worktree"
at `7d0dcdc875` (branch `docs/planning-cli-git-zfs-add-time-git-lock-design`,
read this round for its "The moments of an add" D0 to D7 and the restated D6,
its "A failed add,
and the fallback when the clone route fails",
its "The finder,
and the lock's part in it",
"The record whose nonce the lock's proof rests on",
"How the wrapper's own removal releases the lock",
its F8 and F11 and its M2),
"Where a removal's persisted destruction plan lives,
and its format" at `1921adf393` (branch `docs/planning-cli-git-zfs-removal-destruction-plan-record-design`,
read this round for "What finds a plan",
the finish pass that takes the claim and finishes steps 12 and 13,
its order and kill list,
its fields' origin lines and its `slotName`,
and its choices 5 and 10 to 12)
and "The record that carries origin cleanup after a registration is settled"
at `8cc31f59a4` (branch `docs/planning-cli-git-zfs-origin-cleanup-record-design`,
read this round for "The removal's one order of writes and removals",
"What a kill at each point leaves,
and who finishes it",
"The add that fails and destroys what it made",
its home,
its V2,
V4,
V8,
V13 and V14 and its F21).
At `8cc31f59a4` the merged list of the removal's writes across the three records stands,
with fourteen steps:
its step 11 settles the registration,
its step 12 removes the binding's files,
its step 13 removes the plan,
and its step 14 releases the claim,
and this design cites it for the order and states none of its own.

The dirty-main restore design,
accepted 2026-11-22 and on `main`:
`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
read for its fallback (its "What happens when the proof fails or the restore cannot finish"),
its restore's exchange (its "The restore"),
its allocation (its "Which build step builds which part")
and its "Dependencies on the sibling designs";
this design cites it for what its own A10a,
A10b,
A11 and A20 to A27 take from it.

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
- 2026-11-22T19:07:27-05:00:
  removed the veto option the proposal's own sentences decide (a binding kept until the origin cleanup finishes)
  and corrected V8's statement about the origin cleanup record's staging rule.
- 2026-11-22T19:08:27-05:00:
  kept the add order's A-labels apart from the Assumptions section's,
  which share the same letters and are a different list.
- 2026-11-22T19:56:45-05:00:
  answered the two diff checks:
  R1 (the rows of "The states Git's source can leave",
  walked from `builtin/worktree.c`:495 to :604 and
  `worktree.c`:1105 to :1134,
  the recipe that removes the files before the lock and the directory after it,
  the slot's pointer among the names it removes,
  the failure path's leftover as row 7,
  and T20 to T20c),
  R2 (the finder's names,
  prefix,
  newline,
  relative `gitdir` and second read),
  R3 (the add refuses and the pass or the next removal finishes),
  R4 (A10a and A10b,
  D6's boundary,
  the lock design at `86f122c2b1`,
  the `HEAD` fact from its scan,
  "assumption" beside the assumption labels,
  and four name forms),
  Z1 ("The condition a finisher meets" and the pass as the finisher),
  Z3 (the inventories first and the binding last),
  and Z4 (every statement about the two removal-side records at `b4dc758a76` and `0973f2fc2a`).
- 2026-11-22T19:57:34-05:00:
  corrected the row and recipe line numbers to the source's own (`:526 to :530`,
  `:543 to :544`,
  `:555 to :566`,
  `:601 to :603`,
  and the sanitiser's ranges at `refs.c`).
- 2026-11-22T21:19:32-05:00:
  answered the two third checks and the project's two answers:
  X1 (the recipe empties `refs/` and `logs/` file by file,
  `logs/HEAD` first,
  and row 6 and T20 carry it),
  X2 (the scoped refusal,
  the definition of "owns",
  and the `topic`/`topic1` walk),
  X3 (A10b keeps the copy and A11 exchanges it,
  D6 is A10 alone),
  X5,
  X7 to X10 and X13 to X16,
  Y1 (A20 to A27,
  "The add's own fallback",
  and T29 to T33),
  Y2 (the origin list at `8cc31f59a4` and its fourteen numbers),
  Y5 ("Who may finish that step"),
  Y6 (the key rule),
  Y12 (the condition's second arm),
  Q7 as "A record that cannot be read",
  Q8 as "The settling command" with V21,
  and the lock answer in the decisions,
  A6 and "The record's end" step 4.
- 2026-11-22T22:00:32-05:00:
  answered the fourth check and the fallback record's "Three further answers":
  W1 (the settling command runs the recipe's first step on the slot's pointer,
  and T33 plants it),
  W2 (the guard is two spellings plus the clone GUID's last four digits,
  both masked in the printed line,
  with what it catches and cannot said),
  W3 (the removals follow the recipe and the lost lock is written again),
  W4 (the states after A20 each with their command,
  the busy-unmount line with its new clause,
  and A20's write order),
  W5 (which failures enter the fallback,
  `registered` included,
  and V16 agrees),
  W6 (the fallback at step 11,
  the settling command at step 16,
  and step 10's keep-report-resume),
  W11 to W18 and W21,
  the empty-lock rule,
  the two nothing-found rows,
  and V21 and V22 rewritten.
- 2026-11-22T22:01:05-05:00:
  reconciled the proof-gone refusal with the fallback the restoring design requires:
  the refusal stands only where the clone's identity cannot be proven,
  and elsewhere A20 settles and finishes,
  as that design's "Dependencies on the sibling designs" asks.
- 2026-11-22T22:01:18-05:00:
  broke that paragraph at its clauses for the linter.
- 2026-11-22T22:05:25-05:00:
  answered the fourth removal-side check:
  W2 (the settling command refuses only a plan whose `head` is `gone`,
  removes any other before the inventories and prints what it held,
  never removes or rewrites the origin record,
  and V23 weighed the kept plan's three options then,
  which the project decided on 2026-11-22,
  so V23 now asks the reach question),
  W3 (a refused or partial pool listing is printed and named,
  not a refusal),
  W4 (the phase is no precondition,
  the removal accepts a `settling` binding whose clone stands,
  A21's busy withdrawal and the pending clause,
  and `doctor`'s two lines),
  W7 (`doctor` skips a fallback's worktree,
  and a later removal takes the wrapper's own route),
  W9 (A23's anchor),
  W10 (the last old step numbers),
  W11 (the origin record's two writers),
  and W13 (the plan's current text and the bound's 43,027 against 34,797).
  Each was proved by the searches kept in
  `keep/fourth-removal-check-searches-2026-11-22.txt`.
- 2026-11-23T03:54:59-05:00:
  answered the two fifth reads:
  N1 (V20 restored with its two options and ranking,
  and the sentences and the list's introduction that cite it made true),
  U1 (one rule for an unlocked complete entry:
  refused and named until step 13),
  U2 (`git worktree remove <slot>` in the pending line and in `doctor`),
  U3 and N2 (the settling command takes a `settling` binding whose clone's absence cannot be proved,
  each state after A20 with one true command,
  and V23),
  N3 (a reused name refuses in both,
  the command's order in six steps,
  its refusal for an unreadable plan,
  and the kill after the plan),
  N13 and U5 (T33's arms,
  T29,
  T30,
  and T34 and T35),
  U4 and N5 (A23 recovers a `registered` entry's snapshot identities by its proposed name),
  W14 and the lows (the create's flags,
  the refusal's trigger,
  the empty lock's row,
  the decisions section's wording,
  O's 13,368,
  the lock and plan tips,
  the origin list's destroys,
  the D6 clause,
  the route claim),
  A27 (the accepted design's request:
  nothing printed there),
  and the accepted dirty-main design cited by its headings.
  Each was proved by the searches kept in `keep/fifth-reads-searches-2026-11-22.txt`.
- 2026-11-23T04:37:14-05:00:
  answered the two sixth reads:
  G1 and U13 (the settling command's reach in V23's option A,
  with the finished checkout's records-only mode and the no-`cloneGuid=` refusal,
  and the states after A20 walked again),
  G12 (both commands true when the clone stands),
  G9 ("cannot be proved" defined once),
  G2 (the rerun let through,
  and the printed lines that lead a person to it),
  G3 and U12 (T20b's arms and T20c's),
  G5 (one rule for what a removal names),
  U14 (the re-run names the command),
  G8 (the order cited to the origin design's V13),
  G4 and L6 (T29,
  T30,
  T33,
  the duplicated plan bullet,
  the tips and "The record's end"),
  L4 (A6's sentence),
  and L5 (A21 widened).
  Each was proved by the searches kept in `keep/sixth-reads-searches-2026-11-22.txt`.
- 2026-11-23T05:26:53-05:00 (`cc6e19b439`,
  the round's first,
  a `wip` commit):
  a work-in-progress save of the seventh reads' edits.
- 2026-11-23T05:28:00-05:00 (`5f483f3dd5`):
  finished the seventh reads' edits:
  H1 and U15 (one test,
  "When a checkout has begun",
  cited by the states list,
  read 4,
  "What it removes" and
  V23 A;
  the `index` and lock clauses gone),
  H2 (the printing rule's second clause,
  and the no-plan state),
  U16 (the proved-absence refusal only while the registration stands,
  and the printed lines),
  U17 (V23's third option removed with its reason,
  and the option named by its letter),
  H3 and U22 (T30's arms and the test map),
  H4 to H8 (doctor's lines and T35's arms,
  the rerun rules,
  `cloneGuid=` carved out,
  the removal's rule stated
  once,
  Q7 marked decided,
  the tips),
  U19 (no lost lock in the records-only mode),
  U20 (`--force` for a partial checkout),
  and U23 (the accepted design's quoted sentence and the replacement words in "Where the siblings disagree").
  Each was proved by the searches kept in `keep/seventh-reads-searches-2026-11-22.txt`.
- 2026-11-23T05:28:52-05:00 (`cc141f5133`):
  named the settling command's own kill state in the after-A20 list,
  which U16 of the seventh add-side read asked for.
- 2026-11-23T17:39:46-05:00 (`c0962bb28c`):
  answered the two eighth reads:
  U26 and J1 ("Which command is true for a state" states every state once,
  and the printing rule,
  the fallback's list,
  "A resumed add",
  "`doctor`",
  R3,
  "What a record whose clone is gone means" and the removal's rule cite it),
  J2 (the `doctor` removal line),
  J3 (T33's ten arms,
  T34's rule,
  T30's arms),
  J4 (the `cloneGuid=` carve-out in the add's stop and the removal's rule),
  J5 (`doctor`'s general bullet),
  J8 (one wording for the checkout test),
  U27 (a clone recovered at its proposed name from `snapshotted`,
  and A22's skip),
  U33 (the first entry's real time and the tips sentence),
  and item 7 (the copy of the removal sentence matches "The record's end" word for word).
  Each was proved by the searches kept in `keep/eighth-reads-searches-2026-11-23.txt`.
- 2026-11-23T17:40:16-05:00 (`5be6220648`):
  pointed the one list's citers at the right headings,
  the definition of "cannot be proved" and the sentence in "The add's own fallback" that names the section.
- 2026-11-23T18:06:31-05:00 (`246d302e24`)
  and 2026-11-23T18:06:59-05:00 (`7969d55b6b`):
  answered the two ninth reads:
  K5 and U34 (the false premise that a `settling` binding never has a plan is gone;
  "Which command is true for a state" now has a plan row for a binding of any phase,
  citing the plan design's "Which resume path a removal takes" and its finish pass instead of restating them,
  and the add refuses while a plan stands),
  K2 (the removal's rule keeps the scope "and a `settling` binding,
  and for which no plan stands to resume",
  and the copy offered to the accepted design carries the same scope),
  K1 (the coarse closing paragraph is gone,
  and `doctor`,
  R3 and the add's refusal cite the plan design for a standing plan),
  K3 (T35's begun-checkout arm,
  its `filled` arms,
  and T30's arm for a plan beside a `settling` binding),
  K4 and U35 (the `snapshotted` line,
  T29's sentence,
  A22's pointer,
  and the section's own name for the recovery),
  K6 (the 8- and 16-space continuations normalized),
  and U38 (the tips sentence names `db69df972a` and `ea271e9e1d`,
  and the live "read at" cites are refreshed).
- 2026-11-23T18:42:02-05:00 (`fa6617282c`):
  answered the two tenth reads:
  K7 and U40 (the settling command's proved-absence refusal applies only where no plan stands,
  so a `settling` binding beside a plan whose clone a person destroyed is taken by the command,
  whose read 6 prints and removes the plan;
  `doctor`'s `settling` line carries the same plan condition,
  and T33's twelfth arm and T30's plan arm plant it),
  K8 and U39 (`cloned` or `mounted` with a gone clone and no plan names the settling command,
  `registered` and `snapshotted` leave the row,
  and the fallback's entry list says a gone clone is not a failure of the add's own step),
  K9 and U41 (the five readers of the plan folder are named,
  including the add and `doctor`,
  "A resumed add" and A20's resume carry the plan condition,
  "Where the siblings disagree" records the plan design's sentence,
  and V24 puts the add's refusal to the project),
  K10 and U42 (T34's scope and its plan arm,
  T30's plain arm after A25,
  T32's plan arm,
  and the plan row's two refusals that name no command).
- 2026-11-23T18:47:09-05:00 (`522ebf225b`):
  the plan row names which command accepts a plan beside a `settling` binding,
  which K7 of the tenth asked for by name.
- The commits after this list's last entry record only a corrected path reference or a time,
  and are not listed,
  because a list cannot name a commit's time before that commit exists.
