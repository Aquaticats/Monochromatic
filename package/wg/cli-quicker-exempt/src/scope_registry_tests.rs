//! Verifies that a scope keeps the limits of its origin while it lives:
//! what is recorded when processes join,
//! what is applied when the origin changes in either direction,
//! what is forgotten when a scope ends,
//! and what is reported when the limits cannot be read or applied.

/// The limits a scope carries.
use crate::inherited_limits::InheritedLimits;
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
use std::cell::RefCell;
/// Scripted results in order.
use std::collections::VecDeque;
/// Fixture paths.
use std::path::{Path, PathBuf};
/// Shared recording between the test and the fake.
use std::rc::Rc;

/// The fixture user's id.
const UID: u32 = 1000;
/// Slice used by the tests.
const SLICE: &str = "app-monochromatic\\x2dupdates.slice";
/// The origin cgroup of the cap daemon.
const ORIGIN: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash";

/// Records the limits applied and answers from a script,
/// then succeeds.
struct ScriptedAdopter {
    /// The limits of each `apply_limits` call in order.
    applied: Rc<RefCell<Vec<InheritedLimits>>>,
    /// Results for the next calls.
    script: RefCell<VecDeque<Result<(), AdoptError>>>,
}

/// The scripted fake.
impl Adopter for ScriptedAdopter {
    /// The slice always exists.
    fn ensure_slice(&self, _uid: u32, _gid: u32, _slice: &str, _slice_directory: &Path) -> std::io::Result<()> {
        return Ok(());
    }

    /// Adoption is not under test here.
    fn adopt(&self, _request: &AdoptRequest<'_>) -> Result<Joined, AdoptError> {
        return Err(AdoptError::Failed("adoption is not under test".to_string()));
    }

    /// Records the limits and answers from the script.
    fn apply_limits(&self, request: &LimitRequest<'_>) -> Result<(), AdoptError> {
        self.applied.borrow_mut().push(request.limits.clone());
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
    applied: Rc<RefCell<Vec<InheritedLimits>>>,
    /// The scope's name.
    scope_name: String,
    /// The origin's directory in the fake cgroup tree.
    origin_directory: PathBuf,
    /// The scope's directory in the fake cgroup tree.
    scope_directory: PathBuf,
}

/// Builds a fixture whose origin holds the given `cpu.max` and whose scope exists.
fn fixture(name: &str, cpu_max: &str, script: Vec<Result<(), AdoptError>>) -> Fixture {
    let base = crate::test_scratch::scratch_root().join(format!("wg-quicker-registry-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&base);
    let cgroup_root = base.join("cgroup");
    let origin_directory = cgroup_root.join(ORIGIN.trim_start_matches('/'));
    std::fs::create_dir_all(&origin_directory).expect("make the origin");
    std::fs::write(origin_directory.join("cpu.max"), cpu_max).expect("write the origin limit");
    let scope_name = origin_scope_name(ORIGIN, UID);
    let scope_directory = scope_cgroup_directory(&cgroup_root, UID, SLICE, &scope_name);
    std::fs::create_dir_all(&scope_directory).expect("make the scope");
    let applied = Rc::new(RefCell::new(Vec::new()));
    let setup = RoutingSetup {
        uid: UID,
        gid: UID,
        install_roots: vec![],
        proc_root: base.join("proc"),
        cgroup_root,
        adopter: Box::new(ScriptedAdopter { applied: Rc::clone(&applied), script: RefCell::new(script.into()) }),
    };
    return Fixture { registry: ScopeRegistry::new(), setup, applied, scope_name, origin_directory, scope_directory };
}

/// The limits of an origin that caps the CPU at the given microseconds per second.
fn cpu(quota: u64) -> InheritedLimits {
    return InheritedLimits { cpu_quota_per_sec_usec: Some(quota), ..InheritedLimits::default() };
}

/// Notes that processes joined the scope.
fn join(fixture: &mut Fixture, joined: &Joined, limits: &InheritedLimits) {
    fixture.registry.note_joined(&fixture.setup, SLICE, ORIGIN, &fixture.scope_name, joined, limits);
}

/// A scope this watcher created has the limits it was created with,
/// so following an unchanged origin applies nothing.
#[test]
fn a_created_scope_is_not_given_limits_it_already_has() {
    let mut fixture = fixture("created", "2800000 100000\n", vec![]);
    join(&mut fixture, &Joined::Created, &cpu(28_000_000));
    assert!(fixture.registry.follow_origins(&fixture.setup, SLICE).is_empty());
    assert!(fixture.applied.borrow().is_empty());
}

/// A tightened origin is applied to the live scope,
/// once.
#[test]
fn a_tightened_origin_is_applied_to_the_live_scope_once() {
    let mut fixture = fixture("tighten", "2800000 100000\n", vec![]);
    join(&mut fixture, &Joined::Created, &cpu(28_000_000));
    std::fs::write(fixture.origin_directory.join("cpu.max"), "800000 100000\n").expect("tighten the origin");
    assert!(fixture.registry.follow_origins(&fixture.setup, SLICE).is_empty());
    assert_eq!(*fixture.applied.borrow(), vec![cpu(8_000_000)]);
    assert!(fixture.registry.follow_origins(&fixture.setup, SLICE).is_empty());
    assert_eq!(fixture.applied.borrow().len(), 1, "an unchanged origin is not applied again");
}

/// A relaxed origin is followed too,
/// down to no limit at all,
/// because the owner's rule is that the scope has the origin's cap and not the cap it was born with.
#[test]
fn a_relaxed_origin_is_followed_to_unlimited() {
    let mut fixture = fixture("relax", "800000 100000\n", vec![]);
    join(&mut fixture, &Joined::Created, &cpu(8_000_000));
    std::fs::write(fixture.origin_directory.join("cpu.max"), "max 100000\n").expect("lift the origin cap");
    fixture.registry.follow_origins(&fixture.setup, SLICE);
    assert_eq!(*fixture.applied.borrow(), vec![InheritedLimits::default()]);
}

/// A scope found running that this watcher did not create has unknown limits,
/// so the origin's are applied when processes join it,
/// once.
#[test]
fn a_scope_found_running_is_given_the_origins_limits_when_processes_join() {
    let mut fixture = fixture("found", "2800000 100000\n", vec![]);
    join(&mut fixture, &Joined::Attached, &cpu(28_000_000));
    assert_eq!(*fixture.applied.borrow(), vec![cpu(28_000_000)]);
    join(&mut fixture, &Joined::Attached, &cpu(28_000_000));
    assert_eq!(fixture.applied.borrow().len(), 1, "known limits are not applied again");
}

/// A scope that ended is forgotten without a call,
/// and a scope that ended during the call is forgotten too.
#[test]
fn a_scope_that_ended_is_forgotten() {
    let mut fixture = fixture("ended", "2800000 100000\n", vec![]);
    join(&mut fixture, &Joined::Created, &cpu(28_000_000));
    std::fs::remove_dir_all(&fixture.scope_directory).expect("the scope ends");
    assert!(fixture.registry.follow_origins(&fixture.setup, SLICE).is_empty());
    assert_eq!(fixture.registry.len(), 0);
    assert!(fixture.applied.borrow().is_empty());
    let mut ending = fixture_with_gone_result("ended-during");
    join(&mut ending, &Joined::Created, &cpu(28_000_000));
    std::fs::write(ending.origin_directory.join("cpu.max"), "800000 100000\n").expect("tighten the origin");
    assert!(ending.registry.follow_origins(&ending.setup, SLICE).is_empty());
    assert_eq!(ending.registry.len(), 0);
}

/// A fixture whose adopter answers that the scope is gone.
fn fixture_with_gone_result(name: &str) -> Fixture {
    return fixture(name, "2800000 100000\n", vec![Err(AdoptError::ScopeGone)]);
}

/// An origin whose directory is gone keeps the limits of its scope,
/// and the record stays.
#[test]
fn an_origin_that_is_gone_keeps_the_scopes_limits() {
    let mut fixture = fixture("origin-gone", "2800000 100000\n", vec![]);
    join(&mut fixture, &Joined::Created, &cpu(28_000_000));
    std::fs::remove_dir_all(&fixture.origin_directory).expect("the origin goes");
    assert!(fixture.registry.follow_origins(&fixture.setup, SLICE).is_empty());
    assert!(fixture.applied.borrow().is_empty());
    assert_eq!(fixture.registry.len(), 1);
}

/// A failed application is reported loudly,
/// and tried again on the next follow-up until it succeeds.
#[test]
fn a_failed_application_is_reported_and_retried() {
    let mut fixture = fixture("retry", "2800000 100000\n", vec![Err(AdoptError::Failed("the manager refused".to_string()))]);
    join(&mut fixture, &Joined::Created, &cpu(28_000_000));
    std::fs::write(fixture.origin_directory.join("cpu.max"), "800000 100000\n").expect("tighten the origin");
    let first = fixture.registry.follow_origins(&fixture.setup, SLICE);
    assert!(matches!(first.as_slice(), [Outcome::ScopeLimits { scope, error }] if *scope == fixture.scope_name && error.contains("the manager refused")), "{first:?}");
    assert!(first[0].failure_kind().is_some());
    assert!(fixture.registry.follow_origins(&fixture.setup, SLICE).is_empty());
    assert_eq!(fixture.applied.borrow().len(), 2, "the second follow-up tried again and the fake accepted");
    assert!(fixture.registry.follow_origins(&fixture.setup, SLICE).is_empty());
    assert_eq!(fixture.applied.borrow().len(), 2, "applied limits are not applied again");
}

/// Origin limits that cannot be read are reported,
/// and nothing is applied on a guess.
#[test]
fn unreadable_origin_limits_are_reported_and_nothing_is_guessed() {
    let mut fixture = fixture("unreadable", "2800000 100000\n", vec![]);
    join(&mut fixture, &Joined::Created, &cpu(28_000_000));
    std::fs::write(fixture.origin_directory.join("cpu.max"), "garbage\n").expect("corrupt the origin limit");
    let outcomes = fixture.registry.follow_origins(&fixture.setup, SLICE);
    assert!(matches!(outcomes.as_slice(), [Outcome::ScopeLimits { error, .. }] if error.contains("could not be read")), "{outcomes:?}");
    assert!(fixture.applied.borrow().is_empty());
}
