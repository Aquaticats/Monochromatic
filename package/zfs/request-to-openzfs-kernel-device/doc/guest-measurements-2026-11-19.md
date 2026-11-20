# Guest measurements: the live binding against OpenZFS 2.4.4

**Non-final,
unapproved.**
This records what an agent ran and what the kernel
answered;
it is not the project's acceptance of step 2,
and the owner alone
decides that.

It is the measurement record the implementation plan's step 2 asks for
([`cli-git-zfs-worktree-implementation-plan.md`](../../../../doc/planning/cli-git-zfs-worktree-implementation-plan.md),
"Step 2:
Live requests,
the layout comparison and the guest fixture",
and
"Where the tests run"):
items 12,
13,
14,
17 and 18 of
[`cli-git-zfs-measurements-owed-before-acceptance.md`](../../../../doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md)
re-run through this crate,
item 11's promote controls through its promote
primitive,
and item 5's clone under an unloaded key.
The raw logs live outside
the repository,
in
`/home/user/temp/agent/zfs-step-2-short-review-findings-fixed-2026-11-19/keep/`,
as the plan requires;
this file cites them by name.

This record replaces the one this file held before the short review of
2026-11-19:
its run read the module file's DWARF and left the promoted batches
unasserted,
and this one reads the running module's own BTF and asserts both.

## Where and how

The host was read only:
`uname --kernel-release` printed `7.3.5-1-cachyos` and
`/sys/module/zfs/version` read `2.4.4-1`.
No host pool,
dataset,
mount or
`/dev/zfs` was touched.

Every arm ran in a throwaway `mvm` guest (`libvirt` backend) created from
`mvm`'s Ubuntu 24.04 image,
prepared by
[`e2e/run-fixture.ts`](../e2e/run-fixture.ts):

- kernel:
  `6.8.0-147-generic` (`/proc/sys/kernel/osrelease`).
  `prepare` refuses
  any other release,
  so the plan's pinned kernel is a checked fact of the run,
  not a
  hope about the image.
- OpenZFS userspace and module:
  `2.4.4`,
  built from `zfs-2.4.4.tar.gz` against
  the guest's own kernel headers;
  the tarball's sha256 was checked against the digest the
  recipe pins,
  which is the one the project's signed `zfs-2.4.4.sha256.asc`
  names.
  `zfs version` printed `zfs-2.4.4-1` and `zfs-kmod-2.4.4-1`,
  and
  `/sys/module/zfs/version` printed `2.4.4-1`.
- module source version:
  `B2C6D65530FF8F5A7105ACC` (`/sys/module/zfs/srcversion`).
- module BTF:
  `/sys/kernel/btf/zfs` exists for that module,
  which is the file the
  layout comparison reads (see the next section).
- Git:
  `git version 2.43.0` (Ubuntu's package;
  the design's `2.56.1` runs are
  step 23's,
  and step 2's items do not read Git).
- `pahole` `v1.25` (`dwarves`),
  `strace -- version 6.8`.
- Rust `1.100.0` (`0ef1bc386`) and `cargo-nextest 0.9.152`,
  installed in the
  guest from the pinned rustup release whose installer digest the recipe
  checks;
  the toolchain it installs is current stable,
  which `versions.log`
  records.

The crate was **built inside the guest** and never on the host:
the host's glibc
is newer than Ubuntu 24.04's,
so a host-built test binary would not start there.
The recipe says so in its own header and in
[`e2e/README.md`](../e2e/README.md).

Each arm built its own 1 GiB file-backed pool with `encryption=aes-256-gcm`,
`keyformat=raw` and a guest-local key file,
`recordsize=32K`,
`compression=zstd-3`,
a mounted `home`,
a mounted `checkout`,
and an unmounted
clone parent with `mountpoint=legacy` and `canmount=off`,
which is the earlier
record's fixture.
Each pool was destroyed when its test ended,
and its image and
key folder with it.

Logs:
`recorded-run-2026-11-19-review-fixes/` is one end-to-end run of the recipe's
steps in order
(`create`,
`prepare`,
`run`,
`record`,
`destroy`):
it created the guest,
prepared it,
ran both suites,
recorded the versions and the teardown,
and destroyed the guest.
It holds `root-suite.log`
(194 tests,
all passing,
with the `MEASUREMENT` block quoted here and the account arms
filtered out),
`account-suite.log` (8 tests),
`versions.log`,
`teardown.log`,
`guest-prepare.log`,
`guest-build.log`,
`guest-root-suite.log`,
`guest-account-suite.log`,
`mvm-create.log`,
`mvm-destroy.log` and `mvm-list-after-destroy.log`.
`red-half-2026-11-19-review-fixes/` holds the two deliberately broken copies
whose live assertions failed (the "Red half" section).

## The layout comparison against the module's BTF

The design's inspection is
`pahole --btf_base /sys/kernel/btf/vmlinux -C zfs_cmd /sys/kernel/btf/zfs`
(["ABI gate and tests"][abi]).
An out-of-tree module gets that BTF file because 6.8's
`scripts/Makefile.modfinal` runs its `cmd_btf_ko` command whenever
`CONFIG_DEBUG_INFO_BTF_MODULES` is set (Ubuntu's is),
and that command's own
guard is a `vmlinux` file in the kernel build directory:

```text
ifdef CONFIG_DEBUG_INFO_BTF_MODULES
	$(if $(wildcard $(1)), ...
	@echo '  BTF [M] $@'; \
	... pahole -J --btf_base vmlinux $@ ...
```

`KBUILD_BUILTIN` decides when the command is re-triggered,
not whether it
runs.
Ubuntu's headers ship no `vmlinux`,
which is the whole reason a
tarball-built module used to carry no BTF,
so `prepare` copies the kernel's own
`/sys/kernel/btf/vmlinux` into `/lib/modules/$(uname -r)/build/vmlinux` before
the build,
and asserts `/sys/kernel/btf/zfs` exists after the module is loaded.

The test reads that file with the design's command and compares the same
six structures field by field:
`zfs_cmd`,
`dmu_objset_stats`,
`zinject_record`,
`zfs_share`,
`zfs_stat` and `drr_begin`,
every member's name,
offset and size against
`offset_of!` and `size_of` of the Rust mirror,
each structure's size,
and the
exact member set;
every request number is then compared with `include/sys/fs/zfs.h`
of the release tree.
No DWARF read remains:
the BTF file belongs to the module the
kernel loaded,
so no comparison of a file's source version against
`/sys/module/zfs/srcversion` is needed to know which module was measured.

Tests:
`kernel_device::live_btf_tests::the_running_modules_type_information_matches_the_rust_mirror`
and `...::the_release_header_assigns_every_request_the_number_this_crate_sends`.

Recorded
(`recorded-run-2026-11-19-review-fixes/root-suite.log`,
the layout line):

```text
layout zfs_cmd bytes=13744 dmu_objset_stats=288 zinject_record=368 drr_begin=304 source=/sys/kernel/btf/zfs
```

This is the same layout the vet read from the host's module BTF on 2026-11-18,
and the test refuses a release that moved a field.
The breaking control is in
`red-half-2026-11-19-review-fixes/`:
a copy whose expected
`zc_defer_destroy` offset is eight bytes too high fails this test on the live
assertion.

## Item 5: a clone under an unloaded key

The key was unloaded after unmounting the child dataset and then the parent
(`zfs unmount` has no recursive flag on this release).
With the key unloaded,
`ZFS_IOC_CLONE` of the encrypted snapshot returned 0 and the clone's
`OBJSET_STATS` reply reported:

```text
MEASUREMENT key-unloaded clone keystatus=1 origin=.../promotion@origin
MEASUREMENT key-loaded keystatus=2 (1 is unavailable)
MEASUREMENT encryption=8 keyformat=1
```

`keystatus` is `ZFS_KEYSTATUS_UNAVAILABLE` (1) before the load and
`ZFS_KEYSTATUS_AVAILABLE` (2) after.
The clone's origin is the snapshot it was
made from,
so identity is readable while the key is not.
The control is the
same clone read again after `zfs load-key`.
`encryption` and `keyformat` are the
kernel's enumerated values (8 for `aes-256-gcm`,
1 for `raw`).
Test:
`kernel_device::live_write_tests::a_clone_under_an_unloaded_key_succeeds_and_reports_the_key_state`.
What this does **not** show:
mount behaviour,
which item 5 measured through a
descriptor and which belongs to step 4.

## Item 11 and 13: promote, the moved history and the bookmark

The checkout carried `@earliest`,
`@origin` and `@checkout-later`,
a bookmark
`#from-earliest` was made from `@earliest`,
the clone `promotion` was made from
`checkout@origin`,
and `@worktree-own` was made on the clone.
Before promoting:

```text
MEASUREMENT before-promote checkout snapshots=[("@earliest", 2400177439762010623, 187), ("@origin", 11440608734556653149, 188), ("@checkout-later", 17816584427283206447, 189)] bookmark=(2400177439762010623, 187)
```

(names shortened here;
the log has them whole).
`ZFS_IOC_PROMOTE` returned 0,
and the reply to the next `OBJSET_STATS` reads reported:

```text
MEASUREMENT after-promote checkout origin=.../worktree-clone-datasets/promotion@origin promoted origin='' moved=[("...promotion@earliest", 2400177439762010623, 187), ("...promotion@origin", 11440608734556653149, 188), ("...promotion@worktree-own", 17743501436479443848, 192)]
```

So the origin and the older checkout snapshot moved into the promoted clone
with the **same GUIDs and creation transaction groups**,
the later checkout snapshot
stayed where it was,
and the checkout became a clone of
`promotion@origin`.
The bookmark moved to the promoted clone with the same
`(guid, createtxg)` tuple,
`(2400177439762010623, 187)`.

The assertion is per moved snapshot:
`live_promote_tests::promotion_moves_the_older_history_without_changing_identity`
compares the whole `(name, guid, createtxg)` row of each moved snapshot,
not
its GUID alone,
and then requires the reverse promotion to restore the checkout's and the
promoted clone's snapshot rows exactly.
Controls:
a promotion whose target is not a clone was refused `EINVAL` with no
conflicting name,
and a promotion with a snapshot name on both sides was refused
`EEXIST` with the conflicting name `earliest` in `zc_string`
(`live_promote_tests::a_conflicting_snapshot_name_is_refused_and_named`).

## Item 12: the same name replaced

As root,
clone,
`ZFS_IOC_DESTROY` of the unmounted clone,
clone of the same name
again,
and the identity read each time:

```text
MEASUREMENT replacement guids before=2418104187328297464 after=7144921161776676990
```

A reused name is a different object.
As the delegated account,
the same three
requests (destroy,
then clone) are
`live_account_tests::the_account_replaces_a_clone_name_and_gets_a_new_identity`,
with the same assertion.
The channel program the root alternative needed is not
part of this crate and no arm sends it.

## Items 14, 17 and 18: the snapshot batches, as root and as the account

Item 17 measured the delegated batch on a **promoted** clone and item 18 on an
unpromoted one,
so both shapes run here.
The record arm promotes
`promotion` before either batch,
which makes that clone the holder of the checkout's
`@earliest` and `@origin`,
and then sends both sets through the binding.
It asserts what it prints:
the own batch returns with no per-name failure and leaves
the moved history in place,
and the mixed batch refuses with `EEXIST`,
one per-name
error naming only the dependent snapshot,
and every name still listed.

Own-snapshot set,
one request,
root:

```text
MEASUREMENT promoted-batch own=.../promotion@own-one,.../promotion@own-two carried=0 refused=Some(17) elements=1 kept=[...@earliest, ...@without-dependent, ...@origin, ...@worktree-own, ...@with-dependent]
MEASUREMENT dependent clone origin=.../promotion@with-dependent
```

The reply carried no per-name errors and both own names were destroyed;
the
checkout's moved history kept its GUIDs and transaction groups.
The same two
sets also run in an asserting arm of their own on a promoted clone
(`live_batch_tests::a_promoted_clone_destroys_its_own_snapshots_and_refuses_the_dependent_one`),
beside the unpromoted arms
(`live_batch_tests::a_mixed_batch_keeps_every_name_and_names_the_dependent_one`
and `::an_own_snapshot_batch_destroys_both_and_keeps_the_checkout_history`).

The delegated account repeated both arms on the pool the root run prepared.
Its fixture promotes one clone of the checkout's `@origin` before the grants
are set,
and every other arm clones the checkout's later snapshot,
which the
promotion leaves in place,
so no other arm depends on the moved history:
only the design's three grants (`create,mount` local on the clone parent,
`clone` descendent on the checkout,
`mount,destroy,userprop` descendent on the
clone parent).
The promoted arms report:

```text
MEASUREMENT account promoted own-batch destroyed=2 carried=0 kept=4
MEASUREMENT account promoted mixed-batch refused=Some(17) elements=1 kept=6
```

- `live_account_tests::the_account_destroys_its_own_snapshots_on_a_promoted_clone`:
  return 0,
  no per-name errors,
  both own names gone,
  the four other
  names (the moved `@earliest` and `@origin` and the two the mixed set names)
  still listed.
- `live_account_tests::the_account_batch_with_a_dependent_clone_on_a_promoted_clone_keeps_both_names`:
  `EEXIST` with one per-name error naming the dependent snapshot,
  both kept,
  the listing unchanged.
- `live_account_tests::the_account_destroys_two_own_snapshots_in_one_batch`
  and
  `::the_account_batch_with_a_dependent_clone_keeps_both_names` repeat them on
  an unpromoted clone,
  item 18's shape;
  `::the_account_cannot_create_a_snapshot_of_the_checkout` gets `EPERM`;
  `::the_account_sets_a_user_property_on_its_clone_and_not_on_the_parent`
  is accepted on the clone and gets `EPERM` on the parent,
  because the grant is
  descendent;
  `::the_account_reads_the_prepared_clones_identity_and_snapshots` reads the
  clone's identity,
  its two own snapshots and the prepared children.

Logs:
`recorded-run-2026-11-19-review-fixes/account-suite.log` (8 tests,
all passing)
and its `root-suite.log` (194 tests).

## The reads: cursor, property sources, reply growth

The layout comparison is above.
The remaining reads:

- Cursor:
  `live_read_tests::listing_the_clone_parent_returns_every_child_and_then_its_end`
  follows the kernel's cursor over the clone parent,
  takes the names in the
  kernel's own order (a ZFS name list is not alphabetical),
  checks the first
  step's GUID against `zfs get`,
  and requires `Ok(None)` at the end rather than
  an error.
- Snapshot listing of an unpromoted clone:
  only the clone's own snapshots are
  listed,
  and each step's GUID equals `zfs get guid`.
- Property sources:
  `OBJSET_STATS` reports `recordsize` as `32768` with the
  source `zfsstep2-record`,
  the pool root where it was set,
  which is what
  `zfs get` renders as `inherited from zfsstep2-record`;
  a user property set on
  the checkout reports its value and the checkout as its source.
  Recorded:
  `MEASUREMENT recordsize=32768 source=zfsstep2-record`.
- Reply growth:
  174 user properties of 3000 bytes each made the property list
  bigger than the 256 KiB the crate starts from.
  The kernel answered `ENOMEM`
  with the size it needed,
  and the same read succeeded after one growth:

  ```text
  MEASUREMENT reply initial=262144 requested=553024 properties=174 value_bytes=3000
  ```

  The test also reads the same list through the transparent path,
  which grows
  the buffer itself (`live_read_tests::a_reply_past_the_initial_buffer_is_read_after_one_growth`).

## Permissions read back

`GET_FSACL` on the clone parent and the checkout reported the fixture's grants
keyed by `ul$<uid>` (local) and `ud$<uid>` (descendent) for the account's uid,
with `create,mount` local,
`mount,destroy,userprop` descendent on the clone
parent,
and `clone` descendent on the checkout,
and nothing else
(`live_read_tests::the_granted_delegations_are_what_the_fixture_set`).

## Red half

Two copies of the committed crate,
each with one deliberate break,
ran one live test that had to fail.
`red-half-2026-11-19-review-fixes/` holds each copy's suite log and its
one-hunk patch:

- `red-btf.patch` and `red-btf.log`:
  `zc_defer_destroy`'s expected offset is eight bytes too high in
  `live_btf_fields.rs`;
  `live_btf_tests::the_running_modules_type_information_matches_the_rust_mirror`
  failed with `assertion left == right failed: the offset of zc_defer_destroy`,
  `left: 13648` (the guest's `/sys/kernel/btf/zfs` through `pahole`) and
  `right: 13656` (the broken table).
  That is the layout assertion itself,
  not fixture setup.
- `red-promoted-batch.patch` and `red-promoted-batch.log`:
  `destroy_snaps` packs only the first name (`snapshots.get(..1)`);
  `live_batch_tests::a_promoted_clone_destroys_its_own_snapshots_and_refuses_the_dependent_one`
  failed with
  `the two own names are gone: [...@own-two, ...]`,
  so the batch destroyed
  only `own-one` and the promoted-clone assertion caught it.

Each copy's step failed only when its test failed;
a copy that kept passing
would have stopped the run.

## What this run did not show

- A user hold instead of a dependent clone,
  a `defer` key,
  sets larger than two
  names,
  name sets from more than one dataset,
  and request or batch limits.
  Items 14,
  17 and 18 left those open and this run does not close them.
- A head destroy interleaved with a batch,
  a snapshot created between the
  inspection and the request,
  or any crash or interruption.
  Steps 16 and 17 own
  those.
- Mounts,
  unmounts,
  the descriptor attach,
  the zedlet and boot:
  steps 4,
  9 and
  19.
  No arm mounts anything.
- The delegated account's behaviour against a name outside its grants,
  and a
  concurrent same-name replacement.
- The host kernel `7.3.5-1-cachyos`,
  the release `2.56.1` of Git,
  and the
  architecture range beyond x86-64.
  Step 23 owns the kernel.
- An untested-release refusal and any start-time gate:
  step 6 has no design yet.
  The layout comparison reads the module's BTF as that gate's
  source will,
  but nothing refuses to start on a mismatch yet.

[abi]: ../../../../doc/planning/cli-git-zfs-worktree-design-proposal.md#abi-gate-and-tests
