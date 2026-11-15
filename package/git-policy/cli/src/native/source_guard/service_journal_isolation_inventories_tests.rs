//! What:
//!  The lists the journal-isolation guard judges by,
//! each entry with the reason it stays:
//! the lines that may name the journal's trigger,
//! the programs whose child may inherit the run's standard error,
//! and the starts that may,
//! by the source and by what the child is,
//! with the number of them the source holds.
//! Why:
//!  A list kept in the guard would be a list of excuses;
//! a list with a reason each,
//! and a count that fails when the source gains one more,
//! is where a judgement about a child is written down and read again.
//!  Nothing here excuses a start of the wrapper's service,
//!  a shell that runs a command this file does not show,
//!  or `Stdio::inherit`:
//! those are the forms that put a service's line on a stream the run was given,
//! and the guard has no entry that can allow one
//! (`doc/troubleshooting/git-policy-cli-test-services-write-into-the-user-journal.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const ALLOWED_TRIGGER_LINES: [string, string][] = []; export const INHERITING_PROGRAMS: [string, string][] = [];
//! ```

/// What:
///  The lines of test sources that may name the journal's trigger,
///  and why each may.
/// Why:
///   What is banned is handing the trigger to a child;
///  a line that records it *removed* hands nothing,
///  and a control that reads the start's own record must name it to say what it expects.
///  Each entry's text is checked to be still there,
///  so the list cannot rot into an excuse for a line that went.
///  The one control that stands a journal of its own in is here for the other reason:
/// it hands the trigger to a service deliberately,
///  with an address of its own,
///  and it is the control that proves the native path still works.
pub(crate) const ALLOWED_TRIGGER_LINES: &[(&str, &str)] = &[
    // The support removes the trigger from every control's service, which is the fix the guard keeps.
    ("binary/binary_support_fixture_service.rs", "env_remove(git_policy_cli::journald_native::STREAM_VARIABLE)"),
    // The control that reads the start's env records and expects the removal.
    ("binary/binary_fixture_tests.rs", "journald_native::STREAM_VARIABLE),"),
    // The one control that stands a journal of its own in.
    ("binary/binary_commit_timing_tests.rs", "STREAM_VARIABLE"),
    // The log support names the variable it reads, and reads it to ask about its own standard error: no child is given it.
    ("binary/binary_support_fixture_service_log.rs", "pub const STREAM_VARIABLE"),
    ("binary/binary_support_fixture_service_log.rs", "std::env::var(STREAM_VARIABLE)"),
];

/// What:
///  The program expressions whose child may inherit the run's standard error,
///  with the reason each may.
/// Why:
///   The guard's rule is about the words of this wrapper's commit service.
/// A child that is not this wrapper and cannot run it writes the test's own tool's words,
/// which are the test's output,
/// never a line of a service;
/// a run that puts the journal on the test's standard error shows the test's own output there whatever this package does.
///  A shell is not here:
/// a shell runs a command from its argument,
///  so it can run the wrapper,
///  and a start of one must give the child a standard error of its own.
///  A program that is not here must be given one too,
///  which is how a new tool,
///  or a new spelling of an old one,
///  is a decision rather than a reflex.
pub(crate) const INHERITING_PROGRAMS: &[(&str, &str)] = &[
    // A sleeping child says nothing on any stream.
    ("\"sleep\"", "a `sleep` writes nothing, and a control that leaks it sees a silence"),
    ("\"/bin/sleep\"", "a `sleep` writes nothing"),
    ("\"true\"", "`true` writes nothing and ends at once"),
    ("\"/bin/true\"", "`true` writes nothing and ends at once"),
    ("\"/usr/bin/false\"", "`false` writes nothing and ends at once"),
    // A tool the test runs for its effect, which says nothing on a success.
    ("\"mkfifo\"", "a `mkfifo` says nothing when it makes the file; a failure fails the control on its status"),
    ("\"cp\"", "a `cp` says nothing on a copy that works; a failure fails the control on its status"),
    ("\"/usr/bin/cp\"", "a `cp` says nothing on a copy that works"),
    ("\"/usr/bin/tar\"", "a `tar` says nothing on an archive that works"),
    ("\"sha1sum\"", "a `sha1sum` writes the digest on standard output the control reads through its own pipe, and nothing on standard error"),
    ("\"tee\"", "a `tee` writes the bytes it is handed, and a control that hands it bytes reads them back from the file it names"),
    ("\"cat\"", "a `cat` writes what it is given, and the control that starts it reads it through a pipe of its own"),
    ("\"talk\"", "a program this machine does not have, started to prove a missing program is no failure: it writes nothing"),
    // The test's own Git, and the wrapper the test runs as a client.
    ("REAL_GIT", "the machine's own Git: its words are the test's output, and it cannot be the wrapper's commit service"),
    ("crate::test_support::REAL_GIT", "the machine's own Git: its words are the test's output, and it cannot be the wrapper's commit service"),
    ("\"/usr/bin/git\"", "the machine's own Git: its words are the test's output, and it cannot be the wrapper's commit service"),
    ("\"git\"", "the machine's own Git: its words are the test's output, and it cannot be the wrapper's commit service"),
    ("&context.real_git", "the machine's own Git, borrowed from the context: its words are the test's output"),
    ("&named", "the machine's own Git, under a name the control made: its words are the test's output"),
    ("&old_command[0]", "the machine's own Git under the name of a removed program: it writes nothing, and the control fails on the start itself"),
    // A test process of this crate's own, and a program under a name the control chose.
    ("crate::test_support::this_test_binary().expect(\"test binary\")", "another run of this crate's own test binary, never the wrapper"),
    ("test_binary", "another run of this crate's own test binary, never the wrapper"),
    // A program this machine does not have, started to prove that a refused start is retried and then reported.
    ("\"/nonexistent-cli-git-program\"", "no such program exists: the start fails at once and writes nothing"),
    ("program", "the program of a script this crate's tests wrote, which names no program of the wrapper"),
    ("program.as_path()", "the program of a script this crate's tests wrote, which names no program of the wrapper"),
];

/// What:
///  The helpers that collect their child's output,
///  by name,
///  with the reason their own body cannot show it.
/// Why:
///   The reader reads a helper's shape from the body that starts the child,
///  and these helpers start it one call further on:
/// `output_retrying` hands the start to `OutputStart::start`,
///  which runs `Command::output`.
///  A helper whose shape no entry and no body gives is read as inheriting,
///  which is the safe direction (`fail closed`),
///  so this list is what a helper like this needs to be read as the capture it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const CAPTURING_HELPERS: [string, string][] = [['output_retrying', 'it starts through OutputStart::start, which runs Command::output']];
/// ```
pub(crate) const CAPTURING_HELPERS: &[(&str, &str)] = &[
    ("output_retrying", "it hands the start to `OutputStart::start`, which runs `Command::output`: the child's words are collected by the test"),
];

/// What:
///  The starts that may hand their child the run's standard error,
///  by the source,
///  by what the child is,
///  and by how many of them the source holds,
///  with the reason each stays.
/// Why:
///   Two forms are here,
///  and neither may be excused for any other source:
///  - a shell running a command the source writes out,
///    which names no program of this wrapper:
///    the guard reads the script and reports the entry whose shell now names one;
///  - a start whose command a helper of another file builds,
///    so the guard's reader cannot name its program:
///    the entry is what makes the gap visible,
///    and the count fails when the source gains one more.
///
///  Giving these children a standard error of their own is what removes an entry,
///  and the count is what keeps an entry from covering a start that was added later.
pub(crate) const INHERITING_STARTS: &[(&str, &str, usize, &str)] = &[
    // A shell whose script this control writes out, which names no program of the wrapper and writes nothing of its own.
    ("binary/binary_forwarding_tests.rs", "/bin/sh", 1, "the script sends `kill` to a process the control started"),
    ("binary/binary_merge_isolation_tests.rs", "/bin/sh", 1, "the script is `exit 0`: a process that ends"),
    ("commit/scratch_step_ceiling_stop_tests.rs", "sh", 1, "the script starts a `sleep` in the background, prints its number and waits"),
    ("invocation/startup_gate_tests.rs", "bash", 1, "the script is the removal command this crate writes, which removes a registry directory"),
    ("invocation/startup_gate_tests.rs", "sh", 1, "the script is the removal command this crate writes, which removes a registry directory"),
    ("landing/service_index_lock_tests.rs", "sh", 2, "the script opens the index lock and execs `sleep`"),
    ("landing/service_land_reference_lock_tests.rs", "sh", 1, "the script opens the reference lock and execs `sleep`"),
    ("lock/stale_ref_locks_tests.rs", "sh", 2, "the script opens the reference lock and execs `sleep`"),
    ("process/child_awaited_until_it_exits_the_stop_or_the_ceiling_tests.rs", "sh", 5, "the script starts a `sleep` in the background and waits, so the child has a member to wait for"),
    ("process/process_tree_tests.rs", "sh", 4, "the script starts a `sleep` in the background and waits, so the child has a member to wait for"),
    ("process/wait_after_the_stop_saying_the_kernel_holds_it_tests.rs", "sh", 1, "the script starts a `sleep` in the background and waits"),
    ("push/child_watch_tests.rs", "sh", 5, "the script starts a `sleep` in the background, prints its number and waits"),
    ("push/stop_of_a_push_and_sweep_of_its_locks_tests.rs", "sh", 2, "the script execs `sleep`: a child to hand over"),
    ("testing/run_progress_tests.rs", "sh", 1, "the script starts a `sleep` in the background and prints its number"),
    ("worktree_copying/worktree_copy_interrupted_checkout_tests.rs", "sh", 1, "the script opens the index lock, writes a marker and sleeps"),
    // A shell a helper builds around a script this control writes out.
    ("push/bounded_push_awaiting_tests.rs", "no name", 1, "the helper `shell(...)` runs a script that traps a signal, prints a ready line the control reads through its own pipe, and execs `sleep`"),
    ("push/bounded_push_tests.rs", "no name", 1, "the helper `shell(...)` runs a script that execs `sleep`, with the output the control reads through its own pipe"),
    ("push/bounded_query_tests.rs", "no name", 1, "the helper `shell(...)` runs `sleep`, with its input the pipe the control feeds"),
    // A call on the value a start handed back, which the reader followed to a command in the same file: the start itself is
    // read where it is written, and what this call does with its result starts nothing.
    ("commit/commit_preparation_tests.rs", "no name", 1, "a call on the values of a context's overlay, followed to a command in the same file"),
    ("binary/binary_service_pruned_tests.rs", "no name", 1, "a call on the handle of a thread the control joined, followed to a command in the same file"),
    // A start whose command a support or a helper of this crate builds, so the reader cannot name its program.
    ("binary/binary_support_fixture_service.rs", "no name", 2, "the two bodies of the process-lifetime spawner: the command and its streams are the caller's, and the support adds only a process group and the parent-death signal"),
    ("testing/test_support.rs", "no name", 1, "the body of the older `spawn_owned`: the command and its streams are the caller's"),
    ("binary/binary_fault_stream_tests.rs", "no name", 5, "the machine's own Git and the wrapper a test runs as a client, both built by helpers of this file: their words are the test's output, never a service's line"),
    ("binary/binary_hooks_off_escape_tests.rs", "no name", 1, "a Git this file builds through a helper: its words are the test's output"),
    ("binary/binary_killed_holder_lock_tests.rs", "no name", 1, "the helper `quiet(\"sleep\")`: a child that sleeps and writes nothing"),
    ("binary/binary_push_stacking_tests.rs", "no name", 1, "`direct(...)`, the machine's own Git: its words are the test's output"),
    ("binary/binary_resolution_tests.rs", "no name", 1, "a forwarding loop the control bounds, built by a helper of this file: the wrapper as a client, whose words are the test's output"),
    ("binary/binary_stale_ref_lock_tests.rs", "no name", 2, "`direct(...)`, the machine's own Git, started to hold its own locks: its words are the test's output"),
    ("lock/stale_ref_lock_killed_git.rs", "no name", 4, "`git_command(...)`, the machine's own Git, started to hold its own locks"),
    // A program this crate does not run as the wrapper, under a name the control chose.
    ("binary/binary_service_landing_support.rs", "/usr/bin/timeout", 1, "a `cp --archive` under a time bound: a copy that says nothing when it works"),
    ("landing/service_land_client_gone_tests.rs", "script.as_path()", 1, "a script this control writes: it opens the index lock, waits for a release file and removes the lock"),
];

/// What:
///  The roads to `/run/systemd/journal/socket` in the release program,
///  by the file that holds each,
///  with what decides that a process started by a test does not take it.
/// Why:
///   The task is that no line of a test's service reaches this machine's journal,
/// and a reviewer of 2026-11-14 found a second road after the first was closed:
/// the service's own sink (`journald_native`) and the sender the wrapper uses as a client (`client_journal`).
///  A rule that lists the roads is what keeps the next one from being written in silence:
/// the name of the socket and the constructor of its target appear only in these files,
///  and each reason says what stops a test's process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const ROADS_TO_THE_SOCKET: [string, string, string][] = [['service/journald_native.rs', 'Target::journal()', 'only when $JOURNAL_STREAM names the process its own standard error']];
/// ```
pub(crate) const ROADS_TO_THE_SOCKET: &[(&str, &str, &str)] = &[
    (
        "service/journald_native_socket_target.rs",
        "JOURNAL_SOCKET",
        "the constant and the constructor of the target: they name the socket and start nothing",
    ),
    (
        "service/journald_native.rs",
        "Target::journal()",
        "the service's sink takes the native road only when `$JOURNAL_STREAM` names its own standard error, and a control's service is given neither that variable nor the journal as its stream",
    ),
    (
        "service/client_journal.rs",
        "Target::journal()",
        "the sender the wrapper uses as a client: `record_entry_as` routes a service's entry through the service's sink, and `send_entry_as` refuses inside the service, so only a process that is not the service reaches this",
    ),
];

/// What:
///  The sender functions of the release program,
///  by the file that holds them,
///  with the reason each is the only one.
/// Why:
///   Two `send_to_addr` calls are the two roads;
///  a third would be a road nobody listed,
///  so this rule fails on one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SENDERS: [string, string][] = [['service/journald_native.rs', 'the sink'], ['service/client_journal.rs', 'the client sender']];
/// ```
pub(crate) const SENDERS: &[(&str, &str)] = &[
    ("service/journald_native.rs", "the service's own sink, whose native road its standard error decides"),
    ("service/client_journal.rs", "the sender the wrapper uses as a client, which refuses inside the service"),
];

/// What:
///  The functions of the test support that start a service,
///  by file and by name,
///  each of which must keep the journal off the command it starts.
/// Why:
///   A variable each launcher must remember is the shape that failed once
/// (`FixtureService::serve` forgot it),
///  so each launcher's own body is read for the one call that sets it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SERVICE_LAUNCHERS: [string, string][] = [['binary/binary_support_fixture_service.rs', 'serve']];
/// ```
pub(crate) const SERVICE_LAUNCHERS: &[(&str, &str)] = &[
    ("binary/binary_support_fixture_service.rs", "service_command"),
    ("binary/binary_support_fixture_service.rs", "serve"),
    ("binary/binary_service_upgrade_tests.rs", "start_old_service"),
    ("binary/binary_service_upgrade_one_executable_tests.rs", "activate"),
];

/// The call each of the functions above makes, which keeps the journal off the service it starts.
pub(crate) const KEEPS_THE_JOURNAL_OFF: &str = "keep_the_journal_off(";
