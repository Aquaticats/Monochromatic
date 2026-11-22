# btrfs `BTRFS_IOC_GET_SUBVOL_INFO` answers 1 on success, and the crate reads any nonzero as an error

Status:
diagnosis by the clone rung's fixtures round of 2026-11-22,
unapproved and non-final.
Not fixed here:
the wrapper source it names is unchanged by that round and belongs to the package that owns it.

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

In a 12 GiB btrfs guest,
these fail on this branch and on `main` with the messages above:

- `subvolume::fs_btrfs_ioctl::tests::btrfs_create_snapshot_and_destroy_each_work_once`;
- the four `worktree_copying::worktree_copy_stage_snapshot_origin::tests::` controls,
  through `origin_of_the_source`;
- `worktree_copying::worktree_copy_stage_claim::tests::a_different_snapshot_at_the_stage_name_is_not_taken_for_the_one_a_kill_left`
  and `worktree_copying::worktree_copy_stage_claim::tests::btrfs_the_snapshot_created_before_its_record_is_removed`,
  through `proved_created_after_the_record`.

## What fixes it

Read the result as a system-call return rather than as a status:
`result < 0` is the error,
and the value a successful request answers is not part of the contract.
Every other wrapper in that file asks for a request whose kernel side returns zero,
so only `subvolume_info` misreads one.

## What it does not explain

The other guest failures of that round are other causes:
controls that assume a caller without `CAP_SYS_ADMIN` while the guest runs them as root,
the guest's descriptor limit,
and one control that waits past nextest's timeout.
