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
`/home/user/temp/agent/zfs-step-2-live-requests-and-guest-fixture-2026-11-19/keep/`,
as the plan requires;
this file cites them by name.

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
  `6.8.0-147-generic` (`/proc/sys/kernel/osrelease`)
- OpenZFS userspace and module:
  `2.4.4`,
  built from `zfs-2.4.4.tar.gz` against
  the guest's own kernel headers;
  `zfs version` printed `zfs-2.4.4-1` and
  `zfs-kmod-2.4.4-1`,
  and `/sys/module/zfs/version` printed `2.4.4-1`
- module source version:
  `B2C6D65530FF8F5A7105ACC` (`/sys/module/zfs/srcversion`)
- Git:
  `git version 2.43.0` (Ubuntu's package;
  the design's `2.56.1` runs are
  step 23's,
  and step 2's items do not read Git)
- `pahole` `v1.25` (`dwarves`),
  `strace -- version 6.8`
- Rust `1.100.0` and `cargo-nextest 0.9.152`,
  installed in the guest

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
`recorded-run-final-2026-11-19/` is one end-to-end run of the recipe
(`all`):
it created the guest,
prepared it,
ran both suites,
recorded the
versions and the teardown,
and destroyed the guest.
It holds
`root-suite.log` (196 tests,
all passing,
with the `MEASUREMENT` block quoted
here),
`account-suite.log` (7 tests),
`versions.log`,
`teardown.log`,
`mvm-create.log`,
`mvm-destroy.log` and `mvm-list-after-destroy.log`.
`guest-run-1-red/`,
`guest-run-2-green/`,
`guest-run-3-green/` and
`guest-run-7-values/` are the failing runs that led here,
and
`guest-run-8-values/` is the last run on the guest kept for iteration,
with the
same versions.

## The layout comparison, and where it differs from the design's command

The design's inspection is
`pahole --btf_base /sys/kernel/btf/vmlinux -C zfs_cmd /sys/kernel/btf/zfs`
(["ABI gate and tests"][abi]).
This guest's module carries **no `.BTF`**:
Ubuntu's kernel headers generate module BTF only for in-tree builds
(`$(and $(CONFIG_DEBUG_INFO_BTF_MODULES),$(KBUILD_BUILTIN),vmlinux)` in
`scripts/Makefile.modfinal`),
and `KBUILD_BUILTIN` is empty for an out-of-tree
module such as the release tarball's.
`/sys/kernel/btf/zfs` therefore does not
exist,
and the BTF file the design's command reads cannot be produced here.

The test reads the same type information from the **loaded module's own DWARF**,
which `pahole -C zfs_cmd /lib/modules/$(uname -r)/extra/zfs.ko` prints in the
same layout form,
and it proves the file is the module the kernel loaded by
comparing `/sys/module/zfs/srcversion` with `modinfo -F srcversion` of that
file.
It compares `zfs_cmd`,
`dmu_objset_stats`,
`zinject_record`,
`zfs_share`,
`zfs_stat` and `drr_begin` field by field:
every member's name,
offset and size
against `offset_of!` and `size_of` of the Rust mirror,
plus each structure's
size,
and every request number against `include/sys/fs/zfs.h` of the release
tree.
Test:
`kernel_device::live_btf_tests::the_running_modules_type_information_matches_the_rust_mirror`
and `...::the_release_header_assigns_every_request_the_number_this_crate_sends`.

Recorded:
`sizeof (zfs_cmd_t) = 13744`,
`zc_inject_record` at 13296 with 368
bytes,
`zc_defer_destroy`,
`zc_flags` and `zc_action_handle` at 13648,
13652 and
13656 (`recorded-run-final-2026-11-19/root-suite.log`,
the layout line).
This is the same
layout the vet read from the host's module BTF on 2026-11-18,
and the test would
refuse a release that moved a field.

This is a **deviation from the plan's wording** "a test against the running
module's BTF inside the guest":
the guest's kernel headers cannot produce module
BTF for a tarball-built module,
so the test reads the loaded module's DWARF
instead.
The comparison's content is the same,
and the alternative (a
hand-encoded BTF attached to the module) is untested.
The owner should say
whether step 6's runtime gate needs the BTF file itself.

## Item 5: a clone under an unloaded key

The key was unloaded after unmounting the child dataset and then the parent
(`zfs unmount` has no recursive flag on this release).
With the key unloaded,
`ZFS_IOC_CLONE` of the encrypted snapshot returned 0 and the clone's
`OBJSET_STATS` reply reported:

```text
MEASUREMENT key-unloaded clone keystatus=1 origin=.../promotion@origin
MEASUREMENT key-loaded keystatus=2 (1 is unavailable)
```

`keystatus` is `ZFS_KEYSTATUS_UNAVAILABLE` (1) before the load and
`ZFS_KEYSTATUS_AVAILABLE` (2) after.
The clone's origin is the snapshot it was
made from,
so identity is readable while the key is not.
The control is the
same clone read again after `zfs load-key`.
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
MEASUREMENT before-promote checkout snapshots=[("@earliest", 722454792545308728, 187), ("@origin", 5925684245824955526, 188), ("@checkout-later", 10858864819157758307, 189)] bookmark=(722454792545308728, 187)
```

(names shortened here;
the log has them whole).
`ZFS_IOC_PROMOTE` returned 0,
and the reply to the next `OBJSET_STATS` reads reported:

```text
MEASUREMENT after-promote checkout origin=.../worktree-clone-datasets/promotion@origin promoted origin='' moved=[("...promotion@earliest", 722454792545308728, 187), ("...promotion@origin", 5925684245824955526, 188), ("...promotion@worktree-own", 7907649038118831957, 192)]
```

So the origin and the older checkout snapshot moved into the promoted clone
with the **same GUIDs and creation transaction groups**,
the later checkout
snapshot stayed where it was,
and the checkout became a clone of
`promotion@origin`.
The bookmark moved to the promoted clone with the same
`(guid, createtxg)` tuple,
`(722454792545308728, 187)`.

Controls:
the reverse promotion restored the snapshot map and the bookmark on
the checkout (`live_promote_tests::promotion_moves_the_older_history_without_changing_identity`);
a promotion whose target is not a clone was refused `EINVAL` with no conflicting
name;
and a promotion with a snapshot name on both sides was refused `EEXIST`
with the conflicting name `earliest` in `zc_string`
(`live_promote_tests::a_conflicting_snapshot_name_is_refused_and_named`).

## Item 12: the same name replaced

As root,
clone,
`ZFS_IOC_DESTROY` of the unmounted clone,
clone of the same name
again,
and the identity read each time:

```text
MEASUREMENT replacement guids before=14245730535037168358 after=11368344099117093483
```

A reused name is a different object.
As the delegated account,
the same three
requests (destroy,
then clone) are `live_account_tests::the_account_replaces_a_clone_name_and_gets_a_new_identity`,
with the same assertion.
The channel program the root alternative needed is not
part of this crate and no arm sends it.

## Items 14, 17 and 18: the snapshot batches, as root and as the account

Own-snapshot set,
one request,
root:

```text
MEASUREMENT own-batch result=Ok(Some(NvList { version: 0, nvflag: 1, pairs: [] }))
```

The reply carried no per-name errors and both names were destroyed;
the
checkout's three snapshots kept their GUIDs and transaction groups in the same
test (`live_batch_tests::an_own_snapshot_batch_destroys_both_and_keeps_the_checkout_history`).

Mixed set,
one request,
root:
a name with a dependent clone refused the whole
set with `EEXIST` and named only that snapshot:

```text
MEASUREMENT mixed-batch refused: DESTROY_SNAPS on 'zfsstep2-record' failed: File exists (os error 17) (already exists); .../promotion@with-dependent: File exists (os error 17)
MEASUREMENT dependent clone origin=.../promotion@with-dependent kept=[...@without-dependent, ...@worktree-own, ...@earliest, ...@origin, ...@with-dependent]
```

Both names survived,
which is the all-or-nothing result items 14 and 18
measured through a helper.

The delegated account repeated both arms on the pool the root run prepared,
with
only the design's three grants (`create,mount` local on the clone parent,
`clone` descendent on the checkout,
`mount,destroy,userprop` descendent on the
clone parent):

- `live_account_tests::the_account_destroys_two_own_snapshots_in_one_batch`:
  return 0,
  no per-name errors,
  both names gone.
- `live_account_tests::the_account_batch_with_a_dependent_clone_keeps_both_names`:
  `EEXIST` with one per-name error naming the dependent snapshot,
  both kept.
- `live_account_tests::the_account_cannot_create_a_snapshot_of_the_checkout`:
  `EPERM`,
  the permission the grants withhold.
- `live_account_tests::the_account_sets_a_user_property_on_its_clone_and_not_on_the_parent`:
  accepted on the clone,
  `EPERM` on the parent,
  because the grant is descendent.
- `live_account_tests::the_account_reads_the_prepared_clones_identity_and_snapshots`:
  the clone's identity,
  its two own snapshots and the prepared children.

Log:
`recorded-run-final-2026-11-19/account-suite.log` (7 tests,
all passing).

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
- Module BTF itself:
  see the deviation above.
- An untested-release refusal and any start-time gate:
  step 6 has no design yet.

[abi]: ../../../../doc/planning/cli-git-zfs-worktree-design-proposal.md#abi-gate-and-tests
