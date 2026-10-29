//! The horizontal map of the picture:
//!  the minimap shows the whole width of the text area,
//!  so a position across the minimap is the same share of the text's width.

/// The width of the text the picture covers,
/// in logical pixels:
/// the wrap width,
/// or the width of an unwrapped raster while the view has none.
pub fn text_span(wrap: Option<f32>) -> f32 {
    return wrap.unwrap_or(crate::shaped_text::UNWRAPPED_RASTER_WIDTH);
}

/// What:
///  The distance from the text's left edge,
///  in logical pixels,
///  that the position `x` across a minimap `width` pixels wide shows,
///  for a text `span` pixels wide.
///  A position outside the minimap is held to its edges.
/// Why:
///  The picture is the text's whole width shrunk to the minimap's;
///  a press puts the caret at the character under the pointer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function textX(x: number, width: number, span: number): number;
/// ```
pub fn text_x(x: f32, width: f32, span: f32) -> f32 {
    if width <= 0.0 || span <= 0.0 {
        return 0.0;
    }
    return (x / width).clamp(0.0, 1.0) * span;
}
