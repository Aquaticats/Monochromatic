//! Deriving every role of a scheme,
//! and both schemes from one root.
//!
//! What:
//! `derive_scheme` places each role from its target,
//! and then fits its lightness so every text pair holds AA,
//! and `derive_roles` runs it for the dark and the light scheme.
//! Why:
//! the placement order is what makes the contrast guarantee hold,
//! so it keeps one file,
//! split from `src/theme_roles.rs` to stay within the line budget.

/// What:
///      `use super::{...}` brings the root terms,
///  the scheme,
///  the palette,
///  the role resolution helpers and the search-match shares from the parent module.
/// Why:
///      The derivation reads the root once,
///  and then resolves each role through the parent's helpers,
///  so it does not restate them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RootTerms, Scheme, ThemePalette, resolveOklch, resolveSrgb, MATCH_DARK_LIGHTNESS_SHARE, MATCH_DARK_CHROMA_SHARE, MATCH_LIGHT_LIGHTNESS_SHARE, MATCH_LIGHT_CHROMA_SHARE, HOVER_ALPHA, PRESSED_ALPHA } from "./theme_roles";
/// ```
use super::{
    HOVER_ALPHA,
    MATCH_DARK_CHROMA_SHARE,
    MATCH_DARK_LIGHTNESS_SHARE,
    MATCH_LIGHT_CHROMA_SHARE,
    MATCH_LIGHT_LIGHTNESS_SHARE,
    PRESSED_ALPHA,
    RootTerms,
    Scheme,
    ThemePalette,
    resolve_oklch,
    resolve_srgb,
    root_terms,
};

/// What:
///      `use super::theme_roles_contrast_fit::{...}` brings the lightness search,
///  its per-role rules and the layer builder.
/// Why:
///      Every role of a scheme is placed and then fitted by those helpers,
///  which live in their own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FitRule, container_layers, fit_lightness } from "./theme_roles_contrast_fit";
/// ```
use super::theme_roles_contrast_fit::{
    FitRule,
    container_layers,
    fit_lightness,
};

/// What:
///      `use super::theme_roles_declaration::{...}` brings the dark and light role
///           tables,
///  the finished role set,
///  the table type and the gray tint ceiling.
/// Why:
///      The derivation walks those tables,
///  so their names stay next to the data instead of here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DARK_SPEC, LIGHT_SPEC, RoleSet, SchemeSpec, gray_tint_ceiling } from "./theme_roles_declaration";
/// ```
use super::theme_roles_declaration::{
    DARK_SPEC,
    LIGHT_SPEC,
    RoleSet,
    SchemeSpec,
    gray_tint_ceiling,
};

/// What:
///      `use crate::oklch::{...}` brings the two color types,
///  and the conversions and composites a placed role goes through.
/// Why:
///      A role is resolved in OKLCH,
///  fitted in OKLCH and written out as 8-bit sRGB,
///  so the color module owns every step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Oklch, Srgb8, compositeOver, oklchToSrgb8, srgbToOklch } from "./oklch";
/// ```
use crate::oklch::{
    Oklch,
    Srgb8,
    composite_over,
    oklch_to_srgb8,
    srgb_to_oklch,
};

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
pub fn derive_scheme(
    from_root: RootTerms,
    scheme: Scheme,
) -> RoleSet {
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
    let accent_chroma_floor: f64 = if from_root.completely_neutral {
        0.0
    } else {
        gray_tint_ceiling(&spec)
    };
    let terms: RootTerms = RootTerms {
        accent_chroma_floor,
        ..from_root
    };
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
    let on_container_fit: Oklch = fit_lightness(
        "on-secondary-container",
        on_container_start,
        toward_text,
        &FitRule::Container {
            container: secondary_container,
        },
    );
    let on_secondary_container: Srgb8 = oklch_to_srgb8(on_container_fit);
    let (secondary_container_hovered, secondary_container_pressed) =
        container_layers(secondary_container, on_secondary_container);

    // Neutral text on every surface, under the on-surface hover and pressed layers. Secondary text below holds under
    // the same layers, tinted with on-surface.
    let on_surface_fit: Oklch = fit_lightness(
        "on-surface",
        resolve_oklch(spec.on_surface, terms),
        toward_text,
        &FitRule::Legible {
            surfaces: &surfaces,
            layer: None,
        },
    );
    let on_surface: Srgb8 = oklch_to_srgb8(on_surface_fit);
    let variant_fit: Oklch = fit_lightness(
        "on-surface-variant",
        resolve_oklch(spec.on_surface_variant, terms),
        toward_text,
        &FitRule::Legible {
            surfaces: &surfaces,
            layer: Some(on_surface),
        },
    );
    let on_surface_variant: Srgb8 = oklch_to_srgb8(variant_fit);

    // Outlines are UI parts: 3:1 against the canvas and the deck they border.
    let outline_fit: Oklch = fit_lightness(
        "outline",
        resolve_oklch(spec.outline, terms),
        toward_text,
        &FitRule::Outline {
            canvas,
            deck,
        },
    );
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
    let primary_fit: Oklch = fit_lightness(
        "primary",
        primary_start,
        toward_text,
        &FitRule::Primary {
            surfaces: &surfaces,
            on_surface,
            fills: selected_fill_states,
            light_on_primary,
        },
    );
    let primary: Srgb8 = oklch_to_srgb8(primary_fit);

    // On-primary is fitted against the primary fill and its own pressed layer (the first-run button).
    let on_primary: Srgb8 = match light_on_primary {
        Some(content) => content,
        None => oklch_to_srgb8(fit_lightness(
            "on-primary",
            resolve_oklch(spec.on_primary, terms),
            away_from_text,
            &FitRule::Legible {
                surfaces: &[primary],
                layer: None,
            },
        )),
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
    let match_fit: Oklch = fit_lightness(
        "match-fill",
        match_start,
        neutral_end,
        &FitRule::MatchFill {
            on_surface,
            on_surface_variant,
        },
    );
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
