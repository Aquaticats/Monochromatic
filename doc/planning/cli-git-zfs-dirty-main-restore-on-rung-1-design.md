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

This revision answers the gate review at `~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review.md` (tip `1f24dcc10d` of this document) and its probe logs beside it.
"Answers to the gate review" records,
finding by finding,
what changed and where.

What it read:
the decision record `doc/decision/cli-git-zfs-worktree-design.md`,
the accepted design `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
the build plan `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
the accepted staging design `doc/planning/cli-git-zfs-staging-under-cache-design.md` for its shape,
the conversion fence `doc/planning/cli-git-clone-rung-conversion-fence.md`,
the two dirty-source decisions for the other rungs,
the btrfs design `doc/planning/cli-git-btrfs-worktree.md`,
the measurement record `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
`package/git-policy/cli/SPEC.md`,
the code under `package/git-policy/cli/src/native/`,
`package/git-policy/cli/doc/slot-identity-store-write-cost.md` and `package/git-policy/cli/doc/object-database-and-a-reftable-ref-store.md`,
the gate review and its probe logs,
and the sibling designs named under "Dependencies on the sibling designs",
each at the commit named there.
Every outside fact is read at a named tag:
Git `v2.56.1`,
OpenZFS `zfs-2.4.4` and Linux `v7.3`.

What it ran,
all on this host and all read-only or confined to a scratch folder:
`findmnt`,
`stat`,
`du`,
`gh api` for five Linux files at `v7.3`,
`git log` and `git show` for the sibling branches,
and the citation check the "Citations" section describes.
The only measurements quoted here were taken in this round's first text,
by the reviewer,
or by the records named beside them;
none was taken by this revision.

What it did not run:
no clone was created,
because no ZFS clone route exists on this host yet,
and no guest,
container or virtual machine ran.
Nothing here is a measurement of a cloned worktree,
of a mount of a clone,
of the restore inside one,
or of the fallback after a refusal.

## Answers to the gate review

Each line names the finding,
what changed,
and where.

- H1,
the fields the installed index carries.
Fixed by removing the device-and-inode rewrite from step 5:
every entry keeps the fields its own index recorded,
the private write is given a lookup that answers nothing,
and the only situation that would need more is named as a contingency for [M2][m2].
See "The restore" step 5,
"What it reuses,
and where ZFS differs",
F28 and F29,
and T6.
- H2,
the order of the two publications.
Fixed by writing the captured index under a private name and publishing it and the pointer last.
See "The restore" steps 5 to 8,
"What a resume finds",
T10,
and Q-D.
- M1,
the change time's tick on ZFS and Git's second.
Fixed by replacing the false claim with the sources and the reviewer's probes:
OpenZFS stamps both times from the coarse clock,
the installed Git compares whole seconds,
a same-tick rewrite is invisible,
and the capture now digests the racy set so the window write is refused.
See F2,
F14 to F16,
F22,
F23,
"The frozen proof",
V1,
V4 and T3.
- M2,
the refusals never applied to the capture.
Fixed:
the whole dirty refusal set is re-run against the frozen capture,
and the departure from the plan's wording is listed under "Where the records differ" and put to the project as Q-C.
- M3,
the fallback against the accepted rule.
Fixed:
proposal lines 427 to 431 are quoted,
V5 and Q1 are merged into Q-A,
the design refuses with the pending state until the project answers,
and the settlement is marked as depending on a callable unmount-and-destroy no sibling offers.
- M4,
the resume list and its tests.
Fixed:
the restore is idempotent,
the record carries the proof's phase,
and the states are keyed to evidence each step leaves.
See "What a resume finds" and T5.
- M5,
the tests.
Fixed:
the host stand-in is the fixture tree itself,
every refusal test carries the no-write control,
T3 advances the tick by state,
and the missing controls are added.
See "The tests a build must have".
- M6,
the exceptions to "becomes a checkout".
Fixed:
the closing paragraph is replaced by the list of cases,
with the nested-repository case,
the metadata case,
the index extensions and the corrected issue 6323 sentences.
- M7,
the walks-and-clean bound and the omitted-mount refusal.
Fixed:
both records are quoted under "Decisions and records that bind this design",
and both are applied in the restore;
the F13 sentence is corrected.
- M8,
the build allocation.
Fixed:
D1 to D3 are the proof of every rung 1 add,
clean ones included,
and belong to step 10;
step 11 is the dirty restore and the fallback;
the plan change is veto entry V6.
- M9,
the capture's specification.
Fixed:
absent and other kinds are recorded,
the format,
the folder,
the phase,
the journal fields and the capture's end on every exit are stated,
D1's byte comparison is a veto entry,
and V3(c) is struck.
- M10,
the cost section.
Fixed:
no interpolated number,
a clean-add arm,
a test-only build instead of a shipped switch,
the rung 1 bar moved to Q-B,
and the rule code corrected.
- L1 to L8 are fixed in the sections they name:
one failure sentence,
the inherited refusals re-worded,
concrete examples in the veto list,
V2's ranking made symmetric,
the report's `USE_STDEV` slip withdrawn,
Q2 and Q3 removed,
the files-read list completed,
and the index's churn noted in F25.
- The seven wrong citations and the two loose ones are corrected,
and every other citation was re-checked by the script "Citations" describes.

## The open item and the words it carries

The accepted design lists the item,
quoted whole (`doc/planning/cli-git-zfs-worktree-design-proposal.md:1749` to `:1753`):

> - The dirty-main restore on rung 1.
> The answer's words:
> "Verified restore (Recommended)".
> A clone of the main
> checkout carries whatever other sessions had uncommitted in it,
> so the add needs a defined Git index and
> ignored-state restore and a frozen proof that the clone matches the captured source,
> and neither the restore nor
> its proof is designed here ([Dirty source on rung 1](#project-decisions)).
> Add steps 4 and 5 verify the frozen
> content and fill the clone but restore nothing yet ([Add and Git registration](#add-and-git-registration)).

The answer the item carries is the same file,
"Dirty source on rung 1" (`:1445` to `:1452`):

> - Dirty source on rung 1.
> Decided:
> "Verified restore (Recommended)",
> a verified dirty-main restore comparable to
> Btrfs.
> The unselected clean-only route would have narrowed the proof but lost clone speed on a dirty main
> checkout,
> and the main checkout here usually holds other sessions' uncommitted edits.
> Dirty support needs a
> defined Git/index and ignored-state restore,
> with a frozen proof,
> and
> neither route accepts an unproved source.
> [VM item 1][vm-register] ran `git read-tree HEAD` then reported empty
> tracked status:
> Git saw the fixture's tracked content matching `HEAD`.
> It did not capture the source index and
> tree before the snapshot or compare that proof to frozen content.
> That comparison,
> conversion checks,
> dirty
> restoration and concurrent writes remain unmeasured.

The plan lists the item as step 11's blocker,
"step 11,
blocking,
writer design work" (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:2836` to `:2839`),
and until step 11 lands an add from a dirty main checkout takes rung 3,
by the answer "Ship step 10 first (Recommended)" (`:1463` to `:1468`).

## Decisions and records that bind this design

Each bullet quotes the lines that decide a point,
with path and line.

- The restore's promise:
`package/git-policy/cli/SPEC.md:1253` says "the new worktree is restored to what `git worktree add` makes from a clean checkout".
- The refusals that survive a dirty source:
`SPEC.md:1264` to `:1282`,
"an operation in progress or a held `index.lock`;
any unmerged,
intent-to-add,
skip-worktree or assume-unchanged entry,
or a submodule,
found anywhere in the index",
"a `.gitattributes` file among the changes or untracked anywhere",
whose listing "runs real Git with `--no-literal-pathspecs`" (`:1274` to `:1276`),
"a sparse checkout",
"a source that is a linked worktree",
and "the clone rung,
which is only for clean sources".
- The fields an installed entry may carry:
`SPEC.md:1289` to `:1293`,
"An entry no walk proved keeps the fields its own index recorded,
never fields taken from the file:
fields of the file itself would vouch for its bytes,
and Git would take an edit for the entry's content."
- The restore's runs:
`SPEC.md:1294` to `:1307`,
the three runs in order and "These three runs and nothing else are run for the restore."
(`:1355`).
- The settle's belief:
`SPEC.md:1316` to `:1331`,
the files the stat data cannot vouch for are read and compared with the blobs their entries name,
and the main checkout is read to be snapshotted and listed (`:1381`),
"never written" (`:1382` to `:1384`).
- The left-out line:
`SPEC.md:1386` to `:1393`.
- Writes during the snapshot are out of scope for the btrfs rung:
`SPEC.md:1396` to `:1399`.
- The ignored state's instant:
`SPEC.md:1518`,
"The ignored files the new worktree carries are the source's as of the instant this add's snapshot was taken".
- The records an add writes for itself:
`SPEC.md:4408` to `:4410`,
"Each is a private file created exclusively beside its final name and renamed over it,
so a reader sees the whole record or none,
and the filesystem makes it durable with its next commit."
- The wrapper never asks for a secret:
`SPEC.md:7345` to `:7352`,
"No command the wrapper runs or forwards asks a person for a credential",
with "Avoid ever asking for someone to type a username or password."
and "Passphrases count too (Recommended)".
- The walk and clean bound:
`doc/planning/cli-git-zfs-worktree-design-proposal.md:1073` to `:1083`,
"never hand `git clean` a path under `.claude/worktrees` or force cleanup across a mount.
A mount or a tree containing one is a refusal for destructive cleanup,
not an ordinary ignored directory",
and "double-force dry runs also show why ordinary nested-repository protection is insufficient".
- The omitted-mount refusal:
the same file `:1036` to `:1037`,
"If tracked output would occupy an omitted mount location,
refuse the clone fill rather than writing through a boundary",
and `:403` to `:404`,
"A snapshot omits nested mount contents but can carry real underlay files and stale Git stubs;
remove only the verified clone-local copies,
never the live mounts or source underlays."
- No fallback over an unresolved state:
the same file `:427` to `:431`,
"If snapshot or clone returns `ENOSPC`,
report the operation,
object and pool,
keep the claim,
and clean up only objects whose creation and identity are proven and whose destruction is allowed.
A clone pins its origin even before it is mounted.
If mutation succeeded but its identity was not recorded before interruption,
keep and report it unless identity can be independently recovered.
No silent fallback over an unresolved managed state.
An explicitly recovered Git-only add may finish at the same slot only after the mount and all outstanding intents are settled."
- The add's steps 4 and 5:
the same file `:396` to `:410`,
including "Compare the frozen index and tracked content in the clone itself against the captured proof,
without running Git against that directory",
"Discard copied Git administration before running Git inside the clone",
"Install the saved Git pointer inside the clone",
and "A mount can be externally visible before the fill;
an interrupted add can expose copied `.git`,
which is neither the linked registration nor a success signal."
- The preflight capture:
the same file `:370`,
"capture the index/tree verification inputs".
- No configuration:
`doc/decision/cli-git-zfs-step-8-sweep-choices.md:19` to `:45`,
the project "first answered that everything must work by default with no configuration",
and for no file at all,
"1:
no file needed (Recommended)".
- No lock for a per-item record:
`AGENTS.md` rule `NLK` at `:1149`,
"Concurrency is lock-free first (atomic rename,
`O_EXCL`,
one owner per item);
an unavoidable lock covers the smallest unit,
never a whole repository,
with its reason written beside it."
- The store precedent:
`doc/decision/cli-git-zfs-slot-identity-store-format.md:132` to `:175`,
"the record is created with `O_EXCL` under a hidden staging name,
linked to its real name in one `linkat` (whose `EEXIST` decides the one winner),
and the staging name is then removed",
and its "Who removes a killed writer's staging file" places that removal in `doctor` at plan step 18.
- Shipped limits cite a measurement:
`AGENTS.md` rule `LQM` at `:1567`,
"Shipped resource limits (unit caps,
timeouts,
size bounds) cite a workload measurement or owner decision beside the value".
This design ships no duration and no bound it has not measured,
and names the measurement that would size each.
- Worktrees live under `.claude/worktrees/` of the main checkout (`AGENTS.md` rule `WTU` at `:758`),
and every path comes from the injected home (rule `HOM` at `:1040`).
- The threat model:
`doc/decision/cli-git-zfs-staging-under-cache.md:221` to `:237`,
"the account is trusted,
and root and the account are not a security boundary here",
from the answer "We have to assume my machine isn't compromised.
sudo is passwordless here anyway."
"Hardening the stated threat model may make unnecessary" places each check against it.
- Noisy comparisons and the reason to repeat them:
`AGENTS.md` rule `QNB` at `:362`,
and the machine's normal pressure in rule `NQH` at `:850`.
- The conversion fence:
`doc/planning/cli-git-clone-rung-conversion-fence.md:1660` to `:1672`,
"no path is recorded clean before its bytes are Git's checkout's",
and its limitations at `:3607` and after.
- The other rungs' dirty-source answers:
`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` and `doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`.
Neither decides anything about rung 1's clone of a snapshot.

## Where the records differ, and what this design does about it

- A write during the snapshot is out of scope for the btrfs rung (`SPEC.md:1396` to `:1399`),
and the answer this design serves requires a proof that refuses such a write.
The btrfs rung's sentence stays true of the btrfs rung,
rung 1 refuses what its own proof catches,
and neither claims to defend against a write after the snapshot's transaction group (L3).
- The plan's acceptance wording is narrower than this design:
"Cover recognized and rejected forms,
dirty sources and a tracked write between preflight and snapshot that the frozen proof refuses" (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:1456` to `:1459`,
repeated at `:1614` to `:1617`).
This design takes its frozen capture immediately before the snapshot and re-runs every refusal against it,
so a write between preflight and that capture is handled by the restore rather than refused by the proof.
That is a departure from the plan's sentence,
and Q-C puts it to the project;
taking Q-C's option (c) would remove the departure.
- "The clone rung,
which is only for clean sources" (`SPEC.md:1281` to `:1282`) is the ladder's rung 2,
the same-dataset reflink,
while the proposal's rung 1 is the snapshot clone and the plan's step 3 speaks of "the clone rung" for rung 2.
This document says "rung 1" or "the ZFS snapshot clone" for its own subject and "rung 2" or "the ladder's clone rung" for the other.
- The word "proof" names four different things across these records:
the clean check's stat proof,
the rung 2 proof that a source matches its commit,
the fence's size proof,
and this document's comparison of a clone with a capture.
"Frozen proof" here means D1 to D3 and nothing else.
- Issue 6323 is the conversion defect (`SPEC.md:2736` to `:2738`,
"a clone keeps the source's bytes where the checkout would convert line endings or `ident`"),
and the handover records that it "closes with" the fence's merge (`doc/handover/cli-git-rust-implementation.md:41374`),
while the plan's step 10 and step 11 notes still say they are "also blocked by issue 6323" (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:1499` and `:1628`).
This design takes the handover's reading:
the fence is on `main`,
rung 1 applies it,
and the guest controls T8 are what a dirty restore to a non-`HEAD` commit owes.
It is not 6323's class that a same-size,
same-tick write leaves behind (L4,
V4).
- The step 8 sweep record answers that everything must work with no configuration,
and the handler still reads a root-owned file when one exists.
Nothing in this design reads or writes a configuration file.
- [VM item 1][vm-register] moved the copied administration aside rather than deleting it,
and measured no deletion cost.
This design takes the measured route and owes the deletion's cost to [M3][m3].

## Facts

Each fact names its source,
the lines read,
and,
where it is a measurement,
who ran it and what it showed.
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
`statinfo.c:24` to `:35` (`fill_stat_data`).
- F2.
Git's own stat comparison reads the seconds of the modification time and the change time (`statinfo.c:69` to `:73`),
the owner,
group and inode and the size (`:83` to `:89`),
with `core.checkStat` and `core.trustctime` on by default (`environment.c:776` to `:777`).
The sub-second parts are compared only under `USE_NSEC` (`statinfo.c:75` to `:81`),
which is opt-in (`Makefile:200` to `:205`),
and the device only under `USE_STDEV` (`statinfo.c:91` to `:99`),
which is opt-in too (`Makefile:213` to `:214`).
The installed Git `2.56.1` compares seconds:
a change below the second was not listed while a change of one whole second was,
as F23 records.
- F3.
`git read-tree --reset -u <commit>` rewrites only the paths whose file Git's own stat check can see changed.
`--reset` is `UNPACK_RESET_OVERWRITE_UNTRACKED` (`builtin/read-tree.c:186`);
an entry under a reset never refuses for being out of date (`unpack-trees.c:2238` to `:2241`);
and the update flag is set only where `lstat` fails or the entry's stat data differs from the file (`unpack-trees.c:3036` to `:3044`).
- F4.
`git clean --force -d` removes the untracked files and the untracked directories that no ignore rule covers (`builtin/clean.c:1071`) and refuses without `--force` when `clean.requireForce` is set (`builtin/clean.c:957`).
It keeps a directory that holds its own repository:
`rm_flags` starts as `REMOVE_DIR_KEEP_NESTED_GIT` (`builtin/clean.c:927`),
is cleared only when `--force` is given twice (`:959` to `:960`),
and `remove_dirs` leaves a directory that `is_nonbare_repository_dir` names (`:176` to `:186`).
- F5.
`git checkout-index --force -u --stdin` writes the named paths out of the index through Git's own conversion and records each written path's stat data:
`builtin/checkout-index.c:101`,
`:178`.
- F6.
`git status` refreshes the index and writes it back unless optional locks are off:
`builtin/commit.c:1634` takes the lock only when `use_optional_locks()` is true,
`:1658` writes the index when it did,
and `environment.c:228` to `:231` reads `GIT_OPTIONAL_LOCKS`,
which `--no-optional-locks` clears.
- F7.
`git worktree add` writes `<path>/.git` as a file holding `gitdir: <administrative directory>` and writes the reciprocal `<administrative directory>/gitdir`:
`builtin/worktree.c:532` to `:544`,
through `write_worktree_linking_files` at `worktree.c:1105` to `:1134`.
With `--no-checkout` nothing else is written into the worktree.
- F8.
An index can be written under a private name and published afterwards,
and one already exists that does so:
`worktree_clone_index.rs:111` to `:145` (`PendingIndex`,
whose comment says "This is the moment the worktree starts vouching for its files;
before it,
no index exists there at all") and `subvolume_index_install_into_new_worktree.rs:190` to `:210` (`publish_index_under_a_name`,
one rename that replaces nothing).
- F9.
The restore's own Git runs name their Git directory and work tree through the environment,
and take an index file when one is named:
`worktree_ladder_target.rs:418` to `:440` (`checkout_environment` and `checkout_environment_with_index`).

### OpenZFS, at `zfs-2.4.4`

- F10.
A clone's initial object set is its origin snapshot's block pointer:
`module/zfs/dsl_dataset.c:1225` to `:1227` copies the origin snapshot's block pointer into the new clone,
`dsphys->ds_bp = dsl_dataset_phys(origin)->ds_bp`,
when the clone's dataset object is created.
- F11.
A snapshot's block pointer is the dataset's own at the transaction group being synced (`module/zfs/dsl_dataset.c:1782` to `:1784`),
and a second snapshot of the same transaction group is retried rather than taken (`:1534` to `:1539`,
an `EAGAIN` the caller retries).
- F12.
A ZPL inode number is the object number:
`module/os/linux/zfs/zfs_znode_os.c:600` sets `ip->i_ino = zp->z_id`,
and `:546` takes `z_id` from the dnode's object number.
With F10,
a clone's files carry the snapshot's inode numbers.
- F13.
A snapshot that is a branch point cannot be destroyed:
`module/zfs/dsl_destroy.c:55` to `:89` refuses with `EEXIST` when the snapshot is a branch point,
which a live clone makes it.
The measurement record confirms the refusal for a snapshot with a dependent clone (`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
"Item 2:
one named snapshot has a dependent clone").
- F14.
The ZPL keeps each of the access,
modification and change times as a sixteen-byte value,
seconds and nanoseconds:
`module/zfs/sa.c:1704` to `:1706`.
The nanoseconds are the coarse clock's,
not the write's own (F15).
- F15.
A ZFS write stamps the modification time and the change time from `gethrestime`,
which is the coarse clock:
`include/os/linux/spl/sys/time.h:68` to `:72` (`ktime_get_coarse_real_ts64`),
`module/os/linux/zfs/zfs_znode_os.c:1420` to `:1446` (`zfs_tstamp_update_setup`),
called from the write path at `module/zfs/zfs_vnops.c:1013`.
Two back-to-back rewrites can therefore carry one stamp,
as F22 measured.
- F16.
ZFS takes a change time from the VFS when the VFS sets attributes (`module/os/linux/zfs/zfs_vnops_os.c:2546` to `:2549`) and it does not declare `FS_MGTIME` (`module/os/linux/zfs/zpl_super.c:1092` to `:1099`),
so the kernel does not give the next write a fine-grained stamp after a `stat` on it (F21).
- F17.
The clone's copied `.git` is a directory,
and the nested copy of the slot holds Git's own pointer file:
measured in the guest by [VM item 1][vm-register] (`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md:56`),
"The clone's root had a `.git` directory;
the nested copy of the slot had the exact underlay pointer,
proving the snapshot included the registration."
- F18.
A snapshot of the checkout dataset omits a nested mount's contents but can carry the underlay's real files and stale Git stubs (`doc/planning/cli-git-zfs-worktree-design-proposal.md:403` to `:404`),
so a nested worktree's copy in the clone may not be an empty directory.

### Linux, at `v7.3`

- F19.
Every write sets the change time to the present unless its caller passes `ATTR_CTIME_SET` (`fs/attr.c:465` to `:478`),
and a buffered write reaches it through `file_update_time` (`fs/inode.c:2501`,
`mm/filemap.c:4472`).
- F20.
No userspace call selects the change time:
`utimensat` passes `ATTR_CTIME`,
but only `ATTR_ATIME_SET` or `ATTR_MTIME_SET` for the values it was given (`fs/utimes.c:41` to `:55`).
- F21.
A file system that declares multigrain timestamps gets a fine-grained stamp on the next write after someone queried the time;
one that does not,
keeps the coarse stamp:
`fs/inode.c:2425` to `:2446` and `:2894` to `:2912`.
btrfs declares `FS_MGTIME` (`fs/btrfs/super.c:2232` to `:2233`);
ZFS does not (F16).

### This host, measured

- F22.
The reviewer rewrote one four-byte file 400 times with no wait:
385 of the 399 consecutive pairs kept both the change time and the modification time although the bytes changed,
the stamps moved in steps of one millisecond,
and the 14 pairs that moved are the probe's own positive control (`~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/probe-logs.md`,
"P1",
on `zpcachyos/ROOT/cos/home`).
A same-size rewrite can therefore leave every compared field of F2 and of the wrapper's settle equal.
- F23.
The reviewer moved only the sub-second part of a file's modification time and change time in one test arm and a whole second in a control arm:
`git diff-files --name-only` listed the control's file and not the test's,
so the installed Git compares seconds (probe logs,
"P2",
with the control stated).
- F24.
The reviewer's dry run of `git clean --dry-run -d` in a worktree holding an untracked directory with its own repository named the plain untracked file and not the directory,
while the listing the restore's proof makes (`ls-files --others --exclude-standard --directory`) named it (probe logs,
"P3").
The real removal was not run;
its behaviour is F4.
- F25.
This round's first text measured the main checkout at 24,685 tracked paths,
an index of 3,436,101 bytes,
and 764,137,159 bytes of tracked content;
the reviewer's later read gave 24,685 paths and 3,436,657 bytes of index,
the difference being the checkout's own churn (its index was rewritten at 17:45:57,
probe logs "P4").
The same text measured the listing of the tracked names at 25.6,
40.9 and 29.4 ms,
one `lstat` per path at 175.7,
111.3 and 134.3 ms,
reading the index at 1.4,
1.4 and 1.5 ms,
and a SHA-256 of every tracked file at 9,380.9,
4,852.3 and 5,218.4 ms,
in scripts in this round's scratch folder,
not in the wrapper.
- F26.
This round's first text measured the main checkout's common Git directory at 2,732,668,640 bytes in 634,426 files and 6,708 directories,
its ignored state at 214,899,299,440 bytes under `.claude`,
2,745,349,203 under `package` and 185,182,920 under `node_modules`,
and its status at 5 changed paths.

### The wrapper, in this tree

- F27.
The btrfs rung's dirty restore exists and is reused:
`restore_dirty` (`package/git-policy/cli/src/native/worktree/worktree_dirty_restore.rs:465`) lists the carried ignored paths,
runs `git clean --force -d` captured into the journal (`:478` to `:492`),
removes the files the reset cannot see,
runs `git read-tree --reset -u <commit>`,
and proves the result with `prove_restored` (`:292`,
whose untracked listing is at `:297`).
- F28.
The index install serves a dirty source by keeping the seed's fields:
`install_index` (`worktree_snapshot_index_step.rs:196`) takes `own_index: Option<&[u8]>` and,
for a dirty source,
fills no lookup,
so "An entry the walk did not cover ...
keeps the stat fields the index it came from recorded" (`:268` to `:270`);
`WalkedStats` (`:305` to `:310`) answers nothing for an entry the walk did not cover.
`read_own_index` (`:327`) opens `<copy>/.git/index` below a held root without following a link.
- F29.
The one existing rewrite that takes fields from a file writes the change time too:
when a lookup answers,
the rewrite writes the file's change seconds and nanoseconds,
device,
inode,
owner and group into every entry whose recorded size and modification time already equal the file's (`package/git-policy/cli/src/native/subvolume/subvolume_index.rs:520` to `:570`,
the field list at `:554`),
trusted by `AcceptEveryStat` (`subvolume_index_install_into_new_worktree.rs:116` to `:121`).
There is no mode that writes the device and inode alone.
- F30.
The wrapper's settle does not read a file whose recorded stat data is its own to the nanosecond,
"even when Git calls the entry racy" (`worktree_clean_tree_walk.rs:199` to `:205`,
`:248`).
- F31.
`paths_whose_content_differs` is defined at `worktree_clean_tree_walk.rs:223` and re-exported at `worktree_clean.rs:49`.
- F32.
The btrfs rung hands the copied `.git` to a trash inside the copy in one rename and lets the disposal empty it after the command returns:
`worktree_snapshot.rs:284` with `hand_over.take_entry`.
- F33.
The btrfs rung's fallback completes the worktree with `git reset --hard` in the destination (`worktree_ladder_finish.rs:324`),
and the clone rung's failure sentence is `the clone rung could not finish (<reason>), and Git's own checkout made the worktree.` (`worktree_slow_route.rs:278`;
`SPEC.md:2787`).
- F34.
A split or sparse index is declined by the rewrite,
with the reason `the index uses the <signature> extension (split or sparse index)` (`subvolume_index.rs`,
the `DECLINED` list and its message),
so an index that carries one cannot be installed.
- F35.
The copy journal's folder sweep reads only the names that end in its journal suffix (`worktree_copy_journal.rs:604`),
so a record in another folder is not read by it.

## Assumptions

- A1.
A mounted clone reports the snapshot's own metadata,
because its object set is the snapshot's (F10):
the inode number,
size,
kind,
mode,
and the recorded times of every unchanged file equal the source's at the snapshot.
[M2][m2] measures it.
- A2.
Other sessions write the main checkout through ordinary writes,
editors and Git commands,
so a write between the capture and the snapshot is a VFS write (F19) that lands in some millisecond tick (F15).
A writer that sets the system clock back,
or that writes through a raw device,
is outside the threat model this design designs to.
- A3.
The wrapper's index reader and its stat comparison (F28,
F30) read an index written by Git `2.56.1` and a file on the clone as they read them on btrfs,
so their verdicts carry over.
- A4.
The clone-local trash can be emptied after the add returns by the same disposal that empties a btrfs snapshot's trash,
with the clone mounted at the destination (F32).
- A5.
The capture's cost in the wrapper is the order of the rung 2 proof's own walk,
34 ms over 24,313 tracked files in `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md:57`,
rather than the node probe's F25 figures,
because the probe pays for a scripting runtime per call.
[M1][m1] measures the wrapper's own arms,
and no figure is claimed here.

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
never prevented,
and a write after the snapshot's transaction group is a write in the main checkout,
not in the clone;
the design does not catch it,
which is the risk `SPEC.md:1396` to `:1399` accepts for every rung.
- L4.
The proof's detection rests on a one-millisecond tick (F15,
F22) and on Git's seconds (F2,
F23).
A tracked write that lands in the same tick as the file's own last write leaves every compared field equal;
the design refuses it by digesting the racy set (C4),
and the residual is the same coincidence before the capture,
which the settle cannot see on ZFS as it can on btrfs (F21,
F30,
V4).
The exposure needs the index write,
the file's last write,
this add's `lstat` and the other session's write inside one millisecond;
its size is not measured.
- L5.
The capture's cost figures come from scratch scripts and from another round's record (F25,
A5);
neither is the rung's own timer.
- L6.
The proof needs the clone's copied `.git/index`,
or the record's proof phase.
A resume that finds the copied index gone and no proof phase cannot prove the frozen clone from the capture alone,
because the capture holds the index's digest and the files' facts,
not the files.
- L7.
The capture's size is an estimate:
1,648,834 bytes of path names and a per-line field span,
so about 2.5 MB for this repository's 24,685 paths,
larger than the first text's 2.2 MB because it now carries the kind,
the execute bit and an absent row per deleted path.

## The parts that need a design

### What "the captured source" is

The captured source is read from the live main checkout by the wrapper itself,
with held handles and no Git run of its own;
the only Git runs the capture needs are the untracked listings the refusal set below makes,
which the wrapper already runs for the btrfs rung:

- C1.
The index:
the bytes of the checkout's own index file,
`<checkout>/.git/index`,
with its size,
its modification time and its SHA-256 digest.
The whole refusal set below reads these bytes.
- C2.
Every path in that index:
its kind (regular file,
symbolic link,
or absent),
its size,
its modification time and its change time to the nanosecond,
and the owner's execute bit;
for a symbolic link also the bytes of its target.
A tracked path the source has deleted is recorded as absent,
so a write in the window that brings it back is comparable.
A path whose kind is neither a regular file nor a symbolic link is refused before the snapshot,
as a kind no reset can undo.
- C3.
A SHA-256 of the content,
or of the link's target bytes,
for every path whose index stat data cannot vouch for the file,
judged by the wrapper's existing predicate (F31).
- C4.
A SHA-256 of the content of every path in the racy set:
the entries whose recorded modification time or change time is not at least one tick older than C1's index modification time.
On this host's ZFS that is the set a same-tick write can hide in (L4).
The set is small on the checkout as it commonly stands;
[M7][m7] counts it.
- C5.
The commit the caller's add checks out,
read from the new worktree's `HEAD` file and the ref it names after Git's registration,
so the record names what the restore will write.
The source's own `HEAD` is recorded beside it,
because the fence's tree reads were made against it,
and a ref moved in the window is reported rather than refused:
the commit the restore writes is the one Git registered.
- The refusal set of `SPEC.md:1264` to `:1282` is re-run against these bytes and against the source immediately before the capture is written:
the index-entry kinds from C1's bytes,
an operation in progress or a held `index.lock`,
the untracked `.gitattributes` listing (`git ls-files --others --exclude-standard -- ':!**'`-shaped,
the wrapper's own listing with `:(glob)**/.gitattributes`,
`SPEC.md:1274` to `:1276`),
a sparse checkout,
and a linked source.
A refusal here leaves no managed object.

It is taken twice,
and the two readings are different objects:

- the preflight reading,
at plan step 10's step 1,
whose index bytes decide the cheap refusals;
- the frozen capture,
taken immediately before the snapshot command in step 3,
after the fence's reads and after Git's registration,
with the whole refusal set re-run against it.
It is written into the add's record before the snapshot runs,
and it is the one the proof compares.

The record,
its format and its end:

- Home:
`<common>/monochromatic-git-policy-cli/add-capture/v1/<slot>.capture`,
following the wrapper's record convention for its folder,
one family folder and a grammar version,
the slot as the key,
the account's `0700` folder and `0600` file.
It is not the binding record (that design declined to carry the proof) and not the copy journal's folder (whose sweep reads only its own suffix,
F35).
- Format:
a header line `capture=1`,
the repository's common directory as bytes,
the source root as bytes,
the index's size,
modification time and digest,
the captured source `HEAD`,
the checkout's Git-directory identity,
the entry count,
and a phase word (`captured`,
or `proof=passed`);
then one line per entry,
the path as lowercase hexadecimal and its kind,
size,
both times to the nanosecond and execute bit,
absent entries included;
then one line per digest of C3 and C4.
A reader refuses an unknown version,
a line it cannot parse,
or a count that disagrees with the lines.
- Publication:
an exclusive staging name `<slot>.capture.pending`,
the whole file written,
then one link to the real name that replaces nothing,
then the staging name removed,
which is the store precedent of this repository.
- Size bound:
one line per index entry,
its length bounded by the path the index holds plus a fixed field span,
so the file's bound is the index's own entry count times that span;
no separate total is shipped,
and the shipped span is computed from the index's own bound rather than measured.
The estimate for this repository is L7.
- End:
it is removed when the binding is marked usable,
and on every exit that leaves no resumable add:
a refusal that settles its objects,
a fallback that completes with Git's own checkout,
or a state in which the proof can no longer pass.
It stays while the add is resumable,
which is what a resume proves from.
`doctor` reports and removes a `<slot>.capture.pending` a killed writer left,
as it does for the slot identity store's staging names.

### The frozen proof

The proof runs in the mounted clone after the add's step 4 has verified the mount's identity and the caller's visibility,
and before any byte of the clone is written.
It reads the clone's files below a held handle of the clone's root,
following no link,
and it runs no Git against the clone's copied administration:
a Git run there would read the copied index as its own,
rewrite it,
and could move the copied refs of the checkout.
So the copied index is read directly,
by the wrapper's own reader (F28).

The proof compares,
in this order:

- D1.
The bytes of the clone's copied index,
`<clone>/.git/index`,
against C1:
same length and the same SHA-256,
byte for byte,
with no separate comparison of the entry set,
because equal bytes are equal entries and the entry kinds were judged by the refusal set on these same bytes.
This catches any write to the index between the capture and the snapshot,
which is what a concurrent `git add`,
`git reset`,
`git commit`,
`git stash` or a merge in the main checkout does.
A `git status` in the main checkout that refreshes stat data also rewrites the index and refuses the add,
which is the cost V8 weighs.
- D2.
For every path in the captured index,
the clone's own facts against C2:
the kind,
the size,
the modification time to the nanosecond,
the change time to the nanosecond,
the execute bit,
and the target bytes of a symbolic link.
The device and the inode are not compared,
because a clone is another dataset and F12 and [M2][m2] say its inode numbers are the snapshot's own.
- D3.
For every path C3 recorded a digest for,
the clone file's SHA-256 against it.
- D4.
For every path C4 recorded a digest for,
the clone file's SHA-256 against it.
This is what refuses a write that lands in the same tick as the file's own last write,
the case F22 measures and L4 sizes.

A difference in any of them refuses the clone rung.
The refusal's reason names the path and the field,
and the person reads the one failure sentence (F33) with that reason in it.

How a write between the capture and the snapshot is caught:

- a write to a tracked file in a later tick moves its change time past every compared field of F2 and D2,
so D2 refuses and names it;
- a write in the file's own tick leaves both times and the size equal (F22),
and D4 refuses it because C4 digested that path;
- a write that replaces a file changes its inode,
which D2 does not compare,
but the replacement's change time and often its size differ,
and a replacement inside the same tick is D4's case too;
- a staged change or a commit in the window moves the index,
which D1 catches before D2 runs;
- a write whose bytes and every compared field are equal after the capture is D4's case for the racy set and is either a write before the capture (L4's residual) or nothing to catch.

The proof is not a claim about ignored files or untracked files.
Those are carried exactly as the snapshot carried them,
which is the rung's purpose,
and the restore's own proof is what proves the finished tree.

### The restore

The restore turns the clone into Git's own checkout of the commit the caller asked for,
in nine steps.
The order is this revision's answer to H2:
nothing under a real name is published before the finished tree is proven.

1. Verify the mount and the caller's visibility,
which is the add's step 4 and not this design's.
2. Read the clone's own index bytes with `read_own_index` (F28) and run D1 to D4 before any write (see "The frozen proof").
3. Write the proof's phase into the capture record (`proof=passed`),
so a resume after the first write does not have to prove a tree that has already been written (L6,
"What a resume finds").
4. Remove the clone-local leave-outs with the btrfs rung's own rule:
the copies of other adds' trash,
the copies of nested worktree stubs,
and the destination's own slot directory.
Each is a removal inside the clone's own dataset.
A stub that holds more than an empty directory or Git's pointer file is a refusal that ends the rung,
because the snapshot may have carried the underlay's real files (F18) and the design removes only the copies it can name (`doc/planning/cli-git-zfs-worktree-design-proposal.md:403` to `:404`).
5. Write the captured index under the private name `index.pending` in the new worktree's administrative directory,
with a lookup that answers nothing for every path,
so each entry ends with these fields and no others:
the path,
the mode,
the object name,
and the flags and stage from the captured index;
the change time,
the modification time,
the device,
the inode,
the owner,
the group and the size as that same entry recorded them;
zeroes in every stat field of a path the conversion fence hands to Git;
the `UNTR` and `FSMN` extensions dropped;
and a recomputed checksum.
No field is taken from any file,
which is `SPEC.md:1289` to `:1293` and the reason H1 was raised.
No index exists under the worktree's own name yet.
If [M2][m2] shows that a clone's inode numbers are not the snapshot's,
the install would need a field mask the wrapper does not have:
a rewrite that writes the clone's own device and inode into an entry only when its recorded size,
modification time and change time already equal the file's,
which the proof established,
and never its change time.
That is a change to the shared rewrite,
and [M2][m2]'s answer is what would put it to the project.
6. Run the btrfs rung's own `restore_dirty` with the clone's root as its stage and with `GIT_DIR`,
`GIT_WORK_TREE` and `GIT_INDEX_FILE` naming the private index (F9):
list the ignored paths,
run `git clean --force -d` (untracked files that no ignore rule covers go,
ignored files stay),
settle by content the entries the installed index cannot vouch for,
run `git read-tree --reset -u <the commit the caller asked for>`,
and prove the result with `prove_restored`.
`restore_dirty`'s request environment gains the index parameter that `checkout_environment_with_index` already builds;
that is a change to a shared function,
named in "What it reuses,
and where ZFS differs".
7. Write the fence's paths through Git with `git checkout-index --force -u --stdin` (F5) against the same private index,
so no path is recorded clean while it holds the source's unconverted bytes.
8. Publish,
in this order:
move the copied `.git` into the clone-local trash in one rename (F32);
publish the private index under the real name with one rename that replaces nothing (F8);
write the pointer through a private name in the administrative directory and one rename into `<clone>/.git`,
so the slot lacks a `.git` for the gap between two renames.
Then verify the worktree root,
the common directory,
the reciprocal registration,
the mount identity and the checked-out state.
9. Mark the binding usable,
queue the dependency repair,
hand the trash to the disposal,
remove the capture record,
and print Git's line,
the line about the left-out changes,
and nothing else.

Why this order,
and what a command run in the slot sees at each moment,
is Q-D's subject.
Until step 8 the slot holds the copied `.git` directory,
so a `git status` run there answers from a mirror of the main checkout,
and a mutating command acts on that copy and is lost with it.
From the first rename of step 8 to the second the slot holds no `.git`,
so a Git run there finds the main checkout's repository above it.
Either is a state without the bad property H2 named,
a valid pointer over an unrestored tree;
the third option Q-D lists is that state and is refused.

What the restore guarantees,
and its exceptions,
are listed under "Where the result differs from Git's own checkout".

### What it reuses, and where ZFS differs from btrfs

Reused whole,
without a change:

- the refusal set for a dirty source (`SPEC.md:1264` to `:1282`),
re-run against the frozen capture;
- the restore's three runs,
its content settle and its proof (`restore_dirty`,
`prove_restored`,
F27);
- the index install's dirty form,
which keeps the seed's fields and takes none from a file (F28),
and the copy's own index read (`read_own_index`);
- the leave-out rule and the trash hand-over of the snapshot rung (`prune_snapshot`,
`remove_stub`,
`replace_git_pointer`,
F32),
and the disposal that empties a trash after the command returns;
- the conversion fence,
decided before the snapshot and written through Git after the reset,
with the index's zeroed stat data for the fenced paths (F5,
F28);
- the clone rung's failure sentence and the fallback that Git's own checkout completes (F33);
- the index's private name and its publication (F8).

Changed rather than reused:

- `restore_dirty`'s Git runs gain `GIT_INDEX_FILE`,
a parameter `checkout_environment_with_index` already carries (F9),
and the rung calls the runs with the clone's root as their work tree.
- The install is called through the private-name writer (`write_index_under_a_name`) with a lookup that answers nothing,
then published (F8).
`install_index` itself is unchanged and is not the function this rung calls.
- No device-and-inode rewrite exists and none is added;
[M2][m2] decides whether one is needed,
and "Choices this design marks for the project's veto" carries the contingency.

Where ZFS differs,
and what the design does about each:

- There is no stage and no exchange,
so the fill is visible while it runs and the binding,
not the mount,
says whether the worktree is usable.
Q-D asks what the slot may look like meanwhile.
- The copied administration is on another dataset from the source's trash,
so the hand-over is a rename inside the clone and the disposal reaches it through the published worktree path.
- The inode numbers are shared with the snapshot (F10,
F12) and the device is not;
the proof does not compare the device,
and the index keeps the source's inode until [M2][m2] shows it differs.
- A snapshot holds an empty directory where a nested dataset is mounted in the source,
or the underlay's real files (F18),
so the leave-out refuses a stub it cannot name rather than deleting what it did not make.
- The change time is a coarse tick and Git compares seconds (F2,
F15,
F22,
F23),
so the proof digests the racy set (C4) and the settle's own trust is weaker than btrfs's (F21,
F30,
V4).
- The clone's creation is two delegated requests whose GUIDs and `createtxg` values are recorded,
and an origin snapshot cannot be destroyed while its clone lives (F13),
which fixes the order of any cleanup.
- A ZFS snapshot copies nothing,
so the fence's reads sit on the add's waiting path,
as in the btrfs snapshot rung.

### What happens when the proof fails or the restore cannot finish

The accepted rule this section answers is quoted whole in "Decisions and records that bind this design" (`doc/planning/cli-git-zfs-worktree-design-proposal.md:427` to `:431`):
"No silent fallback over an unresolved managed state.
An explicitly recovered Git-only add may finish at the same slot only after the mount and all outstanding intents are settled."
Its literal reading is that an add whose clone is mounted and refused leaves the state and a person recovers it,
and that is what this design does until the project answers Q-A.
Q-A offers the settlement as its first-ranked option and says what the design becomes if it is taken.

Before the clone is mounted,
a failure destroys only what this invocation made,
whose identity it recorded,
and Git's own checkout completes the worktree at the same slot in the same invocation:

- a refusal before the snapshot (a refusal the btrfs rung already makes,
a refused kind,
a fence that cannot decide) leaves no managed object,
and the rung ends as the btrfs rung's does;
- a refused or failed snapshot or clone request (including `ENOSPC`) keeps the claim,
destroys the clone if it exists and the snapshot after it,
because a snapshot with a dependent clone cannot be destroyed first (F13),
releases the slot and lets Git's own checkout finish it;
- an identity that cannot be proven,
or a mount that does not arrive within its bound,
leaves the clone and its snapshot in place and refuses,
as the `ENOSPC` clause requires:
destroying an object whose identity was not proven is what the removal proof forbids.
`doctor`'s unaccounted origin line reports a snapshot left this way,
and a removal settles it.

After the clone is mounted,
a failure of the proof or of a restore step leaves the pending state:

- the add writes its record's pending state,
keeps the capture record,
prints the failure sentence with the reason and the removal command from the wrapper's existing unfinished-worktree report,
and changes nothing about the clone or its snapshot;
- the person leaves the folder,
runs the removal command,
and adds again,
which then takes whatever rung the ladder chooses;
- the design does not retry:
Q-A's option (c) is the one that would.

The lines the person reads,
in the wrapper's voice,
and what the journal keeps:

- the clone rung's existing sentence,
with the proof's reason in it:
`the clone rung could not finish (<reason>), and Git's own checkout made the worktree.` for a failure before the mount,
and the same reason inside the unfinished worktree's sentence for one after it;
- the reason's own words name the path and the field,
for example `the frozen proof refused AGENTS.md: its change time changed in the main checkout while the add read it`;
- on success,
the line Git's own checkout prints and the line about the left-out changes (F33),
and nothing about the proof.

The journal keeps one entry with the destination,
the rung,
the kind of refusal,
the path,
the field and the capture record's name,
in the shape the left-out note already has (`CLI_GIT_WORKTREE`,
`CLI_GIT_DIRTY_PATHS`,
`CLI_GIT_DIRTY_FIRST`,
`worktree_dirty_restore.rs:104` to `:113`),
so a later reading can tell which field of which path refused without the person's screen.

### What a resume finds

The restore is idempotent from step 4 on:
the leave-out removes what is there,
`git clean --force -d` removes what is untracked,
the settle reads and removes,
`git read-tree --reset -u` rewrites what differs,
and the fence's write rewrites.
So a resume re-runs the restore whole rather than tracking a step,
and the only fact it needs is the proof's phase,
which step 3 writes.
Each state is told from evidence on disk and from the record:

- a claim and no registration:
the add has not run Git;
the same add resumes,
or the removal clears the claim;
- a registration and no clone:
the snapshot or clone did not happen;
the capture record is still the source's own,
so a fresh snapshot is taken,
or the add refuses when the record is gone;
- a clone and no mount:
the clone's identity is checked against the binding and the mount is requested again;
the proof runs after the mount arrives,
from the capture record,
so a source that moved since the snapshot does not fail it;
- a mounted clone with the copied `.git` and no `proof=passed` phase:
the proof runs from the capture record and the restore then runs whole;
- a mounted clone with the copied `.git` and `proof=passed`:
the restore runs whole without the proof,
because the tree may already hold written bytes and a second proof would refuse the add's own work;
- a mounted clone with no copied `.git`,
no pointer and `proof=passed`:
the restore had reached step 8;
it publishes the index and the pointer and verifies the worktree;
- a mounted clone with no copied `.git`,
no pointer and no proof phase:
the clone cannot be proven and the tree may hold the other sessions' bytes.
The add refuses and leaves the pending state (L6);
nothing is destroyed;
- an index under the real name and a pointer,
with the binding not usable:
step 8's verification runs again,
and on success the binding is marked usable without a second restore;
- a binding marked usable:
nothing is resumed;
it is a worktree.

The capture record's end follows the states:
it stays while any row above is still reachable,
and it is removed on the success of step 9 or when the add leaves a state in which the proof can no longer pass and the clone is settled.
A `doctor` line reports a record whose add left none of those states,
and a staging name a killed writer left.

### The cost, against a clean add and against rung 3

No number here is claimed unmeasured,
and each owed measurement names the arm it must be compared against.

- The capture and the proof against a clean add.
[M1][m1] runs three arms in rotating order over this repository's paths:
a dirty source added on rung 1,
a clean source added on rung 1,
and the same adds on a build that never takes rung 1,
which is the identical-binary control.
A fourth arm,
built with the crate's existing `measurement-routes` feature and run after the three,
keeps the capture and skips the proof,
so the proof's own share is visible;
it is never a shipped switch.
It reports each arm's median and range and the ratio of the dirty arm to the clean arm and of both to the control.
No figure is quoted here:
the only numbers in hand are F25's script timings and the rung 2 record's walk (A5),
and neither is this rung's cost.
- The copied administration against inline deletion.
[M3][m3] measures the add with the trash hand-over (F32,
A4) against a control arm that deletes the copied `.git` in place,
on a fixture whose administration holds the files F26 measures,
and reports both ratios and their spread.
- The restore against Git's own checkout.
[M4][m4] measures adds from sources with 5,
300 and 6,000 changed tracked paths,
alternating arms with an identical-binary control,
and reports the ratio to Git's own checkout at each shape.
Whether rung 1 should decline a very dirty source as rung 2 does is Q-B and not a bar this design sets.
- The fallback after a refused proof.
The shipped design has no fallback after the mount (Q-A's interim),
so the only arm is [M5][m5]'s:
an add that refuses and leaves the pending state,
timed to the point it prints,
against the same add on a build that never takes rung 1.
If Q-A is answered with the settlement,
the same probe measures the settlement's path too.
- The refusal rate on a busy checkout.
[M6][m6] measures how often a proof refuses when other sessions write the checkout during the add,
at the window this design leaves,
and how often the main checkout's own `git status` refuses it through D1 (V8).

### The tests a build must have

Each race is forced at a seam the code already offers or gains,
never by timing,
and every refusal test carries its no-write control,
because a test that cannot pass on an unchanged window proves nothing.

- T1.
`the_frozen_proof_refuses_a_tracked_write_between_the_capture_and_the_snapshot`.
The rung's snapshot step gains a seam between the capture and the snapshot request,
in the shape `PlaceSeams` already has (`worktree_snapshot.rs`),
and the control writes a tracked file in it.
The stand-in snapshot is the fixture tree itself,
not a copy:
a copy cannot carry the source's change time,
so a copying stand-in would refuse every file for the wrong reason (M5).
The no-write control asserts the unchanged fixture passes the proof.
Assert the refusal,
the path and the field it names,
and the failure sentence.
Runs on the host.
- T2.
`the_frozen_proof_refuses_an_index_written_between_the_capture_and_the_snapshot`.
The same seam,
with a control that runs `git add` in the fixture.
Assert D1 refuses,
names the index,
and the restore does not run.
Its no-write control is T1's.
- T3.
`a_write_in_the_file's_own_tick_is_refused_by_the_racy_set`.
The seam's control advances the clock's tick deterministically,
by reading the file's change time until it moves and only then writing,
which is a bounded wait on state and not a sleep;
it writes the same length.
Assert D4 refuses through C4's digest.
The control that must pass:
a write after the tick moved is refused by D2 and not by D4,
so the two arms tell the two refusals apart.
- T3b.
`a_same_tick_rewrite_of_a_vouched_path_before_the_capture_is_not_refused`.
The seam's control writes a path twice inside one tick before the capture takes its facts,
and the test asserts the add proceeds and the clone carries that write,
which is exactly L4's residual.
It is the case V4's option (b) closes,
and it is stated rather than left to be discovered.
- T4.
`the_capture_digests_only_the_unvouched_and_racy_entries`.
A fixture with one edited path,
one path written in the current tick and many untouched ones;
assert one digest per set member,
and that a control which touches every file records every path.
This is the test that keeps the capture's cost from drifting to F25's digest arm.
- T5.
`a_resumed_add_takes_the_verdict_of_its_row`.
One case per row of "What a resume finds":
kill the add at that step's boundary through the seam the step has,
resume,
and assert the row's verdict and the capture record's presence or absence.
The kill points are the claim,
the registration,
the snapshot,
the clone,
the mount,
the proof's phase,
each restore step,
the trash rename,
the index's publication and the pointer's.
- T6.
`the_index_the_clone_installs_takes_no_field_from_a_file`.
After step 5,
assert for one edited path that the private index's change time,
modification time,
size,
device and inode equal the captured index entry's,
not the file's;
and that a same-size edit whose modification time was put back is still rewritten by the reset.
The test is red if the install copies a change time,
which is H1's control.
- T7.
`an_untracked_repository_in_the_source_is_refused_before_the_snapshot`.
A fixture whose source holds an untracked,
not ignored directory with its own repository:
assert the preflight refusal,
no managed object,
and Git's own checkout.
Its control is the same fixture without the nested repository,
which the rung serves.
V7 is the choice this test rests on.
- T8.
`a_converting_path_in_a_dirty_clone_is_written_by_git`,
in the guest,
over the four classes issue 6323 named:
a `text` rule,
`core.autocrlf`,
`ident`,
and `text=auto eol=lf` over a CRLF file,
each restored to a commit other than the source's `HEAD`,
compared with a real Git checkout of that commit.
- T9.
`the_clone_restores_a_dirty_source_to_the_commit_the_caller_asked_for`,
in the guest:
one staged edit,
one unstaged edit,
one deletion,
one mode change,
one untracked file that is not ignored,
one ignored file,
one symbolic link,
and one tracked path whose kind is refused.
Assert the served tree equals Git's own checkout plus the ignored file,
and the person reads the left-out line with the count.
- T10.
`no_state_with_a_valid_pointer_over_an_unrestored_tree_exists`,
in the guest:
kill at each of step 8's two renames and at the index's publication,
and assert for each leftover that the slot either holds no `.git` or holds one whose index is the finished one,
and that the next add's verdict matches T5's rows.
- T11.
`a_refused_proof_leaves_the_pending_state_and_the_removal_completes_the_add`,
in the guest:
the T1 refusal at the mount,
the printed command,
the removal,
and the same add again served by Git's own checkout.
Its control is the same fixture whose proof passes.

One control is run beside T1 to T4,
`the_unchanged_window_passes_the_proof`,
because a proof that refused an unchanged fixture would make every refusal test meaningless.

Which need a guest:
T8,
T9,
T10,
T11 and the mount-dependent half of T5.
T1 to T4,
T6 and T7 run on the host,
over the fixture tree itself.

### Where the result differs from Git's own checkout

These are the cases the design knows,
each with its evidence and what the design does:

- An untracked,
not ignored directory holding its own repository stays after `git clean --force -d` (F4,
F24),
and the restore's proof then names it as untracked and refuses.
On btrfs that costs a destroyed stage;
on ZFS,
without a preflight refusal,
it would happen after the snapshot,
the clone and the mount.
The design refuses it before the snapshot,
from the untracked listing the preflight already makes (`SPEC.md:1381`),
with no managed object (V7).
- The modification times of unchanged files and the directories' modes are the source's,
not the checkout's time and umask;
`git status` is unaffected and build tools see older times.
The bytes are equal.
- The ignored state is the snapshot's instant (`SPEC.md:1518`),
so a path the target commit stops ignoring is carried and listed as untracked (`SPEC.md:1335` to `:1342`).
The proof counts it as carried.
- A tracked file whose recorded stat data the settle believes can keep the source's bytes if the source was written in the same tick as the entry (L4,
F30).
The design names this as its one silent-byte residual and offers the settle's racy rule as V4.
- A split or sparse index cannot be installed at all (F34),
which ends the restore after the mount.
- Issue 6323's conversion class is the fence's,
and the fence applies to rung 1;
T8's guest controls are what a dirty restore to a non-`HEAD` commit owes.
- A hidden trash holding a copy of the checkout's administration stays in the worktree until the disposal runs (V5).

## Which build step builds which part

- Plan step 10 builds the add's route from registration to verified fill,
and it builds the frozen proof for every rung 1 add,
clean ones included:
C1,
C2 and C5's commit,
D1 to D4,
the private-name index write,
the discard and the pointer install in this design's order,
and the resume states.
For a clean source C3 and C4 are empty or nearly so,
and "neither route accepts an unproved source" is met by the same proof.
This is a change to the plan's allocation and is veto entry V6.
- Plan step 11 builds the dirty half:
the full refusal set re-run against the frozen capture,
C3 and C4's digests,
the restore's wiring to `restore_dirty` with `GIT_INDEX_FILE`,
the nodes' hand-over of the clone's index,
the pending state,
T1 to T7 and the guest tests T8 to T11.
- The settlement a refused proof makes after the mount,
if Q-A is answered with it,
belongs to the removal's build (steps 14 to 17),
because it is an unmount,
a destruction and a Git settle,
and no sibling design offers it to an add.
- `SPEC.md` gains,
under the add section and beside the btrfs rung's dirty paragraph:
what a clone of a dirty main checkout restores,
what the frozen proof compares (with the racy set),
the refusal when it differs,
the private-name index and the pointer's publication,
the resume states,
and the one failure sentence with its reason.

## Measurements owed

Each is a probe a later round runs as written,
with its control arm,
its rounds and the spread reported (rules `QNB` and `NQH`),
and each says where it runs.

- M1.
The capture and the proof's cost,
with a clean-add arm.
Three arms in rotating order,
ten rounds each:
a clean source on rung 1,
a dirty source on rung 1,
and the same adds on a build that never takes rung 1,
which is the identical-binary control.
A fourth arm,
built with the crate's `measurement-routes` feature and run after the others,
skips the proof so the proof's own share is visible;
it is never shipped.
Each add's own step timers report the capture,
the walk,
the digests and the compare separately.
Report each arm's median and range,
the ratio of the dirty arm to the clean arm,
and both to the control,
with the count of rounds where rung 1 was slower.
Runs on the host.
- M2.
The clone's metadata identity.
In the guest,
on a file-backed pool:
a fixture of 2,000 paths with mixed kinds,
sizes,
modes and one symbolic link;
snapshot,
clone,
mount,
and compare inode,
size,
kind,
mode and both times to the nanosecond for every path between the mount's root and the source;
report every path that differs and how.
The positive control writes one file in the window and asserts the comparison reports exactly that path,
and the reviewer's tick probe (F22) is repeated inside the guest,
because the tick is the kernel's and a guest's need not be one millisecond.
- M3.
The copied administration's hand-over.
In the guest:
an add whose clone carries an administration of 634,426 files in 6,708 directories,
one arm handing it to the clone-local trash and one arm deleting it in place,
alternating with the identical-binary control,
ten rounds;
report each arm's median and range and the disposal's cost separately,
from the journal's own timestamps.
- M4.
The restore against Git's own checkout.
In the guest:
sources with 5,
300 and 6,000 changed tracked paths,
each added with rung 1 and with rung 3,
alternating arms with the identical-binary control,
ten rounds a shape;
report the ratio for each shape with its range.
A shape where rung 1 is slower than rung 3 is material for Q-B.
- M5.
The pending state's cost.
In the guest:
an add whose proof is forced to refuse at the T1 seam,
timed from the command's start to its end,
against the same fixture added on a build that never takes rung 1.
Report the ratio and the range,
and the same measurement for the settlement,
if Q-A takes it.
- M6.
The refusal rate on a busy checkout.
On this host:
100 adds from the main checkout while other sessions write it,
with the capture immediately before the snapshot,
and 100 with the capture at preflight;
report how many of each refused,
which field refused,
and how many refused through D1 because a `git status` in the main checkout refreshed the index.
Nothing here generates load:
the adds are real adds,
and the sessions writing the checkout are whichever sessions are there (rule `NQH`).
- M7.
The racy set's size.
On this host,
at ten moments of a working day:
read the main checkout's index,
list the entries whose recorded modification time or change time is not at least one tick older than the index's own modification time,
and report the count and the sizes of those entries.
This is the cost C4 adds,
and it is the number that decides V4's ranking if it is large.
- M8.
The settle's exposure on ZFS.
In the guest:
a fixture whose tracked file is written twice inside one tick,
with the index written before the second write;
assert whether the btrfs rung's settle believes the file and publishes the second write's bytes.
The positive control writes in a later tick,
which the settle must read.
This is what decides whether the ZFS rung's "comparable to Btrfs" needs V4's rule.

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
a digest for the entries the stat data cannot vouch for,
and a digest for the entries in the racy set (C1 to C4).
  - Option (a),
this one:
an add from the main checkout as it stood on 2026-11-22 reads 3,436,101 bytes of index,
one `lstat` for each of 24,685 paths,
and the content of the 5 changed paths plus whichever entries the racy set holds,
which [M7][m7] counts.
  - Option (b),
a content digest of every tracked path:
the same add reads 764 MB and pays 4.9 to 9.4 s (F25),
and it refuses a same-tick write for every path rather than the racy set's.
  - Option (c),
the index's bytes alone:
the same add pays 1.5 ms and catches a concurrent `git add`,
but a session that edits `AGENTS.md` in the window is carried into the clone and published as Git's checkout.
  - Pros and cons:
(a) closes the window at a cost [M7][m7] measures,
and leaves only the pre-capture tick coincidence (L4);
(b) closes it for every path at nine seconds of the add's time;
(c) is nearly free and leaves the window the answer's own words call unproven.
  - Ranking:
a > b > c,
because a proves the tracked content the answer names at a measured small cost,
b buys a narrower residual for the add's whole time,
and c leaves the window unproven.

- V2.
Which fields the per-path comparison uses.
The design compares the kind,
size,
modification time,
change time,
the execute bit and a link's target,
and does not compare the device or the inode.
  - Option (a),
this one:
a session that edits `AGENTS.md` in the window moves its change time into a later tick,
and the add refuses naming `AGENTS.md` and its change time (F22,
F23).
  - Option (b),
the inode compared and the device not:
the same add refuses every path whose inode the clone did not share,
which [M2][m2] decides;
if the clone shares inodes as F12 says,
every add passes and the comparison is stronger.
  - Option (c),
the size and modification time only:
a session that rewrites a file,
keeps its length and restores its modification time is carried into the clone,
because the change time is not compared.
  - Pros and cons:
(a) rests on the times the kernel sets,
and its equal-times case is the tick (L4),
covered by C4;
(b) adds a field that F12 and [M2][m2] say is shared,
and it adds no protection against a same-tick write,
which only C4 catches;
(c) drops the one field a write cannot hide below a tick.
  - Ranking:
a > b > c,
because a needs no unmeasured sharing and adds nothing a write can hide,
b rests on the same unmeasured sharing as a for no gain against the tick,
and c loses the compared field that catches a write in a later tick.

- V3.
Where the frozen capture lives.
The design writes it in its own family folder under the wrapper's record parent,
`<common>/monochromatic-git-policy-cli/add-capture/v1/<slot>.capture`.
  - Option (a),
this one:
an add killed after the snapshot is resumed the next morning and proves the frozen clone even though the checkout moved meanwhile.
  - Option (b),
beside the claim as `<identifier>.capture` in the copy journal's folder:
the same resume works,
and the record sits where a sweep already lists names,
though the sweep reads only its own suffix (F35).
  - Option (c),
memory only:
the same resume cannot prove the clone,
so it refuses and leaves the pending state (L6).
  - Pros and cons:
(a) buys the resume,
follows the wrapper's own record convention for one family per folder,
and keeps the record out of the sweep's folder;
(b) is the same data beside records whose folder has a sweep that would have to learn its name;
(c) costs nothing and loses every resume.
  - Ranking:
a > b > c,
because a needs no change to the sweep and matches the convention the binding record sets out,
b works but widens a folder another design owns,
and c throws away the clone (a) proves.

- V4.
The settle's racy rule on ZFS.
The design leaves the btrfs settle as it is,
so a tracked file whose recorded stat data is its own to the nanosecond is believed,
even on ZFS where the tick can hide a write before the capture (L4,
F30).
  - Option (a),
this one:
the residual is stated with its size and [M8][m8] measures whether it can land;
nothing in the shared settle changes,
and a same-tick write before the capture can publish that session's byte as clean.
  - Option (b),
the settle judges by content the entries in the racy set,
as Git's own racy rule does at second granularity (`subvolume_index.rs:520` to `:537`),
so a same-tick write before the capture is rewritten;
it changes code the btrfs rung shares and costs a read of [M7][m7]'s count of files.
  - Option (c),
the capture digests every path and the settle judges by content every path it doubts:
the same closure for the settle at F25's 4.9 to 9.4 s.
  - Pros and cons:
(a) is free and leaves a silent wrong byte if a millisecond coincidence lands;
(b) closes it at a measured count of files and a change to shared code;
(c) closes it at the add's whole time.
  - Ranking:
b > a > c,
because b closes the silent byte at a cost [M7][m7] measures,
a states the residual and keeps the shared code,
and c pays nine seconds for what b buys with a handful of reads.
[M8][m8]'s guest case is what decides whether b must land before the build.

- V5.
How the copied administration is discarded.
The design moves it into a trash inside the clone and lets the disposal empty it after the add returns,
as the btrfs rung does.
  - Option (a),
this one:
an add renames 634,426 files in one operation and returns with a hidden trash in the new worktree until the disposal has emptied it.
  - Option (b),
delete it in place:
the same add pays the unlinking of 634,426 files on its waiting path,
which [M3][m3] measures.
  - Pros and cons:
(a) costs one rename and reuses the mechanism the rung has (F32),
at the price of a copy of the checkout's administration living in the worktree until the disposal runs;
(b) leaves nothing behind and pays the whole deletion before the worktree is usable.
  - Ranking:
a > b,
because the btrfs rung already chose one rename over a deletion on the add's path (F32),
and the same account holds every byte of the copy.

- V6.
The plan's allocation of the work.
This design assigns the frozen proof to step 10,
because it is the proof of every rung 1 add,
and leaves step 11 the dirty restore and the fallback.
  - Option (a),
this one:
a clean add on rung 1 is proved from the first build,
and step 11 changes only the capture's dirty facts,
the restore and the pending state.
  - Option (b),
the plan as written:
step 10 ships clean adds on rung 1 unproved,
and step 11 adds the proof for both,
which leaves "neither route accepts an unproved source" unmet between the two builds.
  - Pros and cons:
(a) meets the decision from the first build and edits the plan's text;
(b) keeps the plan's words and leaves a window in which an unproved clone route is shipped.
  - Ranking:
a > b,
because the decision forbids an unproved source at any release and no record says step 10 may ship one.

- V7.
Refusing an untracked nested repository before the snapshot.
The design refuses such a source at preflight,
from the untracked listing it already makes,
and takes rung 3.
  - Option (a),
this one:
a source holding `.claude/odd-project/.git` is refused before anything is written,
and the person gets Git's own checkout,
which keeps the directory as it is.
  - Option (b),
let the restore meet it:
`git clean --force -d` keeps the directory (F4),
the proof names it,
and the add leaves the pending state after paying the snapshot,
the clone and the mount.
  - Option (c),
give the clone the `clean`'s second `--force`:
the directory goes,
which the walk-and-clean record's sentence against nested-repository protection warns about (`doc/planning/cli-git-zfs-worktree-design-proposal.md:1078` to `:1079`).
  - Pros and cons:
(a) costs nothing and keeps the rung out of a case it cannot finish;
(b) needs no refusal code and wastes the whole add;
(c) finishes the add by deleting a repository the source holds.
  - Ranking:
a > b > c,
because a refuses before paying and names Git's own checkout,
b at least keeps the person's repository,
and c deletes what the record warns against.

- V8.
Comparing the copied index byte for byte.
D1 compares the clone's copied index with C1 as bytes.
  - Option (a),
this one:
a `git status` in the main checkout that refreshes one entry's stat data refuses the add,
which leaves the pending state;
[M6][m6] counts how often.
  - Option (b),
comparing only the entries that matter,
the paths in C2 and C3:
a stat-only refresh passes,
and a staged change is still refused;
the reader is larger and a format change passes unnoticed.
  - Pros and cons:
(a) is exact and simple,
and refuses legitimate work another session did;
(b) tolerates refreshes and needs a field reader whose every corner is a control.
  - Ranking:
a > b,
because a proof one byte wide is what the decision asked for,
and b's tolerance buys a reader whose misses would be silent.

## Questions only the project can answer

- Q-A.
When the clone is mounted and the proof refuses or the restore fails,
what may the add do?
The accepted design says (`doc/planning/cli-git-zfs-worktree-design-proposal.md:430` to `:431`):
"No silent fallback over an unresolved managed state.
An explicitly recovered Git-only add may finish at the same slot only after the mount and all outstanding intents are settled."
  - Case:
another session saves `AGENTS.md` while the add runs,
so the proof names `AGENTS.md` and its change time.
  - Option (a),
settle and finish:
the add requests the unmount,
waits for its receipt within the step's bound,
destroys the clone and then its snapshot (F13),
releases the slot,
and finishes with Git's own checkout,
printing the failure sentence.
The person gets the worktree they asked for.
  - Option (b),
refuse and leave:
the add prints the failure sentence and the removal command,
keeps the pending state,
and the person runs the removal and adds again,
which then takes whatever rung the ladder chooses.
  - Option (c),
settle or retry once first:
as (a),
but a fresh capture and snapshot are taken once before settling,
so one stray save still ends in a served clone.
  - Pros and cons:
(a) delivers the decision's purpose and adds a destructive step to an add,
automating what the record calls explicitly recovered;
(b) is the record's literal reading and the cheapest,
and leaves the person without a worktree after a failure that was not theirs;
(c) turns one stray save into a success and doubles the waste of a refusal on a checkout that is written constantly,
at a rate [M6][m6] measures.
  - Ranking:
a > b > c,
because a keeps the command's promise at the cost [M5][m5] measures,
b keeps the record's words and gives up the promise,
and c spends a second snapshot on a refusal rate nobody has measured,
which can become b's or a's once [M6][m6] reports it.
If the owner reads "explicitly recovered" strictly,
the order is b > a > c.
  - Dependency:
no sibling design gives an add a callable unmount-and-destroy;
the origin cleanup design reports a killed add's snapshot and destroys nothing,
and the four removal designs are written for removals.
Option (a) therefore needs an answer from a removal design,
and option (b) needs none.
- Q-B.
May rung 1 decline a very dirty main checkout in favour of Git's own checkout,
as rung 2 does?
The partial-clone decision cuts rung 2 at a changed-file count and states "no add is slower than Git's own checkout" (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`,
"What was decided");
no record says it of rung 1.
  - Case:
a main checkout with 6,000 changed tracked files,
where [M4][m4] would show rung 1 slower than rung 3.
  - Option (a),
decline above a count [M4][m4] measures,
with rung 2's bar.
  - Option (b),
always take rung 1 for a dirty main.
  - Pros and cons:
(a) never slower than Git's own checkout,
and gives up the clone's gain above the cut;
(b) keeps one route and the clone's disk saving,
and can be slower than Git's checkout on a very dirty source.
  - Ranking:
a > b,
because the owner's own rung 2 answer chose "no add slower than Git's own checkout" for a dirty source,
and rung 1's advantage is speed.
The cut itself is a measurement,
not a question.
- Q-C.
Which writes must the frozen proof refuse?
Plan steps 10 and 11 read "a tracked write between preflight and snapshot that the frozen proof refuses" (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:1456` to `:1459`,
`:1614` to `:1617`).
  - Case:
a session stages a new file shortly after the add starts,
while Git registers the worktree.
  - Option (a),
this design:
the capture is taken immediately before the snapshot and the whole refusal set is re-run against it,
so the staged file is captured with the rest and the add proceeds.
  - Option (b),
capture at preflight only,
as the accepted text reads:
the staged file refuses the add if its entry kind is one no reset can undo,
and a tracked write between the two readings is not refused by the proof.
  - Option (c),
capture at both points and refuse when the two readings differ:
the plan's wording holds,
the window is narrow,
and a legitimate staged file refuses the add and leaves the pending state.
  - Pros and cons:
(a) loses no safety,
refuses rarely,
and departs from the plan's sentence;
(b) matches the plan,
leaves the longest window,
and can carry an entry kind the refusals never judged;
(c) satisfies both and turns a handled write into a pending state,
for one more walk.
  - Ranking:
a > c > b,
because a refuses nothing the restore would have handled and judges the index it restores from,
c at least narrows the span at the cost of refusing a staged file,
and b can restore from an index no refusal read.
If the owner wants the plan's sentence kept literally,
c is first.
- Q-D.
What may the slot look like to a command run in it while the restore is filling it?
  - Case:
a session runs `git status` and then `git commit -a` in `.claude/worktrees/new` while the add is still restoring.
The wrapper's own policies for `commit` and `clean` may stop some of these commands.
  - Option (A),
this design:
the copied `.git` stays until step 8,
the restore runs on the registered administrative directory through the environment,
and the real-name index and the pointer come last.
The session sees a mirror of the main checkout's status,
and a `git commit -a` lands in the copy and is lost with it.
  - Option (B),
discard first as step 4 says literally,
and write no pointer and no real-name index until the end:
the session finds no `.git` in the slot,
so Git walks up to the main checkout;
a `git status` shows the main checkout,
and a `git commit -a` commits the main checkout's tracked changes under its branch.
  - Option (C),
the order the first text had,
pointer and real-name index first:
the session sees a registered worktree whose changes are the other sessions' edits,
and a kill leaves an index that can vouch for the copied bytes.
  - Pros and cons:
(A) keeps what the accepted design already accepts and no mutating command can reach the main checkout;
(B) follows step 4's words and makes the slot a plain directory of the main repository for the span;
(C) attributes foreign edits to a valid worktree and publishes an index that can vouch for the wrong bytes after a kill.
  - Ranking:
A > B > C,
because A leaves a mutating command confined to a copy that is thrown away,
B sends one to the main checkout,
and C publishes a worktree whose changes are another person's.
If the owner holds step 4's sentence literally,
B is first.
  - Sibling note:
the add-time lock design's D6 describes a slot with no `.git` after the copied administration is discarded,
which is B's state;
it does not decide the order and this design's A leaves no such span to describe.

## Dependencies on the sibling designs

Each is a place where this design answers with an assumption a later pass reconciles,
read at the commit named.

- The durable binding record (branch `docs/planning-cli-git-zfs-add-binding-record-design`,
read at `428040d45d`):
this design writes the capture in a family folder of its own (V3a),
and the binding design says the content proof "and where it lives across a kill" is not covered by it,
so the two agree.
Its record convention gives the parent folder `<common>/monochromatic-git-policy-cli/` and one family folder per record,
which this design follows;
its list of five name forms is for its own family and does not bind this one.
Its files stay out of the copy journal's folder,
and so do this design's.
- The add-time lock (branch `docs/planning-cli-git-zfs-add-time-git-lock-design`,
read at `a7902e293b`):
the frozen capture is written and the proof runs while the add holds its claim and its managed lock.
Its D6 describes the slot after the discard and before the pointer as holding no `.git`;
Q-D's option A leaves the copied `.git` until step 8,
so that state is Q-D's B and not A.
Nothing in the lock design depends on which the project takes,
and its lock's lifetime covers this design's step 9.
- The removal designs,
for Q-A's option (a):
the cancelled or interrupted removal (`42f01440f3`),
the forced arm (`d191e5b257`),
the deferred-destroy checks (`3e34817184`),
the origin cleanup record (`4daa6c66ce`) and the destruction plan (`a7823980b9`).
None says an add may call its unmount,
destruction or settle.
The origin cleanup design's unaccounted-snapshot line covers a killed add between the snapshot and the clone and "destroys nothing and names the command a person may run";
this design's pre-mount failures settle within the live invocation,
which is a different scene,
and the boundary between them is that the add is still running.
- The persisted destruction plan (`a7823980b9`):
if Q-A takes settlement,
a plan is written before the first of the two destructions,
as that design requires.
Until then this design writes no plan.
- The private propagation after a replacement mount (`5e6a2ff9dc`) and the lost-last-event triggers (`80a874b773`):
this design's step 1 assumes the mount of a fresh clone is verified in the caller's namespace by the add's step 4,
and adds no trigger of its own.
- The start probe and the ABI gate (step 6):
nothing here reads the kernel's layout;
the proof reads files and `stat` only.

## How the accepted sentences are met

- "a verified dirty-main restore comparable to Btrfs":
the restore is the btrfs rung's own,
run with the clone's root as its stage and an index under a private name,
and its proof is the frozen proof.
Where it is weaker than btrfs is stated:
the change time's tick and the settle's trust (L4,
F21,
F30,
V4),
and the guest probe [M8][m8].
- "neither route accepts an unproved source":
the proof is D1 to D4 for every rung 1 add,
clean ones included (V6),
and a refusal leaves the pending state rather than a served worktree.
- "capture the index/tree verification inputs" (design step 1):
C1 to C5,
taken at preflight for the cheap refusals and again as the frozen capture before the snapshot,
with the whole refusal set re-run.
- "Compare the frozen index and tracked content in the clone itself against the captured proof,
without running Git against that directory.
The in-snapshot check proves the bytes" (design step 4):
D1 to D4,
run before any write and before any Git run inside the clone.
- "Discard copied Git administration before running Git inside the clone" (design step 4):
no Git run is given the copied administration,
and its discard is step 8's first rename;
Q-D puts the literal order to the project.
- "Install the saved Git pointer inside the clone.
Use the verified index and the existing conversion checks and tree difference in the new worktree context" (design step 5):
the private-name index of step 5 and the fence's write of step 7.
- "No silent fallback over an unresolved managed state":
the design refuses and leaves the pending state after the mount until Q-A says otherwise,
and its pre-mount failures settle only objects whose creation and identity this invocation proved.
- "No configuration":
nothing here has a setting;
every threshold it needs is a measurement it names.
- The nearest lock-free record shape:
the capture is written with an exclusive staging name and linked into place,
and `doctor` reports a staging name a killed writer left.

## Hardening the stated threat model may make unnecessary

The project's sentence treats the account as trusted and root and the account as no security boundary on this machine.
Each check of this design is placed against it here,
and none is dropped on the strength of the sentence alone.

- The frozen proof's refusal (D1 to D4).
It survives:
the writes it catches are this machine's other sessions saving files and running Git in the main checkout,
which happen daily.
- Comparing the copied index byte for byte (V8).
It survives:
byte equality is the cheaper comparison,
and it also catches a reordering or an extension a field comparison might not read.
- The racy set's digest (C4).
It survives:
it exists because of the kernel's coarse clock (F15) and Git's seconds (F2),
not because of a hostile account.
- Reading the clone's files below a held root without following a link.
It survives as an accident guard,
and the helper is the wrapper's own (`open_root`,
`DirectoryChain`).
- Removing only a clone-local stub it can name,
and refusing a stub that holds more (step 4).
It survives:
the underlay's files are a person's,
and a snapshot may have carried them (F18).
- Verifying the mount's identity at the slot before the proof runs.
It survives:
a mount stacked at the slot is as much a person's own shell as an attacker's,
and the add's step 4 already decides it.
- Checking the capture record's owner and mode before reading it.
This one exists only for a hostile account:
the record lies in the account's own folder,
where the account can write it,
and a mode check stops another account,
of which this machine has none.
It stays,
labelled as such.
- Refusing to end a holder that keeps the clone,
and refusing to retry.
They survive:
the add has no authority to end another process and seeks none,
which is why Q-A's options do not include it.
- `doctor` reporting a staging name or a record an add left.
It survives as tidiness.

## Limitations

- The proof detects a window write and cannot prevent one,
and its detection rests on a millisecond tick (F15,
F22) and on Git's seconds (F2,
F23).
- A tracked write in the same tick as the file's own last write,
before the capture,
can leave every compared field equal,
and on ZFS the settle then believes the file (F30):
the design states this as L4 and offers the settle's racy rule as V4.
The exposure needs the index write,
the file's last write,
this add's `lstat` and the other session's write inside one millisecond,
and it is not measured.
- The clone's fill is visible and writable before it is proven,
and Q-D asks what the slot may look like meanwhile;
in the adopted order the slot holds a copy of the main checkout's administration until step 8,
so a command run there acts on that copy and is lost with it.
- The ignored state the clone carries is the ignored state of the snapshot's instant (`SPEC.md:1518`),
and a path the target commit stops ignoring is carried and listed as untracked.
- A tracked file under a nested dataset's mount location:
the capture reads the source's mounted content and the clone holds the snapshot's underlay,
so the proof refuses such a path and the add leaves the pending state if such a path is tracked.
Refusing before the snapshot would need a check of the source's mount table against the tracked paths,
which this design does not build.
- The copied administration lives inside the new worktree until the disposal empties it (V5),
so a `du` of the new worktree reports it,
and a person who deletes it deletes only a copy.
- The clone shares its blocks with the origin snapshot,
so the origin cannot be destroyed while the clone lives (F13),
which fixes the order of every cleanup.
- Nothing here measures a real clone,
a mount,
a restore inside one or a fallback;
every such claim carries its probe.
- The capture's record is about 2.5 MB on this repository (L7),
a per-add write the add did not have before.

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
and the conversion fence's cost.
- The dependency repair after a clone fill (plan step 21),
which step 9 queues as before.
- The one-time move of the checkout to its own dataset,
the boot units and the migration (plan step 19).

## Citations

Every quotation and every path-and-line citation in this document was checked by a scratch script that printed each cited range beside the text of the lines it names,
and a second pass compared each blockquote with its source.
The document holds 143 citations.
The gate review named 7 wrong and 2 loose,
and those 9 are the ones corrected below.
Every one of the other 134 was printed beside the lines it names and read,
and each says what this document says it says.
A second pass matched each blockquote's words against the lines it quotes.
The corrections are:
`SPEC.md:1355` for the three-runs sentence (the first text cited `:1333` to `:1340`);
`statinfo.c:64` to `:99` for Git's comparison,
with the `USE_NSEC` branch at `:75` to `:81` and the `USE_STDEV` branch at `:91` to `:99` (the first text cited `:64` to `:96` and claimed the nanoseconds);
F15 for the ZFS stamp,
replacing the generic write path the first text cited as F14 (`mm/filemap.c:4472`,
`fs/inode.c:2501`);
F17 for the snapshot's nested copy,
replacing the first text's F13 use of the measurement record;
`SPEC.md:1518` for the ignored state's instant,
without the `:1468` to `:1480` range the first text added;
`AGENTS.md:362` for `QNB` (the first text wrote `NQB`,
which does not exist);
issue 6323's own lines (`SPEC.md:2736` to `:2738`) for the conversion defect it names;
`sa.c:1704` to `:1706` with the words it holds (the first text's sentence named times the lines do not hold);
and `SPEC.md:4408` to `:4410` for the record sentence (the first text cited the heading's line `:4398`).
The script's output,
printing every citation beside the lines it names,
is kept at `~/temp/agent/zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/citations-2026-11-22.md`.

## Revisions

- `069fa35d98` at `2026-11-22T17:35:38-05:00`:
the first text.
The design as reviewed:
the capture taken immediately before the snapshot,
the proof as three comparisons,
the device-and-inode rewrite in step 5,
the pointer and index published before the restore,
V1 to V6 and Q1 to Q3.
- `1f24dcc10d` at `2026-11-22T17:36:49-05:00`:
named the package docs the design read,
in "What it read" and "Sources read".
- `414d9901fc` at `2026-11-22T18:38:11-05:00`:
the gate-review revision.
H1's rewrite removed,
H2's order,
the tick and Git's seconds,
the refusals re-run against the capture,
Q-A to Q-D replacing V5 and Q1 to Q3,
the capture's format and end,
the allocation to step 10,
the cost and test sections rebuilt,
the citations rechecked,
and the sibling passages read at their commits.

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
`module/zfs/zfs_vnops.c`,
`module/os/linux/zfs/zfs_znode_os.c`,
`module/os/linux/zfs/zfs_vnops_os.c`,
`module/os/linux/zfs/zpl_super.c`,
`include/os/linux/spl/sys/time.h`,
`include/sys/zfs_znode.h`.

Linux `v7.3`,
five files fetched with `gh api` at tag `v7.3`:
`fs/attr.c`,
`fs/utimes.c`,
`fs/inode.c`,
`mm/filemap.c`,
`fs/btrfs/super.c`.

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
`doc/handover/cli-git-rust-implementation.md`;
`package/git-policy/cli/SPEC.md`;
`package/git-policy/cli/src/native/worktree/worktree_dirty_restore.rs`,
`worktree_snapshot_index_step.rs`,
`worktree_snapshot.rs`,
`worktree_snapshot_source.rs`,
`worktree_ladder_finish.rs`,
`worktree_ladder_target.rs`,
`worktree_slow_route.rs`,
`worktree_clean.rs`,
`worktree_clean_tree_walk.rs`,
`worktree_clean_stat_verdicts.rs`,
`worktree_clone_index.rs`,
`worktree_clone_conversion_fence.rs`,
`worktree_copy_journal.rs`,
`subvolume/subvolume_index.rs`,
`subvolume/subvolume_index_install_into_new_worktree.rs`,
`subvolume/subvolume_index_files.rs`;
`package/git-policy/cli/doc/slot-identity-store-write-cost.md` and `package/git-policy/cli/doc/object-database-and-a-reftable-ref-store.md`.

The gate review and its probe logs at `~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review.md` and `probe-logs.md`,
and the sibling designs at the commits named under "Dependencies on the sibling designs".

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
#measurements-owed [m7]:
#measurements-owed [m8]:
#measurements-owed [vm-register]:
../planning/cli-git-zfs-measurements-owed-before-acceptance.md
