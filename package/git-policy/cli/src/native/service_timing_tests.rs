//! What:
//!  The service's log lines: the answered request, the start, the retire request and the exit.
//! Why:
//!  A line that loses a phase, counts a stretch twice, or lets a client's text break the line cannot split a slow commit; the
//!  phases of one request have to add up to the time from acceptance to answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(fields(answeredLine(record)).queued_ms).toBe(String(queued));
//! ```

/// The lines and the phase arithmetic.
use super::{Answered, Arrival, ExitReason, LOG_PREFIX, LandTiming, answered_line, exit_line, retire_line, started_line};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Milliseconds each phase is given in the controls; all different, so a field read from the wrong phase shows.
const READ_MS: u64 = 3;
const QUEUED_MS: u64 = 5;
const SETTLE_MS: u64 = 7;
const LOCK_WAIT_MS: u64 = 1_000;
const OBJECTS_MS: u64 = 13;
const JOURNAL_MS: u64 = 17;
const REF_MS: u64 = 19;
const SYNC_MS: u64 = 23;
/// Time of the landing no named phase covers.
const READS_MS: u64 = 29;
const ANSWER_MS: u64 = 31;

/// The `key=value` words of a line (a quoted value is taken whole), after the leading word.
fn fields(line: &str) -> HashMap<String, String> {
    let mut found: HashMap<String, String> = HashMap::new();
    let mut rest: &str = line.split_once(' ').map_or("", |(_, tail)| { return tail; });
    while !rest.is_empty() {
        let Some((key, after)) = rest.split_once('=') else { break };
        let (value, tail): (String, &str) = if let Some(quoted) = after.strip_prefix('"') {
            let mut end: usize = 0;
            let mut escaped: bool = false;
            for (at, character) in quoted.char_indices() {
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == '"' {
                    end = at;
                    break;
                }
            }
            (quoted[..end].to_string(), quoted[end + 1..].trim_start())
        } else {
            let (word, tail) = after.split_once(' ').unwrap_or((after, ""));
            (word.to_string(), tail)
        };
        found.insert(key.to_string(), value);
        rest = tail;
    }
    return found;
}

/// The landing of the controls.
fn landing() -> LandTiming {
    return LandTiming {
        lock_wait: Duration::from_millis(LOCK_WAIT_MS),
        lock_holder: Some(String::from("Process 4242 (git)")),
        objects: Duration::from_millis(OBJECTS_MS),
        journal: Duration::from_millis(JOURNAL_MS),
        ref_update: Duration::from_millis(REF_MS),
        sync: Duration::from_millis(SYNC_MS),
        attempts: 2,
    };
}

/// The line for the controls' request.
fn line_for(request: &str, reference: &str, landing: &LandTiming) -> String {
    let land_total: Duration = landing.lock_wait + landing.objects + landing.journal + landing.ref_update + landing.sync + Duration::from_millis(READS_MS);
    return answered_line(&Answered {
        request,
        reference,
        outcome: "landed",
        batch: 9,
        size: 3,
        position: 2,
        arrival: Arrival { accepted: Instant::now(), read: Duration::from_millis(READ_MS) },
        queued: Duration::from_millis(QUEUED_MS),
        settle: Duration::from_millis(SETTLE_MS),
        landing,
        land_total,
        answer: Duration::from_millis(ANSWER_MS),
        total: Duration::from_millis(READ_MS + QUEUED_MS + SETTLE_MS) + land_total + Duration::from_millis(ANSWER_MS),
    });
}

/// Every field states what the request met.
#[test]
fn the_answered_line_states_each_phase_of_the_wait() {
    let line: String = line_for("req-1", "refs/heads/main", &landing());
    assert!(line.starts_with("answered "), "{line}");
    assert!(!line.contains('\n'));
    let found: HashMap<String, String> = fields(line.as_str());
    assert_eq!(found["request"], "req-1");
    assert_eq!(found["branch"], "refs/heads/main");
    assert_eq!(found["outcome"], "landed");
    assert_eq!(found["batch"], "9");
    assert_eq!(found["size"], "3");
    assert_eq!(found["position"], "2");
    assert_eq!(found["read_ms"], READ_MS.to_string());
    assert_eq!(found["queued_ms"], QUEUED_MS.to_string());
    assert_eq!(found["settle_ms"], SETTLE_MS.to_string());
    assert_eq!(found["lock_wait_ms"], LOCK_WAIT_MS.to_string());
    assert_eq!(found["lock_holder"], "Process 4242 (git)");
    assert_eq!(found["reads_ms"], READS_MS.to_string());
    assert_eq!(found["objects_ms"], OBJECTS_MS.to_string());
    assert_eq!(found["journal_ms"], JOURNAL_MS.to_string());
    assert_eq!(found["ref_ms"], REF_MS.to_string());
    assert_eq!(found["sync_ms"], SYNC_MS.to_string());
    assert_eq!(found["attempts"], "2");
    assert_eq!(found["answer_ms"], ANSWER_MS.to_string());
}

/// The phases of the landing add up to its total, and the request's phases to the total from acceptance to answer.
#[test]
fn the_phases_add_up_to_the_totals() {
    let found: HashMap<String, String> = fields(line_for("req-2", "refs/heads/main", &landing()).as_str());
    let number = |name: &str| -> u64 { return found[name].parse::<u64>().expect("a number"); };
    let landing_phases: u64 = ["lock_wait_ms", "reads_ms", "objects_ms", "journal_ms", "ref_ms", "sync_ms"].iter().map(|name| { return number(name); }).sum();
    assert_eq!(landing_phases, number("landing_ms"));
    let request_phases: u64 = ["read_ms", "queued_ms", "settle_ms", "landing_ms", "answer_ms"].iter().map(|name| { return number(name); }).sum();
    assert_eq!(request_phases, number("total_ms"));
}

/// A landing that waited for nobody names no holder.
#[test]
fn a_landing_with_no_holder_names_none() {
    let mut unheld: LandTiming = landing();
    unheld.lock_holder = None;
    unheld.lock_wait = Duration::ZERO;
    let line: String = line_for("req-3", "refs/heads/main", &unheld);
    assert!(!line.contains("lock_holder"), "{line}");
    assert_eq!(fields(line.as_str())["lock_wait_ms"], "0");
}

/// Text a client chose cannot end the line or hide a field.
#[test]
fn a_hostile_request_text_stays_one_line_and_reads_back() {
    let hostile: &str = "id\" outcome=forged\nanswered request=\"x";
    let reference: &str = "refs/heads/a\"b\\c\td";
    let line: String = line_for(hostile, reference, &landing());
    assert!(!line.contains('\n'), "{line}");
    assert_eq!(line.matches("outcome=").count(), 2, "the forged word is inside the quoted request: {line}");
    let found: HashMap<String, String> = fields(line.as_str());
    assert_eq!(found["outcome"], "landed");
    assert_eq!(found["request"], format!("{hostile:?}").trim_matches('"'));
}

/// The reads are what the named phases leave of the landing, never negative.
#[test]
fn the_reads_are_the_remainder_of_the_landing() {
    let phases: LandTiming = landing();
    let named: Duration = phases.lock_wait + phases.objects + phases.journal + phases.ref_update + phases.sync;
    assert_eq!(phases.reads(named + Duration::from_millis(READS_MS)), Duration::from_millis(READS_MS));
    assert_eq!(phases.reads(Duration::ZERO), Duration::ZERO);
}

/// The start line names the release, the protocol and the instance; a development run says it serves any.
#[test]
fn the_start_line_names_release_protocol_and_instance() {
    let served: HashMap<String, String> = fields(started_line("0.5.8", 7, Some("abc123")).as_str());
    assert_eq!(served["release"], "0.5.8");
    assert_eq!(served["protocol"], "7");
    assert_eq!(served["pid"], std::process::id().to_string());
    assert_eq!(served["instance"], "abc123");
    assert_eq!(fields(started_line("0.5.8", 7, None).as_str())["instance"], "any");
}

/// The retire line names the release of the service that is asked to go.
#[test]
fn the_retire_line_names_the_release_that_retires() {
    let line: String = retire_line("0.5.7");
    assert!(line.contains("\"0.5.7\""), "{line}");
    assert!(line.starts_with("retire requested"), "{line}");
}

/// Each reason has its word, and the exit line states the uptime.
#[test]
fn the_exit_line_states_reason_and_uptime() {
    let up: Duration = Duration::from_millis(READS_MS * 1_000);
    for (reason, word) in [(ExitReason::Idle, "idle"), (ExitReason::Retire, "retire"), (ExitReason::Error, "error")] {
        let found: HashMap<String, String> = fields(exit_line(reason, up).as_str());
        assert_eq!(found["reason"], word);
        assert_eq!(found["uptime_ms"], up.as_millis().to_string());
    }
    assert!(LOG_PREFIX.ends_with(": "));
}
