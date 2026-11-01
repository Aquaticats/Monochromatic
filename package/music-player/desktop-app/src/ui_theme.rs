//! Pushes the derived theme roles into the window and keeps them current when the accent changes.
//!
//! What: `install_accent_tracking` derives both schemes from the theme root (`theme_roles.rs`) and writes them into
//! the `Theme` global's `dark-roles` and `light-roles`; the window's `accent-changed` callback repeats that whenever
//! the root changes (the OS accent reaches Slint through the desktop portal, DWM or `NSColor`, and the portal's
//! `SettingChanged` signal updates it live).
//! Why: the root is the only thing taken from the platform (D108); every other color is derived in OKLCH in Rust,
//! so the palette can be unit-tested without a window. The light or dark choice stays inside `theme.slint`, so a
//! scheme change needs no Rust round trip.

use music_player::accent_override::{ACCENT_ENV, accent_override};
use music_player::oklch::Srgb8;
use music_player::theme_roles::{self, ThemePalette};
use slint::{Color, ComponentHandle};

use crate::{AppWindow, RoleSet, Theme};

/// What:
///      `fn slint_color(color: Srgb8) -> Color`: the window's color type from a derived 8-bit color.
/// Why:
///       The derived colors are opaque, so alpha is 255.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return { red, green, blue, alpha: 255 };
/// ```
fn slint_color(color: Srgb8) -> Color {
    return Color::from_rgb_u8(color.red, color.green, color.blue);
}

/// What:
///      `pub(crate) fn slint_roles(set: &theme_roles::RoleSet) -> RoleSet`: copies one derived scheme into the
///  generated Slint struct, field for field. `&` lends the set read-only.
/// Why:
///       The Slint struct is the window's type; the Rust struct is the testable one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return { canvas: slintColor(set.canvas), /* ... one line per role ... */ };
/// ```
pub(crate) fn slint_roles(set: &theme_roles::RoleSet) -> RoleSet {
    return RoleSet {
        canvas: slint_color(set.canvas),
        deck: slint_color(set.deck),
        current_row: slint_color(set.current_row),
        surface_high: slint_color(set.surface_high),
        on_surface: slint_color(set.on_surface),
        on_surface_variant: slint_color(set.on_surface_variant),
        outline: slint_color(set.outline),
        outline_variant: slint_color(set.outline_variant),
        primary: slint_color(set.primary),
        on_primary: slint_color(set.on_primary),
        secondary_container: slint_color(set.secondary_container),
        secondary_container_hovered: slint_color(set.secondary_container_hovered),
        secondary_container_pressed: slint_color(set.secondary_container_pressed),
        on_secondary_container: slint_color(set.on_secondary_container),
        match_fill: slint_color(set.match_fill),
        error: slint_color(set.error),
        error_container: slint_color(set.error_container),
        on_error_container: slint_color(set.on_error_container),
    };
}

/// What:
///      `pub(crate) fn root_of(accent: Color) -> Option<Srgb8>`: the theme root a reported accent gives; a
///  transparent color means "no accent available" and gives `None` (the baseline).
/// Why:
///       Slint reports the accent as transparent until a platform value exists (alpha 0 means unavailable).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return accent.alpha > 0 ? { red: accent.red, green: accent.green, blue: accent.blue } : undefined;
/// ```
pub(crate) fn root_of(accent: Color) -> Option<Srgb8> {
    if accent.alpha() == 0 {
        return None;
    }
    return Some(Srgb8 { red: accent.red(), green: accent.green(), blue: accent.blue() });
}

/// What:
///      `pub(crate) fn apply_palette(app: &AppWindow, accent: Color)`: derives both schemes from the accent and
///  writes them into the `Theme` global.
/// Why:
///       The one place the window's palette changes, at startup and on every accent change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const palette = deriveRoles(rootOf(accent));
/// app.theme.darkRoles = slintRoles(palette.dark);
/// app.theme.lightRoles = slintRoles(palette.light);
/// ```
pub(crate) fn apply_palette(app: &AppWindow, accent: Color) {
    tracing::info!(
        red = accent.red(),
        green = accent.green(),
        blue = accent.blue(),
        alpha = accent.alpha(),
        "applying the theme roles for the accent"
    );
    let palette: ThemePalette = theme_roles::derive_roles(root_of(accent));
    let theme = app.global::<Theme>();
    theme.set_dark_roles(slint_roles(&palette.dark));
    theme.set_light_roles(slint_roles(&palette.light));
}

/// What:
///      `pub(crate) fn apply_accent_override_from_env(app: &AppWindow)`: sets `Theme.accent-override` from
///  `MUSIC_PLAYER_ACCENT`, or leaves it transparent (follow the system) when the variable is unset or malformed.
/// Why:
///       Renders and verification runs need a deterministic accent; a malformed value is logged and ignored, the
///           same way a malformed color scheme override is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { const accent = accentOverride(process.env.MUSIC_PLAYER_ACCENT); if (accent) app.theme.accentOverride = accent; }
/// catch (error) { warn(error); }
/// ```
pub(crate) fn apply_accent_override_from_env(app: &AppWindow) {
    let value: Option<String> = std::env::var(ACCENT_ENV).ok();
    match accent_override(value.as_deref()) {
        Ok(Some(accent)) => {
            tracing::info!(red = accent.red, green = accent.green, blue = accent.blue, "using the accent override");
            app.global::<Theme>().set_accent_override(slint_color(accent));
        }
        Ok(None) => {
            tracing::debug!("no accent override, following the system accent");
        }
        Err(error) => {
            tracing::warn!(error = %error, "ignoring the accent override");
        }
    }
}

/// What:
///      `pub(crate) fn install_accent_tracking(app: &AppWindow)`: applies the palette for the current accent now
///  and registers the callback that repeats it when the accent changes.
/// Why:
///       Slint's `changed` handler fires on change, not on the first value, so the first derivation is explicit.
///           The closure holds a weak handle so it does not keep the window alive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// applyPalette(app, app.theme.accentSource);
/// app.onAccentChanged((accent) => applyPalette(app, accent));
/// ```
pub(crate) fn install_accent_tracking(app: &AppWindow) {
    apply_palette(app, app.global::<Theme>().get_accent_source());
    let weak = app.as_weak();
    app.on_accent_changed(move |accent| {
        tracing::info!("the theme accent changed");
        if let Some(window) = weak.upgrade() {
            apply_palette(&window, accent);
        } else {
            tracing::warn!("the window is gone, the accent change is dropped");
        }
    });
}

#[cfg(test)]
#[path = "ui_theme_tests.rs"]
mod tests;
