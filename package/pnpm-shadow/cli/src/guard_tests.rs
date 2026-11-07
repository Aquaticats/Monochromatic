//! The unit tests of the loop guard:
//! the text round trip,
//! rejection of foreign text,
//! and which process a guard is meant for.

/// Imports the functions and types under test.
use crate::guard::{Guard, GuardKind, format_guard, is_meant_for, parse_guard};

/// Imports the owned path type.
use std::path::PathBuf;

/// A process id chosen apart from the ones a test asks about,
/// so equality is never an accident.
const WRITER_PID: u32 = 4242;

fn sample(kind: GuardKind) -> Guard {
    return Guard {
        kind,
        pid: WRITER_PID,
        hops: 2,
        skipped: vec![PathBuf::from("/a/launcher"), PathBuf::from("/b/with:colon/pnpm")],
    };
}

#[test]
fn a_guard_survives_the_text_round_trip_including_a_path_with_a_colon() {
    for kind in [GuardKind::Exec, GuardKind::Spawn] {
        let guard: Guard = sample(kind);

        assert_eq!(parse_guard(&format_guard(&guard)), Some(guard));
    }
}

#[test]
fn text_that_is_not_a_guard_is_ignored() {
    let texts: [&str; 6] = ["", "x", "e", "e\nnot-a-number\n1", "e\n7\nnot-a-number", "q\n7\n1\n/a"];
    for text in texts {
        assert_eq!(parse_guard(text), None, "text {text:?}");
    }
}

#[test]
fn an_exec_guard_is_meant_only_for_the_process_with_the_writers_id() {
    let guard: Guard = sample(GuardKind::Exec);

    assert!(is_meant_for(&guard, WRITER_PID, WRITER_PID + 1));
    // The writer's child has another id, whatever its parent is: a script the real pnpm runs must not inherit the guard.
    assert!(!is_meant_for(&guard, WRITER_PID + 1, WRITER_PID));
    assert!(!is_meant_for(&guard, WRITER_PID + 1, WRITER_PID + 2));
}

#[test]
fn a_spawn_guard_is_meant_only_for_the_direct_child_of_the_writer() {
    let guard: Guard = sample(GuardKind::Spawn);

    assert!(is_meant_for(&guard, WRITER_PID + 1, WRITER_PID));
    assert!(!is_meant_for(&guard, WRITER_PID, WRITER_PID + 1));
    assert!(!is_meant_for(&guard, WRITER_PID + 1, WRITER_PID + 2));
}
