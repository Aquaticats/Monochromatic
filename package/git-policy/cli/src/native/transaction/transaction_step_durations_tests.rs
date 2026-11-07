//! Controls for the ledger of step durations: the bound and its count of dropped steps, the order by start, the names that carry
//!  nothing but a program and a subcommand, offsets from an origin, and the two forms a record writes.

use super::*;

/// The step times of a control, in whole milliseconds.
const SHORT: Duration = Duration::from_millis(5);

/// Empty the process-wide ledger, which other controls of the same process may have filled.
fn clean() {
    let _taken = drain_steps(Instant::now());
}

/// Steps come back ordered by when they began, with offsets from the origin and whole-millisecond durations, and a second drain
///  finds nothing.
#[test]
fn steps_are_ordered_by_start_and_measured_from_the_origin() {
    clean();
    let origin: Instant = Instant::now();
    let later: Instant = origin + Duration::from_millis(40);
    let earlier: Instant = origin + Duration::from_millis(10);
    // The later one ended first, as steps of two threads may.
    note_step("git commit", later, Duration::from_millis(7));
    note_step("git add", earlier, SHORT);
    let (figures, dropped): (Vec<StepFigure>, usize) = drain_steps(origin);
    assert_eq!(dropped, 0);
    assert_eq!(
        figures,
        vec![
            StepFigure { name: String::from("git add"), offset_ms: 10, duration_ms: 5 },
            StepFigure { name: String::from("git commit"), offset_ms: 40, duration_ms: 7 },
        ]
    );
    assert!(drain_steps(origin).0.is_empty());
}

/// A step that began before the origin starts at zero.
#[test]
fn a_step_that_began_before_the_origin_starts_at_zero() {
    clean();
    let early: Instant = Instant::now();
    let origin: Instant = early + Duration::from_millis(50);
    note_step("git rev-parse", early, SHORT);
    let (figures, _dropped): (Vec<StepFigure>, usize) = drain_steps(origin);
    assert_eq!(figures[0].offset_ms, 0);
}

/// The list holds `MAX_STEP_RUNS` steps and counts the rest, so a record never grows with the work and never hides that it was cut.
#[test]
fn the_list_is_bounded_and_counts_what_did_not_fit() {
    clean();
    let origin: Instant = Instant::now();
    let extra: usize = 3;
    for _ in 0..(MAX_STEP_RUNS + extra) {
        note_step("git rev-parse", origin, SHORT);
    }
    let (figures, dropped): (Vec<StepFigure>, usize) = drain_steps(origin);
    assert_eq!(figures.len(), MAX_STEP_RUNS);
    assert_eq!(dropped, extra);
    // The count starts again with the next record.
    assert_eq!(drain_steps(origin).1, 0);
}

/// A name carries a program and a subcommand and nothing else: a path, an option value or a separator is replaced.
#[test]
fn a_name_never_carries_a_path_or_a_separator() {
    clean();
    let origin: Instant = Instant::now();
    note_step("git /home/person/secret;rm=1+2@3\n\"", origin, SHORT);
    let (figures, _dropped): (Vec<StepFigure>, usize) = drain_steps(origin);
    let name: &str = figures[0].name.as_str();
    assert!(name.chars().all(|character| { return character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_' | '?'); }), "{name}");
    assert!(!name.contains('/') && !name.contains(';') && !name.contains('@') && !name.contains('+'), "{name}");
}

/// The JSON and the log word of the same steps.
#[test]
fn the_record_forms_name_every_step() {
    let figures: Vec<StepFigure> = vec![
        StepFigure { name: String::from("git add"), offset_ms: 1, duration_ms: 2 },
        StepFigure { name: String::from("git commit"), offset_ms: 3, duration_ms: 1004 },
    ];
    assert_eq!(
        steps_json(figures.as_slice()),
        "[{\"name\":\"git add\",\"offsetMs\":1,\"durationMs\":2},{\"name\":\"git commit\",\"offsetMs\":3,\"durationMs\":1004}]"
    );
    assert_eq!(steps_word(figures.as_slice()), "git add@1+2;git commit@3+1004");
    assert_eq!(steps_json(&[]), "[]");
    assert_eq!(steps_word(&[]), "");
}
