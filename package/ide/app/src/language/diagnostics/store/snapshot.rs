//! The latest-value snapshot of the displayed text:
//!  the sets stamped with its revision,
//!  grouped by source,
//!  and ordered so an unchanged state stays equal.

/// The snapshot it builds,
///  the diagnostic records it groups,
///  and the conversion that drops one outside the text.
use super::super::{
    Diagnostic,
    DiagnosticsSnapshot,
    SourceGroup,
    add_to_group,
    convert,
};
/// The store this reads.
use super::DiagnosticStore;
/// Conversion happens against the rope of the displayed revision.
use helix_core::Rope;

/// Orders source groups by their source text.
fn by_source(
    left: &SourceGroup,
    right: &SourceGroup,
) -> std::cmp::Ordering {
    return left.source.cmp(&right.source);
}

/// Orders diagnostics by their start,
///  then their end.
fn by_range(
    left: &Diagnostic,
    right: &Diagnostic,
) -> std::cmp::Ordering {
    return (left.start, left.end).cmp(&(right.start, right.end));
}

/// Building the latest-value snapshot.
impl DiagnosticStore {
    /// What:
    ///  Build the latest-value snapshot against the displayed text,
    ///  or nothing when no file
    ///       is displayed.
    ///  Only sets stamped with the displayed revision take part.
    /// Why:
    ///  Sets of an earlier revision are removed from display,
    ///  never carried forward.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshot(text: Rope): DiagnosticsSnapshot | undefined
    /// ```
    pub(crate) fn snapshot(
        &self,
        text: &Rope,
    ) -> Option<DiagnosticsSnapshot> {
        // The trailing `?` returns `None` when no document is tracked.
        let tracked = self.document.as_ref()?;
        let mut groups: Vec<SourceGroup> = Vec::new();
        for set in &self.sets {
            if set.stamp != tracked.stamp {
                continue;
            }
            for raw in &set.items {
                // `if let Some(x) = ...` runs the block only when conversion produced a value.
                if let Some(item) = convert(raw, text, set.encoding, &set.server, set.freshness) {
                    // What: `as_deref()` turns `Option<String>` into `Option<&str>`;
                    //       `unwrap_or("")` substitutes empty text for "nothing".
                    // Why: A server that names no source still gets one group.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // addToGroup(groups, raw.source ?? '', item);
                    // ```
                    add_to_group(&mut groups, raw.source.as_deref().unwrap_or(""), item);
                } else {
                    tracing::debug!(server = %set.server.name, "dropped a diagnostic outside the displayed text");
                }
            }
        }
        // What: `sort_by` orders in place; `cmp` compares two values and returns their ordering.
        // Why: A stable order keeps equal snapshots equal, so unchanged state is not republished.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // groups.sort((left, right) => left.source.localeCompare(right.source));
        // ```
        groups.sort_by(by_source);
        for group in groups.iter_mut() {
            group.items.sort_by(by_range);
        }
        return Some(DiagnosticsSnapshot {
            stamp: tracked.stamp,
            groups,
        });
    }
}
