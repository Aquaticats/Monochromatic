//! What a new width redoes:
//!  only the work it needs,
//!  and the lines near the view first.

use crate::fixtures::{
    CODE,
    SHOWN,
    block_at,
    error,
    everywhere,
    hint,
    start_of,
    store,
};
use ide_app::{
    language::identity::DocumentStamp,
    shaped_text::TextShaper,
};

/// A new width packs the lines that have hints again and measures no message row twice;
///  the same width again does no shaping,
/// and only rows wider than the new width are broken by shaping.
#[test]
fn a_new_width_does_only_the_work_it_needs() {
    let mut text = String::new();
    for number in 0..40 {
        text.push_str(&format!("{CODE} {number}\n"));
    }
    let hints = vec![
        hint(start_of(&text, 3) + 10, ": a"),
        hint(start_of(&text, 9) + 10, ": b"),
    ];
    let diagnostics = vec![
        error(
            start_of(&text, 5) + 4,
            start_of(&text, 5) + 9,
            "short message",
        ),
        error(
            start_of(&text, 7) + 4,
            start_of(&text, 7) + 9,
            "expected `u32`, found `&str` because the argument named first_item is a string slice here",
        ),
    ];
    let (mut annotations, document) = store(&text, hints, diagnostics);
    let mut shaper = TextShaper::new();
    assert_eq!(annotations.work().packed, 0);
    annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(900.0),
        &everywhere(),
    );
    let first = annotations.work();
    assert_eq!(first.packed, 2, "one packing for each line that has hints");
    assert!(first.measured >= 2, "every message row was measured once");
    annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(900.0),
        &everywhere(),
    );
    assert_eq!(
        annotations.work(),
        first,
        "the same width again does no shaping"
    );
    annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(260.0),
        &everywhere(),
    );
    let narrow = annotations.work();
    assert_eq!(
        narrow.packed,
        first.packed + 2,
        "a new width packs the two hinted lines again"
    );
    assert_eq!(
        narrow.measured, first.measured,
        "no message row is measured twice"
    );
    assert!(
        narrow.broken >= 1 && narrow.broken <= first.measured,
        "only rows wider than the width are broken by shaping: {}",
        narrow.broken
    );
    // A width nothing is wider than breaks nothing at all.
    let before = annotations.work();
    annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(5000.0),
        &everywhere(),
    );
    assert_eq!(annotations.work().broken, before.broken);
    assert_eq!(annotations.work().measured, before.measured);
}

/// A new width packs and breaks only the lines near the view;
/// the others keep what they had until the view comes
///  near,
/// and `stale_in` tells the window when that is.
#[test]
fn a_new_width_applies_to_the_lines_near_the_view_first() {
    let mut text = String::new();
    for number in 0..40 {
        text.push_str(&format!("{CODE} {number}\n"));
    }
    let hints = vec![
        hint(start_of(&text, 3) + 90, ": a"),
        hint(start_of(&text, 30) + 90, ": b"),
    ];
    let long =
        "expected `u32`, found `&str` because the argument named first_item is a string slice here";
    let diagnostics = vec![
        error(start_of(&text, 5) + 4, start_of(&text, 5) + 9, long),
        error(start_of(&text, 33) + 4, start_of(&text, 33) + 9, long),
    ];
    let (mut annotations, document) = store(&text, hints, diagnostics);
    let mut shaper = TextShaper::new();

    let wide = annotations.assemble(
        SHOWN,
        &document,
        &mut shaper,
        1.0,
        Some(900.0),
        &everywhere(),
    );
    assert_eq!(annotations.work().packed, 2);
    assert!(!annotations.stale_in(Some(900.0), &everywhere()));
    let (near, far) = (0..10, 20..40);
    let narrow = annotations.assemble(SHOWN, &document, &mut shaper, 1.0, Some(260.0), &near);
    assert_eq!(
        annotations.work().packed,
        3,
        "only the hinted line near the view is packed again"
    );
    assert!(
        !annotations.stale_in(Some(260.0), &near),
        "the lines near the view are exact"
    );
    assert!(
        annotations.stale_in(Some(260.0), &far),
        "the lines far from it are stale"
    );
    assert_ne!(
        block_at(&narrow, 3).hints[0].x,
        block_at(&wide, 3).hints[0].x
    );
    assert_ne!(block_at(&narrow, 5).messages, block_at(&wide, 5).messages);
    assert_eq!(
        block_at(&narrow, 30).hints,
        block_at(&wide, 30).hints,
        "a far line keeps its packing"
    );
    assert_eq!(
        block_at(&narrow, 33).messages,
        block_at(&wide, 33).messages,
        "a far line keeps its rows"
    );
    // The view comes near the other lines.
    let after = annotations.assemble(SHOWN, &document, &mut shaper, 1.0, Some(260.0), &far);
    assert_eq!(
        annotations.work().packed,
        4,
        "the far hinted line is packed when the view comes near"
    );
    assert!(!annotations.stale_in(Some(260.0), &everywhere()));
    assert_ne!(block_at(&after, 30).hints, block_at(&wide, 30).hints);
    assert_ne!(block_at(&after, 33).messages, block_at(&wide, 33).messages);
    // Hints of a snapshot that no longer describes the text leave no stale packing behind.
    let other = DocumentStamp {
        file: 9,
        revision: 9,
    };
    annotations.assemble(
        other,
        &document,
        &mut shaper,
        1.0,
        Some(500.0),
        &everywhere(),
    );
    assert!(!annotations.stale_in(Some(500.0), &everywhere()));
}
