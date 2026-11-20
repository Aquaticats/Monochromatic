# zfs/request-to-openzfs-kernel-device

A small Rust crate that speaks the Linux `ioctl` interface of OpenZFS through
`/dev/zfs`,
for the Git wrapper's ZFS worktree route
(`doc/decision/cli-git-zfs-worktree-design.md`).
It holds no policy about Git worktrees or snapshots,
needs no ZFS library and never starts the `zfs` program,
so a program that links it still starts on a machine with no OpenZFS at all.
The requests are Linux only,
and the numbers and layouts are those of
OpenZFS `zfs-2.4.4`.

## What a device file is, for someone who knows Git

Git writes files and folders and asks the kernel for nothing more than that.
ZFS keeps its datasets,
snapshots and properties inside the kernel,
and the `zfs` command talks to it through one device file,
`/dev/zfs`.
The conversation on that file is an `ioctl`:
one system call carrying
a request number and one block of memory that the kernel reads and writes in
place.
The `zfs` command itself is a thin program around those calls.
`/dev/zfs` is owned by root,
and the kernel decides per request whether the
calling account may make it,
the same way Git decides whether a folder may be
written.

The request number selects the operation,
for example `0x5a18` to destroy one
dataset or snapshot.
The memory block is the C struct `zfs_cmd_t`:
fixed-size name fields,
a few
integers,
and two pairs of `(pointer, size)` fields for name-value lists.
A name-value list (`nvlist`) is how the kernel carries variable data:
pairs of
a name and a typed value,
packed into one byte buffer.
The same native list format serves every release,
but the command struct around it differs between releases,
as `PROVENANCE.md` records for `2.2.2` and `2.4.4`,
so a caller that knows the struct
and the list format needs no library from OpenZFS.
This crate owns the struct,
packs the lists,
and decodes the kernel's replies.

## The requests this crate can express

The design picked one small set of requests,
named by the C constants in
`include/sys/fs/zfs.h` of OpenZFS `zfs-2.4.4`.
Each is a typed Rust function;
there is no general-purpose ioctl call and no
generic property setter.

### Reads

- `ZFS_IOC_OBJSET_STATS` (`0x5a12`):
  the dataset type,
  identity,
  origin,
  and the value and source of its
  properties.
- `ZFS_IOC_DATASET_LIST_NEXT` (`0x5a14`):
  the next immediate child dataset,
  with the kernel's cursor in `zc_cookie`.
- `ZFS_IOC_SNAPSHOT_LIST_NEXT` (`0x5a15`):
  the next snapshot of one dataset,
  with the same cursor.
- `ZFS_IOC_GET_BOOKMARKS` (`0x5a44`):
  every bookmark of one dataset;
  the optional property names travel as
  boolean flags in the input list.
- `ZFS_IOC_GET_FSACL` (`0x5a29`):
  the delegations granted on one dataset,
  for `doctor`.

### Writes

- `ZFS_IOC_SET_PROP` (`0x5a16`):
  one or more properties on a named clone.
- `ZFS_IOC_DESTROY` (`0x5a18`):
  one named,
  unmounted dataset or snapshot,
  with the defer mark only when the
  caller asks for it.
- `ZFS_IOC_PROMOTE` (`0x5a22`):
  one clone,
  named in `zc_name`;
  the kernel returns a conflicting snapshot
  name in `zc_string`.
- `ZFS_IOC_SNAPSHOT` (`0x5a23`):
  snapshots named in the input list's `snaps` member,
  created from one pool.
- `ZFS_IOC_DESTROY_SNAPS` (`0x5a3b`):
  an explicit snapshot name set,
  without a `defer` member,
  so no snapshot is
  marked for deferred destroy.
- `ZFS_IOC_CLONE` (`0x5a42`):
  a clone whose name is `zc_name`,
  its `origin` snapshot and optional `props`
  in the input list.
- `ZFS_IOC_CHANNEL_PROGRAM` (`0x5a48`):
  one fixed root-owned program,
  its argument and its limits;
  the request
  exists for the root removal alternative the project did not select,
  and no
  production caller in this repository sends it.

## What a failed request tells you

Every failure returns the operation and the dataset or pool it named.
When the kernel answered with an error number,
that number comes too,
spelled out as the operating system's message for it and a hint.
A failure the crate raises before or instead of a kernel answer,
such as a refused reply buffer or a reply that does not decode,
names itself and the values it judged instead.
A write that the kernel refuses per element also returns those names,
for example the snapshots in a `DESTROY_SNAPS` batch that failed one by one:

```text
DESTROY_SNAPS on 'pool/dataset' failed: Device or resource busy (os error 16) (busy: mounted, held, or has children); pool/dataset@s1: Device or resource busy (os error 16); pool/dataset@s2: Device or resource busy (os error 16)
```

The device-free test `a_busy_snapshot_batch_prints_the_line_the_readme_shows`
compares what the crate prints with this file's copy of the line,
reading `README.md` itself,
so the two cannot drift apart.

A listing that reached its end says so by the kernel's `ESRCH`,
which is not a
failure:
`Error::is_end_of_list` is true only for that error number,
so a
caller never mistakes an exhausted cursor for a lost read.

## The live calls

`kernel_device::ZfsDevice` opens `/dev/zfs` once and carries a packed request to
the kernel:

```text
let device = ZfsDevice::open()?;                       // /dev/zfs, read and write
let identity = device.objset_stats("pool/dataset")?;   // stats and properties
let names = device.get_bookmarks("pool/dataset", &[])?;
let listed = device.snapshot_list_next("pool/dataset", 0)?; // None at the end
```

- One `unsafe` call,
  in `src/kernel_device/ioctl_call.rs`,
  sends the number and
  the command buffer's address;
  everything else is safe Rust.
- A read that the kernel answers with `ENOMEM` and a size past the buffer grows
  the buffer to it and asks again,
  at most twice;
  a third answer that still does
  not fit is reported,
  never looped on.
  This is how a property list larger than
  the 256 KiB the crate starts from is read.
- A write is never sent twice.
  A write whose reply was too small is reported as a `ReplyRefusal`,
  naming the size the kernel asked for and the size the buffer held.
  That refusal carries no error number and no per-name failures:
  the reply was never decoded,
  so it has no names,
  and the kernel's `ENOMEM` is the condition the refusal recognised rather than
  a field of its own.
  A refusal the kernel answered with per-name failures travels as
  `Error::Elements`,
  with its error number and those names.
- A listing that reached its end is `Ok(None)`,
  not an error:
  only `ESRCH` on a
  listing request means that,
  and `Error::is_end_of_list` says so too.
- `objset_stats` returns `DatasetIdentity`,
  the kernel's statistics (identity,
  origin,
  flags) and the property values with their sources;
  each listing step
  returns `ListedObject`,
  with the name and cursor the kernel wrote.
- `promote` exists in the live handle for tests only (`#[cfg(test)]`),
  because
  promoting a worktree clone moves checkout history into a removable dataset.
  Its refusal carries the conflicting snapshot's name the kernel leaves in
  `zc_string`.

## The guest fixture

A live request must never reach this host's pool,
so the live tests are
`#[ignore]`d and run only in a throwaway `mvm` guest:

```text
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- all
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- run --name my-guest
```

A plan check run on this host reports every live test as skipped,
and
`--run-ignored` on a host does not touch the device either:
with
`MONOCHROMATIC_ZFS_FIXTURE_GUEST` unset a live test prints why it did not run and
returns.
Inside the guest the variable is set.
A missing part of the fixture there,
or a delegated arm running as the wrong account,
panics instead of returning:
a returning test is a passing test.
The recipe goes further.
Each suite must also print no skip line from a live arm,
and a `MEASUREMENT` line only a live arm writes,
so a guest run whose arms skipped fails.
`e2e/README.md` lists the guest's contents (Ubuntu 24.04 on the plan's
`6.8.0-147-generic`,
OpenZFS 2.4.4 built from the release tarball against the guest's own kernel
headers with a module that carries BTF,
Git,
`pahole`,
`strace`,
Rust built inside the guest),
the variables,
the pools each arm builds and the teardown evidence.

The layout comparison reads the module BTF the kernel exposes at
`/sys/kernel/btf/zfs`,
with `pahole --btf_base /sys/kernel/btf/vmlinux -C zfs_cmd /sys/kernel/btf/zfs`.
That is the file and the command the design's ABI gate names,
and the recipe builds the guest's module so the file exists.

Between the arms,
the root run prepares a delegated pool whose account holds
only the design's three grants (`create,mount` on the clone parent,
`clone` on
the checkout,
`mount,destroy,userprop` on the clone parent),
and the account
suite runs as that account,
so a delegated refusal names the permission the
kernel wanted.

## What this crate does not do

- It checks no kernel compatibility at start.
  The start-time gate that compares the loaded module's release,
  architecture,
  endianness and layout is step 6;
  the guest tests compare the fields and
  request numbers now,
  and no program refuses to start on a mismatch yet.
- It never retries a mutating request.
  A reply buffer grows only for reads;
  a write whose outcome is unknown is
  reported,
  never replayed.
- It does not care which machine or user runs it.
  Privileges and delegations are the kernel's decision,
  made when the request
  arrives.
- It starts no `zfs` program and links no OpenZFS library.
  The guest tests use the `zfs` program to build their fixtures,
  which is a
  test's own setup,
  not this crate's work.

## The codec and its origin

`src/nvlist/` is adapted from `Ukko-Ylijumala/zfsdev-rs` at
commit `4559ffc711`,
files `src/nvlist.rs` and `fuzz/fuzz_targets/nvlist.rs`,
under the MIT or Apache-2.0 license.
The MIT and Apache-2.0 texts ship in `LICENSES/`,
and `PROVENANCE.md` records
the revision,
the original paths and every change.
`src/layout/zfs_cmd_t.rs` follows the same project's layout work but mirrors
OpenZFS 2.4.4's union,
where its own copy mirrors 2.2.2.
Nothing is vendored:
this crate links no C library,
loads no CDDL code and
starts no `zfs` program.

## The gate image of step 2

`gate.Containerfile` is the image this package's merge checks run in,
as the plan's "How it merges" names it:
the wrapper's gate base image,
the two lock-row tools (`file-enforcer` and `monochromatic-lint`),
both crates' locked dependencies,
and the browserslist tarballs an offline `file-enforcer check` reads.
Its base is passed as `GATE_BASE_IMAGE`,
the tag the wrapper's resolver prints
(`mise run //package/git-policy/cli:test:git-image`),
never typed in the recipe.
`build-gate-image.ts` is the repository file that assembles the build context
the Containerfile reads
(the two lock-row archives,
checked against `toolchain.lock.json`'s sha256,
and the crate with its fuzz sidecar under `deps/`);
it prints the tag it built:

```text
mise run //package/zfs/request-to-openzfs-kernel-device:build:gate-image -- --tag <tag>
```

The checks run there as `podman run --rm --init --network none --cpus 4
--memory 4g --pids-limit 512`,
with no home folder,
no `/dev/zfs` and the worktree mounted read-only:
`cargo nextest run --release`,
Clippy with `-D warnings`,
`cargo fmt --check` for both crates,
`cargo doc` with `-Dwarnings`,
the sidecar's `cargo check` and Clippy,
`monochromatic-lint package/zfs`
and `file-enforcer check`.
The live tests are in that run and are reported as skipped:
they are
`#[ignore]`d,
and the guest recipe is the only thing that runs them.
The Dylint closure rule (`rust/avoidable-closure`) is **not checked**:
that image has no `cargo dylint`,
so the linter prints its skip for this package as it does for every package.

## Using it

```text
use monochromatic_zfs_request_to_openzfs_kernel_device::ZfsDevice;

let device = ZfsDevice::open()?;                                 // /dev/zfs
device.clone("pool/clones/work", "pool/checkout@origin", None)?; // ZFS_IOC_CLONE
let identity = device.objset_stats("pool/clones/work")?;         // stats + properties
let own = device.snapshot_list_next("pool/clones/work", 0)?;     // None at the end
```

A crate that only builds requests calls `snapshot`,
`clone` and the other
constructors and reads the same fields without a device,
which is what the
device-free tests do.

The device-free tests run through `mise run //package/zfs/request-to-openzfs-kernel-device:test`
on the host,
while this step's gate runs the same commands inside the wrapper's gate container image,
and the bounded fuzz smoke through
`mise run //package/zfs/request-to-openzfs-kernel-device.fuzz:smoke`.
