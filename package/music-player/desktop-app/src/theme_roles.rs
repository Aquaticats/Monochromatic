//! The theme color roles,
//! derived from one root color in OKLCH (D108,
//! A3).
//!
//! What:
//! `derive_roles` turns the root (the system accent,
//! or the Fold baseline when none exists) into every color
//! role the window uses,
//! for the dark and the light scheme.
//! Each role has a lightness target,
//! a chroma share and a
//! hue offset,
//! all moved in OKLCH (`oklch.rs`) and converted to 8-bit sRGB with chroma-reduction gamut mapping.
//! Why:
//! one pure module owns every derived color,
//! so it is unit-testable (hue sweeps,
//! contrast,
//! gamut) and the
//! window only receives finished values (`ui_theme.rs`).
//! Grays (D118):
//! every gray role (surfaces,
//! deck,
//! outlines,
//! text variants) carries the root's hue at its own fixed
//! tint,
//! whatever the root's chroma,
//! unless the root is completely neutral (its three 8-bit channels are equal);
//! then every gray role is a true gray.
//! Accent roles still scale with the root's chroma,
//! but for a root that is not
//! completely neutral they never fall below the largest gray tint of their scheme,
//! so an accent is never grayer than
//! the surfaces (the orchestrator's reading of D118,
//! open to the human's veto).
//! Fixed on purpose:
//! the dark and light canvas are pure black (D41) and pure white (D34,
//! D45),
//! which are not grays
//! and cannot carry a tint;
//! the error roles are semantic constants that follow no accent.
//! Contrast guarantee:
//! after the lightness targets are placed,
//! every text role is moved in lightness (chroma and
//! hue kept,
//! gamut-mapped again) until its WCAG ratio holds on every surface and state layer it is drawn on,
//! so AA
//! holds for ANY root color.
//! The baseline passes without any adjustment.

/// Import the OKLCH color math the derivation moves colors through.
use crate::oklch::{
    Oklch, Srgb8, composite_over, contrast_ratio, mix_oklch, oklch_to_srgb8, srgb_to_oklch,
};

/// What:
///      `pub const BASELINE_ROOT: Srgb8`:
/// the Fold capture primary of the light scheme,
/// `#4E5E8B`.
/// Why:
///       D108 names the fixed baseline from the Fold captures as the root when no accent exists;
/// the role table
///           below was fitted so that this root reproduces the captured values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BASELINE_ROOT = { red: 0x4e, green: 0x5e, blue: 0x8b };
/// ```
pub const BASELINE_ROOT: Srgb8 = Srgb8 { red: 0x4E, green: 0x5E, blue: 0x8B };

/// Largest root chroma that scales the roles,
/// so a neon accent cannot turn every container garish.
pub const ROOT_CHROMA_CAP: f64 = 0.15;

/// Contrast ratio the derivation fits text to;
/// AA asks 4.5,
/// and the small margin keeps another renderer's
/// rounding of a translucent layer (one 8-bit step) from dropping below it.
pub const TEXT_TARGET_RATIO: f64 = 4.52;

/// Contrast ratio the derivation fits UI parts (outlines) to;
/// AA for non-text asks 3.
pub const NON_TEXT_TARGET_RATIO: f64 = 3.02;

/// Alpha of the hover layer (Material 3 state layer).
pub const HOVER_ALPHA: f64 = 0.08;

/// Alpha of the pressed layer (Material 3 state layer).
/// The focus layer uses the same strength and lasts as long as
/// the keyboard focus does,
/// so text must hold AA under it,
/// not only while a button is held.
pub const PRESSED_ALPHA: f64 = 0.12;

/// Layer strengths text must survive:
/// hover,
/// then pressed (which is also the focus strength).
pub const TEXT_LAYER_ALPHAS: [f64; 2] = [HOVER_ALPHA, PRESSED_ALPHA];

/// Search match fill in the light scheme:
/// lightness share moved toward white (mirrors Android `SEARCH_LIGHT_WHITE_MIX`).
const MATCH_LIGHT_LIGHTNESS_SHARE: f64 = 0.72;
/// Search match fill in the light scheme:
/// chroma share removed (mirrors Android `SEARCH_LIGHT_CHROMA_MIX`).
const MATCH_LIGHT_CHROMA_SHARE: f64 = 0.55;
/// Search match fill in the dark scheme:
/// lightness share moved toward black (mirrors Android `SEARCH_DARK_BLACK_MIX`).
const MATCH_DARK_LIGHTNESS_SHARE: f64 = 0.62;
/// Search match fill in the dark scheme:
/// chroma share removed (mirrors Android `SEARCH_DARK_CHROMA_MIX`).
const MATCH_DARK_CHROMA_SHARE: f64 = 0.35;

/// What:
///      `pub enum Scheme { Dark, Light }`.
/// Why:
///       The two schemes use different lightness targets and move text in opposite directions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scheme = "dark" | "light";
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme {
    /// Dark scheme,
    /// true-black canvas.
    Dark,
    /// Light scheme,
    /// white canvas.
    Light,
}

/// What:
///      `mod theme_roles_declaration;` declares the role declaration module:
///      the `RoleSet` fields,
/// the fixture names and both scheme's role tables.
/// Why:
///       Split from this module so the declaration keeps its own line budget;
///       the derivation below reads the finished tables through the imports that follow.
#[path = "theme_roles_declaration.rs"]
mod theme_roles_declaration;

/// What:
///      `pub use theme_roles_declaration::RoleSet;` re-exports the finished-color set at this module's path.
/// Why:
///       callers name `theme_roles::RoleSet`;
///       the struct is declared with the roles themselves in the declaration module.
pub use theme_roles_declaration::RoleSet;

/// What:
///      `use theme_roles_declaration::{...}` brings the role tables here.
/// Why:
///       `derive_scheme` reads the scheme tables and builds each role from them.
use theme_roles_declaration::{
    DARK_SPEC, LIGHT_SPEC, Role, SchemeSpec, gray_tint_ceiling,
};

/// What:
///      `pub struct ThemePalette { dark, light }`:
/// both schemes derived from one root.
/// Why:
///       The window picks between them itself,
/// so a scheme change needs no Rust round trip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ThemePalette = { dark: RoleSet; light: RoleSet };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePalette {
    /// Roles of the dark scheme.
    pub dark: RoleSet,
    /// Roles of the light scheme.
    pub light: RoleSet,
}

/// What:
///      `struct RoleSpec { lightness, chroma, hue_offset }`:
/// where one role sits relative to the root,
/// measured
///  when the root is the baseline.
/// Why:
///       `chroma` is the role's chroma at the baseline root and `hue_offset` its hue minus the root hue in degrees,
///           so the role keeps its relation to the root for any accent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RoleSpec = { lightness: number; chroma: number; hueOffset: number };
/// ```
#[derive(Clone, Copy, Debug)]
struct RoleSpec {
    /// The role's lightness target in OKLCH.
    lightness: f64,
    /// The role's chroma at the baseline root.
    chroma: f64,
    /// The role's hue offset in degrees.
    hue_offset: f64,
}

/// What:
///      `struct RootTerms { hue, scale, completely_neutral, accent_chroma_floor }`:
/// the things the derivation takes
///  from the root.
/// Why:
///       `hue` places every role on the color wheel,
/// `scale` is the root chroma relative to the baseline's,
/// capped by
///           `ROOT_CHROMA_CAP`,
/// and `completely_neutral` (D118) is true only when the root's three 8-bit channels are
///           equal;
/// the root's own lightness is never used,
/// since each role has its own target.
///           `accent_chroma_floor` is the least chroma an accent role may have:
/// `root_terms` leaves it 0 because it
///           depends on the scheme,
/// and `derive_scheme` sets it to the scheme's largest gray tint unless the root is
///           completely neutral.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RootTerms = { hue: number; scale: number; completelyNeutral: boolean; accentChromaFloor: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RootTerms {
    /// Root hue in degrees,
    /// 0 up to 360.
    pub hue: f64,
    /// Root chroma over the baseline chroma,
    /// capped;
    /// exactly 0 for a completely neutral root.
    pub scale: f64,
    /// True only when the root's red,
    /// green and blue are equal;
    /// then every gray role has chroma 0.
    pub completely_neutral: bool,
    /// Least chroma of any accent role;
    /// set per scheme by `derive_scheme`,
    /// 0 from `root_terms`.
    pub accent_chroma_floor: f64,
}

/// What:
///      `pub fn root_terms(root: Option<Srgb8>) -> RootTerms`:
/// reads the root;
/// `None` (no system accent) is the
///  baseline.
/// `Option` is Rust's "value or nothing",
/// like `T | undefined`.
/// Why:
///       D118:
/// a root is completely neutral only when its three 8-bit channels are equal,
/// which is exactly "no chroma
///           at all" for a stored color.
/// Equality is the test,
/// not a chroma threshold:
/// the 256 equal-channel roots
///           measure at most 3.7e-8 in OKLCH chroma (float noise) and the least chromatic unequal root,
/// #FEFFFF,
///           measures 1.06e-3,
/// so any threshold between them agrees with equality,
/// and equality needs no tuned
///           constant that Rust and Kotlin could disagree on.
/// A neutral root has no hue (the baseline hue is kept so
///           the value is deterministic) and scale 0;
/// any other root,
/// however weakly chromatic,
/// keeps its own hue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const used = root ?? BASELINE_ROOT;
/// if (used.red === used.green && used.green === used.blue) return { hue: baselineHue, scale: 0, completelyNeutral: true };
/// const lch = toOklch(used);
/// return { hue: lch.hue, scale: Math.min(lch.chroma, 0.15) / baselineChroma, completelyNeutral: false };
/// ```
pub fn root_terms(root: Option<Srgb8>) -> RootTerms {
    let baseline: Oklch = srgb_to_oklch(BASELINE_ROOT);
    let used: Srgb8 = root.unwrap_or(BASELINE_ROOT);
    if used.red == used.green && used.green == used.blue {
        tracing::debug!(channel = used.red, "the root is completely neutral, every gray role is a true gray");
        return RootTerms { hue: baseline.hue, scale: 0.0, completely_neutral: true, accent_chroma_floor: 0.0 };
    }
    let lch: Oklch = srgb_to_oklch(used);
    let scale: f64 = lch.chroma.min(ROOT_CHROMA_CAP) / baseline.chroma;
    return RootTerms { hue: lch.hue, scale, completely_neutral: false, accent_chroma_floor: 0.0 };
}

/// What:
///      `fn resolve_oklch(role: Role, terms: RootTerms) -> Oklch`:
/// the role's target color before gamut mapping.
/// Why:
///       `Neutral` chroma is the role's own tint for every root that is not completely neutral (D118:
/// a weak root does
///           not shrink it and a vivid root does not grow it) and 0 for a completely neutral root;
/// `Accent` chroma
///           scales with the root in both directions but never falls below `accent_chroma_floor`,
/// so for a root with
///           any chroma an accent is never grayer than the grays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// neutral: { lightness, chroma: terms.completelyNeutral ? 0 : chroma, hue: (rootHue + hueOffset) % 360 }
/// accent:  { lightness, chroma: Math.max(chroma * terms.scale, terms.accentChromaFloor), hue: (rootHue + hueOffset) % 360 }
/// ```
fn resolve_oklch(role: Role, terms: RootTerms) -> Oklch {
    return match role {
        Role::Fixed(color) => srgb_to_oklch(color),
        Role::Neutral(spec) => Oklch {
            lightness: spec.lightness,
            chroma: if terms.completely_neutral { 0.0 } else { spec.chroma },
            hue: (terms.hue + spec.hue_offset).rem_euclid(360.0),
        },
        Role::Accent(spec) => Oklch {
            lightness: spec.lightness,
            chroma: (spec.chroma * terms.scale).max(terms.accent_chroma_floor),
            hue: (terms.hue + spec.hue_offset).rem_euclid(360.0),
        },
    };
}

/// What:
///      `fn resolve_srgb(role: Role, terms: RootTerms) -> Srgb8`:
/// the finished color of a role without any contrast fit.
/// Why:
///       `Fixed` colors are used as written (no OKLCH round trip),
/// the rest go through `oklch_to_srgb8`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return "fixed" in role ? role.fixed : oklchToSrgb8(resolveOklch(role, terms));
/// ```
fn resolve_srgb(role: Role, terms: RootTerms) -> Srgb8 {
    if let Role::Fixed(color) = role {
        return color;
    }
    return oklch_to_srgb8(resolve_oklch(role, terms));
}

/// What:
///      `trait FitCheck { fn accepts(&self, candidate: Srgb8) -> bool; }`:
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
trait FitCheck {
    /// What:
    /// `accepts` is true when the 8-bit color meets the role's contrast targets.
    /// Why:
    /// the fit keeps the closest lightness that it accepts.
    fn accepts(&self, candidate: Srgb8) -> bool;
}

/// A plain function of the color is a contrast test.
impl<F: Fn(Srgb8) -> bool> FitCheck for F {
    /// What:
    /// `accepts` calls the function.
    /// Why:
    /// it lets a test pass a function.
    fn accepts(&self, candidate: Srgb8) -> bool {
        return self(candidate);
    }
}

/// What:
///      `enum FitRule`:
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
enum FitRule<'surfaces> {
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
    fn accepts(&self, candidate: Srgb8) -> bool {
        match self {
            FitRule::Container { container } => {
                let (hovered, pressed) = container_layers(*container, candidate);
                return contrast_ratio(candidate, *container) >= TEXT_TARGET_RATIO
                    && contrast_ratio(candidate, hovered) >= TEXT_TARGET_RATIO
                    && contrast_ratio(candidate, pressed) >= TEXT_TARGET_RATIO;
            }
            FitRule::Legible { surfaces, layer } => {
                return legible_on(candidate, surfaces, layer.unwrap_or(candidate), &TEXT_LAYER_ALPHAS, TEXT_TARGET_RATIO);
            }
            FitRule::Outline { canvas, deck } => {
                return contrast_ratio(candidate, *canvas) >= NON_TEXT_TARGET_RATIO
                    && contrast_ratio(candidate, *deck) >= NON_TEXT_TARGET_RATIO;
            }
            FitRule::Primary { surfaces, on_surface, fills, light_on_primary } => {
                if !legible_on(candidate, surfaces, *on_surface, &TEXT_LAYER_ALPHAS, TEXT_TARGET_RATIO) {
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
            FitRule::MatchFill { on_surface, on_surface_variant } => {
                return contrast_ratio(*on_surface, candidate) >= TEXT_TARGET_RATIO
                    && contrast_ratio(*on_surface_variant, candidate) >= TEXT_TARGET_RATIO;
            }
        }
    }
}

/// What:
///      `fn fit_lightness(name, start, endpoint, rule) -> Oklch`:
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
fn fit_lightness(name: &str, start: Oklch, endpoint: f64, rule: &impl FitCheck) -> Oklch {
    if rule.accepts(oklch_to_srgb8(start)) {
        return start;
    }
    let at_endpoint: Oklch = Oklch { lightness: endpoint, ..start };
    if !rule.accepts(oklch_to_srgb8(at_endpoint)) {
        tracing::warn!(role = name, endpoint, "no lightness reaches the contrast target, using the endpoint");
        return at_endpoint;
    }
    let mut failing: f64 = start.lightness;
    let mut passing: f64 = endpoint;
    for _step in 0..40 {
        let middle: f64 = (failing + passing) / 2.0;
        if rule.accepts(oklch_to_srgb8(Oklch { lightness: middle, ..start })) {
            passing = middle;
        } else {
            failing = middle;
        }
    }
    tracing::debug!(role = name, from = start.lightness, to = passing, "moved lightness to reach the contrast target");
    return Oklch { lightness: passing, ..start };
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
fn legible_on(content: Srgb8, backgrounds: &[Srgb8], layer: Srgb8, layer_alphas: &[f64], target: f64) -> bool {
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
///      `fn container_layers(container, content) -> (Srgb8, Srgb8)`:
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
fn container_layers(container: Srgb8, content: Srgb8) -> (Srgb8, Srgb8) {
    let from: Oklch = srgb_to_oklch(container);
    let to: Oklch = srgb_to_oklch(content);
    let hovered: Srgb8 = oklch_to_srgb8(mix_oklch(from, to, HOVER_ALPHA));
    let pressed: Srgb8 = oklch_to_srgb8(mix_oklch(from, to, PRESSED_ALPHA));
    return (hovered, pressed);
}

/// What:
///      `pub fn derive_scheme(from_root: RootTerms, scheme: Scheme) -> RoleSet`:
/// every role of one scheme.
/// Why:
///       Places each role from its target,
/// then fits lightness so each text pair holds AA:
/// content on its surface
///           and layers,
/// accent text on every surface it is drawn on,
/// outlines against the canvas and deck.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spec = scheme === "dark" ? DARK_SPEC : LIGHT_SPEC;
/// const surfaces = [canvas, deck, currentRow, surfaceHigh].map(resolve);
/// // fit onSecondaryContainer, onSurface, onSurfaceVariant, outline, primary, onPrimary, then derive matchFill
/// ```
pub fn derive_scheme(from_root: RootTerms, scheme: Scheme) -> RoleSet {
    tracing::debug!(
        hue = from_root.hue,
        scale = from_root.scale,
        completely_neutral = from_root.completely_neutral,
        scheme = ?scheme,
        "deriving the scheme roles"
    );
    let spec: SchemeSpec = match scheme {
        Scheme::Dark => DARK_SPEC,
        Scheme::Light => LIGHT_SPEC,
    };
    // Text moves toward white on a dark scheme and toward black on a light one to gain contrast.
    let toward_text: f64 = if scheme == Scheme::Dark { 1.0 } else { 0.0 };
    // D118 reading: every accent role is at least as chromatic as the strongest gray of this scheme, unless the root
    // is completely neutral, where every accent stays at chroma 0.
    let accent_chroma_floor: f64 = if from_root.completely_neutral { 0.0 } else { gray_tint_ceiling(&spec) };
    let terms: RootTerms = RootTerms { accent_chroma_floor, ..from_root };
    let away_from_text: f64 = 1.0 - toward_text;

    let canvas: Srgb8 = resolve_srgb(spec.canvas, terms);
    let deck: Srgb8 = resolve_srgb(spec.deck, terms);
    let current_row: Srgb8 = resolve_srgb(spec.current_row, terms);
    let surface_high: Srgb8 = resolve_srgb(spec.surface_high, terms);
    // Settings has no surface of its own (D109): it is drawn on the canvas, so no further surface needs fitting.
    let surfaces: [Srgb8; 4] = [canvas, deck, current_row, surface_high];

    // Selected fill and its content: the content is fitted against the fill and both state layers.
    let secondary_container: Srgb8 = resolve_srgb(spec.secondary_container, terms);
    let on_container_start: Oklch = resolve_oklch(spec.on_secondary_container, terms);
    let on_container_fit: Oklch = fit_lightness("on-secondary-container", on_container_start, toward_text, &FitRule::Container { container: secondary_container });
    let on_secondary_container: Srgb8 = oklch_to_srgb8(on_container_fit);
    let (secondary_container_hovered, secondary_container_pressed) =
        container_layers(secondary_container, on_secondary_container);

    // Neutral text on every surface, under the on-surface hover and pressed layers. Secondary text below holds under
    // the same layers, tinted with on-surface.
    let on_surface_fit: Oklch = fit_lightness("on-surface", resolve_oklch(spec.on_surface, terms), toward_text, &FitRule::Legible { surfaces: &surfaces, layer: None });
    let on_surface: Srgb8 = oklch_to_srgb8(on_surface_fit);
    let variant_fit: Oklch = fit_lightness(
        "on-surface-variant",
        resolve_oklch(spec.on_surface_variant, terms),
        toward_text,
        &FitRule::Legible { surfaces: &surfaces, layer: Some(on_surface) },
    );
    let on_surface_variant: Srgb8 = oklch_to_srgb8(variant_fit);

    // Outlines are UI parts: 3:1 against the canvas and the deck they border.
    let outline_fit: Oklch = fit_lightness("outline", resolve_oklch(spec.outline, terms), toward_text, &FitRule::Outline { canvas, deck });
    let outline: Srgb8 = oklch_to_srgb8(outline_fit);

    // Primary is both a fill and text (rail letter, links, action labels), so it holds AA on every surface and under
    // the hover and pressed (also focus) layers, on the selected fill at rest and under the layers the rail cell draws
    // over it, and against the light scheme's white on-primary. The OKLCH hovered and pressed fills carry only
    // on-secondary-container text (the deck segment), so primary is not fitted to them.
    let selected_fill_states: [Srgb8; 3] = [
        secondary_container,
        composite_over(on_secondary_container, HOVER_ALPHA, secondary_container),
        composite_over(on_secondary_container, PRESSED_ALPHA, secondary_container),
    ];
    let primary_start: Oklch = resolve_oklch(spec.primary, terms);
    let light_on_primary: Option<Srgb8> = match scheme {
        Scheme::Dark => None,
        Scheme::Light => Some(resolve_srgb(spec.on_primary, terms)),
    };
    let primary_fit: Oklch = fit_lightness("primary", primary_start, toward_text, &FitRule::Primary { surfaces: &surfaces, on_surface, fills: selected_fill_states, light_on_primary });
    let primary: Srgb8 = oklch_to_srgb8(primary_fit);

    // On-primary is fitted against the primary fill and its own pressed layer (the first-run button).
    let on_primary: Srgb8 = match light_on_primary {
        Some(content) => content,
        None => oklch_to_srgb8(fit_lightness("on-primary", resolve_oklch(spec.on_primary, terms), away_from_text, &FitRule::Legible { surfaces: &[primary], layer: None })),
    };

    // Search match fill (D59): the primary moved toward the neutral end in OKLCH, with the Android shares, then
    // fitted so both text roles stay legible on it.
    let (lightness_share, chroma_share, neutral_end): (f64, f64, f64) = match scheme {
        Scheme::Dark => (MATCH_DARK_LIGHTNESS_SHARE, MATCH_DARK_CHROMA_SHARE, 0.0),
        Scheme::Light => (MATCH_LIGHT_LIGHTNESS_SHARE, MATCH_LIGHT_CHROMA_SHARE, 1.0),
    };
    let primary_lch: Oklch = srgb_to_oklch(primary);
    let match_start: Oklch = Oklch {
        lightness: primary_lch.lightness + (neutral_end - primary_lch.lightness) * lightness_share,
        chroma: (primary_lch.chroma * (1.0 - chroma_share)).max(terms.accent_chroma_floor),
        hue: primary_lch.hue,
    };
    let match_fit: Oklch = fit_lightness("match-fill", match_start, neutral_end, &FitRule::MatchFill { on_surface, on_surface_variant });
    let match_fill: Srgb8 = oklch_to_srgb8(match_fit);

    return RoleSet {
        canvas,
        deck,
        current_row,
        surface_high,
        on_surface,
        on_surface_variant,
        outline,
        outline_variant: resolve_srgb(spec.outline_variant, terms),
        primary,
        on_primary,
        secondary_container,
        secondary_container_hovered,
        secondary_container_pressed,
        on_secondary_container,
        match_fill,
        error: resolve_srgb(spec.error, terms),
        error_container: resolve_srgb(spec.error_container, terms),
        on_error_container: resolve_srgb(spec.on_error_container, terms),
    };
}

/// What:
///      `pub fn derive_roles(root: Option<Srgb8>) -> ThemePalette`:
/// both schemes from one root;
/// `None` is the baseline.
/// Why:
///       The single entry point the window glue calls at startup and on every accent change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const terms = rootTerms(root);
/// return { dark: deriveScheme(terms, "dark"), light: deriveScheme(terms, "light") };
/// ```
pub fn derive_roles(root: Option<Srgb8>) -> ThemePalette {
    tracing::info!(root = ?root, "deriving the theme roles from the root color");
    let terms: RootTerms = root_terms(root);
    return ThemePalette {
        dark: derive_scheme(terms, Scheme::Dark),
        light: derive_scheme(terms, Scheme::Light),
    };
}

/// What:
///      `#[cfg(test)] mod tests;` declares the derivation's unit-test sidecar.
/// Why:
///       sidecar tests keep this module's line budget for production code.
#[cfg(test)]
#[path = "theme_roles_tests.rs"]
mod tests;

/// What:
///      `#[cfg(test)] mod fixture_tests;` declares the shared-fixture drift-test sidecar.
/// Why:
///       the sidecar reads the Android theme vector beside the derivation it guards.
#[cfg(test)]
#[path = "theme_roles_fixture_tests.rs"]
mod fixture_tests;
