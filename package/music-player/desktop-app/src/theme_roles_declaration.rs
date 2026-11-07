//! The role declaration:
//! the `RoleSet` fields and the role target tables of both schemes.
//!
//! What:
//!  `define_role_set!` declares every role once,
//!  as the field name plus the name the shared Android fixture spells it with,
//!  and expands both the `RoleSet` struct and the test-only field walk from that one list;
//!  the `SchemeSpec` tables below then give each role its baseline targets.
//! Why:
//!  Rust cannot enumerate a struct's fields,
//!  so a second hand-written list would silently skip a role added later;
//!  one invocation keeps the declaration and the fixture guard in step.
//!  The tables live beside the roles they describe,
//!  apart from the derivation that consumes them,
//!  so neither file carries the whole role declaration and its math.

/// Import the finished-color type that every `RoleSet` field holds.
use crate::oklch::Srgb8;

/// Import the per-role baseline targets that the table constructors build.
use super::RoleSpec;

/// What:
///      `define_role_set!` declares `RoleSet` once:
///      the struct's fields and the fixture guard's role list both expand from its single token list.
/// Why:
///       Rust cannot enumerate a struct's fields,
///       so a second,
///       hand-written list of them silently skips any role added later;
///       generating both from one invocation makes the Android theme-vector test compare every role the struct has,
///       and fail until the shared fixture carries it.
///       Each entry's right-hand side is the role's Kotlin `ThemeRoles` field name,
///       which is how the shared fixture spells it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ROLE_FIELDS = [['canvas', 'canvas'], /* ... */] as const;
/// type RoleSet = Record<(typeof ROLE_FIELDS)[number][0], Srgb8>;
/// ```
macro_rules! define_role_set {
    (
        $(
            $(#[$field_doc:meta])*
            $field:ident => $fixture_name:literal,
        )*
    ) => {
        /// What:
        ///      `pub struct RoleSet { ... }`:
        /// every color role of one scheme as finished sRGB.
        /// Why:
        ///       The window receives one of these per scheme;
        ///       names mirror the Slint `RoleSet` struct field for field.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// type RoleSet = { canvas: Srgb8; deck: Srgb8; /* ... one field per role ... */ };
        /// ```
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct RoleSet {
            $(
                $(#[$field_doc])*
                pub $field: Srgb8,
            )*
        }

        /// What:
        ///      `role_fields(set)` lists every role of one scheme with the name the shared fixture gives it.
        /// Why:
        ///       the fixture test compares one token per role;
        ///       expanding this list from the same declaration that builds the struct
        ///       means a new role cannot go uncompared.
        #[cfg(test)]
        pub(crate) fn role_fields(set: &RoleSet) -> Vec<(&'static str, Srgb8)> {
            return vec![
                $(
                    ($fixture_name, set.$field),
                )*
            ];
        }
    };
}

define_role_set! {
    /// Page background.
    canvas => "canvas",
    /// Playback deck background.
    deck => "deck",
    /// Background of the playing row.
    current_row => "currentRow",
    /// Menus,
    /// search header and snackbar surface.
    surface_high => "surfaceHigh",
    /// Primary text and icons.
    on_surface => "onSurface",
    /// Secondary text and icons.
    on_surface_variant => "onSurfaceVariant",
    /// Borders and strong dividers.
    outline => "outline",
    /// Hairline dividers.
    outline_variant => "outlineVariant",
    /// Accent fill and accent text.
    primary => "primary",
    /// Content on the accent fill.
    on_primary => "onPrimary",
    /// Selected segment and rail cell fill.
    secondary_container => "secondaryContainer",
    /// Selected fill under the hover layer.
    secondary_container_hovered => "secondaryContainerHovered",
    /// Selected fill under the pressed layer.
    secondary_container_pressed => "secondaryContainerPressed",
    /// Content on the selected fill.
    on_secondary_container => "onSecondaryContainer",
    /// Search match highlight (D59),
    /// opaque.
    match_fill => "matchFill",
    /// Error text and icons.
    error => "error",
    /// Error card fill.
    error_container => "errorContainer",
    /// Content on the error card.
    on_error_container => "onErrorContainer",
}

/// What:
///      `enum Role { Fixed, Neutral, Accent }`:
/// how one role is obtained.
/// Why:
///       `Fixed` ignores the root (pure black,
/// pure white and the semantic error colors);
///       `Neutral` roles are the grays:
///       they take the root hue at their own tint,
///       never shrunk by a weak root and never grown by a vivid one (D118),
///       and are true grays only when the root is completely neutral;
///       `Accent` roles scale their chroma with the root chroma.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Role = { fixed: Srgb8 } | { neutral: RoleSpec } | { accent: RoleSpec };
/// ```
#[derive(Clone, Copy, Debug)]
pub(super) enum Role {
    /// A color used as written,
    /// with no OKLCH round trip.
    Fixed(
        /// The finished color.
        Srgb8,
    ),
    /// A gray,
    /// carrying the root's hue at its own tint (D118).
    Neutral(
        /// The role's baseline targets.
        RoleSpec,
    ),
    /// An accent,
    /// scaling its chroma with the root's.
    Accent(
        /// The role's baseline targets.
        RoleSpec,
    ),
}

/// What:
///      `const fn neutral(...)`,
/// `accent(...)` and `fixed(...)`:
/// short constructors for the role tables.
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
///      `struct SchemeSpec { ... }`:
/// how each table-driven role of one scheme is obtained.
/// Why:
///       The two constants below are the only place lightness targets live.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SchemeSpec = Record<"canvas" | "deck" | /* ... */ "onErrorContainer", Role>;
/// ```
#[derive(Clone, Copy, Debug)]
pub(super) struct SchemeSpec {
    /// The page background.
    pub(super) canvas: Role,
    /// The playback deck background.
    pub(super) deck: Role,
    /// The background of the playing row.
    pub(super) current_row: Role,
    /// Menus,
    /// search header and snackbar surface.
    pub(super) surface_high: Role,
    /// Primary text and icons.
    pub(super) on_surface: Role,
    /// Secondary text and icons.
    pub(super) on_surface_variant: Role,
    /// Borders and strong dividers.
    pub(super) outline: Role,
    /// Hairline dividers.
    pub(super) outline_variant: Role,
    /// Accent fill and accent text.
    pub(super) primary: Role,
    /// Content on the accent fill.
    pub(super) on_primary: Role,
    /// Selected segment and rail cell fill.
    pub(super) secondary_container: Role,
    /// Content on the selected fill.
    pub(super) on_secondary_container: Role,
    /// Error text and icons.
    pub(super) error: Role,
    /// Error card fill.
    pub(super) error_container: Role,
    /// Content on the error card.
    pub(super) on_error_container: Role,
}

/// What:
///      `DARK_SPEC`:
/// the baseline targets of the dark scheme.
/// Why:
///       Targets fitted to the Fold captures (the baseline root reproduces them);
///       `ui/theme.slint` before D108 held dark primary #BAC5EE,
/// light primary #4E5E8B,
///       secondary container #353B4D and #DDE2F9,
/// and so on.
pub(super) const DARK_SPEC: SchemeSpec = SchemeSpec {
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

/// What:
///      `LIGHT_SPEC`:
/// the baseline targets of the light scheme.
/// Why:
///       Two light roles were retuned darker in lightness only (B4):
///       primary from 0.4883 to 0.4545 and on-surface-variant from 0.4844 to 0.4685,
///       the smallest steps that put every drawn state at AA at the baseline
///       (pressed layer on surface-high for both,
///       and the rail letter under the hover and pressed layers on the selected fill for primary).
///       The baseline therefore needs no contrast adjustment,
///       as `the_baseline_needs_no_contrast_adjustment` checks.
pub(super) const LIGHT_SPEC: SchemeSpec = SchemeSpec {
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
///      `fn gray_tint_ceiling(spec: &SchemeSpec) -> f64`:
/// the largest chroma among the gray (`Neutral`) rows of one
///  scheme's table.
/// Why:
///       D118's reading that an accent is never grayer than the grays needs one number per scheme:
///       the strongest tint any gray of that scheme carries
///       (the dark scheme's is 0.0153,
/// the light scheme's 0.0193).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.max(...Object.values(spec).filter((role) => "neutral" in role).map((role) => role.neutral.chroma))
/// ```
pub(super) fn gray_tint_ceiling(spec: &SchemeSpec) -> f64 {
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
        let Role::Neutral(neutral) = row else {
            continue;
        };
        if neutral.chroma > ceiling {
            ceiling = neutral.chroma;
        }
    }
    return ceiling;
}
