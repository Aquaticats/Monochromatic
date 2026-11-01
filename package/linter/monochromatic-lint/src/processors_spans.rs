//! What:
//!  Original-host byte/line/column resolution through immutable map layers.
//! Why:
//!  Synthetic text has no reporting address,
//!  while nested comments preserve authored positions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Intersect an authored span at each layer, then resolve the physical host's positions.
//! ```

/// Import common source coordinates and processor models.
use crate::diagnostic::Span;
/// Use the established Markdown UTF-16 position convention.
use crate::markdown_positions::MarkdownPositions;
/// Borrow immutable layer and language records.
use crate::processors_model::{Mapping, ProcessorLanguage};
/// Resolve Rust host positions using its existing byte-column convention.
use crate::rust_source::RustSource;

/// Map one range to its authored intersection in the direct parent.
pub(crate) fn direct(mapping: &Mapping, start: usize, end: usize) -> Option<(usize, usize)> {
    if mapping.text.is_empty() && mapping.lines.is_empty() && start == 0 && end == 0 {
        // An empty authored container has an anchor, not synthetic bytes to suppress.
        return Some((mapping.anchor, mapping.anchor));
    }
    // Keep synthetic-only spans absent instead of pinning them to an unrelated source byte.
    let mut first: Option<usize> = None;
    let mut last: usize = 0;
    for line in &mapping.lines {
        if start == end {
            if line.start <= start && start < line.end
                || start == mapping.text.len() && line.end == start
            {
                let point: usize = line.parent_start + start - line.start;
                return Some((point, point));
            }
        } else if line.start < end && line.end > start {
            let mapped_start: usize = line.parent_start + start.max(line.start) - line.start;
            last = line.parent_start + end.min(line.end) - line.start;
            if first.is_none() {
                first = Some(mapped_start);
            }
        }
    }
    // Return a present authored intersection, otherwise preserve explicit absence for synthetic text.
    if let Some(offset) = first {
        return Some((offset, last));
    }
    return None;
}

/// Resolve a nested range to the original host;
///  malformed spans fail before slicing.
pub(crate) fn host_range(mapping: &Mapping, start: usize, end: usize) -> Option<(usize, usize)> {
    if start > end
        || end > mapping.text.len()
        || !mapping.text.is_char_boundary(start)
        || !mapping.text.is_char_boundary(end)
    {
        return None;
    }
    let mut current: &Mapping = mapping;
    let mut range: (usize, usize) = (start, end);
    while let Some(parent) = &current.parent {
        range = direct(current, range.0, range.1)?;
        current = parent;
    }
    return Some(range);
}

/// Resolve an extraction failure or empty snippet to its authored container.
pub(crate) fn anchor(mapping: &Mapping) -> usize {
    if let Some(parent) = &mapping.parent {
        // Child anchors use direct-parent coordinates, not prepared-source offsets.
        if let Some((offset, _)) = host_range(parent, mapping.anchor, mapping.anchor) {
            return offset;
        }
        return anchor(parent);
    }
    return mapping.anchor.min(mapping.text.len());
}

/// Resolve line and column units from the original host's language.
pub(crate) fn host_span(root: &Mapping, start: usize, length: usize) -> Span {
    if root.language == ProcessorLanguage::Rust {
        let parsed: RustSource = RustSource::new(root.filename.clone(), root.text.clone());
        let mut span: Span = parsed.span(start, length);
        // A mapped range remains exact even when it crosses multiple original physical lines.
        span.length = length;
        return span;
    }
    return MarkdownPositions::new(root.text.as_str()).span(start, length);
}
