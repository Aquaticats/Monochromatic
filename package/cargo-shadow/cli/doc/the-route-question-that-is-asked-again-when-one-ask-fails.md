# The route question that is asked again when one ask fails

What a whole run of this package's suite met on 2026-11-27,
what it measured,
and what the package now does about it (issue 6619).

Status:
**unapproved and non-final**.
Written by a Pi run on branch
`fix/cargo-shadow-cli-issue-6619-ten-scope-tests-failed-together-once-and-passed-in-the-next-two-runs`.
Every claim about behaviour here names what it rests on:
a command this round ran,
a line of this package,
or another program's own words.
What is still open is at the end.

## What happened

One whole run of `mise run //package/cargo-shadow/cli:test` on 2026-11-27,
in a worktree at `aa5200d1dd`,
failed ten tests,
and the next two whole runs of the same tree passed 941 of 941 each.
Every one of the ten needs a delegated control group,
and every one failed in ten to thirteen milliseconds.
The run's own command piped the task through `tail`,
so the messages of those ten are not in any record.

## What the ten share

Each of the ten is a control that needs a delegated scope,
so each begins with `this_test_needs_a_delegated_scope`
(`src/cgroup_test_support.rs`),
which asks `detect_route` whether this host can give one.
Between that question and the work the control measures,
a control makes a scope,
and making one takes a tenth of a second or more.

Measured 2026-11-27 on this host:

- `cgroup_probe_tests::probe_0_the_host_can_give_the_tests_a_delegated_scope`,
  which is that question and nothing else,
  took 16 ms
  (`cargo nextest run --release -E 'test(=cgroup_probe_tests::probe_0_the_host_can_give_the_tests_a_delegated_scope)'`).
- the ten tests alone,
  once:
  ten passed,
  in 0.099 s to 5.100 s each.
  The slowest is
  `bounded_run_end::retire_tests::the_question_at_a_run_end_keeps_the_bound_the_promise_names`,
  which reads a promise of five seconds.
- the ten tests with a `systemctl` on `PATH` that refuses every request:
  ten failed,
  in 0.008 s and 0.009 s each,
  every one of them printing

  ```text
  cargo-shadow tests: not run [<the test>]: this host cannot give it a delegated control group (the user manager cannot start app.slice (systemctl failed (Failed to start app.slice: stand-in refusal)))
  ```

A run in which all ten failed in about ten milliseconds is therefore a run in which each of them
answered no to the question and died there.
The one red run's ten failed in ten to thirteen milliseconds,
which is that shape and not the shape of any assertion or of a scope that was started and failed.

## What was ruled out

- Another round's work starting and stopping scopes.
  With a run of the scope-heavy modules (91 tests,
  starting and stopping scopes throughout) beside the ten,
  twice,
  the ten passed in both rounds (0.353 s to 5.104 s each):
  the churn of another suite of this crate does not make them fail.
- The manager's own answer under that churn.
  2,000 asks of the question `detect_route` asks (`systemctl --user start app.slice` and
  `systemctl --user show app.slice --property=ControlGroup --value`) over 100 seconds,
  in one scope with that suite running in the same one,
  every ask answered with the path of the slice.
  A slice that empties is not the case here:
  `app.slice` holds this account's application scopes and its agent units,
  so it is never collected.
- The tests' own assertions and the stopping of a scope.
  Nothing behind the question finishes in ten milliseconds;
  a control that failed at an assertion would have taken at least the tenth of a second a scope takes.
- A scope that could not be started.
  A `systemd-run` that fails leaves the control waiting out `STATE_WAIT` (twenty seconds)
  for a member that never appears,
  and the ten failed in ten milliseconds.

## What the one red run met, and what is not measured

The question was answered no.
Which of its answers was no is not measured,
because the messages were lost.
The answers that do not depend on the moment cannot have been it:
a host that is not a cgroup v2 mount,
no `systemd-run` on `PATH`,
a slice without `cgroup.kill`,
the inode of the cgroup namespace,
and the view of the slice directory are all the same on every ask this host gives.
What is left is the two answers that are about a moment:
a program this package starts (`systemctl`) that did not run or did not answer,
and the read of the caller's control group's limits,
which is four files of
`/sys/fs/cgroup` under the caller's own group.
The first of those is what a refused ask looks like,
and it is the one the controls here plant;
the second is not planted by any control of this package.

The host's own record of that minute,
read with `journalctl --user` and `journalctl --since`,
holds two things that could have disturbed a program started then:
the user manager reexecuted at 14:07:27,
and systemd logged that the kernel's OOM killer had killed processes in `user.slice` and `-.slice` at 14:07:28.
`app.slice`'s own `memory.events` reports `oom_kill 6` and `max 4579289` over the slice's life,
and this kernel's journal holds no OOM line for the whole day,
so those counters cannot be dated to that minute from here.
None of this is shown to be the cause,
and this round did not try to make either one happen:
the rules of this machine ban generating load,
and a reexec of the account's manager is not a round's to ask for.

## What the package does now about it

`detect_route` asks `ask_where_the_slice_is` (`src/control_group_route_discovery.rs`),
which asks the manager up to `ASKING_AGAIN_ATTEMPTS` (three) times,
`ASKING_AGAIN_PAUSE` (fifty milliseconds,
ten polls of `DISCOVERY_POLL`) apart,
and answers the path the first good ask reported.
One failure of that program is no longer the host's answer about itself.
An ask that spent its own deadline is not put again:
a manager that answered nothing inside its bound is what a host without one looks like,
and a second ask would only spend a second bound.
The words of the last failure are still the refusal,
so a run that goes on in the weaker path,
and a control that reports `not run`,
name which ask failed and what it said.

Measured 2026-11-27 on this host,
with a `systemctl` on `PATH` that refuses the first ask of every process that calls it,
so each of the ten meets a refused ask at its own start:

- with the change:
  the ten passed (0.117 s to 5.180 s each),
  and the two controls of the change passed.
- with `ASKING_AGAIN_ATTEMPTS` set back to one and nothing else changed:
  the ten failed in 0.010 s to 0.012 s each,
  every one of them printing
  `not run [...] this host cannot give it a delegated control group (the user manager cannot start app.slice (systemctl failed (the stand-in refused this ask)))`,
  and `a_question_that_fails_at_once_is_put_again_and_the_route_is_found` failed while
  `a_manager_that_answers_nothing_is_asked_once_and_ends_the_discovery_with_its_own_words` passed.

The controls are `control_group_route_discovery::asking_again_tests`.

## A failing run leaves its messages

The messages of the one red run were piped through `tail` and lost.
The package's `test` task now keeps the whole output of a failing run in
`target/test-runs/cargo-shadow-tests-<moment>-<pid>.log` under the package,
which the repository's `.gitignore` excludes,
and its last lines name that file.
The suite is still streamed to the terminal as it is written,
so a run that takes minutes is not hidden while it happens.

Measured 2026-11-27 on this host,
with one planted panic in `git_sha1_tests::the_digest_of_nothing_is_the_published_one`:
the task exited 100 with `946 tests run: 945 passed, 1 failed, 26 skipped`,
its last line named the file it kept,
and that file held the whole run:
the build's own warning,
the `FAIL` line,
the panic's message and the summary.

## Open

- Which answer the one red run got,
  and what made that program fail at 14:07 that day.
  A run of the ten beside other work does not produce it here (above),
  and the two host events in that minute are not shown to be it.
- Whether three asks fifty milliseconds apart cover the moment that failed.
  The length of that moment is not measured;
  a failure that outlasts two pauses still ends as a refusal,
  and its messages are now kept,
  so the next red run answers this.
- A control for the other answer that can change:
  a caller's limits that cannot be read.
  No control of this package plants a read of `/sys/fs/cgroup` that fails for a moment.
- The repository linter's `rust/max-lines` finding on `src/lib.rs` (304 code lines,
  limit 300) is on this branch's base already and is not this round's.
