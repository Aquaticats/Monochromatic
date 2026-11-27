//! The lines a stored hint answer covers:
//!  its own range,
//!  the union of answers that overlap or touch but not the gap between them,
//!  one text revision only,
//!  and an empty or refused answer leaving the hints it replaced.

use crate::fixtures::{
    EDITED,
    LONG,
    SHOWN,
    TEXT,
    answered,
    answered_with,
    blocks,
    hint,
    hint_texts,
    ranged,
    unwrapped,
};
use helix_core::Rope;
use ide_app::annotation::Annotations;

/// The lines a stored answer asked about are covered,
///  the line before the first and the line the request ended before are not,
///  and an empty range is never covered.
#[test]
fn the_lines_a_stored_answer_asked_about_are_covered() {
    let mut store = Annotations::default();
    assert!(
        !store.answered_covers(SHOWN, 0..1),
        "lines were covered before any answer"
    );
    answered(&mut store, SHOWN, 35, 60);
    assert!(
        store.answered_covers(SHOWN, 35..60),
        "the asked lines are not covered"
    );
    assert!(
        store.answered_covers(SHOWN, 40..60),
        "lines inside the asked lines are not covered"
    );
    assert!(
        !store.answered_covers(SHOWN, 34..60),
        "the line before the asked lines was covered"
    );
    assert!(
        !store.answered_covers(SHOWN, 35..61),
        "the line the request ended before was covered"
    );
    assert!(
        !store.answered_covers(SHOWN, 40..40),
        "an empty range was covered"
    );
}

/// Lines asked about by two answers are covered together when the answers overlap or touch,
///  and a gap between them is not covered.
#[test]
fn answers_that_overlap_or_touch_cover_their_union_but_not_a_gap() {
    let mut store = Annotations::default();
    answered(&mut store, SHOWN, 0, 30);
    answered(&mut store, SHOWN, 50, 80);
    assert!(
        !store.answered_covers(SHOWN, 20..60),
        "the gap between two answers was covered"
    );
    answered(&mut store, SHOWN, 30, 40);
    assert!(
        !store.answered_covers(SHOWN, 20..60),
        "a gap that two touching answers left was covered"
    );
    answered(&mut store, SHOWN, 38, 52);
    assert!(
        store.answered_covers(SHOWN, 20..60),
        "three overlapping answers do not cover the lines together"
    );
    assert!(store.answered_covers(SHOWN, 0..80));
    assert!(!store.answered_covers(SHOWN, 0..81));
}

/// Lines an answer of one revision asked about say nothing about another revision:
///  the other revision is not covered before any answer for it,
///  and its first answer drops what the earlier revision covered.
#[test]
fn covered_lines_belong_to_one_text_revision() {
    let mut store = Annotations::default();
    answered(&mut store, SHOWN, 35, 60);
    assert!(
        !store.answered_covers(EDITED, 40..60),
        "lines asked in one revision were covered in the next, before any answer for it"
    );
    answered(&mut store, EDITED, 0, 10);
    assert!(
        !store.answered_covers(EDITED, 40..60),
        "an answer for the next revision kept the lines the earlier revision had covered"
    );
    assert!(store.answered_covers(EDITED, 0..10));
    assert!(
        !store.answered_covers(SHOWN, 40..60),
        "the earlier revision stayed covered"
    );
}

/// A snapshot the store refuses (another revision than the displayed one) covers nothing,
///  and forgetting what was asked keeps the stored hints.
#[test]
fn a_refused_snapshot_covers_nothing_and_forgetting_keeps_the_hints() {
    let text = Rope::from(TEXT);
    let mut store = Annotations::default();
    let late = unwrapped(ranged(EDITED, (0, 5), Vec::new()));
    assert!(!store.accept_hints(SHOWN, &text, late));
    assert!(
        !store.answered_covers(SHOWN, 0..5) && !store.answered_covers(EDITED, 0..5),
        "a refused snapshot covered lines"
    );
    let first = unwrapped(ranged(SHOWN, (0, 5), vec![hint(5, ": i32")]));
    assert!(store.accept_hints(SHOWN, &text, first));
    assert!(store.answered_covers(SHOWN, 0..5));
    store.forget_answered();
    assert!(
        !store.answered_covers(SHOWN, 0..5),
        "forgetting what was asked left the lines covered"
    );
    assert_eq!(
        hint_texts(&blocks(&mut store, SHOWN), 0),
        [": i32"],
        "forgetting what was asked removed the stored hints"
    );
}

/// An answer that put no hint on the lines it asked about covers nothing:
///  a server that is still loading answers with no hint or with some,
///  and the view that settles over the same lines must ask again.
#[test]
fn an_answer_with_no_hint_on_its_lines_covers_nothing() {
    let mut store = Annotations::default();
    answered_with(&mut store, SHOWN, (35, 60), Vec::new());
    assert!(
        !store.answered_covers(SHOWN, 35..60) && !store.answered_covers(SHOWN, 40..41),
        "an answer with no hint covered the lines it asked about"
    );
    // A hint that stands on a line the answer did not ask about does not make the asked lines covered.
    let text = Rope::from(LONG);
    answered_with(
        &mut store,
        SHOWN,
        (35, 60),
        vec![hint(text.line_to_char(90), ": far")],
    );
    assert!(
        !store.answered_covers(SHOWN, 35..60),
        "a hint on an unasked line made the asked lines covered"
    );
    // The same lines answered with a hint on one of them are covered.
    answered_with(
        &mut store,
        SHOWN,
        (35, 60),
        vec![hint(text.line_to_char(40), ": near")],
    );
    assert!(
        store.answered_covers(SHOWN, 35..60),
        "an answer with a hint on its lines does not cover them"
    );
}

/// An answer replaces the hints of the lines it asked about,
///  so one that put no hint there takes those lines out of what earlier answers covered,
///  and the lines around them stay covered.
#[test]
fn an_empty_answer_uncovers_the_lines_it_replaced() {
    let mut store = Annotations::default();
    answered(&mut store, SHOWN, 35, 60);
    answered_with(&mut store, SHOWN, (40, 50), Vec::new());
    assert!(store.answered_covers(SHOWN, 35..40));
    assert!(store.answered_covers(SHOWN, 50..60));
    assert!(
        !store.answered_covers(SHOWN, 40..50) && !store.answered_covers(SHOWN, 35..60),
        "lines an empty answer replaced stayed covered"
    );
}

/// An empty answer takes out only the lines it asked about,
///  and leaves answers that lie apart from them covered.
#[test]
fn an_empty_answer_leaves_answers_apart_from_its_lines_covered() {
    let mut store = Annotations::default();
    answered(&mut store, SHOWN, 0, 30);
    answered(&mut store, SHOWN, 50, 80);
    answered_with(&mut store, SHOWN, (35, 45), Vec::new());
    assert!(store.answered_covers(SHOWN, 0..30));
    assert!(store.answered_covers(SHOWN, 50..80));
    assert!(
        !store.answered_covers(SHOWN, 30..35),
        "an empty answer made the lines before its own covered"
    );
    assert!(
        !store.answered_covers(SHOWN, 45..50),
        "an empty answer made the lines after its own covered"
    );
}
