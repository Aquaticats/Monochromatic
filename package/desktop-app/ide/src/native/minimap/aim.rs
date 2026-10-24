//! A press on the minimap,
//! mapped through the layout the window has when it lands,
//! and aimed again once the
//!  layout changes while nothing else has moved the view.
//!
//! A press means "the place the minimap shows here".
//!  The window knows that place only through its layout,
//! and the layout is not final for a while after the
//!  press:
//! the hints of lines that were never asked about arrive later and take rows.
//!  The press is therefore kept as the pointer's place on the track (layout-independent) together with what it
//!  left behind (the scroll offset,
//! the caret,
//! the text,
//! the sizes),
//!  and a layout change that finds all of that still in place maps the same place again through the new layout.

/// The pointer's place under the caret hit test.
use crate::native::input::pointer_position;
/// The state the press reads and changes,
/// and the stamp of the text it was mapped on.
use crate::native::{State, annotate::displayed, rows};
/// Reading position sides.
use ide_app::document::ReadingAffinity;
/// The stamp naming the displayed text.
use ide_app::language::identity::DocumentStamp;
/// The geometry of the minimap and the map of a position across it to the text's width.
use ide_app::minimap::{Geometry, Target, text_span, text_x};
/// One code row,
/// the unit of the vertical mapping.
use ide_app::row_map::CODE_ROW;
/// The work a line costs to shape,
/// which bounds a press on a very long line.
use ide_app::shaped_text::line_work;
/// A wrap width must be a positive number of pixels.
use ide_app::wrap::has_wrap_width;

/// What:
///  Where a press stood on the minimap and whether Shift was held:
/// everything about the press that does not depend on the layout.
/// Why:
///  The same press is mapped again through a later layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Press = { y: number; x: number; shift: boolean };
/// ```
#[derive(Clone, Copy)]
pub(in crate::native) struct Press {
    /// Distance from the top of the minimap,
    /// in logical pixels.
    pub(in crate::native) y: f32,
    /// Distance from the left edge of the minimap,
    /// in logical pixels.
    pub(in crate::native) x: f32,
    /// Whether the press extends the selection from the old anchor.
    pub(in crate::native) shift: bool,
}

/// What:
///  What a press left behind,
/// which stays equal for as long as nothing else has touched the view:
///  the offset it scrolled to,
///  the caret it chose,
///  the text it was mapped on,
///  and the sizes of the window and the minimap.
/// Why:
///  A layout change aims the press again only when every one of them still holds,
///  which means the reader has done nothing since.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Left = { offset: number; caret: [number, number, Affinity]; text: DocumentStamp; view: [number, number]; minimap: [number, number] };
/// ```
#[derive(Clone, Copy)]
struct Left {
    /// The scroll offset the press,
    /// or the last aim,
    /// took.
    offset: f32,
    /// Anchor,
    /// head,
    /// and side of the selection the press,
    /// or the last aim,
    /// chose.
    caret: (usize, usize, ReadingAffinity),
    /// File generation and content revision of the text.
    text: DocumentStamp,
    /// Width and height of the view.
    view: (f32, f32),
    /// Width and height of the minimap column.
    minimap: (f32, f32),
}

/// What:
///  The last press on the minimap that nothing has followed:
/// the press itself,
///  the share of the scroll range the view stood at before it,
///  and what it left.
/// Why:
///  The share of the range,
/// not the offset,
/// is what the picture's slide depended on,
/// so the same press maps
///  through a layout of another height as it would have if that layout had been there at the press.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Aim = { press: Press; before: number; left: Left };
/// ```
#[derive(Clone, Copy)]
pub(in crate::native) struct Aim {
    /// The press.
    press: Press,
    /// The scroll offset before the press,
    /// as a share of the scroll range then.
    before: f32,
    /// What the press left.
    left: Left,
}

/// What:
///  Take in what the press at `offset` left behind,
/// to be aimed again if the layout changes.
/// Why:
///  Called right after the press scrolled and chose its caret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function remember(state: State, press: Press, share: number, placed: [offset: number, view: [number, number]]): void;
/// ```
pub(in crate::native) fn remember(
    current: &mut State,
    press: Press,
    share: f32,
    placed: (f32, (f32, f32)),
) {
    let (offset, view) = placed;
    current.minimap.aim = Some(Aim {
        press,
        before: share,
        left: left_by(current, offset, view),
    });
}

/// What:
///  The share of the scroll range that `offset` is in `geometry`,
/// from zero to one;
/// zero when nothing scrolls.
/// Why:
///  The picture slides by that share of its overhang,
/// so it is what a press depended on besides its own place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function share(geometry: Geometry, offset: number): number;
/// ```
pub(in crate::native) fn share(geometry: &Geometry, offset: f32) -> f32 {
    let range = geometry.max_offset();
    if range <= 0.0 {
        return 0.0;
    }
    return (offset / range).clamp(0.0, 1.0);
}

/// What the state holds now,
/// given the scroll `offset` and the sizes of the `view`.
fn left_by(current: &State, offset: f32, view: (f32, f32)) -> Left {
    let position = current.document.position();
    return Left {
        offset,
        caret: (position.anchor, position.head, position.affinity),
        text: displayed(current),
        view,
        minimap: current.minimap.size,
    };
}

/// What:
///  Forget the press when the view is placed anywhere but where the press put it.
/// Why:
///  Every scroll the window itself makes,
/// and every scroll the reader makes,
/// passes through the view's one
///  placement:
/// a wheel step,
/// a key,
/// a drag,
/// a find jump,
/// a tree click.
///  After any of them the press has been followed by scrolling and is never aimed again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forgetUnless(state: State, offset: number): void;
/// ```
pub(in crate::native) fn forget_unless(current: &mut State, offset: f32) {
    if let Some(aim) = current.minimap.aim
        && aim.left.offset != offset
    {
        current.minimap.aim = None;
    }
}

/// What:
///  The scroll offset a press at `before` (the offset the view stood at) gives in the layout the state holds,
///  and the visual row it names;
/// the caret goes to the pressed place,
/// with the old anchor kept for a Shift press.
/// Why:
///  CodeGlance Pro's defaults (`jumpOnMouseDown = MOUSE_DOWN`,
/// `clickType = CODE_POSITION`,
/// `moveOnly = false`):
///  the caret goes to the clicked position,
/// which is then centered,
/// and a Shift press selects up to it.
///  A press and an aim of it are one mapping,
/// so they cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function map(state: State, press: Press, before: number, scale: number, height: number): [Target, number];
/// ```
pub(in crate::native) fn map(
    current: &mut State,
    press: Press,
    before: f32,
    scale: f32,
    height: f32,
) -> (Target, f32) {
    let geometry = Geometry::new(current.row_map.height(), height);
    let target = geometry.target(&current.row_map, press.y, before);
    let top = current.row_map.code_top(target.line) + target.row as f32 * CODE_ROW;
    // The picture is the text's whole width shrunk to the minimap's, so a position across it is the same share of
    // the text's width.
    let across = text_x(
        press.x,
        current.minimap.size.0,
        text_span(has_wrap_width(current.width).then_some(current.width)),
    );
    // A line too long to shape within a press's allowance puts the caret at its start.
    let (hit, side) = if line_work(&current.document, target.line) == usize::MAX {
        (
            current.document.text().line_to_char(target.line),
            ReadingAffinity::Downstream,
        )
    } else {
        pointer_position(current, target.line, (across, top + CODE_ROW / 2.0), scale)
    };
    let mut position = current.document.position();
    position.head = hit;
    if !press.shift {
        position.anchor = hit;
    }
    position.affinity = side;
    current.document.select(position);
    let offset = geometry
        .centered(top + CODE_ROW / 2.0)
        .clamp(0.0, rows::limit(&current.row_map, height))
        .round();
    return (target, offset);
}

/// What:
///  The scroll offset the remembered press gives in the layout the state holds now,
///  or nothing when the press has been followed by anything:
///  another scroll,
/// a caret move,
/// an edit or another file,
/// a resize.
///  The caret goes to the place the press names now,
/// and what the press left is updated to match,
///  so a later layout change can aim it once more.
///  A drag that the press started and the pointer has not yet moved follows the new offset.
/// Why:
///  The hints of the first answer after a press on a far place are in the layout when the press is mapped,
/// and
///  those of the next answer are not:
/// the press lands where the incomplete layout puts it,
///  and the same press lands elsewhere a moment later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aimAgain(state: State, scale: number, view: [offset: number, width: number, height: number]): number | undefined;
/// ```
pub(in crate::native) fn aim_again(
    current: &mut State,
    scale: f32,
    view: (f32, f32, f32),
) -> Option<f32> {
    let (offset, width, height) = view;
    let aim = current.minimap.aim?;
    let now = left_by(current, offset, (width, height));
    if now.offset != aim.left.offset
        || now.caret != aim.left.caret
        || now.text != aim.left.text
        || now.view != aim.left.view
        || now.minimap != aim.left.minimap
    {
        current.minimap.aim = None;
        return None;
    }
    let range = Geometry::new(current.row_map.height(), height).max_offset();
    let (target, wanted) = map(current, aim.press, aim.before * range, scale, height);
    current.minimap.aim = Some(Aim {
        left: left_by(current, wanted, (width, height)),
        ..aim
    });
    // The box follows the pointer from the place the press left it.
    if let Some(drag) = current.minimap.drag.as_mut() {
        drag.start_offset = wanted;
    }
    tracing::debug!(
        line = target.line,
        row = target.row,
        from = offset,
        to = wanted,
        "minimap press aimed again at the new layout"
    );
    return Some(wanted);
}
