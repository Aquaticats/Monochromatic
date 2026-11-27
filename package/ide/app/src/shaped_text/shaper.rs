//! The shaper's own steps:
//!  its construction over the bundled faces,
//!  the row and whole-file shaping entry points,
//!  the counters a resize is measured by,
//!  and the fork and forget a reload needs.
//!  The steps of one line,
//! of a pass and of reading a view live in their own files.

/// The shaper type whose steps these are,
///  and the request one shaped row takes.
use super::{
    RowRequest,
    TextShaper,
};
/// The frame types these steps hand back,
///  and the clip rule they fit a viewport with.
use super::{
    ShapedView,
    Viewport,
    fit_viewport,
};
/// Canonical source remains in the read-only document.
use crate::document::Document;
/// Variable roman and real italic blobs retain stable cache identities;
///  the interface face sets virtual rows.
use crate::font_asset::{
    code_faces,
    row_face,
};
/// The one vertical mapping:
///  where each line's code row starts and how tall rows and the caret are.
use crate::row_map::{
    CODE_ROW,
    RowMap,
};
/// One shaped source line,
/// kept by the row cache.
use crate::shaped_row::ShapedRow;
/// Shaped rows kept by line for one text revision and scale.
use crate::shaped_row_cache::RowCache;
/// Syntax classifications stay independent of pixels.
use crate::source_style::StyleSpan;
/// Immutable font requests validate continuous weights before shaping.
use crate::source_typography::SourceTypography;
/// Tabs end on pixel tab stops,
///  whatever script precedes them.
use crate::tab_layout::{
    layout_with_tabs,
    layout_wrapped_with_tabs,
};
/// Source/display byte maps keep tabs and Unicode out of hit-test heuristics.
use crate::text_projection::project_line;
/// Construction failures identify unsupported source typography.
use anyhow::Result;
/// Font and paragraph layout are supplied by the same Parley stack Slint uses.
use parley::{
    FontContext,
    FontFeature,
    LayoutContext,
};

/// Constructing a default shaper registers the bundled source font.
impl Default for TextShaper {
    /// Keep default construction on the same embedded-font and feature policy as explicit construction.
    fn default() -> Self {
        return Self::new();
    }
}

/// Shape complete source lines while retaining a reusable font context.
impl TextShaper {
    /// Register the embedded primary face;
    ///  system fonts supply other scripts.
    pub fn new() -> Self {
        return Self::with_typography(SourceTypography::default())
            .expect("valid default source typography");
    }

    /// Create a source shaper with explicit OpenType features,
    ///  without changing source characters.
    /// Settings are immutable for this shaper;
    ///  replacing it also requires invalidating native frame state.
    pub fn with_features(features: Vec<FontFeature>) -> Self {
        let typography = SourceTypography {
            features,
            ..SourceTypography::default()
        };
        return Self::with_typography(typography).expect("valid default source weight");
    }

    /// Select continuous variable weight and the real roman or italic face.
    pub fn with_typography(typography: SourceTypography) -> Result<Self> {
        typography.validate()?;
        let mut fonts = FontContext::new();
        // Registration includes both faces, so italic requests never depend on a host font or fake slant.
        for blob in code_faces() {
            fonts.collection.register_fonts(blob, None);
        }
        // Virtual rows are set in the bundled interface face, never in whatever the host calls by that name.
        fonts.collection.register_fonts(row_face(), None);
        return Ok(Self {
            fonts,
            layouts: LayoutContext::new(),
            typography,
            space: None,
            rows: RowCache::default(),
            shaped_rows: 0,
            rebroken_rows: 0,
            tab_passes: 0,
        });
    }

    /// Shape one source line with syntax roles on a given common baseline.
    /// `place` pairs the line with the top of its code row in logical pixels;
    ///  `wrap` is the wrap width in logical pixels,
    /// or nothing for an unbroken line.
    pub(super) fn shape_row(
        &mut self,
        document: &Document,
        place: (usize, f32),
        scale: f32,
        styles: &[StyleSpan],
        baseline: f32,
        wrap: Option<f32>,
    ) -> ShapedRow {
        let (row, top) = place;
        let text = document.text();
        let source_start = text.line_to_char(row);
        let source = text.line(row).to_string();
        let projection = project_line(&source);
        let source_len = projection.source_to_byte.len().saturating_sub(1);
        let mut roles = Vec::new();
        // What: Spans ascend and do not overlap, so a binary search finds the first one that reaches this line.
        // Why: A pass over a whole file shapes every line; scanning every span for each would be quadratic.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const relevant = styles.slice(lowerBound(styles, (span) => span.end > sourceStart));
        // ```
        let source_end = source_start + text.line(row).len_chars();
        // A halving search finds the first span that reaches this line.
        let mut reaching = 0;
        let mut high = styles.len();
        while reaching < high {
            let middle = reaching + (high - reaching) / 2;
            if styles[middle].end <= source_start {
                reaching = middle + 1;
            } else {
                high = middle;
            }
        }
        for span in styles[reaching..].iter() {
            // The spans that start at or after the line's end do not touch it.
            if span.start >= source_end {
                break;
            }
            let start = span.start.saturating_sub(source_start).min(source_len);
            let end = span.end.saturating_sub(source_start).min(source_len);
            if start < end {
                roles.push((
                    projection.source_to_byte[start],
                    projection.source_to_byte[end],
                    span.style as u32,
                ));
            }
        }
        let layout = match wrap {
            Some(width) => layout_wrapped_with_tabs(self, &projection, scale, width, &roles),
            None => layout_with_tabs(self, &projection, scale, &roles),
        };
        let natural = layout
            .lines()
            .next()
            .expect("source line layout")
            .metrics()
            .baseline;
        return ShapedRow {
            row,
            top,
            source_start,
            projection,
            layout,
            baseline,
            baseline_shift: baseline - natural,
            wrap,
        };
    }

    /// Shape one arbitrary source line for reading geometry,
    ///  without syntax colors.
    /// Caret movement uses it for lines outside the materialized viewport;
    ///  `row` is clamped to the last line.
    /// The row has no vertical position:
    ///  its `top` is zero,
    ///  and only its horizontal geometry is meaningful.
    pub fn row(
        &mut self,
        document: &Document,
        row: usize,
        scale: f32,
    ) -> ShapedRow {
        let last = document.text().len_lines().saturating_sub(1);
        let baseline = self.baseline(scale);
        return self.shape_row(document, (row.min(last), 0.0), scale, &[], baseline, None);
    }

    /// What:
    ///  Shape one arbitrary source line broken at a wrap width,
    ///  as the source view would show it when wrapping is on.
    /// Why:
    ///  A line with several visual rows needs its row geometry,
    /// and this is the only path that makes one.
    ///  A width that is not positive shapes the unbroken line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// wrappedRow(document: Document, row: number, scale: number, width: number): ShapedRow;
    /// ```
    pub fn wrapped_row(
        &mut self,
        document: &Document,
        row: usize,
        scale: f32,
        width: f32,
    ) -> ShapedRow {
        let last = document.text().len_lines().saturating_sub(1);
        let baseline = self.baseline(scale);
        return self.shape_row(
            document,
            (row.min(last), 0.0),
            scale,
            &[],
            baseline,
            Some(width),
        );
    }

    /// Prepare a visible viewport of a text without virtual rows:
    ///  line `n` starts at `n` code rows.
    pub fn prepare(
        &mut self,
        document: &Document,
        viewport: Viewport,
        styles: &[StyleSpan],
    ) -> ShapedView {
        let map = RowMap::plain(document.text().len_lines());
        return self.prepare_view(document, viewport, styles, &map, false);
    }

    /// What:
    ///  Prepare a visible viewport using native font advances rather than character cells,
    ///  with every
    ///       row placed by `map`.
    ///  `&RowMap` lends the vertical mapping;
    ///  the view keeps its own copy.
    /// Why:
    ///  Lines with virtual rows above them start lower than their line number alone says;
    ///  the frame,
    ///      its selection rectangles,
    ///  and its raster all take row positions from this one mapping.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prepareRows(document: Document, viewport: Viewport, styles: StyleSpan[], map: RowMap): ShapedView;
    /// ```
    pub fn prepare_rows(
        &mut self,
        document: &Document,
        viewport: Viewport,
        styles: &[StyleSpan],
        map: &RowMap,
    ) -> ShapedView {
        return self.prepare_view(document, viewport, styles, map, true);
    }

    /// Source rows shaped since this shaper was made:
    ///  a row taken from the cache is not counted.
    /// The structural tests compare it before and after a scroll to prove that no row was shaped.
    pub fn shaped_rows(&self) -> u64 {
        return self.shaped_rows;
    }

    /// Corrective passes the tab correction of wrapped lines has shaped since this shaper was made;
    ///  the structural tests compare it with the bound a line's length allows.
    pub fn tab_passes(&self) -> u64 {
        return self.tab_passes;
    }

    /// Typography this shaper was built with,
    /// so a second shaper can produce the same glyphs.
    pub fn typography(&self) -> &SourceTypography {
        return &self.typography;
    }

    /// A shaper with this one's fonts and typography and its own layout context and empty caches.
    /// What:
    /// `FontContext::clone` copies the scanned font registry instead of discovering system fonts again.
    /// Why:
    /// A second font discovery takes as long as the first,
    /// and a worker would otherwise pay it on its first tile.
    pub fn fork(&self) -> Self {
        return Self {
            fonts: self.fonts.clone(),
            layouts: LayoutContext::new(),
            typography: self.typography.clone(),
            space: None,
            rows: RowCache::default(),
            shaped_rows: 0,
            rebroken_rows: 0,
            tab_passes: 0,
        };
    }

    /// Forget the cached rows,
    /// after a change the cache key does not describe,
    /// such as new syntax styles.
    pub fn forget_rows(&mut self) {
        self.rows.clear();
    }

    /// What:
    ///  Prepare the viewport rows,
    ///  taking a row from the cache when `cached` allows it and the row was shaped for this text revision and scale.
    /// Why:
    ///  A re-centered tile shares most of its lines with the previous one,
    ///  and shaping is the expensive part of preparing a tile.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function prepareView(document: Document, viewport: Viewport, styles: StyleSpan[], map: RowMap, cached: boolean): ShapedView;
    /// ```
    fn prepare_view(
        &mut self,
        document: &Document,
        viewport: Viewport,
        styles: &[StyleSpan],
        map: &RowMap,
        cached: bool,
    ) -> ShapedView {
        let text = document.text();
        let last = viewport
            .first
            .saturating_add(viewport.count)
            .min(text.len_lines());
        let baseline = self.baseline(viewport.scale);
        if cached {
            self.rows.sync(document.revision(), viewport.scale);
        }
        let mut shaped_rows = Vec::new();
        for row in viewport.first..last {
            // What: A cached row is copied; a miss is shaped, or a kept row of another wrap width broken again, and kept.
            // Why: Shaping ignores the vertical position, so the same row serves every map.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const shaped = cache.get(row) ?? shapeAndKeep(row);
            // ```
            let request = RowRequest {
                row,
                scale: viewport.scale,
                wrap: viewport.wrap,
                baseline,
            };
            if cached {
                self.ensure_row(document, styles, request);
            }
            let hit = if cached {
                self.rows.get(row, viewport.wrap).cloned()
            } else {
                None
            };
            let shaped = match hit {
                Some(shaped) => shaped,
                None => {
                    self.shaped_rows += 1;
                    self.shape_row(
                        document,
                        (row, 0.0),
                        viewport.scale,
                        styles,
                        baseline,
                        viewport.wrap,
                    )
                }
            };
            shaped_rows.push(shaped);
        }
        if cached {
            self.rows.trim(viewport.first + viewport.count / 2);
        }
        // What: When lines wrap, the rows these lines really have replace the map's counts for them.
        // Why: The map may still count an unmeasured line as one row; the frame must place its rows by what it shaped.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const placed = wrap ? map.withMeasured(rows.map((r) => [r.row, r.visualRows])) : map;
        // ```
        let placed = if viewport.wrap.is_some() {
            let mut measured: Vec<(usize, usize)> = Vec::new();
            for shaped in shaped_rows.iter() {
                measured.push((shaped.row, shaped.visual_rows()));
            }
            map.with_measured(&measured)
        } else {
            map.clone()
        };
        let mut rows = Vec::new();
        for shaped in shaped_rows {
            let top = placed.code_top(shaped.row);
            rows.push(ShapedRow {
                top,
                ..shaped
            });
        }
        let width = (viewport.width.max(1.0) * viewport.scale).ceil() as u32;
        // The raster spans every materialized line with its block, or the clip of them; an empty frame is still one
        // row tall.
        let first_top = placed.block_top(viewport.first);
        let bottom = placed.block_top(last);
        // The clip the window asked for was chosen from the heights its map held, which the rows just shaped may
        // exceed, so the frame is cut to the raster limit here.
        let fitted = fit_viewport(viewport, first_top, bottom, u64::from(width));
        let (origin, tall) = match fitted.clip {
            Some(clip) => (first_top + clip.top, clip.bottom - clip.top),
            None => (first_top, bottom - first_top),
        };
        let extent = tall.max(CODE_ROW);
        let height = (extent * fitted.scale).ceil() as u32;
        let mut view = ShapedView {
            rows,
            viewport: fitted,
            width,
            height,
            origin,
            // The frame keeps the mapping it placed its rows by, so it stays consistent while the window's own moves on.
            map: placed,
            selections: Vec::new(),
            matches: Vec::new(),
            annotations: None,
        };
        view.selections = view.selection(document);
        return view;
    }
}
