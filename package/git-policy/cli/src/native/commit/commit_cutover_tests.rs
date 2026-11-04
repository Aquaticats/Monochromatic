//! What:
//!  Controls of plan step 3 (owner decision 23, "New lock name"): the cutover fence, and the service side of the lifecycle behind the
//!  `--lane` switch of the real `cli-git-service` binary: the startup sweep, the periodic pass that is the failure owner of a unit that died
//!  after its execution, and the doorbell a client rings.
//! Why:
//!  An old client that probes the old lock and holds it without sweeping, overlapping the new implementation, recreated the schedule where
//!  a new client's helper loses to the probe and the probe owner dies before it forks (review 15). The new protocol takes a lock file name
//!  nobody else uses, so no old holder can hold it. Old and new flushers may run at once: both force the same receipts, and forcing twice is
//!  harmless. The shipped service is unchanged unless it is started with `--lane`; the shipped unit files do not pass it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('an old probe holder does not stop a new flusher', ...); test('the service serves a lane it is told about', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_helper_launch_tests::flusher_launch_code;
use crate::commit_state_log_tests::lane_with_envelope;
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_disposal::{DisposalAuthorization, dispose_saved_content};
use git_policy_cli::commit_flusher::{FLUSHER_LOCK, FlusherLock, LEGACY_FLUSHER_LOCK, RECEIPT_DIR, RECEIPT_ERRORS, flush_lane, hold_legacy_lock, launch_flusher};
use git_policy_cli::commit_flusher_unit::{ManagerRefused, ManagerRoute};
use git_policy_cli::commit_identity::set_fault_point;
use git_policy_cli::commit_lane_lifecycle::{ClientStep, Routed, client_after_accept};
use git_policy_cli::commit_lane_service::{bind_ring, ring_running_service};
use git_policy_cli::commit_state_log::{ClosureEntry, LaneDocument, StateWriter, create_saved_content, select};
use std::cell::RefCell;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

/// The longest a test waits for a service to act; the work is a few file operations.
const WAIT: Duration = Duration::from_secs(10);

/// A request identity for a receipt.
const RECEIPT_ID: &str = "0123456789abcdef0123456789abcdef";

/// A lane with a receipt directory.
fn lane_with_receipts(label: &str) -> PathBuf {
    let lane: PathBuf = scratch(label);
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    return lane;
}

/// A receipt that cannot be forced (a dangling link): any executor that handles it records it.
fn unforceable(lane: &Path, name: &str) {
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join(name)).expect("a receipt that cannot be forced");
}

/// Waits until `name` is recorded in the lane's error record.
fn wait_recorded(lane: &Path, name: &str) {
    let deadline: Instant = Instant::now() + WAIT;
    while !fs::read_to_string(lane.join(RECEIPT_ERRORS)).is_ok_and(|text| { return text.lines().any(|line| { return line.split('\t').next() == Some(name); }); }) {
        assert!(Instant::now() < deadline, "{} was never recorded", name);
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The new protocol's lock has a name no old implementation uses.
#[test]
fn the_new_protocol_takes_a_lock_name_the_old_probe_does_not_use() {
    assert_ne!(FLUSHER_LOCK, LEGACY_FLUSHER_LOCK);
    let lane: PathBuf = lane_with_receipts("cutover-names");
    let new = FlusherLock::try_acquire(&lane).expect("lock").expect("the new lock");
    assert!(lane.join(FLUSHER_LOCK).exists() && !lane.join(LEGACY_FLUSHER_LOCK).exists(), "the new protocol touched only its own lock file");
    drop(new);
}

/// Review 15's schedule against the fenced lane: an old probe holder holds the old lock without sweeping; a new client launches; its helper
/// takes the new lock and serves the receipt, and the old holder's death changes nothing.
#[test]
fn an_old_probe_holder_does_not_stop_a_new_clients_helper() {
    let lane: PathBuf = lane_with_receipts("cutover-old-holder");
    let old = hold_legacy_lock(&lane).expect("the old holder");
    unforceable(&lane, "e");
    let client_lane: PathBuf = lane.clone();
    assert_eq!(in_single_threaded_process(move || { return flusher_launch_code(launch_flusher(&client_lane)); }), Some(0));
    wait_recorded(&lane, "e");
    drop(old);
    assert!(FlusherLock::try_acquire(&lane).is_ok());
}

/// A new flush finds the lane's new lock free while an old holder holds the old one, and does its work (it is not `Held`).
#[test]
fn a_new_flush_is_not_held_by_an_old_holder() {
    let lane: PathBuf = lane_with_receipts("cutover-not-held");
    let _old = hold_legacy_lock(&lane).expect("the old holder");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt");
    let outcome = flush_lane(&lane).expect("the flush");
    assert!(matches!(outcome, git_policy_cli::commit_flusher::Flushed::Finished { forced: 1 }), "{:?}", outcome);
}

/// Old and new flushers overlap: both sweep the same receipts, and forcing twice is harmless.
#[test]
fn an_old_flusher_and_a_new_one_may_force_the_same_receipt_twice() {
    let lane: PathBuf = lane_with_receipts("cutover-overlap");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt");
    let _old = hold_legacy_lock(&lane).expect("the old flusher holds the old lock and forces the receipt itself");
    git_policy_cli::commit_flusher::sync_path(&lane.join(RECEIPT_DIR).join("r1")).expect("the old flusher's forcing");
    assert!(flush_lane(&lane).is_ok(), "the new flusher forces it again");
    assert!(fs::read(lane.join(RECEIPT_DIR).join("r1")).is_ok(), "nothing was lost by the second forcing");
}

/// A service child with a thread that reads its standard error line by line, so that a test waits for what the service says it has done and
/// not for a time. Killed (and waited for within a bound) when the test ends.
struct Service {
    child: Child,
    lines: Receiver<String>,
    heard: RefCell<Vec<String>>,
}

impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let until: Instant = Instant::now() + WAIT;
        while Instant::now() < until && matches!(self.child.try_wait(), Ok(None)) {
            std::thread::sleep(Duration::from_millis(5));
        }
        if matches!(self.child.try_wait(), Ok(None)) {
            // Not reaped in its bound: handed to a thread of the test that waits, so the test does not.
            let mut stuck: Child = std::mem::replace(&mut self.child, Command::new("true").spawn().expect("a placeholder child"));
            std::thread::spawn(move || { let _ = stuck.wait(); });
        }
    }
}

impl Service {
    /// Waits until the service has written a line containing `needle` (counting the lines heard so far), at most `WAIT`.
    fn wait_for_line(&self, needle: &str) {
        if self.heard.borrow().iter().any(|line| { return line.contains(needle); }) {
            return;
        }
        let deadline: Instant = Instant::now() + WAIT;
        loop {
            let left: Duration = deadline.saturating_duration_since(Instant::now());
            match self.lines.recv_timeout(left.max(Duration::from_millis(1))) {
                Ok(line) => {
                    let found: bool = line.contains(needle);
                    self.heard.borrow_mut().push(line);
                    if found {
                        return;
                    }
                }
                Err(_) => panic!("the service never said {:?}; it said {:?}", needle, self.heard.borrow()),
            }
        }
    }

    /// How many lines containing `needle` the service has written so far (lines are drained first).
    fn count_lines(&self, needle: &str) -> usize {
        while let Ok(line) = self.lines.try_recv() {
            self.heard.borrow_mut().push(line);
        }
        return self.heard.borrow().iter().filter(|line| { return line.contains(needle); }).count();
    }

    /// Whether the process is still running.
    fn alive(&mut self) -> bool {
        return matches!(self.child.try_wait(), Ok(None));
    }
}

/// Starts the real `cli-git-service` with a journal of its own and an abstract socket of its own; `lane` and `interval_millis` are the lane
/// switch and its interval (the service is the shipped form without them).
fn start_service(label: &str, lane: Option<&Path>, interval_millis: Option<&str>) -> Service {
    let journal: PathBuf = scratch(&format!("{}-journal", label));
    let mut command = Command::new(env!("CARGO_BIN_EXE_cli-git-service"));
    command.arg("--journal").arg(&journal).arg("--abstract").arg(format!("cutover-{}-{}", label, std::process::id())).args(["--idle-seconds", "120"]);
    if let Some(path) = lane {
        command.arg("--lane").arg(path);
    }
    if let Some(millis) = interval_millis {
        command.arg("--lane-interval-millis").arg(millis);
    }
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
    let mut child: Child = command.spawn().expect("the service starts");
    let stderr = child.stderr.take().expect("a piped standard error");
    let (sender, lines) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                return;
            }
        }
    });
    return Service { child, lines, heard: RefCell::new(Vec::new()) };
}

/// The real service, started with the lane switch, sweeps the lane at its start: a receipt that was there before it started is served, and
/// the service says when the startup sweep is complete.
#[test]
fn the_real_service_sweeps_the_lane_at_its_start() {
    let lane: PathBuf = lane_with_receipts("service-startup");
    unforceable(&lane, "before");
    let service = start_service("startup", Some(&lane), None);
    service.wait_for_line("lane startup complete");
    wait_recorded(&lane, "before");
}

/// The periodic pass is the failure owner of a unit that died after its execution. The orphan is created only after the service said that
/// its startup sweep is complete, and nothing rings: the pass that serves it is a periodic one (a pass line after the startup line).
#[test]
fn the_real_services_periodic_pass_serves_a_receipt_nobody_rang_for() {
    let lane: PathBuf = lane_with_receipts("service-periodic");
    let service = start_service("periodic", Some(&lane), Some("50"));
    service.wait_for_line("lane startup complete");
    // The first periodic pass is observed first: one pass is due at the start, so only a pass after it shows that the loop goes on.
    let deadline: Instant = Instant::now() + WAIT;
    while service.count_lines("lane pass complete") < 1 {
        assert!(Instant::now() < deadline, "no periodic pass was reported after the startup sweep");
        std::thread::sleep(Duration::from_millis(10));
    }
    let before: usize = service.count_lines("lane pass complete");
    unforceable(&lane, "orphan");
    wait_recorded(&lane, "orphan");
    while service.count_lines("lane pass complete") <= before {
        assert!(Instant::now() < deadline, "no further periodic pass was reported");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The doorbell: with a long interval no periodic pass can come, and the startup sweep is complete before the receipt exists, so the pass
/// that serves it is the one the ring started.
#[test]
fn a_ring_makes_the_real_service_serve_the_lane_and_a_ring_with_no_service_says_so() {
    let lane: PathBuf = lane_with_receipts("service-ring");
    assert!(!ring_running_service(&lane).expect("a ring with no service is not an error"), "a ring was reported answered with no service");
    let service = start_service("ring", Some(&lane), Some("3600000"));
    service.wait_for_line("lane startup complete");
    let before: usize = service.count_lines("lane pass complete");
    unforceable(&lane, "rung");
    assert!(ring_running_service(&lane).expect("ring"));
    wait_recorded(&lane, "rung");
    let pass_deadline: Instant = Instant::now() + WAIT;
    while service.count_lines("lane pass complete") <= before {
        assert!(Instant::now() < pass_deadline, "the ring started no pass");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The client step with a real ring: a running service takes it (nothing is launched); with no service the flusher is launched.
#[test]
fn the_client_step_rings_the_real_service_and_launches_only_when_nobody_answers() {
    struct Spy(usize);
    impl ManagerRoute for Spy {
        fn start(&mut self, _lane: &Path) -> Result<String, ManagerRefused> {
            self.0 += 1;
            return Ok("unit".to_string());
        }
    }
    let lane: PathBuf = lane_with_receipts("service-client");
    let mut spy: Spy = Spy(0);
    let alone = client_after_accept(&lane, RECEIPT_ID, &mut |asked: &Path| { return ring_running_service(asked); }, &mut spy).expect("the step");
    assert_eq!(alone, ClientStep::Launched(Routed::Manager { unit: "unit".to_string() }));
    assert_eq!(spy.0, 1);
    let service = start_service("client", Some(&lane), Some("3600000"));
    service.wait_for_line("lane startup complete");
    let mut second_spy: Spy = Spy(0);
    let rung = client_after_accept(&lane, "fedcba9876543210fedcba9876543210", &mut |asked: &Path| { return ring_running_service(asked); }, &mut second_spy).expect("the step");
    assert_eq!(rung, ClientStep::Rung);
    assert_eq!(second_spy.0, 0, "a running service took the ring, so nothing was launched");
}

/// The switch: the same binary without `--lane` is alive and touches no lane, and the shipped unit file does not pass the flag.
#[test]
fn the_real_service_without_the_lane_switch_is_alive_and_does_not_touch_a_lane() {
    assert!(!git_policy_cli::service_provision::SERVICE_UNIT.contains("--lane"), "the shipped unit passes the lane switch");
    let lane: PathBuf = lane_with_receipts("service-no-switch");
    unforceable(&lane, "untouched");
    let mut service = start_service("no-switch", None, Some("50"));
    service.wait_for_line("ready");
    std::thread::sleep(Duration::from_millis(800));
    assert!(service.alive(), "the service ended");
    assert_eq!(service.count_lines("lane "), 0, "the service spoke of a lane it was not told about");
    assert!(!lane.join(RECEIPT_ERRORS).exists() && !lane.join(FLUSHER_LOCK).exists() && !lane.join("service.ring").exists(), "the service worked on a lane it was not told about");
}

/// Restart between a ring and its processing: a stand-in service takes the ring and dies without processing it; the receipt is still there,
/// and the next service's startup sweep serves it (the ring is a hint, never the obligation).
#[test]
fn a_service_that_dies_between_a_ring_and_its_pass_leaves_the_receipt_to_the_next_services_sweep() {
    let lane: PathBuf = lane_with_receipts("service-restart");
    let stand_in = bind_ring(&lane).expect("a stand-in service holds the doorbell");
    unforceable(&lane, "lost-ring");
    assert!(ring_running_service(&lane).expect("ring"), "the stand-in took the ring");
    drop(stand_in);
    assert!(!ring_running_service(&lane).expect("ring"), "the stand-in is gone");
    let service = start_service("restart", Some(&lane), Some("3600000"));
    service.wait_for_line("lane startup complete");
    wait_recorded(&lane, "lost-ring");
}

/// Two services on one lane: the second stops at once with the failure and the first goes on.
#[test]
fn a_second_service_on_one_lane_stops_at_once_and_the_first_goes_on() {
    let lane: PathBuf = lane_with_receipts("service-two");
    let mut first = start_service("two-a", Some(&lane), Some("3600000"));
    first.wait_for_line("lane startup complete");
    let mut second = start_service("two-b", Some(&lane), Some("3600000"));
    let deadline: Instant = Instant::now() + WAIT;
    let status = loop {
        if let Ok(Some(status)) = second.child.try_wait() {
            break status;
        }
        assert!(Instant::now() < deadline, "the second service kept running on a lane that has a service");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(status.code(), Some(2));
    assert_eq!(second.count_lines("ready"), 0, "the second service announced itself ready");
    assert!(first.alive());
}

/// An interval of zero and a lane that others can write stop the service before it is ready.
#[test]
fn the_real_service_refuses_a_zero_interval_and_a_lane_others_can_write() {
    use std::os::unix::fs::PermissionsExt;
    let lane: PathBuf = lane_with_receipts("service-refusals");
    let mut zero = start_service("zero", Some(&lane), Some("0"));
    let deadline: Instant = Instant::now() + WAIT;
    while zero.alive() {
        assert!(Instant::now() < deadline, "the service ran with an interval of zero");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(zero.child.try_wait().expect("status").and_then(|status| { return status.code(); }), Some(2));
    assert_eq!(zero.count_lines("ready"), 0);
    fs::set_permissions(&lane, fs::Permissions::from_mode(0o777)).expect("a lane everyone can write");
    let mut open = start_service("open", Some(&lane), Some("50"));
    let open_deadline: Instant = Instant::now() + WAIT;
    while open.alive() {
        assert!(Instant::now() < open_deadline, "the service ran on a lane others can write");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(open.child.try_wait().expect("status").and_then(|status| { return status.code(); }), Some(2));
}

/// A disposal the person authorized and a crash interrupted is finished by a later pass of the real service when the state writer was busy
/// at the startup sweep: the retry is on every pass, not only at the start.
#[test]
fn the_real_service_finishes_an_interrupted_disposal_that_the_startup_sweep_found_busy() {
    let (lane, state, envelope) = lane_with_envelope("service-disposal");
    drop(envelope);
    let saved: ClosureEntry = create_saved_content(&lane, b"the content to dispose of").expect("saved content");
    let name: String = saved.path().file_name().expect("name").to_string_lossy().to_string();
    let closure: Vec<ClosureEntry> = git_policy_cli::commit_lane_lifecycle::capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("document");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the first record");
    let (child_lane, child_state, child_name) = (lane.clone(), state.clone(), name.clone());
    let status: Option<i32> = in_single_threaded_process(move || {
        set_fault_point(Some(die_after_the_line));
        let _ = dispose_saved_content(&child_lane, &child_state, &DisposalAuthorization::explicit(&child_name));
        return 0;
    });
    assert_eq!(status, Some(77), "the disposal did not reach the crash point after its line");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    let busy: StateWriter = StateWriter::acquire(&state).expect("the state writer, held while the service starts");
    let service = start_service("disposal", Some(&lane), Some("50"));
    service.wait_for_line("lane startup complete");
    assert!(lane.join("saved").join(&name).exists(), "the startup sweep finished the disposal although the writer was held");
    drop(busy);
    let deadline: Instant = Instant::now() + WAIT;
    while lane.join("saved").join(&name).exists() {
        assert!(Instant::now() < deadline, "no later pass finished the interrupted disposal");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(select(&state).expect("readable").is_some());
}

/// Dies after the disposal's line is appended.
fn die_after_the_line(point: &str) {
    if point == "disposal:after-line" {
        // SAFETY: `_exit` takes an integer and never returns.
        unsafe { libc::_exit(77) };
    }
}

/// The guard behind a unit-name collision: with the clock frozen, two launches still get different nonces, because the random source is the
/// primary one. Without it (the clock alone) the names collide.
#[test]
fn a_frozen_clock_does_not_make_two_launches_share_a_unit_name() {
    git_policy_cli::commit_flusher_unit::inject_frozen_clock(Some(42));
    let (first, second) = (git_policy_cli::commit_flusher_unit::fresh_nonce(), git_policy_cli::commit_flusher_unit::fresh_nonce());
    git_policy_cli::commit_flusher_unit::inject_frozen_clock(None);
    assert_ne!(first, second);
}
