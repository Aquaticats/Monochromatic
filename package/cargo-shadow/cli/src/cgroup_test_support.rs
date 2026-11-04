//! What:
//! Helpers the tests of the delegated scope share:
//! a scope started the way a run starts it, stopped when the test ends however it ends,
//! and the proof that a test used delegated containment.
//! Why:
//! A test that needs a delegated scope must fail loudly where none can be had, not pass without proving anything;
//! `CARGO_SHADOW_ALLOW_NO_CGROUP=1` turns that into a printed skip for hosts that cannot delegate (some CI runners).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function delegatedRoute(): Route | undefined
//! export function startInScope(command: Command): TestScope | undefined
//! ```

use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::control_group::{ControlGroup, Record, Route, RouteSettings, detect_route, parse_record};

/// The environment variable that lets the tests that need a delegated scope skip, with a printed reason, where none can be had.
pub(crate) const ALLOW_NO_CGROUP: &str = "CARGO_SHADOW_ALLOW_NO_CGROUP";

/// How long a test gives a scope to stop.
const TEST_STOP: Duration = Duration::from_secs(10);

/// How long a test waits for a state it is looking for.
const STATE_WAIT: Duration = Duration::from_secs(20);

/// What:
/// `pub(crate) fn delegated_route() -> Option<Route>` finds the route of this host,
/// or panics with the missing capability,
/// or returns `None` after printing a skip when `CARGO_SHADOW_ALLOW_NO_CGROUP` is set.
/// Why:
/// See the module comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function delegatedRoute(): Route | undefined
/// ```
pub(crate) fn delegated_route() -> Option<Route> {
    return match detect_route(&RouteSettings::default()) {
        Ok(found) => Some(found),
        Err(unavailable) => {
            assert!(
                std::env::var_os(ALLOW_NO_CGROUP).is_some(),
                "this host cannot give the test a delegated scope ({unavailable}); set {ALLOW_NO_CGROUP}=1 to skip the tests that need one"
            );
            eprintln!("SKIPPED: no delegated scope ({unavailable})");
            None
        }
    };
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
    /// `fn drop(&mut self)` kills the program, stops the scope and reaps the program.
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
/// `pub(crate) fn start_in_scope(command: &mut Command) -> Option<TestScope>` starts the command in a fresh delegated scope,
/// through the same launcher a run uses,
/// and returns once the program is in the scope (a member is listed).
/// Why:
/// The kernel-behavior probes need a scope with a program in it, without the runner around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function startInScope(command: Command): TestScope | undefined
/// ```
pub(crate) fn start_in_scope(command: &mut Command) -> Option<TestScope> {
    let scope: TestScope = start_in_scope_without_waiting(command)?;
    wait_until("the program in the scope", &|| { return scope.group.members().map(|found| { return !found.is_empty(); }).unwrap_or(false); });
    return Some(scope);
}

/// What:
/// `pub(crate) fn start_in_scope_without_waiting(command: &mut Command) -> Option<TestScope>` starts the command in a fresh
/// delegated scope and returns at once.
/// Why:
/// A program that leaves the scope as its first act (the escape probe) is never listed as a member when the caller looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function startInScopeWithoutWaiting(command: Command): TestScope | undefined
/// ```
pub(crate) fn start_in_scope_without_waiting(command: &mut Command) -> Option<TestScope> {
    let route: Route = delegated_route()?;
    let group: ControlGroup = ControlGroup::plan(&route).expect("a scope name");
    let mut launch: Command = group.launcher(&route, command, false);
    launch.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::inherit());
    let child: Child = launch.spawn().expect("starts the launcher");
    return Some(TestScope { child, group });
}

/// What:
/// `pub(crate) fn wait_until(what: &str, condition: &dyn Fn() -> bool)` waits for a condition, up to twenty seconds.
/// Why:
/// A handshake with a state of the system, not a sleep: it returns at the first turn the condition holds.
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
