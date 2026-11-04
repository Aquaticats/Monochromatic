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

/// The contained runner returns a job's value however long the job takes, and survives a panic.
#[test]
fn the_runner_waits_for_a_slow_job_and_contains_a_panic() {
    assert_eq!(run_contained(|| return 7_u8), Some(7));
    let pause: Duration = Duration::from_millis(1500);
    let slow: Option<u8> = run_contained(move || {
        std::thread::sleep(pause);
        return 2_u8;
    });
    assert_eq!(slow, Some(2), "a slow job is waited for");
    assert_eq!(run_contained(|| -> u8 { panic!("job failure") }), None);
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
