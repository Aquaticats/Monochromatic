//! Controls for the facts the waiting notice of the scratch `git commit` reports:
//! that every figure reaches the record,
//! that a program seen twice is one entry,
//! and that the list of names is bounded.

use super::*;

/// The facts of a build that was stopped at a ceiling after a program ran below it.
fn stopped_facts() -> ScratchStepFacts {
    let mut facts: ScratchStepFacts = ScratchStepFacts::unarmed(Some(Duration::from_millis(16_332)));
    facts.armed = true;
    facts.stopped_after = Some(Duration::from_millis(16_400));
    facts.notice = Duration::from_millis(7);
    facts.wrote = Duration::from_millis(3);
    facts.looks = 164;
    facts.looks_found = 163;
    facts.below_cpu = Duration::from_millis(210);
    facts.below_unended = 3;
    facts.blind_looks = 2;
    facts.blocked_looks = 61;
    facts.credit("ssh-keygen", Duration::from_millis(250));
    return facts;
}

/// Every figure of one build reaches the record under the names the reader expects,
/// and a figure the build does not have is `null` rather than zero.
#[test]
fn every_figure_of_a_build_reaches_the_record() {
    let kept: ScratchStepFacts = stopped_facts();
    let stated: String = scratch_steps_json(&[kept]);
    assert!(stated.starts_with("[{"), "{stated}");
    assert!(stated.contains("\"ceilingMs\":16332"), "{stated}");
    assert!(stated.contains("\"armed\":true"), "{stated}");
    assert!(stated.contains("\"stoppedAfterMs\":16400"), "{stated}");
    assert!(stated.contains("\"noticeMs\":7"), "{stated}");
    assert!(stated.contains("\"wroteMs\":3"), "{stated}");
    assert!(stated.contains("\"looks\":164") && stated.contains("\"looksFound\":163"), "{stated}");
    assert!(stated.contains("\"below\":[{\"name\":\"ssh-keygen\",\"seenMs\":250}]"), "{stated}");
    assert!(stated.contains("\"belowCpuMs\":210") && stated.contains("\"blindLooks\":2") && stated.contains("\"blockedLooks\":61"), "{stated}");
    assert!(stated.contains("\"belowUnended\":3"), "a program the stop could not end reaches the record: {stated}");
    let unarmed: String = scratch_steps_json(&[ScratchStepFacts::unarmed(None)]);
    assert!(unarmed.contains("\"ceilingMs\":null") && unarmed.contains("\"stoppedAfterMs\":null"), "{unarmed}");
    assert!(unarmed.contains("\"armed\":false") && unarmed.contains("\"below\":[]"), "{unarmed}");
}

/// A program seen at two looks is one entry holding the time of both,
/// and a commit that ran no scratch step states an empty list.
#[test]
fn a_program_seen_twice_is_one_entry() {
    let mut facts: ScratchStepFacts = ScratchStepFacts::unarmed(Some(Duration::from_millis(1)));
    facts.credit("ssh-keygen", Duration::from_millis(250));
    facts.credit("ssh-keygen", Duration::from_millis(250));
    facts.credit("git-lfs", Duration::from_millis(250));
    assert_eq!(facts.below.len(), 2, "{:?}", facts.below);
    assert_eq!(facts.below[0], SeenProgram { name: String::from("ssh-keygen"), seen: Duration::from_millis(500) });
    assert_eq!(scratch_steps_json(&[]), "[]");
}

/// The names below one step stop at the cap,
/// so a record never grows with the work.
#[test]
fn the_names_below_one_step_stop_at_the_cap() {
    let mut facts: ScratchStepFacts = ScratchStepFacts::unarmed(None);
    for index in 0..(MAX_BELOW_NAMES + 4) {
        facts.credit(format!("program{index}").as_str(), Duration::from_millis(250));
    }
    assert_eq!(facts.below.len(), MAX_BELOW_NAMES);
    assert_eq!(facts.below[MAX_BELOW_NAMES - 1].name, format!("program{}", MAX_BELOW_NAMES - 1));
}

/// One record takes every build of the command it describes,
/// in the order the builds ended,
/// and the next command starts from nothing.
#[test]
fn the_record_takes_every_build_and_the_next_starts_empty() {
    let _taken: Vec<ScratchStepFacts> = drain_scratch_steps();
    note_scratch_step(stopped_facts());
    note_scratch_step(ScratchStepFacts::unarmed(None));
    let taken: Vec<ScratchStepFacts> = drain_scratch_steps();
    assert_eq!(taken.len(), 2);
    assert!(taken[0].armed && !taken[1].armed, "{taken:?}");
    assert!(drain_scratch_steps().is_empty());
}
