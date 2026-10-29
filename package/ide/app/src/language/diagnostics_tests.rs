//! Diagnostics versioning,
//!  the unversioned freshness hold,
//!  invalidation,
//!  and aggregation,
//!  without a server.

use super::store::PushVerdict;
use super::{DiagnosticStore, Freshness, Severity};
use crate::language::identity::{DocumentStamp, ServerIdentity};
use helix_core::Rope;
use helix_lsp::{OffsetEncoding, lsp};

const FIRST: DocumentStamp = DocumentStamp {
    file: 1,
    revision: 0,
};
const SECOND: DocumentStamp = DocumentStamp {
    file: 1,
    revision: 1,
};

fn server(name: &str) -> ServerIdentity {
    return ServerIdentity {
        name: name.to_string(),
        instance: 1,
    };
}

fn item(line: u32, source: &str, message: &str) -> lsp::Diagnostic {
    return lsp::Diagnostic {
        range: lsp::Range::new(lsp::Position::new(line, 0), lsp::Position::new(line, 3)),
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        code: Some(lsp::NumberOrString::String("E0308".to_string())),
        source: Some(source.to_string()),
        message: message.to_string(),
        ..Default::default()
    };
}

fn text() -> Rope {
    return Rope::from_str("one\ntwo\nthree\n");
}

fn messages(store: &DiagnosticStore, text: &Rope) -> Vec<String> {
    let snapshot = store.snapshot(text).expect("a file is displayed");
    let mut found = Vec::new();
    for group in snapshot.groups {
        for diagnostic in group.items {
            found.push(diagnostic.message);
        }
    }
    return found;
}

#[test]
fn versioned_push_is_accepted_only_for_the_displayed_version() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    assert_eq!(
        store.push(
            &server("a"),
            OffsetEncoding::Utf16,
            Some(1),
            vec![item(0, "rustc", "future")],
            4
        ),
        PushVerdict::WrongVersion,
        "a set for another document version was accepted"
    );
    assert!(messages(&store, &text).is_empty());
    assert_eq!(
        store.push(
            &server("a"),
            OffsetEncoding::Utf16,
            Some(0),
            vec![item(0, "rustc", "current")],
            4
        ),
        PushVerdict::AcceptedVersioned
    );
    let snapshot = store.snapshot(&text).expect("snapshot");
    assert_eq!(snapshot.stamp, FIRST);
    let diagnostic = &snapshot.groups[0].items[0];
    assert_eq!(diagnostic.freshness, Freshness::Versioned);
    assert_eq!(diagnostic.severity, Some(Severity::Error));
    assert_eq!(diagnostic.code.as_deref(), Some("E0308"));
    assert_eq!((diagnostic.start, diagnostic.end), (0, 3));
}

#[test]
fn reload_removes_every_pushed_set_of_the_previous_revision() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    store.push(
        &server("a"),
        OffsetEncoding::Utf16,
        Some(0),
        vec![item(0, "rustc", "versioned")],
        4,
    );
    store.push(
        &server("b"),
        OffsetEncoding::Utf16,
        None,
        vec![item(1, "lint", "unversioned")],
        4,
    );
    assert_eq!(messages(&store, &text).len(), 2);
    store.reload(SECOND, 1, &[server("a"), server("b")]);
    let snapshot = store.snapshot(&text).expect("snapshot");
    assert_eq!(snapshot.stamp, SECOND);
    assert!(
        snapshot.groups.is_empty(),
        "pushed diagnostics of the previous revision were carried forward"
    );
}

#[test]
fn unversioned_push_is_held_until_the_server_answers_for_the_new_revision() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    store.reload(SECOND, 1, &[server("a")]);
    assert!(store.is_held(&server("a")));
    assert_eq!(
        store.push(
            &server("a"),
            OffsetEncoding::Utf16,
            None,
            vec![item(0, "lint", "during hold")],
            4
        ),
        PushVerdict::Held,
        "an unversioned set was accepted during the hold"
    );
    assert!(
        !store.answered(&server("a"), FIRST),
        "an answer for the previous revision ended the hold"
    );
    assert!(store.is_held(&server("a")));
    assert!(store.answered(&server("a"), SECOND));
    assert_eq!(
        store.push(
            &server("a"),
            OffsetEncoding::Utf16,
            None,
            vec![item(0, "lint", "after hold")],
            4
        ),
        PushVerdict::AcceptedUnversioned
    );
    let snapshot = store.snapshot(&text).expect("snapshot");
    assert_eq!(
        snapshot.groups[0].items[0].freshness,
        Freshness::Unversioned,
        "an unversioned set must never be presented as version-checked"
    );
}

#[test]
fn hold_ends_after_the_fixed_delay_of_its_own_reload_only() {
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    let first = store.reload(SECOND, 1, &[server("a")]);
    let second = store.reload(
        DocumentStamp {
            file: 1,
            revision: 2,
        },
        2,
        &[server("a")],
    );
    assert!(
        !store.hold_expired(first),
        "the timer of an earlier reload ended a later hold"
    );
    assert!(store.is_held(&server("a")));
    assert!(store.hold_expired(second));
    assert!(!store.is_held(&server("a")));
}

#[test]
fn versioned_push_is_not_subject_to_the_hold() {
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    store.reload(SECOND, 1, &[server("a")]);
    assert_eq!(
        store.push(
            &server("a"),
            OffsetEncoding::Utf16,
            Some(1),
            vec![item(0, "rustc", "exact")],
            4
        ),
        PushVerdict::AcceptedVersioned
    );
}

#[test]
fn unversioned_set_with_a_range_past_the_last_line_is_discarded_whole() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    assert_eq!(
        store.push(
            &server("a"),
            OffsetEncoding::Utf16,
            None,
            vec![item(0, "lint", "fits"), item(4, "lint", "past the end")],
            4
        ),
        PushVerdict::LineOutOfRange,
        "an unversioned set naming a missing line was accepted"
    );
    assert!(messages(&store, &text).is_empty());
}

#[test]
fn later_set_from_the_same_server_replaces_the_earlier_one() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    store.push(
        &server("a"),
        OffsetEncoding::Utf16,
        None,
        vec![item(0, "lint", "first")],
        4,
    );
    store.push(
        &server("a"),
        OffsetEncoding::Utf16,
        None,
        vec![item(1, "lint", "second")],
        4,
    );
    assert_eq!(messages(&store, &text), vec!["second".to_string()]);
    store.push(&server("a"), OffsetEncoding::Utf16, None, Vec::new(), 4);
    assert!(messages(&store, &text).is_empty());
}

#[test]
fn diagnostics_are_grouped_by_source_and_ordered_by_position() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    store.push(
        &server("a"),
        OffsetEncoding::Utf16,
        Some(0),
        vec![
            item(2, "rustc", "late"),
            item(0, "rust-analyzer", "native"),
            item(0, "rustc", "early"),
        ],
        4,
    );
    store.pulled_full(
        &server("b"),
        OffsetEncoding::Utf16,
        FIRST,
        None,
        vec![item(1, "rustc", "pulled")],
    );
    let snapshot = store.snapshot(&text).expect("snapshot");
    let mut sources: Vec<&str> = Vec::new();
    for group in snapshot.groups.iter() {
        sources.push(group.source.as_str());
    }
    assert_eq!(sources, vec!["rust-analyzer", "rustc"]);
    let mut rustc: Vec<&str> = Vec::new();
    for found in snapshot.groups[1].items.iter() {
        rustc.push(found.message.as_str());
    }
    assert_eq!(rustc, vec!["early", "pulled", "late"]);
    assert_eq!(snapshot.groups[1].items[1].freshness, Freshness::Pulled);
    assert_eq!(snapshot.groups[1].items[1].server, server("b"));
}

#[test]
fn pull_answer_for_another_revision_is_dropped() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    store.reload(SECOND, 1, &[server("a")]);
    assert!(
        !store.pulled_full(
            &server("a"),
            OffsetEncoding::Utf16,
            FIRST,
            None,
            vec![item(0, "ts", "stale")]
        ),
        "a pull answer for an old revision was accepted"
    );
    assert!(messages(&store, &text).is_empty());
    assert!(store.pulled_full(
        &server("a"),
        OffsetEncoding::Utf16,
        SECOND,
        None,
        vec![item(0, "ts", "current")]
    ));
    assert_eq!(messages(&store, &text), vec!["current".to_string()]);
}

#[test]
fn unchanged_pull_answer_restamps_the_stored_set_for_the_new_revision() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    assert_eq!(store.previous_result_id(&server("a")), None);
    store.pulled_full(
        &server("a"),
        OffsetEncoding::Utf16,
        FIRST,
        Some("r1".to_string()),
        vec![item(0, "ts", "kept")],
    );
    assert_eq!(
        store.previous_result_id(&server("a")),
        Some("r1".to_string())
    );
    store.reload(SECOND, 1, &[server("a")]);
    assert!(
        messages(&store, &text).is_empty(),
        "a pulled set of the previous revision stayed on display"
    );
    assert!(store.pulled_unchanged(&server("a"), SECOND, "r2".to_string()));
    assert_eq!(messages(&store, &text), vec!["kept".to_string()]);
    assert_eq!(
        store.previous_result_id(&server("a")),
        Some("r2".to_string())
    );
}

#[test]
fn server_exit_and_close_clear_what_they_cover() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    store.push(
        &server("a"),
        OffsetEncoding::Utf16,
        None,
        vec![item(0, "lint", "from a")],
        4,
    );
    store.push(
        &server("b"),
        OffsetEncoding::Utf16,
        None,
        vec![item(0, "lint", "from b")],
        4,
    );
    store.pulled_full(
        &server("a"),
        OffsetEncoding::Utf16,
        FIRST,
        Some("r1".to_string()),
        Vec::new(),
    );
    store.server_exited(&server("a"));
    assert_eq!(messages(&store, &text), vec!["from b".to_string()]);
    assert_eq!(store.previous_result_id(&server("a")), None);
    store.close();
    assert!(store.snapshot(&text).is_none());
    assert_eq!(
        store.push(&server("b"), OffsetEncoding::Utf16, None, Vec::new(), 4),
        PushVerdict::NoDocument
    );
}

#[test]
fn ranges_use_the_column_unit_of_the_sending_server() {
    let text = Rope::from_str("\u{1F600}ab\n");
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    let mut wide = item(0, "ts", "after the emoji");
    wide.range = lsp::Range::new(lsp::Position::new(0, 4), lsp::Position::new(0, 5));
    store.push(&server("a"), OffsetEncoding::Utf8, Some(0), vec![wide], 2);
    let snapshot = store.snapshot(&text).expect("snapshot");
    assert_eq!(
        (
            snapshot.groups[0].items[0].start,
            snapshot.groups[0].items[0].end
        ),
        (1, 2)
    );
}

/// Push one unversioned set from server `a` with a single item on line 0.
fn push_unversioned(store: &mut DiagnosticStore, message: &str) -> PushVerdict {
    return store.push(
        &server("a"),
        OffsetEncoding::Utf16,
        None,
        vec![item(0, "lint", message)],
        4,
    );
}

#[test]
fn set_pushed_during_the_hold_is_shown_when_the_server_answers() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    store.reload(SECOND, 1, &[server("a")]);
    assert_eq!(push_unversioned(&mut store, "only push"), PushVerdict::Held);
    assert!(
        messages(&store, &text).is_empty(),
        "a set pushed during the hold was shown before the hold ended"
    );
    assert!(store.answered(&server("a"), SECOND));
    let snapshot = store.snapshot(&text).expect("snapshot");
    assert_eq!(snapshot.stamp, SECOND);
    assert_eq!(
        messages(&store, &text),
        vec!["only push".to_string()],
        "the set pushed during the hold was dropped when the hold ended"
    );
    assert_eq!(
        snapshot.groups[0].items[0].freshness,
        Freshness::Unversioned
    );
}

#[test]
fn set_pushed_during_the_hold_is_shown_when_the_fixed_delay_passes() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    let serial = store.reload(SECOND, 1, &[server("a")]);
    push_unversioned(&mut store, "first during hold");
    push_unversioned(&mut store, "latest during hold");
    assert!(messages(&store, &text).is_empty());
    assert!(store.hold_expired(serial));
    assert_eq!(
        messages(&store, &text),
        vec!["latest during hold".to_string()],
        "the latest set pushed during the hold was not the one shown"
    );
}

#[test]
fn newer_set_replaces_the_one_kept_during_the_hold() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    store.reload(SECOND, 1, &[server("a")]);
    push_unversioned(&mut store, "kept");
    assert_eq!(
        store.push(
            &server("a"),
            OffsetEncoding::Utf16,
            Some(1),
            vec![item(1, "rustc", "versioned")],
            4
        ),
        PushVerdict::AcceptedVersioned
    );
    assert!(store.answered(&server("a"), SECOND));
    assert_eq!(
        messages(&store, &text),
        vec!["versioned".to_string()],
        "a set kept during the hold replaced a newer versioned set when the hold ended"
    );
    assert_eq!(
        push_unversioned(&mut store, "after hold"),
        PushVerdict::AcceptedUnversioned
    );
    assert_eq!(messages(&store, &text), vec!["after hold".to_string()]);
}

#[test]
fn kept_set_ends_with_its_revision_its_server_and_its_file() {
    let text = text();
    let third = DocumentStamp {
        file: 1,
        revision: 2,
    };
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    store.reload(SECOND, 1, &[server("a")]);
    push_unversioned(&mut store, "for the second revision");
    store.reload(third, 2, &[server("a")]);
    assert!(store.answered(&server("a"), third));
    assert!(
        messages(&store, &text).is_empty(),
        "a set kept during an earlier revision's hold was shown for a later revision"
    );
    store.reload(
        DocumentStamp {
            file: 1,
            revision: 3,
        },
        3,
        &[server("a")],
    );
    push_unversioned(&mut store, "from an ended process");
    store.server_exited(&server("a"));
    assert!(!store.is_held(&server("a")));
    assert!(messages(&store, &text).is_empty());
    store.open(
        DocumentStamp {
            file: 2,
            revision: 0,
        },
        0,
    );
    assert!(!store.is_held(&server("a")));
}

#[test]
fn set_with_a_missing_line_is_not_kept_during_the_hold() {
    let text = text();
    let mut store = DiagnosticStore::new();
    store.open(FIRST, 0);
    let serial = store.reload(SECOND, 1, &[server("a")]);
    assert_eq!(
        store.push(
            &server("a"),
            OffsetEncoding::Utf16,
            None,
            vec![item(9, "lint", "past the end")],
            4
        ),
        PushVerdict::LineOutOfRange
    );
    assert!(store.hold_expired(serial));
    assert!(
        messages(&store, &text).is_empty(),
        "a set naming a line the text lacks was kept and shown"
    );
}
