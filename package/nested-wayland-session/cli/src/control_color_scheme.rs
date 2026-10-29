//! Apply the runtime `color-scheme` control verb to the private appearance portal only.

/// What:
///      A grouped `use` of the private portal handle,
///  its value and outcome types,
///  and
///           the wire response.
///  The braces avoid repeating the common `crate::` prefix
///           (`crate` is this package's own root).
/// Why:
///       This function maps one typed control request onto one control-socket response.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type AppearancePortal, type ColorSchemePreference, type SwitchOutcome } from "./appearance_portal";
/// import { type Response } from "./protocol";
/// ```
use crate::{
    appearance_portal::{AppearancePortal, ColorSchemePreference, SwitchOutcome},
    protocol::Response,
};

/// Switch the private portal's color scheme,
///  or explain why no private portal exists.
///
/// What:
///      `pub fn switch(portal: Option<&AppearancePortal>, preference:
///           ColorSchemePreference) -> Response`.
///  `Option<&AppearancePortal>` is either a
///           borrowed portal handle or nothing;
///  Rust has no `null`.
///  Taking the handle as a
///           parameter,
///  instead of the whole compositor,
///  lets tests call this without a display.
/// Why:
///       A session started without `--color-scheme` has no private bus.
///  The only other
///           bus this process could reach is the host session bus,
///  so the request is refused
///           rather than redirected there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function switchColorScheme(portal: AppearancePortal | undefined, preference): Response { ... }
/// ```
///
/// @example
/// ```ts
/// switchColorScheme(portal, "light"); // => { ok: true, data: "changed" }
/// switchColorScheme(portal, "light"); // => { ok: true, data: "unchanged" }
/// switchColorScheme(undefined, "light"); // => { ok: false, error: "color-scheme needs ..." }
/// ```
pub fn switch(portal: Option<&AppearancePortal>, preference: ColorSchemePreference) -> Response {
    // What:     `let Some(active) = portal else { ... }` binds the present handle, or runs the
    //           `else` block, which must leave the function. `Response::Err(...)` builds the
    //           failure response; `.to_string()` copies the literal into an owned `String`.
    // Why:      Name the affected command, the reason, and the one way to make it work.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (portal === undefined) return { ok: false, error: "color-scheme needs ..." };
    // ```
    let active = match portal {
        Some(candidate) if candidate.serves_color_scheme() => candidate,
        _ => {
            return Response::Err(
                "color-scheme needs the private appearance portal, which this session did not start; \
                 restart it with --color-scheme dark or --color-scheme light"
                    .to_string(),
            );
        }
    };

    // What:     `match` covers the three results of the switch: two success outcomes and one
    //           failure. `Response::OkWith(...)` carries the outcome word as response data.
    //           `{error:#}` formats the error with its whole chain of causes.
    // Why:      `unchanged` tells a test that no hosted client was notified, so it must not
    //           wait for a repaint.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return { ok: true, data: portal.setColorScheme(preference) }; }
    // catch (error) { return { ok: false, error: String(error) }; }
    // ```
    match active.set_color_scheme(preference) {
        Ok(SwitchOutcome::Changed) => return Response::OkWith("changed".to_string()),
        Ok(SwitchOutcome::Unchanged) => return Response::OkWith("unchanged".to_string()),
        Err(error) => return Response::Err(format!("{error:#}")),
    }
}
