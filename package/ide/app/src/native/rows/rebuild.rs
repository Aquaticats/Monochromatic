//! What the blocks and the map are made of:
//!  the key they change with,
//!  the steps that rebuild them from the displayed text and its annotations,
//!  and the installation of the result.

/// The source state.
use crate::native::State;
/// The stamp and the blocks the displayed text was last built for.
use crate::native::annotate::displayed;
/// Measured visual rows of every line.
use ide_app::line_rows::LineRows;
/// The mapping itself.
use ide_app::row_map::RowMap;
/// A wrap width must be a positive number of pixels.
use ide_app::wrap::has_wrap_width;
/// The stamp naming a displayed text,
///  and one line's block of virtual rows.
use ide_app::{
    language::identity::DocumentStamp,
    virtual_row::Block,
};
/// Line ranges,
///  and the shared pointer that keeps the blocks alive.
use std::{
    ops::Range,
    sync::Arc,
};

/// What:
///  What the map key is made of:
///  the displayed text's stamp,
///  the annotation store's change counter,
///  the display scale as exact bits,
///  the counter of changes to the measured visual rows,
///  and whether a line near the view still holds blocks made for another wrap width;
///  `type` names the tuple once.
/// Why:
///  Blocks are assembled again only when the stamp,
/// the version,
/// the scale,
/// or the staleness changed:
///  a hint stands where its character is drawn and a message breaks to the text area,
///  so a new width matters exactly when a line near the view holds blocks made for another one;
///  the map is rebuilt when any of the five did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RowKey = [stamp: DocumentStamp, version: number, scale: number, measured: number, stale: boolean];
/// ```
pub(in crate::native) type RowKey = (DocumentStamp, u64, u32, u64, bool);

/// What:
///  A freshly assembled vertical layout that is not installed yet:
///  its key,
///  the blocks of every annotated
///       line,
///  and the map they call for.
/// Why:
///  Whether the view can take it now is decided before anything in the state changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rebuilt = { key: RowKey; blocks: Block[]; map: RowMap };
/// ```
pub(super) struct Rebuilt {
    /// What the layout was built from.
    pub(super) key: RowKey,
    /// Blocks of virtual rows,
    ///  in line order.
    pub(super) blocks: Vec<Arc<Block>>,
    /// The mapping the blocks call for.
    pub(super) map: RowMap,
}

/// Make the measured visual rows describe the displayed text:
///  a table made for another text starts again with one row for every line.
fn ensure_wrap_rows(current: &mut State) {
    let stamp = displayed(current);
    let lines = current.document.text().len_lines();
    if current.wrap_text == Some(stamp) && current.wrap_rows.lines() == lines.max(1) {
        return;
    }
    // A reload or another file renumbers the lines, so what was measured for the old text does not carry over.
    current.wrap_rows = LineRows::plain(lines);
    current.wrap_text = Some(stamp);
    current.wrap_version += 1;
    tracing::debug!(lines, "measured rows started again for a new text");
}

/// What:
///  Record the exact visual rows of the given `(line, rows)` pairs in the table of measured rows,
///  and answer whether any count changed.
/// Why:
///  A tile that shaped its lines,
/// a frame built on the window thread,
/// and the background pass all learn
///  heights the vertical mapping must follow;
///  the next `settle` installs them and keeps the view still.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function absorb(current: State, measured: Iterable<[line: number, rows: number]>): boolean;
/// ```
pub(in crate::native) fn absorb(
    current: &mut State,
    measured: impl IntoIterator<Item = (usize, usize)>,
) -> bool {
    ensure_wrap_rows(current);
    let changed = current.wrap_rows.apply(measured);
    if changed {
        current.wrap_version += 1;
    }
    return changed;
}

/// The width of the text area blocks are made for,
/// or nothing while the view has none.
fn wrap_of(current: &State) -> Option<f32> {
    return has_wrap_width(current.width).then_some(current.width);
}

/// What:
///  The lines whose blocks are made exact for the wrap width:
/// the requested tile and one tile's worth of lines
///  on each side.
/// Why:
///  A new width packs the lines that have hints again,
/// which shapes them on the window thread;
///  only the lines near the view need it at once,
/// and the rest wait until the view comes near.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function focusOf(current: State): Range;
/// ```
fn focus_of(current: &State) -> Range<usize> {
    let span = current.count.max(1);
    return current.first.saturating_sub(span)..current.first.saturating_add(2 * span);
}

/// The key the displayed text,
///  its annotations,
///  the display `scale`,
///  and the measured rows call for now.
pub(super) fn wanted_key(
    current: &State,
    scale: f32,
) -> RowKey {
    // `to_bits` turns the float into an integer that compares exactly.
    return (
        displayed(current),
        current.annotations.version(),
        scale.to_bits(),
        current.wrap_version,
        current
            .annotations
            .stale_in(wrap_of(current), &focus_of(current)),
    );
}

/// What:
///  Assemble the blocks and the map the displayed text,
///  its annotations,
///  the display `scale`,
///  and the measured visual rows call for,
///       or nothing when the state's layout was built from exactly these;
///  `Option<Rebuilt>` is that result.
/// Why:
///  Blocks are assembled only when the text,
///  the annotation store,
///  the scale,
///  or the wrap width changed;
///  a change of the measured rows keeps the blocks and only rebuilds the map,
///  so the pass over the whole text does not pack hints again.
///  A new wrap width packs the lines that have hints again and breaks only the message rows wider than it.
///  Nothing in the state changes here except the store's packing cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rebuilt(current: State, scale: number): Rebuilt | undefined;
/// ```
pub(super) fn rebuilt(
    current: &mut State,
    scale: f32,
) -> Option<Rebuilt> {
    ensure_wrap_rows(current);
    let key = wanted_key(current, scale);
    if current.row_key == Some(key) {
        // `None`: nothing a block depends on changed.
        return None;
    }
    let blocks_current = match current.row_key {
        Some(known) => (known.0, known.1, known.2, known.4) == (key.0, key.1, key.2, key.4),
        None => false,
    };
    // What: Destructuring `&mut State` lends fields separately.
    // Why: Assembly changes the store's packing cache and uses the shaper while the document is only read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const { document, shaper, annotations } = current;
    // ```
    let wrap = wrap_of(current);
    let focus = focus_of(current);
    let State {
        document,
        shaper,
        annotations,
        blocks: kept,
        wrap_rows,
        ..
    } = &mut *current;
    let blocks = if blocks_current {
        // `clone` copies the list of shared block pointers, not the blocks.
        kept.clone()
    } else {
        annotations.assemble(key.0, document, shaper, scale, wrap, &focus)
    };
    let mut raised = Vec::new();
    for block in &blocks {
        raised.push((block.line, block.height()));
    }
    let map = RowMap::new(document.text().len_lines(), &raised).with_rows(wrap_rows);
    // `Some(...)` hands the assembled layout to the caller, which decides when to install it.
    // The lines near the view are exact now, so the key records the state after the assembly.
    let assembled = (
        key.0,
        key.1,
        key.2,
        key.3,
        annotations.stale_in(wrap, &focus),
    );
    return Some(Rebuilt {
        key: assembled,
        blocks,
        map,
    });
}

/// What:
///  Make `layout` the state's layout and answer the previous map when positions changed;
///       `Option<RowMap>` is that map or nothing.
///  `Rebuilt` is moved in.
/// Why:
///  The caller compares positions in both maps to keep the view still.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function install(current: State, layout: Rebuilt): RowMap | undefined;
/// ```
pub(super) fn install(
    current: &mut State,
    layout: Rebuilt,
) -> Option<RowMap> {
    current.blocks = layout.blocks;
    current.row_key = Some(layout.key);
    if layout.map == current.row_map {
        return None;
    }
    // What: `std::mem::replace` stores the new map and hands back the old one.
    // Why: The old positions are needed once more, to keep the view still.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const previous = current.rowMap; current.rowMap = layout.map; return previous;
    // ```
    let previous = std::mem::replace(&mut current.row_map, layout.map);
    tracing::debug!(
        lines = current.row_map.lines(),
        blocks = current.blocks.len(),
        height = current.row_map.height(),
        "row map rebuilt"
    );
    return Some(previous);
}
