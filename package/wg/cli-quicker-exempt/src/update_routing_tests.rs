//! Verifies the update traffic policy on a fake procfs tree with allowed and rejected fixtures:
//! what is moved,
//! what is left alone,
//! how failures are retried,
//! and what the moved scope carries.

/// The slice name.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// Seam implemented by the scripted fake.
use crate::scope_adoption::{AdoptError, AdoptRequest, Adopter};
/// Install roots of the fixture user and the match reasons.
use crate::update_entry_points::{install_roots, MatchReason};
/// Outcomes under test.
use crate::update_report::Outcome;
/// Policy under test.
use crate::update_routing::{adoptable_cgroup, retry_backoff, RoutingSetup, UpdateRouting};
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
/// A cgroup inside the user manager's subtree.
const SCOPE: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/app-ghostty-surface-transient-1.scope";

/// What the fake adopter was asked to do.
#[derive(Debug, Eq, PartialEq)]
struct Recorded {
    /// Pids requested.
    pids: Vec<i32>,
    /// Group id passed for the privilege drop.
    gid: u32,
}

/// Records adoptions and answers from a script,
/// then succeeds.
struct ScriptedAdopter {
    /// Requests in order.
    requests: Rc<RefCell<Vec<Recorded>>>,
    /// Results for the next adoptions.
    script: RefCell<VecDeque<Result<(), AdoptError>>>,
}

/// The scripted fake.
impl Adopter for ScriptedAdopter {
    /// The slice always exists.
    fn ensure_slice(&self, _uid: u32, _gid: u32, _slice: &str, _slice_directory: &Path) -> std::io::Result<()> {
        return Ok(());
    }

    /// Records the request and answers from the script.
    fn adopt(&self, request: &AdoptRequest<'_>) -> Result<(), AdoptError> {
        self.requests.borrow_mut().push(Recorded { pids: request.pids.to_vec(), gid: request.gid });
        return self.script.borrow_mut().pop_front().unwrap_or(Ok(()));
    }
}

/// Engine,
/// recording
/// and fake trees for one test.
struct Fixture {
    /// Fake procfs root.
    proc_root: PathBuf,
    /// Recording of adoption requests.
    requests: Rc<RefCell<Vec<Recorded>>>,
    /// Engine under test.
    routing: UpdateRouting,
}

/// Builds a fixture whose adopter answers from `script`.
fn fixture(name: &str, script: Vec<Result<(), AdoptError>>) -> Fixture {
    let base = crate::test_scratch::scratch_root().join(format!("wg-quicker-routing-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&base);
    let proc_root = base.join("proc");
    std::fs::create_dir_all(&proc_root).expect("make the fake procfs");
    let requests = Rc::new(RefCell::new(Vec::new()));
    let adopter = ScriptedAdopter { requests: Rc::clone(&requests), script: RefCell::new(script.into()) };
    let routing = UpdateRouting::new(RoutingSetup {
        uid: UID,
        gid: USER_GID,
        install_roots: install_roots(Path::new("/home/tester"), Some(Path::new("/var/home/tester"))),
        proc_root: proc_root.clone(),
        adopter: Box::new(adopter),
    });
    return Fixture { proc_root, requests, routing };
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

/// A matching mise process in an adoptable scope.
fn mise(pid: i32) -> Fake<'static> {
    return Fake {
        pid,
        exe: "/usr/bin/mise",
        args: &["mise", "run", "toolchain:sync"],
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

/// A forged first word does not get a process moved.
#[test]
fn a_process_that_only_claims_the_name_is_not_moved() {
    let mut fixture = fixture("forged", vec![]);
    make_process(&fixture.proc_root, &Fake { pid: 25, exe: "/usr/bin/curl", args: &["mpm", "https://example.org"], ..mise(25) });
    make_process(&fixture.proc_root, &Fake { pid: 26, exe: "/usr/bin/curl", args: &["mise", "run", "toolchain:sync"], ..mise(26) });
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
    assert!(skipped.is_failure());
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
        adopter: Box::new(ScriptedAdopter { requests: Rc::clone(&fixture.requests), script: RefCell::new(VecDeque::new()) }),
    });
    make_process(&fixture.proc_root, &Fake { namespace: Some("pid:[4026533586]"), ..mise(40) });
    make_process(&fixture.proc_root, &Fake { namespace: Some("pid:[4026531836]"), ..mise(41) });
    make_process(&fixture.proc_root, &mise(42));
    let sandboxed = routing.consider(40);
    assert!(matches!(&sandboxed, Outcome::Skipped { pid: 40, why } if why.contains("another pid namespace")), "{sandboxed:?}");
    assert!(matches!(routing.consider(41), Outcome::Adopted { .. }));
    assert!(matches!(routing.consider(42), Outcome::Adopted { .. }));
    assert_eq!(fixture.requests.borrow().len(), 2);
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
    assert!(first.is_failure());
    assert_eq!(fixture.routing.consider_at(40, start + Duration::from_millis(500)), Outcome::Ignored);
    assert_eq!(fixture.requests.borrow().len(), 1, "no attempt before the wait is over");
    fixture.routing.begin_pass();
    let second = fixture.routing.consider_at(40, start + Duration::from_secs(1));
    assert!(matches!(&second, Outcome::Failed { attempt: 2, retry_in, .. } if *retry_in == Duration::from_secs(2)), "{second:?}");
    assert_eq!(fixture.routing.consider_at(40, start + Duration::from_millis(2500)), Outcome::Ignored);
    fixture.routing.begin_pass();
    let third = fixture.routing.consider_at(40, start + Duration::from_secs(3));
    assert!(matches!(third, Outcome::Adopted { pid: 40, .. }), "{third:?}");
    assert_eq!(fixture.routing.consider_at(40, start + Duration::from_secs(100)), Outcome::Ignored);
    assert_eq!(fixture.requests.borrow().len(), 3);
}

/// The wait doubles from one second and stops growing at a minute.
#[test]
fn the_retry_wait_doubles_up_to_a_cap() {
    let waits: Vec<u64> = (1..=9).map(|attempt| return retry_backoff(attempt).as_secs()).collect();
    assert_eq!(waits, vec![1, 2, 4, 8, 16, 32, 60, 60, 60]);
}

/// A process that exits while it is being moved is not a failure.
#[test]
fn a_vanished_process_is_not_reported() {
    let mut fixture = fixture("vanish", vec![Err(AdoptError::Vanished)]);
    make_process(&fixture.proc_root, &mise(40));
    assert_eq!(fixture.routing.consider(40), Outcome::Ignored);
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
    assert_eq!(outcomes.iter().filter(|outcome| return matches!(outcome, Outcome::Failed { .. })).count(), 3);
    assert_eq!(outcomes.iter().filter(|outcome| return matches!(outcome, Outcome::Deferred)).count(), 2);
    assert_eq!(fixture.requests.borrow().len(), 3);
    fixture.routing.begin_pass();
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
    fixture.routing.begin_pass();
    assert_eq!(fixture.routing.consider_at(41, start + Duration::from_secs(5)), Outcome::Deferred);
    assert_eq!(fixture.requests.borrow().len(), 1);
    fixture.routing.begin_pass();
    assert!(matches!(fixture.routing.consider_at(41, start + Duration::from_secs(31)), Outcome::Adopted { .. }));
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
    assert!(first.is_failure());
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
