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
worktree_copying::, 496 tests, instrumented         6    0.34 to 0.79 ms    6 of 6
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

## The discarded prune removed a directory whose change time did not move (issue 6544)

`conflict::discarded_prune::guard_tests::a_file_added_after_the_listing_keeps_the_directory`
writes a file into a directory the pass has just listed,
and expects the pass to keep the directory and say why.
Measured on 2026-11-27 on `main`,
the control failed in one run of three of itself and in two runs of three of `conflict::`,
and that run failed with exactly the panic issue 6544 recorded:

```text
thread 'conflict::discarded_prune::guard_tests::a_file_added_after_the_listing_keeps_the_directory' panicked at src/native/conflict/discarded_prune_guard_tests.rs:450:59:
kept: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

A run passes when a tick boundary falls between the listing and the control's write,
which is what the runs that passed show and what the failing runs do not contradict;
I did not instrument the span between the two,
so how often a run lands on one side is a measurement of the runs above,
not of the span.

### What the pass assumed

`remove_entry` (`src/native/conflict/discarded_prune_removal.rs`)
compared the identity it had listed with the identity of the directory it opened,
change time included,
and removed the directory without reading its names whenever the two were equal.
The comment beside it said why a moved change time is not a replacement by itself:
another pass emptying the directory moves it too.
The unstated half was that a change time that did not move means the directory gained nothing,
and a write inside the listing's own tick makes that half false.

### The fix

The pass compares device and inode alone for the identity of the directory,
which is what `remove_empty_directory` already compares,
and reads the names every time:
only a directory that holds nothing the listing did not see is removed,
and a directory that gained a file is kept and reported as `EntryChanged`.
No clock is read on the path that decides a removal.

Repeats with the change in place:

```text
the control alone, 3 runs     1 passed; 0 failed each
the prune module, 3 runs      32 passed; 0 failed each
```

A second control pins the decision without racing a clock:
`a_directory_whose_change_time_did_not_move_is_judged_by_what_it_holds`
hands the removal the identity the directory has now,
change time included,
and a listing that never named the file.
Taken with the change out,
it failed `0 passed; 1 failed` on the read of the added file,
and with the change in it passed.

### What it costs

The added look is one further read of a removed directory's names,
and of every directory below it:
two more `getdents64` calls for each directory level,
and one more `newfstatat` for each name it holds.
Measured with `strace -c` on one control that removes two one-file directories in a single pass
(`discarded_prune::tests::a_pass_removes_the_chosen_directories_and_audits_each`),
before the fix and after it,
one run each twice,
the same counts in both runs of each arm:

```text
               getdents64  newfstatat  unlinkat  fstat  total
before the fix         35          14        12     27     88
after the fix          39          16        12     29     96
```

So these system calls rise by a ratio of 1.09 (96 to 88),
for two removed directories that hold one file each,
and a removed directory costs one further names read.
Wall time is not where that shows.
Five pairs of the prune module,
before and after,
interleaved in one round,
gave the ratios below,
with the runs themselves spread from 0.13 s to 1.05 s,
so the added look is inside this host's spread of the same controls:

```text
pair   before   after   ratio
1      0.51 s   1.05 s  2.06
2      0.41 s   0.32 s  0.78
3      0.25 s   0.21 s  0.84
4      0.20 s   0.13 s  0.65
5      0.24 s   0.30 s  1.25
```

## The snapshot index control kept two change times one tick can make equal (issue 6593)

`worktree::worktree_snapshot_index_step::tests::the_index_a_source_with_changes_installs_keeps_the_fields_that_prove_an_edit`
failed once on 2026-11-27,
in a threaded run of `worktree::` on `main` at `1c8acd5bfb`,
beside two controls of another incident,
and passed in the same run after that round's fix,
which did not touch it
(`3309 passed; 3 failed`).
The failure's own lines,
from that round's log
(`~/temp/agent/issue-6582-two-worktree-copying-controls-hold-in-a-wide-threaded-run-scratch/keep/red-wide-run-main.log`,
which expires with that round's scratch):

```text
---- worktree::worktree_snapshot_index_step::tests::the_index_a_source_with_changes_installs_keeps_the_fields_that_prove_an_edit stdout ----

thread 'worktree::worktree_snapshot_index_step::tests::the_index_a_source_with_changes_installs_keeps_the_fields_that_prove_an_edit' (1575770) panicked at src/native/worktree/worktree_snapshot_index_step_tests.rs:158:5:
the installed index records the edited file's own stat fields (ctime 1795784500:2034409, recorded 1795784500:2034409), so Git takes the edit for the entry's content
```

### The two times the control compared

The control installs a source's own index,
whose entry for `a.txt` a `git add` recorded before the fixture replaced that
file's bytes with an edit of the same length.
It then asserts that the installed entry's stat fields do not match the file's own.
It compared the file's change time (`MetadataExt::ctime` and `ctime_nsec`) with the change time the entry recorded,
which was the one `git add` read from that file before the edit.

Every other compared field is equal by the fixture's own design:
the edit keeps the length,
the inode,
the owner and the second of the modification time,
and `hide_an_edit` retries until the change time's **second** agrees too,
which is what hides the edit from `git diff-files`.
That left the change time's nanosecond count as the one field carrying the assertion.

### What granularity the file system keeps, and what makes two of them equal

Measured on 2026-11-27,
kernel `7.3.5-2-cachyos` with `CONFIG_HZ=1000`,
OpenZFS `2.4.4-1`,
with probes of this round on the dataset the fixtures use
(`~/temp/agent/issue-6593-snapshot-index-step-change-time-control-holds-under-load-scratch/keep/`,
whose logs are `measurement-two-writes-one-change-time.log`,
`measurement-coarse-clock-lag.log` and `measurement-record-to-edit-gap.log`,
and which expire with that round's scratch):

```text
two writes of one file, 30 pairs at each gap (ctime-pair-gap-probe.py):
gap_us trials equal_ctime  ctime_deltas_ns
     0     30          29  [0, 999997]
    50     30          29  [0, 999997]
   100     30          26  [0, 999996, 999997]
   200     30          24  [0, 999996, 999997]
   400     30          17  [0, 999996, 999997, 11999960]
   600     30          12  [0, 999996, 999997]
   800     30           6  [0, 999996, 999997]
  1000     30           0  [999996, 999997, 1999993, 47999839]
  1200     30           0  [999996, 999997, 1999993, 48999837]
  2000     30           0  [1999993, 1999994, 2999990, 50999830]
  4000     30           0  [3999986, 3999987, 50999830, 53999819]

the coarse clock's lag behind CLOCK_REALTIME, 4,176,020 samples over 20 s
(coarse-clock-lag-long-probe.py):
  min 0.15 ms  median 0.69 ms  p99 1.18 ms  p99.99 1.22 ms
```

The delta column lists every step observed at that gap,
and the few large ones
(11.9 ms,
47.9 ms and their kind)
are samples whose reading thread was descheduled between the two writes,
not the clock:
the lag figures come from a fine-clock read bracketed around the coarse one,
and the 37 samples above 5 ms all had a reading window of about 50 ms,
so the largest lag over a window this thread did not spend descheduled is the 1.22 ms of the p99.99.

So the granule is one step of about a millisecond,
and the coarse clock is never more than about 1.3 ms behind the fine clock:
two change-time-setting events share a change time exactly when they fall in one step,
which is what the failing assertion's equality means.
The step itself drifts a few nanoseconds from a whole millisecond,
which is the kernel's adjusted tick,
not a limit of how the time is stored.

### Which pair the fixture left

The fixture sets a change time twice in each attempt:
the `utimes` before its `git add`,
and the write and `utimes` after it,
so the entry's recorded change time and the file's are separated by the whole `git add`.
96 rounds of that sequence,
sixteen at a time in one process
(`hidden-edit-threads-probe.py`),
left:

```text
rounds 96 equal_pairs 0 edits_git_could_see 0
record to edit gap ns: min 13999953 median 42999856 max 108999635
git add us: min 14715.2 median 42338.1 max 108374.7
```

Both figures are an order of magnitude outside one step,
so the fixture's own two change-time-setting writes cannot be the pair that came out equal:
the record the failing run's installed index carried was one whose change time was the file's own.
The same measurement found the one way this fixture's record can be older than the add it seems to stand for:
`git add` does not rewrite an entry it judges unchanged,
which left the recorded change time where it was in 58 of 60 adds of a loop that wrote the file and re-added
it inside one second
(`hidden-edit-tick-probe.py`);
in the fixture as it stands the wait for a new second makes every add rewrite its entry,
measured 6 of 6.

Neither the failing run's journal nor its log shows what refreshed that entry,
and this round did not reproduce the equality by running the control:
what it can say is that the assertion rested on a clock field the file system may leave equal,
with no fixture-side guarantee that it differ,
and that the fixture's own sequence cannot make the pair differ by construction.

### The fix

`hide_an_edit` now states the difference instead of leaving it to the clock:
the entry records a modification time whose nanosecond part is `HIDDEN_EDIT_RECORDED_NANOS`
(250 ms),
and the edit gives the file `HIDDEN_EDIT_EDITED_NANOS`
(750 ms),
both inside `HIDDEN_EDIT_SECONDS`
(`worktree_ladder_test_support.rs`).
Git's own stat check compares the second of the modification time and not its nanoseconds,
so the edit is still one that check cannot see
(measured with this host's Git 2.56.1:
a file whose modification time differs only after the decimal point is still reported unchanged).

The control now asserts two things,
both of which the fixture decides:
the installed entry's stored fields equal the source index's entry for that path,
which is a field comparison and reads no clock,
and `matches_stored` does not match the file's own fields,
which it cannot because the two modification-time nanosecond parts differ by the fixture's choice.
The failure message names the modification time's nanoseconds rather than the change time,
since that is the field that now carries the difference.

### Red and green

Planted first,
on the code and the fixture as they stood,
with the index's record doctored to carry the change time the file had after the edit,
which is what two writes inside one step leave
(`worktree_snapshot_index_step_tests.rs`,
the plant reverted after its run):

```text
the installed index records the edited file's own stat fields (ctime 1795788673:3675360, recorded 1795788673:3675360), so Git takes the edit for the entry's content
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 10794 filtered out; finished in 0.56s
```

With the fix in place and the same plant,
`cargo test --locked --offline --lib --features test-only-switches worktree::worktree_snapshot_index_step::tests -- --skip guest --skip _arm_tests`
answered:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10794 filtered out; finished in 0.61s
```

And the guard the control is about,
shown failing (its plant reverted after its run):
`install_index` was made to hand the install the stage's own stat fields for every entry,
with the rewrite's `same_time` guard removed,
and the control failed naming the change time the refresh took from the file:

```text
assertion `left == right` failed: the installed index keeps every stat field the source's own index recorded, so no field was taken from the file
  left: StoredStat { ctime_seconds: 1795789514, ctime_nanos: 6863498, ... }
 right: StoredStat { ctime_seconds: 1795789514, ctime_nanos: 1863514, ... }
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 10794 filtered out; finished in 0.91s
```

With both plants reverted the module answered `test result: ok. 2 passed; 0 failed`.

### The same dependency elsewhere

The fixture's stated nanosecond parts also settle the edit for the controls that restore a hidden edit:
`worktree::worktree_dirty_restore::tests::a_restore_replaces_an_edit_that_keeps_its_files_size_and_times`
expected the walk to settle that file by content,
which before this change rested on the same change-time nanoseconds collapsing into one step.
The controls of this package whose assertions rest on two file times differing,
and what each rests on now,
are listed in `package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
"Every wait has a deadline".

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
