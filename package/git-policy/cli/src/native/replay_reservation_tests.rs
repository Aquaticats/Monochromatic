//! Controls for the starvation reservation on disposable registries:
//!  request bytes, the earned threshold, grant order, holders seen by others, release, and the grant
//!  after the configured number of lost races.

use super::*;
use crate::test_support::{ActionSink, SharedSink, fixture, remove};
use crate::transaction_owner::{current_transaction_owner, encode_transaction_owner};
use crate::transaction_registry::{OWNER_FILENAME, TRANSACTION_ROOT_NAME, ensure_transaction_root};

/// Transaction IDs of the fixtures, canonical UUIDs.
const FIRST: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";
/// The second transaction of the fixtures.
const SECOND: &str = "1c7d3d2f-7a6c-4e1f-8b66-4a6e9f3f7b21";

/// A transaction directory under `registry` owned by this live process,
///  or by a foreign identity when `live` is false.
fn transaction(registry: &Path, id: &str, created_at: &str, live: bool) -> PathBuf {
    let directory: PathBuf = registry.join(id);
    std::fs::create_dir_all(&directory).expect("transaction directory");
    let mut owner = current_transaction_owner(id, created_at).expect("current owner");
    if !live {
        owner.owner_identity = String::from("linux:1");
    }
    std::fs::write(directory.join(OWNER_FILENAME), encode_transaction_owner(&owner)).expect("owner record");
    return directory;
}

/// A fixture registry root with the transaction root created.
fn registry_of(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let registry: PathBuf = root.join(TRANSACTION_ROOT_NAME);
    ensure_transaction_root(registry.as_path()).expect("registry root");
    return (root, registry);
}

/// The request file names the incumbent's object with its lost-race count and a trailing newline.
#[test]
fn request_bytes_match_the_incumbent_object() {
    assert_eq!(
        reservation_request_bytes(2),
        b"{\"schemaVersion\":2,\"state\":\"reservation-request\",\"lostRaces\":2}\n".to_vec()
    );
}

/// A lost race earns the reservation from the configured count on, never before.
#[test]
fn the_reservation_is_earned_at_the_configured_count() {
    assert!(!reservation_earned(0, 1));
    assert!(reservation_earned(1, 1));
    assert!(!reservation_earned(2, 3));
    assert!(reservation_earned(3, 3));
    assert!(reservation_earned(4, 3));
}

/// Without any registry, nobody requests; a request without a live owner does not count.
#[test]
fn requesters_are_live_transactions_with_a_request() {
    let (root, registry) = registry_of("replay-reservation-requesters");
    assert!(live_requesters(registry.as_path()).expect("empty").is_empty());
    let live: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    assert!(live_requesters(registry.as_path()).expect("no request").is_empty());
    std::fs::write(live.join(RESERVATION_REQUEST_FILENAME), reservation_request_bytes(1)).expect("request");
    assert_eq!(live_requesters(registry.as_path()).expect("requester"), vec![String::from(FIRST)]);
    let dead: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-09T00:00:00.000Z", false);
    std::fs::write(dead.join(RESERVATION_REQUEST_FILENAME), reservation_request_bytes(1)).expect("request");
    assert_eq!(live_requesters(registry.as_path()).expect("dead excluded"), vec![String::from(FIRST)]);
    remove(root.as_path());
}

/// The oldest live requester is first in grant order.
#[test]
fn the_oldest_requester_is_granted_first() {
    let (root, registry) = registry_of("replay-reservation-order");
    for (id, created) in [(FIRST, "2026-10-10T00:00:05.000Z"), (SECOND, "2026-10-10T00:00:01.000Z")] {
        let directory: PathBuf = transaction(registry.as_path(), id, created, true);
        std::fs::write(directory.join(RESERVATION_REQUEST_FILENAME), reservation_request_bytes(1)).expect("request");
    }
    assert_eq!(
        live_requesters(registry.as_path()).expect("order"),
        vec![String::from(SECOND), String::from(FIRST)]
    );
    remove(root.as_path());
}

/// A reservation held by one transaction is seen by the others until it is released;
///  release removes the request and the lock, and the grant also needs the oldest request.
#[test]
fn a_held_reservation_is_seen_by_others_until_released() {
    let (root, registry) = registry_of("replay-reservation-holder");
    let first_directory: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    let second_directory: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-10T00:00:01.000Z", true);
    let mut first: LandingReservation = LandingReservation::open(registry.as_path(), first_directory.as_path(), FIRST, false);
    let second: LandingReservation = LandingReservation::open(registry.as_path(), second_directory.as_path(), SECOND, false);
    assert!(first.reserve(1).expect("granted"));
    assert!(first_directory.join(RESERVATION_REQUEST_FILENAME).exists());
    assert!(!first.held_by_another().expect("own view"));
    assert!(second.held_by_another().expect("other view"));
    first.release().expect("released");
    assert!(!first_directory.join(RESERVATION_REQUEST_FILENAME).exists());
    assert!(!second.held_by_another().expect("free"));
    remove(root.as_path());
}

/// An inherited landing lease neither requests nor waits: `reserve` returns false and writes nothing.
#[test]
fn a_nested_landing_neither_requests_nor_waits() {
    let (root, registry) = registry_of("replay-reservation-nested");
    let directory: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    let mut nested: LandingReservation = LandingReservation::open(registry.as_path(), directory.as_path(), FIRST, true);
    assert!(!nested.reserve(1).expect("not applicable"));
    assert!(!directory.join(RESERVATION_REQUEST_FILENAME).exists());
    assert!(!nested.held_by_another().expect("nested never waits"));
    nested.await_slot().expect("returns at once");
    remove(root.as_path());
}

/// The reservation is taken on the lost race the configuration names:
///  earlier races only count, the granted request carries the count, and the grant happens once.
#[test]
fn the_grant_follows_the_configured_lost_races() {
    let (root, registry) = registry_of("replay-reservation-threshold");
    let directory: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    let configured: u64 = 3;
    let mut reservation: LandingReservation = LandingReservation::open(registry.as_path(), directory.as_path(), FIRST, false);
    let mut granted_at: Option<u64> = None;
    for lost in 1..=configured {
        if reservation_earned(lost, configured) && reservation.reserve(lost).expect("reserve") {
            granted_at = Some(lost);
            break;
        }
    }
    assert_eq!(granted_at, Some(configured));
    let request: Vec<u8> = std::fs::read(directory.join(RESERVATION_REQUEST_FILENAME)).expect("request");
    assert_eq!(request, reservation_request_bytes(configured));
    reservation.release().expect("released");
    remove(root.as_path());
}

/// A landing waits for a released reservation and then takes the landing lock.
#[test]
fn a_landing_takes_the_landing_lock_once_the_reservation_is_free() {
    let (root, registry) = registry_of("replay-reservation-yield");
    let holder_directory: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    let mut holder: LandingReservation = LandingReservation::open(registry.as_path(), holder_directory.as_path(), FIRST, false);
    assert!(holder.reserve(1).expect("granted"));
    holder.release().expect("released");
    let waiter_directory: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-10T00:00:01.000Z", true);
    let waiter: LandingReservation = LandingReservation::open(registry.as_path(), waiter_directory.as_path(), SECOND, false);
    let landing_lock: PathBuf = registry.join("landing.lock");
    let lock: OwnerLock = acquire_reserved_landing_lock(&waiter, || {
        let acquired = try_acquire_owner_lock(landing_lock.as_path(), Some(SECOND))
            .map_err(ReservationError::Lock)?;
        return acquired.ok_or_else(|| { return ReservationError::Registry(RecoveryError(String::from("landing lock busy"))); });
    })
    .expect("landing lock after the reservation is free");
    lock.release().expect("released landing lock");
    remove(root.as_path());
}

/// Releasing a reservation whose request file is already gone is not an error.
#[test]
fn releasing_without_a_request_file_is_quiet() {
    let (root, registry) = registry_of("replay-reservation-release-absent");
    let directory: PathBuf = transaction(&registry, FIRST, "2026-10-10T00:00:00.000Z", true);
    LandingReservation::open(&registry, &directory, FIRST, false)
        .release()
        .expect("an absent request is not an error");
    remove(root.as_path());
}

/// A free reservation is not granted to a transaction that is not the oldest live requester.
#[test]
fn a_free_reservation_is_not_granted_to_a_younger_requester() {
    let (root, registry) = registry_of("replay-reservation-younger");
    let older: PathBuf = transaction(&registry, FIRST, "2026-10-10T00:00:00.000Z", true);
    std::fs::write(older.join(RESERVATION_REQUEST_FILENAME), reservation_request_bytes(1)).expect("older request");
    let younger: PathBuf = transaction(&registry, SECOND, "2026-10-10T00:00:01.000Z", true);
    // The younger request waits for the older one for a fixed limit, and the test joins it before the fixture goes:
    // nothing writes into the fixture while it is removed, and a grant would end the wait early with `Ok(true)`.
    let registry_for_thread: PathBuf = registry.clone();
    let younger_for_thread: PathBuf = younger.clone();
    let (outcome, waiting) = finishes_in_time(move || {
        let mut waiting: LandingReservation =
            LandingReservation::open(&registry_for_thread, &younger_for_thread, SECOND, false);
        let outcome: Result<bool, ReservationError> =
            waiting.reserve_noticing(1, Duration::from_millis(300), &mut Vec::<u8>::new());
        return (outcome, waiting);
    });
    assert!(matches!(outcome, Err(ReservationError::WaitTimedOut { holder_pid: None, .. })), "{outcome:?}");
    assert!(!registry.join(RESERVATION_LOCK_NAME).exists());
    waiting.release().expect("the younger request is withdrawn");
    remove(root.as_path());
}

/// Run `work` on its own thread and fail the test when it has not finished after a ceiling,
///  so a wait that lost its limit fails instead of hanging the run.
fn finishes_in_time<Output: Send + 'static>(work: impl FnOnce() -> Output + Send + 'static) -> Output {
    let (sender, receiver) = std::sync::mpsc::channel::<Output>();
    std::thread::spawn(move || {
        let _sent = sender.send(work());
    });
    return receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("the wait did not end at its limit");
}

/// A live reservation that is never released ends another transaction's wait at the limit,
///  naming the holder and leaving the reservation in place.
#[test]
fn a_reservation_held_past_the_limit_ends_the_wait() {
    let (root, registry) = registry_of("replay-reservation-await-timeout");
    let first_directory: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    let second_directory: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-10T00:00:01.000Z", true);
    let mut holder: LandingReservation =
        LandingReservation::open(registry.as_path(), first_directory.as_path(), FIRST, false);
    assert!(holder.reserve(1).expect("granted"));
    let waiter: LandingReservation =
        LandingReservation::open(registry.as_path(), second_directory.as_path(), SECOND, false);
    let started: Instant = Instant::now();
    let failure: ReservationError = finishes_in_time(move || {
        return waiter.await_slot_for(Duration::from_millis(200)).expect_err("held past the limit");
    });
    assert!(started.elapsed() >= Duration::from_millis(200), "{:?}", started.elapsed());
    match failure {
        ReservationError::WaitTimedOut { waited, holder_pid } => {
            assert!(waited >= Duration::from_millis(200), "{waited:?}");
            assert_eq!(holder_pid, Some(i64::from(std::process::id())));
        }
        other => panic!("expected a refused wait, got {other}"),
    }
    assert!(holder.held_by_another().is_ok());
    holder.release().expect("released");
    remove(root.as_path());
}

/// A short cadence for the controls: a first notice after 60 ms and a repeat every 60 ms.
const QUICK: Cadence = Cadence {
    grace: Duration::from_millis(60),
    interval: Duration::from_millis(60),
};

/// A transaction waiting behind a reservation holder prints a notice that names the holder and repeats it every interval.
///  The third notice makes the holder let go, so the wait ends on that event and exactly three notices are out,
///  however slowly the host runs the waiting thread.
#[test]
fn a_wait_behind_a_reservation_holder_names_it_and_repeats() {
    let (root, registry) = registry_of("replay-reservation-notice-holder");
    let first_directory: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    let second_directory: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-10T00:00:01.000Z", true);
    let mut holder: LandingReservation =
        LandingReservation::open(registry.as_path(), first_directory.as_path(), FIRST, false);
    assert!(holder.reserve(1).expect("granted"));
    let waiter: LandingReservation =
        LandingReservation::open(registry.as_path(), second_directory.as_path(), SECOND, false).with_cadence(QUICK);
    let shared: SharedSink = SharedSink::default();
    let mut sink: ActionSink = ActionSink::new(&shared, 3, move || { holder.release().expect("the holder lets go"); });
    finishes_in_time(move || {
        waiter
            .await_slot_noticing(Duration::from_secs(30), &mut sink)
            .expect("the wait ends when the holder lets go");
    });
    let text: String = shared.text();
    let holder_text: String = format!("still waiting for process {} to release the landing reservation", std::process::id());
    assert_eq!(text.matches(holder_text.as_str()).count(), 3, "{text}");
    assert_eq!(text.lines().count(), 3, "{text}");
    assert!(text.contains("giving up after 30 seconds"), "{text}");
    assert_eq!(text.matches("Hint:").count(), 1, "{text}");
    remove(root.as_path());
}

/// A transaction that waits for the grant behind an older request is told its number in line,
///  and the notice says whose turn comes first.
///  The third notice withdraws the older request, so the wait ends on that event with the grant and exactly three
///  notices are out.
#[test]
fn a_queued_requester_is_told_its_place_in_line() {
    let (root, registry) = registry_of("replay-reservation-notice-place");
    let older: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    let older_request: PathBuf = older.join(RESERVATION_REQUEST_FILENAME);
    std::fs::write(&older_request, reservation_request_bytes(1)).expect("older request");
    let younger: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-10T00:00:01.000Z", true);
    let shared: SharedSink = SharedSink::default();
    let mut sink: ActionSink = ActionSink::new(&shared, 3, move || {
        std::fs::remove_file(&older_request).expect("the older request is withdrawn");
    });
    let (granted, reservation) = finishes_in_time(move || {
        let mut reservation: LandingReservation =
            LandingReservation::open(registry.as_path(), younger.as_path(), SECOND, false).with_cadence(QUICK);
        let granted: bool = reservation
            .reserve_noticing(2, Duration::from_secs(30), &mut sink)
            .expect("the grant follows the older request");
        return (granted, reservation);
    });
    assert!(granted);
    let text: String = shared.text();
    assert_eq!(
        text.matches("This is number 2 of 2 commits in line, so 1 will be served first.").count(),
        3,
        "{text}"
    );
    assert_eq!(text.matches("still waiting for older commits to take the landing reservation").count(), 3, "{text}");
    reservation.release().expect("released");
    remove(root.as_path());
}

/// A wait that reached its limit was announced first: the notice that is due goes out before the limit is checked,
///  so this holds however slowly the host runs the waiting thread.
#[test]
fn a_refused_wait_behind_a_holder_was_announced_first() {
    let (root, registry) = registry_of("replay-reservation-notice-refused");
    let first_directory: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    let second_directory: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-10T00:00:01.000Z", true);
    let mut holder: LandingReservation =
        LandingReservation::open(registry.as_path(), first_directory.as_path(), FIRST, false);
    assert!(holder.reserve(1).expect("granted"));
    let waiter: LandingReservation =
        LandingReservation::open(registry.as_path(), second_directory.as_path(), SECOND, false).with_cadence(QUICK);
    let (failure, text) = finishes_in_time(move || {
        let mut sink: Vec<u8> = Vec::new();
        let failure: ReservationError = waiter
            .await_slot_noticing(Duration::from_millis(300), &mut sink)
            .expect_err("held past the limit");
        return (failure, String::from_utf8(sink).expect("text"));
    });
    let holder_text: String = format!("still waiting for process {} to release the landing reservation", std::process::id());
    assert!(text.matches(holder_text.as_str()).count() >= 1, "{text}");
    assert!(matches!(failure, ReservationError::WaitTimedOut { holder_pid: Some(_), .. }), "{failure}");
    holder.release().expect("released");
    remove(root.as_path());
}

/// The same for a transaction that waits for the grant behind an older request.
#[test]
fn a_refused_wait_behind_an_older_request_was_announced_first() {
    let (root, registry) = registry_of("replay-reservation-notice-refused-queue");
    let older: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    std::fs::write(older.join(RESERVATION_REQUEST_FILENAME), reservation_request_bytes(1)).expect("older request");
    let younger: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-10T00:00:01.000Z", true);
    let (failure, text, reservation) = finishes_in_time(move || {
        let mut reservation: LandingReservation =
            LandingReservation::open(registry.as_path(), younger.as_path(), SECOND, false).with_cadence(QUICK);
        let mut sink: Vec<u8> = Vec::new();
        let failure: ReservationError = reservation
            .reserve_noticing(2, Duration::from_millis(300), &mut sink)
            .expect_err("never first in line");
        return (failure, String::from_utf8(sink).expect("text"), reservation);
    });
    assert!(text.contains("This is number 2 of 2 commits in line, so 1 will be served first."), "{text}");
    assert!(matches!(failure, ReservationError::WaitTimedOut { holder_pid: None, .. }), "{failure}");
    reservation.release().expect("released");
    remove(root.as_path());
}

/// A wait that ends before the grace prints nothing.
#[test]
fn a_wait_that_ends_within_the_grace_prints_nothing() {
    let (root, registry) = registry_of("replay-reservation-notice-quiet");
    let first_directory: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    let second_directory: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-10T00:00:01.000Z", true);
    let mut holder: LandingReservation =
        LandingReservation::open(registry.as_path(), first_directory.as_path(), FIRST, false);
    assert!(holder.reserve(1).expect("granted"));
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        return holder.release();
    });
    let waiter: LandingReservation = LandingReservation::open(registry.as_path(), second_directory.as_path(), SECOND, false)
        .with_cadence(Cadence { grace: Duration::from_secs(30), interval: Duration::from_secs(30) });
    let mut sink: Vec<u8> = Vec::new();
    waiter.await_slot_noticing(Duration::from_secs(10), &mut sink).expect("released in time");
    releaser.join().expect("releaser").expect("released");
    assert!(sink.is_empty(), "{}", String::from_utf8_lossy(&sink));
    remove(root.as_path());
}

/// A wait is over at its limit exactly, not before it.
#[test]
fn a_wait_is_over_at_its_limit_exactly() {
    let limit: Duration = Duration::from_millis(200);
    assert!(!limit_reached(Duration::from_millis(199), limit));
    assert!(limit_reached(limit, limit));
    assert!(limit_reached(Duration::from_millis(201), limit));
}

/// The default wait holds while another transaction's reservation stands,
///  and ends once it is released.
#[test]
fn the_default_wait_lasts_as_long_as_the_reservation() {
    let (root, registry) = registry_of("replay-reservation-await-default");
    let first_directory: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    let second_directory: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-10T00:00:01.000Z", true);
    let mut holder: LandingReservation =
        LandingReservation::open(registry.as_path(), first_directory.as_path(), FIRST, false);
    assert!(holder.reserve(1).expect("granted"));
    let waiter: LandingReservation =
        LandingReservation::open(registry.as_path(), second_directory.as_path(), SECOND, false);
    let (sender, receiver) = std::sync::mpsc::channel::<Result<(), ReservationError>>();
    std::thread::spawn(move || {
        let _sent = sender.send(waiter.await_slot());
    });
    assert!(
        receiver.recv_timeout(Duration::from_millis(400)).is_err(),
        "the wait ended while the reservation stood"
    );
    holder.release().expect("released");
    receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("the wait did not end after the release")
        .expect("the wait succeeds");
    remove(root.as_path());
}

/// Releasing a reservation that never wrote a request is not an error,
///  and a request that cannot be removed for any other reason is.
#[cfg(unix)]
#[test]
fn a_release_without_a_request_succeeds_and_other_failures_are_reported() {
    let (root, registry) = registry_of("replay-reservation-release-kinds");
    let directory: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    LandingReservation::open(registry.as_path(), directory.as_path(), FIRST, false)
        .release()
        .expect("no request to remove");
    let file: PathBuf = root.join("not-a-directory");
    std::fs::write(&file, b"x").expect("file");
    let failure: ReservationError = LandingReservation::open(registry.as_path(), file.as_path(), FIRST, false)
        .release()
        .expect_err("the request path cannot be inspected");
    assert!(matches!(failure, ReservationError::Write(_)), "{failure}");
    remove(root.as_path());
}

/// An older live request that is never granted ends a younger transaction's wait for the grant at the limit,
///  and the younger transaction's request is withdrawn by its release.
#[test]
fn an_older_request_held_past_the_limit_ends_the_wait_for_the_grant() {
    let (root, registry) = registry_of("replay-reservation-reserve-timeout");
    let older: PathBuf = transaction(registry.as_path(), FIRST, "2026-10-10T00:00:00.000Z", true);
    std::fs::write(older.join(RESERVATION_REQUEST_FILENAME), reservation_request_bytes(1)).expect("older request");
    let younger: PathBuf = transaction(registry.as_path(), SECOND, "2026-10-10T00:00:01.000Z", true);
    let registry_for_thread: PathBuf = registry.clone();
    let younger_for_thread: PathBuf = younger.clone();
    let (failure, reservation) = finishes_in_time(move || {
        let mut reservation: LandingReservation = LandingReservation::open(
            registry_for_thread.as_path(),
            younger_for_thread.as_path(),
            SECOND,
            false,
        );
        let failure: ReservationError = reservation
            .reserve_for(2, Duration::from_millis(200))
            .expect_err("never first in line");
        return (failure, reservation);
    });
    match failure {
        ReservationError::WaitTimedOut { waited, holder_pid } => {
            assert!(waited >= Duration::from_millis(200), "{waited:?}");
            assert_eq!(holder_pid, None);
        }
        other => panic!("expected a refused wait, got {other}"),
    }
    assert!(younger.join(RESERVATION_REQUEST_FILENAME).exists());
    reservation.release().expect("released");
    assert!(!younger.join(RESERVATION_REQUEST_FILENAME).exists());
    assert!(older.join(RESERVATION_REQUEST_FILENAME).exists(), "the older request is untouched");
    remove(root.as_path());
}

/// The refusal names the holder or the older request, the wait and the variable.
#[test]
fn a_refused_wait_is_described_and_other_failures_keep_their_debug_text() {
    assert_eq!(
        ReservationError::WaitTimedOut { waited: Duration::from_millis(61_900), holder_pid: Some(77) }.to_string(),
        "gave up after waiting 61 seconds for the next landing slot because process 77 holds the reservation; \
         nothing was taken from that process. To go on, let it finish and run the command again, \
         check with `ps` whether it is stuck and stop it if so, or raise the limit with \
         CLI_GIT_LOCK_WAIT_SECONDS (seconds)"
    );
    assert!(
        ReservationError::WaitTimedOut { waited: Duration::from_secs(2), holder_pid: None }
            .to_string()
            .contains("because an older live landing request holds the next slot")
    );
    let other: ReservationError = ReservationError::Identity(String::from("probe failed"));
    assert_eq!(other.to_string(), format!("{other:?}"));
}
