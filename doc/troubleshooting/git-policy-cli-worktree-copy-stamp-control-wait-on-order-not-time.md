# A worktree-copy control measured the time between a write and its look instead of the file's own change time

`a_stamp_comes_only_from_a_clone_of_a_settled_file`
in `package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_tests.rs`
failed once inside a whole host run of `mise run //package/git-policy/cli:native:test:host`
on a branch that changes nothing in that folder,
and passed three times when run alone.

## Symptom

Reproduced on this host (2026-11-08,
the package's own test binary alone in a scope allowed 3% of a processor):

```text
test worktree_copying::worktree_copy_stage::tests::a_stamp_comes_only_from_a_clone_of_a_settled_file ... FAILED

thread 'worktree_copying::worktree_copy_stage::tests::a_stamp_comes_only_from_a_clone_of_a_settled_file' (2756533) panicked at src/native/worktree_copying/worktree_copy_stage_tests.rs:373:5:
assertion `left == right` failed: a file changed this instant is within the timestamp granularity
  left: Some(FileStamp { size: 7, modified: (1794194236, 988248219), changed: (1794194236, 988248219), inode: 407554940 })
 right: None
```

The same test passed at a 1% allowance in the same session,
which is the shape of a wall window rather than a load threshold.

## Root cause

The control wrote a 7-byte source,
copied it straight away,
and asserted the copy carried no stamp,
taking "straight away" to mean within `RECENT_CHANGE_NANOS` (20 ms)
of the file's change time.
`copy_file_exclusive_with` decides that with `stamp_is_settled`:

```rust
let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
    return false;
};
```

So the interval under test was the host's time for two opens,
one clone of 7 bytes and two metadata reads,
against a 20 ms constant.
Nothing bounded that step,
and the control read the clock one time only,
at the end,
so a host that took 30 ms for it turned the assertion into
`left: Some(FileStamp { .. })`,
`right: None`.

## Verification

Version under test:
the package's own test binary at
`target/debug/build/monochromatic-git-policy-cli/<hash>/out/git_policy_cli-<hash>`,
built from this worktree on 2026-11-08.

Before the change,
run as written:

```sh
systemd-run --user --scope -p CPUQuota=3% -p MemoryMax=2G -p TasksMax=256 -p IOWeight=10 -- \
  <test binary> a_stamp_comes_only_from_a_clone_of_a_settled_file --test-threads 1
```

fails with the symptom above about one run in two.
The failing pattern is the first copy of a file just written;
the pattern that passed in every run was the second copy,
40 ms later,
where the 20 ms granularity had been waited out.

## The control now

The clock is the control's:
`clock_reads(nanos)` pins the reading on the control's thread for as long as the returned guard lives,
and `stamp_is_settled` reads it through `clock_nanos`.
Only test builds carry the pinning,
and no pin installed means `SystemTime::now` as before.
The control states three instants against the source's own change time
(`stamp_nanos`):

- the change itself,
  which proves nothing,
- one nanosecond inside `RECENT_CHANGE_NANOS`,
  which proves nothing either,
- a whole `RECENT_CHANGE_NANOS` later,
  which carries the stamp and equals the source's.

No sleep and no elapsed-time bet remain in the control,
and it ends in 0.09 to 0.28 s.

Shown failing,
each plant reverted after its run:

- a `stamp_is_settled` that trusts a recent stamp,
failed in 0.01 s:

```text
thread 'worktree_copying::worktree_copy_stage::tests::a_stamp_comes_only_from_a_clone_of_a_settled_file' (2848674) panicked at src/native/worktree_copying/worktree_copy_stage_tests.rs:384:5:
assertion `left == right` failed: a file whose change time is the clock's own instant is within the timestamp granularity
  left: Some(FileStamp { size: 7, modified: (1794194596, 433271172), changed: (1794194596, 433271172), inode: 407729896 })
 right: None
```

- a `stamp_is_settled` that never trusts a stamp,
failed in 0.01 s with `assertion failed: settled.is_some()`.

After the change,
six runs at the 3% allowance that reproduced it passed.

## What does not work

Waiting longer before the first copy,
which is what the 40 ms sleep before the second copy does.
It moves the window rather than removing it:
the assertion is about the file's change time against the clock,
and any sleep is a bet that the host's next step fits in the slack.

## What is not claimed

- The real clock path is not exercised by this control,
because a control cannot make the wall clock read what it needs.
It is exercised by every other control that copies a file with the real clone request
(`stage_mirrors_selected_entries_with_modes_and_links`,
`a_large_manifest_is_copied_in_slices_and_arrives_whole`).
- `RECENT_CHANGE_NANOS` (20 ms) is unchanged and still has no measurement beside it.
This record measured the control,
  not the constant.

## Upstream filing decision

Nothing to file.
The defect is in this repository's control.
The filesystem's timestamp granularity is what the constant describes,
and the control was wrong to treat it as a deadline for the host.
