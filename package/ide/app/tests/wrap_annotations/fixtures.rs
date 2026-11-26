//! The line, stamp, palette, store, hints and messages every test builds:
//!  a line that wraps to several rows,
//!  a stamped annotation store,
//!  a hint and a diagnostic at a chosen place,
//!  and the viewport of the tile.

use ide_app::{
    annotation::Annotations,
    annotation_layout::AnnotationColors,
    document::Document,
    language::{
        diagnostics::{
            Diagnostic,
            DiagnosticsSnapshot,
            Freshness,
            Severity,
            SourceGroup,
        },
        hints::{
            HintKind,
            HintsSnapshot,
            InlayHint,
        },
        identity::{
            DocumentStamp,
            ServerIdentity,
        },
    },
    shaped_text::Viewport,
    virtual_row::Block,
};
use std::{
    ops::Range,
    sync::Arc,
};

/// A line that wraps to several rows at the widths used here,
/// with a clear place for hints on its later rows.
pub const CODE: &str = "let value = compute_total(first_item, second_item); // a long comment that keeps going and going";

/// The stamp of the displayed text.
pub const SHOWN: DocumentStamp = DocumentStamp {
    file: 1,
    revision: 1,
};

/// Palette of the painted checks;
/// the hint ink is dark gray so it cannot be taken for the error ink.
pub const COLORS: AnnotationColors = AnnotationColors {
    hint: [20, 20, 20, 255],
    error: [200, 0, 0, 255],
    warning: [200, 100, 0, 255],
    information: [0, 0, 200, 255],
    suggestion: [90, 90, 90, 255],
};

/// The character offset where line `line` of `text` starts.
pub fn start_of(
    text: &str,
    line: usize,
) -> usize {
    return Document::new(text).text().line_to_char(line);
}

/// The block that stands above `line`.
pub fn block_at(
    blocks: &[Arc<Block>],
    line: usize,
) -> Arc<Block> {
    for block in blocks {
        if block.line == line {
            return Arc::clone(block);
        }
    }
    panic!("block");
}

/// The words of all message rows,
///  in order.
pub fn words(rows: &[ide_app::virtual_row::MessageRow]) -> Vec<String> {
    let mut texts: Vec<&str> = Vec::new();
    for row in rows {
        texts.push(row.text.as_str());
    }
    let mut found: Vec<String> = Vec::new();
    for word in texts.join(" ").split_whitespace() {
        found.push(word.to_string());
    }
    return found;
}

/// The process every record names.
pub fn server() -> ServerIdentity {
    return ServerIdentity {
        name: "scripted".to_string(),
        instance: 1,
    };
}

/// One hint at `position`.
pub fn hint(
    position: usize,
    label: &str,
) -> InlayHint {
    return InlayHint {
        position,
        label: label.to_string(),
        kind: Some(HintKind::Type),
        padding_left: false,
        padding_right: false,
        server: server(),
    };
}

/// One error over `start..end`.
pub fn error(
    start: usize,
    end: usize,
    message: &str,
) -> Diagnostic {
    return Diagnostic {
        start,
        end,
        severity: Some(Severity::Error),
        code: None,
        message: message.to_string(),
        server: server(),
        freshness: Freshness::Versioned,
    };
}

/// A store holding `hints` and `diagnostics` for the shown text.
pub fn store(
    text: &str,
    hints: Vec<InlayHint>,
    diagnostics: Vec<Diagnostic>,
) -> (Annotations, Document) {
    let document = Document::new(text);
    let mut annotations = Annotations::default();
    let lines = document.text().len_lines();
    let snapshot = HintsSnapshot {
        stamp: SHOWN,
        first_line: 0,
        last_line: lines,
        hints,
    };
    let group = DiagnosticsSnapshot {
        stamp: SHOWN,
        groups: vec![SourceGroup {
            source: "scripted".to_string(),
            items: diagnostics,
        }],
    };
    annotations.replace(
        SHOWN,
        document.text(),
        Some(Arc::new(snapshot)),
        Some(Arc::new(group)),
    );
    return (annotations, document);
}

/// Every line is near the view.
pub fn everywhere() -> Range<usize> {
    return 0..usize::MAX;
}

/// A viewport of `count` lines wrapped at `wrap`.
pub fn viewport(
    count: usize,
    wrap: f32,
) -> Viewport {
    return Viewport {
        first: 0,
        count,
        width: wrap + 24.0,
        scale: 1.0,
        wrap: Some(wrap),
        clip: None,
    };
}
