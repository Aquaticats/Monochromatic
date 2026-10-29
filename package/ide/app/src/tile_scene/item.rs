//! The data of a tile's paint commands:
//!  glyph runs,
//!  strokes,
//!  clips,
//!  and the scene with the size of the texture it is drawn into.

/// The font a run is drawn with.
use vello::peniko::FontData;
/// One positioned glyph of a run,
/// in the layout's own pixel units,
/// the scene,
/// and the path of a stroke.
use vello::{Glyph, Scene, kurbo::BezPath};

/// What:
///  One run of glyphs in one ink,
///  with the font and the place of its layout.
/// Why:
///  A run is the unit Vello draws,
///  and the unit a test reads the ink and the glyph count of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GlyphItem = { font: Font; size: number; coords: number[]; origin: [number, number]; ink: Rgba; glyphs: Glyph[]; reach: [number, number] };
/// ```
#[derive(Clone)]
pub struct GlyphItem {
    /// The font face of the run.
    pub font: FontData,
    /// Font size in physical pixels.
    pub size: f32,
    /// Normalized variation coordinates of the face.
    pub coords: Vec<i16>,
    /// Physical position of the layout's origin.
    pub origin: (f32, f32),
    /// Straight RGBA ink.
    pub ink: [u8; 4],
    /// Glyph ids and positions relative to `origin`.
    pub glyphs: Vec<Glyph>,
    /// Physical x of the left edge of the first glyph's box and of the right edge of the last glyph's box,
    ///  in the tile's pixels:
    ///  the part of a row where the run can have ink.
    pub reach: (f32, f32),
}

/// What:
///  One stroked path:
///  a diagnostic underline or a stroke of the wrap arrow.
/// Why:
///  Wavy,
///  dashed,
///  and dotted lines are one stroke each,
///  so their style is data a test can read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StrokeItem = { path: Path; width: number; dash: number[]; round: boolean; ink: Rgba };
/// ```
#[derive(Clone)]
pub struct StrokeItem {
    /// The center line.
    pub path: BezPath,
    /// Stroke width in physical pixels.
    pub width: f32,
    /// On and off lengths of the dashes in physical pixels;
    ///  empty for a solid line.
    pub dash: Vec<f64>,
    /// Round caps and joins instead of butt caps.
    pub round: bool,
    /// Straight RGBA ink.
    pub ink: [u8; 4],
}

/// What:
///  One paint command of a tile,
///  in drawing order.
/// Why:
///  The painter produces a list of these and `encode` writes them to a scene,
///  so what a tile draws is checked without a GPU.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Item = { glyphs: GlyphItem } | { clip: [number, number, number, number] } | 'unclip' | { stroke: StrokeItem } | { fill: Rect };
/// ```
#[derive(Clone)]
pub enum Item {
    /// Draw a run of glyphs.
    Glyphs(
        /// The run to draw.
        GlyphItem,
    ),
    /// Start drawing only inside this physical rectangle:
    ///  between two x positions and between two y positions,
    ///  the band of the one row whose ink the clip splits.
    Clip {
        /// Left edge.
        left: f32,
        /// Right edge.
        right: f32,
        /// Top edge,
        ///  from the tile's top.
        top: f32,
        /// Bottom edge,
        ///  from the tile's top.
        bottom: f32,
    },
    /// End the clip that began last.
    Unclip,
    /// Draw a stroked path.
    Stroke(
        /// The path to stroke.
        StrokeItem,
    ),
    /// Fill a rectangle in one ink,
    /// or only a frame of the given thickness along its edges,
    /// for the minimap's selection and match marks.
    Fill {
        /// Left edge in physical pixels.
        left: f32,
        /// Top edge in physical pixels.
        top: f32,
        /// Right edge in physical pixels,
        /// not inside.
        right: f32,
        /// Bottom edge in physical pixels,
        /// not inside.
        bottom: f32,
        /// Thickness of the frame to draw along the edges inside them,
        /// or zero for the whole rectangle.
        /// A frame is one path,
        /// so its four sides leave no seam where they meet.
        inset: f32,
        /// Straight RGBA ink.
        ink: [u8; 4],
    },
}

/// What:
///  A scene and the size of the texture it is drawn into.
/// Why:
///  The window thread hands this to the GPU module,
///  which makes the texture and renders the scene into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TileScene = { scene: Scene; width: number; height: number };
/// ```
pub struct TileScene {
    /// The Vello scene,
    ///  in physical pixels from the tile's top left corner.
    pub scene: Scene,
    /// Texture width in physical pixels.
    pub width: u32,
    /// Texture height in physical pixels.
    pub height: u32,
}

/// What:
///  The scenes of a tile that is drawn across several textures,
///  top first,
///  and the size of the tile.
/// Why:
///  A tile past what one texture may be is stacked from several,
///  each with its own scene,
///  and the window shows them one below the other with no gap,
///  so the tile is as wide as they are and as tall as they are together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TileStack = { textures: TileScene[]; width: number; height: number };
/// ```
pub struct TileStack {
    /// The scene of each texture,
    ///  top first;
    ///  each one starts on the row where the one before it ends.
    pub textures: Vec<TileScene>,
    /// Width of every texture in physical pixels.
    pub width: u32,
    /// Height of the textures together in physical pixels.
    pub height: u32,
}
