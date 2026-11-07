# `PR_SET_PDEATHSIG` fires when the spawning thread ends, not the process

Note 2026-11-04T22:23:58-05:00: the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders, logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

The binary controls of `package/git-policy/cli` start each fixture's commit service with `PR_SET_PDEATHSIG` set to `SIGKILL`
(`FixtureService::serve` in `src/native/binary_support.rs`), so a control that panics or is killed leaves no service behind.
`lfs_hang::two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other` failed after 60
seconds in gate 26 with "the service never settled the after-commit work in .../repo/.git/cli-git/work", having passed in
twelve earlier gates in under 2.4 seconds.

## What was measured

- Run alone, one after another, in a unit capped at 100 to 200 percent of a CPU: the control failed 3 times in 22 runs
  (run 3, run 12 and run 7 of three series), each after 60 s. The failure message now lists the work directory: both
  items, `push-<oid>` and `maintenance-<oid>`, were in `claimed`, `pending` and `done` were empty, and the two claim file names
  (which carry the holder's last renewal) were unchanged for the whole minute. A live holder renews three times per
  120-second lease, so the holder was gone, not slow.
- The control starts its two commits on two threads. The first wrapped command of the fixture starts the services of every
  worktree, from whichever thread gets there first. `man 2 PR_SET_PDEATHSIG` says the signal is "sent upon subsequent
  termination of the parent thread". When that thread's commit finished first, the kernel killed every service it had
  spawned, including the one the other thread's commit was still waiting on.
- `a_fixture_service_outlives_the_thread_that_started_it` (`src/native/binary_fixture_tests.rs`) starts a service from a
  thread that then ends, and fails on that behavior deterministically (the socket refuses after the thread ends).

## Fix

The services are spawned by one thread that lives as long as the test process (`spawn_for_the_process_lifetime`), so the
signal still fires when the process ends and no longer when a control's worker thread does. Production is not affected: the
real service is started by the user manager and sets no parent-death signal.
