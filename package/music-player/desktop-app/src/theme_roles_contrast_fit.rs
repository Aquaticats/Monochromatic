//! Moving a color in lightness until its contrast target holds.
//!
//! What:
//! `FitCheck` is the one question a fit asks,
//! `FitRule` is the per-role answer with the colors it measures against,
//! `fit_lightness` is the search that keeps chroma and hue,
//! `legible_on` is the text test,
//! and `container_layers` builds the hover and pressed fills.
//! Why:
//! the contrast guarantee is the whole reason the roles are derived in OKLCH,
//! so it keeps one file,
//! split from `src/theme_roles.rs` to stay within the line budget.

/// What:
///      `use super::{...}` brings the target ratios and the layer strengths the fit
///           rules measure against.
/// Why:
///      A rule compares a candidate with the same published targets the derivation
///           promises,
///  so one place holds the numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { HOVER_ALPHA, PRESSED_ALPHA, TEXT_LAYER_ALPHAS, TEXT_TARGET_RATIO, NON_TEXT_TARGET_RATIO } from "./theme_roles";
/// ```
use super::{
    HOVER_ALPHA,
    NON_TEXT_TARGET_RATIO,
    PRESSED_ALPHA,
    TEXT_LAYER_ALPHAS,
    TEXT_TARGET_RATIO,
};

/// What:
///      `use crate::oklch::{...}` brings the color type,
///  the 8-bit color type and the contrast,
///  mix and conversion functions a fit calls.
/// Why:
///      The search runs in OKLCH and every check runs on the quantized 8-bit color,
///  so both types and their conversions come from the one color module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Oklch, Srgb8, compositeOver, contrastRatio, mixOklch, oklchToSrgb8, srgbToOklch } from "./oklch";
/// ```
use crate::oklch::{
    Oklch,
    Srgb8,
    composite_over,
    contrast_ratio,
    mix_oklch,
    oklch_to_srgb8,
    srgb_to_oklch,
};

/// What:
///      `pub trait FitCheck { fn accepts(&self, candidate: Srgb8) -> bool; }`:
/// the contrast test that a lightness fit moves a color until it passes.
/// Why:
///       Each role is fitted against its own surfaces and layers,
/// so the test is a value that carries them (`FitRule`),
/// and a plain function of the color is a test too,
/// which the unit tests use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface FitCheck { accepts(candidate: Srgb8): boolean }
/// ```
pub trait FitCheck {
    /// What:
    /// `accepts` is true when the 8-bit color meets the role's contrast targets.
    /// Why:
    /// the fit keeps the closest lightness that it accepts.
    fn accepts(
        &self,
        candidate: Srgb8,
    ) -> bool;
}

/// A plain function of the color is a contrast test.
impl<F: Fn(Srgb8) -> bool> FitCheck for F {
    /// What:
    /// `accepts` calls the function.
    /// Why:
    /// it lets a test pass a function.
    fn accepts(
        &self,
        candidate: Srgb8,
    ) -> bool {
        return self(candidate);
    }
}

/// What:
///      `pub enum FitRule`:
/// the contrast test of each role that `derive_scheme` fits,
/// with the colors it is measured against.
/// Why:
///       Every fit passes a different set of surfaces and layers,
/// and a value that names the role and carries the colors replaces a closure that captured them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FitRule =
///   | { kind: "container"; container: Srgb8 }
///   | { kind: "legible"; surfaces: Srgb8[]; layer: Srgb8 | null }
///   | { kind: "outline"; canvas: Srgb8; deck: Srgb8 }
///   | { kind: "primary"; surfaces: Srgb8[]; onSurface: Srgb8; fills: Srgb8[]; lightOnPrimary: Srgb8 | null }
///   | { kind: "matchFill"; onSurface: Srgb8; onSurfaceVariant: Srgb8 };
/// ```
pub enum FitRule<'surfaces> {
    /// Text on the selected fill:
    /// legible on the fill and under its hover and pressed layers.
    Container {
        /// The selected fill.
        container: Srgb8,
    },
    /// Text legible on every surface and under the text layers over each.
    Legible {
        /// The surfaces the text is drawn on.
        surfaces: &'surfaces [Srgb8],
        /// The layer color drawn over a surface,
        /// or `None` when it is the text color itself.
        layer: Option<Srgb8>,
    },
    /// An outline that holds the non-text target against the canvas and the deck.
    Outline {
        /// The canvas surface.
        canvas: Srgb8,
        /// The deck surface.
        deck: Srgb8,
    },
    /// The accent text:
    /// legible on every surface,
    /// on the selected fill states,
    /// and against the light on-primary content when the scheme has one.
    Primary {
        /// The surfaces the text is drawn on.
        surfaces: &'surfaces [Srgb8],
        /// The text color whose layers are drawn over the surfaces.
        on_surface: Srgb8,
        /// The selected fill and its hover and pressed states.
        fills: [Srgb8; 3],
        /// The fixed content color of a light scheme's primary,
        /// when there is one.
        light_on_primary: Option<Srgb8>,
    },
    /// The match fill:
    /// both text colors read on it.
    MatchFill {
        /// The main text color.
        on_surface: Srgb8,
        /// The secondary text color.
        on_surface_variant: Srgb8,
    },
}

/// The contrast test of one role.
impl FitCheck for FitRule<'_> {
    /// What:
    /// `accepts` runs the role's own contrast checks on the 8-bit candidate.
    /// Why:
    /// the lightness fit asks every role the same question.
    fn accepts(
        &self,
        candidate: Srgb8,
    ) -> bool {
        match self {
            FitRule::Container {
                container,
            } => {
                let (hovered, pressed) = container_layers(*container, candidate);
                return contrast_ratio(candidate, *container) >= TEXT_TARGET_RATIO
                    && contrast_ratio(candidate, hovered) >= TEXT_TARGET_RATIO
                    && contrast_ratio(candidate, pressed) >= TEXT_TARGET_RATIO;
            }
            FitRule::Legible {
                surfaces,
                layer,
            } => {
                return legible_on(
                    candidate,
                    surfaces,
                    layer.unwrap_or(candidate),
                    &TEXT_LAYER_ALPHAS,
                    TEXT_TARGET_RATIO,
                );
            }
            FitRule::Outline {
                canvas,
                deck,
            } => {
                return contrast_ratio(candidate, *canvas) >= NON_TEXT_TARGET_RATIO
                    && contrast_ratio(candidate, *deck) >= NON_TEXT_TARGET_RATIO;
            }
            FitRule::Primary {
                surfaces,
                on_surface,
                fills,
                light_on_primary,
            } => {
                if !legible_on(
                    candidate,
                    surfaces,
                    *on_surface,
                    &TEXT_LAYER_ALPHAS,
                    TEXT_TARGET_RATIO,
                ) {
                    return false;
                }
                for fill in fills {
                    if contrast_ratio(candidate, *fill) < TEXT_TARGET_RATIO {
                        return false;
                    }
                }
                if let Some(content) = light_on_primary {
                    return contrast_ratio(candidate, *content) >= TEXT_TARGET_RATIO;
                }
                return true;
            }
            FitRule::MatchFill {
                on_surface,
                on_surface_variant,
            } => {
                return contrast_ratio(*on_surface, candidate) >= TEXT_TARGET_RATIO
                    && contrast_ratio(*on_surface_variant, candidate) >= TEXT_TARGET_RATIO;
            }
        }
    }
}

/// What:
///      `pub fn fit_lightness(name, start, endpoint, rule) -> Oklch`:
/// moves `start` in lightness toward `endpoint`
///  (0 black or 1 white),
/// keeping chroma and hue,
/// until `rule` accepts the 8-bit result;
/// returns the closest
///  accepted lightness to `start`.
/// Why:
///       This is the contrast guarantee:
/// contrast depends on luminance,
/// so lightness is the one coordinate moved,
///           and the rule always sees the quantized color that reaches the screen.
/// If even the endpoint fails,
/// the
///           endpoint is returned and a warning logged,
/// because no lightness can do better.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (passes(toSrgb8(start))) return start;
/// let [failing, passing] = [start.lightness, endpoint];
/// for (let i = 0; i < 40; i++) { const mid = (failing + passing) / 2; passes(toSrgb8({ ...start, lightness: mid })) ? (passing = mid) : (failing = mid); }
/// return { ...start, lightness: passing };
/// ```
pub fn fit_lightness(
    name: &str,
    start: Oklch,
    endpoint: f64,
    rule: &impl FitCheck,
) -> Oklch {
    if rule.accepts(oklch_to_srgb8(start)) {
        return start;
    }
    let at_endpoint: Oklch = Oklch {
        lightness: endpoint,
        ..start
    };
    if !rule.accepts(oklch_to_srgb8(at_endpoint)) {
        tracing::warn!(
            role = name,
            endpoint,
            "no lightness reaches the contrast target, using the endpoint"
        );
        return at_endpoint;
    }
    let mut failing: f64 = start.lightness;
    let mut passing: f64 = endpoint;
    for _step in 0..40 {
        let middle: f64 = (failing + passing) / 2.0;
        if rule.accepts(oklch_to_srgb8(Oklch {
            lightness: middle,
            ..start
        })) {
            passing = middle;
        } else {
            failing = middle;
        }
    }
    tracing::debug!(
        role = name,
        from = start.lightness,
        to = passing,
        "moved lightness to reach the contrast target"
    );
    return Oklch {
        lightness: passing,
        ..start
    };
}

/// What:
///      `fn legible_on(content, backgrounds, layer, layer_alphas, target) -> bool`:
/// true when `content` reaches
///  `target` against every background,
/// and also against that background with `layer` at each of `layer_alphas` over it.
/// Why:
///       Text is drawn on a surface and also under a hover,
/// pressed or focus layer (alpha-only,
/// D108);
/// the caller
///           passes every layer strength the text must survive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return backgrounds.every(b => ratio(content, b) >= target
///   && alphas.every(a => ratio(content, compositeOver(layer, a, b)) >= target));
/// ```
fn legible_on(
    content: Srgb8,
    backgrounds: &[Srgb8],
    layer: Srgb8,
    layer_alphas: &[f64],
    target: f64,
) -> bool {
    for background in backgrounds {
        if contrast_ratio(content, *background) < target {
            return false;
        }
        for layer_alpha in layer_alphas {
            let layered: Srgb8 = composite_over(layer, *layer_alpha, *background);
            if contrast_ratio(content, layered) < target {
                return false;
            }
        }
    }
    return true;
}

/// What:
///      `pub fn container_layers(container, content) -> (Srgb8, Srgb8)`:
/// the selected fill under the hover and under the
///  pressed state,
/// moved from the container toward its content color in OKLCH.
/// A tuple `(A, B)` is a fixed pair.
/// Why:
///       Replaces the sRGB `mix` the segmented button used;
/// the move is in lightness,
/// chroma and hue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return [toSrgb8(mixOklch(c, on, 0.08)), toSrgb8(mixOklch(c, on, 0.12))];
/// ```
pub fn container_layers(
    container: Srgb8,
    content: Srgb8,
) -> (Srgb8, Srgb8) {
    let from: Oklch = srgb_to_oklch(container);
    let to: Oklch = srgb_to_oklch(content);
    let hovered: Srgb8 = oklch_to_srgb8(mix_oklch(from, to, HOVER_ALPHA));
    let pressed: Srgb8 = oklch_to_srgb8(mix_oklch(from, to, PRESSED_ALPHA));
    return (hovered, pressed);
}
