//! The accent color on the real private bus,
//!  read and followed the way Slint 1.18.1 does.

use std::{collections::HashMap, sync::mpsc, time::Duration};

use anyhow::Context;
use zbus::zvariant::OwnedValue;

use super::*;

/// Bounds how long a test waits for a signal that must already be on the private bus.
const SIGNAL_TIMEOUT: Duration = Duration::from_secs(2);

/// Starts a portal serving the given settings on a fresh private bus.
fn portal_with(scheme: Option<ColorSchemePreference>, accent: Option<&str>) -> anyhow::Result<AppearancePortal> {
    let accent_color = match accent {
        Some(text) => Some(AccentColor::parse(text).map_err(anyhow::Error::msg)?),
        None => None,
    };
    return AppearancePortal::serve(PortalStart {
        color_scheme: scheme,
        accent_color,
        bus: Arc::new(PrivateBus::start()?),
    });
}

/// Opens a client Settings proxy against the portal's well-known name.
fn client(portal: &AppearancePortal) -> anyhow::Result<zbus::blocking::Proxy<'static>> {
    let connection = zbus::blocking::connection::Builder::address(portal.bus_address())?.build()?;
    return Ok(zbus::blocking::Proxy::new(
        &connection,
        PORTAL_BUS_NAME,
        PORTAL_OBJECT_PATH,
        SETTINGS_INTERFACE,
    )?);
}

/// Reads the accent color with Slint's exact decode:
///  `downcast_ref::<(f64, f64, f64)>()`.
fn read_accent(proxy: &zbus::blocking::Proxy<'_>) -> anyhow::Result<(f64, f64, f64)> {
    let value: OwnedValue = proxy.call("ReadOne", &(APPEARANCE_NAMESPACE, ACCENT_COLOR_KEY))?;
    return Ok(value.downcast_ref::<(f64, f64, f64)>()?);
}

/// An accent-only portal answers the accent through every read method and refuses the color scheme.
#[test]
fn accent_only_portal_serves_the_accent_and_not_the_scheme() -> anyhow::Result<()> {
    let portal = portal_with(None, Some("FF8000"))?;
    let proxy = client(&portal)?;
    assert_eq!(read_accent(&proxy)?, (1.0, 128.0 / 255.0, 0.0));
    let read: OwnedValue = proxy.call("Read", &(APPEARANCE_NAMESPACE, ACCENT_COLOR_KEY))?;
    assert_eq!(read.downcast_ref::<(f64, f64, f64)>()?, (1.0, 128.0 / 255.0, 0.0));
    let all: HashMap<String, HashMap<String, OwnedValue>> = proxy.call("ReadAll", &Vec::<String>::new())?;
    let appearance = all.get(APPEARANCE_NAMESPACE).context("ReadAll omitted the appearance namespace")?;
    assert!(appearance.contains_key(ACCENT_COLOR_KEY));
    assert!(!appearance.contains_key(COLOR_SCHEME_KEY));
    let scheme = proxy.call::<_, _, OwnedValue>("ReadOne", &(APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY));
    assert!(scheme.is_err(), "an accent-only portal must not invent a color scheme");
    assert!(!portal.serves_color_scheme() && portal.serves_accent_color());
    assert!(portal.set_color_scheme(ColorSchemePreference::Dark).is_err());
    return Ok(());
}

/// A scheme-only portal behaves exactly as before:
///  no accent is served or switchable.
#[test]
fn scheme_only_portal_does_not_serve_an_accent() -> anyhow::Result<()> {
    let portal = portal_with(Some(ColorSchemePreference::Light), None)?;
    let proxy = client(&portal)?;
    let accent = proxy.call::<_, _, OwnedValue>("ReadOne", &(APPEARANCE_NAMESPACE, ACCENT_COLOR_KEY));
    assert!(accent.is_err(), "no accent was requested, so none may be served");
    let all: HashMap<String, HashMap<String, OwnedValue>> = proxy.call("ReadAll", &Vec::<String>::new())?;
    let appearance = all.get(APPEARANCE_NAMESPACE).context("ReadAll omitted the appearance namespace")?;
    assert!(!appearance.contains_key(ACCENT_COLOR_KEY));
    assert!(portal.set_accent_color(AccentColor::parse("112233").map_err(anyhow::Error::msg)?).is_err());
    return Ok(());
}

/// A portal started with both options serves both and switches each on its own.
#[test]
fn both_settings_are_served_and_switch_independently() -> anyhow::Result<()> {
    let portal = portal_with(Some(ColorSchemePreference::Dark), Some("3584E4"))?;
    let proxy = client(&portal)?;
    let all: HashMap<String, HashMap<String, OwnedValue>> = proxy.call("ReadAll", &Vec::<String>::new())?;
    let appearance = all.get(APPEARANCE_NAMESPACE).context("ReadAll omitted the appearance namespace")?;
    assert_eq!(appearance.len(), 2);
    let red = AccentColor::parse("E01B24").map_err(anyhow::Error::msg)?;
    assert_eq!(portal.set_accent_color(red)?, SwitchOutcome::Changed);
    assert_eq!(read_accent(&proxy)?, red.portal_value());
    let scheme: OwnedValue = proxy.call("ReadOne", &(APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY))?;
    assert_eq!(scheme.downcast_ref::<u32>()?, 1);
    return Ok(());
}

/// `SettingChanged` carries `(s, s, v)` with a `(ddd)` structure,
///  decoded as Slint decodes it,
///  and only a real change emits.
#[test]
fn accent_change_reaches_a_subscriber_in_slints_shape() -> anyhow::Result<()> {
    let portal = portal_with(None, Some("3584E4"))?;
    let proxy = client(&portal)?;
    let signals = proxy.receive_signal("SettingChanged")?;
    let (sender, receiver) = mpsc::sync_channel(8);
    let _forwarder = std::thread::Builder::new()
        .name("accent-changed-forwarder".to_owned())
        .spawn(move || {
            for message in signals {
                if sender.send(message).is_err() {
                    return;
                }
            }
        })?;
    let same = AccentColor::parse("3584E4").map_err(anyhow::Error::msg)?;
    let orange = AccentColor::parse("FF7800").map_err(anyhow::Error::msg)?;
    // The unchanged request is the negative control: had it emitted, its signal would arrive first.
    assert_eq!(portal.set_accent_color(same)?, SwitchOutcome::Unchanged);
    assert_eq!(portal.set_accent_color(orange)?, SwitchOutcome::Changed);
    let message = receiver.recv_timeout(SIGNAL_TIMEOUT).context("no SettingChanged arrived")?;
    let body = message.body();
    assert_eq!(body.signature().to_string_no_parens(), "ssv");
    let (namespace, key, value): (String, String, OwnedValue) = body.deserialize()?;
    assert_eq!((namespace.as_str(), key.as_str()), (APPEARANCE_NAMESPACE, ACCENT_COLOR_KEY));
    assert_eq!(value.downcast_ref::<(f64, f64, f64)>()?, orange.portal_value());
    // The served value was already the announced one when the signal was delivered.
    assert_eq!(read_accent(&proxy)?, orange.portal_value());
    assert!(receiver.recv_timeout(Duration::from_millis(200)).is_err(), "the unchanged request must not emit");
    return Ok(());
}
