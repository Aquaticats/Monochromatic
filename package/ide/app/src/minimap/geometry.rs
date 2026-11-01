//! The minimap's geometry:
//! where the picture,
//! the viewport box,
//! and a pointer position are,
//!  as pure functions of the text's height,
//! the view's height,
//! and the scroll offset.

/// How many bands cover the text,
/// and which way the view is heading,
/// which names the bands beyond the track.
use super::plan::{Heading, band_count};
/// The vertical mapping the minimap's rows come from and go back to.
use crate::row_map::{CODE_ROW, RowMap};
/// The bands to keep are a range of numbers.
use std::ops::Range;

/// What:
///  Logical width of the minimap at full size,
/// in pixels,
/// one pixel for each source column.
/// Why:
///  CodeGlance Pro's `CodeGlanceConfig.mainWidth` default is 110,
/// and the picture shows no column past it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const WIDTH = 110;
/// ```
pub const WIDTH: f32 = 110.0;

/// What:
///  Narrowest logical width a narrow view shrinks the minimap to.
/// Why:
///  CodeGlance Pro's `GlancePanel.getLogicalWidth` never goes below 15 when it shrinks the minimap beside a split.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const NARROWEST = 15;
/// ```
pub const NARROWEST: f32 = 15.0;

/// What:
///  The view's width divided by this is the widest the minimap gets in a narrow view.
/// Why:
///  CodeGlance Pro takes one twelfth of the editor beside a split;
/// the source view's sidebar split is the
///  nearest equivalent,
/// and one sixth keeps the full width from a view 660 pixels wide on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const VIEW_SHARE = 6;
/// ```
pub const VIEW_SHARE: f32 = 6.0;

/// What:
///  Logical pixels of the picture for each source row.
/// Why:
///  CodeGlance Pro's `pixelsPerLine` default is 4;
/// the picture keeps that scale however long the text is,
///  and a text taller than the view slides past the track with the scroll
///  (CodeGlance Pro's proportional size mode),
///  so a row of the picture is always one of the window's own rows averaged and never a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const PIXELS_PER_ROW = 4;
/// ```
pub const PIXELS_PER_ROW: f32 = 4.0;

/// What:
///  Shortest the viewport box is drawn,
/// in logical pixels.
/// Why:
///  CodeGlance Pro's `MIN_VIEWPORT_HEIGHT` is 20 (below it the box has square corners);
/// it sets no floor,
///  so a box over a 50,000 line text would be a fraction of a pixel and disappear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MIN_BOX = 20;
/// ```
pub const MIN_BOX: f32 = 20.0;

/// What:
///  The width of the minimap for a view `view_width` logical pixels wide:
///  [`WIDTH`],
/// or a sixth of the view when that is narrower,
/// and never under [`NARROWEST`].
/// Why:
///  The text area is what is left of the view,
/// so a narrow window must not lose its text to the minimap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function widthFor(viewWidth: number): number;
/// ```
pub fn width_for(view_width: f32) -> f32 {
    return (view_width / VIEW_SHARE).clamp(NARROWEST, WIDTH).round();
}

/// What:
///  One visual row of the text:
/// its source line and its place among that line's rows.
/// Why:
///  A click on the minimap and the picture's rows both name rows this way,
/// as the vertical mapping does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Target = { line: number; row: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    /// Zero-based source line.
    pub line: usize,
    /// Zero-based visual row within that line.
    pub row: usize,
}

/// What:
///  The minimap column for a text `content` logical pixels tall in a view `view` pixels tall:
///  the scale of its picture,
/// the track the picture shows,
/// how far the picture has slid past the track,
/// the viewport box,
/// and the maps between a minimap position,
/// a scroll offset,
/// and a visual row.
/// Why:
///  The Slint markup moves the box from the same numbers,
/// and the native code turns a click or a drag back into a
///  scroll offset with them;
/// one type holds both directions so a test can hold them against each other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Geometry { constructor(content: number, view: number); scale(): number; track(): number; /* ... */ }
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    /// Height of the whole text with its virtual rows,
    /// in logical pixels;
    /// never negative.
    content: f32,
    /// Height of the view,
    /// and of the minimap beside it,
    /// in logical pixels;
    /// never negative.
    view: f32,
}

/// The sizes and positions of one minimap column.
impl Geometry {
    /// The geometry of a text `content` pixels tall in a view `view` pixels tall;
    /// negative inputs count as zero.
    pub fn new(content: f32, view: f32) -> Self {
        return Self {
            content: content.max(0.0),
            view: view.max(0.0),
        };
    }

    /// Logical pixels of the picture for each pixel of the text:
    ///  [`PIXELS_PER_ROW`] over one code row,
    /// whatever the text's length.
    pub fn scale(&self) -> f32 {
        return PIXELS_PER_ROW / CODE_ROW;
    }

    /// Height of the track the picture is seen through:
    /// the whole text at [`Self::scale`] while that fits the view,
    /// otherwise the view.
    pub fn track(&self) -> f32 {
        return (self.content * self.scale()).min(self.view);
    }

    /// How far the picture has slid up past the track,
    /// in logical pixels,
    /// for a view scrolled to `offset`:
    ///  the picture's overhang past the track,
    /// by the offset's share of the scroll range.
    ///  Zero for a text that fits.
    pub fn slide(&self, offset: f32) -> f32 {
        let range = self.max_offset();
        if range <= 0.0 {
            return 0.0;
        }
        let overhang = (self.content * self.scale() - self.track()).max(0.0);
        return overhang * (offset / range).clamp(0.0, 1.0);
    }

    /// Height of the viewport box:
    /// the view at [`Self::scale`],
    /// at least [`MIN_BOX`],
    /// and never more than the track.
    pub fn box_height(&self) -> f32 {
        return (self.view * self.scale()).max(MIN_BOX).min(self.track());
    }

    /// How far the box can travel down the track:
    /// the track less the box.
    pub fn room(&self) -> f32 {
        return (self.track() - self.box_height()).max(0.0);
    }

    /// The largest scroll offset:
    /// the text less the view,
    /// zero for a text that fits.
    pub fn max_offset(&self) -> f32 {
        return (self.content - self.view).max(0.0);
    }

    /// Top of the viewport box for a view scrolled to `offset`:
    ///  the offset's share of the scroll range,
    /// as a share of the box's room.
    ///  The first offset puts the box at the top and the last at the bottom of the track.
    pub fn box_top(&self, offset: f32) -> f32 {
        let range = self.max_offset();
        if range <= 0.0 {
            return 0.0;
        }
        return self.room() * (offset / range).clamp(0.0, 1.0);
    }

    /// The text position,
    /// in logical pixels from its top,
    /// the minimap position `y` shows with the view scrolled to `offset`.
    ///  A position above or below the track is clamped to its ends.
    pub fn content_y(&self, y: f32, offset: f32) -> f32 {
        let track = self.track();
        if track <= 0.0 {
            return 0.0;
        }
        return ((y.clamp(0.0, track) + self.slide(offset)) / self.scale()).min(self.content);
    }

    /// The minimap position of the text position `content_y` with the view scrolled to `offset`,
    /// held to the track.
    pub fn y_of(&self, content_y: f32, offset: f32) -> f32 {
        let y = content_y.clamp(0.0, self.content) * self.scale() - self.slide(offset);
        return y.clamp(0.0, self.track());
    }

    /// What:
    ///  The scroll offset a drag of the viewport box gives:
    ///  the box was grabbed at `start_y` with the view at `start_offset`,
    /// and the pointer is at `y`.
    ///  The box follows the pointer one to one,
    /// so the offset moves by the pointer's distance over the box's room,
    ///  times the scroll range.
    /// Why:
    ///  CodeGlance Pro's `ScrollBar.mouseDragged` scales the drag by `maxViewportStart / movableHeight`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// dragOffset(startOffset: number, startY: number, y: number): number;
    /// ```
    pub fn drag_offset(&self, start_offset: f32, start_y: f32, y: f32) -> f32 {
        let room = self.room();
        let range = self.max_offset();
        if room <= 0.0 {
            return start_offset.clamp(0.0, range);
        }
        return (start_offset + (y - start_y) * range / room).clamp(0.0, range);
    }

    /// What:
    ///  The scroll offset that puts the text position `content_y` in the middle of the view,
    ///  held to the scroll range.
    /// Why:
    ///  CodeGlance Pro's click scrolls its target to the center (`ScrollType.CENTER`),
    /// also near the ends,
    ///  where the range stops the view short of the middle.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// centered(contentY: number): number;
    /// ```
    pub fn centered(&self, content_y: f32) -> f32 {
        return (content_y - self.view / 2.0).clamp(0.0, self.max_offset());
    }

    /// The visual row of `map` the minimap position `y` shows with the view scrolled to `offset`.
    ///  A position in the virtual rows above a line shows that line's first row.
    pub fn target(&self, map: &RowMap, y: f32, offset: f32) -> Target {
        let content_y = self.content_y(y, offset);
        let place = map.locate(content_y);
        let row = if place.in_block {
            0
        } else {
            map.visual_row_at(place.line, content_y)
        };
        return Target {
            line: place.line,
            row,
        };
    }

    /// The minimap position of the top of `target`'s visual row with the view scrolled to `offset`:
    /// the inverse of [`Self::target`] up to the pixel.
    pub fn y_of_row(&self, map: &RowMap, target: Target, offset: f32) -> f32 {
        let line = target.line.min(map.lines() - 1);
        let row = target.row.min(map.rows_of(line) - 1);
        return self.y_of(map.code_top(line) + row as f32 * CODE_ROW, offset);
    }

    /// What:
    ///  The bands of the picture to keep with the view scrolled to `offset` and at rest,
    /// as a range of band numbers:
    ///  the bands `step` logical pixels tall,
    /// counted from the top of the text,
    /// that the track shows,
    ///  and one more on each side.
    ///  A view that is heading somewhere keeps more on that side (`Self::bands_toward`).
    /// Why:
    ///  Only the bands near the visible part of the picture are drawn;
    ///  the extra band on each side lets a scroll of up to a band move the picture without a new one.
    ///  A picture of more bands than one texture holds is drawn as several textures that stack,
    ///  so the window passes no cap.
    ///  A caller that can show only `most` bands gets the bands from the one the track starts in,
    ///  and the rest of the track is left without a picture.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bandsFor(offset: number, step: number, most: number): [number, number];
    /// ```
    pub fn bands_for(&self, offset: f32, step: f32, most: usize) -> Range<usize> {
        return self.bands_toward(offset, step, most, Heading::Still { at: 0.0 });
    }

    /// What:
    ///  The text position of the top of the part of the picture the track shows,
    /// in logical pixels of the text,
    /// with the view scrolled to `offset`.
    /// Why:
    ///  The bands and the heading are counted along the text,
    /// and the track's top is where a scroll moves them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trackTop(offset: number): number;
    /// ```
    pub fn track_top(&self, offset: f32) -> f32 {
        return self.slide(offset) / self.scale();
    }

    /// What:
    ///  The bands of the picture to keep with the view scrolled to `offset` and heading as `heading` says,
    /// as a range of band numbers:
    ///  the bands the track shows,
    ///  and as many more beyond each end as the heading names (`Heading::margins`):
    ///  one on each side at rest,
    ///  `LEAD` on the side a fling is heading toward.
    /// Why:
    ///  A picture the view reaches the end of before the next one arrives leaves the rows beyond it blank.
    ///  Bands in the direction of the scroll are the ones the next picture would be asked for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// bandsToward(offset: number, step: number, most: number, heading: Heading): [number, number];
    /// ```
    pub fn bands_toward(
        &self,
        offset: f32,
        step: f32,
        most: usize,
        heading: Heading,
    ) -> Range<usize> {
        if self.content <= 0.0 || step <= 0.0 {
            return 0..0;
        }
        let (behind, ahead) = heading.margins();
        let total = band_count(self.content, step);
        let from = self.slide(offset) / self.scale();
        let to = (self.slide(offset) + self.track()) / self.scale();
        let cap = most.max(1);
        let mut first =
            (((from - behind as f32 * step) / step).floor().max(0.0) as usize).min(total - 1);
        let end = (((to + ahead as f32 * step) / step).ceil() as usize).clamp(first + 1, total);
        if end - first > cap {
            // The texture cannot hold the margin bands as well: start at the band the track starts in.
            first = ((from / step).floor().max(0.0) as usize).clamp(first, end - 1);
        }
        return first..end.min(first.saturating_add(cap));
    }
}
