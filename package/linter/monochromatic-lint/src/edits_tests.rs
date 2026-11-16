//! What:
//!  Atomic-fix regression controls.
//! Why:
//!  Flattened edit application can accept half of a finding's fix and corrupt otherwise valid source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('atomic grouped edits', () => { /* boundaries, conflicts and Unicode */ });
//! ```

/// Import the production edit planner and owned input types.
use super::{Edit, Fix, FixClass, FixScope, apply_fixes};

/// Build a fixture replacement while keeping byte ranges explicit at every call.
fn edit(start: usize, end: usize, replacement: &str) -> Edit {
    return Edit {
        start,
        end,
        replacement: String::from(replacement),
    };
}

/// Empty plans preserve bytes,
///  and independent edits apply in source order rather than argument order.
#[test]
fn empty_and_reversed_edit_plans_preserve_source_order() {
    let original = apply_fixes("abc", &[]).expect("no fixes");
    assert_eq!(original.source, "abc");
    assert!(original.applied.is_empty());
    let fixes = [Fix {
        class: FixClass::Fix,
        edits: vec![edit(3, 3, "]"), edit(0, 0, "[")],
    }];
    let snapshot = fixes.clone();
    let applied = apply_fixes("abc", &fixes).expect("two insertions");
    assert_eq!(applied.source, "[abc]");
    assert_eq!(applied.applied, [0]);
    assert_eq!(fixes, snapshot);
}

/// A late conflict rejects the complete multi-edit fix,
///  including its earlier nonconflicting edit.
#[test]
fn conflicting_multi_edit_fix_is_never_applied_partly() {
    let fixes = [
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(2, 4, "X")],
        },
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(0, 1, "A"), edit(3, 5, "Y")],
        },
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(0, 1, "Z")],
        },
    ];
    let applied = apply_fixes("abcdef", &fixes).expect("conflict selection");
    assert_eq!(applied.source, "ZbXef");
    assert_eq!(applied.applied, [0, 2]);
    assert_eq!(applied.rejected, [1]);
}

/// A replacement cannot span a previously selected interval that starts later.
#[test]
fn later_interval_conflict_rejects_the_new_fix() {
    let fixes = [
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(4, 5, "E")],
        },
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(1, 6, "whole")],
        },
    ];
    let applied = apply_fixes("abcdef", &fixes).expect("successor overlap");
    assert_eq!(applied.source, "abcdEf");
    assert_eq!(applied.rejected, [1]);
}

/// Same-position insertions conflict,
///  but insertions at a replacement's exclusive end remain valid.
#[test]
fn insertion_boundaries_have_explicit_ordering() {
    let fixes = [
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(0, 1, "A")],
        },
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(1, 1, "X")],
        },
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(1, 1, "Y")],
        },
    ];
    let applied = apply_fixes("ab", &fixes).expect("boundary insertions");
    assert_eq!(applied.source, "AXb");
    assert_eq!(applied.applied, [0, 1]);
    assert_eq!(applied.rejected, [2]);
}

/// Adjacent edits within one fix and a replacement ending at an earlier insertion are compatible.
#[test]
fn adjacency_is_not_overlap_in_either_selection_order() {
    let adjacent = [Fix {
        class: FixClass::Fix,
        edits: vec![edit(0, 1, "A"), edit(1, 2, "B")],
    }];
    assert_eq!(
        apply_fixes("ab", &adjacent).expect("adjacent edits").source,
        "AB"
    );
    let reverse = [
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(1, 1, "X")],
        },
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(0, 1, "A")],
        },
    ];
    let applied = apply_fixes("ab", &reverse).expect("replacement ending at insertion");
    assert_eq!(applied.source, "AXb");
    assert_eq!(applied.applied, [0, 1]);
    assert!(applied.rejected.is_empty());
}

/// Overlapping edits inside one fix are rejected as an invalid plan.
#[test]
fn internal_overlap_and_duplicate_insertions_are_rejected() {
    for edits in [
        vec![edit(0, 2, "A"), edit(1, 3, "B")],
        vec![edit(1, 1, "A"), edit(1, 1, "B")],
    ] {
        let error = apply_fixes(
            "abc",
            &[Fix {
                class: FixClass::Fix,
                edits,
            }],
        )
        .expect_err("internal conflict");
        assert!(error.message.contains("overlapping"));
    }
}

/// Bounds and UTF-8 boundaries are validated before any slice is formed.
#[test]
fn invalid_ranges_are_errors_not_panics() {
    for invalid in [
        edit(2, 1, "x"),
        edit(0, 9, "x"),
        edit(usize::MAX, usize::MAX, "x"),
    ] {
        let error = apply_fixes(
            "abc",
            &[Fix {
                class: FixClass::Fix,
                edits: vec![invalid],
            }],
        )
        .expect_err("invalid range");
        assert!(error.message.contains("out-of-range"));
    }
    for invalid in [edit(0, 1, "x"), edit(1, 2, "x")] {
        let error = apply_fixes(
            "é",
            &[Fix {
                class: FixClass::Fix,
                edits: vec![invalid],
            }],
        )
        .expect_err("split scalar");
        assert!(error.message.contains("UTF-8-boundary"));
    }
    assert_eq!(
        apply_fixes(
            "éx",
            &[Fix {
                class: FixClass::Fix,
                edits: vec![edit(0, 2, "a")]
            }]
        )
        .expect("whole scalar")
        .source,
        "ax"
    );
}

/// Empty rewriting is refused without changing the caller's original source.
#[test]
fn nonempty_sources_cannot_be_erased() {
    let source = String::from("abc");
    let error = apply_fixes(
        source.as_str(),
        &[Fix {
            class: FixClass::Fix,
            edits: vec![edit(0, 3, "")],
        }],
    )
    .expect_err("empty rewrite");
    assert_eq!(source, "abc");
    assert!(error.message.contains("empty output"));
    assert_eq!(error.to_string(), error.message);
    assert_eq!(apply_fixes("", &[]).expect("already empty").source, "");
    assert_eq!(
        apply_fixes(
            "",
            &[Fix {
                class: FixClass::Fix,
                edits: vec![edit(0, 0, "new")]
            }]
        )
        .expect("insert into empty")
        .source,
        "new"
    );
}

/// A valid no-op group does not fabricate source changes.
#[test]
fn empty_groups_and_identical_replacements_are_stable() {
    let fixes = [
        Fix {
            class: FixClass::Fix,
            edits: Vec::new(),
        },
        Fix {
            class: FixClass::Fix,
            edits: vec![edit(0, 1, "a")],
        },
    ];
    let applied = apply_fixes("abc", &fixes).expect("no-op groups");
    assert_eq!(applied.source, "abc");
    assert!(applied.rejected.is_empty());
}

/// The three flags select classes exactly as Oxlint 1.93.0 does,
///  measured with it and read in `FixOptions::fix_kind` and `FixKind::can_apply`:
///  `--fix` allows the safe fix alone,
///  `--fix-suggestions` the suggestion alone,
///  and `--fix-dangerously` every class.
#[test]
fn the_three_flags_select_oxlints_classes() {
    let none = FixScope::NONE;
    for class in [
        FixClass::Fix,
        FixClass::Suggestion,
        FixClass::DangerousFix,
        FixClass::DangerousSuggestion,
    ] {
        assert!(!none.allows(class), "{class:?} without a flag");
    }
    let plain = FixScope::from_flags(true, false, false);
    assert!(plain.allows(FixClass::Fix));
    assert!(!plain.allows(FixClass::Suggestion));
    assert!(!plain.allows(FixClass::DangerousFix));
    assert!(!plain.allows(FixClass::DangerousSuggestion));
    let suggestions = FixScope::from_flags(false, true, false);
    assert!(!suggestions.allows(FixClass::Fix));
    assert!(suggestions.allows(FixClass::Suggestion));
    assert!(!suggestions.allows(FixClass::DangerousFix));
    let both = FixScope::from_flags(true, true, false);
    assert!(both.allows(FixClass::Fix));
    assert!(both.allows(FixClass::Suggestion));
    assert!(!both.allows(FixClass::DangerousFix));
    let dangerous = FixScope::from_flags(false, false, true);
    for class in [
        FixClass::Fix,
        FixClass::Suggestion,
        FixClass::DangerousFix,
        FixClass::DangerousSuggestion,
    ] {
        assert!(dangerous.allows(class), "{class:?} under --fix-dangerously");
    }
    assert_eq!(dangerous, FixScope::ALL);
    // The default is the scope of a run with no fix flag.
    assert_eq!(FixScope::default(), FixScope::NONE);
}

/// A dangerous class needs the dangerous flag and its own qualifier,
///  so a scope that allows only `Fix` withholds `DangerousFix`.
#[test]
fn a_dangerous_class_needs_the_dangerous_bit() {
    for class in [FixClass::DangerousFix, FixClass::DangerousSuggestion] {
        assert_ne!(class.bits() & 4, 0, "{class:?} carries the dangerous bit");
        assert!(!FixScope::from_flags(true, true, false).allows(class));
        assert!(FixScope::from_flags(true, true, true).allows(class));
    }
}
