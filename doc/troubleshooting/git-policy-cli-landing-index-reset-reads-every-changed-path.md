# A landing's index reset reads every path the commit changed

A landing installs the index entry of every path its commit changed,
and it records the worktree's stat data for a path only when the file provably holds the bytes the commit stores
(`package/git-policy/cli/src/native/index/index_reset_worktree_stat.rs`).
The proof hashes the file's raw bytes,
so a landing reads and hashes every changed path once.

This is a cost and not a defect on its own:
all-zero stat data would cost the next commit one hash per path instead
(the same file says so).
What it costs is the point of this record:
on a change list past `MAX_CHANGES` the landing is the slowest part of the commit by far,
and it is measured here because a control whose body lands such a list was failing on it.

## What was measured

`package/git-policy/cli`,
branch `fix/git-policy-cli-issue-6546-awkward-change-list-control-holds-under-load`,
debug build,
2026-11-27 between 04:00 and 05:00 -05:00,
one host under other sessions' builds,
which is why every figure below carries the load average it was taken at.

The control is
`service_landing_state::awkward_change_lists_land_as_native_git_commits_them`
(`src/native/binary/binary_service_landing_state_tests.rs`).
Its fixture holds 4,196 paths,
of which one change list of 4,097 is landed twice through the service and committed once by native Git per case,
in two byte copies of the same repository.

A scratch instrument put a `std::time::Instant` around each arm of `compare`
(`src/native/binary/binary_service_landing_support.rs`) and around the steps of the landing's index update
(`src/native/landing/service_land_follow.rs`,
`src/native/landing/service_land_finish_after_the_move.rs`).
Both changes were reverted,
and the figures are one run each:

```text
# one run of the control alone, load average 123 to 186
SCRATCH PROBE everything staged: the two copies took 503.323919ms
SCRATCH PROBE everything staged: native Git took 595.457563ms
SCRATCH PROBE everything staged: the wrapper took 33.5254866s
SCRATCH PROBE everything staged, with a message from a file: the two copies took 428.925955ms
SCRATCH PROBE everything staged, with a message from a file: native Git took 607.481398ms
SCRATCH PROBE everything staged, with a message from a file: the wrapper took 34.86086076s
```

The same run's breakdown of the wrapper's arm:

```text
SCRATCH BREAKDOWN the conclusion cleanup: 407.959µs
SCRATCH BREAKDOWN make the copy: 754.207µs
SCRATCH BREAKDOWN put the live times back (first): 4.063216ms
SCRATCH BREAKDOWN the stat refresh (Kept): 143.753409ms
SCRATCH BREAKDOWN keep the stat data: 152.322527ms
SCRATCH BREAKDOWN build the index: 30.4584357s
SCRATCH BREAKDOWN install the index: 12.839953ms
```

and of the second:

```text
SCRATCH BREAKDOWN the conclusion cleanup: 530ns
SCRATCH BREAKDOWN make the copy: 246.849µs
SCRATCH BREAKDOWN put the live times back (first): 12.224936ms
SCRATCH BREAKDOWN the stat refresh (Kept): 2.177798635s
SCRATCH BREAKDOWN keep the stat data: 2.241141487s
SCRATCH BREAKDOWN build the index: 32.051642081s
SCRATCH BREAKDOWN install the index: 15.500714ms
```

`build the index` is `index_reset_file::reset_index_file`,
the landing's in-process reset of the change list,
and its two parts that are measured separately here are
`PrivateCopy::make` (0.25 to 0.75 ms) and the stat refresh (0.14 to 2.18 s).
So the reset itself is 30.3 s and 29.8 s of those two 30.5 s and 32.1 s,
and the arms differ by a factor of 56 and 57 for the same command on the same repository state.

The service's own timing line agrees:
`index_update_ms=30471` for the first landing of a run whose `steps=` field names no Git step between
`git update-ref` at 747 ms and `git add` at 30,144 ms,
so the 30 s is the wrapper's own in-process work and no real Git process.

## Why it is the reset

The reset writes the commit's paths into the private index as `git reset --no-refresh` does,
whose stat data is all zeros,
and then records the worktree's stat data for each path whose file proves it holds the commit's bytes
(`index_reset_worktree_stat.rs`,
the five conditions).
The second of those conditions is the file's raw bytes hashing to the entry's object name,
so each of the 4,097 paths is opened,
read and hashed.

4,097 reads are cheap on a quiet disk and are not cheap on this one under load:
the change list cost 7.3 ms a path in that run,
which is the 30 s.
The copies and the fixture are not the cost:
`cp --archive` of the repository took 0.43 to 0.50 s for both copies together,
and in the run of 70.25 s that the first measurement of the control took,
the fixture's own creation,
its 4,196 files written and staged,
took 6.0 s of it,
measured from the service's own `started` line against the run's own duration.

## What was not done, and why

The cheap form is a design change for the owner,
not a repair:

- The live index already holds each staged path's blob and stat data.
  Where its blob equals the commit's blob and the file's `lstat` still matches that stat data,
  in the racy-clean sense Git compares them,
  the entry could keep that stat data and the file would not be read.
  That is the comparison native Git makes everywhere,
  and it is what would make this landing cost one `lstat` per path instead of one read and one hash.
- It changes the stat data the installed index carries,
  which the differential controls compare against Git's own `reset` byte for byte
  (`index_reset_file_tests.rs`),
  and a wrong answer there hides a person's edit from the next commit and from `git status`.
  So it needs its own controls and the owner's decision.
- Nothing of it was attempted in the round that measured it
  (issue 6546),
  whose task was the control's child deadline.

## What the round for issue 6546 did instead

The control's body is dominated by this cost:
two landings of 4,097 paths at 30 s each under load,
against 0.6 s of native Git for each case.
Its child,
which runs the whole body because the fixture root refuses a name that is not UTF-8 on this host,
was killed at `CHILD_TEST_DEADLINE`,
120 s.
That record is in
`doc/troubleshooting/git-policy-cli-controls-that-wait-on-a-guessed-bound-or-a-flush.md`,
"What the awkward change list control's child deadline measured".

A commit that changes thousands of paths is the case a person meets this in too:
a merge that lands a vendor drop,
or `git add --all` after an import,
waits one read of every changed file where native Git waits none.
This record exists so that cost is a named,
measured one and not a control's flake.
