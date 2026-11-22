# The dirty-main restore on rung 1 and its frozen proof

Non-final,
unapproved design proposal by an agent.
It is not approved by the project owner and nothing in it is final.
It designs one open item of the accepted ZFS worktree design,
"The dirty-main restore on rung 1",
and it decides nothing the design,
the project's answers or the plan's own records did not decide,
beyond the writer's choices that "Choices this design marks for the project's veto" lists for the project to settle.
It edited no other file.

What it read:
the decision record `doc/decision/cli-git-zfs-worktree-design.md`,
the accepted design `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
the build plan `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
the accepted staging design `doc/planning/cli-git-zfs-staging-under-cache-design.md` for its shape,
the conversion fence `doc/planning/cli-git-clone-rung-conversion-fence.md`,
the two dirty-source decisions for the other rungs,
the btrfs design `doc/planning/cli-git-btrfs-worktree.md`,
`package/git-policy/cli/doc/` for the records it keeps about the wrapper's own costs and its object database,
the measurement record `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
`package/git-policy/cli/SPEC.md`,
and the code under `package/git-policy/cli/src/native/` that the btrfs rung's restore and the clone rung's fence are built from.
Every outside fact is read at a named tag:
Git `v2.56.1`,
OpenZFS `zfs-2.4.4` and Linux `v7.3`.

What it ran,
all on this host and all read-only or confined to a scratch folder:
`findmnt`,
`stat`,
`du`,
`git ls-files`,
`git status` and a small `git rev-parse` set against the main checkout;
one rewrite of a four-byte file in the scratch folder to read the change time's resolution;
and two measurement scripts in the scratch folder that walk the main checkout's tracked paths.
It ran no ZFS command that changes anything,
started no guest,
no container and no virtual machine,
touched no pool and mounted nothing.

What it did not run:
no clone was created,
because no ZFS clone route exists on this host yet (the checkout still lives in the home dataset,
and the dataset layout of the design is plan step 19's work).
Nothing here is a measurement of a cloned worktree,
of a mount of a clone,
of the restore inside one,
or of the fallback after a refusal.
Every claim that rests on kernel,
ZFS or mount behaviour is marked unmeasured and carries the probe that would measure it under "Measurements owed".

## The open item and the words it carries

The accepted design lists the item,
quoted whole (`doc/planning/cli-git-zfs-worktree-design-proposal.md:1749` to `:1753`):

> - The dirty-main restore on rung 1.
> The answer's words:
> "Verified restore (Recommended)".
> A clone of the main
>   checkout carries whatever other sessions had uncommitted in it,
>   so the add needs a defined Git index and
>   ignored-state restore and a frozen proof that the clone matches the captured source,
>   and neither the restore nor
>   its proof is designed here ([Dirty source on rung 1](#project-decisions)).
>   Add steps 4 and 5 verify the frozen
>   content and fill the clone but restore nothing yet ([Add and Git registration](#add-and-git-registration)).

The answer the item carries is the same file,
"Dirty source on rung 1" (`:1445` to `:1452`):

> - Dirty source on rung 1.
> Decided:
> "Verified restore (Recommended)",
> a verified dirty-main restore comparable to
>   Btrfs.
>   The unselected clean-only route would have narrowed the proof but lost clone speed on a dirty main
>   checkout,
>   and the main checkout here usually holds other sessions' uncommitted edits.
>   Dirty support needs a
>   defined Git/index and ignored-state restore,
>   with a frozen proof,
>   and
>   neither route accepts an unproved source.
>   [VM item 1][vm-register] ran `git read-tree HEAD` then reported empty
>   tracked status:
>   Git saw the fixture's tracked content matching `HEAD`.
>   It did not capture the source index and
>   tree before the snapshot or compare that proof to frozen content.
>   That comparison,
>   conversion checks,
>   dirty
>   restoration and concurrent writes remain unmeasured.

The plan lists the item as step 11's blocker,
"step 11,
blocking,
writer design work" (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:2836` to `:2839`),
and until step 11 lands an add from a dirty main checkout takes rung 3,
by the answer "Ship step 10 first (Recommended)" (`:1463` to `:1468`).

## Decisions and records that bind this design

Each bullet quotes the lines that decide a point,
with path and line.

- The restore's promise and its refusals are the btrfs rung's.
`package/git-policy/cli/SPEC.md:1253` says "the new worktree is restored to what `git worktree add` makes from a clean checkout",
and `:1256` to `:1281` names every refusal that survives a dirty source:
an operation in progress or a held `index.lock`;
any unmerged,
intent-to-add,
skip-worktree or assume-unchanged entry,
or a submodule,
found anywhere in the index;
a `.gitattributes` file among the changes or untracked anywhere;
a sparse checkout;
a source that is a linked worktree;
and a clone rung for clean sources only.
- The restore's method is decided by the same section,
`SPEC.md:1283` to `:1400`:
the copy's own index is read inside the copy before its `.git` is replaced,
installed as it is with the stat data kept,
then the ignored paths are listed,
`git clean --force -d` runs,
the paths the index cannot vouch for are settled by content,
`git read-tree --reset -u` writes the commit's tree,
and the result is proven before it is published.
- A source with changes is restored to the commit the add checks out,
not only to its own `HEAD`:
`SPEC.md:1333` to `:1340` says the three runs and nothing else are run for the restore,
and `:1396` to `:1399` says "Changes made to the main checkout while the snapshot is taken or restored are out of scope,
as for every rung:
using the command is accepting that risk,
and no step defends against it."
- What the add writes,
and how a record is published without a lock:
`SPEC.md:4398` and its text,
"Each is a private file created exclusively beside its final name and renamed over it,
so a reader sees the whole record or none,
and the filesystem makes it durable with its next commit."
- The wrapper never asks for a secret:
`SPEC.md:7345` to `:7352`,
"No command the wrapper runs or forwards asks a person for a credential",
with the owner's words "Avoid ever asking for someone to type a username or password."
and "Passphrases count too (Recommended)".
- No configuration:
`doc/decision/cli-git-zfs-step-8-sweep-choices.md:17` to `:68` records that the project "first answered that everything must work by default with no configuration",
and for the case with no file at all,
"1:
no file needed (Recommended)".
- No lock for a per-item record:
`AGENTS.md` rule `NLK`,
"Concurrency is lock-free first (atomic rename,
`O_EXCL`,
one owner per item);
an unavoidable lock covers the smallest unit,
never a whole repository,
with its reason written beside it."
- The store precedent this design weighs:
`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
"the record is created with `O_EXCL` under a hidden staging name,
linked to its real name in one `linkat` (whose `EEXIST` decides the one winner),
and the staging name is then removed",
and its "Who removes a killed writer's staging file" places that removal in `doctor` at plan step 18.
- Shipped limits cite a measurement or an owner decision:
`AGENTS.md` rule `LQM`,
"Shipped resource limits (unit caps,
timeouts,
size bounds) cite a workload measurement or owner decision beside the value".
This design ships no duration and no bound it has not measured,
and names the measurement that would size each.
- Worktrees live under `.claude/worktrees/` of the main checkout (`AGENTS.md` rule `WTU`),
and every path comes from the injected home,
never a hard-coded user name (rule `HOM`).
- The threat model,
whose answer was "We have to assume my machine isn't compromised.
sudo is passwordless here anyway.",
recorded as "the account is trusted,
and root and the account are not a security boundary here" (`doc/decision/cli-git-zfs-staging-under-cache.md:221` to `:237`).
"Hardening the stated threat model may make unnecessary" says,
for each check of this design,
whether its argument survives that sentence.
- The conversion fence the project chose,
option B,
with its own limits:
"no path is recorded clean before its bytes are Git's checkout's" (`doc/planning/cli-git-clone-rung-conversion-fence.md:1660` to `:1672`),
and its limitations at `:3607` and after.
- The two other rungs' dirty-source answers:
`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` waives the merge bar for the rung 2 proof's cost,
and `doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md` decides that rung 2 clones the files its proof shows unchanged and lets Git write the rest.
Neither decides anything about rung 1's clone of a snapshot.

## Where the records differ, and what this design does about it

- A write during the snapshot is out of scope for the btrfs rung (`SPEC.md:1396` to `:1399`),
and the answer this design serves requires a proof that refuses such a write.
The two are not in conflict:
the btrfs rung's sentence stays true of the btrfs rung,
rung 1 refuses what its own proof catches,
and neither claims to defend against a write after the snapshot's transaction group (L3).
- "The clone rung,
which is only for clean sources" (`SPEC.md:1281`) is the ladder's rung 2,
the same-dataset reflink,
while the proposal's rung 1 is the snapshot clone and the plan's step 3 speaks of "the clone rung" for rung 2.
This document says "rung 1" or "the ZFS snapshot clone" for its own subject,
and "rung 2" or "the ladder's clone rung" for the other,
so no record's word is silently moved to mean the other route.
- The word "proof" names four different things across these records:
the clean check's stat proof,
the rung 2 proof that a source matches its commit,
the fence's size proof,
and this document's comparison of a clone with a capture.
"Frozen proof" here means D1 to D3 and nothing else.
- The plan's step 11 is "also blocked by issue 6323",
and the fence's proposal gives the snapshot rung the fence's write-through with its own residual.
This design reads them together:
the fence applies to rung 1,
as the proposal's step 5 says with "the existing conversion checks",
and the residual the fence names stays a limitation of both rungs rather than a decision this design makes.
- The step 8 sweep record answers that everything must work with no configuration,
and the handler still reads a root-owned file when one exists.
Nothing in this design reads or writes a configuration file:
the capture reads the checkout's index and files,
and every bound it will ever need is a measurement it names.
- [VM item 1][vm-register] moved the copied administration aside rather than deleting it,
and measured no deletion cost.
This design takes the measured route (a rename into a trash) and owes the deletion's cost to [M3][m3],
so no record is read as having measured what it did not.

## Facts

Each fact names its source,
the lines read,
and,
where it is a measurement,
the run and its numbers.
Nothing here is a measurement of a clone.

### Git, at `v2.56.1`

- F1.
An index entry records the path and,
beside it,
the change time,
the modification time,
the device,
the inode,
the mode,
the owner,
the group and the size of the file the entry was written against:
`statinfo.c:24` to `:36` (`fill_stat_data`),
whose fields are the ones the entry's `stat_data` holds.
- F2.
Git compares an entry with its file by the modification time and the change time to the nanosecond,
the inode,
the owner,
the group and the size,
under `core.checkStat` (full by default,
`environment.c:777`) and `core.trustctime` (on by default,
`environment.c:776`);
the device is compared only when Git is built with `USE_STDEV`,
which the `Makefile:213` comment presents as opt-in and Linux does not define:
`statinfo.c:64` to `:96`.
- F3.
`git read-tree --reset -u <commit>` rewrites only the paths whose file Git's own stat check can see changed.
`--reset` is `UNPACK_RESET_OVERWRITE_UNTRACKED` (`builtin/read-tree.c:186`);
an entry under a reset never refuses for being out of date (`unpack-trees.c:2238` to `:2241`);
and the update flag is set only where `lstat` fails,
or the entry's stat data differs from the file (`unpack-trees.c:3036` to `:3044`).
- F4.
`git clean --force -d` removes the untracked files and directories that no ignore rule covers (`builtin/clean.c:163`,
`:1071`) and refuses without `--force` when `clean.requireForce` is set (`builtin/clean.c:957`).
- F5.
`git checkout-index --force -u --stdin` writes the named paths out of the index through Git's own conversion,
and records each written path's stat data:
`builtin/checkout-index.c:101`,
`:178`.
- F6.
`git status` refreshes the index and writes it back unless optional locks are off:
`builtin/commit.c:1634` takes the lock only when `use_optional_locks()` is true,
`:1658` writes the index when it did,
and `environment.c:228` to `:231` reads `GIT_OPTIONAL_LOCKS`,
which `--no-optional-locks` clears.
- F7.
`git worktree add` writes `<path>/.git` as a file holding `gitdir: <administrative directory>`,
and writes the reciprocal `<administrative directory>/gitdir`:
`builtin/worktree.c:532` to `:544`,
through `write_worktree_linking_files` at `worktree.c:1105` to `:1134`.
With `--no-checkout` nothing is written into the worktree beyond that pointer.

### OpenZFS, at `zfs-2.4.4`

- F8.
A clone's initial object set is its origin snapshot's block pointer:
`module/zfs/dsl_dataset.c:1225` to `:1227` copies the origin snapshot's block pointer into the new clone,
`dsphys->ds_bp = dsl_dataset_phys(origin)->ds_bp`,
when the clone's dataset object is created.
The clone therefore holds the snapshot's own dnodes,
and not a copy of them.
- F9.
A snapshot's block pointer is the dataset's own at the transaction group being synced:
`module/zfs/dsl_dataset.c:1782` to `:1784`,
and ZFS refuses a second snapshot of the same transaction group (`:1534` to `:1539`),
so a snapshot is one consistent instant.
- F10.
A ZPL inode number is the object number:
`module/os/linux/zfs/zfs_znode_os.c:600` sets `ip->i_ino = zp->z_id`,
and `:546` takes `z_id` from the dnode's object number.
With F8,
a clone's files carry the snapshot's inode numbers.
- F11.
A snapshot that is a branch point cannot be destroyed:
`module/zfs/dsl_destroy.c:56` to `:87` refuses with `EEXIST` when the snapshot is a branch point,
which a live clone makes it.
The measurement record confirms the refusal for a snapshot with a dependent clone:
`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
"Item 2:
one named snapshot has a dependent clone",
where `lzc_destroy_snaps` answered `EEXIST` and named the dependent snapshot.
- F12.
The ZPL stores each of the access,
modification and change times as a sixteen-byte value,
seconds and nanoseconds:
`module/zfs/sa.c:1704` to `:1706`.
- F13.
The clone's copied `.git` is a directory and the nested copy of the destination holds Git's own pointer file:
measured in the guest by [VM item 1][vm-register] (`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
"1.
Register,
snapshot,
attach and fill",
"The clone's root had a `.git` directory;
the nested copy of the slot had the exact underlay pointer,
proving the snapshot included the registration.").

### Linux, at `v7.3`

- F14.
Every write sets the change time to the present:
a buffered write calls `file_update_time` (`mm/filemap.c:4472`),
which is `fs/inode.c:2501`,
and the attribute path sets the change time to the present unless its caller passes `ATTR_CTIME_SET` (`fs/attr.c:465` to `:478`).
- F15.
No userspace call selects the change time:
`utimensat` passes `ATTR_CTIME`,
but only `ATTR_ATIME_SET` or `ATTR_MTIME_SET` for the values it was given (`fs/utimes.c:41` to `:55`),
so the change time a file has can only be one the kernel set,
never one a writer chose.

### This host, measured

- F16.
A rewrite of a four-byte file with four other bytes,
inside one second,
moved the change time from `2026-11-22 17:17:23.821264374 -0500` to `2026-11-22 17:17:23.825264370 -0500`.
That is the ZFS dataset `zpcachyos/ROOT/cos/home`,
which `findmnt` reports as `zfs`.
ZFS reports both times to the nanosecond here,
and the change time moves within the same second.
Run 2026-11-22T17:17:23-05:00 on `ctime-probe.txt` in this round's scratch folder.
- F17.
The main checkout at `/home/user/Monochromatic` holds 24,685 tracked paths,
an index of 3,436,101 bytes,
and 764,137,159 bytes of tracked file content:
`git ls-files`,
`git rev-parse --git-common-dir` and `stat` on 2026-11-22,
and one `lstat` per path.
- F18.
Capturing costs,
three rounds of `capture-arms.ts` in this round's scratch folder against the main checkout:
listing the tracked names with `git ls-files -z` took 25.6,
40.9 and 29.4 ms;
one `lstat` per path with `bigint` taken 175.7,
111.3 and 134.3 ms;
reading the 3,436,101-byte index took 1.4,
1.4 and 1.5 ms;
and a SHA-256 of every tracked file took 9,380.9,
4,852.3 and 5,218.4 ms,
the first round being the cold one.
- F19.
The main checkout's common Git directory holds 2,732,668,640 bytes in 634,426 files and 6,708 directories,
its ignored state includes `.claude` at 214,899,299,440 bytes,
`package` at 2,745,349,203 bytes and `node_modules` at 185,182,920 bytes,
and its status lists 5 changed paths:
`du --summarize --bytes` and `git status --porcelain` on 2026-11-22.
- F20.
The tracked paths' names occupy 1,648,834 bytes,
one byte each of termination included,
and the longest is 192 bytes:
`ls-files -z` counted in a scratch script on 2026-11-22.

### The wrapper, in this tree

- F21.
The btrfs rung's dirty restore exists and is reused whole:
`restore_dirty` (`package/git-policy/cli/src/native/worktree/worktree_dirty_restore.rs:465`) lists the carried ignored paths,
runs `git clean --force -d` captured into the journal,
removes the files the reset cannot see,
runs `git read-tree --reset -u <commit>`,
and proves the result with `prove_restored` (`:292`).
- F22.
The index install already serves a dirty source with the copy's own index:
`install_index` (`worktree_snapshot_index_step.rs:196`) takes `own_index: Option<&[u8]>`,
and `read_own_index` (`:327`) opens `<copy>/.git/index` below a held root without following a link.
- F23.
The btrfs rung hands the copied `.git` to a trash inside the copy in one rename,
and lets the disposal empty it after the command returns:
`worktree_snapshot.rs:284` (`replace_git_pointer`) with `hand_over.take_entry`.
- F24.
The wrapper's own stat comparison ignores the device and compares the size,
inode,
owner,
group and both times,
reading a recorded nanosecond count of zero as any count:
`package/git-policy/cli/src/native/subvolume/subvolume_index_files.rs:248` to `:257`.
- F25.
The btrfs rung's fallback completes the worktree with `git reset --hard` in the destination,
which works there because a refused snapshot rung leaves the destination holding nothing but Git's pointer,
(`worktree_ladder_finish.rs`,
`checkout_like_git` and `complete_like_git`).
- F26.
The clone rung's failure sentence and the line about left-out changes are the wrapper's own:

> the clone rung could not finish (<reason>),
> and Git's own checkout made the worktree.

> the uncommitted changes of the main checkout (<count> paths) were left out of the new worktree,
> which holds what Git's own checkout holds.

The first is `worktree_slow_route.rs:278` and `SPEC.md:2787`;
the second is `worktree_dirty_restore.rs:85` and `SPEC.md:1386` to `:1393`.

## Assumptions

- A1.
A mounted clone reports the snapshot's own metadata,
because its object set is the snapshot's (F8):
the inode number,
size,
kind,
mode,
and both times of every unchanged file equal the source's at the snapshot.
[M2][m2] measures it.
- A2.
Other sessions write the main checkout through ordinary writes,
editors and Git commands,
so a write between the capture and the snapshot is a VFS write that moves the change time (F14,
F15).
A writer that sets the system clock back,
or that writes through a raw device,
is outside the threat model this design designs to.
- A3.
The wrapper's index reader and its stat comparison (F24) read an index written by Git `2.56.1`,
and a file on the clone exactly as they read them on btrfs,
so their verdicts carry over unchanged.
- A4.
The clone-local trash can be emptied after the add returns by the same disposal that empties a btrfs snapshot's trash,
with the clone mounted at the destination,
because the trash is a directory inside the published worktree in both rungs (F23).
- A5.
The frozen capture's cost in the wrapper is the order the rung 2 proof's own walk measured,
34 ms over 24,313 tracked files in `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`,
rather than the node probe's 111 to 176 ms in F18,
because the probe pays for a scripting runtime per call and the wrapper does not.
[M1][m1] measures the wrapper's own arms.

## Limits

- L1.
No clone exists on this host:
the checkout is still a directory in `zpcachyos/ROOT/cos/home`,
not the checkout dataset of the design,
and the route's own code is unreleased.
Nothing in this document is a measurement of a clone,
a clone mount,
or a restore inside one.
- L2.
No guest ran in this round,
so every ZFS and mount claim rests on the source at `zfs-2.4.4` or on the guest records,
and is marked unmeasured where the guest has not already measured it.
- L3.
The window write is detected,
never prevented.
A write after the snapshot's transaction group is a write in the main checkout,
not in the clone,
and the design does not catch it;
that is the risk `SPEC.md:1396` to `:1399` already accepts for every rung.
- L4.
The proof is only as strong as the fields it compares.
A write that happened before the capture,
in the same second as the index entry,
with the same size,
is invisible to the capture and to the snapshot proof;
the restore's content settle is the only thing that catches it,
and only for entries the stat data cannot vouch for.
That is issue 6323's class,
and this design neither widens nor closes it.
- L5.
The capture's cost figures come from scratch scripts and from another round's record (F18,
A5);
neither is the rung's own timer.
- L6.
The proof needs the clone's copied `.git/index`.
A resume that finds it gone cannot prove the frozen clone from the capture alone,
because the capture holds the index's digest and the files' facts,
not the files.

## The parts that need a design

### What "the captured source" is

The captured source is three things read from the live main checkout by the wrapper itself,
with held handles and without a Git run:

- C1.
The index:
the bytes of the checkout's own index file,
`<checkout>/.git/index`,
with its size,
its modification time and its SHA-256 digest.
The refusal checks of `SPEC.md:1256` to `:1281` read these same bytes,
so the capture and the refusals describe one index.
- C2.
Every path in that index:
its kind (regular file or symbolic link),
its size,
its modification time and its change time to the nanosecond,
and the owner's execute bit;
for a symbolic link also the bytes of its target.
The names come from the index itself,
so the capture runs no `git ls-files`.
- C3.
A SHA-256 of the content,
or of the link's target bytes,
for every path whose index stat data cannot vouch for the file.
That predicate is the wrapper's existing one (`paths_whose_content_differs`,
`worktree_clean.rs`),
the one the restore's settle and the clean check already judge a file by,
so the capture reads exactly the files the restore will read again.

It is taken twice,
and the two readings are different objects:

- the preflight reading,
at plan step 10's step 1,
whose index bytes are the ones the refusals parse,
and which costs the index read of F18 and no walk;
- the frozen capture,
taken immediately before the snapshot command in step 3,
after the fence's reads and after Git's registration.
It is written into the add's own record before the snapshot runs,
and it is the one the proof compares.

The commit the caller's add checks out is recorded beside it,
read from the new worktree's `HEAD` after Git's registration (F7),
so the record names both what the clone must hold and what the restore will write.
The tree of that commit is not captured:
the restore reads it from the object database,
which the clone's discarded administration and the real one share as files.

What the capture costs on this repository,
measured and derived:

- the index read,
1.4 to 1.5 ms for 3,436,101 bytes (F18);
- one `lstat` per tracked path,
111.3 to 175.7 ms in the scratch probe (F18) and 34 ms for the wrapper's own walk over 24,313 paths in the rung 2 record (A5),
for 24,685 paths (F17);
- the content digests of the unvouched set,
which on the checkout as it stood on 2026-11-22 was 5 paths (F19),
and which is the whole tracked tree only if every entry's stat data disagrees,
at 4,852.3 to 9,380.9 ms (F18);
- the record:
1,648,834 bytes of path names (F20) plus 24 bytes of fields for each of 24,685 entries,
so about 2.2 MB per add,
written once with an `O_EXCL` staging name and a link,
and read whole only by a resume.

The record is the add's own,
in the shape `SPEC.md:4398` gives every record an add writes for itself:
a private name beside the claim in the Git common directory,
written whole and renamed or linked into place,
readable by the account at `0600`,
flushed by nobody,
because what it guards is a command that was killed and a power cut may cost the record but never a byte of a person's.
Its name carries the add's identifier,
`<identifier>.capture`,
beside the claim's `<identifier>.stage` and the destination record's `<identifier>.add`,
and `doctor` reports a staging name a killed writer left,
as `doctor` does for the slot identity store's staging names.

### The frozen proof

The proof runs in the mounted clone after the add's step 4 has verified the mount's identity and the caller's visibility,
and before any byte of the clone is written.
It reads the clone's files below a held handle of the clone's root,
following no link,
and it runs no Git against the clone's copied administration:
a Git run there would read the copied index as its own,
rewrite it,
and could move the copied refs of the checkout,
so it would destroy the very object the proof compares and would decide from the wrong worktree.

The proof compares,
in this order:

- D1.
The bytes of the clone's copied index,
`<clone>/.git/index`,
against C1:
same length,
same SHA-256,
and the parsed entry set the same.
This catches any write to the index between the capture and the snapshot,
which is what a concurrent `git add`,
`git reset`,
`git commit`,
`git stash` or a merge in the main checkout does.
- D2.
For every path in the captured index,
the clone's own facts against C2:
the kind,
the size,
the modification time to the nanosecond,
the change time to the nanosecond,
and the execute bit,
plus the target bytes for a symbolic link.
The device and the inode are not compared,
because a clone is another dataset and,
as F8 and F10 say and [M2][m2] measures,
its inode numbers are the snapshot's own while its device is not.
- D3.
For every path C3 recorded a digest for,
the clone file's SHA-256 against it.

A difference in any of the three refuses the clone rung,
and the refusal names the path and the field that differed,
in the wrapper's own words for a file that changed under a copy.

How a write between the capture and the snapshot is caught:

- a write to a tracked file moves that file's change time to the present (F14),
no userspace call can set a change time (F15),
and ZFS keeps the nanoseconds (F12,
F16),
so D2's change-time comparison differs for that path and names it;
- a write to a tracked file's content also moves its modification time and,
for a different length,
its size,
which D2 compares too,
so the refusal does not depend on the change time alone;
- a write that replaces a file (a rename over it,
an editor's write-and-rename) changes the inode as well,
which D2 does not compare,
but the new file's change time and often its size differ,
and a replacement with the same size and the same second is the residue L4 names;
- a staged change or a commit in the window moves the index,
which D1 catches before D2 runs;
- a write whose bytes happen to keep every compared field is what D3 catches for the paths C3 covers,
and what L4 leaves for the paths it does not.

The proof is not a claim about ignored files or untracked files.
Those are carried exactly as the snapshot carried them,
which is the rung's purpose,
and the restore's own proof is what proves the finished tree.

A write that lands after the snapshot is not a window write:
the snapshot's transaction group already froze the bytes (F9),
and the write reaches the main checkout only,
which is L3.

### The restore

The clone arrives holding the source's working tree,
its ignored state,
its untracked files and a copy of the checkout's whole Git administration.
The restore turns it into Git's own checkout of the commit the caller asked for,
in nine steps,
each of which is either an existing step of the btrfs rung or a comparison from the frozen proof:

1. Verify the mount and the caller's visibility,
which is the add's step 4 and not this design's.
2. Run the frozen proof,
D1 to D3,
before any write.
3. Read the clone's own index bytes with `read_own_index` (F22).
They are the captured index,
already proven equal to C1,
and they are one instant with the clone's files,
because the snapshot froze both (F9).
4. Prune the clone-local leave-outs with the btrfs rung's own rule (`worktree_copy_prune` through `prune_snapshot`):
the copies of other adds' trash,
and the copies of nested worktree stubs and of the destination's own slot directory,
removed with `remove_stub` (the destination's stub also loses the pointer file Git wrote into it).
Every one of these is a rename or a removal inside the clone's own dataset,
so none of them is the cross-dataset hand-over the btrfs rung makes.
Nothing here can touch a live mount,
because the clone's copies are plain directories:
the snapshot holds an empty directory where a nested dataset is mounted in the source (F13).
5. Install the captured index into the new worktree's own administrative directory with `install_index` (F22),
whose dirty form already keeps the source's stat data,
drops the `UNTR` and `FSMN` extensions,
and zeroes the stat data of every path the conversion fence hands to Git.
The install adds the fields of the clone's own files:
the device and inode of every path,
read from the clone during the proof's walk,
are written in place of the source's,
while the size,
both times and the mode stay the capture's.
Every field written is then true of the clone's file,
and the verdicts that decide what the reset rewrites are the source's own.
6. Move the copied `.git` into a trash inside the clone in one rename and write the saved pointer bytes as `<clone>/.git`,
which is the btrfs rung's `replace_git_pointer` (F23).
The trash is kept out of the index by the exclude line the wrapper already writes for the trash place,
and the disposal empties it after the add returns (A4).
Deleting 634,426 files of administration on the add's waiting path (F19) is the alternative,
"Choices this design marks for the project's veto" weighs.
7. Run the btrfs rung's own `restore_dirty` with the clone's root as its stage (F21):
list the ignored paths under the source's rules,
run `git clean --force -d` (untracked files that no ignore rule covers go,
ignored files stay),
settle by content the entries the installed index cannot vouch for,
run `git read-tree --reset -u <the commit the caller asked for>`,
and prove the result with `prove_restored`.
8. Write the fence's paths through Git with `git checkout-index --force -u --stdin` (F5),
which is the clone rung's own step,
so no path is recorded clean while it holds the source's unconverted bytes.
9. Verify the worktree root,
the common directory,
the reciprocal registration,
the mount identity and the checked-out state,
mark the binding usable,
queue the dependency repair,
hand the trash to the disposal,
and print Git's line,
the line about the left-out changes,
and nothing else.

The clone therefore becomes a checkout of the commit the caller asked for:
the target tree is written by the reset and by the fence's write,
other sessions' uncommitted tracked edits are not carried,
because the reset replaces every path whose stat data marks it,
and the settle removes the ones the reset cannot see,
the index is made whole by the reset and by the proof,
the ignored state is kept because nothing in the restore addresses it,
and untracked files that no ignore rule covers are removed by the clean,
exactly as Git's own checkout leaves none.

### What it reuses, and where ZFS differs from btrfs

Reused whole,
without a change:

- the refusal set for a dirty source (`SPEC.md:1256` to `:1281`),
including the `.gitattributes` refusal and the linked-worktree refusal,
both of which the capture inherits because it reads the same index;
- the restore's three runs and its content settle (`restore_dirty`,
F21),
its proof (`prove_restored`),
and its two lines to the person (F26);
- the index install and the copy's own index read (`install_index`,
`read_own_index`,
F22);
- the leave-out rule and the trash hand-over of the snapshot rung (`prune_snapshot`,
`remove_stub`,
`replace_git_pointer`,
F23),
and the disposal that empties a trash after the command returns;
- the conversion fence,
decided before the snapshot and written through Git after the reset,
with the index's zeroed stat data for the fenced paths (F5,
F22);
- the clone rung's failure sentence and the fallback that Git's own checkout completes (F25,
F26).

Where ZFS differs,
and what the design does about each:

- There is no stage and no exchange.
On btrfs the restore happens in a private snapshot and only the proven result is swapped over the destination,
so no person ever sees a half-restored tree.
On ZFS the clone is the destination from the moment it is mounted,
so the fill is visible while it runs,
and the add's binding is the only thing that says whether the worktree is usable.
"Questions only the project can answer" asks whether the clone should be mounted read-only until the fill is proven.
- The copied administration is on another dataset from the source's trash,
so the hand-over the btrfs rung makes across one filesystem is a rename inside the clone here,
and the disposal reaches it through the published worktree path.
- The inode numbers are shared with the snapshot (F8,
F10) and the device is not,
which is why the proof does not compare the device and why the index install writes the clone's own device and inode.
- A snapshot of the checkout dataset holds an empty directory where a nested dataset is mounted in the source,
so the btrfs rung's stand-ins (`worktree_snapshot_placeholders`) have no counterpart;
the stub removal alone is the rule.
- The clone's creation is two delegated requests,
a snapshot and a clone,
whose GUIDs and `createtxg` values are recorded;
an origin snapshot cannot be destroyed while its clone lives (F11),
which fixes the order of any cleanup the fallback makes.
- A ZFS snapshot copies nothing,
so the fence's reads sit on the add's waiting path,
as they do in the btrfs snapshot rung,
and not beside a copy as in the clone rung.

### What happens when the proof fails or the restore cannot finish

The design divides the add at the mount,
because the two sides leave different things behind.

Before the clone is mounted,
a failure destroys only what this invocation made,
whose identity it recorded,
and Git's own checkout completes the worktree at the same slot in the same invocation:

- a refusal before the snapshot (an unreadable index,
a refusal the btrfs rung already makes,
a fence that cannot decide) leaves no managed object,
and the rung ends as the btrfs rung's does;
- a refused or failed snapshot or clone request (including `ENOSPC`,
which the design's step 10 already handles) keeps the claim,
destroys the clone if it exists and the snapshot after it,
because a snapshot with a dependent clone cannot be destroyed first (F11),
releases the slot and lets Git's own checkout finish it;
- an identity that cannot be proven,
or a mount that does not arrive within its bound,
leaves the clone and its snapshot in place and refuses,
because destroying an object whose identity was not proven is what the design's removal proof forbids;
`doctor` reports it and a removal settles it.

After the clone is mounted,
a failure is one of three,
and each has its own answer:

- The proof refuses (D1,
D2 or D3 found a difference).
The add prints the clone rung's failure sentence with the reason,
names the path and the field that differed,
and says the main checkout was written while the add read it.
It then tries to settle the clone in the same invocation:
request the unmount,
wait for its receipt within the step's bound,
destroy the clone,
destroy the origin snapshot in that order (F11),
release the slot,
and run Git's own checkout at the same slot,
which is `checkout_like_git` reached through the settled registration.
This destroys only objects this invocation created and recorded,
which the design's own rule for a failed request allows.
When the unmount cannot be confirmed within its bound,
or a holder keeps the clone,
the add refuses instead,
leaves the clone mounted and registered,
writes its record's pending state,
and prints what to do:
leave the folder and run the removal command,
then the same add again.
"Questions only the project can answer" asks whether that automatic settlement should be the add's,
or a removal's.
- A restore step fails after step 3 (the install,
the pointer,
the three runs,
the fence's write).
Nothing of a person's is at stake:
the clone holds the source's bytes and its ignored state,
and the source still holds all of them.
The add refuses in the same way,
settles the clone through the removal route and completes the worktree with Git's own checkout,
or leaves the pending state when the settle cannot be proven.
- The finished tree cannot be verified (the worktree root,
the common directory,
the registration or the mount differs).
This is the same pending state:
the add refuses,
records it,
and prints the removal command and the add to repeat.
It never marks the binding usable.

The lines the person reads,
in the wrapper's voice,
and what the journal keeps:

- for D1:

> the clone was not proven:
> the main checkout's index changed while the add read it,
> and Git's own checkout made the worktree.

- for D2:

> the clone was not proven:
> <path> changed in the main checkout while the add read it (<field>),
> and Git's own checkout made the worktree.

- for D3:

> the clone was not proven:
> the content of <path> changed in the main checkout while the add read it,
> and Git's own checkout made the worktree.
- for a settle that cannot be proven,
the existing sentence for an unfinished worktree,
with the removal command beside it,
from the report the wrapper already gives.
- on success,
the line Git's own checkout prints and the line about the left-out changes (F26),
and nothing about the proof.

The journal keeps the destination,
the rung,
the kind of refusal and the path,
as one entry,
in the shape the left-out note already has (F26),
so a later reading can tell which field of which path refused without the person's screen.

What a resume finds,
and what it does with each state:

- a claimed slot with no registration:
the add has not run Git;
the claim is resumed by the same add,
or cleared by the removal of a claim that never made anything;
- a registration with no clone:
Git's `worktree add --no-checkout` ran and the snapshot or clone did not happen;
the add destroys nothing and either takes a fresh snapshot,
because the frozen capture was taken before this state and is still the source's own,
or refuses when the capture's record is gone;
- a clone with no mount:
the clone's identity is checked against the record and the mount is requested again;
the proof runs after the mount arrives,
from the persisted capture,
so a source that moved since the snapshot does not fail the proof;
- a mounted clone whose copied `.git/index` is still there:
the proof runs from the persisted capture and the restore continues from the step the evidence shows,
(the copied pointer file present or absent,
the trash's manifest present or absent,
the index installed or not);
- a mounted clone whose copied index is gone:
the clone cannot be proven,
so the add refuses and settles it as a failed proof does (L6);
- a filled clone whose binding is not marked usable:
the verification of step 9 runs again,
and on success the binding is marked usable without a second restore;
- a clone with the binding marked usable:
nothing is resumed;
it is a worktree.

Every one of these states is derived from evidence on disk and from the add's record,
never from a progress field alone,
and none of them writes into the main checkout.

### The cost, against a clean add and against rung 3

No number here is claimed unmeasured,
and each owed measurement names the arm it must be compared against.

- The capture and the proof,
against a clean add of the same repository:
[M1][m1] measures the wrapper's own capture and compare arms on this host over the 24,685 paths of F17,
in position-balanced rounds with an identical-binary control,
and reports the ratio with its range.
The scratch probe's numbers (F18) bound what a scripting runtime pays and are not the wrapper's cost.
- The copied administration,
against the same add with the deletion done inline:
[M3][m3] measures the add's time with the trash hand-over (F23,
A4) against a control arm that deletes the copied `.git` in place,
on a fixture whose administration holds the files F19 measures,
and reports both ratios and their spread.
- The restore,
against Git's own checkout of the same commit from the same source:
[M4][m4] measures adds from sources with 5,
300 and 6,000 changed tracked paths,
alternating arms with an identical-binary control,
and reports the ratio to Git's own checkout.
The rung 2 partial-clone decision's cut of 6,000 changed files and its whole-tree threshold are the comparison the project already accepted for the other rung,
which `doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md` records.
Rung 1 must not be slower than rung 3 at the changed-path counts the project's own checkout shows.
- The fallback after a refused proof,
against rung 3 alone:
[M5][m5] measures an add that pays the snapshot,
the clone,
the mount,
the proof and the settle,
then Git's checkout,
against the same add on a build that never takes rung 1.
- The proof's own refusal rate on a busy checkout,
which is a cost in wasted work rather than time:
[M6][m6] measures how often a proof refuses when other sessions write the checkout during the add,
at the window this design leaves (capture immediately before the snapshot),
and compares it with the longer window a preflight capture would leave.

### The tests a build must have

Each race is forced at a seam the code already offers or gains,
never by timing.

- T1.
`the_frozen_proof_refuses_a_tracked_write_between_the_capture_and_the_snapshot`.
The rung's snapshot step gains a seam between the capture and the snapshot request,
in the shape `PlaceSeams` already has (`worktree_snapshot.rs`),
and the control writes a tracked file in it.
Assert the refusal,
the path it names,
and that the person reads the clone rung's sentence and gets Git's own checkout.
Runs on the host,
over a stand-in snapshot that copies a fixture tree,
because the proof and the capture never need a ZFS call.
- T2.
`the_frozen_proof_refuses_an_index_written_between_the_capture_and_the_snapshot`.
The same seam,
with a control that runs `git add` in the fixture in the seam.
Assert the refusal is D1's,
names the index,
and does not run the restore.
- T3.
`a_write_that_keeps_the_size_and_the_modification_time_is_caught_by_the_change_time`.
The seam's control writes the same length and copies the modification time back over it,
and the test asserts D2's change-time comparison refuses it,
which is the one field no writer can set (F15).
- T4.
`the_capture_reads_by_content_only_the_entries_the_stat_data_cannot_vouch_for`.
A fixture with one edited path and many untouched ones;
assert one digest was recorded,
and that a control which touches every file records every path.
This is the test that keeps the capture's cost from drifting to F18's digest arm.
- T5.
`a_resumed_add_proves_the_frozen_clone_from_the_persisted_capture`.
Kill the add after the snapshot,
move the source,
resume,
and assert the proof passes from the record and the restore completes.
The positive control removes the record and asserts the resume refuses and settles the clone (L6).
- T6.
`the_index_the_clone_installs_vouches_for_no_path_the_fence_leaves_to_git`,
the twin of the btrfs control of the same name,
extended to the clone's install,
and `a_converting_path_in_a_clone_is_written_by_git`,
the twin of the btrfs control that compares the served path with a real Git checkout of the same commit.
- T7.
`the_clone_restores_a_dirty_source_to_the_commit_the_caller_asked_for`.
A guest fixture with one staged edit,
one unstaged edit,
one deletion,
one mode change,
one untracked file that is not ignored,
one ignored file and one symbolic link,
cloned and restored to a commit other than the source's `HEAD`;
assert the result equals Git's own checkout of that commit plus the ignored file,
and that the person reads the left-out line with the count.
- T8.
`a_proof_failure_settles_the_clone_and_completes_with_gits_checkout`,
in the guest,
because it needs the handler's unmount and the clone's destruction;
and its control where a holder keeps the clone,
asserting the add refuses with the pending state and names the removal.

Which need a guest:
T6's converting arm,
T7,
T8 and its control,
and T1 and T5 run twice,
once on the host over the stand-in snapshot and once in the guest over a real clone.
T2,
T3 and T4 run on the host.

## Which build step builds which part

- Plan step 10 builds the add's route as far as the verified fill,
with the interim the owner decided:
a dirty main checkout takes rung 3,
and the frozen proof is not in it.
- Plan step 11 builds C1 to C3,
the capture's record and its publication,
D1 to D3,
the snapshot seam T1 and T2 drive,
the clone's own index install with the clone's device and inode,
the leave-out on a clone root,
the pointer hand-over into a clone-local trash,
and the wiring of `restore_dirty` and the fence's write to the clone root.
It also builds the refusal sentences and the resume states of "What happens when the proof fails or the restore cannot finish".
- The settlement a refused proof makes after the mount belongs to the removal's build,
steps 14 to 17,
because it is an unmount,
a destruction and a Git settle;
step 11 calls it and cannot finish its fallback before it exists.
- `SPEC.md` gains,
under the add section and beside the btrfs rung's dirty paragraph:
what a clone of a dirty main checkout restores,
what the proof compares,
the refusal when it differs,
the index install's device and inode rule,
and the resume states.

## Measurements owed

Each is a probe a later round runs as written,
with its control arm,
its rounds and the spread reported (rules `NQB` and `NQH`),
and each says where it runs.

- M1.
The capture and the proof's cost in the wrapper.
Arm A:
the shipped build.
Arm B:
the same binary with the proof and capture disabled behind an environment switch,
which is the identical-binary control.
Arm C:
the frozen build again,
so the rounds rotate through A,
B,
C.
Twenty rounds on this host,
the add run through the wrapper on a fixture of 24,685 paths of the sizes F17 gives,
with each add's own timer reporting the capture,
the walk and the compare separately.
Report each arm's median and range and the ratio A to B,
and the count of rounds where A was slower.
Runs on the host.
- M2.
The clone's metadata identity.
In the guest,
on a file-backed pool:
create a fixture of 2,000 paths with mixed kinds,
sizes,
modes and one symbolic link;
snapshot,
clone,
mount,
and compare inode,
size,
kind,
mode,
and both times to the nanosecond for every path between the mount's root and the source;
report every path that differs and how.
The positive control writes one file in the window and asserts the comparison reports exactly that path,
so a null result is trusted only after a case that must differ does (rule `QPC`).
- M3.
The copied administration's hand-over.
In the guest:
an add whose clone carries an administration of 634,426 files in 6,708 directories,
one arm handing it to the clone-local trash and one arm deleting it in place,
alternating with the identical-binary control,
ten rounds;
report each arm's median and range and the disposal's cost separately,
measured from the journal's own timestamps,
so nothing on the add's waiting path is hidden in a background pass.
- M4.
The restore against Git's own checkout.
In the guest:
sources with 5,
300 and 6,000 changed tracked paths,
each added with rung 1 and with rung 3,
alternating arms with the identical-binary control,
ten rounds a shape;
report the ratio for each shape with its range.
A shape where rung 1 is slower than rung 3 is a finding for the project,
not a limit to be waived without asking.
- M5.
The fallback after a refused proof.
In the guest:
an add whose proof is forced to refuse at the T1 seam,
timed from the command's start to its end,
against the same fixture added on a build that never takes rung 1.
Report the ratio and the range,
and the same measurement for the pending-state arm,
where no fallback runs.
- M6.
The refusal rate on a busy checkout.
On this host:
100 adds from the main checkout while other sessions write it,
with the capture immediately before the snapshot,
and 100 with the capture at preflight;
report how many of each refused and the changed paths that caused them.
Nothing here generates load:
the adds are real adds,
and the sessions writing the checkout are whichever sessions are there (rule `NQH`).

## Choices this design marks for the project's veto

Each entry says what the design does,
which options it had,
a concrete example for each option and the result a person would see,
the pros and cons,
and the ranking with the reason for each adjacent pair.
A veto returns the entry to the writer;
taking none leaves the design as it stands.

- V1.
What the frozen capture holds.
The design records the index's bytes and digest,
every tracked path's facts,
and a content digest for the entries the stat data cannot vouch for.
  - Option (a),
this one:
an add from the main checkout as it stood on 2026-11-22 reads 3,436,101 bytes of index and 24,685 `lstat`s,
plus 5 files' content,
and costs the order of 40 to 180 ms in the wrapper.
  - Option (b),
a content digest of every tracked path:
the same add reads 764 MB and pays 4.9 to 9.4 s (F18),
and catches nothing (a) does not,
because a window write always moves the change time.
  - Option (c),
the index's bytes alone:
the same add pays 1.5 ms and catches a concurrent `git add`,
but a session editing a tracked file in the window is carried into the clone and published as Git's checkout.
  - Pros and cons:
(a) closes the window the decision names at a measured cost;
(b) buys nothing for this window and costs the add's whole time;
(c) is nearly free and leaves the window the answer's own words call unproven.
  - Ranking:
a > c > b,
because a proves the tracked content the answer names,
while c leaves it unproven and b pays nine seconds for the same proof (a) has.

- V2.
Which fields the per-path comparison uses.
The design compares kind,
size,
modification time,
change time,
the execute bit and a link's target,
and takes the device and inode from the clone for the index install.
  - Option (a),
this one:
a session that edits `AGENTS.md` in the window moves its change time,
and the add refuses naming `AGENTS.md` and its change time.
  - Option (b),
the inode and device compared too:
the same add refuses every path whose inode or device the clone did not share,
which [M2][m2] decides;
if the clone shares neither,
every add refuses.
  - Option (c),
size and modification time only:
a session that rewrites a file,
keeps its length and copies its modification time back leaves the clone carrying the other session's bytes,
published as clean.
  - Pros and cons:
(a) is the strongest set that cannot refuse a faithful clone;
(b) adds two fields whose sharing is a measurement,
not a fact of the design;
(c) is one comparison cheaper and drops the field no writer can set (F15).
  - Ranking:
a > b > c,
because a cannot refuse a faithful clone while b can,
and c loses the only field a write cannot hide.

- V3.
Where the frozen capture lives.
The design writes it as the add's own record beside its claim,
created with an exclusive staging name and linked into place,
and `doctor` reports a staging name a killed writer left.
  - Option (a),
this one:
an add killed after the snapshot is resumed the next morning,
and proves the frozen clone even though the checkout moved meanwhile.
  - Option (b),
memory only:
the same resume cannot prove the clone,
so it refuses,
destroys the clone and pays a new snapshot and clone.
  - Option (c),
a field of the durable binding record another round designs:
the same bytes,
in a record whose home and format that round owns,
so this design would depend on its answer for the capture's own shape.
  - Pros and cons:
(a) buys the resume and keeps the publication shape the add's other records already use;
(b) costs a snapshot and clone on every resumed add whose source moved;
(c) is the same data with a dependency that cannot be settled here.
  - Ranking:
a > c > b,
because a needs no other design's answer while c waits for one,
and b throws away the clone (a) proves.

- V4.
When the frozen capture is taken.
The design takes it immediately before the snapshot request,
after the fence's reads and after Git's registration,
and uses the preflight reading only for the refusals.
  - Option (a),
this one:
the window is one snapshot request wide,
and the add pays one extra `lstat` walk.
  - Option (b),
at preflight only:
the window spans Git's registration too,
so a session that saves a file while Git runs makes the proof refuse,
at no extra walk.
  - Option (c),
after the snapshot:
the capture would read the clone,
which proves nothing about the source.
  - Pros and cons:
(a) narrows the window to the smallest span the design can reach,
for one walk of the order A5 gives;
(b) is one walk cheaper and refuses more often on this machine,
whose checkout is rarely still (rule `NQH`);
(c) is no proof at all.
  - Ranking:
a > b > c,
because a spends a walk to remove Git's registration from the window,
and b's saving buys the refusals [M6][m6] counts.

- V5.
What the add does when the proof refuses after the mount.
The design settles the clone through the removal route and completes the worktree with Git's own checkout,
and refuses with a pending state when the settle cannot be proven.
  - Option (a),
this one:
a proof that refuses for one edited path costs the add the snapshot,
the clone,
the mount,
the settle and Git's checkout,
and the person ends with a worktree,
as Git's own add would have made.
  - Option (b),
never fall back once the snapshot exists:
the same add costs less and ends with no worktree,
a pending clone,
and the removal command to run before trying again.
  - Option (c),
leave the clone registered and run Git's checkout at a second destination:
the person ends with two registered worktrees and a second path they did not name.
  - Pros and cons:
(a) keeps the command's promise that an add either makes a worktree or refuses,
at the cost of [M5][m5]'s time;
(b) is cheapest and leaves the person without what they asked for;
(c) leaves state a person did not ask for.
  - Ranking:
a > b > c,
because a delivers the worktree the person asked for while b does not,
and c leaves two worktrees where one was asked for.

- V6.
How the copied administration is discarded.
The design renames it into a trash inside the clone and lets the disposal empty it after the add returns,
as the btrfs rung does.
  - Option (a),
this one:
an add moves 634,426 files with one rename and returns with a hidden trash in the new worktree until the disposal has emptied it.
  - Option (b),
delete it in place:
the same add pays the unlinking of 634,426 files on its waiting path,
which [M3][m3] measures.
  - Pros and cons:
(a) costs one rename and reuses the mechanism the rung already has,
at the price of a copy of the checkout's administration living in the new worktree until the disposal runs;
(b) leaves nothing behind and pays the whole deletion before the worktree is usable.
  - Ranking:
a > b,
because the btrfs rung already decided that a copy of the administration is worth one rename,
rather than a deletion on the add's path (F23),
and the same account already holds every byte of it.

## Questions only the project can answer

- Q1.
May an add destroy objects it made minutes earlier,
to complete the worktree with Git's own checkout?
  - Option (a),
the design's reading:
yes,
when the object's creation and identity are this invocation's own and are recorded,
so a refused proof settles the clone and its snapshot and the add finishes at the same slot with Git's own checkout.
Example:
one edited `AGENTS.md` in the window makes the proof refuse;
the add unmounts,
destroys,
checks out,
and the person has the worktree they asked for,
after [M5][m5]'s delay.
  - Option (b):
no;
an add only creates,
and a removal destroys.
Example:
the same edit makes the add refuse and print the removal command;
the person leaves the folder,
runs the removal,
and runs the add again,
and the second add takes rung 3.
  - Option (c):
yes for the clone,
never for the origin snapshot,
which stays until a removal destroys it.
Example:
the same refusal leaves a snapshot pinned to no clone,
which `doctor` reports until a removal destroys it.
  - Pros and cons:
(a) keeps the add's promise and adds a destructive step to a command that is otherwise additive;
(b) keeps destruction in the removal's hands and leaves the person without a worktree after a failure that was not theirs;
(c) splits the difference by keeping the destruction in the removal's hands for the object that holds history,
and leaves an object that holds none of a person's data.
  - Ranking:
a > c > b,
because a finishes the command the person typed while c leaves a stray snapshot and b leaves them with nothing.

- Q2.
Is the frozen capture kept after the worktree is usable?
  - Option (a),
the design's reading:
it is removed when the binding is marked usable,
so nothing accumulates.
Example:
an add of this repository writes about 2.2 MB,
and the record is gone when the add returns.
  - Option (b):
it is kept beside the binding for the life of the worktree,
so a later `doctor` can re-prove a filled clone against what the source held.
Example:
the same add leaves 2.2 MB per worktree,
and 50 worktrees leave 110 MB.
  - Option (c):
it is kept until the next `doctor` run,
which reports and removes it.
Example:
the same add leaves 2.2 MB until `doctor` runs.
  - Pros and cons:
(a) costs nothing and gives up the only record that could re-prove the fill later;
(b) buys that re-proof at a standing cost per worktree;
(c) buys it for a while and needs step 18's `doctor` to be the remover.
  - Ranking:
a > c > b,
because a costs nothing for a proof whose value expires when the fill is verified,
c keeps it briefly for a later inspection,
and b makes every worktree pay for it.

- Q3.
Is the clone writable from the moment it is mounted,
before the fill is proven?
  - Option (a),
the design's reading:
yes;
the binding,
not the mount,
says whether a worktree is usable,
and a person who writes there is writing into a tree the restore is about to replace.
Example:
a `touch` in the slot during the fill is replaced by the reset,
or makes the proof refuse if it touches a tracked path.
  - Option (b):
the handler mounts the clone read-only until the wrapper has proven the fill,
then remounts it writable.
Example:
the same `touch` fails with `EROFS` while the fill runs,
and succeeds after the add returns;
the cost is a second mount and a new handler mode that the mount designs do not have.
  - Option (c):
the wrapper refuses any add whose slot a second session could reach,
which on this machine means refusing when another session's shell is inside `.claude/worktrees`.
Example:
the add refuses whenever a person is looking at a worktree,
which is most of the time here.
  - Pros and cons:
(a) costs nothing and leaves a window in which a person's write into the slot is discarded,
which is the same window Git's own checkout has;
(b) closes that window and adds a mount transition and a handler state other designs would have to grow;
(c) closes it by refusing ordinary use.
  - Ranking:
a > b > c,
because a matches what Git's own checkout does to a worktree it is still filling,
b buys more than this window is worth and reaches into the handler's design,
and c refuses the common case.

## Dependencies on the sibling designs

Each is a place where this design answers with an assumption a later pass reconciles.

- The durable binding record (its home and format,
another round's document):
this design writes the frozen capture as the add's own record beside its claim (V3a),
and assumes the binding record does not have to hold it.
A later answer that puts the capture in the binding record changes V3 to its option (c) and nothing else.
- The add-time Git lock (steps 10 and 13):
the frozen capture is written and the proof runs while the add holds its claim and its managed lock.
This design assumes the lock's coverage reaches the snapshot request and the proof,
and that it does not order a second session's write to the checkout,
which nothing can.
- The removal designs,
for the fallback after a refused proof (V5a,
Q1a):
this design calls the unmount,
the clone's destruction,
the origin snapshot's destruction in that order,
and the Git settle.
It assumes the cancellation,
deferred-destroy and forced-removal designs leave those steps callable from an add's own invocation,
and that the add's own cleanup needs no forced arm;
if a holder keeps the clone,
this design refuses rather than ending it.
- The persisted destruction plan (another round's document):
the fallback's settlement is a destruction of two objects,
and this design assumes a plan is written before the first of them,
as the removal design requires.
- The private propagation after a replacement mount,
and the lost last event's triggers:
this design's step 1 assumes the mount of a fresh clone is verified in the caller's namespace,
by the add's step 4 before the proof runs,
exactly as the design requires,
and adds no trigger of its own.
- The start probe and the ABI gate (step 6):
nothing here reads the kernel's layout;
the proof reads files and `stat` only.

## How the accepted sentences are met

- "a verified dirty-main restore comparable to Btrfs":
the restore is the btrfs rung's own,
run with the clone's root as its stage,
and its proof is the frozen proof this document designs.
- "neither route accepts an unproved source":
the clone rung refuses whenever D1 to D3 differ,
and the person gets Git's own checkout,
which is what the other route would have given.
- "capture the index/tree verification inputs" (design step 1):
C1 to C3,
taken at preflight for the refusals and again as the frozen capture before the snapshot.
- "Compare the frozen index and tracked content in the clone itself against the captured proof,
without running Git against that directory.
The in-snapshot check proves the bytes" (design step 4):
D1 to D3,
run before any write and before any Git run inside the clone.
- "Use the verified index and the existing conversion checks and tree difference in the new worktree context" (design step 5):
the index install of step 5 and the fence's write of step 8.
- "A clone of the main checkout carries whatever other sessions had uncommitted in it,
so the add needs a defined Git index and ignored-state restore":
the index is the captured one,
installed and then rewritten by the reset;
the ignored state is carried untouched.
- "No configuration":
nothing here has a setting;
every threshold it needs is a measurement it names.
- The nearest lock-free record shape:
the capture is written with an exclusive staging name and linked into place,
and `doctor` reports a staging name a killed writer left.

## Hardening the stated threat model may make unnecessary

The project's sentence treats the account as trusted and root and the account as no security boundary on this machine.
Each check of this design is placed against it here,
and none is dropped on the strength of the sentence alone;
an entry that also answers an accident says so,
because an accident is what this machine's own sessions produce most of.

- The frozen proof's refusal (D1 to D3).
It survives the sentence:
the writes it catches are this machine's other sessions saving files and running Git in the main checkout,
which happen daily,
and it catches a person's mistake in reading which worktree they were in.
- Comparing the copied index byte for byte rather than field by field.
It survives:
byte equality is the cheaper comparison and it also catches a reordering or an extension a field comparison might not read.
- Reading the clone's files below a held root without following a link.
It survives as an accident guard:
a symbolic link in the clone is a person's,
and a replaced directory is a mistake,
and the helper is the wrapper's own (`open_root`,
`DirectoryChain`).
- Writing the clone's device and inode into the installed index.
It survives:
without it a faithful clone's index would disagree with its own files on every path,
which is a correctness need and not a defence.
- Verifying the mount's identity at the slot before the proof runs.
It survives:
a mount stacked at the slot is as much a person's own shell as an attacker's,
and the add's step 4 already decides it.
- Checking the capture record's owner and mode before reading it.
This one exists only for a hostile account:
the record lies in the account's own Git common directory,
where the account can write it,
and a mode check stops another account,
of which this machine has none.
It stays,
labelled as such,
because it is one `stat` and it names a record that a second account wrote.
- Refusing to end a holder that keeps the clone.
It survives:
the add has no authority to end another process and seeks none,
which is why a held clone makes the add refuse instead of settling.
- Refusing to substitute one source for another.
It is decided,
and it survives:
a linked source restored instead of the main checkout would be a silent wrong answer,
not an attack.
- `doctor` reporting a staging name a killed writer left.
It survives as tidiness:
the name holds no person's byte and costs one line.

## Limitations

- The proof detects a window write and cannot prevent one,
and its detection rests on the change time,
which a writer can only move forward,
never set.
- A write that happened before the capture,
in the same second as the index entry and with the same length,
is invisible to the proof,
as it is to the btrfs rung's restore;
issue 6323's class stays open for both.
- The clone's fill is visible and writable before it is proven (Q3a),
so a person can write into a tree the restore is about to replace,
and the proof refuses if that write touched a tracked path.
- The ignored state the clone carries is the ignored state of the snapshot's instant,
not of the add's return,
which is the accepted meaning of a snapshot on the btrfs rung too (`SPEC.md:1468` to `:1480`,
and `:1518`).
- The copied administration lives inside the new worktree until the disposal empties it (V6a),
so a `du` of the new worktree reports it and a person who deletes it deletes only a copy.
- The clone shares its blocks with the origin snapshot,
so the origin cannot be destroyed while the clone lives (F11),
which fixes the order of every cleanup and is why a fallback that cannot destroy the clone cannot destroy the snapshot either.
- Nothing here measures a real clone,
a mount,
a restore inside one or a fallback;
every such claim carries the probe that measures it.
- The capture's record is about 2.2 MB on this repository (F20),
which is a per-add write the add did not have before.

## What this design does not cover

- The mount and unmount themselves,
the handler's sweep,
the staging folder and the reattach,
which their own designs own.
- The removal's lineage proof,
the destruction plan,
the deferred-destroy checks,
the cancel command and the forced arm.
- The durable binding record's home and format,
the managed Git lock's write path,
and `doctor`'s report of the states this design names.
- The rung 2 partial clone of a dirty source,
issue 6323's own fix,
and the conversion fence's cost,
which its own proposal carries.
- The dependency repair after a clone fill (plan step 21),
which step 9 queues as before.
- The one-time move of the checkout to its own dataset,
the boot units and the migration (plan step 19).

## Revisions

- 2026-11-22T17:35:03-05:00,
first text.
Written from the records named at the top,
with the probes of F16 to F20 run in this round's scratch folder (`ctime-probe.txt`,
`stat-walk.ts` and `capture-arms.ts`,
with `capture-arms.ts` run over three rounds and the path-name count taken by a fourth script).

## Sources read

Git `v2.56.1`,
cloned into this round's scratch folder from `git/git` at tag `v2.56.1`:
`statinfo.c`,
`read-cache.c`,
`read-cache-ll.h`,
`unpack-trees.c`,
`builtin/read-tree.c`,
`builtin/clean.c`,
`builtin/checkout-index.c`,
`builtin/commit.c`,
`builtin/worktree.c`,
`worktree.c`,
`environment.c`,
`Makefile`,
`config.mak.uname`.

OpenZFS `zfs-2.4.4`,
cloned into this round's scratch folder from `openzfs/zfs` at tag `zfs-2.4.4`:
`module/zfs/dsl_dataset.c`,
`module/zfs/dsl_destroy.c`,
`module/zfs/zfs_ioctl.c`,
`module/zfs/sa.c`,
`module/os/linux/zfs/zfs_znode_os.c`,
`include/sys/zfs_znode.h`.

Linux `v7.3`,
cloned into this round's scratch folder from `torvalds/linux` at tag `v7.3`:
`fs/attr.c`,
`fs/utimes.c`,
`fs/inode.c`,
`mm/filemap.c`.

This repository,
read at `5dd8883a60`:
`AGENTS.md`;
`doc/decision/cli-git-zfs-worktree-design.md`;
`doc/decision/cli-git-zfs-staging-under-cache.md`;
`doc/decision/cli-git-zfs-step-8-sweep-choices.md`;
`doc/decision/cli-git-zfs-slot-identity-store-format.md`;
`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`;
`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`;
`doc/planning/cli-git-zfs-worktree-design-proposal.md`;
`doc/planning/cli-git-zfs-worktree-implementation-plan.md`;
`doc/planning/cli-git-zfs-staging-under-cache-design.md`;
`doc/planning/cli-git-clone-rung-conversion-fence.md`;
`doc/planning/cli-git-btrfs-worktree.md`;
`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`;
`package/git-policy/cli/SPEC.md`;
`package/git-policy/cli/src/native/worktree/worktree_dirty_restore.rs`,
`worktree_snapshot_index_step.rs`,
`worktree_snapshot.rs`,
`worktree_snapshot_source.rs`,
`worktree_ladder_finish.rs`,
`worktree_slow_route.rs`,
`worktree_clean.rs`,
`worktree_clean_stat_verdicts.rs`,
`worktree_clone_conversion_fence.rs`,
`subvolume/subvolume_index_files.rs`.

## Links

- The item's decision:
`doc/decision/cli-git-zfs-worktree-design.md`.
- The accepted design:
`doc/planning/cli-git-zfs-worktree-design-proposal.md`.
- The build plan:
`doc/planning/cli-git-zfs-worktree-implementation-plan.md`.
- The accepted staging design,
for this document's shape:
`doc/planning/cli-git-zfs-staging-under-cache-design.md`.
- The conversion fence:
`doc/planning/cli-git-clone-rung-conversion-fence.md`.
- The other rungs' dirty-source answers:
`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` and `doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`.
- The measurement record:
`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`.

[m1]:
#measurements-owed [m2]:
#measurements-owed [m3]:
#measurements-owed [m4]:
#measurements-owed [m5]:
#measurements-owed [m6]:
#measurements-owed [vm-register]:
../planning/cli-git-zfs-measurements-owed-before-acceptance.md
