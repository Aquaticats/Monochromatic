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
The same buffer layout serves every release,
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

Every failure returns the operation,
the dataset or pool it named,
and the
kernel's error number,
spelled out the way `zfs` prints it.
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

## What this crate does not do

- It sends no request by itself.
  Step 2 of the implementation plan adds the live calls that open `/dev/zfs`
  and carry a packed request there.
- It checks no kernel compatibility.
  The start-time gate that compares the loaded module's release,
  architecture,
  endianness and BTF layout is step 6.
- It never retries a mutating request.
  A reply buffer grows only for reads;
  a write whose outcome is unknown is
  reported,
  never replayed.
- It does not care which machine or user runs it.
  Privileges and delegations are the kernel's decision,
  made when the request
  arrives.

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

## Using it

```rust
use monochromatic_zfs_request_to_openzfs_kernel_device::{snapshot, NvList};

let mut request = snapshot("pool", &["pool/dataset@before-merge"], None)?;
// Step 2 carries `request.command_abi_address()` to `libc::ioctl`; the packed
// input list and the reply buffer live inside `request` for that call.
let reply = request.take_reply()?;
```

The device-free tests run through `mise run //package/zfs/request-to-openzfs-kernel-device:test`
inside the wrapper's gate container image,
and the bounded fuzz smoke through
`mise run //package/zfs/request-to-openzfs-kernel-device.fuzz:smoke`.
