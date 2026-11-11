//! Verifies that a scope keeps the limits of its origin while it lives:
//! what is recorded when processes join,
//! what is applied when the origin changes in either direction,
//! what is forgotten when a scope ends,
//! what is reported when the limits cannot be read or applied,
//! and that a hung manager costs a scan one call,
//! a failing origin waits out a backoff,
//! and a scan stops at its time budget.

/// The limits a scope carries.
use crate::inherited_limits::InheritedLimits;
/// The clock the registry reads.
use crate::routing_support::Clock;
/// Seam implemented by the scripted fake.
use crate::scope_adoption::{AdoptError, AdoptRequest, Adopter, Joined, LimitRequest};
/// Naming and layout.
use crate::scope_naming::{origin_scope_name, scope_cgroup_directory};
/// Registry under test.
use crate::scope_registry::ScopeRegistry;
/// Outcomes under test.
use crate::update_report::Outcome;
/// The engine's setup.
use crate::update_routing::RoutingSetup;
/// Shared recording between the test and the fake.
use std::cell::{Cell, RefCell};
/// Scripted results in order.
use std::collections::VecDeque;
/// Fixture paths.
use std::path::{Path, PathBuf};
/// Shared recording between the test and the fake.
use std::rc::Rc;
/// Time that a test moves by hand.
use std::time::{Duration, Instant};

/// The fixture user's id.
const UID: u32 = 1000;
/// Slice used by the tests.
const SLICE: &str = "app-monochromatic\\x2dupdates.slice";
/// The origin cgroup of the cap daemon.
const ORIGIN: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash";

/// A clock that only the test moves.
struct FakeClock {
    /// The time now.
    now: Cell<Instant>,
}

/// The fake clock.
impl Clock for FakeClock {
    /// The time the test set.
    fn now(&self) -> Instant {
        return self.now.get();
    }
}

/// Records the limits applied and answers from a script,
/// then succeeds.
/// Each call moves the fake clock by `step`,
/// the time a call to the manager takes.
struct ScriptedAdopter {
    /// The scope and limits of each `apply_limits` call in order.
    applied: Rc<RefCell<Vec<(String, InheritedLimits)>>>,
    /// Results for the next calls.
    script: RefCell<VecDeque<Result<(), AdoptError>>>,
    /// The clock to move.
    clock: Rc<FakeClock>,
    /// How long each call takes.
    step: Duration,
}

/// The scripted fake.
impl Adopter for ScriptedAdopter {
    /// The slice always exists.
    fn ensure_slice(&self, _uid: u32, _gid: u32, _slice: &str, _slice_directory: &Path) -> std::io::Result<()> {
        return Ok(());
    }

    /// No scope of an origin runs in this fixture.
    fn existing_scope_name(&self, _uid: u32, _slice: &str, _origin: &str) -> Option<String> {
        return None;
    }

    /// Adoption is not under test here.
    fn adopt(&self, _request: &AdoptRequest<'_>) -> Result<Joined, AdoptError> {
        return Err(AdoptError::Failed("adoption is not under test".to_string()));
    }

    /// Records the limits,
    /// takes its time
    /// and answers from the script.
    fn apply_limits(&self, request: &LimitRequest<'_>) -> Result<(), AdoptError> {
        self.applied.borrow_mut().push((request.scope_name.to_string(), request.limits.clone()));
        self.clock.now.set(self.clock.now.get() + self.step);
        return self.script.borrow_mut().pop_front().unwrap_or(Ok(()));
    }
}

/// Registry,
/// setup
/// and fake trees for one test.
struct Fixture {
    /// Registry under test.
    registry: ScopeRegistry,
    /// Setup the registry reads.
    setup: RoutingSetup,
    /// Limits applied through the fake.
    applied: Rc<RefCell<Vec<(String, InheritedLimits)>>>,
    /// The clock the registry reads.
    clock: Rc<FakeClock>,
    /// The origin cgroup paths,
    /// in the order they were made.
    origins: Vec<String>,
    /// The cgroup root of the fake tree.
    cgroup_root: PathBuf,
}

/// The path of the origin with the given index.
fn origin_path(index: usize) -> String {
    return if index == 0 { ORIGIN.to_string() } else { format!("{ORIGIN}-{index}") };
}

/// Builds a fixture with the given number of origins,
/// each holding the given `cpu.max` and having a running scope.
fn fixture(name: &str, origins: usize, cpu_max: &str, step: Duration, script: Vec<Result<(), AdoptError>>) -> Fixture {
    let base = crate::test_scratch::scratch_root().join(format!("wg-quicker-registry-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&base);
    let cgroup_root = base.join("cgroup");
    let paths: Vec<String> = (0..origins).map(origin_path).collect();
    for origin in &paths {
        let directory = cgroup_root.join(origin.trim_start_matches('/'));
        std::fs::create_dir_all(&directory).expect("make the origin");
        std::fs::write(directory.join("cpu.max"), cpu_max).expect("write the origin limit");
        std::fs::create_dir_all(scope_cgroup_directory(&cgroup_root, UID, SLICE, &origin_scope_name(origin, UID))).expect("make the scope");
    }
    let applied = Rc::new(RefCell::new(Vec::new()));
    let clock = Rc::new(FakeClock { now: Cell::new(Instant::now()) });
    let setup = RoutingSetup {
        uid: UID,
        gid: UID,
        install_roots: vec![],
        proc_root: base.join("proc"),
        cgroup_root: cgroup_root.clone(),
        adopter: Box::new(ScriptedAdopter { applied: Rc::clone(&applied), script: RefCell::new(script.into()), clock: Rc::clone(&clock), step }),
    };
    return Fixture { registry: ScopeRegistry::new(), setup, applied, clock, origins: paths, cgroup_root };
}

/// The limits of an origin that caps the CPU at the given microseconds per second.
fn cpu(quota: u64) -> InheritedLimits {
    return InheritedLimits { cpu_quota_per_sec_usec: Some(quota), ..InheritedLimits::default() };
}

/// Notes that processes joined the scope of the origin with the given index.
/// Returns whether the manager did not answer in time.
fn join(fixture: &mut Fixture, index: usize, joined: &Joined, limits: &InheritedLimits) -> bool {
    let origin = fixture.origins[index].clone();
    let now = fixture.clock.now.get();
    return fixture.registry.note_joined(&fixture.setup, SLICE, &origin, &origin_scope_name(&origin, UID), joined, limits, now);
}

/// Makes the origin with the given index have a new `cpu.max`.
fn set_origin_cpu(fixture: &Fixture, index: usize, cpu_max: &str) {
    let directory = fixture.cgroup_root.join(fixture.origins[index].trim_start_matches('/'));
    std::fs::write(directory.join("cpu.max"), cpu_max).expect("change the origin limit");
}

/// Runs one follow-up scan.
fn follow(fixture: &mut Fixture) -> crate::scope_registry::FollowResult {
    return fixture.registry.follow_origins(&fixture.setup, SLICE, fixture.clock.as_ref());
}

/// Moves the fake clock.
fn advance(fixture: &Fixture, by: Duration) {
    fixture.clock.now.set(fixture.clock.now.get() + by);
}

/// A scope this watcher created has the limits it was created with,
/// so following an unchanged origin applies nothing.
#[test]
fn a_created_scope_is_not_given_limits_it_already_has() {
    let mut fixture = fixture("created", 1, "2800000 100000\n", Duration::ZERO, vec![]);
    join(&mut fixture, 0, &Joined::Created, &cpu(28_000_000));
    assert!(follow(&mut fixture).outcomes.is_empty());
    assert!(fixture.applied.borrow().is_empty());
}

/// A tightened origin is applied to the live scope,
/// once.
#[test]
fn a_tightened_origin_is_applied_to_the_live_scope_once() {
    let mut fixture = fixture("tighten", 1, "2800000 100000\n", Duration::ZERO, vec![]);
    join(&mut fixture, 0, &Joined::Created, &cpu(28_000_000));
    set_origin_cpu(&fixture, 0, "800000 100000\n");
    assert!(follow(&mut fixture).outcomes.is_empty());
    assert_eq!(fixture.applied.borrow().iter().map(|(_scope, limits)| return limits.clone()).collect::<Vec<_>>(), vec![cpu(8_000_000)]);
    assert!(follow(&mut fixture).outcomes.is_empty());
    assert_eq!(fixture.applied.borrow().len(), 1, "an unchanged origin is not applied again");
}

/// A relaxed origin is followed too,
/// down to no limit at all,
/// because the owner's rule is that the scope has the origin's cap and not the cap it was born with.
#[test]
fn a_relaxed_origin_is_followed_to_unlimited() {
    let mut fixture = fixture("relax", 1, "800000 100000\n", Duration::ZERO, vec![]);
    join(&mut fixture, 0, &Joined::Created, &cpu(8_000_000));
    set_origin_cpu(&fixture, 0, "max 100000\n");
    follow(&mut fixture);
    assert_eq!(fixture.applied.borrow()[0].1, InheritedLimits::default());
}

/// A scope found running that this watcher did not create has unknown limits,
/// so the origin's are applied when processes join it,
/// once.
#[test]
fn a_scope_found_running_is_given_the_origins_limits_when_processes_join() {
    let mut fixture = fixture("found", 1, "2800000 100000\n", Duration::ZERO, vec![]);
    join(&mut fixture, 0, &Joined::Attached, &cpu(28_000_000));
    assert_eq!(fixture.applied.borrow().len(), 1);
    assert_eq!(fixture.applied.borrow()[0].1, cpu(28_000_000));
    join(&mut fixture, 0, &Joined::Attached, &cpu(28_000_000));
    assert_eq!(fixture.applied.borrow().len(), 1, "known limits are not applied again");
}

/// A scope that ended is forgotten without a call,
/// and a scope that ended during the call is forgotten too.
#[test]
fn a_scope_that_ended_is_forgotten() {
    let mut fixture = fixture("ended", 1, "2800000 100000\n", Duration::ZERO, vec![]);
    join(&mut fixture, 0, &Joined::Created, &cpu(28_000_000));
    std::fs::remove_dir_all(scope_cgroup_directory(&fixture.cgroup_root, UID, SLICE, &origin_scope_name(&fixture.origins[0], UID))).expect("the scope ends");
    assert!(follow(&mut fixture).outcomes.is_empty());
    assert_eq!(fixture.registry.len(), 0);
    assert!(fixture.applied.borrow().is_empty());
    let mut ending = fixture_with_gone_result("ended-during");
    join(&mut ending, 0, &Joined::Created, &cpu(28_000_000));
    set_origin_cpu(&ending, 0, "800000 100000\n");
    assert!(follow(&mut ending).outcomes.is_empty());
    assert_eq!(ending.registry.len(), 0);
}

/// A fixture whose adopter answers that the scope is gone.
fn fixture_with_gone_result(name: &str) -> Fixture {
    return fixture(name, 1, "2800000 100000\n", Duration::ZERO, vec![Err(AdoptError::ScopeGone)]);
}

/// An origin whose directory is gone keeps the limits of its scope,
/// and the record stays.
#[test]
fn an_origin_that_is_gone_keeps_the_scopes_limits() {
    let mut fixture = fixture("origin-gone", 1, "2800000 100000\n", Duration::ZERO, vec![]);
    join(&mut fixture, 0, &Joined::Created, &cpu(28_000_000));
    std::fs::remove_dir_all(fixture.cgroup_root.join(fixture.origins[0].trim_start_matches('/'))).expect("the origin goes");
    assert!(follow(&mut fixture).outcomes.is_empty());
    assert!(fixture.applied.borrow().is_empty());
    assert_eq!(fixture.registry.len(), 1);
}

/// A failed application is reported loudly,
/// and tried again once its backoff is over,
/// until it succeeds.
#[test]
fn a_failed_application_is_reported_and_retried_after_a_backoff() {
    let mut fixture = fixture("retry", 1, "2800000 100000\n", Duration::ZERO, vec![Err(AdoptError::Failed("the manager refused".to_string()))]);
    join(&mut fixture, 0, &Joined::Created, &cpu(28_000_000));
    set_origin_cpu(&fixture, 0, "800000 100000\n");
    let first = follow(&mut fixture);
    assert!(matches!(first.outcomes.as_slice(), [Outcome::ScopeLimits { scope, error }] if *scope == origin_scope_name(ORIGIN, UID) && error.contains("the manager refused")), "{first:?}");
    assert!(first.outcomes[0].failure_kind().is_some());
    assert!(!first.timed_out);
    assert!(follow(&mut fixture).outcomes.is_empty());
    assert_eq!(fixture.applied.borrow().len(), 1, "no second call during the backoff");
    advance(&fixture, Duration::from_secs(1));
    assert!(follow(&mut fixture).outcomes.is_empty());
    assert_eq!(fixture.applied.borrow().len(), 2, "tried again after the backoff and the fake accepted");
    assert!(follow(&mut fixture).outcomes.is_empty());
    assert_eq!(fixture.applied.borrow().len(), 2, "applied limits are not applied again");
}

/// The wait doubles with every failure in a row.
#[test]
fn the_backoff_grows_with_repeated_failures() {
    let refusal = || return Err(AdoptError::Failed("refused".to_string()));
    let mut fixture = fixture("growing", 1, "2800000 100000\n", Duration::ZERO, vec![refusal(), refusal(), refusal()]);
    join(&mut fixture, 0, &Joined::Created, &cpu(28_000_000));
    set_origin_cpu(&fixture, 0, "800000 100000\n");
    follow(&mut fixture);
    advance(&fixture, Duration::from_millis(999));
    follow(&mut fixture);
    assert_eq!(fixture.applied.borrow().len(), 1, "one second after the first failure");
    advance(&fixture, Duration::from_millis(1));
    follow(&mut fixture);
    assert_eq!(fixture.applied.borrow().len(), 2);
    advance(&fixture, Duration::from_millis(1_999));
    follow(&mut fixture);
    assert_eq!(fixture.applied.borrow().len(), 2, "two seconds after the second failure");
    advance(&fixture, Duration::from_millis(1));
    follow(&mut fixture);
    assert_eq!(fixture.applied.borrow().len(), 3);
}

/// Origin limits that cannot be read are reported,
/// and nothing is applied on a guess.
#[test]
fn unreadable_origin_limits_are_reported_and_nothing_is_guessed() {
    let mut fixture = fixture("unreadable", 1, "2800000 100000\n", Duration::ZERO, vec![]);
    join(&mut fixture, 0, &Joined::Created, &cpu(28_000_000));
    set_origin_cpu(&fixture, 0, "garbage\n");
    let result = follow(&mut fixture);
    assert!(matches!(result.outcomes.as_slice(), [Outcome::ScopeLimits { error, .. }] if error.contains("could not be read")), "{result:?}");
    assert!(fixture.applied.borrow().is_empty());
}

/// A manager that does not answer costs a scan one call:
/// the scan stops at the first timeout instead of asking about every other origin,
/// and says so,
/// so the engine pauses its moves.
#[test]
fn a_hung_manager_costs_a_scan_one_call_and_pauses_the_engine() {
    // The call is quick in the fake, so only the timeout rule and not the time budget can stop the scan.
    let mut fixture = fixture("hung", 3, "2800000 100000\n", Duration::from_millis(100), vec![Err(AdoptError::TimedOut("busctl did not finish".to_string()))]);
    for index in 0..3 {
        join(&mut fixture, index, &Joined::Created, &cpu(28_000_000));
        set_origin_cpu(&fixture, index, "800000 100000\n");
    }
    let result = follow(&mut fixture);
    assert!(result.timed_out);
    assert_eq!(fixture.applied.borrow().len(), 1, "the other origins were not asked");
    assert!(matches!(result.outcomes.as_slice(), [Outcome::ScopeLimits { error, .. }] if error.contains("did not finish")), "{result:?}");
}

/// A failing origin waits out its backoff while the others go on.
#[test]
fn a_failing_origin_waits_while_the_others_are_followed() {
    let mut fixture = fixture("others", 2, "2800000 100000\n", Duration::ZERO, vec![Err(AdoptError::Failed("refused".to_string()))]);
    for index in 0..2 {
        join(&mut fixture, index, &Joined::Created, &cpu(28_000_000));
        set_origin_cpu(&fixture, index, "800000 100000\n");
    }
    let first = follow(&mut fixture);
    assert_eq!(first.outcomes.len(), 1);
    assert_eq!(fixture.applied.borrow().len(), 2, "the first failed and the second was applied");
    advance(&fixture, Duration::from_millis(500));
    follow(&mut fixture);
    assert_eq!(fixture.applied.borrow().len(), 2, "the failed origin is still waiting");
    advance(&fixture, Duration::from_millis(500));
    follow(&mut fixture);
    assert_eq!(fixture.applied.borrow().len(), 3, "the failed origin is tried again after its second");
}

/// A scan stops at its time budget and leaves the rest for the next one.
#[test]
fn a_scan_stops_at_its_time_budget_and_the_next_one_goes_on() {
    let mut fixture = fixture("budget", 4, "2800000 100000\n", Duration::from_millis(600), vec![]);
    for index in 0..4 {
        join(&mut fixture, index, &Joined::Created, &cpu(28_000_000));
        set_origin_cpu(&fixture, index, "800000 100000\n");
    }
    follow(&mut fixture);
    assert_eq!(fixture.applied.borrow().len(), 2, "two calls of 600 ms used the one second budget");
    follow(&mut fixture);
    assert_eq!(fixture.applied.borrow().len(), 4, "the next scan applied the rest");
    follow(&mut fixture);
    assert_eq!(fixture.applied.borrow().len(), 4);
}

/// Processes that join a scope whose application is waiting out a backoff do not trigger a call,
/// and a join that finds the manager hung says so.
#[test]
fn joining_does_not_ask_the_manager_during_a_backoff_and_reports_a_timeout() {
    let mut fixture = fixture("join", 1, "2800000 100000\n", Duration::ZERO, vec![Err(AdoptError::TimedOut("busctl did not finish".to_string()))]);
    assert!(join(&mut fixture, 0, &Joined::Attached, &cpu(28_000_000)), "the manager did not answer in time");
    assert_eq!(fixture.applied.borrow().len(), 1);
    assert!(!join(&mut fixture, 0, &Joined::Attached, &cpu(28_000_000)));
    assert_eq!(fixture.applied.borrow().len(), 1, "no call during the backoff");
    advance(&fixture, Duration::from_secs(1));
    assert!(!join(&mut fixture, 0, &Joined::Attached, &cpu(28_000_000)));
    assert_eq!(fixture.applied.borrow().len(), 2);
}
