//! The `color-scheme` control verb's responses,
//!  exercised without a display or the host bus.

/// The same function the compositor's control dispatcher calls for this verb.
use nested_wayland_session::{
    appearance_portal::AppearancePortal, control_color_scheme::switch, protocol::Response,
    ColorSchemePreference,
};

/// Without `--color-scheme` there is no private bus,
///  and the verb must not find another one.
#[test]
fn switch_without_private_portal_is_refused_with_its_remedy() {
    let response = switch(None, ColorSchemePreference::Light);
    // let-else binds the refusal message or stops the test with the unexpected response.
    let Response::Err(message) = response else {
        panic!("expected a refusal, got {response:?}");
    };
    assert!(message.contains("--color-scheme"), "{message}");
    assert!(message.contains("private"), "{message}");
}

/// A real private portal reports whether hosted clients were notified.
#[test]
fn switch_reports_changed_only_when_the_served_value_changes() -> anyhow::Result<()> {
    let bus = std::sync::Arc::new(nested_wayland_session::private_bus::PrivateBus::start()?);
    let portal = AppearancePortal::start(ColorSchemePreference::Dark, bus)?;
    let changed = Response::OkWith("changed".to_string());
    let unchanged = Response::OkWith("unchanged".to_string());
    assert_eq!(switch(Some(&portal), ColorSchemePreference::Dark), unchanged);
    assert_eq!(switch(Some(&portal), ColorSchemePreference::Light), changed);
    assert_eq!(switch(Some(&portal), ColorSchemePreference::Light), unchanged);
    assert_eq!(switch(Some(&portal), ColorSchemePreference::Dark), changed);
    return Ok(());
}
