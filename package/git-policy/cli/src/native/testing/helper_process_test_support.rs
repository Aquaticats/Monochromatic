//! What:
//!  The one shape for a control that runs in a process of its own:
//!  a fresh fixture,
//!  this test binary re-started on exactly that control with an environment variable naming the fixture,
//!  a verdict file the helper writes the sentences it failed with,
//!  and a bounded wait on both.
//! Why:
//!  A handler, a signal disposition and a recorded stop belong to a process,
//!  and a crate's tests run as threads of one process,
//!  so a control that installs one or leaves one set reaches every other control of a threaded run
//!  (`process/signal_relay_tests.rs`, `service/service_client_interrupted_tests.rs`, issue 6463).
//!  Written once here, every such control answers the same way:
//!  the helper reports every check instead of panicking,
//!  so the verdict always exists and a parent never waits for a file a panic kept from being written,
//!  and every wait is bounded,
//!  so a helper that hangs fails its control within the step timeout instead of holding the whole run.
//! Gotcha:
//!  The variable holds a directory, so the helper body is entered by the same test function:
//!  `helper_directory` there answers the fixture and the body runs,
//!  and in the parent it answers nothing and the control starts the helper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = inAHelper('MODE_VARIABLE', 'fixture-name', 'module::tests::the_control'); // string[] of failures
//! ```

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use super::child_test_process::{ChildTest, started};
use super::test_support::{fixture, remove};

/// The file a helper writes the sentences it failed with to,
/// one per line.
///  An empty file is a helper that passed.
pub(crate) const HELPER_VERDICT: &str = "verdict";

/// How long a control waits on its helper before it fails,
/// so a helper that hangs fails the control within a minute instead of holding the whole run.
pub(crate) const HELPER_STEP_TIMEOUT: Duration = Duration::from_secs(60);

/// How often a wait here looks again.
const LOOK_EVERY: Duration = Duration::from_millis(10);

/// What:
///  `fn helper_directory(variable: &str) -> Option<PathBuf>` answers the fixture directory when this process is the helper
///  `variable` names,
///  and nothing in the control that started it.
///  `Option<PathBuf>` is "a path or nothing" (sibling: `PathBuf`, which needs a second value to say "none").
/// Why:
///  The helper body is the same test function as the control,
///  so the variable is what tells the two apart,
///  and one branch at the top of the control keeps the body and the parent's assertions in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function helperDirectory(variable: string): string | undefined
/// ```
pub(crate) fn helper_directory(variable: &str) -> Option<PathBuf> {
    let named: Option<OsString> = std::env::var_os(variable);
    return named.map(PathBuf::from);
}

/// What:
///  `fn verdict_in_a_helper(directory: &Path, failures: Vec<String>) -> i32` writes the helper's verdict and answers the
///  exit status the helper ends with.
///  `i32` is a 32-bit signed integer (siblings: `u32`, `i64`), which is the shape a process exit status has.
/// Why:
///  The parent reads the sentences,
///  and a status that agrees with them means a parent that read only one of the two still learns the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verdictInAHelper(directory: string, failures: string[]): number
/// ```
pub(crate) fn verdict_in_a_helper(directory: &Path, failures: Vec<String>) -> i32 {
    std::fs::write(directory.join(HELPER_VERDICT), failures.join("\n")).expect("the verdict record");
    return if failures.is_empty() { 0 } else { 1 };
}

/// What:
///  `fn settle_after_a_helper(helper: Child, root: &Path) -> Vec<String>` waits until the helper wrote its verdict and ended,
///  and answers the sentences it failed with,
///  plus one when it ended with a status that is not success.
/// Why:
///   A wait that only knew the step timeout would spend the whole bound on a helper that ended early,
///  and the failure would name the wrong thing;
///  the sentences the helper wrote name what it checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleAfterAHelper(helper: Child, root: string): string[] {
///   await waitBounded(() => exists(join(root, VERDICT)), stepTimeout, () => helper.exited);
///   const failures = readLines(join(root, VERDICT)); const status = await waitFor(() => helper.exited, stepTimeout);
///   if (!status.success) failures.push(`the helper ended with ${status}`); return failures;
/// }
/// ```
pub(crate) fn settle_after_a_helper(mut helper: Child, root: &Path) -> Vec<String> {
    let verdict: PathBuf = root.join(HELPER_VERDICT);
    let begun: Instant = Instant::now();
    while !verdict.exists() {
        assert!(begun.elapsed() < HELPER_STEP_TIMEOUT, "{} never appeared", verdict.display());
        if let Some(status) = helper.try_wait().expect("the helper's status") {
            panic!("the helper ended before {} appeared: {status:?}", verdict.display());
        }
        std::thread::sleep(LOOK_EVERY);
    }
    let text: String = std::fs::read_to_string(verdict.as_path()).expect("the verdict");
    let ending_from: Instant = Instant::now();
    let status: std::process::ExitStatus = loop {
        if let Some(status) = helper.try_wait().expect("the helper's status") {
            break status;
        }
        assert!(ending_from.elapsed() < HELPER_STEP_TIMEOUT, "the helper did not end");
        std::thread::sleep(LOOK_EVERY);
    };
    let mut failures: Vec<String> = text.lines().map(String::from).collect();
    if !status.success() {
        failures.push(format!("the helper ended with {status:?}"));
    }
    return failures;
}

/// What:
///  `fn in_a_helper(variable: &str, fixture_name: &str, exact_test: &str) -> Vec<String>` starts this test binary on
///  `exact_test` with `variable` naming a fresh fixture,
///  and answers the sentences that helper failed with.
/// Why:
///  A control's body runs where its process-wide state cannot reach another control of the run,
///  and `spawn_owned` puts the helper in a process group of its own,
///  which the test thread's fixtures end however the control ends,
///  so a helper still waiting cannot outlive the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inAHelper(variable: string, fixtureName: string, exactTest: string): string[]
/// ```
pub(crate) fn in_a_helper(variable: &str, fixture_name: &str, exact_test: &str) -> Vec<String> {
    return in_a_helper_then(variable, fixture_name, exact_test, |_root, _helper| {});
}

/// What:
///  `fn in_a_helper_then(variable, fixture_name, exact_test, before) -> Vec<String>` starts the helper like `in_a_helper`,
///  runs `before` with the fixture's path and the running helper,
///  and then answers the sentences the helper failed with.
/// Why:
///   A control whose helper must be told something at the right moment (a signal once it is inside the read it measures)
///  needs the fixture and the process the shared helper owns,
///  so it is handed them instead of starting a second helper of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inAHelperThen(variable, fixtureName, exactTest, before: (root, helper) => void): string[]
/// ```
pub(crate) fn in_a_helper_then(variable: &str, fixture_name: &str, exact_test: &str, before: impl FnOnce(&Path, &mut Child)) -> Vec<String> {
    let root: PathBuf = fixture(fixture_name);
    // The helper answers nothing the parent reads from a stream: everything it reports goes through the verdict file.
    let environment: [(&str, &std::ffi::OsStr); 1] = [(variable, root.as_os_str())];
    let mut helper: Child = started(ChildTest {
        exact_test,
        environment: environment.as_slice(),
        standard_output: std::process::Stdio::null(),
        standard_error: std::process::Stdio::null(),
        ..ChildTest::default()
    });
    before(root.as_path(), &mut helper);
    let failures: Vec<String> = settle_after_a_helper(helper, root.as_path());
    remove(root.as_path());
    return failures;
}

/// What:
///  `fn wait_for_file_while_a_helper_runs(path: &Path, helper: &mut Child)` waits until the helper wrote `path`,
///  and fails at the step timeout or when the helper ended first.
/// Why:
///   A control that answers its helper at a moment of the helper's own choosing waits for a file the helper writes,
///  and a wait that only knew the timeout would spend the whole bound on a helper that failed early.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitForFileWhileAHelperRuns(path: string, helper: Child): void
/// ```
pub(crate) fn wait_for_file_while_a_helper_runs(path: &Path, helper: &mut Child) {
    let begun: Instant = Instant::now();
    while !path.exists() {
        assert!(begun.elapsed() < HELPER_STEP_TIMEOUT, "{} never appeared", path.display());
        if let Some(status) = helper.try_wait().expect("the helper's status") {
            panic!("the helper ended before {} appeared: {status:?}", path.display());
        }
        std::thread::sleep(LOOK_EVERY);
    }
}
