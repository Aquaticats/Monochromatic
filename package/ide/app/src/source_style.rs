//! Source classifications remain independent of any native display geometry.

/// What:
///  Arc shares immutable classifications across the worker and native rendering state.
/// Why:
///  Unlike Rc it can cross threads;
///  unlike Vec cloning it does not copy every span.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SourceStyles = ReadonlyArray<StyleSpan>;
/// ```
use std::sync::Arc;

/// Immutable shared source classifications;
///  replacing them creates a new snapshot.
pub type SourceStyles = Arc<[StyleSpan]>;

/// What:
///  Palette role of a Markdown heading;
///  roles number the entries of `syntax::SCOPES` from one,
///  so this is the fourteenth scope.
/// Why:
///  Markdown's own captures are `markup.*`;
///  each needs a role of its own so the raster can give it a color and a face.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const HEADING_ROLE = 14;
/// ```
pub const HEADING_ROLE: u32 = 14;
/// Palette role of Markdown strong text.
pub const BOLD_ROLE: u32 = 15;
/// Palette role of Markdown emphasis.
pub const ITALIC_ROLE: u32 = 16;
/// Palette role of Markdown struck-through text.
pub const STRIKETHROUGH_ROLE: u32 = 17;
/// Palette role of an inline code span.
pub const RAW_INLINE_ROLE: u32 = 18;
/// Palette role of link text and link labels.
pub const LINK_ROLE: u32 = 19;
/// Palette role of a link destination.
pub const LINK_URL_ROLE: u32 = 20;
/// Palette role of list markers.
pub const LIST_ROLE: u32 = 21;
/// Palette role of block quote text.
pub const QUOTE_ROLE: u32 = 22;

/// What:
///  The second channel of a palette role besides its color:
///  a weight change relative to the source weight,
///  and whether the real italic face is used.
///  `f32` is a 32-bit float (sibling `i32`).
/// Why:
///  The raster paints glyph runs only,
///  so underline and strikethrough cannot be drawn;
///  weight and the italic face are the style channels that exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RoleFace = { weightOffset: number; italic: boolean };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoleFace {
    /// Added to the source weight and clamped to the bundled axis range 100 to 800.
    pub weight_offset: f32,
    /// Use the real italic face for this range.
    pub italic: bool,
}

/// What:
///  The face a palette role asks for,
///  or `None` when the role keeps the weight and style of the surrounding source.
///  `Option<RoleFace>` is a value or nothing.
/// Why:
///  Headings and strong text are bold,
///  emphasis and quotes are italic,
///  links and struck text move the weight,
///  so color is never the only channel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function roleFace(role: number): RoleFace | undefined
/// ```
pub fn role_face(role: u32) -> Option<RoleFace> {
    return match role {
        HEADING_ROLE | BOLD_ROLE | LIST_ROLE => Some(RoleFace {
            weight_offset: 300.0,
            italic: false,
        }),
        LINK_ROLE => Some(RoleFace {
            weight_offset: 200.0,
            italic: false,
        }),
        STRIKETHROUGH_ROLE => Some(RoleFace {
            weight_offset: -100.0,
            italic: false,
        }),
        ITALIC_ROLE | QUOTE_ROLE | LINK_URL_ROLE => Some(RoleFace {
            weight_offset: 0.0,
            italic: true,
        }),
        _ => None,
    };
}

/// What:
///  A source highlight range,
///  independent of native pixels.
/// Why:
///  Parser results can be validated and tested without a window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StyleSpan = {start: number; end: number; style: number};
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleSpan {
    /// Inclusive source character offset.
    pub start: usize,
    /// Exclusive source character offset.
    pub end: usize,
    /// Semantic palette entry.
    pub style: usize,
}
