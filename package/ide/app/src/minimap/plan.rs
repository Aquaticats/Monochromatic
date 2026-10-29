//! The slices of the picture:
//!  the bands the picture is cut into,
//!  the rows of the whole picture that a run of bands makes up,
//!  and the chunks a slice is rendered in.

/// A texture's bytes are bounded as the source tile's are.
use crate::shaped_text::MAX_RASTER_BYTES;
/// The bands are counted along the text's height.
use std::ops::Range;

/// What:
///  Least display scale the picture is built at.
/// Why:
///  A scale of zero or less would make every texture empty;
///  no real display is this small.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MIN_SCALE = 0.25;
/// ```
pub const MIN_SCALE: f32 = 0.25;

/// What:
///  Most rows of the picture one render of the GPU draws.
/// Why:
///  Vello 0.11 sizes its scratch buffers with fixed constants,
///  and a scene that needs more than they hold renders blank without an error
///  (`doc/planning/ide-vello-buffer-sizes.md`):
///  the densest text the editor can show fills them somewhere between 4,096 and 5,120 rows of one render
///  (measured on 2026-10-23,
///  `src/native/minimap_dense_tests.rs`;
///  before selected rows were drawn once the limit was between 1,024 and 2,048).
///  A slice is therefore drawn in chunks of at most this many rows,
///  which keeps the densest text a factor of eight inside the buffers;
///  the tests only require a factor of two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const CHUNK_ROWS = 512;
/// ```
pub const CHUNK_ROWS: u32 = 512;

/// What:
///  The rows of the chunks a slice of `rows` is drawn in,
///  each at most [`CHUNK_ROWS`] tall,
///  from the slice's first row.
/// Why:
///  The worker builds one scene for each chunk and the GPU renders them one after another
///  into the one texture of the slice,
///  so the cuts must be whole picture rows that two builds of the same slice agree on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chunkRows(rows: [number, number]): [number, number][];
/// ```
pub fn chunk_rows(rows: &Range<usize>) -> Vec<Range<usize>> {
    let mut chunks = Vec::new();
    let mut start = rows.start;
    while start < rows.end {
        let end = (start + CHUNK_ROWS as usize).min(rows.end);
        chunks.push(start..end);
        start = end;
    }
    return chunks;
}

/// What:
///  The most rows one texture of the minimap's picture may have when it is `width` pixels wide,
///  on a device whose textures are at most `limit` pixels on a side:
///  the side limit,
///  or fewer when the bytes of the texture would pass the rule the source tile keeps
///  ([`MAX_RASTER_BYTES`]).
/// Why:
///  Vello 0.11 renders a texture of more than about 128 MiB wrong without any error
///  (`doc/troubleshooting/vello-silent-blank-or-corrupt-frames-at-fixed-buffer-limits.md`),
///  so the picture is stacked from textures that stay well inside that,
///  whatever the device's limit allows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function textureRows(width: number, limit: number): number;
/// ```
pub fn texture_rows(width: u32, limit: u32) -> u32 {
    let by_bytes = MAX_RASTER_BYTES / 4 / u64::from(width.max(1));
    return u64::from(limit).min(by_bytes).min(u64::from(u32::MAX)) as u32;
}

/// What:
///  How the chunks of a picture,
///  given by their `heights` from the top,
///  are grouped into the textures that are
///  stacked to show it:
///  the chunk numbers of each texture,
///  top first,
///  each texture at most [`texture_rows`] tall for a picture `width` pixels wide on a device of `limit`.
///  A chunk taller than that is a texture of its own,
///  which the GPU then refuses with a message.
/// Why:
///  A texture is made of whole chunks,
///  because the GPU renders and copies a chunk as it was built,
///  and a stack of textures that abut shows the same rows as one texture would,
///  so the cut between two textures is a cut between two chunks,
///  which is already seamless.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function textureRuns(heights: number[], width: number, limit: number): [number, number][];
/// ```
pub fn texture_runs(heights: &[u32], width: u32, limit: u32) -> Vec<Range<usize>> {
    let most = u64::from(texture_rows(width, limit));
    let mut runs = Vec::new();
    let mut start = 0;
    let mut rows: u64 = 0;
    for (index, height) in heights.iter().enumerate() {
        if index > start && rows + u64::from(*height) > most {
            runs.push(start..index);
            start = index;
            rows = 0;
        }
        rows += u64::from(*height);
    }
    if start < heights.len() {
        runs.push(start..heights.len());
    }
    return runs;
}

/// What:
///  How tall a band of the text is,
///  in logical pixels of the text:
///  thirty-two code rows,
///  which are 128 rows of the picture at its fixed scale.
/// Why:
///  The window names the bands the track shows and the worker draws the lines they hold,
///  so a scroll that stays inside the same bands moves the picture in the markup and draws nothing;
///  the cuts stand at multiples of this from the top of the text,
///  so scrolling moves the window over them and never a cut.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const BAND = 768;
/// ```
pub const BAND: f32 = 768.0;

/// What:
///  How many bands the picture holds beyond the track on the side the view is heading toward.
/// Why:
///  A fling moves the track by several bands in the time a picture takes to be requested,
///  built,
/// and drawn (about a tenth of a second);
///  a picture that held one band ahead ran out under the track and left the rows beyond it blank
///  (`doc/handover/ide-gpu-stages-5-6.md`,
/// "The minimap picture in a fling").
///  The side the view is leaving keeps one band,
/// as a view at rest does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const LEAD = 8;
/// ```
pub const LEAD: usize = 8;

/// What:
///  Which way the view last moved far enough to tell,
///  and the position of the top of the track (in logical pixels of the text) where that was decided.
/// Why:
///  The window of bands a request names must not change with every step of a scroll,
///  or each step would post a new request;
///  a heading changes only when the track has moved a whole band against it,
///  so the lead stays on the same side through a fling and after it,
///  and a view that only creeps back and forth within a band keeps its heading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Heading = { kind: 'still' | 'down' | 'up'; at: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Heading {
    /// No movement of a whole band seen yet.
    Still {
        /// The top of the track when the heading began.
        at: f32,
    },
    /// The track moved toward the end of the text.
    Down {
        /// The top of the track when the heading was last confirmed.
        at: f32,
    },
    /// The track moved toward the start of the text.
    Up {
        /// The top of the track when the heading was last confirmed.
        at: f32,
    },
}

/// Following the track and naming the margins a heading calls for.
impl Heading {
    /// What:
    ///  The heading after the top of the track is seen at `from`,
    ///  with bands `step` tall.
    ///  A move of at least a band from the position where the heading was decided sets the heading to that direction
    ///  and moves the position there;
    ///  a smaller move changes nothing.
    /// Why:
    ///  The position is where the heading was last confirmed,
    ///  not the previous step,
    ///  so a fling that moves a band every few steps confirms its direction again and again
    ///  and a creep of less than a band never flips it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// follow(from: number, step: number): Heading;
    /// ```
    #[must_use]
    pub fn follow(self, from: f32, step: f32) -> Self {
        let moved = from - self.at();
        if moved.abs() < step {
            return self;
        }
        if moved > 0.0 {
            return Self::Down { at: from };
        }
        return Self::Up { at: from };
    }

    /// The position of the top of the track where the heading was decided.
    fn at(self) -> f32 {
        return match self {
            Self::Still { at } | Self::Down { at } | Self::Up { at } => at,
        };
    }

    /// What:
    ///  The bands a window holds before the track and after it:
    ///  one on each side at rest,
    ///  and `LEAD` on the side the view is heading toward.
    /// Why:
    ///  The picture a fling needs next is on the side it is heading toward.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// margins(): [number, number];
    /// ```
    #[must_use]
    pub fn margins(self) -> (usize, usize) {
        return match self {
            Self::Still { .. } => (1, 1),
            Self::Down { .. } => (1, LEAD),
            Self::Up { .. } => (LEAD, 1),
        };
    }
}

/// What:
///  How many bands of `step` logical pixels cover a text `height` logical pixels tall.
/// Why:
///  The window thread and the worker count the bands the same way,
///  so a range of band numbers means the same bands to both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bandCount(height: number, step: number): number;
/// ```
pub fn band_count(height: f32, step: f32) -> usize {
    if height <= 0.0 || step <= 0.0 {
        return 0;
    }
    return (height / step).ceil() as usize;
}

/// What:
///  How many bands of the picture fit in one texture `limit` pixels tall,
///  at display `scale` and `ratio` rows of the picture for each logical pixel of the text.
///  Never under one.
/// Why:
///  The texture a slice is drawn into is bounded by the device.
///  The window stacks textures for a picture of more bands than that
///  (`texture_runs`),
///  so this is the size of one of them and no cap on the picture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mostBands(limit: number, scale: number, ratio: number): number;
/// ```
pub fn most_bands(limit: u32, scale: f32, ratio: f64) -> usize {
    let rows = f64::from(BAND) * f64::from(scale.max(MIN_SCALE)) * ratio;
    return ((f64::from(limit) / rows).floor() as usize).max(1);
}

/// What:
///  The rows of the whole picture of a text `height` logical pixels tall,
///  at display `scale` and `ratio` rows of the picture for each logical pixel of the text.
/// Why:
///  The picture is as tall as the text at its fixed scale,
///  rounded up to whole pixels with a margin for the float error of the product.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pictureRows(height: number, scale: number, ratio: number): number;
/// ```
pub fn picture_rows(height: f32, scale: f32, ratio: f64) -> usize {
    return ((f64::from(height) * f64::from(scale) * ratio) - 1e-3)
        .ceil()
        .max(0.0) as usize;
}

/// What:
///  The rows of the whole picture that the bands `window` make up:
///  from the first row of the first band to the first row of the band after the last,
///  and to the end of the picture when the window reaches the text's end.
/// Why:
///  The edges are whole picture rows and the same for every slice,
///  so two slices that share a band agree on where it stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sliceRows(window: [number, number], height: number, scale: number, ratio: number): [number, number];
/// ```
pub fn slice_rows(window: &Range<usize>, height: f32, scale: f32, ratio: f64) -> Range<usize> {
    let total = picture_rows(height, scale, ratio);
    let per_band = f64::from(BAND) * f64::from(scale.max(MIN_SCALE)) * ratio;
    let start = ((window.start as f64 * per_band).round() as usize).min(total);
    let end = if window.end >= band_count(height, BAND) {
        total
    } else {
        ((window.end as f64 * per_band).round() as usize).min(total)
    };
    return start.min(end)..end;
}
