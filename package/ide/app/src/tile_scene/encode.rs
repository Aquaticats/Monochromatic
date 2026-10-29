//! Paint items to a Vello scene:
//!  the one place a scene is written.

/// The items to write.
use super::Item;
/// Vello's scene,
/// shapes,
/// and brushes.
use vello::{
    Scene,
    kurbo::{Affine, BezPath, Cap, Join, Rect, Stroke},
    peniko::{Color, Fill},
};

/// What:
///  A straight-alpha brush color from four bytes.
/// Why:
///  Inks are straight RGBA bytes everywhere in the application,
///  and Vello's transparent output is straight alpha too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function brush([red, green, blue, alpha]: Rgba): Color;
/// ```
fn brush(ink: [u8; 4]) -> Color {
    return Color::from_rgba8(ink[0], ink[1], ink[2], ink[3]);
}

/// What:
///  Write `items` to a new scene for a texture `height` pixels tall.
///  A clip item clips to its two x positions and its band of y positions,
///  cut to the texture.
/// Why:
///  The window thread and the tile worker both end a build here,
///  so the scene of a tile does not depend on which of them built it.
///  Glyphs are drawn unhinted at their fractional positions,
/// as the layout placed them.
///  The band keeps a clip as tall as one row and not as tall as the texture:
///  Vello gives every tile a clip covers its own slot in a buffer of fixed size,
///  and a clip as tall as a texture of 8,000 rows costs about 36,000 slots per selected row,
///  so about 58 selected rows used the whole buffer and the render drew nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function encode(items: Item[], height: number): Scene;
/// ```
pub fn encode(items: &[Item], height: f32) -> Scene {
    let mut scene = Scene::new();
    for item in items {
        match item {
            Item::Glyphs(run) => {
                scene
                    .draw_glyphs(&run.font)
                    .font_size(run.size)
                    .transform(Affine::translate((
                        f64::from(run.origin.0),
                        f64::from(run.origin.1),
                    )))
                    .normalized_coords(&run.coords)
                    .hint(false)
                    .brush(brush(run.ink))
                    .draw(Fill::NonZero, run.glyphs.iter().copied());
            }
            Item::Clip {
                left,
                right,
                top,
                bottom,
            } => {
                // A band that reaches past the texture is cut to it, and a band wholly outside it is empty.
                let band_top = top.clamp(0.0, height);
                let band_bottom = bottom.clamp(band_top, height);
                scene.push_clip_layer(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Rect::new(
                        f64::from(*left),
                        f64::from(band_top),
                        f64::from(*right),
                        f64::from(band_bottom),
                    ),
                );
            }
            Item::Unclip => scene.pop_layer(),
            Item::Fill {
                left,
                top,
                right,
                bottom,
                inset,
                ink,
            } => {
                let outer = Rect::new(
                    f64::from(*left),
                    f64::from(*top),
                    f64::from(*right),
                    f64::from(*bottom),
                );
                let thick = f64::from(*inset);
                // A frame too thin to have an inside is the whole rectangle.
                if thick > 0.0 && outer.width() > 2.0 * thick && outer.height() > 2.0 * thick {
                    let inner = Rect::new(
                        outer.x0 + thick,
                        outer.y0 + thick,
                        outer.x1 - thick,
                        outer.y1 - thick,
                    );
                    // The inner rectangle runs the other way round, so the winding number is zero inside it.
                    // Even-odd filling would read a pixel that holds both rectangles' edges wrongly:
                    // the renderer folds the sum of the two partial areas,
                    // which is not the area between them.
                    let mut frame = BezPath::new();
                    frame.move_to((outer.x0, outer.y0));
                    frame.line_to((outer.x1, outer.y0));
                    frame.line_to((outer.x1, outer.y1));
                    frame.line_to((outer.x0, outer.y1));
                    frame.close_path();
                    frame.move_to((inner.x0, inner.y0));
                    frame.line_to((inner.x0, inner.y1));
                    frame.line_to((inner.x1, inner.y1));
                    frame.line_to((inner.x1, inner.y0));
                    frame.close_path();
                    scene.fill(Fill::NonZero, Affine::IDENTITY, brush(*ink), None, &frame);
                } else {
                    scene.fill(Fill::NonZero, Affine::IDENTITY, brush(*ink), None, &outer);
                }
            }
            Item::Stroke(line) => {
                let mut style = Stroke::new(f64::from(line.width));
                if line.round {
                    style = style.with_caps(Cap::Round).with_join(Join::Round);
                } else {
                    style = style.with_caps(Cap::Butt).with_join(Join::Miter);
                }
                if !line.dash.is_empty() {
                    style = style.with_dashes(0.0, line.dash.iter().copied());
                }
                scene.stroke(&style, Affine::IDENTITY, brush(line.ink), None, &line.path);
            }
        }
    }
    return scene;
}
