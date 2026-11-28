//! Unit tests of the program-start seam:
//! the real runner on harmless programs,
//! the toolchain environment,
//! and the
//! recording fake.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use super::fakes::FakeRunner;
use super::runner::{
    ProcessRunner,
    RunRequest,
    Runner,
    toolchain_path_env,
};
use super::scope::{
    EXPOSURE_DIR_RELATIVE,
    EnvMap,
    repo_scope,
};
use crate::test_scratch::scratch;

#[test]
fn the_real_runner_captures_output_and_the_exit_code() {
    let outcome = ProcessRunner.run(&RunRequest::new(
        "sh",
        &["-c", "echo out; echo err 1>&2; exit 3"],
    ));
    assert_eq!(outcome.stdout, "out\n");
    assert_eq!(outcome.stderr, "err\n");
    assert_eq!(outcome.code, 3);
    assert!(!outcome.succeeded());
    assert_eq!(outcome.failure_text(), "err");
}

#[test]
fn failure_text_falls_back_to_standard_output() {
    let outcome = ProcessRunner.run(&RunRequest::new("sh", &["-c", "echo only-out; exit 1"]));
    assert_eq!(outcome.failure_text(), "only-out");
}

#[test]
fn a_program_that_cannot_start_reports_code_one_and_the_reason() {
    let outcome = ProcessRunner.run(&RunRequest::new(
        "/nonexistent/definitely-not-a-program",
        &[],
    ));
    assert_eq!(outcome.code, 1);
    assert!(
        outcome
            .stderr
            .contains("cannot start /nonexistent/definitely-not-a-program"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn an_explicit_environment_replaces_the_inherited_one() {
    let mut env = BTreeMap::new();
    env.insert("ONLY_THIS".to_string(), "yes".to_string());
    env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
    let outcome = ProcessRunner
        .run(&RunRequest::new("sh", &["-c", "echo \"$ONLY_THIS:${HOME:-unset}\""]).with_env(env));
    assert_eq!(outcome.stdout, "yes:unset\n");
}

#[test]
fn the_working_directory_is_honoured() {
    let directory = scratch();
    let outcome =
        ProcessRunner.run(&RunRequest::new("pwd", &[]).with_cwd(directory.path().to_path_buf()));
    let printed = std::fs::canonicalize(outcome.stdout.trim()).expect("a real directory");
    assert_eq!(
        printed,
        std::fs::canonicalize(directory.path()).expect("a real directory")
    );
}

#[test]
fn the_toolchain_environment_sets_home_and_leads_path_with_the_scope_directories() {
    let env = EnvMap::from_pairs(&[
        ("HOME", "/real/home"),
        ("PATH", "/usr/bin:/bin"),
        ("KEEP", "me"),
        ("CARGO_HOME", "/cargo"),
    ]);
    let scope = repo_scope(std::path::Path::new("/work"), &env).expect("a scope");
    let merged = toolchain_path_env(&scope, &env);
    assert_eq!(merged.get("HOME").map(String::as_str), Some("/real/home"));
    assert_eq!(merged.get("KEEP").map(String::as_str), Some("me"));
    let path = merged.get("PATH").expect("PATH");
    let parts: Vec<&str> = path.split(':').collect();
    assert_eq!(
        parts[0],
        Path::new("/work")
            .join(EXPOSURE_DIR_RELATIVE)
            .to_str()
            .expect("a path of utf-8 names")
    );
    assert!(parts.contains(&"/cargo/bin"));
    assert_eq!(&parts[parts.len() - 2..], &["/usr/bin", "/bin"]);
}

#[test]
fn the_fake_records_requests_answers_by_prefix_and_defaults_to_success() {
    let runner = FakeRunner::new();
    runner.answer("cargo binstall", "installed", 0);
    runner.fail("rustup toolchain install", "no such toolchain");
    let installed = runner.run(&RunRequest::new(
        "/some/dir/cargo",
        &["binstall", "--no-confirm", "fd-find"],
    ));
    assert_eq!(installed.stdout, "installed");
    let failed = runner.run(&RunRequest::new(
        "rustup",
        &["toolchain", "install", "nightly"],
    ));
    assert_eq!(failed.code, 1);
    assert_eq!(failed.stderr, "no such toolchain");
    let unmatched = runner.run(&RunRequest::new("anything", &["else"]));
    assert!(unmatched.succeeded());
    assert_eq!(
        runner.lines(),
        vec![
            "cargo binstall --no-confirm fd-find".to_string(),
            "rustup toolchain install nightly".to_string(),
            "anything else".to_string()
        ]
    );
    assert_eq!(
        runner.calls()[0].line(),
        "/some/dir/cargo binstall --no-confirm fd-find"
    );
}

/// What:
/// How a control reads the lines the wait for a program prints,
/// without writing them to the test's own standard error.
/// Why:
/// The one line a long wait prints is the behaviour under test,
/// so a control has to see it,
/// and the shipped call's sink writes to standard error rather than to anything a test can read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said: string[] = [];
/// const say = (line: string) => { said.push(line); };
/// ```
fn the_lines_said() -> (std::rc::Rc<std::cell::RefCell<Vec<String>>>, impl Fn(&str)) {
    let said: std::rc::Rc<std::cell::RefCell<Vec<String>>> =
        std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let sink = std::rc::Rc::clone(&said);
    let say = move |line: &str| {
        sink.borrow_mut().push(line.to_string());
    };
    return (said, say);
}

/// Starts `sh -c <script>` and waits for it as the real runner does,
/// with a bound a control can wait out and a sink the control can read.
fn wait_for_a_shell(
    script: &str,
    bound: std::time::Duration,
    say: &dyn Fn(&str),
) -> (std::process::Output, RunRequest) {
    let request = RunRequest::new("sh", &["-c", script]);
    let child = std::process::Command::new(&request.command)
        .args(&request.args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("start the stand-in");
    let output = super::runner::wait_for_the_program(child, &request, bound, say)
        .expect("read the stand-in's answer");
    return (output, request);
}

/// A wait that ends within the bound says nothing:
/// the line is for a wait that has outlasted an ordinary answer,
/// not for every start.
#[test]
fn a_wait_that_ends_within_the_bound_is_silent() {
    let (said, say) = the_lines_said();
    let (output, _) = wait_for_a_shell("echo answered", Duration::from_secs(30), &say);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "answered\n");
    assert!(
        said.borrow().is_empty(),
        "an ordinary answer prints no line about waiting: {:?}",
        said.borrow()
    );
}

/// What:
/// The one line a wait prints once it has passed the bound,
/// which names the program and every argument it was started with.
/// Why:
/// A run whose `rustup` never answers prints nothing else,
/// so this line is the only thing that tells a person what is outstanding;
/// the command it names has to be the one the run is waiting on.
#[test]
fn a_wait_that_passes_the_bound_says_what_is_being_waited_for() {
    let (said, say) = the_lines_said();
    let bound = Duration::from_millis(50);
    let (output, request) = wait_for_a_shell("sleep 0.4; echo answered", bound, &say);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "answered\n",
        "the answer is still read whole"
    );
    let lines: Vec<String> = said.borrow().clone();
    assert_eq!(lines.len(), 1, "the wait is named once: {lines:?}");
    assert!(
        lines[0].contains(&format!("still waiting for `{}` to answer", request.line())),
        "the line names the command the run waits on: {lines:?}"
    );
    assert!(
        lines[0].contains(&format!(
            "{:.1} s after this command started it",
            bound.as_secs_f64()
        )),
        "the line says how long the bound is, so a reader can judge the wait: {lines:?}"
    );
    assert!(
        lines[0].contains("no deadline ends that wait"),
        "the line says no cut-off follows, which is what the run does: {lines:?}"
    );
}

/// A wait that outlasts the bound several times over is named once,
/// not once per bound:
/// a program that runs for a minute prints one line.
#[test]
fn a_wait_that_outlasts_the_bound_twice_is_still_named_once() {
    let (said, say) = the_lines_said();
    let (output, _) = wait_for_a_shell("sleep 0.6; echo answered", Duration::from_millis(50), &say);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "answered\n");
    assert_eq!(
        said.borrow().len(),
        1,
        "one line however long the wait lasts: {:?}",
        said.borrow()
    );
}

/// The output of a program that prints more than a pipe holds is read while it runs,
/// which is what the wait's own reading thread is for:
/// a child blocked on a full pipe would otherwise never end,
/// and no wait would end with it.
#[test]
fn a_programs_whole_output_is_read_even_when_it_is_larger_than_a_pipe() {
    let (said, say) = the_lines_said();
    let (output, _) = wait_for_a_shell(
        "i=0; while [ $i -lt 4000 ]; do echo 0123456789012345678901234567890123456789; i=$((i+1)); done",
        Duration::from_secs(30),
        &say,
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).lines().count(),
        4000,
        "every line the program printed is in the answer"
    );
    assert!(
        said.borrow().is_empty(),
        "reading a large answer needs no line about waiting: {:?}",
        said.borrow()
    );
}

/// What:
/// The shipped bound is the one the measurement supports,
/// and the wait for an ordinary answer is shorter than it.
/// Why:
/// The line is meant for a wait that has outlasted an ordinary call,
/// so the constant has to sit far above what an ordinary answer costs;
/// the measurement in `runner.rs` puts an ordinary `rustup` question at a median of 12.85 ms
/// and its slowest of 26.58 ms,
/// and this control holds the shipped bound above both by a wide margin.
#[test]
fn the_shipped_bound_is_far_above_an_ordinary_answer() {
    // What: the slowest of the forty ordinary `which` and `component list` answers this release measured,
    //  26.58 ms, rounded up to the next whole millisecond.
    // Why: the bound exists for a wait that has outlasted an ordinary answer, so the margin to that answer is the
    //  property under test; the figure is the round's measurement, not a value this test chose.
    let the_slowest_ordinary_answer_measured: Duration = Duration::from_millis(27);
    assert!(
        super::runner::A_WAIT_LONGER_THAN_ORDINARY >= the_slowest_ordinary_answer_measured * 20,
        "the bound is at least twenty times the slowest ordinary question measured: {:?}",
        super::runner::A_WAIT_LONGER_THAN_ORDINARY
    );
}

/// What:
/// The shipped bound is the two seconds the record promises,
/// not merely a value far above an ordinary answer.
/// Why:
/// `README.md` says the line arrives "once it has passed 2 s",
/// and the control above holds only that the bound is at least twenty times an ordinary answer,
/// so a bound of 600 ms passed it while every sentence of the record said 2 s.
/// The number here is the record's figure, which is what the control holds the code to.
#[test]
fn the_shipped_bound_is_the_two_seconds_the_record_promises() {
    // What: the bound the record promises, as a `Duration`.
    // Why: a comparison of the shipped constant with this value is what fails when the constant moves
    //  without the record.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const thePromisedBound = 2000;
    // ```
    let the_promised_bound: Duration = Duration::from_millis(2000);
    assert_eq!(
        super::runner::A_WAIT_LONGER_THAN_ORDINARY,
        the_promised_bound,
        "the bound the README promises is the one that ships"
    );
}

/// What:
/// The script the fixture below runs:
/// it sleeps past the shipped bound and then answers.
/// Why:
/// The control has to see the line the shipped runner prints,
/// which only a wait longer than
/// `A_WAIT_LONGER_THAN_ORDINARY` reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_PROGRAM_THAT_DOES_NOT_ANSWER_IN_TIME = 'sleep 2.4; echo answered';
/// ```
const A_PROGRAM_THAT_DOES_NOT_ANSWER_IN_TIME: &str = "sleep 2.4; echo answered";

/// The name this fixture is filtered by,
/// the way the control below starts it.
const THE_PROGRAM_THAT_DOES_NOT_ANSWER_IN_TIME: &str =
    "toolchain::runner_tests::the_program_that_does_not_answer_in_time";

/// What:
/// A program the shipped runner waits for,
/// started in a process of its own.
/// Why:
/// The line the wait prints goes to standard error of whichever process waits,
/// and a test cannot read its own standard error,
/// so the fixture runs in a child whose standard error the control reads.
/// The child runs with `--nocapture`,
/// because the test harness otherwise captures what the fixture's own thread writes
/// and the control would read nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('the program that does not answer in time', () => { const outcome = ProcessRunner.run(sh(A_PROGRAM_THAT_DOES_NOT_ANSWER_IN_TIME)); expect(outcome.stdout).toBe('answered\n'); });
/// ```
#[test]
#[ignore = "a fixture a control starts: it waits for a program past the shipped bound"]
fn the_program_that_does_not_answer_in_time() {
    let outcome = ProcessRunner.run(&RunRequest::new(
        "sh",
        &["-c", A_PROGRAM_THAT_DOES_NOT_ANSWER_IN_TIME],
    ));
    assert_eq!(outcome.stdout, "answered\n", "{outcome:?}");
    assert_eq!(outcome.code, 0, "{outcome:?}");
}

/// What:
/// The line the shipped runner prints for a wait that passes its own bound,
/// read from the standard error of a process that ran it.
/// Why:
/// The controls above pass their own bound and their own sink,
/// so none of them holds that the shipped program prints the line at all or that its bound is reachable;
/// this one starts the fixture with nothing changed.
#[test]
fn the_shipped_runner_names_a_program_whose_answer_passes_the_bound() {
    let output = std::process::Command::new(
        std::env::current_exe().expect("the test binary's own path"),
    )
    .arg("--exact")
    .arg(THE_PROGRAM_THAT_DOES_NOT_ANSWER_IN_TIME)
    .arg("--ignored")
    .arg("--nocapture")
    .arg("--test-threads=1")
    .output()
    .expect("start the fixture the shipped runner waits for");
    assert!(
        output.status.success(),
        "the fixture's own wait ended with an answer: {:?} {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let printed: String = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        printed.contains(&format!(
            "still waiting for `sh -c {A_PROGRAM_THAT_DOES_NOT_ANSWER_IN_TIME}` to answer"
        )),
        "the shipped runner names the program it waits for: {printed}"
    );
    assert_eq!(
        printed.matches("still waiting for").count(),
        1,
        "one line however long the wait lasts: {printed}"
    );
}

/// What:
/// Starts the fixture above with the standard error the caller gives it,
/// and answers what the fixture's process did.
/// Why:
/// The line the wait prints goes to the fixture's standard error,
/// so the only way to hold what happens when that stream refuses the line is to give the fixture
/// a stream that refuses it,
/// and the fixture is the shipped runner started with nothing changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theFixtureWith(stderr: Stdio): { status: string; stdout: string; stderr: string }
/// ```
fn the_fixture_with_standard_error_on(standard_error: std::process::Stdio) -> std::process::Output {
    return std::process::Command::new(
        std::env::current_exe().expect("the test binary's own path"),
    )
    .arg("--exact")
    .arg(THE_PROGRAM_THAT_DOES_NOT_ANSWER_IN_TIME)
    .arg("--ignored")
    .arg("--nocapture")
    .arg("--test-threads=1")
    .stderr(standard_error)
    .output()
    .expect("start the fixture the shipped runner waits for");
}

/// What:
/// The fixture's own assertions, in one place:
/// it ran ("1 passed")
/// and ended successfully.
/// Why:
/// A control that only reads the status cannot tell a fixture that passed from one that never ran
/// (a filter that matched nothing ends 0);
/// the count is what shows the fixture ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertTheFixtureRan(output: Output): void
/// ```
fn assert_the_fixture_ran(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "the fixture's wait ended with an answer even though a line it printed was refused: {:?} {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let printed: String = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        printed.contains("1 passed"),
        "the fixture ran and passed, so the wait it holds really happened: {printed}"
    );
}

/// What:
/// A wait past the shipped bound whose line standard error refuses
/// (`/dev/full`, an open stream that answers every write `ENOSPC`)
/// still ends with the program's answer and a successful command.
/// Why:
/// `0.2.7-alpha.20` wrote the line with `eprintln!`,
/// which panics on a write error,
/// and the release profile sets `panic = "abort"`,
/// so an install whose standard error was a full disk stopped with `SIGABRT` at the first program
/// that took more than 2 s.
/// The line is a courtesy and not the command's result,
/// so the command has to finish in this state as it did in `0.2.7-alpha.19`.
#[test]
fn a_wait_notice_a_full_standard_error_refuses_does_not_end_the_command() {
    // What: `/dev/full` opened for writing: the one stream on this machine that is open and refuses
    //  every byte with `ENOSPC`.
    // Why: the defect needs a stream that answers an error rather than a closed descriptor,
    //  because the standard library ignores a closed one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const full = fs.openSync('/dev/full', 'w');
    // ```
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("/dev/full is open for writing on this machine");
    let output = the_fixture_with_standard_error_on(std::process::Stdio::from(full));
    assert_the_fixture_ran(&output);
}

/// What:
/// A wait past the shipped bound whose line a pipe refuses
/// (a pipe whose reader has left: every write answers `EPIPE`)
/// still ends with the program's answer and a successful command.
/// Why:
/// A caller that stops reading (`| head`, a wrapper that exits) leaves exactly this stream,
/// and `0.2.7-alpha.20` ended such an install with `SIGABRT` at the first long wait.
#[test]
fn a_wait_notice_a_departed_reader_refuses_does_not_end_the_command() {
    // What: the fixture started with a pipe for its standard error, which the next line then closes
    //  on this side (dropping the read end is what makes the writer answer `EPIPE`).
    // Why: this is the state a pipeline creates when its reader exits early,
    //  and it is the second state the release aborted in.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const child = spawn(exe, args, { stdio: ['ignore', 'pipe', 'pipe'] });
    // child.stderr.destroy();
    // ```
    let mut child = std::process::Command::new(
        std::env::current_exe().expect("the test binary's own path"),
    )
    .arg("--exact")
    .arg(THE_PROGRAM_THAT_DOES_NOT_ANSWER_IN_TIME)
    .arg("--ignored")
    .arg("--nocapture")
    .arg("--test-threads=1")
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .expect("start the fixture the shipped runner waits for");
    // What: `child.stderr.take()` takes the read end out of the child handle and the discard drops it,
    //  which closes the only reader the pipe had.
    // Why: the fixture keeps the write end and must meet a refusal on its first line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // child.stderr.destroy();
    // ```
    let _ = child.stderr.take();
    let output = child
        .wait_with_output()
        .expect("read what the fixture printed");
    assert_the_fixture_ran(&output);
}
