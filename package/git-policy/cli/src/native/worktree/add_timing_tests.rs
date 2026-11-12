//! What:
//!  The add timing record's step accounting,
//! its line,
//! and its writer.
//! Why:
//!  A record that counts one stretch twice,
//! loses a stretch,
//! or breaks its line cannot split a slow `git worktree add`;
//!  one that changes what the add does would be a new way to break work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(sumOfSelfTimes(record) + record.otherMs).toBe(record.totalMs);
//! ```

/// The recorder under test.
use super::{ADD_TIMING_FILE, Recorder, begin, begin_step, begin_work, end_step, end_work, finish, git_step_label, note};
use crate::command_test_support::{fixture, remove};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

/// Milliseconds of the controls' clock;
/// all different,
/// so a field read from the wrong step shows.
const OUTER_ENTER_MS: u64 = 10;
const INNER_ENTER_MS: u64 = 30;
const INNER_LEAVE_MS: u64 = 70;
const OUTER_LEAVE_MS: u64 = 100;
const SECOND_ENTER_MS: u64 = 110;
const SECOND_LEAVE_MS: u64 = 125;
const TOTAL_MS: u64 = 200;

/// The recorder's clock reading `millis` after it began.
fn at(millis: u64) -> Duration {
    return Duration::from_millis(millis);
}

/// A recorder with an outer step that holds an inner one,
/// and a later step of the outer's name.
fn nested() -> Recorder {
    let mut recorder: Recorder = Recorder::new(PathBuf::from("/nonexistent"));
    recorder.enter("outer", at(OUTER_ENTER_MS));
    recorder.enter("inner", at(INNER_ENTER_MS));
    recorder.leave(at(INNER_LEAVE_MS));
    recorder.leave(at(OUTER_LEAVE_MS));
    recorder.enter("outer", at(SECOND_ENTER_MS));
    recorder.leave(at(SECOND_LEAVE_MS));
    return recorder;
}

/// The value of a JSON line parsed.
fn parsed(line: &str) -> serde_json::Value {
    return serde_json::from_str(line.trim_end()).expect("the record is one JSON object");
}

/// The milliseconds of the step `name` in a parsed record.
fn step_ms(record: &serde_json::Value, name: &str) -> f64 {
    return record["steps"][name]["ms"].as_f64().expect("the step has milliseconds");
}

/// A step inside another is taken out of the outer one,
/// so the steps are disjoint and add up to at most the total.
#[test]
fn a_step_inside_another_is_not_counted_twice() {
    let record = parsed(nested().line(at(TOTAL_MS), 0).as_str());
    let inner: f64 = (INNER_LEAVE_MS - INNER_ENTER_MS) as f64;
    let outer_first: f64 = (OUTER_LEAVE_MS - OUTER_ENTER_MS) as f64 - inner;
    let outer_second: f64 = (SECOND_LEAVE_MS - SECOND_ENTER_MS) as f64;
    assert_eq!(step_ms(&record, "inner"), inner);
    assert_eq!(step_ms(&record, "outer"), outer_first + outer_second);
}

/// The same name entered twice is one step with two calls.
#[test]
fn a_step_entered_twice_counts_its_calls() {
    let record = parsed(nested().line(at(TOTAL_MS), 0).as_str());
    assert_eq!(record["steps"]["outer"]["calls"], 2);
    assert_eq!(record["steps"]["inner"]["calls"], 1);
}

/// What no step covers is reported,
/// never hidden.
#[test]
fn the_steps_and_the_rest_add_up_to_the_total() {
    let record = parsed(nested().line(at(TOTAL_MS), 0).as_str());
    let covered: f64 = step_ms(&record, "outer") + step_ms(&record, "inner");
    let other: f64 = record["otherMs"].as_f64().expect("other milliseconds");
    assert_eq!(covered + other, TOTAL_MS as f64);
    assert_eq!(record["totalMs"].as_f64(), Some(TOTAL_MS as f64));
}

/// The exit code,
/// the schema version and a note reach the line,
/// with a note's quote escaped.
#[test]
fn the_line_carries_the_exit_code_and_notes_escaped() {
    let mut recorder: Recorder = nested();
    recorder.note("route", "snapshot \"quoted\"\nline");
    let text: String = recorder.line(at(TOTAL_MS), 3);
    assert_eq!(text.matches('\n').count(), 1, "one record is one line");
    let record = parsed(text.as_str());
    assert_eq!(record["exitCode"], 3);
    assert_eq!(record["schemaVersion"], 1);
    assert_eq!(record["notes"]["route"], "snapshot \"quoted\"\nline");
}

/// With no recorder begun,
/// a step is its body and nothing else.
#[test]
fn a_step_without_a_recorder_returns_its_body() {
    note("route", "ignored");
    let entered: bool = begin_step("anything");
    end_step(entered);
    assert!(!entered, "no recording took the step");
}

/// A step whose body begins another step nests it through the real clock too.
#[test]
fn steps_nest_through_the_thread_recorder() {
    let root: PathBuf = fixture("add-timing-nest");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("common directory");
    begin(common.clone(), false);
    let outer: bool = begin_step("outer");
    let inner: bool = begin_step("inner");
    end_step(inner);
    end_step(outer);
    finish(0);
    let text: String = std::fs::read_to_string(common.join(ADD_TIMING_FILE)).expect("the record was appended");
    let record = parsed(text.lines().next().expect("one line"));
    assert_eq!(record["steps"]["outer"]["calls"], 1);
    assert_eq!(record["steps"]["inner"]["calls"], 1);
    remove(root.as_path());
}

/// A common directory that does not exist gets no record and no directory.
#[test]
fn a_missing_common_directory_is_not_created() {
    let root: PathBuf = fixture("add-timing-missing");
    let common: PathBuf = root.join("never-made");
    begin(common.clone(), false);
    finish(0);
    assert!(!common.exists());
    remove(root.as_path());
}

/// A second record is a second line of the same file.
#[test]
fn records_append_as_lines() {
    let root: PathBuf = fixture("add-timing-append");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("common directory");
    for _ in 0..2 {
        begin(common.clone(), false);
        finish(0);
    }
    let text: String = std::fs::read_to_string(common.join(ADD_TIMING_FILE)).expect("the records were appended");
    assert_eq!(text.lines().count(), 2);
    remove(root.as_path());
}

/// The subcommand is named past the global options that take a value and the flags that do not.
#[test]
fn a_git_run_is_named_by_its_subcommand() {
    let words = |list: &[&str]| -> Vec<OsString> { return list.iter().map(OsString::from).collect(); };
    assert_eq!(git_step_label(words(&["-C", "status", "--no-optional-locks", "ls-files", "--others"]).as_slice()), "git ls-files");
    assert_eq!(git_step_label(words(&["-c", "core.x=1", "read-tree", "-m"]).as_slice()), "git read-tree");
    assert_eq!(git_step_label(words(&["--no-optional-locks"]).as_slice()), "git");
}

/// Work done on other threads is summed by name beside the steps,
/// outside the sum rule,
/// because threads overlap in time.
#[test]
fn work_on_other_threads_is_summed_by_name() {
    let root: PathBuf = fixture("add-timing-work");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("common directory");
    begin(common.clone(), false);
    std::thread::scope(|scope| {
        for _worker in 0..2 {
            scope.spawn(|| {
                let began = begin_work();
                std::thread::sleep(Duration::from_millis(5));
                end_work("clone", began);
            });
        }
    });
    finish(0);
    let text: String = std::fs::read_to_string(common.join(ADD_TIMING_FILE)).expect("the record was appended");
    let record = parsed(text.lines().next().expect("one line"));
    assert_eq!(record["work"]["clone"]["calls"], 2);
    assert!(record["work"]["clone"]["ms"].as_f64().expect("milliseconds") >= 10.0, "two sleeps of five milliseconds add up");
    assert!(record["steps"].get("clone").is_none(), "work is not a step");
    remove(root.as_path());
}

/// Work with no recording begun is its body,
/// and a later recording does not inherit it.
#[test]
fn work_without_a_recording_is_forgotten() {
    end_work("early", begin_work());
    let root: PathBuf = fixture("add-timing-work-forgotten");
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("common directory");
    begin(common.clone(), false);
    finish(0);
    let text: String = std::fs::read_to_string(common.join(ADD_TIMING_FILE)).expect("the record was appended");
    let record = parsed(text.lines().next().expect("one line"));
    assert!(record["work"].get("early").is_none());
    remove(root.as_path());
}

/// The line carries the quiet gap of each step and tool the add ran,
/// by name.
#[test]
fn the_line_carries_the_quiet_gaps() {
    let mut recorder: Recorder = nested();
    recorder.quiet = vec![(String::from("pnpm install"), Duration::from_millis(1500)), (String::from("git ls-files"), Duration::from_millis(20))];
    let record = parsed(recorder.line(at(TOTAL_MS), 0).as_str());
    assert_eq!(record["quiet"]["pnpm install"].as_f64(), Some(1500.0));
    assert_eq!(record["quiet"]["git ls-files"].as_f64(), Some(20.0));
    assert_eq!(parsed(nested().line(at(TOTAL_MS), 0).as_str())["quiet"], serde_json::json!({}));
}

/// What:
///  An add the ladder does not serve still writes a record,
/// and the record says why it ran plainly.
/// Why:
///   An add the wrapper hands to plain Git is the one a person is most likely to wait through with
/// nothing
///  said,
/// and the record is what explains the wait afterwards;
/// without it the review of release
///  0.6.1-alpha.12 measured a first add of 64.8 seconds that left no trace at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beginOnRoute(common, false, whyPlainAdd(Some(destination))); finish(0); expect(record).toMatch(/"route":"plain"/);
/// ```
#[test]
fn an_add_the_ladder_does_not_serve_records_the_route_it_took() {
    let root: PathBuf = fixture("add-timing-plain-route");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("a common directory");
    let clause: String = String::from("because the folder it goes in, \"/never/made\", does not exist yet");
    super::begin_on_route(common.clone(), false, Some(clause.clone()));
    note("steps", "none of the ladder's");
    finish(0);
    let text: String = std::fs::read_to_string(common.join(ADD_TIMING_FILE)).expect("the record was appended");
    let record: serde_json::Value = parsed(text.lines().next().expect("one line"));
    assert_eq!(record["notes"]["route"], "plain", "the record says the add ran as plain Git: {text}");
    assert_eq!(record["notes"]["whyPlain"], clause.as_str(), "and why it did: {text}");
    remove(root.as_path());
}
