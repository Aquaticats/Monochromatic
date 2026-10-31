//! Verifies the update traffic policy on a fake procfs tree with allowed and rejected fixtures,
//! the start notice,
//! and the loud failure reports.

/// The slice name.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// Seam implemented by the recording fake.
use crate::scope_adoption::{AdoptRequest, Adopter};
/// Install roots of the fixture user.
use crate::update_entry_points::install_roots;
/// Policy under test.
use crate::update_routing::{adoptable_cgroup, describe_outcome, policy_failure, policy_notice, Outcome, UpdateRouting};
/// Shared recording between the test and the engine.
use std::cell::RefCell;
/// Fixture paths.
use std::path::{Path, PathBuf};
/// Shared recording between the test and the engine.
use std::rc::Rc;

/// The fixture user's id.
const UID: u32 = 1000;
/// A cgroup inside the user manager's subtree.
const SCOPE: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/app-ghostty-surface-transient-1.scope";

/// Records adoptions and optionally fails them.
struct RecordingAdopter {
    /// Pid groups adopted in order.
    adopted: Rc<RefCell<Vec<Vec<i32>>>>,
    /// Whether adoption fails.
    fail: bool,
}

impl Adopter for RecordingAdopter {
    fn ensure_slice(&self, _uid: u32, _gid: u32, _slice: &str, _slice_directory: &Path) -> std::io::Result<()> {
        return Ok(());
    }

    fn adopt(&self, request: &AdoptRequest<'_>) -> std::io::Result<()> {
        if self.fail {
            return Err(std::io::Error::other("the manager refused the job"));
        }
        self.adopted.borrow_mut().push(request.pids.to_vec());
        return Ok(());
    }
}

/// Engine,
/// recording
/// and fake procfs for one test.
struct Fixture {
    /// Fake procfs root.
    proc_root: PathBuf,
    /// Recording of adoptions.
    adopted: Rc<RefCell<Vec<Vec<i32>>>>,
    /// Engine under test.
    routing: UpdateRouting,
}

/// Builds a fixture whose adopter fails or succeeds.
fn fixture(name: &str, fail: bool) -> Fixture {
    let proc_root = crate::test_scratch::scratch_root().join(format!("wg-quicker-routing-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&proc_root);
    std::fs::create_dir_all(&proc_root).expect("make the fake procfs");
    let adopted = Rc::new(RefCell::new(Vec::new()));
    let adopter = RecordingAdopter { adopted: Rc::clone(&adopted), fail };
    let routing = UpdateRouting::new(
        UID,
        UID,
        install_roots(Path::new("/home/tester"), Some(Path::new("/var/home/tester"))),
        proc_root.clone(),
        Box::new(adopter),
    );
    return Fixture { proc_root, adopted, routing };
}

/// Describes one fake process.
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
    std::fs::write(base.join("status"), format!("Name:\tx\nUid:\t{0}\t{0}\t{0}\t{0}\nGid:\t{0}\t{0}\t{0}\t{0}\n", fake.uid))
        .expect("write status");
    std::fs::write(base.join("cgroup"), format!("0::{}\n", fake.cgroup)).expect("write cgroup");
    let zeros = vec!["0"; 17].join(" ");
    std::fs::write(base.join("stat"), format!("{} (x y) S {} {zeros} {} 0\n", fake.pid, fake.parent, fake.start)).expect("write stat");
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
    };
}

/// The start notice names every kind of traffic that leaves and says there is no switch.
#[test]
fn the_start_notice_names_the_traffic_and_the_absence_of_a_switch() {
    let notice = policy_notice();
    for phrase in [
        "POLICY NOTICE",
        "without any switch",
        "update entry points",
        "mpm-shadow install, update and outdated",
        "toolchain install roots",
        "monochromatic-dev data directory",
        "update-host list",
        "outside the tunnel",
    ] {
        assert!(notice.contains(phrase), "missing {phrase:?} in {notice}");
    }
}

/// A failure report names the policy that is not in force.
#[test]
fn a_failure_report_names_the_policy_not_in_force() {
    let report = policy_failure("the updates slice cannot be created: busctl exited with 1");
    assert!(report.contains("POLICY NOT IN FORCE"));
    assert!(report.contains("process watch"));
    assert!(report.contains("busctl exited with 1"));
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
    let mut fixture = fixture("adopt", false);
    make_process(&fixture.proc_root, &mise(10));
    make_process(&fixture.proc_root, &Fake { pid: 11, exe: "/usr/bin/vim", args: &["vim"], uid: UID, cgroup: SCOPE, parent: 1, start: 5 });
    make_process(&fixture.proc_root, &Fake { pid: 12, uid: 2000, ..mise(12) });
    assert_eq!(fixture.routing.consider(10), Outcome::Adopted { pid: 10, reason: crate::update_entry_points::MatchReason::MiseTask, count: 1 });
    assert_eq!(fixture.routing.consider(10), Outcome::Ignored);
    assert_eq!(fixture.routing.consider(11), Outcome::Ignored);
    assert_eq!(fixture.routing.consider(12), Outcome::Ignored);
    assert_eq!(fixture.routing.consider(999), Outcome::Ignored);
    assert_eq!(*fixture.adopted.borrow(), vec![vec![10]]);
}

/// Executables under the toolchain roots are adopted by path alone.
#[test]
fn an_executable_under_the_install_roots_is_adopted() {
    let mut fixture = fixture("path", false);
    let node = "/var/home/tester/.local/share/monochromatic-dev/toolchain/node/26.13.0/bin/node";
    make_process(&fixture.proc_root, &Fake { pid: 20, exe: node, args: &["node", "x.js"], uid: UID, cgroup: SCOPE, parent: 1, start: 1 });
    make_process(&fixture.proc_root, &Fake { pid: 21, exe: "/usr/bin/node", args: &["node", "x.js"], uid: UID, cgroup: SCOPE, parent: 1, start: 1 });
    assert!(matches!(fixture.routing.consider(20), Outcome::Adopted { reason: crate::update_entry_points::MatchReason::ToolchainPath, .. }));
    assert_eq!(fixture.routing.consider(21), Outcome::Ignored);
}

/// Leaves a process already in the slice alone,
/// and reports a process it cannot move by naming the policy.
#[test]
fn processes_already_in_the_slice_or_outside_the_subtree_are_not_moved() {
    let mut fixture = fixture("skip", false);
    let inside = format!("/user.slice/user-1000.slice/user@1000.service/app.slice/{TOOLCHAIN_UPDATES_SLICE}/x.scope");
    make_process(&fixture.proc_root, &Fake { cgroup: &inside, ..mise(30) });
    make_process(&fixture.proc_root, &Fake { cgroup: "/user.slice/user-1000.slice/session-3.scope", ..mise(31) });
    assert_eq!(fixture.routing.consider(30), Outcome::AlreadyInSlice);
    let skipped = fixture.routing.consider(31);
    assert!(matches!(skipped, Outcome::Skipped { pid: 31, .. }));
    let line = describe_outcome(&skipped).expect("a skip is reported");
    assert!(line.contains("POLICY NOT IN FORCE"));
    assert!(line.contains("pid 31"));
    assert!(fixture.adopted.borrow().is_empty());
}

/// A failing adoption is reported loudly and nothing else changes.
#[test]
fn a_failed_adoption_is_reported_loudly() {
    let mut fixture = fixture("fail", true);
    make_process(&fixture.proc_root, &mise(40));
    let failed = fixture.routing.consider(40);
    assert!(matches!(failed, Outcome::Failed { pid: 40, .. }));
    let line = describe_outcome(&failed).expect("a failure is reported");
    assert!(line.contains("POLICY NOT IN FORCE"));
    assert!(line.contains("the manager refused the job"));
    assert_eq!(fixture.routing.consider(40), Outcome::Ignored);
}

/// Moves the matched process together with its live descendants of the same user.
#[test]
fn descendants_of_the_same_user_move_with_the_matched_process() {
    let mut fixture = fixture("descendants", false);
    make_process(&fixture.proc_root, &mise(50));
    make_process(&fixture.proc_root, &Fake { pid: 51, parent: 50, exe: "/usr/bin/sh", args: &["sh"], ..mise(51) });
    make_process(&fixture.proc_root, &Fake { pid: 52, parent: 51, exe: "/usr/bin/curl", args: &["curl"], ..mise(52) });
    make_process(&fixture.proc_root, &Fake { pid: 53, parent: 50, uid: 2000, exe: "/usr/bin/sudo", args: &["sudo"], ..mise(53) });
    make_process(&fixture.proc_root, &Fake { pid: 54, parent: 1, exe: "/usr/bin/sh", args: &["sh"], ..mise(54) });
    assert!(matches!(fixture.routing.consider(50), Outcome::Adopted { count: 3, .. }));
    assert_eq!(*fixture.adopted.borrow(), vec![vec![50, 51, 52]]);
}

/// A rescan finds matches the event stream missed and forgets exited processes,
/// so a reused pid with a new start time is considered again.
#[test]
fn a_rescan_finds_missed_processes_and_forgets_exited_ones() {
    let mut fixture = fixture("rescan", false);
    make_process(&fixture.proc_root, &mise(60));
    make_process(&fixture.proc_root, &Fake { pid: 61, exe: "/usr/bin/vim", args: &["vim"], ..mise(61) });
    let outcomes = fixture.routing.rescan();
    assert_eq!(outcomes.iter().filter(|outcome| return matches!(outcome, Outcome::Adopted { .. })).count(), 1);
    assert!(fixture.routing.rescan().iter().all(|outcome| return matches!(outcome, Outcome::Ignored)));
    std::fs::remove_dir_all(fixture.proc_root.join("60")).expect("the process exits");
    let _unused = fixture.routing.rescan();
    make_process(&fixture.proc_root, &Fake { start: 200, ..mise(60) });
    assert!(matches!(fixture.routing.consider(60), Outcome::Adopted { .. }));
    assert_eq!(fixture.adopted.borrow().len(), 2);
}

/// Quiet outcomes produce no log line and an adoption names the slice.
#[test]
fn only_adoptions_and_problems_are_logged() {
    assert_eq!(describe_outcome(&Outcome::Ignored), None);
    assert_eq!(describe_outcome(&Outcome::AlreadyInSlice), None);
    let line = describe_outcome(&Outcome::Adopted { pid: 5, reason: crate::update_entry_points::MatchReason::MpmShim, count: 2 }).expect("adoption is logged");
    assert!(line.contains(TOOLCHAIN_UPDATES_SLICE));
    assert!(line.contains("pid 5"));
}
