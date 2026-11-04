//! What:
//!  Controls of plan step 2's lane lifecycle (owner decisions 22 and 23): the client's call site (receipt first, ring a running service,
//!  otherwise launch through the manager route with the double fork as fallback), the syncer's pass with the retry of recorded
//!  receipt errors, the replacement owner's startup sweep, the planned-shutdown seal with its final sweep and reclaiming checkpoint, and the
//!  transient unit's command line.
//! Why:
//!  Each function is the call site of a component that had none: the receipt is the registration of the obligation to force, so it exists
//!  before anything is rung or launched; a refused manager route must not leave a receipt unserved; a recorded error is retried by the
//!  running owner and not left to the next start; the seal is the real reclaimer against which a departure's acknowledgment must still be
//!  covered.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('the receipt exists before the ring', ...); test('a refused manager falls back to the fork', ...);
//! ```
#![cfg(feature = "test-seams")]

use crate::commit_helper_launch_tests::flusher_launch_code;
use crate::commit_state_log_tests::lane_with_envelope;
use crate::model_fixture::{in_single_threaded_process, scratch};
use git_policy_cli::commit_flusher::{FlushError, FlusherLaunchError, RECEIPT_DIR, RECEIPT_ERRORS};
use git_policy_cli::commit_flusher_unit::{ManagerRefused, ManagerRoute, transient_unit_arguments, unit_name};
use git_policy_cli::commit_identity::set_fault_point;
use git_policy_cli::commit_lane_lifecycle::{ClientError, ClientStep, Routed, capture_lane_closure, client_after_accept, departure_acknowledge, seal_for_shutdown, startup_sweep, syncer_pass};
use git_policy_cli::commit_state_log::{ClosureEntry, Consumption, ConsumptionKind, EntryKind, LaneDocument, LaneState, Marker, RequestPhase, RequestState, StateWriter, read, state_covers, sync_prefix};
use std::cell::Cell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The longest a test waits for a launched flusher; the work is a few file operations.
const FLUSHER_WAIT: Duration = Duration::from_secs(10);

/// A request identity for a receipt.
const RECEIPT_ID: &str = "0123456789abcdef0123456789abcdef";

/// A manager route a test controls.
struct FakeRoute {
    /// What `start` answers.
    answer: Result<String, String>,
    /// How many times `start` was called, and whether the receipt existed at each call.
    calls: Vec<bool>,
}

impl ManagerRoute for FakeRoute {
    fn start(&mut self, lane: &Path) -> Result<String, ManagerRefused> {
        self.calls.push(lane.join(RECEIPT_DIR).join(RECEIPT_ID).exists());
        return self.answer.clone().map_err(ManagerRefused);
    }
}

/// A lane for a client.
fn lane_for(label: &str) -> PathBuf {
    return scratch(label);
}

/// The receipt is the registration of the obligation to force: it exists before the doorbell rings, and a running service that takes the
/// ring launches nothing.
#[test]
fn the_receipt_exists_before_the_ring_and_a_rung_service_launches_nothing() {
    let lane: PathBuf = lane_for("life-ring");
    let mut route: FakeRoute = FakeRoute { answer: Ok("unit".to_string()), calls: Vec::new() };
    let existed_at_ring: Cell<Option<bool>> = Cell::new(None);
    let step: ClientStep = client_after_accept(
        &lane,
        RECEIPT_ID,
        &mut |asked: &Path| { existed_at_ring.set(Some(asked.join(RECEIPT_DIR).join(RECEIPT_ID).exists())); return Ok(true); },
        &mut route,
    )
    .expect("the client step");
    assert_eq!(step, ClientStep::Rung);
    assert_eq!(existed_at_ring.get(), Some(true), "the receipt was created before the ring");
    assert!(route.calls.is_empty(), "a rung service needs no launch");
}

/// A ring that is not answered (no service, or an error reaching it) launches the flusher through the manager route, with the receipt
/// already in place.
#[test]
fn an_unanswered_ring_launches_through_the_manager_with_the_receipt_in_place() {
    for ring in [Ok(false), Err(io::Error::from(io::ErrorKind::ConnectionRefused))] {
        let lane: PathBuf = lane_for("life-launch");
        let mut route: FakeRoute = FakeRoute { answer: Ok("cli-git-flusher-x".to_string()), calls: Vec::new() };
        let mut answer: Option<io::Result<bool>> = Some(ring);
        let step: ClientStep = client_after_accept(&lane, RECEIPT_ID, &mut |_lane: &Path| { return answer.take().expect("one ring"); }, &mut route).expect("the client step");
        assert_eq!(step, ClientStep::Launched(Routed::Manager { unit: "cli-git-flusher-x".to_string() }));
        assert_eq!(route.calls, [true], "one manager start, the receipt existing at that call");
    }
}

/// A manager that refuses leaves the receipt to the double fork, which forces it: a dangling receipt in the lane is recorded by the helper
/// the fallback launched.
#[test]
fn a_refused_manager_falls_back_to_the_fork_and_the_receipts_are_served() {
    let lane: PathBuf = lane_for("life-fallback");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("dangling")).expect("a receipt that cannot be forced");
    let client_lane: PathBuf = lane.clone();
    let code: Option<i32> = in_single_threaded_process(move || {
        let mut route: FakeRoute = FakeRoute { answer: Err("no manager".to_string()), calls: Vec::new() };
        return match client_after_accept(&client_lane, RECEIPT_ID, &mut |_lane: &Path| { return Ok(false); }, &mut route) {
            Ok(ClientStep::Launched(Routed::Fork)) => 0,
            Ok(_) => 90,
            Err(ClientError::Launch(error)) => 100 + flusher_launch_code(Err(error)),
            Err(ClientError::Receipt(_)) => 91,
        };
    });
    assert_eq!(code, Some(0), "the fallback launched a helper");
    let deadline: Instant = Instant::now() + FLUSHER_WAIT;
    loop {
        if fs::read_to_string(lane.join(RECEIPT_ERRORS)).is_ok_and(|text| { return text.contains("dangling"); }) {
            break;
        }
        assert!(Instant::now() < deadline, "the fallback helper never swept the lane");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The single-thread requirement binds the fork and not the manager: a process with another thread starts the flusher through the manager,
/// and is refused (with every receipt recorded) when it must fall back to the fork.
#[test]
fn a_process_with_a_second_thread_uses_the_manager_and_is_refused_the_fork() {
    let lane: PathBuf = lane_for("life-threads");
    let client_lane: PathBuf = lane.clone();
    let code: Option<i32> = in_single_threaded_process(move || {
        let extra = std::thread::spawn(|| { std::thread::sleep(Duration::from_secs(5)); });
        let mut manager: FakeRoute = FakeRoute { answer: Ok("unit".to_string()), calls: Vec::new() };
        let via_manager = client_after_accept(&client_lane, RECEIPT_ID, &mut |_lane: &Path| { return Ok(false); }, &mut manager);
        if !matches!(via_manager, Ok(ClientStep::Launched(Routed::Manager { .. }))) {
            return 80;
        }
        let mut refusing: FakeRoute = FakeRoute { answer: Err("no manager".to_string()), calls: Vec::new() };
        let via_fork = client_after_accept(&client_lane, "fedcba9876543210fedcba9876543210", &mut |_lane: &Path| { return Ok(false); }, &mut refusing);
        drop(extra);
        return match via_fork {
            Err(ClientError::Launch(FlusherLaunchError::Launch(git_policy_cli::commit_helper_launch::LaunchError::Threaded(_)))) => 0,
            _ => 81,
        };
    });
    assert_eq!(code, Some(0));
    let record: String = fs::read_to_string(lane.join(RECEIPT_ERRORS)).expect("the refused launch recorded the receipts");
    assert!(record.contains(RECEIPT_ID) && record.contains("fedcba9876543210fedcba9876543210"), "{}", record);
}

/// The syncer's pass retries what the record names: the record is cleared once the receipts are forced, and kept (with its lines) when the
/// forcing still fails.
#[test]
fn the_syncer_retries_recorded_errors_and_clears_the_record_only_when_forced() {
    let lane: PathBuf = lane_for("life-syncer");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    let dangling: PathBuf = lane.join(RECEIPT_DIR).join("r1");
    std::os::unix::fs::symlink(lane.join("nowhere"), &dangling).expect("a receipt that cannot be forced");
    fs::write(lane.join(RECEIPT_ERRORS), "r1\tinjected\nr9\tfor a receipt that is no longer in the directory\n").expect("a recorded error");
    let failed = syncer_pass(&lane);
    assert!(matches!(failed, Err(FlushError::Forcing(_))), "{:?}", failed.map(|pass| { return pass.forced; }));
    let kept: String = fs::read_to_string(lane.join(RECEIPT_ERRORS)).expect("the record");
    assert!(kept.contains("r1") && kept.contains("r9"), "the record still names every receipt it named: {}", kept);
    fs::remove_file(&dangling).expect("the cause is gone");
    fs::write(&dangling, b"receipt").expect("a receipt that can be forced");
    let pass = syncer_pass(&lane).expect("the pass");
    assert_eq!(pass.retried, 2, "both recorded lines, the one for the receipt that is gone included");
    assert!(!lane.join(RECEIPT_ERRORS).exists() || fs::read_to_string(lane.join(RECEIPT_ERRORS)).expect("the record").is_empty(), "the record is cleared once its receipts are forced");
}

/// A record that arrives while a pass is retrying is not lost: only the lines the pass took are removed.
#[test]
fn an_error_recorded_during_a_retry_survives_the_pass() {
    let lane: PathBuf = lane_for("life-syncer-race");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt");
    fs::write(lane.join(RECEIPT_ERRORS), "r1\told\n").expect("a recorded error");
    // The recorder opened the record before the pass began and writes through that descriptor while the pass forces (review of step 2).
    let mut late = fs::OpenOptions::new().append(true).open(lane.join(RECEIPT_ERRORS)).expect("the record, opened before the pass");
    let pass = git_policy_cli::commit_lane_lifecycle::syncer_pass_with(&lane, &mut || {
        io::Write::write_all(&mut late, b"r2\tlate\n").expect("a late line through the old descriptor");
    })
    .expect("the pass");
    assert_eq!(pass.retried, 1);
    assert_eq!(fs::read_to_string(lane.join(RECEIPT_ERRORS)).expect("the record"), "r1\told\nr2\tlate\n", "the late line survives, and the line the pass retried is retried again (idempotent) instead of being cut under a writer");
}

/// Competing passes: the second pass runs completely inside the first (through its hook) and clears the record; the first pass then finds
/// the record changed and removes nothing. No pass removes another pass's snapshot, because there is no snapshot file.
#[test]
fn competing_syncer_passes_do_not_lose_or_duplicate_records() {
    let lane: PathBuf = lane_for("life-syncer-compete");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(lane.join(RECEIPT_DIR).join("r1"), b"receipt").expect("a receipt");
    fs::write(lane.join(RECEIPT_ERRORS), "r1\told\n").expect("a recorded error");
    let inner_lane: PathBuf = lane.clone();
    let outer = git_policy_cli::commit_lane_lifecycle::syncer_pass_with(&lane, &mut || {
        let inner = syncer_pass(&inner_lane).expect("the inner pass");
        assert_eq!(inner.retried, 1);
    })
    .expect("the outer pass");
    assert_eq!(outer.retried, 1);
    assert!(fs::read_to_string(lane.join(RECEIPT_ERRORS)).unwrap_or_default().is_empty());
}

/// The replacement owner's startup sweep forces every receipt in the directory whether or not the record names it, and reads the state log
/// once.
#[test]
fn the_startup_sweep_forces_every_receipt_whether_or_not_the_record_names_it() {
    let (lane, state, envelope) = lane_with_envelope("life-startup");
    let request: ClosureEntry = ClosureEntry::capture(&lane, &envelope, EntryKind::InboxEnvelope).expect("capture");
    let document: Vec<u8> = LaneDocument::from_closure(std::slice::from_ref(&request)).expect("document").encode();
    StateWriter::acquire(&state).expect("writer").append_with(7, &document, std::slice::from_ref(&request)).expect("a record");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(lane.join(RECEIPT_DIR).join("unnamed"), b"receipt").expect("a receipt the record does not name");
    let recovery = startup_sweep(&lane, &state).expect("the sweep");
    assert_eq!(recovery.generation, Some(7));
    assert!(recovery.forced >= 1);
    assert!(recovery.state_error.is_none());
}

/// The shipped state of a lane stays readable: the seal's closure holds everything the lane holds, marks and the order file included.
fn lane_with_everything(label: &str) -> (PathBuf, PathBuf, String) {
    let (lane, state, envelope) = lane_with_envelope(label);
    let id: String = envelope.file_name().expect("name").to_string_lossy().to_string();
    fs::create_dir_all(lane.join("decision")).expect("decision directory");
    fs::write(lane.join("decision").join(format!("{}.withdraw", id)), b"").expect("a mark");
    fs::write(lane.join("order"), b"adopt a\n").expect("an order file");
    git_policy_cli::commit_state_log::create_saved_content(&lane, b"objects of b").expect("saved content");
    return (lane, state, id);
}

/// The seal captures what the lane holds and writes a reclaiming checkpoint: the predecessor segment is gone, the document names every
/// reference, and the previous state (marker, tail) is kept.
#[test]
fn the_seal_writes_a_reclaiming_checkpoint_of_everything_the_lane_holds() {
    let (lane, state, id) = lane_with_everything("life-seal");
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    assert!(closure.len() >= 4, "request, mark, order file and saved content: {}", closure.len());
    let previous: LaneState = LaneState { marker: Marker::Recovering, tail: vec![id.clone()], ..LaneState::default() };
    let first: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible").with_state(previous.clone());
    StateWriter::acquire(&state).expect("writer").append_with(1, &first.encode(), &closure).expect("the state before the seal");
    let sealed = seal_for_shutdown(&lane, &state).expect("the seal");
    assert_eq!(sealed.generation, 2);
    assert!(!state.join("state.log.1").exists(), "the contained predecessor was reclaimed");
    let (generation, bytes) = read(&state).expect("readable").expect("state");
    assert_eq!(generation, 2);
    let document: LaneDocument = LaneDocument::parse(&bytes).expect("a checked document");
    assert_eq!(document.state(), &previous, "the seal keeps the state it found");
}

/// The step-1 row's component test: a departure acknowledged a record and the seal (the real reclaimer) replaced its segment; the
/// acknowledgment is still covered when the record was a checked document. Over an opaque record the seal reclaims nothing and the
/// acknowledgment is not covered: the departure's verification says so, and the segment stays.
#[test]
fn a_departures_acknowledgment_survives_the_seal_that_replaces_its_segment() {
    let (lane, state, _id) = lane_with_everything("life-seal-departure");
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the promised record");
    let promised = sync_prefix(&state).expect("the departure's acknowledgment");
    seal_for_shutdown(&lane, &state).expect("the seal");
    assert!(!state.join("state.log.1").exists(), "the seal reclaimed the segment that holds the acknowledged record");
    assert!(state_covers(&state, &promised).expect("readable"), "the acknowledgment is covered by the sealed checkpoint");
}

/// Over an opaque record the seal refuses (the fifth review: a newer checked root with an empty state would have been the one recovery
/// selects), the opaque record stays the root, and the acknowledgment of it is still covered.
#[test]
fn a_departures_acknowledgment_of_an_opaque_record_is_kept_because_the_seal_refuses_it() {
    let (lane, state, _id) = lane_with_everything("life-seal-opaque");
    StateWriter::acquire(&state).expect("writer").append(1, b"an opaque document").expect("an opaque record");
    let promised = sync_prefix(&state).expect("acknowledged");
    assert!(seal_for_shutdown(&lane, &state).is_err(), "the seal replaced a root it could not read");
    assert!(state.join("state.log.1").exists(), "the opaque record's segment is still there");
    assert!(state_covers(&state, &promised).expect("readable"), "the opaque record is still the root, so its acknowledgment is covered");
}

/// A seal refuses what it cannot capture: a lane file that the representation cannot hold makes the seal fail rather than write a document
/// that omits it.
#[test]
fn the_seal_refuses_when_the_lane_changed_under_its_capture() {
    let (lane, state, id) = lane_with_everything("life-seal-changed");
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible").with_state(LaneState { tail: vec![id.clone()], ..LaneState::default() });
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the state before the seal");
    fs::remove_file(lane.join("inbox").join(&id)).expect("the request vanished while the state still tails it");
    assert!(seal_for_shutdown(&lane, &state).is_err(), "a state that names a request the lane no longer holds was sealed");
    assert!(state.join("state.log.1").exists());
}

/// The transient unit's command line: a user unit with a fresh name that is unloaded after it ran, started and waited for (so a refusal is
/// seen), running the service program in its internal flusher mode on the lane. No `--no-block`: a start that was not confirmed would leave
/// the receipt unserved.
#[test]
fn the_transient_unit_runs_the_internal_flusher_entry_and_waits_for_the_start() {
    let lane: PathBuf = PathBuf::from("/lane with space/x");
    let program: PathBuf = PathBuf::from("/opt/cli-git/cli-git-native");
    let name: String = unit_name(&lane, 0x1234);
    let arguments: Vec<String> = transient_unit_arguments(&name, &program, &lane).iter().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
    assert_eq!(arguments[0], "--user");
    for wanted in ["--collect", "--quiet"] {
        assert!(arguments.iter().any(|argument| { return argument == wanted; }), "{}", wanted);
    }
    assert!(!arguments.iter().any(|argument| { return argument == "--no-block"; }));
    assert!(arguments.contains(&format!("--unit={}", name)));
    let at: usize = arguments.iter().position(|argument| { return argument == "--"; }).expect("the command follows --");
    assert_eq!(arguments[at + 1..], ["/opt/cli-git/cli-git-native", "cli-git", "service", "run", "--flush-lane", "/lane with space/x"]);
    assert_ne!(unit_name(&lane, 1), unit_name(&lane, 2), "every launch has its own unit name, so a refusal never means a flusher already runs");
    assert!(name.starts_with("cli-git-flusher-") && name.chars().all(|c| { return c.is_ascii_alphanumeric() || c == '-'; }), "{}", name);
}

/// The real manager, on a disposable lane, with the real service program: the unit runs `--flush-lane` and the flusher records a receipt it
/// cannot force. Skipped (loudly) when there is no user manager. The unit name is unique and `--collect` unloads it; it touches nothing but
/// the scratch lane.
#[test]
fn the_real_user_manager_runs_the_flusher_entry_of_the_service_program() {
    use git_policy_cli::commit_flusher_unit::TransientUnitRoute;
    let mut probe_command = std::process::Command::new("systemd-run");
    probe_command.args(["--user", "--collect", "--quiet", "--wait", "--", "/bin/true"]);
    let probe = git_policy_cli::commit_flusher_unit::run_within(probe_command, FLUSHER_WAIT);
    if probe.is_err() {
        eprintln!("SKIPPED: no user manager answers systemd-run --user here ({:?})", probe);
        return;
    }
    let lane: PathBuf = scratch("life-real-manager");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    std::os::unix::fs::symlink(lane.join("nowhere"), lane.join(RECEIPT_DIR).join("dangling")).expect("a receipt that cannot be forced");
    let mut route: TransientUnitRoute = TransientUnitRoute::new(PathBuf::from("systemd-run"), PathBuf::from(env!("CARGO_BIN_EXE_cli-git-native")));
    let unit: String = route.start(&lane).expect("the manager started the unit");
    assert!(unit.starts_with("cli-git-flusher-"));
    let deadline: Instant = Instant::now() + FLUSHER_WAIT;
    loop {
        if fs::read_to_string(lane.join(RECEIPT_ERRORS)).is_ok_and(|text| { return text.contains("dangling"); }) {
            break;
        }
        assert!(Instant::now() < deadline, "the unit {} never swept the lane", unit);
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// A manager that cannot be reached is a refusal and not a hang: a program that does not exist is a refusal, and a start that does not
/// finish within its wait is ended and is a refusal too; the caller turns both into the fallback.
#[test]
fn a_missing_manager_program_and_an_unfinished_start_are_refusals() {
    use git_policy_cli::commit_flusher_unit::{TransientUnitRoute, run_within};
    let lane: PathBuf = scratch("life-refusals");
    let mut missing: TransientUnitRoute = TransientUnitRoute::new(PathBuf::from("/nonexistent/systemd-run"), PathBuf::from("/nonexistent/cli-git-service"));
    assert!(missing.start(&lane).is_err());
    let wait: Duration = Duration::from_millis(50);
    let started: Instant = Instant::now();
    let mut slow = std::process::Command::new("sleep");
    slow.arg("30");
    let refusal = run_within(slow, wait);
    assert!(refusal.is_err(), "an unfinished start is a refusal");
    assert!(started.elapsed() >= wait && started.elapsed() < FLUSHER_WAIT, "the wait ends at the route's own limit: {:?}", started.elapsed());
    let mut quick = std::process::Command::new("true");
    quick.arg("--ignored");
    assert!(run_within(quick, FLUSHER_WAIT).is_ok(), "a start that finishes in time is not a refusal");
    let mut failing = std::process::Command::new("false");
    failing.arg("--ignored");
    assert!(run_within(failing, FLUSHER_WAIT).is_err(), "a start that exits non-zero is a refusal");
}

/// What the race does in the window between the departure's capture of the selected segment and its open of the path.
enum Race {
    /// The seal replaces and removes the segment (the real reclaimer).
    Seal,
    /// An opaque checkpoint at a higher generation becomes the root and the segment stays.
    UnrelatedCheckpoint,
}

thread_local! {
    /// The lane, the state directory and the action of the race, set by the test on its own thread; the fault point is a plain function.
    static RACE: std::cell::RefCell<Option<(PathBuf, PathBuf, Race)>> = const { std::cell::RefCell::new(None) };
}

/// Runs the race once at the capture window of `sync_prefix`, then clears it.
fn race_at_capture(point: &str) {
    if point != "state:after-capture" {
        return;
    }
    let Some((lane, state, action)) = RACE.with(|cell| { return cell.borrow_mut().take(); }) else { return };
    match action {
        Race::Seal => {
            seal_for_shutdown(&lane, &state).expect("the seal in the window");
        }
        Race::UnrelatedCheckpoint => {
            let generation: u64 = read(&state).expect("readable").expect("state").0 + 1;
            let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
            StateWriter::acquire(&state).expect("writer").checkpoint(generation, b"an unrelated document", &closure).expect("the unrelated checkpoint");
        }
    }
}

/// The step-1 review's departure race, on the component: the departure captured the selected segment, the seal unlinks it, and the open
/// of the path fails; the departure selects again and acknowledges the sealed root, which covers its promise (finding 5, the ENOENT schedule).
#[test]
fn the_open_of_a_segment_the_seal_unlinked_fails_with_enoent() {
    let (lane, state, _id) = lane_with_everything("life-race-seal");
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the record to acknowledge");
    assert!(git_policy_cli::commit_state_log::sync_prefix(&state).is_ok(), "no race: the first attempt works");
    RACE.with(|cell| { *cell.borrow_mut() = Some((lane.clone(), state.clone(), Race::Seal)); });
    set_fault_point(Some(race_at_capture));
    let first = git_policy_cli::commit_state_log::sync_prefix(&state);
    set_fault_point(None);
    assert!(matches!(&first, Err(error) if error.kind() == io::ErrorKind::NotFound), "the open of the unlinked path fails with ENOENT: {:?}", first.map(|prefix| { return prefix.number; }));
    assert!(!state.join("state.log.1").exists(), "the seal removed the segment the departure had captured");
}

/// The departure's own retry: the open of the unlinked path fails inside `departure_acknowledge`, which selects the sealed root.
#[test]
fn a_departure_that_finds_its_captured_segment_unlinked_acknowledges_the_sealed_root() {
    let (lane, state, _id) = lane_with_everything("life-race-seal-retry");
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the record to acknowledge");
    RACE.with(|cell| { *cell.borrow_mut() = Some((lane.clone(), state.clone(), Race::Seal)); });
    set_fault_point(Some(race_at_capture));
    let prefix = departure_acknowledge(&state, 3).expect("the departure selects again after the open of the unlinked path fails");
    set_fault_point(None);
    assert!(state_covers(&state, &prefix).expect("readable"));
    assert_eq!(prefix.generation, 2, "the acknowledged root is the seal's checkpoint");
}

/// The same window with an unrelated higher-generation checkpoint: the segment stays, the forcing succeeds, and `state_covers` is false for
/// the old capture. The departure does not return on the survival of the old file; it captures the new root.
#[test]
fn a_departure_whose_capture_is_superseded_by_an_unrelated_root_captures_again() {
    let (lane, state, _id) = lane_with_everything("life-race-unrelated");
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let document: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible");
    StateWriter::acquire(&state).expect("writer").append_with(1, &document.encode(), &closure).expect("the record to acknowledge");
    RACE.with(|cell| { *cell.borrow_mut() = Some((lane.clone(), state.clone(), Race::UnrelatedCheckpoint)); });
    set_fault_point(Some(race_at_capture));
    let old = git_policy_cli::commit_state_log::sync_prefix(&state).expect("the forcing of the surviving segment works");
    set_fault_point(None);
    assert!(!state_covers(&state, &old).expect("readable"), "the old capture is not covered by the unrelated root");
    RACE.with(|cell| { *cell.borrow_mut() = Some((lane.clone(), state.clone(), Race::UnrelatedCheckpoint)); });
    set_fault_point(Some(race_at_capture));
    let prefix = departure_acknowledge(&state, 3).expect("the departure acknowledges the root it ends with");
    set_fault_point(None);
    assert!(state_covers(&state, &prefix).expect("readable"));
    assert!(prefix.generation >= 3, "the acknowledged root is the newest: {}", prefix.generation);
}

/// A log that keeps changing, or is absent, is a bounded failure and not a loop: the attempts end and the error says so.
#[test]
fn a_departure_that_cannot_acknowledge_within_its_attempts_fails() {
    let (_lane, state, _id) = lane_with_everything("life-race-absent");
    let failed = departure_acknowledge(&state, 2);
    assert!(failed.is_err(), "an absent log was acknowledged");
}

/// Request states, the adoption order position, the consumption map and the head survive the seal and a later recovery: the sealed root,
/// read by a replacement owner's startup sweep, parses to the state that was carried.
#[test]
fn request_order_and_consumption_data_survive_the_seal_and_a_recovery() {
    let (lane, state, id) = lane_with_everything("life-roundtrip");
    let adopt: String = format!("adopt {} 0badc0de\n", id);
    fs::write(lane.join("order"), &adopt).expect("an order file that holds the request's adopt line");
    let closure: Vec<ClosureEntry> = capture_lane_closure(&lane).expect("capture");
    let carried: LaneState = LaneState {
        marker: Marker::Materializing,
        head: Some("a".repeat(40)),
        tail: vec![id.clone()],
        requests: vec![RequestState { id: id.clone(), phase: RequestPhase::Publishing, resume: "P1".to_string(), position: Some(0) }],
        consumption: vec![Consumption { path: b"dir/f.txt".to_vec(), kind: ConsumptionKind::Install, request: id.clone(), version: 1, entry: vec![1, 2, 3] }],
    };
    let first: LaneDocument = LaneDocument::from_closure(&closure).expect("expressible").with_state(carried.clone());
    StateWriter::acquire(&state).expect("writer").append_with(1, &first.encode(), &closure).expect("the state before the seal");
    seal_for_shutdown(&lane, &state).expect("the seal");
    let recovery = startup_sweep(&lane, &state).expect("the replacement owner's sweep");
    assert_eq!(recovery.generation, Some(2));
    let bytes: Vec<u8> = read(&state).expect("readable").expect("state").1;
    assert_eq!(LaneDocument::parse(&bytes).expect("a checked document").state(), &carried);
}

/// The manager's start confirmation is not a worker's readiness: a unit that executes and dies at once is reported as started (the execution
/// was confirmed), and the receipt stays in the directory for the next sweep. The test records this limit; it is not a promise that the
/// receipt was served. Skipped without a user manager.
#[test]
fn a_unit_that_executes_and_dies_is_reported_started_and_leaves_the_receipt_for_the_next_sweep() {
    use git_policy_cli::commit_flusher_unit::{ManagerRoute, TransientUnitRoute, run_within};
    let mut probe_command = std::process::Command::new("systemd-run");
    probe_command.args(["--user", "--collect", "--quiet", "--wait", "--", "/bin/true"]);
    if run_within(probe_command, FLUSHER_WAIT).is_err() {
        eprintln!("SKIPPED: no user manager answers systemd-run --user here");
        return;
    }
    let lane: PathBuf = scratch("life-dies-at-once");
    fs::create_dir_all(lane.join(RECEIPT_DIR)).expect("receipt directory");
    fs::write(lane.join(RECEIPT_DIR).join(RECEIPT_ID), b"receipt").expect("the receipt");
    let mut route: TransientUnitRoute = TransientUnitRoute::new(PathBuf::from("systemd-run"), PathBuf::from("/bin/false"));
    assert!(route.start(&lane).is_ok(), "the program was executed, so the start is confirmed");
    assert!(lane.join(RECEIPT_DIR).join(RECEIPT_ID).exists(), "the receipt stays for the next sweep");
    assert!(syncer_pass(&lane).is_ok(), "and the sweep serves it");
}
