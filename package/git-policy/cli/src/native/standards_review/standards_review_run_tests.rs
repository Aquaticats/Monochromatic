//! Controls for the run, the options and the bounded runner, against the loopback mock and a fixed clock.

use super::*;
use crate::standards_review_mock::{Answer, Mock, SILENT_AT_MOST, http_reply};
use crate::standards_review_entry::SHORT_SHA_LENGTH;
use crate::standards_review_input::{CODE_SUFFIXES, SHARED_LOG_RELATIVE, TRACKED_LOG_RELATIVE};
use crate::test_support::{fixture, remove};
use std::cell::Cell;

/// Milliseconds the fixed clock advances between its first and second reading.
const ELAPSED_MS: u64 = 1250;

/// The fixed clock's first reading.
const START_MS: u64 = 1_791_344_041_123;

/// A quiet window short enough for the timeout test.
const SHORT: Duration = Duration::from_millis(400);

/// A clock that reads `START_MS`, then `START_MS + ELAPSED_MS`, and stays there.
fn clock() -> impl Fn() -> u64 {
    let readings: Cell<u64> = Cell::new(0);
    return move || {
        let count: u64 = readings.get();
        readings.set(count + 1);
        return START_MS + if count == 0 { 0 } else { ELAPSED_MS };
    };
}

/// One code name.
fn code_names() -> Vec<String> {
    return vec![format!("src/a{}", CODE_SUFFIXES[0]), String::from("README.md")];
}

/// A reader that returns a small commit with the code file's text.
fn reader(names: &[String]) -> Result<LandedCommit, String> {
    let mut commit: LandedCommit = LandedCommit { sha: String::from("sha"), subject: String::from("s"), patch: String::from("patch\n"), ..LandedCommit::default() };
    commit.contents.insert(names[0].clone(), String::from("code\n"));
    return Ok(commit);
}

/// Options writing to `directory/log.md` at `endpoint`.
fn options(directory: &Path, endpoint: &str, key: Option<&str>) -> ReviewOptions {
    // The log is not made here: the run creates it on its first delivery.
    return ReviewOptions { log_path: directory.join("log.md"), api_key: key.map(String::from), endpoint: String::from(endpoint), quiet_window: SHORT, call_ceiling: Duration::from_secs(3600), trusted_pem: None };
}

/// A full review: the request the mock sees is the built body, and the entry holds the verdict, date and seconds.
#[test]
fn a_review_sends_the_built_input_and_logs_the_verdict() {
    let directory: PathBuf = fixture("run-review");
    let answer: &str = r#"{"answers":[{"choice":"partial","probabilities":[{"value":"yes","probability":0.3},{"value":"partial","probability":0.6},{"value":"no","probability":0.1}]}]}"#;
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", answer)));
    let opts: ReviewOptions = options(directory.as_path(), mock.url.as_str(), Some("k"));
    let names: Vec<String> = code_names();
    let commit: LandedCommit = reader(&names[..1]).expect("commit");
    let expected_body: String = request_body(build_input(&commit, &names[..1]).model_input.as_str());
    let outcome: ReviewOutcome = review_landed(&names, "0123456789abcdefff", "the subject", &|files| return reader(files), &opts, &clock());
    assert!(matches!(&outcome, ReviewOutcome::Reviewed(judged) if judged.top == "partial"), "{outcome:?}");
    assert!(mock.finish().ends_with(expected_body.as_str()));
    let log: String = std::fs::read_to_string(opts.log_path.as_path()).expect("log");
    assert!(log.contains(&format!("  {}\n", format_iso_milliseconds(i64::try_from(START_MS + ELAPSED_MS).expect("fits")))));
    assert!(log.contains(&format!("  {} s\n", seconds_with_tenth(ELAPSED_MS))));
    assert!(log.contains("cost_verdict: partial (yes 0.3 partial 0.6 no 0.1)"));
    assert!(log.contains("  0123456789ab `the subject`\n"));
    remove(directory.as_path());
}

/// No credential, an empty credential and a failing gateway each write an entry that says why, and `unjudged` results.
#[test]
fn failures_are_logged_as_entries() {
    let directory: PathBuf = fixture("run-failures");
    let names: Vec<String> = code_names();
    for (key, sha) in [(None, "aaaaaaaaaaaa"), (Some(""), "bbbbbbbbbbbb")] {
        let opts: ReviewOptions = options(directory.as_path(), "http://127.0.0.1:1/unused", key);
        let outcome: ReviewOutcome = review_landed(&names, sha, "s", &|files| return reader(files), &opts, &clock());
        assert!(matches!(&outcome, ReviewOutcome::Reviewed(judged) if judged.outcome == format!("Luna failed: {NO_KEY_MESSAGE}")));
    }
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("500 Internal Server Error", "boom")));
    let opts: ReviewOptions = options(directory.as_path(), mock.url.as_str(), Some("k"));
    review_landed(&names, "cccccccccccc", "s", &|files| return reader(files), &opts, &clock());
    mock.finish();
    let log: String = std::fs::read_to_string(opts.log_path.as_path()).expect("log");
    assert_eq!(log.matches("Named superlinear cost:\n  unjudged\n").count(), 3);
    assert!(log.contains("Luna failed: HTTP 500 boom"));
    remove(directory.as_path());
}

/// A silent gateway ends the run when it has been silent for the quiet window and still logs the timeout.
#[test]
fn a_silent_gateway_costs_at_most_the_quiet_window() {
    let directory: PathBuf = fixture("run-silent");
    let mock: Mock = Mock::start(Answer::Silent(SILENT_AT_MOST));
    let opts: ReviewOptions = options(directory.as_path(), mock.url.as_str(), Some("k"));
    review_landed(&code_names(), "sha", "s", &|files| return reader(files), &opts, &clock());
    mock.finish();
    assert!(std::fs::read_to_string(opts.log_path.as_path()).expect("log").contains("Luna failed: The operation was aborted due to timeout"));
    remove(directory.as_path());
}

/// Log-only and code-free commits never call the reader; the first writes nothing, the second the skip entry.
#[test]
fn silent_and_skipped_commits_read_nothing() {
    let directory: PathBuf = fixture("run-skip");
    let opts: ReviewOptions = options(directory.as_path(), "http://127.0.0.1:1/unused", Some("k"));
    let forbidden = |_: &[String]| -> Result<LandedCommit, String> { panic!("the reader must not be called") };
    assert_eq!(review_landed(&[], "sha", "s", &forbidden, &opts, &clock()), ReviewOutcome::NoEntry);
    assert_eq!(review_landed(&[String::from(TRACKED_LOG_RELATIVE)], "sha", "s", &forbidden, &opts, &clock()), ReviewOutcome::NoEntry);
    assert!(!opts.log_path.exists(), "no entry creates no log");
    assert_eq!(review_landed(&[String::from("doc/a.md")], "sha", "s", &forbidden, &opts, &clock()), ReviewOutcome::Skipped);
    assert!(std::fs::read_to_string(opts.log_path.as_path()).expect("log").starts_with("- Skipped:\n"));
    remove(directory.as_path());
}

/// A commit that cannot be read leaves no entry, and an unwritable log is reported, not raised.
#[test]
fn read_and_write_failures_are_outcomes() {
    let directory: PathBuf = fixture("run-faults");
    let opts: ReviewOptions = options(directory.as_path(), "http://127.0.0.1:1/unused", None);
    let unread = review_landed(&code_names(), "sha", "s", &|_| return Err(String::from("no such commit")), &opts, &clock());
    assert_eq!(unread, ReviewOutcome::Unread(String::from("no such commit")));
    assert!(!opts.log_path.exists(), "an unread commit creates no log");
    // A file where the log's directory should be: the directory cannot be created, so the log cannot be written.
    std::fs::write(directory.join("not-a-directory"), "").expect("a file in the way");
    let mut blocked: ReviewOptions = opts.clone();
    blocked.log_path = directory.join("not-a-directory").join("log.md");
    assert!(matches!(review_landed(&code_names(), "sha", "s", &|files| return reader(files), &blocked, &clock()), ReviewOutcome::Unwritable(_)));
    remove(directory.as_path());
}

/// The environment gives the live log under the common Git directory by default, honors the override, and an empty first key stays empty.
#[test]
fn options_follow_the_script_environment() {
    let root: &Path = Path::new("/repo");
    let common: &Path = Path::new("/repo/.git");
    let none = |_: &str| -> Option<String> { return None };
    let defaults: ReviewOptions = options_from_environment(common, &none);
    assert_eq!(defaults.log_path, common.join(SHARED_LOG_RELATIVE));
    assert_ne!(defaults.log_path, root.join(TRACKED_LOG_RELATIVE), "the tracked log is never the default");
    assert_eq!((defaults.api_key.clone(), defaults.endpoint.as_str(), defaults.quiet_window), (None, ENDPOINT, SCRIPT_QUIET_WINDOW));
    let set = |name: &str| -> Option<String> {
        if name == LOG_VARIABLE {
            return Some(String::from("/elsewhere.md"));
        }
        if name == KEY_VARIABLES[0] {
            return Some(String::new());
        }
        if name == KEY_VARIABLES[1] {
            return Some(String::from("fallback"));
        }
        return None;
    };
    let given: ReviewOptions = options_from_environment(common, &set);
    assert_eq!(given.log_path, PathBuf::from("/elsewhere.md"));
    assert_eq!(given.api_key.as_deref(), Some(""), "an empty first key does not fall back");
    let only_second = |name: &str| -> Option<String> { return (name == KEY_VARIABLES[1]).then(|| return String::from("fallback")) };
    assert_eq!(options_from_environment(common, &only_second).api_key.as_deref(), Some("fallback"));
    let allow = |name: &str| -> Option<String> { return (name == ALLOW_VARIABLE).then(|| return String::from("1")) };
    assert!(full_review_requested(&allow));
    assert!(!full_review_requested(&none));
}

/// The contained runner returns a job's value however long the job takes up to the ceiling, and survives a panic.
#[test]
fn the_runner_waits_for_a_slow_job_and_contains_a_panic() {
    let roomy: Duration = Duration::from_secs(3600);
    assert_eq!(run_contained(|| return 7_u8, roomy), Contained::Finished(7));
    let pause: Duration = Duration::from_millis(1500);
    let slow: Contained<u8> = run_contained(move || {
        std::thread::sleep(pause);
        return 2_u8;
    }, roomy);
    assert_eq!(slow, Contained::Finished(2), "a slow job is waited for");
    assert_eq!(run_contained(|| -> u8 { panic!("job failure") }, roomy), Contained::Panicked);
}

/// Owner, 2026-11-04, "Silence plus a ceiling (Recommended)": a job that is still running when the ceiling passes is not waited for
/// any longer, though it is working; the runner answers while the job still runs.
#[test]
fn the_runner_stops_waiting_for_a_job_at_its_ceiling() {
    let ceiling: Duration = Duration::from_millis(600);
    let (release, blocked) = mpsc::channel::<()>();
    let began: std::time::Instant = std::time::Instant::now();
    let outcome: Contained<u8> = run_contained(move || {
        let _ = blocked.recv();
        return 1_u8;
    }, ceiling);
    let waited: Duration = began.elapsed();
    drop(release);
    assert_eq!(outcome, Contained::CeilingReached(ceiling));
    assert!(waited >= ceiling && waited < ceiling * 20, "{waited:?}");
}

/// A commit the log already names is not asked about again, and a log that does not exist yet is created by the first entry.
#[test]
fn a_repeated_item_asks_nothing_and_a_missing_log_is_created() {
    let directory: PathBuf = fixture("run-repeat");
    let mock: Mock = Mock::start(Answer::Bytes(http_reply("200 OK", r#"{"answers":[{"choice":"no"}]}"#)));
    let opts: ReviewOptions = options(directory.as_path(), mock.url.as_str(), Some("k"));
    assert!(!opts.log_path.exists());
    let names: Vec<String> = code_names();
    let first = review_landed(&names, "0123456789abcdefff", "s", &|files| return reader(files), &opts, &clock());
    assert!(matches!(first, ReviewOutcome::Reviewed(_)));
    mock.finish();
    let again = review_landed(&names, "0123456789abcdefff", "s", &|_| panic!("a logged commit is not read again"), &opts, &clock());
    assert_eq!(again, ReviewOutcome::AlreadyLogged);
    let text: String = std::fs::read_to_string(opts.log_path.as_path()).expect("the first entry created the log");
    assert_eq!(text.matches(&"0123456789abcdefff"[..SHORT_SHA_LENGTH]).count(), 1, "the repeated item added nothing");
    remove(directory.as_path());
}

/// A job the runner stopped waiting for is outstanding until it ends, and no second job starts behind it (sol-2, B): the second is
/// refused without a thread, and the runner takes work again once the first has ended.
#[test]
fn no_second_job_starts_while_a_job_the_runner_stopped_waiting_for_is_still_running() {
    static OWN: Containment = Containment::new();
    let (release, blocked) = mpsc::channel::<()>();
    let first: Contained<u8> = run_contained_in(&OWN, move || {
        let _ = blocked.recv();
        return 1_u8;
    }, Duration::from_millis(300));
    assert_eq!(first, Contained::CeilingReached(Duration::from_millis(300)));
    assert_eq!(OWN.outstanding(), 1);
    let started: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&started);
    let second: Contained<u8> = run_contained_in(&OWN, move || {
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
        return 2_u8;
    }, Duration::from_secs(60));
    assert_eq!(second, Contained::Busy);
    assert!(!started.load(std::sync::atomic::Ordering::SeqCst), "the refused job never ran");
    drop(release);
    let waiting_since: std::time::Instant = std::time::Instant::now();
    while OWN.outstanding() > 0 {
        assert!(waiting_since.elapsed() < Duration::from_secs(20), "the first job never ended");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(run_contained_in(&OWN, || return 3_u8, Duration::from_secs(60)), Contained::Finished(3));
}

/// A job that ends just as the runner gives up is not left counted as outstanding: whichever side is first, the count returns to
/// zero and the runner takes work.
#[test]
fn a_job_that_ends_as_the_runner_gives_up_leaves_nothing_outstanding() {
    static OWN: Containment = Containment::new();
    for round in 0..200_u64 {
        let ceiling: Duration = Duration::from_micros(round * 20);
        let outcome: Contained<u8> = run_contained_in(&OWN, || return 1_u8, ceiling);
        assert!(matches!(outcome, Contained::Finished(1) | Contained::CeilingReached(_)), "{outcome:?}");
        let waiting_since: std::time::Instant = std::time::Instant::now();
        while OWN.outstanding() > 0 {
            assert!(waiting_since.elapsed() < Duration::from_secs(20), "round {round}: a count was left behind");
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

/// A review that panics after the runner stopped waiting for it is no longer outstanding (sol-2 round 2, P1): the accounting is
/// released by a guard that runs on unwinding, so later reviews are not refused for ever. The control times out, lets the worker
/// panic, sees zero outstanding, and starts a new review.
#[test]
fn a_review_that_panics_after_its_timeout_leaves_nothing_outstanding() {
    static OWN: Containment = Containment::new();
    let (release, blocked) = mpsc::channel::<()>();
    let first: Contained<u8> = run_contained_in(&OWN, move || {
        let _ = blocked.recv();
        panic!("the review panics after the runner stopped waiting");
    }, Duration::from_millis(300));
    assert_eq!(first, Contained::CeilingReached(Duration::from_millis(300)));
    assert_eq!(OWN.outstanding(), 1);
    release.send(()).expect("let the worker go on to its panic");
    let waiting_since: std::time::Instant = std::time::Instant::now();
    while OWN.outstanding() > 0 {
        assert!(waiting_since.elapsed() < Duration::from_secs(20), "the panicked review stayed outstanding");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(run_contained_in(&OWN, || return 5_u8, Duration::from_secs(60)), Contained::Finished(5), "a new review starts");
}

/// Two callers that arrive together are never both admitted (sol-2 round 2, P2): each job counts how many jobs run at once while it
/// does, so the control sees a second admission whichever way the threads are scheduled, and the count never passes one.
#[test]
fn two_callers_that_arrive_together_are_never_admitted_at_once() {
    static OWN: Containment = Containment::new();
    static ACTIVE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    static MOST: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    for round in 0..100 {
        let barrier: std::sync::Arc<std::sync::Barrier> = std::sync::Arc::new(std::sync::Barrier::new(2));
        let callers: Vec<std::thread::JoinHandle<Contained<u8>>> = (0..2)
            .map(|_| {
                let shared = std::sync::Arc::clone(&barrier);
                return std::thread::spawn(move || {
                    shared.wait();
                    return run_contained_in(&OWN, || {
                        let now: usize = ACTIVE.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                        MOST.fetch_max(now, std::sync::atomic::Ordering::SeqCst);
                        std::thread::sleep(Duration::from_millis(15));
                        ACTIVE.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                        return 1_u8;
                    }, Duration::from_secs(60));
                });
            })
            .collect();
        let outcomes: Vec<Contained<u8>> = callers.into_iter().map(|caller| { return caller.join().expect("caller"); }).collect();
        assert!(outcomes.iter().all(|outcome| { return matches!(outcome, Contained::Finished(1) | Contained::Busy); }), "round {round}: {outcomes:?}");
        assert_eq!(MOST.load(std::sync::atomic::Ordering::SeqCst), 1, "round {round}: two jobs ran at once");
    }
}

/// A worker that released the runner and stalls before it sends its value (sol-1 round 3, P1 D): the caller's wait for the value is
/// bounded, so the call returns, as a ceiling, and the next job runs. The control has a deadline of its own around the call.
#[test]
fn a_worker_that_stalls_between_its_release_and_its_send_does_not_hold_the_caller() {
    static OWN: Containment = Containment::with_handoff_wait(Duration::from_millis(300));
    let gate = OWN.handoff_gate.lock().expect("the control holds the handoff");
    let (sender, receiver) = std::sync::mpsc::channel::<Contained<u32>>();
    std::thread::spawn(move || { let _sent = sender.send(run_contained_in(&OWN, || { return 7_u32; }, Duration::from_millis(1))); });
    // The 1 ms ceiling passes while the worker waits at the gate or has not yet started; either way the call must come back.
    let answer: Contained<u32> = receiver.recv_timeout(Duration::from_secs(30)).expect("the caller came back without the value");
    assert!(matches!(answer, Contained::CeilingReached(_) | Contained::Finished(7)), "{answer:?}");
    drop(gate);
    // The runner is free again once the worker is let go, and the next job runs.
    let waiting_since: std::time::Instant = std::time::Instant::now();
    while OWN.outstanding() > 0 || OWN.admit().map(|generation| { return OWN.change(generation, RUNNING, IDLE); }).is_none() {
        assert!(waiting_since.elapsed() < Duration::from_secs(30), "the runner stayed closed");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(run_contained_in(&OWN, || { return 8_u32; }, Duration::from_secs(30)), Contained::Finished(8));
}

/// While a job is admitted and running, no other is: one controlled step, no scheduling luck (sol-1 round 3).
#[test]
fn a_runner_with_a_running_job_admits_no_other() {
    static OWN: Containment = Containment::new();
    let first: u64 = OWN.admit().expect("an idle runner admits");
    assert_eq!(OWN.admit(), None, "a running job closes the runner");
    assert!(OWN.change(first, RUNNING, ABANDONED));
    assert_eq!(OWN.admit(), None, "an abandoned job closes it too");
    assert_eq!(OWN.outstanding(), 1);
    assert!(OWN.change(first, ABANDONED, IDLE));
    assert!(OWN.admit().is_some_and(|second| { return second > first; }), "a released runner admits the next generation");
}
