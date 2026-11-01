//! The `accent-color` control verb against a real portal and a client that reads it like Slint.

use std::sync::Arc;

use nested_wayland_session::{
    accent_color::AccentColor,
    appearance_portal::{AppearancePortal, PortalStart},
    control_accent::switch,
    private_bus::PrivateBus,
    protocol::Response,
    ColorSchemePreference,
};
use zbus::{blocking::Proxy, zvariant::OwnedValue};

/// Parses a color the way the verb does.
fn color(text: &str) -> AccentColor {
    return AccentColor::parse(text).expect("a valid test color");
}

/// Without `--accent-color` the verb is refused and names the option,
///  even when a scheme portal runs.
#[test]
fn switch_without_an_accent_is_refused_with_its_remedy() -> anyhow::Result<()> {
    let Response::Err(none) = switch(None, color("112233")) else {
        panic!("expected a refusal without a portal");
    };
    assert!(none.contains("--accent-color RRGGBB"), "{none}");
    let bus = Arc::new(PrivateBus::start()?);
    let portal = AppearancePortal::start(ColorSchemePreference::Dark, bus)?;
    let Response::Err(scheme_only) = switch(Some(&portal), color("112233")) else {
        panic!("expected a refusal from a scheme-only portal");
    };
    assert!(scheme_only.contains("--accent-color RRGGBB"), "{scheme_only}");
    return Ok(());
}

/// A change reaches a client that read the setting first and then follows `SettingChanged`.
#[test]
fn accent_change_reaches_a_slint_style_client() -> anyhow::Result<()> {
    let bus = Arc::new(PrivateBus::start()?);
    let portal = AppearancePortal::serve(PortalStart {
        color_scheme: None,
        accent_color: Some(color("3584E4")),
        bus,
    })?;
    let connection = zbus::blocking::connection::Builder::address(portal.bus_address())?.build()?;
    let settings = Proxy::new(
        &connection,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.Settings",
    )?;
    // Slint subscribes before its initial read.
    let mut changes = settings.receive_signal("SettingChanged")?;
    let initial: OwnedValue = settings.call("ReadOne", &("org.freedesktop.appearance", "accent-color"))?;
    assert_eq!(initial.downcast_ref::<(f64, f64, f64)>()?, color("3584E4").portal_value());

    assert_eq!(switch(Some(&portal), color("3584E4")), Response::OkWith("unchanged".to_string()));
    assert_eq!(switch(Some(&portal), color("FF7800")), Response::OkWith("changed".to_string()));
    let message = changes.next().expect("the signal stream stays open");
    let (namespace, key, value): (String, String, OwnedValue) = message.body().deserialize()?;
    assert_eq!((namespace.as_str(), key.as_str()), ("org.freedesktop.appearance", "accent-color"));
    assert_eq!(value.downcast_ref::<(f64, f64, f64)>()?, (1.0, 120.0 / 255.0, 0.0));
    let reread: OwnedValue = settings.call("ReadOne", &("org.freedesktop.appearance", "accent-color"))?;
    assert_eq!(reread.downcast_ref::<(f64, f64, f64)>()?, (1.0, 120.0 / 255.0, 0.0));
    return Ok(());
}
