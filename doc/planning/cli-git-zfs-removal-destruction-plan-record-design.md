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
- the sibling record designs,
  each read with `git show` at the commit named here:
  "The record that carries origin cleanup" at `4daa6c66ce`,
  and "Where the add's durable binding record lives,
  and its format" at `1e9bc35f1e`;
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
Every citation in this document was printed beside the place it cites by
`scratch/citation-check.ts`,
and each now matches.

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
  read at `1e9bc35f1e`,
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
  the one number that is a workload choice is a labelled placeholder whose measurement is named,
  and it may not ship before that measurement lands.
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
- `foundation/path_bytes_in_records.rs:127` states the hex form a record gives a path,
  "hex_of_path is the form such a record always takes",
  and `worktree_removal_interrupted_destroy.rs:276` uses it for a path in a record.
- The per-slot claim is the add's binding record's (`<slot>.claim.<generation>`,
  read at `1e9bc35f1e`,
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
  which is 44,112 bytes at 100 tuples and 350,112 at 1,000.
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
  and in the binding record's design at `1e9bc35f1e`,
  whose use of the claim for removals is that document's proposal and not a project decision.
- F16.
  The crate has a measurement-only feature,
  `measurement-routes`,
  off by default and never enabled by a shipped build
  (`package/git-policy/cli/Cargo.toml:71 to 75`).
  It is the only seam the guest kill arms can use (section 8).

## Assumptions

- A1.
  The add's durable binding record is found by the slot's name,
  as the binding design keys it at `1e9bc35f1e`.
  Its home and key are that design's,
  not this one's;
  this design reads it by slot,
  compares its repository,
  registration and clone GUID fields with the live objects,
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
  The origin cleanup record is published by the origin cleanup design's writer before this removal's first snapshot
  delete,
  at that design's moment (its V4,
  read at `4daa6c66ce`,
  and the fix its round is told to make).
  This design reads it and never writes it.
  No project record fixes that moment,
  so this order depends on a choice still open to the project's veto.
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
  The tuple ceiling in section 2 is a labelled placeholder,
  not a measured limit,
  and it may not ship before M1 lands.
- L6.
  The absence rule's head-gone arm rests on F4's kernel rule.
  No plan has ever been resumed.
- L7.
  F5's ordering premise (one pool) is unstated by any record and unmeasured;
  M6 is the check.

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
  the folder exists only while a removal runs or after a kill.
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
slotName=tank/wt/fix-flaky-test
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
`originSnapshotCreatetxg` is 15,
at or below the origin snapshot's own group,
which is what a transferred set looks like ([VM item 13](cli-git-zfs-measurements-owed-before-acceptance.md)).

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
- `slotName` is the clone's full dataset name,
  verbatim.
  A ZFS name holds no newline and no `=` (F2),
  so the value cannot forge a line,
  and it is the name the destroy requests use.
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
  hex of them,
  as every path in a wrapper record is written
  (`foundation/path_bytes_in_records.rs:127`,
  `worktree_removal_interrupted_destroy.rs:276`).
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
and the first stage is why the count can be trusted:

1. it opens the file without following a link,
   reads at most 10,128 bytes,
   and parses the version line and the fields,
   requiring the count before the tuples;
2. it then reads at most `340 * count` more bytes from the same descriptor,
   and refuses the record if the file holds one byte more.

Both stages are one new reader around the repository's `read_plain_file` shape
(`private_storage.rs:817`),
which takes its limit before the file is read.
The writer refuses to publish a record its own reader would refuse,
which is the receipt family's rule (`SPEC.md:6154 to 6155`).

The count itself is bounded by a **placeholder ceiling of 4,096 tuples**,
about 1.4 MB (F11),
which is labelled a placeholder and not a measured limit:
it exists so steps 14 to 18 can be built without a number nobody has measured,
it is written as one named constant with this sentence beside it,
and it is replaced by M1's measured ceiling when step 22 records that measurement beside the value.
Until then a build that reaches the ceiling refuses before the first delete
and names the count and the ceiling,
which costs nothing a person cannot see;
no release ships the placeholder (L5).
The alternative shape,
a plan split across several records,
is Choice 3.

The bound costs what a clone costs,
not what a wait costs:
a hundred tuples are 44,112 bytes and a thousand 350,112 (F11),
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

The removal's writes,
in order:

1. The lineage and ownership proof of proposal step 1,
   and the printed list of names to be lost (step 2,
   lines 886 to 887).
2. The removal intent:
   the durable `unmount` property on the clone,
   which is the request the handler answers (`SPEC.md:6093`,
   "the removal intent is step 14's").
   This route writes no begun record beside the registration (Choice 6).
3. The handler's verified unmount,
   and the removal's check of the receipt's own fields for its nonce.
4. The plan's first write:
   `head=present`,
   every verified tuple as `planned`,
   `tuples=<count>`,
   the clone and origin identities of step 1,
   `unmountNonce` and `unmountMountId` from the verified receipt.
   A plan over the ceiling refuses here,
   before anything is deleted.
5. The pre-delete refusals:
   the deferred-origin check and the fresh all-namespace verification,
   both of which read the plan.
   A refusal leaves the plan beside a live clone (section 4).
6. Confirm the origin cleanup record stands,
   written by the origin cleanup design's writer at its own moment (A7).
   A removal that finds no such record refuses before its first delete,
   because the origin's identity would otherwise have no home once the clone is gone.
7. For each snapshot request:
   a rewrite with that request's tuples as `requested`,
   the request,
   a rewrite with the request's per-name outcomes.
8. Before the head destroy:
   a rewrite with `head=requested`,
   the destroy,
   a rewrite with `head=gone`,
   `refused` or `unknown`.
9. The registration and underlay settle (proposal step 5).
10. The plan's removal (section 7).

The first write is after the unmount and not before it,
which is the proposal's order and also the honest one:
a plan written earlier would name an unmount that never happened,
and a later run would have to tell a real plan from one made for a request that was abandoned.

Step 6 is a dependency on a choice the project has not made.
The origin cleanup design's adopted moment is before the removal's first delete
(its V4 at `4daa6c66ce`),
and its round is told to write the record after this removal's pre-delete refusals.
No project record fixes that moment,
so if the project vetoes it,
this step and the origin design's reopen together:
this design's order is only sound while a record carrying the origin's identity stands before this removal's first
delete.

### What a kill at each point leaves

- Before step 2 completes:
  no plan.
  The clone and every snapshot stand,
  and a later removal starts at step 1.
- Between step 2 and step 3:
  no plan,
  the durable intent stands,
  and the handler may still act.
  A later removal finds the intent and the intact clone and verifies again.
- Between step 3 and step 4:
  no plan and a verified unmount.
  A later removal verifies the unmount again and writes the plan;
  nothing was destroyed.
- Between step 4 and step 5:
  a plan with every tuple `planned` and the origin cleanup record not yet written.
  A refusal at step 5 leaves the same plan beside a live clone.
  Nothing was destroyed.
- Between step 5 and step 6:
  the same plan;
  a removal that finds no origin record refuses before its first delete and keeps the plan.
- Between step 6 and step 7:
  the origin record stands,
  the plan is `planned`,
  the registration stands,
  nothing is
  destroyed.
- Inside step 7:
  the record says `requested` for the whole request,
  the origin record stands,
  and the registration stands.
  That is the unknown outcome of section 4,
  and a later run proves each tuple by the live inventory before it does anything.
- Inside step 8,
  or after the head destroy and before its outcome write:
  `head=requested`,
  and the head may be gone.
  The `unmount` intent went with the head,
  so a later run takes the head-gone path of section 4.
  The origin record stands and names the origin.
- Between step 8's outcome and step 9:
  `head=gone` and a registration.
  The finish pass or the next removal settles the registration.
- Between step 9 and step 10:
  no registration,
  `head=gone`,
  the origin record stands,
  and a plan.
  The finish pass removes the plan on its next visit (section 5);
  `doctor` names it until then.
  No window here has neither the registration nor a carrier of the origin:
  the origin record was written at step 6 and stays until its own proof of absence.

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
  not a wait for the disk,
  unless the project chooses another flush rule
  (Choice 5).
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

The record is published without a flush,
following the wrapper's rule for its own records (`SPEC.md:4400`,
F13).
The cost is argued on ZFS rather than on the btrfs reason that rule gives
(`SPEC.md:4421 to 4423`),
and it has two halves:

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

What a crash can cost is the evidence of what was destroyed:
a later removal then refuses a bookmark whose snapshot is gone,
because no recorded tuple explains it,
and a person destroys that bookmark by hand
(`zfs destroy <clone>#<bookmark>`,
the proposal's own sentence at lines 1632 to 1634).
That cost,
against a flush rule,
is Choice 5 for the project.

## 4. Progress, unknown outcomes and objects that are partly gone

### The states

A tuple's outcome moves through these states and no others:

```text
planned   -> requested -> destroyed
                       -> refused  -> requested
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
  and `unknown` to `destroyed`:
  the absence rule proves the object gone.
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
the per-name list names `@sanoid-hourly` with `EEXIST`,
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
  not only its name:
  a child that carries the recorded GUID is the clone under another name,
  and the removal refuses.
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
5. Check the count against the ceiling,
   run the deferred-origin check,
   and submit the remaining set,
   as one request or as the largest fitting prefix of the remaining recorded tuples,
   sorted by name
   (proposal lines 915 to 917).
   Write `requested`,
   submit,
   write the outcomes.
6. Repeat from step 3 until the snapshot set is settled.
7. Before the head:
   re-enumerate.
   The head is refused while own snapshots remain (F4),
   and a new own snapshot is named and added only under the
   amended-plan rule.
   Write `head=requested`,
   destroy,
   write `gone`.

**The head-gone path.**

1. Read the plan.
   No lineage proof runs,
   because there is no clone to prove,
   and no `unmount` intent is read,
   because the intent is a property of the clone and went with the head (F8).
2. Prove the head's GUID absent from the pinned parent's children (the absence rule's second arm).
3. Settle every remaining snapshot tuple and the head as `destroyed` by F4's kernel rule.
4. If the registration still stands,
   settle it and the underlay (proposal step 5);
   the origin cleanup record already stands (A7).
5. Remove the plan (section 7).

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
  and a record whose header line is not `version 1`.
  Each refusal names the path,
  the field or line,
  and what was read,
  and says that nothing was destroyed by this refusal.
  A plan that does not read is never moved aside or deleted (Choice 10).
- An oversized record:
  a file larger than `10,128 + 340 * declared count`,
  or a declared count above the shipped ceiling.
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
  and that it deleted nothing;
  a foreign or oversized entry is printed as not this removal's record.
- The finish pass refuses that one plan and touches nothing,
  then names the plan it could not use;
  every other plan is finished as before.
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

It deletes no snapshot and no plan,
it never rewrites an outcome,
and a record it cannot read is printed as unreadable with the path,
followed by the manual route (`zfs destroy <clone>#<bookmark>` for a refused bookmark,
`doctor` for the state).
What "guarded" means for the command is the sibling design's answer;
this design only says what its reader does with a record.

### The finish pass that settles a registration

The wrapper already finishes a removal a kill cut short before every forwarded command
(`finish_interrupted_destroys`,
`worktree_removal_finishes_an_interrupted_destroy.rs`),
and that pass is this design's driver,
in the same place and with a different folder:

1. It stats the plan folder.
   A folder that is not there ends its part of the command with that one `stat`.
2. A folder that is there is listed once,
   and each plan is read.
3. A plan whose `admin` and `targetHex` match a live registration,
   and whose `head` is `gone`,
   settles that
   registration and its underlay,
   and then removes the plan.
4. A plan whose `head` is `gone` and whose registration is already gone is removed.
5. A plan whose `head` is present or `requested` is **not** acted on:
   no forwarded command destroys a snapshot.
   It is left for the removal command and named by `doctor`.
6. It writes nothing into any plan:
   the outcomes were the removal's,
   and a settle is not an outcome.

That is the driver the earlier revision lacked:
no pointer file,
no listing of `worktrees/` per command,
and no pass that reads a plan without a reason to.
The one cost is the `stat` per command while the folder exists,
which is one `stat` in the healthy state and a listing only while a leftover stands.
Choice 9 weighs the pointer alternative.

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
so the store's rule would break a running removal (Choice 12).

`doctor` never creates the folder,
never removes a plan,
and never rewrites one.

### Readers that do not exist

- The mount sweep (plan step 8) reads no plan.
  A plan is not an instruction to mount or unmount,
  and the sweep's trigger is the intent property,
  not a record in the common directory.
- The add (plan step 10) reads no plan.
  A new clone is a new GUID,
  so a plan of an earlier clone at the same path cannot be its plan.
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
  and two recordings can disagree with nothing to decide between them.
- the origin cleanup record that carries the origin after the clone's destruction,
  whose home,
  fields and end are the sibling design's.
  This design reads it and refuses before its first delete when it does not stand (A7).
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

The removal that owns the plan removes it,
as its last step,
under one condition:

1. the head is proven gone by the absence rule;
2. the registration and the underlay are settled (proposal step 5);
3. the plan is unlinked,
   and the folder is removed when its last record goes,
   as `forget_destroy` does (F13).

The origin cleanup record is not part of the condition:
it already stood before the first delete (A7),
so nothing the plan carries is the only copy of the origin's identity once the head is gone.
That is the one end condition the earlier revision lacked.

A plan left with `head=gone` and no registration is removed by the finish pass on its next visit,
because the pass starts from the folder's `stat` and acts on exactly that state (section 5).
`doctor` names it until then.
No command needs the registration,
a second record,
or a person for that case.

### What a leftover plan means

- A removal was killed or refused.
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
- Nothing tells a person to delete a plan except the removal's own next run or a person's decision after `doctor`
  names it.
  A plan is never removed because it is old,
  because age proves nothing about a clone.

### No expiry

This design sets no age,
no count of runs and no retention window for a plan.
The alternative,
expiring a record after a period,
would erase the only evidence of a destroyed snapshot and turn a resume into a fresh,
unexplained set;
that is Choice 8.

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
  forced by a ceiling one tuple below the count;
- the two stages of the reader:
  a file that grows between stage one and stage two is refused,
  and a file whose declared count needs more bytes than
  it holds is refused.

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

- a plan with `head=gone` and a live registration,
  settled exactly once,
  then removed;
- a plan with `head=gone` and no registration,
  removed by the pass;
- a plan with `head=present` or `requested`,
  left untouched by the pass and named by `doctor`;
- an empty folder case:
  no folder,
  one `stat`,
  no listing (asserted by a control folder that counts opens);
- a control that writes nothing inside a registration and asserts no file of this design ever appears under
  `<common>/worktrees/` (F7 is why).

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
- The head-refused arm with the clone's own snapshots already gone,
  asserting `head=refused` and that the record names the holder or the new snapshot.
- The absence rule's two arms:
  a live snapshot whose name matches and whose GUID differs (destroyed by the GUID,
  not the name);
  a head destroyed and another dataset created under the recorded name,
  and the head-gone path refusing to settle
  while a child carries the recorded GUID,
  then settling once it is gone.
- The bookmark arm of the decided rule over a plan tuple,
  and the arm whose bookmark is refused because its snapshot went before the plan was made.
- The deferred-origin arm of plan step 15,
  with the mark set before the first delete and between the two checks.
- The delegated-account arm,
  destroying the set under the proposed descendant grants.
- The finish pass after a real head destroy,
  asserting the registration settles from the plan and the plan is then
  removed.

Which of these need a guest,
and why:
every arm above reads a live ZFS identity,
sends an ioctl,
or proves absence with a GUID,
so it needs a real pool;
the format,
lifecycle,
folder,
finish-pass and refusal arms need none.

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
  the
  ceiling check,
  the refusal of a record that does not read or does not match,
  and the bookmark rule's read.
- Step 15 builds the deferred-origin check's read of the plan's clone and origin identities,
  and its refusal when the record is missing or unreadable.
- Step 16 builds the progress writes,
  the batch's read,
  the head's states,
  the absence rule's two arms,
  the
  head-gone path,
  the re-plan of new own snapshots,
  the finish pass,
  and the plan's removal after the settle.
- Step 17 builds the cancel command's read and its report.
- Step 18 builds `doctor`'s listing,
  its reports of foreign,
  oversized,
  wrongly owned,
  unreadable entries,
  its
  staging-name rule conditioned on the plan's owner,
  and the placeholder ceiling's report.
- Step 22 replaces the placeholder ceiling with M1's measured ceiling and records it beside the value.
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
and the rename machinery that moves `monochromatic-git-policy-cli/` moves it too.

Pros:
the writer is already the account;
the folder outlives the clone and its registration;
the wrapper's record primitives and `doctor` work unchanged;
a person who finds one wrapper record finds this one.

Cons:
the account can rewrite the only evidence of what it destroyed,
which is the accepted same-account residual;
the record is inside a Git directory,
so a person who deletes the repository deletes it.

Option B,
the binding design's new top-level folder,
`monochromatic-git-policy-cli-worktree-clone/v1/<slot>.plan` (at `1e9bc35f1e`).

Example:
the same removal records at
`/home/u/repo/.git/monochromatic-git-policy-cli-worktree-clone/v1/fix-flaky-test.plan`.

Pros:
the route's three records would live in one folder,
as the binding design proposes;
one header and one key convention for the whole route.

Cons:
the wrapper's rename machinery has no entry for that new top-level name (
`foundation/kept_names_in_the_git_directories.rs:45 to 48`),
while it already moves `monochromatic-git-policy-cli/`;
the plan's key would have to be the slot,
which Choice 2 ranks below the clone's GUID;
the binding design's own key rule is one document's proposal and not a project decision.

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
A beats B because the folder the wrapper's rename machinery already moves and `doctor` already looks at is one
record's home,
and B would add a top-level name that machinery must learn.
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
the ceiling is known before the first delete;
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
the ceiling cannot be checked before the first delete,
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
and the ceiling must count attempts rather than objects.

Ranking:
A > B.
A beats B because the only question a later run asks is what the pool says now,
and B's history costs a merge rule and a bound that no longer tracks the objects.

### Choice 5: the flush rule

Option A,
no flush for any write,
adopted.

Example:
a snapshot destroy returns,
the outcome write is staged and renamed,
and the machine loses power before the
page cache reaches the disk;
the next boot's plan says `requested`,
and the next removal proves the snapshot gone before it does anything.

Pros:
no wait on the removal's path;
F5's ordering says a write made before the destroy is on disk when the destroy returns (A8);
and a write made after it,
if lost,
reads as `requested` and refuses.

Cons:
a power cut can cost the evidence of what was destroyed,
so a bookmark whose snapshot went may make a later removal refuse until a person destroys the bookmark.

Option B,
flush only the writes made before a destroy (`planned`,
`requested`,
`head=requested`).

Example:
the same destroy,
the `requested` write flushing the file and its directory;
the power cut still leaves `requested`,
because the outcome write was the one lost.

Pros:
the write-ahead record is durable even if F5's premise is wrong.

Cons:
under F5 the flush is redundant,
and it pays a commit before every request.

Option C,
flush every write,
including outcomes.

Example:
the outcome write flushes,
so the next boot's plan says `destroyed`.

Pros:
the outcome survives a crash once written.

Cons:
every outcome write pays a commit,
and the crash window between the call and the write is not closed,
so an unknown outcome still exists.

Ranking:
A > B > C.
A beats B because F5's ordering already carries the write-ahead record,
one pool assumed,
so B pays for a case the
source says cannot happen.
B beats C because C pays on every outcome and still leaves the window before the write.
The review's merged question four ranks a set-wide rule C > A > B for the three records together,
because the binding's write after its object exists is the one a flush buys something for;
if the owner answers set-wide,
that answer overrides this ranking for the plan too.

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
ceiling;
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

### Choice 10: what a reader does with a plan it cannot read

Option A,
refuse,
change nothing,
and name the file,
adopted.

Example:
a power cut leaves `7828944690444299989` cut short;
the removal prints the path and the line it could not read,
and says nothing was destroyed.

Pros:
the only evidence of a destroyed snapshot and of an owed origin stays.

Cons:
the removal waits for a person.

Option B,
move it into an `unreadable/` folder and go on,
as the copy records do (`SPEC.md:4863`).

Example:
the same file is renamed into `clone-destruction-plans/unreadable/`,
and the removal proceeds.

Pros:
the bytes are kept and the removal can proceed.

Cons:
the bookmark rule loses the tuples the record held,
and a pinned origin has no record naming it.

Option C,
say it once and drop it,
as the existing finish pass does
(`worktree_removal_finishes_an_interrupted_destroy.rs:241`).

Example:
the same file is named on one command and removed.

Pros:
nothing waits.

Cons:
the evidence is gone.

Ranking:
A > B > C.
A beats B because this record's one job is to be evidence and B's convenience spends it.
B beats C because B keeps the bytes for a person.
The review's merged question five asks the same question across the three records.

### Choice 11: one record with a ceiling, or a folder of chunks

Option A,
one record with the labelled placeholder ceiling and a two-stage reader,
adopted.

Example:
a clone with 40,000 snapshots would need a 13.6 MB record;
the removal refuses before the first delete,
prints `40,000` and the ceiling,
and the clone and its history stay.

Pros:
one read gives the whole plan,
so the bookmark rule needs one record and nothing must be reconciled;
the declared count makes a torn record detectable;
the refusal destroys nothing.

Cons:
a set larger than the ceiling cannot be removed until the ceiling is measured;
the ceiling is a placeholder until M1.

Option B,
one record per chunk plus a small index record.

Example:
the same clone records ten chunk files of 4,000 tuples and an index naming them;
the bookmark rule reads the index and then the chunks it needs.

Pros:
no ceiling on a clone's set;
each file stays small.

Cons:
the union of the files is the plan,
so a kill between two chunk writes leaves a union that is not a plan and every
reader must detect that;
the index and the chunks can disagree;
a person's repair is several files.

Ranking:
A > B.
A beats B because the common case is one batch and a handful of tuples,
and the state B fixes is a removal A refuses without destroying anything:
a refusal a person can read is cheaper than a set of files every reader must reconcile.

### Choice 12: who removes a killed writer's staging name

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

## Questions only the project can answer

One question remains after this round.
The earlier revision's question about mounting a clone with a partial-destruction plan is dropped:
the mount sweep reads no plan (section 5),
so its options needed the root handler to read an account-owned plan,
which the decision record's
"not Git or automatic deletion" (lines 31 to 33) and the proposal's handler sentence (lines 483 to 485) exclude;
what a clone in that state mounts into belongs to the cancelled,
timed-out or killed removal design and the reattach
it decides.

### What trace of a destruction outlives the removal?

The scene:
a removal printed `@earliest` and `@origin` as lost and finished;
a week later the person asks what went.

Option A,
nothing beyond the removal's printed report and the records while they stand,
ranked first.

Example:
the person who asks a week later finds no record.

Pros:
no new store;
the report is where the person looked;
`doctor` names a plan while any work is owed.

Cons:
a person who missed the line has no answer.

Option B,
one line in the wrapper's audit log per removal and per origin cleanup,
beside the discarded-side audit
(`foundation/kept_names_in_the_git_directories.rs:45`).

Example:
the log gains
`2026-11-22T18:52:03-05:00 removed fix-flaky-test; destroyed 2 snapshots; origin snapshot destroyed`.

Pros:
bounded to one line per event;
readable later;
an existing family.

Cons:
a new retention question for a log that grows with every removal.

Option C,
a notice the next commands print.

Example:
the next `git status` prints one line about the destroyed clone.

Pros:
the loss reaches a person who did not read the removal's output.

Cons:
a second record of the same fact,
printed on unrelated commands,
about work the wrapper destroyed and not kept.

Option D,
an append-only history `doctor` reports.

Example:
`doctor` prints every clone destroyed in the repository's life.

Pros:
always answerable.

Cons:
unbounded,
with no reader that acts on it.

Ranking:
A > B > C > D.
A beats B because the project asked for no new store in this record's reasoning and the report already prints the loss.
B beats C because an audit line is bounded and silent until read,
while a notice repeats on commands that had nothing
to do with it.
C beats D because a notice can be bounded and a history cannot.

## Where this design and the sibling records differ

The review names fourteen points on which the three record designs disagreed.
Each is answered here with what this design does now and which record,
if any,
decides it.
The siblings are read at their branch tips,
the origin cleanup record at `4daa6c66ce`
and the binding record at `1e9bc35f1e`,
and both are unreviewed:
nothing below adopts a sibling's answer as decided,
and neither document was edited.

1.  **Folder family.**
    This design keeps its folder inside `monochromatic-git-policy-cli/`,
    as the origin cleanup record does;
    the binding record proposes a new top-level `monochromatic-git-policy-cli-worktree-clone/v1/`.
    No project record decides.
    This design's choice is Choice 1,
    with the binding record's choice beside it;
    it does not agree with the binding record,
    and the wrapper's rename machinery is the reason.
2.  **Key.**
    This design keys by the clone's dataset GUID,
    the binding record by the slot and the origin cleanup record by the
    origin snapshot's name.
    No project record decides.
    This design's choice is Choice 2,
    with both siblings' choices beside it.
3.  **Encoding of names and the shape of the file.**
    This design now agrees with both siblings:
    a `#` version line first,
    `key=value` lines in one fixed order,
    ZFS names verbatim (F2 shows a name holds no
    newline and no `=`),
    paths in hex,
    GUIDs and groups in canonical decimal.
    It changed from hex names this round,
    for the reason the review gives:
    the `key=value` grammar makes verbatim safe,
    and hex was only longer.
    The tuple lines stay positional with the name last after `name=`,
    because a tuple is a record with fields of its
    own.
    No project record decides the file shape;
the nearest precedent,
    the slot store's `#` line,
    is followed.
4.  **The moment of the origin carrier (X1).**
    This design reads the origin cleanup record and never writes it;
    its step 6 requires the record to stand before the first delete (A7).
    That is the origin cleanup design's V4 option A,
    its adopted option at `4daa6c66ce`,
    with the fix its round is told to make:
    written after this removal's pre-delete refusals.
    No project record fixes the moment,
    so this design's order depends on that choice;
    if the project vetoes it,
    this order and the origin design's reopen together.
    The binding record's step 3 to step 5 order publishes the record after the settle,
    which this design does not
    follow for that reason.
5.  **Whether a blocked origin keeps the slot (X2).**
    Not this record's.
    The binding record keeps its binding while the origin cleanup is blocked;
    the origin cleanup record lets a new add take the slot at once.
    The proposal's "Settling frees the slot but requires recovery without its Git entry" (lines 1462 to 1463) and
    "settling does not wait for the origin" (lines 469 to 470) decide for the origin record.
    This design's end does not depend on the answer,
    because the plan is removed once the head is proven gone and the
    registration is settled,
    whatever the origin's state.
6.  **The serializer of two removals.**
    This design takes the binding record's per-slot claim (A2,
    F15),
    and now agrees with the binding record.
    The origin cleanup record names "the removal route's per-slot claim" without naming its owner;
    it is the same object.
    The proposal's per-slot-claim sentence (lines 909 to 910) and the claim file decide.
7.  **Durability.**
    This design publishes without a flush and argues it on ZFS (F5,
    A8);
    the binding record flushes every publication on its own ZFS argument;
    the origin cleanup record decides neither.
    Choice 5 states this design's options and ranking,
    and the review's merged question four is the same choice for the
    three records together;
    a set-wide answer overrides it.
8.  **What a reader does with a bad entry.**
    Partial,
    foreign and oversized:
    all three refuse and change nothing,
    and this design now agrees with both.
    Wrongly owned:
    this design and the binding record refuse;
    the origin cleanup record has no rule,
    which is its own
    finding (O5) and not this document's to fix.
    Unreadable:
    this design refuses and never moves the record aside (Choice 10),
    the binding record refuses and asks its own Q1,
    and the SPEC's copy records move theirs aside.
    No project record decides;
    the review's merged question five is this design's Choice 10.
9.  **Who removes staging names.**
    This design removes one only when a plan with that GUID stands and its `owner` is gone (Choice 12);
    the binding record narrows by "no live claim",
    and the origin cleanup record copies the store's unconditional rule.
    The store's decision covers the store only.
    The reason applies the same way to the other two records,
    and each round owns its own rule;
    this design does not edit theirs.
10. **Listing on the command path.**
    This design stats its folder before every forwarded command and lists it only when it exists (Choice 9);
    the origin cleanup record lists its folder on every command;
    the binding record lists only in `doctor`.
    The store's "no listing per sweep" is about a sweep and a folder that always exists,
    so it does not decide.
    This design's Choice 9 names the pointer alternative it dropped.
11. **Who `doctor` runs as.**
    This design assumes the account (A6);
    the origin cleanup record says the account,
    and the binding record says root may read and no root mode does.
    This design agrees with both,
    and the assumption replaces the earlier revision's unsupported "runs as root".
12. **What ends each record and who removes it.**
    This design removes the plan once the head is proven gone and the registration is settled (section 7),
    never waiting for the origin,
    because the origin record already stands (A7).
    The origin cleanup record is removed by its own absence proof.
    The binding record keeps its binding while the origin cleanup is blocked (X2,
    point 5).
    The proposal's steps 4 and 5 (lines 947 to 956) decide the order only partly;
    this design's condition is one the proposal's "never from a missing name alone" (line 964) supports.
13. **Whether a removal has a record of its own.**
    This design says the `unmount` property is the intent and the plan is the record (Choice 6);
    the binding record says what identifies the removal's state is the plan's presence and the live facts,
    which agrees
    with this design;
    the proposal's "removal record" words (lines 884 to 885 and 940) do not name a second file.
    No project record decides,
    and Choice 6 lists the option of a record beside the registration.
14. **Bounds and caps.**
    This design's bound is the exact sum of named field bounds with no rounding (F11),
    and its only cap is the labelled
    placeholder that M1 replaces (Choice 11).
    The binding record derives a 34,502-byte fixed part and ships no inventory cap;
    the origin cleanup record uses a fixed 16,384.
    The accepted store computed its bound from named terms without rounding
    (`doc/decision/cli-git-zfs-slot-identity-store-format.md` lines 140 to 141),
    which this design follows.
    No project record decides a cap on a set the kernel does not bound;
    M1 is the measurement that would.

## Dependencies on sibling designs

- The add's durable binding record (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
  read at `1e9bc35f1e`).
  This design reads it by the slot's name and compares its repository,
  registration and clone fields (A1),
  and takes its
  per-slot claim (A2).
  If that design changes its key or its claim,
  this reader changes with it.
  Its document is unreviewed,
  so this is a dependency and not an adopted answer.
- The record that carries origin cleanup (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
  read at `4daa6c66ce`).
  This design requires that record to stand before its first delete and refuses otherwise (A7,
  step 6).
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
  the tuple ceiling.
  In the guest,
  after the snapshot arms of items 13,
  14,
  17 and 18,
  count the clone's own snapshots and the origin-side
  inventory at their widest
  (`zfs list -H -p -t snapshot -o name -r <clone>` and the same under the origin),
  and record the largest set one removal submits.
  The ceiling is derived from that count with headroom,
  and plan step 22 replaces the placeholder with it.
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
  a head destroyed,
  another dataset created under the recorded name,
  and the head-gone path refusing while a child carries the recorded GUID and settling once it is gone.
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
  only if Choice 5 becomes the flushed form or the owner chooses a set-wide
  flush:
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
  Choice 5's ranking is reopened.

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
- The tuple ceiling is a placeholder until M1,
  and its refusal is a state a person can meet before that measurement
  lands.

## What this design does not cover

- The binding record's home,
  key and format (the binding design's),
  and the origin cleanup record's (its own).
- The deferred-destroy checks' own design,
  and what each of their refusals prints.
- The cancel command's proof of the slot it may reverse,
  and what a cancelled removal restores.
- What a clone with a partial-destruction plan mounts into:
  that is the cancelled or killed removal design's question.
- The forced arm,
  the holder identification,
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
- [The wrapper specification](../../package/git-policy/cli/SPEC.md)
- The sibling record designs,
  read at the commits named in "Where this design and the sibling records differ":
  the origin cleanup record and the add's binding record.
- The gate review this round answers,
  at
  `~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/review.md`.
- OpenZFS at tag `zfs-2.4.4` (commit `71a9f95`) and Git at `v2.56.1`,
  read in shallow clones this round kept in its scratch folder.

## This round

What ran:

- the OpenZFS clone and the Git clone at their tags,
  and the reads cited as F1 to F9 and F14 to F16;
- the registration-inside-file probe cited as F7 and F10,
  its log at `keep/registration-inside-file-probe.json`;
- the revised byte arithmetic cited as F11,
  its log at `keep/plan-byte-arithmetic-revised.json`;
- the citation check over every path-and-line citation in this document,
  whose output is
  `scratch/citation-check-before.txt`.

What did not run:

- no ZFS command,
  no ioctl,
  no guest,
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
  A later commit that only adds or corrects this list is not listed.
