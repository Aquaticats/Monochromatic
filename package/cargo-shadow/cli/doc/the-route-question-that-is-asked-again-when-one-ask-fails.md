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

## What did not reproduce, and what the timings rule out

- Another round's work starting and stopping scopes did not make them fail here.
  With a run of the scope-heavy modules (91 tests,
  starting and stopping scopes throughout) beside the ten,
  twice,
  the ten passed in both rounds (0.353 s to 5.104 s each).
  That is not the same as "churn is not the cause":
  it says that this shape,
  at this rate,
  does not produce the failure on this host.
- The manager's own answer under that churn answered every time.
  2,000 asks of the question `detect_route` asks (`systemctl --user start app.slice` and
  `systemctl --user show app.slice --property=ControlGroup --value`) over 100 seconds,
  in one scope with that suite running in the same one,
  every ask answered with the path of the slice.
  A slice that empties and is collected is not the case here either:
  `app.slice` holds this account's application scopes and its agent units,
  so it does not empty.
- The tests' own assertions and the stopping of a scope are ruled out by the timings.
  Nothing behind the question finishes in ten milliseconds;
  a control that failed at an assertion would have taken at least the tenth of a second a scope takes,
  and a `systemd-run` that fails leaves the control waiting out `STATE_WAIT` (twenty seconds)
  for a member that never appears.
- A fixture folder,
  or one test executable,
  shared by two whole runs is ruled out.
  `scratch()` makes a randomly named directory for each control,
  and the two runs of that minute read different build folders
  (`~/.cargo/build/f3/…` and `~/.cargo/build/toolchain/nightly-2026-11-16/…`,
  from the journal's own test-scratch paths).

## What the one red run met, and what is not measured

The question was answered no.
Which of its answers was no is not measured,
because the messages were lost.
The answers the discovery can give were read from its own refusals
(`detect_route` in `src/control_group_route_discovery.rs`),
and the ones that do not depend on the moment cannot have been it:
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
An ask whose program spent its own deadline is not put again:
a manager that answered nothing inside its bound is what a host without one looks like,
and a second ask would only spend a second bound.
Each program an ask runs is timed on its own (`spent_its_bound`),
because the discovery starts two of them and a slow success followed by a fast failure must still be put again.
The words of the last failure are still the refusal,
so a run that goes on in the weaker path,
and a control that reports `not run`,
name which ask failed and what it said.

Measured 2026-11-27 on this host,
with a `systemctl` on `PATH` that refuses the first ask of every process that calls it,
so each of the ten meets a refused ask at its own start:

- with the change:
  the ten passed (0.109 s to 5.181 s each),
  and the controls of the change passed,
  four of them:
  the question put again after a failed ask,
  a manager that answered nothing asked once,
  a program that ended inside its bound read as no spent deadline,
  and a question that keeps failing still refused with the manager's own words.
- with `ASKING_AGAIN_ATTEMPTS` set back to one and nothing else changed:
  the ten failed in 0.010 s to 0.012 s each,
  every one of them printing
  `not run [...] this host cannot give it a delegated control group (the user manager cannot start app.slice (systemctl failed (the stand-in refused this ask)))`,
  and two of the four controls failed with them.

The controls are `control_group_route_discovery::asking_again_tests`:
the question is put again after a failed ask,
a manager that answered nothing is asked once,
a program that ended inside its bound is no spent deadline,
and a question that keeps failing still refuses with the manager's own words.
The last two are not made red by the change being taken out:
the bound rule is a rule about one program,
and the manager that answers nothing is asked once either way.
The question that fails at once and the question that keeps failing are:
with `ASKING_AGAIN_ATTEMPTS` at one they both fail,
the second on its own claim that the question was put more than once.

## A failing run leaves its messages

The messages of the one red run were piped through `tail` and lost.
The package's `test` task now keeps the whole output of a failing run in
`target/test-runs/cargo-shadow-tests-<moment>-<pid>.log` under the package,
which the repository's `.gitignore` excludes,
and its last lines name that file.
The suite is still streamed to the terminal as it is written,
so a run that takes minutes is not hidden while it happens.
A degraded run keeps both of its runs when it fails:
the one that looked for the tests this host cannot run,
and the report run.
What this does not cover,
and why:
the output is held in the task's own memory and written when the run ends,
so a task killed outright,
by the kernel's OOM killer among others,
keeps nothing;
and the kept file is standard output followed by standard error,
not the order the two streams interleaved in.

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
- A kept run is written when the run ends,
  not as it streams,
  so a task that is killed before its suite ends keeps nothing (above).
  Writing it as it streams would cost a file per run,
  including the runs that pass,
  and this round did not measure whether that is worth it.
