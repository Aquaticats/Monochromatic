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
use crate::commit_prepare_part_durations::PreparePart;
use crate::time_spent_outside_the_phases::OutsideThePhases;
use crate::command_test_support::{fixture, remove};
use crate::discard_audit::AUDIT_FILE;
use crate::service_instance::instance_of;
use crate::service_protocol::{LandingFigures, LandingSync, RELEASE};
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
/// The figures a control landing states:
/// how long it held the index,
/// how many commits queued behind it,
/// and the reads and syncs it names.
const LANDING_HOLD_MS: u64 = 47;
const LANDING_QUEUED_BEHIND: u64 = 3;
const LANDING_READS_MS: u64 = 53;
const LANDING_READ_BRANCH_MS: u64 = 2;
const LANDING_READ_WITNESS_MS: u64 = 5;
const LANDING_SYNC_INDEX_MS: u64 = 7;

/// The stretches of the preparation the controls record,
/// two of one name,
/// so a sum and its count both show.
const LAYOUT_MS: u64 = 7;
const CAPTURE_MS: u64 = 11;
/// How many stretches the controls say were dropped.
const PREPARE_PARTS_DROPPED: usize = 3;

/// Time the controls give the two costs outside the phases.
const SCRATCH_REMOVAL_MS: u64 = 47;
const REMOVALS_THAT_DID_NOT_FINISH: u64 = 2;
const STANDARD_ERROR_MS: u64 = 53;

/// The figures of the landing the controls state.
fn landing_figures() -> LandingFigures {
    return LandingFigures {
        lock_hold_ms: LANDING_HOLD_MS,
        queued_behind: LANDING_QUEUED_BEHIND,
        reads_ms: LANDING_READS_MS,
        reads: vec![(String::from("branch"), LANDING_READ_BRANCH_MS), (String::from("witness"), LANDING_READ_WITNESS_MS)],
        syncs: vec![
            LandingSync { what: String::from("built-index"), waited: true, ms: LANDING_SYNC_INDEX_MS },
            LandingSync { what: String::from("intent-file"), waited: false, ms: 0 },
        ],
    };
}

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
        landing: Some(landing_figures()),
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
        holder_kind: Some("proven-alive"),
        reconnects: 1,
        sends: 2,
    };
    timing.after_landing = Duration::from_millis(AFTER_LANDING_MS);
    timing.gate = Duration::from_millis(GATE_MS);
    timing.handoff = Duration::from_millis(HANDOFF_MS);
    timing.prepare_parts = vec![
        PreparePart { name: String::from("layout"), took: Duration::from_millis(LAYOUT_MS) },
        PreparePart { name: String::from("capture"), took: Duration::from_millis(CAPTURE_MS) },
        PreparePart { name: String::from("layout"), took: Duration::from_millis(LAYOUT_MS) },
    ];
    timing.prepare_parts_dropped = PREPARE_PARTS_DROPPED;
    timing.outside = OutsideThePhases {
        scratch_removal: Duration::from_millis(SCRATCH_REMOVAL_MS),
        removals_that_did_not_finish: REMOVALS_THAT_DID_NOT_FINISH,
        standard_error: Duration::from_millis(STANDARD_ERROR_MS),
    };
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
    assert_eq!(value["lockHolderKind"], "proven-alive");
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
    assert_eq!(value["scratchRemovalMs"], SCRATCH_REMOVAL_MS);
    assert_eq!(value["removalsThatDidNotFinish"], REMOVALS_THAT_DID_NOT_FINISH);
    assert_eq!(value["stderrMs"], STANDARD_ERROR_MS);
    assert_eq!(value["prepareParts"]["layout"]["ms"], LAYOUT_MS * 2);
    assert_eq!(value["prepareParts"]["layout"]["calls"], 2);
    assert_eq!(value["prepareParts"]["capture"]["ms"], CAPTURE_MS);
    assert_eq!(value["prepareParts"]["capture"]["calls"], 1);
    assert_eq!(value["preparePartsDropped"], PREPARE_PARTS_DROPPED);
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

/// The system clock in whole milliseconds since the Unix epoch,
/// read on its own.
fn epoch_milliseconds() -> u128 {
    return std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("a clock after 1970").as_millis();
}

/// The stamp is the wall clock at the start of the command,
/// so the record can be found among the service's journal lines by
///  time:
/// a reading taken between two readings of the system clock lies between them,
/// in the field and in the line.
#[test]
fn the_stamp_is_the_wall_clock_at_the_start() {
    let before: u128 = epoch_milliseconds();
    let timing: CommitTiming = CommitTiming::start();
    let after: u128 = epoch_milliseconds();
    assert!(before <= timing.start_millis && timing.start_millis <= after, "{before} <= {} <= {after}", timing.start_millis);
    let value: serde_json::Value = parsed(timing.line(Duration::ZERO, 0).as_str());
    assert_eq!(value["startMillis"].as_u64().map(u128::from), Some(timing.start_millis));
}

/// A duration is counted in whole milliseconds:
/// the part of a millisecond is dropped,
/// not rounded up or carried.
#[test]
fn whole_milliseconds_drop_the_part_of_a_millisecond() {
    let kept: u64 = SUBMIT_MS;
    let part: Duration = Duration::from_micros(999);
    assert_eq!(whole_milliseconds(Duration::from_millis(kept) + part), u128::from(kept));
    assert_eq!(whole_milliseconds(part), 0);
}

/// A waiting notice that names a process proven to hold the lock is `proven-alive`;
/// one that says the owner cannot be proven alive is `unproven`.
/// A record of a commit that did not wait states null.
#[test]
fn the_holder_kind_follows_what_the_notice_says() {
    use super::holder_kind_of;
    use crate::service_index_attempts::COMMIT_WITHDRAWN;
    use crate::wait_ceilings::LANDING_LOCK_CEILING;
    let end: crate::index_lock_evidence::WaitEnd<'_> = crate::index_lock_evidence::WaitEnd { within: LANDING_LOCK_CEILING, ending: COMMIT_WITHDRAWN };
    let proven: String = crate::index_lock_evidence::proven_holder_line(42, true, Some("git"), Path::new("/r/.git/index.lock"), end);
    assert_eq!(holder_kind_of(proven.as_str()), "proven-alive");
    assert_eq!(
        holder_kind_of(crate::index_lock_evidence::unproven_wait_line(Path::new("/r/.git/index.lock"), None, end).as_str()),
        "unproven"
    );
    assert_eq!(
        holder_kind_of(crate::index_lock_evidence::dead_owner_diagnostic(Path::new("/r/.git/index.lock"), 9, "this commit was not made", crate::index_lock_evidence::DEAD_OWNER_LOCK_LEFT).as_str()),
        "unproven",
        "the diagnostic that ends a wait is not a holder proven alive"
    );
    let root: PathBuf = fixture("timing-holder-null");
    let (mut timing, covered) = filled(root.as_path());
    timing.submit.holder_kind = None;
    let value: serde_json::Value = parsed(timing.line(covered, 0).as_str());
    assert!(value["lockHolderKind"].is_null(), "{value}");
    remove(root.as_path());
}

/// The record carries the identity the commit's request carries,
/// the key that joins it to the service's
/// `answered` line (`request=<id>`) and to the journal field `CLI_GIT_REQUEST_ID`;
/// a commit that never made a request says null.
#[test]
fn the_line_carries_the_request_identity_that_joins_it_to_the_service() {
    let root: PathBuf = fixture("timing-request");
    let (mut timing, covered) = filled(root.as_path());
    timing.request = Some(String::from("30cbb78e-677f-41ff-b059-87d9f49d3409"));
    let value: serde_json::Value = parsed(timing.line(covered, 0).as_str());
    assert_eq!(value["requestId"], "30cbb78e-677f-41ff-b059-87d9f49d3409");
    timing.request = None;
    let without: serde_json::Value = parsed(timing.line(covered, 0).as_str());
    assert!(without["requestId"].is_null(), "{without}");
    remove(root.as_path());
}

/// The record states when it was written,
/// in the form every other record of the wrapper uses,
/// and a level so that a
/// command that stopped stands out from one that landed or ended.
#[test]
fn the_line_states_its_end_time_and_a_level() {
    let root: PathBuf = fixture("timing-time-level");
    let (mut timing, covered) = filled(root.as_path());
    let value: serde_json::Value = parsed(timing.line(covered, 0).as_str());
    let time: &str = value["time"].as_str().expect("a time");
    assert_eq!(time.len(), "2026-11-02T19:48:24.367Z".len(), "{time}");
    assert!(time.ends_with('Z') && time.as_bytes()[10] == b'T', "{time}");
    assert_eq!(value["level"], "info");
    timing.outcome = "ended";
    assert_eq!(parsed(timing.line(covered, 1).as_str())["level"], "info");
    timing.outcome = "stopped";
    assert_eq!(parsed(timing.line(covered, 2).as_str())["level"], "warn");
    remove(root.as_path());
}

/// The record states the longest quiet gap of the steps that ended by themselves,
/// and every step's gap by name,
/// with a step that
///  was ended for being quiet listed apart and left out of the longest.
#[test]
fn the_record_carries_the_quiet_gaps_of_the_steps() {
    let root: PathBuf = fixture("timing-quiet");
    let (mut timing, covered) = filled(root.as_path());
    timing.quiet = vec![
        (String::from("git add"), Duration::from_millis(250)),
        (String::from("git rev-parse"), Duration::from_millis(40)),
        (format!("git reset{}", crate::quiet_ledger::ENDED_SUFFIX), Duration::from_millis(9000)),
    ];
    let value: serde_json::Value = parsed(timing.line(covered, 0).as_str());
    assert_eq!(value["longestQuietMs"].as_f64(), Some(250.0), "only steps that ended by themselves count: {value}");
    assert_eq!(value["quietMs"]["git rev-parse"].as_f64(), Some(40.0));
    assert_eq!(value["quietMs"]["git reset [ended for being quiet]"].as_f64(), Some(9000.0));
    let empty: serde_json::Value = parsed(filled(root.as_path()).0.line(covered, 0).as_str());
    assert_eq!(empty["quietMs"], serde_json::json!({}));
    assert_eq!(empty["longestQuietMs"].as_f64(), Some(0.0));
    remove(root.as_path());
}

/// The record states how each build of the private index was made and whether it kept the real index's cached stat data,
/// in the order built,
/// so the build of a slow first preparation is told apart from a rebuild after the branch moved,
/// and both are in the record without verbose mode.
#[test]
fn the_record_states_every_index_build_with_whether_it_kept_the_stat_data() {
    use crate::commit_index_builds::{IndexBuildKind, note_index_build};
    let root: PathBuf = fixture("timing-index-builds");
    let (timing, _covered) = filled(root.as_path());
    note_index_build(IndexBuildKind::First, true);
    note_index_build(IndexBuildKind::Retry, false);
    timing.finish(0);
    let text: String = std::fs::read_to_string(root.join(".git").join(TIMING_FILE)).expect("timing file");
    let value: serde_json::Value = parsed(text.as_str());
    let expected: serde_json::Value = serde_json::from_str(r#"[{"kind":"first","statDataKept":true},{"kind":"retry","statDataKept":false}]"#).expect("the list of two builds");
    assert_eq!(value["indexBuilds"], expected, "every build of the private index is stated in the order built: {value}");
    remove(root.as_path());
}

/// A proof is never undone by a later notice,
/// and a notice that proves an earlier unproven wait counts.
#[test]
fn a_holder_proven_alive_stays_proven_after_any_later_notice() {
    use super::holder_kind_after;
    use crate::service_index_attempts::COMMIT_WITHDRAWN;
    use crate::wait_ceilings::LANDING_LOCK_CEILING;
    let end: crate::index_lock_evidence::WaitEnd<'_> = crate::index_lock_evidence::WaitEnd { within: LANDING_LOCK_CEILING, ending: COMMIT_WITHDRAWN };
    let proven: String = crate::index_lock_evidence::proven_holder_line(7, true, None, Path::new("/r/.git/index.lock"), end);
    let unproven: String = crate::index_lock_evidence::unproven_wait_line(Path::new("/r/.git/index.lock"), None, end);
    assert_eq!(holder_kind_after(None, proven.as_str()), "proven-alive");
    assert_eq!(holder_kind_after(Some("proven-alive"), unproven.as_str()), "proven-alive");
    assert_eq!(holder_kind_after(Some("unproven"), proven.as_str()), "proven-alive");
    assert_eq!(holder_kind_after(Some("unproven"), unproven.as_str()), "unproven");
    assert_eq!(holder_kind_after(None, unproven.as_str()), "unproven");
}

/// What:
///  The record states what the landing of this commit spent and held:
/// the reads it names,
/// the time it held what others queued on,
/// how many commits queued behind it,
/// and each of its syncs with its time.
/// Why:
///   The landing runs in the resident service,
/// so the record of the commit said nothing about it,
/// and a landing that held the index for a second
///  under a queue of others could only be found in a second file.
#[test]
fn the_record_states_what_the_landing_spent_and_held() {
    let root: PathBuf = fixture("commit-timing-landing");
    let (timing, total) = filled(root.as_path());
    let record: serde_json::Value = parsed(timing.line(total, 0).as_str());
    let landing: &serde_json::Value = &record["landing"];
    assert_eq!(landing["lockHoldMs"], LANDING_HOLD_MS, "{record}");
    assert_eq!(landing["queuedBehind"], LANDING_QUEUED_BEHIND, "{record}");
    assert_eq!(landing["readsMs"], LANDING_READS_MS, "{record}");
    assert_eq!(landing["reads"]["branch"], LANDING_READ_BRANCH_MS, "{record}");
    assert_eq!(landing["reads"]["witness"], LANDING_READ_WITNESS_MS, "{record}");
    let syncs: &Vec<serde_json::Value> = landing["syncs"].as_array().expect("the syncs");
    assert_eq!(syncs.len(), 2, "{record}");
    assert_eq!(syncs[0]["what"], "built-index");
    assert_eq!(syncs[0]["waited"], true);
    assert_eq!(syncs[0]["ms"], LANDING_SYNC_INDEX_MS);
    assert_eq!(syncs[1]["what"], "intent-file");
    assert_eq!(syncs[1]["waited"], false);
    remove(root.as_path());
}

/// A commit whose service sent no figures says so in the record,
/// so a reader never takes a zero for a figure nobody measured.
#[test]
fn a_record_of_a_service_without_figures_states_none() {
    let root: PathBuf = fixture("commit-timing-no-landing");
    let (mut timing, total) = filled(root.as_path());
    timing.submit.landing = None;
    let record: serde_json::Value = parsed(timing.line(total, 0).as_str());
    assert!(record["landing"].is_null(), "{record}");
    remove(root.as_path());
}
