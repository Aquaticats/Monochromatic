//! The minimap's row cache:
//!  shaped source lines kept by what shaped them,
//!  so a new slice of the picture shapes only the lines it has not seen.
//!
//! A line is named by the digest of its characters and of the syntax classes that reach it,
//!  not by its number or the text's revision,
//!  so an edit that changes one line leaves the shaping of every other line valid,
//!  even when the lines after it move to other numbers.

/// The line is read from the document and the shaped row is kept.
use crate::document::Document;
/// One shaped line.
use crate::shaped_row::ShapedRow;
/// Syntax classes by character offset.
use crate::source_style::StyleSpan;
/// The cache is a hash map.
use std::collections::HashMap;

/// What:
///  A 64-bit digest of bytes written in any number of pieces.
/// Why:
///  The key of a line holds the digest of its text and of its syntax classes instead of copies;
///  the digest is FNV-1a,
/// whose result does not depend on how a rope cut its text into chunks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Digest { bytes(piece: Uint8Array): void; number(value: number): void; finish(): bigint }
/// ```
struct Digest {
    /// The running hash.
    state: u64,
}

/// Feeding and reading a digest.
impl Digest {
    /// FNV-1a's offset basis.
    const BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    /// FNV-1a's prime.
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    /// An empty digest.
    fn new() -> Self {
        return Self { state: Self::BASIS };
    }

    /// Feed `piece`.
    fn bytes(&mut self, piece: &[u8]) {
        for byte in piece {
            self.state = (self.state ^ u64::from(*byte)).wrapping_mul(Self::PRIME);
        }
    }

    /// Feed `value` as eight bytes.
    fn number(&mut self, value: usize) {
        self.bytes(&(value as u64).to_le_bytes());
    }
}

/// What:
///  What shaped a line:
///  the digests of its characters and of its syntax classes,
///  the wrap width,
///  and the display scale.
/// Why:
///  Shaping reads nothing else of the document,
///  and the row's place is set again by whoever takes the row from the cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LineKey = { text: bigint; styles: bigint; wrap: number; scale: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LineKey {
    /// The digest of the line's characters.
    text: u64,
    /// The digest of the syntax classes that reach the line,
    /// clipped to it and measured from its first character.
    styles: u64,
    /// Exact bits of the wrap width,
    /// zero for none.
    wrap: u32,
    /// Exact bits of the display scale.
    scale: u32,
}

/// The digest of the syntax classes that reach the characters `start` to `end`,
/// each clipped to them and measured from `start`:
/// what the shaper makes of the classes for that line.
fn styles_digest(styles: &[StyleSpan], start: usize, end: usize) -> u64 {
    // Spans ascend and do not overlap, so a halving search finds the first one that reaches the line.
    let mut reaching = 0;
    let mut high = styles.len();
    while reaching < high {
        let middle = reaching + (high - reaching) / 2;
        if styles[middle].end <= start {
            reaching = middle + 1;
        } else {
            high = middle;
        }
    }
    let mut digest = Digest::new();
    for span in &styles[reaching..] {
        if span.start >= end {
            break;
        }
        digest.number(span.start.max(start) - start);
        digest.number(span.end.min(end) - start);
        digest.number(span.style);
    }
    return digest.state;
}

/// What:
///  The key of the shaped row of `line` of `document`,
///  broken at `wrap` and shaped at `scale` with `styles`.
/// Why:
///  Two lines with the same characters and classes shape to the same glyphs wherever they stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineKey(document: Document, styles: StyleSpan[], line: number, wrap: number | undefined, scale: number): LineKey;
/// ```
pub fn line_key(
    document: &Document,
    styles: &[StyleSpan],
    line: usize,
    wrap: Option<f32>,
    scale: f32,
) -> LineKey {
    let text = document.text();
    let start = text.line_to_char(line);
    let slice = text.line(line);
    let mut digest = Digest::new();
    for chunk in slice.chunks() {
        digest.bytes(chunk.as_bytes());
    }
    digest.number(slice.len_chars());
    return LineKey {
        text: digest.state,
        styles: styles_digest(styles, start, start + slice.len_chars()),
        wrap: wrap.map_or(0, f32::to_bits),
        scale: scale.to_bits(),
    };
}

/// A kept row and the build that last used it.
struct Kept {
    /// The number of the build that last took this row.
    used: u64,
    /// The shaped row.
    row: ShapedRow,
}

/// What:
///  The shaped rows of the lines the last builds drew,
///  by what shaped them.
/// Why:
///  Shaping is the expensive part of a slice;
///  a slide brings a few lines in,
///  an edit changes one,
///  and the others are taken from here.
///  When the cache holds more rows than its capacity,
///  a build ends by dropping those it did not use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LineCache { rows: Map<LineKey, ShapedRow>; capacity: number }
/// ```
pub struct LineCache {
    /// The kept rows.
    rows: HashMap<LineKey, Kept>,
    /// Most rows kept after a build.
    capacity: usize,
    /// The number of the build in progress.
    build: u64,
}

/// A cache that keeps a few thousand lines.
impl Default for LineCache {
    /// A cache of [`Self::CAPACITY`] rows.
    fn default() -> Self {
        return Self::with_capacity(Self::CAPACITY);
    }
}

/// Starting a build,
/// taking rows,
/// and keeping new ones.
impl LineCache {
    /// Rows kept after a build:
    /// several slices of a tall window,
    /// at the 3.2 KB a shaped line costs 13 MB at most.
    pub const CAPACITY: usize = 4096;

    /// A cache that keeps at most `capacity` rows after a build.
    pub fn with_capacity(capacity: usize) -> Self {
        return Self {
            rows: HashMap::new(),
            capacity: capacity.max(1),
            build: 0,
        };
    }

    /// Start a build:
    /// the rows it takes are marked as used by it.
    pub fn begin(&mut self) {
        self.build += 1;
    }

    /// The row kept for `key`,
    /// marked as used by this build.
    pub fn take(&mut self, key: LineKey) -> Option<ShapedRow> {
        let kept = self.rows.get_mut(&key)?;
        kept.used = self.build;
        return Some(kept.row.clone());
    }

    /// Keep `row` for `key`.
    pub fn put(&mut self, key: LineKey, row: ShapedRow) {
        self.rows.insert(
            key,
            Kept {
                used: self.build,
                row,
            },
        );
    }

    /// End a build:
    /// past the capacity,
    /// drop the rows this build did not use.
    pub fn finish(&mut self) {
        if self.rows.len() > self.capacity {
            let mut stale: Vec<LineKey> = Vec::new();
            for (key, kept) in &self.rows {
                if kept.used != self.build {
                    stale.push(*key);
                }
            }
            for key in stale {
                self.rows.remove(&key);
            }
        }
    }

    /// Drop every row,
    /// after a change of typography.
    pub fn clear(&mut self) {
        self.rows.clear();
    }

    /// How many rows are kept.
    pub fn len(&self) -> usize {
        return self.rows.len();
    }

    /// Whether no row is kept.
    pub fn is_empty(&self) -> bool {
        return self.rows.is_empty();
    }
}
