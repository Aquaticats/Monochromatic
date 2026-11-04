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
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_flusher::{FLUSHER_LOCK, FlusherLock, LEGACY_FLUSHER_LOCK, RECEIPT_DIR, RECEIPT_ERRORS, flush_lane, hold_legacy_lock, launch_flusher};
use git_policy_cli::commit_flusher_unit::{ManagerRefused, ManagerRoute};
use git_policy_cli::commit_lane_lifecycle::{ClientStep, Routed, client_after_accept};
use git_policy_cli::commit_lane_service::ring_running_service;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
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

/// A service child that is stopped when the test ends.
struct Service(Child);

impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Starts the real `cli-git-service` with the lane switch (or without it), a journal of its own and an abstract socket of its own.
fn start_service(label: &str, lane: Option<&Path>, interval_millis: Option<u64>) -> Service {
    let journal: PathBuf = scratch(&format!("{}-journal", label));
    let mut command = Command::new(env!("CARGO_BIN_EXE_cli-git-service"));
    command.arg("--journal").arg(&journal).arg("--abstract").arg(format!("cutover-{}-{}", label, std::process::id())).args(["--idle-seconds", "120"]);
    if let Some(path) = lane {
        command.arg("--lane").arg(path);
    }
    if let Some(millis) = interval_millis {
        command.arg("--lane-interval-millis").arg(millis.to_string());
    }
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    return Service(command.spawn().expect("the service starts"));
}

/// Waits until the service answers the doorbell of `lane`.
fn wait_for_the_doorbell(lane: &Path) {
    let deadline: Instant = Instant::now() + WAIT;
    while !ring_running_service(lane).unwrap_or(false) {
        assert!(Instant::now() < deadline, "the service never bound its doorbell");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The real service, started with the lane switch, sweeps the lane at its start: a receipt that was there before it started is served.
#[test]
fn the_real_service_sweeps_the_lane_at_its_start() {
    let lane: PathBuf = lane_with_receipts("service-startup");
    unforceable(&lane, "before");
    let _service = start_service("startup", Some(&lane), None);
    wait_recorded(&lane, "before");
}

/// The periodic pass is the failure owner of a unit that died after its execution: with no ring and no flusher, a receipt created after the
/// service started is served within the interval.
#[test]
fn the_real_services_periodic_pass_serves_a_receipt_nobody_rang_for() {
    let lane: PathBuf = lane_with_receipts("service-periodic");
    // Readiness is read from the startup sweep (a receipt there before the start is recorded), never by ringing: a ring would run a pass
    // itself and the test would not tell the periodic pass from it.
    unforceable(&lane, "ready");
    let _service = start_service("periodic", Some(&lane), Some(50));
    wait_recorded(&lane, "ready");
    unforceable(&lane, "orphan");
    wait_recorded(&lane, "orphan");
}

/// The doorbell: with a long interval, a receipt is served because the client rang, and the ring says whether a service answered.
#[test]
fn a_ring_makes_the_real_service_serve_the_lane_and_a_ring_with_no_service_says_so() {
    let lane: PathBuf = lane_with_receipts("service-ring");
    assert!(!ring_running_service(&lane).expect("a ring with no service is not an error"), "a ring was reported answered with no service");
    let _service = start_service("ring", Some(&lane), Some(3_600_000));
    wait_for_the_doorbell(&lane);
    unforceable(&lane, "rung");
    assert!(ring_running_service(&lane).expect("ring"));
    wait_recorded(&lane, "rung");
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
    let _service = start_service("client", Some(&lane), Some(3_600_000));
    wait_for_the_doorbell(&lane);
    let mut second_spy: Spy = Spy(0);
    let rung = client_after_accept(&lane, "fedcba9876543210fedcba9876543210", &mut |asked: &Path| { return ring_running_service(asked); }, &mut second_spy).expect("the step");
    assert_eq!(rung, ClientStep::Rung);
    assert_eq!(second_spy.0, 0, "a running service took the ring, so nothing was launched");
}

/// The switch: the same binary without `--lane` touches no lane, which is how the shipped units run it.
#[test]
fn the_real_service_without_the_lane_switch_does_not_touch_a_lane() {
    let lane: PathBuf = lane_with_receipts("service-no-switch");
    unforceable(&lane, "untouched");
    let _service = start_service("no-switch", None, Some(50));
    std::thread::sleep(Duration::from_millis(800));
    assert!(!lane.join(RECEIPT_ERRORS).exists() && !lane.join(FLUSHER_LOCK).exists(), "the service worked on a lane it was not told about");
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
