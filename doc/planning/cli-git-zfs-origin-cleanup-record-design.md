# The record that carries origin cleanup after a registration is settled

## Status

Non-final,
unapproved design proposal by an agent.
Nothing in it is approved,
and only the project owner declares a design of this route final.
It designs the open item "The record that carries origin cleanup" of the accepted ZFS worktree design.
The implementation plan lists that item for step 16 as
"blocking for the settle-while-blocked acceptance item,
writer design work,
with the owner asked only if its home changes the settle decision"
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md:2823` to `:2827`).
**The home does not change the settle decision:**
the folder is the account's and the destroy authority is the account's,
so settling proceeds exactly as the accepted design says,
whether or not the record is still owed.
That answers the one condition the plan puts on an owner question about the home,
and no other part of this design asks the owner for the home.

This is the second round of this document.
Its first commit `3428988897` wrote it,
and a gate review from another vendor read it whole at tip `4daa6c66ce`
(`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review.md`).
That review called this document O and found sixteen findings,
two findings about the set of three record designs (X1 and X2),
fourteen points where the three record designs disagree,
and some wrong citations.
This round fixes every finding,
marks every elision in a quotation,
checks every citation against the place it names,
reads the two sibling documents at the commits named under "Sources read",
adds "Where this design and its siblings disagree" and "Revisions",
and changes no decision the project's records make.
"Revisions" lists each commit with its committer time and what it changed.

The review's condition on reaching the owner still stands and is now met from this side:
the moment this document fixes (X1) and the answer about the slot (X2) are stated here as a single order and a single
rule that the two sibling documents are being brought to,
and "Where this design and its siblings disagree" says where each one stood when this round read it.
The three should go to the owner together.

This is the third round.
The second gate pass read it at `987a3ba886`
(`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-second-pass.md`)
with the two sibling documents beside it,
and found every first finding fixed.
Its verdict on this document is "fit after the listed fixes",
and on the set of three "not yet",
because the three are not one order of writes (its X3),
each still states things about the others that the others no longer say (X4),
this document's record shape still differs from the other two without that option in its own list (N9),
and the three carry the owner's questions in three different forms.
This round fixes X3 with one merged order of every write and removal a removal makes,
which the other two rounds are told to cite;
corrects every statement about the two sibling documents at the commits named under "Sources read" (X4);
fixes N5 to N13;
turns the flush question into a veto entry with the corrected ZFS source reading (N7);
and replaces the five questions with the review's four.
Their wording is the review's.
Where a fix forced a change to text the second pass found sound,
the change is named in "Revisions":
the order section is rewritten because X3 asks for the merged list,
the proof's second step changes because of N5,
and the questions change because the review's set of four replaces this document's five.

This is the fourth round.
The diff check read it at `0973f2fc2a`
(`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-diff-check.md`)
with the two sibling documents and the three decision records of the evening,
and found the merged list the one order it asked for and the tail sound.
Its verdict on this document is "fit after the listed fixes",
and on the set of three "not yet",
for one short round of edits that it names and that are not owner questions.
This round:
corrects the promote guard to the origin snapshot's group (Z2),
with a pool-free control and a guest arm;
corrects every statement about the two sibling documents at the commits under "Sources read" (Z4);
fixes the step numbers and adds the missing kill points (Z6 and Z7);
takes the evening's three decisions into the merged list and the doctor lines
(the early mark look,
the second check,
the forced request,
and the add that destroys what it made);
and puts the four questions in the diff check's current wording.
Question one's scene uses a hold (Z9).
It also takes the diff check's small fixes (Z10).
Where a fix forced a change to text the check found sound,
the change is named in "Revisions".

This round wrote this document and nothing else.
It merged `main` into its branch with `git merge --no-ff`,
which brought the three decision records and nothing this document owns,
and then edited this document alone.

It edited no code,
no file under `doc/handover/` or `doc/decision/`,
no other planning document,
and nothing `file-enforcer` generates.
It ran no `pnpm install` in this worktree,
started no guest,
no container and no service manager,
ran no ZFS command that changes anything,
and never asked the system service manager to change or signal anything.

What it read:
the gate review whole;
the decision record whole;
the proposal's sections named under "Sources read";
the plan's steps 14 to 19 and its open-item tables;
the accepted slot-identity-store decision and its design;
the step 8 sweep answers;
the staging decision;
the start-probe decision's answer about a failed arm;
the measurement record's items 11,
13,
14,
17 and 18;
the specification's Doctor,
descriptor-mount,
recorded-identity,
intent,
receipt,
readiness and claim sections;
the credential rule;
the removal passages;
the code under `package/git-policy/cli/src/native/` and
`package/zfs/request-to-openzfs-kernel-device/src/` that this design reuses;
and the two sibling documents at the commits named below,
read through `git show`.
Outside this repository it read OpenZFS at tag `zfs-2.4.4` and Git at tag `v2.56.1`,
from shallow clones in this round's scratch folder,
and it cites file and line at those tags.

What it ran:
read-only ZFS probes on this host,
a Git slot-reuse probe in a throwaway repository whose exit codes and output are kept,
and the repository's Markdown linter on this document.
"What this round ran" states each probe,
its command and its output.
It started no guest,
so every claim about kernel,
pool or mount behaviour that the repository has not measured is marked as source reading or as unmeasured,
and "Measurements owed" carries the probe that would measure it.

Quotations here keep the words of the quoted record.
A quotation's line breaks follow the repository's prose rules,
which ask for a break after every clause,
so a quotation broken across lines differs from its source in line breaks only.
Where a quotation drops words from its source,
it marks the gap with `[...]`,
and every citation names the exact range it quotes.

## The open item and the words that bind it

The item states what has no design
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:1738` to `:1742`):

"The record that carries origin cleanup.
The answer's words:
"Settle + own record":
the Git registration is settled once the clone is destroyed,
and a separate durable record carries the origin cleanup.
That record has no home,
no writer,
no fields and no state machine,
and nothing says what a settled registration means while the record still holds an origin to destroy."

The answer itself,
which names the moment in its own words
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:1460` to `:1466`):

"Decided:
"Settle + own record",
against the ranking the question round carried,
which put keeping the registration first:
the registration is settled once the clone is destroyed,
and a separate durable record carries the origin cleanup.
Settling frees the slot but requires recovery without its Git entry,
and that record has no design or home yet `[...]`
If clone lineage itself is unproved,
removal refuses and keeps the registration."

What settling releases,
and what it does not wait for
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:466` to `:470`):

"A verified wrapper lock is released when the registration is settled,
in the authorized registration-cleanup step,
and only after clone destruction is proven.
The settle-and-own answer separates that settlement from a blocked origin cleanup:
settling does not wait for the origin,
and the separate durable record carries that cleanup".

Who may destroy the origin
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:1409` to `:1413`):

"Decided:
"Account grant",
against the ranking the question round carried,
which put root-owned cleanup first:
descendants-only `mount,destroy` on the checkout.
Delegation keeps privilege out of the handler but allows the account to destroy any checkout snapshot,
sanoid's included."

What that authority must prove
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:1414` to `:1415`):

"[VM item 11][vm-promote] showed why both authorities must prove origin identity and lineage before submission;
a dependency refusal does not protect other snapshots."

The removal step this record ends
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:947` to `:951`):

"For separate origin cleanup,
prove the exact recorded origin dataset and snapshot GUIDs and confirm that the snapshot still belongs to that dataset.
Apply the same identity rule using the delegated authority the project decided `[...]`
The kernel's `snapshot has dependent clones` refusal is the dependency check,
not a scan of children or a name test."

And what that step leaves while the origin stands
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:952` to `:953`):

"Leave a refused origin and report it;
never destroy a dependent clone to free it.
Preserve the separate durable record,
which has no design yet,
while an origin remains."

The settle that follows it
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:954`):

"Settle only the matching Git registration and pointer after clone destruction is proven."

What the settle carries,
at `doc/planning/cli-git-zfs-worktree-design-proposal.md:955`:

"The project decided to settle them even while origin cleanup is blocked"

Which triggers the project allows,
in the owner's words
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:1393` to `:1396`):

"Triggers only,
and we control the triggers,
which makes the con of "unavailable until next trigger" moot.
The triggers the design installs or writes are the boot unit's start reconcile,
the verified checkout provider's post-key-load readiness hook,
and the clone-local sweep nudge the wrapper writes on a caller's behalf `[...]`"

Only the main checkout is an origin
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:1381` to `:1384`):

"Decided:
"Main checkout only (Recommended)".
A linked origin would preserve that worktree's ignored state but chains dependencies and can block origin cleanup.
`[...]` Neither route makes promotion automatic."

And the rule that follows it
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:1385`):

"Do not substitute a different origin from the one the caller supplied."

What the decision record keeps out of the root handler
(`doc/decision/cli-git-zfs-worktree-design.md:32` to `:33`):

"The root-installed ZED handler and its lifecycle unit own clone mount and unmount,
not Git or automatic deletion."

What the same record means by destroying at once
(`doc/decision/cli-git-zfs-worktree-design.md:36` to `:37`):

"A normal authorized removal destroys the clone and its own snapshots in that invocation",
and the answer that sentence carries is the one recorded as destroying at once.

The threat model is the project's own sentence,
and the record reads it so
(`doc/decision/cli-git-zfs-staging-under-cache.md:224` to `:226`):

"In ~/.cache .
We have to assume my machine isn't compromised.
sudo is passwordless here anyway."

And the record states what that model is for this machine
(`doc/decision/cli-git-zfs-staging-under-cache.md:230` to `:232`):

"The threat model that answer states is the project's own for this machine:
the account is trusted,
and root and the account are not a security boundary here."

No configuration decides whether this route works
(`doc/decision/cli-git-zfs-step-8-sweep-choices.md:19`):

"The project first answered that everything must work by default with no configuration"

And the case with no file at all is answered
(`doc/decision/cli-git-zfs-step-8-sweep-choices.md:44` to `:45`):

"The project answered "1:
no file needed (Recommended)"."

The wrapper never asks a person for a secret
(`package/git-policy/cli/SPEC.md:7345`):

"No command the wrapper runs or forwards asks a person for a credential",

under the owner decision at `package/git-policy/cli/SPEC.md:7348`:

"Avoid ever asking for someone to type a username or password.",

with `package/git-policy/cli/SPEC.md:7350`:

"Passphrases count too (Recommended)".

## What the item asks, and where this design answers it

The item has eight parts.
Each is answered in the section named beside it.

1.  The home,
    its path,
    owner,
    mode,
    name grammar and key,
    given that the Git registration is gone once the record matters:
    "The home,
    its name grammar and its key".
2.  The writer and the moment it writes,
    so that a kill leaves either the registration or the record:
    "The removal's one order of writes and removals",
    with its kill list.
3.  The fields with their types and encodings,
    and what proves the origin's identity and lineage later:
    "The fields and what proves the origin later".
4.  The state machine,
    its states,
    what moves them,
    what blocks them and what ends it:
    "The state machine".
5.  What a settled registration means while the record still holds an origin,
    and whether a new add may take the same slot:
    "A settled registration and a pending record".
6.  Who carries the cleanup later,
    on which event,
    and what happens when the event never comes:
    "Who carries the cleanup and when",
    with the commands that may destroy a leftover snapshot decided by the project's seven answers.
7.  What `doctor` reports,
    and what a person can do about a blocked record:
    "What doctor reports and what a person can do".
8.  The tests a build must have,
    and which need a guest:
    "The tests a build must have".

"Which build step builds which part" assigns every part to a plan step.
"Measurements owed" carries every number this design refuses to invent.

## Sources read

The repository's records,
each whole unless a line range is given:

- `doc/decision/cli-git-zfs-worktree-design.md`,
  whole.
- `doc/decision/cli-git-zfs-start-probe.md`,
  the decision and its Q2 answer (`:183` to `:193`).
- `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
  sections "Add and Git registration" (`:340` to `:435`),
  the passages about unmounted,
  pruned or moved worktrees (`:438` to `:483`),
  "Removal and recovery" (`:796` to `:1025`),
  "Project decisions" (`:1260` to `:1470`),
  "Open items the decisions leave" (`:1673` to `:1787`),
  and "VM measurements preceding acceptance" (`:1788` to `:1830`).
- `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
  "Step 14" to "Step 19" (`:1780` to `:2160`),
  "Open items and the step that needs each" (`:2770` to `:2856`),
  and "Gaps the design's open-item list does not name" (`:2858` to `:2895`).
- `doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  whole.
- `doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
  `:17` to `:68`.
- `doc/decision/cli-git-zfs-staging-under-cache.md`,
  `:200` to `:260`.
- `doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
  whole,
  the seven answers this round builds to.
- `doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
  its "Four further answers" and "One more answer".
- `doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
  its "Three further answers" and "A further answer".
- `doc/decision/cli-git-zfs-add-time-git-lock-put-back-when-lost.md`,
  whole.
- `doc/planning/cli-git-zfs-staging-under-cache-design.md`,
  `:1` to `:120`.
- `doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
  its Status,
  Built,
  Precedent and first section.
- `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
  items 11,
  13,
  14,
  17 and 18.

The sibling documents,
each read at the commit named with `git show <branch>:<path>` and none read as decided:

- `doc/planning/cli-git-zfs-add-binding-record-design.md`,
  branch `docs/planning-cli-git-zfs-add-binding-record-design`,
  commit `7d041d773b`.
- `doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
  branch `docs/planning-cli-git-zfs-removal-destruction-plan-record-design`,
  commit `1921adf393`.
- `doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
  branch `docs/planning-cli-git-zfs-dirty-main-restore-on-rung-1-design`,
  commit `04ce55a880`,
  its fallback's settle,
  its settle marker and its resume (`:1735` to `:1800`,
  `:2085` to `:2145`),
  and its dependency on this design.

The review:

- `~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review.md`,
  whole,
  which read this document at `4daa6c66ce`.
- `~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-second-pass.md`,
  whole,
  which read this document at `987a3ba886`,
  the two sibling documents at `76849b8d9b` and `428040d45d`,
  and OpenZFS at `zfs-2.4.4`.
- `~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-diff-check.md`,
  whole,
  which read this document at `0973f2fc2a`,
  the two sibling documents at `b4dc758a76` and `c144499eb9`,
  and the three decision records of the evening.
- `~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-third-check.md`,
  whole,
  which read this document at `8cc31f59a4`,
  the plan design at `1921adf393`,
  the binding design at `ec2b0b14f3`,
  the dirty-main design at `4dadc5fd27`,
  and the seven answers and the later decisions.
- `~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-fourth-check.md`,
  whole,
  which read this document at `22be706b09`,
  the binding design at `07858070bb`,
  the destruction plan design at `3cd520b6a0`,
  the dirty-main design at `04ce55a880`,
  `fb92dda66d` and `5237948862`,
  and the dirty-main record's "Three further answers".

The specification's sections
(`package/git-policy/cli/SPEC.md`):

- "Doctor" (`:683` to `:770`).
- The removal passages (`:2932` to `:2943` and `:3064` to `:3190`).
- "The descriptor mount of a ZFS worktree clone" (`:5408` to `:5560`),
  with "The recorded identity" and "The recorded identities store".
- The passages about the clone's mount intent,
  its receipts and its readiness (`:6072` to `:6300`),
  with "The receipts" and "The per-slot claim".
- The credential rule (`:7340` to `:7355`).

The code this design reuses,
all under `package/git-policy/cli/src/native/`:

- `worktree_clone_mount/slot_identity_store.rs`,
  `slot_identity_store_publication.rs`,
  `slot_identity_store_reading.rs`,
  `slot_identity_store_records.rs`,
  `slot_identity_store_control.rs`,
  `slot_name.rs`
  and `foundation/publishing_entries_in_one_open_folder.rs`.
- `worktree_clone_mount_handler/per_slot_root_claim.rs`
  and `sweep_clone_check.rs`.
- `lock/single_owner_claim.rs`.
- `foundation/private_storage.rs`,
  `private_storage_replacing_a_record.rs`,
  `plain_file.rs`,
  `path_bytes_in_records.rs`
  and `json_record.rs`.
- `worktree/worktree_removal_leaving_that_outlives_its_record.rs`
  and `worktree/worktree_removal_leavings_a_command_queues_again.rs`.
- `worktree_copying/worktree_copy_stage_claim.rs`,
  `worktree_copy_journal.rs`
  and `worktree_copy_stage_claim_reaping.rs`.
- `health/doctor.rs`,
  `health/doctor_filesystem.rs`
  and `conflict/discarded_report.rs`.
- `worktree_clone_mount_protocol/dataset_guid.rs`
  and `worktree_clone_mount_protocol/clone_mount_receipts_folder_refusals.rs`.

The ZFS binding,
under `package/zfs/request-to-openzfs-kernel-device/src/`:

- `kernel_device/read_requests.rs`,
  `kernel_device/write_requests.rs`,
  `kernel_device/listed_object.rs`,
  `requests/snapshot_requests.rs`
  and `layout/zfs_cmd_t.rs`.

Outside this repository,
from shallow clones at the tags named,
with each clone's push address set to `DISABLED` before anything else ran:

- OpenZFS at `zfs-2.4.4`:
  `module/zfs/zfs_ioctl.c`,
  `module/zfs/dsl_destroy.c`,
  `module/zfs/dsl_dataset.c`,
  `module/zfs/dsl_deleg.c`,
  `module/zfs/dsl_synctask.c`,
  `module/zfs/zcp.c`,
  `module/zfs/zil.c`,
  `module/zfs/zfs_log.c`,
  `module/zfs/zfs_vnops.c`,
  `man/man8/zfs-promote.8`,
  `module/zcommon/zfs_prop.c`,
  `include/sys/fs/zfs.h`,
  `include/sys/dmu.h`,
  `lib/libzfs/libzfs_dataset.c`,
  `cmd/zfs/zfs_main.c`,
  and `man/man8/zfs-destroy.8`.
- Git at `v2.56.1`:
  `builtin/worktree.c`
  and `worktree.c`.

## Precedent in this repository

This design invents no primitive.
Each step it orders is one this package already ships,
and its job is to pick among them and say why.

- The leaving record is the closest shape.
  It is `<common-git-dir>/monochromatic-git-policy-cli/leavings/<id>`,
  one file per subvolume a removal renamed aside,
  "written before the rename and settled by nothing"
  (`worktree/worktree_removal_leaving_that_outlives_its_record.rs:1` to `:25`).
  It holds short `key=value` lines followed by one JSON object
  (`:108` to `:128`),
  and it is published whole by one rename
  (`:132` to `:138`).
  Its look runs for every command of the repository,
  "so a repository in which no removal left anything pays one open that finds no folder"
  (`worktree/worktree_removal_leavings_a_command_queues_again.rs:82` to `:84`).
  Its reader proves what it can and leaves what it cannot,
  "because a person's directory must never be deleted for a leaving this program lost"
  (`:24` to `:27`).
  It destroys nothing:
  "Nothing here destroys a subvolume"
  (`:22` to `:24`).
- `foundation/private_storage.rs` writes a record whole or not at all.
  `staging_beside` makes a hidden sibling named `.<name>.<random>.partial` with mode `0600`
  (`:198` to `:210`),
  `replace_private_file` publishes it by one rename after a flush,
  `replace_private_file_unsynced` is the variant a command's own path uses,
  `create_private_directory_all` makes the folder at mode `0700`
  (`:546` to `:564`),
  and `read_regular_file` reads a plain file under a byte bound.
- `worktree_clone_mount/publishing_entries_in_one_open_folder.rs` is the folder held by descriptor:
  `create_new_entry` makes one entry with `O_EXCL`,
  `link_entry` gives it its record's name and replaces nothing,
  and `rename_entry_replacing` is the one rename that replaces.
- `foundation/path_bytes_in_records.rs` is how a path enters a text record without losing a byte:
  a text field when the path is text,
  and a hexadecimal field always (`:1` to `:20`).
- `foundation/json_record.rs` reads a JSON object the way the incumbent does.
  Its `MAX_SAFE_INTEGER` is why no 64-bit GUID may travel as a JSON number (`:36`).
- `lock/single_owner_claim.rs` is the wrapper's lock-free claim (`:1` to `:45`),
  and the root handler's per-slot claim builds on it
  (`worktree_clone_mount_handler/per_slot_root_claim.rs`).
- `worktree_clone_mount/slot_identity_store.rs` is the wrapper's root-owned record folder:
  one file per item,
  published with an exclusive creation and one link that replaces nothing,
  with the folder held by descriptor,
  and with the `doctor` role this design copies for its own folder
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`).
- The receipts rule is the shape of a record of state a different account wrote:
  a root-owned directory of mode `0755`,
  files of mode `0644`,
  names that are canonical numbers and never nonces,
  and a byte bound derived from the fields
  (`doc/planning/cli-git-zfs-worktree-design-proposal.md:568` to `:570`).
  This design takes the shape and not the root ownership,
  and the veto entry for the home says why.
- `conflict/discarded_report.rs` is how a `doctor` section reports a folder of records:
  one prefix,
  one line per finding,
  a ceiling on the read with a line for a read that did not finish,
  and no change to the exit status (`:190` to `:240`).
- The ZFS binding is how this route reads and writes OpenZFS without starting the `zfs` program.
  `objset_stats` answers a dataset's or a snapshot's statistics and its property values with their sources
  (`kernel_device/read_requests.rs:45` to `:62`),
  `snapshot_list_next` walks a dataset's snapshots one call at a time
  (`:83` to `:96`),
  `destroy_snaps` submits one explicit name set and sends no `defer` key
  (`kernel_device/write_requests.rs:93` to `:104`,
  `requests/snapshot_requests.rs:68` to `:75`),
  and `DatasetGuid` is the canonical numeric spelling a GUID enters any record with
  (`worktree_clone_mount_protocol/dataset_guid.rs:39` to `:60`).

Two precedents are not reused,
and each place that matters says so.
The staging design's `~/.cache` home is not reused,
because a cache is a folder a system may drop,
and this record is the only memory of a snapshot to destroy.
The root-owned store is not reused either,
because after the clone is destroyed there is no root-side request channel to write through,
and the authority to destroy is the account's.

## Facts

Each fact is something this round read at the tag it names,
or measured on this host.
A fact's number is its handle in the rest of this document.

F1 The ZFS binding offers the reads and the write this design needs,
with no `zfs` program.
`objset_stats` is `ZFS_IOC_OBJSET_STATS`
(`requests/read_requests.rs:34` to `:44`)
and answers a dataset's or a snapshot's statistics and its property values with their sources
(`kernel_device/read_requests.rs:45` to `:62`).
`snapshot_list_next` walks one dataset's snapshots one call at a time,
answering the name the kernel wrote,
the cursor,
the statistics and the properties
(`:83` to `:96`).
The statistics carry `dds_guid`,
`dds_creation_txg`,
`dds_num_clones`,
`dds_is_snapshot` and `dds_origin`
(`layout/zfs_cmd_t.rs:73` to `:107`),
and the kernel fills the clone count only for a snapshot,
leaving it zero for a dataset
(`module/zfs/dsl_dataset.c:2881` to `:2899`).
F2 The property list those two reads return is `dsl_prop_get_all`'s
(`module/zfs/zfs_ioctl.c:2244` to `:2253`,
OpenZFS `zfs-2.4.4`),
which adds `guid`,
`createtxg`,
`userrefs` and `defer_destroy` for a dataset or a snapshot
(`module/zfs/dsl_dataset.c:2842` to `:2857`).
F3 `defer_destroy` is a read-only property of snapshots only,
spelled `defer_destroy` and typed as a boolean index
(`module/zcommon/zfs_prop.c:544` to `:546`),
and `zfs destroy` uses `defer_destroy` and `userrefs`
"to determine eligibility and marked status"
(`man/man8/zfs-destroy.8:117` to `:123`).
F4 The kernel refuses a snapshot destroy with `EBUSY` while the snapshot has user references,
and with `EEXIST` when it has more than one child,
which is a clone
(`module/zfs/dsl_destroy.c:75` to `:86`).
F5 A destroy request that names a snapshot which does not exist is silently ignored,
and the kernel treats it as already destroyed
(`module/zfs/dsl_destroy.c:105` to `:109`).
Such names are also removed from the request before the destroy,
so a snapshot created in between is not destroyed unchecked
(`module/zfs/zfs_ioctl.c:855` to `:869`).
F6 A delegated snapshot destroy requires descendant `mount` and `destroy` permissions
(`module/zfs/zfs_ioctl.c:824` to `:845`),
and the check's own comment at `module/zfs/zfs_ioctl.c:842` to `:845` says
"Destroying snapshots with delegated permissions requires descendant mount and destroy permissions."
F7 A head dataset cannot be destroyed while it has snapshots:
the kernel refuses with `EBUSY` on the branch check
(`module/zfs/dsl_destroy.c:783` to `:790`)
and with `EEXIST` while it has child datasets (`:795` to `:800`).
The snapshot-name table is destroyed with the head
(`:1120` to `:1122`),
and destroying a clone's head can reap an origin snapshot that is already marked for deferred destroy
(`:1028` to `:1031`,
which sets `rmorigin`,
and `:1172`,
which destroys that snapshot).
F8 Dataset and snapshot GUIDs are drawn from the kernel's random source
(`module/zfs/dsl_dataset.c:1203` to `:1204` for a dataset,
`:1768` to `:1769` for a snapshot),
so two GUIDs are distinct in practice and not against a 64-bit collision.
F9 A dataset's full name is at most 255 bytes,
from `ZFS_MAX_DATASET_NAME_LEN` 256 including the terminator
(`include/sys/fs/zfs.h:88`).
F10 The `zfs` program refuses a snapshot with a dependent clone before it sends any ioctl.
Its pre-check runs before the gather
(`cmd/zfs/zfs_main.c:1605` to `:1616`),
and it prints
"snapshot has dependent clones"
with the snapshot named in the message
and with the dependent datasets' names
(`:1437` to `:1450`).
Measurements owed item 14 recorded that refusal with the ioctl count at 0
(`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md:560` to `:575`).
F11 The kernel's own per-name error path prints
"cannot destroy snapshot <name>"
with the auxiliary text
"snapshot is cloned"
for `EEXIST`,
and a holds message naming `zfs holds -r` for `EBUSY`
(`lib/libzfs/libzfs_dataset.c:3972` to `:3996`).
The two records' wordings differ,
so no design of this route may decide anything by parsing a message.
F12 `zfs destroy -n` skips the destroy entirely
(`cmd/zfs/zfs_main.c:1785` to `:1789`)
and is documented as a dry run that deletes no data
(`man/man8/zfs-destroy.8:74` to `:78`).
F13 Measured on this host.
The kept log is `keep/host-zfs-snapshot-facts.log`,
written at 2026-11-22T17:22:37-05:00 after the commands ran:
`zfs get -Hp -o name,property,value defer_destroy,userrefs,guid,createtxg`
on one snapshot of `zpcachyos/ROOT/cos/home` printed `defer_destroy off`,
`userrefs 0`,
and the snapshot's `guid` and `createtxg` as decimal numbers.
The same dataset held 25 own snapshots at that time,
so the absence proof on such a dataset is 25 listing calls in the worst case.
F14 Measured on this host.
The kept log is `keep/host-zfs-missing-dataset.log`,
written at 2026-11-22T17:22:37-05:00 after the commands ran:
`zfs get` and `zfs list` on a dataset name that does not exist printed
"dataset does not exist"
with the dataset named in the message,
and exited 1.
The kernel's snapshot listing answers `ESRCH` for a dataset it cannot hold,
which is the same answer as the end of a listing
(`module/zfs/zfs_ioctl.c:2503` to `:2506`).
F15 Measured on this host,
with Git 2.56.1 through the installed wrapper in a throwaway repository whose `HOME` and `XDG_RUNTIME_DIR`
are under this round's scratch folder.
The kept log is `keep/git-worktree-slot-reuse-exitcodes.log`,
written at 2026-11-22T18:28:25-05:00:
the add at a fresh slot exited 0,
the add at the same path with the registration standing exited 128 with
"'<path>' is a missing but already registered worktree;
use 'add -f' to override,
or 'prune' or 'remove' to clear",
`git worktree prune` exited 0,
and the add at the same path after the prune exited 0.
The registration's name was the slot folder's basename.
The message is Git's own
(`builtin/worktree.c:336` to `:339` at `v2.56.1`).
F16 The wrapper already makes one look per command at a folder of durable records,
and in a repository with no such record that look is
"one open that finds no folder"
(`worktree/worktree_removal_leavings_a_command_queues_again.rs:82` to `:84`).
The same module's record is written whole by one rename
(`worktree/worktree_removal_leaving_that_outlives_its_record.rs:132` to `:138`).
That look destroys nothing;
it queues a delete for the service to run (`:26` to `:28`).
F17 The wrapper's `doctor` reads a folder of records under a ceiling and prints one line per finding,
keeping the exit status
(`conflict/discarded_report.rs:190` to `:240`),
and its filesystem section is informational text on standard error after the hook report
(`health/doctor_filesystem.rs:1` to `:35`).
F18 The wrapper's claims are lock-free.
The shared claim is made whole in a private file and published with one exclusive `link`
under the next generation number,
so exactly one claimant wins and none waits
(`lock/single_owner_claim.rs:1` to `:45`),
and the root handler's per-slot claim builds on it.
The binding design takes a per-slot claim for an add and a removal
(its `:684` to `:732`).
F19 The closest record bound in this package is the leaving record's 16,384 bytes,
derived from three short lines and one work item that names three paths
(`worktree/worktree_removal_leaving_that_outlives_its_record.rs:63` to `:69`).
F20 A promote moves more than the snapshot it names.
`zfs-promote(8)` says
"The snapshot that was cloned,
and any snapshots previous to this snapshot,
are now owned by the promoted clone"
(`man/man8/zfs-promote.8:53` to `:54` at `zfs-2.4.4`),
and item 11 measured exactly that:
the checkout's origin snapshot and its older snapshot moved to the worktree with their GUIDs unchanged,
the checkout acquired the worktree's snapshot as its origin,
and the later checkout snapshot stayed
(`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md:378` to `:384`).
Item 13 repeated it and saw the same tuples after the move
(`:552` to `:555`).
Three consequences follow.
A promote of the clone that depends on the record's snapshot moves that snapshot under the promoted clone,
and the dataset the record names gains it as its own origin.
A promote of a later clone moves the later snapshot and every snapshot older than it,
which includes the record's snapshot,
and the dataset then gains the later snapshot as its origin,
whose name is not the record's.
Either way the recorded snapshot stays alive under another dataset,
and the dataset the record names carries an origin,
whose `createtxg` is at or above the recorded snapshot's.
F21 Source reading,
not measured:
the destroy-snapshots request runs a ZCP program in a sync task
(`module/zfs/dsl_destroy.c:656` to `:659`),
a sync task commits its transaction and then waits for that group
(`module/zfs/zcp.c:1160` to `:1162`
and `module/zfs/dsl_synctask.c:87` to `:101`),
and transaction groups sync in order.
A record written and published before the destroy is therefore on disk when the destroy returns,
if the Git directory and the pool are on one machine and one pool's commit paths are what the record's folder
inherits.
The premise is not stated by any of the three record designs (L8).
F22 Source reading,
not measured:
a snapshot's permission check treats the snapshot as a descendant only,
so local permissions do not apply
(`module/zfs/dsl_deleg.c:574` to `:579`),
and the walk starts at the snapshot's own dataset directory with the descendant flag
(`:588` to `:589`).
With `zfs_secpolicy_destroy_perms` requiring `mount` then `destroy` (F6),
a descendants-only `mount,destroy` grant on the checkout covers destroying the checkout's own snapshots.
F23 The store decision puts the removal of known staging names in plan step 18's `doctor`,
unconditionally,
for the store folder
(`doc/decision/cli-git-zfs-slot-identity-store-format.md:161` to `:177`).
That decision covers the store and does not decide this folder.
F24 The binding design,
read at `7d041d773b` in `doc/planning/cli-git-zfs-add-binding-record-design.md`,
puts its folder inside the wrapper's own folder,
at `<common>/monochromatic-git-policy-cli/worktree-clones/v1/` (`:1069`),
keys its files by the slot name (`:1127` to `:1210`),
writes ZFS names verbatim,
paths in hex and GUIDs in canonical decimal (`:1338` to `:1359`),
flushes every write that falls on the flushed side of the route's rule (`:1360` to `:1370`),
refuses a record whose owner or mode is not the account's,
and lets `doctor` remove a staging name only when the folder holds no live claim (`:1371` to `:1382`).
Its record's end accepts a binding at any phase whose clone still stands,
`phase=settling` included (`:3146` to `:3162`),
and its fifth step removes the two inventories first and `<slot>.binding` last (`:3182`).
Its "The condition a finisher meets" states the condition and quotes who may finish it:
"the plan design's finish pass,
which runs before every forwarded command,
or the next removal of the slot while a registration still stands;
never an add,
except the add's own fallback at A20 to A27,
and only where `phase=settling` says the add itself stopped mid-fallback" (`:3244` to `:3300`).
Its A20 to A27 are the add's own fallback (`:2092` to `:2207`):
`phase=settling` and `settleReason=` written before the first destroy,
the unmount,
the clone's destroy,
the snapshot's destroy with `ENOENT` counted as done,
and the origin record carrying the snapshot if that destroy is refused (A23 at `:2162` to `:2178`).
That is the record's moment for an add,
and A23 gives it as the record standing from before A22,
or before A23 when A22 is skipped (W9 of the fourth check).
the lock released with the registration kept,
Git's own checkout,
the two inventories and then the binding removed,
and the claim released last.
Its "The settling command" is the project's command (`:3347` to `:3480`):
it compares the origin cleanup record and never removes or rewrites it,
refuses a plan whose head is `gone`,
removes any other plan after the confirmation and prints what it held,
and refuses a binding at `phase=settling`.
Its byte bound is 43,027 (`:1690` to `:1735`).
F25 The plan design,
read at `1921adf393` in `doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
puts its folder at `<common>/monochromatic-git-policy-cli/clone-destruction-plans/<clone GUID>`
(`:648` to `:739` of that document at that commit),
keys the plan by the clone's dataset GUID (`:776` to `:844`),
writes a `#` header line then `key=value` lines with ZFS names verbatim,
paths in hex and numbers in decimal (`:846` to `:996`),
and takes the per-slot claim for a removal (`:572` to `:578`).
It cites this design's merged list at `8cc31f59a4` instead of stating an order of its own,
under that list's two headings and with its fourteen step numbers (`:1092` to `:1216`).
It publishes unflushed (`:1356` to `:1378`),
and its finish pass takes the claim,
proves the head's absence from the live pool under the route's gate,
settles the registration,
finishes this list's step 12 for the binding that names the plan's clone,
removes the plan at step 13 and releases the claim at step 14 (`:1829` to `:1944`).
Its bound is `10,128 + 340 * count`,
with memory bounded by the file's own length,
and no count ceiling;
its header's seven-digit count is a format width and not a policy (`:996` to `:1030`).
F26 The start-probe decision answers that a probe arm that ran and failed
"refuses new managed attaches,
and every other operation keeps its route"
(`doc/decision/cli-git-zfs-start-probe.md:183` to `:193`).
F27 Source reading,
not measured:
OpenZFS logs directory intents synchronously by default.
`zil_itx_create` sets `itx_sync = B_TRUE` as the default
(`module/zfs/zil.c:2561`),
and the create,
remove,
link,
symlink and rename log functions never clear it
(`module/zfs/zfs_log.c:299`,
`:413`,
`:448`,
`:471` and `:530`),
while only the write,
truncate,
setattr and ACL arms set it to the caller's choice
(`:685`,
`:722`,
`:784`,
`:816`,
`:882` and `:920`).
`zfs_fsync` calls `zil_commit` unless the dataset's `sync` is `disabled`
(`module/zfs/zfs_vnops.c:111` to `:123`),
and `zil_get_commit_list` moves each open transaction group's whole synchronous list onto the commit list
(`module/zfs/zil.c:2944` to `:2957`).
A flush of a file and its directory therefore makes a rename durable at `sync=standard`,
and the earlier claim that OpenZFS logs a rename only when `sync` is `always` is wrong.

### Where two records disagree on the refusal's wording

The proposal says
"The kernel's `snapshot has dependent clones` refusal is the dependency check"
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:950` to `:951`),
and the measurement record quotes that same sentence from a command's output
(`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md:573`).
At `zfs-2.4.4` that sentence is printed by the `zfs` program's own pre-check,
before any ioctl (F10),
and the kernel's answer to the same request is `EEXIST` with the text
"snapshot is cloned" (F11).

The difference matters to this design in one way and no other.
The dependency check is still the kernel's:
a request that names a snapshot with a clone is refused whatever the program printed,
and a clone of the snapshot is what the kernel counts (F4).
The wording is not a decision input,
which is why the attempt reads the errno and the live counts and never parses a message.
A build that read the `zfs` program's output instead of the binding's answer
would be deciding from the pre-check's text and not from the kernel's refusal,
and this design does not do that (F1).

## Assumptions

Each assumption is a thing this design needs that it did not verify,
and each names what would confirm or refute it.

A1 The add's durable binding record is readable by the removal before any destroy,
and it carries the origin dataset and snapshot identities that the removal copies into this record.
The binding design keys that record by the slot name (F24),
so the removal reads it by the slot as the binding design's own reader does.
If the binding record is unreadable at removal time,
the removal refuses before its first delete,
which is the accepted identity rule and not a new one (the proposal `:947`).
A2 The account's delegated `mount,destroy` on the checkout covers destroying the origin snapshot,
as the decision says and as the kernel's descendant rule expects (F6 and F22).
M4 is the guest check of the grants the wrapper really sets.
A3 The removal's step 15 check of the origin's deferred-destroy mark is available to this design's attempt as a read,
because the sibling design of the deferred-destroy checks owns how that read is made.
Until it lands,
this attempt refuses a snapshot whose `defer_destroy` is on,
and treats an unreadable value as a refusal (F3).
A4 Every worktree of one repository can compute the repository's common Git directory,
which the wrapper already asks Git for
(`health/doctor.rs:287` to `:290`).
A5 A folder listing of a folder with no entries costs one open and one read,
which F16 has for the leavings folder and which this design assumes for its own folder.
M1 sizes it.
A6 The `zfs` program's behaviour in F10,
F11 and F12 stands as a description of the same kernel that the binding reaches.
This design decides nothing by that wording (F11),
and reads every value through the binding (F1).
A7 The record's folder and the pool share one machine and one durability path,
which F21's ordering argument needs (L8).
A8 A snapshot's absence can be proved only by reading its dataset,
so a record whose dataset is gone stays `unproved` and waits for a person.
That is this design's reading of the accepted "never from a missing name alone",
and O1 in the review is the reason it is stated.

## Limitations

L1 The identity proof rests on 64-bit random GUIDs (F8),
so it excludes a collision in practice and not in theory.
L2 `createtxg` order is not proof of provenance for other topologies,
encryption roots or objects created in one transaction group,
as the proposal's step 1 says (`doc/planning/cli-git-zfs-worktree-design-proposal.md:879`).
This record copies the value;
it does not re-derive lineage.
L3 The account owns the folder,
so the account can rewrite or delete a record.
Under the stated threat model the account is trusted,
and the same authority can destroy the snapshot directly,
so root ownership would buy nothing this design can use.
L4 Deleting a repository's Git directory deletes its pending records with it,
and nothing in this design finds them again.
The origin snapshot then stays until a person or another program destroys it,
and the unaccounted-snapshot line of `doctor` is the only sight of it.
L5 A record's folder is found through the repository,
so a repository moved to a new path keeps its records and a repository whose path vanished does not.
L6 No timer exists (the trigger answer at the proposal `:1393`),
so a repository in which no command runs keeps its record and its snapshot indefinitely.
L7 The record names the dependent clone only by count,
because the binding exposes no dependents iteration.
`doctor` prints the dry-run command that names them (F12).
L8 The durability rule this design is written for is "V14 The flush rule"'s option C,
and it assumes one machine and one pool:
the record's folder is on the pool whose destroy waits for the transaction group (F21).
A Git directory on another pool,
a network file system or a different machine breaks that argument,
and the record then needs the flush that V14's option A buys.
L9 The record's bound is derived from the field widths and the JSON escape rule
in "The bound".
It is a bound on what the reader accepts,
not a measured write size.
L10 A same-account process can substitute objects between the identity read and the destroy.
That is the accepted residual of the delegated answer
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:1416` to `:1421`),
and this design narrows nothing about it.
L11 A promote leaves the record unprovable rather than complete (F20).
The snapshot lives on under the promoted dataset,
the named dataset gains an origin at or above the recorded snapshot's group,
and the record stays `unproved` until a person decides.
A named dataset whose origin is older than the record,
as a checkout cloned from a backup has,
does not block the proof (Z2).
The guard is the group and not the name,
because a promote of a later clone moves the recorded snapshot under an origin named for the later one.
Nothing here follows the snapshot to its new dataset:
doing so would need a lineage proof the proposal does not design.
L12 A record whose dataset was destroyed outside the wrapper is also `unproved` (A8),
because the snapshot may have moved or the dataset name may now name another dataset.
L13 The look adds one folder open and,
when a record stands,
a few reads on a worktree command's path.
No ceiling is shipped until M1 measures it,
and every such read is a request to a pool the command already uses.
L14 The folder stays after its last record (O10 in the review),
so a worktree command in a repository that once cleaned up pays one open and one listing of an empty folder,
where F16's leavings precedent pays one open of a folder that is not there.

## What this round ran

Three probes.
Their logs are under
`~/temp/agent/zfs-origin-cleanup-record-design-2026-11-22/keep/`,
and each states its own time.

### P1 The snapshot properties the identity proof reads

Commands,
run on this host as the account,
with no `sudo` and no change to anything.
Its kept log was written at 2026-11-22T17:22:37-05:00:

```sh
zfs version
zfs get -Hp -o name,property,value defer_destroy,userrefs,guid,createtxg 'zpcachyos/ROOT/cos/home@autosnap_2026-11-21_22:00:00_hourly'
zfs list -Hp -t snapshot -d 1 -o name,guid,createtxg 'zpcachyos/ROOT/cos/home'
zfs get -Hp -o name,property,value guid 'zpcachyos/ROOT/cos/home/definitely-not-here'
```

Selected output:

```text
zfs-2.4.4-1
zfs-kmod-2.4.4-1
zpcachyos/ROOT/cos/home@autosnap_2026-11-21_22:00:00_hourly	defer_destroy	off
zpcachyos/ROOT/cos/home@autosnap_2026-11-21_22:00:00_hourly	userrefs	0
zpcachyos/ROOT/cos/home@autosnap_2026-11-21_22:00:00_hourly	guid	3745620968172523788
zpcachyos/ROOT/cos/home@autosnap_2026-11-21_22:00:00_hourly	createtxg	93823
cannot open 'zpcachyos/ROOT/cos/home/definitely-not-here': dataset does not exist
```

What it showed:
the four properties this design compares are readable as plain values on this host,
a deferral mark reads `off` when it is absent,
and a dataset that does not exist is an error and not an empty listing (F13 and F14).
The host's dataset held 25 own snapshots,
which is the observed size of the absence proof on a sanoid-pruned checkout (F13).

### P2 A slot after its registration and without its folder

Commands,
inside a throwaway repository whose `HOME` and `XDG_RUNTIME_DIR` are under this round's scratch folder.
Its kept log was written at 2026-11-22T18:28:25-05:00:

```sh
git init --quiet repo4
git worktree add ../wt4
rm -rf /home/user/temp/agent/zfs-origin-cleanup-record-design-2026-11-22/scratch/git-probe/wt4
git worktree add ../wt4
git worktree prune --verbose
git worktree add ../wt4
```

Output,
with each step's exit code:

```text
### B: add at a fresh slot
exit=0

### D: add at the same path with the registration standing
exit=128
fatal: '../wt4' is a missing but already registered worktree;
use 'add -f' to override, or 'prune' or 'remove' to clear

### E: prune
exit=0

### F: add at the same path after the prune
exit=0
```

What it showed:
a missing registration blocks a new add at the same slot until it is settled,
the settle that clears it is Git's own `prune`,
and the registration's name is reused (F15).

### P3 The linter

Command,
in this worktree:

```sh
/home/user/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-lint/0.5.2-alpha.8/monochromatic-lint-0.5.2-alpha.8-x86_64-unknown-linux-gnu/monochromatic-lint doc/planning/cli-git-zfs-origin-cleanup-record-design.md
```

What it showed:
the rule set enforces a line break after every prose clause,
so this document is written one clause per line.
A control file in this round's scratch folder,
written with a pipe table and with clauses left unbroken,
made the same run report both rules,
so the empty result on this document is a result and not a silence.
The final run reports nothing,
and "Revisions" records when.

### P4 The citation check

Every quotation and every path-and-line citation in this document was checked by a scratch script that printed
each cited range beside the words the document quotes.
"Revisions" says how many the document holds and what the check corrected.

## The removal's one order of writes and removals

A removal of a ZFS clone worktree writes or removes four things besides the clone itself:
the Git registration,
this design's origin cleanup record,
the add's binding and its two inventories,
and the plan design's destruction plan.
It also holds the per-slot claim from its first step to its last.
The three record designs must agree on one order,
because a kill between two of them decides what a later run reads,
and this design is the one that holds the order,
so the list is here.

This is the list the two sibling rounds are told to cite,
and neither states an order of its own.
Steps 1 to 9 are the accepted design's or a sibling's,
step 10 is this design's,
step 11 is the accepted design's,
and steps 12 to 14 are the three designs' tail,
whose order "V13 The tail's order" marks for the project's veto.
The only mapping this list needs is to the binding design's own five-step list:
this list's step 1 is its step 1,
its step 6 is the binding's step 2,
its steps 7 to 9 are the binding's step 3,
its step 11 is the binding's step 4,
and its steps 12 and 14 are the two halves of the binding's step 5.
This list's step 10 is the proposal's step 4,
and its step 8 is the second deferred-destroy check the evening's decision adds.
The plan design states no step numbers of its own and cites this list,
so it needs no mapping.
Both sibling documents cite this list at `8cc31f59a4`,
under its two headings and with its fourteen step numbers.

The steps:

1.  The removal takes the per-slot claim for the slot (the binding design's claim,
    `lock/single_owner_claim.rs:1` to `:45`),
    proves lineage and ownership from the binding and both inventories,
    looks for the origin's deferred-destroy mark inside that proof,
    and prints the names that will be lost:
    the proposal's step 1 and the binding design's step 1.
    A mark found here is a refusal with the worktree still mounted and nothing written,
    which is the third refusal point the evening's deferred-destroy decision adds.
2.  It writes the removal intent,
    the clone's local `unmount` property,
    and requests the unmount:
    the proposal's step 2.
    With `--force` and holders its scan saw,
    it also writes the request the installed root handler reads,
    which makes the handler unmount the clone in each holder's namespace and end only the tasks that block that
    unmount (the evening's forced-removal decision).
3.  It verifies the handler's unmount and the receipt's own fields for its nonce,
    and a forced receipt may cover more than one namespace.
    Without `--force`,
    a holder the unmount's request or its receipt names cancels the removal here,
    before any delete,
    and the clone is attached at its slot again with no plan written.
4.  It writes the plan's first record,
    with every verified tuple `planned`:
    the plan design's step 4.
5.  It makes every refusal that must precede a delete:
    the first deferred-destroy check,
    the fresh root all-namespace verification,
    and the reproof immediately before the request:
    the plan design's step 5 and the proposal's step 3.
    A holder seen here without `--force` cancels the same way and leaves the plan standing with every tuple
    `planned` beside the reattached clone.
6.  **It publishes the origin cleanup record with `owed`,
    before it submits the first destroy request of this invocation.**
    This is this design's moment (V4).
    The plan design's step 6 and the binding design's step 2 confirm the same act and write nothing there.
    A rerun that finds a record for the same snapshot name compares its identity with the binding's:
    when they agree it keeps the record and writes nothing,
    and when they differ it refuses.
    A removal whose write fails destroys nothing and refuses before its first delete,
    because the record is what carries the cleanup past the clone.
7.  It submits each snapshot request,
    and the plan records that request's tuples and their outcomes:
    the plan design's step 7.
8.  It makes the second deferred-destroy check,
    after the clone's own snapshots are gone and before the head destroy:
    the evening's deferred-destroy decision.
    A refusal here leaves the removal pending with its record of what it destroyed,
    the clone unmounted and out of use,
    the plan saying `head=present`,
    and the origin record saying `owed`,
    and the cancel command is the person's way back.
9.  It submits the head destroy,
    and the plan records `head=requested` and then the outcome:
    the plan design's step 8 and the end of the binding design's step 3.
10. It attempts the origin cleanup:
    the identity proof,
    one single-name destroy,
    the absence proof,
    and the record's removal once the absence is proved:
    the proposal's step 4.
11. It settles the matching registration,
    pointer and underlay,
    and releases the wrapper's managed Git lock:
    the proposal's step 5 and the binding design's step 4.
    A removal that finds the lock gone writes it again,
    says so,
    and then releases it,
    and the finish pass,
    which holds the claim and has proved the registration,
    does the same rather than being exempt (the lock record's answer).
12. It removes the binding's files,
    `<slot>.origin-inventory` and `<slot>.clone-inventory` first and `<slot>.binding` last,
    and proves them gone:
    the binding design's step 5,
    whose sentence states that order.
    Inventories first means a kill inside this step can never leave an inventory without its binding,
    which would refuse the next add at the slot.
13. It removes the plan,
    and only after step 12 has proved the binding's files gone:
    the plan design's step 10.
14. It releases the per-slot claim last:
    the binding design's step 5.

The proposal fixes two of those places and no record fixes the rest.
It puts the origin cleanup at step 4 and the settle at step 5
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:947` to `:951`):
"For separate origin cleanup,
prove the exact recorded origin dataset and snapshot GUIDs and confirm that the snapshot still belongs to that
dataset."
and its step 5,
"Settle only the matching Git registration and pointer after clone destruction is proven."
(`:954`).
The attempt therefore precedes the settle,
and the success case is why the plan design's "the origin record stands after the settle" means its blocked case only.

The tail after the settle is not decided by any record.
The rule this design takes,
and asks the other two to take,
is one sentence:
**the plan is the last witness,
and it is removed only after the binding's files are proved gone,
with the claim released last.**
"V13 The tail's order" weighs the alternative,
which removes the plan before the binding.

The rule binds a finisher as much as the removal,
and the finisher needs a condition of its own,
because after the ordinary success path the origin record is gone by step 10.
A finisher meets the binding design's condition when a plan stands for the clone's GUID with `head=gone` and the
five origin fields equal to the binding's (the diff check's Z1),
and the origin record's absence is not a failure then.
The binding design's condition names a second arm,
no plan standing with the origin record equal;
that arm has no finisher today,
because the pass reads plans and a removal writes a plan before it publishes the record,
so it is a removal's own confirmation and a reader that meets it reports a removal killed before it wrote its plan.
Who may finish step 12,
in the binding design's words its round wrote for a sibling to quote:
"the plan design's finish pass,
which runs before every forwarded command,
or the next removal of the slot while a registration still stands;
never an add,
except the add's own fallback at A20 to A27,
and only where `phase=settling` says the add itself stopped mid-fallback."
The pass is the only command that can finish step 12 after both the clone and its registration are gone,
because no next removal of that slot can run then.

### The add that fails and destroys what it made

The dirty-main record's "Three further answers",
first answer,
decides who writes this record for a failed add,
in its words:
"Before destroying anything,
the add records the snapshot as owed cleanup (the same record a removal uses).
You get a finished worktree made by Git's own checkout,
one line saying the snapshot stays for now,
and the next `git worktree` command destroys it once the hold is gone."
That is this record's rule for a second writer.
It is not a veto entry any more.
The project decided it.

Who writes it,
and when.
The add itself is the writer,
as the second caller of this design's writer,
and its moment is after the add's unmount is verified and before the add's first destroy.
The unmount is not a destroy,
so the record's words "before destroying anything" hold at that moment.
The add's first destroy is the clone's (A22 of the binding design),
or the snapshot's own (A23) when the add made no clone and A22 is skipped.
The binding design's A23 gives the anchor as the record standing from before A22,
or before A23 when A22 is skipped (W9 of the fourth check),
and the dirty-main restore design,
which first published the record before its unmount request,
has been told to take this order.

With which identities.
The add's origin snapshot,
its dataset,
GUID and group,
the clone's dataset name and GUID when a clone was made,
and the slot's path.
An add that made no clone has no `cloneGuid`:
its `cloneDataset` holds only the proposed name,
and the reader says the name is proposed and the clone was never made
("The identity lines and the outcome lines" and W9).

What the next `git worktree` command does.
The look proves the snapshot's state and destroys it once the block is gone,
removes the record after the absence proof,
and prints one line when it destroys it.
A refused destroy leaves the snapshot standing and the record `owed`,
and a finish of the worktree by Git's own checkout does not need the record.

Four cases follow,
and only the third and the fourth leave a record standing:

- The add's own unmount is refused as busy (the dirty-main record's "A further answer"):
  the add writes its request back to the mounted state,
  stays pending with the clone mounted,
  and destroys nothing,
  so nothing is destroyed and no record is written.
  This record's moment is after the unmount is verified,
  and a refused unmount never reaches it.
- The clone dies and the origin snapshot's destroy succeeds or answers `ENOENT`:
  the attempt's absence proof removes the record in the same invocation,
  and nothing is owed.
- The clone dies and the snapshot destroy is refused,
  by a hold or a dependent clone (`EEXIST` or `EBUSY`):
  the snapshot stands and the record stays `owed`,
  and the look finishes it when the block goes.
  The dirty-main design states the same at `04ce55a880`:
  a marked snapshot another clone or a hold pins
  "answers `EEXIST` or `EBUSY`,
  and that is not a failed settle:
  the snapshot stands,
  the origin cleanup record stays `owed`,
  and the look that design describes finishes it later,
  which is what the person's text says."
  The binding design's A23 says the same from its side:
  the record "carries the snapshot if this destroy is refused",
  the record and its attempt are this design's,
  and the add's call on the writer is the dirty-main design's.
- The add is killed after the clone's destroy and before the snapshot's,
  with the record already published:
  the record stands `owed` and the look proves the snapshot's state and finishes it,
  because the clone that pinned it is gone.

The binding's end in these cases is the binding design's,
not this record's:
its `phase=settling` is written before the add's first destroy,
its A20 to A27 close the binding with the two inventories first and the binding last,
and a resumed add continues under that phase (its A20 and A26).
This design's record owes nothing to the binding's end and holds nothing about it.
The interrupted states the dirty-main design's settle marker covers are that design's,
and this record's contribution to them is the `owed` record the third case leaves.

### What a kill at each point leaves, and who finishes it

Each point names the kill,
what stands afterwards,
and which record a later run or `doctor` finishes the work from.
A look is the five worktree commands' look.
"Where in each command the look runs" gives each command's place.

- Before step 1 completes.
  Nothing is written.
  The clone and every snapshot stand.
  A later removal starts at step 1.
- Inside 1.
  The early mark look is the point.
  The rest of this bullet is what a refusal there leaves.
  The removal refuses with the worktree still mounted and nothing written,
  and only the claim stands;
  the claim module's liveness rule lets the next claimant take the next generation.
- Between 1 and 2.
  The slot's claim file stands and names the killed command.
  The claim module's own liveness rule lets the next claimant take the next generation
  (`lock/single_owner_claim.rs:1` to `:45`),
  and nothing else reads it.
- Between 2 and 3.
  The intent stands and the handler may still act.
  For a forced removal the request also stands,
  and a deadline that ends the wrapper's wait is not a cancellation.
  The record decides how far that reaches:
  the handler ends processes only while the very command that asked is still running,
  checked by its process identity before every signal and every unmount,
  so a killed command takes its authority to end processes with it;
  afterwards the standing request may still cause mount work that ends nothing.
  No plan and no origin record stand.
  Nothing was destroyed.
  A later removal re-verifies the unmount and writes the plan.
- Between 3 and 4.
  A verified unmount stands,
  or a holder the unmount named without `--force` has cancelled the removal and the clone is attached again.
  Nothing was destroyed.
  A later removal writes the plan.
- Between 4 and 5.
  The plan stands with every tuple `planned` and `head=present`.
  No origin record stands.
  Nothing has been destroyed yet.
  A refusal at the first mark check leaves the removal pending with the clone unmounted and out of use,
  and the plan stands;
  the cancel command is the way back.
  The finish pass leaves the plan,
  because its head is not `gone`;
  the next removal of the clone resumes it and removes the plan at step 13.
  A holder the fresh scan sees without `--force` cancels and reattaches the clone,
  and the plan stays with every tuple `planned`.
- Between 5 and 6.
  The same holds with every refusal passed.
  The origin record is absent and nothing was destroyed,
  which is why step 6 sits after every refusal and before every destroy.
- Inside 6.
  A kill between the staged bytes and the link leaves a staging name and no record,
  and nothing was destroyed.
  The next look skips the staging name,
  the next removal publishes the record,
  and `doctor` reports the staging name until step 18 removes it.
- Between 6 and 7.
  The origin record says `owed`,
  the plan says `planned`,
  and the registration,
  the clone and every snapshot stand.
  A look reads the live identity,
  sees the clone that still pins the snapshot,
  and records `blocked` with reason `dependency`.
  Only the cancel command is the way back here,
  because a holder-seen cancel runs before step 6;
  it leaves the same record,
  and what it restores is the cancelled-removal design's.
- Inside 7.
  The plan says `requested` for the whole request,
  the origin record says `owed`,
  and the registration stands.
  The plan's next run proves each tuple against the live inventory.
- Between 7 and 8.
  The same holds inside 8.
  The clone's own snapshots are gone,
  the plan says `head=present` with their outcomes,
  the origin record says `owed`,
  and the clone is unmounted.
  A refusal at the second mark check leaves exactly that state pending,
  and the cancel command is the person's way back.
- Inside 9.
  The same holds between the head destroy and its outcome write:
  `head=requested` stands and the head may be gone,
  and the intent went with the head.
  The plan's next run takes the head-gone path,
  and a look may destroy the origin snapshot,
  which is harmless to the settle.
  A holder that surfaces here cancels the removal,
  which the forced-removal record decides as "mount again and report":
  the clone is attached again with nobody present,
  each snapshot the batch destroyed is reported,
  the plan keeps the batch's outcomes and its head,
  the origin record stays `owed`,
  and the look then finds the live clone's dependency.
- Between 9's outcome and 10.
  The plan says `head=gone`,
  `refused` or `unknown`.
  On `gone` the origin record stands and the registration stands;
  the finish pass or the next removal settles the registration,
  and a look may finish the origin cleanup.
  On `refused` the clone still stands and nothing settles;
  the origin record stays `owed`,
  the look finds the live clone's dependency,
  and the plan's next run resumes it.
  On `unknown` nothing settles on a guess:
  the plan design's section 4 is where the resume proves the head's state,
  and this design's look blocks on whatever still stands.
- Inside 10.
  The same after the destroy is submitted:
  the record names a snapshot that may be gone.
  The next look proves the absence and removes the record,
  so the possible lie is bounded by one look.
- Between 10's success and 11.
  No origin record stands,
  the registration stands,
  the plan stands with `head=gone`,
  and the binding stands.
  The finish pass settles the registration,
  finishes step 12 under the condition above,
  and then removes the plan.
- Between 10's blocked outcome and 11.
  The origin record stands blocked,
  the registration stands,
  and the plan stands.
  This is the blocked case of the plan design's "the origin record stands" after a settle,
  and the success case is "or none".
- Inside 11.
  A settle is more than one removal of Git's files,
  so a kill inside it can leave the registration's directory partly emptied.
  The directory's identity is its object id,
  which a partial emptying does not change,
  so the finish pass matches the same registration and repeats the settle.
  The plan,
  the binding and the origin record stand as they did.
- Between 11 and 12.
  No registration and no lock stand;
  the binding stands,
  the plan stands,
  and the origin record stands only if its attempt was blocked.
  The plan design's finish pass finishes step 12,
  and it is the only command that can do it after the registration is gone;
  the add is not a finisher except at its own fallback's `phase=settling`.
  The pass then removes the plan.
- Inside 12.
  The inventories go first and the binding last.
  A kill between two of the three leaves the remaining files,
  and because the binding is last no standing inventory is left without its binding.
  A finisher treats a standing inventory as part of step 12 and removes it before the plan.
- Between 12 and 13.
  The binding's files are proved gone,
  the plan stands,
  and the origin record stands only if it was blocked.
  The plan is the last witness here,
  and the finish pass or the next removal removes it after proving that nothing of the binding stands.
  No state reachable by a kill now leaves a binding with no plan and no registration,
  which is the state the binding design's third reading calls a clone destroyed outside the wrapper:
  after this order that reading means what it says.
- Inside 13.
  The plan's own unlink may be the point of the kill.
  A kill before it leaves the plan for the finish pass or the next removal to remove,
  and a kill after it leaves no plan.
  Either is a neighbouring point's state.
- Between 13 and 14.
  The plan is gone,
  the origin record stands only if it was blocked,
  and the claim's files remain.
  The claim module's liveness rule lets the next claimant take the next generation,
  and a look finishes a blocked record.
- After 14.
  Only a blocked origin record can stand.
  A look finishes it,
  and `doctor` names it until then.

The invariant this list gives is the item's sentence in the form that survives a promote:
an origin snapshot that is still owed always has its record standing,
because step 6 publishes it before the first destroy and step 10 removes it only after the absence proof;
and every point before step 6 leaves the registration,
the clone and every snapshot whole.
No point leaves a standing record that the next reader would read as another event.

## The home,
its name grammar and its key

### The cleanup folder

One folder per repository,
beside the folder the wrapper already keeps in a common Git directory:

```text
<checkout>/.git/monochromatic-git-policy-cli/   account  0700
  origin-cleanups/                              account  0700
    git-worktree-clone-origin-4f0c1d3ae5b28c6907df1a2b3c4d5e6f   account  0600
    git-worktree-clone-origin-9ab18c2de4f507b3a1c8d2e6f4b09a7c   account  0600
```

The path is the repository's common Git directory,
the wrapper's own folder there,
and `origin-cleanups` under it.
It sits where the leavings record's folder sits
(`worktree/worktree_removal_leaving_that_outlives_its_record.rs:51`,
which names `monochromatic-git-policy-cli/leavings`),
inside the wrapper's own folder in the common Git directory.
That placement is what the repository's rename machinery moves when it moves that folder
(`foundation/kept_names_in_the_git_directories.rs:48`,
and `foundation/wrapper_names_old_and_new.rs:74` to `:77`),
so only the inner name `origin-cleanups` is a new constant.

The folder is not the binding design's own folder family,
which its revision moved inside the same parent,
to `monochromatic-git-policy-cli/worktree-clones/v1/` (F24).
Both now share the wrapper's folder in the common Git directory,
and only the inner name differs;
that is the first of the disagreements the two designs carry,
and "Where this design and its siblings disagree" states both choices and who could decide between them.

Why the common directory and not a worktree's own Git directory:
a linked worktree's Git directory is removed when that worktree is settled,
and this record must outlive the worktree,
its registration and its folder.
The common directory survives all three (F15 and the Git source at `builtin/worktree.c:212` to `:246`,
which prunes entries inside `worktrees/` and nothing beside it).
It is also where the wrapper already keeps durable records a command finds without a scan of the machine,
and where `doctor` already looks.

Why not the slot's folder or the checkout:
the slot's path is reused by the next add at that slot (F15),
and a record that lived in the slot would be destroyed with it or mistaken for the next clone's.

### The name grammar and the key

A record's file name is exactly the origin snapshot's name:
the prefix `git-worktree-clone-origin-`
(`worktree_clone_mount_handler/sweep_clone_check.rs:53`)
followed by the add's nonce,
which is 32 lowercase ASCII hexadecimal digits
(`package/git-policy/cli/SPEC.md:6100`).
The whole name is 59 bytes,
well under the 255-byte dataset-name limit (F9),
and the prefix is the one the add already gives the snapshot it clones from.

The key is that name.
The reader walks the folder,
accepts only a name the grammar admits,
and treats it as one record of one origin snapshot.
A name that does not match is a foreign entry:
never read,
never destroyed,
and reported by `doctor`.
A name that begins with a dot is a staging name of this package's publication
(`staging_beside` names `.<name>.<random>.partial`),
never a record,
and `doctor` reports it (V8).

Why the snapshot's name and not a fresh record identifier:
the snapshot is the thing the record is about,
the name already carries the add's nonce,
and it is unique for as long as the snapshot exists.
A new record identifier would add a second key that nothing proves against the pool.

### The owner and the modes

The folder is owned by the account and has mode `0700`,
and each record is owned by the account and has mode `0600`,
created with `create_private_directory_all` and `staging_beside`'s mode
(`foundation/private_storage.rs:546` to `:564`,
and the staging file's `0600` at `:208`).

The account owns both because the account is the writer,
the reader and the authority that destroys (the decision at the proposal `:1409`),
and because no root-side channel remains once the clone is gone.
A root-owned folder would need a root writer,
and the only root writer this route has is the mount handler,
which acts on a clone's property and has no clone to act on here.
The handler also owns mount and unmount and not deletion (the decision record `:32` to `:33`).

Under the stated threat model this is enough (the decision's own sentence at `:230`:
"the account is trusted,
and root and the account are not a security boundary here").
L3 says what it gives up:
the account can rewrite or delete a record,
and the same authority can destroy the snapshot directly,
so root ownership protects nothing against that account.

### What a reader refuses about owner and mode

The look destroys a snapshot on a record's word,
so it refuses a record,
a folder or an entry it cannot prove is this account's:

- a record whose owner is not this account,
  or whose mode lets any account but its owner write it;
- a folder whose owner or mode fails the same test;
- an entry that is not a regular file,
  which includes a link and a folder where a record belongs
  (`read_regular_file` refuses both,
  and the owner and mode check is this design's addition);
- a record whose bytes are not its grammar ("The state machine").

Under the threat model the account is trusted,
so this check catches a mistake rather than an attack,
and it stays for the same reason the binding design keeps it (F24):
the record decides what a destroy may target,
and a mistake there is not recoverable.
It also keeps the rule the plan and the binding designs hold,
so the three records treat a wrongly owned file the same way.

## The fields and what proves the origin later

### One record

A record on its second attempt,
blocked because a clone of the snapshot still exists:

```text
# monochromatic-git-policy-cli origin cleanup, version 1
state=blocked
attemptedAt=2026-11-22T18:31:04-05:00
blockedReason=dependency
blockedErrno=0
numClones=1
userRefs=0
{"originDataset":"zpcachyos/ROOT/cos/home/monochromatic-git-checkout","originDatasetGuid":"12039048377182978123","originSnapshot":"git-worktree-clone-origin-4f0c1d3ae5b28c6907df1a2b3c4d5e6f","originSnapshotGuid":"3745620968172523788","originSnapshotCreatetxg":"93823","cloneDataset":"zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/add-cache-layer","cloneGuid":"9012873465091283655","slotPath":"/home/user/Monochromatic/.claude/worktrees/add-cache-layer","slotPathHex":"2f686f6d652f757365722f4d6f6e6f6368726f6d617469632f2e636c617564652f776f726b74726565732f6164642d63616368652d6c61796572","startedAt":"2026-11-22T18:38:41-05:00"}
```

The shape is the leaving record's,
short `key=value` lines followed by one JSON object
(F16 and `worktree/worktree_removal_leaving_that_outlives_its_record.rs:108` to `:128`),
because it needs no escaping decision beyond the wrapper's own JSON writer,
and because a person reading a blocked record sees the reason before the long line.
A `blocked` record writes `blockedErrno=0` when the block came from the counts and no request was made,
and a nonzero errno when a request was refused ("The state machine").

### The identity lines and the outcome lines

The JSON object holds the identity.
It is written once and never changes.
The lines before it hold the outcome,
and they are the only bytes a later attempt may replace.
A rewrite with an unchanged outcome writes the same bytes,
which control T1 asserts.

The short lines:

- `# monochromatic-git-policy-cli origin cleanup, version 1` is the header.
  A reader refuses a file whose first line is not exactly this,
  and it refuses one whose version it does not know.
- `state` is one of `owed` and `blocked` ("The state machine").
- `attemptedAt` is an ISO 8601 instant with its offset to the second,
  anchored to the command's clock,
  and absent until an attempt first records an outcome.
- `blockedReason` is one of `dependency` or `deferred` or `unproved` or `refused`,
  and absent unless the state is `blocked`.
- `blockedErrno` is the errno of a refused request as a decimal integer.
  It is `0` when the attempt submitted no request,
  which a `dependency` block read from the counts does,
  and when a refusal carried no errno.
- `numClones` is the live clone count the attempt read (F1).
- `userRefs` is the live user-hold count the attempt read (F2).

The JSON object's keys,
in the fixed order the writer emits and the reader requires:

- `originDataset` is the checkout dataset's full name as the add recorded it,
  at most 255 bytes (F9).
- `originDatasetGuid` is the dataset's GUID as a decimal string of a `u64`.
- `originSnapshot` is the snapshot's own name,
  which must equal the file's name and the grammar of "The name grammar and the key".
- `originSnapshotGuid` is the snapshot's GUID as a decimal string.
- `originSnapshotCreatetxg` is the transaction group that created the snapshot,
  as a decimal string.
- `cloneDataset` is the clone's full dataset name,
  at most 255 bytes.
  An add that made no clone writes the clone name it proposed,
  and then `cloneGuid` is absent and the name is only proposed.
- `cloneGuid` is the clone's GUID as a decimal string,
  and it is absent exactly when no clone was made.
  A record without it says its clone was never made,
  and a reader that needs a clone (none of this design's proof does) reports that it has none.
- `slotPath` is the slot's path as text when the path needs no JSON escape,
  and absent when it does.
- `slotPathHex` is the slot path's bytes as lowercase hexadecimal,
  always.
- `startedAt` is when the writing command first wrote the record,
  a removal or a failed add,
  as an ISO 8601 instant with its offset to the second.

Why every GUID and transaction group is a JSON string and not a number:
the wrapper's JSON reader follows the incumbent's semantics,
where a number is a double,
and its `MAX_SAFE_INTEGER` is 9,007,199,254,740,991
(`foundation/json_record.rs:36`),
while a dataset GUID fills a `u64`.
A GUID written as a number would lose its low digits in a reader and then disagree with the pool.
A decimal string goes through `DatasetGuid`'s grammar,
which refuses a sign,
a leading zero and a value beyond a `u64`
(`worktree_clone_mount_protocol/dataset_guid.rs:39` to `:60`).

Why the path travels twice:
a text record cannot hold a path that is not text
(`foundation/path_bytes_in_records.rs:1` to `:20`),
and the hexadecimal field is the only form that loses no byte.
The text field is written only when it needs no escape,
which is what keeps the bound below sound (O9).
Nothing compares the two fields with each other;
the hex field is what `doctor` prints for a person whose terminal can show it.

Why there is no free-text field:
a record is read by a report,
and a field a person can write into is a field that steers the report (L8).
The failure name a person needs is the snapshot's,
and the numbers are the counts and the errno.

Why there is no attempt counter:
no cap on attempts is shipped ("Who carries the cleanup and when"),
because a cap is a number that needs a measurement,
and a counter would force a durable write on every attempt rather than only when the outcome changes.

Why there is no line naming the command that wrote the record:
no window depends on the removal's liveness ("The removal's one order of writes and removals"),
and the attempt re-reads the world,
so a live clone shows up as a dependency rather than as a missing fact.
The staging name is reported by `doctor` and read by nothing (V8).

### The bound

The reader accepts at most 13,368 bytes of one record.
That number is the widest record these fields can spell,
field by field:

- The header line and its line feed:
  57 bytes.
- The short lines at their widest:
  `state=` with `blocked` at 14,
  `attemptedAt=` with an instant at 38,
  `blockedReason=` with `dependency` at 25,
  `blockedErrno=` with a four-digit errno at 18,
  `numClones=` with twenty digits at 31,
  and `userRefs=` with twenty digits at 30.
  That is 156 bytes.
- The JSON object's keys,
  its quotes and separators at 195 bytes.
- Its values at their widest:
  two dataset names at 255 each (F9),
  the snapshot name at 59,
  four decimal strings at 20 each,
  a slot path of 4,095 bytes that needs no escape,
  its hexadecimal form at 8,190 bytes,
  and one instant at 25.
- A final line feed.

Those terms sum to 13,368 bytes.
No rounding is applied,
because a rounded power of two is a pick the accepted store's bound does not make
(`doc/decision/cli-git-zfs-slot-identity-store-format.md:140` to `:141`).
A new field or a new escape changes the version number and the bound with it.
The writer refuses to publish a record its own reader would refuse,
which is the rule the receipts already hold to
(`package/git-policy/cli/SPEC.md:6154` to `:6155`).

### What proves the origin's identity and lineage later

The proof is made of live reads at the moment of use,
compared with the record's immutable fields.
Nothing is proved by a name,
a folder,
the slot's path,
the Git registration or the record's own existence.

The attempt runs these steps in this order.

1.  Read the record whole under the bound.
    A record that does not read is a refusal,
    and nothing is destroyed ("The state machine").
2.  Read the named origin dataset's live identity with `objset_stats` (F1):
    its `dds_guid`,
    its `dds_is_snapshot`
    and its `dds_origin`.
    When `dds_origin` is not empty,
    read that snapshot's own `objset_stats` for its `dds_creation_txg`.
    The attempt refuses as `unproved` when the dataset cannot be read,
    when its GUID differs from `originDatasetGuid`,
    when it is itself a snapshot,
    when its origin cannot be read,
    or when its origin's `createtxg` is at or above the record's `originSnapshotCreatetxg`.
    The last is the promote guard,
    and it compares groups and not names.
    `zfs promote` moves "the snapshot that was cloned,
    and any snapshots previous to this snapshot",
    to the promoted clone (F20),
    so a promote of a later clone moves the record's snapshot too,
    and the dataset's origin then names the later snapshot.
    A guard that compared names would ignore that origin,
    list the dataset,
    find no GUID,
    and remove the record while the snapshot lives under the promoted clone:
    that is the diff check's Z2,
    and its fix is this group.
    Only a promote makes a dataset's origin newer than a snapshot the dataset once held,
    because the recorded snapshot was made after the dataset existed;
    a checkout that is itself a clone of a backup has an origin older than itself,
    and so older than every record,
    and an older origin is read and then ignored.
3.  List the named dataset's snapshots with `snapshot_list_next` (F1),
    and compare every `dds_guid` with `originSnapshotGuid`.
    No match is a completion,
    but only on the strength of step 2's checks:
    a dataset whose identity did not match,
    or whose origin is at or above the recorded group,
    never reaches this completion.
    A match is the live snapshot,
    and its name must equal `originSnapshot`
    and its `dds_creation_txg` must equal `originSnapshotCreatetxg`.
    A match with any other name or group is `unproved`:
    nothing is destroyed.
4.  Read the live snapshot's `dds_num_clones`,
    its `userrefs` and its `defer_destroy` (F1 to F3).
    A nonzero clone count or user-reference count is `dependency`,
    and a mark that is on is `deferred`.
    Either one ends the attempt before any destroy.
5.  Submit one request naming exactly that snapshot,
    through `destroy_snaps` (F1 and `requests/snapshot_requests.rs:68` to `:75`),
    which sends no `defer` key.
    The request never names a dataset,
    never recurses,
    and never names a second snapshot.
6.  Prove absence by repeating steps 2 and 3.
    A recorded GUID that is still listed is a refusal to settle,
    and the record says so.
    The next trigger re-reads the identity and may try again,
    which is safe because a destroy of a name that is gone is a no-op (F5)
    and because the identity is compared before every destroy.
7.  Remove the record only after step 6's absence proof.
    A removal that finds the name already gone is a completion of the same work.

An attempt that cannot prove absence answers `unproved` and keeps the record:
that is what a dataset the attempt cannot read,
a dataset whose name now holds another dataset,
a dataset whose origin is at or above the recorded group (F20),
and a dataset that is gone all produce.
None of them completes,
which is the accepted rule that a destroy is settled
"only after proving the expected objects are gone,
never from a missing name alone"
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:963` to `:964`).
The dataset-gone case is the sharpest:
the snapshot may be alive under a promoted dataset (F20),
so its absence from a dataset that is not there proves nothing,
and the record waits for a person (L11 and L12).

The kernel's dependency check is the dependency proof,
not a scan of children or a name test (the proposal `:950`),
and the counts in step 4 are read to report the block before a request that would be refused anyway.
A kernel refusal is reported by its errno and never by its wording,
because the two layers spell the same refusal differently (F11).

What is not proof:

- The snapshot's name (F5 allows a name to be absent and then present again,
  and a name can be reused by a new snapshot with another GUID).
- A missing dataset name (F20).
- A dataset whose snapshot list holds no matching GUID while the dataset's origin is at or above the recorded
  group (F20).
- `createtxg` alone (L2).
- The record's file name being the snapshot's name.
  It is the key and not the identity,
  because the name is what a person's `zfs destroy` can recreate.
- A destroy that returned success.
  Success of a name-based request proves only that the name is not there now (F5),
  and the absence proof is what settles the record.
- A blocked record's own words (L8).

The same-account window between step 3 and step 5 stays open,
and L10 records it as the accepted residual.

### The options and their ranking

The record's shape is weighed in "V2 The record's shape".
The identity field set is weighed in "V5 The identity fields".
The chosen shape is the leaving record's,
and the chosen field set proves both accepted identities plus the group that ties the snapshot to the add.

## The state machine

### The states

Two states are stored,
and one end is proved.

```text
(nothing)
  |  the removal publishes the record after every refusal and before the first destroy
  v
owed  --attempt: recorded GUID absent, dataset proved---> (record removed)
  |  \--attempt: destroy succeeded and absence proved--> (record removed)
  |   \--attempt refused or unproved--> blocked
  |
blocked --attempt: recorded GUID absent, dataset proved---> (record removed)
  |   \--attempt succeeded-------------------------> (record removed)
  \--attempt refused again--> blocked (no rewrite when unchanged)
```

`owed` means the record exists and no attempt has recorded an outcome.
`blocked` means an attempt ran and could not finish,
and its reason is one of the four below.
Completion is not a state:
it is the record's absence,
and the absence is only written by removing the file after the proof.

### What moves each state

The removal writes `owed` once (step 6 of "The removal's one order of writes and removals"),
and its own invocation then attempts immediately at step 10.
That is the normal path:
a clone with no other holder is destroyed,
the origin snapshot has nothing depending on it,
and the record is removed in the same invocation before the settle.

The look every command makes ("Who carries the cleanup and when") attempts every record it finds.
An attempt re-reads the identity (the seven steps in "What proves the origin's identity and lineage later")
and ends in one of three ways:
completion,
a recorded block,
or no durable write at all when the outcome is unchanged.

### What blocks it

`dependency`:
the live snapshot has at least one clone,
or at least one user hold (F4).
This is the ordinary blocked case:
the removal's own record was written before its first destroy,
and a kill or a cancel left the clone standing,
or another clone was made from the same origin snapshot by something this route does not know.
It clears when the last clone or hold goes,
and the next command's attempt then completes the record.
The record carries the counts and not the names (L7).

`deferred`:
the snapshot's `defer_destroy` is on (F3).
A marked snapshot may still be cloned and held (the man page at `:114` to `:115`),
and the wrapper never clears another writer's mark.
It clears when the mark's owner destroys the snapshot,
which the next attempt proves,
or when an administrator removes the mark.
The sibling design of the deferred-destroy checks owns how a removal reads the mark;
this design reads the same property for the cleanup (A3).

`unproved`:
the live identity disagreed with the record,
the named dataset's origin is at or above the recorded group (F20),
the dataset is gone,
which leaves the snapshot's survival unproved (F20),
or a value the proof needs could not be read.
Nothing is destroyed in this state.
It clears only when the live identity matches again,
which is not expected,
or when the recorded GUID is absent under a dataset that passes step 2.
That absence is a completion.
A person should read this state as "the objects the record names are not the objects the pool has".

`refused`:
the destroy request itself was refused,
or it answered success and the absence proof still found the GUID.
`blockedErrno` carries the errno when there was one (F11).
It clears when a later attempt succeeds.
A permission refusal is the case an administrator must look at (A2 and M4).

An unknown outcome is not a fifth reason.
A request that answered with an error may still have taken effect,
and a request whose answer never arrived leaves the pool unread.
Nothing in this design concludes anything from an error code or from silence:
the attempt records what it could read,
and the next attempt's live read decides whether the snapshot is still there.
While that read cannot be made,
the record stays exactly as it was and `doctor` reports the last reason it holds.

A record whose bytes cannot be read,
or whose owner or mode is not the account's,
is not a state.
Nothing may be concluded from a record that does not parse,
writing over it would destroy the only evidence of what was owed,
so the attempt refuses,
`doctor` reports the file,
and a person destroys the snapshot by hand if they can.
After that the file is a record whose GUID is absent,
but a reader that cannot parse it cannot prove that either,
so a person removes the file.
`doctor` names that as the way out,
and the project decided that such a record is refused and named ("A record that cannot be read").

### What ends it

The record ends in exactly two ways,
and each is proved by an absence and not by an intention.

1.  The attempt proves the recorded GUID absent from a dataset that passes step 2.
2.  A person or another program destroys the snapshot,
    and the next command's attempt proves the absence under a dataset that passes step 2.

A destroyed origin dataset is not a third way.
It takes its snapshot-name table with it (F7),
but the same promote that moves a snapshot out of its dataset (F20) is why a gone dataset proves nothing,
so that case stays `unproved`,
`doctor` reports it,
and a person decides (L12).
A record is removed with one `unlink`,
and a second removal that finds it gone is the same completion.

What never happens:
a trigger abandons a record,
a timer expires it,
root removes it,
or a person's command destroys a snapshot whose identity it did not just compare.

### The options and their ranking

The state vocabulary is weighed in "V6 The state vocabulary".
The chosen vocabulary stores two states,
makes completion an absence,
and keeps the four reasons as codes.

## A settled registration and a pending record

### What settling leaves behind

Settling removes the matching Git registration and pointer,
releases the wrapper's lock,
and frees the slot (the proposal `:466` to `:470` and `:1462` to `:1463`).
What it does not remove is the origin snapshot,
which the record names,
and which the pool keeps until something destroys it.

The measured Git behaviour is what makes the record's key independent of the registration (F15):
a missing registration refuses a new add at the same path,
a prune clears it,
and the next add takes the same path under the same registration name.
So the registration's name is reusable,
the slot's path is reusable,
and neither can be the record's key.
The record holds the dataset and snapshot GUIDs instead,
and its `slotPath` field is history that no check compares.

### The recovery the settle's sentence asks for

The answer says settling "requires recovery without its Git entry" (the proposal `:1462` to `:1463`).
This design's half of that recovery is:

1.  The record is found from the repository's common Git directory,
    by its own name grammar,
    with no Git question asked.
    A command in any worktree of the repository finds it,
    and `doctor` finds it even when no registration stands.
2.  The origin cleanup needs nothing from Git:
    its identity proof is a pool read,
    its destroy is a pool request,
    and its completion is a pool read.
3.  A slot that another worktree now holds changes nothing,
    because the record never reads or writes the slot,
    and the four rules of "What the pending cleanup and the new add do to each other" keep the two apart.

What this design does not recover is the registration itself.
A removal killed after the clone is gone but before the settle
is the plan's finish pass and the registration's own recovery.
That is the plan design's business (F25),
and this record neither holds nor needs a Git entry.

### Whether a new add may take the same slot

Yes,
and at once.
The accepted design's own words are that settling frees the slot (the proposal `:1462` to `:1463`),
and nothing about the pending snapshot depends on the slot:
the snapshot belongs to the checkout dataset,
and the slot is where a clone of it used to be mounted.

A new add at that slot writes a new snapshot with a fresh nonce,
clones that snapshot into a new dataset with a new GUID,
and registers Git under the same name.
Its path through the pool touches none of the record's fields:
the record names the previous snapshot and the previous clone,
and the new add names neither.

The binding design's revision removed the binding whether or not the origin cleanup is still blocked (F24),
so it now frees the slot at the settle as the accepted words say.
Its Q2 still keeps a slot blocked while a clone's absence is unproven,
which is a different case:
there the binding is the only record that names the origin,
and keeping it is the safe arm.
For the blocked-origin case this design and the binding design agree,
and "Where this design and its siblings disagree" records it.

### What the pending cleanup and the new add do to each other

They meet in four places,
and each one is safe by a rule that already exists.

1.  The cleanup folder.
    Both commands may read it.
    The cleanup lists it and the add never opens it.
    A listing and a record's removal do not disturb each other.
2.  The pool's dependency check.
    The pending cleanup names the old snapshot.
    If anything still depends on that snapshot,
    the kernel refuses (F4),
    and the record stays blocked.
    The new clone depends on its own new snapshot and never on the old one,
    because an add never substitutes a different origin (the proposal `:1385`).
3.  The identity comparison.
    The cleanup destroys only a snapshot whose live GUID and group equal its record,
    under a dataset that passes the proof's step 2.
    The new add's snapshot has another name and another GUID,
    so the comparison cannot match it.
    A reused nonce would show as `unproved` rather than as a destroy.
4.  The slot's claim.
    A removal takes the per-slot claim (F18 and the binding design's `:684` to `:732`),
    and this cleanup takes no claim at all,
    so a pending record never holds a new add's slot,
    and a new add never waits on a cleanup.

The one thing the cleanup and the add share is the kernel's serialization of two requests on one pool,
which is the kernel's business and not a lock of this design.

### What a person sees

A person who adds a worktree where a cleanup is pending is not warned.
The add is safe by the four rules above,
and a warning would ask a person to decide something no check needs.
`doctor` shows the pending record and the new worktree side by side,
and the record's line names the slot path as the old clone's history.

### Decided, and not a choice here

Whether a new add may take the slot is decided by the proposal:
"Settling frees the slot but requires recovery without its Git entry"
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:1462` to `:1463`),
and "settling does not wait for the origin" (`:469` to `:470`).
The answer is that it may,
at once,
and this document states it as the accepted rule rather than as a veto entry.

## Who carries the cleanup and when

### The carrier

The cleanup is carried by the wrapper itself,
as the account,
in a look that runs inside the commands of the `git worktree` family.
The project decided which commands may destroy a leftover origin snapshot
(`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
"Which commands may destroy a leftover origin snapshot"):
"Only `git worktree add`,
`remove`,
`list`,
`prune` and `repair` destroy it,
printing one line;
`doctor` stays read-only and prints the `zfs destroy` command."
No other command of the repository runs the look,
and `git status` therefore prints nothing and destroys nothing.
The cost the record states with the answer is that a repository that never runs one of those keeps the snapshot and
its space.

The leavings look is the precedent for the shape:
it runs once per command,
it pays one open when its folder is absent (F16),
and it acts only on what it can prove.
It queues work for the service;
this look destroys a snapshot itself.

### Where in each command the look runs

The look is one function and each command runs it once.

- `git worktree remove` runs it at step 10 of "The removal's one order of writes and removals",
  inside its own work,
  after the clone is destroyed and before the settle.
  That is the removal's own attempt,
  and the removal does not run the look again when it returns.
- `git worktree add` runs it at the end of its own work,
  after the add has settled:
  a successful add wrote no record,
  so the look pays one open and finds nothing of its own,
  and a failed add's fallback reaches its origin-snapshot step,
  which is the look itself,
  under the binding record's `phase=settling` (A23 of that design).
  A failed add therefore runs the look as its own fallback's step and not a second time when it returns.
- `git worktree list`,
  `git worktree prune` and `git worktree repair` run it as the last step of their own work,
  after their own answer is decided and before the process returns.

Each command's look prints its lines on standard error,
which keeps them out of `git worktree list`'s standard output,
and none of them changes the command's exit status.
A command that runs the look is not one of the mount route's triggers (the proposal `:1393`),
and the decided answer is what authorizes it.

### What the look does, step by step

The look is ordered so that a command pays the least when nothing is owed.

1.  Open the cleanup folder.
    There is no folder in a repository that never removed a ZFS clone,
    and the look ends here.
2.  List the folder.
    Names that the record grammar admits are records.
    A name beginning with a dot is a staging name and is skipped.
    Any other entry is foreign and is reported and never read.
3.  For each record:
    read it under the bound,
    and refuse as unreadable anything that is not the grammar or is not this account's.
    The project decided that a record which cannot be read is refused and named
    under the decision headed "A record that cannot be read",
    in its words:
    "Change nothing,
    name the file,
    and wait for a person".
4.  Read the record's identity and the live identity ("What proves the origin's identity and lineage later").
    A recorded GUID that is absent under a dataset that passes the proof's step 2 completes the record:
    remove it and print nothing.
5.  If the live snapshot is there,
    read its counts and its mark.
    A block is recorded when its reason tuple changed since the record's last attempt,
    and the look prints one line when it did ("What the look prints").
6.  Otherwise submit one single-name destroy and prove the absence.
    Print one line naming the snapshot that was destroyed.
7.  Remove the record after the absence proof,
    or record the refusal with its reason and errno.

The look never reads Git,
never takes a claim,
never waits for a person,
and never changes the command's exit status.
Its lines go to standard error,
as every informational line of this program does (F17).

### When the route's gate refuses, or the device cannot be opened

The look makes no ioctl when the route's runtime gate refuses
(the implementation plan's step 6 check)
or when `/dev/zfs` cannot be opened.
A failed probe arm refuses new attaches and keeps every other operation's route (F26),
and a gate that refuses means the request layout has not been approved for this kernel,
so an attempt would be a request this design cannot justify.

The rule is:

- With no record standing,
  the look opens no device and prints nothing.
- With a record standing,
  the look prints one line naming the gate's refusal or the open's error and the record,
  and does nothing else.

How often that second line appears is the same rule the project decided for a pending record,
under the heading "How often a pending origin cleanup speaks",
in its words:
"One line when it destroys,
or when the reason it is blocked changes;
silence while nothing changed".
The gate's refusal is that record's reason while the gate refuses,
and the failing gate step is the reason's detail:
the line prints when the record first meets the gate and when the failing step changes,
and nothing prints while the same gate step keeps refusing.

`doctor` reports the gate's refusal and the standing record together,
and neither the look nor `doctor` decides that the gate's refusal authorizes removing a record.

### What the look prints

The look prints one line when it destroys a snapshot,
and one line when a record's reason changes.
The project decided the second,
under the heading "How often a pending origin cleanup speaks",
in its words:
"One line when it destroys,
or when the reason it is blocked changes;
silence while nothing changed."

A record's reason is the pair its line names:
the reason code
(`dependency`,
`deferred`,
`unproved`,
`refused` or the gate refusal)
and the detail the attempt read for it.
The detail is what a person would act on:
the clone count and the hold count for a dependency,
the presence of the mark for a deferral,
the errno of a refusal,
or the failing step of the gate.
The reason changed when the code or any part of the detail changed:
one clone becoming two prints,
the same count read again does not,
and a record that goes from blocked to destroyed prints the destruction's line.
The record's own bytes are never printed,
and the line names the snapshot it destroyed and the refusal it first met.

### What the look costs

No ceiling is shipped.
The empty case is one open of a folder that may not be there (F16),
and after a repository's first cleanup it is one open and one listing of an empty folder (L14).
A record that is owed but pinned costs one `objset_stats` of the dataset,
one listing step per snapshot of that dataset,
one `objset_stats` of the snapshot,
and no write.
A blocked record that stays blocked costs the same and no durable write.
Each of those reads is a request to the pool the command already uses,
so the look adds no failure domain the command did not have.
"Measurements owed" M1 sizes the added cost of a worktree command,
which is the only kind that runs the look.

### Two commands at once

Two commands of one repository may attempt the same record at once.
Nothing breaks:
both read the same identity,
the kernel ignores a destroy of a name that is gone (F5),
both prove the absence,
and the second removal of the record finds it gone and is a completion.
The one odd outcome is a writer that read the record before another removed it and then records a refusal after it,
which re-creates a record whose snapshot is already gone.
The next command's look proves the absence and removes it again.
That is self-healing and costs at most one extra look,
which is why this design takes no claim ("V7 Serializing attempts"),
and control T17 forces the whole sequence.

### When the event never comes

The record waits,
and the snapshot stays.
Nothing expires either of them (L6).
The ways it still ends are the two in "What ends it",
and each begins with a command in that repository or with a person who read a `doctor` line.
A repository whose path is gone and a repository whose Git directory was deleted keep nothing,
and those are L4 and L5.

## What doctor reports and what a person can do

### Where the lines sit

The lines join the ZFS part of `doctor`'s filesystem section,
which plan step 18 builds
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md:2017` to `:2060`,
which already lists "each clone's origin and retention").
They keep that section's rules (F17):
standard error,
the same prefix for every line of one report,
the section's ceiling with a line of its own when the read does not finish,
and no change to the exit status.
`doctor` runs as the account,
which no record states and which this design assumes (A4).
The plan design assumes the opposite (F25).
It never destroys a snapshot,
never rewrites a record,
and never removes a record or a staging name (V8).

### The owed line

A record that no attempt has ended:

```text
git monochromatic-git-policy-cli doctor: origin cleanup: zpcachyos/ROOT/cos/home/monochromatic-git-checkout@git-worktree-clone-origin-4f0c1d3ae5b28c6907df1a2b3c4d5e6f is owed; recorded 2026-11-22T18:38:41-05:00 before the clone zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets/add-cache-layer was destroyed; the next command that runs the cleanup attempts it.
```

It names the snapshot,
when the removal recorded it,
the clone it belonged to,
and the trigger that will act.
It names the slot's path on a second line when the path is text,
labelled as the old clone's slot,
because the slot may belong to another worktree by the time a person reads the line.

### The blocked lines

One line per reason,
and each names the fact that blocks the record,
the last attempt,
and the commands that show more.

```text
git monochromatic-git-policy-cli doctor: origin cleanup: <dataset>@<snapshot> is blocked by a dependency: the snapshot has 1 clone and 0 holds; the last attempt was 2026-11-22T18:31:04-05:00; `zfs destroy -n '<dataset>@<snapshot>'` names the datasets that depend on it and `zfs holds '<dataset>@<snapshot>'` names its holds; the next command that runs the cleanup retries it once nothing depends on it.
```

```text
git monochromatic-git-policy-cli doctor: origin cleanup: <dataset>@<snapshot> is blocked because it is marked for deferred destroy; the mark is not the wrapper's and the wrapper does not clear it; `zfs get -H -o name,value defer_destroy '<dataset>@<snapshot>'` reads it; the snapshot's owner destroys it, or an administrator removes the mark, and the next cleanup attempt finishes it.
```

```text
git monochromatic-git-policy-cli doctor: origin cleanup: <dataset>@<snapshot> is blocked because the pool's identity for the named objects is not the record's identity, or because the dataset has an origin of its own: nothing was destroyed and nothing will be destroyed from this record until the identities agree; compare the record at <path> with `zfs get -H -o name,value guid,createtxg,origin '<dataset>@<snapshot>'` and with `zfs get -H -o value guid,origin '<dataset>'`; this needs an administrator.
```

```text
git monochromatic-git-policy-cli doctor: origin cleanup: <dataset>@<snapshot> is blocked because the destroy was refused with errno 13; nothing else was destroyed; an administrator checks the account's delegated `mount,destroy` on the checkout and its descendants; the next cleanup attempt retries after that.
```

Each of the four is the record's own reason ("The state machine") plus the world's counts and the errno.
None of them quotes the record's text and none repeats a kernel message (F11 and L8).
The third line is also what a person reads when a promote left the record unprovable (F20 and L11),
and it says so in the words that name an origin.

### The unreadable and wrongly owned lines

```text
git monochromatic-git-policy-cli doctor: origin cleanup: <path> cannot be read as a record of this program, or is not this account's: nothing acts on it and it is left as it is; if the snapshot it names is already gone, remove the file, and if it is not, an administrator decides which snapshot it meant.
```

A record whose bytes do not parse,
or whose owner or mode is not the account's,
is the case where nothing can be proved and the file must not be overwritten ("The state machine").
Question five carries the wider choice about unreadable records.

### The litter line

```text
git monochromatic-git-policy-cli doctor: origin cleanup: the folder <path> holds 2 staging names from commands that were killed while they wrote a record; no reader reads them; no command removes them while a cleanup attempt may hold one; delete them by hand when no wrapper command is running, or leave them, and this report keeps naming them.
```

Staging names are `.<record name>.<random>.partial`,
and a live writer's name looks exactly like a killed writer's (F19 and the publication's shape).
This design therefore removes none,
and V8 weighs that against removing them unconditionally or by age.

### The unaccounted origin snapshot line

A third kind of line comes from the pool and not from a record:

```text
git monochromatic-git-policy-cli doctor: origin cleanup: <dataset>@git-worktree-clone-origin-9ab18c2de4f507b3a1c8d2e6f4b09a7c has no clone, no record and no live binding; nothing in this route will destroy it; confirm it is not another writer's snapshot of the same name and, if it is not, `zfs destroy '<dataset>@<snapshot>'` destroys it and the next report prints nothing.
```

It is found by listing the checkout dataset's snapshots,
keeping the names that begin with the wrapper's origin prefix,
and dropping every snapshot that has a clone,
a record,
or a name a live binding record carries.
The last of those is what an add in flight looks like (F24):
the binding records the snapshot's name before the snapshot exists,
so a snapshot with a live binding is an add and not litter.
`doctor` reads the binding design's folder for those names and nothing else,
and when that folder is absent it drops nothing for it.

A snapshot with a clone is some live worktree's origin.
A snapshot with a record is reported by the record's own line.
An add that fails destroys its own snapshot,
and one whose own destroy is refused has this design's record by then,
so the line meets neither as litter:
it names only a snapshot the route created and can no longer account for.
That is a different state from litter.
It is:
an add killed between the snapshot and the clone,
a record lost with a repository (L4),
or a record a person removed by hand.
The account can list these without privilege,
which is why the line costs no root channel.
It destroys nothing and names the command a person may run.
"V11 What doctor lists beyond the records" weighs the line.

### What a person can do, reason by reason

- A dependency by a clone.
  Remove that clone with an ordinary removal,
  or leave the snapshot and its space,
  and a later cleanup attempt finishes it.
  `zfs destroy -n` names the dependents without deleting anything (F12).
- A dependency by a hold.
  `zfs holds` names the holder,
  and F11 records that command in the kernel's own refusal text,
  and the holder releases it.
  The wrapper never releases another writer's hold.
- A deferred mark.
  The mark's owner destroys the snapshot or an administrator removes the mark;
  the wrapper does not clear it,
  and `zfs destroy -d` is the command that sets one (F3).
- An unproved or refused record.
  An administrator compares the record with the pool (the commands above)
  and decides whether the snapshot is the record's.
  A promoted snapshot (F20) is the ordinary cause,
  and the administrator decides whether the record is finished or kept.
  The wrapper does not guess.
- An unreadable or wrongly owned record.
  A person may destroy the snapshot it appears to name and then remove the file.
  Nothing else is safe,
  because a reader that cannot parse the file can prove nothing from it.
- Every blocked record.
  Any command that runs the lookup attempts it again,
  and the attempt prints one line naming the snapshot it destroys.

No step of that list asks a person to type a secret,
and no step needs a program to prompt (the credential rule at `:7345`).

### The settling command is the binding record's

A slot whose recorded clone is gone and whose absence cannot be proved is the binding design's case,
and the project decided that it gets a settling command a person confirms,
under the seven answers' heading "A slot whose recorded clone is gone and cannot be proved gone"
(`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md:77` to `:88`).
That record and its command are the binding design's,
and its round names the command and the guard it carries.
The command compares this design's record and never removes or rewrites it,
and refuses only when the record's fields disagree with the binding's;
it also removes a plan a cancellation kept,
and prints the snapshot names the plan held,
under the record's further answer on the settling command and a kept plan.
`doctor` prints that design's line and the command it names,
and this design designs no second command:
a person who runs it is acting on the binding record,
and this design's record is untouched by it.

### What doctor cannot show

A record in a repository whose path is gone,
and a record in a Git directory a person deleted,
are both beyond any command's reach (L4 and L5).
The unaccounted-snapshot line reaches the first of them whenever a repository at that path exists again,
and it reaches the second whenever the checkout dataset and its snapshots still exist.

## The tests a build must have

A test is listed here when it is needed to show a sentence of this design,
and each names the race or the refusal it forces.
The controls follow the package's test practices:
module tests in sidecar files,
and facts built apart from wording where a report is tested.

### How a control forces a point

The points this design names are forced in two ways and no third.

- A unit control uses the package's own test-only hooks,
  which exist in the test build only:
  `AFTER_THE_STAGING_FILE_IS_WRITTEN`
  (`foundation/private_storage_replacing_a_record.rs:93` to `:94`),
  the binding's fixture device
  (`kernel_device/fixture.rs` and `fixture_reply.rs`),
  the store's publication hooks
  (`worktree_clone_mount/slot_identity_store_publication.rs:507` to `:522`),
  and the seam the look offers between a record's read and its write.
- A guest arm runs the crate's own test binary in the throwaway guest.
  The test binary reaches the same hooks,
  so a guest arm that must be killed at a publication,
  at a destroy,
  or between a read and a write
  kills that process there with `SIGKILL`,
  and the arm asserts what the next command's look does.
  No production seam is added for a test,
  and a guest arm that cannot use the test binary asserts only what a real command shows.

### Controls that need no pool

T1 The record's bytes round-trip.
A record built from the fields in "The fields and what proves the origin later" encodes and decodes to itself,
and a rewrite with an unchanged outcome is byte-identical.
A second arm is an add's record that made no clone:
no `cloneGuid`,
`cloneDataset` the proposed name,
`startedAt` its own,
which decodes with the clone marked never made and encodes back unchanged.
Forced by construction:
two encodes of one value and a compare.
T2 The reader's refusals.
Each is its own control:
a first line that is not the header;
a version the reader does not know;
an unknown `state`;
an unknown `blockedReason`;
a missing `attemptedAt` in a blocked record;
a `blockedErrno` that is no decimal integer;
a JSON object with a key the record does not have;
a key out of order;
a GUID with a sign or a leading zero;
a `snapshot` field that differs from the file's name;
a file wider than 13,368 bytes;
and an empty file.
T3 A file name outside the grammar is never read.
A planted `notes.txt` and a planted `git-worktree-clone-origin-<31 hex>` are listed and skipped,
and the look reports them without opening them.
The seam is the folder and not a mock:
the entries are planted and the control asserts the look's own report.
T4 A wrong owner or mode is refused.
Each of the three is planted with one owner or mode the design rejects,
and the look reports it and destroys nothing.
T5 A killed publication leaves the old record and one staging name.
Forced with `AFTER_THE_STAGING_FILE_IS_WRITTEN`,
which fails the write after the bytes are staged and before the rename.
The old record reads exactly as before,
the staging name is the only new entry,
and the look skips it.
T6 A reader racing a rewrite sees one whole record.
Forced by running a read from inside the same hook,
so the read lands between the staged bytes and the rename,
which is the window a second command's look occupies.
The reader sees the previous whole record and never a prefix.
T7 The first write never replaces a record.
Two writers for one snapshot name publish at once,
by one exclusive create and one link that replaces nothing,
and one wins while the loser reports the existing record instead of overwriting it.
The build adds the same shape of test-only hook to this record's own publication function,
as the store's module has in `BETWEEN_THE_BYTES_AND_THE_LINK`
(`worktree_clone_mount/slot_identity_store_publication.rs:389`,
where it is called,
and `:510`,
where it is declared),
and the shared primitive keeps none,
so the seam lives in the module under control and not in `foundation/publishing_entries_in_one_open_folder.rs`.
T8 The attempt reads before it writes.
With the binding's fixture device,
a recorded identity that differs in GUID or in group is never followed by a destroy request,
and the record says `unproved`.
The control asserts the request sequence the fixture saw,
so the order is forced by the scripted answers and not by timing.
T9 The absence proof completes without a destroy.
The fixture lists a snapshot with the same name and another GUID under a dataset that passes step 2,
and the control asserts that no destroy was submitted and the record was removed.
T10 A dataset with a newer origin or a different GUID is never a completion.
Four arms:
an origin whose group equals the recorded one,
one whose group is above it,
one whose group is below it,
and another dataset GUID.
The boundary is the point,
so equality and one above and one below are three fixtures.
The fixture answers step 2 with each,
and the control asserts `unproved` and no destroy for equality,
for the group above and for the other GUID,
and that the group below reaches the listing.
The three origin arms are the Z2 control:
a guard that compared names would pass an origin named for another snapshot,
which is what a promote of a later clone leaves,
and a guard that refused every origin would refuse the older one.
T11 A dataset that is gone is `unproved`.
The fixture answers the dataset read with the kernel's missing answer,
and the record stays `unproved` with no destroy.
T12 The counts block before a request.
The fixture reports one clone and then one hold,
and the record says `dependency` with the count,
and no destroy request was submitted either time.
T13 The empty case lists nothing.
A control asserts that a look in a repository with no cleanup folder opens the folder and calls no listing,
which is the store's own "a run that must read nothing" seam
(`worktree_clone_mount/slot_identity_store.rs:51`).
T14 A refused gate or an unopenable device does nothing.
With the gate refusing,
and then with the device's open failing,
a look with a record standing prints one line and submits no request;
a look with no record prints nothing.
T15 A blocked record that stays blocked is not rewritten.
The fixture answers the same refusal twice and the control counts the record's writes:
one write for the first block and none for the second attempt.
T16 The `doctor` lines.
Each is built from facts,
and the ceiling's fallback line is a control of its own,
as the filesystem section's wording is tested now (F17).
T17 The resurrected record heals.
A control pauses attempt A after its read,
lets attempt B complete and unlink,
resumes A so it writes a refusal,
and asserts that the next look proves the absence and removes the record.
T18 The look never reads Git and takes no claim.
A control with a recording Git context
(`testing/recording_git.rs`)
asserts that no Git question is asked while a record is attempted,
and that no claim file is created.

### Controls that need a guest

G1 A real kill between the record's publication and the first destroy.
The crate's test binary is killed at the publication hook inside the throwaway guest,
and the next command's look sees `owed` and a live clone,
records `dependency`,
and destroys nothing.
G2 A real dependent clone blocks and then clears.
One clone of the origin snapshot exists;
the look records `dependency`;
the clone is removed by an ordinary removal;
and the next look destroys the snapshot and removes the record.
G3 A real deferred mark blocks.
The guest marks the snapshot with `zfs destroy -d`,
the look records `deferred`,
and the snapshot survives.
G4 A real name reuse completes.
The snapshot is destroyed outside the wrapper and one of the same name is created;
the look removes the record and destroys nothing,
and the new snapshot survives.
G5 A real new add over a slot with a pending record.
A pending record exists,
a new add takes the same slot (F15's sequence),
and the new clone and its snapshot survive the look while the old record stays blocked by its own dependency.
G6 A real promote leaves the record unproved.
Three arms.
The first promotes the clone that depends on the record's snapshot,
and the second promotes a later clone made from a newer checkout snapshot,
so the record's snapshot moves under an origin named for the later one (Z2).
In both the look records `unproved`,
destroys nothing,
the recorded snapshot survives under the promoted clone,
and `doctor` names the origin (F20 and L11).
The third promotes an earlier clone made from a snapshot older than the record's,
which must not be refused:
the record's snapshot stays on the checkout,
so the look reaches it and may destroy it.
G7 The account's delegated destroy of a checkout snapshot.
This is M4's arm run as a control:
the snapshot is destroyed by the account under the grants the wrapper really sets,
and the record completes.
G8 A kill during the destroy and a kill after the absence proof.
Two arms of one control:
the test binary submits one destroy and is killed before it reads the answer,
so the next look finds the snapshot in either state and settles by the proof;
the second kills it after the absence proof,
so the next look only removes the file.
G9 A power cut leaves the record.
The guest writes the record and cuts power before the first destroy,
and the record is there after boot (M6 and L8).

### What a green run shows and what it would not

A green run shows that every refusal and every state transition this design names happens for the reason it names,
that the empty case costs one open,
and that no record path reads Git,
takes a claim or waits on a person.
It does not show that the account's delegation covers the origin snapshot in the wrapper's real grant set (M4),
that the added cost is small (M1 and M2),
or that a real power cut preserves the record (M6).
Those are measurements,
not controls.

## Where this design and its siblings disagree

This round read the two sibling documents at the commits under "Sources read":
the plan design at `1921adf393` and the binding design at `7d041d773b`.
Neither is decided,
and this section records what each says at those commits,
whether this design now agrees,
and who could decide where no project record does.
It also says which statements an earlier revision of this section made about a sibling that the sibling has
since withdrawn,
so a reader of the second gate pass's X4 can see each one corrected.
The fourteen points are the review's,
in its order.

### 1. The folder family

All three now put their folder inside the wrapper's own folder in the common Git directory:
this design at `monochromatic-git-policy-cli/origin-cleanups/` (V1),
the plan design at `monochromatic-git-policy-cli/clone-destruction-plans/<clone GUID>` (F25),
and the binding design at `monochromatic-git-policy-cli/worktree-clones/v1/` (F24).
The binding design's convention asks one folder per family and a version either in the folder name or in the first
line (F24),
and all three meet it:
this design and the plan design carry a `#` version line,
and the binding design a `v1/` segment.
An earlier revision of this section weighed the binding design's old top-level
`monochromatic-git-policy-cli-worktree-clone/v1/`,
which that design withdrew in the same revision that moved it under the parent.
Only the inner names differ now,
and "V1 The home and the folder's name" weighs this design's inner name with theirs.

### 2. The key

The three keys differ and no record decides:
the plan design keys by the clone's dataset GUID (F25),
this design by the origin snapshot's name (V3),
and the binding design by the slot name (F24).
The binding design's convention says a uniform key is not possible and that the set reaches from any one key through
its own `cloneGuid=`,
`originSnapshot=` and `slot=` fields (F24),
which is true of the three files as they stand.
The plan design's Choice 2,
this design's V3 and the binding design's V2 list the alternatives.

### 3. The encoding of names and the shape of the file

The plan design and the binding design now write the same shape:
a `#` version line,
the ZFS names verbatim,
paths in hex,
GUIDs and transaction groups in canonical decimal,
and no 64-bit value in a JSON number (F24 and F25).
This design writes a `#` version line and the outcome lines,
then one JSON object holding the identities,
with the path as text when no escape is needed plus hex.
That is a real difference and not a misunderstanding:
the second gate pass's N9 asks for the option both siblings write to be listed,
and "V2 The record's shape" now lists it as its option D,
with this design's reason for not taking it.
The plan design's earlier claim that it "agrees with both siblings" was true of the binding design and not of this
one;
that sentence is the plan design's to correct,
and this design states its own side here.
An earlier revision of this section said the plan design writes ZFS names as hex and its numbers as lines,
which was true of an earlier revision of that document and is not true of `76849b8d9b`.

### 4. The moment of the origin carrier

All three now agree in substance and are one order:
this design publishes the record at step 6 of "The removal's one order of writes and removals",
the plan design confirms it at its step 6 and writes nothing there,
and the binding design confirms it at its step 2 and writes nothing there (F24 and F25).
No project record fixes the moment;
this design's V4 is the veto,
and both sibling documents say their confirmation depends on it.
What the second gate pass's X3 asked for was one list of the tail,
and that list is "The removal's one order of writes and removals".
An earlier revision of this section said the plan design writes the origin record after the registration's settle,
and that the binding design published it there too;
both statements were true of earlier revisions and neither is true of the commits named here.

### 5. Whether a blocked origin keeps the slot

The accepted words decide it:
"Settling frees the slot but requires recovery without its Git entry"
(the proposal `:1462` to `:1463`),
and settling does not wait for the origin (`:469` to `:470`).
This design states the rule in "A settled registration and a pending record".
The binding design's revision removed the binding whether or not the origin's cleanup is still blocked (F24),
so it agrees.
The plan design says the same in its two states:
no origin record stands after a successful attempt,
and the origin record stands blocked after a blocked one (`:1287` to `:1291`).
An earlier revision of this section said the binding design keeps its binding while the origin cleanup is blocked,
which that design withdrew,
and that the plan design's list lacked the "or none",
which it has since added.

### 6. The serializer of two removals

All three agree:
a removal takes the per-slot claim,
and the proposal decides it in one sentence,
"A per-slot claim only serializes cooperating wrapper operations"
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:909` to `:910`).
The plan design takes it for a removal (F25),
and the binding design owns it (F24).
This design's look takes no claim (V7),
because the two attempts it may run at once are idempotent and a claim would be litter.
An earlier revision of this section said the plan design rests its serialization on the Git lock,
which was true of an earlier revision of that document;
`76849b8d9b` takes the claim.

### 7. Durability

The three positions differ and are one rule by position ("V14 The flush rule"):
the plan design publishes unflushed (F25),
this design publishes unflushed (L8 and F21),
and the binding design flushes every write (F24).
No project record fixes one rule for the three.
This design's V14 carries the rule and the source reading,
and the earlier question about it is gone because a flush does make a rename durable (F27).
An earlier revision of this section said the plan design writes unflushed "on the wrapper's btrfs rule";
`76849b8d9b` argues on ZFS,
with the destroy's transaction group as its reason,
which is the same premise as F21.

### 8. What a reader does with a bad entry

All three refuse a partial,
a foreign and an oversized record and change nothing,
and all three now refuse a wrongly owned one (F24 and F25,
and this design's "What a reader refuses about owner and mode").
For an unreadable record all three refuse by default:
this design refuses and reports,
the plan design refuses and never moves a plan aside (F25),
and the binding design's folder has a `<slot>.unreadable` form for a record a reader moves aside (F24),
which is the copy journal's shape in the specification.
The project decided that a reader refuses an unreadable record and names it
(the seven answers,
under the heading "A record that cannot be read"),
so the plan design's Choice 10 and the binding design's Q7 retire.
An earlier revision of this section said the plan design refuses and never moves a plan aside,
which is still true,
and that the binding design "refuses and asks",
which is its Q7 and Q8 and not a single rule.

### 9. Who removes staging names

The three rules differ and the route's convention gives one:
a staging name is removed only when its writer is provably gone.
The plan design reads its `owner` line and removes a name only when that process is gone (F25),
the binding design reads whether the folder holds a live claim (F24),
and this design has neither an owner line nor a claim,
so the rule gives "never" here and this design reports and removes none (V8).
The store decision gives its own folder's staging names to step 18's `doctor` unconditionally (F23),
which covers the store and not these three folders.

### 10. Listing on the command path

The three differ by design:
the plan design stats its folder before every command and lists it only when it exists (F25),
the binding design reads its folder only in `doctor` and says no forwarded command reads it (F24),
and this design runs the look in the five worktree commands only,
which the project decided,
under the seven answers' heading "Which commands may destroy a leftover origin snapshot".
The store's "no listing per sweep" is about the store and decides none of these.
This design states its cost in L13 and L14 and sizes it with M1.
An earlier revision of this section said the plan design refuses a per-command listing and uses a pointer;
`76849b8d9b` has no pointer and stats its folder per command.

### 11. Who `doctor` runs as

All three now assume the account:
the plan design states it (F25),
this design assumes it (A4),
and the binding design says root may read its folder and no root-run mode exists (F24).
No project record decides it,
and nothing in the three designs needs root to read a record.
An earlier revision of this section said the plan design runs as root;
`76849b8d9b` assumes the account.

### 12. What ends each record and who removes it

The order is now one list:
"The removal's one order of writes and removals".
It ends the origin record by its absence proof (step 10),
the registration by the settle (step 11),
the binding's files by step 12,
the plan by step 13,
and the claim by step 14.
The plan design removes the plan after the head is gone and the registration settled,
and its finish pass proves the head's absence under the claim,
finishes step 12 and then removes the plan (F25).
The binding design removes its binding after the confirmation and the settle,
whether or not the origin cleanup is blocked (F24).
Both sibling designs cite the one list at `8cc31f59a4` with its fourteen step numbers,
and the binding design states the order among its three files and the condition a finisher meets,
with the no-plan arm named as having no finisher (F24).
An earlier revision of this section said the binding design keeps its binding when the origin cleanup is blocked,
which it withdrew.

### 13. Whether a removal has a record of its own

All three agree in substance:
the removal intent is the clone's `unmount` property,
the destruction plan is the plan design's record,
and this design and the binding design write no removal record of their own.
The plan design's Choice 6 covers the records a removal changes,
and the plan's and the proposal's "removal record" words name no second file.
An earlier revision of this section called that entry "its Choice 5";
`76849b8d9b` numbers it Choice 6.

### 14. Bounds and caps

All three derive their bounds from named terms with no rounding:
the plan design `10,128 + 340 * count` (F25),
this design 13,368,
and the binding design 43,027 (F24).
The plan design's ceiling is gone (F25):
its reader is bounded by the file's own length,
and only its header's seven-digit count is a width.
This design ships no cap on a count,
and the binding design ships none on its inventories.
An earlier revision of this section said the plan design ships "a derived bound plus a declared-count term and a
placeholder ceiling",
which was true of `76849b8d9b` and is not of `b4dc758a76`,
and it objected to rounding,
which none of the three now does.

### What the two sibling texts still say about this one

Nothing at the commits under "Sources read" is older than this design's tip:

- the plan design at `1921adf393` cites this list at `8cc31f59a4` under its two headings and with its fourteen step
  numbers,
  and its two earlier statements about this design's record shape and the claim's owner are corrected there;
- the binding design at its tip read last cites the same list with the same numbers,
  and states the order among its three files and the condition a finisher meets,
  including that the condition's no-plan arm has no finisher today.

The two earlier rounds' remaining stale sentences are therefore gone,
and this subsection stays as the place a later reconciliation records what it finds.

## Which build step builds which part

Step 16 builds most of this design,
because it is the step that owns the origin cleanup and the settle
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md:1862` to `:1947`).

The parts step 16 builds:

- The cleanup folder,
  its name grammar,
  its key and its modes,
  and the owner-and-mode rule of the reader.
- The order and the moment,
  with the record published at step 6 of that order.
- The fields,
  the bound and the identity proof,
  with the seven steps and the `unproved` answers.
- The states and the attempt's rules.
- The settle's order and the answer that a new add may take the slot.
- The look,
  its preconditions,
  its printing rule and its notices,
  in the five worktree commands the project decided.
- The controls T1 to T18,
  and the guest arms G1 to G9.

The parts step 18 builds,
because it is the step that builds `doctor`
(`:2017` to `:2060`):

- The owed,
  blocked,
  unreadable,
  wrongly owned,
  litter and unaccounted lines of
  "What doctor reports and what a person can do".
- The control T16.

Two steps build what this design reads and does not own.
Step 10 writes the add's binding record whose origin identities the removal copies (A1),
and steps 10 and 13 own the Git lock whose release settling performs (the proposal `:466` to `:468`).

The list's other points belong to the steps that own them:
the two deferred-destroy checks to step 15,
the forced request to step 17,
and the add that destroys what it made to step 11 (and to the dirty-main and binding designs that build its
fallback).
The look's five call sites are built where each command's own work is:
`remove` at step 16,
`add` at step 10 with the binding design's fallback,
and `list` and `prune` and `repair` where the wrapper forwards them,
which step 16's SPEC passage names.
This design states them so that the three records have one order,
and it designs none of them;
"What this design does not cover" says so for each.

Step 16's build writes three passages into `package/git-policy/cli/SPEC.md`:

- The removal section gains the cleanup record's folder,
  name grammar,
  record lines,
  states,
  bound,
  the owner-and-mode refusal and the other refusals,
  with the pointer to this design as the staging design's `SPEC.md` passages point to it.
- The same section gains the look,
  its preconditions,
  its notices,
  and its rules that it reads no Git,
  takes no claim and changes no exit status.
- The doctor section gains the lines and their rules
  (`package/git-policy/cli/SPEC.md:683` to `:770`).

The code belongs where the removal route lives,
under `package/git-policy/cli/src/native/worktree/`,
with the folder rules `LYD` and `LYL` applied to any folder it adds,
and `doctor`'s lines join the ZFS part of `health/doctor_filesystem.rs`.

The plan needs three amendments when this design is accepted:

- Step 16's "What it builds" gains
  "the origin cleanup record at its moment in the order,
  the look the five worktree commands make at it,
  and the place each of them runs it".
- Step 18's `doctor` list gains the record's lines and the unaccounted snapshot line.
- Step 16's acceptance tests gain the settle-while-blocked arm with a pending record.

## Measurements owed

Each measurement is unrun.
Each names the probe that would run it,
the arm that must be its control,
and what in this design would change if it came out one way or the other.

The timing measurements follow the repository's rule for comparing noisy work:
an interleaved control arm from the same binary in the same round,
a ratio with its range,
and no finding from a difference smaller than the run-to-run spread.

M1 What one command pays for the look.
Arms,
interleaved in one round on the acceptance workload's repository:
a command with no cleanup folder,
a command with one owed record whose snapshot is pinned,
a command with one blocked record,
and a control command with the look removed.
Measure wall time and the ioctl count (`strace -c -e trace=ioctl`) per arm.
This sizes L13 and the decided rule's cost for the five worktree commands.
If the added cost is above what the project accepts for those commands,
the record's answer would have to be revisited,
and it changes only the call site.
M2 What the absence proof costs on the checkout dataset.
The same round,
on a checkout dataset holding the snapshots the acceptance workload has
(25 own snapshots on this host's home dataset at 2026-11-22T17:22:37-05:00,
which is a size and not a cost).
Count the `snapshot_list_next` calls and time one complete listing,
and compare it with the same listing on a dataset with one snapshot.
M3 What the binding's destroy answers for the four cases.
In the throwaway guest,
through the binding and not through the `zfs` program:
a snapshot with one dependent clone (expect the kernel's `EEXIST` and a per-name list),
a held snapshot (expect `EBUSY`),
an absent name (expect success and no list),
and a name marked with `zfs destroy -d` and submitted without the `defer` key.
The same round reads one snapshot's statistics through `objset_stats`,
which no host run has done,
and which the proof's first step needs.
This tests F4 and F5 through the code this design uses,
and it decides whether `blockedErrno` can carry an errno in each case.
M4 The account's delegated destroy of a checkout snapshot,
under the grants the wrapper really sets.
The source reading in F22 answers the first half from the kernel:
a snapshot is checked as a descendant,
so the descendants-only grant covers it.
What only a guest can show is that the grant set the build really issues is the planned one,
that the request succeeds,
and which refusal each missing half produces.
The arm writes the grant set the build issues,
destroys one checkout snapshot as the account,
and repeats with the `mount` half and the `destroy` half removed.
This decides A2 and nothing in the design's wording.
M5 The deferred mark's read and the refusal path.
In the throwaway guest:
mark a snapshot with `zfs destroy -d`,
read `defer_destroy` through `objset_stats` and through the `zfs get` command,
and submit one destroy without the `defer` key.
This is shared with the sibling design of the deferred-destroy checks,
and it decides nothing here beyond the `deferred` reason's read (A3).
M6 Whether a record written before the first destroy survives a power cut,
and whether a flush buys what V14 says it buys.
This is the guest check of "V14 The flush rule" and of the one-pool premise L8.
Two arms:
one writes this design's record and a plan's `requested` outcome unflushed and cuts the guest's power before the
destroy,
and one writes a binding after its clone exists,
flushed and unflushed,
and cuts power after each.
Each arm proves the record or its absence after boot,
and the second shows whether the flushed write survives where the unflushed one may not (F27).
If the guest cannot cut power honestly,
the arm is a process kill and the power-cut half stays a documented limit.
M7 What the record's own write costs the removal.
In one interleaved round:
a removal arm with the record's publication,
and a control arm without it,
on the same pool and the same clone shape.
This is owed because the write is on the path a person waits on.

## Choices this design marks for the project's veto

Each entry below is a choice the project's records do not decide.
This design adopts one option in each so that it is whole,
and a veto of the adopted option replaces it with another whole option.
Every example names an input and what a person would see for it.
V9 is not listed:
the proposal decides that settling frees the slot
(`doc/planning/cli-git-zfs-worktree-design-proposal.md:1462` to `:1463`),
so the slot's freedom is stated in "A settled registration and a pending record" and not put here.

### V1 The home and the folder's name

The adopted option is A.
The parent is agreed with the two sibling designs,
so the choice below is the parent,
the inner name and the store together.

- A The wrapper's own folder in the repository's common Git directory,
with the record's folder `origin-cleanups` directly under it.
  Example.
  A blocked removal in `/home/user/Monochromatic` leaves
  `.git/monochromatic-git-policy-cli/origin-cleanups/git-worktree-clone-origin-4f0c1d3ae5b28c6907df1a2b3c4d5e6f`,
  and a worktree command of that repository prints the cleanup line.
  Pros:
  no privilege beyond the account's;
  found from any worktree without a machine-wide scan;
  survives the registration's settling and the slot's reuse;
  a command that finds no folder pays one open;
  one level below the wrapper's folder,
  with no dependence on the binding design's inner names.
  Cons:
  the record dies with the repository (L4 and L5);
  the account can rewrite it (L3);
  a repository moved to a new path takes its records with it and a repository deleted by hand takes them away.
- B The same parent with the route's families under one version segment:
  `<common>/monochromatic-git-policy-cli/worktree-clones/v1/origin-cleanups/`.
  Example.
  A person lists `worktree-clones/v1/` and sees `add-cache-layer.binding`,
  `add-cache-layer.origin-inventory`,
  a folder for this record and the plan's folder under one version.
  Pros:
  the route's records share one versioned family inside the wrapper's folder;
  a later grammar cannot be read as this one;
  the parent the rename machinery moves is unchanged.
  Cons:
  this record's folder depends on the binding design's inner names;
  a person must open two levels to reach it;
  two accounts of one route's records are told apart only by a folder name.
- C A root-owned store beside the slot identity store:
  `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-origin-cleanups/`.
  Example.
  The same blocked removal leaves that root-owned path,
  a person reads it with `sudo ls /var/lib/monochromatic-git-policy-cli/`,
  and an administrator's adoption moves it as the store's files are moved.
  Pros:
  outlives the repository;
  the account cannot rewrite it;
  one machine-wide place for `doctor` to list.
  Cons:
  the writer is the account but the folder is root's,
  so a root writer and channel have to exist after the clone is destroyed and none does;
  the destroy authority is still the account's,
  so root ownership protects only against a hostile account the threat model does not have;
  it needs a new `doctor` read as root.
- D The account's cache:
  `~/.cache/monochromatic-git-policy-cli/origin-cleanups/`.
  Example.
  The same record appears under the cache,
  and a `systemd-tmpfiles` run that cleans the account's cache removes it while the snapshot stays.
  Pros:
  account-owned like the staging decision's record;
  no root channel.
  Cons:
  a cache is a folder a system may drop;
  it is one more place a person must know about;
  losing it loses the only memory of the snapshot.
- E A user property on the origin snapshot:
  `org.monochromatic:git-worktree-clone-origin-cleanup`.
  Example.
  `zfs get -H -o name,value org.monochromatic:git-worktree-clone-origin-cleanup '<dataset>@<snapshot>'`
  prints the record,
  and destroying the snapshot destroys the record with it.
  Pros:
  the record lives exactly as long as its subject;
  completion needs no removal step at all;
  no folder to walk.
  Cons:
  writing it needs a `userprop` grant the project never decided;
  a property has its own size bound and value grammar;
  it cannot record a cleanup whose snapshot a person already destroyed outside the wrapper;
  and the project's delegation decision names `mount` and `destroy` and nothing else.

The ranking is A then B then C then D then E.
A over B:
A is one level and depends on no sibling's inner names,
while B shares one versioned family at the cost of that dependence.
B over C:
B needs no privilege the route has not got,
and after the clone is destroyed the route has no root-side writer to give C its record.
C over D:
C is durable state an administrator owns,
while D is a cache a system may drop by policy.
D over E:
D needs no new grant and no property grammar,
while E needs a delegation the project did not decide and a value bound nothing here sizes.

### V2 The record's shape

The adopted option is A.

- A Short `key=value` lines and one JSON object,
as the leaving record already reads,
with ZFS names verbatim,
paths as text only when no escape is needed plus hex always,
and GUIDs as decimal strings.
  Example.
  A person runs `cat .git/monochromatic-git-policy-cli/origin-cleanups/git-worktree-clone-origin-4f0c1d3ae5b28c6907df1a2b3c4d5e6f`
  and reads `state=blocked` and `blockedReason=dependency` before the long JSON line.
  Pros:
  the state reads first for a person;
  a header line carries the version;
  the JSON writer and reader already exist;
  it matches the leaving record and the binding design's header line.
  Cons:
  two grammars in one file;
  a reader must reject a key out of order.
- B One space-separated line,
as the recorded identities store writes,
with ZFS names as hex and numbers as fields.
  Example.
  The same record reads
  `cleanup v1 owed 12039048377182978123 zpcachyos/...@git-worktree-clone-origin-4f0c... 3745620968172523788 93823 ...`
  on one long line.
  Pros:
  one grammar;
  a person's `grep` finds a field;
  no escape question.
  Cons:
  a path with a space needs an escape the store never needed;
  the state hides in the middle of a long line;
  the JSON writer would go unused;
  hex names are longer than the ZFS names every other record in this route writes verbatim.
- C One JSON object with everything.
  Example.
  The same record reads `{"version":1,"state":"owed","attemptedAt":null,...}` in one line.
  Pros:
  one grammar;
  trivially parsed.
  Cons:
  a person reads the state after the longest fields if the order is not chosen;
  nothing to check the file's first bytes against;
  a truncated file parses as nothing and gives no version.

- D All `key=value` lines,
as the plan design writes and as the binding design's convention says,
with ZFS names verbatim,
the path in hex and GUIDs as decimal lines.
  Example.
  The same record reads
  `# monochromatic-git-policy-cli origin cleanup, version 1`,
  then `state=owed`,
  then `originDatasetGuid=12039048377182978123`,
  then `originSnapshotGuid=3745620968172523788`,
  then `slotPathHex=2f686f6d65...`,
  one key per line.
  Pros:
  one grammar for the three records,
  which is the route's own convention;
  a person's `grep` finds a field;
  no JSON object to keep in a fixed key order.
  Cons:
  the immutable identity and the mutable outcome share one key space,
  so a reader must refuse a missing or out-of-order key among twenty;
  the path's text form is gone and a person reads only the hex field;
  the leaving record's shape,
  which this design cites as its precedent,
  is then only this design's.
  This is the option both sibling designs now write,
  and this design does not take it:
  its reason for the JSON object was that a 64-bit GUID cannot travel as a JSON number,
  and that reason falls away for D,
  whose GUIDs are decimal lines.
  What remains is the leaving record's shape and the one place a rewrite can compare byte for byte,
  and the second gate pass's N9 asks for the option to be listed beside them.

The ranking is A then D then C then B.
A over D:
A keeps the immutable identity in one object that a rewrite re-emits byte for byte,
and it follows the leaving record's shape,
while D flattens both kinds of field into one key space a reader must police for order and presence.
D over C:
D writes ZFS names verbatim and keeps the path out of the grammar,
and it is what both sibling designs write,
while C escapes the path into one JSON line.
C over B:
C needs no escaping for a path with a space and no new field grammar,
while B would need a new escape rule the store's grammar does not have.

### V3 The name grammar and the key

The adopted option is A.

- A The origin snapshot's name.
  Example.
  `ls .git/monochromatic-git-policy-cli/origin-cleanups/` prints
  `git-worktree-clone-origin-4f0c1d3ae5b28c6907df1a2b3c4d5e6f`,
  which is the snapshot's own name.
  Pros:
  one key that is already unique and already in the pool;
  the nonce is already the add's;
  the name is what a person compares with `zfs list`.
  Cons:
  the name must be refused when it does not match the grammar;
  a name that a person recreates is a different snapshot under the same key.
- B A fresh record name of `<32 hex>.record` with the snapshot inside.
  Example.
  `ls` prints `9d21e0f7a4b3c2d1e0f9a8b7c6d5e4f3.record`,
  and the snapshot's name is inside the file.
  Pros:
  no relation between a name and a pool object;
  a person cannot mistake it for a snapshot.
  Cons:
  a second key nothing needs;
  a reader must open the file to know which snapshot it is about;
  the pool holds no name to compare it with.
- C The slot's name,
as the binding design keys its record.
  Example.
  `ls` prints `add-cache-layer`,
  and the next add at that slot overwrites the file.
  Pros:
  a person finds it by the slot they know;
  all three records could be found from one name.
  Cons:
  the slot is reused by the next add (F15);
  a record must survive exactly that reuse;
  one snapshot's cleanup would erase another's.

The ranking is A then C then B.
A over C:
A's key is unique for the life of the snapshot and survives the slot's reuse,
while C's key is reused by the very add the record must outlive.
C over B:
C is a name a person and a reader already hold,
while B adds a key that nothing in the pool can confirm.

### V4 The moment of the first write

The adopted option is A.

- A Immediately before the first destroy request,
after every refusal that must precede a delete.
  Example.
  A removal killed during the clone's snapshot batch leaves a record saying `owed` beside its live clone,
  and the next look records `blocked` with reason `dependency`.
  A removal that refuses at the deferred-destroy check leaves no record at all.
  Pros:
  no refusal leaves a record beside a live clone;
  the record exists at every point from the first destroy onwards;
  the tuple was just proved and copied from the binding.
  Cons:
  the removal pays a durable write before its first destroy;
  a kill between the write and the first destroy leaves a record that blocks on its own live clone.
- B Before the removal's refusals,
just after the unmount.
  Example.
  A removal whose origin is marked for deferred destroy refuses,
  and leaves a record saying `owed` beside a live unmounted clone,
  which every look then blocks on its own clone.
  Pros:
  the record is written earlier;
  one fewer point to reason about.
  Cons:
  every refusal and every auto-cancel leaves a record that is not owed (O2);
  `owed` stops meaning what it says;
  every look pays a listing for it until the clone is removed.
- C After the head destroy.
  Example.
  A removal killed between the head destroy and the settle leaves the registration and no record,
  and the tuple exists only in the destruction plan.
  Pros:
  a record exists only when nothing depends on the snapshot;
  no record for a removal that never destroyed anything.
  Cons:
  the tuple depends on another record's home and lifetime;
  a kill in that window leaves a leak nothing records (X1).
- D At add time,
with the binding record.
  Example.
  Every healthy worktree has a record saying `owed`,
  and `doctor` prints a pending cleanup beside each one.
  Pros:
  the record exists even if the removal never runs.
  Cons:
  `owed` would be true for every live clone;
  the report is noise for every worktree;
  the record's absence stops meaning anything.

The ranking is A then C then D then B.
A over C:
A's tuple never depends on a record this design does not own,
while C's does in the one window that matters.
C over D:
C never writes a record for a healthy clone,
while D makes `owed` true everywhere.
D over B:
D's records at least mean something while they stand,
while B writes records for removals that refused and destroyed nothing.

### V5 The identity fields

The adopted option is A.

- A The dataset GUID,
the snapshot GUID,
the snapshot's creation group,
the names and the clone's identity,
with the dataset's live `dds_origin` read at every attempt.
  Example.
  A live snapshot whose name matches and whose GUID differs makes the attempt record `unproved`,
  and nothing is destroyed.
  A dataset that gained an origin through a promote does the same.
  Pros:
  both identities the accepted rule names;
  the group ties the snapshot to the add's record;
  the promote signature is read rather than assumed;
  the comparison a later reviewer can check field by field.
  Cons:
  more fields to encode and to refuse;
  a repair outside the wrapper can invalidate a record that one GUID would still match.
- B The snapshot GUID alone.
  Example.
  A live snapshot with the recorded GUID is destroyed,
  and the dataset is not compared.
  Pros:
  the smallest proof;
  the fewest refusals.
  Cons:
  the accepted rule proves the dataset too;
  a mis-filed record with a valid GUID from another dataset would pass;
  a promote's moved snapshot is indistinguishable from a deleted one.
- C The dataset GUID and the snapshot GUID,
without the group.
  Example.
  A snapshot whose GUID and dataset match is destroyed,
  and a group that differs changes nothing.
  Pros:
  both accepted identities;
  one fewer field.
  Cons:
  the add's group is lost;
  the record cannot say when the snapshot was made in the pool's own order.

The ranking is A then C then B.
A over C:
the group and the live origin check are what separate a promoted snapshot from a destroyed one (F20),
and C would complete on a promote.
C over B:
the dataset half states that the snapshot belongs to the named dataset,
which a GUID alone leaves implicit.

### V6 The state vocabulary

The adopted option is A.

- A Two stored states,
with completion proved by the record's absence.
  Example.
  A removal whose origin is free destroys the snapshot and leaves no file,
  and `ls` in the folder prints nothing for it.
  Pros:
  no tombstone to reap;
  a killed writer can never leave a completed lie;
  the file's existence means work is still owed.
  Cons:
  a person cannot see history of what was cleaned up;
  the only evidence is the line the command printed.
- B A stored `completed` state that a later pass removes.
  Example.
  The same removal leaves `state=completed`,
  and `doctor` prints one line for a tombstone it would remove.
  Pros:
  a person can read what happened;
  a completion has a durable record.
  Cons:
  every tombstone is a record `doctor` must report and something must remove;
  the file's existence stops meaning work is owed.
- C Three stored states:
  `owed` or `attempting` or `blocked`.
  Example.
  A command killed mid-attempt leaves `attempting`,
  and a later reader must decide whether that attempt still runs.
  Pros:
  an in-flight attempt is visible;
  a second command could skip it.
  Cons:
  `attempting` needs a liveness rule the leaving record has only for its owner line;
  the attempt is idempotent and safe to repeat.

The ranking is A then B then C.
A over B:
no tombstone is created and the file's existence stays meaningful,
while B makes `doctor` reap records that mean nothing is owed.
B over C:
a tombstone is conclusive,
while C needs a liveness decision this design does not have.

### V7 Serializing attempts

The adopted option is A.

- A No claim.
Each attempt re-reads the identity and acts only on a match,
and two attempts are idempotent.
  Example.
  Two commands attempt one owed record at once:
  both print the line naming the destroyed snapshot,
  and one of them found the snapshot already gone.
  Pros:
  no folder of claim files;
  no litter the claim module never removes for its newest generation;
  no waiting between commands;
  rule `NLK`'s preference for lock-free work.
  Cons:
  both commands may submit a destroy of the same name;
  a writer that read an old record may re-create a blocked record once,
  and the next look removes it (T17).
- B The shared lock-free claim per record.
  Example.
  The same two commands:
  the second prints nothing,
  and the record is attempted again on the next trigger.
  Pros:
  exactly one attempt at a time;
  one report per attempt;
  the claim's own liveness rule already exists (F18).
  Cons:
  a second folder keyed by every record ever attempted;
  the claim module keeps the newest generation of each on purpose;
  a claim is a lock in the smallest unit and rule `NLK` allows one only where it cannot be avoided.

The ranking is A then B.
A over B:
the effort is idempotent and the only cost of a race is one extra look,
while B adds a durable folder whose entries nothing removes on purpose.

### V8 The staging names doctor removes

The route's convention gives one rule for the three folders:
a staging name is removed only when its writer is provably gone,
and each record's evidence is its own.
The plan design reads its `owner` line and removes a name only when that process is gone,
and the binding design reads whether the folder holds a live claim (F24 and F25).
This design's folder has neither a claim nor an owner line,
so the shared rule gives "never" for it,
which is the adopted option.
The options below are what this design could add to that rule.

The adopted option is A.

- A Report them and remove none.
  Example.
  A killed writer leaves `.git-worktree-clone-origin-4f0c....<random>.partial`,
  `doctor` prints its name and says to delete it when no command runs,
  and the look never reads it.
  Pros:
  a live attempt's staging name is never removed under it;
  no second rule needs the writer's liveness;
  nothing a person did not ask for is deleted.
  Cons:
  a killed writer's name stays until a person removes it;
  the report repeats on every `doctor` run.
- B Remove them unconditionally,
as the store decision gives to step 18's `doctor` for the store.
  Example.
  Two `doctor` runs while a slow removal is publishing leave the removal's rename failing once,
  and its record write is retried or the removal refuses.
  Pros:
  no litter at all;
  one rule for the store and this folder.
  Cons:
  a live writer's file is removed under it;
  the store's writer holds no claim and its decision covers the store (F23);
  a cleanup attempt's write fails for a reason a person caused.
- C Remove them only when older than a bound.
  Example.
  A staging name older than the bound is removed;
  a younger one is left.
  Pros:
  litter is bounded;
  a live writer is unlikely to be affected.
  Cons:
  the bound is a number nothing measured (rule `LQM`);
  a slow pool or a long queue can still race it.

The ranking is A then C then B.
A over C:
A needs no unmeasured number and no race,
while C's bound is exactly the number this design refuses to invent.
C over B:
C at least cannot remove a file a live writer wrote a moment ago,
while B can.

### V10 A person's way out of a record that cannot clear

The adopted option is A.

- A Nothing new.
`doctor` prints the commands that show the cause and the command that destroys the snapshot,
and the record clears itself on the next attempt after the absence.
  Example.
  A record is `unproved` after a promote;
  a person reads the origin with `zfs get origin`,
  decides the snapshot is the one the record names,
  destroys it,
  and the next attempt finds the dataset's GUID matching and the snapshot absent.
  Pros:
  no new command to design,
  test and guard;
  the record ends by proof and not by assertion;
  the same path works for every reason.
  Cons:
  a person must read and act;
  an unreadable record needs a person to remove a file by hand;
  no command records why a person abandoned a record.
- B A wrapper command that abandons a record.
  Example.
  `git monochromatic-git-policy-cli abandon-origin-cleanup '<snapshot>'` removes the file
  and prints that the snapshot stays.
  Pros:
  one documented action;
  the wrapper can print exactly what is given up.
  Cons:
  a new destructive command with its own guard and tests;
  it is a way to forget a snapshot that may still be pinned;
  nothing in the project's records asks for it.
- C Expiry after a fixed age.
  Example.
  A record older than thirty days is deleted and its snapshot is forgotten.
  Pros:
  no person is needed;
  the folder does not grow.
  Cons:
  the age is a number nothing measured;
  it silently forgets space that may be large;
  it needs a pass that notices time,
  which triggers alone do not provide.

The ranking is A then B then C.
A over B:
A needs no new destructive command and ends only by proof,
while B would let a person assert a cleanup away.
B over C:
B's act is explicit and can say what it gives up,
while C forgets by a number nobody measured.

### V11 What doctor lists beyond the records

The adopted option is A.

- A The unaccounted origin snapshot line,
report only.
  Example.
  `doctor` prints that `<dataset>@git-worktree-clone-origin-9ab1...` has no clone,
  no record and no live binding,
  and prints the `zfs destroy` command a person may run.
  Pros:
  a lost record or a killed add becomes visible;
  it reaches a record lost with a repository (L4);
  it destroys nothing on a name alone.
  Cons:
  one listing of the checkout's snapshots when `doctor` runs;
  a snapshot another writer made under the same prefix would be named;
  it reads the binding design's folder for live names,
which no record asks a report to do.
- B Nothing beyond the records.
  Example.
  `doctor` prints nothing for that snapshot,
  and it stays until a person lists the pool by hand.
  Pros:
  the report says only what a record proves;
  one less listing.
  Cons:
  a lost record is invisible;
  the space stays with nothing pointing at it.
- C The line plus a command that destroys them.
  Example.
  `git monochromatic-git-policy-cli doctor --remove-orphan-origins` destroys the snapshot the line names.
  Pros:
  the space comes back without a person typing a ZFS command.
  Cons:
  it destroys on a name and a missing record,
  which the proposal forbids;
  a snapshot another writer made under the same prefix would be destroyed;
  a `doctor` run would stop being read-only.

The ranking is A then C then B.
A over C:
A shows the snapshot and lets a person confirm its identity,
while C destroys on a name and a missing record.
C over B:
C at least names what it acts on and can be refused,
while B leaves the space invisible to every command.

### V13 The tail's order

The accepted design fixes the steps before this choice:
the origin attempt is its step 4 and the settle its step 5,
quoted in "The removal's one order of writes and removals".
What is left is which of the two records goes first after the settle,
and when the claim is released.
The adopted option is A.

- A The binding's files first,
then the plan,
then the claim.
  Example.
  A removal settles the registration,
  removes `<slot>.binding` and the two inventories,
  then unlinks the plan,
  then releases the claim.
  Pros:
  the plan is the last witness of what was destroyed,
  so a kill leaves exactly one record that names the removal's work;
  the state a binding with no plan and no registration leaves,
  which the binding design reads as a clone destroyed outside the wrapper,
  cannot arise from an ordinary kill;
  a finish pass that finds a plan finishes the binding first and then the plan.
  Cons:
  a failed binding removal stops the tail,
  so the plan stays until a later run proves the binding's files gone.
- B The plan first,
then the binding's files,
then the claim.
  Example.
  The same removal unlinks the plan first,
  and a kill before the binding's removal leaves no registration,
  no plan and no origin record on the normal path,
  with a standing binding.
  Pros:
  the plan,
  which records a finished removal,
  is gone before anything can act on it again.
  Cons:
  the standing binding is exactly the state the binding design's third reading calls a clone destroyed outside the
  wrapper,
  so a later reader keeps the binding and may refuse the next add at the slot;
  a normal kill then reads as a foreign event.
- C One combined step that removes the plan and the binding's files and reports success only when both are gone,
with the claim released after it.
  Example.
  A kill inside the step leaves one of the two.
  Pros:
  one function to read and one control to write.
  Cons:
  two unlinks cannot be one operation,
  so a kill still lands between them and the order is hidden rather than removed;
  a reader needs both states' rules anyway.

The ranking is A then C then B.
A over C:
A names the one last witness,
so a kill leaves one known state to finish from,
while C hides the order and leaves either state.
C over B:
B's kill state is the one the binding design reads as an outside destruction,
while C at least handles both states in one place and lets no ordinary kill look foreign.

### V14 The flush rule

The adopted option is C.
This is the rule the three record designs share by position,
and the wording the plan design's Choice 5 and the binding design's V6 and Q9 can cite.

What a flush buys is a source reading and not a question (F27):
directory intents are synchronous by default,
so a flush of a file and its directory makes a rename durable at `sync=standard`,
and the earlier claim that a rename is logged only at `sync=always` is wrong.
The plan design and this design write unflushed;
the binding design flushes every write.

- C By position:
no flush for a write made before an irreversible ZFS step,
which is this design's record and the plan's `requested` and `head=requested` writes;
a flush for a write made after an object exists that nothing else can prove,
which is the binding's first write.
  Example.
  Power is cut right after the origin record says `owed`.
  The destroy has not returned,
  so the record is on disk when it does (F21),
  and the next look proves the snapshot there or gone.
  Power is cut right after the binding's first write:
  the clone may exist with no record naming it,
  so that write is flushed.
  Pros:
  one premise (L8);
  a flush is paid only where a loss would leave an object nothing can prove;
  the three designs already do exactly this.
  Cons:
  two rules to keep,
  and the premise needs the guest arm of M6.
- A Flush every write.
  Example.
  Every outcome write waits for a log commit,
  and a power cut after any write leaves a record naming the object it described.
  Pros:
  one rule;
  the binding's orphan case is protected;
  a flush does make the name durable (F27).
  Cons:
  a log commit on every outcome write,
  including the writes whose loss costs one repeated attempt;
  this design's first write sits on the removal's path a person waits on.
- B No flush anywhere.
  Example.
  The binding's first write can be lost while the clone stands,
  leaving a clone that no record names.
  Pros:
  no write waits for a log commit.
  Cons:
  an orphaned clone with no record is the one loss nothing can prove;
  the plan and this design already refuse to take that risk for the binding.

The ranking is C then A then B.
C over A:
C spends a log commit only where a lost write leaves an object nothing can prove,
and A pays it on outcome writes whose loss is one repeated attempt.
A over B:
A protects the binding's orphan case at the price of a log commit,
while B leaves that case open.

M6 is the guest arm either way:
a record written before a destroy must survive a power cut without a flush,
and a flushed binding write must survive where an unflushed one may not.

## The seven answers the project gave

The project answered the one set of questions the three record designs shared,
in `doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
accepted 2026-11-22 and recorded at 2026-11-22T21:16:30-05:00.
All seven answers are here as decided sections,
with what this design does about each,
and the options not chosen are listed once at the end in the record's own words.
They leave this document's question list:
nothing remains for the owner to decide here,
and this design's remaining open choices are its veto entries.

The record decides the first three for this design's own behaviour,
so "Who carries the cleanup and when" states them:
which commands run the look and where in each,
what the look and `doctor` print,
and when a pending record's line,
or the route's gate refusal,
prints.
The fourth is this design's own reader rule.
The last three are records of the two sibling designs,
and this design cites them and designs no second.

### The commands that may destroy a leftover origin snapshot

The record's answer,
headed "Which commands may destroy a leftover origin snapshot"
(`:22` to `:40`),
is the "B" option it names "worktree commands only".
in its words:
"Only `git worktree add`,
`remove`,
`list`,
`prune` and `repair` destroy it,
printing one line;
`doctor` stays read-only and prints the `zfs destroy` command."

This design states it in two places.
"Where in each command the look runs" gives each of the five commands its place,
with the removal's own attempt at step 10,
the add's at the end of its work or at its fallback's origin-snapshot step,
and the other three at the end of their own work.
"Who carries the cleanup and when" states that no other command of the repository runs the look,
so `git status` prints nothing and destroys nothing,
and that `doctor` prints the `zfs destroy` command and destroys nothing.
The cost the record states with the answer is that a repository which never runs one of the five keeps the
snapshot and its space,
and L6 states it.

### What trace outlives the removal

The record's answer,
headed "What trace of a destruction outlives the removal"
(`:42` to `:51`),
is the "A" option it names "the printed report only".
in its words:
"Nothing beyond what the removal printed,
and its records while they stand."

This design prints the destruction's line at step 10,
keeps no audit line,
and opens no second store:
the record while it stands is the only record,
and `doctor` prints nothing for a record whose cleanup finished.
A person who missed the line has the record until the snapshot is gone and nothing afterwards,
which is the cost the record states.

### How often a pending record speaks

The record's answer,
headed "How often a pending origin cleanup speaks"
(`:53` to `:60`),
is the "A" option it names "on a change only".
in its words:
"One line when it destroys,
or when the reason it is blocked changes;
silence while nothing changed."

This design states what a reason is and when it changes in "What the look prints":
the reason is the code with the detail the line names,
and the detail is the counts for a dependency,
the mark for a deferral,
the errno for a refusal,
or the failing step of the gate.
The gate case is answered by the same rule,
which "When the route's gate refuses" states:
the gate's refusal is the record's reason while it refuses,
the failing gate step is the detail,
and the line prints when the record first meets the gate and when the failing step changes,
nothing while the same gate step keeps refusing.
An unreadable record's line follows the same rule:
one line when the record first cannot be read,
and nothing while it stays the same.

### A record that cannot be read

The record's answer,
headed "A record that cannot be read"
(`:62` to `:75`),
is the "A" option it names "refuse and name the file".
in its words:
"Change nothing,
name the file,
and wait for a person."

This design refuses it in the look,
names it in the look's line and in `doctor`'s unreadable line,
never moves it,
never rewrites it and never removes it,
and leaves a person the way out that "The state machine" and "The unreadable and wrongly owned lines" state.
That is the record's answer and not a choice of this design.

### A slot whose recorded clone is gone and cannot be proved gone

The record's answer,
headed "A slot whose recorded clone is gone and cannot be proved gone"
(`:77` to `:88`),
is the "B" option it names "offer a settling command",
in its words:
"Print an explicit command that removes the record after you confirm."
The recommended option was to keep the slot blocked and report;
the project did not take it.

This is the binding design's record and command,
not this design's:
the record's "What this record does not decide" leaves the command's name,
its guard and what it proves to that design (`:134` to `:138`),
and that design's "The settling command" is where its round names them.
Its second answer of that round decides what the command does with a plan a cancellation kept:
it "prints the snapshot names the plan recorded as destroyed,
then removes the plan with the other records and frees the slot"
(`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
the section on the settling command and a plan a cancellation kept),
and the binding design states that.
What the command does with this design's record is stated there and not here:
it compares the record and never removes or rewrites it,
and refuses only when its fields disagree with the binding's.
This design states only its own half:
a record whose absence cannot be proved stays `unproved`,
its look never settles on a guess,
and `doctor` prints the binding record's line and the settling command that design names.
This design designs no second command,
and a person who runs the settling command is acting on the binding record,
not on this one.

### How long the plan a cancellation keeps stays

The record's answer,
headed "How long the plan a cancellation keeps stays"
(`:90` to `:101`),
is the "A" option it names "until that clone's next removal".
in its words:
"The next removal of that clone reads the record,
proves it,
carries the loss forward and ends it."

The plan is the plan design's record.
A kept plan says `head=present` or `refused` or `unknown`,
so it is never a finisher's plan,
and this design's look never reads a plan.
The next removal of that clone resumes it,
which "The removal's one order of writes and removals" states at its steps 4 and 5 and 9.
This design cites the answer and designs nothing for it.

### Whether a forced removal records who it ended

The record's answer,
headed "Whether a forced removal records who it ended,
beyond the journal"
(`:103` to `:110`),
is the "A" option it names "the journal alone".
in its words:
"The handler's journal lines are the record."

This design's record names snapshots and their identities and nothing about processes,
and the request at step 2 of the order ends nothing itself:
the handler does the ending,
and its journal is where a person reads it.
This design adds nothing,
and the forced-removal design states the same.

### The options the project did not choose

The record lists them once,
in its "What was not chosen" (`:112` to `:132`),
and this document does not repeat them elsewhere:

- Any wrapped Git command,
  only by hand,
  and a user timer,
  for the leftover snapshot.
- An audit-log line,
  a notice on later commands,
  and a history `doctor` prints,
  for the trace.
- A line on every attempt,
  and nothing outside `doctor`,
  for the pending cleanup.
- Moving an unreadable record aside,
  and dropping it after one line.
- Keeping a slot blocked with no way to settle it.
- Keeping the plan until a person acknowledges it,
  and for a fixed time.
- The handler's receipt naming the processes,
  a count in the plan record,
  and a record of its own,
  for a forced removal's ended processes.
- The add owning the path alone,
  with no owed-cleanup record and the worktree pending until the same add is run again,
  for a refused snapshot destroy after a failed add
  (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
  "Who records what when the add undoes what it made").

The record's "What this record does not decide" leaves the settling command's name,
its guard and what it proves to the design that carries it,
and leaves every choice the three designs list for the project's veto to those lists (`:134` to `:138`).

## What this design does not cover

Each item below belongs to another record,
and this design states the dependency it makes on each.

- Where the add's durable binding record lives and what its format is.
  This design reads that record's origin identities before the first destroy (A1 and F24).
- Where a removal's persisted destruction plan lives and what its format is.
  This design deliberately does not reuse it,
  because the plan may be finished when the removal is,
  and the record must outlive the removal (V4 and F25).
- How the add-time Git lock is written,
  told from an operator's lock,
  and released.
  This design only assumes that the settle releases it (the proposal `:466` to `:468`).
- How the deferred-destroy checks are designed.
  This design's list carries the early look at its step 1 and the second check at its step 8,
  and this design reads the same mark for its own attempt (A3).
  What each check reads,
  and what its refusal prints,
  is the sibling design's.
- The forced arm of a removal:
  which holders it ends,
  by what channel,
  and how the caller's session is spared (the evening's forced-removal decision).
  This design's list carries the request at its step 2 and nothing else,
  and this design destroys only a snapshot that nothing holds,
  so the forced arm changes nothing else here.
- What a cancelled or interrupted removal leaves,
  and what the cancel command proves.
  This design's record is written before the first destroy,
  so a cancelled removal that had not destroyed anything leaves none,
  and one that had destroyed something leaves the record beside the unmounted or reattached clone (O2);
  reversing a removal's intent is the cancelled-removal design's business.
- Staging under `~/.cache` and the unmount from a held descriptor.
  This design starts after the unmount has been verified.
- What re-applies private propagation after a replacement mount.
- Which triggers a lost last mount event waits for.
  This design's look runs in the five worktree commands the project decided,
  and the mount route's triggers are the proposal's.
- The Git settle itself:
  which Git command removes the registration and the pointer,
  and what it prints.
  This design writes nothing to Git and reads nothing from it.
- Whether the account's delegation covers a checkout snapshot's destroy in the wrapper's real grant set.
  F22 answers it from the kernel and M4 is the guest check.
- Where a cleanup in a repository whose path is gone is reported from.
  L4 and L5 say it is not reported,
  and the unaccounted-snapshot line reaches only the cases a live repository can see.
- A machine-wide index of pending cleanups.
  The unaccounted-snapshot line is per checkout dataset,
  and nothing lists every account's repositories.
- A removal record of its own.
  This design writes none (disagreement 13).
- The settling command for a slot whose recorded clone's absence cannot be proved.
  The project decided that such a slot gets one (the seven answers,
  "A slot whose recorded clone is gone and cannot be proved gone"),
  and the binding design carries it;
  this design cites it and designs no second ("The settling command is the binding record's").
- How long the destruction plan a cancellation keeps stays.
  The project decided it lives until that clone's next removal (the seven answers),
  and the plan design owns the record;
  this design cites the answer and reads no plan.
- Whether a forced removal records who it ended.
  The project decided the handler's journal alone is that record (the seven answers),
  and the forced-removal design owns it;
  this design's record names snapshots and no process.
- Migration of written records.
  Nothing of the ZFS route is released,
  so no record exists to migrate
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md:77` to `:78`).

## Revisions

Each commit whose pathspec is this file,
with the committer time `git log --format=%cI` reports and what it changed.

- `3428988897`,
  2026-11-22T17:34:23-05:00:
  the first version,
  written in the round before the gate review,
  with the home,
  the writer,
  the fields,
  the states,
  the carrier,
  `doctor`,
  the tests,
  the veto list and the questions.
- `377c74abe8`,
  2026-11-22T17:37:36-05:00:
  the carrier,
  the `doctor` lines and the veto list of the first version.
- `649f4f7b19`,
  2026-11-22T17:37:57-05:00:
  the kernel fact that the clone count is filled only for a snapshot.
- `f42eb80232`,
  2026-11-22T17:38:08-05:00:
  the linter's positive control and what the first round ran.
- `4daa6c66ce`,
  2026-11-22T17:38:19-05:00:
  what `doctor` prints for a finished cleanup,
  which is the tip the gate review read.
- `e92b4bb9ec`,
  2026-11-22T18:34:45-05:00:
  the revision that answers the gate review.
  It rewrites the order and the moment (X1 and O2),
  the lifetime and the promote proof (O1),
  the carrier's place in the questions (O3 and the merged question one),
  the look's preconditions (O4),
  the owner-and-mode rule (O5),
  the first write's primitive and the guest seams (O6),
  durability (O7 and the flush rule now in V14),
  the resurrect race's control (O8),
  the bound (O9),
  the folder's staying and the cost claims (O10),
  the order's attempt before the settle (O11),
  the unaccounted line's live bindings (O12),
  M4 (O13),
  the printing rule (O14),
  the settle-decision sentence (O15),
  the marked elisions (O16),
  the recovery without a Git entry (X2),
  the fourteen disagreements,
  and this section.
- `faaa78e7c2`,
  2026-11-22T18:35:17-05:00:
  the citation counts of this section and the last citation corrections.
- `71bd31191e`,
  2026-11-22T18:35:54-05:00:
  the record bound's errno term and two cross-references to the proof's steps.
A later commit that touches only wording is named by `git log --format=%cI` for this file.

This round's citation check found 98 citations that name their path,
each resolved to a real file and inside that file by a scratch script,
and 255 further `:line` spans that were read beside the path their paragraph names.
Seventy-nine of those spans are the second half of a range,
and twelve are citations into the two sibling documents at the commits named above.
Every one was read at the place it names.
The round corrected 14 of them:
the man page's `defer_destroy` sentence (from `:116` to `:123`),
the receipts range (from `:567` to `:568`),
the `zfs_cmd_t` statistics range (from `:73` to `:112` into `:73` to `:107`),
the proposal's authority quote (split into `:1409` to `:1413` and `:1414` to `:1415`),
its removal quote (split into `:947` to `:951` and `:952` to `:953`,
with the dropped appositive marked),
its settle quote (split into `:954` and `:955`),
its clone-origin quote (split into `:1381` to `:1384` and `:1385`),
its trigger quote (ending at `:1396`),
its "no replacement" sentence (which this revision drops,
because the recovery sentence of X2 replaces it),
the credential rule (split into `:7345` or `:7348` or `:7350`),
the finish pass's path (which named no folder),
the binding design's citations (which now name their document and commit),
the plan design's citations (which now name their document and commit),
and the store decision's staging rule (whose range now covers the whole section).
The earlier round had already corrected eight others,
which the review's citation list and this round's check both record:
the leaving record's folder and record ranges,
`kept_names_in_the_git_directories.rs:48`,
`slot_identity_store.rs:51`,
`json_record.rs:36`,
`dataset_guid.rs:39` to `:60`,
`SPEC.md:6154` to `:6155`,
and the proposal's steps 3 to 5 at `:894` to `:957`.

- `b1166908fe`,
  2026-11-22T19:07:14-05:00:
  the third round,
  which answers the second gate pass.
  Its findings and where each is fixed:
  - X3,
    the removal's tail:
    "The removal's one order of writes and removals" replaces "The order,
    and what a kill at each point leaves".
    It merges every write and removal a removal makes,
    across this record,
    the plan,
    the binding and the Git registration,
    states what a kill at each point leaves and who finishes it,
    and adds "V13 The tail's order" with the alternative.
    The origin attempt stays before the settle because the proposal orders it there.
  - X4,
    the sibling statements:
    "Where this design and its siblings disagree" is rewritten against the plan design at `76849b8d9b` and the
    binding design at `428040d45d`,
    with each withdrawn statement named;
    F24 and F25 are rewritten at those commits;
    the home text and V1 and V2 no longer weigh the binding design's withdrawn top-level folder;
    "What the two sibling texts still say about this one" names their remaining stale statements.
  - N5:
    the proof's second step refuses a dataset only when its own origin is the recorded snapshot,
    so a checkout that is itself a clone of a backup does not leave every record unprovable.
  - N6:
    question one's option B no longer has `doctor` destroy;
    it prints the `zfs destroy` command.
  - N7:
    the flush question becomes "V14 The flush rule",
    with F27's source reading (a flush does make a rename durable at `sync=standard`) and M6's two arms.
  - N8:
    the rerun rule and the failed-write rule are back at step 6 of the merged order.
  - N9:
    "V2 The record's shape" gains option D,
    the all-`key=value` shape both sibling designs write,
    and says why this design does not take it.
  - N10:
    the gate line's frequency moves under question three,
    and the kill list is written for either call-site answer.
  - N11:
    T7's seam is this record's own publication function,
    citing the store's hook where it is called and where it is declared,
    and "Sources read" names `foundation/publishing_entries_in_one_open_folder.rs`;
    F16 cites `:26` to `:28`.
  - N12:
    the gate rule's first item has its list dash back.
  - N13:
    the bound's JSON holds one instant and not two,
    which is why its terms then summed to 13,375.
  - The questions:
    this document's five become the second pass's four,
    with the list of what is already decided above them.
    V9 is retired as a veto entry because the proposal decides that settling frees the slot.
  - Also corrected,
    outside the second pass's list:
    a sentence duplicated in T4.
  Text the second pass found sound that one of these fixes forced to change:
  the order section,
  which X3 replaces;
  the proof's second step,
  which N5 narrows;
  the questions,
  which the second pass's four replace;
  the home paragraph and V1's option B,
  whose statements about the binding design's folder X4 corrects;
  and F24 and F25,
  which X4 rewrites at the commits named.
  Nothing else the second pass found sound was changed.
- `65a6c4c008`,
  2026-11-22T19:08:38-05:00:
  this revision entry,
  with the findings and the statement of what changed above.
  A later commit that touches only this list is the one `git log --format=%cI` reports for this path after
  `65a6c4c008`.
- `0973f2fc2a`,
  2026-11-22T19:08:52-05:00:
  the revision entry's own commit line.
- `9c530c80ad`,
  2026-11-22T19:58:53-05:00:
  the fourth round,
  which answers the diff check,
  after merging `main` at `e3c26ae8b7` with `git merge --no-ff` into `a95a179338`,
  which brought the evening's three decisions and changed nothing this document owns.
  Its findings and where each is fixed:
  - Z2,
    the promote guard:
    the proof's second step now refuses a dataset whose origin snapshot's `createtxg` is at or above the recorded
    one,
    with F20 corrected to the man page's "and any snapshots previous to this snapshot" and item 11's measurement,
    L11 restated,
    T10's three arms and G6's two promote arms;
    the name comparison of `0973f2fc2a` is gone.
  - Z4,
    the sibling statements:
    the "Sources read" list,
    the fourteen-point section and F24 and F25 are rewritten at the plan design's `b4dc758a76`
    and the binding design's `7d041d773b`,
    with the remaining stale statements named,
    including the step-number shift the second check causes.
  - Z6:
    point 12's tail numbers are 10 to 14,
    the stale "step 7 publishes" sentence is gone with the rewritten subsection,
    and the finisher's bullet now finishes the binding before the plan.
  - Z7:
    the kill list gains points inside 6,
    inside 12,
    inside 13,
    at the second check between 7 and 8,
    and inside 1's early look.
  - The evening's forced-removal decision:
    step 2 gains the request the root handler reads,
    the kill list gains the point while a forced request stands and says a deadline is not a cancellation,
    and the cancel after step 6 is the manual cancel only.
  - The evening's deferred-destroy decision:
    step 1 gains the early look,
    step 5's refusal is pending with the clone unmounted and out of use,
    and the second check is the new step 8 with the state its refusal leaves.
  - The evening's dirty-main decision:
    "The add that fails and destroys what it made" makes the add a second caller of the writer and the attempt,
    and the unaccounted line says why it calls none of that litter.
  - Z9:
    question one's scene is a hold rather than a second clone.
  - Z10:
    the blank line before "What this design does not cover" and the implementation plan's step 6 in the gate rule.
  - The questions:
    the four are in the diff check's current wording,
    and the list of what an accepted record decides gains the evening's three decisions.
  Text the check found sound that a fix forced to change:
  the merged list,
  which gains the second check and the forced request and so renumbers its tail;
  the proof's second step (Z2);
  question one's scene and wording (Z9);
  and the doctor section's unaccounted line (the dirty-main decision).
  Nothing else the check found sound was changed.
- `86444eef51`,
  2026-11-22T20:00:55-05:00:
  the revision entry for this round,
  with the build-step section's correction.
  The commits that touch only this list are the ones `git log` reports for this path after `86444eef51`.
- `cca16e0464`,
  2026-11-22T21:28:22-05:00:
  the fifth round,
  which answers the third check and builds to the seven answers.
  Its findings and where each is fixed:
  - Item 1,
    the answers:
    "The seven answers the project gave" replaces "Questions only the project can answer",
    the look runs in the five worktree commands with its place in each ("Where in each command the look runs"),
    "What the look prints" defines a reason and its change,
    and "When the route's gate refuses" applies the same rule to the gate.
  - Y1:
    "The add that fails and destroys what it made" gains its four cases and the refused-destroy rule,
    and the refused-destroy entry for a failed add lists its choice once.
  - Y4:
    a plan whose head is not `gone` is left by the pass and resumed by the next removal.
  - Y5:
    the finisher paragraph quotes the binding design's own sentence and drops the add and the reader.
  - Y7:
    F25 and the mapping paragraph and the stale-statement subsection are rewritten at the plan design's
    `1921adf393` and the binding design's `07858070bb`.
  - Y9:
    the `zfs-promote(8)` range is `:53` to `:54`.
  - Y10:
    a kill inside 13 names both outcomes.
  - Y11:
    the kill list gains inside 11,
    the holder that surfaces at the head destroy,
    and the head that answers `refused` or `unknown`;
    the capital after a semicolon is gone.
  - Y16:
    T10 has four arms across the boundary and G6 a third promote arm that must not be refused.
  - The lock record:
    step 11 writes a lost lock again and says so,
    and the pass does the same.
  - The forced removal's later answers:
    the kill point between 2 and 3 carries the two halves of the authority.
  - The veto merge:
    "V1 The home and the folder's name" carries the inner-name choice and V12 is gone.
  - Also corrected:
    the build-step section's stale amendment and its call sites.
  Text the check found sound that a fix forced to change:
  the carrier and gate sections (item 1),
  the questions section (the seven answers replace it),
  the add subsection (Y1),
  the finisher paragraph (Y5),
  and the kill list (Y4 and Y10 and Y11).
  Nothing else the check found sound was changed.
- `b0f6b9e676`,
  2026-11-22T21:29:04-05:00:
  this revision entry.
  The commits that touch only this list are the ones `git log` reports for this path after `b0f6b9e676`.
- `52b2e7ccea`,
  2026-11-22T22:11:12-05:00:
  the sixth round,
  which answers the fourth check,
  after merging `main` at `4ccd98b565` with `git merge --no-ff` into `e20e68b889`,
  which brought the dirty-main record's "Three further answers" and the settling command's plan answer.
  Its findings and where each is fixed:
  - Item 1,
    the add as second writer:
    "The add that fails and destroys what it made" states who writes,
    when,
    with which identities,
    what the next worktree command does,
    and that the dirty-main design takes this order.
  - W9:
    `cloneGuid` is absent for an add that made no clone,
    `cloneDataset` is then the proposed name and the reader says so,
    the record's removal-only start key becomes `startedAt` for both writers,
    and the moment is "before the add's first destroy".
  - W13:
    the binding design's bound is 43,027 wherever this document quotes it,
    and its F24 and the fourteen-point section say so.
  - W14:
    F24 is rewritten at `7d041d773b`,
    with A20 to A27 and "The settling command".
  - W16:
    the step mapping is rewritten to the binding design's own five-step list,
    and the sentence that all three numbered the same steps is corrected.
  - V15:
    it leaves the veto list,
    the dirty-main record's first further answer is quoted in the add's section,
    and its unchosen option is listed once with the seven answers' unchosen options.
  - The settling command:
    its plan answer and its comparison of this record are stated where this document describes it,
    citing the record and the binding design.
  Text the check found sound that a fix forced to change:
  F24,
  the fields,
  the bound (13,368),
  T2's width,
  the example's `startedAt`,
  the fourteen-point bound,
  the step mapping,
  the add subsection,
  the two settling-command sections,
  and Sources read.
  Nothing else the check found sound was changed.
- The commit that adds this entry is the one `git log --format=%cI` reports for this path after `52b2e7ccea`.

## References

The route's own records:

- [`doc/decision/cli-git-zfs-worktree-design.md`](../decision/cli-git-zfs-worktree-design.md),
the accepted decision with its open items.
- [`doc/planning/cli-git-zfs-worktree-design-proposal.md`](cli-git-zfs-worktree-design-proposal.md),
the design and its evidence.
- [`doc/planning/cli-git-zfs-worktree-implementation-plan.md`](cli-git-zfs-worktree-implementation-plan.md),
the build order and the open-item table.
- [`doc/decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md),
the record folder this design's shape follows.
- [`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`](cli-git-zfs-slot-identity-store-per-slot-files-design.md),
that folder's design.
- [`doc/decision/cli-git-zfs-staging-under-cache.md`](../decision/cli-git-zfs-staging-under-cache.md),
the threat model this design designs to.
- [`doc/planning/cli-git-zfs-staging-under-cache-design.md`](cli-git-zfs-staging-under-cache-design.md),
the staging design whose shape this document follows.
- [`doc/decision/cli-git-zfs-step-8-sweep-choices.md`](../decision/cli-git-zfs-step-8-sweep-choices.md),
the no-configuration answer.
- [`doc/decision/cli-git-zfs-start-probe.md`](../decision/cli-git-zfs-start-probe.md),
the answer that a failed arm refuses new attaches only.
- [`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`](cli-git-zfs-measurements-owed-before-acceptance.md),
the guest measurements this design cites.

The sibling designs this design reads and does not edit:

- `doc/planning/cli-git-zfs-add-binding-record-design.md`,
  read at `1e9bc35f1e`.
- `doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
  read at `a7823980b9`.

The other sibling designs this design depends on and does not read as decided:

- `doc/planning/cli-git-zfs-add-time-git-lock-design.md`.
- `doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`.
- `doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`.
- `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`.
- `doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`.
- `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`.

The review and the round's kept evidence:

- `~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review.md`.
- `~/temp/agent/zfs-origin-cleanup-record-design-2026-11-22/keep/host-zfs-snapshot-facts.log`,
  which F13 reads.
- `~/temp/agent/zfs-origin-cleanup-record-design-2026-11-22/keep/host-zfs-missing-dataset.log`,
  which F14 reads.
- `~/temp/agent/zfs-origin-cleanup-record-design-2026-11-22/keep/defer-read.txt`,
  which F13 reads.
- `~/temp/agent/zfs-origin-cleanup-record-design-2026-11-22/keep/git-worktree-slot-reuse-exitcodes.log`,
  which F15 reads.
