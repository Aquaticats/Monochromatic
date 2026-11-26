# OpenZFS 2.4.4 on this host stamps a write once per 1 ms tick, so a control that writes inside the stamp's own tick cannot tell the write from it

The control `worktree_copy_compare::tests::a_source_written_after_the_stamp_fails_the_final_check`
(`package/git-policy/cli/src/native/worktree_copying/worktree_copy_compare_tests.rs`)
writes the source file again after its stamp was recorded and expects the final check to fail.
It answered `Ok` now and then in threaded runs of that package's tests.
The control's claim was right and its precondition was not:
this host's ZFS rounds a file's change and modification times to the kernel's coarse clock,
which advances once per tick,
and the control's write landed inside the tick its stamp came from.

Measured on 2026-11-26 (issue 6520),
kernel `7.3.5-2-cachyos` with `CONFIG_HZ=1000`,
OpenZFS `2.4.4-1`.

## Symptom

In threaded runs of

```sh
CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1 cargo test --locked --offline --lib --features test-only-switches \
  worktree_copying:: -- --skip guest --skip _arm_tests
```

the control failed once on a branch and once in two runs of `main` on 2026-11-26,
and passed in the third:

```text
test worktree_copying::worktree_copy_compare::tests::a_source_written_after_the_stamp_fails_the_final_check ... FAILED
thread '...' panicked at src/native/worktree_copying/worktree_copy_compare_tests.rs:389:6:
test result: FAILED. 495 passed; 1 failed; 0 ignored; 0 measured; 10230 filtered out; finished in 1.35s
```

Line 389,
in the file as it stood then,
was `.expect_err("written")` on the second final check:
the check answered `Ok` after the write,
so the source's stamp still matched the one recorded before it.

## What the control assumed

It writes five bytes over five bytes,
so the size cannot tell the two states apart,
and it relies on the file's times:

```rust
// Same size, so only the times can tell; the change time moves on every write.
std::fs::write(source.join("cache/file"), b"other").expect("write");
```

That holds only when the write lands in a later tick than the stamp.
The control recorded the stamp,
ran the first final check,
and wrote 0.34 to 0.79 ms later in the failing runs,
which is shorter than one tick.

## Root cause

A write's stamp in ZFS is taken from the kernel's coarse real-time clock:

- `module/zfs/zfs_vnops.c`,
  inside `zfs_write` once the bytes are accepted:
  [`zfs_tstamp_update_setup(zp, CONTENT_MODIFIED, mtime, ctime);`][vnops-stamp]
- `module/os/linux/zfs/zfs_znode_os.c`,
  the definition of that helper,
  which reads the clock once
  for both times:
  `zfs_tstamp_update_setup(znode_t *zp, uint_t flag, uint64_t mtime[2], uint64_t ctime[2])`
  calls [`gethrestime(&now);`][znode-stamp],
  and the two times are encoded from it below
  (`zpl_inode_set_ctime_to_ts`).
- `include/os/linux/spl/sys/time.h`,
  where that name is the coarse clock:
  [`gethrestime(inode_timespec_t *ts) { ktime_get_coarse_real_ts64(ts); }`][spl-time]

`ktime_get_coarse_real_ts64` answers the tick the clock last struck,
not the instant of the call,
so two writes in one tick carry the same change and modification times.
This host runs `CONFIG_HZ=1000`,
one tick per millisecond.

Measured with a probe program (98 lines,
not in the tree) that writes,
reads the stamp,
writes again and reads it again,
with the four fields `stamp_of` compares taken through `std::os::unix::fs::MetadataExt`:

```text
consecutive stamps of one file, one write and one read apart:
  (1795730696, 292136248)
  (1795730696, 293136245)
  (1795730696, 294136242)
  (1795730696, 295136239)
bare write pair, two writes back to back, 1000 rounds on the dataset the fixtures use:
  983 rounds left the stamp identical, and the longest identical write took 358.7 us
control-shaped cycle, about 15 us between the stamp and the write, 300 rounds:
  274 rounds left the stamp identical
```

Every step is exactly 1,000,000 ns with a few microseconds of drift,
which is the tick,
not a limit of how the time is stored.

What the control leaves between recording its stamp and writing is what decides it,
and that span depends on how much its neighbours have warmed the process's runtime:

```text
run                                              runs   span of the write   stamp identical
worktree_copying::, 496 tests threaded              6    0.34 to 0.79 ms    6 of 6
worktree_copying::worktree_copy_compare::, 14      30    1.83 to 2.62 ms    0 of 30
the control alone, --exact                          1    2.65 ms            0 of 1
```

In the narrow module and alone,
the first final check builds the process's lazy runtime,
which costs about 2 ms and takes the write out of the stamp's tick.
Beside the other 495 tests the runtime is already built,
the two manifest walks of the first check take a few hundred microseconds,
and the write stays in the stamp's tick.

## Reproduction

The failing shape is one module of the package's test binary,
threaded,
run more than once:

```sh
CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1 <test binary> worktree_copying::
```

Four runs of that shape on `main`'s own file,
with `origin/main` merged into the branch,
failed three times with `496 passed; 1 failed` and passed once with `497 passed; 0 failed`.
Six earlier runs,
before the merge and with a temporary `eprintln!` in the control
(run with `--nocapture`,
printing the recorded stamp,
the stamp after the write and the span),
failed six times,
`495 passed; 1 failed` each,
in 1.7 to 1.8 s per run.
That instrument is not in the tree.

The granule alone needs no package code:
write a file,
read its size,
times and inode number,
write it again,
and read them again,
repeated.
Count the rounds where the four are equal.
The probe above does that,
plus the control's own sequence,
and takes about 20 s for 300 rounds.

## The controls now

A control in this package states the file times it needs instead of waiting for a clock to move,
which is what `RECENT_CHANGE_NANOS` already says for the copy's own trust decision
(`worktree_copy_stage_leaf_copying.rs`,
"How recent a change time may be and still be too close to the clock").

`stamped_pair` now calls `settle_the_modification_time` before it takes the stamp,
which gives the fixture source a modification time `RECENT_CHANGE_NANOS` (20 ms) before now.
The control's write then lands a whole margin after the stamp whatever the host rounds a write's time to,
so the difference the control is about is there on every run.
The margin is stated on the modification time rather than the change time
because no caller can set a change time,
and a write moves both.

Repeats with the change in place:

```text
worktree_copying::, 497 tests threaded   10 runs   497 passed; 0 failed each
worktree_copying::worktree_copy_compare:: 30 runs  14 passed; 0 failed each
```

Shown failing,
each plant reverted after its run:

- the call to `settle_the_modification_time` taken out of `stamped_pair`:
  4 of 4 runs of `worktree_copying::` failed,
  `495 passed; 1 failed` each,
  the control above;
- the stamp comparison in `entry_matches_stamped` replaced with `Ok(true)` for a stamped entry:
  3 of 3 runs of `worktree_copying::worktree_copy_compare::` failed,
  two tests each,
  `a_source_written_after_the_stamp_fails_the_final_check` among them,
  which is what keeps the control's own claim from going hollow.

## What is not claimed

- Only ZFS was measured,
  on two datasets of this host's pool:
  `/home` and the build directory under it.
  A filesystem that stamps from the fine clock has a nanosecond granule.
- The coarse clock's resolution is taken from the measurement and from `CONFIG_HZ=1000`,
  not read from the kernel's implementation:
  this host's kernel source tree (`/usr/src/linux-cachyos`) carries headers and configuration only,
  no `fs/inode.c`.
- `RECENT_CHANGE_NANOS` is 20 ms by this crate's own decision,
  not by a measurement in this record.
  It is 20 ticks of this host's clock,
  and the copy trusts it for the same question.
- A filesystem that rounded write times to a whole second would still put the control's write inside the stamp's
  tick,
  and the margin above would not cover it.
- A different control of the same module,
  `the_source_walk_takes_a_place_of_the_processs_bound`,
  failed 3 times in 60 runs of the 14 test module
  on 2026-11-26,
  before and after this change,
  and passed 30 of 30 runs after the merge.
  It holds the process's one blocking-pool place and waits `PATIENCE` for the source walk to ask for it.
  That is its own incident,
  and this record does not explain it.

## Upstream filing decision

Nothing to file.
Filesystem timestamps have a granularity by design,
and this crate's own `RECENT_CHANGE_NANOS` exists because of it.
A filesystem reading the coarse clock is a choice,
not a defect that a report could move,
and the defect here was in this repository's control.

[vnops-stamp]: https://github.com/openzfs/zfs/blob/zfs-2.4.4/module/zfs/zfs_vnops.c#L1013
[znode-stamp]: https://github.com/openzfs/zfs/blob/zfs-2.4.4/module/os/linux/zfs/zfs_znode_os.c#L1420-L1425
[spl-time]: https://github.com/openzfs/zfs/blob/zfs-2.4.4/include/os/linux/spl/sys/time.h#L69-L72
