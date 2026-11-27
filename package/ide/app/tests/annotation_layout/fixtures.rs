//! What the four groups of tests below share:
//!  the production shaper and layout,
//!  one labelled hint and one message row,
//!  a text laid out with one block above a line,
//!  and the pixel comparison the assertions use.

/// The production layout and packing,
///  the records they consume,
///  and the vertical mapping blocks feed.
use ide_app::{
    annotation::{
        Label,
        Mark,
        Visible,
    },
    annotation_layout::{
        AnnotationColors,
        AnnotationFrame,
        lay_out,
        pack,
    },
    document::Document,
    language::diagnostics::Severity,
    row_map::RowMap,
    shaped_text::{
        ShapedView,
        TextShaper,
        Viewport,
    },
    virtual_row::{
        Block,
        MessageRow,
    },
};
/// Blocks are shared between the window state and the frames that paint them.
use std::sync::Arc;

/// Inks are irrelevant to geometry;
///  any fixed set will do.
pub const COLORS: AnnotationColors = AnnotationColors {
    hint: [1, 2, 3, 255],
    error: [200, 0, 0, 255],
    warning: [200, 100, 0, 255],
    information: [0, 0, 200, 255],
    suggestion: [90, 90, 90, 255],
};

/// What:
///  Shape `source` and lay out `visible` against it;
///  the tuple returns the document,
///  the view,
///  and the frame.
/// Why:
///  Every assertion compares annotation positions with the geometry the window paints and hit-tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function laidOut(source: string, visible: Visible): [Document, ShapedView, AnnotationFrame];
/// ```
pub fn laid_out(
    source: &str,
    visible: &Visible,
) -> (Document, ShapedView, AnnotationFrame) {
    let document = Document::new(source);
    let mut shaper = TextShaper::new();
    let viewport = Viewport {
        first: 0,
        count: 8,
        width: 900.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
    let view = shaper.prepare(&document, viewport, &[]);
    let frame = lay_out(&view, visible, &mut shaper, COLORS);
    return (document, view, frame);
}

/// What:
///  One hint label at `position`;
///  `&str` lends the text,
///  which the label copies.
/// Why:
///  Packing takes labels exactly as the annotation store keeps them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function label(position: number, text: string): Label;
/// ```
pub fn label(
    position: usize,
    text: &str,
) -> Label {
    return Label {
        position,
        text: text.to_string(),
    };
}

/// One message row of `severity` aligned to source character `start`.
pub fn message(
    start: usize,
    continued: bool,
    severity: Severity,
    text: &str,
) -> MessageRow {
    return MessageRow {
        start,
        continued,
        severity,
        gap: None,
        text: text.to_string(),
    };
}

/// What:
///  Pack `labels` above line `line` of `source`,
///  add `messages`,
///  and shape and lay out the whole text with
///       that one block in the vertical mapping.
///  The tuple returns the view,
///  the frame,
///  and the block.
/// Why:
///  This is the production path:
///  packing against an unstyled row,
///  then painting against the frame's rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withBlock(source: string, line: number, labels: Label[], messages: MessageRow[]): [ShapedView, AnnotationFrame, Block];
/// ```
pub fn with_block(
    source: &str,
    line: usize,
    labels: &[Label],
    messages: Vec<MessageRow>,
) -> (ShapedView, AnnotationFrame, Arc<Block>) {
    let document = Document::new(source);
    let mut shaper = TextShaper::new();
    let row = shaper.row(&document, line, 1.0);
    let (hint_rows, hints) = pack(&row, labels, &mut shaper, 1.0, None);
    let block = Arc::new(Block {
        line,
        hint_rows,
        hints,
        messages,
        held: (0.0, 0.0),
    });
    let map = RowMap::new(document.text().len_lines(), &[(line, block.height())]);
    let viewport = Viewport {
        first: 0,
        count: 8,
        width: 900.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
    let view = shaper.prepare_rows(&document, viewport, &[], &map);
    let visible = Visible {
        blocks: vec![Arc::clone(&block)],
        marks: Vec::new(),
    };
    let frame = lay_out(&view, &visible, &mut shaper, COLORS);
    return (view, frame, block);
}

/// Compare two pixel positions within a thousandth of a pixel;
///  sums of floats may round differently.
pub fn close(
    actual: f32,
    expected: f32,
) -> bool {
    return (actual - expected).abs() < 0.001;
}

/// A mark of `severity` over `start..end`.
pub fn mark(
    start: usize,
    end: usize,
    severity: Severity,
) -> Mark {
    return Mark {
        start,
        end,
        severity,
    };
}
