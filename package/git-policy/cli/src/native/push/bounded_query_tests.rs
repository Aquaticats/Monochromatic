//! Controls for the deadline-bound Git query:
//! real children that answer,
//! fail,
//! stall or flood.

use super::*;
use crate::test_support::scratch_environment;
/// The pipe a control gives a writer.
use std::process::ChildStdin;

/// A shell command running `script`.
#[cfg(unix)]
fn shell(script: &str) -> Command {
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg(script);
    return command;
}

/// The query window is the push's stall window,
/// set by the same variable,
/// with no ceiling of its own.
#[test]
fn the_query_window_is_the_push_stall_window() {
    let set = |value: &str| -> Vec<(OsString, OsString)> {
        return vec![(
            OsString::from(crate::bounded_push::PUSH_DEADLINE_VARIABLE),
            OsString::from(value),
        )];
    };
    assert_eq!(query_window(&[]), crate::bounded_push::DEFAULT_PUSH_DEADLINE);
    // A window above what used to be the query ceiling is followed exactly.
    let high: u64 = crate::bounded_push::DEFAULT_PUSH_DEADLINE.as_secs() * 10;
    assert_eq!(query_window(&set(high.to_string().as_str())), Duration::from_secs(high));
}

/// A finished query keeps its exit code and its two streams apart.
#[cfg(unix)]
#[test]
fn a_finished_query_keeps_code_and_streams_apart() {
    let output: QueryOutput = run_bounded_query(
        shell("echo out; echo err >&2; exit 3"),
        "fixture",
        Duration::from_secs(60),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(
        output,
        QueryOutput {
            code: Some(3),
            stdout: b"out\n".to_vec(),
            stderr: b"err\n".to_vec(),
        }
    );
}

/// A query ended by a signal has no code.
#[cfg(unix)]
#[test]
fn a_query_ended_by_a_signal_has_no_code() {
    let output: QueryOutput = run_bounded_query(
        shell("kill -9 $$"),
        "fixture",
        Duration::from_secs(60),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert_eq!(output.code, None);
}

/// A silent query is stopped when it has shown no sign of work for the window and reported as a stall naming its arguments.
#[cfg(unix)]
#[test]
fn a_stalled_query_is_stopped_and_reported() {
    let started = std::time::Instant::now();
    let failure: SingleFlightError = run_bounded_query(
        shell("exec sleep 8"),
        "rev-parse HEAD",
        Duration::from_millis(300),
        Duration::from_secs(5),
        scratch_environment().as_slice(),
    )
    .expect_err("stalls");
    assert!(started.elapsed() >= Duration::from_millis(300), "{:?}", started.elapsed());
    match &failure {
        SingleFlightError::TimedOut { arguments, seconds } => {
            assert_eq!(arguments, "rev-parse HEAD");
            assert_eq!(*seconds, 0);
        }
        other => panic!("expected a timeout, got {other}"),
    }
    assert!(
        failure
            .to_string()
            .starts_with("git rev-parse HEAD showed no sign of work for 0 seconds and was stopped; CLI_GIT_PUSH_DEADLINE_SECONDS"),
        "{failure}"
    );
}

/// A query that floods either stream is stopped and reported as a Git failure.
#[cfg(unix)]
#[test]
fn a_flooding_query_is_stopped() {
    for script in [
        "head -c 20000000 /dev/zero; exec sleep 8",
        "head -c 20000000 /dev/zero >&2; exec sleep 8",
    ] {
        let failure: SingleFlightError = run_bounded_query(
            shell(script),
            "log",
            Duration::from_secs(60),
            Duration::from_secs(5),
            scratch_environment().as_slice(),
        )
        .expect_err("floods");
        match &failure {
            SingleFlightError::Git { arguments, code, stderr } => {
                assert_eq!(arguments, "log");
                assert_eq!(*code, None);
                assert!(stderr.contains("bytes, so its output is not taken"), "{stderr}");
            }
            other => panic!("expected a Git failure, got {other}"),
        }
    }
}

/// What:
///  A query that prints past the cap and ends before the next look at its files is a flood,
/// never a success.
/// Why:
///   The second pass reads blob contents from this output,
/// so bytes cut at the cap with exit status zero would reach the merge as Git's whole answer.
/// The fixture writes one byte past the cap and ends at once,
/// which is the run the waiter sees end before it can look.
#[cfg(unix)]
#[test]
fn a_query_that_floods_and_ends_is_not_a_success() {
    let failure: SingleFlightError = run_bounded_query(
        shell("exec head -c 4194305 /dev/zero"),
        "cat-file --batch",
        Duration::from_secs(60),
        Duration::from_secs(5),
        scratch_environment().as_slice(),
    )
    .expect_err("more than the cap was printed, so the answer is not whole");
    match &failure {
        SingleFlightError::Git { arguments, code, stderr } => {
            assert_eq!(arguments, "cat-file --batch", "{failure}");
            assert_eq!(*code, None);
            assert!(stderr.contains("bytes, so its output is not taken as Git's answer"), "{stderr}");
        }
        other => panic!("expected a Git failure, got {other}"),
    }
}

/// What:
///  A query whose input thread cannot start is stopped with the same bounded stop as any stopped query.
/// Why:
///   The command never got its request,
/// and the end of a child the kernel will not release is not waited for:
/// the
///  stop asks it to end,
/// forces the end past the grace,
/// and hands what it cannot collect to the sweep.
/// The
///  fixture takes no polite stop,
/// so the grace is what bounds the answer.
#[cfg(unix)]
#[test]
fn a_query_whose_input_cannot_be_given_is_stopped_within_its_grace() {
    use std::os::unix::process::CommandExt as _;
    let grace: Duration = Duration::from_millis(400);
    let root: std::path::PathBuf = crate::test_support::fixture("query-input-thread-refused");
    let shim: std::path::PathBuf = root.join("git-that-ignores-the-polite-stop");
    std::fs::write(shim.as_path(), "#!/bin/sh\ntrap '' TERM\nexec sleep 30\n").expect("stand-in");
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(shim.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    }
    // The stop signals the command as soon as it is started, and a shell that has not reached its own `trap` yet dies of
    //  the signal (measured on a loaded container: a stop within a millisecond of the start kills it, one after five
    //  milliseconds does not). The disposition is therefore set before the program is executed, so what this control
    //  measures is the grace and not how far the shell had got.
    let mut program: Command = Command::new(shim.as_path());
    // SAFETY: the closure runs in the child between `fork` and `exec`, where only async-signal-safe work is allowed and
    //  where `signal` is; it touches no memory of this process.
    unsafe {
        program.pre_exec(|| {
            let _previous = libc::signal(libc::SIGTERM, libc::SIG_IGN);
            return Ok(());
        });
    }
    let _refused = crate::scoped_start::RefusedStarts::here();
    let began: std::time::Instant = std::time::Instant::now();
    let failure: SingleFlightError = run_bounded_git(
        program,
        "hash-object --stdin",
        Duration::from_secs(60),
        None,
        grace,
        scratch_environment().as_slice(),
        Some(b"x".to_vec()),
        MAX_PUSH_OUTPUT,
    )
    .expect_err("the command never got its request");
    let waited: Duration = began.elapsed();
    assert!(failure.to_string().contains("no thread could be started"), "{failure}");
    assert!(waited >= grace, "the command was asked to end and given its grace before the kill: {waited:?}");
    assert!(waited < Duration::from_secs(10), "the stop is bounded, not a wait for the child: {waited:?}");
    crate::test_support::remove(root.as_path());
}

/// A program that cannot start is a start failure.
#[test]
fn a_program_that_cannot_start_is_a_start_failure() {
    let failure: SingleFlightError = run_bounded_query(
        Command::new("/nonexistent/cli-git-query-fixture"),
        "x",
        Duration::from_secs(5),
        Duration::from_secs(1),
        scratch_environment().as_slice(),
    )
    .expect_err("no such program");
    assert!(matches!(failure, SingleFlightError::Start(_)), "{failure}");
}

/// A query that keeps writing is never stopped however long it runs:
/// the window judges quiet time,
/// not wall time.
///  The fixture writes a line every pause for a multiple of the window,
/// with no busy loop.
#[cfg(unix)]
#[test]
fn a_query_that_keeps_working_is_never_stopped_however_long_it_runs() {
    let window: Duration = Duration::from_millis(1200);
    let started = std::time::Instant::now();
    let output: QueryOutput = run_bounded_query(
        shell("n=0; while [ $n -lt 15 ]; do echo line; sleep 0.2; n=$((n + 1)); done"),
        "log",
        window,
        Duration::from_secs(5),
        scratch_environment().as_slice(),
    )
    .expect("a query that keeps working is never stopped");
    assert_eq!(output.code, Some(0));
    assert!(started.elapsed() > window * 2, "the fixture ran only {:?}", started.elapsed());
}

/// A query on a host whose `/proc/<pid>/io` cannot be read still ends:
/// the window it is given is then the whole time it may run,
/// and without that fallback nothing at all would end it.
/// The seam is the clock the shipped query builds,
/// made unmeasured as such a host makes it.
#[test]
fn a_query_where_nothing_is_measured_ends_at_its_window() {
    let window: Duration = Duration::ZERO;
    let mut query: Clock = query_clock(0, window).unmeasured();
    assert!(matches!(query.check(), Some(StopCause::Deadline)), "the query's clock ends the run at its window");
    // The control: the same zero window ends nothing without the fallback, so the window above is live.
    let mut unjudged: Clock = query_clock(0, window).unmeasured().without_wall_fallback();
    assert!(unjudged.check().is_none(), "the same window ends nothing when the run may not be ended by wall time");
}

/// What:
///  The thread that gives a second-pass command its input gives up when the command takes none of it.
/// Why:
///   A command the kernel holds without reading keeps its end of the pipe open,
/// and a writer that waits in a full pipe would
///  hold the request and its thread for as long as that lasts;
/// the bound is the query's own quiet window applied to the
///  input instead of the output.
#[cfg(unix)]
#[test]
fn an_input_writer_gives_up_when_the_command_takes_nothing() {
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg("exec sleep 10").stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null());
    let mut child: Child = command.spawn().expect("a child that never reads");
    let pipe: Option<ChildStdin> = child.stdin.take();
    let window: Duration = Duration::from_millis(200);
    let writer: InputWriter = feed_standard_input(pipe, vec![b'x'; 1 << 20], window, None).expect("a thread to write the input");
    // The join is bounded here too: a writer that waited for the child would outlast this wait.
    let (done, waited) = std::sync::mpsc::channel::<()>();
    let observer = std::thread::spawn(move || {
        let _ = writer.join();
        let _ = done.send(());
    });
    let gave_up: bool = waited.recv_timeout(Duration::from_secs(3)).is_ok();
    let _killed = child.kill();
    let _reaped = child.wait();
    observer.join().expect("the observer ends");
    assert!(gave_up, "the writer gave up on a command that took none of its input");
}

/// What:
///  A query whose request could not be written whole is a failure,
/// never a success made from a prefix of it.
/// Why:
///   A command can keep its own quiet meter busy with progress on one stream
/// while it postpones reading its request,
/// the writer then gives up and closes the input,
/// and the command answers from the prefix it has;
/// that answer is not made from the
///  request the caller wrote.
#[cfg(unix)]
#[test]
fn a_request_that_could_not_be_written_whole_is_a_failure() {
    let root: std::path::PathBuf = crate::test_support::fixture("query-request-cut");
    let shim: std::path::PathBuf = root.join("command-that-reads-late");
    // It shows work every 300 ms for three seconds without reading its input, then reads what is there and exits 0, which is
    //  longer than the writer waits and shorter than the quiet window of the query.
    std::fs::write(shim.as_path(), "#!/bin/sh\ni=0\nwhile [ $i -lt 10 ]; do echo tick >&2; sleep 0.3; i=$((i+1)); done\ncat > /dev/null\nexit 0\n").expect("stand-in");
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(shim.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    }
    let failure: SingleFlightError = run_bounded_git(
        Command::new(shim.as_path()),
        "hash-object --stdin",
        Duration::from_secs(2),
        None,
        Duration::from_millis(500),
        scratch_environment().as_slice(),
        Some(vec![b'x'; 1 << 20]),
        MAX_PUSH_OUTPUT,
    )
    .expect_err("the request was cut, so the answer is not the command's answer to it");
    match &failure {
        SingleFlightError::Git { arguments, code, stderr } => {
            assert_eq!(arguments, "hash-object --stdin");
            assert_eq!(*code, None);
            assert!(stderr.contains("could not be written whole"), "{stderr}");
            assert!(stderr.contains("took none of its input"), "{stderr}");
        }
        other => panic!("expected a Git failure, got {other}"),
    }
    crate::test_support::remove(root.as_path());
}

/// What:
///  A command that reads a prefix of its input,
/// closes its standard input and answers with status zero is not an answer
///  to the whole request.
/// Why:
///   The answer was made from the part of the request that arrived.
/// The input is larger than the pipe holds,
/// so the writer still had bytes to write
///  when the command closed its end,
/// and those bytes are exactly what the answer never saw.
#[cfg(unix)]
#[test]
fn a_command_that_closes_its_input_early_is_not_an_answer() {
    let root: std::path::PathBuf = crate::test_support::fixture("query-input-closed-early");
    let shim: std::path::PathBuf = root.join("command-that-closes-its-input");
    // It reads a prefix, closes standard input, prints an answer and exits zero.
    std::fs::write(shim.as_path(), "#!/bin/sh\nhead -c 10 > /dev/null\nexec 0<&-\necho answer\nexit 0\n").expect("stand-in");
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(shim.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    }
    let failure: SingleFlightError = run_bounded_git(
        Command::new(shim.as_path()),
        "hash-object --stdin",
        // The window main's control was written against is the only bound it needs: no total ceiling and the push report's cap.
        Duration::from_secs(30),
        None,
        Duration::from_millis(500),
        scratch_environment().as_slice(),
        Some(vec![b'x'; 1 << 20]),
        MAX_PUSH_OUTPUT,
    )
    .expect_err("the request was cut, so the answer is not the command's answer to it");
    match &failure {
        SingleFlightError::Git { arguments, code, stderr } => {
            assert_eq!(arguments, "hash-object --stdin");
            assert_eq!(*code, None);
            assert!(stderr.contains("could not be written whole"), "{stderr}");
            assert!(stderr.contains("not written"), "{stderr}");
        }
        other => panic!("expected a Git failure, got {other}"),
    }
    crate::test_support::remove(root.as_path());
}

/// What:
///  A request that was written whole is not reported as cut when the writer publishes its count after the command has
///  ended.
/// Why:
///   The write of the bytes and the count the answer reads are two steps,
/// so a command that read everything,
/// closed its
///  input and exited zero can be seen before that count is published;
/// the answer then has to come from the writer's own
///  finished result,
/// within a bound,
/// and not from a count it may not have published yet.
#[cfg(unix)]
#[test]
fn a_request_written_whole_is_not_cut_when_the_writer_publishes_late() {
    let root: std::path::PathBuf = crate::test_support::fixture("query-writer-publishes-late");
    let shim: std::path::PathBuf = root.join("command-that-reads-it-all");
    let marker: std::path::PathBuf = root.join("the-command-exited");
    // It reads all ten bytes, closes its input, prints its answer, leaves the marker and exits zero.
    std::fs::write(
        shim.as_path(),
        format!("#!/bin/sh\nhead -c 10 > /dev/null\nexec 0<&-\necho answer\n: > '{}'\nexit 0\n", marker.display()),
    )
    .expect("stand-in");
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(shim.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    }
    // The writer is held between its write and the count it publishes until this control lets it go.
    let (release, held) = std::sync::mpsc::channel::<()>();
    let hold = move || {
        let _ = held.recv_timeout(Duration::from_secs(10));
    };
    let environment: Vec<(OsString, OsString)> = scratch_environment();
    let path: std::path::PathBuf = shim.clone();
    let (answered, answer) = std::sync::mpsc::channel::<Result<QueryOutput, SingleFlightError>>();
    let running = std::thread::spawn(move || {
        let _sent = answered.send(run_bounded_git_with_hold(
            Command::new(path.as_path()),
            "hash-object --stdin",
            Duration::from_secs(30),
            Duration::from_millis(500),
            environment.as_slice(),
            Some(vec![b'x'; 10]),
            Box::new(hold),
        ));
    });
    // Wait until the command has ended: it read the whole request before it marked itself done.
    let began = std::time::Instant::now();
    while !marker.exists() && began.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(marker.exists(), "the stand-in ran");
    // The answer must still be waiting for the writer's own result, and not read the count that is still to come.
    let early: Result<Result<QueryOutput, SingleFlightError>, std::sync::mpsc::RecvTimeoutError> = answer.recv_timeout(Duration::from_millis(150));
    assert!(early.is_err(), "the answer waited for the writer to finish: {early:?}");
    let _released = release.send(());
    let output: QueryOutput = answer
        .recv_timeout(Duration::from_secs(10))
        .expect("the answer comes once the writer is let go")
        .expect("the request was written whole, so its answer is the command's");
    assert_eq!(output.code, Some(0), "the command's own status is the answer");
    assert_eq!(output.stdout, b"answer\n".to_vec());
    running.join().expect("the query thread ends");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A writer still held at the ceiling is the case the count names,
/// and the answer comes at that ceiling instead of waiting for the writer.
/// Why:
///   The wait for the writer's own result has a bound of its own,
/// because a writer that stays blocked must not hold the
///  answer for as long as the command's window:
/// the answer is the prefix count it can see,
/// and the words say so.
///  A control that removed the bound would wait for the release below,
/// which this one never sends before it has the answer.
#[cfg(unix)]
#[test]
fn a_writer_still_held_at_the_ceiling_is_the_cut_request() {
    let root: std::path::PathBuf = crate::test_support::fixture("query-writer-held-past-the-ceiling");
    let shim: std::path::PathBuf = root.join("command-that-reads-it-all");
    let marker: std::path::PathBuf = root.join("the-command-exited");
    // It reads all ten bytes, closes its input, prints its answer, leaves the marker and exits zero.
    std::fs::write(
        shim.as_path(),
        format!("#!/bin/sh\nhead -c 10 > /dev/null\nexec 0<&-\necho answer\n: > '{}'\nexit 0\n", marker.display()),
    )
    .expect("stand-in");
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(shim.as_path(), std::fs::Permissions::from_mode(0o755)).expect("executable");
    }
    // The writer stays between its write and its count until this control lets it go, which is after the answer.
    let (release, held) = std::sync::mpsc::channel::<()>();
    let hold = move || {
        let _ = held.recv_timeout(Duration::from_secs(30));
    };
    let environment: Vec<(OsString, OsString)> = scratch_environment();
    let path: std::path::PathBuf = shim.clone();
    let (answered, answer) = std::sync::mpsc::channel::<Result<QueryOutput, SingleFlightError>>();
    let running = std::thread::spawn(move || {
        let _sent = answered.send(run_bounded_git_with_hold(
            Command::new(path.as_path()),
            "hash-object --stdin",
            Duration::from_secs(30),
            Duration::from_millis(500),
            environment.as_slice(),
            Some(vec![b'x'; 10]),
            Box::new(hold),
        ));
    });
    let began = std::time::Instant::now();
    while !marker.exists() && began.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(marker.exists(), "the stand-in ran");
    // The answer arrives while the writer is still held: the wait for it ends at the ceiling.
    let failure: SingleFlightError = answer
        .recv_timeout(WRITER_SETTLE_CEILING + Duration::from_secs(5))
        .expect("the answer ends at the ceiling, and does not wait for the writer to be let go")
        .expect_err("the count the answer can see is a prefix, so the request is reported cut");
    match &failure {
        SingleFlightError::Git { arguments, code, stderr } => {
            assert_eq!(arguments, "hash-object --stdin");
            assert_eq!(*code, None);
            assert!(stderr.contains("could not be written whole"), "{stderr}");
            assert!(stderr.contains("after 0 of 10 bytes"), "{stderr}");
        }
        other => panic!("expected a Git failure, got {other}"),
    }
    let _released = release.send(());
    running.join().expect("the query thread ends");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A system that refuses every thread leaves a query's request unwritten and says which limit refused it.
/// Why:
///  The write half of a push query is a thread of its own, because a command may answer before it reads its
///  input (`bounded_query_input.rs`); without the writer the command reads nothing and its answer is not about
///  the request. A person who meets this has to be told the limit that refused the thread, because nothing in
///  the command can be changed to work around it. The child is refused every thread for real (`RLIMIT_NPROC`),
///  and the reader it writes to is started while a process may still start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // under prlimit --nproc=1: expect(feedStandardInput(pipe, bytes)).rejects.toContain('ulimit -u');
/// ```
#[cfg(unix)]
#[test]
fn where_no_thread_can_start_the_input_writer_names_the_limit() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_the_input_writer_names_the_limit") {
        return;
    }
    // The reader is a process of its own, which the refusal refuses as well, so it starts while the limit still allows it.
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises");
    let mut reader: Command = shell("sleep 20");
    let mut reading: Child = reader.stdin(Stdio::piped()).spawn().expect("the reader starts");
    let pipe: ChildStdin = reading.stdin.take().expect("the input");
    crate::no_threads_child_test_support::refuse_threads_again().expect("the process limit falls");
    let refused: Result<InputWriter, String> = feed_standard_input(Some(pipe), b"request".to_vec(), Duration::from_secs(1), None);
    let _killed = reading.kill();
    let _waited = reading.wait();
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises again");
    let said: String = refused.err().expect("no thread could be started to give the command its request");
    assert!(said.contains("ulimit -u") && said.contains("pids.max"), "the refusal names the limits that refused the thread: {said}");
}
