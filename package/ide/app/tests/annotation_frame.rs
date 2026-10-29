//! The frame stamp repaints for a different set of visible annotations or inks,
//!  and reuses the image for an
//! equal set,
//!  whatever happened to the snapshots outside the materialized rows.

/// The stamp,
///  its inputs,
///  and the visible annotation records.
use ide_app::{
    annotation::{Mark, Visible},
    annotation_layout::AnnotationColors,
    code_colors::CodeColors,
    document::Document,
    language::diagnostics::Severity,
    shaped_text::Viewport,
    source_frame::FrameStamp,
    source_style::SourceStyles,
    virtual_row::{Block, HintPlace, MessageRow, ROW_HEIGHT},
};
/// Visible annotations are shared with the layout.
use std::sync::Arc;

/// Dark annotation inks.
const INKS: AnnotationColors = AnnotationColors {
    hint: [150, 160, 170, 179],
    error: [255, 153, 164, 255],
    warning: [252, 225, 0, 255],
    information: [96, 205, 255, 255],
    suggestion: [180, 180, 180, 255],
};

/// The stamp of `document` with `visible` annotations painted in `inks`.
fn stamp(document: &Document, visible: Visible, inks: AnnotationColors) -> FrameStamp {
    let viewport = Viewport {
        first: 0,
        count: 27,
        width: 1044.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
    let colors = CodeColors {
        foreground: [240, 240, 240, 255],
        selected: [255, 255, 255, 255],
        dark: true,
    };
    // `Arc::new` shares the visible list, as the renderer shares it with the layout.
    return FrameStamp::new(document, viewport, colors, SourceStyles::from([]))
        .with_annotations(Arc::new(visible), inks);
}

/// The block above line 0 with one hint `label`,
///  one message row `message`,
///  and `held` message space.
fn block(label: &str, message: &str, held: f32) -> Block {
    return Block {
        line: 0,
        hint_rows: 1,
        hints: vec![HintPlace {
            row: 0,
            position: 3,
            x: 27.0,
            width: 30.0,
            text: label.to_string(),
        }],
        messages: vec![MessageRow {
            start: 0,
            continued: false,
            severity: Severity::Error,
            gap: None,
            text: message.to_string(),
        }],
        held: (0.0, held),
    };
}

/// One block with a hint and a message,
///  and one mark of `severity`.
fn shown(label: &str, severity: Severity) -> Visible {
    return Visible {
        blocks: vec![Arc::new(block(label, "Error: wrong", 0.0))],
        marks: vec![Mark {
            start: 0,
            end: 3,
            severity,
        }],
    };
}

/// Equal visible annotations reuse the image;
///  a different label,
///  message,
///  held space,
///  mark,
///  or ink repaints.
#[test]
fn visible_annotations_and_inks_are_paint_inputs() {
    let document = Document::new("let x = 1;");
    let original = stamp(&document, shown(": i32", Severity::Error), INKS);
    assert!(
        original == stamp(&document, shown(": i32", Severity::Error), INKS),
        "equal blocks in separate allocations must reuse the image"
    );
    assert!(
        original != stamp(&document, shown(": u32", Severity::Error), INKS),
        "a different hint label must repaint"
    );
    assert!(
        original != stamp(&document, shown(": i32", Severity::Warning), INKS),
        "a different severity must repaint"
    );
    assert!(
        original != stamp(&document, Visible::default(), INKS),
        "removing annotations must repaint"
    );
    let mut reworded = shown(": i32", Severity::Error);
    reworded.blocks = vec![Arc::new(block(": i32", "Error: other", 0.0))];
    assert!(
        original != stamp(&document, reworded, INKS),
        "a different message row must repaint"
    );
    // Held space changes where rows are inside the frame although no text differs.
    let mut taller = shown(": i32", Severity::Error);
    taller.blocks = vec![Arc::new(block(": i32", "Error: wrong", 3.0 * ROW_HEIGHT))];
    assert!(
        original != stamp(&document, taller, INKS),
        "a different block height must repaint"
    );
    let mut light = INKS;
    light.error = [196, 43, 28, 255];
    assert!(
        original != stamp(&document, shown(": i32", Severity::Error), light),
        "a different annotation ink must repaint"
    );
}
