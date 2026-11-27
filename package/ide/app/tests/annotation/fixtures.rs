//! The documents,
//! stamps,
//! snapshots,
//! hints and problems every test in this crate builds:
//!  the five-line displayed text,
//!  a server identity,
//!  an inlay hint and a diagnostic,
//!  a hint answer for a line range,
//!  the blocks a frame assembles,
//!  and the second revision an edit makes.

use helix_core::Rope;
use ide_app::{
    annotation::Annotations,
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
    shaped_text::TextShaper,
    virtual_row::Block,
};
use std::sync::Arc;

/// The displayed text every test uses:
///  five lines,
///  the third empty,
///  the last without a terminator.
pub const TEXT: &str = "let a = 1;\nlet bb = 22;\n\nfn main() {\n}";

/// The stamp of the displayed text.
pub const SHOWN: DocumentStamp = DocumentStamp {
    file: 3,
    revision: 7,
};

/// The process every test record names.
pub fn server() -> ServerIdentity {
    return ServerIdentity {
        name: "scripted".to_string(),
        instance: 1,
    };
}

/// One hint at `position` with `label`.
pub fn hint(
    position: usize,
    label: &str,
) -> InlayHint {
    return InlayHint {
        position,
        label: label.to_string(),
        kind: Some(HintKind::Type),
        padding_left: true,
        padding_right: false,
        server: server(),
    };
}

/// What:
///  One diagnostic over `start..end`;
///  `Option<Severity>` lets a test omit the severity.
/// Why:
///  Records are built exactly as the Language module builds them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function problem(start: number, end: number, severity: Severity | undefined, message: string): Diagnostic;
/// ```
pub fn problem(
    start: usize,
    end: usize,
    severity: Option<Severity>,
    message: &str,
) -> Diagnostic {
    return Diagnostic {
        start,
        end,
        severity,
        code: None,
        message: message.to_string(),
        server: server(),
        freshness: Freshness::Versioned,
    };
}

/// Shared hint snapshot for `stamp` that answers for every line of the text.
pub fn hints(
    stamp: DocumentStamp,
    items: Vec<InlayHint>,
) -> Option<Arc<HintsSnapshot>> {
    return ranged(stamp, (0, 5), items);
}

/// Shared hint snapshot for `stamp` that answers for the lines `asked.0` up to,
///  not including,
///  `asked.1`.
pub fn ranged(
    stamp: DocumentStamp,
    asked: (usize, usize),
    items: Vec<InlayHint>,
) -> Option<Arc<HintsSnapshot>> {
    return Some(Arc::new(HintsSnapshot {
        stamp,
        first_line: asked.0,
        last_line: asked.1,
        hints: items,
    }));
}

/// What:
///  The blocks the store assembles for the text `stamp`,
///  packed by the production shaper at scale one.
///       `&mut Annotations` lends the store for change,
///  because assembly fills its packing cache.
/// Why:
///  Blocks are what the vertical mapping and every frame take from the store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blocks(store: Annotations, stamp: DocumentStamp): Block[];
/// ```
pub fn blocks(
    store: &mut Annotations,
    stamp: DocumentStamp,
) -> Vec<Arc<Block>> {
    let document = Document::new(TEXT);
    let mut shaper = TextShaper::new();
    return store.assemble(stamp, &document, &mut shaper, 1.0, None, &(0..usize::MAX));
}

/// The hint texts of the block above `line`,
///  or nothing when the line has no block.
pub fn hint_texts(
    assembled: &[Arc<Block>],
    line: usize,
) -> Vec<String> {
    let mut texts = Vec::new();
    for block in assembled {
        if block.line == line {
            for hint in &block.hints {
                texts.push(hint.text.clone());
            }
        }
    }
    return texts;
}

/// The hint snapshot a test expects to exist.
pub fn unwrapped(snapshot: Option<Arc<HintsSnapshot>>) -> Arc<HintsSnapshot> {
    return snapshot.expect("hint snapshot");
}

/// The diagnostics snapshot of one `rustc` problem with `message`.
pub fn problems(
    stamp: DocumentStamp,
    message: &str,
) -> Arc<DiagnosticsSnapshot> {
    return diagnostics(
        stamp,
        vec![("rustc", vec![problem(4, 5, Some(Severity::Error), message)])],
    )
    .expect("diagnostics snapshot");
}

/// The line each block stands above,
///  in order.
pub fn block_lines(assembled: &[Arc<Block>]) -> Vec<usize> {
    let mut lines = Vec::new();
    for block in assembled {
        lines.push(block.line);
    }
    return lines;
}

/// Shared diagnostics snapshot for `stamp` with one group per `(source, items)` pair.
pub fn diagnostics(
    stamp: DocumentStamp,
    groups: Vec<(&str, Vec<Diagnostic>)>,
) -> Option<Arc<DiagnosticsSnapshot>> {
    let mut built = Vec::new();
    for (source, items) in groups {
        built.push(SourceGroup {
            source: source.to_string(),
            items,
        });
    }
    return Some(Arc::new(DiagnosticsSnapshot {
        stamp,
        groups: built,
    }));
}

/// Another revision of the same file,
///  as an edit makes it.
pub const EDITED: DocumentStamp = DocumentStamp {
    file: 3,
    revision: 8,
};

/// Store the answer of `stamp` that asked about the lines `first` up to,
///  not including,
///  `last`.
pub fn answered(
    store: &mut Annotations,
    stamp: DocumentStamp,
    first: usize,
    last: usize,
) {
    let text = Rope::from(LONG);
    let items = vec![hint(text.line_to_char(first), ": T")];
    let snapshot = unwrapped(ranged(stamp, (first, last), items));
    assert!(store.accept_hints(stamp, &text, snapshot));
}

/// Store the answer of `stamp` that asked about the lines `first` up to,
///  not including,
///  `last`,
///  and put the hints `items` in it.
pub fn answered_with(
    store: &mut Annotations,
    stamp: DocumentStamp,
    asked: (usize, usize),
    items: Vec<InlayHint>,
) {
    let text = Rope::from(LONG);
    let snapshot = unwrapped(ranged(stamp, asked, items));
    assert!(store.accept_hints(stamp, &text, snapshot));
}

/// A hundred short lines,
///  so answers for the lines of a long file can hold hints.
pub const LONG: &str = "a\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\n\
a\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\n\
a\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\n\
a\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\n\
a\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\na\n";
