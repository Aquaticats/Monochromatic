//! The wrap indicator the gutter shows on a row that starts after a soft wrap.
//!
//! IntelliJ marks such a row with the first usable pair of symbols in
//! `CompositeSoftWrapPainter` (U+2926 and U+2925,
//! U+21B2 and U+21B3,
//! U+2936 and U+2937,
//! U+21A9 and U+21AA),
//!  and draws an arrow itself (`ArrowSoftWrapPainter`) when no pair can be displayed.
//! A pair is usable only when its font can display both symbols (`TextBasedSoftWrapPainter`);
//!  the symbol of the row after the wrap is the second of the pair.
//! The mark is painted in `EditorColors.SOFT_WRAP_SIGN_COLOR`,
//! `c0c0c0` in the Default scheme,
//!  which the Darcula scheme inherits.
//! Here the mark is set in the same shaper and the same fallback fonts as the source text,
//!  so what the coverage check says is what the gutter shows.

/// One code row is the height of the mark's cell.
use crate::row_map::CODE_ROW;
/// The shaper decides which symbols its fonts display,
/// and sets the one that is chosen.
use crate::shaped_text::TextShaper;
/// The scene the mark is drawn from and the strokes of the drawn arrow.
use crate::tile_scene::{Item, StrokeItem, TileScene, encode, layout_scene};
/// Failures of the layout walk reach the caller.
use anyhow::Result;
/// Glyph ids tell a displayable symbol from the font's missing-glyph box.
use parley::PositionedLayoutItem;
/// The arrow's path.
use vello::kurbo::{Affine, BezPath};

/// IntelliJ's `SOFT_WRAP_SIGN_COLOR` in the Default scheme,
/// which the Darcula scheme inherits:
///  the same straight RGBA ink in the light and the dark scheme.
pub const SIGN_INK: [u8; 4] = [0xC0, 0xC0, 0xC0, 0xFF];

/// What:
///  IntelliJ's candidate pairs in its order:
///  the symbol before a soft wrap,
/// and the symbol after it that marks the continuation row.
/// Why:
///  `CompositeSoftWrapPainter.SYMBOLS` lists these four before its private-use pair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SIGN_PAIRS = [['⤦', '⤥'], ['↲', '↳'], ['⤶', '⤷'], ['↩', '↪']];
/// ```
pub const SIGN_PAIRS: [(char, char); 4] = [
    ('\u{2926}', '\u{2925}'),
    ('\u{21B2}', '\u{21B3}'),
    ('\u{2936}', '\u{2937}'),
    ('\u{21A9}', '\u{21AA}'),
];

/// What:
///  What the gutter draws on a continuation row:
///  a symbol the fonts display,
///  or an arrow drawn by hand.
/// Why:
///  Shape is one of the two channels that tell the mark from a line number;
///  its color is the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Sign = { glyph: string } | 'arrow';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    /// The symbol after the wrap in the first pair whose symbols the fonts display.
    Glyph(
        /// The symbol that marks the row after the wrap.
        char,
    ),
    /// No pair is displayable,
    /// so an arrow is drawn.
    Arrow,
}

/// What:
///  Which symbols the fonts can show.
///  A type that implements this trait answers for one symbol at a time.
/// Why:
///  The coverage is a parameter of `choose_sign` so a test can state which symbols the fonts have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface SignCoverage { canDisplay(character: string): boolean }
/// ```
pub trait SignCoverage {
    /// Whether the fonts show `character`.
    fn can_display(&mut self, character: char) -> bool;
}

/// What:
///  The first pair of `SIGN_PAIRS` whose two symbols `coverage` accepts,
///  as the symbol of the row after the wrap;
///  an arrow when there is none.
/// Why:
///  The coverage is a parameter so a test can state which symbols the fonts have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chooseSign(coverage: SignCoverage): Sign;
/// ```
pub fn choose_sign(mut coverage: impl SignCoverage) -> Sign {
    for (before, after) in SIGN_PAIRS {
        if coverage.can_display(before) && coverage.can_display(after) {
            return Sign::Glyph(after);
        }
    }
    return Sign::Arrow;
}

/// Coverage check and painting of the sign.
impl TextShaper {
    /// What:
    ///  Whether the source font or its fallbacks display `character` at `scale`:
    ///  the character shapes to at least one glyph and none of them is the missing-glyph box.
    /// Why:
    ///  Glyph zero is the font's box for a character it cannot show;
    ///  the shaper picks a fallback font when the primary lacks the character.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// canDisplay(character: string, scale: number): boolean;
    /// ```
    pub fn can_display(&mut self, character: char, scale: f32) -> bool {
        let layout = self.line_layout(&character.to_string(), scale, &[], &[]);
        let mut glyphs = 0_usize;
        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(run) = item else {
                    continue;
                };
                for glyph in run.positioned_glyphs() {
                    if glyph.id == 0 {
                        return false;
                    }
                    glyphs += 1;
                }
            }
        }
        return glyphs > 0;
    }

    /// The sign this shaper's fonts can show,
    /// chosen from IntelliJ's pairs.
    pub fn choose_sign(&mut self, scale: f32) -> Sign {
        // The same walk as the free `choose_sign`, with the coverage check of this shaper at `scale`.
        for (before, after) in SIGN_PAIRS {
            if self.can_display(before, scale) && self.can_display(after, scale) {
                return Sign::Glyph(after);
            }
        }
        return Sign::Arrow;
    }

    /// What:
    ///  The scene of `sign` in a cell `width` logical pixels wide and one code row tall at `scale`,
    ///  in [`SIGN_INK`].
    ///  A symbol is set like source text and ends at the cell's right edge,
    /// as line numbers do.
    /// Why:
    ///  The gutter shows the texture this scene is drawn into,
    /// so the symbol is the one the coverage check shaped.
    ///
    /// # Errors
    /// The symbol's layout holds an inline box,
    /// which source text never produces.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// signScene(sign: Sign, scale: number, width: number): TileScene;
    /// ```
    pub fn sign_scene(&mut self, sign: Sign, scale: f32, width: f32) -> Result<TileScene> {
        let pixel_width = (width * scale).ceil().max(1.0) as u32;
        let pixel_height = (CODE_ROW * scale).ceil() as u32;
        match sign {
            Sign::Glyph(character) => {
                let layout = self.line_layout(&character.to_string(), scale, &[], &[]);
                let advance = layout.full_width();
                let left = pixel_width as f32 - advance;
                return layout_scene(&layout, (left, 0.0), SIGN_INK, (pixel_width, pixel_height));
            }
            Sign::Arrow => {
                return Ok(arrow_scene(pixel_width, pixel_height, scale));
            }
        }
    }
}

/// What:
///  An arrow drawn by hand in [`SIGN_INK`]:
///  a stem down from the upper left that turns to the right and ends in a head,
///  the shape of the symbol it stands in for.
/// Why:
///  `ArrowSoftWrapPainter` is IntelliJ's answer when no font shows a symbol of the pairs.
///  The stroke is round-capped and 1.25 logical pixels wide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function arrowScene(width: number, height: number, scale: number): TileScene;
/// ```
fn arrow_scene(width: u32, height: u32, scale: f32) -> TileScene {
    let cell = width as f32 / scale;
    let (stem, turn, tip) = (f64::from(cell * 0.3), 15.0_f64, f64::from(cell - 2.0));
    // The stem and the horizontal run, then the two strokes of the head, in logical pixels,
    // scaled to physical pixels as a whole.
    let mut path = BezPath::new();
    path.move_to((stem, 6.0));
    path.line_to((stem, turn));
    path.line_to((tip, turn));
    path.move_to((tip - 4.0, turn - 4.0));
    path.line_to((tip, turn));
    path.line_to((tip - 4.0, turn + 4.0));
    path.apply_affine(Affine::scale(f64::from(scale)));
    let items = [Item::Stroke(StrokeItem {
        path,
        width: 1.25 * scale,
        dash: Vec::new(),
        round: true,
        ink: SIGN_INK,
    })];
    return TileScene {
        scene: encode(&items, height as f32),
        width,
        height,
    };
}
