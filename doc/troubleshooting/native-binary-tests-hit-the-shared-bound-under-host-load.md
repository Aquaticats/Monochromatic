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

## What would make the commit itself faster

The commit's 40 starts and 36 syncs are the subject of `doc/planning/cli-git-spawn-and-sync-reduction.md`; the budget
tests ratchet them down as slices land.
