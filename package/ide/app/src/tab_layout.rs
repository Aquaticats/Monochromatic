//! Shape one projected line so that every tab ends on a pixel tab stop.
//!
//! The shaping engine has no tab stops.
//!  Each tab is shaped as one space,
//! and that space's advance is corrected through letter spacing on its single byte.
//! A line whose tabs follow other text is shaped twice:
//!  once to measure where each tab starts,
//! once with the corrections.
//!  A line whose tabs are all leading needs no measuring pass.

/// The shaper owns fonts and scratch space;
///  this module only decides tab corrections.
use crate::shaped_text::{LineRequest, TextShaper};
/// Pixel tab-stop arithmetic,
///  independent of fonts.
use crate::tab_stop::tab_spacings;
/// The projection marks which display bytes stand in for source tabs.
use crate::text_projection::Projection;
/// What:
///  Import the shaping engine's paragraph and caret types.
/// Why:
///  A caret's position before a tab is exactly where that tab starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Affinity, Cursor, Layout } from 'parley';
/// ```
use parley::{Affinity, Cursor, Layout};

/// Physical x of the boundary before display byte `byte`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function xBefore(layout: Layout, byte: number): number;
/// ```
fn x_before(layout: &Layout<u32>, byte: usize) -> f32 {
    let cursor = Cursor::from_byte_index(layout, byte, Affinity::Downstream);
    // What: `as f32` narrows the engine's 64-bit coordinate (sibling `f64`) to the advance unit.
    // Why: Tab-stop arithmetic works in the same 32-bit floats as glyph advances.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return cursor.geometry(layout, 1).x0;
    // ```
    return cursor.geometry(layout, 1.0).x0 as f32;
}

/// Shape `projection` at `scale` with syntax `roles`,
///  widening each tab to its stop.
///
/// Measurements come back in physical pixels while letter spacing is given in logical pixels,
/// so each correction is divided by `scale` before it is handed to the shaper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function layoutWithTabs(shaper: TextShaper, projection: Projection, scale: number,
///   roles: [number, number, number][]): Layout;
/// ```
pub(crate) fn layout_with_tabs(
    shaper: &mut TextShaper,
    projection: &Projection,
    scale: f32,
    roles: &[(usize, usize, u32)],
) -> Layout<u32> {
    if projection.tabs.is_empty() {
        return shaper.line_layout(&projection.text, scale, roles, &[]);
    }
    let space = shaper.space_advance(scale);
    // What: `Vec::new()` creates an empty growable list (siblings: fixed `[f32; N]`, borrowed `&[f32]`).
    // Why: The number of tabs differs per line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const naturals: number[] = [];
    // ```
    let mut naturals = Vec::new();
    // The n-th tab is leading exactly when n tabs, and nothing else, precede it.
    let mut leading = true;
    for (ordinal, byte) in projection.tabs.iter().enumerate() {
        if *byte != ordinal {
            leading = false;
        }
    }
    if leading {
        // Each stand-in space is one space advance wide, so leading tabs start at known positions.
        for ordinal in 0..projection.tabs.len() {
            naturals.push(ordinal as f32 * space);
        }
    } else {
        // The roles carry weights and the italic face, which can change an advance, so they are measured too.
        let measured = shaper.line_layout(&projection.text, scale, roles, &[]);
        for byte in &projection.tabs {
            naturals.push(x_before(&measured, *byte));
        }
    }
    let extras = tab_spacings(&naturals, space);
    let mut spacings = Vec::new();
    // What: `zip` pairs the n-th tab byte with the n-th correction.
    // Why: Both lists are in reading order and have one entry per tab.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // projection.tabs.forEach((byte, index) => spacings.push([byte, extras[index] / scale]));
    // ```
    for (byte, extra) in projection.tabs.iter().zip(extras) {
        spacings.push((*byte, extra / scale));
    }
    return shaper.line_layout(&projection.text, scale, roles, &spacings);
}

/// What:
///  Most passes of the wrapped tab correction of one line.
/// Why:
///  A correction moves the breaks of the rows after it,
/// and each pass fixes the corrections of the rows it measured
///  correctly:
/// row `n` is right after at most `n + 1` passes,
/// because a row depends only on the rows before it.
///  Measured on 2026-10-12,
/// lines of 20,000 to 100,000 characters settled after 4 to 76 passes,
/// never near their
///  255 to 1,265 rows,
/// but the count is not known beforehand and each pass costs one shaping of the whole line.
pub const TAB_PASSES: usize = 40;

/// What:
///  Most characters the corrective passes of one line may shape in all.
/// Why:
///  Every pass shapes the whole line,
/// so a line of a hundred thousand characters would cost forty shapings:
///  measured on 2026-10-12 in the release profile,
/// 2.3 to 4.8 s for such a line on a loaded host.
///  The passes allowed fall with the length of the line,
/// so the work stays near this many characters,
///  and the tabs on the first rows of a very long line are exact while those far down may stand off their stops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const TAB_BUDGET = 1_000_000;
/// ```
pub const TAB_BUDGET: usize = 1_000_000;

/// Fewest corrective passes any line gets,
/// however long it is.
pub const TAB_MIN_PASSES: usize = 4;

/// What:
///  How many corrective passes a line of `length` bytes gets:
/// [`TAB_PASSES`] for a line short enough,
///  otherwise the budget divided by its length,
/// and never fewer than [`TAB_MIN_PASSES`].
/// Why:
///  The bound on the work depends on the line,
/// not only on a count of passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function passesFor(length: number): number { return clamp(TAB_BUDGET / Math.max(length, 1), TAB_MIN_PASSES, TAB_PASSES); }
/// ```
pub fn passes_for(length: usize) -> usize {
    return (TAB_BUDGET / length.max(1)).clamp(TAB_MIN_PASSES, TAB_PASSES);
}

/// Two corrections closer than this in logical pixels are the same correction.
const SPACING_TOLERANCE: f32 = 0.001;

/// What:
///  Whether every tab (display bytes in `tabs`) lies on the first visual row of `layout`.
/// Why:
///  A tab on the first row starts at a place that does not depend on where the line breaks,
///  so its widening survives a new break;
///  a tab on a continuation row does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tabsOnFirstRow(layout: Layout, tabs: number[]): boolean;
/// ```
pub(crate) fn tabs_on_first_row(layout: &Layout<u32>, tabs: &[usize]) -> bool {
    let Some(first) = layout.lines().next() else {
        return true;
    };
    let end = first.text_range().end;
    for byte in tabs.iter() {
        if *byte >= end {
            return false;
        }
    }
    return true;
}

/// Index of the visual row that holds display byte `byte`,
/// a byte on a row boundary being on the lower row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rowOf(layout: Layout, byte: number): number;
/// ```
fn row_of(layout: &Layout<u32>, byte: usize) -> usize {
    // What: Row starts ascend, so the rows that start at or before the byte are a prefix; a binary search finds its end.
    // Why: A line of thousands of rows asks this once for every tab in every pass.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const count = partitionPoint(rows, (row) => row.start <= byte); return Math.max(count - 1, 0);
    // ```
    let (mut low, mut high) = (0, layout.len());
    while low < high {
        let middle = low + (high - low) / 2;
        let mut starts_by_byte = false;
        if let Some(line) = layout.get(middle) {
            starts_by_byte = line.text_range().start <= byte;
        }
        if starts_by_byte {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    return low.saturating_sub(1);
}

/// Extra advance for each tab of a wrapped layout,
/// in logical pixels,
/// so every tab ends on a stop.
///
/// Each tab's position is taken from `layout` with the corrections in `applied` removed from its row,
///  so the answer does not depend on which corrections were applied.
/// Tab stops restart on every visual row,
/// because a row starts at its own offset (the continuation indent).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wrappedSpacings(layout: Layout, tabs: number[], applied: Spacing[], space: number): Spacing[];
/// ```
fn wrapped_spacings(
    layout: &Layout<u32>,
    projection: &Projection,
    applied: &[(usize, f32)],
    scale: f32,
    space: f32,
) -> Vec<(usize, f32)> {
    let mut naturals = Vec::new();
    let mut rows = Vec::new();
    // Physical correction already applied to the tabs of the current row before this one.
    let mut applied_on_row = 0.0;
    for (index, byte) in projection.tabs.iter().enumerate() {
        let row = row_of(layout, *byte);
        if rows.last() != Some(&row) {
            applied_on_row = 0.0;
        }
        naturals.push(x_before(layout, *byte) - applied_on_row);
        rows.push(row);
        let extra = match applied.get(index) {
            Some((_, found)) => *found,
            None => 0.0,
        };
        applied_on_row += extra * scale;
    }
    let mut spacings = Vec::new();
    let mut start = 0;
    while start < naturals.len() {
        let mut end = start + 1;
        while end < naturals.len() && rows[end] == rows[start] {
            end += 1;
        }
        let extras = tab_spacings(&naturals[start..end], space);
        for (offset, extra) in extras.iter().enumerate() {
            spacings.push((projection.tabs[start + offset], *extra / scale));
        }
        start = end;
    }
    return spacings;
}

/// Whether two correction lists name the same tabs with the same extra advance,
/// within `SPACING_TOLERANCE`.
fn same_spacings(left: &[(usize, f32)], right: &[(usize, f32)]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (first, second) in left.iter().zip(right) {
        if !(first.0 == second.0 && (first.1 - second.1).abs() <= SPACING_TOLERANCE) {
            return false;
        }
    }
    return true;
}

/// Total extra advance of a correction list,
/// the measure by which the fitting of two lists is compared.
fn total_extra(spacings: &[(usize, f32)]) -> f32 {
    let mut total = 0.0;
    for (_, extra) in spacings.iter() {
        total += *extra;
    }
    return total;
}

/// What:
///  Shape `projection` broken at `width` with each tab widened to its stop on its own visual row.
///  The first layout has no corrections;
///  each later pass removes the corrections of the previous pass from every row,
///  recomputes the stops,
///  and stops when the corrections no longer change.
/// Why:
///  A correction can move a break,
/// so the passes repeat until the rows agree.
///  Row `n` depends only on rows before it,
/// so the passes settle within the number of rows plus one.
///  The one exception is a tab that ends its row:
/// widened to its stop it no longer fits and moves to the next row,
///  where the stop needs no widening,
/// and unwidened it fits again.
///  Such a pair of corrections repeats forever,
/// so a correction equal to the one two passes back ends the loop
///  with the layout that has the smaller corrections,
/// the one in which the tab fits.
///  At most `passes_for` passes run in any case,
/// so the result is always deterministic and bounded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wrappedLayoutWithTabs(shaper: TextShaper, projection: Projection, scale: number, width: number): Layout;
/// ```
pub(crate) fn layout_wrapped_with_tabs(
    shaper: &mut TextShaper,
    projection: &Projection,
    scale: f32,
    width: f32,
    roles: &[(usize, usize, u32)],
) -> Layout<u32> {
    let mut spacings: Vec<(usize, f32)> = Vec::new();
    let first_request = LineRequest {
        text: &projection.text,
        scale,
        roles,
        spacings: &spacings,
        wrap: Some(width),
    };
    let mut layout = shaper.wrapped_line_layout(first_request);
    if projection.tabs.is_empty() {
        return layout;
    }
    let space = shaper.space_advance(scale);
    // The corrections and the layout of the pass before the current one.
    let mut before: Option<(Vec<(usize, f32)>, Layout<u32>)> = None;
    for _ in 0..passes_for(projection.text.len()) {
        let next = wrapped_spacings(&layout, projection, &spacings, scale, space);
        if same_spacings(&next, &spacings) {
            break;
        }
        // A correction that repeats the one before the current pass is a cycle.
        if let Some((earlier, earlier_layout)) = before.take()
            && same_spacings(&next, &earlier)
        {
            if total_extra(&earlier) < total_extra(&spacings) {
                return earlier_layout;
            }
            return layout;
        }
        let corrected = LineRequest {
            text: &projection.text,
            scale,
            roles,
            spacings: &next,
            wrap: Some(width),
        };
        shaper.count_tab_pass();
        let relaid = shaper.wrapped_line_layout(corrected);
        before = Some((spacings, std::mem::replace(&mut layout, relaid)));
        spacings = next;
    }
    return layout;
}

/// Unit tests of the work bound of the tab correction live in a sidecar file.
#[cfg(test)]
#[path = "tab_layout_tests.rs"]
mod tests;
