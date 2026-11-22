# cli-git ZFS slot identity store as a folder with one file per slot

## Status

Design by an agent,
unapproved and non-final.
Written 2026-11-21T16:32:25-05:00 and fixed in the gate-review round of 2026-11-21.
It answers every item that
[`doc/decision/cli-git-zfs-slot-identity-store-format.md`](../decision/cli-git-zfs-slot-identity-store-format.md)
left to a design.
A gate review has read it and found two high,
six medium and eleven low findings;
this round fixed them,
the section [This round](#this-round) at the end names what moved,
and its recheck left eight low findings,
R1 to R8,
which the build answered as the [Built](#built) note says.
Every choice inside an item is proposed with its options and a ranking,
and none of them is decided until the owner accepts it.

## Built

This design was built on branch `feat/git-policy-cli-zfs-slot-identity-store-per-slot-files`,
on 2026-11-21,
after the decision record accepted it,
with the gate's recheck's low findings R1 to R8 answered in this text and in the code:
R1 in the removal of the two known staging names,
which the project placed with plan step 18's `doctor` while the sweep stays unlisted,
and in the sweep's control that a run lists nothing,
as `doc/decision/cli-git-zfs-slot-identity-store-format.md` records under
"Who removes a killed writer's staging file",
and in sections 1,
2,
4 and 5,
R2's citations,
length and status sentences,
R3's grammar argument,
R5's wording about `O_EXCL`,
R6's variant counts and option B's owed controls,
R7's Limitations line,
and R8's two evidence points.
R4's missing per-option pros and cons are not added,
because the owner has accepted the first-ranked option of every
choice and a ranking nobody decides any more would be prose about nothing;
each open choice keeps the one reasoned paragraph it already had.
Nothing of this design is approved or final,
and only the owner declares it so.

## The decision's six items, and the section that answers each

The decision leaves six items to a design,
and this document answers them in seven sections:

- the folder's names,
  owners and modes:
  section 1;
- how the parent's record is replaced:
  section 3;
- a partly written slot file,
  and one that appears while a reader reads:
  sections 2 and 4;
- the per-file size bound,
  and what replaces the whole-store bound:
  section 4;
- what plan step 8's sweep reads,
  and in which order:
  sections 5 and 7;
- the issue 6327 size check and the lock-hold document:
  sections 6 and 7.

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
which holds for each worktree slot the identities the attach checks compare against (the folders' device,
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
the store at `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities` becomes a folder at
that same path,
and a machine that never ran the root setup has neither shape to convert.

Those words are kept in the letter the build can keep:
the file is created with `O_EXCL`,
and exclusion at the record's own name is `linkat`'s `EEXIST`,
not the creation's,
because the creation is under a random staging name that excludes no second recorder.
The slot's record file is created with `O_EXCL`,
under a staging name that begins with a dot,
and one `linkat` is what makes the record's own name exclusive,
and its record name is given to it by one `linkat` that never replaces,
after which the staging name is removed (section 2).
The option that section 2 ranks second makes the record instead with `O_TMPFILE` and one `linkat`,
so no file is ever created with `O_EXCL` on its common path:
that departure from the accepted words is a flagged choice the project must accept explicitly,
never the baseline this design proposes.

The repository's concurrency rule is the reason the folder was chosen,
and it is the rule this design must keep (`AGENTS.md`,
rule `NLK`):
"Concurrency is lock-free first (atomic rename,
`O_EXCL`,
one owner per item);
an unavoidable lock covers the smallest unit,
never a whole repository,
with its reason written beside it."
The store's single file could not be made safe by an atomic rename,
because two first-recording sweeps that both rebuilt the file would let the later rename record an identity
taken from a folder that changed in between,
which is the refresh the accepted design forbids (`package/git-policy/cli/SPEC.md:5475`,
and the design's own rule at
[`doc/planning/cli-git-zfs-worktree-design-proposal.md:612` to
`:613`](cli-git-zfs-worktree-design-proposal.md)):
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
  the device field's anonymity and the unmeasured reboot question (`:5386` to `:5401`),
  the one-mebibyte read bound and the append that may not outgrow it (`:5407` to `:5421`),
  the root-control checks (`:5423` to `:5440`),
  the write-once rule (`:5441` to `:5450`),
  the lock and its measured wait (`:5451` to `:5474`),
  the paragraph that calls the lock the route's one non-lock-free place (`:5475` to `:5489`),
  the two parent writers it names (`:5490` to `:5499`),
  and the one-store-one-checkout paragraph (`:5500` to `:5508`).
  Three passages outside that range decide things this design touches:
  "The refusals" (`:5610`),
  whose store,
  store-control and append bullets (`:5629` to `:5638`) name the line number,
  the torn-record repair and the folder's free space,
  and "The administrator's adoption" (`:5644`),
  whose surviving-slots sentence is `:5658`.
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
  and `slot_identity_store_tests.rs`,
  read whole.
- The attach itself,
  `package/git-policy/cli/src/native/worktree_clone_mount/clone_slot_walk.rs`:
  `hold_clone_slot`,
  which reads the store at `:333`,
  refuses `NoParentRecord` at `:336`,
  and compares the walked parent at `:338` before it looks the slot up at `:342` and compares it at `:346`,
  and `hold_clone_slot_under_root_control` at `:384`.
  `HoldCloneSlotRequest.store` is a path,
  and almost every control in `clone_slot_walk_tests.rs`,
  `attach_post_check_tests.rs` and `parent_adoption_decision_tests.rs` (whose `recording_slots` writes a
  version 1 file and calls `SlotIdentityStore::read`) builds one.
- The publication primitives in
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_publish_file.rs`:
  `open_unnamed` at `:218`,
  `link_unnamed` at `:282` (which takes a destination path) and `move_temporary_into_place` at `:333`,
  all of them private `fn`s,
  beside `package/git-policy/cli/src/native/foundation/rename_that_never_replaces.rs` (`rename_noreplace`,
  `exchange`,
  `rename_unsupported`) and `private_storage.rs`'s `staging_beside` at `:210` and `sync_directory`.
- The ACL capture and the bound it enforces,
  `clone_folder_identity_acl.rs`:
  `MAXIMUM_ACL_BYTES = 65536` at `:34`,
  and `read_acl`'s refusal above it at `:143`.
- The two source guards that name the store and fail on a stale entry:
  `source_guard/service_lock_inventory_tests.rs:124` (its test at `:320` panics at `:325`) and
  `source_guard/blocking_code_off_the_runtime_rules_tests.rs:408` (whose `:197` says the test fails on an
  entry that matches nothing).
- The root-free fixture seam,
  `root_namespace_test_support.rs`,
  which is how a control that needs a root-owned store builds one,
  and `e2e/mount-git-worktree-clone-datasets-guest.ts:5`,
  whose doc comment names the store path the guest arm writes.
- Step 5's mode,
  `package/git-policy/cli/src/native/worktree_clone_mount_handler/adopt_worktree_parent_mode.rs`
  (`recorded_slots`,
  `read_slot`,
  `adopt_at`),
  and the decision table it feeds,
  `parent_adoption_decision.rs` (`AdoptionFacts`,
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
  the recheck rule at `:425` ("Recheck recorded identities,
  the mount and the visible pointer before resuming"),
  the sweep's grammar and per-slot claim at `:559` to `:561`,
  the receipts' shape at `:564` to `:571`,
  "Never refresh a slot identity" at `:612` to `:613`,
  and "Descriptor mount and unmount" at `:597` to `:623`,
  whose sentence at `:623` decides section 3's constraint:
  "Never overwrite surviving slot identities or implicitly adopt a replacement for an existing binding",
  with the quiescence sentence at `:625` ("Until adoption,
  refuse managed mutations on the mismatched parent") and the unmount's recorded-parent reopen at `:674`.
- The implementation plan,
  [`doc/planning/cli-git-zfs-worktree-implementation-plan.md`](cli-git-zfs-worktree-implementation-plan.md):
  step 4's "It also reads and rechecks the slot identity store,
  at this path" at `:597`,
  step 8's sweep,
  which records "the slot's first-observation identity before attach",
  and `doctor`'s report of "the recorded parent and slot identities with the failing field or component" at
  `:1636`.
- Issue 6327,
  now closed,
  whose body names the defect this design must not reintroduce:
  "`replace_the_parent_record` ...
  rewrites the store without the size check `append_under_the_lock` now has ...
  A store of exactly the bound (1 MiB),
  adopted with a wider parent record,
  ended at 1,048,985 bytes,
  and the next read refused it."
- The publication precedent in this package,
  named under "Precedent in this repository".

### Measured

One probe,
`probe_publish_primitives.py`,
kept in this round's scratch folder at
`/home/user/temp/agent/zfs-slot-identity-store-per-slot-files-design-2026-11-21/scratch/keep/`,
run once on 2026-11-21 as the account,
on kernel `7.3.5-1-cachyos`,
in its own capped scope,
which is a transient systemd scope with a runtime,
CPU,
memory,
task and I/O weight limit sized to one probe,
inside `/home/user/temp/agent/zfs-slot-identity-store-per-slot-files-design-2026-11-21/scratch/probe-home`,
which `findmnt` reports as `zpcachyos/ROOT/cos/home`,
file system `zfs`,
`rw,noatime,xattr,posixacl,casesensitive`.
It printed these lines,
and every fact below is one of them:

```text
MEASURE store-folder = /home/user/temp/agent/zfs-slot-identity-store-per-slot-files-design-2026-11-21/scratch/probe-home/store
MEASURE O_TMPFILE-value-chosen = 0o20200000 (__O_TMPFILE | O_DIRECTORY)
MEASURE O_TMPFILE-value-the-earlier-probe-used = 0o20000000 (__O_TMPFILE alone, refused EINVAL)
MEASURE openat-O_TMPFILE-accepted = ok, nlink 0
MEASURE tmpfile-nlink-before-the-proc-arm = 0
MEASURE linkat-proc-arm-on-an-unnamed-file = ok
MEASURE tmpfile-nlink-after-the-proc-arm = 1
MEASURE linkat-proc-arm-onto-a-taken-name = refused EEXIST
MEASURE tmpfile-nlink-before-the-empty-path-arm = 0
MEASURE linkat-AT_EMPTY_PATH-arm-on-an-unnamed-file = ok
MEASURE linkat-AT_EMPTY_PATH-arm-onto-a-taken-name = refused EEXIST
MEASURE published-mode-owner-size = 0o644 uid=1000 bytes=7
MEASURE published-bytes = b'record\n'
MEASURE listing-unchanged-after-an-unlinked-file-closed = True
MEASURE directory-fsync = ok (no exception)
MEASURE renameat2-RENAME_NOREPLACE-onto-a-taken-name = refused EEXIST
MEASURE staged-name-survives-that-refusal = True
MEASURE renameat2-RENAME_NOREPLACE-onto-an-absent-name = ok
MEASURE staged-name-after-that-publish = False
MEASURE slot-b-bytes-after-that-publish = b'record\n'
MEASURE openat-O_EXCL-on-a-dot-staging-name = ok, mode 0o644
MEASURE staging-nlink-before-the-link = 1
MEASURE linkat-from-a-staged-name-onto-an-absent-name = ok
MEASURE linkat-from-a-staged-name-onto-a-taken-name = refused EEXIST
MEASURE unlinkat-of-the-staged-name = ok
MEASURE record-bytes-after-the-staged-name-was-removed = b'record\n'
MEASURE record-nlink-after-the-staged-name-was-removed = 1
MEASURE directory-fsync-after-the-staged-publication = ok (no exception)
MEASURE renameat2-RENAME_EXCHANGE = ok
MEASURE slot-b-bytes-after-the-exchange = b'second\n'
MEASURE renameat2-plain-replacing-rename = ok
MEASURE slot-new-after-that-rename = False
MEASURE openat-O_EXCL-onto-a-taken-name = refused EEXIST
MEASURE create-a-255-byte-name = ok
MEASURE create-a-256-byte-name = refused ENAMETOOLONG
MEASURE posix-acl-of-65524-bytes = accepted, read back 65524 bytes
MEASURE posix-acl-of-65532-bytes = accepted, read back 65532 bytes
MEASURE posix-acl-of-65540-bytes = refused E2BIG
MEASURE user-xattr-of-65536-bytes = accepted, read back 65536 bytes
MEASURE user-xattr-of-65537-bytes = refused E2BIG
MEASURE stat-device-of-/var/lib = 31
MEASURE stat-device-of-/home = 57
MEASURE store-parent-exists = False
```

Each fact the design rests on,
and the printed line that showed it:

- The kernel's `O_TMPFILE` is `0o20200000`,
  which is the flag together with `O_DIRECTORY`,
  and a file opened with it starts at link count zero (`O_TMPFILE-value-chosen`,
  `openat-O_TMPFILE-accepted`,
  `tmpfile-nlink-before-the-proc-arm`).
  The value `0o20000000` that the earlier round's probe used is the flag alone
  (`O_TMPFILE-value-the-earlier-probe-used`),
  and that is why its `O_TMPFILE` arm answered `EINVAL` and its two `linkat` arms never ran.
- Both `linkat` spellings give an unnamed file its name,
  each tried first on a file that still has no name (`linkat-proc-arm-on-an-unnamed-file`,
  `linkat-AT_EMPTY_PATH-arm-on-an-unnamed-file`),
  and each answers `EEXIST` onto a name that is taken (`linkat-proc-arm-onto-a-taken-name`,
  `linkat-AT_EMPTY_PATH-arm-onto-a-taken-name`).
- A file made with `O_EXCL` under a dot staging name keeps mode `0644`
  (`openat-O_EXCL-on-a-dot-staging-name`),
  one `linkat` from that staged name onto a free name publishes it
  (`linkat-from-a-staged-name-onto-an-absent-name`),
  the same call onto a taken name answers `EEXIST` (`linkat-from-a-staged-name-onto-a-taken-name`),
  and after the staged name is unlinked the record keeps its bytes and one link
  (`unlinkat-of-the-staged-name`,
  `record-bytes-after-the-staged-name-was-removed`,
  `record-nlink-after-the-staged-name-was-removed`).
- The published file is mode `0644`,
  owner `1000`,
  and holds its bytes whole (`published-mode-owner-size`,
  `published-bytes`);
  an unnamed file closed without a link leaves the listing unchanged
  (`listing-unchanged-after-an-unlinked-file-closed`).
- The folder's own `fsync` succeeds,
  and answers by not raising (`directory-fsync`,
  `directory-fsync-after-the-staged-publication`).
  Python's `os.fsync` returns `None` on success,
  so the earlier round's `== 0` test printed "failed" whatever happened.
- `renameat2` with `RENAME_NOREPLACE` answers `EEXIST` onto a taken name and leaves the staged name in place
  (`renameat2-RENAME_NOREPLACE-onto-a-taken-name`,
  `staged-name-survives-that-refusal`),
  publishes onto an absent name and removes the staged name (`renameat2-RENAME_NOREPLACE-onto-an-absent-name`,
  `staged-name-after-that-publish`);
  `RENAME_EXCHANGE` succeeds and the name it was exchanged with holds the other's bytes (`renameat2-RENAME_EXCHANGE`,
  `slot-b-bytes-after-the-exchange`,
  which prints one of the two names' bytes,
  so the other name's are not among the printed lines);
  a plain replacing rename replaces in one step (`renameat2-plain-replacing-rename`).
- `O_EXCL` at a name that exists answers `EEXIST` (`openat-O_EXCL-onto-a-taken-name`).
- A 255-byte name is created in that folder and a 256-byte one is refused `ENAMETOOLONG`
  (`create-a-255-byte-name`,
  `create-a-256-byte-name`).
- A `system.posix_acl_access` value of 65,532 bytes is accepted and read back at that size,
  and 65,540 bytes is refused `E2BIG` (`posix-acl-of-65532-bytes`,
  `posix-acl-of-65540-bytes`).
  ACL values move in steps of eight bytes,
  so that pair alone places the ceiling anywhere from 65,533 to 65,539;
  a `user` attribute of 65,536 bytes is accepted and 65,537 is refused `E2BIG` (`user-xattr-of-65536-bytes`,
  `user-xattr-of-65537-bytes`),
  which is what places the attribute ceiling at `XATTR_SIZE_MAX`,
  65,536 bytes.
- `/var/lib` answers device `31` and `/home` answers `57` (`stat-device-of-/var/lib`,
  `stat-device-of-/home`),
  and `/var/lib/monochromatic-git-policy-cli` does not exist on this host (`store-parent-exists`),
  so the store's own device is not among the devices its records hold.

The earlier round kept a probe whose constant,
success test and missing arms made three of these facts wrong;
this round replaced it,
and the file above is the one that printed the lines quoted here.
No fact in this document comes from the replaced probe.

### Assumed, not measured

- That the store's own dataset answers the same for `linkat`,
  `unlinkat`,
  `renameat2` and a folder `fsync`,
  and for `O_TMPFILE` if the project chooses section 2's option B.
  Both are ZFS datasets of one pool on this host and both are mounted `xattr,posixacl`,
  but the probe ran on `home`,
  because nothing on this host may write under `/var/lib` as the account.
  A guest arm or a gate image must settle it,
  and section 7 lists that arm.
- That `/var/lib/monochromatic-git-policy-cli` is created root-owned with mode `0755` by the setup run of plan
  step 19,
  which does not exist yet.
  This design fixes what that run must do and cannot cite a line that already does it.
- That a checkout's slot name is at most `NAME_MAX` bytes.
  The sweep derives a slot name under the accepted design's grammar (lowercase letters and digits with
  interior hyphens,
  bounded by the slot component limit and the remaining dataset-name budget),
  Git refuses a worktree path whose last component the filesystem refuses,
  and this design adds no new refusal for a name that fits.
- That two sweeps meeting on one slot is possible but not frequent.
  Nothing in this repository measures how often ZED delivers two triggers for one clone,
  the accepted design serialises sweeps per slot under a per-slot root claim
  ([`doc/planning/cli-git-zfs-worktree-design-proposal.md:561`](cli-git-zfs-worktree-design-proposal.md)),
  and the design is safe at any rate.

## Precedent in this repository

This design invents no primitive.
Each step this design orders is one this package already ships,
and the design's job is to pick among them and say why.

- `package/git-policy/cli/src/native/worktree_copying/worktree_copy_publish_file.rs` publishes one staged file
  at a destination name without a lock:
  `open_unnamed` makes a file with no name (`O_TMPFILE`),
  `link_unnamed` gives it the destination name and nothing else's (through `/proc/self/fd` first,
  then `AT_EMPTY_PATH`),
  and `move_temporary_into_place` is the fallback for a file that had to be filled under a name:
  `renameat2` with `RENAME_NOREPLACE`,
  then `linkat` and `unlinkat` where the filesystem refuses that flag.
  Its reason is this design's reason word for word:
  "A file created under its final name and filled afterwards exists,
  for as long as the copy runs,
  with a prefix of the bytes ...
  so a kill in that window leaves a file at that name that is not what the source holds".
- `package/git-policy/cli/src/native/foundation/private_storage.rs` writes a record whole or not at all:
  `write_then_publish`,
  `staging_beside` (a `.<name>.<random>.partial` name created `O_EXCL`,
  mode `0600`,
  never following a link),
  `persist_noclobber`,
  `read_regular_file` with a byte bound,
  and `sync_directory`,
  which opens a folder read-only and syncs it.
- `package/git-policy/cli/src/native/foundation/private_storage_replacing_a_record.rs` replaces a record in
  one rename (`replace_private_file`),
  because "A record this program rewrites is read by a later run as a whole,
  and a truncating write leaves it empty for a kill that lands between the truncation and the bytes".
- `package/git-policy/cli/src/native/foundation/rename_that_never_replaces.rs` holds `rename_noreplace`,
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

Three of those primitives are not reusable as they stand,
and section 6 says where they must go:
`open_unnamed`,
`link_unnamed` and `move_temporary_into_place` are private `fn`s of
`worktree_copying/worktree_copy_publish_file.rs` (`:218`,
`:282`,
`:333`),
`link_unnamed` takes a destination path rather than a held folder,
and `rename_that_never_replaces` and `private_storage::staging_beside` (`:210`) are path-based too.
`worktree_clone_mount/uses/` links only `foundation` and `subvolume`,
and rules `LYD`,
`LYL` and `LYF` police that,
so a store that names entries relative to one held folder needs descriptor-relative forms of these primitives
in `foundation/`.
That is a change the build makes,
not a primitive this design may claim already exists:
"every record,
link,
rename,
unlink and `fsync` is relative to that one descriptor" is a requirement on code that is not written yet.

## 1. The folder, its files, their names, owners and modes

### The example store

One store for the checkout at `/home/user/Monochromatic`,
after the setup run and two sweeps,
carrying the access ACL bytes the specification's own example uses (`package/git-policy/cli/SPEC.md:5362`):

```text
/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities/   root 0755
  .parent                                                                   root 0644
  add-cache-layer                                                           root 0644
  fix-flaky-test                                                            root 0644
```

```text
# /var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities/.parent
# monochromatic-git-policy-cli worktree clone slot identities, version 2
parent - 57:66304:1000:0755:0200000001000700ffffffff02000500e803000004000500ffffffff10000500ffffffff20000500ffffffff:-
```

```text
# /var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities/add-cache-layer
# monochromatic-git-policy-cli worktree clone slot identities, version 2
slot add-cache-layer 57:67410:1000:0755:-:-
```

```text
# /var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities/fix-flaky-test
# monochromatic-git-policy-cli worktree clone slot identities, version 2
slot fix-flaky-test 57:67833:1000:0755:-:-
```

The ACL sits on the parent's record and not on a slot's,
which is where the specification's own example puts it (its `parent` line carries the access ACL and its
`slot` line carries `-:-`).
The value is 44 bytes:
a four-byte version word and five eight-byte entries,
the second of them a named user whose id is `1000`.
That is the shape the store's control for the specification's example asserts (`access.len() % 8 == 4` and a
mask entry among the tags,
`slot_identity_store_tests.rs:765` to `:769`),
so the example store here is one that control accepts.

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
  Exactly two forms exist and no third:
  `.<32 lowercase hex>`,
  which section 2's publication makes for a slot's record,
  and `..parent.<random>.partial`,
  which `private_storage::staging_beside` makes for `.parent` (its own form is `.<name>.<random>.partial`,
  and the name there is `.parent`).
  No reader reads a staging entry as a record,
  and a dot entry that matches neither form is reported by `doctor`,
  because a third form means a writer this design has not accounted for.
  A leading dot is what the slot grammar already refuses,
  so no slot can ever be named like one.
  Who removes them is section 2's answer:
  a writer removes its own,
  the adoption removes the two known forms a killed writer left,
  and plan step 18's `doctor` reports and removes them
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  "Who removes a killed writer's staging file"),
  so the adoption is their only remover until that step lands.
  No sweep run lists the folder,
  so no sweep run removes one,
  and a leftover affects no reader,
  because no reader reads a staging name.

No suffix and no second level is needed,
and the reason is the grammar of names that can arrive,
not only the filesystem's limit:
the accepted design's sweep derives a slot name from the dataset name (lowercase letters and digits with
interior hyphens,
bounded by the slot component limit and the remaining dataset-name budget,
[`doc/planning/cli-git-zfs-worktree-design-proposal.md:559` to
`:561`](cli-git-zfs-worktree-design-proposal.md)),
`slot_name_refusal` sets no length limit,
and the probe created a 255-byte name in the store's folder and was refused a 256-byte one
(`create-a-255-byte-name`,
`create-a-256-byte-name`).
How long a name the sweep can derive is not measured:
the remaining dataset-name budget bounds it below 255 by the length of the parent dataset's path,
and this design does not compute that maximum for a real path.
A file named `<slot>.slot` would refuse any name near that limit,
which is a refusal for a name Git made whichever way the budget falls,
and option C's rejection therefore rests on not wanting a length rule at all rather than on a measured name.
A subfolder of slots would fit every name too,
and it is option B of this section.

### The modes

Record files are mode `0644`,
root-owned,
which is what the route's receipts are and what the current store is read as (its reader refuses only group
and other write).
The record's bytes are written with mode `0644` set before the name exists,
so a record is never readable at a wider mode than the one it keeps.

### The options and their ranking

#### Option A: flat, the slot's name as its file's name and `.parent` for the parent

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

#### Option B: two levels, `parent` beside a `slots/` folder

On the example store the top level lists `parent` and `slots`,
and `slots/` lists `add-cache-layer` and `fix-flaky-test`.

Pros:
no reserved name,
and the parent's record is visible.

Cons:
a second descriptor and a second control check,
two listings for the adoption,
and the parent's record sits outside the folder a reader already holds.

#### Option C: one level with a suffix per kind, `<slot>.slot` and `parent.slot`

On the example store the folder lists `parent.slot`,
`add-cache-layer.slot` and `fix-flaky-test.slot`.

Pros:
every name says its kind.

Cons:
a 255-byte slot name has no room for the suffix,
so either a name the sweep can produce is refused or the name is hashed and no longer legible.

#### Ranking: A, then B, then C

A is first because the second level buys only a visible parent name,
while making every read and every listing reach a second descriptor whose control terms must be checked again,
and the reserved name costs one sentence of grammar.
B is second because C cannot name a slot whose name is near `NAME_MAX`,
which the probe measured as 255 bytes created and 256 refused,
and the design must not refuse a folder Git made.

## 2. How a slot's first record is published

The decision's words for this step are "Each slot's file is made with `O_EXCL`",
and the proposed publication keeps them:
the file is created with `O_EXCL` under a staging name that begins with a dot,
and its record name is given to it by one `linkat` that never replaces,
after which the staging name is removed.
This section's choice is the one place where the design could depart from those words,
so its first option is the literal one and the departure is the flagged option B.

### The publisher's first step: the parent is compared before anything is published

A publication begins by reading the folder,
not by writing to it:

1.  A name that is no plain slot name refuses `SlotName` before anything is created,
    which is the refusal `main` makes at `slot_identity_store.rs:325`.
2.  It opens `.parent` on the held folder descriptor and compares the `.claude/worktrees` folder its walk
    captured against that record.
    An absent `.parent` refuses `NoParentRecord`,
    which is the refusal `main`'s append makes at `slot_identity_store.rs:333` ("so there is nothing to
    compare that folder against"),
    and a record whose fields differ refuses `ParentIdentity` with every field that moved,
    the same fields and sentence the attach's own refusal carries (`clone_slot_walk.rs:338` to `:340`).

Why this order,
and not the record first:

- The accepted design refuses managed mutations on a parent that differs from the recorded one ("Until
  adoption,
  refuse managed mutations on the mismatched parent",
  [`doc/planning/cli-git-zfs-worktree-design-proposal.md:625`](cli-git-zfs-worktree-design-proposal.md)),
  and a slot record is what later lets an attach proceed under that parent.
- A record published before the parent was compared would be a first observation taken under a parent nobody
  had compared,
  and an adoption that later replaced that parent would count it as a surviving slot that agrees.
- `main` already refuses `NoParentRecord` before an append,
  so keeping it costs no new invariant:
  `.parent` is never removed,
  so the check cannot go stale.

The compare is check-then-act.
A `.parent` replacement landing between the read and the link means the record lands under the parent read a
moment earlier,
and the rest of this section and section 5 say what that costs:
a slot record holds no field of the parent's identity,
so the record stays true of its slot folder,
and the adoption's listing and the next adoption's reconciliation are what notice it.

### The steps

A record's bytes are the version 2 header line and one record line,
which is exactly what the example store's files hold.
Publication fills a staging file and then gives that file its record name in one step that replaces nothing,
so the record's name never holds a prefix of the bytes:

1.  A sweep that finds no record for a slot has already passed the parent's compare,
    and captures the slot folder's identity from the descriptor it holds,
    which is the plan's step 8 first-observation record.
2.  It formats the two lines and checks their length against the per-file bound of section 4.
    The bytes are known before anything is created,
    so a record too large to read is refused before a name exists.
3.  It creates a staging file in the held folder descriptor with an exclusive creation:
    `openat(folder, ".<32 lowercase hex>", O_CREAT | O_EXCL | O_WRONLY | O_NOFOLLOW | O_CLOEXEC, 0o644)`,
    where the hex is random per attempt.
    The probe created such a name and read back mode `0644` (`openat-O_EXCL-on-a-dot-staging-name`).
4.  It writes the two lines,
    `fchmod`s the descriptor to `0644`,
    and `fsync`s it,
    so the bytes are durable before any name can hold them.
5.  It gives the file the slot's name with one `linkat` that never replaces:
    `linkat(folder, ".<32 lowercase hex>", folder, name, 0)`.
    The probe published a staged file this way (`linkat-from-a-staged-name-onto-an-absent-name`) and measured
    `EEXIST` onto a name that exists (`linkat-from-a-staged-name-onto-a-taken-name`),
    which is the race's whole answer.
6.  It removes the staging name and syncs the folder:
    `unlinkat(folder, ".<32 lowercase hex>", 0)`,
    then `fsync(folder)`,
    so the record's name is durable and the staging name is gone.
    After the unlink the probe read the record's bytes back whole with one link
    (`record-bytes-after-the-staged-name-was-removed`,
    `record-nlink-after-the-staged-name-was-removed`).
7.  A failure at any step removes the staging name this attempt made,
    and a folder that refuses both calls refuses the write,
    naming the folder and both calls,
    so nothing is attached on a record the store did not keep.

No reader ever sees a partial record:
the record's name is absent until step 5 and then holds every byte,
and the staging name is a dot name no reader reads.

### The fallback where a folder refuses a link

A folder that refuses `linkat` of a regular file (`EPERM`,
`EOPNOTSUPP` or a filesystem with no hard links) publishes with
`renameat2(folder, ".<32 lowercase hex>", folder, name, RENAME_NOREPLACE)`,
which the probe measured answering `EEXIST` onto a taken name and leaving the staged name in place,
and publishing onto an absent name in one step with the staged name gone
(`renameat2-RENAME_NOREPLACE-onto-a-taken-name`,
`staged-name-survives-that-refusal`,
`renameat2-RENAME_NOREPLACE-onto-an-absent-name`,
`staged-name-after-that-publish`).
That call also removes the staging name,
so step 6's unlink is skipped on that arm.

Where the folder refuses that flag as well (`rename_unsupported`),
the publication refuses as a `Write` naming the folder and both calls,
and it never falls back to a rename that replaces,
because a publication that may replace another's record is the refresh this design forbids.

### What a second recorder of one slot gets

The kernel answers exactly one writer.
The loser's `linkat` answers `EEXIST`,
and the loser then does three things and reports:

1.  It removes its own staging name and,
    if that succeeds,
    `fsync`s the folder,
    which is one call and also makes the winner's name durable before the loser reads it.
    Without that sync a crash could leave the loser with a record read from a name the winner had not flushed,
    and the mount does not survive such a crash,
    so this is the cheap end of that window.
2.  It opens the name that is now there and reads it,
    which is one `openat` on the folder descriptor it already holds.
3.  It answers `SlotAlreadyRecorded { path, slot, identity }`,
    carrying the winner's record,
    and the sweep carries on with that identity:
    the folder compared against is the published one,
    never this run's own capture,
    and the identity compare refuses with the fields that moved where the two captures differ,
    which is the same refusal a single sweep would have produced.

So a sweep that loses a race receives an error carrying the winner's record,
not a bare failure and not a silent success,
and nothing is rewritten:
a slot's record still has exactly one version for its whole life.

Two recorders meeting is rare and needs no coordination:
the accepted design serialises sweeps per slot under a per-slot root claim
([`doc/planning/cli-git-zfs-worktree-design-proposal.md:561`](cli-git-zfs-worktree-design-proposal.md)),
so two triggers for one clone usually meet at that claim and not here,
and the store's exclusivity is the backstop that holds if they do meet.

### Durability and the kill window

A kill leaves one of three states,
and each is readable:

- before step 5:
  a staging entry and no record;
- between step 5 and step 6:
  the record,
  whole,
  and a staging entry naming the same file;
- after step 6:
  the record alone.

Nothing reads a staging entry,
and each party that removes one a kill left does so at a moment its own precondition makes safe:

- a writer removes its own staging name on its normal path and on every failure path,
  so a writer that returned never leaves one;
- the adoption lists the folder as its first step and removes every entry matching either known staging form
  before it stages its own record,
  and its quiescence precondition is what makes that safe,
  because no other writer is live to own those names.

A sweep run lists no name at all (choice 9),
so no sweep run removes one,
and the adoption is the only remover until plan step 18 lands.
That step's `doctor` reports the known forms with their count and age and any dot entry that matches neither,
and removes the known forms,
which is the answer the project gave on 2026-11-21
(`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
"Who removes a killed writer's staging file").
A machine that runs no adoption keeps a killed writer's litter until that report removes it,
and nothing is affected meanwhile:
no reader reads a staging name,
so a leftover entry changes no read,
no compare and no mount.
That is the reach the decision asked for when it added the removal beside the adoption (R1),
and it is why option A's cons say what a machine that runs no remover keeps.
No run needs a lock or a timeout to be sure another writer is gone.

### The alternative this design flags: a file with no name and one link

The other way to publish the same record is to give no name to the file until it holds every byte:
`openat(folder, ".", O_TMPFILE | O_DIRECTORY | O_RDWR | O_CLOEXEC, 0o644)`,
then write,
`fchmod` and `fsync`,
then `linkat(AT_FDCWD, "/proc/self/fd/<n>", folder, name, AT_SYMLINK_FOLLOW)` and
`linkat(fd, "", folder, name, AT_EMPTY_PATH)` where the first answers `ENOENT` or `EPERM`.
The probe measured the file starting at link count zero and both spellings answering zero on a fresh unnamed
file,
each answering `EEXIST` onto a taken name (`openat-O_TMPFILE-accepted`,
`tmpfile-nlink-before-the-proc-arm`,
`linkat-proc-arm-on-an-unnamed-file`,
`linkat-AT_EMPTY_PATH-arm-on-an-unnamed-file`,
`linkat-proc-arm-onto-a-taken-name`,
`linkat-AT_EMPTY_PATH-arm-onto-a-taken-name`),
and a file closed without a link leaving no entry (`listing-unchanged-after-an-unlinked-file-closed`).

It is flagged because it departs from the decision's words:
no file is created with `O_EXCL` on that path,
and its fallback's staging name would be the only `O_EXCL` the store uses.
Its other costs are measured or read:

- the second spelling needs `CAP_DAC_READ_SEARCH`;
  the kernel answered `ENOENT` from `do_linkat` without it up to v6.9,
  and only from v6.10 accepts the call where the descriptor's open-time credentials are the caller's
  (`fs/namei.c`,
  read at v6.8,
  v6.9 and v6.10 with `gh api repos/torvalds/linux/contents/fs/namei.c`).
  The route's floor is Linux 6.6 and the demonstrated guest is 6.8,
  so the `/proc` spelling must stay first and the empty-name spelling is only a fallback.
- a root without `CAP_DAC_READ_SEARCH` and without `/proc` would have both spellings answer `ENOENT` or
  `EPERM` while the file system never refused the file's creation,
  so a build of this option takes the staged `O_EXCL` path above in that case,
  or refuses as a `Write` naming both calls.
  A build that reused `write_refusal` unchanged would tell the operator the store is missing,
  which is the wrong report (`slot_identity_store_refusals.rs:209` to `:214`).
- the store's own dataset's support for the unnamed file is not measured (section "Assumed,
  not measured"),
  while the first option needs only a hard link.

### The options and their ranking

#### Option A: `O_EXCL` under a staging dot name, one link, then the unlink

On the example store `add-cache-layer` appears whole at its name,
and a `.a1b2c3d4e5f60718293a4b5c6d7e8f90` entry exists only for the instant between the link and the unlink.

Pros:
the file is made with `O_EXCL`,
as the decision's own words say,
the same one publication on every filesystem that makes a hard link,
no unnamed-file support to assume,
no partial record at the record's name,
and the same staging remover the parent's replacement needs anyway.

Cons:
a kill leaves a dot entry,
which the writer's own path and the adoption remove,
and which plan step 18's `doctor` is to report and remove,
but which stands on a machine that runs neither until then;
one arm of the publication is the rename that never replaces,
with its own two controls and its double-refusal arm;
and the common path is two names and two calls more than a single create.

#### Option B: a file with no name and one link, with the staged fallback

On the example store `add-cache-layer` appears whole at its name,
and a kill before the link leaves nothing at all.

Pros:
no name exists before the record is complete,
so a kill leaves no litter on the common path,
and both link spellings are the package's own code.

Cons:
it departs from the decision's words,
so the project must accept that explicitly,
its empty-name spelling carries the capability and kernel-version boundary above,
the store's dataset's unnamed-file support is unmeasured,
and it owes four more arms than option A:
both link spellings refusing (the staged path or a `Write` naming both calls),
a filesystem that refuses the unnamed file (the staged path,
with the file-making call a parameter),
the `AT_EMPTY_PATH` spelling on its own,
and each of those against a folder that refuses a hard link.

#### Option C: `O_EXCL` at the record's name and a completeness marker

On the example store `add-cache-layer` exists from the first `openat`,
and a reader must treat it as incomplete until a marker line is written,
so a kill between the two leaves a name that refuses every read of that slot until root removes it.

Pros:
the record's name is created with `O_EXCL`,
as the decision's words say.

Cons:
a partial record is visible at the record's name,
a reader must retry rather than trust,
and a killed writer leaves permanent damage.

#### Option D: `O_EXCL` at the record's name and a reader that retries a short file

On the example store the same partial name exists,
and a reader that finds fewer than two complete lines waits and reads again.

Pros:
no second file and no marker.

Cons:
every reader carries a retry and a wait no measurement sizes,
and a torn or empty file from a killed writer is indistinguishable from one being written,
so the store can neither refuse nor read it.

#### Ranking: A, then B, then C, then D

A is first because it keeps the accepted words literally,
collapses the publication to one variant on every filesystem,
and asks the store's dataset for nothing the probe did not measure (a hard link and a folder `fsync`).
B is second because its only real gain over A,
no litter after a kill,
is a gain the adoption's remover already covers,
and it pays for that with a departure the project must accept,
an unmeasured filesystem capability and a capability floor.
C is third because a marker at least tells a killed writer's leftover from a live one,
which D's retry cannot,
so D both waits on nothing and cannot refuse.

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

1.  The mode reads `.parent` under root control and decides from it,
    and reconciles every recorded slot exactly as the decision table already orders (`decide_adoption`'s walk,
    identified device and inode,
    policy,
    bindings,
    slots,
    fresh lookup).
2.  It formats the new record (the version 2 header and one `parent - ...` line) and checks its length against
    the per-file bound.
3.  It makes a staged file in the held folder descriptor,
    writes the two lines,
    `fsync`s it,
    and sets mode `0644`,
    which is `private_storage`'s `staging_beside` plus `sync_when_durable`.
    The name that form makes here is `..parent.<random>.partial`,
    the second of the two staging forms section 1 names,
    and `.parent` is never removed,
    so the rename's target always exists.
4.  It publishes with one rename that replaces the target:
    `renameat2(folder, staged, folder, ".parent", 0)`,
    which the probe measured replacing in one step,
    and which is what `replace_private_file` already does by `tempfile`'s `persist`.
5.  It `fsync`s the folder,
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
which the specification already separates in words at `package/git-policy/cli/SPEC.md:5490`.
Today one writer serves both intents,
because `record_parent_identity_under_root_control`'s `replace_the_parent_record` replaces the record whenever
it is called,
so the setup run that plan step 19 still owes would silently replace an adopted identity unless it knew which
intent it had.
The design carries the specification's split into two writers:

- `publish_parent_identity_under_root_control` publishes `.parent` only where it is absent,
  by the same no-replace step a slot record uses,
  and a second setup run compares the live parent with the record it finds:
  equality is success,
  and only a difference refuses `ParentAlreadyRecorded`.
  The plan's step 19 boot unit runs on every boot,
  so a second run is the normal case and must not report an error.
  The compare runs before the name is made,
  and the exclusivity is the publication's own no-replace step:
  two first runs that both find `.parent` absent still publish once,
  because the second `linkat` answers `EEXIST` and its run compares what it finds.
- `replace_parent_identity_under_root_control` is the adoption's one writer,
  and it refuses where `.parent` is absent.

Both keep the root-control checks inside the step that writes.

### The options and their ranking

#### Option A: one file replaced by one rename

On the example store the adoption leaves `.parent` holding this one record line:

`parent - 57:67990:1000:0755:0200000001000700ffffffff02000500e803000004000500ffffffff10000500ffffffff20000500ffffffff:-`

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

#### Option B: re-read `.parent` and compare its identity just before the rename

On the example store the adoption reads `.parent` again immediately before the rename,
compares its device and inode with the record the decision was made from,
and refuses where they differ.

Pros:
narrows the window without ever exposing an unjudged record,
and costs one `openat`,
one `fstat` and one parse.

Cons:
still check-then-act,
so a replacement landing after that read is missed as it is under A,
and it proves the record only as of that read.

#### Option C: `RENAME_EXCHANGE` and a comparison of what it displaced

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

#### Option D: `RENAME_EXCHANGE` used only to make "must exist" atomic

On the example store the adoption exchanges the staged record with `.parent`,
then checks that the file it displaced is the record the decision read,
and removes it;
where the check fails it exchanges back and refuses.

Pros:
a publish that cannot create `.parent` out of nothing,
even if a future change ever removed it,
which becomes a refusal instead of a silent first write.

Cons:
two renames on the common path,
and the same unjudged window as C,
because the exchange publishes before the check runs.

#### Option E: generation-numbered parent files, `.parent.0007`, and the highest complete one read

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

#### Ranking: A, then B, then C, then D, then E

What two concurrent adoptions do is parity with `main`,
not a loss:
`adopt_at` reads the store and decides (`adopt_worktree_parent_mode.rs:300` to `:325`) before the write takes
the lock (`:326`),
so `main` also leaves the later writer's record.
Both decisions were individually valid,
and the first administrator's reported success is overwritten.
A is first because it keeps one name and one step,
and because `.parent` is never removed,
so "must exist" already holds without a second rename.
B is second because it narrows that race for one read without ever exposing an unjudged record.
C and D are third and fourth because each buys a detection with a window in which an unjudged record stands,
and D's extra rename buys only the must-exist proof that the never-removed invariant already gives.
E is last because it moves a "highest complete generation" ladder into every parent read and adds a removal
step.

## 4. What a reader refuses

The route's reader is one function over a held folder descriptor and one name,
and the folder's listing is a second function over the same descriptor.
Every refusal names the file it judged and the person or command that repairs it,
as `StoreRefusal`'s sentences already do.

### A record file's bound, and what replaces the whole-store bound

One file may hold at most `RECORD_MAXIMUM_BYTES`,
and that number is derived from two limits rather than chosen:

- the widest identity token this kernel can produce is 262,203 bytes:
  two ACLs of `MAXIMUM_ACL_BYTES`,
  65,536 each (`clone_folder_identity_acl.rs:34`),
  at two hexadecimal characters a byte,
  which is 262,144,
  plus the device,
  inode,
  owner and mode fields of at most 20,
  20,
  10 and 4 digits,
  and five colons,
  which is 59 more;
- the widest record line is `slot ` plus a 255-byte name plus one space plus that token,
  which is 262,464 bytes,
  and 262,465 with its newline;
- the widest file is that line and the 72-byte version 2 header with its newline,
  which is 262,538 bytes;
- so `RECORD_MAXIMUM_BYTES` is 262,538 bytes,
  and a build writes it as the constant expression over `MAXIMUM_ACL_BYTES` and the name limit rather than as
  a literal (rule `MGN`).

No record can exceed it,
and the reason is not this bound but the capture's own:
`read_acl` refuses any attribute above `MAXIMUM_ACL_BYTES` when it reads a folder
(`clone_folder_identity_acl.rs:143`),
so a folder whose ACL the kernel would not hold never reaches a record,
and a folder that has one is refused as a read rather than written.

The two measured facts behind those numbers are the attribute ceiling and the name ceiling:
a `user` attribute of 65,536 bytes is accepted and 65,537 refused `E2BIG`,
which places the attribute ceiling at `XATTR_SIZE_MAX`,
while the ACL pair alone (65,532 accepted,
65,540 refused) could only place it anywhere from 65,533 to 65,539,
because ACL values move in steps of eight bytes;
and a 255-byte name is created with a 256-byte name refused `ENAMETOOLONG`.

The whole-store bound does not become a per-folder sum:
the store has no total bound,
and what replaces it is that one read costs one file at most.
Its old purpose,
"a runaway or replaced file must be named rather than read into memory" (`slot_identity_store.rs:52`;
the specification's own words are "a larger file is refused by that bound rather than read into memory",
`package/git-policy/cli/SPEC.md:5407` to `:5409`),
is now the per-file bound.
The refusal for a file over it names the file,
its size and the bound,
and says that no command in this repository removes a record or raises the bound,
which is what the store's `WouldNotFit` sentence says today.

What bounds a store's size is therefore not the store but the account's own clone count:
root adds one record per clone the account can make,
so a listing's length is the number of slots the checkout has,
and what limits that in practice is the delegated `zfs` grants the store does not control.
`doctor` reports the entry count beside the records,
so a store that grew is visible before it is walked.

### A missing folder or a missing record

A missing store folder refuses `Missing`,
naming the setup run of plan step 19,
as today's `Missing` names the run that writes the file.
A missing `.parent` refuses `NoParentRecord`,
which the attach,
the adoption and section 2's publisher all raise,
and which says what the record is for:
there is nothing to compare the parent folder against.
A missing slot record refuses `NoSlotRecord` to a reader that needs one,
and to the sweep it is not a refusal but the first observation section 2 publishes.
The publisher also refuses a name that is no plain slot name (`SlotName`,
`slot_identity_store.rs:325`),
and both refusals are made before anything is created,
which is the pair `main` tests and this design keeps (section 7).

### A partial record

A file that does not hold the version 2 header line followed by exactly one record line refuses,
naming the file and the line where it stopped,
which is today's `Line` refusal with a file in place of a store:
"Nothing is attached to a slot until the file reads whole."
With section 2's publication a partial file is not something a writer of this repository makes,
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
A staging entry (a leading dot that is not `.parent`) is ignored by every reader and by the listing,
and a dot entry that matches neither of the two known staging forms is reported by `doctor` rather than
refused or removed,
because it means a writer this design has not accounted for.
`doctor` reports the two known forms too,
with their count and age,
and removes them,
so a machine that runs no adoption shows what a killed writer left and can have it removed there
(`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
"Who removes a killed writer's staging file").

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

The folder that holds the store,
`/var/lib/monochromatic-git-policy-cli`,
is not checked a second time by a read,
and that costs nothing:
today's `check_the_folder_control` checks the folder of the store file,
which in this layout is the store folder itself,
and an account-owned stand-in for `/var/lib/monochromatic-git-policy-cli` would have to be root-owned to pass
the store folder's own owner test,
which an account cannot make.
The setup run creates that folder root-owned,
mode `0755` (plan step 19).

### The whole of a read

One read is:
one `openat` on the held folder descriptor with `O_RDONLY | O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC` (`O_NONBLOCK`
is what today's open carries at `private_storage.rs:761`,
so a pipe planted at a record's name is refused rather than waited on),
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

1.  The no-crossing walk from the pinned checkout reaches `.claude/worktrees` and captures its identity from
    the descriptor it opened.
    The walk needs no store file;
    the compare needs exactly one,
    `.parent`,
    opened on the store's folder descriptor.
2.  The walk reaches the slot and captures the slot folder's identity from the descriptor it opened.
    The compare needs exactly two files,
    `.parent` and the slot's own,
    `add-cache-layer` in the example.
3.  `fsmount` and `move_mount` need no store file,
    and the post-attach check compares the mount's identity against what the walk and the attach proved,
    never against the store.
4.  The recheck rule at `doc/planning/cli-git-zfs-worktree-design-proposal.md:425` ("Recheck recorded
    identities,
    the mount and the visible pointer before resuming") is satisfied without a second read of the slot's
    record,
    because that record has exactly one version for its whole life:
    section 2 publishes it once,
    no code path replaces or removes it,
    and a second read would be a read of the same bytes.
    That rests on one open question,
    and the dependency is stated rather than hidden:
    the Limitations section leaves "may a slot's record be removed when its clone is destroyed" to the owner,
    and a build that lets it be removed would let a held read go stale,
    so the recheck rule would then need the second read.
    The parent's record can change,
    and the attach's only exposure to that is the quiescence rule the adoption mode already carries,
    with a stale read refusing rather than permitting.

So an attach reads two files whatever the store holds,
and a store of four thousand slots costs an attach the same as a store of one.
That is the per-slot read the decision's fifth item asks for,
and nothing in the attach's checks needs another slot's identity.

### The sweep, plan step 8

1.  The sweep enumerates the pinned parent's immediate children itself,
    which is its own walk and not the store's listing,
    and derives each slot's name.
2.  For each slot it opens the slot's record,
    which is one file,
    and where that name is absent it publishes by section 2.
    The publication's first step is the parent's compare:
    a sweep whose walked `.claude/worktrees` differs from `.parent` publishes nothing,
    and its attach refuses `ParentIdentity` before it compares the slot,
    which is the order `hold_clone_slot` already keeps (`clone_slot_walk.rs:338` to `:346`).
    So no record is written for a folder reached through a parent nobody compared,
    and that is the rule that keeps the adoption's listings free of records taken under the replacement.
3.  It attaches through step 4,
    which reads `.parent` and the slot's record.
4.  A slot that another sweep recorded first costs it one more read of that record,
    which is section 2's loser path.

The sweep lists no names and reads no record it did not reach,
so a slot that is recorded but whose clone is gone costs it nothing,
and a store entry for a slot the walk did not reach is outside the sweep's business:
the decision's lost single read is exactly that no check sees the set,
and no check in this step needs it.
The owner answered choice 9 "A:
no listing per sweep (Recommended)",
so a sweep run lists no name and a staging file a killed writer left is not the sweep's to remove:
the adoption removes it,
and plan step 18's `doctor` reports and removes it
(`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
"Who removes a killed writer's staging file").

### The adoption, and the listing it needs

The adoption is the one reader that needs the whole set,
because the decision table refuses a replacement that leaves a recorded slot unaccounted for
(`AdoptionRefusal::SlotNotListed`).

1.  It reads `.parent`,
    which is the policy the decision compares the replacement against (`ParentPolicy::from_recorded`).
2.  It lists the store folder once,
    with `readdir` on the held descriptor,
    keeps every name that is a slot record,
    refuses a foreign name by section 4,
    ignores the two staging forms,
    and removes those before it stages its own record:
    the mode's quiescence precondition is what makes that removal safe,
    because no writer is live to own those names.
    An entry that matches neither staging form is left in place and reported by `doctor`.
3.  It reads each listed slot's record,
    which is one file per name,
    and compares it against the live folder below the held parent,
    as `read_slot` already does,
    reaching `Agrees`,
    `Differs` or an unresolved binding.
4.  It decides,
    and before publishing it lists the folder a second time and refuses where the set of slot names changed:
    a name that appeared reaches the decision as `SlotNotListed`,
    and a name that vanished refuses as a record the store no longer holds,
    which nothing in this repository could have removed.
5.  It publishes `.parent` by section 3.

The second listing is a choice,
and it is proposed because it is what restores part of what one read gave:
the adoption verifies exactly the set it publishes beside.
Section 2's parent compare has already narrowed that set:
a sweep that has not compared the parent publishes nothing,
so during a recreated-parent adoption no new record lands through that parent at all.
What is left for the second listing is a sweep that passed its compare before the replacement stood and linked
its record after it,
which is stated under "The one property lost,
and what is not restored".

### The other readers

- The unmount of plan step 9 reopens the parent and the slot and compares mount id,
  device,
  inode and source,
  so it reads `.parent` and that slot's record,
  two files (`doc/planning/cli-git-zfs-worktree-design-proposal.md:674` for the recorded parent's reopen).
- `doctor`,
  plan step 18,
  reports "the recorded parent and slot identities with the failing field or component"
  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:1636`),
  so it is the one reader that reads every record,
  and it is read-only except for the staging names it removes.
  It lists the folder and reads each record,
  reports the entry count beside them,
  reports a record filed under a name its key does not match,
  reports a foreign entry,
  and reports a dot entry that matches neither of the two known staging forms.
  It removes the two known staging forms a killed writer left,
  with their count and age,
  which is the removal the project placed at plan step 18 while the sweep stays unlisted
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  "Who removes a killed writer's staging file"),
  and it raises nothing.
- The removal workflow and its cancel command read what plan step 4 and the unmount read:
  the recorded parent and,
  for the slot they act on,
  that slot's record.
  One search of the plan and the accepted design found no removal step that reads a record of its own,
  so this says what those steps read and does not claim a reader of its own.

### The one property lost, and what is not restored

The single file gave one read of the parent and every slot in one state.
This layout gives one read of one record,
and no operation can prove "these are all the records,
at one instant".

The guard that costs is worth naming plainly,
because the record-then-attach order of `main` is where it lived:
`main`'s attach read the whole store and refused on any line it could not read and on any key recorded twice
(`slot_identity_store.rs:142`,
`package/git-policy/cli/SPEC.md:5403` to `:5406`,
"because a skipped or doubled line would let a slot be compared against nothing or against two identities").
So a corrupt,
misfiled or foreign record for another slot stopped every attach.
This layout stops an attach only for its own slot's file,
and the attach cannot report what it never read.
What detects those states now:

- a partial or unparseable record refuses the read of that record;
- a record whose key is not its file's name refuses (section 4);
- a foreign name refuses the listing,
  which the adoption and `doctor` run;
- `doctor` reports a misfiled record,
  a foreign entry and the entry count.

That is adequate for a root-only writer,
and it is not the old guard:
a record for slot X misfiled under another name can only be put there by root,
the sweep does not list and so records X afresh,
and nothing else in this route reads a record for another slot.
The listing half of that guard can come back at one `readdir` per sweep run,
which is choice 9:
a foreign name and the entry count.
The content half,
a record of another slot that does not parse or whose key is not its name,
needs every record read on every sweep,
which this design does not propose and `doctor` does on demand.

What each reader could already do,
and what it still cannot:

- the attach never read the set,
  so it lost nothing it read for its own decision,
  and its two reads are its own slot's record,
  which cannot change,
  and `.parent`,
  which can,
  with a stale read refusing rather than permitting;
- the sweep does not ask,
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
  so a record published during an adoption is as true of its own folder afterwards as before it,
  and the replacement's own identity is what the adoption verified.
  The residual therefore costs a slot that the adoption's report does not name,
  not an identity the store gets wrong.
  Where the reviewer prefers the set to be frozen rather than reported,
  the answer is a lock over the folder,
  which rule `NLK` would require to be justified by a recorded failure,
  and none is recorded.

### A racing adoption, step by step

Two sweeps and one adoption meet on the example store,
this time with the replacement parent already standing,
which is the state the adoption exists for,
and the timings are the ones the hooks of section 7 force:

1.  The administrator has recreated `.claude/worktrees`,
    so the live parent is the replacement,
    device `57` and inode `67990`,
    and `.parent` still holds inode `66304`.
2.  Sweep one reaches `fix-flaky-test` and finds its record.
    It reads `.parent`,
    compares the parent its walk holds (inode `67990`) against the record (inode `66304`),
    and refuses `ParentIdentity`,
    naming both inodes and the device.
    That compare is `clone_slot_walk.rs:338` to `:340`,
    before the slot lookup at `:342` and its compare at `:346`,
    so the sweep never reaches the slot and attaches nothing.
3.  Sweep two reaches a new slot `rework-login`,
    reads no record for it,
    and publishes nothing either:
    section 2's first step reads `.parent`,
    compares,
    and refuses the same way.
    A folder reached through an uncompared parent gets no record.
4.  The adoption lists the folder once,
    reads `.parent`,
    `add-cache-layer` and `fix-flaky-test`,
    verifies both slots against the live folders,
    checks the replacement's identity,
    and takes its second listing before it publishes anything,
    which is section 5's order.
    That listing sees the same two slot names it saw at the first,
    because no sweep could publish while the parent differed.
    No set changed,
    so nothing refuses,
    and the second listing's job narrows to the one case below.
5.  It stages the new parent record and renames it over `.parent`,
    which now holds `57:67990:1000:0755:...`,
    and the two slot files keep their inodes and their bytes.
6.  The case it still covers:
    a sweep whose walk and `.parent` read both predate the recreation.
    Its compare passed against the old folder (inode `66304` on both sides),
    and it links its record after the replacement stands.
    That record is a first observation of a slot folder the replacement may no longer hold,
    so it is true of the folder it was taken from and unverified by this adoption;
    the second listing catches it if it lands before that listing,
    and the next adoption's reconciliation is what names it if it lands after.
7.  A sweep that holds a read of `.parent` from before the rename compares the replacement folder against
    inode `66304` and refuses,
    naming the inode and the device:
    a stale read is a refusal and never permission to attach.

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
  and the lock-hold thread-local hooks (`BETWEEN_THE_READ_AND_THE_APPEND`,
  `BEFORE_THE_STORE_OPENS`,
  `THE_LOCK_WAIT_ALLOWED`).
  The store takes no lock,
  and no constant replaces the wait because no write waits for another.
- The ignored measurement control `a_store_write_s_hold_is_measured_for_the_lock_wait`.
  Its subject is the lock's hold,
  so it goes with the lock;
  the publication's `fsync` cost is measured by the arm in section 7 instead.
- `STORE_MAXIMUM_BYTES`,
  replaced by `RECORD_MAXIMUM_BYTES` with the derived basis of section 4,
  and `check_the_store_fits_one_read`,
  replaced by a per-record length check.
- The `WouldNotFit` and `DuplicateKey` refusals.
  A record too wide for one read refuses as a per-file size refusal,
  and a key recorded twice cannot exist where the key is the file's name.
- The store's two source-guard entries,
  which name a lock that is gone and fail on a stale entry:
  `source_guard/service_lock_inventory_tests.rs:124` (its test at `:320` panics at `:325`,
  "is on the inventory and holds no shipped lock any more") and
  `source_guard/blocking_code_off_the_runtime_rules_tests.rs:408` (whose `:197` says the test fails on an
  entry that matches nothing).
  The inventory scans `slot_identity_*` files by prefix,
  so the new sibling files stay covered by it.

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
  so its parts split into sibling files in the same folder (`slot_identity_store_records.rs` keeps the
  grammar,
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
- `append_slot_identity_under_root_control` becomes `publish_slot_identity_under_root_control`,
  by section 2,
  and answers `SlotAlreadyRecorded` carrying the record that won on `EEXIST`,
  after it removes its own staging name and syncs the folder.
  It reads `.parent` and compares the walked parent before anything is created,
  answering `NoParentRecord` and `ParentIdentity`;
  `ParentIdentity` carries the same fields and sentence as the attach's own refusal.
- `record_parent_identity_under_root_control` splits into section 3's two writers.
- `slot_identity_store_refusals.rs`:
  `Missing`,
  `NoParentRecord`,
  `NoSlotRecord`,
  `SlotAlreadyRecorded`,
  `SlotName`,
  `NotUnderRootControl` and `Write` keep their sentences with the folder in place of the file,
  and four causes are new:
  a record over the per-file bound,
  a record whose key is not its file's name,
  a foreign entry in the folder,
  and a walked parent that differs from `.parent` (`ParentIdentity`).
  `Missing`'s sentence names the setup run that creates the folder,
  and a new `ParentAlreadyRecorded` answers a second setup run.
- `adopt_worktree_parent_mode.rs`:
  `recorded_slots` lists the folder and reads one file per name,
  and the mode lists twice by section 5.
- `parent_adoption_decision.rs`:
  `AdoptionFacts.store` becomes the slot names the first listing produced,
  and its doc's promise survives the change:
  "A caller cannot hand in a list of its own" (`:63` to `:66`) stays true,
  because the names still come from the folder and never from a caller's account of it.
  The facts carry the first listing's states,
  and the second listing's names are compared against them before the publish,
  so the table keeps its order and its refusals,
  and `SlotNotListed` now also answers a slot that appeared between the two listings.
- The attach itself,
  `clone_slot_walk.rs`:
  `hold_clone_slot` reads the folder store at `:333` and compares the parent at `:338` before the slot at
  `:342`,
  which stays the order,
  and `hold_clone_slot_under_root_control` at `:384` opens the folder under root control once.
  `HoldCloneSlotRequest.store` stays the store's path,
  now the folder's,
  so the controls that build one keep their shape.
  Named with it because they drive the store through that request:
  `clone_slot_walk_tests.rs`,
  `attach_post_check_tests.rs`,
  `parent_adoption_decision_tests.rs`'s `recording_slots`,
  and `mod.rs`'s re-exports and top doc at `:36` to `:43`,
  which is the folder's file list and gains the new sibling files.
- The publication primitives and their new home:
  `open_unnamed`,
  `link_unnamed` and `move_temporary_into_place` are private `fn`s of
  `worktree_copying/worktree_copy_publish_file.rs` (`:218`,
  `:282`,
  `:333`),
  `link_unnamed` takes a destination path rather than a held folder,
  and `rename_that_never_replaces::rename_noreplace` and `private_storage::staging_beside` are path-based.
  `worktree_clone_mount/uses/` links only `foundation` and `subvolume`,
  so section 2's `linkat(folder, staged, folder, name, 0)` and section 3's rename need descriptor-relative
  forms of these primitives in `foundation/`,
  with their own `uses/` folder and their tests moved beside them;
  the alternative is a second `uses/` link from `worktree_clone_mount` to `worktree_copying`,
  which the layering rules would then have to allow and this design does not propose.
- The specification,
  `package/git-policy/cli/SPEC.md:5348` to `:5509`:
  "The recorded identities store" is rewritten for the folder,
  the per-file grammar and version 2,
  the publication of section 2,
  the two parent writers of section 3,
  the per-file bound of section 4,
  and the readers of section 5.
  The grammar block at `:5359` to `:5364` becomes three file examples,
  the bound's sentences at `:5407` to `:5421` become the per-file bound,
  the lock's paragraphs at `:5451` to `:5474` and the "That lock is the one place this route is not lock-free"
  paragraph at `:5475` to `:5489` are deleted,
  `:5490` to `:5499` keeps its split of the two parent writers with the two new writers' names,
  and `:5500` to `:5508` keeps its conclusion with the folder's own reason (a second checkout's `.parent`
  cannot be published beside the first).
- The specification's three passages outside `:5348` to `:5509`:
  "The refusals" at `:5610`,
  whose store,
  store-control and append bullets (`:5629` to `:5638`) name a line number,
  a torn-record repair and the folder's free space,
  "The administrator's adoption" at `:5644`,
  whose surviving-slots sentence at `:5658` is the folder's own `slot_names`,
  and the guest driver's doc comment at
  `package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts:5`,
  which names the store path it writes.
- Plan step 4's `:597` to `:600`,
  which names the store as one path the attach reads,
  gains the folder and the two files the attach reads;
  step 8's first-observation sentence gains "compares `.parent` before it publishes" and "publishes it by one
  link that replaces nothing";
  step 19's setup run,
  which does not exist yet,
  is where the folder's creation at mode `0755` and `.parent`'s first publication belong.
- The clone-mount sweep,
  `package/git-policy/cli/src/native/worktree_clone_mount_handler/attaching_sweep_mode.rs`,
  which lists nothing and removes nothing:
  its stub refuses its step without opening the store,
  and the removal of the two known staging forms left the sweep for plan step 18's `doctor`.
- `doctor`'s store section,
  which plan step 18 builds:
  it reports the entry count,
  a misfiled record,
  a foreign entry,
  the two known staging forms with their count and age,
  and any dot entry matching neither form,
  and it removes the two known staging forms
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  "Who removes a killed writer's staging file").
- `package/git-policy/cli/e2e/README.md:838`,
  which says the guest adoption arm writes the store file,
  and `package/git-policy/cli/src/native/binary/binary_zfs_adoption_guest_tests.rs`,
  whose `write_store` writes the single file with `std::fs::write`.
- The dated records keep their text and are not rewritten (rule `WR6`):
  `package/git-policy/cli/doc/step-5-second-executable-2026-11-20.md:93` keeps its `Missing { path: ... }`
  line as the round that saw it,
  and `doc/planning/cli-git-zfs-staging-under-cache-design.md:208` keeps "records live in `/var/lib/...`",
  which stays true of the folder.

### The documents

- `package/git-policy/cli/doc/slot-identity-store-lock-hold.md` loses its subject.
  It is rewritten,
  not deleted,
  because two durable facts in it have no other home:
  the kernel's `fsync` of a record reaching the pool's device (60 samples,
  a median of 8 sectors and two ZIL commits,
  against three controls that moved the device not once),
  and `probe-reaches-storage.py`,
  which is kept so that question can be asked again.
  Its name would then lie about its content (rule `VGW`),
  so it becomes `package/git-policy/cli/doc/slot-identity-store-write-cost.md` with its probe folder beside
  it,
  the lock sections and every arithmetic line about the wait deleted,
  and the publication's own measured cost added where section 7's arm measures it.
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
  which becomes section 2's publish-once arm and its loser;
- the adoption replaces only the parent record,
  which becomes section 3's arm that also proves the slot files' inodes and bytes unchanged;
- the append holds one lock and the adoption waits for it,
  the append opens the store after the lock,
  an append that cannot take the lock,
  and a link at the lock's name:
  deleted with the lock;
- the store larger than the read accepts,
  an append that would outgrow the store,
  an adoption that would outgrow the store,
  and an adoption that lands exactly on the bound:
  replaced by the per-file bound's arms of section 4;
- the control's own root terms and the store this account could rewrite:
  kept,
  with the folder and a record file in place of the file and the lock file;
- the specification's store example,
  which becomes the example of section 1;
- `adopt_worktree_parent_mode_tests.rs`'s store fixtures,
  `attach_arm_tests.rs`'s store arms,
  and `binary_zfs_adoption_guest_tests.rs`'s `store_bytes` and `write_store`.
- Two arms `main` tests and this design keeps are moved,
  and section 7 says where each lands:
  the `NoParentRecord` arm of `a_slot_is_recorded_once_and_never_refreshed` (`slot_identity_store.rs:332` to
  `:334`,
  asserted at `slot_identity_store_tests.rs:199` to `:201`) becomes the publisher's `NoParentRecord` control
  (section 7's control 12),
  and that test's name loop (`""`,
  `one/two`,
  `.`,
  `..`,
  `.hidden`,
  `with space`,
  `with<TAB>tab`,
  `slot_identity_store_tests.rs:194` to `:197`) becomes the publisher's `SlotName` control (section 7's
  control 13).

### The fixture seam the controls run on

`main` keeps two ways into the same grammar:
a root-free `read`,
`append_slot_identity` and `record_parent_identity`,
and the `_under_root_control` forms the route uses,
and its controls drive the root-free pair on fixture files they own (`slot_identity_store_tests.rs`'s `store`
helper,
`parent_adoption_decision_tests.rs`'s `recording_slots`).
The split survives,
and section 7 depends on it:
the grammar reader still opens a folder a test owns and reads one record,
while the `_under_root_control` forms are the only ones that check the owner and the mode.
Without that split,
the root-free controls of section 7 could not run on a fixture folder the account owns,
because the route's open would refuse a folder that is not root's;
the alternative is hosting those controls in the user namespace that `root_namespace_test_support.rs` already
provides,
which costs a re-execution per control and is what the two root-control arms use.

## 7. The tests a build of this design must have

Each control is shown to fail before it passes (rule `GFP`),
and each refusal arm has its accepted fixture as well as its rejected one (rule `THR`).

### Controls that need no root

1.  `a_record_is_published_whole_and_a_reader_never_sees_it_partly_written`:
    a hook between the staging file's `fsync` and its link,
    asserting the record's name is absent there,
    the staging entry is there,
    and the record is whole after the link and the unlink.
2.  `two_recorders_of_one_slot_publish_once_and_the_second_reads_the_first`:
    the race section 2 orders,
    forced by a hook that lets a second writer publish while the first is held before its link,
    asserting one record,
    the first writer's bytes,
    the loser's own staging name removed,
    and the loser's `SlotAlreadyRecorded` carrying the winner's identity.
3.  `a_publication_that_fails_leaves_no_record_and_no_staging_entry`:
    a failure injected after the bytes are durable,
    on the link arm and on the `RENAME_NOREPLACE` arm.
4.  `a_killed_writer_leaves_one_staging_entry_and_no_record`:
    the writer is killed between its bytes and its link,
    and the arm asserts the dot entry,
    the absent record,
    and nothing else;
    the same file also covers the kill between the link and the unlink,
    which leaves the record whole beside a dot entry naming the same file.
5.  `the_adoption_removes_the_staging_entries_a_killed_writer_left`:
    both known forms planted in a folder the account owns,
    an unknown dot entry planted beside them,
    asserting the two known forms gone,
    the record untouched,
    and the unknown one left for `doctor`.
6.  `the_publication_falls_back_where_the_folder_refuses_a_link`:
    the link is refused by an injected error,
    and the record is published by `RENAME_NOREPLACE` with the staging name gone.
7.  `the_publication_refuses_where_the_folder_refuses_both_a_link_and_rename_that_never_replaces`:
    both calls refused,
    asserting one `Write` refusal naming the folder and both calls,
    and no record.
8.  `a_parent_record_is_replaced_by_one_rename_and_a_reader_sees_one_whole_record`:
    section 3's race,
    forced by a hook between the staged parent's `fsync` and the rename,
    asserting a reader in the window reads the old record whole and `.parent` is never absent.
9.  `the_adoption_writes_no_slot_file`:
    inodes,
    modification times and bytes of every slot file unchanged across an adoption.
10. `a_slot_file_that_appears_between_the_two_listings_refuses_the_adoption`:
    a hook between the listings,
    asserting `SlotNotListed` names the slot and `.parent` is unchanged.
11. `a_publisher_whose_walked_parent_differs_from_the_parent_record_publishes_nothing`:
    the live folder's identity differs from `.parent`,
    asserting `ParentIdentity` with every field that moved,
    no record for a new slot,
    and no staging entry.
12. `a_publisher_in_a_store_with_no_parent_record_publishes_nothing`:
    `NoParentRecord`,
    which is the arm `main` asserts at `slot_identity_store_tests.rs:199` to `:201`.
13. `a_publisher_refuses_a_name_that_is_no_plain_slot_name`:
    the loop `main` drives at `slot_identity_store_tests.rs:194` to `:197` (`""`,
    `one/two`,
    `.`,
    `..`,
    `.hidden`,
    `with space`,
    `with<TAB>tab`),
    each `SlotName`,
    and nothing created.
14. `a_record_larger_than_the_bound_is_refused_by_name_and_size`:
    a fixture at the bound and one byte over.
15. `a_record_whose_key_is_not_its_file_name_is_refused`:
    a good record copied,
    and a hard link of it,
    each refused.
16. `a_foreign_entry_refuses_the_listing_and_every_staging_form_does_not`:
    a name with a space,
    both known staging forms,
    an unknown dot entry,
    and the folder's own listing arm,
    with `doctor` reporting the unknown one and the entry count.
17. `a_record_this_account_could_rewrite_is_refused`:
    the owner and mode arms,
    through the user-namespace helper the current control already uses.
18. `an_absent_store_folder_asks_for_the_setup_run`,
    and `a_link_at_a_record_name_is_never_read_through`.
19. `a_second_setup_run_compares_the_live_parent_with_the_parent_record`:
    equality is success,
    and a difference refuses `ParentAlreadyRecorded`.
    The plan's step 19 boot unit runs on every boot,
    so a second run must not be an error.
20. `doctor_reports_a_misfiled_record_and_a_foreign_entry`:
    the two states the whole-store guard used to refuse,
    each reported by name,
    and the store left as it was.
    This control is owed to `doctor`'s section,
    which plan step 18 builds;
    the listing refusal and the removal it depends on are controls 16 and 5 here,
    and the report itself has no code yet.
21. `the_specification_s_store_example_is_a_store_this_reader_reads`,
    over the three files of section 1,
    including the parent's access ACL that the specification's own control asserts.

The sweep's own run stands beside these:
`a_sweep_run_lists_nothing` holds a hook at the store's open and another at the store's listing,
and asserts that a run reaches neither and still answers its step's refusal,
which is choice 9's answer and the reason the removal waits for plan step 18.

If the project accepts section 2's option B instead,
one more control is owed:
both link spellings answering `ENOENT` or `EPERM` takes the staged path or a `Write` refusal naming both
calls,
which is the case section 2 states.

### Guest arms, and why each needs root or a real pool

22. `the_store_folder_under_var_lib_publishes_a_record_as_root_and_reads_it_back`:
    `/var/lib` is root's,
    so this runs in the gate image or the throwaway guest,
    and it is the arm that settles the design's one assumption about the store's own dataset:
    `linkat`,
    `unlinkat`,
    `renameat2` and the folder's `fsync` on `zpcachyos/ROOT/cos/root`,
    with the `O_TMPFILE` spelling only if option B is accepted.
23. `a_killed_writer_between_its_bytes_and_its_link_leaves_no_record`:
    root,
    because the store is under `/var/lib`;
    the writer is killed after its bytes are durable and before its link,
    and the arm asserts no record,
    one staging entry,
    and the adoption removing it.
24. The publication's `fsync` arm:
    `probe-reaches-storage.py` re-run against a per-slot publication on a folder of the store's own pool,
    which needs the real pool and its counters.
    It is what replaces the lock-hold record's subject with the publication's.
25. The owed boot arm stays owed and unchanged:
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
that every refusal of section 4 is reached and says what repairs it,
and that the store reads under root control on the dataset it lives on.
It does not show that two sweeps meeting is rare,
that a machine crash between the folder's `fsync` and a reboot leaves the record,
or that a filesystem the gate never mounts answers the fallbacks the way the arm forces them to.

## Limitations

- The primitives were measured on this host's `zpcachyos/ROOT/cos/home` dataset as the account,
  not on `zpcachyos/ROOT/cos/root` as root.
  Both are ZFS datasets of one pool mounted `xattr,posixacl`,
  and the arm of section 7 is what closes the gap.
- The attribute ceiling was measured twice:
  a `user` attribute of 65,536 bytes accepted and 65,537 refused `E2BIG`,
  which is `XATTR_SIZE_MAX` exactly,
  and a `system.posix_acl_access` value of 65,532 accepted and 65,540 refused.
  The widest record is arithmetic on that ceiling and not a measured file,
  and `read_acl` refuses an ACL above `MAXIMUM_ACL_BYTES` at capture,
  so no folder this route records can reach the arithmetic's token.
- ZFS's own folder `fsync`,
  and its behaviour for a machine crash,
  were not measured;
  the design assumes POSIX durability for the two `fsync` calls it orders.
  Its support for an unnamed file matters only if the project accepts section 2's option B.
- The number of sweeps that meet on one slot,
  and the number of adoptions that meet on one parent record,
  are unmeasured;
  both races are handled by the kernel's own exclusivity and by one refusal,
  and neither is sized.
- The per-file bound is 262,538 bytes,
  derived from `MAXIMUM_ACL_BYTES` and the name limit rather than chosen,
  so a bound that is too small to refuse or too large to read is not a risk;
  a record larger than it can only come from a writer outside this repository.
- The store's folder has no total bound,
  which the decision accepted.
  What sizes a listing is the account's clone count,
  not a foreign writer:
  root adds one record per clone the account can make,
  and the delegated `zfs` grants that limit that count are outside the store.
- The adoption's second window stays open:
  a slot published between the second listing and the rename is not verified by that adoption,
  and the design's argument that this is safe rests on a slot record holding no field of the parent's
  identity.
  Section 2's parent compare narrows that window to a sweep that passed its compare before the replacement
  stood.
- The parent compare before a publication is check-then-act,
  so a `.parent` replacement that lands between that read and the link is not detected by the publisher;
  what notices it is the adoption's second listing and the next adoption's reconciliation.
- Two adoptions that run at once are left with whichever renamed last,
  exactly as `main` leaves them,
  which is parity and not a loss.
  The mode's quiescence precondition forbids them and no program checks it,
  and nothing in the store detects one.
- This design does not decide whether a slot's record may be removed when its clone is destroyed and its
  registration settled,
  which today nothing does.
  The layout makes that one `unlinkat`,
  and the decision is the owner's,
  because it is what would let a reused slot name be recorded anew;
  a build that allowed it would also make a held read of a record able to go stale (section 5).
- A dot entry that matches neither known staging form is reported by `doctor` and removed by nobody,
  which means a writer this design has not accounted for stays visible until an administrator acts.
- A known staging form a killed writer left stands until the next adoption or until plan step 18's `doctor` runs,
  because no sweep run lists the folder to find it;
  no reader reads a staging name,
  so it changes no read,
  no compare and no mount.
  Only the adoption removes it before that step
  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
  "Who removes a killed writer's staging file").
- No measurement here is a release-profile build,
  and none was taken under a loaded pool.
- The probe those primitive numbers came from lives in this round's scratch folder and not in the repository,
  so they are not reproducible from a checkout until the arm of section 7 lands beside the code.
- The design has had one gate review,
  which found what [This round](#this-round) lists and fixed,
  and no independent cross-vendor review since;
  it has no implementation,
  and its `SPEC.md`,
  plan and code edits are unwritten.
  Only the owner declares it approved or final.

## The choices this design leaves open, with their rankings

Each choice is argued in its own section,
and none is decided until the owner accepts it.
The first option of each is the one this design proposes,
and the last two are flagged because they need a decision this design cannot make.

1.  Names:
    flat with `.parent` (A),
    two levels with `slots/` (B),
    or a suffix per kind (C).
    Ranking A > B > C.
    A is cheapest because one held descriptor and one reserved name buy every legal slot name;
    B buys a visible parent name with a second descriptor and a second control check;
    C cannot name a slot whose name is near `NAME_MAX`.
2.  The slot record's publication:
    `O_EXCL` under a staging dot name,
    one link,
    then the unlink (A,
    the decision's own words),
    a file with no name and one link (B,
    flagged:
    no file is created with `O_EXCL` on that path),
    `O_EXCL` at the record's name with a completeness marker (C),
    or `O_EXCL` at the record's name with a reader that retries (D).
    Ranking A > B > C > D.
    A keeps the accepted words,
    needs one publication variant on every filesystem,
    and asks the store's dataset only for a hard link and a folder `fsync`;
    B leaves no litter after a kill but departs from the words,
    adds two link spellings with a capability and kernel floor,
    and rests on unnamed-file support the probe did not measure on the store's dataset;
    C and D both expose a partial record at the record's name,
    and D cannot tell a killed writer's leftover from a live write.
3.  The parent record's replacement:
    one rename (A),
    re-read `.parent` and compare it just before the rename (B),
    `RENAME_EXCHANGE` compared against what it displaced (C),
    `RENAME_EXCHANGE` only to make "must exist" atomic (D),
    or generations (E).
    Ranking A > B > C > D > E.
    Two concurrent adoptions are parity with `main`,
    which also leaves the later writer's record,
    so A's single step needs no detection;
    B narrows that race without ever exposing an unjudged record;
    C and D publish before they judge,
    and D's second rename buys only what `.parent` never being removed already gives;
    E moves a generation ladder into every parent read and needs a removal step.
4.  What a second recorder of one slot does:
    read the record that won and carry on with it (A),
    or refuse and ask for a fresh run (B).
    Ranking A > B.
    A is what the store's own exclusivity already gives:
    the winner's record is what every attach compares against,
    so carrying on does a single sweep's work and can loosen nothing;
    B costs a whole sweep,
    and its only gain would be a report of a race the design treats as expected.
5.  The per-file bound:
    the exact derived bound,
    262,538 bytes (A),
    one mebibyte (B),
    512 KiB (C),
    or 300 KiB (D).
    Ranking A > B > C > D.
    A refuses every file no folder this kernel describes could produce and is derived,
    not chosen;
    B is the number the whole store already carried,
    which is incumbency and not a workload argument,
    though it costs nothing since it is 3.99 of the widest record;
    C and D are neither derived nor the old number,
    and both are larger than the widest record by an amount nothing chose.
6.  Who may read the store:
    root-owned `0755` with `0644` records (A),
    or `0700` with `0600` records (B).
    Ranking A > B.
    A is the receipts' shape and keeps an unprivileged report of the recorded identities possible;
    B would make `doctor` need root for a read it could not explain.
7.  The lock-hold record,
    settled by the repository's rules rather than offered:
    it is rewritten and renamed as the publication's write cost.
    The decision says the document is "rewritten or removed",
    rule `DL4` says a document goes only when its durable facts have a new home,
    and rule `VGW` says a name must not lie about its content,
    so `slot-identity-store-write-cost.md` is the recorded action and no alternative is proposed.
8.  The adoption's second listing:
    taken,
    and a changed set refused (A),
    or not taken,
    and the residual widened to the first listing (B).
    Ranking A > B.
    One `readdir` narrows the window the decision paid for and needs no lock,
    and section 2's parent compare has already removed every record taken through an unadopted parent from
    that window.
9.  Whether a sweep lists the store:
    no listing,
    one record read per slot reached (A),
    or one `readdir` per sweep run that refuses a foreign name before any attach (B).
    Ranking A > B.
    B restores the name half of the whole-store guard `main` had,
    at one listing per sweep,
    and does not restore the half that reads another slot's record:
    a misfiled record needs a content read,
    which `doctor` makes and a sweep would pay on every run.
    Only root writes the store,
    and the adoption and `doctor` already run that listing,
    so the sweep's copy buys detection of a state a root-only writer rarely makes.
    The project answered A on 2026-11-21,
    so a sweep run lists nothing and the adoption is the only remover of the two known staging forms
    until plan step 18 lands,
    and that step's `doctor` reports and removes them;
    a leftover affects no reader,
    because no reader reads a staging name
    (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
    "Who removes a killed writer's staging file").

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
- The store's write-cost record:
  `package/git-policy/cli/doc/slot-identity-store-write-cost.md`,
  which was `slot-identity-store-lock-hold.md` until the build renamed it on 2026-11-21,
  when the lock it measured went away.
- The publication precedent:
  `package/git-policy/cli/src/native/worktree_copying/worktree_copy_publish_file.rs` and
  `package/git-policy/cli/src/native/foundation/private_storage.rs`.
- The probe:
  `probe_publish_primitives.py` and its printed output in
  `/home/user/temp/agent/zfs-slot-identity-store-per-slot-files-design-2026-11-21/scratch/keep/`,
  which is this round's scratch and not a repository file.
- The gate review this round answers,
  and its own probe checks:
  `/home/user/temp/agent/gate-review-zfs-slot-identity-store-per-slot-files-design-2026-11-21/keep/review.md`,
  `probe-outputs.txt` and `probe_checks_of_the_reviewer.py` beside it,
  also outside the repository.

## This round

This is the gate-review round of 2026-11-21,
and it changed the design in the five ways below.
Nothing here is approved or final,
and only the owner accepts any of it.

- The evidence was replaced (finding H2).
  The previous round's probe defined `O_TMPFILE` as `0o20000000`,
  which is the flag without `O_DIRECTORY` and is refused `EINVAL`,
  tested `os.fsync`'s `None` return with `== 0` so its success line always printed "failed",
  and had no arm for the name ceiling,
  the attribute ceiling or the device numbers it claimed.
  The probe in this round's `keep/` folder fixes all three,
  runs each `linkat` spelling first on a file that still has no name,
  adds the arms this design needs for its own first option,
  and its printed lines are quoted whole under "Measured",
  each fact attributed to its line.
  It is Python,
  following the repository's one kept probe,
  `package/git-policy/cli/doc/slot-identity-store-write-cost/probe-reaches-storage.py`
  (then named `slot-identity-store-lock-hold/probe-reaches-storage.py`,
  and renamed with its record on 2026-11-21),
  and the reason it is not the `.ts` that rule `NMJ` prefers is mechanical:
  `node` has no binding for `linkat(AT_EMPTY_PATH)`,
  `renameat2`'s flags or `setxattr`,
  which are exactly the calls this probe must make,
  so a `.ts` probe could only wrap a Python one.
  The design's own arms live in Rust (section 7),
  where those calls are already in `libc`.
- The publication's ranking was reversed to keep `O_EXCL` (finding M1).
  The earlier draft ranked an `O_TMPFILE` route first,
  which does not make the slot's file with `O_EXCL` as the decision's words say.
  The first option is now `O_EXCL` under a staging dot name with one link,
  the departure is option B and is flagged for explicit acceptance,
  and the option lists gained the single-path alternative the review named.
- A publication now compares the parent before it publishes (finding H1).
  The earlier draft let a sweep publish first and attach second,
  which would record an identity under a parent nobody had compared.
  Section 2 makes the compare the publisher's first step,
  the sweep's order follows it,
  and the racing example was rewritten around the state the adoption exists for,
  a live parent that differs from `.parent`.
  The example the review broke was not a race that can happen,
  because `hold_clone_slot` refuses `ParentIdentity` at `clone_slot_walk.rs:338` to `:340`,
  before any slot compare.
- Staging entries got one owner (finding M2),
  both of their name forms are named in section 1,
  the adoption removes the known forms before it stages its own,
  `doctor` reports a dotted entry that matches neither,
  and section 7's control 5 is the arm that proves the removal.
- The two refusals `main` tests and the earlier draft dropped are back (finding M3):
  the publisher refuses `NoParentRecord` and `SlotName` before anything is created,
  and section 6 says which control each of `main`'s arms moves to.

What the other findings changed:

- Section 6 was completed against the code (M4):
  the attach and its controls,
  the two source-guard entries,
  the specification passages outside the store's own range,
  the guest driver's doc comment,
  the fixture seam,
  and the primitives' new home in `foundation/`.
- What the attach loses was said plainly (M5):
  the whole-store read refused any bad line or doubled key anywhere,
  and what detects those states now is named,
  with choice 9 offering the old guard back at one `readdir` per sweep.
- Choice 3's reason leads with parity to `main` rather than a reading of quiescence,
  and choices 4 to 9 carry option lists with pros and cons (M6).
  Choice 7 is recorded as the repository's rules settle it rather than offered.
- The example store now carries the specification's own ACL bytes,
  44 of them,
  on the parent's record (L1),
  which is where the specification puts them and what its control asserts.
- Section 2 states the kernel boundary for the empty-name link spelling and what a build does when both
  spellings refuse (L2).
- Three citations were corrected (L3):
  the device paragraph is `:5386` to `:5401`,
  the "runaway or replaced file" sentence is `slot_identity_store.rs:52` and not `SPEC.md:5407`,
  and the no-refresh sentence is `proposal:612` to `:613`.
- The read open now carries `O_NONBLOCK`,
  and the folder that holds the store is accounted for (L4).
- The bound is derived from `MAXIMUM_ACL_BYTES` and the name limit and is 262,538 bytes,
  the exact bound is now an option,
  and the ranking is argued from the workload (L5);
  the probe's `user` attribute arm is what places the attribute ceiling at 65,536.
- The decision's six items are mapped to sections instead of the wrong claim that the sections are those items
  in order (L6).
- Option C's rejection rests on the grammar of names that can arrive,
  not only on `NAME_MAX` (L7).
- The removal readers are described as what plan step 4 and the unmount read,
  and the recheck rule's dependence on the open removal question is stated (L8).
- The store's growth is attributed to the account's clone count,
  and `doctor` reports the entry count (L9).
- The prose rules are met (L10):
  the status is a heading and a plain sentence,
  option labels are headings,
  pros and cons are not bold labels,
  and numbered markers are padded to four columns.
- The loser syncs the folder before it reads (L11),
  what a losing sweep receives is named,
  the per-slot claim that makes two recorders rare is cited,
  and a second setup run treats an equal `.parent` as success.

No finding is dismissed.
