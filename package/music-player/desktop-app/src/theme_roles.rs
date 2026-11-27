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
//! The lightness search lives in `src/theme_roles_contrast_fit.rs`,
//! the two derivation entry points in `src/theme_roles_derivation.rs`,
//! and the role tables themselves in `src/theme_roles_declaration.rs`;
//! this file holds the role types and the resolution of one role.

/// Imports the OKLCH types and the conversions the resolution of one role moves colors through.
use crate::oklch::{
    Oklch,
    Srgb8,
    oklch_to_srgb8,
    srgb_to_oklch,
};

/// What:
///      `use crate::oklch::{...}` brings the contrast ratio and the layer composite
///           that only the unit tests name.
/// Why:
///       `theme_roles_tests.rs` reaches them through its `use super::*`,
///  and production code here does not,
///  so the import exists in the test build alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { compositeOver, contrastRatio } from "./oklch";
/// ```
#[cfg(test)]
use crate::oklch::{
    composite_over,
    contrast_ratio,
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
pub const BASELINE_ROOT: Srgb8 = Srgb8 {
    red: 0x4E,
    green: 0x5E,
    blue: 0x8B,
};

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
///      `use theme_roles_declaration::Role;` brings the role value this module
///           resolves into a color.
/// Why:
///       `resolve_oklch` and `resolve_srgb` take one role,
///  and the tables the derivation walks are read there.
use theme_roles_declaration::Role;

/// What:
///      `use theme_roles_declaration::{...}` brings both role tables,
///           their type and the gray tint ceiling.
/// Why:
///       Only `theme_roles_tests.rs` names them,
///  through its `use super::*`,
///  so the import exists in the test build alone,
///  and the derivation names the same items by its own path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DARK_SPEC, LIGHT_SPEC, SchemeSpec, gray_tint_ceiling } from "./theme_roles_declaration";
/// ```
#[cfg(test)]
use theme_roles_declaration::{
    DARK_SPEC,
    LIGHT_SPEC,
    SchemeSpec,
    gray_tint_ceiling,
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
        tracing::debug!(
            channel = used.red,
            "the root is completely neutral, every gray role is a true gray"
        );
        return RootTerms {
            hue: baseline.hue,
            scale: 0.0,
            completely_neutral: true,
            accent_chroma_floor: 0.0,
        };
    }
    let lch: Oklch = srgb_to_oklch(used);
    let scale: f64 = lch.chroma.min(ROOT_CHROMA_CAP) / baseline.chroma;
    return RootTerms {
        hue: lch.hue,
        scale,
        completely_neutral: false,
        accent_chroma_floor: 0.0,
    };
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
fn resolve_oklch(
    role: Role,
    terms: RootTerms,
) -> Oklch {
    return match role {
        Role::Fixed(color) => srgb_to_oklch(color),
        Role::Neutral(spec) => Oklch {
            lightness: spec.lightness,
            chroma: if terms.completely_neutral {
                0.0
            } else {
                spec.chroma
            },
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
fn resolve_srgb(
    role: Role,
    terms: RootTerms,
) -> Srgb8 {
    if let Role::Fixed(color) = role {
        return color;
    }
    return oklch_to_srgb8(resolve_oklch(role, terms));
}

/// What:
///      `mod theme_roles_contrast_fit;` loads the sibling
///           `src/theme_roles_contrast_fit.rs`,
///  the lightness search that holds a role's contrast target.
/// Why:
///       The fit is the whole contrast guarantee of the derivation,
///           so it keeps a file of its own while this module holds the role types
///           and their resolution.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as themeRolesContrastFit from "./theme_roles_contrast_fit";
/// ```
#[path = "theme_roles_contrast_fit.rs"]
mod theme_roles_contrast_fit;

/// What:
///      `use theme_roles_contrast_fit::fit_lightness;` brings the lightness search
///           into the unit tests' scope.
/// Why:
///       Only `theme_roles_tests.rs` names it,
///  through its `use super::*`,
///  so the import exists in the test build alone,
///  and the derivation names the same function by its own path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fit_lightness } from "./theme_roles_contrast_fit";
/// ```
#[cfg(test)]
use theme_roles_contrast_fit::fit_lightness;

/// What:
///      `mod theme_roles_derivation;` loads the sibling
///           `src/theme_roles_derivation.rs`,
///  which derives both schemes from one root.
/// Why:
///       The placement order of the roles is what makes the contrast guarantee
///           hold,
///  so it is read on its own and this module stays within the line budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as themeRolesDerivation from "./theme_roles_derivation";
/// ```
#[path = "theme_roles_derivation.rs"]
mod theme_roles_derivation;

/// What:
///      `pub use theme_roles_derivation::{...}` re-exports the two derivation entry
///           points at this module's path.
/// Why:
///       Callers name `theme_roles::derive_roles` and `theme_roles::derive_scheme`,
///  and only where they are declared moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { derive_scheme, derive_roles } from "./theme_roles_derivation";
/// ```
pub use theme_roles_derivation::{
    derive_roles,
    derive_scheme,
};

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
