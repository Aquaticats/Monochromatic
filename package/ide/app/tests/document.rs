//! Consumer-interface tests for external reloads and reading correspondence.

/// What:
///  Import the same interface consumed by the native application.
/// Why:
///  These tests must exercise the production mapping,
///  not a duplicate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Document, ReadingPosition } from 'ide-app/document';
/// ```
use ide_app::document::{Document, ReadingAffinity, ReadingPosition};

/// Verify the user's caret example without any file writes.
#[test]
fn caret_follows_surviving_word() {
    // What: mut permits calling state-changing document methods.
    // Why: External reloads change the snapshot, never the project files.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const doc = new Document('I am a big cat.');
    // ```
    let mut doc = Document::new("I am a big cat.");
    doc.select(ReadingPosition {
        anchor: 9,
        head: 9,
        viewport: 7,
        affinity: ReadingAffinity::Downstream,
    });
    let reload = doc.prepare_reload("I was a big cat.");
    // What: assert macros terminate a test when its condition fails.
    // Why: The exact caret and viewport mapping are the acceptance boundary.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert(doc.applyReload(reload));
    // expect(doc.position()).toEqual({anchor: 10, head: 10, viewport: 8});
    // ```
    assert!(doc.apply_reload(reload));
    assert_eq!(
        doc.position(),
        ReadingPosition {
            anchor: 10,
            head: 10,
            viewport: 8,
            affinity: ReadingAffinity::Downstream,
        }
    );
}

/// Verify replacement selection cannot relocate to a later identical string.
#[test]
fn selected_region_follows_replacement_not_literal_match() {
    // Create a mutable document owned by this test.
    let mut doc = Document::new("I am a big cat");
    doc.select(ReadingPosition {
        anchor: 2,
        head: 6,
        viewport: 0,
        affinity: ReadingAffinity::Downstream,
    });
    let reload = doc.prepare_reload("I was a big cat, but now I am a human!");
    assert!(doc.apply_reload(reload));
    assert_eq!(doc.selected_text(), "was a");
    assert_eq!(doc.position().anchor, 2);
    assert_eq!(doc.position().head, 7);
}

/// Verify selection direction is not lost when its content is replaced.
#[test]
fn backwards_selection_preserves_direction() {
    let mut doc = Document::new("I am a big cat");
    doc.select(ReadingPosition {
        anchor: 6,
        head: 2,
        viewport: 0,
        affinity: ReadingAffinity::Downstream,
    });
    let reload = doc.prepare_reload("I was a big cat, but now I am a human!");
    assert!(doc.apply_reload(reload));
    assert_eq!(doc.selected_text(), "was a");
    assert_eq!(doc.position().anchor, 7);
    assert_eq!(doc.position().head, 2);
}

/// Verify a late worker cannot replace a newer displayed revision.
#[test]
fn stale_reload_is_rejected() {
    let mut doc = Document::new("before");
    let first = doc.prepare_reload("first");
    let late = doc.prepare_reload("late");
    assert!(doc.apply_reload(first));
    assert!(!doc.apply_reload(late));
    // What: to_string copies the current rope for an assertion.
    // Why: Assert source content through the public read interface.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(doc.text.toString()).toBe('first');
    // ```
    assert_eq!(doc.text().to_string(), "first");
}

/// Verify moving the selection during comparison is not undone by its result.
#[test]
fn reload_maps_latest_reading_position() {
    let mut doc = Document::new("I am a big cat");
    let reload = doc.prepare_reload("I was a big cat");
    doc.select(ReadingPosition {
        anchor: 11,
        head: 14,
        viewport: 7,
        affinity: ReadingAffinity::Downstream,
    });
    assert!(doc.apply_reload(reload));
    assert_eq!(doc.selected_text(), "cat");
}

/// Verify character positions are not accidentally treated as UTF-8 bytes.
#[test]
fn unicode_prefix_preserves_selection() {
    let mut doc = Document::new("猫 am a big cat");
    doc.select(ReadingPosition {
        anchor: 2,
        head: 6,
        viewport: 0,
        affinity: ReadingAffinity::Downstream,
    });
    let reload = doc.prepare_reload("猫 was a big cat, am a");
    assert!(doc.apply_reload(reload));
    assert_eq!(doc.selected_text(), "was a");
}

/// Verify empty snapshots and complete deletion leave valid positions.
#[test]
fn deletion_clamps_positions() {
    let mut doc = Document::new("abc");
    doc.select(ReadingPosition {
        anchor: 0,
        head: 3,
        viewport: 2,
        affinity: ReadingAffinity::Downstream,
    });
    let reload = doc.prepare_reload("");
    assert!(doc.apply_reload(reload));
    assert_eq!(doc.position(), ReadingPosition::default());
    assert_eq!(doc.selected_text(), "");
}

/// The side of a wrap offset is part of the position:
/// selecting keeps it,
///  and a reload keeps it only for a head that did not move.
#[test]
fn the_side_of_a_wrap_offset_is_kept_by_selection_and_by_a_reload_that_leaves_the_head() {
    let mut doc = Document::new("I was a big cat.");
    doc.select(ReadingPosition {
        anchor: 9,
        head: 9,
        viewport: 0,
        affinity: ReadingAffinity::Upstream,
    });
    assert_eq!(
        doc.position().affinity,
        ReadingAffinity::Upstream,
        "a selection keeps the side it names"
    );
    let after = doc.prepare_reload("I was a big cat, again.");
    assert!(doc.apply_reload(after));
    assert_eq!(doc.position().head, 9);
    assert_eq!(
        doc.position().affinity,
        ReadingAffinity::Upstream,
        "a reload that leaves the head keeps its side"
    );
    let before = doc.prepare_reload("So I was a big cat, again.");
    assert!(doc.apply_reload(before));
    assert_eq!(doc.position().head, 12, "the insertion moved the head");
    assert_eq!(
        doc.position().affinity,
        ReadingAffinity::Downstream,
        "a head that moved takes the default side"
    );
}
