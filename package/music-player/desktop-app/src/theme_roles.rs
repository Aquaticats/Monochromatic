//! The theme color roles, derived from one root color in OKLCH (D108, A3).
//!
//! What: `derive_roles` turns the root (the system accent, or the Fold baseline when none exists) into every color
//! role the window uses, for the dark and the light scheme. Each role has a lightness target, a chroma share and a
//! hue offset, all moved in OKLCH (`oklch.rs`) and converted to 8-bit sRGB with chroma-reduction gamut mapping.
//! Why: one pure module owns every derived color, so it is unit-testable (hue sweeps, contrast, gamut) and the
//! window only receives finished values (`ui_theme.rs`).
//! Grays (D118): every gray role (surfaces, deck, outlines, text variants) carries the root's hue at its own fixed
//! tint, whatever the root's chroma, unless the root is completely neutral (its three 8-bit channels are equal);
//! then every gray role is a true gray. Accent roles still scale with the root's chroma, but for a root that is not
//! completely neutral they never fall below the largest gray tint of their scheme, so an accent is never grayer than
//! the surfaces (the orchestrator's reading of D118, open to the human's veto).
//! Fixed on purpose: the dark and light canvas are pure black (D41) and pure white (D34, D45), which are not grays
//! and cannot carry a tint; the error roles are semantic constants that follow no accent.
//! Contrast guarantee: after the lightness targets are placed, every text role is moved in lightness (chroma and
//! hue kept, gamut-mapped again) until its WCAG ratio holds on every surface and state layer it is drawn on, so AA
//! holds for ANY root color. The baseline passes without any adjustment.

use crate::oklch::{
    Oklch, Srgb8, composite_over, contrast_ratio, mix_oklch, oklch_to_srgb8, srgb_to_oklch,
};

/// What:
///      `pub const BASELINE_ROOT: Srgb8`: the Fold capture primary of the light scheme, `#4E5E8B`.
/// Why:
///       D108 names the fixed baseline from the Fold captures as the root when no accent exists; the role table
///           below was fitted so that this root reproduces the captured values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BASELINE_ROOT = { red: 0x4e, green: 0x5e, blue: 0x8b };
/// ```
pub const BASELINE_ROOT: Srgb8 = Srgb8 { red: 0x4E, green: 0x5E, blue: 0x8B };

/// Largest root chroma that scales the roles, so a neon accent cannot turn every container garish.
pub const ROOT_CHROMA_CAP: f64 = 0.15;

/// Contrast ratio the derivation fits text to; AA asks 4.5, and the small margin keeps another renderer's
/// rounding of a translucent layer (one 8-bit step) from dropping below it.
pub const TEXT_TARGET_RATIO: f64 = 4.52;

/// Contrast ratio the derivation fits UI parts (outlines) to; AA for non-text asks 3.
pub const NON_TEXT_TARGET_RATIO: f64 = 3.02;

/// Alpha of the hover layer (Material 3 state layer).
pub const HOVER_ALPHA: f64 = 0.08;

/// Alpha of the pressed layer (Material 3 state layer). The focus layer uses the same strength and lasts as long as
/// the keyboard focus does, so text must hold AA under it, not only while a button is held.
pub const PRESSED_ALPHA: f64 = 0.12;

/// Layer strengths text must survive: hover, then pressed (which is also the focus strength).
pub const TEXT_LAYER_ALPHAS: [f64; 2] = [HOVER_ALPHA, PRESSED_ALPHA];

/// Search match fill in the light scheme: lightness share moved toward white (mirrors Android `SEARCH_LIGHT_WHITE_MIX`).
const MATCH_LIGHT_LIGHTNESS_SHARE: f64 = 0.72;
/// Search match fill in the light scheme: chroma share removed (mirrors Android `SEARCH_LIGHT_CHROMA_MIX`).
const MATCH_LIGHT_CHROMA_SHARE: f64 = 0.55;
/// Search match fill in the dark scheme: lightness share moved toward black (mirrors Android `SEARCH_DARK_BLACK_MIX`).
const MATCH_DARK_LIGHTNESS_SHARE: f64 = 0.62;
/// Search match fill in the dark scheme: chroma share removed (mirrors Android `SEARCH_DARK_CHROMA_MIX`).
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
    /// Dark scheme, true-black canvas.
    Dark,
    /// Light scheme, white canvas.
    Light,
}

/// What:
///      `pub struct RoleSet { ... }`: every color role of one scheme as finished sRGB.
/// Why:
///       The window receives one of these per scheme; names mirror the Slint `RoleSet` struct field for field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RoleSet = { canvas: Srgb8; deck: Srgb8; /* ... one field per role ... */ };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoleSet {
    /// Page background.
    pub canvas: Srgb8,
    /// Playback deck background.
    pub deck: Srgb8,
    /// Background of the playing row.
    pub current_row: Srgb8,
    /// Menus, search header and snackbar surface.
    pub surface_high: Srgb8,
    /// Primary text and icons.
    pub on_surface: Srgb8,
    /// Secondary text and icons.
    pub on_surface_variant: Srgb8,
    /// Borders and strong dividers.
    pub outline: Srgb8,
    /// Hairline dividers.
    pub outline_variant: Srgb8,
    /// Accent fill and accent text.
    pub primary: Srgb8,
    /// Content on the accent fill.
    pub on_primary: Srgb8,
    /// Selected segment and rail cell fill.
    pub secondary_container: Srgb8,
    /// Selected fill under the hover layer.
    pub secondary_container_hovered: Srgb8,
    /// Selected fill under the pressed layer.
    pub secondary_container_pressed: Srgb8,
    /// Content on the selected fill.
    pub on_secondary_container: Srgb8,
    /// Search match highlight (D59), opaque.
    pub match_fill: Srgb8,
    /// Error text and icons.
    pub error: Srgb8,
    /// Error card fill.
    pub error_container: Srgb8,
    /// Content on the error card.
    pub on_error_container: Srgb8,
}

/// What:
///      `pub struct ThemePalette { dark, light }`: both schemes derived from one root.
/// Why:
///       The window picks between them itself, so a scheme change needs no Rust round trip.
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
///      `struct RoleSpec { lightness, chroma, hue_offset }`: where one role sits relative to the root, measured
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
    lightness: f64,
    chroma: f64,
    hue_offset: f64,
}

/// What:
///      `enum Role { Fixed, Neutral, Accent }`: how one role is obtained.
/// Why:
///       `Fixed` ignores the root (pure black, pure white and the semantic error colors); `Neutral` roles are the
///           grays: they take the root hue at their own tint, never shrunk by a weak root and never grown by a vivid
///           one (D118), and are true grays only when the root is completely neutral; `Accent` roles scale their
///           chroma with the root chroma.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Role = { fixed: Srgb8 } | { neutral: RoleSpec } | { accent: RoleSpec };
/// ```
#[derive(Clone, Copy, Debug)]
enum Role {
    Fixed(Srgb8),
    Neutral(RoleSpec),
    Accent(RoleSpec),
}

/// What:
///      `const fn neutral(...)`, `accent(...)` and `fixed(...)`: short constructors for the role tables.
/// Why:
///       Keeps each table row on one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const neutral = (lightness, chroma, hueOffset) => ({ neutral: { lightness, chroma, hueOffset } });
/// ```
const fn neutral(lightness: f64, chroma: f64, hue_offset: f64) -> Role {
    return Role::Neutral(RoleSpec { lightness, chroma, hue_offset });
}

/// See `neutral`.
const fn accent(lightness: f64, chroma: f64, hue_offset: f64) -> Role {
    return Role::Accent(RoleSpec { lightness, chroma, hue_offset });
}

/// See `neutral`.
const fn fixed(red: u8, green: u8, blue: u8) -> Role {
    return Role::Fixed(Srgb8 { red, green, blue });
}

/// What:
///      `struct SchemeSpec { ... }`: how each table-driven role of one scheme is obtained.
/// Why:
///       The two constants below are the only place lightness targets live.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SchemeSpec = Record<"canvas" | "deck" | /* ... */ "onErrorContainer", Role>;
/// ```
#[derive(Clone, Copy, Debug)]
struct SchemeSpec {
    canvas: Role,
    deck: Role,
    current_row: Role,
    surface_high: Role,
    on_surface: Role,
    on_surface_variant: Role,
    outline: Role,
    outline_variant: Role,
    primary: Role,
    on_primary: Role,
    secondary_container: Role,
    on_secondary_container: Role,
    error: Role,
    error_container: Role,
    on_error_container: Role,
}

// Targets fitted to the Fold captures (the baseline root reproduces them): `ui/theme.slint` before D108 held
// dark primary #BAC5EE, light primary #4E5E8B, secondary container #353B4D and #DDE2F9, and so on. Two light roles
// were then retuned darker in lightness only (B4): primary from 0.4883 to 0.4545 and on-surface-variant from 0.4844
// to 0.4685, the smallest steps that put every drawn state at AA at the baseline (pressed layer on surface-high for
// both, and the rail letter under the hover and pressed layers on the selected fill for primary). The baseline
// therefore needs no contrast adjustment, as `the_baseline_needs_no_contrast_adjustment` checks.
const DARK_SPEC: SchemeSpec = SchemeSpec {
    // D41, D118: the canvas is pure black under every accent (black is not a gray and, at lightness 0, no chroma can
    // show). The deck is a gray: the baseline reproduces the old literal #0A0A0D exactly, and every other root gives
    // it its own hue at the same tint.
    canvas: fixed(0x00, 0x00, 0x00),
    deck: neutral(0.1463, 0.0064, 17.0),
    current_row: neutral(0.1879, 0.0103, 7.9),
    surface_high: neutral(0.2408, 0.0115, 9.4),
    on_surface: neutral(0.9244, 0.0149, 13.4),
    on_surface_variant: neutral(0.8288, 0.0139, 12.9),
    outline: neutral(0.6546, 0.0135, 7.4),
    outline_variant: neutral(0.3957, 0.0153, 7.1),
    primary: accent(0.8286, 0.0589, 4.7),
    on_primary: accent(0.3053, 0.0695, -1.9),
    secondary_container: accent(0.3544, 0.0324, 1.8),
    on_secondary_container: accent(0.9159, 0.0317, 7.4),
    // Semantic constants: Material 3 error tones, matching the captured feedback colors.
    error: fixed(0xFF, 0xB4, 0xAB),
    error_container: fixed(0x8C, 0x1D, 0x18),
    on_error_container: fixed(0xF9, 0xDE, 0xDC),
};

const LIGHT_SPEC: SchemeSpec = SchemeSpec {
    // D34, D45: a flat white canvas with hairline dividers (the outline-variant role).
    canvas: fixed(0xFF, 0xFF, 0xFF),
    deck: neutral(0.9669, 0.0093, 24.2),
    current_row: neutral(0.9669, 0.0093, 24.2),
    surface_high: neutral(0.9308, 0.0094, 24.2),
    on_surface: neutral(0.3188, 0.0162, 7.0),
    on_surface_variant: neutral(0.4685, 0.0193, 9.6),
    outline: neutral(0.5706, 0.0140, 7.3),
    outline_variant: neutral(0.8288, 0.0139, 12.9),
    primary: accent(0.4545, 0.0753, 0.0),
    // Pure white, not tinted: its chroma is 0, so the accent leaves it white.
    on_primary: accent(1.0, 0.0, 0.0),
    secondary_container: accent(0.9159, 0.0317, 7.4),
    on_secondary_container: accent(0.2260, 0.0341, 2.4),
    error: fixed(0xBA, 0x1A, 0x1A),
    error_container: fixed(0xF9, 0xDE, 0xDC),
    on_error_container: fixed(0x41, 0x0E, 0x0B),
};

/// What:
///      `struct RootTerms { hue, scale, completely_neutral, accent_chroma_floor }`: the things the derivation takes
///  from the root.
/// Why:
///       `hue` places every role on the color wheel, `scale` is the root chroma relative to the baseline's, capped by
///           `ROOT_CHROMA_CAP`, and `completely_neutral` (D118) is true only when the root's three 8-bit channels are
///           equal; the root's own lightness is never used, since each role has its own target.
///           `accent_chroma_floor` is the least chroma an accent role may have: `root_terms` leaves it 0 because it
///           depends on the scheme, and `derive_scheme` sets it to the scheme's largest gray tint unless the root is
///           completely neutral.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RootTerms = { hue: number; scale: number; completelyNeutral: boolean; accentChromaFloor: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RootTerms {
    /// Root hue in degrees, 0 up to 360.
    pub hue: f64,
    /// Root chroma over the baseline chroma, capped; exactly 0 for a completely neutral root.
    pub scale: f64,
    /// True only when the root's red, green and blue are equal; then every gray role has chroma 0.
    pub completely_neutral: bool,
    /// Least chroma of any accent role; set per scheme by `derive_scheme`, 0 from `root_terms`.
    pub accent_chroma_floor: f64,
}

/// What:
///      `pub fn root_terms(root: Option<Srgb8>) -> RootTerms`: reads the root; `None` (no system accent) is the
///  baseline. `Option` is Rust's "value or nothing", like `T | undefined`.
/// Why:
///       D118: a root is completely neutral only when its three 8-bit channels are equal, which is exactly "no chroma
///           at all" for a stored color. Equality is the test, not a chroma threshold: the 256 equal-channel roots
///           measure at most 3.7e-8 in OKLCH chroma (float noise) and the least chromatic unequal root, #FEFFFF,
///           measures 1.06e-3, so any threshold between them agrees with equality, and equality needs no tuned
///           constant that Rust and Kotlin could disagree on. A neutral root has no hue (the baseline hue is kept so
///           the value is deterministic) and scale 0; any other root, however weakly chromatic, keeps its own hue.
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
///      `fn resolve_oklch(role: Role, terms: RootTerms) -> Oklch`: the role's target color before gamut mapping.
/// Why:
///       `Neutral` chroma is the role's own tint for every root that is not completely neutral (D118: a weak root does
///           not shrink it and a vivid root does not grow it) and 0 for a completely neutral root; `Accent` chroma
///           scales with the root in both directions but never falls below `accent_chroma_floor`, so for a root with
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
///      `fn resolve_srgb(role: Role, terms: RootTerms) -> Srgb8`: the finished color of a role without any contrast fit.
/// Why:
///       `Fixed` colors are used as written (no OKLCH round trip), the rest go through `oklch_to_srgb8`.
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
///      `fn fit_lightness(name, start, endpoint, passes) -> Oklch`: moves `start` in lightness toward `endpoint`
///  (0 black or 1 white), keeping chroma and hue, until `passes` accepts the 8-bit result; returns the closest
///  accepted lightness to `start`.
/// Why:
///       This is the contrast guarantee: contrast depends on luminance, so lightness is the one coordinate moved,
///           and `passes` always sees the quantized color that reaches the screen. If even the endpoint fails, the
///           endpoint is returned and a warning logged, because no lightness can do better.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (passes(toSrgb8(start))) return start;
/// let [failing, passing] = [start.lightness, endpoint];
/// for (let i = 0; i < 40; i++) { const mid = (failing + passing) / 2; passes(toSrgb8({ ...start, lightness: mid })) ? (passing = mid) : (failing = mid); }
/// return { ...start, lightness: passing };
/// ```
fn fit_lightness(name: &str, start: Oklch, endpoint: f64, passes: &dyn Fn(Srgb8) -> bool) -> Oklch {
    if passes(oklch_to_srgb8(start)) {
        return start;
    }
    let at_endpoint: Oklch = Oklch { lightness: endpoint, ..start };
    if !passes(oklch_to_srgb8(at_endpoint)) {
        tracing::warn!(role = name, endpoint, "no lightness reaches the contrast target, using the endpoint");
        return at_endpoint;
    }
    let mut failing: f64 = start.lightness;
    let mut passing: f64 = endpoint;
    for _step in 0..40 {
        let middle: f64 = (failing + passing) / 2.0;
        if passes(oklch_to_srgb8(Oklch { lightness: middle, ..start })) {
            passing = middle;
        } else {
            failing = middle;
        }
    }
    tracing::debug!(role = name, from = start.lightness, to = passing, "moved lightness to reach the contrast target");
    return Oklch { lightness: passing, ..start };
}

/// What:
///      `fn legible_on(content, backgrounds, layer, layer_alphas, target) -> bool`: true when `content` reaches
///  `target` against every background, and also against that background with `layer` at each of `layer_alphas` over it.
/// Why:
///       Text is drawn on a surface and also under a hover, pressed or focus layer (alpha-only, D108); the caller
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
///      `fn container_layers(container, content) -> (Srgb8, Srgb8)`: the selected fill under the hover and under the
///  pressed state, moved from the container toward its content color in OKLCH. A tuple `(A, B)` is a fixed pair.
/// Why:
///       Replaces the sRGB `mix` the segmented button used; the move is in lightness, chroma and hue.
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
///      `fn gray_tint_ceiling(spec: &SchemeSpec) -> f64`: the largest chroma among the gray (`Neutral`) rows of one
///  scheme's table.
/// Why:
///       D118's reading that an accent is never grayer than the grays needs one number per scheme: the strongest tint
///           any gray of that scheme carries (the dark scheme's is 0.0153, the light scheme's 0.0193).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.max(...Object.values(spec).filter((role) => "neutral" in role).map((role) => role.neutral.chroma))
/// ```
fn gray_tint_ceiling(spec: &SchemeSpec) -> f64 {
    let rows: [Role; 7] = [
        spec.deck,
        spec.current_row,
        spec.surface_high,
        spec.on_surface,
        spec.on_surface_variant,
        spec.outline,
        spec.outline_variant,
    ];
    let mut ceiling: f64 = 0.0;
    for row in rows {
        if let Role::Neutral(row_spec) = row {
            ceiling = ceiling.max(row_spec.chroma);
        }
    }
    return ceiling;
}

/// What:
///      `pub fn derive_scheme(from_root: RootTerms, scheme: Scheme) -> RoleSet`: every role of one scheme.
/// Why:
///       Places each role from its target, then fits lightness so each text pair holds AA: content on its surface
///           and layers, accent text on every surface it is drawn on, outlines against the canvas and deck.
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
    let on_container_fit: Oklch = fit_lightness("on-secondary-container", on_container_start, toward_text, &|candidate| {
        let (hovered, pressed) = container_layers(secondary_container, candidate);
        return contrast_ratio(candidate, secondary_container) >= TEXT_TARGET_RATIO
            && contrast_ratio(candidate, hovered) >= TEXT_TARGET_RATIO
            && contrast_ratio(candidate, pressed) >= TEXT_TARGET_RATIO;
    });
    let on_secondary_container: Srgb8 = oklch_to_srgb8(on_container_fit);
    let (secondary_container_hovered, secondary_container_pressed) =
        container_layers(secondary_container, on_secondary_container);

    // Neutral text on every surface, under the on-surface hover and pressed layers. Secondary text below holds under
    // the same layers, tinted with on-surface.
    let on_surface_fit: Oklch = fit_lightness("on-surface", resolve_oklch(spec.on_surface, terms), toward_text, &|candidate| {
        return legible_on(candidate, &surfaces, candidate, &TEXT_LAYER_ALPHAS, TEXT_TARGET_RATIO);
    });
    let on_surface: Srgb8 = oklch_to_srgb8(on_surface_fit);
    let variant_fit: Oklch = fit_lightness(
        "on-surface-variant",
        resolve_oklch(spec.on_surface_variant, terms),
        toward_text,
        &|candidate| {
            return legible_on(candidate, &surfaces, on_surface, &TEXT_LAYER_ALPHAS, TEXT_TARGET_RATIO);
        },
    );
    let on_surface_variant: Srgb8 = oklch_to_srgb8(variant_fit);

    // Outlines are UI parts: 3:1 against the canvas and the deck they border.
    let outline_fit: Oklch = fit_lightness("outline", resolve_oklch(spec.outline, terms), toward_text, &|candidate| {
        return contrast_ratio(candidate, canvas) >= NON_TEXT_TARGET_RATIO
            && contrast_ratio(candidate, deck) >= NON_TEXT_TARGET_RATIO;
    });
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
    let primary_fit: Oklch = fit_lightness("primary", primary_start, toward_text, &|candidate| {
        if !legible_on(candidate, &surfaces, on_surface, &TEXT_LAYER_ALPHAS, TEXT_TARGET_RATIO) {
            return false;
        }
        for fill in selected_fill_states {
            if contrast_ratio(candidate, fill) < TEXT_TARGET_RATIO {
                return false;
            }
        }
        if let Some(content) = light_on_primary {
            return contrast_ratio(candidate, content) >= TEXT_TARGET_RATIO;
        }
        return true;
    });
    let primary: Srgb8 = oklch_to_srgb8(primary_fit);

    // On-primary is fitted against the primary fill and its own pressed layer (the first-run button).
    let on_primary: Srgb8 = match light_on_primary {
        Some(content) => content,
        None => oklch_to_srgb8(fit_lightness("on-primary", resolve_oklch(spec.on_primary, terms), away_from_text, &|candidate| {
            return legible_on(candidate, &[primary], candidate, &TEXT_LAYER_ALPHAS, TEXT_TARGET_RATIO);
        })),
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
    let match_fit: Oklch = fit_lightness("match-fill", match_start, neutral_end, &|candidate| {
        return contrast_ratio(on_surface, candidate) >= TEXT_TARGET_RATIO
            && contrast_ratio(on_surface_variant, candidate) >= TEXT_TARGET_RATIO;
    });
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
///      `pub fn derive_roles(root: Option<Srgb8>) -> ThemePalette`: both schemes from one root; `None` is the baseline.
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

#[cfg(test)]
#[path = "theme_roles_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "theme_roles_fixture_tests.rs"]
mod fixture_tests;
