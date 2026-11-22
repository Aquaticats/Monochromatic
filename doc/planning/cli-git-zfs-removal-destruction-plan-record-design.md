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
- the proposal's "Removal and recovery" (lines 796 to 976),
  "Clone removal identity protection" (lines 1513 to 1603),
  "Bookmarks on a clone being removed" (lines 1604 to 1672),
  its "Open items the decisions leave" (lines 1673 to 1787),
  and its project-decision entry for the bookmark rule (lines 1339 to 1342);
- the implementation plan's steps 14,
  15,
  16,
  17,
  18 and 22,
  its "Open items and the step that needs each" (lines 2770 to 2856),
  its "Gaps the design's open-item list does not name" (lines 2858 to 2895),
  and "Where the design is silent:
  this plan's suggestions" (lines 2897 to 3010);
- `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md` items 13 and 14;
- `doc/decision/cli-git-zfs-slot-identity-store-format.md` whole,
  `doc/decision/cli-git-zfs-staging-under-cache.md` (the threat model,
  lines 210 to 240),
  `doc/decision/cli-git-zfs-step-8-sweep-choices.md` (lines 15 to 70),
  and the accepted staging design's opening and its record/place sections;
- `package/git-policy/cli/SPEC.md`:
  "The records an add writes for itself" (line 4398),
  "A record that cannot be read" (line 4831),
  "The descriptor mount of a ZFS worktree clone" (line 5408),
  "The clone's mount intent,
  receipts and readiness" (line 6072),
  "Records of a worktree that is removed" (line 16471),
  and "Doctor" (line 683);
- the code the wrapper already keeps its own records with,
  named where each is used in "What the repository already has";
- OpenZFS `zfs-2.4.4`,
  cloned shallow at tag `zfs-2.4.4` (commit `71a9f95`),
  for the name bounds,
  the property bound and the destroy request's error path.

What it ran:

- the OpenZFS clone and the reads inside it,
  under `scratch/openzfs`,
  a read-only source checkout with its push URL set to `DISABLED`;
- a disposable Git repository probe of the registration names Git gives a linked worktree,
  and of what happens when a worktree is added again at the same path,
  as `git version 2.56.1` with the wrapper shadowing it;
- an arithmetic probe of the widest bytes each line of the record can take,
  from the grammar in section 2 and the bounds in F1 and F2.

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
Both probes are cited as F5 and F6 and again where their results are used.

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

What the plan must hold,
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
  `doc/decision/cli-git-zfs-worktree-design.md` lines 46 to 51:
  "A guarded `cancel-worktree-removal <slot>` stays as the manual way back,
  including for a removal whose wrapper was killed after it recorded the unmount or that ran past its deadline.
  It reports any snapshots already destroyed and cannot restore them."
  The plan is the only place that report can come from.
- The deferred-destroy check before the first delete,
  the same record lines 52 to 56:
  "The check that the origin is not marked for deferred destroy also runs before the first snapshot delete,
  so such a removal refuses before this run deletes anything."
  That check compares planned identities with live ones,
  and this design says where a reader finds them.
- No configuration,
  `doc/decision/cli-git-zfs-step-8-sweep-choices.md` lines 17 to 19 and 28:
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
  The record's byte bound is derived from field bounds that are facts of OpenZFS and the kernel's own path limit,
  and the one number that is a workload choice is named as a measurement owed,
  never picked here.
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
  16,384 bytes for "four short lines,
  the hexadecimal form of the place a renamed subvolume stands (twice the
  length of a path),
  and the worktree's path"),
  and dropped when the removal ends (`forget_destroy`,
  line 294,
  which also removes the folder when its last record goes).
  It names its writer by process id and birth identity so a later command tells a running removal from one a kill
  abandoned (`record_destroy_with_owner`,
  line 134).
- `worktree_remove_hand_over.rs:805` names `monochromatic-git-policy-cli/removals-begun`,
  one record per registration,
  and line 103 names the identity record the wrapper writes beside the registration
  (`monochromatic-git-policy-cli-removal-identity`),
  written by `write_beside_the_registration` (line 635),
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
  (`private_storage.rs:132 to 146`),
  and `private_storage.rs:83` and `:87` are the folder mode `0700` and the file mode `0600` every such record takes.
  Its own doc says what a lost flush costs (line 82):
  "A record whose meaning a machine crash must keep stays on `replace_private_file`."
- The receipt family in `SPEC.md` line 6072 gives the reading rules a record here needs:
  a name that is "only the clone's canonical numeric dataset GUID,
  with no sign and no leading zero" (lines 6138
  to 6140),
  a byte bound "derived from the receipt's own field bounds" (line 6147),
  a writer that "refuses a receipt its own reader would not read back" (line 6150),
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
- `SPEC.md` line 4400 states the durability rule this design starts from:
  "An add's own bookkeeping is published whole and flushed by nobody",
  with its measured reason (a native `git worktree add` makes no `fsync`;
  the same add through the wrapper made twelve,
  line 4416).
- The mount intent and its receipts fix two more precedents:
  one short token as a dataset property (43 bytes at its longest,
  `SPEC.md` line 6102)
  and a root record under `/run` keyed by the clone's GUID.
  The claim's own reason for living in `/run` is that "a claim means nothing across a boot"
  (`SPEC.md` lines 6263 to 6264).
- `doctor` is where a leftover record is reported,
  and the project already put the removal of a killed writer's staging file in plan step 18's `doctor`
  rather than in a waiting path
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  "Who removes a killed writer's staging file").

This design reuses each of those,
and says so where it does.
It invents one primitive:
nothing.

## Facts read and measured

Numbers here are cited by number later,
so each can be checked alone.
"Read" means a source at a named tag or a repository file;
"measured" means a run whose log is in the keep folder.

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
  A name may hold a space and a `%`,
  besides letters,
  digits,
  `-`,
  `_`,
  `.` and `:`
  (`module/zcommon/zfs_namecheck.c:59 to 65`).
  A space-separated or percent-interpreting line grammar therefore cannot carry a name verbatim.
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
  (`module/zfs/dsl_destroy.c:786 to 789`).
  Read at `zfs-2.4.4`.
- F5.
  A linked worktree's registration name is a plain name derived from the path's last component,
  sanitized (a space became `-`),
  and the same name is used again when a worktree is added again at the same path.
  Measured on this host with `git version 2.56.1` and wrapper `0.6.1-alpha.44`:
  `first` and `second` were both `["probe-slot"]`,
  and `reusedOnReAdd` was `true`;
  the odd names were `["a-slot-with-a-space"]` and `["a.slot.with.dots"]`.
  The probe's log is `keep/worktree-registration-id-probe.json`.
- F6.
  The record's widest bytes,
  from the grammar in section 2:
  a tuple line is at most 593 bytes,
  the header at most 10,812 bytes,
  a hundred tuples fit in about 76,384 bytes and a thousand in about 616,384.
  Measured by `scratch/plan-byte-arithmetic.ts`,
  logged in `keep/plan-byte-arithmetic.json`.
- F7.
  The wrapper's own records are folder `0700`,
  file `0600`,
  staged with `O_EXCL` and published by one rename,
  and read with a fixed byte limit per record
  (`private_storage.rs:83` and `:87`;
  `worktree_removal_interrupted_destroy.rs:65`;
  `SPEC.md:4408`).
- F8.
  The removal's unmount intent is one short dataset property,
  at most 43 bytes
  (`SPEC.md:6102`),
  and the property carries no path and no dataset name (`SPEC.md:6104`).
- F9.
  A receipt is named by the clone's canonical numeric dataset GUID,
  with no sign and no leading zero
  (`SPEC.md:6138 to 6140`),
  and a value is local only when its source is the clone's own dataset (`SPEC.md:6112 to 6114`).
- F10.
  The wrapper's own numbers for a slot:
  the canonical slot path's bound is 4,095 bytes (`SPEC.md:6156`),
  and an object identity is `device:inode:seconds:nanoseconds`
  (`package/git-policy/cli/src/native/subvolume/fs_beneath.rs`,
  `ObjectId::encode`).
- F11.
  A registration's identity record beside it is a device and inode pair per directory
  (`worktree_remove_hand_over.rs:103` and `:584 to 592`),
  and a worktree added again at one path is another directory with another inode
  (`worktree_remove_hand_over.rs:566 to 592`).

## Assumptions

- A1.
  The add's durable binding record is reachable from the registration identity and the clone GUID.
  Its home is being designed on a sibling branch
  (`doc/planning/cli-git-zfs-add-binding-record-design.md`),
  and this design does not decide it.
  If that record cannot be found from those two facts,
  a reader here refuses,
  which is the same outcome as finding it unreadable.
- A2.
  The wrapper's managed Git lock is held by a removal for the registration it removes,
  as the sibling design of `doc/planning/cli-git-zfs-add-time-git-lock-design.md` decides it.
  A removal that cannot take it does not write this record.
- A3.
  Two cooperating removals of one slot never run at once,
  because A2's lock serializes them.
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
  No ZFS or kernel behaviour beyond the measurements record and the source reads in F1 to F4 is claimed.
  The record's publication relies on the repository's own implementation,
  not on a fresh read of the kernel at a tag (L2).

## Limits of this round

- L1.
  Nothing about a pool,
  a destroy,
  an unmount or an absence check is measured here.
  Every refusal arm below that needs ZFS is unmeasured and is listed under "Measurements owed".
- L2.
  The one-rename publication and the `O_EXCL` staging file are the repository's own primitives,
  read here as code and as its own record of them (F7).
  They were not re-read in a kernel at a tag this round,
  so the design claims what the repository claims and no more.
- L3.
  The trap F5 measures was run in a disposable repository with plain `git`;
  it says nothing about a ZFS clone,
  which has no registration until plan step 10 makes one.
- L4.
  The arithmetic in F6 is over the grammar and the bounds,
  not over real records:
  no plan has ever been written.
- L5.
  The byte ceiling in section 2 is a workload choice and is owed to a measurement,
  so this design names no number for it.

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
  which one clone with more than about a dozen tuples exceeds at 600 bytes per tuple (F6);
- the property belongs to the clone,
  so the head destroy takes it at the moment its own outcome must be written;
- the origin is a different dataset shared with other clones,
  so a property there would be one record for many removals.
  The mount intent shows the shape a property serves:
  one short token,
  at most 43 bytes (F8).

The plan is not root state under `/var/lib` beside the slot identity store:

- the writer is the account,
  under delegated destroy (the decision,
  lines 34 to 35),
  and every outcome write would need a privileged channel to reach root state;
- the project already moved one record of this route out of root state for exactly that reason
  (`doc/decision/cli-git-zfs-staging-under-cache.md`,
  the answer "In ~/.cache" and its threat model at lines 221 to 237),
  and the account-owned Git common directory is where the wrapper's other removal records already live.

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
with root state and a clone property as the alternatives.

### The folder, its owner and its modes

- The folder is `monochromatic-git-policy-cli/clone-destruction-plans`,
  a sibling of the wrapper's `recorded-destroys`,
  `removals-begun`,
  `removal-snapshots` and `leavings` folders,
  so a person who knows one knows where the others are.
- The `monochromatic-git-policy-cli` parent already exists in most repositories,
  and a repository that has none gets it from the account that writes the first record,
  by `create_private_directory_all`,
  which is what the existing records do
  (`worktree_removal_interrupted_destroy.rs:134 to 143`).
- The folder is owned by the account that owns the common directory and is mode `0700`;
  each record is owned by that account and mode `0600`.
  That is the wrapper's private-record pair (F7),
  and it keeps the clone's snapshot inventory out of other accounts' reach.
  `doctor` runs as root in plan step 18 and reads it regardless of the mode.
- The folder is created by the removal and by nothing else.
  `doctor` never creates it:
  a root-run `doctor` that made the folder would leave it root-owned inside the account's tree,
  and every later account write would fail.
  That refusal is cheap and is the design's own.
- A folder whose owner is not the common directory's owner,
  or whose mode has any of the group or other write bits,
  is refused by every reader,
  and `doctor` reports it.
  Under the project's threat model this catches a person's mistake and a root run that went wrong,
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
which is the rule the slot identity store's design already states for its own folder
(`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
"A record file's bound,
and what replaces the whole-store bound").

### The key, and why it is the clone's GUID

The key is the clone dataset's GUID.
It is the one fact that names this clone and no other for the whole life of the removal:
it survives the unmount,
the snapshot destroys and the head destroy,
and it is readable before any of them
(`zfs get -H -p -o value guid`,
the read [VM item 13](cli-git-zfs-measurements-owed-before-acceptance.md) made on this release).

The other two candidates,
with the measurement that decides against one and the reasoning that decides
against the other:

- The registration's name.
  Git's registration name is a plain,
  sanitized name (F5),
  and the wrapper already keys `recorded-destroys` and `removals-begun` by it.
  But the probe showed the same name is used again when a worktree is added again at one path
  (`reusedOnReAdd: true`,
  F5),
  so a record keyed that way is one name for the clone removed today and the clone added at that path tomorrow.
  The record's own registration identity would have to decide between them,
  which is a check the key could have made unnecessary,
  and one that a fix-up would be tempted to skip after the registration is gone.
- A digest of the slot path.
  Two clones at one path produce one name.
  The digest would also have to be computed from canonical path bytes,
  and a path that no longer exists cannot be canonicalized,
  so the cancel command could not compute the key for the one case it exists for:
  a removal whose wrapper was killed after the head went.

Choice 2 puts the three to the project with examples.

### The pointer beside the registration

The plan lives in the common directory,
which outlives the registration;
the registration does not outlive the plan.
A command that holds the registration and not the clone cannot compute the plan's key once the head is gone,
because the GUID is no longer readable.
So the removal writes one pointer file beside the registration:

```text
<registration>/monochromatic-git-policy-cli-clone-destruction-plan
```

a single line holding the plan's key,
which is the clone's GUID,
written after the plan's first write and read by the finish pass (section 5).
The name follows the wrapper's two existing beside-the-registration records
(F7,
`worktree_remove_hand_over.rs:103`),
and the file goes with the registration when the removal settles it,
so it needs no cleanup of its own.
A pointer naming a plan that is not there is reported by `doctor` and refused by the finish pass.

The order matters:

1. the plan's first write,
2. the pointer,
3. the first snapshot request.

A kill between 1 and 2 leaves a plan that only a run holding the live clone finds by GUID,
which is exactly the run that can still re-derive everything.
A pointer written first would name evidence that does not exist yet.

Choice 8 puts the pointer against a listing by every command.

## 2. The fields

### The grammar

The record is a text file of lines.
The first line is `plan=1`,
the format marker,
and every following line is a `key=value` pair in one fixed order,
with the tuples last,
one per line,
in ascending byte order of their hexadecimal name.
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
for a clone of this repository's own shape,
with the pool and dataset names shortened to fit the page
(the fields carry the real bytes,
up to the bound in "The size bound"):

```text
# <common git dir>/monochromatic-git-policy-cli/clone-destruction-plans/7828944690444299989
plan=1
kind=clone-destruction-plan
run=6f1c9d2a7b4e40a1b8c1d3e5f7091a2b
owner=48213:1729 1122 3 4
started=2026-11-22T17:24:09-05:00
slotNameHex=74616e6b2f77742f6669782d666c616b792d74657374
cloneGuid=7828944690444299989
cloneCreatetxg=17
originDatasetHex=74616e6b2f6d6f6e6f2d636865636b6f7574
originDatasetGuid=11050373805378911967
originSnapshotHex=74616e6b2f6d6f6e6f2d636865636b6f7574406f726967696e
originSnapshotGuid=11477418097495754031
originSnapshotCreatetxg=13
registrationIdHex=6669782d666c616b792d74657374
admin=2049:1234567:1729:1122
targetHex=2f686f6d652f752f7265706f2f2e636c617564652f776f726b74726565732f6669782d666c616b792d74657374
unmountNonce=0a1b2c3d4e5f60718293a4b5c6d7e8f9
unmountMountId=none
head=requested
tuples=2
tuple=v1 snapshot requested 11477418097495754031 13 - 74616e6b2f77742f6669782d666c616b792d74657374406561726c69657374
tuple=v1 snapshot requested 17157080670377245572 18 - 74616e6b2f77742f6669782d666c616b792d7465737440776f726b747265652d6f776e
```

The example's GUIDs and transaction groups are [VM item 13](cli-git-zfs-measurements-owed-before-acceptance.md)'s,
so the example is a shape a real clone and its origin can have,
not an invented one.
`originSnapshotGuid` is the clone's direct origin snapshot,
which is the one step 1 requires the record to name.

### Field by field

- `plan=1` is the format version,
  a literal.
  A reader that does not know the version refuses the record and never guesses.
- `kind=clone-destruction-plan` is the marker that keeps this record apart from every other file in the common
  directory.
  A reader tests this value before it does anything else.
- `run=<32 lowercase hex>` is the removal run's nonce,
  the same shape as the mount intent's nonce (`SPEC.md:6100`),
  so a record is tied to one invocation and to one person's removal.
- `owner=<pid>:<birth identity>` names the process that wrote it,
  as `record_destroy_with_owner` does (F7),
  so a later command tells a live removal from an abandoned one.
- `started=<ISO 8601 with offset, seconds>` is for `doctor` and for a person.
  It is never a proof:
  no reader decides anything by age,
  and this design sets no expiry (section 7).
- `slotNameHex` is the clone's full dataset name,
  hex of its bytes,
  because a name may hold a space and a `%` (F2).
  It is the name the destroy requests use and the name the receipt's slot path was checked against.
- `cloneGuid` and `cloneCreatetxg` are the clone's own identity,
  canonical decimal `uint64`.
  `cloneGuid` is also the record's key,
  and a disagreement between the name and the field refuses the record.
- `originDatasetHex`,
  `originDatasetGuid`,
  `originSnapshotHex`,
  `originSnapshotGuid` and
  `originSnapshotCreatetxg` are the origin the plan was made against,
  in the same spellings.
  These are the identities the deferred-destroy check compares with live,
  and the ones the origin cleanup later uses.
- `registrationIdHex` is the bytes of the registration's directory name under `worktrees`,
  hex encoded for the same reason `slotNameHex` is.
  With `admin` it proves the record was written for the registration standing there,
  and it is what the pointer beside the registration is cross-checked with.
- `admin=<ObjectId>` is the registration directory's identity,
  `device:inode:seconds:nanoseconds` (F10),
  the encoding the wrapper's existing records use.
  A worktree added again at the same path is another inode (F11),
  so a stale record cannot be read as the new one's.
- `targetHex` is the slot path's bytes,
  hex of them,
  as every path in every wrapper record is written (F7,
  `SPEC.md:4408`).
  The bound is the kernel's 4,095 bytes (F10).
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
- A tuple line is `tuple=v1 <kind> <outcome> <guid> <createtxg> <errno> <nameHex>`:
  - `<kind>` is `snapshot`.
    The decided design records no bookmark entry (proposal step 3,
    lines 895 to 896),
    so a reader that meets another kind refuses the record rather than skipping the line.
  - `<outcome>` is one of `requested`,
    `destroyed`,
    `refused`,
    `intact` and `unknown` (section 4).
  - `<guid>` and `<createtxg>` are the tuple's canonical decimal `uint64` values.
  - `<errno>` is the decimal error number the per-name result carried for a `refused` tuple,
    and `-` for every other outcome.
  - `<nameHex>` is the snapshot's full name,
    hex of its bytes (F1,
    F2).

No field carries a bookmark,
a path other than the slot's,
a mount tree,
a person's words or a free-text note.
A record with such a line is foreign and is refused.

### The size bound and its basis

The bound is derived,
not picked:

- a tuple line is at most 593 bytes:
  `tuple=v1` and spaces (15),
  the kind (8),
  the outcome (9),
  two canonical `uint64` values (40),
  the errno (11),
  the hex of a 255-byte name (510),
  and the newline (1),
  per F6;
- the header is at most 10,812 bytes,
  from the same field bounds,
  the widest candidate being `targetHex` at twice the kernel's path limit (8,190 bytes),
  plus the three hex names and the remaining numbers (F6).

So the reader's byte bound is

```text
16,384 + 600 * <the record's declared tuple count>
```

where 16,384 is the wrapper's existing per-record limit for a record holding a hex path
(`worktree_removal_interrupted_destroy.rs:65`),
and 600 is the tuple line's widest bytes rounded up.
A reader refuses a file larger than that for its declared count,
and refuses a declared count above a shipped ceiling.
The writer refuses to publish a record its own reader would refuse,
which is the receipt family's rule (`SPEC.md:6150`).

The ceiling itself is a workload choice and this design names no number for it.
The writer's refusal names the count it would record and the ceiling,
and it happens before the first delete,
because the plan is written before it.
The measurement that sizes it is M1,
and plan step 22 is where a shipped limit is recorded beside its measurement (`LQM`).
The alternative shape,
a plan split across several records,
is Choice 3.

The bound costs what a clone costs,
not what a wait costs:
a hundred tuples are about 76,384 bytes and a thousand about 616,384 (F6),
and the record is written and renamed on the removal's own path,
which already waits for an unmount and a destroy.

### What the plan refuses to hold

- No bookmark entry,
  decided by the proposal (lines 895 to 896).
  A bookmark is explained by a snapshot tuple or it is not,
  and nothing in the plan destroys a bookmark.
- No names the removal has not verified.
  A tuple enters the plan when the removal submits it,
  never when it prints the loss list,
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
   which is the request the handler answers
   (`SPEC.md:6093`,
   "the removal intent is step 14's").
   This route writes no begun record beside the registration (Choice 5).
3. The handler's verified unmount,
   and the removal's check of the receipt's own fields for its nonce.
4. The plan's first write:
   `head=present`,
   `tuples=0`,
   the identities of step 1,
   `unmountNonce` and `unmountMountId` from the verified receipt.
5. The pointer beside the registration.
6. For each snapshot request:
   a rewrite with that request's tuples as `requested`,
   the request,
   a rewrite with the request's per-name outcomes.
7. Before the head destroy:
   a rewrite with `head=requested`,
   the destroy,
   a rewrite with `head=gone`,
   `refused` or `unknown`.
8. The registration and underlay settle (proposal step 5).
9. The origin-cleanup record,
   whose home is a sibling design,
   then the plan's removal (section 7).

The first write is after the unmount and not before it,
which is the proposal's order and also the honest one:
a plan written earlier would name an unmount that never happened,
and a later run would have to tell a real plan from one made for a request that was abandoned.

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
  nothing was destroyed,
  so there is nothing to lose.
- Between step 4 and step 5:
  a plan,
  no pointer.
  A run holding the live clone finds it by GUID,
  and re-proving replaces it.
- Between step 5 and step 6:
  a plan with no tuples,
  and a pointer.
  Nothing was destroyed.
- Inside step 6:
  the record says `requested` for the whole request.
  That is the unknown outcome of section 4,
  and a later run proves each tuple by the live inventory before it does anything.
- Inside step 7:
  `head=requested`,
  and the same proving rule.
- Between step 7's outcome and step 8:
  `head=gone` and a registration.
  The finish pass or the next removal settles the registration.
- Between step 8 and step 9:
  no registration,
  `head=gone`,
  a plan.
  Only `doctor` lists it,
  and the origin cleanup is the work left (section 7).

Every one of those states is one a reader can refuse:
the record never says "gone" before the destroy returned,
and it never says "requested" after an outcome was written,
because a rewrite is whole or not at all.

### One writer, one publication

- The record is published by `replace_private_file_unsynced`,
  so the name holds the old record or the new one and never a torn one (F7).
- The rewrite is whole,
  so a reader needs no lock,
  and rule `NLK` is met without a lock.
  A rewrite costs the record's bytes,
  not a wait for the disk,
  unless the project chooses the flushed form
  (Choice 4).
- Two cooperating writers do not exist (A2,
  A3).
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
F7),
and the cost is exact:

- a machine crash can leave the record at an earlier state than the disk's,
  including no record at all;
- an earlier state is either "nothing requested" or "requested",
  both of which a later run proves again;
- a crash cannot leave a record that claims a destroy which did not happen,
  because the outcome is written after the call and the call's own effects are the kernel's committed ones.

What a crash can cost is the evidence of what was destroyed:
a later removal then refuses a bookmark whose snapshot is gone,
because no recorded tuple explains it,
and a person destroys that bookmark by hand
(`zfs destroy <clone>#<bookmark>`,
the proposal's own sentence at lines 1632 to 1634).
That cost,
against a flush on every outcome write,
is Choice 4 for the project.

## 4. Progress, unknown outcomes and objects that are partly gone

### The states

A tuple's outcome moves forward only:

```text
submitted -> requested -> destroyed
                       -> refused
                       -> intact
                       -> unknown
```

- `requested` means this removal submitted the name in a request whose answer is not recorded yet.
- `destroyed` means the removal proved the object gone (the absence rule below).
- `refused` means the kernel's per-name result carried an error for this name,
  which the `errno` field keeps.
- `intact` means the request returned non-zero and this name was not among the failures,
  so the kernel's own guarantee that a refused request destroys none applies
  (F4,
  `lib/libzfs_core/libzfs_core.c:448 to 456`).
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
`submitted` is the moment before the first write and is never a value the record holds.

An outcome never moves back.
A later run that proves an `unknown` tuple intact rewrites it as `intact`,
not as `requested`:
the proof is newer than the request.

### The absence rule

Adopted whole from the proposal (lines 964 to 969):

- while the head is present,
  the clone must answer with the recorded live GUID and an empty live snapshot list,
  and a snapshot tuple is `destroyed` only when its name is gone from that inventory;
- once the head is gone,
  a snapshot or head tuple is settled `destroyed` only when its name is absent and no child of the pinned parent
  dataset carries its recorded GUID.

A name's absence alone proves nothing (a person can rename an intact dataset,
and a replacement can take a name),
which is why the GUID must also be absent.
Neither observable was measured;
M3 is the probe that would.

### What a later run does

The removal resumes in this order,
and the order is the design:

1. Read the plan,
   or find it by the pointer and the registration.
   A record that does not read,
   or that disagrees with the live clone and registration,
   refuses before anything is deleted.
2. Take the inventory and prove the lineage of proposal step 1 again,
   against the plan's recorded identities and the binding's origin-side inventory.
3. Settle each `requested`,
   `unknown` or `intact` tuple by the absence rule,
   rewriting the plan once with the settled set.
   A tuple that cannot be settled stays `unknown`,
   the run stops,
   and nothing is regenerated from the names that are left (the proposal's own rule,
   line 968).
4. Re-enumerate the remaining set and re-plan only the tuples that are still live and still verified.
   Every earlier tuple and its outcome stays in the record,
   so a bookmark of a destroyed snapshot is still explained (proposal,
   lines 1643 to 1644).
5. Continue at step 6 of section 3's order with the remaining set.
6. Before the head,
   re-enumerate:
   the head is refused while own snapshots remain (`F4`,
   `module/zfs/dsl_destroy.c:786 to 789`),
   and its `refused` outcome names the new snapshot and stops.
7. Settle only when the head's absence and GUID are proven,
   never from a missing name.

Step 3 is why a partly gone set is safe to resume:
the record keeps what was destroyed,
the run proves what the disk says,
and the two are compared before any new request.

### A worked example

A plan for `tank/wt/fix-flaky-test` whose removal was killed inside the snapshot request:

```text
head=present
tuples=2
tuple=v1 snapshot requested 11477418097495754031 13 - <@earliest>
tuple=v1 snapshot requested 17157080670377245572 18 - <@worktree-own>
```

The next removal reads it,
takes the live inventory and finds `@earliest` absent and `@worktree-own` still listed.
Absence alone is not enough,
so it reads the pinned parent `tank/wt` for a child carrying GUID `11477418097495754031`,
finds none,
and settles `@earliest` as `destroyed`;
`@worktree-own` is still live,
so it is `intact`.
The plan is rewritten once with those two outcomes,
the lineage is proved again,
and the remaining set (`@worktree-own`) is submitted in a new request after the deferral check.
Nothing is re-sent for `@earliest`,
which also means the request's own tolerance of missing names (F4) is never needed.

The two other states a person meets:

- planned snapshots all gone with the head refused:
  every tuple is `destroyed`,
  `head=refused` names the holder or the dependent clone,
  and the removal stops with the clone standing and its history gone.
  The cancel command reports it (section 5).
- clone gone with its recorded origin retained:
  `head=gone`,
  the registration and underlay settle,
  and the origin remains for the separate record (section 7).

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

### What every reader refuses

- A record that cannot be read:
  no such file,
  a permission the reader lacks,
  an I/O error,
  a record that stops before its declared tuple count,
  a record whose last line is unfinished,
  a record with a key outside the fixed order,
  a record whose numbers are not canonical,
  and a record whose `plan` version is not `1`.
  Each refusal names the path,
  the field or line,
  and what was read,
  and says that nothing was destroyed by this refusal.
  A plan that does not read is never moved aside or deleted,
  which is where this design differs from the copy records' policy (`SPEC.md:4863`):
  a damaged plan may be the only evidence that a snapshot was destroyed,
  so moving it would destroy the evidence,
  and a person's decision is the only remover.
- An oversized record:
  a file larger than `16,384 + 600 * declared count`,
  or a declared count above the shipped ceiling.
  It is refused,
  never truncated.
- A foreign entry:
  a name that is not a canonical GUID,
  a directory,
  a link,
  a record whose first line is not the marker,
  a record whose name and `cloneGuid` disagree,
  a record with a tuple kind other than `snapshot`.
  No reader reads it,
  and `doctor` reports it.
- A wrongly owned entry:
  a record whose owner is not the folder's owner,
  a folder not owned by the common directory's owner,
  or a folder whose mode is not `0700`.
  Under the project's threat model (lines 221 to 237) this is a mistake-catcher,
  not a boundary:
  the account can write its own records anyway,
  so the check exists for a root run that made the folder,
  a person who copied a record,
  and a restored backup with other ownership.
  It is kept rather than dropped,
  as the staging design kept the checks a hostile account motivated,
  and it is open to the project's veto with Choice 1's entries.
- A staging entry:
  read by nobody,
  removed by its writer,
  and removed by plan step 18's `doctor` when a writer was killed,
  which is where the project already put that removal for the store
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  "Who removes a killed writer's staging file").
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
- The finish pass refuses that one registration and touches nothing,
  then names the pointer and the plan it could not use;
  every other registration is finished as before.
- `doctor` reports each of the five by path and reason and changes nothing,
  except the known staging forms,
  which it removes.

### The bookmark rule

Proposal step 1,
before the first delete.

It reads the plan's tuples as the "recorded plan tuple" of the decided rule
(lines 1339 to 1342),
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
  which stops the attempt (section 4,
  step 3).

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

The command reads the plan and prints,

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
`worktree_removal_finishes_an_interrupted_destroy.rs`).
For this route the pass reads the pointer beside a registration it is already finishing,
opens the plan it names,
and does exactly four things:

1. refuses if the record does not read or disagrees with the registration's identity;
2. settles the registration and the underlay only when `head=gone` and the absence rule proved it;
3. writes nothing to the plan:
   the outcomes were the removal's,
   and a settle is not an outcome;
4. leaves the plan for the origin record and for `doctor`,
   and removes it only when the origin work is done (section 7).

A pass that runs before the pointer exists reads nothing.
A plan whose pointer is missing is not found by this pass,
which is why a kill between the plan's first write and the pointer is found by the next removal instead.
The pass lists no folder,
so a command pays one `stat` of the pointer and one record read at most.

### `doctor`

Plan step 18.

`doctor` lists the folder once per run and reports,
per entry:

- the record's key and its `cloneGuid`,
  with a line when they disagree;
- its state:
  `head`,
  the tuples' outcomes,
  and the tuples that are not proven;
- its owner and the folder's owner and mode,
  with a line when they differ from the common directory's;
- a plan whose pointer is missing,
  and a pointer whose plan is missing;
- a plan whose registration is gone,
  naming the origin work it holds;
- every foreign entry and every dot entry,
  and the known staging forms,
  which it removes,
  with their count and age;
- a record that does not read,
  reported as unreadable with its path and the fact that nothing was destroyed by reading it.

`doctor` never creates the folder,
never removes a plan,
and never rewrites one.
That is the difference between reporting leftovers and destroying evidence,
and it is also why the project's answer for the store's staging files does not stretch to the plans.

### Readers that do not exist

- The mount sweep (plan step 8) reads no plan.
  A plan is not an instruction to mount or unmount,
  and the sweep's trigger is the intent property,
  not a record in the common directory.
- The add (plan step 10) reads no plan.
  A new clone is a new GUID,
  so a plan of an earlier clone at the same path cannot be its plan.
- The commit service's other passes read no plan.
  The ZFS removal writes no `recorded-destroys` entry and no `removals-begun` entry,
  for the reason Choice 5 gives:
  those passes act on a removal shape that has no dataset behind it.

## 6. The binding record and the removal intent

### What is copied, and what is referred to

Copied into the plan,
because the plan must stand alone when the binding record is unreadable or its registration is gone:

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
- the origin-cleanup record that carries what is left after the clone is destroyed,
  whose home is a sibling design.

The plan names no path to either record.
A reader finds the binding by the registration identity and the clone GUID (A1),
and the origin record by whatever its own design makes its key;
where that is not yet decided the plan stands (section 7).

### When they disagree

No record wins.
The removal refuses before the first delete,
names the three readings (the binding's,
the plan's,
the live one),
and changes none of them.
The plan is never rewritten to match live,
which is the proposal's own rule for the recorded lineage (lines 874 to 875,
"Never update
the recorded lineage to match an external change"),
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
for the reason Choice 5 gives,
so a reader never takes one for this removal's intent.

The plan is written after the intent's answer is verified,
and its `unmountNonce` names the request it was verified under.
When the plan's nonce and the live property's nonce differ:

- if the plan has no `unknown` outcome and its identities still match live,
  the run re-proves everything and rewrites the plan with the new nonce,
  keeping every tuple and outcome;
- otherwise it refuses,
  and the cancel command is the way forward.

A plan with no live intent at all is refused by the removal,
because nothing then proves the unmount it claims;
that a person removed the property is not proof of anything.

### The wrapper's other removal records

The plan is this route's counterpart of `recorded-destroys`,
not a second record beside it.
A managed clone removal writes the plan and no `recorded-destroys` entry,
and no `removals-begun` entry,
because those records are read by passes that would treat a clone as an ordinary worktree removal.
The two families cannot be confused even if one is seen by the other's reader:
the folders differ,
and the plan's first line is the marker the other readers do not know.
Choice 5 puts that to the project.

## 7. Its end

### When it is removed

The removal that owns the plan removes it,
as its last step,
in this order:

1. the head is proven gone by the absence rule;
2. the registration and the underlay are settled (proposal step 5);
3. the origin facts are in the origin-cleanup record,
   or the clone had no origin left to clean;
4. the plan is unlinked,
   and the folder is removed when its last record goes,
   as `forget_destroy` does (F7).

The pointer went with the registration at step 2.
A kill between steps 2 and 4 leaves a plan with no registration,
which only `doctor` lists and the origin work still needs:
that is the intended residue,
not a leak.

The finish pass removes a plan in exactly the same conditions (section 5),
so a killed removal's plan does not wait for a person.

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
- Nothing tells a person to delete it except the removal's own next run or a person's decision after `doctor`
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
that is Choice 7.

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
  comparing the bytes to the example in section 2,
  so the grammar is exercised,
  not just accepted;
- a refusal for each malformed form:
  missing key,
  extra key,
  key order,
  repeated key,
  wrong version,
  wrong marker,
  a declared count above and below the line count,
  a tuple kind other than `snapshot`,
  an outcome outside the five,
  a non-canonical GUID (`+1`,
  `007`,
  an upper-case digit,
  21 digits),
  a non-canonical `createtxg`,
  a name that is not lower-case hex,
  an errno on a non-`refused` tuple,
  a missing errno on a `refused` tuple;
- a truncated record at every field boundary of the header and at the last tuple line,
  asserting the reader refuses and changes nothing;
- an oversized record (one byte past `16,384 + 600 * count`),
  refusing without reading past the bound;
- a foreign entry,
  a directory,
  a link and a wrongly owned record,
  each refused and each reported by `doctor`;
- a record whose name and `cloneGuid` disagree,
  refused;
- a staging entry `.<GUID>.<random>.partial`,
  asserting no reader reads it and `doctor` removes it;
- a folder with mode `0775`,
  refused;
  a folder `0700`,
  read;
- a plan written by the removal,
  then a second write,
  asserting the earlier record or the new one and never a mixture
  (the existing publication control shape);
- a writer that refuses a record its reader would refuse,
  forced by a ceiling one tuple below the count.

The pointer and the finish pass:

- a registration with a pointer and a plan whose `head=gone`,
  settled exactly once,
  asserting the registration is gone and the plan is kept until the origin step;
- a pointer whose plan is missing,
  refused and reported;
- a plan with no pointer,
  not found by the pass and found by the next removal by GUID;
- a pointer written before the plan,
  impossible by order,
  asserted with the write hook the primitives already expose for a staged file.

The interrupted states,
forced without timing luck:

- the existing trick for a record whose owner is gone
  (`record_destroy_with_owner`'s owner argument,
  `worktree_removal_interrupted_destroy.rs:134`)
  is the shape here:
  a control writes a plan with `requested` tuples and an owner that has ended,
  and the next run must prove,
  not re-request;
- a plan with one `destroyed`,
  one `intact` and one `unknown` tuple,
  asserting the run settles the first two,
  stops on the third,
  and deletes nothing;
- a second writer that publishes a conflicting outcome between two steps,
  asserted through the documented `AFTER_THE_STAGING_FILE_IS_WRITTEN` hook
  (`private_storage_replacing_a_record.rs`),
  showing a lost update reads as `requested` and refuses.

### Guest arms

These need a pool,
a clone,
a real destroy or an absence check:

- one arm per state of section 4 on a real clone:
  `requested`,
  `destroyed`,
  `refused`,
  `intact` and `unknown`,
  by killing the removal between the write and the request,
  between the request and the outcome,
  and by planting a per-name failure (a dependent clone,
  as [VM item 14](cli-git-zfs-measurements-owed-before-acceptance.md) did);
- the batch refusal arm and the own-snapshot control of the same item,
  repeated through the built wrapper;
- the head-refused arm with the clone's own snapshots already gone,
  asserting `head=refused` and that the record names the holder or the new snapshot;
- the absence rule:
  destroy the head,
  then create another dataset carrying the recorded name,
  and assert the run refuses to settle;
- the bookmark arm of the decided rule over a plan tuple,
  and the arm whose bookmark is refused because its snapshot went before the plan was made;
- the deferred-origin arm of plan step 15,
  with the mark set before the first delete and between the two checks;
- the delegated-account arm,
  destroying the set under the proposed descendant grants;
- the finish pass after a real head destroy,
  asserting the registration settles from the plan.

Which of these need a guest,
and why:
every arm above reads a live ZFS identity,
sends an ioctl,
or proves absence with a GUID,
so it needs a real pool;
the format,
pointer,
`doctor` and refusal arms need none.

### What a green run would not show

- that a power cut between an outcome write and its flush leaves the earlier record
  (M4 proposes a control-hook stand-in,
  which is not the kernel);
- the kernel's own crash ordering for the plan's file and the destroy's transaction group;
- any concurrent removals of one slot,
  which A2 excludes by design rather than by a measurement.

## Which plan step builds which part

- Step 14 builds the folder,
  the record's name grammar,
  the fields,
  the first write,
  the pointer,
  the refusal of a record that does not read or does not match,
  and the bookmark rule's read.
- Step 15 builds the deferred-origin check's read of the plan's clone and origin identities,
  and its refusal when the record is missing or unreadable.
- Step 16 builds the progress writes,
  the batch's read,
  the head's states,
  the absence rule's settle,
  the re-plan of new own snapshots,
  the finish pass,
  and the plan's removal after the origin hand-off.
- Step 17 builds the cancel command's read and its report.
- Step 18 builds `doctor`'s listing,
  its reports of foreign,
  oversized,
  wrongly owned,
  unreadable and pointer-less entries,
  and its removal of the known staging forms.
- Step 22 records the shipped byte ceiling beside M1's measurement.
- Step 9 supplies the verified unmount the plan's `unmountNonce` names,
  and nothing of this record is written before it.

## Choices this design marks for the project's veto

Each choice is a writer's decision,
adopted so the design is whole,
and the project can veto it at acceptance.
Each gives the options with a concrete example a person would see,
the pros and cons,
and a ranking with the reason for each adjacent pair.
The record's field spellings,
the folder's name and the modes are the design's own detail and follow the precedents section 1 cites;
they are open to the same veto with the rest.

### Choice 1: where the record lives

Option A,
the account-owned Git common directory,
`monochromatic-git-policy-cli/clone-destruction-plans/<GUID>`,
adopted.

Example:
a person removes `fix-flaky-test`;
the record appears at
`/home/u/repo/.git/monochromatic-git-policy-cli/clone-destruction-plans/7828944690444299989`,
beside the wrapper's `recorded-destroys` folder,
owned by the person,
mode `0600`.

Pros:
the writer is already the account;
no privileged channel;
the folder outlives the clone and its registration;
the wrapper's own record primitives and `doctor` work unchanged;
a person who finds one wrapper record finds this one.

Cons:
the account can rewrite the only evidence of what it destroyed,
which is the accepted same-account residual;
the record is inside a Git directory,
so a person who deletes the repository deletes it.

Option B,
root state under `/var/lib/monochromatic-git-policy-cli/`,
beside the slot identity store.

Example:
the same removal records under
`/var/lib/monochromatic-git-policy-cli/git-worktree-clone-destruction-plans/7828944690444299989`,
root-owned `0644`.

Pros:
a record the account cannot forge,
which matters only if the account is not trusted;
one place for every root-owned state of the route.

Cons:
every outcome write needs a root channel,
which the decided delegated route does not have;
the account could not read its own removal's evidence without a privileged read;
the staging record was moved out of root state for exactly this reason;
root and the account are not a boundary here (the decided threat model).

Option C,
a dataset property on the clone,
`org.monochromatic:git-worktree-clone-destruction-plan`.

Example:
`zfs get org.monochromatic:git-worktree-clone-destruction-plan tank/wt/fix-flaky-test`
prints one value of a few hundred bytes.

Pros:
no file at all,
and the record travels with the dataset;
the intent protocol already shows a property serves a short request.

Cons:
8,192 bytes (F3) hold about a dozen tuples at 600 bytes each,
and an ordinary clone has more;
the head destroy takes the property with the head,
so the outcome of its own destroy cannot be recorded;
the origin belongs to other clones,
so a property there is one record for many removals;
a property is inherited and copied by `zfs send`,
so a record could travel into a clone it was not made for.

Ranking:
A > B > C.
A beats B because the writer of every outcome is the account and B would need a channel the decided route does
not have,
for a boundary the project's threat model does not draw.
B beats C because B can hold the whole record and outlive the head,
and C can do neither.

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
it identifies the clone before the unmount,
and it cannot be misread for another clone's after it;
a new clone can never collide with an earlier one;
the receipt family already keys by this GUID,
so a person who reads one record reads the other.

Cons:
after the head is gone nothing can compute the key,
so the pointer beside the registration and `doctor`'s listing are needed for the last stretch;
a person cannot guess the name from the path.

Option B,
the registration's name.

Example:
the probe's two worktrees at one path were both `probe-slot` (F5),
so the removed clone's plan and the re-added clone's plan share one name.

Pros:
a person can see it in `worktrees/`;
the folder's contents line up with the wrapper's other records;
a command holding the registration computes it without reading ZFS.

Cons:
the name is reused (F5),
so the record must carry a registration identity to tell two clones apart,
and a reader that skips that check would adopt a plan across two lives;
the name goes with the registration the settle removes,
so the last stretch of a removal has no key at all.

Option C,
a digest of the slot path's bytes.

Example:
two clones at `/home/u/repo/.claude/worktrees/fix-flaky-test` produce one name for both.

Pros:
computable from the argument a person typed,
without Git and without ZFS;
no reuse across different paths.

Cons:
one name for two clones at one path;
the digest needs canonical bytes,
and a path that no longer exists cannot be canonicalized,
which is the cancel command's own case;
the name says nothing about the dataset it is about.

Ranking:
A > B > C.
A beats B because the GUID is unique for a clone's whole life while the registration name is reused and dies with
the registration.
B beats C because B at least names the right clone while its registration lives,
and C can name two clones with one
file.

### Choice 3: one record with a ceiling, or a folder of chunks

Option A,
one file whose byte bound is derived and whose tuple count has a shipped ceiling,
with a refusal past it,
adopted.

Example:
a clone with 40,000 snapshots would need a 24 MB record;
the removal refuses before the first delete,
prints `40,000` and the ceiling,
and the clone and its history stay.

Pros:
one read gives the whole plan,
so the bookmark rule needs one record and the reader has nothing to reconcile;
the record's own declared count makes a torn record detectable;
the ceiling's basis is a measurement (M1) and the refusal destroys nothing.

Cons:
a snapshot set larger than the ceiling cannot be removed by the wrapper at all,
which is a new refusal for a state a person can reach with a sanoid schedule;
the ceiling is a shipped number that has to be maintained.

Option B,
one record per chunk plus a small index record.

Example:
a clone with 40,000 snapshots records four chunk files of 10,000 tuples and an index naming them;
the bookmark rule reads the index and then the chunks it needs.

Pros:
no ceiling on a clone's set;
each file stays small.

Cons:
the union of the files is the plan,
so a kill between two chunk writes leaves a union that is not a plan,
and every reader must detect that;
the index and the chunks can disagree;
a person's repair is now several files.

Ranking:
A > B.
A beats B because the common case is one batch and a handful of tuples,
and the state that B fixes is a removal that A refuses without destroying anything:
a refusal a person can read is cheaper than a set of files every reader must reconcile.

### Choice 4: the flush

Option A,
publish without a flush,
adopted.

Example:
a snapshot destroy returns,
the outcome write is staged and renamed,
and the machine loses power before the page cache reaches the disk;
the next boot's plan says `requested`,
and the next removal proves the snapshot gone before it does anything.

Pros:
the rule the wrapper's records already follow (`SPEC.md:4400`);
no wait for the disk on the removal's own path;
every possible loss reads as `requested`,
which refuses safely.

Cons:
a power cut can cost the evidence of what was destroyed,
so a bookmark whose snapshot went may make a later removal refuse until a person destroys the bookmark.

Option B,
flush each outcome write and its directory,
through `replace_private_file`.

Example:
the same destroy,
the outcome write syncing the file and the directory,
then the power cut;
the next boot's plan says `destroyed`.

Pros:
the outcome survives a crash once written;
a person's later removal is not stranded by a lost record.

Cons:
every outcome write waits for the disk,
and a removal writes one per request and one per head,
so the removal's own path grows;
the crash window between the call and the write is not closed by it,
so an unknown outcome still exists.

Ranking:
A > B.
A beats B because the window that a flush would close is not the window that loses the truth:
the call has already returned when the write starts,
so a crash before the write leaves `requested` under either option,
and the flush only buys a durable outcome at a wait on the path that also waits for the destroy itself.

### Choice 5: what the plan replaces

Option A,
the plan is the ZFS route's only destroy record,
and the removal writes no `recorded-destroys` and no `removals-begun` entry,
adopted.

Example:
a managed clone removal leaves one file in `clone-destruction-plans` and nothing in
`recorded-destroys`,
and the existing finish pass finds nothing of its own to settle.

Pros:
one record decides one destroy,
so two records cannot disagree;
no existing pass acts on a clone it does not understand;
a released wrapper's own pass cannot mistake a plan for its own shape.

Cons:
a released wrapper that meets a half-removed clone has no record it understands,
so the repair is this release's or a person's;
`doctor` must report the ZFS removal state itself.

Option B,
both records.

Example:
the same removal leaves the plan and a `recorded-destroys` entry with the registration's id.

Pros:
an older release's finish pass would at least see that something was destroyed.

Cons:
two records,
two lifetimes and two readers for one destroy,
and the existing pass would drop its record as soon as the clone's path still stands,
which is exactly the state a ZFS removal is in before the head goes.

Ranking:
A > B.
A beats B because a second record read by a pass that misreads this route's state is worse than no record for that
pass,
and the older release's repair is not a behaviour this route can promise anyway.

### Choice 6: copied identities or a reference to the binding

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
a damaged binding record blocks removals that the plan alone could have completed;
the reader needs the sibling design's key for the binding.

Option B,
the plan copies the origin-side inventory too.

Example:
the plan records every origin-side snapshot and bookmark tuple,
so a removal completes without the binding record.

Pros:
no dependency on the binding's readability at removal time.

Cons:
the origin-side inventory is unbounded and grows with every sanoid run,
so the record's bound would be a second ceiling;
it is a removal-time copy of an add-time fact,
and the decided rule forbids updating the recorded lineage to match an external change.

Ranking:
A > B.
A beats B because the plan's own bound and its single reading stay bounded,
and a damaged binding record is a state the removal should refuse rather than paper over.

### Choice 7: no expiry

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
and `doctor` reports it on every run until a person settles the clone or decides.

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
and B's saving is that
file's bytes against the loss of the only record.

### Choice 8: the pointer beside the registration

Option A,
the removal writes one pointer file beside the registration and the finish pass reads it,
adopted.

Example:
after the head destroy,
a command in another worktree finds the registration,
reads `monochromatic-git-policy-cli-clone-destruction-plan`,
opens the plan by its GUID,
and settles the registration without listing anything.

Pros:
no listing on the command path,
and the pointer goes with the registration so it needs no cleanup;
a command pays one `stat` and one record read.

Cons:
a kill between the plan and the pointer leaves a plan nothing points at,
which `doctor` reports and the next removal finds by GUID.

Option B,
every command lists the plan folder.

Example:
the same command lists `clone-destruction-plans` and finds the plan by its recorded target.

Pros:
no pointer file and no ordering between two writes.

Cons:
every forwarded command pays a directory listing,
which is what the store's design refused for its own sweep (`no listing per sweep`),
and a listing is not needed to know which plan belongs to the registration in hand.

Ranking:
A > B.
A beats B because the registration is the one object a command already holds,
and the pointer turns the plan's
GUID name into something a command can find without looking at everything.

## Questions only the project can answer

### Question 1: may a clone with a partial-destruction plan be mounted?

The scene:
a removal destroyed `@earliest` and was killed before the head;
the clone stands unmounted,
and a later mount request arrives for it
(the handler's own intent,
`SPEC.md:6072`).

Option A,
mount it,
keep the plan,
and let the next removal adopt it after proving every tuple,
ranked first.

Example:
the handler mounts `tank/wt/fix-flaky-test`;
a later `doctor` prints the plan,
and a later removal finishes the set and the head.

Pros:
the clone's files are intact and a person may need them;
the evidence stays;
nothing is decided on a person's behalf.

Cons:
a clone in use looks like any other while it lacks history,
and a person who does not read `doctor` may not know.

Option B,
a person must run `cancel-worktree-removal` before the clone may mount again.

Example:
the mount request is refused with "the clone has a destruction plan with 1 unknown outcome;
run the cancel command".

Pros:
the state is settled before the clone goes back to work.

Cons:
a person who only wants their files back must first learn a command about destruction,
and the cancel command does not restore the history it settles.

Option C,
refuse mounts while any plan with an unknown outcome stands.

Example:
the same request is refused while the plan stands,
whatever the person intends.

Pros:
a clone that lost part of its history cannot be mistaken for one that did not.

Cons:
it blocks use of intact files on a state the removal itself left,
and the killed removal's plan may need a guest's help to settle,
so the clone could stay unmountable for a long time.

Ranking:
A > B > C.
A beats B because the clone's files are what a person needs and the plan already reports the loss without a new
command;
B beats C because B offers a way forward in the same session while C offers none.

### Question 2: does anything outlive the plan that tells a person what was destroyed?

Option A,
nothing beyond the removal's own output and the plan while it stands,
ranked first.

Example:
a removal prints `@earliest` and `@origin` as lost and finishes;
a week later a person asks what went and finds no record.

Pros:
no new store,
no growth,
and the loss report is printed where the decision was made;
`doctor` names a plan while any work is owed.

Cons:
a person's later question has no answer,
and a killed invocation's report may never have been read.

Option B,
one notice line carried by the next commands,
in the wrapper's notices family.

Example:
the next `git status` in the checkout prints
"monochromatic-git-policy-cli:
the clone fix-flaky-test was destroyed with its snapshots @earliest and @origin on
2026-11-22".

Pros:
the loss reaches a person who did not read the removal's output;
the notices family already exists.

Cons:
a notice is a second record of the same fact and can outlive the clone it describes;
the wrapper's notices are about work it kept,
not work it destroyed.

Option C,
an append-only history of every destruction,
reported by `doctor`.

Example:
`doctor` prints every clone destroyed in the repository's life,
with its snapshots.

Pros:
the question is always answerable.

Cons:
an unbounded store of destructive events,
with no reader that acts on it,
and nothing in the route's records asks for it.

Ranking:
A > B > C.
A beats B because the removal already prints the loss before it deletes and the plan keeps it while work is owed,
so a notice duplicates a fact at the cost of a second store;
B beats C because a short notice is bounded while C is
not.

## Dependencies on sibling designs

Each is a design being written in its own worktree,
and none is decided here.

- The add's durable binding record
  (`doc/planning/cli-git-zfs-add-binding-record-design.md`).
  The plan refers to it for the origin-side inventory (section 6).
  Assumption A1:
  a reader holding the registration identity and the clone GUID finds it.
  If its home turns out to be unreadable at removal time by design,
  a removal refuses at the third check,
  which is the same outcome as an unreadable binding.
- The record that carries origin cleanup after a registration is settled
  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`).
  The plan is removed only after the origin facts are in that record,
  or after the origin needs nothing.
  Assumption:
  the removal writes that record in the same invocation,
  before the plan's removal;
  if it does not,
  the plan stands as the origin's record until it does,
  which section 7 states.
- The deferred-destroy checks
  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`).
  The checks read the plan's clone and origin identities as the planned ones.
  Assumption:
  they read this record's fields and refuse when it does not read.
- The cancelled,
  timed-out or killed removal and the cancel command
  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`).
  The cancel command reads the plan for its report and keeps it.
  Assumption:
  its report names tuples and outcomes this record holds,
  and it does not rewrite or remove the record.
- The add-time Git lock
  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`).
  Assumption A2:
  a removal holds it while it writes,
  which is what makes the last-writer-wins publication safe among cooperating runs.
- The forced arm
  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`).
  No dependency of this record:
  a head destroy a holder blocks is `head=refused` here,
  and what ends the holder is that design's answer.
- The dirty-main restore,
  the private propagation after a replacement mount,
  and the lost-last-event triggers:
  no dependency.
  None of them reads or writes this record.

## Measurements owed

No guest ran this round.
Each item names the probe that would settle it and the step that owes it.

- M1,
  the tuple ceiling.
  In the guest,
  after the snapshot arms of items 13,
  14,
  17 and 18,
  count the clone's own snapshots and the origin-side inventory at their widest
  (`zfs list -H -p -t snapshot -o name -r <clone>` and the same under the origin),
  and record the largest set one removal submits.
  The ceiling is derived from that count with headroom,
  and plan step 22 records it beside the constant.
  Until then no number ships.
- M2,
  the kill arms of section 4.
  Kill the removal between the `requested` write and the request,
  and between the request and the outcome,
  on a clone with a dependent snapshot for the refusal arm;
  assert the record's reading and the next run's refusal.
  Guest,
  because the request needs a pool.
- M3,
  the absence rule.
  Destroy a clone's head,
  create another dataset carrying the recorded name and no recorded GUID,
  and assert the run refuses to settle;
  then destroy that dataset and assert it settles.
  Guest.
- M4,
  the loss a crash can cause.
  A control that fails between the staging write and the rename
  (`AFTER_THE_STAGING_FILE_IS_WRITTEN`),
  asserting the old record is read and the loss reads as `requested`;
  a kernel power cut cannot be forced,
  so the kernel's own ordering stays unmeasured and is stated as such.
- M5,
  the write cost on the removal's path,
  only if Choice 4 becomes the flushed form:
  the plan's write time against an interleaved control that writes the same bytes without the sync,
  reported as a ratio with its spread under rule `QNB`.
  Host,
  under the acceptance workload.

## Limitations

- The absence rule is the proposal's proposed observable and stays unmeasured here;
  the record's settle step inherits that limit.
- The record cannot distinguish "the kernel destroyed nothing" from "the kernel destroyed nothing and a name was
  missing",
  because the request ignores missing names (F4);
  a name that vanished between the enumeration and the request reads `intact` where it is gone,
  and the next absence proof corrects it to `destroyed`.
  That is one rewrite of the record,
  not a wrong delete.
- A record's key cannot be computed after the head is gone,
  so exactly one part of a removal depends on the pointer or on a listing,
  and a plan whose pointer was never written waits for `doctor`.
- The plan is evidence,
  not authority.
  A forged plan can make a bookmark pass,
  which the accepted same-account residual already allows,
  and nothing here narrows it.
- A plan cannot report an outcome it never recorded.
  A kill inside an ioctl leaves `requested`,
  and the proof that follows is the only truth the record can carry.

## What this design does not cover

- The binding record's home and format,
  and the origin-cleanup record's.
- The deferred-destroy checks' own design,
  and what each of their refusals prints.
- The cancel command's proof of the slot it may reverse,
  and what a cancelled removal restores.
- The forced arm,
  the holder identification,
  and the caller's session.
- The unmount protocol,
  the staging folder and the reattach.
- The origin's destruction and the underlay's removal,
  beyond naming where the plan's facts feed them.
- The root-destruction alternative the project did not select,
  whose plan would live under root control (proposal,
  lines 1642 to 1643).
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
- [The staging design](cli-git-zfs-staging-under-cache-design.md)
  and [its decision](../decision/cli-git-zfs-staging-under-cache.md)
- [The step 8 sweep choices](../decision/cli-git-zfs-step-8-sweep-choices.md)
- [The wrapper specification](../../package/git-policy/cli/SPEC.md)
- OpenZFS at tag `zfs-2.4.4` (commit `71a9f95`),
  read in a shallow clone that this round kept in its scratch folder.

## This round

What ran:

- an OpenZFS shallow clone at `zfs-2.4.4` and the reads cited as F1 to F4;
- the registration-name probe cited as F5,
  its log at `keep/worktree-registration-id-probe.json`
  and its script at `scratch/worktree-registration-id-probe.ts`;
- the byte arithmetic cited as F6,
  its log at `keep/plan-byte-arithmetic.json`
  and its script at `scratch/plan-byte-arithmetic.ts`.

What did not run:

- no ZFS command,
  no ioctl,
  no guest,
  no container,
  no mount or unmount,
  and no timing.

The commits that wrote this document are listed in the round's report,
each with the time `git log --format=%cI` prints.
The document is unapproved and non-final.
