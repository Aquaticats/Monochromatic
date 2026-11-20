# The guest fixture the live requests run in

A live request opens `/dev/zfs`,
and this repository's host pool `zpcachyos` must
never be touched by a test.
Every live test in this crate therefore builds its own
file-backed encrypted pool inside a throwaway `mvm` guest,
and the guest is
disposable:
the recipe creates it,
prepares it,
runs the suites and destroys it.

The plan names this recipe in
[`doc/planning/cli-git-zfs-worktree-implementation-plan.md`](../../../../doc/planning/cli-git-zfs-worktree-implementation-plan.md),
step 2 ("The reproducible guest fixture,
as an `mvm` recipe"),
and the
measurements it repeats are in
[`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`](../../../../doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md).

## Running it

```sh
# package/zfs/request-to-openzfs-kernel-device: run from anywhere in the repository
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- all
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- create --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- prepare --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- run --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- record --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- destroy --name my-guest
```

- `create` makes the guest with `mvm`,
  whose default `libvirt` backend runs it on
  this machine.
  The `hetzner` backend is never used:
  it rents an outside server.
- `prepare` checks the guest runs the plan's kernel release,
  installs the packages,
  copies the kernel's own BTF into the kernel build directory so the module is
  built with BTF,
  builds OpenZFS 2.4.4 from the release tarball against the
  guest's own kernel headers,
  loads that module,
  and installs Rust and
  `cargo-nextest`.
  Its two downloads are checked against the sha256 in this
  recipe (`zfs-2.4.4.tar.gz` against OpenZFS's signed `sha256.asc`,
  `rustup-init` against the digest of the rustup release the recipe pins).
- `run` pushes the crate,
  builds it inside the guest,
  runs the live suites as root,
  then hands the tree to the guest's `ubuntu` account and runs the delegated arms as
  that account.
  Each suite must pass.
  It must print no skip line from a live arm,
  and a `MEASUREMENT` line only a live arm writes,
  so a run whose arms skipped fails instead of reporting a green suite.
  The root suite excludes the delegated arms and the account
  suite runs only them,
  each as the account the arm needs.
- `record` writes the versions and the teardown state to the log directory and pulls
  the suite logs back to it.
- `destroy` destroys the guest and records what `mvm list` holds afterwards.
- `all` runs those five in order,
  destroys the guest even when a step fails,
  so a failed run leaves nothing running,
  and pulls the suite logs of a failed run before it does;
  `--keep-guest` skips the destruction so the guest can be inspected.
  `--log-dir <dir>` moves the logs (default
  `target/e2e-logs` under this package),
  and `--crate <dir>` names the crate to
  push.

Nothing runs as root on this host,
and the host's `/dev/zfs` is never opened by a
test:
the fixture variables are set only inside the guest,
and each live test also
refuses to run unless the machine's hostname is the guest's.

## What the guest carries

The guest is Ubuntu 24.04 LTS from `mvm`'s cached cloud image,
whose kernel is the
image's own `linux-generic` release.
The plan pins `6.8.0-147-generic`,
which is the release the design's measurements ran.
`prepare` refuses any other release rather than measure it;
moving that constant is a plan change,
not a recipe detail.

- OpenZFS userspace and module:
  `2.4.4`,
  built from `zfs-2.4.4.tar.gz` against
  `linux-headers-$(uname -r)`.
  Both report `2.4.4-1`,
  the release the design pins.
- Git:
  Ubuntu's packaged Git.
  The measurements used `2.43.0`.
- `pahole`:
  Ubuntu's `dwarves`,
  for the layout comparison.
- Module BTF:
  Ubuntu's headers ship no `vmlinux`,
  and 6.8's
  `scripts/Makefile.modfinal` runs `pahole -J --btf_base vmlinux $@` for an
  out-of-tree module whenever `CONFIG_DEBUG_INFO_BTF_MODULES` is set (Ubuntu's
  is).
  `prepare` copies the kernel's own `/sys/kernel/btf/vmlinux` into the kernel
  build directory before the build,
  so the module carries `.BTF` and the kernel
  exposes it at `/sys/kernel/btf/zfs`.
  That file is what the layout
  comparison reads,
  the same file and command the design's ABI gate names.
- `strace`:
  Ubuntu's `strace`,
  for the plan's later arms.
- Rust and `cargo-nextest`:
  the pinned Rust `1.100.0` through the pinned `rustup` release,
  then the pinned `cargo-nextest 0.9.152`;
  `versions.log` records what a run actually used.
  The apt packages float with the image,
  which `versions.log` records too.

The crate is **built inside the guest**,
not on this host and pushed:
the host's
glibc is newer than Ubuntu 24.04's,
so a host-built binary would not start there,
and building in the guest keeps the test binary matched to the guest's libc without
a cross toolchain.

## The fixture variables

Every live test reads five variables and the release tree's path.

- `MONOCHROMATIC_ZFS_FIXTURE_GUEST`:
  the guest's hostname.
  A run on another machine panics with the reason;
  with the variable
  unset,
  a live test prints a `SKIP` line naming itself and the variable and returns.
- `MONOCHROMATIC_ZFS_FIXTURE_SBIN`:
  `/opt/openzfs-2.4.4/sbin`,
  the userspace the
  tests use to build their fixtures.
- `MONOCHROMATIC_ZFS_FIXTURE_ROOT`:
  `/home/zfs-step-2-fixture`,
  the directory each
  pool's image and mountpoints live under.
- `MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT`:
  `ubuntu`,
  the account the delegated arms run
  as.
- `MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT_POOL`:
  `zfsstep2-account`,
  the pool the root
  run prepares for the delegated arms.
- `MONOCHROMATIC_ZFS_FIXTURE_SOURCE`:
  `/root/zfs-2.4.4`,
  the unpacked release whose
  headers carry the request numbers.

The live tests are `#[ignore]`d with a reason naming this recipe,
so a gate run on
the host reports each one as skipped.
Running them with `--run-ignored` on a host
does not reach the device either:
with `MONOCHROMATIC_ZFS_FIXTURE_GUEST` unset
they print `SKIP <test>: ...` and return.
Inside the guest the variable is set,
and then a
live arm that cannot reach its fixture,
or a delegated arm that is not running as the
guest's account,
panics with the reason instead of returning:
a returning test is a
passing test,
which would hide a live assertion that never ran.

## The pools and what each arm proves

Each root-side test creates a pool named `zfsstep2-<arm>` under the fixture root and
destroys it when it ends,
however it ends.
A pool is a 1 GiB image with
`encryption=aes-256-gcm`,
`keyformat=raw` and a key file in the pool's folder,
`recordsize=32K`,
`compression=zstd-3`,
a mounted `home` dataset,
a mounted
`checkout` dataset,
and an unmounted clone parent with `mountpoint=legacy` and
`canmount=off`,
which is the measurements' fixture.

- `zfsstep2-layout`:
  every field of `zfs_cmd`,
  `dmu_objset_stats`,
  `zinject_record`,
  `zfs_share`,
  `zfs_stat` and `drr_begin` at the offset and size
  the running module's own type information reports,
  and every request number
  against the release header.
- `zfsstep2-cursor` and `zfsstep2-snapshots`:
  the child and snapshot listings follow
  the kernel's cursor and end with `Ok(None)`.
- `zfsstep2-properties`:
  property values and their sources match `zfs get`.
- `zfsstep2-snapshot`:
  one `ZFS_IOC_SNAPSHOT` request creates a snapshot of the checkout and one of
  a clone with a creation-time user property;
  the reply decodes with no per-name
  failure,
  each snapshot's GUID matches `zfs get`,
  and the property reads back with the
  snapshot as its source.
- `zfsstep2-grants`:
  `GET_FSACL` reports the design's grants,
  and nothing else under the account's
  local and descendent delegation keys.
- `zfsstep2-resize`:
  a property list past 256 KiB is reported as too big and read
  after one growth.
- `zfsstep2-create-props` and `zfsstep2-refused-props`:
  a creation-time user
  property is accepted and reads back;
  a real user property whose value is at the
  kernel's `ZAP_MAXVALUELEN` is refused with `E2BIG` and creates
  nothing.
- `zfsstep2-set-prop`:
  `SET_PROP` after creation sets the user property.
- `zfsstep2-replacement`:
  a head destroy removes the clone and the name can be
  reused with a new GUID.
- `zfsstep2-key`:
  a clone succeeds with the pool's key unloaded and `keystatus`
  reads `UNAVAILABLE`.
- `zfsstep2-mixed-batch` and `zfsstep2-own-batch`:
  a set with a dependent clone
  refuses with `EEXIST` and one per-name error;
  an own-snapshot set destroys both in
  one request and the checkout's GUIDs stay.
- `zfsstep2-promoted-batch`:
  the same two sets on a clone promoted before them,
  so the
  dataset whose own snapshots the batch destroys also holds the checkout's
  transferred history,
  which must survive.
- `zfsstep2-promote`,
  `zfsstep2-promote-conflict` and `zfsstep2-promote-plain`:
  promotion moves the older history without changing GUIDs,
  the reverse restores it,
  a name conflict names the snapshot,
  and a non-clone is refused.
- `zfsstep2-account`:
  the delegated account reads,
  destroys its own snapshots on a
  promoted clone and on an unpromoted one,
  is refused the dependent set on both,
  snapshots
  the checkout with the design's local `snapshot` grant and removes that
  snapshot,
  clones the checkout with a creation-time user property,
  sets a property on its clone
  and not the parent,
  and
  replaces a clone name with a new identity.
  Root prepares one clone of the checkout's
  `@origin` and promotes it before the grants are set;
  every other arm clones the
  checkout's later snapshot,
  which the promotion leaves in place,
  so no other arm depends
  on the moved history.

## Teardown evidence

`record` writes `teardown.log`:
`zpool list -H -o name` and
`zfs list -H -o name -r`,
and a `find` for `pool.img` under the fixture root.
A clean root run leaves the delegated pool `zfsstep2-account` in place,
because
the account suite reads it afterwards,
and every other pool is gone with its
image and key folder.
`destroy` removes what is left with the guest and writes
`mvm-destroy.log` and `mvm-list-after-destroy.log`.
Keep those with
`versions.log` and the two suite logs when recording a run.
