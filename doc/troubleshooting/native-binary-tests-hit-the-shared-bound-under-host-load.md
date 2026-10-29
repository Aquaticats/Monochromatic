# Native binary tests hit the shared 5-second bound under host load

The binary tests of `package/git-policy/cli` run every wrapped command under `timeout --signal=KILL 5`
(`TIME_BOUND_SECONDS` in `src/native/binary_support.rs`).
On a host whose load average was 50 or more, three `discard::` controls failed with
`the command hit the 5-second bound` and passed alone in under a second.

## What was measured

Debug build, Git 2.56.0, fixtures on the `build.build-dir` volume (btrfs on dm-crypt).
Per-command wall times came from 25 reruns of each single test; process starts from `strace --follow-forks`.

- A wrapped explicit-path `commit` starts 40 Git processes and makes 36 sync calls (the exact budget of
  `src/native/binary_budget_tests.rs`), uses 82 to 88 ms of CPU and took 0.15 to 0.45 s of wall time, against 4 ms for
  direct Git. Capped to a tenth of a core it took 0.84 s, so CPU contention alone needs a share under 2 percent to reach
  5 s; the rest of a slow commit is waiting for sync calls. A probe of write, fsync and directory fsync on that volume
  took a median of 12.5 ms, p99 60 ms and a maximum of 139 ms while one suite ran, and 12.6 ms median, 16 ms maximum
  when idle.
- Every other wrapped command of the three tests took at most 0.12 s. The wrapper adds one `rev-parse` (and, for
  `stash`, one `version`) to a forwarded command.
- `git restore --help` makes Git run `man`, which starts `preconv`, `tbl`, `nroff`, `col`, `groff`, `troff` and `grotty`:
  11 process starts for one command. It took 18 ms median, 0.7 s on a cold start and 3.2 s once in 12 runs while the load
  average was 57.
- Not reproduced: a 5 s stall of an ordinary command. Three whole-suite runs under 100, 200 and 800 percent CPU
  caps, with a probe beside one, never showed a non-commit wrapped command of the `discard::`, `dependent_version::`
  and `controls::` tests above 0.07 s, so the bound stays at 5 s for single Git commands (about 70 times that).
- `dependent_version` and `controls` tests: at most 0.07 s and 0.05 s per command (173 process starts for the 50
  commands of `every_control_before_the_subcommand_is_removed`, 3 or 4 per command). No mechanism was found for them.

## Cause and fix

The bound was applied to two things it does not fit: a commit transaction, which the repository already bounds with
`COMMIT_BOUND_SECONDS` (120 s) in `binary_commit_tests.rs`, and a manual-page render that is Git's work and not the
wrapper's. `observe_raw` in `binary_support.rs` now panics, on every host and without reading a clock, when a command
under the 5 s bound is a commit the wrapper lands through its transaction or a `--help` with no `man` stub in the
fixture's `bin` directory. The discard controls use `run_commit` and `stub_manual_viewer`.

## A forwarding test that hit the bound with no commit and no manual page

`forwarding::exit_codes_and_stderr_match_native_git` failed once in a mutation baseline (`native:mutation:scoped`,
`--file src/native/command_push.rs`; log `~/temp/agent/mutpar-w4-command_push.log.2`, run of 2568 tests in the
2 CPU, 2 GiB container while the host load average was above 35) with `the command hit the 5-second bound` at
`binary_support.rs:233` after 5.628 s.
The same log shows the two sibling tests of that module started just before it taking 1.10 s
(`argument_bytes_reach_git_unchanged`) and 2.25 s (`environment_overlay_reaches_git`), against 0.05 s each on the host
when alone: the whole container was 20 to 50 times slower at that moment, not one command.

### What was measured

- The test wraps 12 commands (two `rev-parse`, two `diff`, `show`, `log`, three usage errors, `status --porcelain`,
  `branch --list`) and runs each once more through direct Git. None is a commit and none renders a manual page, so the
  guard in `shared_bound_misuse` does not apply to it.
- `strace --follow-forks` of each wrapped command: at most one Git process started by the wrapper (the `rev-parse` it
  adds) beside the process it becomes, and no `fsync`, `fdatasync`, `flock`, `nanosleep` or `clock_nanosleep` call. The
  usage errors start no `rev-parse` at all. Unlike a commit, there is no sync wait to be slowed by a busy volume.
- Wall time per wrapped command through `timeout --signal=KILL 5`, 300 runs of each of the 12 commands (3600 starts)
  in one fixture while the load average was 28 to 39: minimum 2.7 ms, median 3.7 to 5.9 ms, 99th percentile at most
  9.3 ms, and a maximum of 63 ms (one `rev-parse --verify`), 13 ms for the next largest.
  The direct runs took 0.6 to 1.3 ms.
- The same bound was hit in other baselines by unrelated tests
  (`discard::every_named_alternative_runs_in_the_main_worktree` and
  `discard::the_hatch_after_the_subcommand_runs_each_form` in `mutpar-w5-markdown_linter.log.3`;
  `worktree_copy::an_interrupted_main_checkout_copy_is_recovered_by_the_next_creation` in
  `mutpar-w3-doctor.log.2`), so no command is common to the stalls.

### What was not found

No mechanism that makes one of the 12 commands slow was found, and none is claimed.
A command that normally takes 4 ms needs a stall of more than 1000 times that to reach 5 s; the measurements above show
no work in these commands that can wait on its own (no sync call, sleep, lock or stdin read), so the stall happened in
the container or host under the command (CPU quota, memory pressure or disk reads of an evicted executable are
candidates, none verified).
The test is left on the shared bound: moving it to the commit bound would hide a real forwarding loop, which is what
the short bound is for, and making the wrapper faster has nothing to act on, since the wrapper adds one `rev-parse` and
no syscall that waits.

### What changed

The failure message of `observe_raw` now names the stalled command (its arguments and working directory, built only
when the bound was hit, so a passing run costs nothing).
The next occurrence then says which command stalled; the unnamed `assertion left != right failed` message above is why
this one cannot be attributed.
If a later occurrence names a command with no sync call or sleep again, the next measurement to take is the process
start and file read times inside the container during the stall, not the command.

## What would make the commit itself faster

The commit's 40 starts and 36 syncs are the subject of `doc/planning/cli-git-spawn-and-sync-reduction.md`; the budget
tests ratchet them down as slices land.
