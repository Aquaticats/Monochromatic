//! Apply the runtime `accent-color` control verb to the private appearance portal only.

/// Imports the items this module uses from `crate`.
use crate::{
    accent_color::AccentColor,
    appearance_portal::{AppearancePortal, SwitchOutcome},
    protocol::Response,
};

/// Switch the private portal's accent color,
///  or explain why no private accent color exists.
///
/// What:
///      `pub fn switch(portal: Option<&AppearancePortal>, color: AccentColor) -> Response`.
///  `ok changed` after notifying subscribed clients,
///  `ok unchanged` for the color already served.
/// Why:
///       A session started without `--accent-color` serves no accent color.
///  The only other bus this process could reach is the host session bus,
///  so the request is refused rather than redirected there.
pub fn switch(portal: Option<&AppearancePortal>, color: AccentColor) -> Response {
    let active = match portal {
        Some(candidate) if candidate.serves_accent_color() => candidate,
        _ => {
            return Response::Err(
                "accent-color needs the private accent color, which this session did not start; \
                 restart it with --accent-color RRGGBB"
                    .to_string(),
            );
        }
    };
    match active.set_accent_color(color) {
        Ok(SwitchOutcome::Changed) => return Response::OkWith("changed".to_string()),
        Ok(SwitchOutcome::Unchanged) => return Response::OkWith("unchanged".to_string()),
        Err(error) => return Response::Err(format!("{error:#}")),
    }
}
