//! What:
//! Helpers the tests of the delegated scope share:
//! a scope started the way a run starts it,
//! stopped when the test ends however it ends,
//! and the proof that a test used delegated containment.
//! Why:
//! A test that needs a delegated scope never passes on a host that cannot give it one:
//! it panics with a line that names the test and says the test did not run,
//! because a runner counts a test that returns early as passed.
//! The package's `test` task reads those lines:
//! it asks the question first,
//! through `cgroup_probe_tests::probe_0_the_host_can_give_the_tests_a_delegated_scope`,
//! and where the answer is no it leaves those tests out of the run by name,
//! so its totals are of the tests that ran.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function delegatedRoute(): Route  // throws, naming the test, where none can be had
//! export function startInScope(command: Command): TestScope
//! ```

/// Reads the records the control group writes.
use std::fs;
/// The owned path of the scope directory a test creates.
use std::path::PathBuf;
/// The launcher a test starts inside the scope,
/// with its standard streams.
use std::process::{Child, Command, Stdio};
/// The wait bound,
/// and the clock polled until the scope reaches the state a test expects.
use std::time::{Duration, Instant};

/// The control-group types the tests drive,
/// and the detection of the route they take.
use crate::control_group::{ControlGroup, Record, Route, RouteSettings, detect_route, parse_record};

/// How long a test gives a scope to stop.
const TEST_STOP: Duration = Duration::from_secs(10);

/// How long a test waits for a state it is looking for.
const STATE_WAIT: Duration = Duration::from_secs(20);

/// The prefix of the line a test that cannot run prints,
/// which names the test between brackets.
/// A reader or a script can find it, and the `test` task reads it to leave those tests out of its run by name.
pub(crate) const NOT_RUN_PREFIX: &str = "cargo-shadow tests: not run [";

/// What:
/// `pub(crate) fn delegated_route() -> Route` finds the route of this host,
/// and panics with a line naming this test where the host has none.
/// Why:
/// See the module comment:
/// no test may pass where the host cannot give it a scope,
/// and a runner reports an early return as a pass,
/// so the test fails with the reason and the command that runs it elsewhere instead.
/// The test's own name is the name the runner gave its thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function delegatedRoute(): Route  // throws, naming the test, where none can be had
/// ```
pub(crate) fn delegated_route() -> Route {
    return match detect_route(&RouteSettings::default()) {
        Ok(found) => found,
        Err(unavailable) => {
            let test: String = std::thread::current().name().map(str::to_string).unwrap_or_else(|| { return "<a test the runner did not name>".to_string(); });
            panic!("{NOT_RUN_PREFIX}{test}]: this host cannot give it a delegated control group ({unavailable}); it runs where a user manager answers on a cgroup v2 mount, for example inside `systemd-run --user --scope --property=Delegate=yes -- mise run //package/cargo-shadow/cli:test`");
        }
    };
}

/// What:
/// `pub(crate) fn this_test_needs_a_delegated_scope()` says that the test it starts in cannot pass without a route,
/// and panics like `delegated_route` where the host has none.
/// Why:
/// Most tests that need a scope do not read the route,
/// so they say what they need in one line that a reader understands,
/// instead of discarding a value.
pub(crate) fn this_test_needs_a_delegated_scope() {
    let _route: Route = delegated_route();
}

/// A scope started the way a run starts it,
/// stopped when it drops.
pub(crate) struct TestScope {
    /// The program started in the scope (the process that became it).
    pub(crate) child: Child,
    /// The scope.
    pub(crate) group: ControlGroup,
}

/// How the guard cleans up.
impl Drop for TestScope {
    /// What:
    /// `fn drop(&mut self)` kills the program,
    /// stops the scope and reaps the program.
    /// Why:
    /// A failed assertion must not leave a process behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.group.stop(TEST_STOP);
        let _ = self.child.wait();
    }
}

/// What:
/// `pub(crate) fn start_in_scope(command: &mut Command) -> TestScope` starts the command in a fresh delegated scope,
/// through the same launcher a run uses,
/// and returns once the program is in the scope (a member is listed).
/// Why:
/// The kernel-behavior probes need a scope with a program in it,
/// without the runner around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function startInScope(command: Command): TestScope
/// ```
pub(crate) fn start_in_scope(command: &mut Command) -> TestScope {
    let scope: TestScope = start_in_scope_without_waiting(command);
    wait_until("the program in the scope", &|| { return scope.group.members().map(|found| { return !found.is_empty(); }).unwrap_or(false); });
    return scope;
}

/// What:
/// `pub(crate) fn start_in_scope_without_waiting(command: &mut Command) -> TestScope` starts the command in a fresh
/// delegated scope and returns at once.
/// Why:
/// A program that leaves the scope as its first act (the escape probe) is never listed as a member when the caller looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function startInScopeWithoutWaiting(command: Command): TestScope
/// ```
pub(crate) fn start_in_scope_without_waiting(command: &mut Command) -> TestScope {
    let route: Route = delegated_route();
    let group: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let mut launch: Command = group.launcher(&route, command, false);
    launch.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::inherit());
    let child: Child = launch.spawn().expect("starts the launcher");
    return TestScope { child, group };
}

/// What:
/// `pub(crate) fn wait_until(what: &str, condition: &dyn Fn() -> bool)` waits for a condition,
/// up to twenty seconds.
/// Why:
/// A handshake with a state of the system,
/// not a sleep:
/// it returns at the first turn the condition holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function waitUntil(what: string, condition: () => boolean): void
/// ```
pub(crate) fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let deadline: Instant = Instant::now() + STATE_WAIT;
    while !condition() {
        assert!(Instant::now() < deadline, "{what} did not happen in time");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// What:
/// `pub(crate) fn run_until_the_payload_started<T>(first: Duration, reset: &dyn Fn(), started: &dyn Fn() -> bool, attempt: &mut dyn FnMut(Duration) -> T) -> (T, Duration)`
/// runs `attempt` with a bound,
/// and repeats it with a doubled bound while the payload did not start inside it.
/// Why:
/// The bound of a run counts from the start of the run,
/// scope startup included.
/// A bound that ended while the scope was still being
/// made says nothing about stopping a payload at its bound;
/// on a host whose disk or user manager is busy that is what a short
/// bound measures (a bound of 400 ms failed a full run for it).
/// The attempt that is judged is the first in which the payload
/// really started;
/// each earlier attempt is reset and repeated,
/// so the wait follows the event and not the clock.
/// An attempt where the payload does not start even with the longest bound is returned too,
/// and the caller's assertions fail on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runUntilThePayloadStarted<T>(first: number, reset: () => void, started: () => boolean, attempt: (limit: number) => T): [T, number]
/// ```
pub(crate) fn run_until_the_payload_started<T>(first: Duration, reset: &dyn Fn(), started: &dyn Fn() -> bool, attempt: &mut dyn FnMut(Duration) -> T) -> (T, Duration, Duration) {
    let mut limit: Duration = first;
    let mut began: Instant = Instant::now();
    let mut result: T = attempt(limit);
    for _doubling in 0..LONGEST_ESCALATION {
        if started() {
            break;
        }
        reset();
        limit *= 2;
        began = Instant::now();
        result = attempt(limit);
    }
    // The judged attempt's own duration comes with it, so a test can still require the run to have ended at its bound (fifth
    // certification pass: retrying must not disable the bounded-lifetime observation).
    return (result, limit, began.elapsed());
}

/// How many times a bound is doubled while the payload has not started (400 ms doubled six times is about 26 s).
const LONGEST_ESCALATION: u32 = 6;

/// What:
/// `pub(crate) fn assert_scope_record(groups_file: &str)` asserts that a journal's record names a delegated scope.
/// Why:
/// The strong-path tests must show that they used delegated containment and not the weaker path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function assertScopeRecord(groupsFile: string): void
/// ```
pub(crate) fn assert_scope_record(groups_file: &str) {
    let text: String = fs::read_to_string(groups_file).expect("the record");
    let first: &str = text.lines().next().expect("a first record line");
    assert!(matches!(parse_record(first), Ok(Record::Scope { .. })), "the run did not use a delegated scope: {text:?}");
}

/// What:
/// `pub(crate) fn scope_directory_of(groups_file: &str) -> PathBuf` is the path of the first scope a journal recorded.
/// Why:
/// A test that left a writer in a scope stops it through the recorded path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function scopeDirectoryOf(groupsFile: string): string
/// ```
pub(crate) fn scope_directory_of(groups_file: &str) -> PathBuf {
    let text: String = fs::read_to_string(groups_file).expect("the record");
    for line in text.lines() {
        if let Ok(Record::Scope { path, .. }) = parse_record(line) {
            return path;
        }
    }
    panic!("no scope record in {text:?}");
}

/// What:
/// `pub(crate) fn wrapper_group() -> String` is the control group line of this process (the wrapper) as `/proc/self/cgroup` gives it.
/// Why:
/// A test that claims a run used a delegated scope must show the program's group differs from the wrapper's own,
/// which may itself be
/// a scope of the harness (9b 3.5).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function wrapperGroup(): string
/// ```
pub(crate) fn wrapper_group() -> String {
    return fs::read_to_string("/proc/self/cgroup").expect("the wrapper's own group").trim().to_string();
}

/// What:
/// `pub(crate) fn assert_ran_in_recorded_scope(groups_file: &str, program_group: &str, wrapper_group: &str)` asserts that the program
/// ran in the very scope the journal recorded for the run,
/// and not in the group the wrapper already had.
/// Why:
/// Finding a route is not selecting it,
/// and a group that ends in `.scope` is not the run's scope (9b 3.5):
/// a downgraded run inherits
/// the harness's own scope and would satisfy a suffix check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function assertRanInRecordedScope(groupsFile: string, programGroup: string, wrapperGroup: string): void
/// ```
pub(crate) fn assert_ran_in_recorded_scope(groups_file: &str, program_group: &str, wrapper_group: &str) {
    let recorded: PathBuf = scope_directory_of(groups_file);
    let relative: &str = recorded.to_str().expect("a UTF-8 scope path").strip_prefix(crate::control_group::CGROUP_ROOT).expect("a scope below the cgroup root");
    assert_eq!(program_group.trim(), format!("0::{relative}"), "the program did not run in the scope the journal recorded for the run");
    assert_ne!(program_group.trim(), wrapper_group.trim(), "the program ran in the wrapper's own group");
}
