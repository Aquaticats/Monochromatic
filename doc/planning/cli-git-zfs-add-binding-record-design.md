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

It read the accepted design,
the proposal that carries it,
the build plan,
the wrapper's specification,
the code whose records it reuses,
and the OpenZFS `zfs-2.4.4` and Git `v2.56.1` sources it cites.
It ran plain `git` twice over:
once on this host,
where plain `git` is the wrapper `monochromatic-git-policy-cli 0.6.1-alpha.44` shadowing Git `2.56.1`,
and once as real Git `2.56.1` inside this repository's gate image,
with no folder of the home mounted.
Each result says which of the two produced it,
and what the wrapper adds to those subcommands is named beside it.
It also read read-only `zfs` commands on this host.
It ran no ZFS command that changes anything,
opened no `/dev/zfs`,
started no guest,
no container and no service manager,
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
This design takes the second half of that suggestion,
the account-owned Git common directory,
and rejects "beside that claim" as a wording:
it keeps the claim and the binding as two files in one folder of its own,
for the reasons under "Its two files,
and which one is exclusive".

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

When each identity is written ([proposal](cli-git-zfs-worktree-design-proposal.md):383 to :387 and :405 to :407):

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

The claim's own sentence,
which this design keeps as written ([proposal](cli-git-zfs-worktree-design-proposal.md):373 to :376):

> Claim the destination as the account with an exclusive per-add record in the Git common directory.
> Write the proposed clone name and unique `git-worktree-clone-origin-<nonce>` snapshot name before any mutation beyond
> the claim.

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
Every bound below is either derived from a kernel constant named with it,
or left out with the measurement that would size it under "Measurements owed".

The wrapper never asks a person for a secret,
at `package/git-policy/cli/SPEC.md`:7348 to :7350:
"Avoid ever asking for someone to type a username or password."
and "Passphrases count too (Recommended)".
This record needs no authority it does not have,
so it never prompts.

## The wrapper's own records this design reuses

The wrapper already keeps three kinds of small durable record in the Git common directory,
and this design reuses their primitives and their grammar before proposing anything new.

- The destination record `<id>.add`,
  published whole by a rename and flushed by nobody,
  with `key=value` lines and a `phase=` line,
  written before real Git runs and rewritten as the add learns more (`package/git-policy/cli/SPEC.md`:4400 to :4408,
  `src/native/worktree/worktree_add_binding.rs`:262 to :280).
  Its identity fields are an `ObjectId` per directory and the pointer's bytes
  in lowercase hexadecimal (`worktree_add_binding.rs`:96 to :114).
- The per-add stage claim `<id>.stage`,
  whose name carries no owner and whose last line does,
  so a command killed between the claim and its bytes is a claim no owner owns (`SPEC.md`:6252 to :6272,
  `src/native/worktree_copying/worktree_copy_stage_claim.rs`:380 to :407).
- The slot identity store,
  one file per item,
  each published with an exclusive creation and one link,
  no lock,
  and with a bound computed from the kernel's name and access-list limits (`SPEC.md`:5525 to :5546,
  `src/native/worktree_clone_mount/slot_identity_store_publication.rs`,
  and [`cli-git-zfs-slot-identity-store-per-slot-files-design.md`](cli-git-zfs-slot-identity-store-per-slot-files-design.md)).

It also has one record *inside* a worktree's registration and says why that is safe:
`monochromatic-git-policy-cli-worktree-added`,
whose module says "The registration's directory is deleted with the worktree,
so the record needs no removal of its own" (`src/native/worktree/worktree_added_by_the_wrapper.rs`:1 to :26).
This design's record cannot live there,
for the reason "What Git does to a file it does not know here" measures.

The primitives it reuses rather than writes:

- `take_claim` and `live_owner_of` of `src/native/lock/single_owner_claim.rs`,
  the wrapper's lock-free claim:
  one numbered generation file published with one exclusive `link`,
  `EEXIST` deciding the winner,
  an owner named by process birth and PID namespace,
  a newer generation taken only when the older records a release or a dead owner,
  the newest generation never removed,
  and both the file and the folder flushed (`single_owner_claim.rs`:1 to :40 and :370 to :404).
- `PRIVATE_DIRECTORY_MODE` and `PRIVATE_FILE_MODE` of `src/native/foundation/private_storage.rs`:82 to :86,
  and `replace_private_file` at `src/native/foundation/private_storage_replacing_a_record.rs`:50 to :59.
- `create_new_entry`,
  `fill_entry_and_sync`,
  `link_entry`,
  `remove_entry` and `sync_folder` of `src/native/foundation/publishing_entries_in_one_open_folder.rs`:1 to :100,
  which is how the store publishes a record that replaces nothing.
- `DatasetGuid` (`src/native/worktree_clone_mount_protocol/dataset_guid.rs`:94 to :129):
  a GUID's canonical spelling is its decimal digits with no sign and no leading zero.
- `RequestNonce` (`src/native/worktree_clone_mount_protocol/request_nonce.rs`:1 to :20):
  32 lowercase hexadecimal digits.
- `ObjectId` (`src/native/subvolume/fs_beneath.rs`:85 to :114):
  `device:inode:seconds:nanoseconds`,
  which is what the btrfs binding records for both directories it binds.
- `slot_name_refusal` (`src/native/worktree_clone_mount/slot_name.rs`:22 to :40):
  a slot name is one plain folder name,
  with no separator,
  no leading dot and no space or control character.

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
and `git worktree remove` calls `delete_git_dir` whatever the force level (`builtin/worktree.c`:1466 to :1474).
No file inside a registration outlives the worktree's removal.

F3.
`git worktree prune` prunes with `expire = TIME_MAX` when no `--expire` is given (`builtin/worktree.c`:211 to :259),
and it prunes an entry whose `gitdir` file is missing,
or whose `gitdir` names a place that does not exist and whose registration holds no `index`,
or an `index` no newer than the expiry (`worktree.c`:960 to :1022).

F4.
`git gc` runs `git worktree prune --expire <gc.worktreePruneExpire>`,
whose default is `3.months.ago` (`builtin/gc.c`:144,
:181 to :183,
:338 to :347 and :718 to :719).

F5.
`git worktree list` reads `<common>/worktrees` only (`builtin/worktree.c`:211 to :244);
no Git command lists,
reads or writes any other folder of the common directory.

F6.
Measured,
this round,
on Git `2.56.1`,
in two runs that agree arm for arm.
A file in a new folder beside the copy journal's folder and a file inside a registration were placed in each arm,
and each command's effect recorded.

Run A ran plain `git` on this host,
which is the wrapper `monochromatic-git-policy-cli 0.6.1-alpha.44` shadowing Git `2.56.1`,
in throwaway repositories whose every `git` run had its own `HOME`,
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
in one throwaway container,
with `podman run --rm --interactive --network none --cpus 2 --memory 512m --pids-limit 128`,
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
  This is F1's counter,
  measured rather than read.

So every Git claim about these four commands is decided by run B and by the source at `v2.56.1`,
and run A agrees with it.
The transcripts are this round's `/home/user/temp/agent/zfs-add-binding-record-design-2026-11-22/keep/`.

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
A flush on a ZFS file is a ZIL commit for that dataset,
not a commit of a whole filesystem:
`zfs_fsync` calls `zil_commit` (`module/zfs/zfs_vnops.c`:110 to :122,
OpenZFS `zfs-2.4.4`).
Directory operations are committed to the ZIL only when the dataset's `sync` property
is `always` (`module/os/linux/zfs/zfs_vnops_os.c`:866,
:1228,
:1423,
:1554,
:2668,
:3273 and :3473,
each guarded by `os_sync == ZFS_SYNC_ALWAYS`),
and a transaction group is committed at most `zfs_txg_timeout` seconds after its first change,
whose default is 5 (`module/zfs/txg.c`:116,
OpenZFS `zfs-2.4.4`).

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
and it stands on the add's waiting path" (`SPEC.md`:4400 to :4420).
That reason names btrfs,
and this record lives on whichever filesystem holds the checkout,
which for this route is a ZFS dataset.

## Assumptions

A1.
The checkout's Git common directory is on the checkout dataset,
because `.git` is in the checkout's own folder and the route gives the checkout a dataset of
its own ([proposal](cli-git-zfs-worktree-design-proposal.md),
"Dataset and path layout").
Nothing measured this round;
the route's step 19 migration creates that dataset and is not built.

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
Which command performs that settle is not this design's.

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
The sibling designs named under "Dependencies on the sibling designs" place their records in other folders or
under other names,
and this design's folder holds only its own four name forms.

## Limits

L1.
Nothing in this design was run against ZFS.
The record's publication,
its flush,
the inventory it builds and the resume states it describes are unmeasured on any pool;
M1 to M6 say what would measure each.

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
A power cut is not a killed process.
A killed process loses nothing of the record,
because the page cache keeps it.
A power cut can lose the newest name even after a flush,
because OpenZFS logs directory operations to the ZIL only for a dataset whose `sync` property is `always` (F12),
and then the reader meets a clone whose record never became durable,
which is the orphan case "The record's end" describes.

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
F6 measured `git 2.56.1` on this host's btrfs-backed scratch folder,
not on ZFS.
Git's own logic does not depend on the filesystem,
but the link,
rename and flush calls this design makes do,
and those are M5.

L9.
The record names the repository by its common directory's path and `ObjectId` (F14,
`fs_beneath.rs`:85 to :114).
A restored or recreated `.git` answers another `ObjectId`,
so its records refuse.
That is intended and it is a limit:
a person who rebuilds `.git` from a backup must settle the records by hand or with `doctor`'s printed steps.

L10.
No configuration file,
environment variable or flag changes this design's home,
names,
modes or bounds.

## The home

The record lives in one folder of the Git common directory,
account-owned,
with no configuration:

```text
<git common dir>/monochromatic-git-policy-cli-worktree-clone/v1/
```

The folder's name carries the program,
as the wrapper's other record folders do (`SPEC.md`:4400 to :4404),
then the route it serves (`worktree-clone`,
the route's own word in [proposal](cli-git-zfs-worktree-design-proposal.md),
"Add and Git registration"),
then a version,
so a later grammar cannot be read as this one.
It sits beside the copy journal's folder (`<common>/monochromatic-git-policy-cli-worktree-copy/v1`,
`worktree_copy_journal.rs`:130 to :136) and never inside it,
because that folder's sweep reads every `<id>.add` and `<id>.stage` it finds there and would take this design's files
for an interrupted ignored-state copy (`SPEC.md`:4840 to :4890).

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
as the slot identity store refuses a record filed under the wrong name (`slot_identity_store_records.rs`:147 to :160).

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
"Choices this design marks for the project's veto",
V2,
lists the alternatives with their costs.

## Its two files, and which one is exclusive

The claim and the binding are two files,
and the claim is the exclusive one.

The claim is exclusive because of the kernel call that publishes it,
not because of a lock:
`take_claim` writes a private file and gives it the next generation's name with one `hard_link`,
so exactly one command creates `<slot>.claim.<n + 1>`,
and every other command's link answers `EEXIST` and sends it back to read the winner (`single_owner_claim.rs`:1
to :40,
:236 to :260 and :370 to :404).
A claim whose owner is alive stops the second command with the owner's process id;
a claim whose owner is dead,
or that records a release,
lets the next command take the next generation.
The newest generation is never removed,
so no two commands can create one number,
and a command that took a number another command has overtaken withdraws it.

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
than merged (`publishing_entries_in_one_open_folder.rs`:1 to :100).

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

## The record family

### The claim

`<slot>.claim.<generation>` is `take_claim`'s own record and stays that:
a header line,
then either `released` or the owner's `pid=`,
`birth=` and `namespace=` lines (`single_owner_claim.rs`:163 to :199).
This design adds no field to it.
Its generation is the only ordering evidence among operations on one slot,
and the binding's `claim=` line says which generation wrote the binding's current bytes.

### The binding

`<slot>.binding` is the record the proposal calls the durable binding.
Its grammar is the wrapper's own `key=value` grammar (`worktree_add_binding.rs`:53 to :66),
one header line,
one field per line,
written in one fixed order:

```text
# monochromatic-git-policy-cli zfs worktree clone binding, version 1
claim=<generation>
nonce=<32 lowercase hexadecimal digits>
phase=<claimed|registered|snapshotted|cloned|mounted|filled>
repository=<lowercase hexadecimal of the common directory's path bytes>
repositoryIdentity=<device>:<inode>:<seconds>:<nanoseconds>
registration=<lowercase hexadecimal of the registration's path bytes>
registrationIdentity=<device>:<inode>:<seconds>:<nanoseconds>
slot=<lowercase hexadecimal of the slot's path bytes>
slotIdentity=<device>:<inode>:<seconds>:<nanoseconds>
pool=<pool's dataset name>
poolGuid=<decimal>
originDataset=<dataset name>
originDatasetGuid=<decimal>
originSnapshot=<snapshot name>
originSnapshotGuid=<decimal>
originSnapshotCreatetxg=<decimal>
cloneDataset=<dataset name>
cloneGuid=<decimal>
cloneCreatetxg=<decimal>
pointer=<lowercase hexadecimal of the `.git` file's bytes>
```

A field whose fact is not confirmed yet is absent,
and the phase says how far the add got.
A field present is a fact the writer confirmed before that write,
never a guess,
and a line whose key this release does not know refuses the record rather than being skipped.

`phase` is the last step whose object was confirmed,
in this order:
the claim taken;
Git's registration made;
the checkout snapshot made;
the clone made;
the mount verified at the slot in the caller's namespace;
the pointer installed and the checkout verified.
The proposal's "Mark the binding usable only after the fill
is verified" ([proposal](cli-git-zfs-worktree-design-proposal.md):407) is the `filled` write,
and no earlier write says usable.

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
`bounds` is the clone dataset itself.

`entries=` is the completeness count:
a reader counts the `entry=` lines and refuses a file whose count disagrees,
so a truncated inventory is refused rather than mistaken for a short history.
Nothing else about the count is a bound:
the number of an origin's snapshots is no kernel constant (F9,
F11).

`kind=` must agree with the name the file was found under,
as the slot store requires of a record's key (`slot_identity_store_records.rs`:147 to :160),
so a copied file refuses under the wrong name.

### The fields, with their types and their encodings

Every type below is already the wrapper's,
so a value cannot have two spellings:

- A path is its bytes in lowercase hexadecimal,
  which is how the btrfs binding writes a destination and a registration (`worktree_add_binding.rs`:96 to :114).
  The encoding exists because a path may hold a newline,
  and a newline would forge a line of this record.
- A ZFS name is written verbatim,
  because the kernel's own charset holds no newline and no `=` (F9),
  so the line's first `=` separates the key from the value and nothing in the value can add a line.
- A GUID is `DatasetGuid`'s canonical decimal spelling,
  with no sign and no leading zero (`dataset_guid.rs`:94 to :129).
- A `createtxg` is the same canonical decimal spelling,
  because the property is an unsigned 64-bit number printed as decimal (F11).
- A folder identity is `ObjectId`'s `device:inode:seconds:nanoseconds` (`fs_beneath.rs`:85 to :114).
- The nonce is `RequestNonce`'s 32 lowercase hexadecimal digits (`request_nonce.rs`:1 to :20).
- The claim generation is decimal digits with no leading zero,
  which is the generation file's own name (`single_owner_claim.rs`:206 to :213).
- The pointer is the `.git` file's bytes in lowercase hexadecimal,
  which is how the btrfs binding records it (`worktree_add_binding.rs`:110 to :114).
- The phase is one of the six words above and nothing else.

### The size bound and its basis

No file of this family is bounded in total,
and each line and each fixed part is bounded from a kernel constant:

- One binding line is at most a path's hexadecimal form,
  `2 × 4096 = 8192` bytes (F14,
  `PATH_MAX` 4096),
  and the binding's whole fixed part is the sum of these terms:
  3 paths at 8192 bytes,
  the pointer at `2 × (8 + 4096) = 8208` bytes,
  3 folder identities at 61 bytes,
  4 ZFS names at 255 bytes (F9),
  8 numbers at 20 bytes,
  the nonce at 32,
  the phase at 6,
  the claim at 20,
  the header at 63,
  and 12 bytes of keys and separators per line.
  The sum is 34,206 bytes,
  and the constant is computed from those named terms the way the slot store computes
  its record's bound (`slot_identity_store_reading.rs`:63 to :85),
  so changing a term changes the bound.
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
one per add and destroyed by that add's removal's origin cleanup ([proposal](cli-git-zfs-worktree-design-proposal.md):945 to :975),
and any snapshot a person takes.
The measurement that would size a cap,
if the project wants one,
is M3.

### An example

One add at `.claude/worktrees/add-cache-layer` in the checkout `tank/monochromatic-git-checkout`,
after its mount is verified and before the fill:

```text
# <common>/monochromatic-git-policy-cli-worktree-clone/v1/add-cache-layer.binding
# monochromatic-git-policy-cli zfs worktree clone binding, version 1
claim=1
nonce=9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f
phase=mounted
repository=2f686f6d652f757365722f6d6f6e6f6368726f6d617469632f2e676974
repositoryIdentity=2049:131074:1763750400:123456789
registration=2f686f6d652f757365722f6d6f6e6f6368726f6d617469632f2e6769742f776f726b74726565732f6164642d63616368652d6c61796572
registrationIdentity=2049:131081:1763750412:987654321
slot=2f686f6d652f757365722f6d6f6e6f6368726f6d617469632f2e636c617564652f776f726b74726565732f6164642d63616368652d6c61796572
slotIdentity=2049:131089:1763750420:135792468
pool=tank
poolGuid=17293822569102704641
originDataset=tank/monochromatic-git-checkout
originDatasetGuid=4815162342109876543
originSnapshot=tank/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f
originSnapshotGuid=16371409254608711923
originSnapshotCreatetxg=121293
cloneDataset=tank/monochromatic-git-worktree-clone-datasets/add-cache-layer
cloneGuid=4821193048571903102
cloneCreatetxg=121294
pointer=6769746469723a202f686f6d652f757365722f6d6f6e6f6368726f6d617469632f2e6769742f776f726b74726565732f6164642d63616368652d6c617965720a
```

and its origin inventory,
whose 25 entries are the checkout's hourlies at or below that `createtxg`:

```text
# <common>/monochromatic-git-policy-cli-worktree-clone/v1/add-cache-layer.origin-inventory
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

## The phases, and a kill at each step

### The five resume states and the phases that leave each

The five states are the proposal's ([proposal](cli-git-zfs-worktree-design-proposal.md):424 to :426).
Each row of this list names the state,
the phases a kill can leave it at,
and the live facts that prove which state it is.
The phases may lag the live state by the step in flight,
and never lead it.

- A claimed empty slot:
  phase `claimed`,
  with no registration at the recorded `registration=` path and no dataset at `cloneDataset=`.
  Proven by the slot's folder being absent or empty and Git's registration folder absent.
- Registration without a dataset,
  including a possible origin only:
  phases `claimed` and `registered`.
  Proven by the registration existing whose `gitdir` names the slot's `.git` path and
  whose identity equals `registrationIdentity=`,
  with no clone dataset at `cloneDataset=`.
  With phase `snapshotted` the origin snapshot at `originSnapshot=` exists and its GUID equals `originSnapshotGuid=`;
  that is the "possible origin only" arm.
- An unmounted clone:
  phases `registered`,
  `snapshotted` and `cloned`.
  Proven by the clone dataset existing with `cloneGuid=` and no mount at the slot whose mount source is that dataset.
- A mounted but unfinished clone:
  phases `cloned` and `mounted`.
  Proven by the mount at the slot naming the clone dataset,
  with the slot's identity differing from the checkout's underlay.
- A verified checkout:
  phase `filled`.
  Proven by the registration's back-link,
  the pointer's bytes equalling `pointer=`,
  and the checkout's index and tracked content matching what the fill verified.

### What the writer confirms before each write

The writer's rule is one sentence:
it writes a phase only after the object that phase names answers with the values it is about to record,
and it writes that phase before it takes the next step.
So the record never claims a fact that did not answer,
and a kill at any instruction leaves a phase at or behind the truth.

- The claim is taken,
  then the binding's first write carries the claim's generation,
  the nonce,
  the slot's path and identity,
  the proposed clone name and the proposed `git-worktree-clone-origin-<nonce>` snapshot name,
  with phase `claimed`.
  Nothing has mutated yet.
- Git's registration is made with `worktree add --no-checkout`.
  The `.git` pointer's bytes are read,
  the registration's `gitdir` is read back and must name that pointer,
  and both folder identities are read (the wrapper's own binding does exactly this,
  `worktree_add_binding.rs`:396 to :430).
  Then phase `registered`,
  with `registration=`,
  `registrationIdentity=` and `pointer=`.
- The checkout snapshot is made.
  Then phase `snapshotted`,
  with `originSnapshot=`,
  `originSnapshotGuid=` and `originSnapshotCreatetxg=` read back from the pool,
  and the origin inventory published.
- The clone is made.
  Then phase `cloned`,
  with `cloneGuid=` and `cloneCreatetxg=` read back,
  and the clone's initial inventory published before anything can snapshot that clone.
- The mount is verified at the slot in the caller's namespace.
  Then phase `mounted`.
- The pointer is installed and the checkout verified.
  Then phase `filled`,
  which is the proposal's "usable".

A kill between an object's creation and its phase write leaves the phase one step behind,
and the reader meets exactly the case the proposal allows for:
"If mutation succeeded but its identity was not recorded before interruption,
keep and report it unless identity can be independently recovered" ([proposal](cli-git-zfs-worktree-design-proposal.md):429 to :430).
The recovery each reader makes is:

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
- The clone's initial inventory is not rebuildable once the clone holds any snapshot of its own,
  which is why it is published in the same step that records the clone and before the mount.
  A reader that finds a clone,
  a phase `cloned` or later,
  and no clone inventory refuses to use that clone and reports it:
  the clone's provenance can no longer be separated from history another hand put there.
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
the slot is the recorded slot when its identity equals `slotIdentity=`;
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
  the folder flushed (`single_owner_claim.rs`:236 to :260).
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
  the staging name is removed and the folder flushed (`publishing_entries_in_one_open_folder.rs`:1 to :100,
  and the store's own use of it at `slot_identity_store_publication.rs`:225 to :260).
  A kill before the link leaves `.<name>.<32 hex>.partial`.

So two staging forms exist in the folder,
`.cli-git-claim.tmp.*` and `.<record>.<random>.partial`,
and no reader reads either as a record.
`doctor` removes both forms only when the folder holds no live claim,
and otherwise names them and leaves them,
because a writer holding the claim may own one.
That is a narrower rule than the store's,
whose doctor removes its two known forms unconditionally ([`../decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md),
"Who removes a killed writer's staging file"):
a ZFS add holds its claim for minutes across a snapshot and a clone,
where the store's writer holds nothing,
so the store's rule would break a running add.

### Durability, and why it differs from the wrapper's other add records

The wrapper's add records are flushed by nobody because the flush would be "a commit of the whole btrfs filesystem,
which waits behind every other writer's data,
and it stands on the add's waiting path" (`SPEC.md`:4416 to :4420).
On the filesystem this record lives on,
a flush is a ZIL commit for one dataset and nothing else (F12),
so the reason does not carry over,
and this record is flushed at every publication.

The second reason is what each fact would cost if lost.
The identities are already durable in the pool when they are recorded,
so the record must not be the weaker half;
and the clone's initial inventory cannot be rebuilt once the clone holds history of its own.
What a lost record would leave is a clone dataset nobody can prove,
which is the orphan case under "The record's end".

What the flush does not buy is the name's durability under a power cut:
OpenZFS logs directory operations to the ZIL only for a dataset whose `sync` property is `always` (F12),
and this design does not set that property,
does not ask for it,
and does not wait for a transaction group's commit (5 seconds by default,
F12).
A power cut can therefore leave a clone dataset whose binding record never became durable.
The design's answer is not a wait but an observation:
the record is flushed at every publication,
and the case where it is lost is the orphan case,
reported and never guessed at.
M1 measures what the flushes cost the add.

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
  or the file's name and its `slot=` disagree.
  A record copied from another repository or restored with a rebuilt `.git` refuses here (L9);
- oversized:
  a line over the bound or a file over the binding's bound is refused by size,
  with both numbers printed;
- wrongly owned:
  the file's owner is not this account's,
  or its mode lets another account write it.
  Under this project's threat model that is a mistake rather than an attack,
  and the record still decides which dataset a destruction may target,
  so the refusal stays and prints `chown`'s and the settling command's words;
- ahead of the live facts:
  a phase whose object does not answer,
  or answers with another identity.
  This is the one arm a reader meets when another hand destroyed an object (see "What no reader may conclude");
- missing its claim:
  a binding whose `claim=` generation does not exist as a file refuses,
  because the generations' liveness is the only evidence that no command is working the slot.

An add that finds no record takes the claim and writes one,
which is the normal path.

### The removal's lineage proof

The removal takes the claim,
then reads the binding and both inventories,
and refuses before its first delete when any of them is missing or unreadable.
It never rebuilds the origin inventory,
following the proposal's rule "never regenerate an authorized set from whatever names
are left" ([proposal](cli-git-zfs-worktree-design-proposal.md):957 to :959),
and never updates a recorded identity to match an external change ("Never update the recorded lineage to match
an external change",
[proposal](cli-git-zfs-worktree-design-proposal.md):880 to :881).
It uses the binding's fields exactly as the proposal's removal steps say:
the pool,
clone and origin GUIDs;
the origin snapshot's `createtxg`;
the clone's own `createtxg` to order its snapshots;
and the origin inventory for the third check ([proposal](cli-git-zfs-worktree-design-proposal.md):866 to :884).

### `doctor`

`doctor` lists the folder once per run,
which is the one listing on any path,
and reports:

- the number of claims and of records,
  and each claim whose owner is alive,
  with its process id,
  which is `live_owner_of`'s answer (`single_owner_claim.rs`:406 to :430);
- each slot with a binding and no clone,
  each clone with a binding and no registration,
  and each registration with no binding,
  naming the fields that disagree;
- each inventory whose `entries=` disagrees with its lines,
  each record filed under a name its `slot=` or `kind=` refuses,
  each file whose owner or mode is not this account's,
  and each entry whose name is none of the four forms;
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
and that is a decision.
No forwarded command reads this folder,
so an add's record costs nothing on any other command's path.
The one exception the design names is `live_owner_of`,
which an add or a removal calls before taking a claim,
and which reads one claim file when the folder exists.

## The record's end

A removal writes nothing into the binding.
Its own plan is another record,
whose home is the sibling design's ("Dependencies on the sibling designs"),
and the binding's phase stays at `filled` for the whole removal.
What identifies the removal's state is the plan's presence and the live facts.

The order that ends the record is:

1. The removal takes the claim and proves lineage from the binding and both inventories.
2. It destroys the clone's own snapshots and the clone head,
  and proves absence by the recorded GUID:
  the name is absent and no child of
  the pinned clone parent carries `cloneGuid=` ([proposal](cli-git-zfs-worktree-design-proposal.md):956 to :962).
3. It settles the registration,
  which Git removes with its folder (F2).
  The binding is not inside it,
  so it survives.
4. It publishes the origin cleanup record that the sibling design homes,
  which carries `originDataset=`,
  `originDatasetGuid=`,
  `originSnapshot=`,
  `originSnapshotGuid=` and `originSnapshotCreatetxg=`,
  or confirms that a record already published carries them.
5. Only then does it remove `<slot>.binding`,
  `<slot>.origin-inventory` and `<slot>.clone-inventory`,
  and release its claim,
  which leaves the newest generation file as a released claim.
  It prints what it removed.

If step 4 cannot be completed,
or the origin cleanup is blocked,
the binding stays.
That is deliberate:
the binding is then the only home of the origin's identity,
and its presence is what a later run and `doctor` read as "clone gone,
origin retained",
which is one of the proposal's resume states ([proposal](cli-git-zfs-worktree-design-proposal.md):962 to :968).

### What a record whose clone is gone means

Three causes,
told apart by what else stands:

- The clone is gone,
  the plan record exists and the removal intent stands:
  a removal that has not finished.
  The record is the removal's left hand until step 4 lands,
  and `doctor` prints the standing intent and the origin names.
- The clone is gone,
  the plan exists and reports the head's destroy as unresolved:
  the same,
  with a named unknown outcome.
  Nothing in the binding is regenerated and nothing is settled on a guess.
- The clone is gone and no plan exists:
  the clone was destroyed outside this wrapper.
  The binding proves which GUID is missing,
  so the reader can require "the name is absent and no child of the pinned parent carries the recorded GUID" before
  it settles anything,
  and it reports the case as an outside change.
  It never removes the binding on the strength of a name being absent alone.

A binding whose clone is gone and whose absence cannot be proven is kept,
and the slot it names refuses the next add until a person settles it.
That is question Q2 for the project,
because it trades a blocked slot against a guessed settlement.

## What Git does to a file it does not know here

Measured,
this round,
exactly as F6 records it,
in run B through real Git `2.56.1` inside the gate image and repeated in run A through the wrapper,
arm for arm.
The two candidate places were measured together in each arm,
and the result decides the home:

- Inside a registration (`<common>/worktrees/<id>/<slot>.binding`),
  the record is deleted by `git worktree remove` whatever the force level (F2,
  arm 2b),
  by an explicit `git worktree prune` the moment the slot's `.git` pointer is missing (F3,
  arm 3),
  and by `git gc --prune=now` once the registration's `index` is absent or older than `gc.worktreePruneExpire` (F3,
  F4,
  arms 5,
  7 and 8).
  The very state that makes the record most needed,
  an unmounted clone whose underlay pointer was lost,
  is the state in which the explicit prune deletes it at once.
- Beside the copy journal's folder (`<common>/monochromatic-git-policy-cli-worktree-clone/v1/...`),
  every one of those four commands left both files exactly as they were,
  and `git worktree repair` left them too (arms 1,
  2b,
  3 and 7).

Git never lists,
reads or writes that folder (F5),
and it creates `<common>/worktrees/` and nothing else under the common directory for a worktree.

The one Git behaviour the home must live with is F1's counter,
which arm 8 of run B measured:
a stale registration of the same name makes Git name the new one `<name>1`.
The design answers it by recording the registration's path and identity from Git's own answer,
never by deriving it from the slot,
and by refusing a binding whose `registration=` no longer answers with `registrationIdentity=`.

## Which step of the plan builds which part

- Step 10 builds the home,
  the claim's use of `take_claim`,
  the binding's phases and fields,
  the two inventories and the resume rules:
  the folder and its modes,
  the four name forms,
  the publication and flush of each write,
  the checks a resumed add makes,
  and the refusals.
  Its "Design sections" gain this document,
  and `SPEC.md`'s "### The records an add writes for itself" gains the claim and binding records,
  as the plan already says ([plan](cli-git-zfs-worktree-implementation-plan.md):1439 to :1443).
- Step 14 builds the removal's reading of the binding and both inventories,
  the refusals of "The removal's lineage proof",
  and the claim the removal takes.
- Step 16 builds the record's end:
  absence proven by the recorded GUID,
  the hand-over to the origin cleanup record,
  the removal of the three files,
  and the released claim.
- Step 18 builds `doctor`'s listing,
  the four checks of "doctor" and the staging-name removal rule.
- Step 12 needs no part of it:
  a pruned or moved worktree is read from Git's own registrations,
  and the binding is not consulted for it.

## The tests a build must have

Each test names the race or the state it forces and how it forces it without timing luck.
The wrapper's own seams are the model:
a `thread_local` hook between two steps,
as the store's publication has (`slot_identity_store_publication.rs`:395 to :420) and as `take_claim`
has (`single_owner_claim.rs`:352 to :356`).

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
`a_clone_whose_initial_inventory_is_missing_is_refused`:
phase `cloned` and no clone inventory refuses,
and the refusal names the clone.

T8.
`a_record_under_another_slots_name_refuses`,
`a_record_naming_another_repository_refuses`,
`a_record_naming_another_slot_identity_refuses`:
three planted records,
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
an arm where step 4 fails,
and the three files stand;
an arm where it succeeds,
and they are gone with the released claim left.

Guest tests,
because they need a real pool,
a real mount or the root handler:

T17.
`the_origin_inventory_records_the_hourlies_and_a_bookmark`:
a fixture pool whose origin dataset holds the fixture's hourlies and a bookmark;
the record's 11 fixed lines and its entry lines equal the `zfs list` answer,
`entries=` matching,
and a bookmark's GUID and `createtxg` equal the snapshot's (F10).

T18.
`a_kill_between_the_clone_and_its_record_is_resumed`:
the add is killed by the planned seam after `zfs clone` returns and before the phase write;
the resumed add records the recovered GUID and finishes;
the clone the fixture holds is the one the record names.

T19.
`the_record_survives_a_prune_while_the_clone_is_unmounted`:
a real `git worktree prune` after deleting the underlay pointer,
with the record in the place this design chooses;
the record and the clone dataset stand,
and the registration does not (the measured arm 3,
repeated on the route's own dataset).

T20.
`the_claim_and_the_binding_are_durable_across_a_power_cut_simulated_by_dropping_the_page_cache`:
the fixture flushes,
then the guest's own simulated crash drops the caches;
the record read back is the one written,
and the name durability F12 predicts is asserted rather than assumed.

T21.
`a_record_on_the_checkout_dataset_is_read_after_the_checkout_is_remounted`:
the record is written,
the checkout is unmounted and mounted by the guest's own ZFS,
and the record reads whole.

T22.
`ENOSPC_at_the_snapshot_leaves_the_claim_and_no_orphan`:
the fixture fills the pool,
the snapshot refuses,
and the claim,
the slot and the registration stand with nothing else made.

Every guest test prints `SKIP` with the variable's name outside the guest,
in the shape `binary_zfs_adoption_guest_tests.rs`:52 to :60 already uses,
so the gate says what it did not run.
T1 to T16 run in the package's own suite and need no guest.

## Measurements owed

M1.
The flush cost of one add's record writes.
Measure,
in the guest,
the wall time of the six publications this design makes with a flush against the same six without one,
both arms interleaved in one round,
and report the ratio with its spread.
This sizes the design's choice to flush every write (rule `LQM`,
and `TVC`).

M2.
The inventory's cost and its widest real shape.
Measure,
in the guest,
the time to build the origin inventory from `zfs list` for an origin holding the fixture's hourlies and one bookmark,
and the bytes it took;
then repeat with 500 snapshots and with 5,000,
and report both counts' bytes and times,
so the size an inventory reaches is measured and not assumed.

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
the store's design requires on the ZFS dataset that holds `.git`,
which F6 measured only on this host's btrfs and on the gate image's overlay-backed container folder.

M6.
The name's durability against a power cut.
Measure,
in the guest,
whether a directory `fsync` after a link or a rename makes the name survive the guest's simulated power loss,
at `sync=standard` and at `sync=always`,
which settles whether L4's orphan case is a window of 5 seconds or of none.

## Dependencies on the sibling designs

- "The add-time Git lock of a ZFS clone worktree" decides where the lock's record lives.
  This design's folder is its own,
  and its four name forms are reserved;
  if that design chooses this folder,
  its name must be a fifth form and `doctor`'s listing must know it.
  The assumption is A9.
- "Where a removal's persisted destruction plan lives,
  and its format" decides the plan this design's removal reads beside the binding,
  and whose presence tells "The record's end"'s three cases apart.
  This design assumes the plan lives in another folder,
  or in this one under a name that is not one of its four.
- "The record that carries origin cleanup after a registration is settled" decides the record that must exist before
  this design's binding is removed,
  and it must carry the origin's dataset name,
  GUID,
  snapshot name,
  GUID and `createtxg`.
  If it instead decides to reuse `<slot>.binding` as its carrier,
  then the binding stays until cleanup finishes and gains a field;
  the reconciliation point is step 5 of "The record's end".
- "The dirty-main restore on rung 1 and its frozen proof" decides where the frozen proof of
  the source's content lives.
  This design deliberately does not carry it:
  the binding records identities of objects,
  never content,
  and a resumed add that finds its record has no proof to compare the clone against.
  If the proof needs to survive a kill,
  its home is that design's,
  and a resumed add must refuse when it is gone.
- "The deferred-destroy checks of a removal" reads the origin identity this design records,
  and gives it no new field.
- "The forced arm of a removal" and "What a cancelled,
  timed-out or killed removal leaves" both end or cancel a removal that holds this design's claim,
  so each must say that it releases the claim;
  this design assumes it does,
  because a claim whose owner is dead is taken over anyway.
- "What re-applies private propagation after a replacement mount of the checkout" and "Which triggers
  a lost last event waits for and how nudges are merged" share no fact with this record.

## Choices this design marks for the project's veto

Each entry is a writer's choice adopted so the design is whole.
Each gives the options,
an example for each,
a pros and cons list,
and a ranking with the reason for each adjacent pair.

V1.
The folder's name and its version segment.

- A,
  `<common>/monochromatic-git-policy-cli-worktree-clone/v1/`:
  example `.git/monochromatic-git-policy-cli-worktree-clone/v1/add-cache-layer.binding`.
  Pros:
  the program's name,
  the route's name and a version,
  as the copy journal has;
  the route's own word "clone" keeps the two folders apart in a person's eye.
  Cons:
  a long path,
  and a second folder whose name resembles the copy journal's.
- B,
  `<common>/monochromatic-git-policy-cli-worktree-binding/v1/`:
  pros:
  names the record's own word,
  so no reader confuses it with the copy journal.
  Cons:
  "binding" is the wrapper's word for two other records (`worktree_add_binding.rs`),
  so a person may expect a registration's binding.
- C,
  inside the copy journal's folder:
  `<common>/monochromatic-git-policy-cli-worktree-copy/v1/<slot>.zfs-binding`.
  Pros:
  one folder for the wrapper's per-worktree records.
  Cons:
  that folder's sweep reads every `<id>.add` and `<id>.stage` there and would need a new exclusion for this name,
  and its version is the copy's version.
- D,
  inside the program-named folder the wrapper already has:
  `<common>/monochromatic-git-policy-cli/worktree-clone/v1/`.
  Pros:
  one program's folder,
  which the wrapper already writes `removal-snapshots/` into (`SPEC.md`:16475 to :16478).
  Cons:
  `doctor` and every reader would list a folder that other commands' records share,
  and the wrapper's own top-level record folders (`monochromatic-git-policy-cli-worktree-copy/`) are
  the closer precedent.
- Ranking:
  A,
  then B,
  then C,
  then D.
  A beats B because the route's name in the folder keeps the ZFS route's records together beside the sweep and
  the receipts that also serve it.
  B beats C because C puts this record under a sweep it must then teach to skip it,
  which is a rule with no second purpose.
  C beats D because a folder of its own is what makes one listing exact,
  and D's folder is shared with snapshots this design never reads.

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
  and a reader must check that the binding's `claim=` generation exists.
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
  and its temp names appear in this folder.
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
  six flushes on the add's path,
  each a ZIL commit (F12),
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
  litter waits while any claim in the folder is live.
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
  removed after the origin cleanup record exists (adopted).
  Pros:
  the origin's identity is never homeless,
  and a blocked cleanup leaves a record `doctor` can read.
  Cons:
  a removal cannot fully clean the slot until the sibling's record is published.
- B,
  removed when the clone is destroyed and the registration settled.
  Pros:
  the slot is clean at settlement.
  Cons:
  the origin's identity would be held only by the sibling's record,
  so this design would have to trust a record it does not own before it exists in the build order.
- C,
  never removed,
  with a `settled` phase.
  Pros:
  the removal's history survives in the slot's record.
  Cons:
  one file per slot forever,
  and a slot's reuse must distinguish a settled record from a standing one.
- Ranking:
  A,
  then B,
  then C.
  A beats B because step 16's settle happens before the origin cleanup may finish,
  and A keeps the origin provable across that gap.
  B beats C because C turns every slot into permanent state that `doctor` must explain.

## Questions only the project can answer

Q1.
An add that finds another add's unreadable record for its slot:
refuse,
or move it aside and continue?

- A,
  refuse (recommended):
  the person sees the file,
  the two settling commands and `doctor`,
  and nothing changes.
  Example:
  `add-cache-layer.binding` holds a line this release cannot read;
  `git worktree add` at that slot stops before Git runs and prints the path.
  Pros:
  no record is destroyed,
  and a record that might name a standing clone is never discarded.
  Cons:
  the slot is blocked until a person acts.
- B,
  move it aside into `unreadable/` and continue,
  as the copy journal's pass does.
  Example:
  the same file is renamed into `.unreadable/` and the add runs.
  Pros:
  the slot stays usable.
  Cons:
  if the record named a standing clone,
  its clone dataset and origin snapshot become unprovable,
  and the origin stays pinned with no record naming it.
- Ranking:
  A,
  then B.
  A beats B because this record's one job is proving a clone,
  and B's convenience is bought with the only evidence that could prove it.
  This is a values question,
  not a measurement.

Q2.
A binding whose clone is gone and whose absence cannot be proven:
keep the slot blocked,
or let an explicit command settle it?

- A,
  keep it blocked and report it (recommended).
  Example:
  `cloneGuid=4821...` matches no child of the clone parent,
  and the last snapshot of it cannot be listed;
  the slot refuses a new add and `doctor` prints the recorded identities and the two checks that failed.
  Pros:
  no destroy is authorized from a guess.
  Cons:
  the slot waits for an administrator.
- B,
  print an explicit settling command that removes the binding after a person confirms.
  Example:
  the same,
  plus `git-policy-cli settle-worktree-clone-record <slot>` that a person runs.
  Pros:
  the slot can be recovered without an administrator.
  Cons:
  a command that removes evidence is one typo away from a clone nobody then cleans up.
- Ranking:
  A,
  then B.
  A beats B because the blocked state costs one slot and B's costs are unrecoverable.
  Which is preferred is the project's call.

Q3.
One released claim file per slot name,
left forever,
or removed when a removal settles?

- A,
  leave it (recommended).
  Example:
  `add-cache-layer.claim.3` holds `released` after its removal.
  Pros:
  follows the reused primitive's rule that the newest generation is never removed.
  Cons:
  one small file per slot name ever used.
- B,
  remove it at settlement.
  Example:
  the same file is deleted with the record,
  and a later add starts at generation
  1. Pros:
  no litter.
  Cons:
  it gives up the number monotonicity the primitive's rule exists for,
  and needs an argument about stale holders.
- Ranking:
  A,
  then B.
  A beats B because the file costs 60 bytes and B costs a correctness argument.
  This is a cost question,
  and its answer changes no other part of the design.

## What this design does not cover

- The destruction plan's home and format,
  the origin cleanup record's,
  and the add-time Git lock's:
  each is its own sibling design,
  named under "Dependencies on the sibling designs".
- The content proof of the clone's fill (the frozen index and tree comparison) and where it lives across a kill.
- The mount's own records:
  the intent property,
  the receipts and the readiness record are steps 7 and 8's.
- The claim's effect on the checkout's own conversion or migration,
  which uses the same primitive for a different item.
- Any root-side copy of the lineage identities,
  which the delegated route does not need (A2).

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
4265 to 4330,
4355 to 4425,
4840 to 4900,
5525 to 5560,
6234 to 6275 and 7340 to 7355;
and the wrapper's code under `package/git-policy/cli/src/native/`:
`lock/`,
`foundation/`,
`worktree/`,
`worktree_copying/`,
`worktree_clone_mount/` and `worktree_clone_mount_protocol/`.

Read at a tag,
in clones under this round's scratch folder:
Git `v2.56.1` at `03b4152` (`builtin/worktree.c`,
`builtin/gc.c`,
`worktree.c`,
`refs.c`) and OpenZFS `zfs-2.4.4` at `71a9f95` (`module/zcommon/zfs_namecheck.c`,
`module/zfs/dsl_bookmark.c`,
`module/zfs/zfs_vnops.c`,
`module/os/linux/zfs/zfs_vnops_os.c`,
`module/zfs/txg.c`,
`include/sys/fs/zfs.h`,
`man/man7/zfsprops.7`).

Ran,
twice over,
in the shape F6 records.
Run A:
the eight-arm probe with plain `git` `2.56.1` on this host,
where that name is the wrapper `monochromatic-git-policy-cli 0.6.1-alpha.44`,
each repository getting its own `HOME`,
`XDG_RUNTIME_DIR`,
`XDG_CACHE_HOME` and `XDG_CONFIG_HOME` under this round's scratch folder;
one commit per fixture made with `git write-tree`,
`git commit-tree` and `git update-ref`,
because the wrapper lands a commit through a resident service a scratch home does not run.
Run B:
the same arms with real Git `2.56.1` inside the gate image `localhost/git-policy-native-test:development` that
was already present,
in one `podman run --rm --interactive --network none --cpus 2 --memory 512m --pids-limit 128` container with
no folder of the home mounted and every repository made inside it;
no image was built,
pulled or removed.
Also `zfs list`,
`zfs get` and `stat`/`df` read-only on this host,
and `getconf PATH_MAX` and `getconf NAME_MAX`.

Not run:
any ZFS command that changes anything,
any mount,
any guest,
any container,
any service manager,
any `pnpm install`,
and any test suite.

## Revisions

- 2026-11-22T17:52:00-05:00:
  the first draft,
  written on `docs/planning-cli-git-zfs-add-binding-record-design` from `main` at `5dd8883a60`.
