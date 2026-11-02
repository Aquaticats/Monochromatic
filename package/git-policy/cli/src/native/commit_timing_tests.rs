//! What:
//!  The timing record's line,
//! its sum rule,
//! its escaping,
//! its place and its appending.
//! Why:
//!  A record that loses a phase,
//! counts one stretch twice or breaks its line cannot split a slow commit;
//! one that fails the
//!  command it describes,
//! or creates a directory that was not there,
//! would be a new way to break work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(JSON.parse(timing.line(total, 0)).otherMs).toBe(total - sumOfPhases);
//! ```

/// The record and its writer.
use super::{CommitTiming, ReachTrace, SubmitTrace, TIMING_FILE, TIMING_SCHEMA_VERSION, UNANNOUNCED, whole_milliseconds};
use crate::command_test_support::{fixture, remove};
use crate::discard_audit::AUDIT_FILE;
use crate::service_instance::instance_of;
use crate::service_protocol::RELEASE;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Milliseconds each phase is given in the controls;
/// all different,
/// so a field read from the wrong phase shows.
const PREPARE_MS: u64 = 11;
const BUILD_MS: u64 = 13;
const PROVISION_MS: u64 = 17;
const SUBMIT_MS: u64 = 400;
const REACH_MS: u64 = 19;
const START_WAIT_MS: u64 = 23;
const ANSWER_WAIT_MS: u64 = 29;
const AFTER_LANDING_MS: u64 = 31;
const GATE_MS: u64 = 37;
const HANDOFF_MS: u64 = 41;
/// Time the controls leave to no phase.
const UNCOVERED_MS: u64 = 43;

/// A timing with every phase set,
/// and the sum of those the record counts as disjoint.
fn filled(root: &Path) -> (CommitTiming, Duration) {
    let mut timing: CommitTiming = CommitTiming::start();
    std::fs::create_dir_all(root.join(".git")).expect("git directory");
    timing.place_at(root.join(".git").as_path());
    timing.reference = Some(String::from("refs/heads/main"));
    timing.outcome = "landed";
    timing.attempts = 2;
    timing.prepare = Duration::from_millis(PREPARE_MS);
    timing.build = Duration::from_millis(BUILD_MS);
    timing.provision = Duration::from_millis(PROVISION_MS);
    timing.submit = SubmitTrace {
        total: Duration::from_millis(SUBMIT_MS),
        reach: ReachTrace {
            tries: 5,
            refused: 3,
            start_waited: Duration::from_millis(START_WAIT_MS),
            retires_sent: 1,
            releases_seen: vec![String::from("0.5.7"), String::from(RELEASE)],
            answered_by: Some(String::from(RELEASE)),
            elapsed: Duration::from_millis(REACH_MS),
        },
        answer_wait: Duration::from_millis(ANSWER_WAIT_MS),
        positions: vec![2, 1],
        waiting_notices: 4,
        reconnects: 1,
        sends: 2,
    };
    timing.after_landing = Duration::from_millis(AFTER_LANDING_MS);
    timing.gate = Duration::from_millis(GATE_MS);
    timing.handoff = Duration::from_millis(HANDOFF_MS);
    let covered: Duration = timing.prepare + timing.build + timing.provision + timing.submit.total + timing.after_landing + timing.gate + timing.handoff;
    return (timing, covered);
}

/// The line as JSON.
fn parsed(line: &str) -> serde_json::Value {
    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1, "one line: {line}");
    return serde_json::from_str(line.trim_end()).expect("JSON");
}

/// Every field states what the timing holds.
#[test]
fn the_line_states_each_phase_and_what_reached_the_service() {
    let root: PathBuf = fixture("timing-line");
    let (timing, covered) = filled(root.as_path());
    let total: Duration = covered + Duration::from_millis(UNCOVERED_MS);
    let value: serde_json::Value = parsed(timing.line(total, 3).as_str());
    assert_eq!(value["schemaVersion"], TIMING_SCHEMA_VERSION);
    assert_eq!(value["pid"], std::process::id());
    assert_eq!(value["release"], RELEASE);
    assert_eq!(value["instance"], instance_of(root.join(".git").as_path()));
    assert_eq!(value["gitDir"], root.join(".git").to_string_lossy().to_string());
    assert_eq!(value["branch"], "refs/heads/main");
    assert_eq!(value["outcome"], "landed");
    assert_eq!(value["exitCode"], 3);
    assert_eq!(value["totalMs"], u64::try_from(total.as_millis()).expect("fits"));
    assert_eq!(value["prepareMs"], PREPARE_MS);
    assert_eq!(value["attempts"], 2);
    assert_eq!(value["buildMs"], BUILD_MS);
    assert_eq!(value["provisionMs"], PROVISION_MS);
    assert_eq!(value["submitMs"], SUBMIT_MS);
    assert_eq!(value["sends"], 2);
    assert_eq!(value["reachMs"], REACH_MS);
    assert_eq!(value["connectTries"], 5);
    assert_eq!(value["refusedConnects"], 3);
    assert_eq!(value["startWaitMs"], START_WAIT_MS);
    assert_eq!(value["updateRounds"], 1);
    assert_eq!(value["releasesSeen"], serde_json::Value::from(vec!["0.5.7", RELEASE]));
    assert_eq!(value["answeredByRelease"], RELEASE);
    assert_eq!(value["answerWaitMs"], ANSWER_WAIT_MS);
    assert_eq!(value["positions"], serde_json::Value::from(vec![2_u64, 1]));
    assert_eq!(value["waitingNotices"], 4);
    assert_eq!(value["reconnects"], 1);
    assert_eq!(value["afterLandingMs"], AFTER_LANDING_MS);
    assert_eq!(value["gateMs"], GATE_MS);
    assert_eq!(value["handoffMs"], HANDOFF_MS);
    assert!(value["startMillis"].as_u64().is_some_and(|stamp| { return stamp > 0; }), "{value}");
    remove(root.as_path());
}

/// What no phase covers is stated,
/// so the phases and the remainder add up to the total.
#[test]
fn the_phases_and_the_remainder_add_up_to_the_total() {
    let root: PathBuf = fixture("timing-sum");
    let (timing, covered) = filled(root.as_path());
    let total: Duration = covered + Duration::from_millis(UNCOVERED_MS);
    let value: serde_json::Value = parsed(timing.line(total, 0).as_str());
    assert_eq!(value["otherMs"], UNCOVERED_MS);
    let phases: u64 = ["prepareMs", "buildMs", "provisionMs", "submitMs", "afterLandingMs", "gateMs", "handoffMs", "otherMs"]
        .iter()
        .map(|name| { return value[*name].as_u64().expect("a number"); })
        .sum();
    assert_eq!(phases, value["totalMs"].as_u64().expect("a number"));
    // The inner stretches lie inside their phase, so they never exceed it.
    assert!(value["reachMs"].as_u64().expect("number") + value["answerWaitMs"].as_u64().expect("number") <= value["submitMs"].as_u64().expect("number"));
    // A total shorter than the phases (a clock read between two edges) never goes negative.
    let short: serde_json::Value = parsed(timing.line(covered.saturating_sub(Duration::from_millis(1)), 0).as_str());
    assert_eq!(short["otherMs"], 0);
    remove(root.as_path());
}

/// A branch name with a quote,
/// a backslash or a line break stays one line and reads back as it was.
#[test]
fn a_branch_name_with_delimiters_stays_one_line() {
    let root: PathBuf = fixture("timing-escape");
    let (mut timing, covered) = filled(root.as_path());
    let hostile: String = String::from("refs/heads/a\"b\\c\nd\u{1}e,{\"x\":1}");
    timing.reference = Some(hostile.clone());
    let value: serde_json::Value = parsed(timing.line(covered, 0).as_str());
    assert_eq!(value["branch"], hostile.as_str());
    remove(root.as_path());
}

/// A commit that found no repository has no place,
/// no branch and no release that answered,
/// and the line says null.
#[test]
fn a_commit_without_a_place_or_a_reach_says_null() {
    let timing: CommitTiming = CommitTiming::start();
    let value: serde_json::Value = parsed(timing.line(Duration::ZERO, 2).as_str());
    assert_eq!(value["instance"], serde_json::Value::Null);
    assert_eq!(value["gitDir"], serde_json::Value::Null);
    assert_eq!(value["branch"], serde_json::Value::Null);
    assert_eq!(value["answeredByRelease"], serde_json::Value::Null);
    assert_eq!(value["positions"], serde_json::Value::from(Vec::<u64>::new()));
    assert_eq!(value["releasesSeen"], serde_json::Value::from(Vec::<u64>::new()));
    assert_eq!(value["outcome"], "stopped");
}

/// Records of several commands append in order beside the audit file,
/// and a repository with no common directory gets none.
#[test]
fn records_append_in_order_beside_the_audit_file_and_never_create_a_directory() {
    let root: PathBuf = fixture("timing-append");
    let (mut timing, _) = filled(root.as_path());
    timing.finish(0);
    timing.outcome = "ended";
    timing.finish(1);
    let text: String = std::fs::read_to_string(root.join(".git").join(TIMING_FILE)).expect("timing file");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(parsed(format!("{}\n", lines[0]).as_str())["outcome"], "landed");
    assert_eq!(parsed(format!("{}\n", lines[1]).as_str())["exitCode"], 1);
    assert_ne!(TIMING_FILE, AUDIT_FILE, "the audit file keeps its own lines");
    let missing: PathBuf = root.join("absent");
    let mut stray: CommitTiming = CommitTiming::start();
    stray.place_at(missing.join(".git").as_path());
    stray.finish(0);
    assert!(!missing.exists());
    CommitTiming::start().finish(0);
    remove(root.as_path());
}

/// A linked worktree's record lands in the common directory its `commondir` names,
/// and carries the worktree's own instance.
#[test]
fn a_linked_worktrees_record_goes_to_the_common_directory() {
    let root: PathBuf = fixture("timing-linked");
    let common: PathBuf = root.join("main/.git");
    let own: PathBuf = common.join("worktrees/w");
    std::fs::create_dir_all(&own).expect("worktree git directory");
    std::fs::write(own.join("commondir"), "../..\n").expect("commondir");
    let mut timing: CommitTiming = CommitTiming::start();
    timing.place_at(own.as_path());
    timing.finish(0);
    let text: String = std::fs::read_to_string(common.join(TIMING_FILE)).expect("timing file in the common directory");
    let value: serde_json::Value = parsed(text.as_str());
    assert_eq!(value["instance"], instance_of(own.as_path()));
    assert!(!own.join(TIMING_FILE).exists());
    remove(root.as_path());
}

/// Releases are listed once each,
/// in the order seen,
/// and a greeting with no release is named for what it is.
#[test]
fn releases_are_listed_once_in_the_order_seen() {
    let mut trace: ReachTrace = ReachTrace::default();
    trace.saw_release(Some("0.5.7"));
    trace.saw_release(None);
    trace.saw_release(Some("0.5.7"));
    trace.saw_release(Some(RELEASE));
    assert_eq!(trace.releases_seen, vec![String::from("0.5.7"), String::from(UNANNOUNCED), String::from(RELEASE)]);
}

/// The system clock in whole milliseconds since the Unix epoch, read on its own.
fn epoch_milliseconds() -> u128 {
    return std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("a clock after 1970").as_millis();
}

/// The stamp is the wall clock at the start of the command, so the record can be found among the service's journal lines by
///  time: a reading taken between two readings of the system clock lies between them, in the field and in the line.
#[test]
fn the_stamp_is_the_wall_clock_at_the_start() {
    let before: u128 = epoch_milliseconds();
    let timing: CommitTiming = CommitTiming::start();
    let after: u128 = epoch_milliseconds();
    assert!(before <= timing.start_millis && timing.start_millis <= after, "{before} <= {} <= {after}", timing.start_millis);
    let value: serde_json::Value = parsed(timing.line(Duration::ZERO, 0).as_str());
    assert_eq!(value["startMillis"].as_u64().map(u128::from), Some(timing.start_millis));
}

/// A duration is counted in whole milliseconds: the part of a millisecond is dropped, not rounded up or carried.
#[test]
fn whole_milliseconds_drop_the_part_of_a_millisecond() {
    let kept: u64 = SUBMIT_MS;
    let part: Duration = Duration::from_micros(999);
    assert_eq!(whole_milliseconds(Duration::from_millis(kept) + part), u128::from(kept));
    assert_eq!(whole_milliseconds(part), 0);
}
