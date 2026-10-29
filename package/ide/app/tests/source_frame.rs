//! Source-image invalidation must preserve caret geometry without needless painting.

/// Consumer APIs use the same stamp as the native rendering boundary.
use ide_app::{
    code_colors::CodeColors,
    document::{Document, ReadingPosition},
    find::{FindRange, FindRanges},
    shaped_text::Viewport,
    source_frame::FrameStamp,
    source_style::{SourceStyles, StyleSpan},
};

/// Source-image fixture with explicit physical geometry and system colors.
fn stamp(document: &Document) -> FrameStamp {
    // Borrow source and empty syntax without transferring their ownership.
    return FrameStamp::new(document, viewport(), colors(), SourceStyles::from([]));
}

/// Standard logical viewport,
///  with a nonintegral scale to exercise physical identity.
fn viewport() -> Viewport {
    return Viewport {
        first: 0,
        count: 27,
        width: 1044.0,
        scale: 1.25,
        wrap: None,
        clip: None,
    };
}

/// Native dark foreground and selection ink.
fn colors() -> CodeColors {
    return CodeColors {
        foreground: [240, 240, 240, 255],
        selected: [255, 255, 255, 255],
        dark: true,
    };
}

/// Moving an empty selection changes caret position,
///  not source-image pixels.
#[test]
fn caret_only_movement_reuses_image() {
    // What: mut permits selection updates; Document still exposes no file-writing operation.
    // Why: Exercise actual reading-state changes rather than two fabricated cache keys.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const document = new Document('猫 and Latin');
    // const before = stamp(document); document.select({anchor: 5, head: 5, viewport: 0});
    // ```
    let mut document = Document::new("猫 and Latin");
    let before = stamp(&document);
    document.select(ReadingPosition {
        anchor: 5,
        head: 5,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    // Assert the exact rendering gate remains closed for caret-only movement.
    assert!(before == stamp(&document));
}

/// Both adding and removing selection need new foreground pixels.
#[test]
fn selection_invalidates_but_direction_does_not() {
    let mut document = Document::new("猫 and Latin");
    let empty = stamp(&document);
    document.select(ReadingPosition {
        anchor: 1,
        head: 5,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    let selected = stamp(&document);
    assert!(empty != selected);
    document.select(ReadingPosition {
        anchor: 5,
        head: 1,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    assert!(selected == stamp(&document));
    document.select(ReadingPosition {
        anchor: 5,
        head: 5,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    assert!(selected != stamp(&document));
}

/// Every tile,
///  palette,
///  and syntax input affects paint identity.
#[test]
fn materialization_scale_theme_and_syntax_invalidate() {
    let document = Document::new("猫 and Latin");
    let original = stamp(&document);
    let mut area = viewport();
    area.first = 1;
    assert!(original != FrameStamp::new(&document, area, colors(), SourceStyles::from([])));
    area = viewport();
    area.count += 1;
    assert!(original != FrameStamp::new(&document, area, colors(), SourceStyles::from([])));
    area = viewport();
    area.width += 1.0;
    assert!(original != FrameStamp::new(&document, area, colors(), SourceStyles::from([])));
    area = viewport();
    area.scale = 1.5;
    assert!(original != FrameStamp::new(&document, area, colors(), SourceStyles::from([])));
    // The wrap width decides where every line breaks, so a frame at another width is another frame.
    area = viewport();
    area.wrap = Some(500.0);
    assert!(original != FrameStamp::new(&document, area, colors(), SourceStyles::from([])));
    area.wrap = Some(501.0);
    assert!(
        FrameStamp::new(&document, area, colors(), SourceStyles::from([]))
            != FrameStamp::new(
                &document,
                Viewport {
                    wrap: Some(500.0),
                    clip: None,
                    ..area
                },
                colors(),
                SourceStyles::from([])
            )
    );
    let mut palette = colors();
    palette.dark = false;
    assert!(original != FrameStamp::new(&document, viewport(), palette, SourceStyles::from([])));
    palette = colors();
    palette.foreground = [0, 0, 0, 255];
    assert!(original != FrameStamp::new(&document, viewport(), palette, SourceStyles::from([])));
    palette = colors();
    palette.selected = [0, 0, 0, 255];
    assert!(original != FrameStamp::new(&document, viewport(), palette, SourceStyles::from([])));
    let styles = [StyleSpan {
        start: 0,
        end: 1,
        style: 1,
    }];
    assert!(
        original != FrameStamp::new(&document, viewport(), colors(), SourceStyles::from(styles))
    );
}

/// Reloads must invalidate even when reading position and geometry do not change.
#[test]
fn source_revision_invalidates() {
    let mut document = Document::new("old source");
    let original = stamp(&document);
    let reload = document.prepare_reload("new source");
    document.apply_reload(reload);
    assert!(original != stamp(&document));
}

/// In-file find matches are a paint input:
///  a different match list is a different frame.
#[test]
fn find_matches_invalidate_and_equal_lists_reuse() {
    let document = Document::new("needle and needle");
    let without = stamp(&document);
    // What: `FindRanges::from` moves a fixed list into shared immutable storage.
    // Why: The renderer hands the stamp the same shared list the worker produced.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const first: ReadonlyArray<FindRange> = [{ start: 0, end: 6 }];
    // ```
    let first = FindRanges::from([FindRange { start: 0, end: 6 }]);
    let both = FindRanges::from([
        FindRange { start: 0, end: 6 },
        FindRange { start: 11, end: 17 },
    ]);
    // What: `clone` on a shared list copies its pointer, not its ranges.
    // Why: Two stamps holding the same allocation must compare equal without scanning.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const marked = stamp(document).withMatches(first);
    // ```
    let marked = stamp(&document).with_matches(first.clone());
    assert!(
        without != marked,
        "adding matches must invalidate the frame"
    );
    assert!(
        marked == stamp(&document).with_matches(first.clone()),
        "the same shared match list must reuse the frame"
    );
    assert!(
        marked == stamp(&document).with_matches(FindRanges::from([FindRange { start: 0, end: 6 }])),
        "an equal match list in another allocation must reuse the frame"
    );
    assert!(
        marked != stamp(&document).with_matches(both),
        "a different match list must invalidate the frame"
    );
    assert!(
        without == stamp(&document).with_matches(FindRanges::from([])),
        "an empty match list is the frame without matches"
    );
}
