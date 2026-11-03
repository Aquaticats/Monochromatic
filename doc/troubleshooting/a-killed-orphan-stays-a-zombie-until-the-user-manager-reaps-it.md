# A killed orphan stays a zombie until the user manager reaps it

`child_watch::tests::a_command_past_the_bound_is_stopped_with_the_processes_it_started` in `package/git-policy/cli` failed on
the host with "the helper the step started is still running after the step was stopped",
and passed in every container gate.

## What was measured

- Run alone in a capped `systemd-run --user --scope`,
  one after another:
  1 failure in 25 runs,
  and in a later series the
  old check failed again in the first full run.
  The test was changed to print the helper's `/proc/<pid>/status` at the
  failure:
  `Name: sleep`,
  `State: Z (zombie)`,
  `PPid: 5698`.
  Process 5698 is `/usr/lib/systemd/systemd --user`.
- So the helper was killed.
  The step's `sh` died of the bound and the helper,
  an orphan,
  was adopted by the user's service
  manager,
  which is a subreaper;
  the manager reaps it a moment after the signal lands.
  Until then the process table keeps
  the zombie,
  and `kill(pid, 0)` succeeds for a zombie,
  so the check that used it read a dead helper as a running one.
- In the container gates the adopter is the init that `podman run --init` starts,
  which reaps at once,
  so the window is
  too narrow to fall into.
- The product (`stop_process` in `src/native/child_watch.rs`,
  which signals the child and every process below it) was not at
  fault:
  with the descendants' kill removed,
  the test fails for the right reason (the helper runs for its 25 seconds).

## Fix

The test asks the process table what the helper is (`helper_is_running`,
through `process_birth_identity`,
which answers
nothing for a zombie or a dead process) and not whether the signal can be delivered.
A control makes a zombie of its own
child,
checks that `kill(pid, 0)` still succeeds,
and checks that the new observation calls it not running.

## Rule

Never read `kill(pid, 0) == 0` as "the process runs" in a test or in code that must tell a killed process from a live one;
read the state from `/proc/<pid>/stat` (`Z` and `X` have exited).
