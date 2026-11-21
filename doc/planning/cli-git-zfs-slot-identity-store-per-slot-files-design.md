# cli-git ZFS slot identity store as a folder with one file per slot

**Status:
design by an agent,
unapproved and non-final.**
Written 2026-11-21T16:32:25-05:00.
It answers every item that
[`doc/decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md)
left to a design,
and no review has read it.
The seven numbered sections here are those seven items,
in that order,
so a sentence that says "item 3" means section 3.
Every choice inside an item is proposed with its options and a ranking,
and none of them is decided until this design's own review accepts it.
Nothing here is built:
the code,
the `SPEC.md` passages and the plan amendments it names are the build's work,
and this round touches no `doc/handover/` file,
no `doc/decision/` file and no code.

## What this design carries out

The project answered "B:
folder,
one file per slot" on 2026-11-21,
and the decision record states the shape it accepted:
"The slot identity store,
which holds for each worktree slot the identities the attach checks compare against
(the folders' device,
inode,
owner,
permissions and access lists),
becomes a folder with one file per slot.
Each slot's file is made with `O_EXCL`,
and the store takes no lock."
It also accepted the cost,
in its own words:
"The single consistent read of every slot is given up.
Step 8's checks must be designed to need only what a per-slot layout can give."
Nothing of the ZFS route is released,
so no written store is migrated:
the store at
`/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities`
becomes a folder at that same path,
and a machine that never ran the root setup has neither shape to convert.

The repository's concurrency rule is the reason the folder was chosen,
and it is the rule this design must keep
(`AGENTS.md`,
rule `NLK`):
"Concurrency is lock-free first (atomic rename,
`O_EXCL`,
one owner per item);
an unavoidable lock covers the smallest unit,
never a whole repository,
with its reason written beside it."
The store's single file could not be made safe by an atomic rename,
because two first-recording sweeps that both rebuilt the file
would let the later rename record an identity taken from a folder that changed in between,
which is the refresh the accepted design forbids
(`package/git-policy/cli/SPEC.md:5475`,
and the design's own rule at
[`doc/planning/cli-git-zfs-worktree-design-proposal.md:611`](cli-git-zfs-worktree-design-proposal.md)):
"Never refresh a slot identity just because a new inode or permissions appeared there."
One file per slot removes that reason with the file:
a slot's record is published once and never replaced,
so no later writer can take an identity from a folder that moved under it.

## What was read, what was measured, what is assumed

### Read

- The decision record,
  [`doc/decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md),
  whole,
  including its list of what it leaves to a design.
- `package/git-policy/cli/SPEC.md:5348` to `:5509`,
  "The recorded identities store":
  the path and the one-record-per-line grammar (`:5350` to `:5392`),
  the device field's anonymity and the unmeasured reboot question (`:5394` to `:5406`),
  the one-mebibyte read bound and the append that may not outgrow it (`:5407` to `:5421`),
  the root-control checks (`:5423` to `:5440`),
  the write-once rule (`:5441` to `:5450`),
  the lock and its measured wait (`:5451` to `:5474`),
  the paragraph that calls the lock the route's one non-lock-free place (`:5475` to `:5489`),
  the two parent writers it names (`:5490` to `:5499`),
  and the one-store-one-checkout paragraph (`:5500` to `:5508`).
  Line `:5490` already states the split this design carries out:
  "The parent folder's record is written by the setup run (the plan's step 19),
  and only the administrator's adoption replaces it".
- The module the store lives in,
  `package/git-policy/cli/src/native/worktree_clone_mount/`:
  `slot_identity_store.rs` (`SlotIdentityStore::read_under_root_control`,
  `append_slot_identity_under_root_control`,
  `record_parent_identity_under_root_control`,
  `replace_the_parent_record`,
  `hold_the_store_lock`,
  `lock_a_store_that_is_there`,
  `check_the_store_fits_one_read`,
  `STORE_MAXIMUM_BYTES`,
  `STORE_LOCK_WAIT`,
  `STORE_LOCK_PAUSE`),
  `slot_identity_store_records.rs` (`STORE_HEADER`,
  `StoredIdentity`,
  `parse_records`,
  `slot_name_refusal`),
  `slot_identity_store_refusals.rs` (`StoreRefusal`),
  `clone_folder_identity.rs` (`CloneFolderIdentity::encode` and `decode`),
  and `slot_identity_store_tests.rs`.
- Step 5's mode,
  `package/git-policy/cli/src/native/worktree_clone_mount_handler/adopt_worktree_parent_mode.rs`
  (`recorded_slots`,
  `read_slot`,
  `adopt_at`),
  and the decision table it feeds,
  `parent_adoption_decision.rs`
  (`AdoptionFacts`,
  `SlotNotListed`,
  `SlotDiffers`,
  `decide_adoption`).
- `package/git-policy/cli/doc/slot-identity-store-lock-hold.md`,
  whole:
  its two measured runs,
  the `fsync` control,
  and the arithmetic behind `STORE_LOCK_WAIT`.
- The accepted design,
  [`doc/planning/cli-git-zfs-worktree-design-proposal.md`](cli-git-zfs-worktree-design-proposal.md):
  the recheck rule at `:425`
  ("Recheck recorded identities,
  the mount and the visible pointer before resuming"),
  the receipts' shape at `:564` to `:571`,
  and "Descriptor mount and unmount" at `:597` to `:623`,
  whose sentence at `:623` decides item 3's constraint:
  "Never overwrite surviving slot identities or implicitly adopt a replacement for an existing binding."
- The implementation plan,
  [`doc/planning/cli-git-zfs-worktree-implementation-plan.md`](cli-git-zfs-worktree-implementation-plan.md):
  step 4's "It also reads and rechecks the slot identity store,
  at this path" at `:597`,
  step 8's sweep,
  which records "the slot's first-observation identity before attach",
  and `doctor`'s report of "the recorded parent and slot identities with the failing field or component" at `:1636`.
- Issue 6327,
  now closed,
  whose body names the defect this design must not reintroduce:
  "`replace_the_parent_record` ... rewrites the store without the size check `append_under_the_lock` now has ...
  A store of exactly the bound (1 MiB),
  adopted with a wider parent record,
  ended at 1,048,985 bytes,
  and the next read refused it."
- The publication precedent in this package,
  named under "Precedent in this repository".

### Measured

One probe,
`probe_publish_primitives.py`,
kept in this round's scratch folder,
run on this host on 2026-11-21 in its own capped scope,
as the account,
inside
`/home/user/temp/agent/zfs-slot-identity-store-per-slot-files-design-2026-11-21/scratch/probe-home`,
which `findmnt` reports as `zpcachyos/ROOT/cos/home`,
file system `zfs`,
`rw,noatime,xattr,posixacl,casesensitive`.
It read:

- `/var/lib` is on `/`,
  source `zpcachyos/ROOT/cos/root`,
  file system `zfs`,
  device `31`,
  and `/home` answers device `57`,
  so the store's own device is not among the devices its records hold.
  `/var/lib/monochromatic-git-policy-cli` does not exist on this host.
- `openat(dir, ".", O_TMPFILE | O_DIRECTORY | O_RDWR, 0o600)` is accepted on that ZFS dataset,
  and the file it makes takes each of the two names it was given:
  `linkat(fd, "", dir, "<name>", AT_EMPTY_PATH)` answered zero,
  and `linkat(AT_FDCWD, "/proc/self/fd/<n>", dir, "<name>", AT_SYMLINK_FOLLOW)` answered zero,
  both as user `1000` and not only as root.
  The published file kept mode `0644` and read back its bytes whole.
- `renameat2(dir, staged, dir, "slot-a", RENAME_NOREPLACE)` where `slot-a` exists answered `EEXIST`
  and left the staged name in place;
  onto an absent name it published the staged bytes in one step and removed the staged name.
- `renameat2(dir, "slot-new", dir, "slot-b", RENAME_EXCHANGE)` answered zero,
  and each name held the other's bytes afterwards.
- A plain `renameat2(dir, staged, dir, "slot-b", 0)` replaced the target in one step.
- `openat(dir, "slot-a", O_CREAT | O_EXCL | O_WRONLY, 0o644)` where `slot-a` exists answered `EEXIST`.
- `getconf NAME_MAX` on that folder answered `255`;
  a name of 255 bytes was created and removed,
  and a name of 256 bytes was refused `ENAMETOOLONG`.
- `fsync` of a descriptor opened `O_RDONLY | O_DIRECTORY` on that folder answered zero.
- A `system.posix_acl_access` attribute of 65,532 bytes (8,187 named users) was accepted on a folder there and read
  back at that size,
  and 65,540 bytes was refused `E2BIG`,
  so the ceiling this kernel enforces is `XATTR_SIZE_MAX`,
  65,536 bytes of the whole attribute value including its four-byte version word.

### Assumed, not measured

- That the store's own dataset answers the same for `O_TMPFILE`,
  `linkat`,
  `renameat2` and a directory `fsync`.
  Both are ZFS datasets of one pool on this host and both are mounted `xattr,posixacl`,
  but the probe ran on `home`,
  because nothing on this host may write under `/var/lib` as the account.
  A guest arm or a gate image must settle it,
  and item 7 lists that arm.
- That `/var/lib/monochromatic-git-policy-cli` is created root-owned with mode `0755` by the setup run of plan step 19,
  which does not exist yet.
  This design fixes what that run must do and cannot cite a line that already does it.
- That a checkout's slot names stay under `NAME_MAX`.
  Git itself refuses a worktree path whose last component the filesystem refuses,
  and this design adds no new refusal for a name that fits.
- That two sweeps meeting on one slot is possible but not frequent.
  Nothing in this repository measures how often ZED delivers two triggers for one clone,
  and the design is safe at any rate.
- That `linkat` onto a name that is already taken answers `EEXIST`.
  The probe published two names it created and never aimed a link at a name that existed,
  and `linkat(2)`'s `EEXIST` is the documented answer the fallback's own arm relies on.

## Precedent in this repository

This design invents no primitive.
Each step this design orders is one this package already ships,
and the design's job is to pick among them and say why.

- `package/git-policy/cli/src/native/worktree_copying/worktree_copy_publish_file.rs`
  publishes one staged file at a destination name without a lock:
  `open_unnamed` makes a file with no name (`O_TMPFILE`),
  `link_unnamed` gives it the destination name and nothing else's
  (through `/proc/self/fd` first,
  then `AT_EMPTY_PATH`),
  and `move_temporary_into_place` is the fallback for a file that had to be filled under a name:
  `renameat2` with `RENAME_NOREPLACE`,
  then `linkat` and `unlinkat` where the filesystem refuses that flag.
  Its reason is this design's reason word for word:
  "A file created under its final name and filled afterwards exists,
  for as long as the copy runs,
  with a prefix of the bytes ... so a kill in that window leaves a file at that name that is not what the source holds".
- `package/git-policy/cli/src/native/foundation/private_storage.rs`
  writes a record whole or not at all:
  `write_then_publish`,
  `staging_beside` (a `.<name>.<random>.partial` name created `O_EXCL`,
  mode `0600`,
  never following a link),
  `persist_noclobber`,
  `read_regular_file` with a byte bound,
  and `sync_directory`,
  which opens a folder read-only and syncs it.
- `package/git-policy/cli/src/native/foundation/private_storage_replacing_a_record.rs`
  replaces a record in one rename (`replace_private_file`),
  because "A record this program rewrites is read by a later run as a whole,
  and a truncating write leaves it empty for a kill that lands between the truncation and the bytes".
- `package/git-policy/cli/src/native/foundation/rename_that_never_replaces.rs`
  holds `rename_noreplace`,
  `exchange` (the rename that hands back what stood at the target),
  and `rename_unsupported`,
  which is how a caller tells a filesystem without the flag from a name that is taken.
- The route's own receipts rule,
  `doc/planning/cli-git-zfs-worktree-design-proposal.md:564` to `:571`,
  is the shape of a root-owned store of one file per thing:
  "The root-owned directory is mode `0755`;
  receipts are root-owned mode `0644`,
  read-only to the wrapper,
  and opened relative to the trusted directory.
  Nonces appear only as validated data in the receipt,
  never in a filename."
- The store's own module already holds the name grammar and the control checks this design reuses:
  `slot_name_refusal` and `store_control_refusal`.

## 1. The folder, its files, their names, owners and modes

### The example store

One store for the checkout at `/home/user/Monochromatic`,
after the setup run and two sweeps,
with the access ACL bytes the specification's own example uses:

```text
/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities/   root 0755
  .parent                                                                   root 0644
  add-cache-layer                                                           root 0644
  fix-flaky-test                                                            root 0644
```

```text
# /var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities/.parent
# monochromatic-git-policy-cli worktree clone slot identities, version 2
parent - 57:66304:1000:0755:-:-
```

```text
# /var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities/add-cache-layer
# monochromatic-git-policy-cli worktree clone slot identities, version 2
slot add-cache-layer 57:67410:1000:0755:0200000001000700ffffffff0200050004000500ffffffff10000500ffffffff20000500ffffffff:-
```

```text
# /var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities/fix-flaky-test
# monochromatic-git-policy-cli worktree clone slot identities, version 2
slot fix-flaky-test 57:67833:1000:0755:-:-
```

Two owners appear in that store and only one of them owns the store:
the folder and its record files are root's,
because only root writes them,
while the `owner` field inside `1000:...` is the user that owns the recorded folder,
which is the account's `.claude/worktrees` and its slots.
Device `57` is this host's `zpcachyos/ROOT/cos/home`,
which holds the checkout,
and the store's own folder is on device `31`,
the `zpcachyos/ROOT/cos/root` dataset.

### The folder

The folder keeps the store's path,
`/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities`,
under the root-owned folder that holds it.
It is root-owned,
mode `0755`,
which is the receipts' shape:
root writes it,
and an account may read it but may not add,
rename or remove an entry in it.
`store_control_refusal`'s two terms stay the reader's test,
owner `0` and no group or other write bit,
and they now apply to the folder itself and to each record file opened in it.

The current code opens the store file and its folder by path and checks each through the descriptor it opened
(`check_control`,
`check_the_folder_control`,
`SlotIdentityStore::read_under_root_control`).
This design opens the folder once per operation,
`O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC` on the store's path,
checks that descriptor by `fstat`,
and then names every record,
staging entry and rename relative to that one descriptor:
`openat(folder, name, O_RDONLY | O_NOFOLLOW | O_CLOEXEC)`,
`linkat(folder, ...)`,
`renameat2(folder, ...)`,
`unlinkat(folder, ...)`,
`fsync(folder)`.
No path component between the store's folder and a record is resolved again,
so a name the account could change below the root-owned folder is never followed,
and the object judged is the object read.
`O_NOFOLLOW` on the folder refuses a link planted at the store's own name,
which the current `read_under_root_control` already refuses for the file.

### The names

Three kinds of name live in the folder,
and one rule tells them apart:

- `.parent` is the parent folder's record,
  one file,
  the only name this design reserves.
- A slot's record is named exactly the slot's name,
  and the name grammar is the one the store already enforces,
  `slot_name_refusal`:
  not empty,
  not `.` or `..`,
  no `/`,
  no leading dot,
  no whitespace and no control character.
- An entry whose name begins with a dot and is not exactly `.parent` is a staging entry.
  It is never a record,
  a reader ignores it,
  and a writer removes its own.
  A leading dot is what the slot grammar already refuses,
  so no slot can ever be named like one.

No suffix and no second level is needed,
and the measured reason is `NAME_MAX`:
the probe created a 255-byte name and was refused a 256-byte one,
and a slot name may be 255 bytes,
so a file named `<slot>.slot` could not record a legal long slot without a new refusal for a name Git allows.
A subfolder of slots would fit every name too,
and it is option B of section 1.

### The modes

Record files are mode `0644`,
root-owned,
which is what the route's receipts are and what the current store is read as
(its reader refuses only group and other write).
The record's bytes are written with mode `0644` set before the name exists,
so a record is never readable at a wider mode than the one it keeps.

### The options and their ranking

- **A,
  flat,
  the slot's name as its file's name and `.parent` for the parent.**
  On the example store the folder lists `.parent`,
  `add-cache-layer` and `fix-flaky-test`,
  and a reader opens each by the name it already has.
  Pros:
  one held descriptor,
  one control check,
  no parsing of names,
  and every legal slot name fits.
  Cons:
  the parent's record is hidden from a plain `ls`,
  and one name is reserved.
- **B,
  two levels,
  `parent` beside a `slots/` folder.**
  On the example store the top level lists `parent` and `slots`,
  and `slots/` lists `add-cache-layer` and `fix-flaky-test`.
  Pros:
  no reserved name,
  and the parent's record is visible.
  Cons:
  a second descriptor and a second control check,
  two listings for the adoption,
  and the parent's record sits outside the folder a reader already holds.
- **C,
  one level with a suffix per kind,
  `<slot>.slot` and `parent.slot`.**
  On the example store the folder lists `parent.slot`,
  `add-cache-layer.slot` and `fix-flaky-test.slot`.
  Pros:
  every name says its kind.
  Cons:
  a 255-byte slot name has no room for the suffix,
  so either a legal name is refused or the name is hashed and no longer legible.

Ranking:
A > B > C.
A beats B because the second level buys only a visible parent name while making every read and every listing reach a
second descriptor whose control terms must be checked again,
and the reserved name costs one sentence of grammar.
B beats C because C cannot name a slot at `NAME_MAX`,
which the probe measured as 255 bytes with 256 refused,
and the design must not refuse a folder Git made.

## 2. How a slot's first record is published

### The steps

A record's bytes are the version 2 header line and one record line,
which is exactly what the example store's files hold.
Publication is one step that gives a complete file a name and never replaces one:

1. A sweep that finds no record for a slot captures the slot folder's identity from the descriptor it holds,
   which is the plan's step 8 first-observation record.
2. It formats the two lines and checks their length against the per-file bound of item 4.
   The bytes are known before anything is created,
   so a record too large to read is refused before a name exists.
3. It makes a file with no name in the held folder descriptor:
   `openat(folder, ".", O_TMPFILE | O_DIRECTORY | O_RDWR | O_CLOEXEC, 0o644)`,
   which the probe accepted on this host's ZFS dataset.
4. It writes the two lines,
   `fchmod`s the descriptor to `0644`,
   and `fsync`s it,
   so the bytes are durable before any name can hold them.
5. It gives the file the slot's name with no replacement possible:
   `linkat(AT_FDCWD, "/proc/self/fd/<n>", folder, name, AT_SYMLINK_FOLLOW)`,
   and `linkat(fd, "", folder, name, AT_EMPTY_PATH)` where the first answers `ENOENT` or `EPERM`,
   which is `link_unnamed`'s order and both of its arms,
   and the probe measured both answering zero as the account.
   `EEXIST` means another writer published first and is the race's whole answer.
6. It `fsync`s the folder descriptor,
   so the name is durable as well as the bytes.

A filesystem that refuses `O_TMPFILE` with `EOPNOTSUPP`,
`ENOSYS` or `EINVAL` takes the fallback the package already ships,
and the probe measured each step of it here:

1. It makes a staging file beside the record's name with an exclusive creation,
   `openat(folder, ".<32 hex>", O_CREAT | O_EXCL | O_WRONLY | O_NOFOLLOW | O_CLOEXEC, 0o644)`,
   where the hex is random per attempt.
   The name begins with a dot,
   so no reader reads it as a record and no slot can be named like it.
2. It writes the same two lines,
   `fsync`s the file,
   and `fchmod`s it `0644`.
3. It publishes with `renameat2(folder, staged, folder, name, RENAME_NOREPLACE)`,
   which the probe measured answering `EEXIST` and leaving the staged name in place where the name is taken.
4. Where the filesystem refuses that flag (`rename_unsupported`),
   it publishes with `linkat` from the staged name and then `unlinkat`s the staged name,
   which is one step to every reader and answers `EEXIST` where the name is taken,
   as `linkat(2)` documents and as this round's probe did not aim at a taken name.
5. It `fsync`s the folder,
   and a failure at any step removes the staging name this attempt made.

A filesystem that answers neither operation refuses the write,
naming the folder and the two calls,
and nothing is attached on a record the store did not keep.
No checked-then-renamed fallback is offered,
unlike `private_storage::settle_exclusive_publish`,
because there the fallback exists so a commit never fails,
and here a publication that may replace another's record is the refresh this design forbids.

### What a second recorder of one slot gets

The kernel answers exactly one writer.
The loser's publication answers `EEXIST`,
and it then opens the name that is now there and reads it,
which is one `openat` on the folder descriptor it already holds.
It reports that another sweep recorded that slot while this run held no record of it,
naming the file,
and it carries on with the record it found:
the identity compared against the slot folder is the published one,
never this run's own capture.
Nothing is rewritten,
so a slot's record still has exactly one version for its whole life,
and where the two captures differ the attach's identity compare refuses with the fields that moved,
which is the same refusal a single sweep would have produced.

### Durability and the kill window

A kill between those steps leaves either no name or a complete durable record,
because the name is given to a file that already holds every byte,
and the folder's own `fsync` is what makes the name survive a machine crash.
A kill in the fallback leaves a staging entry and no record,
which no reader reads,
and which the adoption removes,
because the adoption's own precondition is that managed operations are quiesced
(`adopt_worktree_parent_mode.rs`'s module doc states that the mode runs with managed operations quiesced).
Nothing else removes a staging entry,
so no run needs a lock or a timeout to be sure another writer is gone.

### The options and their ranking

- **A,
  a file with no name and one link,
  with the package's named fallback.**
  On the example store,
  `add-cache-layer` appears whole at its name,
  and a reader that opens it either finds no name or the two complete lines.
  Pros:
  no reader can ever see a partial record,
  a kill leaves no name and no litter on the common path,
  and both arms are the package's own code.
  Cons:
  two system calls more than a create,
  and a filesystem-dependent fallback to test.
- **B,
  `O_EXCL` at the record's name and a completeness marker.**
  On the example store `add-cache-layer` exists from the first `openat`,
  and a reader must treat it as incomplete until a marker line is written,
  so a kill between the two leaves a name that refuses every read of that slot until root removes it.
  Pros:
  the record's name is created with `O_EXCL`,
  as the decision's own words say.
  Cons:
  a partial record is visible at the name,
  a reader must retry rather than trust,
  and a killed writer leaves permanent damage where option A leaves none.
- **C,
  `O_EXCL` at the record's name and a reader that retries a short file.**
  On the example store the same partial name exists,
  and a reader that finds fewer than two complete lines waits and reads again.
  Pros:
  no second file and no marker.
  Cons:
  every reader carries a retry and a wait no measurement sizes,
  and a torn or empty file from a killed writer is indistinguishable from one being written,
  so the store can neither refuse nor read it.

Ranking:
A > B > C.
A beats B because a name that only ever holds complete bytes needs no marker and leaves no permanently
unreadable entry,
and `O_EXCL` is kept where it belongs,
on the staging name the fallback creates.
B beats C because a marker at least tells a killed writer's leftover from a live one,
which C's retry cannot,
so C both waits on nothing and cannot refuse.

## 3. Replacing the parent record when the adoption changes it

The constraint is the accepted design's sentence at
`doc/planning/cli-git-zfs-worktree-design-proposal.md:623`:
"Never overwrite surviving slot identities or implicitly adopt a replacement for an existing binding."
This layout meets that sentence with its shape and not with a check:
the parent's record is one file that holds nothing but the parent's identity,
so replacing it cannot touch a slot's record,
and the adoption opens no slot file for writing,
renames none and removes none.

### The steps

1. The mode reads `.parent` under root control and decides from it,
   and reconciles every recorded slot exactly as the decision table already orders
   (`decide_adoption`'s walk,
   identified device and inode,
   policy,
   bindings,
   slots,
   fresh lookup).
2. It formats the new record (the version 2 header and one `parent - ...` line)
   and checks its length against the per-file bound.
3. It makes a staged file in the held folder descriptor,
   writes the two lines,
   `fsync`s it,
   and sets mode `0644`,
   which is `private_storage`'s `staging_beside` plus `sync_when_durable`.
4. It publishes with one rename that replaces the target:
   `renameat2(folder, staged, folder, ".parent", 0)`,
   which the probe measured replacing in one step,
   and which is what `replace_private_file` already does by `tempfile`'s `persist`.
5. It `fsync`s the folder,
   so the new name is durable.

`rename_that_never_replaces::rename_noreplace` is deliberately not used here:
the name must be replaced,
and the primitive that never replaces is the slot record's publisher.

### What a reader racing the replacement sees

Either file,
whole,
and never a third state:
the old record's inode answers every open that began before the rename,
the new record answers every open after it,
and a reader that resolved the name at any instant reads two complete lines whose record it parses,
because the rename moves a name and never a byte.
A reader that read the old record and then compares it against the replacement folder refuses,
naming the fields that moved,
so a stale read is a refusal and never permission to attach.
Nothing ever sees `.parent` absent,
because a rename over an existing name removes the name only by replacing it in the same step,
and the mode refuses `NoParentRecord` rather than creating it.

### Two writers with two intents

The parent record has two writers and they must not be confused:
the setup run writes the parent's first record,
and the administrator's adoption replaces it,
which the specification already separates in words at
`package/git-policy/cli/SPEC.md:5490`.
Today one writer serves both intents,
because `record_parent_identity_under_root_control`'s `replace_the_parent_record` replaces the record
whenever it is called,
so the setup run that plan step 19 still owes would silently replace an adopted identity unless it knew which
intent it had.
The design carries the specification's split into two writers:

- `publish_parent_identity_under_root_control` publishes `.parent` only where it is absent,
  by the same no-replace step a slot record uses,
  and a second setup run refuses `ParentAlreadyRecorded`;
- `replace_parent_identity_under_root_control` is the adoption's one writer,
  and it refuses where `.parent` is absent.

Both keep the root-control checks inside the step that writes.

### The options and their ranking

- **A,
  one file replaced by one rename.**
  On the example store the adoption leaves `.parent` holding
  `parent - 57:67990:1000:0755:-:-`,
  and `add-cache-layer` and `fix-flaky-test` keep their inodes,
  their bytes and their modification times.
  Pros:
  one name,
  one read per lookup,
  no growth,
  no second state a reader must resolve,
  and a slot record cannot be dropped because no code path opens one for writing.
  Cons:
  two adoptions that run at once leave whichever renamed last,
  and a reader that read the old record refuses until it reads again.
- **B,
  `RENAME_EXCHANGE` and a comparison of what it displaced.**
  On the example store the adoption stages the new record,
  exchanges it with `.parent`,
  reads the file it displaced,
  and puts it back with a second exchange where it is not the record the decision was made from.
  Pros:
  detects a parent record that changed between the decision's read and the publish.
  Cons:
  a window in which the store holds a record the check has not yet judged,
  a crash inside that window leaves the unjudged record standing although the check never ran,
  and readers inside the window see an identity that may be withdrawn a moment later.
- **C,
  generation-numbered parent files,
  `.parent.0007`,
  and the highest complete one read.**
  On the example store the folder holds `.parent.0006` and `.parent.0007` after one adoption,
  and every parent read becomes a listing plus a parse of the highest name,
  falling back to the next where the highest is incomplete.
  Pros:
  no reader ever reads a replacement while it is published,
  and the previous identity is kept.
  Cons:
  every parent read costs a listing,
  a killed writer's incomplete generation hides a complete lower one unless every reader carries the ladder,
  the folder grows with every adoption,
  and it needs a removal step whose own race this design would have to specify.

Ranking:
A > B > C.
A beats B because B's gain is a check against two concurrent adoptions,
which the mode's quiescence precondition already forbids,
and that gain costs a window in which an unchecked record stands,
which A never has.
B beats C because C moves the "highest complete generation" ladder into every parent read and adds a
removal step,
where B and A keep one name and one read.

## 4. What a reader refuses

The route's reader is one function over a held folder descriptor and one name,
and the folder's listing is a second function over the same descriptor.
Every refusal names the file it judged and the person or command that repairs it,
as `StoreRefusal`'s sentences already do.

### A record file's bound, and what replaces the whole-store bound

One file may hold at most `RECORD_MAXIMUM_BYTES`,
and the number stays the store's one mebibyte,
1,048,576 bytes.
Its basis is measured and arithmetic:

- a record's two ACL tokens are the kernel's own extended-attribute bytes in hexadecimal,
  and the probe found this kernel's ceiling at `XATTR_SIZE_MAX`,
  65,536 bytes,
  accepting 65,532 and refusing 65,540 with `E2BIG`;
- the widest identity token is therefore 262,203 bytes
  (two ACLs at 131,072 hexadecimal characters each,
  plus four numbers of at most 20,
  20,
  10 and 4 digits and five separators);
- the widest record line is 262,465 bytes with a 255-byte slot name,
  and the widest file is 262,538 bytes with the 72-byte version 2 header;
- one mebibyte is 3.99 of that widest file,
  so a legitimate record cannot reach the bound,
  and a file that does is one no folder this kernel describes can produce.

The whole-store bound does not become a per-folder sum:
the store has no total bound,
and what replaces it is that one read costs one file at most,
and that only root adds entries.
Its old purpose,
"a runaway or replaced file must be named rather than read into memory"
(`package/git-policy/cli/SPEC.md:5407`),
is now the per-file bound.
The refusal for a file over it names the file,
its size and the bound,
and says that no command in this repository removes a record or raises the bound,
which is what the store's `WouldNotFit` sentence says today.

### A missing folder or a missing record

A missing store folder refuses `Missing`,
naming the setup run of plan step 19,
as today's `Missing` names the run that writes the file.
A missing `.parent` refuses `NoParentRecord`,
which the attach and the adoption both raise,
and which says what the record is for:
there is nothing to compare the parent folder against.
A missing slot record refuses `NoSlotRecord` to a reader that needs one,
and to the sweep it is not a refusal but the first observation item 2 publishes.

### A partial record

A file that does not hold the version 2 header line followed by exactly one record line refuses,
naming the file and the line where it stopped,
which is today's `Line` refusal with a file in place of a store:
"Nothing is attached to a slot until the file reads whole."
With item 2's publication a partial file is not something a writer of this repository makes,
so a partial file means a writer outside it or a filesystem that lost bytes after an `fsync`,
and both are the administrator's to repair,
never a state to retry.

### A foreign entry, and a record under another name

The listing refuses a name that is neither `.parent`,
nor a slot name,
nor a staging entry,
and it names that entry:
a foreign name could be a slot whose record a later read would miss,
which is the same reason today's reader refuses a line this repository did not write.
A staging entry (a leading dot that is not `.parent`) is ignored by every reader and by the listing.

A record whose own key is not the name it is filed under refuses,
because the record line carries its key:
`add-cache-layer`'s file must hold `slot add-cache-layer ...`,
and `.parent`'s must hold `parent - ...`.
That is what refuses a record copied,
hard-linked or restored to another name,
and it is why the file's content can be trusted without a second index of what each name means.

### A record the account could rewrite

A record file,
like the folder,
refuses when its owner is not root or when its mode has a group or other write bit,
on `store_control_refusal`'s own terms,
and the check reads the descriptor that was opened,
so a swap between the name and the read cannot change which object was judged.
A name that holds a link,
a folder or another non-regular file refuses,
which `O_NOFOLLOW` and the `is_file` test already do today.

### The whole of a read

One read is:
one `openat` on the held folder descriptor,
one `fstat` for the owner and the mode,
one bounded read of at most `RECORD_MAXIMUM_BYTES`,
one UTF-8 check,
and the two lines' parse.
Nothing in it lists the folder,
so a reader pays nothing for the number of slots,
which is the property the single file's one read could not give.

## 5. What each check reads, and in what order

### The example store again

Every check this section names is described against the store in section 1:
`.parent` holding device `57` and inode `66304`,
`add-cache-layer` holding inode `67410`,
and `fix-flaky-test` holding inode `67833`,
each on device `57`.

### The attach's checks, plan step 4

1. The no-crossing walk from the pinned checkout reaches `.claude/worktrees` and captures its identity from the
   descriptor it opened.
   The walk needs no store file;
   the compare needs exactly one,
   `.parent`,
   opened on the store's folder descriptor.
2. The walk reaches the slot and captures the slot folder's identity from the descriptor it opened.
   The compare needs exactly two files,
   `.parent` and the slot's own,
   `add-cache-layer` in the example.
3. `fsmount` and `move_mount` need no store file,
   and the post-attach check compares the mount's identity against what the walk and the attach proved,
   never against the store.
4. The recheck rule at `doc/planning/cli-git-zfs-worktree-design-proposal.md:425`
   ("Recheck recorded identities,
   the mount and the visible pointer before resuming")
   is satisfied without a second read of the slot's record,
   because that record has exactly one version for its whole life:
   item 2 publishes it once,
   no code path replaces or removes it,
   and a second read would be a read of the same bytes.
   The parent's record can change,
   and the attach's only exposure to that is the quiescence rule the adoption mode already carries,
   with a stale read refusing rather than permitting.

So an attach reads two files whatever the store holds,
and a store of four thousand slots costs an attach the same as a store of one.
That is the per-slot read the design's item 5 asks for,
and nothing in the attach's checks needs another slot's identity.

### The sweep, plan step 8

1. The sweep enumerates the pinned parent's immediate children itself,
   which is its own walk and not the store's listing,
   and derives each slot's name.
2. For each slot it opens the slot's record,
   which is one file,
   and where that name is absent it captures the folder's identity and publishes the record by item 2.
3. It attaches through step 4,
   which reads `.parent` and the slot's record.
4. A slot that another sweep recorded first costs it one more read of that record,
   which is item 2's loser path.

The sweep never lists the store,
so a slot that is recorded but whose clone is gone costs it nothing,
and a store entry for a slot the walk did not reach is outside the sweep's business:
the decision's lost single read is exactly that no check sees the set,
and no check in this step needs it.

### The adoption, and the listing it needs

The adoption is the one reader that needs the whole set,
because the decision table refuses a replacement that leaves a recorded slot unaccounted for
(`AdoptionRefusal::SlotNotListed`).

1. It reads `.parent`,
   which is the policy the decision compares the replacement against
   (`ParentPolicy::from_recorded`).
2. It lists the store folder once,
   with `readdir` on the held descriptor,
   and keeps every name that is a slot record,
   refusing a foreign name by item 4 and ignoring staging entries.
3. It reads each listed slot's record,
   which is one file per name,
   and compares it against the live folder below the held parent,
   as `read_slot` already does,
   reaching `Agrees`,
   `Differs` or an unresolved binding.
4. It decides,
   and before publishing it lists the folder a second time and refuses where the set of slot names changed:
   a name that appeared reaches the decision as `SlotNotListed`,
   and a name that vanished refuses as a record the store no longer holds,
   which nothing in this repository could have removed.
5. It publishes `.parent` by item 3.

The second listing is a choice,
and it is proposed because it is what restores part of what one read gave:
the adoption verifies exactly the set it publishes beside.
What it does not restore is stated under "The one property lost,
and what is not restored".

### The other readers

- The unmount of plan step 9 reopens the parent and the slot and compares mount id,
  device,
  inode and source,
  so it reads `.parent` and that slot's record,
  two files.
- `doctor`,
  plan step 18,
  reports "the recorded parent and slot identities with the failing field or component"
  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:1636`),
  so it is the one reader that reads every record,
  and it is read-only.
  It lists the folder and reads each record,
  and its report needs no state the layout cannot give.
- The removal workflow and its cancel command read the slot's record only,
  because that is all either decides on.

### The one property lost, and what is not restored

The single file gave one read of the parent and every slot in one state.
This layout gives one read of one record,
and no operation can prove "these are all the records,
at one instant":

- the attach never asked,
  so it loses nothing,
  and its two reads are of records that cannot change;
- the sweep never asked,
  and its set comes from its own walk of the parent's children;
- the adoption asks,
  and gets a set that can change between its first listing and its second,
  and again between its second listing and the publish.
  The second listing closes the first window,
  and the second window stays open:
  a slot published after that listing is a record the adoption did not verify.
  That residual is safe and exactly why:
  a slot's record is a first observation of a slot folder,
  and no field of it depends on the parent's identity,
  so a record published during an adoption is as valid afterwards as before it,
  and the replacement's own identity is what the adoption verified.
  The residual therefore costs a slot that the adoption's report does not name,
  not an identity the store gets wrong.
  Where the reviewer prefers the set to be frozen rather than reported,
  the answer is a lock over the folder,
  which rule `NLK` would require to be justified by a recorded failure,
  and none is recorded.

### A racing adoption, step by step

Two sweeps and one adoption meet on the example store,
with the store folder opened once by each,
and the timings are the ones the hooks of item 7 force:

1. The adoption lists the folder and reads `.parent`,
   `add-cache-layer` and `fix-flaky-test`,
   and verifies both slots against their live folders.
2. Sweep one reaches `fix-flaky-test`,
   finds its record,
   and compares it:
   device `57`,
   inode `67833`,
   owner `1000`,
   mode `0755`,
   no ACLs,
   so the attach proceeds.
3. Sweep two reaches a new slot `rework-login`,
   reads no record for it,
   captures device `57` and inode `68120`,
   and publishes.
   The publication is one link,
   so `rework-login` holds two complete lines at no instant in between.
4. The adoption's second listing now sees `rework-login` and refuses `SlotNotListed`,
   naming it,
   and `.parent` still holds inode `66304`.
   The sweep two attach is unaffected,
   because `.parent` did not change and its own slot's record is what it compares.
5. The administrator runs the adoption again,
   now listing three slots,
   verifies `rework-login` as a surviving slot,
   stages the new parent record and renames it over `.parent`,
   and the folder's three slot files keep their inodes and bytes.
6. A sweep that opened `.parent` a moment before that rename compares the replacement folder against inode `66304`
   and refuses,
   naming the inode and the device:
   a stale read is a refusal.

## 6. What becomes of the code, the documents and the controls on `main`

Nothing here is built by this round;
this is the list the build follows and the review judges.

### Deleted

- `hold_the_store_lock`,
  `lock_a_store_that_is_there`,
  `store_lock_path`,
  `check_lock_control`,
  `STORE_LOCK_MODE`,
  `the_lock_wait_allowed`,
  `STORE_LOCK_WAIT`,
  `STORE_LOCK_PAUSE`,
  the `Locked` refusal,
  and the lock-hold thread-local hooks
  (`BETWEEN_THE_READ_AND_THE_APPEND`,
  `BEFORE_THE_STORE_OPENS`,
  `THE_LOCK_WAIT_ALLOWED`).
  The store takes no lock,
  and no constant replaces the wait because no write waits for another.
- The ignored measurement control
  `a_store_write_s_hold_is_measured_for_the_lock_wait`.
  Its subject is the lock's hold,
  so it goes with the lock;
  the publication's `fsync` cost is measured by the arm in item 7 instead.
- `STORE_MAXIMUM_BYTES`,
  replaced by `RECORD_MAXIMUM_BYTES` with the same value and the new basis of item 4,
  and `check_the_store_fits_one_read`,
  replaced by a per-record length check.
- The `WouldNotFit` and `DuplicateKey` refusals.
  A record too wide for one read refuses as a per-file size refusal,
  and a key recorded twice cannot exist where the key is the file's name.

### Rewritten

- `slot_identity_store.rs` becomes the folder and its held descriptor:
  `SLOT_IDENTITY_STORE_FOLDER_PATH`,
  `open_the_store_folder_under_root_control`,
  `SlotIdentityStore` holding the descriptor and the folder's path,
  `parent()` and `slot(name)` reading one file each,
  and `slot_names()` listing the folder.
  It keeps `STORE_OWNER`,
  `WRITABLE_BY_OTHERS` and `store_control_refusal`,
  and it grows past the repository's line limit,
  so its parts split into sibling files in the same folder
  (`slot_identity_store_records.rs` keeps the grammar,
  and the publisher,
  the reader and the refusals get one file each),
  re-exported by `mod.rs`,
  listed in the folder's top doc in dependency order (rule `LYF`).
- `slot_identity_store_records.rs`:
  `STORE_HEADER` becomes version 2,
  `parse_records` becomes `parse_one_record` over a two-line file,
  the duplicate-key set goes,
  and a record whose key is not its file's name refuses.
  `slot_name_refusal` is kept unchanged and is now also the record-name grammar.
- `append_slot_identity_under_root_control` becomes
  `publish_slot_identity_under_root_control`,
  by item 2,
  and answers `SlotAlreadyRecorded` on `EEXIST` after it reads the record that won.
- `record_parent_identity_under_root_control` splits into item 3's two writers.
- `slot_identity_store_refusals.rs`:
  `Missing`,
  `NoParentRecord`,
  `NoSlotRecord`,
  `SlotAlreadyRecorded`,
  `SlotName`,
  `NotUnderRootControl` and `Write` keep their sentences with the folder in place of the file,
  and three causes are new:
  a record over the per-file bound,
  a record whose key is not its file's name,
  and a foreign entry in the folder.
  `Missing`'s sentence names the setup run that creates the folder,
  and a new `ParentAlreadyRecorded` answers a second setup run.
- `adopt_worktree_parent_mode.rs`:
  `recorded_slots` lists the folder and reads one file per name,
  and the mode lists twice by item 5.
- `parent_adoption_decision.rs`:
  `AdoptionFacts.store` becomes the enumeration the listing produced,
  so the table keeps its order and its refusals,
  and `SlotNotListed` now also answers a slot that appeared between the two listings.
- The specification,
  `package/git-policy/cli/SPEC.md:5348` to `:5509`:
  "The recorded identities store" is rewritten for the folder,
  the per-file grammar and version 2,
  the publication of item 2,
  the two parent writers of item 3,
  the per-file bound of item 4,
  and the readers of item 5.
  The grammar block at `:5359` to `:5364` becomes three file examples,
  the bound's sentences at `:5407` to `:5421` become the per-file bound,
  the lock's paragraphs at `:5451` to `:5474` and the
  "That lock is the one place this route is not lock-free" paragraph at `:5475` to `:5489` are deleted,
  `:5490` to `:5499` keeps its split of the two parent writers with the two new writers' names,
  and `:5500` to `:5508` keeps its conclusion with the folder's own reason
  (a second checkout's `.parent` cannot be published beside the first).
- Plan step 4's `:597` to `:600`,
  which names the store as one path the attach reads,
  gains the folder and the two files the attach reads;
  step 8's first-observation sentence gains "publishes it by one link that replaces nothing";
  step 19's setup run,
  which does not exist yet,
  is where the folder's creation at mode `0755` and `.parent`'s first publication belong.
- `package/git-policy/cli/e2e/README.md:838`,
  which says the guest adoption arm writes the store file,
  and
  `package/git-policy/cli/src/native/binary/binary_zfs_adoption_guest_tests.rs`,
  whose `write_store` writes the single file with `std::fs::write`.
- The dated records keep their text and are not rewritten (rule `WR6`):
  `package/git-policy/cli/doc/step-5-second-executable-2026-11-20.md:93`
  keeps its `Missing { path: ... }` line as the round that saw it,
  and `doc/planning/cli-git-zfs-staging-under-cache-design.md:208` keeps "records live in
  `/var/lib/...`",
  which stays true of the folder.

### The documents

- `package/git-policy/cli/doc/slot-identity-store-lock-hold.md` loses its subject.
  It is rewritten,
  not deleted,
  because two durable facts in it have no other home:
  the kernel's `fsync` of a record reaching the pool's device
  (60 samples,
  a median of 8 sectors and two ZIL commits,
  against three controls that moved the device not once),
  and `probe-reaches-storage.py`,
  which is kept so that question can be asked again.
  Its name would then lie about its content (rule `VGW`),
  so it becomes `package/git-policy/cli/doc/slot-identity-store-write-cost.md`
  with its probe folder beside it,
  the lock sections and every arithmetic line about the wait deleted,
  and the publication's own measured cost added where the item 7 arm measures it.
  Its status line stays "measurement record by an agent,
  unapproved and non-final".
- The acceptance evidence for issue 6327 stays closed and is not reopened.
  The defect it names,
  a write that leaves a store its own read refuses,
  cannot recur:
  a record's bytes are measured against the per-file bound before a name exists,
  and no write makes a file it would then refuse.

### The controls that are rewritten

Every control in `slot_identity_store_tests.rs` is rewritten,
because every one of them drives a store that is one file.
Named here by what they prove:

- the written store reads back,
  the absent store asks for the setup run,
  the line this repository did not write,
  and the duplicate key:
  the first three stay with a folder fixture,
  and the duplicate arm is replaced by a record filed under another name;
- the slot recorded once,
  which becomes item 2's publish-once arm and its loser;
- the adoption replaces only the parent record,
  which becomes item 3's arm that also proves the slot files' inodes and bytes unchanged;
- the append holds one lock and the adoption waits for it,
  the append opens the store after the lock,
  an append that cannot take the lock,
  and a link at the lock's name:
  deleted with the lock;
- the store larger than the read accepts,
  an append that would outgrow the store,
  an adoption that would outgrow the store,
  and an adoption that lands exactly on the bound:
  replaced by the per-file bound's arms of item 4;
- the control's own root terms and the store this account could rewrite:
  kept,
  with the folder and a record file in place of the file and the lock file;
- the specification's store example,
  which becomes the example of item 1;
- `adopt_worktree_parent_mode_tests.rs`'s store fixtures,
  `attach_arm_tests.rs`'s store arms,
  and `binary_zfs_adoption_guest_tests.rs`'s `store_bytes` and `write_store`.

## 7. The tests a build of this design must have

Each control is shown to fail before it passes (rule `GFP`),
and each refusal arm has its accepted fixture as well as its rejected one (rule `THR`).

### Controls that need no root

1. `a_record_is_published_whole_and_a_reader_never_sees_it_partly_written`:
   a hook between the durable bytes and the link,
   asserting the record's name is absent there and whole after it.
2. `two_recorders_of_one_slot_publish_once_and_the_second_reads_the_first`:
   item 2's race,
   forced by a hook that lets a second writer publish while the first is held before its link,
   asserting one file,
   the first writer's bytes,
   and the loser's `SlotAlreadyRecorded` report.
3. `a_publication_that_fails_leaves_no_record_and_no_staging_entry`:
   a failure injected after the bytes are durable,
   on the unnamed arm and on the fallback arm.
4. `the_publication_falls_back_where_the_filesystem_has_no_unnamed_file`:
   the request that makes the file is a parameter,
   as `publish_copy_of_file_with` already takes it.
5. `the_publication_falls_back_where_the_filesystem_refuses_a_rename_that_never_replaces`:
   the arm that forces `rename_unsupported`.
6. `a_parent_record_is_replaced_by_one_rename_and_a_reader_sees_one_whole_record`:
   item 3's race,
   forced by a hook between the staged parent's `fsync` and the rename,
   asserting a reader in the window reads the old record whole and `.parent` is never absent.
7. `the_adoption_writes_no_slot_file`:
   inodes,
   modification times and bytes of every slot file unchanged across an adoption.
8. `a_slot_file_that_appears_between_the_two_listings_refuses_the_adoption`:
   a hook between the listings,
   asserting `SlotNotListed` names the slot and `.parent` is unchanged.
9. `a_record_larger_than_the_bound_is_refused_by_name_and_size`:
   a fixture at the bound and one byte over.
10. `a_record_whose_key_is_not_its_file_name_is_refused`:
    a good record copied,
    and a hard link of it,
    each refused.
11. `a_foreign_entry_refuses_the_listing_and_a_staging_entry_does_not`:
    a name with a space,
    a name with a leading dot that is not `.parent`,
    and the folder's own listing arm.
12. `a_record_this_account_could_rewrite_is_refused`:
    the owner and mode arms,
    through the user-namespace helper the current control already uses.
13. `an_absent_store_folder_asks_for_the_setup_run`,
    and `a_link_at_a_record_name_is_never_read_through`.
14. `the_specification_s_store_example_is_a_store_this_reader_reads`,
    over the three files of item 1.

### Guest arms, and why each needs root or a real pool

15. `the_store_folder_under_var_lib_publishes_a_record_as_root_and_reads_it_back`:
    `/var/lib` is root's,
    so this runs in the gate image or the throwaway guest,
    and it is the arm that settles the design's one assumption about the store's own dataset:
    `O_TMPFILE`,
    `linkat`,
    `renameat2` and the folder's `fsync` on `zpcachyos/ROOT/cos/root`.
16. `a_killed_writer_between_its_bytes_and_its_link_leaves_no_record`:
    root,
    because the store is under `/var/lib`;
    the writer is killed after its bytes are durable and before its link,
    and the arm asserts no name and no partial record.
17. The publication's `fsync` arm:
    `probe-reaches-storage.py` re-run against a per-slot publication on a folder of the store's own pool,
    which needs the real pool and its counters.
    It is what replaces the lock-hold record's subject with the publication's.
18. The owed boot arm stays owed and unchanged:
    record a parent identity and a slot identity,
    reboot the guest,
    read the store and the folders' `stat --format=%d` again,
    and repeat with an export and import in between.
    This design changes where the records live and not what they hold,
    so that arm's question and its answer stay step 8's.

### What a green run shows, and what it would not

A green run shows that every publication leaves no partial name,
that exactly one of two recorders records a slot,
that a parent replacement is one whole record at every instant,
that the adoption writes no slot file,
that every refusal of item 4 is reached and says what repairs it,
and that the store reads under root control on the dataset it lives on.
It does not show that two sweeps meeting is rare,
that a machine crash between the folder's `fsync` and a reboot leaves the record,
or that a filesystem the gate never mounts answers the fallbacks the way the arm forces them to.

## Limitations

- The primitives were measured on this host's `zpcachyos/ROOT/cos/home` dataset as the account,
  not on `zpcachyos/ROOT/cos/root` as root.
  Both are ZFS datasets of one pool mounted `xattr,posixacl`,
  and the arm of item 7 is what closes the gap.
- The kernel's ACL ceiling was measured at 65,532 bytes accepted and 65,540 refused `E2BIG`,
  which places it at `XATTR_SIZE_MAX` for the whole attribute value,
  and the design's widest record is arithmetic on that number and not a measured file.
- ZFS's own directory `fsync`,
  its `O_TMPFILE` support under another mount option set,
  and its behaviour for a machine crash were not measured;
  the design assumes POSIX durability for the two `fsync` calls it orders.
- The number of sweeps that meet on one slot,
  and the number of adoptions that meet on one parent record,
  are unmeasured;
  both races are handled by the kernel's own exclusivity and by one refusal,
  and neither is sized.
- The per-file bound is one mebibyte,
  which is 3.99 of the widest record this kernel's limits allow;
  a filesystem whose ACL attribute may exceed `XATTR_SIZE_MAX` would need the bound raised,
  and no such filesystem is in this route.
- The store's folder has no total bound,
  and nothing bounds a listing but the number of slots a checkout has.
  Only root adds entries,
  so a large listing means a large checkout and not a foreign writer.
- The adoption's second window stays open:
  a slot published between the second listing and the rename is not verified by that adoption,
  and the design's argument that this is safe rests on a slot record holding no field of the parent's
  identity.
- Two adoptions that run at once are left with whichever renamed last.
  The mode's quiescence precondition forbids them,
  and nothing in the store detects one.
- This design does not decide whether a slot's record may be removed when its clone is destroyed and its
  registration settled,
  which today nothing does.
  The layout makes that one `unlinkat`,
  and the decision is the owner's,
  because it is what would let a reused slot name be recorded anew.
- No measurement here is a release-profile build,
  and none was taken under a loaded pool.
- The probe those primitive numbers came from lives in this round's scratch folder and not in the repository,
  so they are not reproducible from a checkout until the arm of section 7 lands beside the code.
- The design has had no independent review,
  has no implementation,
  and its `SPEC.md`,
  plan and code edits are unwritten.

## The choices this design leaves open, with their rankings

Each choice is argued in its own section,
and none is decided until this design's review accepts it:

1. Names:
   flat with `.parent` (A),
   two levels with `slots/` (B),
   or a suffix per kind (C).
   Ranking A > B > C,
   because B buys a visible parent name with a second descriptor and C cannot name a 255-byte slot.
2. The slot record's publication:
   a file with no name and one link,
   with the package's named fallback (A),
   `O_EXCL` with a completeness marker (B),
   or `O_EXCL` with a reader that retries (C).
   Ranking A > B > C,
   because A never shows a partial name and never leaves one,
   while C can neither refuse nor read what a killed writer left.
3. The parent record's replacement:
   one rename (A),
   an exchange compared against what it displaced (B),
   or generations (C).
   Ranking A > B > C,
   because B's gain is a race the mode's quiescence precondition forbids and B opens a window A never has,
   while C moves a generation ladder into every parent read.
4. What a second recorder of one slot does:
   read the record that won and carry on with it (A),
   or refuse and ask for a fresh run (B).
   Ranking A > B,
   because the record is what the attach compares against either way,
   so carrying on can loosen nothing and still does a single sweep's work.
5. The per-file bound:
   one mebibyte (A),
   300 KiB (B),
   or 512 KiB (C).
   Ranking A > B > C,
   because A is 3.99 of the widest legitimate record and is the number the whole store already carried,
   so a read costs no more than it did,
   and B and C are new numbers that buy nothing.
6. Who may read the store:
   root-owned `0755` with `0644` records (A),
   or `0700` with `0600` records (B).
   Ranking A > B,
   because A is the receipts' shape and keeps an unprivileged report of the recorded identities possible,
   while B would make `doctor` need root for a read it could not explain.
7. The lock-hold record:
   rewritten and renamed as the publication's write cost (A),
   kept under its name with a note that the lock is gone (B),
   or deleted with its probe (C).
   Ranking A > B > C,
   because B leaves a name that lies about its content and C loses the pool's `fsync` evidence and a probe the
   repository means to keep.
8. The adoption's second listing:
   taken,
   and a changed set refused (A),
   or not taken,
   and the residual widened to the first listing (B).
   Ranking A > B,
   because one `readdir` narrows the window the decision paid for and needs no lock.

## References

- The decision:
  [`doc/decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md).
- The accepted design:
  [`doc/planning/cli-git-zfs-worktree-design-proposal.md`](cli-git-zfs-worktree-design-proposal.md).
- The implementation plan:
  [`doc/planning/cli-git-zfs-worktree-implementation-plan.md`](cli-git-zfs-worktree-implementation-plan.md).
- The store as built:
  `package/git-policy/cli/SPEC.md:5348`,
  and `package/git-policy/cli/src/native/worktree_clone_mount/slot_identity_store.rs`.
- The lock's measurement record:
  `package/git-policy/cli/doc/slot-identity-store-lock-hold.md`.
- The publication precedent:
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_publish_file.rs`
  and `package/git-policy/cli/src/native/foundation/private_storage.rs`.
- The probe:
  `probe_publish_primitives.py` in
  `/home/user/temp/agent/zfs-slot-identity-store-per-slot-files-design-2026-11-21/scratch/`,
  which is this round's scratch and not a repository file.
