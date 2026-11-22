# btrfs `BTRFS_IOC_GET_SUBVOL_INFO` answers 1 on success, and the crate reads any nonzero as an error

Status:
diagnosed by the clone rung's fixtures round of 2026-11-22,
fixed in `c5615e51eb` on 2026-11-22,
unapproved and non-final.
Its guest evidence and the controls that hold it are "The fix" below.

## Symptom

In a btrfs guest,
controls that read a subvolume's own record fail,
and their messages name an ioctl that reported an error for a call the kernel answered:

```text
the facts of the snapshot: PrimitiveError { operation: "BTRFS_IOC_GET_SUBVOL_INFO", path: ".../copy", errno: 22 }
the source is a subvolume
the record of the object its creation made names no object and no source the snapshot was taken from
```

The errno is stale.
A direct call in the same guest,
against the same snapshot,
fills the structure and answers 1:

```text
request=0x81f8943c size=504 result=1 errno=0 (Success) treeid=1256 otransid=459 parent_uuid_set=1
request=0x81f8943c size=504 result=1 errno=0 (Success) treeid=1255 otransid=459 parent_uuid_set=0
request=0x81f8943c size=504 result=0 errno=0 (Success) treeid=5 otransid=0 parent_uuid_set=0
```

The three lines are a snapshot,
the subvolume it was taken from,
and the filesystem's top-level subvolume.

## Root cause

The kernel's `btrfs_ioctl_get_subvol_info` leaves a value from its root-backref search in its result.
It searches the root tree for a `BTRFS_ROOT_BACKREF_KEY` whose offset is zero,
the stored key's offset is the parent subvolume's id,
so `btrfs_search_slot` answers 1 for every subvolume that has a backref,
and the function copies its structure out and returns that 1.
The top-level subvolume skips the search and returns zero.

Read at `fs/btrfs/ioctl.c`,
`btrfs_ioctl_get_subvol_info`:
v6.8 line 2133 and v6.12 line 2227,
where neither `ret` from the search nor the copy is reset to zero on this path.

`package/git-policy/cli/src/native/subvolume/fs_btrfs_ioctl.rs`,
`subvolume_info`,
checks the result against zero:

```rust
let result: libc::c_int = unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_GET_SUBVOL_INFO, &mut arguments) };
if result != 0 {
    return Err(PrimitiveError::last_os_error(operation, path));
}
```

So a successful read of any nested subvolume becomes a `PrimitiveError`,
carrying whatever `errno` the process last set,
which in a nextest worker is often 22 from another call.

btrfs-progs accepts the same answer,
`libbtrfsutil/subvolume.c` line 397 at v6.6.3:

```c
ret = ioctl(fd, BTRFS_IOC_GET_SUBVOL_INFO, &info);
if (ret == -1)
    return BTRFS_UTIL_ERROR_GET_SUBVOL_INFO_FAILED;
```

## Which controls it explains

In a 12 GiB btrfs guest these failed on this branch and on `main` with the messages above;
"The guest evidence" below shows them red on `main` and green on the fix:

- `subvolume::fs_btrfs_ioctl::tests::btrfs_create_snapshot_and_destroy_each_work_once`;
- the four `worktree_copying::worktree_copy_stage_snapshot_origin::tests::` controls,
  through `origin_of_the_source`;
- three `worktree_copying::worktree_copy_stage_claim::tests::` controls,
  through `proved_created_after_the_record`:
  `a_different_snapshot_at_the_stage_name_is_not_taken_for_the_one_a_kill_left`,
  `a_snapshot_holding_a_newer_entry_is_kept`
  and `btrfs_the_snapshot_created_before_its_record_is_removed`.

## The fix

`subvolume_info` reads the answer as a system-call return through `is_a_failure`
(`package/git-policy/cli/src/native/subvolume/fs_btrfs_ioctl.rs`):
a negative answer is a failure and `errno` is read only then,
so an answer of 1 and an answer of 0 both return the structure the kernel filled.
The other wrappers in that file ask for requests whose kernel side answers zero on success,
so this one misread a success:

- `BTRFS_IOC_SUBVOL_CREATE` and `BTRFS_IOC_SNAP_CREATE_V2` answer what `__btrfs_ioctl_snap_create` returns,
  0 or a negative error;
- `BTRFS_IOC_SNAP_DESTROY` and `BTRFS_IOC_SNAP_DESTROY_V2` answer `err` from `btrfs_ioctl_snap_destroy`,
  0 or a negative error;
- `BTRFS_IOC_INO_LOOKUP` with `objectid` 256 returns before any search and otherwise copies only when the answer is 0.

Two controls hold the reading:

```text
subvolume::fs_btrfs_ioctl::tests::a_positive_ioctl_answer_is_not_a_failure
subvolume::fs_btrfs_ioctl::tests::btrfs_a_positive_answer_to_the_subvolume_record_is_a_success
```

The first checks the reader itself on every host,
btrfs or not:
1 and 0 are successes,
`-1` is the failure.
The second makes the call itself for a created subvolume,
requires the wrapper to answer with the record the kernel wrote,
and prints one line saying the answer was zero when the kernel resets it (v7.2 and later).

## Which kernels answer 1

Read from `fs/btrfs/ioctl.c` of each of these:

- v5.6 (the request's first version) line 2623;
- v6.1 line 3005;
- v6.8 line 2133;
- v6.12 line 2160;
- v6.18 line 2061;
- v7.0 line 1948;
- v7.1 line 1948.

`btrfs_ioctl_get_subvol_info` leaves its root-backref search's return value in its result in every one of them
(v6.8 line 2200,
v7.1 line 2015)
and copies the structure out anyway
(v6.8 line 2238,
v7.1 line 2053),
so it answers 1 for a subvolume that has a parent backref and 0 for the top-level subvolume.
v7.2 (line 1957) moved the body into `_btrfs_ioctl_get_subvol_info`,
which resets the value at line 2051,
and its wrapper copies only when the answer is 0,
so from v7.2 every success is 0.
A reader that accepts every non-negative answer serves both.

## The guest evidence

One 12 GiB btrfs guest mounted its loop file at `/mnt/btrfs-fixtures` as `CLI_GIT_TEST_ROOT` and set `user_subvol_rm_allowed`;
its versions were those of the `mvm` Ubuntu image,
kernel 6.8.0-147-generic,
Git 2.43.0
and btrfs-progs 6.6.3.
The driver,
filterset and ioctl probe are in `~/temp/agent/btrfs-subvol-info-success-value-2026-11-22/keep/`.
That kernel answered a created subvolume with `result=1 errno=0 (Success) treeid=256 otransid=8`
and the top-level subvolume with `result=0`:

- on `main` (`8db0b80095`),
  `8 tests run: 0 passed, 8 failed`,
  the eight controls above with the messages quoted here;
- on the fix (`c5615e51eb`),
  `10 tests run: 10 passed`,
  the eight and both controls of "The fix";
- with the fix's source and only `is_a_failure(&result)` written back to `result != 0`,
  `10 tests run: 1 passed, 9 failed`,
  the pass being `a_positive_ioctl_answer_is_not_a_failure` and the new control failing with
  `PrimitiveError { operation: "BTRFS_IOC_GET_SUBVOL_INFO", errno: 22 }`,
  so the control is red exactly when a positive answer is read as an error.

The mount option matters to one of the eight.
`btrfs_create_snapshot_and_destroy_each_work_once` expects `EPERM` from the destroy where the mount lacks `user_subvol_rm_allowed`,
and a root caller destroys the subvolume anyway;
the first fix-arm run failed that control with "the mount option is absent",
the root-caller cause the gate's ZFS-host document records,
before the guest was remounted with the option.

## What it does not explain

The other guest failures of that round are other causes:
controls that assume a caller without `CAP_SYS_ADMIN` while the guest runs them as root,
the guest's descriptor limit,
and one control that waits past nextest's timeout.
