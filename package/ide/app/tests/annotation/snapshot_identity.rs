//! Which snapshot a frame trusts:
//!  a stale one paints nothing and shows no card,
//!  two snapshots of one answer are recognized by identity,
//!  and an accept stores only snapshots of the displayed text.

use crate::fixtures::{
    SHOWN,
    TEXT,
    blocks,
    diagnostics,
    hint,
    hint_texts,
    hints,
    problem,
    problems,
    unwrapped,
};
use helix_core::Rope;
use ide_app::{
    annotation::Annotations,
    language::identity::DocumentStamp,
};

/// Snapshots accepted for another file generation or another revision paint nothing for the displayed text:
/// no block,
///  no underline,
///  and no problem at the caret.
#[test]
fn stale_snapshots_paint_nothing_and_show_no_card() {
    let text = Rope::from(TEXT);
    for stamp in [
        DocumentStamp {
            file: 2,
            revision: 7,
        },
        DocumentStamp {
            file: 3,
            revision: 6,
        },
        DocumentStamp {
            file: 3,
            revision: 8,
        },
    ] {
        // The snapshots are accepted while their own text is displayed; then the displayed text moves on.
        let mut annotations = Annotations::default();
        annotations.replace(
            stamp,
            &text,
            hints(stamp, vec![hint(5, ": i32")]),
            diagnostics(stamp, vec![("rustc", vec![problem(4, 5, None, "unused")])]),
        );
        assert_eq!(
            blocks(&mut annotations, stamp).len(),
            1,
            "positive control: {stamp:?}"
        );
        let stale = blocks(&mut annotations, SHOWN);
        let mut hinted = 0;
        let mut messaged = 0;
        for block in &stale {
            hinted += block.hints.len();
            messaged += block.messages.len();
        }
        assert_eq!(hinted, 0, "a stale hint was painted: {stamp:?}");
        assert_eq!(messaged, 0, "a stale message was painted: {stamp:?}");
        assert!(
            annotations.marks(SHOWN, &text, 0, 5).is_empty(),
            "a stale mark was painted: {stamp:?}"
        );
        assert!(annotations.at(SHOWN, 4).is_empty());
    }
    let mut current = Annotations::default();
    current.replace(
        SHOWN,
        &text,
        hints(SHOWN, vec![hint(5, ": i32")]),
        diagnostics(SHOWN, vec![("rustc", vec![problem(4, 5, None, "unused")])]),
    );
    let shown = blocks(&mut current, SHOWN);
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].hints.len(), 1);
    assert_eq!(shown[0].messages.len(), 1);
    assert_eq!(current.marks(SHOWN, &text, 0, 5).len(), 1);
    // A snapshot handed over for text that is not displayed is never grouped or painted.
    let mut refused = Annotations::default();
    refused.replace(
        SHOWN,
        &text,
        hints(
            DocumentStamp {
                file: 3,
                revision: 8,
            },
            vec![hint(5, ": i32")],
        ),
        None,
    );
    assert!(blocks(&mut refused, SHOWN).is_empty());
}

/// Handing back the same shared snapshots is recognized,
///  so a poll does not rebuild the index.
#[test]
fn identical_snapshots_are_recognized_by_identity() {
    let hint_snapshot = hints(SHOWN, vec![hint(5, ": i32")]);
    let problem_snapshot = diagnostics(SHOWN, vec![("rustc", vec![problem(4, 5, None, "x")])]);
    // `clone` on an `Option<Arc<...>>` copies the pointer, not the snapshot.
    let mut annotations = Annotations::default();
    annotations.replace(
        SHOWN,
        &Rope::from(TEXT),
        hint_snapshot.clone(),
        problem_snapshot.clone(),
    );
    assert!(annotations.holds(&hint_snapshot, &problem_snapshot));
    let equal_copy = hints(SHOWN, vec![hint(5, ": i32")]);
    assert!(!annotations.holds(&equal_copy, &problem_snapshot));
    assert!(!annotations.holds(&None, &problem_snapshot));
    assert!(Annotations::default().holds(&None, &None));
}

/// The Language poll's entry points store only snapshots of the displayed text,
///  hand them out only for that
/// text,
///  keep what was held when a snapshot is refused,
///  and index accepted diagnostics for painting.
#[test]
fn accepting_stores_only_snapshots_of_the_displayed_text() {
    let text = Rope::from(TEXT);
    let older = DocumentStamp {
        file: 3,
        revision: 6,
    };
    let mut store = Annotations::default();
    assert!(
        !store.accept_hints(
            SHOWN,
            &text,
            unwrapped(hints(older, vec![hint(5, ": old")]))
        ),
        "hints for an earlier revision were stored"
    );
    assert!(
        !store.accept_diagnostics(SHOWN, &text, problems(older, "old")),
        "diagnostics for an earlier revision were stored"
    );
    assert!(store.hints(SHOWN).is_none() && store.diagnostics(SHOWN).is_none());
    assert!(store.accept_hints(
        SHOWN,
        &text,
        unwrapped(hints(SHOWN, vec![hint(5, ": i32")]))
    ));
    assert!(store.accept_diagnostics(SHOWN, &text, problems(SHOWN, "current")));
    assert!(
        store.hints(older).is_none(),
        "hints were handed out for another revision"
    );
    assert!(
        store.diagnostics(older).is_none(),
        "diagnostics were handed out for another revision"
    );
    assert_eq!(store.hints(SHOWN).expect("held hints").hints.len(), 1);
    assert_eq!(
        store
            .diagnostics(SHOWN)
            .expect("held diagnostics")
            .groups
            .len(),
        1
    );
    // Accepted diagnostics are indexed: the frame and the caret find them.
    assert_eq!(hint_texts(&blocks(&mut store, SHOWN), 0), [": i32"]);
    assert_eq!(
        store.marks(SHOWN, &text, 0, 5).len(),
        1,
        "accepted diagnostics were not indexed"
    );
    assert_eq!(store.at(SHOWN, 4)[0].message, "current");
    // A refused snapshot leaves the held ones, and their index, in place.
    assert!(!store.accept_diagnostics(SHOWN, &text, problems(older, "late")));
    assert!(!store.accept_hints(SHOWN, &text, unwrapped(hints(older, Vec::new()))));
    assert_eq!(store.at(SHOWN, 4)[0].message, "current");
    assert_eq!(hint_texts(&blocks(&mut store, SHOWN), 0), [": i32"]);
    // A newer accepted snapshot replaces the index and the message rows.
    assert!(store.accept_diagnostics(SHOWN, &text, problems(SHOWN, "replaced")));
    assert_eq!(store.at(SHOWN, 4).len(), 1);
    assert_eq!(store.at(SHOWN, 4)[0].message, "replaced");
    let rows = blocks(&mut store, SHOWN);
    assert_eq!(rows[0].messages.len(), 1);
    assert_eq!(rows[0].messages[0].text, "Error (rustc): replaced");
}
