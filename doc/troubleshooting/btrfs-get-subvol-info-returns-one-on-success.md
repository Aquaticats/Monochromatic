# btrfs `BTRFS_IOC_GET_SUBVOL_INFO` answers 1 on success, and the crate reads any nonzero as an error

Status:
diagnosed by the clone rung's fixtures round,
whose first record is `cf6708e424` of 2026-11-22T01:32:52-05:00,
fixed in `c5615e51eb` of 2026-11-22T02:07:45-05:00,
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
so `btrfs_search_slot` answers 1 for a subvolume that has a backref,
and the function copies its structure out and returns that value.
A search that ends past the last item of a leaf is followed by `btrfs_next_leaf`,
whose answer of 0 stands in the result and is returned with the structure filled the same way,
so a subvolume that has a backref answers 1,
or 0 when its search ends at that boundary.
The top-level subvolume skips the search and returns zero.

Read at `fs/btrfs/ioctl.c`,
`btrfs_ioctl_get_subvol_info`:
v6.8 function line 2133 and search line 2200,
with the leaf-boundary call at line 2205,
and v6.12 function line 2160 and search line 2227,
where neither `ret` from the search nor the copy is reset to zero on this path.

`package/git-policy/cli/src/native/subvolume/fs_btrfs_ioctl.rs`,
`subvolume_info`,
checked the result against zero:

```rust
let result: libc::c_int = unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_GET_SUBVOL_INFO, &mut arguments) };
if result != 0 {
    return Err(PrimitiveError::last_os_error(operation, path));
}
```

So a successful read of any nested subvolume became a `PrimitiveError`,
carrying whatever `errno` the process last set,
which in a nextest worker is often 22 from another call.

btrfs-progs accepts the same answer,
`libbtrfsutil/subvolume.c` lines 396 and 397 at v6.6.3:

```c
ret = ioctl(fd, BTRFS_IOC_GET_SUBVOL_INFO, &info);
if (ret == -1)
    return BTRFS_UTIL_ERROR_GET_SUBVOL_INFO_FAILED;
```

## Which controls it explains

In a 12 GiB btrfs guest these failed on `main` and on the clone rung's branch,
with the messages above;
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

`subvolume_info` hands the answer to one function,
`record_from_answer`
(`package/git-policy/cli/src/native/subvolume/fs_btrfs_ioctl.rs`),
which reads it through `is_a_failure`:
a negative answer is a failure and `errno` is read only then,
so an answer of 1 and an answer of 0 both return the structure the kernel filled.
The call site holds no answer of its own to compare,
so the one place that decides is the one place a control reaches on any filesystem.

The other wrappers in that file ask for requests whose kernel side answers zero on success,
so this one misread a success.
Lines read here at `fs/btrfs/ioctl.c` v6.8:

- `BTRFS_IOC_SUBVOL_CREATE` and `BTRFS_IOC_SNAP_CREATE_V2` answer what
  `__btrfs_ioctl_snap_create` (line 1246) returns,
  which is `btrfs_mksubvol`'s answer (line 957),
  returned at line 1010,
  0 or a negative error;
- `BTRFS_IOC_SNAP_DESTROY` and `BTRFS_IOC_SNAP_DESTROY_V2` answer `err` from
  `btrfs_ioctl_snap_destroy` (line 2344),
  0 or a negative error;
- `BTRFS_IOC_INO_LOOKUP` with `objectid` 256 returns before any search
  (line 2067 jumps to `out` with `ret` 0)
  and otherwise copies only when the answer is 0 (line 2082).

`FICLONE` answers 0 on success for the same reason:
`ioctl_file_clone` in `fs/ioctl.c` v6.8 (line 231) sets the answer to 0 at line 247
where the cloned length matches the request.

The controls that hold the reading:

```text
subvolume::fs_btrfs_ioctl::tests::a_positive_ioctl_answer_is_not_a_failure
subvolume::fs_btrfs_ioctl::tests::answers_of_one_and_zero_return_the_subvolume_record_and_a_negative_answer_fails
subvolume::fs_btrfs_ioctl::tests::btrfs_a_positive_answer_to_the_subvolume_record_is_a_success
subvolume::fs_btrfs_ioctl::tests::btrfs_the_top_level_subvolumes_answer_of_zero_is_read_as_a_record
```

`a_positive_ioctl_answer_is_not_a_failure` checks the reader itself on every host,
btrfs or not:
1 and 0 are successes,
`-1` is the failure.
`answers_of_one_and_zero_return_the_subvolume_record_and_a_negative_answer_fails`
drives the whole reading,
`record_from_answer`,
with an answer of 1 and an answer of 0 and an answer of -1,
on any filesystem,
and requires the record the structure held,
so a comparison against zero written back where the answer arrives fails it on every host,
including a host with no btrfs fixture root and a kernel from v7.2 whose every success is 0.
`btrfs_a_positive_answer_to_the_subvolume_record_is_a_success` makes the call itself for a created subvolume,
requires the wrapper to answer with the record the kernel wrote,
and says in one line when the kernel answered zero,
which happens on v7.2 and later and when the backref search ends at a leaf boundary.
`btrfs_the_top_level_subvolumes_answer_of_zero_is_read_as_a_record`
reads the fixture root's own record,
whose answer is the top-level subvolume's 0
("What stays unchecked" says where it has run).

## Which kernels answer 1

Read from `fs/btrfs/ioctl.c` of each of these,
as the function line,
the root-backref search line
and the `copy_to_user` line:

- v4.18 (the request's first version):
  function 2593,
  search 2666,
  copy 2702;
- v5.5 and v5.6:
  function 2623,
  search 2696,
  copy 2732;
- v6.1:
  function 3005,
  search 3072,
  copy 3110;
- v6.8:
  function 2133,
  search 2200,
  copy 2238;
- v6.12:
  function 2160,
  search 2227,
  copy 2265;
- v6.18:
  function 2061,
  search 2128,
  copy 2166;
- v7.0 and v7.1:
  function 1948,
  search 2015,
  copy 2053.

v4.17 holds no `btrfs_ioctl_get_subvol_info` at all.
`btrfs_ioctl_get_subvol_info` leaves its root-backref search's return value in its result in every version above
and copies the structure out anyway,
so it answers 1 for a subvolume whose search ends inside a leaf,
0 when that search ends at a leaf boundary
(v6.8 line 2205 overwrites the 1 with `btrfs_next_leaf`'s 0),
and 0 for the top-level subvolume.
v7.2 (line 1957) moved the body into `_btrfs_ioctl_get_subvol_info`,
which resets the value at line 2051,
and its wrapper copies only when the answer is 0,
so from v7.2 every success is 0.
A reader that accepts every non-negative answer serves both.

## The guest evidence

One 12 GiB btrfs guest mounted its loop file at `/mnt/btrfs-fixtures` as `CLI_GIT_TEST_ROOT`
and set `user_subvol_rm_allowed`;
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
  the eight plus `a_positive_ioctl_answer_is_not_a_failure`
  and `btrfs_a_positive_answer_to_the_subvolume_record_is_a_success`;
- with the fix's source and only the call site `is_a_failure(&result)` written back to `result != 0`,
  `10 tests run: 1 passed, 9 failed`,
  the pass being `a_positive_ioctl_answer_is_not_a_failure`
  and `btrfs_a_positive_answer_to_the_subvolume_record_is_a_success` failing with
  `PrimitiveError { operation: "BTRFS_IOC_GET_SUBVOL_INFO", errno: 22 }`,
  so the btrfs control is red exactly when a positive answer is read as an error,
  and the reader control stays green because it never saw that call site.

That write-back was the call site as the fix had it then,
and the reader control did not see it.
In the shape this document now describes the call site holds no answer,
so the comparison is written back where the answer arrives:
on a ZFS host with no btrfs fixture root,
`answered != 0` in `record_from_answer`
leaves `a_positive_ioctl_answer_is_not_a_failure` green and fails
`answers_of_one_and_zero_return_the_subvolume_record_and_a_negative_answer_fails`,
so the reading is held on a host where no ioctl answers 1 either.

The mount option matters to one of the eight.
`btrfs_create_snapshot_and_destroy_each_work_once` expects `EPERM` from the destroy
where the mount lacks `user_subvol_rm_allowed`,
and a root caller destroys the subvolume anyway;
the first fix-arm run failed that control with "the mount option is absent",
the root-caller cause the gate's ZFS-host document records,
before the guest was remounted with the option.

## What stays unchecked

- A kernel from v7.2 or later:
  the reset at v7.2 line 2051 and the wrapper's `if (!ret && copy_to_user(...))` at line 2119
  are read from the source,
  not run;
  no guest of that kernel exists here.
- A file system other than btrfs that answers this request number with a positive value:
  `subvolume_info` reads every non-negative answer as a filled record,
  so such a driver would give a zeroed record where the earlier code gave an error.
  No such driver is known,
  and none was searched for.
- Whether the crate's controls reach the top-level subvolume's 0 answer:
  `answers_of_one_and_zero_return_the_subvolume_record_and_a_negative_answer_fails`
  drives the reading with 0 on every host,
  so the reading of 0 is held;
  that a real top-level subvolume answers 0,
  and that the wrapper reads it,
  is held only by `btrfs_the_top_level_subvolumes_answer_of_zero_is_read_as_a_record`,
  which needs a btrfs fixture root.
  This ZFS host has none and the container gate's fixture root is a tmpfs,
  so that control did not run in either,
  and its own reading is unverified until one does.
- The top-level subvolume's `uuid` being all zero,
  which a source at the top level would make the record's parent UUID:
  the path is unchanged from `main` and no control reads it.
- Non-Linux builds,
  where the module is `cfg(target_os = "linux")`.

## What it does not explain

The other guest failures of that round are other causes:
controls that assume a caller without `CAP_SYS_ADMIN` while the guest runs them as root,
the guest's descriptor limit,
and one control that waits past nextest's timeout.
