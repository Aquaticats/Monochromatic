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
# the step 3 arm, on a guest the two steps above prepared:
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- wrapper --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- arm --name my-guest
# the step 4 attach arm, on the same guest:
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-attach-pool --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-attach-arm --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-gate-arm --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-gate-version-arm --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-gate-change-arm --name my-guest
# the step 7 arms (the intent property, the receipt and the nudge), in this order, on the same guest:
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-protocol-pool --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-protocol-arm --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-protocol-teardown --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-protocol-record --name my-guest
# the step 16 snapshot-batch arm, on the same guest:
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-batch-pool --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-batch-arm --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-batch-teardown --name my-guest
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-batch-record --name my-guest
```

The four `zfs-protocol-*` steps are the step 7 arm of
`doc/planning/cli-git-zfs-worktree-implementation-plan.md`'s
"Step 7:
The intent property,
the receipts and the readiness record".
`zfs-protocol-pool` makes a 1 GiB encrypted pool and starts a real `zfs-zed` with
one test zedlet,
`zfs-protocol-arm` pushes the wrapper's packages,
builds its tests inside the guest and runs
`mount_protocol_arm_intent_receipt_and_nudge`
(refusing when the arm skips or prints no `MEASUREMENT mount-protocol` line),
`zfs-protocol-teardown` destroys the pool and records what it left,
and `zfs-protocol-record` reads the guest's versions and pulls the logs.
The run's own log names are `step7-pool.log`,
`step7-wrapper-build.log`,
`step7-suite.log`,
`step7-teardown.log` and
`step7-versions.log`.

- `create` makes the guest with `mvm`,
  on the backend this run names (see "The backend the guest is made on"),
  and refuses a name `mvm list` already reports,
  so a run never takes over a guest another run holds.
- `prepare` checks the guest runs the plan's kernel release,
  stops and masks the image's unattended-upgrade services and timers,
  waits for the `dpkg` lock they may still hold,
  installs the packages,
  copies the kernel's own BTF into the kernel build directory so the module is
  built with BTF,
  builds OpenZFS 2.4.4 from the release tarball against the
  guest's own kernel headers,
  loads that module (see "`prepare` in a guest that is already prepared"),
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
- `wrapper` runs the step 3 arm of the plan
  ([`doc/planning/cli-git-zfs-worktree-implementation-plan.md`](../../../../doc/planning/cli-git-zfs-worktree-implementation-plan.md),
  step 3):
  it pushes the wrapper's packages and builds their tests inside the guest,
  makes a pool of its own,
  runs the wrapper's ZFS clone controls and its disk measurement,
  runs H1 gate 7,
  destroys that pool,
  and pulls the logs back.
  `arm` runs `create`,
 `prepare` and `wrapper` in order and destroys the guest,
  which is how one green run is made from nothing.
- `record` writes the versions and the teardown state to the log directory and pulls
  the suite logs back to it.
- `destroy` destroys the guest and records what `mvm list` holds afterwards.
- `all` runs those five in order,
  destroys the guest its own `create` returned even when a later step fails,
  so a failed run leaves nothing running,
  and attempts `record` first when its own `create` returned,
  which pulls both suite logs when a run reached both suites;
  a `create` that threw is never followed by `destroy`,
  because the guest is then either absent or not this run's to remove,
  and the run says so instead.
  `--keep-guest` skips the destruction,
  so a failed run's guest can be inspected and its logs pulled by hand.
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

## The backend the guest is made on

The recipe takes its backend from one place,
`e2e/guest-backend.ts`,
which reads what `mvm` itself reads:
`--backend`,
then the `MVM_BACKEND` environment variable,
then `libvirt`.
A run that names nothing makes the local guest it always made,
and the module holds what each backend answers for the facts a guest step meets:

- the guest's hostname,
  which is the name the run gave on `libvirt`
  and `mvm-<name>` on `ovh`
  (`OVHcloud turns an instance name into its hostname`);
- what runs a command as root,
  which is nothing on `libvirt`,
  whose guest agent is root,
  and `sudo --non-interactive --` on `ovh`,
  whose commands run over SSH as the image's account;
- where `mvm push` puts a file and where the guest reads it,
  which is the guest's `/mnt/shared` share on `libvirt`
  and the account's `/home/ubuntu` on `ovh`;
- whether a step may reboot the guest or cut its power,
  which `ovh` cannot do at all:
  `mvm` never stops,
  pauses,
  shelves or resumes an instance on it.

Each backend names the image a run that says nothing is made from:
`libvirt` takes `mvm`'s own Ubuntu 24.04 template,
and `ovh` takes the catalogue's `Ubuntu 24.04`,
which is the release the recipe's packages and kernel pin are written for
(the record's `defaultGuestImage`,
read by a driver's `create` with no change of
its own).
`--guest-image <name>` names an image in a cloud backend's own catalogue,
which `prepare` still installs and builds against;
`--image <template>` is the libvirt custom template,
which `prepare` only asserts.
`--kernel-release <release>` names the kernel the prepared guest must boot,
and defaults to the release `libvirt`'s baked image carries,
or to the release a cloud guest answers with,
printed before anything is built against it.

The steps whose subject is a reboot or a power cut refuse on a backend without that,
by name,
before the run creates anything:
`zfs-add`,
`zfs-add-boot-*` and `zfs-add-power-*`.

The folder the host's files land in is `/mnt/shared` on either backend:
`mvm create` mounts a libvirt guest's share there,
and a backend that mounts none has the folder made once,
as root,
owned by the account every `scp` and `ssh` runs as
(`makesThePushedFolder`).
Every push goes into it and every pulled log comes out of it,
so a step's scripts are the same on both backends.

A run on `ovh` costs money by the second until the machine is deleted,
so the round that makes one lists the project before the create,
makes one machine at a time,
destroys it by its name (never `--all`,
which deletes other rounds' machines),
and ends with a listing that shows none of its own.
The recipe's `destroy` step does the first half of that;
the listing is the round's.

The first runs of this kind,
2026-11-28,
made `b3-16` in `BHS5` from the image `Ubuntu 24.04`
(the size `mvm`'s own README names as an example of a larger flavour;
the guest's own processor count and memory were not read before it was destroyed)
(the recipe's own run,
and then a wrapper driver's through its `mise` task with
`MVM_BACKEND=ovh`),
prepared one (kernel `6.8.0-136-generic`,
OpenZFS 2.4.4 built in the guest),
ran the step 4 attach arm on the first (6 passed,
teardown clean) and a wrapper driver's guest arm on the second,
and destroyed each by its name.
Its timings and what it refused first are in
`package/git-policy/cli/e2e/README.md`,
"The recipe's first OVHcloud run".
The run is unapproved and non-final.

## `prepare` in a guest that is already prepared

`prepare` ends 0 in a guest it has already prepared,
and in a prepared guest whose
arms have since made pools,
and it says so:
it removes nothing and inserts nothing,
so
an imported pool and its mounts keep running.

It reads whether each module is loaded from its own `/sys/module` directory,
and the version each loaded one publishes from `/sys/module/zfs/version` and
`/sys/module/spl/version`,
and
which one this recipe built from the version the installed file states
(`modinfo --field version /lib/modules/$(uname -r)/extra/zfs.ko`).
The same two readings are taken for `spl`.
Those readings leave four states:

- the built module is loaded:
  it says the guest already runs it,
  and changes nothing;
- nothing is loaded,
  as in a fresh guest:
  it inserts the built modules;
- another module is loaded and can be removed:
  it removes it,
  `zfs` first and then
  `spl`,
  inserts the built ones,
  and prints each removal;
- another module is loaded and will not go:
  it refuses,
  naming the version loaded,
  the version built and the file it came from,
  `modprobe`'s own words,
  the kernel's reference count,
  the ZFS datasets the kernel lists as mounted,
  the pools it lists as imported,
  and the two ways out:
  unmount those datasets and export or destroy those pools and run `prepare` again,
  or reboot the guest and run it again.

A mounted dataset is what holds the module.
With one dataset mounted the kernel counted one reference to `zfs`;
with a pool imported and nothing mounted it counted none,
and `modprobe --remove zfs` took the module out.
So the refusal reads the mount table at `/proc/self/mountinfo` and the pool listing at
`/proc/spl/kstat/zfs` from the kernel's own `/proc`,
and names both.

The version string is the whole identity,
so a module this recipe built from the
tarball and then rebuilt reads as already there,
which is what a repeat `prepare`
needs,
while a module of another release,
such as the distribution's `2.2.2`,
never
does.
Every removal and every insert is checked,
so a module that would not go ends the
step in those words instead of an `insmod` that meets a module already there
(issue 6638).

The step 10 arms and the two module-changing gate arms call the same step before they
touch a pool or the module,
so none of them goes on with a module other than the one this recipe built:
each either loads it or ends in the words above.

`mise run //package/zfs/request-to-openzfs-kernel-device:test:tooling` holds that step's
shell on the host without a guest,
against stand-in trees and stand-in `modinfo`,
`modprobe` and `insmod`,
and holds the template guest's step below the same way;
a guest run is what shows the real kernel answering.
[`doc/prepare-in-a-prepared-guest-2026-11-27.md`](../doc/prepare-in-a-prepared-guest-2026-11-27.md)
records those runs,
with each exit and the lines each step printed.

## `prepare` in a guest made from the step 23 template

`prepare --image <template> --kernel-release <release>`
runs `prepareFromTemplate`,
which loads the module the template installs and says which one it loaded,
then checks the template's userspace,
its module type information,
the release tree and the tools.
The load is the step in `e2e/load-the-template-module-step.ts`,
a smaller step than the built module's,
because the template builds nothing:
it installs `linux-cachyos-zfs`,
whose module files are zstd-compressed in
`/lib/modules/$(uname -r)/extramodules/`,
so `modprobe zfs` is the route that loads them,
`insmod` cannot load those files at all,
and there is no `extra/zfs.ko` to compare against and nothing to remove.

It keeps `modprobe`'s own words and its status,
reads the version the file this template installs states
(`modinfo --field version zfs`,
and the file's own path from `modinfo --field filename zfs`),
and compares the loaded module with that file and with the OpenZFS release
this recipe pins,
for both module files.
It reads before it loads,
so a file that cannot be read,
or that carries another release,
is refused with nothing loaded.
Those readings leave five states:

- the guest already runs the module the template installs:
  it says so and loads,
  removes and inserts nothing;
- nothing is loaded,
  as in a guest whose module was taken out:
  `modprobe zfs` loads it,
  and the step says which module it loaded and where it came from;
- the load fails:
  it refuses,
  naming `modprobe`'s own words,
  what is loaded now,
  the file the template installs and the release this recipe pins,
  and what to check before running the step again;
  a `modprobe` that exits 0 and loads nothing is refused the same way,
  because the version read after it would fail with no reason;
- the installed file states another release than the one this recipe pins,
  in either of the two files:
  it refuses before anything is loaded,
  naming both releases and the file;
- a module is loaded that is not the one the file states:
  it refuses,
  naming the version loaded,
  the version the file states,
  the release this recipe pins,
  and the two ways out:
  reboot the guest,
  which loads what the template ships,
  or take the other module out with `modprobe --remove`
  when nothing holds it,
  and run the step again.

A `modinfo` that cannot read the file is a refusal too,
so a template whose module files are gone ends in words rather than in a bare status,
and nothing is loaded.
A refusal after a `modprobe` that failed says what the guest is left running,
because the kernel may have loaded `spl` before refusing `zfs`.
[`doc/prepare-in-a-prepared-guest-2026-11-27.md`](../doc/prepare-in-a-prepared-guest-2026-11-27.md)
records the guest runs,
issue 6644's own,
with the kernel release and the module release the template answered.

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

## The step 3 arm: the wrapper's ZFS clone controls

`wrapper` answers the question the plan's step 3 leaves to the guest:
does rung 2 of the cheap-worktree ladder serve one ZFS dataset,
and does every cross-dataset add end in Git's own checkout?

It needs the guest `prepare` made
(Rust,
 `cargo-nextest`,
 OpenZFS 2.4.4 and the kernel BTF),
and it pushes the wrapper crate's packages,
whose manifests name `../../` paths,
so the repository root is what `--wrapper-repository` names (default:
 the one
this recipe lives in).
The wrapper's tests are **built inside the guest** for the same reason the
binding crate's are:
the host's glibc is newer than Ubuntu 24.04's.

The pool is `zfsstep3` under `/home/zfs-step-3-fixture`,
built the way the measurements' fixture was:
1 GiB,
 `aes-256-gcm`,
 a raw key,
 `recordsize=32K`,
 `compression=zstd-3`.
Its datasets are
`checkout` (the wrapper's fixture root),
`second` (another dataset under the same key),
`diffkey` (an encryption root of its own,
 so under another key),
and an empty snapshot `checkout@step-3-clone` for the controls that need a
mounted clone.
The controls read these variables,
 which only this step sets:

- `CLI_GIT_TEST_ROOT`:
 `/home/zfs-step-3-fixture/checkout`,
 the fixture root the
  binary controls make their own directories in.
- `CLI_GIT_ZFS_FIXTURE_GUEST`:
 the guest's hostname.
  A ZFS arm changes datasets,
  so `binary_support_zfs_controls.rs` refuses to run anywhere else,
  and prints `ZFS CONTROL NOT RUN` there instead.
- `CLI_GIT_ZFS_TOOL_DIRECTORY`:
 `/opt/openzfs-2.4.4/sbin`,
  the `zfs` and `zpool` the controls make the clone and read the counters with.
- `CLI_GIT_ZFS_SECOND_DATASET`,
 `CLI_GIT_ZFS_DIFFERENT_KEY_DATASET`,
  `CLI_GIT_ZFS_CLONE_SNAPSHOT`:
 the second dataset's directory,
  the other key's,
  and the snapshot a mounted clone is made from.
- `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1`:
 this guest has no btrfs root,
 and no btrfs control
  runs in this step.
- `CLI_GIT_ZFS_ABSENT_ACKNOWLEDGED` is **left unset**,
 so
  `zfs_a_real_mount_answers_with_its_kind` (a library control) really runs here.

The arms run **one at a time** (`--test-threads 1`):
the pool's block-clone counters count the whole pool,
and the arm that proves the clone rung ends where cloning cannot happen turns
the kernel's one block-cloning switch off for its own length.

### What the arms read

ZFS answers `FIEMAP` with no physical extent
(`filefrag` reports `FIBMAP/FIEMAP unsupported` on a ZFS file),
so the btrfs controls' shared-extent reading has no ZFS form.
The readings are the pool's own,
after `zpool sync` has committed the arm's transaction group:

- `bcloneused`:
 the space the pool's cloned blocks take.
  It grows when an arm shares blocks,
  and a byte copy never adds to it.
- `bclonesaved` and `bcloneratio`:
 the bytes a clone saved and the pool's own
  ratio of the two.
- `allocated`:
 what the pool holds in all,
  which is how an arm that wrote its own bytes shows it spent space.

The measurement (`binary_worktree_clone_zfs_measurement_tests.rs`) runs Git's
checkout and the wrapper's add over the same tree,
alternating them for two rounds,
and prints:

```text
MEASUREMENT zfs clone saved 22979072 bytes of 34719432 in 2456 files over 2 alternating rounds (Git's checkout used +29997056 bytes, the clone rung used +7017984 bytes and counted +11663872 cloned bytes; pool bcloneused 11663872, bclonesaved 23327744, bcloneratio 3.00x)
```

That run of 2026-11-20 measured the tree as this crate's own `src`:
 2,456 files
and 34,719,432 bytes,
written by Git's checkout at 29,997,056 pool bytes and by the clone rung at
7,017,984,
so a clean same-dataset add of that tree saved about 22.98 MB of pool space
against Git's checkout.

### H1 gate 7

`step3GateSeven` runs the gate the plan's "Descriptor safety and mount
authority" quotes:

- the permitted host mount of the clone is the control,
  because a refusal alone proves nothing;
- a confined user namespace (`unshare --user --map-root-user --mount`) is asked to
  attach the same clone with no flags and with `-o ro`,
 `-o rw`,
 `-o zfsutil` and
  `-o ro,zfsutil`;
  every attempt is refused with `unable to open the dataset`,
  although OpenZFS declares `FS_USERNS_MOUNT` (the log counts the declaration in
  the release tree the guest built);
- four late-mount arms start a confined consumer before the clone is mounted and
  read whether it sees the clone's marker:
  shared and private propagation,
  each with and without a read-only `rbind` of the fixture root.
  What they measured on 2026-11-20:
  shared without the bind `VISIBLE`,
 private without it `HIDDEN`,
  shared with the bind `VISIBLE`,
 private with the bind `HIDDEN`.
  So a read-only parent bind hides a mount made afterwards only under a
  private checkout;
  the design proposal's "Confined callers" says it "can hide" one,
 which holds
  for the private arm.
- the same-dataset reflink fallback runs **inside** that confined consumer:
  the wrapper is started as the guest's account in a user namespace,
 and the
  pool must count cloned bytes over that add.

`kernel.apparmor_restrict_unprivileged_userns` is 1 on this image,
which would make a confined consumer's mount attempt fail for the wrong
reason (AppArmor refusing the user namespace),
so the arms write 0 to it,
print it before and during,
and say so in the log.
The file is the kernel's and the guest is thrown away with its pool.

### What the arm's logs are

`step3-pool.log`,
 `step3-suite.log`,
 `step3-gate-7.log`,
`step3-wrapper-build.log`,
 `step3-teardown.log` and `step3-versions.log`
land in the log directory.
The suite log is the whole nextest run rather than the tail the step prints,
and `step3-teardown.log` shows the pool gone and the datasets gone,
and names the pool's `pool.img` file,
which `zpool destroy` leaves behind in the guest's fixture root and which goes
with the guest.

## The step 4 attach arm

`zfs-attach-arm` runs step 4's guest arms of the wrapper's descriptor attach
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 4,
"VM measurements")
in a guest this recipe prepared:
the clone mounted at the slot the walk held,
item 7's setuid control on that mount's enforced flags,
item 2's refusals with an account-owned parent and slot,
the rename that lands an attach beside its slot and its descriptor rollback,
a mount made beside the held slot that the post-attach check's first proof refuses,
and the decoy control of H1 gate 1.
The check's first proof reads the mount at the held target's name beneath the held parent
(a fresh `openat2` beneath that descriptor),
never the held target's own descriptor,
which keeps naming the mount it was opened on.
`zfs-attach-pool` makes the arm's own pool and fixtures alone,
so a run that only needs them does not repeat the build.

The arm's pool is `zfsstep4`:
a 1 GiB encrypted image at `/home/zfs-step-4-attach-fixture`,
a `checkout` dataset the tests' checkouts stand on,
an empty snapshot of it,
and the unmounted clones `clone-one` and `clone-beside`,
which are legacy mountpoints,
so ZFS never mounts them by itself
and an arm's own kernel mount is the only one they see.
The arm's tests live in the wrapper crate,
`src/native/worktree_clone_mount/attach_arm_tests.rs`,
and are `#[ignore]`d;
one that cannot reach its fixture prints `SKIP worktree_clone_mount::attach_arm_tests::...`
and returns,
and the step fails on that line,
so an arm that never ran is never counted as passed.

Everything in this arm is a root-run integration test of the wrapper's module,
not part of the account-side suite:
the `fsopen`,
`fsconfig`,
`fsmount` and `move_mount` calls are the test's own,
and the wrapper's own copy is what the arms present after step 5
(`git_policy_cli::worktree_clone_mount::attach_the_clone`,
reached through the root modes' `mount-path-descendant` and the clone attach).

The step leaves the guest as it found it:
the teardown takes down every mount still below the fixture root
(a killed test cannot run its own guard),
destroys the pool,
removes the fixture root,
and records what is left,
and the guest itself is destroyed by the run that made it.

## The step 6 gate arms

The recipe carries step 6's guest arms of the start-time layout comparison
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 6,
"VM measurements")
beside the recipe itself:

- `run` sets `MONOCHROMATIC_ZFS_FIXTURE_GATE_EXPECT=compatible`,
  so its root suite runs the control against the guest's approved module,
  fails on a `SKIP gate_arm` line,
  and requires the `MEASUREMENT gate` line.
  The control also reads the header length the module's own type-information
  file states and asserts two things about it:
  the length the checker reports is that same value,
  and it is one of the two shapes `btf_header.rs` knows,
  the 24-byte header Linux 6.8 wrote and the 32-byte header a current tool
  writes.
  No length is written into the arm,
  so a guest whose tools write either header passes,
  and a module whose file states a length between them,
  or a report that disagrees with the file,
  fails.
- `zfs-gate-arm` runs that same control again on a prepared guest.
- `zfs-gate-version-arm` patches the built module's release text from
  `2.4.4-1` to `2.4.4-2` and reloads it,
  then expects a compatible verdict.
  The layout and every request number are the approved module's by
  construction,
  so this arm shows that no step reads the release string;
  it is not a second release.
- `zfs-gate-change-arm` first runs a red arm the approved layout must refuse
  (it is told to expect `member:dds_flags`),
  then swaps `zi_iotype` and `zi_duration` in the guest's own
  `include/sys/zfs_ioctl.h`,
  rebuilds and installs the module,
  reboots,
  reloads it,
  and expects the exact member refusal naming `zi_iotype` at 2592 bits where
  2560 was expected.
  The swap moves no total size and no other offset,
  which is the design's owed item 16.

Every arm fails on a `SKIP gate_arm` line or a missing `MEASUREMENT gate`
line,
so an arm that never ran is never counted as passed.
The red arm the change arm runs first also has to print the arm's own panic
naming the module that passed where it should have refused,
so a refusal for another reason fails it.

Each of the two module-changing arms leaves the guest holding the module it
made:
`zfs-gate-change-arm` leaves the rebuilt module with `zi_iotype` and
`zi_duration` swapped,
and `zfs-gate-version-arm` leaves the approved module with its release text
patched to `2.4.4-2` and every pool destroyed.
Neither puts the approved module back,
so `zfs-gate-arm`'s control sees a changed module if it follows either,
and each module-changing arm needs its own guest unless the arm run after it
rebuilds the module from source.
`zfs-gate-change-arm` rebuilds it from the guest's own header,
so it does not depend on the module text `zfs-gate-version-arm` patched:
the integration pass ran the version arm and then the change arm in one guest
and both passed,
and it did not run the change arm before the version arm.
These arms drive the binding's `check_module_layout` against the guest's two
type-information files.
They do not run the product's gate (`zfs_compatibility_gate::check`) or the
handler binary,
and they send no request:
the product half belongs to the step that makes the handler's first request.

## The step 16 arm: the snapshot batch over a real clone

`zfs-batch-*` runs step 16's guest arms of the snapshot batch
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
step 16,
"VM measurements",
items 14,
17 and 18 repeated through the binding).
The piece under test is
`package/git-policy/cli/src/native/worktree_clone_snapshot_batch`,
which
step 16's wiring will call and no command calls yet.

- `zfs-batch-pool` makes `zfsstep16`:
  a 1 GiB encrypted image like the measurements' fixture,
  an unmounted
  `checkout` dataset with the base snapshot `checkout@batch-base`,
  and an unmounted
  `clones` parent.
  Both datasets are `mountpoint=legacy -o canmount=off`,
  so the arm mounts
  nothing and its teardown has no mount to take down.
- `zfs-batch-arm` pushes the wrapper's packages,
  builds the crate's tests inside the
  guest,
  and runs
  `worktree_clone_snapshot_batch::snapshot_batch_arm_tests`
  one test at a time.
  It fails on a `SKIP worktree_clone_snapshot_batch::` line and requires the
  `MEASUREMENT snapshot-batch requests=` line the split arm prints.
- `zfs-batch-teardown` destroys the pool and records what is left,
  and
  `zfs-batch-record` writes the versions and pulls the logs.

The arms read these variables,
which only this step sets:

- `MONOCHROMATIC_ZFS_SNAPSHOT_BATCH_GUEST`:
  the guest's hostname.
  The arm
  destroys snapshots,
  so it refuses to run anywhere else and prints `SKIP` when
  the variable is unset.
- `MONOCHROMATIC_ZFS_SNAPSHOT_BATCH_POOL`:
  `zfsstep16`.
- `MONOCHROMATIC_ZFS_SNAPSHOT_BATCH_SBIN`:
  the guest's OpenZFS `sbin`,
  which
  the arm runs `zfs` from.

The three arms and what each proves:

- one request:
  five snapshots of an arm's own clone,
  sent as the one chunk a budget
  that fits them makes;
  the request returns success,
  every name is `destroyed`,
  and the
  clone itself stands.
- a hold:
  four snapshots,
  one of them held with `zfs hold`;
  the request returns `EBUSY`,
  the per-name list names the held snapshot alone,
  every other submitted name is
  `intact`,
  and every snapshot is still there.
  A refused request destroys
  none of its set
  (`module/zfs/dsl_destroy.c:596` to `:600`,
  OpenZFS `zfs-2.4.4`),
  so the arm
  cannot show the rest being destroyed;
  a removal reports the hold and stops.
- a split:
  six snapshots and a budget that exactly fits two names;
  the plan makes
  three requests,
  each returns success with no per-name failure,
  and the six
  names are gone.
  The arm prints
  `MEASUREMENT snapshot-batch requests=<count> names=<count> budget-bytes=<bytes> encoded-bytes=<bytes>`.

The logs are `step16-pool.log`,
`step16-wrapper-build.log`,
`step16-suite.log`,
`step16-teardown.log` and
`step16-versions.log`.

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

## Moved from the old handover

From lines 43098 to 43099 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  `main` too).
  The author had set `CLI_GIT_ZFS_FIXTURE_GUEST=$(hostname)` on the host to run guarded controls,
  which
  defeats the guard (briefs now forbid it).
  GOING:
  the faster size reader's design and measurement
