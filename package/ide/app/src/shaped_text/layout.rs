//! Shaping of single lines and virtual rows:
//!  the inputs of one source line,
//!  the font setup that source lines and virtual rows share,
//!  and the measurements that depend on it.

/// The paint role of virtual-row glyphs.
use super::{ROW_ROLE, TextShaper};
/// Glyphs are set on a line box as tall as one code row.
use crate::row_map::CODE_ROW;
/// Syntax classifications stay independent of pixels.
use crate::source_style::role_face;
/// Height and text size of the virtual rows above annotated lines.
use crate::virtual_row::{ROW_HEIGHT, ROW_TEXT};
/// IntelliJ's break rules as a Parley override,
/// and the continuation indent of a wrapped line.
use crate::wrap::{INTELLIJ_LINE_BREAK, break_wrapped, has_wrap_width};
/// Font and paragraph layout are supplied by the same Parley stack Slint uses.
use parley::{
    Affinity, Cursor, FontFamily, FontFeatures, FontStyle, FontWeight, Layout, LineHeight,
    OverflowWrap, StyleProperty,
};
/// Font bytes are shared immutably across the font database.
use std::borrow::Cow;

/// Family name of the bundled interface face,
///  as its name table spells it.
const ROW_FAMILY: &str = "Inter Variable";

/// What:
///  The inputs that shape one source line:
///  display text,
/// scale,
/// syntax roles,
/// tab spacings,
/// and an optional wrap width.
/// Why:
///  Plain and wrapped layouts share one builder,
///  and the wrap width is the only difference between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LineRequest = { text: string; scale: number; roles: Role[]; spacings: Spacing[]; wrap?: number };
/// ```
#[derive(Clone, Copy)]
pub(crate) struct LineRequest<'a> {
    /// Display text,
    /// with each tab stood in by one space.
    pub(crate) text: &'a str,
    /// Physical pixels per logical pixel.
    pub(crate) scale: f32,
    /// Syntax role runs as display byte start,
    /// end,
    /// and brush.
    pub(crate) roles: &'a [(usize, usize, u32)],
    /// Extra advance per display byte in logical pixels.
    pub(crate) spacings: &'a [(usize, f32)],
    /// Wrap width in logical pixels,
    /// or nothing for an unbroken line.
    pub(crate) wrap: Option<f32>,
}

/// Shape one line or one virtual row with the font setup the whole module shares.
impl TextShaper {
    /// Shape text with explicit source typography and no soft wrapping.
    /// `spacings` pairs a display byte with extra advance in logical pixels;
    ///  tabs use it to reach their stop.
    pub(crate) fn line_layout(
        &mut self,
        text: &str,
        scale: f32,
        roles: &[(usize, usize, u32)],
        spacings: &[(usize, f32)],
    ) -> Layout<u32> {
        let request = LineRequest {
            text,
            scale,
            roles,
            spacings,
            wrap: None,
        };
        return self.build_line(request);
    }

    /// What:
    ///  Shape text broken at a logical wrap width with IntelliJ's break rules,
    ///  and a hanging continuation indent.
    /// Why:
    ///  Parley breaks by its own rules unless the override says otherwise,
    ///  and a continuation row starts after the indent,
    ///  so the indent is set on the layout before it is broken.
    ///  A width that is not positive gives the unbroken layout,
    ///  because the caller keeps its last positive width instead (`wrap::has_wrap_width`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// wrappedLayout(text: string, scale: number, width: number): Layout;
    /// ```
    pub(crate) fn wrapped_line_layout(&mut self, request: LineRequest<'_>) -> Layout<u32> {
        let width = request.wrap.unwrap_or(0.0);
        if !has_wrap_width(width) {
            let unwrapped = LineRequest {
                wrap: None,
                ..request
            };
            return self.build_line(unwrapped);
        }
        // The advance is physical; the indent that is built from it is in logical pixels and is scaled once more
        // where the layout takes it, so a physical advance would double the indent at scale 2.
        let space = self.space_advance(request.scale) / request.scale;
        let mut layout = self.build_line(request);
        break_wrapped(&mut layout, request.text, request.scale, width, space);
        return layout;
    }

    /// Build and break one line with the shared font setup;
    ///  a request that wraps is left unbroken for `break_wrapped`.
    fn build_line(&mut self, request: LineRequest<'_>) -> Layout<u32> {
        let mut builder =
            self.layouts
                .ranged_builder(&mut self.fonts, request.text, request.scale, true);
        builder.push_default(StyleProperty::FontFamily(FontFamily::Source(
            Cow::Borrowed("JetBrains Mono"),
        )));
        builder.push_default(StyleProperty::FontSize(15.0));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(
            self.typography.weight,
        )));
        let style = if self.typography.italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };
        builder.push_default(StyleProperty::FontStyle(style));
        // Glyphs are set on a line box as tall as one code row; where that row sits is the row map's business.
        builder.push_default(StyleProperty::LineHeight(LineHeight::Absolute(CODE_ROW)));
        builder.push_default(StyleProperty::Brush(0));
        builder.push_default(StyleProperty::FontFeatures(FontFeatures::List(
            Cow::Borrowed(&self.typography.features),
        )));
        if request.wrap.is_some() {
            // What: `OverflowWrap::Anywhere` lets a token wider than the width break inside it.
            // Why: IntelliJ breaks a token without an opportunity at the code point where the width is exceeded.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // style.overflowWrap = 'anywhere';
            // ```
            builder.push_default(StyleProperty::OverflowWrap(OverflowWrap::Anywhere));
            builder.set_line_break_override(Some(INTELLIJ_LINE_BREAK));
        }
        for (start, end, role) in request.roles {
            builder.push(StyleProperty::Brush(*role), *start..*end);
            // What: a role's second channel is a weight offset and the real italic face for its range.
            // Why: The raster paints glyph runs only, so weight and italic are the style channels that exist.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const face = roleFace(role); if (face) { range.weight = clamp(base + face.offset); range.italic ||= face.italic; }
            // ```
            if let Some(face) = role_face(*role) {
                if face.weight_offset != 0.0 {
                    let weight = (self.typography.weight + face.weight_offset).clamp(100.0, 800.0);
                    builder.push(
                        StyleProperty::FontWeight(FontWeight::new(weight)),
                        *start..*end,
                    );
                }
                if face.italic {
                    builder.push(StyleProperty::FontStyle(FontStyle::Italic), *start..*end);
                }
            }
        }
        // The stand-in space of a tab is one byte; spacing on it changes that one advance only.
        for (byte, extra) in request.spacings {
            builder.push(StyleProperty::LetterSpacing(*extra), *byte..*byte + 1);
        }
        let mut layout = builder.build(request.text);
        if request.wrap.is_none() {
            layout.break_all_lines(None);
        }
        return layout;
    }

    /// What:
    ///  Shape the text of one virtual row,
    ///  an inlay hint or one row of a diagnostic message:
    ///  the
    ///       interface face at [`ROW_TEXT`] on a line box of [`ROW_HEIGHT`],
    ///  regular weight,
    ///  upright.
    ///       `&str` lends the text;
    ///  the answer is an owned layout.
    /// Why:
    ///  The reference editor sets these rows in its interface face,
    ///  which tells them from source text at
    ///      a glance.
    ///  The text is never part of a source line,
    ///  and the raster paints it in one given ink.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rowLayout(text: string, scale: number): Layout;
    /// ```
    pub fn row_layout(&mut self, text: &str, scale: f32) -> Layout<u32> {
        return self.build_row(text, scale, None);
    }

    /// What:
    ///  Break the text of one virtual row into pieces that each fit `width` logical pixels,
    ///  in the face and size [`Self::row_layout`] sets it;
    ///  a token wider than the width breaks inside it.
    ///  The answer is the pieces in reading order,
    /// trailing blanks dropped;
    ///  an empty text has none.
    /// Why:
    ///  Hint labels and message rows wrap to the width of the code they stand over,
    ///  and a block's height is the number of pieces,
    ///  so the break comes from the same shaping that paints each piece.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// wrapRowText(text: string, scale: number, width: number): string[];
    /// ```
    pub fn wrap_row_text(&mut self, text: &str, scale: f32, width: f32) -> Vec<String> {
        let layout = self.build_row(text, scale, Some(width));
        let mut pieces = Vec::new();
        for line in layout.lines() {
            let range = line.text_range();
            let piece = text[range.start..range.end.min(text.len())].trim_end();
            if !piece.is_empty() {
                pieces.push(piece.to_string());
            }
        }
        tracing::trace!(width, pieces = pieces.len(), "wrapped virtual row text");
        return pieces;
    }

    /// Shape virtual-row text in the interface face,
    /// broken at `wrap` logical pixels or not at all.
    fn build_row(&mut self, text: &str, scale: f32, wrap: Option<f32>) -> Layout<u32> {
        let mut builder = self
            .layouts
            .ranged_builder(&mut self.fonts, text, scale, true);
        builder.push_default(StyleProperty::FontFamily(FontFamily::Source(
            Cow::Borrowed(ROW_FAMILY),
        )));
        builder.push_default(StyleProperty::FontSize(ROW_TEXT));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(400.0)));
        builder.push_default(StyleProperty::FontStyle(FontStyle::Normal));
        builder.push_default(StyleProperty::LineHeight(LineHeight::Absolute(ROW_HEIGHT)));
        builder.push_default(StyleProperty::Brush(ROW_ROLE));
        if wrap.is_some() {
            // A token wider than the width has to break inside it, as it does in the source text.
            builder.push_default(StyleProperty::OverflowWrap(OverflowWrap::Anywhere));
        }
        let mut layout = builder.build(text);
        // A wrap width is scaled to physical pixels; no width means no wrapping.
        let mut limit = None;
        if let Some(width) = wrap {
            limit = Some(width * scale);
        }
        layout.break_all_lines(limit);
        return layout;
    }

    /// Physical advance of one space in the source font at `scale`,
    ///  measured once per scale.
    /// Tab stops are multiples of this advance,
    ///  so it comes from the font,
    ///  not from an assumed cell width.
    pub(crate) fn space_advance(&mut self, scale: f32) -> f32 {
        // What: `if let Some((known, advance))` runs only when a measurement is stored, naming its two parts.
        // Why: A viewport reshapes many tab lines per frame; the font is asked once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.space && this.space[0] === scale) return this.space[1];
        // ```
        if let Some((known, advance)) = self.space
            && known == scale
        {
            return advance;
        }
        let probe = self.line_layout(" ", scale, &[], &[]);
        // The caret after the only character stands one space advance from the line start.
        let cursor = Cursor::from_byte_index(&probe, 1, Affinity::Upstream);
        let advance = cursor.geometry(&probe, 1.0).x0 as f32;
        self.space = Some((scale, advance));
        return advance;
    }

    /// Baseline of the primary font at `scale`;
    ///  a known primary glyph makes it the same for all source rows.
    pub(super) fn baseline(&mut self, scale: f32) -> f32 {
        let probe = self.line_layout("M", scale, &[], &[]);
        return probe
            .lines()
            .next()
            .expect("primary font line")
            .metrics()
            .baseline;
    }
}
