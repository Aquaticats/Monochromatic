# A control that needs strace fails without it unless the run acknowledges the absence

Decided 2026-11-25 (question asked about 14:50-05:00), for `package/git-policy/cli`'s tests.

## The question

When `strace` is missing on the machine running the wrapper's tests,
what does a control that needs it do?
Before this answer the lock controls in `src/native/lock/git_stop_lock_tests.rs`
printed `STRACE CONTROL NOT RUN: <control> (no strace on PATH)` and passed
unless the run set `CLI_GIT_REQUIRE_STRACE`,
which only the gate image set (the git-lfs shape, `CLI_GIT_REQUIRE_REAL_GIT_LFS`);
other controls that run commands under `strace --inject` reported a skip and checked nothing
(package `README.md` near line 6379).

## Answer

"Fail unless acknowledged (Recommended)".

So every control that needs `strace` follows the btrfs and ZFS shape
(`CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED`, `CLI_GIT_ZFS_ABSENT_ACKNOWLEDGED`, package `README.md` near line 6835):
on a run without `strace` it fails,
and it prints a line naming itself and passes only when the run states the absence is known,
through one acknowledgement variable;
`CLI_GIT_REQUIRE_STRACE` goes once nothing needs it.

## Not chosen

- "Skip unless required": every strace control on the git-lfs shape;
  a host run without `strace` stays green while checking nothing unless the log is read.
- "Leave as is": two behaviours stay, and the silent skips cannot be told from a pass.

`CLI_GIT_REQUIRE_REAL_GIT_LFS` keeps the other shape;
the question did not cover it.
