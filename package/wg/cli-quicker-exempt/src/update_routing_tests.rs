//! Verifies the update traffic policy on a fake procfs tree with allowed and rejected fixtures:
//! what is moved,
//! what is left alone,
//! how failures are retried,
//! and what the moved scope carries.

/// The slice name.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// The limits a scope carries.
use crate::inherited_limits::InheritedLimits;
/// Seam implemented by the scripted fake.
use crate::scope_adoption::{AdoptError, AdoptRequest, Adopter, Joined, LimitRequest};
/// Names and layout of the scopes.
use crate::scope_naming::{origin_scope_name, scope_cgroup_directory};
/// Install roots of the fixture user and the match reasons.
use crate::install_roots::install_roots;
/// Why a process matched.
use crate::update_entry_points::MatchReason;
/// Outcomes under test.
use crate::update_report::Outcome;
/// Policy under test.
use crate::routing_support::{adoptable_cgroup, retry_backoff, NeverStop, StopCheck, SystemClock, RETRY_BACKOFF_BASE, RETRY_BACKOFF_MAX};
/// Policy under test.
use crate::update_routing::{RoutingSetup, UpdateRouting, MEMBER_VANISH_LIMIT};
/// Shared recording between the test and the engine.
use std::cell::RefCell;
/// Scripted results in order.
use std::collections::VecDeque;
/// Fixture paths.
use std::path::{Path, PathBuf};
/// Shared recording between the test and the engine.
use std::rc::Rc;
/// Retry timing.
use std::time::{Duration, Instant};

/// The fixture user's id.
const UID: u32 = 1000;
/// The fixture user's primary group,
/// different from the group of the fake processes.
const USER_GID: u32 = 1500;
/// The group of the fake processes.
const PROCESS_GID: u32 = 1001;
/// A cgroup inside the user manager's subtree that the watcher attaches no mark to itself,
/// so the process watch has to move its processes.
/// A named terminal scope is not one of them:
/// its own cgroup carries the mark.
const SCOPE: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash";

/// What the fake adopter was asked to do.
#[derive(Debug, Eq, PartialEq)]
struct Recorded {
    /// Name of the scope of the origin.
    scope_name: String,
    /// Pids requested.
    pids: Vec<i32>,
    /// Group id passed for the privilege drop.
    gid: u32,
    /// Limits the scope should carry.
    limits: InheritedLimits,
    /// The origin cgroup the processes left.
    origin: String,
    /// Why the process matched.
    reason: MatchReason,
}

/// Records adoptions and answers from a script,
/// then succeeds.
struct ScriptedAdopter {
    /// Requests in order.
    requests: Rc<RefCell<Vec<Recorded>>>,
    /// Results for the next adoptions.
    script: RefCell<VecDeque<Result<Joined, AdoptError>>>,
    /// The scope and limits of each `apply_limits` call in order.
    applied: Rc<RefCell<Vec<(String, InheritedLimits)>>>,
    /// Results for the next `apply_limits` calls.
    apply_script: Rc<RefCell<VecDeque<Result<(), AdoptError>>>>,
}

/// The scripted fake.
impl Adopter for ScriptedAdopter {
    /// The slice always exists.
    fn ensure_slice(&self, _uid: u32, _gid: u32, _slice: &str, _slice_directory: &Path) -> std::io::Result<()> {
        return Ok(());
    }

    /// These fixtures keep no scope under an earlier spelling of its name.
    fn existing_scope_name(&self, _uid: u32, _slice: &str, _origin: &str) -> Option<String> {
        return None;
    }

    /// Records the request and answers from the script.
    fn adopt(&self, request: &AdoptRequest<'_>) -> Result<Joined, AdoptError> {
        self.requests.borrow_mut().push(Recorded {
            scope_name: request.scope_name.to_string(),
            pids: request.pids.to_vec(),
            gid: request.gid,
            limits: request.limits.clone(),
            origin: request.origin.to_string(),
            reason: request.reason,
        });
        return self.script.borrow_mut().pop_front().unwrap_or(Ok(Joined::Created));
    }

    /// Records the limits and answers from the script.
    fn apply_limits(&self, request: &LimitRequest<'_>) -> Result<(), AdoptError> {
        self.applied.borrow_mut().push((request.scope_name.to_string(), request.limits.clone()));
        return self.apply_script.borrow_mut().pop_front().unwrap_or(Ok(()));
    }
}

/// Engine,
/// recording
/// and fake trees for one test.
struct Fixture {
    /// Fake procfs root.
    proc_root: PathBuf,
    /// Fake cgroup filesystem root.
    cgroup_root: PathBuf,
    /// Recording of adoption requests.
    requests: Rc<RefCell<Vec<Recorded>>>,
    /// Recording of the limits applied to live scopes.
    applied: Rc<RefCell<Vec<(String, InheritedLimits)>>>,
    /// Results for the next `apply_limits` calls.
    apply_script: Rc<RefCell<VecDeque<Result<(), AdoptError>>>>,
    /// Engine under test.
    routing: UpdateRouting,
}

/// Builds a fixture whose adopter answers from `script`.
fn fixture(name: &str, script: Vec<Result<Joined, AdoptError>>) -> Fixture {
    let base = crate::test_scratch::scratch_root().join(format!("wg-quicker-routing-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&base);
    let proc_root = base.join("proc");
    let cgroup_root = base.join("cgroup");
    std::fs::create_dir_all(&proc_root).expect("make the fake procfs");
    std::fs::create_dir_all(&cgroup_root).expect("make the fake cgroup tree");
    let requests = Rc::new(RefCell::new(Vec::new()));
    let applied = Rc::new(RefCell::new(Vec::new()));
    let apply_script = Rc::new(RefCell::new(VecDeque::new()));
    let adopter = ScriptedAdopter { requests: Rc::clone(&requests), script: RefCell::new(script.into()), applied: Rc::clone(&applied), apply_script: Rc::clone(&apply_script) };
    let routing = UpdateRouting::new(RoutingSetup {
        uid: UID,
        gid: USER_GID,
        install_roots: install_roots(Path::new("/home/tester"), Some(Path::new("/var/home/tester"))).roots,
        proc_root: proc_root.clone(),
        cgroup_root: cgroup_root.clone(),
        adopter: Box::new(adopter),
    });
    return Fixture { proc_root, cgroup_root, requests, applied, apply_script, routing };
}

/// Describes one fake process.
#[derive(Clone)]
struct Fake<'a> {
    /// Process id.
    pid: i32,
    /// Executable path.
    exe: &'a str,
    /// Command line words.
    args: &'a [&'a str],
    /// Real user id.
    uid: u32,
    /// Unified cgroup path.
    cgroup: &'a str,
    /// Parent process id.
    parent: i32,
    /// Start time.
    start: u64,
    /// Pid namespace link text,
    /// absent when the fake exposes none.
    namespace: Option<&'a str>,
}

/// Writes one fake process into the tree.
fn make_process(proc_root: &Path, fake: &Fake<'_>) {
    let base = proc_root.join(fake.pid.to_string());
    let _removed = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("make the process directory");
    std::os::unix::fs::symlink(fake.exe, base.join("exe")).expect("link the executable");
    let mut cmdline: Vec<u8> = Vec::new();
    for word in fake.args {
        cmdline.extend_from_slice(word.as_bytes());
        cmdline.push(0);
    }
    std::fs::write(base.join("cmdline"), cmdline).expect("write cmdline");
    std::fs::write(
        base.join("status"),
        format!("Name:\tx\nUid:\t{0}\t{0}\t{0}\t{0}\nGid:\t{1}\t{1}\t{1}\t{1}\n", fake.uid, PROCESS_GID),
    )
    .expect("write status");
    std::fs::write(base.join("cgroup"), format!("0::{}\n", fake.cgroup)).expect("write cgroup");
    let zeros = vec!["0"; 17].join(" ");
    std::fs::write(base.join("stat"), format!("{} (x y) S {} {zeros} {} 0\n", fake.pid, fake.parent, fake.start)).expect("write stat");
    if let Some(namespace) = fake.namespace {
        std::fs::create_dir_all(base.join("ns")).expect("make the namespace directory");
        std::os::unix::fs::symlink(namespace, base.join("ns/pid")).expect("link the namespace");
    }
}

/// Writes the member list of a cgroup into the fake cgroup tree.
fn write_members(cgroup_root: &Path, cgroup: &str, pids: &[i32]) {
    let directory = cgroup_root.join(cgroup.trim_start_matches('/'));
    std::fs::create_dir_all(&directory).expect("make the cgroup directory");
    let text: String = pids.iter().map(|pid| return format!("{pid}\n")).collect();
    std::fs::write(directory.join("cgroup.procs"), text).expect("write the member list");
}

/// A matching mise process in an adoptable scope.
fn mise(pid: i32) -> Fake<'static> {
    return Fake {
        pid,
        exe: "/usr/bin/mise",
        args: &["mise", "run", "toolchain:install"],
        uid: UID,
        cgroup: SCOPE,
        parent: 1,
        start: 100,
        namespace: None,
    };
}

/// A process that does not match.
fn other(pid: i32, parent: i32) -> Fake<'static> {
    return Fake { pid, parent, exe: "/usr/bin/sh", args: &["sh"], ..mise(pid) };
}

/// Adopts only cgroups in the user manager's subtree that are not service main processes.
#[test]
fn only_scopes_in_the_user_manager_subtree_are_adoptable() {
    assert_eq!(adoptable_cgroup(SCOPE, UID), Ok(()));
    assert_eq!(adoptable_cgroup("/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash", UID), Ok(()));
    let session = adoptable_cgroup("/user.slice/user-1000.slice/session-3.scope", UID);
    assert!(session.expect_err("outside the subtree").contains("outside the user manager's subtree"));
    let service = adoptable_cgroup("/user.slice/user-1000.slice/user@1000.service/app.slice/app-x.service", UID);
    assert!(service.expect_err("service main process").contains("main process of the service"));
    assert!(adoptable_cgroup("/user.slice/user-2000.slice/user@2000.service/app.slice/x.scope", UID).is_err());
    assert!(adoptable_cgroup("/system.slice/x.service", UID).is_err());
}

/// Moves a matching process once and ignores processes that do not match or belong to another user.
#[test]
fn a_matching_process_is_adopted_once_and_others_are_left_alone() {
    let mut fixture = fixture("adopt", vec![]);
    make_process(&fixture.proc_root, &mise(10));
    make_process(&fixture.proc_root, &Fake { pid: 11, exe: "/usr/bin/vim", args: &["vim"], ..mise(11) });
    make_process(&fixture.proc_root, &Fake { pid: 12, uid: 2000, ..mise(12) });
    assert_eq!(fixture.routing.consider(10), Outcome::Adopted { pid: 10, reason: MatchReason::MiseTask, count: 1, left_behind: 0 });
    assert_eq!(fixture.routing.consider(10), Outcome::Ignored);
    assert_eq!(fixture.routing.consider(11), Outcome::Ignored);
    assert_eq!(fixture.routing.consider(12), Outcome::Ignored);
    assert_eq!(fixture.routing.consider(999), Outcome::Ignored);
    assert_eq!(fixture.requests.borrow().len(), 1);
    assert_eq!(fixture.requests.borrow()[0].pids, vec![10]);
}

/// The group id passed for the privilege drop is the watched user's,
/// never the matched process's.
#[test]
fn the_privilege_drop_uses_the_watched_users_group() {
    let mut fixture = fixture("gid", vec![]);
    make_process(&fixture.proc_root, &mise(10));
    assert!(matches!(fixture.routing.consider(10), Outcome::Adopted { .. }));
    assert_eq!(fixture.requests.borrow()[0].gid, USER_GID);
    assert_ne!(fixture.requests.borrow()[0].gid, PROCESS_GID);
}

/// Executables under the toolchain roots are adopted by path alone.
#[test]
fn an_executable_under_the_install_roots_is_adopted() {
    let mut fixture = fixture("path", vec![]);
    let node = "/var/home/tester/.local/share/monochromatic-dev/toolchain/node/26.13.0/bin/node";
    make_process(&fixture.proc_root, &Fake { pid: 20, exe: node, args: &["node", "x.js"], ..mise(20) });
    make_process(&fixture.proc_root, &Fake { pid: 21, exe: "/usr/bin/node", args: &["node", "x.js"], ..mise(21) });
    assert!(matches!(fixture.routing.consider(20), Outcome::Adopted { reason: MatchReason::ToolchainPath, .. }));
    assert_eq!(fixture.routing.consider(21), Outcome::Ignored);
}

/// A unit the user named after the slice is not the slice:
/// its processes are moved like any other,
/// because the policy reads the slice as a place in the cgroup tree and not as text in a name.
#[test]
fn a_unit_named_after_the_slice_is_still_moved() {
    let mut fixture = fixture("named-like-the-slice", vec![]);
    let lookalike = format!("/user.slice/user-1000.slice/user@1000.service/app.slice/{TOOLCHAIN_UPDATES_SLICE}-task.scope");
    let deeper = format!("/user.slice/user-1000.slice/user@1000.service/app.slice/{TOOLCHAIN_UPDATES_SLICE}/x.scope");
    make_process(&fixture.proc_root, &Fake { cgroup: &lookalike, ..mise(40) });
    make_process(&fixture.proc_root, &Fake { cgroup: &deeper, ..mise(41) });
    assert!(matches!(fixture.routing.consider(40), Outcome::Adopted { pid: 40, .. }));
    assert_eq!(fixture.routing.consider(41), Outcome::AlreadyInSlice);
    assert_eq!(fixture.requests.borrow().len(), 1);
}

/// The origin cgroup and the method that matched reach the adoption,
/// which is what the scope's description says for a person reading `systemctl status`.
#[test]
fn the_origin_and_the_method_reach_the_adoption() {
    let mut fixture = fixture("origin-recorded", vec![]);
    make_process(&fixture.proc_root, &mise(45));
    assert!(matches!(fixture.routing.consider(45), Outcome::Adopted { .. }));
    let recorded = fixture.requests.borrow();
    assert_eq!(recorded[0].origin, SCOPE);
    assert_eq!(recorded[0].reason, MatchReason::MiseTask);
}

/// A forged first word does not get a process moved.
#[test]
fn a_process_that_only_claims_the_name_is_not_moved() {
    let mut fixture = fixture("forged", vec![]);
    make_process(&fixture.proc_root, &Fake { pid: 25, exe: "/usr/bin/curl", args: &["mpm", "https://example.org"], ..mise(25) });
    make_process(&fixture.proc_root, &Fake { pid: 26, exe: "/usr/bin/curl", args: &["mise", "run", "toolchain:install"], ..mise(26) });
    assert_eq!(fixture.routing.consider(25), Outcome::Ignored);
    assert_eq!(fixture.routing.consider(26), Outcome::Ignored);
    assert!(fixture.requests.borrow().is_empty());
}

/// Leaves a process already in the slice alone,
/// and reports a process it cannot move by naming the policy.
#[test]
fn processes_already_in_the_slice_or_outside_the_subtree_are_not_moved() {
    let mut fixture = fixture("skip", vec![]);
    let inside = format!("/user.slice/user-1000.slice/user@1000.service/app.slice/{TOOLCHAIN_UPDATES_SLICE}/x.scope");
    make_process(&fixture.proc_root, &Fake { cgroup: &inside, ..mise(30) });
    make_process(&fixture.proc_root, &Fake { cgroup: "/user.slice/user-1000.slice/session-3.scope", ..mise(31) });
    assert_eq!(fixture.routing.consider(30), Outcome::AlreadyInSlice);
    let skipped = fixture.routing.consider(31);
    assert!(matches!(skipped, Outcome::Skipped { pid: 31, .. }));
    assert!(skipped.failure_kind().is_some());
    assert_eq!(fixture.routing.consider(31), Outcome::Ignored, "a permanent skip is reported once");
    assert!(fixture.requests.borrow().is_empty());
}

/// A process in another pid namespace is reported and left alone,
/// and one in the watcher's namespace or without a readable link is moved.
#[test]
fn a_process_in_another_pid_namespace_is_reported_and_not_moved() {
    let fixture = fixture("namespace", vec![]);
    std::fs::create_dir_all(fixture.proc_root.join("self/ns")).expect("make the self directory");
    std::os::unix::fs::symlink("pid:[4026531836]", fixture.proc_root.join("self/ns/pid")).expect("link the own namespace");
    let mut routing = UpdateRouting::new(RoutingSetup {
        uid: UID,
        gid: USER_GID,
        install_roots: vec![],
        proc_root: fixture.proc_root.clone(),
        cgroup_root: fixture.cgroup_root.clone(),
        adopter: Box::new(ScriptedAdopter { requests: Rc::clone(&fixture.requests), script: RefCell::new(VecDeque::new()), applied: Rc::clone(&fixture.applied), apply_script: Rc::clone(&fixture.apply_script) }),
    });
    make_process(&fixture.proc_root, &Fake { namespace: Some("pid:[4026533586]"), ..mise(40) });
    make_process(&fixture.proc_root, &Fake { namespace: Some("pid:[4026531836]"), ..mise(41) });
    make_process(&fixture.proc_root, &mise(42));
    let sandboxed = routing.consider(40);
    assert!(matches!(&sandboxed, Outcome::Skipped { pid: 40, why } if why.contains("another pid namespace")), "{sandboxed:?}");
    assert!(matches!(routing.consider(41), Outcome::Adopted { .. }));
    let unreadable = routing.consider(42);
    assert!(matches!(&unreadable, Outcome::Skipped { pid: 42, why } if why.contains("pid namespace cannot be read")), "{unreadable:?}");
    assert!(unreadable.failure_kind().is_some());
    assert_eq!(fixture.requests.borrow().len(), 1);
}

/// A watcher that cannot read its own pid namespace says so at start,
/// and treats every process like any other.
#[test]
fn a_watcher_without_its_own_namespace_link_says_so_and_moves_processes_as_before() {
    let mut fixture = fixture("namespace-unknown", vec![]);
    assert!(!fixture.routing.pid_namespace_known());
    make_process(&fixture.proc_root, &Fake { namespace: Some("pid:[4026533586]"), ..mise(40) });
    assert!(matches!(fixture.routing.consider(40), Outcome::Adopted { .. }));
}

/// A failing adoption is reported loudly,
/// tried again after a growing wait,
/// and never tried before the wait is over.
#[test]
fn a_failed_adoption_is_reported_and_retried_after_a_backoff() {
    let refusal = || return Err(AdoptError::Failed("the manager refused the job".to_string()));
    let mut fixture = fixture("retry", vec![refusal(), refusal()]);
    make_process(&fixture.proc_root, &mise(40));
    let start = Instant::now();
    let first = fixture.routing.consider_at(40, start);
    assert!(matches!(&first, Outcome::Failed { pid: 40, attempt: 1, .. }), "{first:?}");
    assert!(first.failure_kind().is_some());
    assert_eq!(fixture.routing.consider_at(40, start + Duration::from_millis(500)), Outcome::Ignored);
    assert_eq!(fixture.requests.borrow().len(), 1, "no attempt before the wait is over");
    fixture.routing.begin_pass_at(start + Duration::from_secs(1));
    let second = fixture.routing.consider_at(40, start + Duration::from_secs(1));
    assert!(matches!(&second, Outcome::Failed { attempt: 2, retry_in, .. } if *retry_in == RETRY_BACKOFF_BASE * 2), "{second:?}");
    assert_eq!(fixture.routing.consider_at(40, start + Duration::from_millis(2500)), Outcome::Ignored);
    fixture.routing.begin_pass_at(start + Duration::from_secs(3));
    let third = fixture.routing.consider_at(40, start + Duration::from_secs(3));
    assert!(matches!(third, Outcome::Adopted { pid: 40, .. }), "{third:?}");
    assert_eq!(fixture.routing.consider_at(40, start + Duration::from_secs(100)), Outcome::Ignored);
    assert_eq!(fixture.requests.borrow().len(), 3);
}

/// The wait doubles from the base and stops growing at the cap.
#[test]
fn the_retry_wait_doubles_up_to_a_cap() {
    let waits: Vec<u64> = (1..=9).map(|attempt| return retry_backoff(attempt).as_secs()).collect();
    let cap = RETRY_BACKOFF_MAX.as_secs();
    assert_eq!(waits[0], RETRY_BACKOFF_BASE.as_secs(), "the first wait is the base");
    assert_eq!(waits.last().copied(), Some(cap), "the wait stops growing at the cap");
    for pair in waits.windows(2) {
        let doubled = pair[0].saturating_mul(2);
        assert!(pair[1] == doubled || pair[1] == cap, "each wait doubles until the cap: {pair:?}");
    }
}

/// A process that exits while it is being moved is not a failure.
#[test]
fn a_vanished_process_is_not_reported() {
    let mut fixture = fixture("vanish", vec![Err(AdoptError::Vanished)]);
    make_process(&fixture.proc_root, &mise(40));
    assert_eq!(fixture.routing.consider(40), Outcome::Ignored);
}

/// A living process whose group lost a member is retried quietly a few times,
/// and a run of them is reported,
/// so a process that never gets moved is not hidden.
#[test]
fn a_run_of_member_vanishes_is_reported_after_the_limit() {
    let script = (0..=MEMBER_VANISH_LIMIT).map(|_attempt| return Err(AdoptError::MemberVanished)).collect();
    let mut fixture = fixture("member-vanish", script);
    make_process(&fixture.proc_root, &mise(40));
    let start = Instant::now();
    for attempt in 1..MEMBER_VANISH_LIMIT {
        assert_eq!(fixture.routing.consider_at(40, start), Outcome::Ignored, "attempt {attempt} is quiet");
    }
    let Outcome::Failed { pid, error, .. } = fixture.routing.consider_at(40, start) else {
        panic!("the run of member vanishes is reported");
    };
    assert_eq!(pid, 40);
    assert!(error.contains("No such process") && error.contains(&MEMBER_VANISH_LIMIT.to_string()), "{error}");
}

/// A vanish of the process itself starts the count again,
/// so only an unbroken run is reported.
#[test]
fn a_vanish_of_the_process_resets_the_member_vanish_count() {
    let mut script: Vec<Result<Joined, AdoptError>> = (1..MEMBER_VANISH_LIMIT).map(|_attempt| return Err(AdoptError::MemberVanished)).collect();
    script.push(Err(AdoptError::Vanished));
    script.extend((1..MEMBER_VANISH_LIMIT).map(|_attempt| return Err(AdoptError::MemberVanished)));
    let total = script.len();
    let mut fixture = fixture("member-reset", script);
    make_process(&fixture.proc_root, &mise(40));
    let start = Instant::now();
    for attempt in 0..total {
        assert_eq!(fixture.routing.consider_at(40, start), Outcome::Ignored, "attempt {attempt} is quiet");
    }
}

/// After a failed move the pass spends at most a few failures and postpones the rest,
/// and a new pass tries them again.
#[test]
fn one_pass_spends_only_a_few_failed_moves() {
    let script = (0..10).map(|_attempt| return Err(AdoptError::Failed("refused".to_string()))).collect();
    let mut fixture = fixture("budget", script);
    let start = Instant::now();
    for pid in 40..45 {
        make_process(&fixture.proc_root, &mise(pid));
    }
    let outcomes: Vec<Outcome> = (40..45).map(|pid| return fixture.routing.consider_at(pid, start)).collect();
    assert_eq!(outcomes.iter().filter(|outcome| return matches!(outcome, Outcome::Failed { .. })).count(), 2);
    assert_eq!(outcomes.iter().filter(|outcome| return matches!(outcome, Outcome::Deferred { .. })).count(), 3);
    assert_eq!(fixture.requests.borrow().len(), 2);
    let postponed = outcomes.iter().find(|outcome| return matches!(outcome, Outcome::Deferred { .. })).expect("a postponed move");
    let line = crate::update_report::describe_outcome(postponed).expect("a postponed move is reported");
    assert!(line.contains("POLICY NOT IN FORCE"));
    assert!(line.contains("at most two failed attempts"), "{line}");
    assert_eq!(postponed.failure_kind(), Some("move postponed"));
    fixture.routing.begin_pass_at(start + Duration::from_millis(1));
    assert!(matches!(fixture.routing.consider_at(43, start + Duration::from_millis(1)), Outcome::Failed { attempt: 1, .. }));
}

/// A client that timed out blocks every move for a while,
/// loudly for the process it hit and quietly for the others.
#[test]
fn a_timeout_blocks_further_moves_for_a_while() {
    let mut fixture = fixture("block", vec![Err(AdoptError::TimedOut("busctl did not finish".to_string()))]);
    make_process(&fixture.proc_root, &mise(40));
    make_process(&fixture.proc_root, &mise(41));
    let start = Instant::now();
    let hit = fixture.routing.consider_at(40, start);
    assert!(matches!(&hit, Outcome::Failed { error, .. } if error.contains("no move is attempted for")), "{hit:?}");
    fixture.routing.begin_pass_at(start + Duration::from_secs(5));
    let paused = fixture.routing.consider_at(41, start + Duration::from_secs(5));
    assert!(matches!(&paused, Outcome::Deferred { pid: 41, why } if why.contains("did not answer in time")), "{paused:?}");
    assert_eq!(fixture.requests.borrow().len(), 1);
    fixture.routing.begin_pass_at(start + Duration::from_secs(11));
    assert!(matches!(fixture.routing.consider_at(41, start + Duration::from_secs(11)), Outcome::Adopted { .. }));
}

/// A pass that has spent its time budget on moves postpones the rest and says why,
/// so a slow manager cannot hold the watcher for long.
#[test]
fn a_pass_postpones_moves_after_its_time_budget() {
    let mut fixture = fixture("time-budget", vec![]);
    make_process(&fixture.proc_root, &mise(40));
    make_process(&fixture.proc_root, &mise(41));
    let start = Instant::now();
    fixture.routing.begin_pass_at(start);
    assert!(matches!(fixture.routing.consider_at(40, start), Outcome::Adopted { .. }));
    let late = fixture.routing.consider_at(41, start + Duration::from_secs(3));
    assert!(matches!(&late, Outcome::Deferred { pid: 41, why } if why.contains("time budget")), "{late:?}");
    fixture.routing.begin_pass_at(start + Duration::from_secs(4));
    assert!(matches!(fixture.routing.consider_at(41, start + Duration::from_secs(4)), Outcome::Adopted { .. }));
}

/// Stops a pass after a number of processes,
/// the way a shutdown signal would.
struct StopAfter {
    /// Checks left before the pass is asked to stop.
    remaining: std::cell::Cell<u32>,
}

/// The counting stop check.
impl StopCheck for StopAfter {
    /// Asks to stop once the count is used up.
    fn stop_requested(&self) -> bool {
        let left = self.remaining.get();
        if left == 0 {
            return true;
        }
        self.remaining.set(left - 1);
        return false;
    }
}

/// A shutdown signal that arrives during a rescan ends it between two processes,
/// and the rest are left for the next rescan.
#[test]
fn a_rescan_stops_between_processes_when_asked_to() {
    let mut fixture = fixture("stop", vec![]);
    for pid in 40..46 {
        make_process(&fixture.proc_root, &mise(pid));
    }
    let outcomes = fixture.routing.rescan_with(&SystemClock, &StopAfter { remaining: std::cell::Cell::new(2) });
    assert_eq!(outcomes.iter().filter(|outcome| return matches!(outcome, Outcome::Adopted { .. })).count(), 2);
    let rest = fixture.routing.rescan();
    assert_eq!(rest.iter().filter(|outcome| return matches!(outcome, Outcome::Adopted { .. })).count(), 4);
}

/// Processes that leave the same origin cgroup join the scope of that origin,
/// whatever their pids:
/// a reused pid of the same origin shares the scope,
/// where a scope per pid and start time left thousands of scopes behind.
#[test]
fn processes_from_one_origin_share_one_scope() {
    let mut fixture = fixture("one-scope", vec![]);
    make_process(&fixture.proc_root, &mise(50));
    assert!(matches!(fixture.routing.consider(50), Outcome::Adopted { .. }));
    make_process(&fixture.proc_root, &Fake { start: 777, ..mise(50) });
    assert!(matches!(fixture.routing.consider(50), Outcome::Adopted { .. }));
    make_process(&fixture.proc_root, &mise(51));
    assert!(matches!(fixture.routing.consider(51), Outcome::Adopted { .. }));
    let names: Vec<String> = fixture.requests.borrow().iter().map(|request| return request.scope_name.clone()).collect();
    assert_eq!(names.len(), 3);
    assert!(names.iter().all(|name| return *name == origin_scope_name(SCOPE, UID)), "{names:?}");
}

/// Processes that leave different origin cgroups get different scopes,
/// so one origin's cap is never shared with another's.
#[test]
fn different_origins_get_different_scopes() {
    let mut fixture = fixture("two-scopes", vec![]);
    let other_origin = "/user.slice/user-1000.slice/user@1000.service/app.slice/app-cli\x2dgit.slice/build";
    make_process(&fixture.proc_root, &mise(50));
    make_process(&fixture.proc_root, &Fake { cgroup: other_origin, ..mise(51) });
    assert!(matches!(fixture.routing.consider(50), Outcome::Adopted { .. }));
    assert!(matches!(fixture.routing.consider(51), Outcome::Adopted { .. }));
    let names: Vec<String> = fixture.requests.borrow().iter().map(|request| return request.scope_name.clone()).collect();
    assert_ne!(names[0], names[1]);
    assert_eq!(names[1], "wg-quicker-update-app.slice-app\\x2dcli\\x2dgit.slice-build.scope");
}

/// A scope follows its origin:
/// when the origin's cap changes,
/// the next rescan applies it to the live scope,
/// once,
/// and a scope that was joined by attaching to a running one is given the limits at once.
#[test]
fn a_scope_follows_its_origin_at_the_next_rescan() {
    let mut fixture = fixture("follow", vec![]);
    make_process(&fixture.proc_root, &mise(80));
    let origin = fixture.cgroup_root.join(SCOPE.trim_start_matches('/'));
    std::fs::create_dir_all(&origin).expect("make the origin cgroup");
    std::fs::write(origin.join("cpu.max"), "800000 100000\n").expect("write cpu.max");
    let name = origin_scope_name(SCOPE, UID);
    std::fs::create_dir_all(scope_cgroup_directory(&fixture.cgroup_root, UID, TOOLCHAIN_UPDATES_SLICE, &name)).expect("the scope exists");
    assert!(matches!(fixture.routing.consider(80), Outcome::Adopted { .. }));
    assert!(fixture.applied.borrow().is_empty(), "a created scope already has its limits");
    assert!(fixture.routing.rescan().iter().all(|outcome| return matches!(outcome, Outcome::AlreadyInSlice | Outcome::Adopted { .. })));
    assert!(fixture.applied.borrow().is_empty(), "an unchanged origin is left alone");
    std::fs::write(origin.join("cpu.max"), "200000 100000\n").expect("the cap daemon tightens the origin");
    let _outcomes = fixture.routing.rescan();
    let applied = fixture.applied.borrow().clone();
    assert_eq!(applied.len(), 1, "{applied:?}");
    assert_eq!(applied[0].0, name);
    assert_eq!(applied[0].1.cpu_quota_per_sec_usec, Some(2_000_000));
    let _again = fixture.routing.rescan();
    assert_eq!(fixture.applied.borrow().len(), 1);
}

/// A move that attached to a scope that was already running gives that scope the origin's limits,
/// because this watcher cannot know what a scope it did not create was given.
#[test]
fn a_scope_joined_by_attaching_is_given_the_limits() {
    let mut fixture = fixture("attached", vec![Ok(Joined::Attached)]);
    make_process(&fixture.proc_root, &mise(80));
    let origin = fixture.cgroup_root.join(SCOPE.trim_start_matches('/'));
    std::fs::create_dir_all(&origin).expect("make the origin cgroup");
    std::fs::write(origin.join("cpu.max"), "800000 100000\n").expect("write cpu.max");
    let name = origin_scope_name(SCOPE, UID);
    std::fs::create_dir_all(scope_cgroup_directory(&fixture.cgroup_root, UID, TOOLCHAIN_UPDATES_SLICE, &name)).expect("the scope exists");
    assert!(matches!(fixture.routing.consider(80), Outcome::Adopted { .. }));
    assert_eq!(fixture.applied.borrow().len(), 1);
    assert_eq!(fixture.applied.borrow()[0].1.cpu_quota_per_sec_usec, Some(8_000_000));
}

/// Moves the matched process together with its live descendants of the same user in the same cgroup.
#[test]
fn descendants_in_the_same_cgroup_move_with_the_matched_process() {
    let mut fixture = fixture("descendants", vec![]);
    make_process(&fixture.proc_root, &mise(50));
    make_process(&fixture.proc_root, &other(51, 50));
    make_process(&fixture.proc_root, &Fake { exe: "/usr/bin/curl", args: &["curl"], ..other(52, 51) });
    make_process(&fixture.proc_root, &Fake { uid: 2000, exe: "/usr/bin/sudo", args: &["sudo"], ..other(53, 50) });
    make_process(&fixture.proc_root, &other(54, 1));
    let elsewhere = "/user.slice/user-1000.slice/user@1000.service/app.slice/other.scope";
    make_process(&fixture.proc_root, &Fake { cgroup: elsewhere, ..other(55, 50) });
    make_process(&fixture.proc_root, &other(56, 55));
    assert!(matches!(fixture.routing.consider(50), Outcome::Adopted { count: 3, left_behind: 0, .. }));
    assert_eq!(fixture.requests.borrow()[0].pids, vec![50, 51, 52]);
}

/// A descendant whose pid is reused between collection and the move is dropped,
/// and a matched process that is replaced makes the whole move stop.
#[test]
fn a_reused_pid_is_not_moved() {
    let mut fixture = fixture("reuse", vec![]);
    make_process(&fixture.proc_root, &mise(50));
    make_process(&fixture.proc_root, &other(51, 50));
    make_process(&fixture.proc_root, &other(52, 50));
    let proc_root = fixture.proc_root.clone();
    let mut swap_child = || make_process(&proc_root, &Fake { start: 999, ..other(51, 50) });
    let outcome = fixture.routing.consider_with(50, Instant::now(), &mut swap_child);
    assert!(matches!(outcome, Outcome::Adopted { count: 2, left_behind: 1, .. }), "{outcome:?}");
    assert_eq!(fixture.requests.borrow()[0].pids, vec![50, 52]);
    make_process(&fixture.proc_root, &mise(60));
    let mut swap_root = || make_process(&proc_root, &Fake { start: 555, ..mise(60) });
    assert_eq!(fixture.routing.consider_with(60, Instant::now(), &mut swap_root), Outcome::Ignored);
    assert_eq!(fixture.requests.borrow().len(), 1, "the replaced process was not moved");
}

/// A process of the watched user that cannot be inspected is reported once,
/// not mistaken for an exit.
#[test]
fn an_unreadable_process_is_reported_once() {
    let mut fixture = fixture("unreadable", vec![]);
    make_process(&fixture.proc_root, &mise(70));
    std::fs::remove_file(fixture.proc_root.join("70/exe")).expect("remove the link");
    std::fs::write(fixture.proc_root.join("70/exe"), b"not a link").expect("replace it by a file");
    let first = fixture.routing.consider(70);
    assert!(matches!(first, Outcome::Unreadable { pid: 70, .. }), "{first:?}");
    assert!(first.failure_kind().is_some());
    assert_eq!(fixture.routing.consider(70), Outcome::Ignored);
}

/// A rescan that cannot list processes says so.
#[test]
fn a_rescan_that_cannot_list_processes_reports_it() {
    let mut fixture = fixture("scan-failed", vec![]);
    std::fs::remove_dir_all(&fixture.proc_root).expect("remove the procfs");
    let outcomes = fixture.routing.rescan();
    assert!(matches!(outcomes.as_slice(), [Outcome::ScanFailed { .. }]), "{outcomes:?}");
}

/// A rescan finds matches the event stream missed and forgets exited processes,
/// so a reused pid with a new start time is considered again.
#[test]
fn a_rescan_finds_missed_processes_and_forgets_exited_ones() {
    let mut fixture = fixture("rescan", vec![]);
    make_process(&fixture.proc_root, &mise(60));
    make_process(&fixture.proc_root, &Fake { exe: "/usr/bin/vim", args: &["vim"], ..mise(61) });
    let outcomes = fixture.routing.rescan();
    assert_eq!(outcomes.iter().filter(|outcome| return matches!(outcome, Outcome::Adopted { .. })).count(), 1);
    assert!(fixture.routing.rescan().is_empty());
    std::fs::remove_dir_all(fixture.proc_root.join("60")).expect("the process exits");
    let _unused = fixture.routing.rescan();
    make_process(&fixture.proc_root, &Fake { start: 200, ..mise(60) });
    assert!(matches!(fixture.routing.consider(60), Outcome::Adopted { .. }));
    assert_eq!(fixture.requests.borrow().len(), 2);
}

/// The scope carries the limits of the cgroup the process leaves,
/// and a limit that cannot be read stops the move.
#[test]
fn the_moved_scope_carries_the_origin_limits() {
    let mut fixture = fixture("limits", vec![]);
    make_process(&fixture.proc_root, &mise(80));
    let origin = fixture.cgroup_root.join(SCOPE.trim_start_matches('/'));
    std::fs::create_dir_all(&origin).expect("make the origin cgroup");
    std::fs::write(origin.join("cpu.max"), "800000 100000\n").expect("write cpu.max");
    std::fs::write(origin.join("memory.max"), "2147483648\n").expect("write memory.max");
    assert!(matches!(fixture.routing.consider(80), Outcome::Adopted { .. }));
    let limits = fixture.requests.borrow()[0].limits.clone();
    assert_eq!(limits.cpu_quota_per_sec_usec, Some(8_000_000));
    assert_eq!(limits.memory_max, Some(2_147_483_648));
    make_process(&fixture.proc_root, &mise(81));
    std::fs::write(origin.join("memory.high"), "plenty\n").expect("garble a limit");
    let refused = fixture.routing.consider(81);
    assert!(matches!(&refused, Outcome::Failed { error, .. } if error.contains("never moved without them")), "{refused:?}");
    assert_eq!(fixture.requests.borrow().len(), 1, "nothing was moved without its limits");
}

/// The group of a matched process is found among the members of its cgroup,
/// not among every process of the host:
/// a hundred processes elsewhere whose files cannot be read are never touched,
/// while the same fixture without a member list reads them and counts them as left behind.
#[test]
fn the_group_is_collected_from_the_member_list_of_the_cgroup() {
    let mut listed = fixture("members", vec![]);
    make_process(&listed.proc_root, &mise(10));
    make_process(&listed.proc_root, &other(11, 10));
    make_process(&listed.proc_root, &other(12, 11));
    make_process(&listed.proc_root, &other(13, 10));
    for decoy in 100..200 {
        let base = listed.proc_root.join(decoy.to_string());
        std::fs::create_dir_all(base.join("status")).expect("a status that cannot be read as a file");
    }
    write_members(&listed.cgroup_root, SCOPE, &[10, 11, 12]);
    let outcome = listed.routing.consider(10);
    assert_eq!(outcome, Outcome::Adopted { pid: 10, reason: MatchReason::MiseTask, count: 3, left_behind: 0 }, "{outcome:?}");
    assert_eq!(listed.requests.borrow()[0].pids, vec![10, 11, 12], "a process missing from the member list is not collected");
    let mut scanned = fixture("members-fallback", vec![]);
    make_process(&scanned.proc_root, &mise(10));
    make_process(&scanned.proc_root, &other(11, 10));
    for decoy in 100..200 {
        std::fs::create_dir_all(scanned.proc_root.join(decoy.to_string()).join("status")).expect("a status that cannot be read as a file");
    }
    let fallback = scanned.routing.consider(10);
    assert!(matches!(&fallback, Outcome::Adopted { count: 2, left_behind, .. } if *left_behind == 100), "{fallback:?}");
}

/// Members of another user and members that left the cgroup meanwhile are not part of the group.
#[test]
fn members_of_another_user_or_gone_are_not_collected() {
    let mut fixture = fixture("members-foreign", vec![]);
    make_process(&fixture.proc_root, &mise(10));
    make_process(&fixture.proc_root, &Fake { uid: 2000, ..other(11, 10) });
    make_process(&fixture.proc_root, &other(12, 10));
    write_members(&fixture.cgroup_root, SCOPE, &[10, 11, 12, 13, 14]);
    assert!(matches!(fixture.routing.consider(10), Outcome::Adopted { count: 2, .. }));
    assert_eq!(fixture.requests.borrow()[0].pids, vec![10, 12]);
}

/// A clock that only the test moves.
struct FixedClock {
    /// The time now.
    now: std::cell::Cell<Instant>,
}

/// The fixed clock.
impl crate::routing_support::Clock for FixedClock {
    /// The time the test set.
    fn now(&self) -> Instant {
        return self.now.get();
    }
}

/// A call to follow an origin that runs past its timeout is reported,
/// pauses the engine's moves for a while like a hung move does,
/// and no origin is followed while they are paused.
#[test]
fn a_hung_manager_during_the_follow_up_pauses_the_moves_and_the_follow_up() {
    let mut fixture = fixture("hung-follow", vec![]);
    make_process(&fixture.proc_root, &mise(80));
    let origin = fixture.cgroup_root.join(SCOPE.trim_start_matches('/'));
    std::fs::create_dir_all(&origin).expect("make the origin cgroup");
    std::fs::write(origin.join("cpu.max"), "800000 100000\n").expect("write cpu.max");
    let name = origin_scope_name(SCOPE, UID);
    std::fs::create_dir_all(scope_cgroup_directory(&fixture.cgroup_root, UID, TOOLCHAIN_UPDATES_SLICE, &name)).expect("the scope exists");
    assert!(matches!(fixture.routing.consider(80), Outcome::Adopted { .. }));
    std::fs::write(origin.join("cpu.max"), "200000 100000\n").expect("the cap daemon tightens the origin");
    fixture.apply_script.borrow_mut().push_back(Err(AdoptError::TimedOut("busctl did not finish".to_string())));
    let clock = FixedClock { now: std::cell::Cell::new(Instant::now()) };
    let outcomes = fixture.routing.rescan_with(&clock, &NeverStop);
    assert!(outcomes.iter().any(|outcome| return matches!(outcome, Outcome::ScopeLimits { error, .. } if error.contains("did not finish"))), "{outcomes:?}");
    assert_eq!(fixture.applied.borrow().len(), 1);
    // The moves are paused.
    make_process(&fixture.proc_root, &Fake { start: 101, ..mise(81) });
    fixture.routing.begin_pass_at(clock.now.get());
    let deferred = fixture.routing.consider_at(81, clock.now.get());
    assert!(matches!(&deferred, Outcome::Deferred { why, .. } if why.contains("did not answer in time")), "{deferred:?}");
    // The follow-up is paused too, even when the backoff of the origin is over.
    clock.now.set(clock.now.get() + Duration::from_secs(2));
    let _paused = fixture.routing.rescan_with(&clock, &NeverStop);
    assert_eq!(fixture.applied.borrow().len(), 1, "the follow-up asked the paused manager");
    // After the pause both go on;
    // the deferred process is gone, so only the follow-up asks the manager.
    std::fs::remove_dir_all(fixture.proc_root.join("81")).expect("the deferred process exits");
    clock.now.set(clock.now.get() + Duration::from_secs(10));
    let _resumed = fixture.routing.rescan_with(&clock, &NeverStop);
    assert_eq!(fixture.applied.borrow().len(), 2, "the follow-up did not resume");
}

/// An attach to a running scope whose limits call times out pauses the moves,
/// because the manager is hung even though the attach itself answered.
#[test]
fn a_hung_manager_when_a_scope_is_joined_pauses_the_moves() {
    let mut fixture = fixture("hung-join", vec![Ok(Joined::Attached)]);
    make_process(&fixture.proc_root, &mise(80));
    fixture.apply_script.borrow_mut().push_back(Err(AdoptError::TimedOut("busctl did not finish".to_string())));
    let now = Instant::now();
    assert!(matches!(fixture.routing.consider_at(80, now), Outcome::Adopted { .. }));
    make_process(&fixture.proc_root, &Fake { start: 101, ..mise(81) });
    fixture.routing.begin_pass_at(now);
    let deferred = fixture.routing.consider_at(81, now + Duration::from_millis(1));
    assert!(matches!(&deferred, Outcome::Deferred { why, .. } if why.contains("did not answer in time")), "{deferred:?}");
}

/// Pi's unit cgroup,
/// which the user's service manager created with the run's resource limits.
const PI_UNIT: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/pi-wg-quicker-model-providers-off-the-tunnel.service";

/// The toolchain node Pi runs on,
/// under the install roots of the fixture user.
const TOOLCHAIN_NODE: &str = "/var/home/tester/.local/share/monochromatic-dev/toolchain/node/26.13.0/node-v26.13.0-linux-x64/bin/node";

/// The gap the owner measured:
/// a Pi process is matched by the toolchain path rule,
/// but the process watch cannot move the main process of a service,
/// so the socket mark attached to the unit's own cgroup covers it instead.
#[test]
fn a_process_in_a_pi_unit_is_covered_by_the_mark_attached_to_its_own_cgroup() {
    let mut fixture = fixture("pi-unit", vec![]);
    make_process(&fixture.proc_root, &Fake { pid: 40, exe: TOOLCHAIN_NODE, args: &["pi"], cgroup: PI_UNIT, ..mise(40) });
    make_process(&fixture.proc_root, &Fake { pid: 41, exe: TOOLCHAIN_NODE, args: &["node", "drive-pi-subagent.ts"], cgroup: PI_UNIT, ..mise(41) });
    assert_eq!(fixture.routing.consider(40), Outcome::Covered { pid: 40 });
    assert_eq!(fixture.routing.consider(41), Outcome::Covered { pid: 41 });
    assert_eq!(fixture.routing.consider(40), Outcome::Ignored, "a covered process is handled once");
    assert!(fixture.requests.borrow().is_empty(), "a service main process is never handed to the manager");
}

/// A unit the watcher does not mark is still reported as not moved,
/// so a gap in the coverage is never silent.
#[test]
fn a_process_in_an_unmarked_service_is_still_reported_as_not_moved() {
    let mut fixture = fixture("unmarked-service", vec![]);
    let other = "/user.slice/user-1000.slice/user@1000.service/app.slice/other.service";
    make_process(&fixture.proc_root, &Fake { pid: 42, exe: TOOLCHAIN_NODE, args: &["node", "x.js"], cgroup: other, ..mise(42) });
    let skipped = fixture.routing.consider(42);
    assert!(matches!(skipped, Outcome::Skipped { pid: 42, .. }), "{skipped:?}");
    assert!(skipped.failure_kind().is_some());
    assert!(fixture.requests.borrow().is_empty());
}

/// A Pi process outside a marked unit is moved,
/// through the toolchain path when it runs the toolchain's node and through its own command line otherwise.
#[test]
fn a_pi_process_outside_a_marked_unit_is_moved() {
    let mut fixture = fixture("pi-scope", vec![]);
    make_process(&fixture.proc_root, &Fake { pid: 43, exe: TOOLCHAIN_NODE, args: &["pi"], ..mise(43) });
    make_process(&fixture.proc_root, &Fake { pid: 44, exe: "/usr/bin/node", args: &["pi"], ..mise(44) });
    assert_eq!(fixture.routing.consider(43), Outcome::Adopted { pid: 43, reason: MatchReason::ToolchainPath, count: 1, left_behind: 0 });
    assert_eq!(fixture.routing.consider(44), Outcome::Adopted { pid: 44, reason: MatchReason::PiAgent, count: 1, left_behind: 0 });
}
