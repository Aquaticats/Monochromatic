# Where a removal records its destruction plan, and what that record holds

## Status

Non-final,
unapproved design proposal by an agent.
It is not approved by the project owner and nothing in it is final.
It designs one gap the implementation plan names and the design's own open-item list does not:
"Where the persisted destruction plan lives",
and what that record holds.
It makes no decision the project's records do not make,
beyond the writer's choices that "Choices this design marks for the project's veto" lists,
and it designs no other open item.

This revision answers the gate review at
`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review.md`
(the review's ids P1 to P21,
X1 and its list of wrong citations),
reads the sibling records as they stand on their branches,
and rewrites what the review found undecided.
"Revisions" lists each commit and what it changed.

This revision also answers the second gate pass at
`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-second-pass.md`
(the review's ids X3,
X4,
N1,
N2,
N3 and the lows in N4),
which read this text at `76849b8d9b` beside the two sibling designs.
It cites the origin cleanup record's one merged list of the removal's writes and removals,
has the finish pass prove the head's absence from the live pool before it settles anything,
drops the tuple-count ceiling,
and rewrites every statement about the two siblings at the commits it now cites.
"Revisions" lists each commit and what it changed.

This revision also answers the third pass,
the diff check at
`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-diff-check.md`
(its ids Z1,
Z3,
Z4,
Z5 and Z8),
which read this text at `b4dc758a76`.
It follows the origin cleanup record's fourteen-step list at `8cc31f59a4`,
takes the binding design's "The condition a finisher meets" and the file order in its "The record's end",
and adds what the evening's three decisions change:
the plan after a cancellation in each state,
the plan's state at each deferred-destroy refusal,
and the sentence that an add which fails writes no plan.
It also carries one question and one veto entry of its own that the forced-removal decision left here:
how long a plan a cancellation keeps lives,
and whether the plan records who a forced removal ended.
"Revisions" lists each commit and what it changed.

This revision also answers the third check at
`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-third-check.md`
(its ids Y3,
Y4,
Y6,
Y8,
Y10,
Y13,
Y14,
Y15 and Y17),
which read this text at `1921adf393`,
and the seven answers the project gave on `main`
(`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`).
It states one rule for a cancellation in both sections,
retires Choice 10,
Choice 13 and question five into "Decided by the seven answers",
renames the dataset field to `cloneDataset` and derives the slot from `targetHex`,
has the pass leave any head that is not `gone` and remove an empty folder,
and adds the settling command's words for a plan whose head is not `gone`.
"Revisions" lists each commit and what it changed.

It wrote this one file and no other,
ran no ZFS command that changes anything,
started no guest,
virtual machine or container,
touched no pool,
mount or unit,
and edited no code,
decision record or handover.

What it read:

- the accepted decision whole at `doc/decision/cli-git-zfs-worktree-design.md` (69 lines);
- `doc/planning/cli-git-zfs-worktree-design-proposal.md`:
  its "Removal and recovery" (lines 796 to 976),
  "Clone removal identity protection" (lines 1513 to 1603),
  "Bookmarks on a clone being removed" (lines 1604 to 1672),
  its "Open items the decisions leave" (lines 1673 to 1787),
  and its project-decision entry for the bookmark rule (lines 1339 to 1342);
- `doc/planning/cli-git-zfs-worktree-implementation-plan.md`:
  its steps 14 to 18 and 22,
  its "Open items and the step that needs each" (lines 2770 to 2856),
  its "Gaps the design's open-item list does not name" (lines 2858 to 2895),
  and "Where the design is silent:
  this plan's suggestions" (lines 2897 to 3010);
- `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md` items 13 and 14;
- `doc/decision/cli-git-zfs-slot-identity-store-format.md` whole,
  `doc/decision/cli-git-zfs-staging-under-cache.md` (the threat model,
  lines 221 to 237),
  `doc/decision/cli-git-zfs-step-8-sweep-choices.md` (lines 15 to 70),
  and the accepted slot store design's "A foreign entry,
  and a record under another name" (lines 1342 to 1361);
- `package/git-policy/cli/SPEC.md`:
  "The records an add writes for itself" (line 4398),
  "A record that cannot be read" (line 4831),
  "The descriptor mount of a ZFS worktree clone" (line 5408),
  "The clone's mount intent,
  receipts and readiness" (line 6072),
  "Records of a worktree that is removed" (line 16471),
  and "Doctor" (line 683);
- the evening's three decisions whole on `main`:
  `doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
  `doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
  and `doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`;
- the seven answers whole at
  `doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`;
- the cancelled-removal design at `eb7a78eccc`,
  where it leaves the plan in each cancellation state and names what it needs of this design;
- the deferred-destroy design at `946578a8e5`,
  where it asks what a retry after a cancel does;
- the sibling record designs,
  each read with `git show` at the commit named here:
  "The record that carries origin cleanup" at `22be706b09`,
  and "Where the add's durable binding record lives,
  and its format" at `07858070bb`;
- the code the wrapper keeps its records and its claims with,
  and the Git and OpenZFS sources named in F1 to F16.

What it ran:

- an OpenZFS shallow clone at tag `zfs-2.4.4` (commit `71a9f95`)
  and a Git shallow clone at tag `v2.56.1`,
  each with its push URL set to `DISABLED`,
  read for F1 to F9 and F14 to F16;
- a disposable Git probe of what Git does to a file inside a linked worktree's registration,
  cited as F10;
- the arithmetic of the record's widest lines and its bound,
  cited as F11;
- the earlier round's registration-name probe,
  still cited as F12.

What it did not run:

- no ZFS command at all,
  reading or changing,
  and no ioctl;
- no guest,
  so nothing about a pool,
  a clone,
  a snapshot,
  a bookmark or an absence check is measured here;
- no mount,
  unmount or namespace probe;
- no timing measurement.

The evidence is in
`~/temp/agent/zfs-removal-destruction-plan-record-design-2026-11-22/keep/`,
and the scripts that ran are in the sibling `scratch/` folder.
The citations this document newly writes or corrects this round were checked for existence and range by
`scratch/citation-check.ts`,
and the sample the second pass opened matched.

## The gap, and the answer's own words

The implementation plan,
"Gaps the design's open-item list does not name" (lines 2870 to 2873):

> - Where the persisted destruction plan lives.
>   "Bookmarks on a clone being removed" says the plan is persisted under every option,
>   and calls its unnamed home "a gap common to all three".
>   Steps 14 and 16 write and read it.

The design proposal's own sentence,
"Bookmarks on a clone being removed" (lines 1637 to 1639):

> What this option still needs is stated here rather than settled.
> The plan is persisted
> under every option ([Removal and recovery](#removal-and-recovery),
> step 3),
> so its unnamed home is a gap common
> to all three,

What the plan holds,
in the proposal's words,
"Removal and recovery" step 3 (lines 894 to 897):

> 3.  After unmount,
>     persist the verified inventory and expected clone identity as the destruction plan.
>     That
>     inventory is step 1's,
>     which lists bookmarks as well as snapshots;
>     the plan records each snapshot tuple this
>     removal verified and submitted for destruction,
>     with that tuple's outcome,
>     and no bookmark entry.

What a later run does with it,
in the proposal's words (lines 963 to 969):

> Resume against the persisted destruction plan and the live mount,
> dataset,
> snapshot and Git identities.
> Settle
> recorded destroys only after proving the expected objects are gone,
> never from a missing name alone.
> While the head
> is present,
> require it to answer with its recorded live GUID and an empty live snapshot list.
> Once the head is gone,
> that read is impossible,
> so settle the head and its snapshots only when its name is absent and no child of the
> pinned parent carries the recorded GUID.

What the split takes,
in the proposal's words (lines 915 to 917):

> If the verified set exceeds the supported request budget,
> the proposed split is to sort by name and take the
> largest fitting prefix of the remaining recorded tuples,
> repeatedly.

The plan's own suggestion,
in its "Where the design is silent" list (lines 2959 to 2963):

> - Where the binding record and the destruction plan live.
>   The design names only the claim record's home.
>   The plan suggests the account-owned Git common directory,
>   beside that claim,
>   for both.

That is a suggestion and not a decision of the project.
This design follows it,
for the reasons section 1 gives,
and marks the choice for the project's veto with the alternatives it weighed.

## What the project decided that binds this design

- Who destroys,
  and in what order.
  `doc/decision/cli-git-zfs-worktree-design.md` lines 34 to 37:
  "The account executes managed clone destruction under delegated `destroy`,
  answering "Delegated destroy (Recommended)".
  A normal authorized removal destroys the clone and its own snapshots in that invocation,
  answering "B:
  Destroy at once"."
  So the account is the writer of everything this record holds,
  and the removal does not stop between the snapshot set and the head.
- The bookmark rule that reads the plan,
  `doc/planning/cli-git-zfs-worktree-design-proposal.md` lines 1339 to 1342:
  "Bookmarks on a clone being removed.
  Decided:
  "A:
  Live or planned (Recommended)",
  a bookmark passes when a live
  passing clone snapshot or a recorded plan tuple explains its tuple."
  The rule decides itself what a bookmark needs;
  this design supplies the recorded tuples and nothing about the rule's other branches.
- The cancel command,
  `doc/decision/cli-git-zfs-worktree-design.md` lines 41 to 44:
  "A guarded `cancel-worktree-removal <slot>` stays as the manual way back,
  including for a removal whose wrapper was killed after it recorded the unmount or that ran past its deadline.
  It reports any snapshots already destroyed and cannot restore them."
  The plan is the only place that report can come from.
- The deferred-destroy check before the first delete,
  the same record lines 45 to 48:
  "The check that the origin is not marked for deferred destroy also runs before the first snapshot delete,
  so such a removal refuses before this run deletes anything."
  That check compares planned identities with live ones,
  and this design says where a reader finds them.
- No configuration,
  `doc/decision/cli-git-zfs-step-8-sweep-choices.md` lines 17 to 19 and 44 to 45:
  "The project first answered that everything must work by default with no configuration",
  and,
  for the case with no file at all,
  "1:
  no file needed (Recommended)".
  This record is found by the wrapper itself,
  from identities it already holds;
  no setting names its place,
  and no setup step has to make its folder.
- The wrapper never asks a person for a secret,
  `package/git-policy/cli/SPEC.md` lines 7348 to 7350:
  ""Avoid ever asking for someone to type a username or password.",
  and "Passphrases count too (Recommended)"".
  Nothing here prompts.
  Where the record cannot be read,
  the removal refuses and names the way forward,
  and a person's repair is an ordinary command,
  not a password.
- The threat model,
  `doc/decision/cli-git-zfs-staging-under-cache.md` lines 221 to 237,
  in the project's words:
  "We have to assume my machine isn't compromised.
  sudo is passwordless here anyway."
  The design reads it as the record does:
  "the account is trusted,
  and root and the account are not a security boundary here".
  Section 5 says for each check whether it survives that sentence,
  and the one check a hostile account motivated is named there rather than dropped.
- One claim per slot across add and removal,
  `doc/planning/cli-git-zfs-worktree-design-proposal.md` lines 909 to 910:
  "A per-slot claim only serializes cooperating wrapper operations."
  The claim itself is the add's binding design's,
  read at `07858070bb`,
  and this design only takes it (F15,
  A2).
- Lock-free first,
  `AGENTS.md` rule `NLK`:
  "Concurrency is lock-free first (atomic rename,
  `O_EXCL`,
  one owner per item);
  an unavoidable lock covers the smallest unit,
  never a whole repository,
  with its reason written beside it."
  The record takes no lock,
  is one file per clone,
  and is published by one rename.
- Measured limits,
  `AGENTS.md` rule `LQM`:
  "Shipped resource limits (unit caps,
  timeouts,
  size bounds) cite a workload measurement or owner decision beside
  the value".
  The record's byte bound is a sum of named field bounds (F11);
  it ships no workload number at all,
  and M1 checks the sum's arithmetic against one real record.
- Where worktrees live,
  `AGENTS.md` rule `WTU`:
  "Every worktree of this repository,
  Git's own or a Btrfs or ZFS clone,
  lives under `.claude/worktrees/` in the main checkout".
  The record names a slot by its path bytes and never by a hardcoded user name (rule `HOM`).

## What the repository already has, and what this design reuses

A record of a removal is not a new idea in this wrapper.
The family it would join is at
`package/git-policy/cli/src/native/worktree/`:

- `worktree_removal_interrupted_destroy.rs:59` names
  `monochromatic-git-policy-cli/recorded-destroys` as
  "The folder of the common Git directory that holds one record per destroy the wrapper started,
  named by the registration's id."
  Its record is written before the kernel call (`record_destroy`,
  line 115),
  marked once the call returned (`record_destroyed`,
  line 181),
  read with a byte limit (`RECORD_LIMIT`,
  line 65,
  which is 16,384 bytes),
  and dropped when the removal ends (`forget_destroy`,
  line 294,
  which also removes the folder when its last record goes).
  It names its writer by process id and birth identity so a later command tells a running removal from one a kill
  abandoned (`record_destroy_with_owner`,
  line 134).
  Its finish pass reads a record that does not parse and says so once
  (`worktree_removal_finishes_an_interrupted_destroy.rs:241`).
- `worktree_remove_hand_over.rs:805` names `monochromatic-git-policy-cli/removals-begun`,
  one record per registration,
  and line 103 names the identity record the wrapper writes beside the registration
  (`monochromatic-git-policy-cli-removal-identity`),
  written by `write_beside_the_registration` (line 635),
  joined onto the registration's own directory (`:635 to :636`),
  whose bytes are a device and inode pair per directory.
- `worktree_removal_rescue.rs:62`,
  `worktree_removal_leaving_that_outlives_its_record.rs:51`,
  `worktree_removal_around_begun.rs:46` and
  `worktree_kept_notices.rs:45` are the same shape under their own names:
  one folder of the common Git directory,
  one file per subject,
  a record the wrapper can read alone.
- The publication is one function:
  `private_storage_replacing_a_record.rs:88`,
  `replace_private_file_unsynced`,
  stages the bytes beside the name and publishes them by one rename,
  with the staging file "created exclusively with mode `0600`" and no link followed
  (`private_storage_replacing_a_record.rs:44 to 49`,
  `private_storage.rs:132 to 146`),
  and `private_storage.rs:83` and `:87` are the folder mode `0700` and the file mode `0600` every such record takes.
  Its own doc says what a lost flush costs (line 82):
  "A record whose meaning a machine crash must keep stays on `replace_private_file`."
- The receipt family in `SPEC.md` line 6072 gives the reading rules a record here needs:
  a name that is "only the clone's canonical numeric dataset GUID,
  with no sign and no leading zero" (lines 6138 to 6140),
  a byte bound "derived from the receipt's own field bounds" (line 6151),
  a writer that "refuses a receipt its own reader would not read back" (lines 6154 to 6155),
  the widest field being "the canonical slot path at 4,095 bytes" (line 6156),
  and publication by "writing a staging sibling,
  syncing it,
  renaming it over the GUID's name"
  (line 6177).
- The unreadable-record policy in `SPEC.md` line 4831 names the two ways a record can fail
  (empty or cut short) and settles it:
  a record whose writer is gone "is renamed into .../unreadable,
  never deleted" (lines 4863 to 4864)
  and one line names what may still stand on disk because of it.
- `SPEC.md` line 4400 states the durability rule this design starts from,
  "An add's own bookkeeping is published whole and flushed by nobody",
  with its btrfs reason at lines 4421 to 4423 and its own exception,
  "The completion marker keeps its flush",
  at lines 4431 to 4432.
- `foundation/path_bytes_in_records.rs:133 to :141` states when a record may carry a path as text and when it
must use hex,
  and `:64` is the hex form itself;
  the record this design writes uses the hex form alone,
  because a path with a newline is still a path.
  The wrapper's own destroy record writes its leaving place in hex and its target raw
  (`worktree_removal_interrupted_destroy.rs:276` and `:278`),
  which is two forms in one record and the reason this one keeps one.
- The per-slot claim is the add's binding record's (`<slot>.claim.<generation>`,
  read at `07858070bb`,
  and `lock/single_owner_claim.rs`),
  taken by both an add and a removal for one slot (F15).
- The mount intent and its receipts fix two more precedents:
  one short token as a dataset property (43 bytes at its longest,
  `SPEC.md` line 6102)
  and a root record under `/run` keyed by the clone's GUID.
  The claim's own reason for living in `/run` is that "a claim means nothing across a boot"
  (`SPEC.md` lines 6263 to 6264).
- `doctor` is where a leftover record is reported,
  and the project already put the removal of a killed writer's staging file in plan step 18's `doctor`
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  "Who removes a killed writer's staging file").
- The crate already has a feature that exists only for measurements,
  `measurement-routes`,
  "Off by default and never enabled by a shipped build"
  (`package/git-policy/cli/Cargo.toml:71 to 75`),
  which is the seam the guest kill arms use (section 8).

This design reuses each of those.
It adds no publication primitive.
It adds one reader,
whose shape section 2's bound explains.

## Facts read, measured and computed

Numbers here are cited by number later,
so each can be checked alone.
"Read" means a source at a named tag or a repository file;
"measured" means a run whose log is in the keep folder;
"computed" means arithmetic over the format and those bounds.

- F1.
  A name OpenZFS accepts is shorter than 256 bytes.
  `module/zcommon/zfs_namecheck.c:192` refuses a path whose length is `>= ZFS_MAX_DATASET_NAME_LEN`,
  which is 256 (`include/sys/fs/zfs.h:88`),
  and `entity_namecheck` is what the dataset,
  bookmark and snapshot checks all call
  (`:301`,
  `:320`,
  `:339`).
  So a full snapshot name (`<dataset>@<snap>`) and a full bookmark name (`<dataset>#<bookmark>`) each take at most
  255 bytes.
  Read at OpenZFS `zfs-2.4.4`.
- F2.
  A name may hold a space (`module/zcommon/zfs_namecheck.c:59 to 65`)
  and a `%` (`:229`),
  besides letters,
  digits,
  `-`,
  `_`,
  `.` and `:`.
  A name therefore holds no newline and no `=`,
  which is what lets a `key=value` line carry one verbatim.
  Read at `zfs-2.4.4`.
- F3.
  A dataset property value is at most 8,192 bytes:
  `ZFS_MAXPROPLEN` is `ZAP_MAXVALUELEN` (`include/libzfs.h:56`),
  which is `1024 * 8` (`include/sys/fs/zfs.h:86`).
  Read at `zfs-2.4.4`.
- F4.
  The snapshot destroy request takes names,
  "Snapshots that do not exist will be silently ignored"
  (`lib/libzfs_core/libzfs_core.c:440`),
  destroys none when it refuses (lines 448 to 456),
  and a failure inside the kernel program returns the errno with no per-name list
  (`module/zfs/dsl_destroy.c:656 to 671`).
  A head destroy is refused while the clone still has its own snapshots
  (`module/zfs/dsl_destroy.c:783 to 790`):
  the branch that answers `EBUSY` sits under the comment "Can't delete a head dataset if there are snapshots of it".
  So a head that is proven gone had no snapshot of its own left.
  Read at `zfs-2.4.4`.
- F5.
  A `ZFS_IOC_DESTROY_SNAPS` request runs as a sync task,
  and the sync task waits for its transaction group before it returns
  (`module/zfs/dsl_synctask.c:84 to 104`,
  reached through `module/zfs/zcp.c:51 to 52` and `:987 to :1003`).
  Source reading,
  not measured:
  a write of another file in the same pool and dataset that completed before that request is therefore in an
  earlier or the same transaction group,
  so it is on disk when the request returns.
  The premise is that the record and the clone are in one pool,
  which the route's layout makes and no record states;
  it is unmeasured (L7).
  Read at `zfs-2.4.4`.
- F6.
  Git's `worktree lock` marks and does not serialize:
  `builtin/worktree.c:1201 to 1212` reads the existing reason and then calls `write_file` on
  `worktrees/<id>/locked`,
  with no exclusive creation.
  Read at Git `v2.56.1`.
- F7.
  Git deletes a file it does not know inside a linked worktree's registration:
  `git worktree remove` deletes the registration directory,
  and `git worktree prune`,
  run once the worktree's `.git` pointer is gone,
  deletes the registration and every file
  in it.
  Measured on this host with `git version 2.56.1` and wrapper `0.6.1-alpha.44` (F10).
  Read at Git `v2.56.1`,
  whose `builtin/worktree.c:212 to 246` prunes entries inside `worktrees/`.
- F8.
  A removal's unmount request is one short dataset property,
  at most 43 bytes
  (`SPEC.md:6102`),
  and the property carries no path and no dataset name (`SPEC.md:6106 to 6107`).
- F9.
  A receipt is named by the clone's canonical numeric dataset GUID,
  with no sign and no leading zero
  (`SPEC.md:6138 to 6140`),
  and a value is local only when its source is the clone's own dataset (`SPEC.md:6112 to 6114`).
- F10.
  The measured Git probe:
  a file written into `<common>/worktrees/<id>/` was there before `git worktree remove` and gone after it,
  and a file written into a second registration was gone after `git worktree prune` removed the registration of a
  worktree whose pointer was lost.
  The log is `keep/registration-inside-file-probe.json`.
- F11.
  The record's widest bytes,
  computed from the grammar in section 2 and F1,
  F2 and F3:
  the version line and eighteen fields are at most 10,128 bytes,
  of which `targetHex` alone is 8,201;
  a tuple line is at most 340 bytes;
  so a record of `n` tuples is at most `10,128 + 340 * n` bytes,
  which is 44,128 bytes at 100 tuples and 350,128 at 1,000.
  The header's 10,128 assumes an `owner` pid of at most seven digits and a `tuples=` count of at most seven,
  which the reader enforces:
  a count with eight or more digits makes the header line wider than the header bound,
  and stage one refuses the record before it reads a tuple byte.
  The script is `scratch/plan-byte-arithmetic-revised.ts` and the log
  `keep/plan-byte-arithmetic-revised.json`.
- F12.
  A linked worktree's registration name is a plain name derived from the path's last component,
  sanitized (a space became `-`),
  and the same name is used again when a worktree is added again at the same path.
  Measured with `git version 2.56.1` and wrapper `0.6.1-alpha.44`;
  the log is `keep/worktree-registration-id-probe.json`.
- F13.
  The wrapper's own records are folder `0700`,
  file `0600`,
  staged with `O_EXCL` and published by one rename,
  and read with a fixed byte limit per record
  (`private_storage.rs:83` and `:87`;
  `worktree_removal_interrupted_destroy.rs:65`).
- F14.
  An object identity is `device:inode:seconds:nanoseconds`
  (`subvolume/fs_beneath.rs`,
  `ObjectId::encode`),
  the canonical slot path's bound is 4,095 bytes (`SPEC.md:6156`),
  and a process birth identity is spelled `linux:<start time>`
  (`process/process_identity.rs:1 to 20`).
- F15.
  One `<slot>.claim.<generation>` file serializes an add and a removal of one slot,
  made by `take_claim` with one hard link,
  so exactly one command creates the next generation;
  a second command is refused with the live owner's process id.
  Read in `lock/single_owner_claim.rs:1 to 40`,
  `:245 to 285` and `:362 to 404`,
  and in the binding record's design at `07858070bb`,
  whose use of the claim for removals is that document's proposal and not a project decision.
- F16.
  The crate has a measurement-only feature,
  `measurement-routes`,
  off by default and never enabled by a shipped build
  (`package/git-policy/cli/Cargo.toml:71 to 75`).
  It is the only seam the guest kill arms can use (section 8).

## Assumptions

- A1.
  The add's durable binding record is found by `<slot>`,
  the last path component of the path `targetHex` holds,
  as the binding design defines the key in its "What the key is made of at each moment" at `07858070bb`.
  Its home and key are that design's,
  not this one's;
  this design reads it by `<slot>`,
  compares its `slot=` with the whole `targetHex` path,
  and its clone,
  registration and origin fields with the live objects,
  and refuses when it cannot be found.
  If that design changes its key,
  this reader changes with it.
- A2.
  A removal takes the per-slot claim (F15) before it writes this record,
  as the binding design proposes for both add and removal.
  A removal that cannot take it does not write this record.
- A3.
  Two cooperating removals of one slot never run at once,
  because the claim serializes them (F15,
  A2).
  A noncooperating process of the same account can still write the record;
  that is the accepted same-account residual (section 5).
- A4.
  The checkout's common Git directory outlives the clone,
  its registration and its dataset,
  which the removal's step 5 settles.
  It is on the checkout's own dataset,
  which plan step 19 makes,
  and nothing in a removal destroys the checkout.
- A5.
  The account that runs the removal owns the checkout and its common Git directory,
  and can create a folder inside it.
- A6.
  `doctor` runs as the account,
  like every other command,
  and no project record gives it a root-run mode.
  It reads the account's folder and reports what it finds;
  a root reader is possible and is not assumed.
- A7.
  The origin cleanup record is published by the origin cleanup design's writer at its own moment,
  which is that design's step 6 and its V4 at `22be706b09`:
  immediately before the first destroy request and after every refusal that must precede a delete.
  The origin cleanup design holds the order of the removal's writes and removals,
  in its "The removal's one order of writes and removals",
  and this design cites that list instead of stating an order of its own (section 3).
  This design reads the record and never writes it.
  The moment is open to the project's veto in that design.
- A8.
  The plan and the clone are in one pool,
  which is the premise of F5's ordering argument.
  No record states it;
  M6 is the check that would.

## Limits of this round

- L1.
  Nothing about a pool,
  a destroy,
  an unmount or an absence check is measured here.
  Every refusal arm that needs ZFS is unmeasured and is listed under "Measurements owed".
- L2.
  The one-rename publication and the `O_EXCL` staging file are the repository's own primitives,
  read here as code and as its own record of them (F13).
  They were not re-read in a kernel at a tag this round.
- L3.
  The traps F7 and F12 measure were run in disposable repositories with plain `git`;
  they say nothing about a ZFS clone,
  which has no registration until plan step 10 makes one.
- L4.
  The arithmetic in F11 is over the grammar and the bounds,
  not over real records:
  no plan has ever been written.
- L5.
  The record has no count ceiling.
  Its byte bound is the computed sum in F11,
  and the reader holds no more than the record's own length;
  M1 is the run that checks F11's arithmetic against a real record.
- L6.
  The absence rule's head-gone arm rests on F4's kernel rule.
  No plan has ever been resumed.
- L7.
  F5's ordering premise (one pool) is unstated by any record and unmeasured;
  M6 is the check.
- L8.
  The finish pass's read of the live pool is unmeasured:
  no command has been run with a plan whose `head` is `gone` beside a pool,
  and no gate has refused that read in a run.

## 1. The home: place, owner, mode, names and key

### The place

The record is one file per clone at

```text
<common Git directory>/monochromatic-git-policy-cli/clone-destruction-plans/<clone dataset GUID>
```

where `<common Git directory>` is what the wrapper already asks Git for,
`git rev-parse --path-format=absolute --git-common-dir`
(`SPEC.md:690 to 695`,
the way `doctor` finds it,
and the way the existing removal records are placed).

The plans are not inside the clone,
and that is forced rather than chosen:

- the removal writes the plan after the clone is unmounted (proposal step 3,
  lines 894 to 895),
  so a plan inside the clone's own tree would be unreachable exactly when it is written;
- the head destroy takes everything inside the clone dataset with it,
  including a plan kept there,
  and the plan has to record that destroy and stand afterwards.

The plan is not a dataset property either:

- a property value is bounded at 8,192 bytes (F3),
  which one clone with more than about two dozen tuples exceeds at 340 bytes per tuple (F11);
- the property belongs to the clone,
  so the head destroy takes it at the moment its own outcome must be written;
- the origin is a different dataset shared with other clones,
  so a property there would be one record for many removals.
  The mount intent shows the shape a property serves:
  one short token,
  at most 43 bytes (F8).

The plan is not root state under `/var/lib` beside the slot identity store:
the writer is the account,
under delegated destroy (the decision,
lines 34 to 35),
so every outcome write would need a privileged channel to reach root state,
and the project's own answer for a sibling record of this route was given under that threat model
(`doc/decision/cli-git-zfs-staging-under-cache.md` lines 221 to 237),
which is why that record lives in the account's tree rather than in root state.
The account-owned Git common directory is where the wrapper's other removal records already live.

The plan is not under `/run`:

- a plan must outlive the boot in which a removal was killed.
  A reboot drops the mounts,
  and a resumed removal then needs to know which tuples an earlier run destroyed and which outcomes are unknown;
  `/run` is right for a claim,
  whose own reason is that "a claim means nothing across a boot" (`SPEC.md:6263 to 6264`),
  and wrong for evidence.

The place is the plan's own suggestion (lines 2959 to 2963),
and this design adds the reasons the suggestion did not give.
The choice is open to the project's veto as Choice 1,
with root state,
a clone property and the binding design's new top-level folder as the alternatives.

### The folder, its owner and its modes

- The folder is `monochromatic-git-policy-cli/clone-destruction-plans`,
  a sibling of the wrapper's `recorded-destroys`,
  `removals-begun`,
  `removal-snapshots` and `leavings` folders,
  so a person who knows one knows where the others are,
  and the rename machinery that moves `monochromatic-git-policy-cli/` moves it too.
- The `monochromatic-git-policy-cli` parent already exists in most repositories,
  and a repository that has none gets it from the account that writes the first record,
  by `create_private_directory_all`,
  which is what the existing records do
  (`worktree_removal_interrupted_destroy.rs:134 to 143`).
- The folder is owned by the account that owns the common directory and is mode `0700`;
  each record is owned by that account and mode `0600`.
  That is the wrapper's private-record pair (F13),
  and it keeps the clone's snapshot inventory out of other accounts' reach.
- The folder is created by the removal and by nothing else.
  `doctor` never creates it,
  and the finish pass never creates it:
  a pass that made the folder would turn the healthy state's one `stat` into a listing on every command.
- A folder whose owner is not the common directory's owner,
  or whose mode has any of the group or other write bits,
  is refused by every reader,
  and `doctor` reports it.
  Under the project's threat model this catches a person's mistake and a restored backup,
  not a hostile account (section 5).

### The record's name grammar

Three kinds of name live in the folder,
and one rule tells them apart:

- A record's name is the clone's canonical numeric dataset GUID:
  lower-case decimal digits only,
  no sign,
  no leading zero,
  1 to 20 digits,
  the same spelling a receipt's name takes (F9).
  `zfs get -H -p -o value guid` prints exactly that spelling,
  and the reader refuses `+1`,
  `007` and an upper-case digit.
- A staging entry is what `replace_private_file_unsynced` makes:
  `.<GUID>.<random>.partial`
  (the crate's own random suffix,
  the form the repository already spells that way at `SPEC.md:13755`).
  No reader reads a staging entry as a record,
  and a writer removes its own.
- Every other entry is foreign:
  a dot entry that matches neither form,
  a name that is not a canonical GUID,
  a directory,
  a link.
  No reader reads it,
  `doctor` reports it by name,
  and nothing removes it.
  A foreign entry is what a writer this design has not accounted for would leave.

The name and the record's own `cloneGuid` field must agree.
A record filed under a name its key does not match is refused by every reader,
which is the rule the slot identity store's design already states under
"A foreign entry,
and a record under another name"
(`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md` lines 1342 to 1361).

### The key, and why it is the clone's GUID

The key is the clone dataset's GUID.
It is the one fact that names this clone and no other for the whole life of the removal:
it survives the unmount,
the snapshot destroys and the head destroy,
and it is readable before any of them
(`zfs get -H -p -o value guid`,
the read [VM item 13](cli-git-zfs-measurements-owed-before-acceptance.md) made on this release).

The other candidates,
with the measurement and the records that decide against them:

- The slot's name,
  which the binding design keys its record by.
  Git's registration name is a plain,
  sanitized name (F12),
  and the wrapper already keys `recorded-destroys` and `removals-begun` by it.
  But the probe showed the same name is used again when a worktree is added again at one path
  (`reusedOnReAdd: true`,
  F12),
  so a plan keyed that way is one name for the clone removed today and the one added at that path tomorrow,
  and a reader would have to prove which clone's plan it holds before it used a single tuple.
- A digest of the slot path.
  Two clones at one path produce one name.
  The digest would also have to be computed from canonical path bytes,
  and a path that no longer exists cannot be canonicalized,
  so a reader holding only the plan could not match it to a registration.
- The origin snapshot's name,
  which the origin cleanup design keys its record by.
  A removal may have no origin of its own,
  and one origin snapshot can serve many clones;
  a plan per origin snapshot would be one file for several removals.

Choice 2 puts the candidates to the project with examples.

### What finds a plan

The finish pass and `doctor` find plans without any other record,
and no record of this route is ever written inside a registration.

- The finish pass runs before every forwarded command,
  as the existing finish pass does,
  and its first act is one `stat` of the plan folder.
  A folder that is not there costs that one `stat` and nothing else,
  which is the healthy state:
  the folder exists while a removal runs,
  and after a kill,
  a refusal or a cancellation,
  for as long as the plan's clone does.
- A folder that is there is listed once,
  and each entry is read as a plan (section 5).
  That listing is the one this design accepts on the command path,
  and it happens only while a leftover exists;
  `doctor` reports it.
  The store's answer "no listing per sweep" is about a folder that always exists and a sweep that always runs,
  which this is not.
- Nothing is written inside `<common>/worktrees/<id>`:
  a file there is deleted by `git worktree remove` and by `git worktree prune` (F7),
  which is the state in which a plan is most needed.
  The wrapper's own identity record beside the registration (`worktree_remove_hand_over.rs:103`)
  lives with that risk and accepts it because the Git lock covers all but a double force;
  this record does not need to take it,
  because the folder's `stat` already finds every plan.
  Choice 9 weighs the alternative,
  a pointer file,
  and why it was dropped.

## 2. The fields

### The grammar

The record is a text file of lines.
The first line names the record and its version,
every following line is a `key=value` pair in one fixed order,
and the tuples are last,
one per line,
in ascending byte order of their names.
The tuple count is declared before them,
so a record that ends early is detectable without guessing.
A key outside the fixed list,
a key out of order,
a repeated key,
a missing key,
a declared count that does not match the lines,
and a line that does not parse each refuse the whole record;
nothing is read as a partial record.

A concrete record,
for an unpromoted clone of this repository's shape,
with the pool and dataset names shortened to fit the page
(the fields carry the real bytes,
up to the bound in "The size bound").
The block's first line names the file for this page;
the record's own first line is the header beneath it.
Its state is the one the first write leaves:

```text
# <common git dir>/monochromatic-git-policy-cli/clone-destruction-plans/7828944690444299989
# monochromatic-git-policy-cli zfs worktree clone destruction plan, version 1
run=6f1c9d2a7b4e40a1b8c1d3e5f7091a2b
owner=48213:linux:9481733
started=2026-11-22T18:40:12-05:00
cloneDataset=tank/wt/fix-flaky-test
cloneGuid=7828944690444299989
cloneCreatetxg=17
originDataset=tank/mono-checkout
originDatasetGuid=4815162342109876543
originSnapshot=tank/mono-checkout@origin
originSnapshotGuid=11050373805378911967
originSnapshotCreatetxg=15
registrationIdHex=6669782d666c616b792d74657374
admin=2049:131081:1763750412:987654321
targetHex=2f686f6d652f752f7265706f2f2e636c617564652f776f726b74726565732f6669782d666c616b792d74657374
unmountNonce=0a1b2c3d4e5f60718293a4b5c6d7e8f9
unmountMountId=none
head=present
tuples=2
tuple=snapshot planned 17157080670377245572 18 - name=tank/wt/fix-flaky-test@worktree-own
tuple=snapshot planned 9332110456720018274 20 - name=tank/wt/fix-flaky-test@sanoid-hourly
```

The example is an unpromoted clone:
the lookup at the refusal arms of proposal step 1 refuses a promoted one,
so this is the shape a removal destroys.
The clone was made at transaction group 17,
both of its own snapshots are above that group,
and every GUID and group is distinct.
`originSnapshotCreatetxg` is 15.
The origin snapshot's group (15) is below the clone's (17),
and both own snapshots are above 17,
which is the separation [VM item 13](cli-git-zfs-measurements-owed-before-acceptance.md) measured.

### Field by field

- The two header lines are the record's kind and version.
  A reader that does not know `version 1` refuses the record and never guesses.
  A `#` first line is what the slot store's record and the sibling records use;
  this record keeps that form.
- `run=<32 lowercase hex>` is the removal run's nonce,
  the same shape as the mount intent's nonce (`SPEC.md:6100`),
  so a record is tied to one invocation and to one person's removal.
- `owner=<pid>:<linux:start time>` names the process that wrote it,
  as `record_destroy_with_owner` does (`worktree_removal_interrupted_destroy.rs:134`),
  so `doctor` can tell a running removal's staging name from an abandoned one (F14,
  section 5).
- `started=<ISO 8601 with offset, seconds>` is for `doctor` and for a person.
  It is never a proof:
  no reader decides anything by age,
  and this design sets no expiry (section 7).
- `cloneDataset` is the clone's full dataset name,
  verbatim.
  A ZFS name holds no newline and no `=` (F2),
  so the value cannot forge a line,
  and it is the name the destroy requests use.
  The binding design names the same value `cloneDataset=`;
  it is not the record's key.
  The key is `<slot>`,
  the last component of the path `targetHex` holds,
  which is the claim's name and the binding's file name (A1).
- `cloneGuid` and `cloneCreatetxg` are the clone's own identity,
  canonical decimal `uint64`.
  `cloneGuid` is also the record's key,
  and a disagreement between the name and the field refuses the record.
- `originDataset`,
  `originDatasetGuid`,
  `originSnapshot`,
  `originSnapshotGuid` and
  `originSnapshotCreatetxg` are the origin the plan was made against,
  in the same spellings.
  These are the identities the deferred-destroy check compares with live,
  and the ones the origin cleanup record already carries (section 6).
- `registrationIdHex` is the bytes of the registration's directory name under `worktrees`,
  hex encoded because that name is Git's bytes and not a ZFS name,
  and it is what a reader matches against the registration standing there.
- `admin=<ObjectId>` is the registration directory's identity,
  `device:inode:seconds:nanoseconds` (F14),
  the encoding the wrapper's existing records use.
  A worktree added again at the same path is another inode,
  so a stale record cannot be read as the new one's
  (`worktree_remove_hand_over.rs:566 to 592`).
- `targetHex` is the slot path's bytes,
  hex of them.
  The wrapper writes a path as text when one line can hold it and as hex when it cannot
  (`foundation/path_bytes_in_records.rs:120 to :143`),
  and this field keeps the hex form alone,
  because any path is representable and one form is one thing a reader refuses.
  The bound is the kernel's 4,095 bytes (F14).
- `unmountNonce=<32 lowercase hex>` and `unmountMountId` tie the record to the unmount the removal verified.
  `unmountMountId` is the mount id the handler's answer named absent,
  or `none` when the answer was an unmount with no mount to identify
  (`SPEC.md:6174 to 6176`).
  Neither is a proof on its own;
  the removal verifies the answer's own fields (`SPEC.md:6100 to 6160`),
  and the record keeps the pairing for a later run and for `doctor`.
- `head` is one of `present`,
  `requested`,
  `gone`,
  `refused` and `unknown`,
  the head destroy's state (section 4).
- `tuples=<count>` declares the number of tuple lines that follow,
  canonical decimal,
  and the reader requires exactly that many.
- A tuple line is `tuple=<kind> <outcome> <guid> <createtxg> <errno> name=<name>`:
  - `<kind>` is `snapshot`.
    The decided design records no bookmark entry (proposal step 3,
    lines 895 to 896),
    so a reader that meets another kind refuses the record rather than skipping the line.
  - `<outcome>` is one of `planned`,
    `requested`,
    `destroyed`,
    `refused`,
    `intact` and `unknown` (section 4).
  - `<guid>` and `<createtxg>` are the tuple's canonical decimal `uint64` values.
  - `<errno>` is the decimal error number the per-name result carried for a `refused` tuple,
    and `-` for every other outcome.
  - `name=` is last,
    so the snapshot's name may hold the space F2 allows and is read to the end of the line.

No field carries a bookmark,
a path other than the slot's,
a mount tree,
a person's words or a free-text note.
A record with such a line is foreign and is refused.

### The size bound and its basis

The bound is a sum of named field bounds (F11),
not a rounded pick:

```text
10,128 + 340 * <the record's declared tuple count>
```

where 10,128 is the version line and eighteen fields at their widest,
and 340 is the widest tuple line:
`tuple=` (6),
the kind (8),
a separator,
the outcome (9),
a separator,
two canonical `uint64` values (40),
a separator,
the errno (11),
a separator,
`name=` (5),
a 255-byte name (F1) and the newline.

A reader reads in two stages,
and the two stages are what bound its memory with no ceiling:

1. it opens the file without following a link,
   reads at most 10,128 bytes,
   and parses the version line and the fields,
   requiring the count before the tuples;
2. it asks the same descriptor for the file's whole length,
   refuses a file longer than `10,128 + 340 * count`,
   and then reads at most `min(340 * count, the bytes the file holds after the header)` more,
   refusing a record that ends before the lines it declares.

Both stages are one new reader around the repository's `read_plain_file` shape
(`foundation/plain_file.rs:164`),
which takes its limit before the file is read.
The writer refuses to publish a record its own reader would refuse,
which is the receipt family's rule (`SPEC.md:6154 to 6155`).

The reader holds no more than the record's own bytes:
the header's 10,128 at most,
and then the tuple bytes the file holds,
capped at 340 for each tuple the record declares.
A record longer than its declared count allows is refused after that length read and before any tuple byte is read,
so a foreign file cannot make the reader work past the formula.
A record that declares its count honestly is read whole,
and its size is what a clone with that many snapshots costs:
a hundred thousand tuples are 34 MB,
and the reader reads them.
There is no policy count ceiling,
and a removal with a larger set is not refused for the record's size.
The header's seven-digit width is a format bound,
not a policy:
when a record declares ten million tuples or more,
the header as a whole is wider than its 10,128-byte bound,
and stage one refuses it as an oversized record before reading a tuple (F11).
The grammar still allows 9,999,999 tuples,
about 3.4 GB at 340 bytes each,
so a ceiling is in effect at that width;
it is the grammar's own limit rather than a workload number,
and the format ships no smaller one.
The alternative shape,
a plan split across several records,
is Choice 10.

The bound costs what a clone costs,
not what a wait costs:
a hundred tuples are 44,128 bytes and a thousand 350,128 (F11),
and the record is written and renamed on the removal's own path,
which already waits for an unmount and a destroy.

### What the plan refuses to hold

- No bookmark entry,
  decided by the proposal (lines 895 to 896).
  A bookmark is explained by a snapshot tuple or it is not,
  and nothing in the plan destroys a bookmark.
- No name the removal has not verified.
  The first write holds the whole verified inventory and nothing else,
  so a plan is never a wish list.
- No origin-side inventory.
  That fact belongs to the add's binding record and is referred to there (section 6).
- No person's text,
  no reason line,
  no free-form note:
  a reader would have to interpret it,
  and nothing in this route may interpret a record.

## 3. Who writes it, and when

### The order

The writer is the removal's own process,
running as the account,
for the reason the proposal gives (lines 1640 to 1642):
"the wrapper runs as the account,
so the account writes the plan under delegated destruction".
Root's handler writes nothing here,
and neither does the sweep.

The order of a removal's writes and removals is not this design's to state.
The origin cleanup design holds it in its "The removal's one order of writes and removals" and its "What a kill
at each point leaves,
and who finishes it" (read at `22be706b09`),
and this design cites that list instead of writing one of its own.
That list has fourteen steps;
its step 8 is the second deferred-destroy check the evening's deferred-destroy decision added,
so every step from 8 onward is one higher than the thirteen-step list the earlier revision cited.
The points below are named by what each does.
What follows is only the plan at each of that list's points:
the plan's first write is its step 4,
its request and outcome writes are its steps 7 and 9,
a second-check refusal leaves it at step 8,
the origin attempt is step 10,
the registration it waits for settles at step 11,
the binding's files go at step 12,
and the plan is removed at step 13,
before the claim is released at step 14.

- Step 1,
  the claim and the proof:
  no plan is written.
  The plan's own preconditions are taken there (A2),
  and the early look for the origin's deferred-destroy mark is made inside that proof,
  so a mark found there refuses with the worktree still mounted and nothing written
  (the evening's deferred-destroy decision).
- Step 2,
  the removal intent:
  no plan.
  A forced removal also writes its request to the root handler here.
- Step 3,
  the verified unmount:
  no plan.
  Without `--force`,
  a holder the unmount's request or receipt names cancels the removal here,
  before any delete,
  with no plan written and the clone attached at its slot again.
- Step 4,
  the plan's first write:
  `head=present`,
  every verified tuple as `planned`,
  `tuples=<count>`,
  the clone and origin identities the proof reads,
  `unmountNonce` and `unmountMountId` from the verified receipt.
  A forced receipt that covers several mount namespaces has no single mount to identify,
  so the plan records `none` there and the nonce;
  the namespaces and mount ids stay in the receipt.
  There is no policy ceiling to check against (section 2).
- Step 5,
  the pre-delete refusals:
  the first deferred-destroy check,
  the fresh all-namespace verification and the reproof,
  all of which read the plan.
  A refusal at the first check leaves the removal pending with the clone unmounted and out of use,
  and the plan stands with every tuple `planned` (section 4).
  A holder the fresh scan sees without `--force` cancels the removal here,
  and the plan stands with every tuple `planned` beside the reattached clone (section 7).
- Step 6,
  the origin cleanup record:
  that design's writer,
  which is this removal,
  publishes it here;
  this design writes nothing into it and reads it (A7).
  A removal whose write fails there refuses before its first delete and keeps the plan it wrote at step 4;
  a rerun that finds a record for the same snapshot name keeps it when its identity agrees and refuses when it
differs,
  which is that design's own rule at this step.
- Step 7,
  the snapshot requests:
  a rewrite with that request's tuples as `requested`,
  the request,
  a rewrite with the request's per-name outcomes.
- Step 8,
  the second deferred-destroy check:
  no plan write.
  A refusal here leaves the plan with the outcomes step 7 wrote and `head=present`,
  the origin record `owed`,
  and the clone unmounted and out of use;
  the cancel command is the person's way back.
  This is not `head=refused`,
  which is a refusal of the head request itself at step 9.
- Step 9,
  the head destroy:
  a rewrite with `head=requested`,
  the destroy,
  a rewrite with `head=gone`,
  `refused` or `unknown`.
  A holder that surfaces only now cancels the removal,
  and the plan keeps the outcomes and stands beside the reattached clone (section 7).
- Step 10,
  the origin attempt:
  no plan write.
  The plan stands with its head outcome,
  and the attempt is the origin cleanup design's own.
- Step 11,
  the settle:
  no plan write.
  The plan stands while the registration,
  pointer and underlay are settled.
- Step 12,
  the binding's files:
  no plan write.
  The plan stands as the last witness while those files are removed in the order the binding design states in its
"The record's end",
  and proved gone.
- Step 13,
  the plan's removal (section 7):
  only after step 12's proof.
- Step 14,
  the claim's release:
  no plan;
  it is already gone.

The first write sits at step 4,
after the unmount and the proof,
which is the proposal's order and also the honest
one:
a plan written earlier would name an unmount that never happened.

The two requirements the earlier revision put on the binding design are answered in its text now:
its "The record's end" states the order among its three files,
and its "The condition a finisher meets" states the condition this design's finish pass uses (section 5).
Its "The record's end" cites the thirteen-step list at `0973f2fc2a`;
by what each step does that list is O's fourteen-step list with each step from 8 onward one lower,
and asking it to cite the fourteen-step list is the only requirement left from this design.
The origin cleanup design's list holds every point this plan needs,
including the second check and a kill inside the origin record's publication;
if a later revision moves the plan's removal before the binding's,
section 7 and the finish pass move with it.

### What a kill at each point leaves

The points are the origin cleanup design's list again,
and each line says only what stands of the plan and who finishes the plan from there.

- Before step 1 completes:
  no plan.
  The clone and every snapshot stand.
- Inside step 1,
  at the early mark look:
  the removal refuses with the worktree still mounted and nothing written,
  so no plan stands and only the claim is left.
- Between 1 and 2:
  no plan.
- Between 2 and 3:
  no plan;
  the intent stands and the handler may still act.
- Between 3 and 4:
  no plan;
  a holder seen without `--force` has cancelled the removal and the clone is attached again.
- Between 4 and 5:
  a plan with every tuple `planned` and `head=present`,
  and no origin record.
  Nothing was destroyed.
  A first-check refusal leaves it pending beside the unmounted clone;
  a holder the fresh scan sees without `--force` cancels and leaves it beside the reattached clone.
  The finish pass leaves it,
  because its head is not gone (section 5),
  and the removal's next run resumes it (section 7).
- Between 5 and 6:
  the same plan,
  and no origin record.
- Inside 6:
  a kill between the staged bytes and the link leaves a staging name and no record,
  and nothing was destroyed;
  the next removal publishes the record and `doctor` reports the staging name until step 18 removes it.
- Between 6 and 7:
  the origin record says `owed`,
  the plan is `planned`,
  and nothing was destroyed.
- Inside 7:
  the plan says `requested` for the whole request.
  That is the unknown outcome of section 4,
  and the removal's next run proves each tuple against the live inventory.
- Between 7 and 8,
  and inside 8:
  the plan holds the batch's outcomes with `head=present`,
  the origin record says `owed`,
  and the clone is unmounted.
  A refusal at the second check leaves exactly that state pending,
  and the cancel command is the person's way back.
  A rerun reads the outcomes and does not send those names again.
- Inside 9:
  `head=requested` stands and the head may be gone.
  The next run takes the head-gone path of section 4.
- Between 9's outcome and 10:
  the plan says `gone`,
  `refused` or `unknown` for the head;
  on a `refused` head the clone stands and the plan records it.
  The registration stands,
  and the origin record stands.
- Inside 10:
  the plan still says its head outcome,
  and the origin record names a snapshot that may be gone.
  The plan is not written by the attempt.
- Between 10's success and 11:
  no origin record stands,
  the registration stands,
  and the plan holds its head outcome.
- Between 10's blocked outcome and 11:
  the origin record stands blocked and the plan holds its head outcome;
  it leaves the blocked record to the origin cleanup design's look.
- Between 11 and 12:
  no registration and no lock stand;
  the binding stands and the plan stands.
  The finish pass or the next removal finishes step 12 under the condition the binding design states in its "The
condition a finisher meets",
  then removes the plan at step 13.
- Inside 12:
  the binding's files go in the order its "The record's end" states,
  so a kill leaves the binding and whatever inventories remain;
  the finish pass treats a standing inventory as the same step's leftover.
  The plan stands.
- Between 12 and 13:
  the binding's files are proved gone and the plan stands as the last witness.
  The next visit removes it.
- Inside 13:
  the plan's own unlink may be the point of the kill.
  A kill before it leaves the plan for the finish pass or the next removal to remove,
  and a kill after it leaves no plan.
  Either is a neighbouring point's state,
  and an empty plan folder between the unlink and the folder's removal is removed by the finish pass.
- Between 13 and 14:
  no plan stands,
  and the claim's files remain;
  the claim's own liveness rule lets the next claimant take the next generation.
- After 14:
  no plan stands.

No point leaves a binding standing after the plan is gone,
because the plan goes at step 13 and the binding at step 12.
Every one of those states is one a reader can refuse:
the record never says `gone` before the destroy returned,
and it never goes back from `destroyed`.

### One writer, one publication

- The record is published by `replace_private_file_unsynced`,
  so the name holds the old record or the new one and never a torn one (F13).
- The rewrite is whole,
  so a reader needs no lock,
  and rule `NLK` is met without a lock.
  A rewrite costs the record's bytes,
not a wait for the disk:
the plan's writes fall in the unflushed class of the three records' one flush rule (O's V14,
  and Durability below).
- Two cooperating writers do not exist:
  the removal holds the per-slot claim (A2,
  A3,
  F15),
  and the binding design's claim refuses a second operation on
  one slot with the live owner's process id.
  If one ever did,
  the later rename wins and the earlier outcome is lost,
  and a lost outcome reads as `requested` at worst,
  which is the unknown state and refuses.
  That is why last-writer-wins is safe here:
  a lost update cannot authorize a delete,
  only demand a proof.
- A noncooperating process of the same account can write anything it likes,
  including a record that explains a bookmark it should not.
  That is the accepted same-account residual the delegated option already accepts
  (proposal,
  "Clone removal identity protection",
  lines 1560 to 1564),
  and this design does not claim to close it.
  A record is trustworthy against a person's mistake,
  a reused slot and a lost update,
  and not against the account itself.

### Durability

The plan's writes are published without a flush,
and that is the three records' one rule by position rather than a choice of this document.
The origin cleanup design's V14 adopts option C for the three (read at `22be706b09`),
and it names this design's Choice 5 as the entry to cite.
This design's writes fall in that rule's unflushed class:

- a write made before a destroy is on disk when the destroy returns,
  because the destroy is a sync task that waits for its transaction group (F5),
  one pool assumed (A8).
  That covers `planned`,
  `requested` and `head=requested`.
- a write made after a destroy,
  the outcome,
  can be lost by a power cut,
  and the state it falls back to is `requested`,
  which the next run proves again.

The rule flushes a write made after an object exists that nothing else can prove,
which is the binding's first write and no write of this plan:
the plan names a clone it verified from other records and snapshots it proved,
and a lost outcome costs one repeated attempt.

What a crash can cost is the evidence of what was destroyed:
a later removal then refuses a bookmark whose snapshot is gone,
because no recorded tuple explains it,
and a person destroys that bookmark by hand
(`zfs destroy <clone>#<bookmark>`,
the proposal's own sentence at lines 1632 to 1634).
Choice 5 states the rule's three options in that wording and marks this record's adoption of it for the project's
veto.

## 4. Progress, unknown outcomes and objects that are partly gone

### The states

A tuple's outcome moves through these states and no others:

```text
planned   -> requested -> destroyed
                       -> refused  -> requested, or destroyed
                       -> intact   -> requested, or destroyed
                       -> unknown  -> requested, or destroyed
```

`destroyed` is the only end,
and nothing returns to `planned`.

- `planned` means the first write recorded the tuple among the whole verified inventory,
  and no request naming it has been made.
- `requested` means this removal submitted the name in a request whose answer is not recorded yet.
- `destroyed` means the removal proved the object gone (the absence rule below),
  or the head's proof covers it.
- `refused` means the kernel's per-name result carried an error for this name,
  which the `errno` field keeps.
- `intact` means the request returned non-zero and this name was not among the failures,
  so the kernel's own guarantee that a refused request destroys none applies
  (F4,
  `lib/libzfs_core/libzfs_core.c:448 to 456`),
  or a later read found the object still there.
- `unknown` means the request's outcome cannot be attributed to names at all:
  a program error with no per-name list (F4,
  `module/zfs/dsl_destroy.c:656 to 671`),
  or a removal killed with `requested` standing.

The head has the same shape without names:
`present`,
`requested`,
`gone`,
`refused`,
`unknown`.

### Every transition, and the rule that allows it

An outcome is replaced only by evidence newer than it or by a newer request that names the tuple.
`destroyed` is final;
nothing moves out of it.

- `planned` to `requested`:
  this run submits the tuple.
- `requested` to `destroyed`,
  `refused`,
  `intact` or `unknown`:
  the request's answer.
- `refused` to `requested`:
  a later run submits the same name again after re-proving it,
  because a hold or a clone may have gone.
- `intact` to `requested`:
  the same,
  when the object is still there and still verified.
- `unknown` to `requested`:
  the same,
  but only after the absence rule has failed to settle the tuple either way and
  the object is proven live with its recorded GUID and group.
- `intact` to `destroyed`,
  `unknown` to `destroyed`,
  and `refused` to `destroyed`:
  the absence rule proves the object gone.
  A refused tuple reaches this arm when the object goes without another request:
  the head's proof covers it,
  or the hold or clone that refused it was released and the snapshot went another way.
  The head-gone path settles every remaining tuple by it.
- Nothing returns a tuple to `planned`,
  and nothing returns a `destroyed` tuple to an earlier state.

The record keeps one line per tuple,
not one per attempt:
its order is the ascending byte order of the names,
and a second line for one name would be a record the reader refuses.
The attempts themselves are not stored,
because the only question a later run asks is what the pool says now.
Choice 4 weighs the other shape,
a line per attempt.

### A walk: a hold, a refused batch, the hold released

A clone `tank/wt/fix-flaky-test` has two own snapshots,
`@worktree-own` and `@sanoid-hourly`,
and a person holds `@sanoid-hourly` with `zfs hold`.
Its plan records both as `planned`.

The first run submits both in one request.
The kernel prechecks the whole request,
and its answer destroys neither (F4):
the per-name list names `@sanoid-hourly` with `EBUSY`,
because a hold is what `EBUSY` names (`module/zfs/dsl_destroy.c:79 to :80`),
while a dependent clone would answer `EEXIST` (`:85 to :86`),
so the record becomes `refused` with that errno for `@sanoid-hourly` and `intact` for `@worktree-own`.
The removal prints the refusal and stops;
the head is not attempted and the clone stands.

The person releases the hold.
The next run reads the plan,
proves lineage again,
settles by the absence rule (both names live with their recorded GUIDs,
so both stay `refused` and `intact`),
and rewrites the remaining two as `requested` in one request.
The request returns 0,
and the absence rule then settles both `destroyed`.
The head follows:
`requested`,
the destroy,
`gone`.

The two rewrites of `requested` are the transition `refused` to `requested` and `intact` to `requested`,
which the earlier revision of this design could not express.

### The absence rule, object by object

The accepted rule is "never from a missing name alone" (proposal line 964),
so each check compares an object's own identity:

- While the head is present,
  the removal reads the clone's live snapshot inventory,
  which answers every snapshot's GUID and its creation group.
  A tuple is `destroyed` only when no live snapshot carries its recorded GUID;
  a live snapshot whose name equals the tuple's name and whose GUID differs is another object,
  and the recorded tuple is `destroyed` beside it,
  not settled by the name.
  A live snapshot that carries the GUID but a different `createtxg` is not this removal's object either,
  and the removal refuses rather than guessing.
  The head itself cannot be settled on this path,
  because a live head proves it is there.
- Once the head is gone,
  no live inventory can be read.
  The removal proves the head's own GUID absent from the children of the pinned parent dataset,
  and its name absent too,
  which is the decided rule's own condition:
  a child that carries the recorded GUID is the clone under another name,
  and the removal refuses.
  The pinned parent is the clone's recorded `cloneDataset` without its last `/`-separated component,
  which the add's layout pins and the plan records in full,
  so no configuration names it.
  A recorded name that carries another GUID is a reused name:
  the clone this plan names is gone by GUID,
  but the decided rule asks for the name's absence as well,
  so the removal refuses and names the reuse rather than settling.
  With the head proven gone,
  every remaining snapshot tuple of that clone is `destroyed`
  by the kernel's own rule that a head with snapshots of its own cannot be destroyed (F4,
  `module/zfs/dsl_destroy.c:783 to 790`),
  so the head's proof covers them.
  That is an object-by-object statement:
  each tuple names a snapshot of that head,
  and the head's absence proves that head and its snapshots gone.

A tuple whose outcome stays `unknown` after both arms is retained and the run stops;
nothing is regenerated from the names that are left.

### What a later run does

The removal resumes on one of two paths,
and which one it takes is the first thing it decides.

**The head-present path.**

1. Read the plan,
   or find it by the folder's listing (section 5).
   A record that does not read,
   or whose `cloneGuid`,
   `cloneCreatetxg`,
   origin fields,
   `admin`,
   `registrationIdHex`
   or `targetHex` disagree with the live clone and registration,
   refuses before anything is deleted.
2. Prove the lineage of proposal step 1 again,
   against the plan's recorded identities and the binding's
   origin-side inventory.
3. Settle every tuple that is not `destroyed` by the head-present arm of the absence rule,
   rewriting the plan once with the settled outcomes.
4. Re-plan only the tuples that are still live and still verified.
   Every earlier tuple and its outcome stays in the record.
   A live own snapshot with no tuple is new history,
   and only the amended-plan rule of section 4's last part adds it.
5. Run the deferred-origin check,
   and submit the remaining set,
   as one request or as the largest fitting prefix of the remaining recorded tuples,
   sorted by name
   (proposal lines 915 to 917).
   Write `requested`,
   submit,
   write the outcomes.
6. Repeat from step 3 until the snapshot set is settled.
7. Run the second deferred-origin check,
   after the clone's own snapshots are gone and before the head destroy (the
list's step 8).
   A refusal leaves the plan with the outcomes step 5 wrote and `head=present`,
   the clone unmounted and out of use,
   and the cancel command the way back;
   `head=refused` is not this state (section 3).
8. Before the head:
   re-enumerate.
   The head is refused while own snapshots remain (F4),
   and a new own snapshot is named and added only under the
   amended-plan rule.
   Write `head=requested`,
   destroy,
   write the outcome.

**The head-gone path.**

1. Read the plan.
   No lineage proof runs,
   because there is no clone to prove,
   and no `unmount` intent is read,
   because the intent is a property of the clone and went with the head (F8).
2. Prove the head's GUID absent from the pinned parent's children (the absence rule's second arm).
3. Settle every remaining snapshot tuple and the head as `destroyed` by F4's kernel rule.
4. If the registration still stands,
   settle it and the underlay (the list's step 11);
   the origin cleanup record stood from the list's step 6,
   and the attempt at its step 10 either removed it or left it blocked.
5. Finish the list's step 12:
   the binding's files are removed in the order the binding design states in its "The record's end" and proved
gone,
   under the condition its "The condition a finisher meets" states (section 5).
6. Remove the plan (the list's step 13,
   and section 7),
   and release the claim (its step 14).

The removal-intent rule of section 6 applies only on the head-present path.
A `head=gone` plan has no live intent by construction,
and refusing it for that reason,
as the earlier revision did,
would have refused every resumable removal after its
head went.

### A worked example

A plan for `tank/wt/fix-flaky-test` whose removal was killed inside the snapshot request:

```text
head=present
tuples=2
tuple=snapshot requested 17157080670377245572 18 - name=tank/wt/fix-flaky-test@worktree-own
tuple=snapshot requested 9332110456720018274 20 - name=tank/wt/fix-flaky-test@sanoid-hourly
```

The next removal reads it,
takes the live inventory and finds no snapshot carrying GUID `17157080670377245572`,
and one carrying `9332110456720018274` at group 20.
`@worktree-own` is `destroyed` by the head-present arm,
and `@sanoid-hourly` is `intact`.
The plan is rewritten once with those two outcomes,
and the remaining set (`@sanoid-hourly`) is submitted in a new request after the deferral check.
No name is compared to decide either outcome:
the GUIDs decide.

If the kill had landed after the head destroy,
the same record would say `head=requested`,
the clone would not answer,
and the head-gone path would prove the head's GUID absent from `tank/wt`'s children and settle both tuples and the
head by F4's rule.

### Re-planning with new snapshots

The proposal's amended-plan rule is kept as written (lines 923 to 927):
a newly created own snapshot is submitted by the wrapper,
not by a person,
after all lineage and unmount checks run again,
and the amended plan keeps every earlier tuple and its outcome.
This design adds only the mechanics:
the amended record is one rewrite,
its `run` field takes the new nonce,
and its tuple count is recomputed,
so the record is never a set of deltas.

A missing or changed recorded identity,
unproved new history,
or an unknown prior outcome stops the attempt before the rewrite.

## 5. The readers

Every reader reads the whole record,
holds no lock,
and refuses rather than repairs.
A reader's own reads of the pool are the checks;
a record's word alone settles nothing.

### What every reader refuses

- A record that cannot be read:
  no such file,
  a permission the reader lacks,
  an I/O error,
  a record that stops before its declared tuple count,
  a record whose last line is unfinished,
  a record with a key outside the fixed order,
  a record whose numbers are not canonical,
  a `cloneDataset` that names no dataset (`entity_namecheck` would refuse it,
  F1 and F2),
  and a record whose header line is not `version 1`.
  Each refusal names the path,
  the field or line,
  and what was read,
  and says that nothing was destroyed by this refusal.
  A plan that does not read is never moved aside or deleted (the seven answers' unreadable rule).
- An oversized record:
  a file larger than `10,128 + 340 * declared count`,
  or one that ends before the lines it declares.
  It is refused,
  never truncated.
- A foreign entry:
  a name that is not a canonical GUID,
  a directory,
  a link,
  a record whose header line is not this record's,
  a record whose name and `cloneGuid` disagree,
  a record with a tuple kind other than `snapshot`.
  No reader reads it,
  and `doctor` reports it.
- A wrongly owned entry:
  a record whose owner is not the folder's owner,
  a folder not owned by the common directory's owner,
  or a folder whose mode is not `0700`.
  Under the project's threat model this is a mistake-catcher,
  not a boundary:
  the account can write its own records anyway,
  so the check exists for a root run that made the folder,
  a person who copied a record,
  and a restored backup with other ownership.
  It is kept rather than dropped,
  as the staging design kept the checks a hostile account motivated.
- A staging entry:
  read by nobody,
  removed by its writer,
  and removed by plan step 18's `doctor` only when the writer is gone (below).
  A dot entry that matches no known staging form is reported and not removed.

### How each reader meets the five cases

Each reader below applies all five cases above,
and each differs only in what it stops:

- The bookmark rule refuses the whole removal before the first delete for any of the five,
  names the record and the field or line,
  and says that nothing was destroyed.
  An unreadable record and a foreign entry are one answer here,
  because a bookmark the plan would have explained cannot be told from one it would have refused.
- The snapshot batch and the head destroy refuse the request they were about to make,
  leave every object as it was,
  and name the record.
  A partial record is refused rather than completed from the live inventory,
  because a missing line may be the one that explained a bookmark.
- The deferred-origin check refuses before the first delete for any of the five,
  and names the record beside the live identities it could not compare.
- The cancel command prints the case rather than acting on it:
  the path,
  what was read,
  the tuples whose outcomes are therefore unknown,
  and that it deleted no snapshot;
  it removes a plan only in the one state section 7 names,
  and a foreign or oversized entry is printed as not this removal's record.
- The finish pass refuses that one plan when it cannot read it or cannot prove the absence,
  and leaves the plan,
  its registration and its binding standing;
  otherwise it finishes the origin cleanup design's list's steps 11 to 14 for that plan (below).
- `doctor` reports each of the five by path and reason and changes nothing,
  except the known staging names of a
  writer that is gone.

### The bookmark rule

Proposal step 1,
before the first delete.

It reads the plan's tuples as the "recorded plan tuple" of the decided rule (lines 1339 to 1342),
and a bookmark passes when a live passing clone snapshot or a plan tuple explains its tuple,
whatever that tuple's outcome is:
an `unknown` tuple still says this clone had that snapshot.
A plan that cannot be read completely refuses every bookmark that would have needed it,
which is the same answer the rule already gives for a bookmark list that cannot be read
(proposal,
lines 1623 to 1624).

### The snapshot batch and the head destroy

Proposal step 3,
plan step 16.

Before each request the batch reads:

- the plan's `cloneGuid` and `cloneCreatetxg` against the live clone,
  and its origin fields against the live origin,
  refusing on any mismatch;
- the tuples as the record of what an earlier chunk did,
  so no name is submitted twice and no destroyed name is re-read as live;
- any `unknown` tuple,
  which stops the attempt (section 4).

It then writes the request's tuples as `requested`,
submits,
and writes the outcomes.
The head destroy reads `head` for the same reasons and writes `requested` then the outcome.

### The deferred-origin checks

Proposal step 3,
plan step 15.

The check reads the plan's clone and origin identities as "the planned ones",
compares them with the live clones and origin,
and reads the origin's deferred-destroy mark live.
It refuses when no plan exists at the point it runs,
because it runs before the first snapshot delete and the plan is written before that.
Its own design is a sibling's;
this design supplies the fields and the refusal when the record does not read.

### The cancel command

Proposal "Cancelling a blocked removal",
plan step 17.

The command reads the plan and prints:

- the slot,
  clone and dataset names,
- each tuple's outcome,
- which snapshots are proven destroyed and which are unknown,
- what remains (the head,
  the registration,
  the origin),
- and that nothing it prints can be restored.

It deletes no snapshot.
It removes a plan only in the one state section 7 names:
a plan whose every tuple is `planned` and whose head is `present`,
beside the reattached clone,
after it has proved the clone and the plan and before it releases the claim.
In every other state it removes nothing.
It never rewrites an outcome,
and a record it cannot read is printed as unreadable with the path,
followed by the manual route (`zfs destroy <clone>#<bookmark>` for a refused bookmark,
`doctor` for the state).
What "guarded" means for the command is the sibling design's answer;
this design only says what its reader does with a record,
and section 7 states one rule for the plan in each state of a cancellation.

### The finish pass that settles a registration

The wrapper already finishes a removal a kill cut short before every forwarded command
(`finish_interrupted_destroys`,
`worktree_removal_finishes_an_interrupted_destroy.rs`),
and that pass is this design's driver,
in the same place and with a different folder.
It does what the origin cleanup design's list says a finish pass does,
at the list's steps 11 to 14 (read at `22be706b09`),
and it settles nothing on a record's word:
it reads the live pool first.
The binding design names this pass as a finisher in its "The record's end",
and the condition it must meet is that design's "The condition a finisher meets":
a plan stands for the clone's GUID with `head=gone` and the plan's five origin fields equal to the binding's,
or,
with no plan,
the origin record stands with those five fields equal.
The origin record's absence is not a failure when a plan stands.

1. It stats the plan folder.
   A folder that is not there ends its part of the command with that one `stat`.
2. A folder that is there is listed once,
   and each plan is read.
   A plan that does not read is reported and left (the seven answers' rule for a record that cannot be read);
   the pass never moves or removes one.
3. A plan whose head is anything other than `gone` is **not** acted on:
   `present`,
   `requested`,
   `refused` and `unknown` alike.
   No forwarded command destroys a snapshot or resumes a destroy.
   It is left for the removal command,
   and for the settling command when the clone is gone;
   `doctor` names it.
   A plan folder left empty by a kill between a plan's unlink and the folder's removal is removed by the pass.
4. A plan whose `head` is `gone` is taken in this order.
   - It takes the per-slot claim for `<slot>`,
     the last path component of the path `targetHex` holds,
     which is the key the binding design defines in its "What the key is made of at each moment",
     not the dataset name in `cloneDataset` (A2,
     F15).
     A claim whose owner is alive ends its part for that plan:
     the live removal will finish,
     and nothing is touched.
     A claim it takes is released last,
     at the list's step 14.
   - It reads the live pool before it acts,
     through the route's gate:
     it reads the start probe's record first,
     because every process that makes a raw request does
     (`SPEC.md:6291 to 6292`),
     and a gate that refuses makes no request at all
     (`SPEC.md:20441 to 20445`).
     A refusing gate,
     or a `/dev/zfs` it cannot open,
     means no settle:
     the registration,
     the binding and the plan stay,
     one line names the plan and the refusal or the error,
     and its frequency is the origin cleanup design's question three,
     which owns how often a pending record speaks.
   - It reads the pinned parent's children for the clone's recorded name and GUID,
     the parent being the plan's `cloneDataset` without its last `/`-separated component.
     A child that carries the recorded clone GUID means the clone stands:
     the plan's `head=gone` is contradicted,
     so the pass settles nothing,
     leaves the registration,
     the binding and the plan,
     and prints one line naming the plan and the live clone,
     and it never rewrites the plan.
     A recorded name carrying another GUID is a reused name:
     the pass has proved this plan's clone gone by GUID.
     When no registration stands for the plan,
     or the one that stands is another removal's,
     and the binding names another clone,
     the pass removes the plan and says so once;
     it refuses on a reused name only while a registration still stands for the plan,
     because then there is something left to settle.
     The name absent and no child carrying the GUID is the proof the rule asks for.
   - With that proof,
     it settles the registration and the underlay only when a registration stands whose identity
     matches the plan's `admin` and `targetHex` (the list's step 11).
     A registration with another identity is another removal's,
     and the pass leaves it.
     When the settle finds the managed Git lock gone,
     it writes it again,
     says so,
     and releases it,
     as the lock record says every command that holds the slot and has proved the registration does;
     the pass is such a command and is not exempt as a finisher.
   - It finishes the list's step 12 for the binding that names this plan's clone,
     under the condition above:
     it reads the binding by `<slot>` and compares its `slot=` with the whole path `targetHex` holds,
     its `cloneGuid=` with `cloneGuid`,
     and its five origin fields with the plan's (A1).
     When they are equal it removes the three files in the order the binding design states in its "The record's
end",
     proves them gone,
     and treats a standing inventory without the binding as the same step's leftover,
     which it removes too.
     A binding that names another clone is a new slot's binding and is left;
     this plan's binding is gone.
     A binding that cannot be read,
     one whose origin fields differ from the plan's,
     or one whose files cannot be proved gone,
     leaves both the binding and the plan,
     prints one line,
     and refuses to remove the plan.
     That is the seven answers' unreadable rule applied to the last witness's predecessor.
   - It removes the plan (the list's step 13),
     and then releases the claim (its step 14),
     as `forget_destroy` removes a record and its empty folder (F13).
5. It writes nothing into any plan:
   the outcomes were the removal's,
   and a settle is not an outcome.

That is the driver the earlier revision lacked:
no pointer file,
no listing of `worktrees/` per command,
and no pass that reads a plan without a reason to.
The pass takes the claim and finishes the list's steps 12 and 13 in the order the list gives,
so a kill in that tail has one command that finishes it without an add.
A kill inside the pass leaves the state of the list's point it was in:
its claim names a dead owner and the next claimant takes the next generation,
an inventory it had not removed is the same step's leftover,
and a plan it had not removed is still the last witness.
The one cost is the `stat` per command while the folder exists,
which is one `stat` in the healthy state and a listing only while a leftover stands;
a `head=gone` plan adds the claim,
the gate,
one parent read and the binding's read.
Choice 9 weighs the pointer alternative.

The pass takes the arm the binding design's Q8 weighs:
it never settles on an unproved absence,
so a binding whose clone's absence cannot be proved stays,
the slot waits,
and the plan stands for the next removal or for `doctor`.
It removes a plan only after the binding that names the same clone is proved gone.

### `doctor`

Plan step 18.

`doctor` runs as the account (A6),
lists the folder once per run when it exists,
and reports,
per entry:

- the record's key and its `cloneGuid`,
  with a line when they disagree;
- its state:
  `head`,
  the tuples' outcomes,
  and the tuples that are not proven;
- its owner and the folder's owner and mode,
  with a line when they differ from the common directory's;
- a plan whose registration is gone,
  naming the origin work it holds;
- every foreign entry and every dot entry;
- the known staging forms,
  whose count and age it reports;
- a record that does not read,
  reported as unreadable with its path and the fact that nothing was destroyed by reading
  it.

It removes a staging name only when a plan with that GUID exists and the plan's `owner` names a process that is gone,
or when a plan with that GUID stands and its `head` is `gone`.
A staging name with no plan beside it is reported and left,
because it can be a live writer's first write.
That is the difference from the store's unconditional removal:
this writer can be alive for the length of a destroy,
so the store's rule would break a running removal (Choice 11).

`doctor` never creates the folder,
never removes a plan,
and never rewrites one.

### Readers that do not exist

- The mount sweep (plan step 8) reads no plan.
  A plan is not an instruction to mount or unmount,
  and the sweep's trigger is the intent property,
  not a record in the common directory.
- The add (plan step 10) reads no plan,
  and an add writes none.
  An add that fails destroys only the clone it made and then its snapshot,
  which is the evening's dirty-main decision and the add design's own path;
  an add whose own unmount is refused as busy withdraws its request and writes none either;
  this design's readers never meet a plan from an add,
  and the absence proof that path uses is the same head-gone proof of section 4.
  A new clone is a new GUID,
  so a plan of an earlier clone at the same path cannot be its plan either.
- The commit service's other passes read no plan.
  The ZFS removal writes no `recorded-destroys` entry and no `removals-begun` entry (Choice 6),
  so those passes never meet a plan.

## 6. The binding record and the removal intent

### What is copied, and what is referred to

Copied into the plan,
because the plan must stand alone for everything it destroys:

- the clone's full name,
  GUID and `createtxg`;
- the origin dataset's full name and GUID;
- the origin snapshot's full name,
  GUID and `createtxg`;
- the registration's name bytes and identity,
  and the slot path.

Referred to,
never copied:

- the origin-side GUID inventory that proposal step 1's third check compares the live clone against.
  That is a fact of the add,
  "written once at add" (proposal,
  lines 1626 to 1627),
  and the decided rule forbids updating it.
  A copy taken at removal time would be a second recording of an add-time fact,
  and two recordings can disagree with nothing to decide between them (Choice 7).
- the origin cleanup record that carries the origin after the clone's destruction,
  whose home,
  fields and end are the sibling design's.
  This design reads it and never writes it;
  the record is published at the list's step 6 by this removal,
  and a removal whose write there fails refuses before its first delete (that design's rule).
  A rerun that finds a record for the same snapshot name keeps it when its identity agrees and refuses when it
differs.
- the binding record itself,
  which this design reads by the slot's name (A1).

The plan names no path to either record.
A reader finds the binding by the slot's name,
and the origin record by the name its own design gives it.

### When they disagree

No record wins.
The removal refuses before the first delete,
names the three readings (the binding's,
the plan's,
the live one),
and changes none of them.
The plan is never rewritten to match live,
which is the proposal's own rule for the recorded lineage (lines 874 to 875,
"Never update the recorded lineage to match an external change"),
and the binding is never edited by a removal.

The one deliberate non-refusal is the amended plan of section 4:
a new own snapshot is added to the record by the wrapper after every check runs again,
and the record keeps every earlier tuple.
That is an addition to the plan,
not an update of a fact about the add.

### The removal intent

The removal intent is the durable `unmount` property on the clone,
which is the request the handler answers (`SPEC.md:6093`),
and nothing else this record knows about.
The wrapper's other rungs also write a begun record beside the registration;
this route does not,
for the reason Choice 6 gives,
so a reader never takes one for this removal's intent.

The plan is written after the intent's answer is verified,
and its `unmountNonce` names the request it was verified under.
When the plan's nonce and the live property's nonce differ,
on the head-present path only:

- if the plan has no `unknown` outcome and its identities still match live,
  the run re-proves everything and rewrites the plan with the new nonce,
  keeping every tuple and outcome;
- otherwise it refuses,
  and the cancel command is the way forward.

A plan with no live intent while the head still stands is refused by the removal,
because nothing then proves the unmount it claims;
that a person removed the property is not proof of anything.
A plan whose head is gone is never refused for that reason (section 4).
An intent that reads `mount` while the head still stands is not an absence:
it is the reattach of a removal that cancelled itself,
so a plan beside it is adopted on the head-present path under the nonce rule above,
and every tuple and outcome it holds is kept.
A retry after a cancel therefore always presents the kept plan:
it adopts it under its own nonce,
keeps every tuple and outcome,
and proceeds to the first deferred check;
the look does not run on it.
A plan's `owner` may name a process that has ended,
and no reader of this design refuses on that field;
only `doctor`'s staging rule reads it.
Section 7 says what a cancellation leaves and what a live `mount` intent means for it.

### The wrapper's other removal records

The plan is this route's counterpart of `recorded-destroys`,
not a second record beside it.
A managed clone removal writes the plan and no `recorded-destroys` entry,
and no `removals-begun` entry,
because those records are read by passes that would treat a clone as an ordinary worktree removal.
The two families cannot be confused even if one is seen by the other's reader:
the folders differ,
and the plan's header line is one the other readers do not know.
Choice 6 puts that to the project,
with the removal record itself.

## 7. Its end

### When it is removed

The removal that owns the plan removes it at the origin cleanup design's list's step 13,
as the last witness of the removal,
and only under the conditions the list gives:

1. the head is proven gone by the absence rule;
2. the registration and the underlay are settled (the list's step 11);
3. the binding's files are removed and proved gone (its step 12),
   under the condition the binding design states in its "The condition a finisher meets";
4. the plan is unlinked,
   and the folder is removed when its last record goes,
   as `forget_destroy` does (F13);
5. the claim is released last (the list's step 14).

The origin cleanup record is not part of the condition:
it stood from the list's step 6,
and the attempt at its step 10 either removed it or left it blocked,
so nothing the plan carries is the only copy of the origin's identity once the head is gone.
The plan's removal comes after the binding's files and before the claim for the reason the list's V13 gives:
the plan is the last witness,
so no kill leaves a binding standing after the plan is gone.

A plan left with `head=gone` and no registration is removed by the finish pass on its next visit,
under the same conditions and the same order (section 5).
`doctor` names it until then.
No command needs the registration,
a second record,
or a person for that case.

### A clone gone whose absence cannot be proved

A plan whose head is not `gone` is the finish pass's no:
it acts only on `gone` (section 5),
and no removal can run when there is no clone.
When the clone's recorded GUID is absent from the pinned parent's children,
but the head word says `present`,
`requested`,
`refused` or `unknown`,
or a person destroyed the clone outside the wrapper,
the plan is the settling command's case,
and the command is the binding design's "The settling command":
this design designs no second command.
The proof is the one that command already reads,
the pinned parent's children for `cloneGuid=` by GUID and by name,
and not the plan's head word.

What the command must do to such a plan,
in words that design can quote:

- A plan whose head is `gone` stays its own finisher's;
  the command refuses it,
  because the pass finishes it.
- A plan standing for the clone with a head other than `gone`,
  when the command's own read finds no child carrying the clone's GUID,
  and the name absent or carrying another GUID,
  is removed by the command after the person's confirmation,
  after the binding's files and before the claim,
  which is the order the origin cleanup design's list gives at its steps 12 to 14.
  The command prints that it removed it and that no finisher acts on a head that is not `gone`.
- A plan whose record cannot be read,
  or whose `cloneGuid` and names disagree with the live facts,
  is the seven answers' unreadable case:
  change nothing,
  name the file,
  wait for a person.

With no plan standing,
the command has nothing of this design to remove;
its own four removals and their proof are the binding design's.

### A cancellation, and the plan it leaves

The evening's forced-removal decision makes a removal without `--force` cancel when its scan sees a holder,
and after part of the history is gone it attaches the clone again and reports each snapshot destroyed.
One rule covers every actor:
a cancellation removes a plan only when the plan witnesses nothing,
and every other plan is kept for the clone's next removal.
By state:

- Nothing destroyed yet.
  A holder at the list's step 3 leaves no plan at all.
  A holder at its step 5 leaves a plan with every tuple `planned` and `head=present` beside the reattached clone,
  and the live intent is `mount`.
  Nothing was destroyed,
  so the plan witnesses nothing.
  Both the removal that cancels itself and the cancel command remove such a plan,
  one whose every tuple is `planned` and whose head is `present`,
  after proving the clone and the plan and before releasing the claim (section 6 tells the `mount` intent from an
absent property).
  A cancellation killed before that removal leaves the plan to the next removal,
  which adopts it on the head-present path,
  re-proves every tuple and rewrites it as its own;
  the two actors do the same thing,
  and neither writes an outcome.
- Some destroyed.
  A holder that surfaces only at the head destroy cancels after the batch,
  and the clone is attached again with every loss reported.
  The plan keeps the batch's outcomes and its head:
  `present` when no head request was made,
  `refused` when the head request was refused,
  `unknown` when its outcome cannot be told.
  It is the record of those losses,
  so neither the cancelling removal nor the cancel command removes or rewrites it.
  The seven answers decide its lifetime (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
  "How long the plan a cancellation keeps stays"):
  until that clone's next removal,
  which reads it,
  proves it against the live clone,
  adopts it under its own nonce,
  keeps every tuple and outcome,
  carries the loss forward,
  and ends it at the list's step 13 after proving the remaining set and the head gone.
  `doctor` names it until then.
  Two costs come with that lifetime:
  the folder stands for the clone's whole life,
  so every forwarded command lists it and reads each plan while it stands;
  and a clone later destroyed outside the wrapper leaves the plan (head `present`) and its binding standing,
  which is the settling command's case,
  not a settled one.
- Neither actor rewrites an outcome,
  and neither removes a plan that records a destroyed snapshot.
  A forced removal whose receipt covers several mount namespaces writes the same plan,
  with `unmountMountId=none` and the nonce (section 3);
  who it ended is the journal alone,
  which the seven answers decide (see "Decided by the seven answers").

### What a leftover plan means

- A removal was killed,
  refused,
  or cancelled.
  The plan is a record of what it did,
  and never authority to do anything:
  every reader proves live identities again.
- The clone may still stand with part of its history gone,
  or may be gone with its origin left.
  The record says which,
  by its `head` and tuple outcomes,
  and `doctor` says the same in words.
- A plan whose tuples are all `planned` and whose head is `present` is a removal that refused before it destroyed
  anything,
  which is the state the pre-delete refusals leave.
  It is safe to resume and safe to leave.
  After a cancellation with nothing destroyed the cancel command removes it (above),
  and a plan that outlives that cancel is adopted by the next removal.
- Nothing tells a person to delete a plan except the removal's own next run,
  the cancel command in the one state above,
  or a person's decision after `doctor` names it.
  A plan is never removed because it is old,
  because age proves nothing about a clone.

### No expiry

This design sets no age,
no count of runs and no retention window for a plan,
including the plan a cancellation keeps.
The alternative,
expiring a record after a period,
would erase the only evidence of a destroyed snapshot and turn a resume into a fresh,
unexplained set;
that is Choice 8,
and the kept plan's lifetime is decided there.

## 8. The tests a build must have

The build is plan steps 14,
15,
16,
17 and 18.
Every control below names what it forces,
and GFP applies:
each guard is shown to fail with the guard removed before it counts.

### Controls that need no guest

The writer and reader of the format,
on a disposable folder:

- an encode and decode round trip of a record with two tuples,
  built from the example's values,
  so the grammar is exercised and not just accepted;
- a refusal for each malformed form:
  missing key,
  extra key,
  key order,
  repeated key,
  a header line that is not `version 1`,
  a declared count above and below the line count,
  a tuple kind other than `snapshot`,
  an outcome outside the six,
  a non-canonical GUID (`+1`,
  `007`,
  an upper-case digit,
  21 digits),
  a non-canonical `createtxg`,
  a name with no `name=` key,
  an errno on a non-`refused` tuple,
  a missing errno on a `refused` tuple;
- a truncated record at every field boundary of the header and at the last tuple line,
  asserting the reader refuses and changes nothing;
- an oversized record (one byte past `10,128 + 340 * count`),
  refusing without reading past either stage;
- a foreign entry,
  a directory,
  a link and a wrongly owned record,
  each refused and each reported by `doctor`;
- a record whose name and `cloneGuid` disagree,
  refused;
- a staging entry `.<GUID>.<random>.partial`,
  asserting no reader reads it and `doctor` removes it only when the
  plan's owner is gone;
- a folder with mode `0775`,
  refused;
  a folder `0700`,
  read;
- a plan written by the removal,
  then a second write,
  asserting the earlier record or the new one and never a mixture;
- a writer that refuses a record its reader would refuse,
  forced by a tuple whose name is one byte past the kernel's bound and makes its line wider than 340 bytes;
- the two stages of the reader with no count ceiling:
  a file that grows between stage one and stage two is refused,
  a file whose declared count needs more bytes than it holds is refused,
  a file longer than `10,128 + 340 * count` is refused after the length read and before a tuple byte is read,
  a declared count with eight digits is refused within the header bound,
  and a record of 5,000 tuples is read whole.

The lifecycle,
without a pool:

- `planned` to `requested` to each outcome,
  and each of `refused`,
  `intact` and `unknown` back to `requested`,
  asserted on records written by the control and read back by the built reader;
- a `destroyed` tuple refused any later transition,
  whatever a control writes over it;
- the hold-released walk of section 4,
  arm by arm,
  with the per-name answers of a refused batch planted as a control
  record;
  this is the walk the earlier revision could not express.

The finish pass and the folder:

- a plan with `head=present` or `requested`,
  left untouched by the pass and named by `doctor`;
- a plan with `head=gone` whose slot's claim a live owner holds,
  asserting the pass changes nothing and waits for the live removal;
- an empty folder case:
  no folder,
  one `stat`,
  no listing (asserted by a control folder that counts opens);
- a control that writes nothing inside a registration and asserts no file of this design ever appears under
  `<common>/worktrees/` (F7 is why).

The finish pass's live read and its tail,
which need a pool except where a state is planted:

- a plan with `head=gone` and a live child carrying the recorded clone GUID is not settled:
  the registration,
  the binding and the plan are unchanged and one line names the plan and the live clone;
- a plan with `head=gone` and a recorded name carrying another GUID is not settled while a registration still stands
for the plan:
  the pass names the reuse and leaves everything;
- a plan with `head=gone`,
  a recorded name carrying another GUID,
  no registration for the plan,
  and a binding naming another clone,
  is removed after one line about the reuse,
  and nothing else is touched (Z5 of the diff check);
- a plan with `head=gone`,
  the name absent and no child carrying the recorded GUID,
  and a binding naming this plan's clone with the five origin fields equal,
  is finished in the list's order:
  the registration settles,
  the two inventories are removed and then the binding,
  all proved gone,
  the plan is removed,
  and the claim is released last;
- the same plan with the binding's origin fields differing is not settled:
  the binding and the plan stay,
  one line names both (the binding design's condition);
- an inventory standing with no binding is removed as the same step's leftover;
- a plan with `head=gone` and a binding that cannot be read leaves the binding and the plan and names both;
- a plan with `head=gone` and a refusing gate,
  or a `/dev/zfs` that cannot be opened,
  is not settled:
  everything stays and one line names the plan and the refusal;
  where the build's gate has no seam for a planted verdict,
  this arm runs in the guest;
- a real head destroy followed by the pass:
  the registration settles,
  the inventories and then the binding are removed and proved gone,
  the plan is removed,
  and the claim is released last.
- a plan with a head of `present`,
  `requested`,
  `refused` or `unknown`,
  one arm each,
  is left by the pass whatever the pool says;
  the settling command is the one that removes it,
  and the words section 7 gives are the arm's expectation
  (the command's own control is the binding design's);
- a binding already gone with the plan standing (a kill between 12 and 13):
  the pass removes the plan and touches nothing else;
- a kill of the pass between the two inventories and the binding:
  the planted state is the binding and one inventory,
  and the pass removes the rest in the binding design's order;
- a registration the settle left partly emptied:
  its directory identity equals the plan's `admin`,
  so the pass matches it and repeats the settle;
- an empty plan folder (a kill between a plan's unlink and the folder's removal):
  the pass removes it and prints nothing;
- a binding whose `slot=` equals the path `targetHex` holds and a binding whose `slot=` differs:
  the first is this plan's and the second refuses,
  and the claim is taken under `<slot>` from `targetHex`,
  never under the dataset name in `cloneDataset`.

The refusals,
a cancellation and a forced receipt,
without a pool:

- a first-check refusal leaves a plan with every tuple `planned` and `head=present`,
  beside an unmounted clone,
  and the rerun reads it and resumes;
- a second-check refusal leaves a plan with the batch's outcomes and `head=present`,
  and the rerun reads those outcomes and sends none of those names again;
- a head request's refusal leaves `head=refused` and the clone standing,
  which is a different state from the second check's;
- a cancellation with nothing destroyed removes a plan whose every tuple is `planned` and whose head is `present`,
  and the next removal of that clone writes its own plan;
  both the removal that cancels itself and the cancel command are driven in this arm,
  and they do the same thing;
- a cancellation after losses keeps the plan,
  and the next removal adopts it and keeps every outcome (the seven answers' kept-plan rule);
- a forced receipt that covers several mount namespaces writes `unmountMountId=none` and the nonce;
- a control that runs an add which fails asserts that no plan appears in the folder.

The interrupted states,
forced without timing luck:

- the owner-gone trick (`record_destroy_with_owner`,
  `worktree_removal_interrupted_destroy.rs:134`):
  a control writes a plan with `requested` tuples and an owner that has ended,
  and the next run must prove,
  not re-request;
- a plan with one `destroyed`,
  one `intact` and one `unknown` tuple,
  asserting the run settles the first two,
  stops on the third,
  and deletes nothing;
- a second writer that publishes a conflicting outcome between two steps,
  asserted through the documented staging hook (`private_storage_replacing_a_record.rs:94 to 95`),
  showing a lost update reads as `requested` and refuses.

### Guest arms

These need a pool,
a clone,
a real destroy or an absence check.
Every one of them kills at a named step,
and the seam is the crate's own measurement-only feature (F16):

- the plan gains a `measurement-routes` route that pauses after each write and before each request,
  the way the clone conversion fence's measurement routes work,
  and the guest kills the paused process at the named step.
  A guest that cannot build that feature runs the crate's test binary inside the guest,
  which is the only other place the staging hook exists;
  no shipped build carries either seam.
- One arm per state of section 4 on a real clone:
  `planned`,
  `requested`,
  `destroyed`,
  `refused`,
  `intact` and
  `unknown`,
  by killing between the write and the request,
  between the request and the outcome,
  and by planting a dependent snapshot for the refusal arm.
- The hold-released walk of section 4 end to end:
  a real `zfs hold`,
  a refused batch,
  the hold released,
  and a second run that destroys the remaining set.
- The batch refusal arm and the own-snapshot control of [VM item 14](cli-git-zfs-measurements-owed-before-acceptance.md),
  repeated through the built wrapper.
- The head-refused arm:
  the head request refused with the clone's own snapshots already gone,
  asserting `head=refused` and that the record names the holder or the new snapshot;
  and the second-check arm beside it,
  a refusal before any head request,
  asserting `head=present` with the batch's outcomes and the clone unmounted.
- The absence rule's two arms:
  a live snapshot whose name matches and whose GUID differs (destroyed by the GUID,
  not the name);
  a head destroyed and another dataset created under the recorded name,
  and the head-gone path refusing to settle while a child carries the recorded GUID or the recorded name carries
another GUID,
  then settling once both are absent.
- The bookmark arm of the decided rule over a plan tuple,
  and the arm whose bookmark is refused because its snapshot went before the plan was made.
- The deferred-origin arm of plan step 15,
  with the mark set before the first delete and between the two checks.
- The delegated-account arm,
  destroying the set under the proposed descendant grants.
- The finish pass after a real head destroy,
  asserting the registration settles,
  the two inventories and then the binding are removed and proved gone,
  the plan is removed,
  and the claim is released last.

Which of these need a guest,
and why:
every arm that reads a live ZFS identity,
sends an ioctl,
or proves absence with a GUID needs a real pool,
which covers the format-independent arms of section 4,
the absence rule,
the batch and head arms,
and every finish-pass arm that reads the pool;
the format,
lifecycle,
folder,
claim-held,
planted-binding and refusal arms need none.

### What a green run would not show

- that a power cut between an outcome write and its flush leaves the earlier record
  (M4 proposes a control-hook stand-in,
  which is not the kernel);
- the one-pool premise of F5's ordering argument (M6);
- any concurrent removals of one slot,
  which the claim excludes by design rather than by a measurement.

## Which plan step builds which part

- Step 14 builds the folder,
  the record's name grammar,
  the fields,
  the first write with every tuple `planned`,
  the reader's two stages with no count ceiling,
  the refusal of a record that does not read or does not match,
  the early mark look's no-plan refusal,
  the bookmark rule's read,
  and the forced receipt's `unmountMountId=none`.
- Step 15 builds both deferred-origin checks' read of the plan's clone and origin identities,
  the first check's pending state and the second check's `head=present` state,
  and their refusals when the record is missing or unreadable.
- Step 16 builds the progress writes,
  the batch's read,
  the head's states,
  the absence rule's two arms,
  the
  head-gone path,
  the re-plan of new own snapshots,
  the finish pass with its claim,
  its live proof,
  its finisher's condition and its tail,
  and the plan's removal at the list's step 13.
- Step 17 builds the cancel command's read and its report,
  what a cancellation does to a plan in each of section 7's states,
  and the removal's adoption of a kept plan.
- Step 18 builds `doctor`'s listing,
  its reports of foreign,
  oversized,
  wrongly owned,
  unreadable entries,
  and its
  staging-name rule conditioned on the plan's owner.
- Step 22 records M1's measured size and cost of a real record beside the arithmetic in section 2.
- Step 9 supplies the verified unmount the plan's `unmountNonce` names,
  and nothing of this record is written before
  it.
- Step 10 and the binding design's build supply the per-slot claim the removal takes (A2) and the binding the removal
  reads by slot (A1).

## Choices this design marks for the project's veto

Each choice is a writer's decision,
adopted so the design is whole,
and the project can veto it at acceptance.
Each gives the options with a concrete example a person would see,
the pros and cons,
and a ranking with the reason for each adjacent pair.
The record's field spellings and the folder's inner name are the design's own detail;
the choices below are the ones that change what a person sees or what another record must do.

The origin cleanup record's moment is not in this list:
it is the origin cleanup design's V4,
and section 3 says this order depends on it (A7).
Choice 11 is this design's own addition:
the staging rule.
The lifetime of a plan a cancellation keeps and the answer for a record that cannot be read
are decided by the seven answers,
and this list no longer asks them.

### Choice 1: where the record lives

Option A,
the wrapper's own folder of the common Git directory,
`monochromatic-git-policy-cli/clone-destruction-plans/<GUID>`,
adopted.

Example:
a person removes `fix-flaky-test`;
the record appears at
`/home/u/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/7828944690444299989`,
beside `recorded-destroys`,
owned by the person,
mode `0600`,
and the rename machinery that moves the shared `monochromatic-git-policy-cli/` parent moves it too.

Pros:
the writer is already the account;
the folder outlives the clone and its registration;
the wrapper's record primitives and `doctor` work unchanged;
a person who finds one wrapper record finds this one;
the folder says what it holds and is one of the three now sharing the parent (the binding design's convention).

Cons:
the account can rewrite the only evidence of what it destroyed,
which is the accepted same-account residual;
the record is inside a Git directory,
so a person who deletes the repository deletes it.

Option B,
the binding design's folder inside the same parent,
`monochromatic-git-policy-cli/worktree-clones/v1/<slot>.plan` (at `07858070bb`).

Example:
the same removal records at
`/home/u/repo/.git/monochromatic-git-policy-cli/worktree-clones/v1/fix-flaky-test.plan`.

Pros:
all three records would live in one inner folder,
and the route's one parent already moves with the wrapper's rename machinery;
one header and one key convention for the whole route.

Cons:
the folder's key is the slot,
so the plan's name would not be the clone's GUID and Choice 2's first option would be lost;
a plan beside an add's binding is read by every reader that opens that folder,
and this plan's reader and the binding's differ;
the binding design's own folder and key are one document's proposal and not a project decision.

Option C,
root state under `/var/lib/monochromatic-git-policy-cli/`.

Example:
the same removal records at
`/var/lib/monochromatic-git-policy-cli/git-worktree-clone-destruction-plans/7828944690444299989`,
root-owned `0644`.

Pros:
a record the account cannot forge,
which matters only if the account is not trusted.

Cons:
every outcome write needs a root channel the decided delegated route does not have;
the account could not read its own removal's evidence without a privileged read;
the project's threat model says root and the account are not a boundary here.

Option D,
a dataset property on the clone.

Example:
`zfs get org.monochromatic:git-worktree-clone-destruction-plan tank/wt/fix-flaky-test` prints one value
of a few hundred bytes.

Pros:
no file at all,
and the record travels with the dataset.

Cons:
8,192 bytes (F3) hold about two dozen tuples at 340 bytes each (F11);
the head destroy takes the property at the moment its own outcome must be written;
the origin is shared,
so a property there would be one record for several removals.

Ranking:
A > B > C > D.
A beats B because the plan's key is the clone's GUID and B's inner folder is keyed by the slot,
and because the two readers that would share B's folder are not one reader.
B beats C because the account writes it without a privilege the route does not have.
C beats D because C can hold the whole record and outlive the head,
and D can do neither.

### Choice 2: the key

Option A,
the clone dataset's canonical GUID,
adopted.

Example:
`fix-flaky-test` is removed and added again at the same path;
the second clone's plan is `17157080670377245572` and the first clone's record is untouched at
`7828944690444299989`.

Pros:
one key for the whole life of the clone;
unique against a re-added slot (F12);
the receipt family already keys by this GUID,
so a person who reads one record reads the other.

Cons:
after the head is gone nothing can compute the key,
so a listing finds the plan and its own fields identify the
clone;
a person cannot guess the name from the path.

Option B,
the slot's name,
as the binding design keys its own record.

Example:
the probe's two worktrees at one path were both `probe-slot` (F12),
so the removed clone's plan and the re-added clone's plan share one name.

Pros:
a person can see it in `worktrees/`;
a command holding the registration computes it without reading ZFS.

Cons:
the name is reused (F12),
so two clones' plans collide and a reader must prove which clone's plan it holds before
it uses one tuple;
the name goes with the registration the settle removes.

Option C,
the slot path's digest.

Example:
two clones at `/home/u/repo/.claude/worktrees/fix-flaky-test` produce one name for both.

Pros:
computable from an argument a person typed,
without Git and without ZFS.

Cons:
one name for two clones at one path;
the digest needs canonical bytes,
and a path that no longer exists cannot be canonicalized.

Option D,
the origin snapshot's name,
as the origin cleanup design keys its own record.

Example:
two clones made from one origin snapshot share the plan name
`git-worktree-clone-origin-9f2c...`,
and one removal's record overwrites the other's.

Pros:
the plan and the origin record would share one key,
and the origin's name already carries the add's nonce.

Cons:
one origin snapshot serves many clones,
so one file would stand for several removals;
a removal's plan outlives the origin's destruction,
so its key could name a snapshot that is gone.

Ranking:
A > B > C > D.
A beats B because the GUID is unique for a clone's whole life while the registration name is reused.
B beats C because B at least names the right clone while its registration lives,
and C can name two clones with one
file.
C beats D because C is unique per slot while D is shared by every clone of one origin snapshot.

### Choice 3: what the first write holds

Option A,
every verified tuple as `planned`,
adopted.

Example:
a clone with two snapshots is proved and unmounted;
the first write records `tuples=2` with both `planned`,
and the removal then refuses at the deferred-origin check
because the origin is marked.
A person runs `doctor` and sees a plan beside a live clone,
nothing destroyed.

Pros:
the plan persists the verified inventory,
as the proposal's words say;
the count is known before the first delete;
the split has "the remaining recorded tuples" to take a prefix of (proposal lines 915 to 917);
a resume never regenerates a set from the names that are left.

Cons:
a refused removal leaves a plan beside a live clone,
which `doctor` must name;
  - the first write is larger than an empty plan would be.

Option B,
tuples enter the plan at submission.

Example:
the same clone's first write records `tuples=0`,
and the first request's rewrite is where the two snapshots
first appear;
a removal killed before its first request leaves an empty plan,
and the next run re-enumerates the live set.

Pros:
the record holds only names that were actually submitted;
a refused removal leaves almost nothing behind.

Cons:
the count cannot be checked before the first delete,
because the count is not known yet;
a resume re-enumerates "whatever names are left",
which is the regeneration the proposal forbids;
the split has no recorded remainder to take a prefix of.

Ranking:
A > B.
A beats B because the proposal's own words make the plan the verified inventory,
and B's resume is the regeneration the proposal's resume rule forbids.

### Choice 4: the tuple lifecycle

Option A,
one line per tuple,
an outcome replaced only by newer evidence or a newer request,
adopted.

Example:
the hold walk of section 4.
After the refused batch the record holds `refused` for `@sanoid-hourly` and `intact` for `@worktree-own`;
after the hold is released and the second request returns,
it holds `destroyed` for both,
on the same two lines.

Pros:
one line per name,
so the reader needs no merge rule and the file stays the size of the set;
the state a later run acts on is the state the pool last showed.

Cons:
the attempts themselves are not kept,
so a person cannot read how many tries a tuple took.

Option B,
one line per attempt,
keyed by name and attempt number.

Example:
the same walk leaves four lines:
`... refused ... attempt=1`,
`... intact ... attempt=1`,
`... destroyed ... attempt=2` and `... destroyed ... attempt=2`.

Pros:
the whole history is readable;
a refused attempt stays visible after a later success.

Cons:
the reader needs a merge rule (the highest attempt wins),
the split's prefix rule must never split one name's
attempts,
and the bound must count attempts rather than objects.

Ranking:
A > B.
A beats B because the only question a later run asks is what the pool says now,
and B's history costs a merge rule and a bound that no longer tracks the objects.

### Choice 5: the flush rule

Option C,
by position,
adopted:
no flush for a write made before an irreversible ZFS step,
which covers this plan's `planned`,
`requested` and `head=requested` writes;
a flush for a write made after an object exists that nothing else can prove,
which is the binding's first write and no write of this plan.
This is the three records' one rule.
The origin cleanup design's V14 adopts it for the three and names this entry and the binding design's V6 and Q9 as
its citations (read at `22be706b09`).

Example:
power is cut right after the plan's `requested` write and before the request returns.
The destroy waits for its transaction group (F5,
one pool assumed by A8),
so the record and the request are in the same or an earlier group,
and the next boot's plan says `requested`,
which the next run proves.
Power is cut right after the plan's outcome write:
the record may fall back to `requested`,
one repeated proof,
and the destroyed snapshot is gone either way.

Pros:
one rule for the three;
a flush is paid only where a loss leaves an object nothing can prove;
the plan's writes are exactly the writes the rule leaves unflushed.

Cons:
two rules to keep,
and the premise needs M6's guest arm.

Option A,
flush every write.

Example:
every outcome write waits for a log commit,
and a power cut after any write leaves a record naming the object it described.

Pros:
one rule;
an outcome survives a crash once written.

Cons:
a log commit on every outcome write whose loss costs one repeated proof,
on the removal's path a person waits on.

Option B,
no flush anywhere.

Example:
the binding's first write can be lost while the clone stands,
leaving a clone that no record names.

Pros:
no write waits for a log commit.

Cons:
that orphaned clone is the one loss nothing can prove.

Ranking:
C > A > B.
C beats A because A pays a log commit for writes whose loss is one repeated proof.
A beats B because A protects the binding's orphan case at the price of a commit,
and B leaves it open.

### Choice 6: what the plan replaces, and whether a removal has a record of its own

Option A,
the plan is the ZFS route's only destroy record,
and the removal intent is the `unmount` property alone,
adopted.

Example:
a managed clone removal leaves one file in `clone-destruction-plans` and nothing in `recorded-destroys` or
`removals-begun`;
a person who wants to know what is happening runs `doctor`,
which names the plan and the standing intent.

Pros:
one record decides one destroy,
so two records cannot disagree;
no existing Btrfs-shaped pass meets a clone it does not understand;
the property is already the request the handler answers.

Cons:
an older release's finish pass has no record it understands for a half-removed clone;
the removal's state is spread over a property,
a plan and the live facts,
not one file.

Option B,
also write a `recorded-destroys` entry beside the plan.

Example:
the same removal leaves the plan and a `recorded-destroys` entry keyed by the registration's id.

Pros:
the existing pass sees that something was destroyed.

Cons:
the existing pass drops a record whose worktree's path still stands
(`worktree_removal_finishes_an_interrupted_destroy.rs:1 to 30`),
which is exactly the state of a clone before its head goes;
two lifetimes and two readers for one destroy.

Option C,
a removal record of its own beside the registration,
plus the plan for the destroy.

Example:
the removal writes `monochromatic-git-policy-cli-removal-state` beside the registration,
and a person reads it to see "unmounted,
plan written,
two snapshots destroyed".

Pros:
a removal's state is one readable file.

Cons:
it is a third record to keep in step with the property and the plan;
a kill between the records leaves them disagreeing,
and nothing proves which is newer;
the proposal's "removal record" words (lines 884 to 885 and 940) do not name a second file.

Ranking:
A > C > B.
A beats C because every fact C would hold is already in the property,
the plan or the live clone,
and a third record is a third thing to disagree.
C beats B because a begun record read by the Btrfs pass misreads the clone's state,
while a record of this route's own shape is at least not read by a pass that would act on it wrongly.

### Choice 7: copied identities or a reference to the binding

Option A,
the plan copies the clone and origin identities and refers to the binding for the origin-side inventory,
adopted.

Example:
the plan names `7828944690444299989` and its origin,
and reads the origin-side snapshot list from the add's binding record when it runs the third check;
a binding that does not read refuses the removal.

Pros:
one recording of the add-time inventory,
so there is nothing to disagree;
the plan stands alone for everything it must prove about the clone and origin it destroys.

Cons:
a damaged binding record blocks removals the plan alone could have completed.

Option B,
the plan copies the origin-side inventory too.

Example:
the plan records every origin-side snapshot and bookmark tuple,
so a removal completes without the binding.

Pros:
no dependency on the binding's readability at removal time.

Cons:
the origin-side inventory is unbounded and grows with every sanoid run,
so the record would carry a second
bound;
it is a removal-time copy of an add-time fact,
and the decided rule forbids updating the recorded lineage.

Ranking:
A > B.
A beats B because the plan's bound and its single reading stay bounded,
and a damaged binding is a state the removal should refuse rather than paper over.

### Choice 8: no expiry

Option A,
a plan is never removed because of its age,
adopted.

Example:
a plan from a removal refused three weeks ago still stands,
`doctor` still names it,
and the next removal of that slot adopts it after proving every tuple.

Pros:
the evidence of a destroyed snapshot survives as long as the clone or the origin does;
no timer decides anything about a person's history.

Cons:
an abandoned plan stays in the folder,
and `doctor` reports it on every run until a person settles the clone.

Option B,
expire a plan after a fixed period.

Example:
the same plan is removed after thirty days,
and a later removal sees a clone whose snapshots are partly gone with no record of why.

Pros:
the folder does not grow.

Cons:
it erases the only evidence of what was destroyed and turns a resume into an unexplained set,
which the proposal's own rule forbids (line 968).

Ranking:
A > B.
A beats B because the folder's growth is one file per clone that a `doctor` line can name,
and B's saving is that file's bytes against the loss of the only record.
A plan a cancellation keeps is not expired by this choice either;
the seven answers' kept-plan rule does.

### Choice 9: what finds a plan

Option A,
one `stat` of the plan folder before every forwarded command,
and a listing only when it exists,
adopted.

Example:
`git status` in a healthy repository does one `stat` of
`.git/monochromatic-git-policy-cli/clone-destruction-plans` and opens nothing;
after a killed removal it lists one file,
settles its registration,
and removes it.

Pros:
one record holds the key,
so no second file can disagree;
nothing is written inside a registration,
where Git deletes what it does not know (F7);
the healthy state costs one `stat`.

Cons:
a leftover costs a listing on every command until it is finished,
which `doctor` reports.

Option B,
a pointer file inside the registration,
as the earlier revision had.

Example:
the pointer at
`<common>/worktrees/fix-flaky-test/monochromatic-git-policy-cli-clone-destruction-plan` names the GUID.

Pros:
no listing at all;
a command that holds the registration finds the plan with one `stat`.

Cons:
F7's probe deleted that pointer with `git worktree prune` in exactly the state a plan is needed,
and `git worktree remove` deletes it always;
the wrapper's own identity record accepts that risk under the Git lock,
and this record does not need to.

Option C,
a pointer file in the wrapper's common folder,
`<common>/monochromatic-git-policy-cli/clone-plan-pointers/<slot>`.

Example:
the pointer survives a prune,
and the finish pass reads it from the registration's id.

Pros:
no listing,
and no file inside a registration.

Cons:
a second record that must be kept in step with the plan and removed with it;
a pointer naming a plan that is gone is one more state doctor must report.

Ranking:
A > C > B.
A beats C because one record with its own key needs no second file to agree with it.
C beats B because C survives what Git does to a registration,
and B does not.

### Choice 10: one record with no count cap, or a folder of chunks

Option A,
one record,
bounded by the exact byte formula and two-stage reader of section 2 with no count cap,
adopted.

Example:
a clone with 40,000 snapshots writes a 13.6 MB record;
the removal is not refused for its size,
the reader reads the record's own bytes,
and the request set is submitted in one request or in the largest fitting prefix (the list's step 7).

Pros:
one read gives the whole plan,
so the bookmark rule needs one record and nothing must be reconciled;
the declared count makes a torn record detectable;
no workload-sized number ships and nothing can refuse a removal for its snapshot count.

Cons:
the record is as big as the set,
and the reader holds that many bytes in memory at once.

Option B,
one record per chunk plus a small index record.

Example:
the same clone records ten chunk files of 4,000 tuples and an index naming them;
the bookmark rule reads the index and then the chunks it needs.

Pros:
no record is large;
the reader's memory per file stays small.

Cons:
the union of the files is the plan,
so a kill between two chunk writes leaves a union that is not a plan and every
reader must detect that;
the index and the chunks can disagree;
a person's repair is several files.

Ranking:
A > B.
A beats B because the common case is one batch and a handful of tuples,
and B's small files cost every reader a union rule for the rare huge set.

### Choice 11: who removes a killed writer's staging name

Option A,
`doctor` removes it only when a plan with that GUID stands and its `owner` is gone,
adopted.

Example:
a removal is mid-write when `doctor` runs in another worktree;
the staging name `.<GUID>.<random>.partial` is left,
the removal's rename lands,
and `doctor` reports the staging name
as a live writer's.

Pros:
a running removal's rename is never broken;
the `owner` field has a reader;
an abandoned staging name is still removed.

Cons:
a staging name with no plan beside it waits for a person,
because it can be a live writer's first write.

Option B,
remove every known staging form unconditionally,
as the store's doctor does.

Example:
the same `doctor` run deletes the staging name,
and the removal's rename answers `ENOENT`.

Pros:
the folder never holds litter.

Cons:
it can break a running removal,
which holds its staging name across a destroy.

Option C,
the removal's own sweep removes them.

Example:
the next `git worktree remove` in the repository removes the staging names it finds.

Pros:
no listing in `doctor`.

Cons:
a repository that runs no removal keeps them,
and plan step 18 is where the project already put this removal.

Ranking:
A > B > C.
A beats B because a live writer can own the name,
and the store's unconditional rule was written for a writer that holds
nothing across a wait.
B beats C because `doctor` is the place the project already named and C would put a sweep on a path that may never run.

## Decided by the seven answers

The project answered the seven questions the three record designs shared,
and the record is `doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
accepted 2026-11-22 and recorded 2026-11-22T21:16:30-05:00,
read whole.
Five of its answers bear on this record;
each is quoted with the record's heading,
and the options not chosen are listed once here,
so this document asks none of them again.

- "What trace of a destruction outlives the removal":
  "Nothing beyond what the removal printed,
  and its records while they stand."
  This design keeps no trace and builds none;
  an audit-log line,
  a notice on later commands and a history `doctor` prints were not chosen.
- "A record that cannot be read":
  "Change nothing,
  name the file,
  and wait for a person."
  Every reader in section 5 refuses and changes nothing,
  and the pass moves and removes none.
  Moving an unreadable record into an `unreadable/` folder and dropping it after one line were not chosen.
- "A slot whose recorded clone is gone and cannot be proved gone":
  "Print an explicit command that removes the record after you confirm."
  The command is the binding design's "The settling command",
  and this design designs no second one;
  section 7's "A clone gone whose absence cannot be proved" states what it must do to a plan whose head is not
  `gone`,
  in words that design can quote.
  Keeping the slot blocked and reporting it were not chosen.
- "How long the plan a cancellation keeps stays":
  "The next removal of that clone reads the record,
  proves it,
  carries the loss forward and ends it."
  Section 7's "A cancellation,
  and the plan it leaves" states the one rule and the two costs of that lifetime.
  Keeping it until a person acknowledges it,
  and for a fixed period,
  were not chosen.
- "Whether a forced removal records who it ended,
  beyond the journal":
  "The handler's journal lines are the record."
  The plan gains no field for it,
  and section 7 says why.
  The handler's receipt naming the ended processes,
  a count line `ended=<count>` in the plan,
  and a record of its
  own were not chosen.

The record's other two answers are the origin cleanup design's:
"Which commands may destroy a leftover origin snapshot" (worktree commands only) and
"How often a pending origin cleanup speaks" (on a change only).
This design cites them where they bear,
in section 5's "Readers that do not exist" and in the pass's gate line.

No question remains for this record.
Where an answer leaves a state to this record,
the section above names it.

## Where this design and the sibling records differ

The review names fourteen points on which the three record designs disagreed.
Each is answered here with what this design does now and which record,
if any,
decides it.
The siblings are read at their branch tips,
the origin cleanup record at `22be706b09`
and the binding record at `07858070bb`,
and both are other rounds' documents,
reviewed by the third pass and not final:
nothing below adopts a sibling's answer as decided,
and neither document was edited.
The removal's one order is the origin cleanup design's list (section 3),
and none of the three writes one of its own.
The binding design's "The record's end" still cites the list at `0973f2fc2a`,
which has thirteen steps;
its steps from 8 onward are one lower than the fourteen-step list it describes,
so the points below name O's numbers and B's meaning.

1.  **Folder family.**
    All three now share the parent `<common>/monochromatic-git-policy-cli/`,
    and differ only in the inner name:
    this design `clone-destruction-plans/`,
    the origin cleanup record `origin-cleanups/`,
    and the binding record `worktree-clones/v1/`.
    The binding design's convention allows either a `v1/` segment or a header line,
    and each document names its own reader's first fact as the key.
    No project record decides an inner name.
    This design's choice is Choice 1,
    with the binding record's inner folder as option B;
    the top-level alternative the earlier revision weighed there is gone,
    because the binding design withdrew it.
2.  **Key.**
    This design keys by the clone's dataset GUID,
    the binding record by the slot and the origin cleanup record by the
    origin snapshot's name.
    No record fixes a uniform key,
    and the binding design's convention says one is not possible;
    the set reaches from any key through the binding's `cloneGuid=`,
    `originSnapshot=` and `slot=` fields.
    This design's choice is Choice 2,
    with both siblings' choices beside it.
3.  **Encoding of names and the shape of the file.**
    This design agrees with the binding record and not with the origin cleanup record.
    The plan writes a `#` version line first,
    `key=value` lines in one fixed order,
    ZFS names verbatim (F2 shows a name holds no newline and no `=`),
    paths in hex,
    and GUIDs and groups in canonical decimal;
    the binding record states the same three encodings.
    The origin cleanup record writes `key=value` lines and then one JSON object,
    with GUIDs as decimal strings and the path as text when no escape is needed plus hex,
    so a reader of its record cannot read this one;
    that difference is listed,
    not an agreement.
    Its V2 now lists the all-`key=value` form as its option D and still takes the JSON object;
    the difference is that document's own choice and its list carries it.
    No project record decides the file shape;
    this design follows the slot store's `#` line,
    and the tuple lines stay positional with the name last after
    `name=`.
4.  **The moment of the origin carrier (X1).**
    This design reads the origin cleanup record and never writes it;
    the record is published at the list's step 6,
    which is that design's V4 option A at `22be706b09`:
    after every refusal and before the first destroy.
    The binding record's step 2 confirms the same act and writes nothing there;
    its step 2 is the list's step 6.
    No project record fixes the moment,
    so this design's order depends on that choice;
    if the project vetoes it,
    this design's step 6 and that design's reopen together.
5.  **Whether a blocked origin keeps the slot (X2).**
    Decided by the proposal's "Settling frees the slot but requires recovery without its Git entry" (lines 1462 to
    1463)
    and "settling does not wait for the origin" (lines 469 to 470).
    The binding record now removes its three files at the list's step 12,
    whether or not the origin cleanup is
    blocked,
    and withdrew the earlier rule that kept it while blocked;
    and the proposal's two sentences above decide it.
    This is no longer a difference between the three,
    and this design's end waits for the list's step 12,
    not for the origin's cleanup.
6.  **The serializer of two removals.**
    All three take the per-slot claim (A2,
    F15):
    the origin cleanup record now names the claim the binding design defines as the one a removal takes,
    and the binding design's claim file is that object.
    The proposal's per-slot-claim sentence (lines 909 to 910) and the claim file decide.
7.  **Durability.**
    The three share one rule by position,
    and all three rank it first:
    the origin cleanup design's V14 adopts option C and names this design's Choice 5 and the binding design's V6
    and V19 as its citations (read at `22be706b09`).
    This design and the origin cleanup record write unflushed;
    the binding's own writes are the flushed ones,
    and its V19 now ranks the by-position rule first too.
    This design's Choice 5 states the rule in that wording and cites it.
8.  **What a reader does with a bad entry.**
    Partial,
    foreign and oversized:
    all three refuse and change nothing.
    Wrongly owned:
    all three refuse,
    including the origin cleanup record's owner-and-mode rule.
    Unreadable:
    all three refuse by default,
    and whether to move a record aside or drop it is the origin cleanup design's
    question four,
    which carries the absence arm too;
    the seven answers decide it,
    and section 5 states it.
    No project record decides it.
9.  **Who removes staging names.**
    This design removes one only when a plan with that GUID stands and its `owner` is gone (Choice 11);
    the binding record removes one only when no live claim stands;
    the origin cleanup record removes none and reports them.
    The store's decision covers the store only.
    Three rules,
    each listed in its own document;
    this design does not edit theirs.
10. **Listing on the command path.**
    This design stats its folder before every forwarded command and lists it only when it exists (Choice 9);
    the origin cleanup record leaves its call site to its question one;
    the binding record reads its folder only in `doctor`.
    The store's "no listing per sweep" is about a sweep and a folder that always exists,
    so it does not decide.
    This design's Choice 9 names the pointer alternative it dropped.
11. **Who `doctor` runs as.**
    This design assumes the account (A6);
    the origin cleanup record says the account,
    and the binding record says root may read and gives no root mode.
    This design agrees with both.
12. **What ends each record and who removes it.**
    The origin cleanup design's list places the tail:
    the origin record is removed by its own absence proof at its step 10 (or stands blocked),
    this plan is removed at its step 13,
    after the binding's files are proved gone at step 12,
    and the claim is released at step 14.
    The binding design's "The record's end" states the order of its three files there and names this design's
    finish pass as a finisher;
    its "The condition a finisher meets" is the condition the pass and this design's section 5 use.
    The proposal's steps 4 and 5 (lines 947 to 956) decide the origin attempt and the settle;
    nothing decided the tail before that list,
    and it is the origin cleanup design's V13.
    This design's finish pass does the same four points (section 5),
    and the origin cleanup record now says the pass leaves a plan whose head is not `gone`,
    which is section 5's rule;
    the two agree there.
13. **Whether a removal has a record of its own.**
    This design says the `unmount` property is the intent and the plan is the record (Choice 6);
    the binding record says what identifies the removal's state is the plan's presence and the live facts,
    which agrees with this design;
    the proposal's "removal record" words (lines 884 to 885 and 940) do not name a second file.
    No project record decides,
    and Choice 6 lists the option of a record beside the registration.
14. **Bounds and caps.**
    This design's bound is the exact sum of named field bounds with no rounding and no count cap (F11,
    section 2,
    Choice 10).
    The binding record's bound is the exact 34,797 bytes its current text states;
    the origin cleanup record's is the exact 13,375.
    All three derive their bounds from named terms without rounding,
    as the accepted store did
    (`doc/decision/cli-git-zfs-slot-identity-store-format.md` lines 140 to 141).
    No project record decides a cap on a set the kernel does not bound,
    and this design ships none.

### What this design needs from the two sibling documents

The two requirements the earlier revision put on the binding design are answered in its text:
"The record's end" states the order among its three files,
and "The condition a finisher meets" states the condition the pass uses and names the pass among the finishers,
so the ground that the pass holds no claim is gone.
Two requirements remain.
B's "The record's end" cites the thirteen-step list at `0973f2fc2a`,
and it should cite the fourteen-step list at the tip this design cites (`22be706b09`),
whose steps from 8 onward are one higher because of the second deferred-destroy check;
by what each step does,
B's numbers and O's name the same points.
And B's "The settling command" refuses a standing plan,
which leaves a plan whose head is not `gone` beside a clone gone outside the wrapper with no path at all;
it must take the exception section 7's "A clone gone whose absence cannot be proved" states,
in that section's words.
The origin cleanup design's list holds every point this plan needs,
including the second check and a kill inside the origin record's publication;
if a later revision moves the plan's removal before the binding's,
section 7 and the finish pass move with it.

## Dependencies on sibling designs

- The add's durable binding record (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
  read at `07858070bb`).
  This design reads it by `<slot>`,
  the last component of the path `targetHex` holds,
  and compares its `slot=`,
  `cloneGuid=`,
  registration and origin fields (A1),
  and takes its per-slot claim (A2).
  If that design changes its key or its claim,
  this reader changes with it.
  It is another round's document,
  so this is a dependency and not an adopted answer.
  Its "The record's end" and "The condition a finisher meets" are what section 5's pass follows;
  the one open requirement on it is its citation of O's thirteen-step list,
  under "What this design needs from the
two sibling documents".
- The record that carries origin cleanup (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
  read at `22be706b09`).
  This design cites its "The removal's one order of writes and removals" for every point of the removal (section 3),
  requires its record to stand before this design's first delete and refuses otherwise (A7,
  step 6),
  and follows its V14 for the flush rule and its question four for a record that cannot be read.
  Its moment is its own V4,
  open to the project's veto.
- The deferred-destroy checks (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`).
  They read the plan's clone and origin identities as the planned ones,
  and refuse when it does not read.
- The cancelled,
  timed-out or killed removal and the cancel command
  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`).
  The cancel command reads the plan for its report and keeps it;
  what a clone in that state mounts into is that design's question,
  not this one's (see Questions).
- The add-time Git lock (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`).
  No dependency:
  this design rests on the per-slot claim,
  not on Git's `worktree lock` (F6).
- The forced arm,
  the dirty-main restore,
  the private propagation and the lost-last-event triggers:
  no dependency.
  None reads or writes this record.

## Measurements owed

No guest ran this round.
Each item names the probe that would settle it and the step that owes it.

- M1,
  the record's real size and its write cost on the removal's path.
  Host,
  under the acceptance workload:
  build one record from a clone's real inventory,
  compare its bytes with F11's `10,128 + 340 * n`,
  and time its unflushed write against an interleaved control that writes the same bytes without the rename,
  reported as a ratio with its spread under rule `QNB`.
  It checks the bound's arithmetic against a real record and sizes no shipped limit.
- M2,
  the kill arms of section 4.
  With the `measurement-routes` build,
  pause after each write and before each request,
  kill at the named step,
  and assert the record's reading and the next run's refusal.
  Guest,
  because the request needs a pool.
- M3,
  the absence rule's two arms.
  A live snapshot whose name matches and whose GUID differs,
  asserting the GUID decides;
  a head destroyed and another dataset created under the recorded name,
  and both readers refusing while a child carries the recorded GUID or the name carries another GUID,
  then settling once both are absent.
  Guest.
- M4,
  the loss a crash can cause.
  A control that fails between the staging write and the rename
  (the staging hook at `private_storage_replacing_a_record.rs:94 to 95`),
  asserting the old record is read and the loss reads as `requested`;
  a kernel power cut cannot be forced,
  so the kernel's ordering stays unmeasured and is stated as such.
- M5,
  the write cost on the removal's path,
  only if the project answers the flush rule with option A (flush every write),
  or with a set-wide answer that flushes this record's writes:
  the plan's write time against an interleaved control that writes the same bytes without the sync,
  reported as a ratio with its spread under rule `QNB`.
  Host,
  under the acceptance workload.
- M6,
  the one-pool premise of F5 (A8).
  In the guest,
  read the mount sources of the common Git directory and of a clone dataset's path and assert one pool,
  then assert F5's ordering with a `strace` of the destroy and a read of the record's file after it returns.
  If the premise fails,
  the flush rule's premise and this design's unflushed writes reopen.

## Limitations

- The absence rule's head-gone arm rests on F4's kernel rule and is unmeasured.
  No plan has ever been resumed,
  so the settle step inherits that limit.
- The record cannot distinguish "the kernel destroyed nothing" from "the kernel destroyed nothing and a name was
  missing",
  because the request ignores missing names (F4);
  a name that vanished between the enumeration and the request reads `intact` where it is gone,
  and the next absence proof corrects it to `destroyed`.
  That is one rewrite of the record,
  not a wrong delete.
- A record's key cannot be computed after the head is gone,
  so exactly one part of a removal depends on the folder's
  listing (Choice 9),
  and a plan whose folder holds a foreign entry waits for `doctor`.
- The plan is evidence,
  not authority.
  A forged plan can make a bookmark pass,
  which the accepted same-account residual already allows,
  and nothing here narrows it.
- A plan cannot report an outcome it never recorded.
  A kill inside an ioctl leaves `requested`,
  and the proof that follows is the only truth the record can carry.
- The record has no count ceiling and its refusal for size is gone;
  the reader's memory follows the record's own length (section 2).
  A very large clone therefore writes a very large record,
  and the reader holds it;
  Choice 10 weighs the chunked alternative.
- A plan a cancellation keeps lives until the clone's next removal adopts it (the seven answers' kept-plan rule),
  and nothing in a run has exercised that adoption.
- The second deferred-destroy check's pending state and the first check's are unmeasured:
  no removal has been refused at either check.
- A forced receipt that covers several mount namespaces is unmeasured here,
  and the plan's `unmountMountId=none` for it is the format's answer,
  not a measured one.
- The settling command's removal of a plan whose head is not `gone` is a rule this design states for another
  design's command,
  and no run exercises it.

## What this design does not cover

- The binding record's home,
  key and format (the binding design's),
  and the origin cleanup record's (its own).
- The deferred-destroy checks' own design,
  and what each of their refusals prints.
- The cancel command's proof of the slot it may reverse,
  and what a cancelled removal restores.
  This design says what a cancellation does to the plan (section 7),
  not what it restores to the person.
- What a clone with a partial-destruction plan mounts into:
  that is the cancelled or killed removal design's question,
  and the evening's forced-removal decision answers the reattach for a cancellation.
- The forced arm,
  the holder identification,
  who ends a holder and by what authority,
  and the caller's session.
- The unmount protocol,
  the staging folder and the reattach.
- The origin's destruction and the underlay's removal,
  beyond naming where the plan's facts feed them.
- The root-destruction alternative the project did not select,
  whose plan would live under root control
  (proposal lines 1643 to 1644).
- Migration:
  nothing of this route is released,
  so no written plan exists to migrate.
- The exact sentences of every refusal.
  This design states what each names and that it says nothing was destroyed;
  the wording belongs with the SPEC section that carries it.

## References

- [The accepted design](../decision/cli-git-zfs-worktree-design.md)
- [The design proposal](cli-git-zfs-worktree-design-proposal.md)
- [The implementation plan](cli-git-zfs-worktree-implementation-plan.md)
- [The measurements owed before acceptance](cli-git-zfs-measurements-owed-before-acceptance.md)
- [The slot identity store format](../decision/cli-git-zfs-slot-identity-store-format.md)
- [The staged folder design](cli-git-zfs-staging-under-cache-design.md)
  and [its decision](../decision/cli-git-zfs-staging-under-cache.md)
- [The step 8 sweep choices](../decision/cli-git-zfs-step-8-sweep-choices.md)
- [The forced removal decision](../decision/cli-git-zfs-forced-removal-authority-and-reach.md),
  [the deferred-destroy refusals decision](../decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md),
  and [the dirty-main restore decision](../decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md)
- [The wrapper specification](../../package/git-policy/cli/SPEC.md)
- The sibling record designs,
  read at the commits named in "Where this design and the sibling records differ":
  the origin cleanup record and the add's binding record.
- The gate reviews this round answers,
  at
  `~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review.md`,
  `~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-second-pass.md`
  and
  `~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review-diff-check.md`.
- OpenZFS at tag `zfs-2.4.4` (commit `71a9f95`) and Git at `v2.56.1`,
  read in shallow clones this round kept in its scratch folder.

## This round

This round ran no probe on any pool and started no guest.
It merged `main` for the seven answers and read the record whole,
read the two sibling documents at their tips (`22be706b09` and `07858070bb`)
and the two designs that state requirements on this one
(the cancelled-removal design at `eb7a78eccc` and the deferred-destroy design at `946578a8e5`),
and checked every path-and-line citation in the changed text.

What ran:

- `git merge --no-ff main`,
  for the seven answers;
- `git show` of the two sibling documents at their tips;
- the citation check over this document's path-and-line citations,
  whose output is `scratch/citation-check.txt`;
- the linter named in the round's brief,
  until it reported nothing.

What did not run this round:

- no probe,
  no guest,
  no ZFS command,
  no ioctl,
  no container,
  no mount or unmount,
  and no timing.

The document is unapproved and non-final.

## Revisions

- `a7823980b9`,
  committer time `2026-11-22T17:27:06-05:00`:
  the first draft,
  written on `docs/planning-cli-git-zfs-removal-destruction-plan-record-design` from `main` at `5dd8883a60`.
  It fixed the home,
  the fields,
  the writer's order and the readers,
  with the pointer beside the registration,
  tuples entering the plan at submission,
  and the origin record written after the settle.
- `645863a547`,
  committer time `2026-11-22T18:34:13-05:00`:
  the revision the gate review asked for.
  It settles what the first write holds (every verified tuple as `planned`,
  Choice 3),
  the tuple lifecycle and its retries (Choice 4),
  the absence rule's two arms and the head-gone resume path,
  the finish pass's driver (one `stat` of the folder,
  no pointer;
  Choice 9),
  the reader's two stages and its labelled placeholder ceiling,
  the per-slot claim instead of the Git lock,
  and the order's step that now reads the origin record instead of writing it (A7).
  It replaces the example with an unpromoted clone whose GUIDs and groups are consistent,
  switches names to verbatim and the header to a `#` version line,
  adds Choices 3,
  4,
  10,
  11 and 12,
  replaces the two questions with one,
  adds "Where this design and the sibling records differ" with the review's fourteen points,
  and corrects every citation the review named and every one the citation check found wrong.
- `0e1107a1f3`,
  committer time `2026-11-22T18:35:17-05:00`:
  two cross-references corrected,
  the plan's chunked alternative from Choice 3 to Choice 11,
  and the copied-identities choice named where the plan refers to the binding.
- `488d95fd17`,
  committer time `2026-11-22T19:22:57-05:00`:
  the revision the second gate pass asked for.
  It cites the origin cleanup record's "The removal's one order of writes and removals" and its kill list instead
of stating an order (section 3),
  places the plan's removal at that list's step 12 after the binding's files at step 11 and before the claim's
release at step 13,
  has the finish pass take the per-slot claim,
  prove the head's absence from the live pool through the route's gate,
  refuse when the live pool contradicts the record,
  and finish the list's steps 10 to 13,
  drops the tuple-count ceiling and bounds the reader by the record's own length,
  restates Choice 5 and Choice 10 in the origin cleanup record's V14 and question four wording,
  rewrites the fourteen sibling points at `0973f2fc2a` and `c144499eb9`,
  and corrects the lows the pass listed:
  `EBUSY` for a hold,
  `refused` to `destroyed`,
  the tuple totals,
  the example's provenance sentence,
  the wrapper's path convention,
  the folder's lifetime,
  the header bound's assumptions and two citations.
  It changed three parts the pass found sound only where a fix forced it:
  the finish pass takes the claim (X3),
  the first write no longer checks a ceiling (N2),
  and the path field is hex alone where the wrapper's convention allows two forms (N4).
- `c3e94386a7`,
  committer time `2026-11-22T20:31:36-05:00`:
  the revision the diff check and the evening's three decisions asked for.
  It follows the origin cleanup record's fourteen-step list at `8cc31f59a4`,
  names the binding design's "The record's end" for the order of its three files and its "The condition a finisher
meets" for the pass's binding condition,
  and states what a cancellation leaves in the plan in each state,
  adds Choice 13 for a kept plan's lifetime,
  gives the first and second deferred-destroy refusals their separate plan states,
  keeps `head=refused` for the head request alone,
  makes the early mark look a no-plan refusal,
  records `unmountMountId=none` for a forced receipt over several namespaces,
  and says an add which fails writes no plan.
  It carries the origin cleanup design's four questions as they stand and adds only question five,
  whether the plan records who a forced removal ended,
  and corrects the two statements about the origin cleanup record that its round fixed,
  the reuse case that printed a line on every command (Z5),
  and the format width that bounds a count (Z8).
  It changed the parts the earlier passes found sound only where a fix forced it:
  the plan's step numbers moved with the origin cleanup list,
  the pass's binding condition became the binding design's (Z1),
  the reuse rule became the pass's (Z5),
  and the count sentence became a format width (Z8).
- `648a31271d`,
  committer time `2026-11-22T20:32:14-05:00`:
  one sentence in section 5 naming what stands when the finish pass itself is killed:
  the claim's next generation,
  a standing inventory as the same step's leftover,
  and a plan not yet removed as the last witness.
- `cd9ec96eff`,
  committer time `2026-11-22T21:36:39-05:00`:
  the revision the third check and the seven answers asked for.
  It states one rule for a cancellation in both the cancel command's section and section 7,
  answers the cancelled-removal design's two requirements and the deferred-destroy design's,
  has the finish pass leave any head that is not `gone` and remove an empty folder,
  adds the settling command's words for a plan whose head is not `gone`,
  renames `slotName` to `cloneDataset` and derives `<slot>` from `targetHex`,
  retires Choice 10,
  Choice 13 and question five into "Decided by the seven answers",
  adds the three planted test arms and lists the receipt option among the answered alternatives,
  and corrects the V9 citation,
  the count sentence,
  the inside-13 state and the add's withdrawn unmount.
  It changed the parts the earlier checks found sound only where a fix forced it:
  the field's name and the key's derivation (Y6),
  the clause on the pass's heads (Y13),
  and the two decided entries (the seven answers).
  A later commit that only adds or corrects this list is not listed.
