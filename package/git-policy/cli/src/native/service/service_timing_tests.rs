//! What:
//!  The service's log lines:
//! the answered request,
//! the start,
//! the retire request and the exit.
//! Why:
//!  A line that loses a phase,
//! counts a stretch twice,
//! or lets a client's text break the line cannot split a slow commit;
//! the
//!  phases of one request have to add up to the time from acceptance to answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(fields(answeredLine(record)).queued_ms).toBe(String(queued));
//! ```

/// The lines and the phase arithmetic.
use super::{
    Answered, Arrival, ExitReason, LOG_PREFIX, LandTiming, SLOW_LOCK_STEP, WaitNotes, answered_line, exit_line, in_the_repository_at, index_update_ms_of, retire_line,
    stamped_line, started_line, wait_line,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Milliseconds each phase is given in the controls;
/// all different,
/// so a field read from the wrong phase shows.
const READ_MS: u64 = 3;
const QUEUED_MS: u64 = 5;
const SETTLE_MS: u64 = 7;
const LOCK_WAIT_MS: u64 = 1_000;
const OBJECTS_MS: u64 = 13;
const JOURNAL_MS: u64 = 17;
const REF_MS: u64 = 19;
const INDEX_UPDATE_MS: u64 = 23;
/// How long the index lock was held in the controls' landing:
/// the branch move and the index build,
/// so within their sum.
const HOLD_MS: u64 = 31;
/// Time of the landing no named phase covers,
/// more than the named reads of `landing_with_named_reads` add up to:
/// the named reads are parts of the residual and never more than it.
const READS_MS: u64 = 137;
const ANSWER_MS: u64 = 31;
/// The flush of the marker and the wait for the holder,
/// which the controls give other numbers than the phases.
const STAGE_MS: u64 = 37;
const PLACE_MS: u64 = 953;

/// How long the readying of the acquisition took in the controls:
/// the watch on the lock's directory and the marker's text,
/// which the answered line states apart from the wait itself.
const SETUP_MS: u64 = 11;

/// How long removing the staged file took once the wait was decided in the controls.
const AFTER_MS: u64 = 5;

/// How many other commits were waiting for this one's landing to finish in the controls.
const QUEUED_BEHIND: usize = 4;

/// The worktree root the controls' batches land in.
/// Why:
///  One journal covers every worktree of the machine,
///  so the line about a batch names the worktree it landed in.
const REPOSITORY: &str = "/home/a person/repo";

/// The `key=value` words of a line (a quoted value is taken whole),
/// after the leading word.
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
        index_update: Duration::from_millis(INDEX_UPDATE_MS),
        hold: Duration::from_millis(HOLD_MS),
        attempts: 2,
        reads_split: super::ReadSplit::default(),
        syncs: Vec::new(),
        lock_notes: WaitNotes {
            holder: Some(String::from("Process 4242 (git)")),
            found: Some(String::from("foreign no-marker size=0 age_ms=40 open_by=[4242 (git)]")),
            last: Some(String::from("service seq=7 pid=77 state=running age_ms=900")),
            looks: 4,
            staging: Duration::from_millis(STAGE_MS),
            stage_kind: "cold",
            setup: Duration::from_millis(SETUP_MS),
            placing: Duration::from_millis(PLACE_MS),
            after: Duration::from_millis(AFTER_MS),
            polled: false,
        },
    };
}

/// The line for the controls' request.
fn line_for(request: &str, reference: &str, landing: &LandTiming) -> String {
    let land_total: Duration = landing.lock_wait + landing.objects + landing.journal + landing.ref_update + landing.index_update + Duration::from_millis(READS_MS);
    return answered_line(&Answered {
        request,
        reference,
        outcome: "landed",
        batch: 9,
        repository: Path::new(REPOSITORY),
        size: 3,
        position: 2,
        queued_behind: QUEUED_BEHIND,
        arrival: Arrival { accepted: Instant::now(), read: Duration::from_millis(READ_MS) },
        queued: Duration::from_millis(QUEUED_MS),
        settle: Duration::from_millis(SETTLE_MS),
        landing,
        land_total,
        answer: Duration::from_millis(ANSWER_MS),
        total: Duration::from_millis(READ_MS + QUEUED_MS + SETTLE_MS) + land_total + Duration::from_millis(ANSWER_MS),
        quiet: &[],
        steps: &[],
        steps_dropped: 0,
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
    assert_eq!(found["repository"], REPOSITORY, "the repository the batch landed in is named, quoted so a path with a space stays one field: {line}");
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
    assert_eq!(found["index_update_ms"], INDEX_UPDATE_MS.to_string());
    assert_eq!(found["lock_hold_ms"], HOLD_MS.to_string(), "the hold is stated, and is no phase of the landing");
    assert_eq!(found["queued_behind"], QUEUED_BEHIND.to_string(), "how many commits waited for this landing is stated");
    assert_eq!(found["attempts"], "2");
    assert_eq!(found["answer_ms"], ANSWER_MS.to_string());
}

/// The phases of the landing add up to its total,
/// and the request's phases to the total from acceptance to answer.
#[test]
fn the_phases_add_up_to_the_totals() {
    let found: HashMap<String, String> = fields(line_for("req-2", "refs/heads/main", &landing()).as_str());
    let number = |name: &str| -> u64 { return found[name].parse::<u64>().expect("a number"); };
    let landing_phases: u64 = ["lock_wait_ms", "reads_ms", "objects_ms", "journal_ms", "ref_ms", "index_update_ms"].iter().map(|name| { return number(name); }).sum();
    assert_eq!(landing_phases, number("landing_ms"));
    let request_phases: u64 = ["read_ms", "queued_ms", "settle_ms", "landing_ms", "answer_ms"].iter().map(|name| { return number(name); }).sum();
    assert_eq!(request_phases, number("total_ms"));
}

/// Milliseconds each named read is given in the controls,
/// all different and all below `READS_MS`,
/// so a field read from the wrong read shows.
const READS_CONFIG_MS: u64 = 2;
const READS_BRANCH_MS: u64 = 3;
const READS_PARENTS_MS: u64 = 5;
const READS_CHANGED_MS: u64 = 7;
const READS_CONTENT_MS: u64 = 11;
const READS_WITNESS_MS: u64 = 13;
const READS_CONCLUSION_MS: u64 = 17;
const READS_OWED_MS: u64 = 19;
const READS_FOLLOW_MS: u64 = 23;

/// A landing whose residual reads are named one by one.
fn landing_with_named_reads() -> LandTiming {
    let mut split: LandTiming = landing();
    split.reads_split = super::ReadSplit {
        config: Duration::from_millis(READS_CONFIG_MS),
        branch: Duration::from_millis(READS_BRANCH_MS),
        parents: Duration::from_millis(READS_PARENTS_MS),
        changed: Duration::from_millis(READS_CHANGED_MS),
        content: Duration::from_millis(READS_CONTENT_MS),
        witness: Duration::from_millis(READS_WITNESS_MS),
        conclusion: Duration::from_millis(READS_CONCLUSION_MS),
        owed: Duration::from_millis(READS_OWED_MS),
        follow: Duration::from_millis(READS_FOLLOW_MS),
    };
    return split;
}

/// What:
///  The residual reads of a landing are named one by one,
/// so a slow landing says which read spent the time.
/// Why:
///   `reads_ms` alone was one number for the branch reads,
/// the comparisons,
/// the content check,
/// the witness,
/// the conclusion copy,
/// the records the batch owes and the head question under the lock;
/// a landing
///  that took a second there could not say where.
#[test]
fn the_answered_line_names_each_read_of_the_landing() {
    let line: String = line_for("req-reads", "refs/heads/main", &landing_with_named_reads());
    let found: HashMap<String, String> = fields(line.as_str());
    assert_eq!(found["reads_config_ms"], READS_CONFIG_MS.to_string());
    assert_eq!(found["reads_branch_ms"], READS_BRANCH_MS.to_string());
    assert_eq!(found["reads_parents_ms"], READS_PARENTS_MS.to_string());
    assert_eq!(found["reads_changed_ms"], READS_CHANGED_MS.to_string());
    assert_eq!(found["reads_content_ms"], READS_CONTENT_MS.to_string());
    assert_eq!(found["reads_witness_ms"], READS_WITNESS_MS.to_string());
    assert_eq!(found["reads_conclusion_ms"], READS_CONCLUSION_MS.to_string());
    assert_eq!(found["reads_owed_ms"], READS_OWED_MS.to_string());
    assert_eq!(found["reads_follow_ms"], READS_FOLLOW_MS.to_string());
    // The named reads are parts of `reads_ms` and never more than it: the landing's own phases stay the whole.
    let named: u64 = ["reads_config_ms", "reads_branch_ms", "reads_parents_ms", "reads_changed_ms", "reads_content_ms", "reads_witness_ms", "reads_conclusion_ms", "reads_owed_ms", "reads_follow_ms"]
        .iter()
        .map(|name| {
            return found[*name].parse::<u64>().expect("a number");
        })
        .sum();
    assert!(named <= READS_MS, "the named reads add up to no more than the residual: {named} of {READS_MS}");
    assert_eq!(found["reads_ms"], READS_MS.to_string(), "the residual still states its whole");
}

/// Milliseconds each sync of the control's landing is given:
/// two the landing waited for,
/// and two it handed to the background.
const SYNC_BUILT_INDEX_MS: u64 = 41;
const SYNC_LOCK_MARKER_MS: u64 = 43;

/// A landing that made two syncs and handed two to the background.
fn landing_with_syncs() -> LandTiming {
    let mut timed: LandTiming = landing();
    timed.syncs = vec![
        crate::service_durable_syncs::SyncFigure { what: "built-index", waited: true, took: Duration::from_millis(SYNC_BUILT_INDEX_MS) },
        crate::service_durable_syncs::SyncFigure { what: "intent-file", waited: false, took: Duration::ZERO },
        crate::service_durable_syncs::SyncFigure { what: "lock-marker", waited: true, took: Duration::from_millis(SYNC_LOCK_MARKER_MS) },
        crate::service_durable_syncs::SyncFigure { what: "intent-dir", waited: false, took: Duration::ZERO },
    ];
    return timed;
}

/// What:
///  The answered line states every sync the landing made,
///  what each was for and how long it took,
///  and how many it handed to the background.
/// Why:
///   A landing that made more syncs and a landing whose syncs met a stalling device
///  looked alike in the phases;
///  the report of one landing in an fsync test spent 107.9 s,
///  and nothing said how many syncs made it.
#[test]
fn the_answered_line_states_each_sync_of_the_landing_with_its_time() {
    let line: String = line_for("req-syncs", "refs/heads/main", &landing_with_syncs());
    let found: HashMap<String, String> = fields(line.as_str());
    assert_eq!(found["syncs_waited"], "2");
    assert_eq!(found["syncs_queued"], "2");
    assert_eq!(found["sync_held_ms"], (SYNC_BUILT_INDEX_MS + SYNC_LOCK_MARKER_MS).to_string());
    assert_eq!(found["syncs"], "built-index=41;intent-file=queued;lock-marker=43;intent-dir=queued");
}

/// A landing that made no sync says so in numbers and in an empty list,
/// so a reader never has to tell a missing field from a landing that made none.
#[test]
fn a_landing_that_made_no_sync_states_none() {
    let found: HashMap<String, String> = fields(line_for("req-nosyncs", "refs/heads/main", &landing()).as_str());
    assert_eq!(found["syncs_waited"], "0");
    assert_eq!(found["syncs_queued"], "0");
    assert_eq!(found["sync_held_ms"], "0");
    assert_eq!(found["syncs"], "");
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

/// The reads are what the named phases leave of the landing,
/// never negative.
#[test]
fn the_reads_are_the_remainder_of_the_landing() {
    let phases: LandTiming = landing();
    let named: Duration = phases.lock_wait + phases.objects + phases.journal + phases.ref_update + phases.index_update;
    assert_eq!(phases.reads(named + Duration::from_millis(READS_MS)), Duration::from_millis(READS_MS));
    assert_eq!(phases.reads(Duration::ZERO), Duration::ZERO);
}

/// The start line names the release,
/// the protocol and the instance;
/// a development run says it serves any.
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

/// Each reason has its word,
/// and the exit line states the uptime.
#[test]
fn the_exit_line_states_reason_and_uptime() {
    let up: Duration = Duration::from_millis(READS_MS * 1_000);
    for (reason, word) in [(ExitReason::Idle, "idle"), (ExitReason::Retire, "retire"), (ExitReason::Error, "error"), (ExitReason::WorktreeGone, "worktree-gone")] {
        let found: HashMap<String, String> = fields(exit_line(reason, up).as_str());
        assert_eq!(found["reason"], word);
        assert_eq!(found["uptime_ms"], up.as_millis().to_string());
    }
    assert!(LOG_PREFIX.ends_with(": "));
}

/// The answered line says what the wait was for:
/// the readying of the acquisition,
/// the flush of the marker,
/// the wait for the holder and the removal of the staged file apart,
/// how many times the lock was found held,
/// and the first and the last description of it.
/// Why the four times are apart:
///  a wait whose whole is longer than their sum spent its time outside the acquisition's own work,
///  and the gate of 2026-11-11T10:29-05:00 read `lock_wait_ms=95277` against `lock_stage_ms=0` and `lock_place_ms=3003`
///  with nothing to say where the other 92 s went.
#[test]
fn the_answered_line_says_what_the_wait_for_the_lock_was_for() {
    let found: HashMap<String, String> = fields(line_for("req-4", "refs/heads/main", &landing()).as_str());
    assert_eq!(found["lock_stage_ms"], STAGE_MS.to_string());
    assert_eq!(found["lock_stage_kind"], "cold");
    assert_eq!(found["lock_place_ms"], PLACE_MS.to_string());
    assert_eq!(found["lock_setup_ms"], SETUP_MS.to_string());
    assert_eq!(found["lock_after_ms"], AFTER_MS.to_string());
    assert_eq!(found["lock_looks"], "4");
    assert_eq!(found["lock_found"], "foreign no-marker size=0 age_ms=40 open_by=[4242 (git)]");
    assert_eq!(found["lock_last"], "service seq=7 pid=77 state=running age_ms=900");
    let mut free: LandTiming = landing();
    free.lock_notes = WaitNotes { staging: Duration::from_millis(STAGE_MS), stage_kind: "pre", ..WaitNotes::default() };
    let line: String = line_for("req-5", "refs/heads/main", &free);
    assert_eq!(fields(line.as_str())["lock_stage_ms"], STAGE_MS.to_string(), "{line}");
    assert_eq!(fields(line.as_str())["lock_stage_kind"], "pre", "{line}");
    assert_eq!(fields(line.as_str())["lock_setup_ms"], "0", "a quick readying is stated as none: {line}");
    assert!(!line.contains("lock_found") && !line.contains("lock_looks"), "a lock that was free names no holder: {line}");
    assert!(fields(line.as_str()).contains_key("lock_place_ms"), "the placement time is always stated: {line}");
}

/// The line for the end of a wait is written when the lock was found held or a step took long,
/// and names the result;
/// a quick free lock leaves nothing.
#[test]
fn the_wait_line_is_written_for_a_held_lock_or_a_slow_step_and_not_for_a_quick_free_one() {
    let notes: WaitNotes = landing().lock_notes;
    let line: String = wait_line(7, Path::new("repo"), Duration::from_millis(990), &notes, true).expect("a held lock is written down");
    assert!(line.starts_with("waited for the index lock batch=7 repository=\"repo\" result=taken lock_wait_ms=990 "), "{line}");
    assert!(line.contains("lock_found=") && line.contains("lock_last="), "{line}");
    assert!(wait_line(7, Path::new("repo"), Duration::from_millis(990), &notes, false).expect("refused").contains("result=refused"));
    let quick: WaitNotes = WaitNotes { staging: Duration::from_millis(3), ..WaitNotes::default() };
    assert_eq!(wait_line(8, Path::new("repo"), Duration::from_millis(5), &quick, true), None);
    let slow_flush: WaitNotes = WaitNotes { staging: SLOW_LOCK_STEP, ..WaitNotes::default() };
    let slow: String = wait_line(9, Path::new("repo"), SLOW_LOCK_STEP, &slow_flush, true).expect("a slow flush is written down");
    assert!(slow.contains(format!("lock_stage_ms={}", SLOW_LOCK_STEP.as_millis()).as_str()) && !slow.contains("lock_found"), "{slow}");
    // A wait that is slow in the readying and in the removal of the staged file says so without naming a holder,
    // so the two spans are read where the lock was free at the only placement.
    let slow_setup: WaitNotes = WaitNotes { setup: SLOW_LOCK_STEP, after: SLOW_LOCK_STEP, ..WaitNotes::default() };
    let slow_readying: String = wait_line(10, Path::new("repo"), SLOW_LOCK_STEP, &slow_setup, true).expect("a slow readying is written down");
    assert!(
        slow_readying.contains(format!("lock_setup_ms={}", SLOW_LOCK_STEP.as_millis()).as_str())
            && slow_readying.contains(format!("lock_after_ms={}", SLOW_LOCK_STEP.as_millis()).as_str()),
        "{slow_readying}"
    );
}

/// Every line states its own time at the end,
/// in whole milliseconds,
/// so the prefix and the first word stay where readers
/// look for them and the time survives a journal that receives the line late.
#[test]
fn a_log_line_states_its_own_time_at_its_end() {
    let line: String = stamped_line("answered request=\"r\" total_ms=3", 1_793_648_794_987);
    assert_eq!(line, format!("{LOG_PREFIX}answered request=\"r\" total_ms=3 at_ms=1793648794987\n"));
    assert_eq!(line.matches('\n').count(), 1);
}

/// What a push,
/// a review or a server said can name a remote with a password in it;
/// the entry carries the URL without it,
/// in the message and in a field,
/// and the other words are as they were.
#[test]
fn the_userinfo_of_a_url_never_reaches_an_entry() {
    use crate::service_timing::capture::entries_of;
    let secret_url: &str = "https://robot:hunter2@example.invalid/r.git";
    let entries = entries_of(|| {
        crate::service_warn!("the review of abc failed: could not reach {secret_url} (status 401)");
        let _delivery = super::log_entry(crate::journald_native::Priority::Info, "x", &[("CLI_GIT_URL", secret_url.to_string())]);
    });
    assert_eq!(entries.len(), 2);
    for entry in &entries {
        assert!(!entry.message.contains("hunter2") && entry.fields.iter().all(|(_, value)| { return !value.contains("hunter2"); }), "{entry:?}");
    }
    assert!(entries[0].message.starts_with("the review of abc failed: could not reach https://<credentials>@example.invalid/<credentials> (status 401) at_ms="), "{}", entries[0].message);
    assert!(entries[1].fields.contains(&(String::from("CLI_GIT_URL"), String::from("https://<credentials>@example.invalid/<credentials>"))), "{:?}", entries[1].fields);
}

/// The macros write at the priority their name says,
/// and the plain one at `info`.
#[test]
fn the_macros_write_at_the_priority_their_name_says() {
    use crate::journald_native::Priority;
    let entries = crate::service_timing::capture::entries_of(|| {
        crate::service_log!("a");
        crate::service_warn!("b");
        crate::service_error!("c");
    });
    assert_eq!(entries.iter().map(|entry| { return entry.priority; }).collect::<Vec<_>>(), vec![Priority::Info, Priority::Warning, Priority::Error]);
}

/// A message with a newline in it is one entry;
/// a native entry keeps the newline,
/// as written,
/// and the line on standard
/// error is one line,
/// so it cannot start a second line there.
#[test]
fn a_message_with_a_newline_is_one_entry_and_one_line_on_standard_error() {
    let entries = crate::service_timing::capture::entries_of(|| { crate::service_log!("first\nsecond"); });
    assert!(entries[0].message.starts_with("first\nsecond at_ms="), "{}", entries[0].message);
    let line: String = crate::service_timing::stamped_line(crate::journald_native::one_line("first\nsecond", crate::journald_native::MESSAGE_LIMIT).as_str(), 7);
    assert_eq!(line, format!("{}first second at_ms=7\n", crate::service_timing::LOG_PREFIX));
}

/// A cut that falls just before the `@` of a URL's userinfo must not leave the password:
/// the text is masked whole,
/// then cut.
#[test]
fn a_cut_just_before_the_at_sign_leaves_no_password() {
    use crate::journald_native::MESSAGE_LIMIT;
    let url_start: &str = "https://u:TOPSECRET";
    let padding: String = "p".repeat(MESSAGE_LIMIT - url_start.len());
    let entries = crate::service_timing::capture::entries_of(|| { crate::service_log!("{padding}{url_start}@host/ tail"); });
    assert!(!entries[0].message.contains("TOPSECRET"), "{}", &entries[0].message[entries[0].message.len().saturating_sub(80)..]);
}

/// The answered line states the longest quiet gap of the steps that ended by themselves,
/// and each step's gap by name,
/// with a
///  step that was ended for being quiet kept out of the longest;
/// a request that met no step states an empty list.
#[test]
fn the_answered_line_states_the_quiet_gaps_of_the_steps() {
    let landing: LandTiming = LandTiming::default();
    let quiet = vec![
        (String::from("git fast-import"), Duration::from_millis(300)),
        (String::from("git update-ref"), Duration::from_micros(2500)),
        (format!("git reset{}", crate::quiet_ledger::ENDED_SUFFIX), Duration::from_millis(9000)),
    ];
    let line: String = answered_line(&Answered {
        request: "r",
        reference: "refs/heads/main",
        outcome: "landed",
        batch: 1,
        repository: Path::new(REPOSITORY),
        size: 1,
        position: 1,
        queued_behind: 0,
        arrival: Arrival { accepted: Instant::now(), read: Duration::ZERO },
        queued: Duration::ZERO,
        settle: Duration::ZERO,
        landing: &landing,
        land_total: Duration::ZERO,
        answer: Duration::ZERO,
        total: Duration::ZERO,
        quiet: quiet.as_slice(),
        steps: &[],
        steps_dropped: 0,
    });
    assert!(line.contains(" quiet_ms=300.000 "), "{line}");
    assert!(
        line.contains("quiet_steps=\"git fast-import=300.000;git update-ref=2.500;git reset [ended for being quiet]=9000.000\""),
        "{line}"
    );
    assert!(line_for("r", "refs/heads/main", &landing).contains(" quiet_ms=0.000 quiet_steps=\"\" "));
}

/// The answered line names every Git step of the service with its offset and duration,
/// and says how many it dropped.
#[test]
fn the_answered_line_names_each_step_with_its_offset_and_duration() {
    let landing: LandTiming = LandTiming::default();
    let steps: Vec<crate::transaction_step_durations::StepFigure> = vec![
        crate::transaction_step_durations::StepFigure { name: String::from("git rev-parse"), offset_ms: 3, duration_ms: 4 },
        crate::transaction_step_durations::StepFigure { name: String::from("git update-ref"), offset_ms: 9, duration_ms: 2 },
    ];
    let dropped: usize = 5;
    let line: String = answered_line(&Answered {
        request: "r",
        reference: "refs/heads/main",
        outcome: "landed",
        batch: 1,
        repository: Path::new(REPOSITORY),
        size: 1,
        position: 1,
        queued_behind: 0,
        arrival: Arrival { accepted: Instant::now(), read: Duration::ZERO },
        queued: Duration::ZERO,
        settle: Duration::ZERO,
        landing: &landing,
        land_total: Duration::ZERO,
        answer: Duration::ZERO,
        total: Duration::ZERO,
        quiet: &[],
        steps: steps.as_slice(),
        steps_dropped: dropped,
    });
    assert!(line.ends_with(" steps=\"git rev-parse@3+4;git update-ref@9+2\" steps_dropped=5"), "{line}");
    assert_eq!(fields(line.as_str()).get("steps_dropped").map(String::as_str), Some("5"));
}

/// An `answered` line written before the index step was named by its role calls the same duration `sync_ms`:
/// the reader of a
/// line finds it under either name,
/// and a line that holds both (never written) is read by the new name.
#[test]
fn a_record_from_before_the_rename_still_gives_its_index_update() {
    let recorded: &str = "answered request=\"r\" branch=\"refs/heads/main\" outcome=landed batch=1 size=1 position=1 read_ms=0 queued_ms=0 settle_ms=0 \
                          lock_wait_ms=0 lock_hold_ms=21 reads_ms=2 objects_ms=3 journal_ms=1 ref_ms=2 sync_ms=19 attempts=1 landing_ms=27";
    assert_eq!(index_update_ms_of(recorded), Some(19), "the old name is read");
    let written: String = line_for("req-3", "refs/heads/main", &landing());
    assert_eq!(index_update_ms_of(written.as_str()), Some(INDEX_UPDATE_MS), "the new name is read");
    assert!(!written.contains(" sync_ms="), "the new line no longer calls the step a sync: {written}");
    let both: String = format!("{recorded} index_update_ms=5");
    assert_eq!(index_update_ms_of(both.as_str()), Some(5));
    assert_eq!(index_update_ms_of("answered request=\"r\""), None, "a line without the step gives none");
}

/// What:
///  Two repositories whose paths differ only in bytes that are no text are named differently,
///  and a path that is text reads as it is.
/// Why:
///  One journal covers every worktree of the machine,
///  and two worktrees can differ only in bytes that cannot be written as text:
///  the readable spelling of both is the same string with U+FFFD where the byte was,
///  so a label built that way names neither and a line cannot be attributed to its repository
///  (a Kimi K3.1 Pro review of this branch,
///  its finding that distinct non-UTF-8 repository paths produce identical repository labels).
///  `path_message_text` keeps the readable spelling for prose,
///  and the exact bytes travel in the field beside it,
///  so the label of a repository is the one place that has to tell them apart.
#[test]
fn two_repositories_whose_paths_differ_only_in_bytes_that_are_no_text_are_named_differently() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt as _;
    let first: PathBuf = PathBuf::from(OsString::from_vec(b"/repo/worktree-\xFF".to_vec()));
    let second: PathBuf = PathBuf::from(OsString::from_vec(b"/repo/worktree-\xFE".to_vec()));
    assert_ne!(first, second, "the two paths are different");
    let first_label: String = in_the_repository_at(first.as_path());
    let second_label: String = in_the_repository_at(second.as_path());
    assert_ne!(first_label, second_label, "the labels tell the two repositories apart: {first_label} and {second_label}");
    assert!(!first_label.contains('\n'), "the label stays one line: {first_label}");
    assert_eq!(in_the_repository_at(Path::new("/repo/worktree")), "in the repository at /repo/worktree", "a path that is text reads as it is");
}

/// What:
///  A worktree whose path holds a space is named in one field,
///  so the line keeps its shape and the journal has the whole path.
/// Why:
///  Paths with spaces are ordinary,
///  and a field written as `repository=/home/a person/repo` splits the moment anything reads the line by its spaces:
///  the field reads `/home/a` and the rest becomes text nothing owns
///  (found by this branch's own control of the `answered` line,
///  whose fixture root holds a space).
///  The `repository` field is therefore written with Rust's quoted form,
///  as `branch` already is.
#[test]
fn a_repository_whose_path_holds_a_space_is_named_in_one_field() {
    let landing: LandTiming = landing();
    let line: String = answered_line(&Answered {
        request: "r",
        reference: "refs/heads/main",
        outcome: "landed",
        batch: 1,
        repository: Path::new(REPOSITORY),
        size: 1,
        position: 1,
        queued_behind: 0,
        arrival: Arrival { accepted: Instant::now(), read: Duration::ZERO },
        queued: Duration::ZERO,
        settle: Duration::ZERO,
        landing: &landing,
        land_total: Duration::ZERO,
        answer: Duration::ZERO,
        total: Duration::ZERO,
        quiet: &[],
        steps: &[],
        steps_dropped: 0,
    });
    assert_eq!(fields(line.as_str())["repository"], REPOSITORY, "the whole path is one field: {line}");
    assert!(line.contains(&format!("repository={REPOSITORY:?}")), "the field is the path's quoted form: {line}");
    let wait: String = wait_line(7, Path::new(REPOSITORY), Duration::from_millis(990), &landing.lock_notes, true).expect("a held lock is written down");
    assert_eq!(fields(wait.as_str())["repository"], REPOSITORY, "the wait line keeps it whole too: {wait}");
}
