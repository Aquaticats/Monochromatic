# Commit services of this package's tests wrote into the user journal of the machine

Date:
 2026-11-14T16:30-05:00.
Subject:
 a service that a test of `package/git-policy/cli` starts said its release into this machine's user journal,
 in the transient scope of the agent that ran the test,
 so a reader of the journal counts commit services that were never this machine's.
Owner's rules this serves:
 "Protected fixtures/verification use disposable fixtures and never real or shared state"
 and "The wrapper should have cleaned up after itself automatically deterministically."

## Symptom

On 2026-11-14,
between 14:48:30 and 14:54:30,
this machine's user journal held twelve lines
`started release="0.6.1-alpha.30" protocol=12 pid=… instance=…`,
whose program (`_EXE`) was a debug build under two agents' worktrees
and whose unit (`_SYSTEMD_USER_UNIT`) was the transient scope those agents ran their tests in:

```text
1  0.0.1-before     a fixture of worktree:pi-issue-6293-…                          run-p2671595-i….scope
1  0.6.1-alpha.30   ?                                                              ?
9  0.6.1-alpha.30   a fixture of worktree:pi-issue-6293-…                          run-p2671595-i….scope
1  0.6.1-alpha.31   a fixture of worktree:pi-commit-service-is-started-by-…        run-p2637007-i….scope
1  0.6.1-alpha.99   a fixture of worktree:pi-issue-6293-…                          run-p2671595-i….scope
9  0.6.1-alpha.31   …/toolchain/cargo-monochromatic-git-policy-cli/0.6.1-alpha.31  monochromatic-git-policy-cli@….service
```

The last row is the installed wrapper serving this machine's own worktrees,
which is what the journal should hold.
The other rows are throwaway fixtures of two test runs,
and one of them names neither program nor unit at all:
journald leaves those fields out when the writer was gone before it read its credentials,
which a fixture service that ends at once does.

The counts come from the coordinator's script over the user journal,
`~/temp/agent/install-of-0-6-1-alpha-20-on-this-machine-2026-11-13/group-the-services-that-said-their-release-at-start-by-program-and-unit.ts`,
run again for 14:40:00 to 15:00:00 while this branch was written:
thirteen fixture lines of the two worktrees,
and nine lines of the installed program in its own units.

## Root cause

### A service's own code decides where its lines go, and both answers can be the journal

`src/native/service/journald_native.rs` reads two things once,
at the service's first entry:

- `$JOURNAL_STREAM`,
  which `systemd.exec(5)` of the systemd 262 installed here defines as the device and inode of the stream a unit's
  standard output or standard error is connected to when that stream is the journal,
  "to allow services to optionally upgrade their used log protocol to the native journal protocol";
- the device and inode of its own descriptor 2
  (`stderr_identity`,
  `stream_matches`).

When the variable names that descriptor,
the service sends every entry as a datagram to
`/run/systemd/journal/socket`
(`journald_native_socket_target.rs`,
`JOURNAL_SOCKET`),
carrying `PRIORITY`,
`SYSLOG_IDENTIFIER=monochromatic-git-policy-cli-service`,
`MESSAGE` and its `CLI_GIT_*` fields.
Otherwise it writes plain lines,
`<priority>monochromatic-git-policy-cli-service: …`,
on its own standard error.

Both reach the journal when that standard error is the journal:

- the datagram arrives at a fixed path of the machine,
  and journald stamps it with the sender's credentials and cgroup,
  which is how `_EXE` and the unit of a test's service came to be named there;
- a plain line written on the journal's stream socket is read by journald,
  which takes the message and the `<priority>` prefix,
  and leaves the structured fields behind.

Measured on this machine at 2026-11-14T15:58-05:00,
one entry of each path:

```text
_TRANSPORT=journal  SYSLOG_IDENTIFIER=monochromatic-git-policy-cli-service  CLI_GIT_RELEASE=…
_TRANSPORT=stdout   SYSLOG_IDENTIFIER=bash                                  no CLI_GIT_ fields
```

### A disposable `HOME` and `XDG_RUNTIME_DIR` move neither

`HOME` and `XDG_RUNTIME_DIR` decide where the wrapper puts its sockets,
its journals and its unit files.
Neither decides where journald listens,
and neither decides what stream the process that runs the tests was given.
So a test that sets both keeps its fixtures private
and still lets a service it starts write into the journal,
whenever the run itself was given the journal on its standard error.

### The wrapper a test runs as a client was already kept off the journal; a service it starts was not

`support::hermetic`
(`src/native/binary/binary_support.rs`)
gives every wrapped command a cleared environment and
`CLI_GIT_TEST_ONLY_JOURNAL_SOCKET=@cli-git-test-journal-nobody-hermetic`,
an abstract name nothing listens at,
so the entries the wrapper-as-client sends reach no journal.
The service support did not do the same:
`service_command` removed the runtime directory and the session bus,
and fourteen starts then gave the service `Stdio::inherit()`,
or gave it nothing at all and so inherited the run's streams,
while the fixture's log pipe copied every line to this process's own standard error as well.

### A run with capture hides it, which is not a design

`cargo nextest run` gives each test a pipe of its own,
so a service whose streams are inherited from the test writes into that pipe
and nothing reaches the journal.
A test binary started by hand,
`cargo test -- --nocapture`,
and nextest's own `--no-capture` all hand the test the run's streams instead,
and under a unit those are the journal.

Reproduced on this branch,
in a unit whose standard error is the journal:

```sh
systemd-run --user --unit=journal-filtering-unit-direct --collect --wait \
  --property=StandardError=journal -- bash -c 'exec <the test binary> --exact --nocapture service_lane::a_service_refuses_a_request_for_another_worktree'
```

The journal then held two entries of that unit,
`_EXE` = `<worktree>/package/git-policy/cli/target/debug/monochromatic-git-policy-cli`,
`CLI_GIT_RELEASE=0.6.1-alpha.31`:
the shape the coordinator counted.

## How to see what a test's service wrote

Read one scope's or one unit's entries,
bounded,
never the whole journal,
because the commit-service units' journal of this machine is very large:

```sh
journalctl --user --no-pager --output=json --since '<time>' '_SYSTEMD_USER_UNIT=<scope or unit>.scope' |
  grep 'started release='
```

A scope writes no entries of its own,
so any line a run in a scope leaves there came from a service of that run.

## What this branch changes

- `service_command`
  (`src/native/binary/binary_support_fixture_service.rs`)
  gives every control's service the bargain the client already had:
  `$JOURNAL_STREAM` is recorded as removed,
  the test-only address the entries go to is `NOBODY_JOURNAL_ADDRESS`,
  and stdin,
  stdout and stderr are `/dev/null` unless a control names its own,
  so a control that forgets its streams gets silence and never the journal.
- The fourteen starts that inherited the run's streams hand their service the fixture's own pipe instead
  (`kept_stderr_pipe`),
  in `binary_service_provision_tests.rs`,
  `binary_long_runtime_directory_tests.rs`,
  `binary_service_upgrade_one_executable_tests.rs`,
  `binary_service_upgrade_tests.rs`,
  `binary_service_tests.rs`,
  `binary_service_lane_tests.rs`,
  `binary_client_service_tests.rs`,
  `binary_service_landing_lock_tests.rs`,
  `binary_service_finished_record_retention_tests.rs`,
  `binary_service_commit_tests.rs`
  and `binary_service_connections_without_a_thread_each_tests.rs`.
- The fixture's log pipe no longer copies what a service said to this process's own standard error
  (`binary_support_fixture_service_log.rs`):
  that stream is the journal whenever the run was given the journal,
  and the file beside the fixture is what a failure quotes
  (`service_log`).
- A log file that cannot be written keeps its words
  (`binary_support_fixture_service_log.rs`):
  the copier keeps the same bytes in this process whatever the file does,
  `service_log` hands them on with the reason the file holds none,
  and one line on this process's standard error names the file and the reason,
  written only after asking,
  by `systemd.exec(5)`'s reading of `$JOURNAL_STREAM`,
  whether that stream is the journal's.
  A review of 2026-11-14 found the earlier code turning the failed open into nothing,
  so one failure of logging cost a failure every word of its service.
- `source_guard::service_journal_isolation_tests` fails when a source compiled for tests only
  gives a child `Stdio::inherit`,
  names the journal's trigger outside the three lines whose reasons say why,
  stops recording the trigger's removal in the service support,
  or copies a service's words to the run's own streams.
- `binary_fixture_tests.rs` holds the two controls of the support:
  the start command removes the trigger and names an address nothing listens at,
  and a fixture service's `started release=` line is in the fixture's own log.

### The rule as the guard states it

The guard's invariant is what a child is given,
not which token the source spells:

- no start of a child passes `Stdio::inherit`;
- every start of a child accounts for the child's standard error,
  either with a setter of its own,
  or with `output()`'s capture,
  or by an inventory entry that names the child and the reason it may inherit;
- a start of this wrapper's service is never excused,
  and neither is a start whose command names a program of this wrapper
  from inside a shell's script;
- a start whose program the guard cannot name needs an entry of its own,
  so a helper that builds a command is a gap that is written down rather than a silence.

`source_guard/service_journal_isolation_starts_tests.rs` is the reader that makes this checkable.
It masks strings,
characters and comments;
splits a source at every `;` outside parentheses and brackets,
and at a block's closing brace;
reads the clause that ends a start together with the clauses of the binding it is called on
(the receiver of `command.spawn()`,
or the command handed to `spawn_owned(&mut command)`);
and excludes a `fn` definition,
a thread's `.spawn(…)`,
`wait_with_output()`,
and an `output()` that captures both streams.
It says where it is coarse:
a command a *helper* of another function builds is a start whose program it cannot name,
and that is exactly what the inventory's `no name` entries mark.

Measured over this crate's sources compiled for tests only at 2026-11-14T22:00-05:00,
with the reader itself:

- 171 starts of a child inherit the run's standard error;
- 52 of them are the counted entries:
  29 shell starts in fifteen entries,
  21 starts whose program the reader cannot name in twelve entries,
  a `/usr/bin/timeout`,
  and a script a test wrote;
- the other 119 are children of the programs the list names
  (`sleep`,
  `true`,
  `mkfifo`,
  `cp`,
  `tar`,
  `sha1sum`,
  `tee`,
  `cat`,
  `talk`,
  this machine's own Git,
  this crate's own test binary,
  and the scripts the tests write);
- and every counted entry is checked against the source,
  so a new start of a shell or of an unnamed command fails the guard until someone writes why it stays.
  No entry can allow a start of the service,
  or a shell whose script names this wrapper.

Nothing in the release program changes:
the installed service decides where its lines go by the same two readings,
and its lines read the same,
word for word.

## Verification

The controls,
and what each reports with the change taken out:

- `source_guard::service_journal_isolation_tests::no_test_source_gives_a_child_the_streams_of_the_run`
  fails with one start's `Stdio::from(kept_stderr_pipe(…))` put back to `Stdio::inherit()`,
  and names the file and the line:

  ```text
  left: ["binary/binary_service_lane_tests.rs:101: .stderr(Stdio::inherit())"]
  ```

- `source_guard::service_journal_isolation_tests::the_service_start_support_keeps_the_journal_off_every_control_s_service`
  fails with the trigger's removal deleted from `service_command`,
  naming the line it no longer holds:

  ```text
  the support that starts every control's service no longer holds `.env_remove(git_policy_cli::journald_native::STREAM_VARIABLE)`
  ```

- `source_guard::service_journal_isolation_tests::no_test_source_but_the_listed_lines_names_the_journal_s_trigger`
  fails with the same line gone,
  because the exception it allows is no longer needed.
- `fixture_guard::a_control_s_service_is_never_handed_the_journal_of_the_machine`
  fails the same way at the recorded environment:

  ```text
  left: None
  right: Some(None)
  ```

- `fixture_guard::a_fixture_service_says_its_release_into_the_fixture_s_log` holds the other half:
  what the service said at its start is in `<fixture>/service.log`.
- `commit_timing::a_service_on_the_journal_stream_sends_native_entries_that_join_the_clients_record`
  keeps the native path proven:
  a control that stands a receiving socket in still gets the entries,
  because the address it names is set after `service_command`.

- `fixture_guard::a_service_s_log_survives_a_log_file_that_cannot_be_written`
  fails with the memory that keeps the words taken out of the copier,
  which is the review's first finding:

  ```text
  the words of the service were lost with the file that could not hold them: the fixture's service log file holds nothing (Is a directory (os error 21)); what its services said is kept here (0 bytes kept):
  ```

- `fixture_guard::the_note_names_the_file_and_the_failure_and_the_reading_is_the_stream_s_own`
  holds the note and the reading that decides whether it is written at all.
- `source_guard::service_journal_isolation_tests::the_rule_fails_on_a_start_that_sets_nothing`
  is the review's second finding as a control:
  a `/bin/sh` start with `.status()` and nothing else must be a finding,
  and it is a finding with the reason `no entry with a reason covers them`.
- The same rule over the crate's own sources,
  in `no_test_source_gives_a_child_the_streams_of_the_run`,
  fails on a leaking start added to a test file:

  ```text
  left: ["binary/binary_fixture_tests.rs: 1 start(s) of /bin/sh inherit the run's standard error and no entry with a reason covers them"]
  ```

  and passes it again when the rule is taken back to the token check,
  which is the review's finding in one line.
- The review's own two scripts,
  run against this branch:
  `log_open_failure.rs` passes,
  and prints the note and the words
  (`the fixture's service log … took nothing (Is a directory (os error 21)); what its services said is kept here (37 bytes kept): service-start-failed: fixture marker`);
  `guard_default_inheritance.rs` fails at the test the review ran,
  naming its own file:
  `default_inherited_service.rs: 1 start(s) of /bin/sh inherit the run's standard error and no entry with a reason covers them`.

The measured before and after,
both in a unit whose standard error was the journal,
each reading the journal for that unit's own entries:

- before,
  the test binary run by hand with `--nocapture`,
  one test:
  two `started release=` entries whose `_EXE` is this worktree's debug build;
- after,
  the same two tests by hand in the same kind of unit:
  none,
  and the two tests pass.

## What does not work

- Setting `HOME` and `XDG_RUNTIME_DIR` for the test:
  neither moves `/run/systemd/journal/socket`,
  and neither moves the stream the run was given.
- Reading `$JOURNAL_STREAM` alone:
  a redirection leaves it set,
  which is why the service compares it with its own descriptor,
  as `systemd.exec(5)` says in words.
- Pointing the test-only address at an abstract name only where a control reads entries:
  a start that names nothing would still take the machine's socket,
  so the support sets the address and a control overrides it.
- An inherited journal for the client side was already prevented
  (`support::hermetic`),
  and that half is unchanged.

## Left alone, and reported

- The two worktrees whose runs wrote the lines above still hold their own starts,
  and this branch does not change them;
  merging this branch's support is what makes the same runs silent.
- The lines already in this machine's journal are not removed:
  the journal is not this package's to rewrite,
  and a reader of 2026-11-14 still counts them.
- A test's service also reaches `${HOME}/temp/agent/rn-…` and `rt-…` runtime directories,
  which are outside the fixture root and shared with every other session
  (`short_runtime_directory`,
  `runtime_for`),
  and the disposable fixture root under the build directory.
  Neither is the journal,
  and neither is changed here.
- The 119 children of programs the inventory names still inherit the run's standard error:
  a `sleep`,
  `true`,
  `mkfifo`,
  `cp`,
  `tar`,
  `sha1sum`,
  `tee`,
  `cat`,
  `talk`,
  this machine's own Git,
  this crate's own test binary,
  or a script a test wrote.
  None of them is this wrapper's service and none can run it,
  so what they write is the test's own tool's words;
  a run that puts the journal on the test's standard error shows the test's own output there whatever this package does.
  Making all of them explicit is a change of 119 call sites and of what a test's output shows,
  and it is not this branch's.
- A refused service start can lose its own explanation:
  `service_error!` hands the line to
  `service_log_lines_off_the_calling_thread`,
  and `main` calls `std::process::exit` before that thread writes it.
  Measured at 2026-11-14T16:38-05:00:
  a start refused because `--git` names the wrapper program wrote nothing on standard error,
  ten times of ten,
  with and without an environment,
  while a refusal that creates its journal directory first wrote its line every time.
  The control `service_mode::a_git_that_is_the_wrapper_is_refused` fails on this for main's own support as well,
  which is how it was told apart from this branch.

- Two more tests of this package are red on this host with `main`'s own support:
  `fault_limits::a_service_that_ends_before_it_listens_leaves_a_recoverable_repository`
  (the limit it measures one MiB under still lets the service run)
  and `service_no_manager::a_commit_with_no_service_manager_stops_plainly_and_changes_nothing`
  (a `cli-git-lock-holders` folder is left in the common directory).
  Both were run alone and with `main`'s own `binary/` support put back,
  and both fail there too,
  so neither is this branch's.

## Upstream filing

Nothing to file:
the decision that sends a service's entries to the machine's journal socket is right for a person's service,
and what was wrong is this package's test support.
The only outside fact used is `$JOURNAL_STREAM` and the socket path,
both documented by systemd.
