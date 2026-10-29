//! Focused tests for nested color-scheme portal isolation.

use super::*;

/// Process spawning, the re-executed test helpers, and timing that the moved bus code no longer brings in.
use std::{process::Command, sync::mpsc, time::Duration};

/// Starts the portal on a fresh private bus that lives as long as the returned portal.
///
/// What:
///      `fn start_portal(preference) -> anyhow::Result<AppearancePortal>`.
/// Why:
///       Production starts one bus and shares it; each test needs its own bus to stay isolated.
fn start_portal(preference: ColorSchemePreference) -> anyhow::Result<AppearancePortal> {
    return AppearancePortal::start(preference, Arc::new(PrivateBus::start()?))
}

/// Confirms CLI accepts exact supported scheme names and rejects others.
#[test]
fn color_scheme_parser_accepts_dark_and_light() {
    assert_eq!(ColorSchemePreference::parse("dark").expect("dark is a valid color scheme"), ColorSchemePreference::Dark);
    assert_eq!(ColorSchemePreference::parse("light").expect("light is a valid color scheme"), ColorSchemePreference::Light);
    assert!(ColorSchemePreference::parse("system").is_err());
}

/// Confirms real private bus serves Slint's `ReadOne` method and cleans up.
#[test]
fn private_portal_serves_light_scheme_without_host_bus() -> anyhow::Result<()> {
    let portal = start_portal(ColorSchemePreference::Light)?;
    let directory = portal.bus.directory().to_path_buf();
    let connection = zbus::blocking::connection::Builder::address(portal.bus_address())?.build()?;
    let proxy = zbus::blocking::Proxy::new(
        &connection,
        PORTAL_BUS_NAME,
        PORTAL_OBJECT_PATH,
        "org.freedesktop.portal.Settings",
    )?;
    let value: OwnedValue = proxy.call("ReadOne", &(APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY))?;
    assert_eq!(value.downcast_ref::<u32>()?, 2);
    drop(proxy);
    drop(connection);
    drop(portal);
    assert!(!directory.exists());
    return Ok(())
}

/// Confirms standard `ReadAll` serves dark value for non-Slint portal clients.
#[test]
fn private_portal_serves_dark_scheme_through_read_all() -> anyhow::Result<()> {
    let portal = start_portal(ColorSchemePreference::Dark)?;
    let connection = zbus::blocking::connection::Builder::address(portal.bus_address())?.build()?;
    let proxy = zbus::blocking::Proxy::new(
        &connection,
        PORTAL_BUS_NAME,
        PORTAL_OBJECT_PATH,
        "org.freedesktop.portal.Settings",
    )?;
    let values: HashMap<String, HashMap<String, OwnedValue>> =
        proxy.call("ReadAll", &Vec::<String>::new())?;
    let appearance = values
        .get(APPEARANCE_NAMESPACE)
        .context("ReadAll omitted appearance namespace")?;
    let value = appearance
        .get(COLOR_SCHEME_KEY)
        .context("ReadAll omitted color-scheme")?;
    assert_eq!(value.downcast_ref::<u32>()?, 1);
    return Ok(())
}

/// Confirms unsupported settings return D-Bus errors instead of invented defaults.
#[test]
fn private_portal_rejects_unknown_setting() -> anyhow::Result<()> {
    let portal = start_portal(ColorSchemePreference::Dark)?;
    let connection = zbus::blocking::connection::Builder::address(portal.bus_address())?.build()?;
    let proxy = zbus::blocking::Proxy::new(
        &connection,
        PORTAL_BUS_NAME,
        PORTAL_OBJECT_PATH,
        "org.freedesktop.portal.Settings",
    )?;
    let result = proxy.call::<_, _, OwnedValue>("ReadOne", &("org.example", "missing"));
    assert!(result.is_err());
    return Ok(())
}

/// Marks the re-executed test process whose data directories hold a planted service file.
const PLANTED_ROLE_VARIABLE: &str = "NESTED_WAYLAND_SESSION_TEST_PLANTED_ROLE";

/// Names the service planted where a stock session bus looks for service files.
const PLANTED_SERVICE: &str = "org.example.MonochromaticPlantedService";

/// Asks the private bus to list and start services;
///  runs only inside the re-executed process.
///
/// A client connected to the private bus is exactly what a hosted toolkit is,
///  so these are the
/// same requests that made the stock configuration try to execute `org.a11y.Bus`.
fn run_planted_role() -> anyhow::Result<()> {
    let portal = start_portal(ColorSchemePreference::Dark)?;
    let connection = zbus::blocking::connection::Builder::address(portal.bus_address())?.build()?;
    let bus = zbus::blocking::fdo::DBusProxy::new(&connection)?;
    let mut activatable = Vec::new();
    for name in bus.list_activatable_names()? {
        activatable.push(name.to_string());
    }
    // Only the bus driver itself: no service directory was read, planted or host.
    assert_eq!(activatable, ["org.freedesktop.DBus"]);
    for service in [PLANTED_SERVICE, "org.a11y.Bus"] {
        let started = bus.start_service_by_name(zbus::names::WellKnownName::try_from(service)?, 0);
        // let-else binds the refusal message or stops the test with the unexpected outcome.
        let Err(zbus::fdo::Error::ServiceUnknown(message)) = started else {
            panic!("expected ServiceUnknown for {service}, got {started:?}");
        };
        assert!(message.contains(service), "{message}");
    }
    return Ok(())
}

/// Confirms a hosted client cannot make the private bus start a service from any service file.
///
/// The test re-executes itself with `XDG_DATA_HOME` and `XDG_DATA_DIRS` naming a disposable
/// directory that holds one planted `.service` file.
///  The stock session configuration searches
/// exactly those directories,
///  so it would list and try to start the planted service;
///  that is
/// the positive control,
///  independent of which services this machine has installed.
#[test]
fn private_bus_never_reads_service_files() -> anyhow::Result<()> {
    if std::env::var_os(PLANTED_ROLE_VARIABLE).is_some() {
        return run_planted_role();
    }
    let root = std::env::temp_dir().join(format!(
        "monochromatic-nested-planted-services-{}",
        std::process::id(),
    ));
    let services = root.join("dbus-1").join("services");
    std::fs::create_dir_all(&services)?;
    // The program path does not exist, so even a stock bus could never run anything here.
    std::fs::write(
        services.join(format!("{PLANTED_SERVICE}.service")),
        format!("[D-BUS Service]\nName={PLANTED_SERVICE}\nExec=/nonexistent/monochromatic-planted-service\n"),
    )?;
    let status = Command::new(std::env::current_exe()?)
        .arg("--exact")
        .arg("appearance_portal::tests::private_bus_never_reads_service_files")
        .arg("--nocapture")
        .env(PLANTED_ROLE_VARIABLE, "planted")
        .env("XDG_DATA_HOME", &root)
        .env("XDG_DATA_DIRS", &root)
        .status();
    std::fs::remove_dir_all(&root)?;
    let finished = status?;
    assert!(finished.success(), "re-executed planted-service role failed: {finished}");
    return Ok(())
}

/// Bounds how long a test waits for a signal that must already be on the private bus.
const SIGNAL_TIMEOUT: Duration = Duration::from_secs(2);

/// Marks the re-executed test process that switches under a decoy session-bus environment.
const SWITCH_ROLE_VARIABLE: &str = "NESTED_WAYLAND_SESSION_TEST_SWITCH_ROLE";

/// Opens a second client connection and Settings proxy the way a hosted toolkit does.
///
/// The destination is the well-known portal name,
///  so zbus applies the same sender filter
/// Slint's `receive_signal("SettingChanged")` subscription relies on.
fn settings_proxy(portal: &AppearancePortal) -> anyhow::Result<zbus::blocking::Proxy<'static>> {
    let connection = zbus::blocking::connection::Builder::address(portal.bus_address())?.build()?;
    let proxy = zbus::blocking::Proxy::new(
        &connection,
        PORTAL_BUS_NAME,
        PORTAL_OBJECT_PATH,
        SETTINGS_INTERFACE,
    )?;
    return Ok(proxy)
}

/// Reads the color scheme through `ReadOne`,
///  deprecated `Read`,
///  and `ReadAll`,
///  in that order.
fn served_values(proxy: &zbus::blocking::Proxy<'_>) -> anyhow::Result<[u32; 3]> {
    let read_one: OwnedValue = proxy.call("ReadOne", &(APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY))?;
    let read: OwnedValue = proxy.call("Read", &(APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY))?;
    let all: HashMap<String, HashMap<String, OwnedValue>> =
        proxy.call("ReadAll", &Vec::<String>::new())?;
    let read_all = all
        .get(APPEARANCE_NAMESPACE)
        .context("ReadAll omitted appearance namespace")?
        .get(COLOR_SCHEME_KEY)
        .context("ReadAll omitted color-scheme")?;
    return Ok([
        read_one.downcast_ref::<u32>()?,
        read.downcast_ref::<u32>()?,
        read_all.downcast_ref::<u32>()?,
    ])
}

/// Decodes one `SettingChanged` body exactly as Slint 1.18.1 does,
///  plus its wire signature.
fn decode_setting_changed(message: &zbus::Message) -> anyhow::Result<(String, String, u32, String)> {
    let body = message.body();
    // The wire header carries the body signature without the outer tuple parentheses.
    let signature = body.signature().to_string_no_parens();
    let (namespace, key, value): (String, String, OwnedValue) = body.deserialize()?;
    return Ok((namespace, key, value.downcast_ref::<u32>()?, signature))
}

/// Confirms every read method serves the switched value,
///  in both directions.
#[test]
fn runtime_switch_changes_value_served_by_every_read_method() -> anyhow::Result<()> {
    let portal = start_portal(ColorSchemePreference::Dark)?;
    let proxy = settings_proxy(&portal)?;
    assert_eq!(served_values(&proxy)?, [1, 1, 1]);
    assert_eq!(portal.set_color_scheme(ColorSchemePreference::Light)?, SwitchOutcome::Changed);
    assert_eq!(served_values(&proxy)?, [2, 2, 2]);
    assert_eq!(portal.set_color_scheme(ColorSchemePreference::Dark)?, SwitchOutcome::Changed);
    assert_eq!(served_values(&proxy)?, [1, 1, 1]);
    return Ok(())
}

/// Confirms `SettingChanged` reaches a toolkit-style subscriber for real changes only.
///
/// Signals arrive in emission order,
///  so the unchanged requests are the negative control:
/// had either emitted,
///  its value would arrive before the value asserted next.
#[test]
fn runtime_switch_emits_setting_changed_only_for_real_changes() -> anyhow::Result<()> {
    let portal = start_portal(ColorSchemePreference::Dark)?;
    let proxy = settings_proxy(&portal)?;
    // Subscribe before switching, as Slint subscribes before its initial read.
    let signals = proxy.receive_signal("SettingChanged")?;
    let reader = settings_proxy(&portal)?;
    let (sender, receiver) = mpsc::sync_channel(8);
    // The blocking iterator has no timeout, so a helper thread forwards into a bounded wait.
    let _forwarder = std::thread::Builder::new()
        .name("setting-changed-forwarder".to_owned())
        .spawn(move || {
            for message in signals {
                // Read back at notification time, as a client that re-reads on change would.
                let read_back = served_values(&reader);
                if sender.send((message, read_back)).is_err() {
                    return;
                }
            }
        })?;

    assert_eq!(portal.set_color_scheme(ColorSchemePreference::Dark)?, SwitchOutcome::Unchanged);
    assert_eq!(portal.set_color_scheme(ColorSchemePreference::Light)?, SwitchOutcome::Changed);
    let (first, first_read_back) = receiver
        .recv_timeout(SIGNAL_TIMEOUT)
        .context("no SettingChanged arrived after switching dark to light")?;
    // The served value must already be the announced one when the signal is delivered.
    assert_eq!(first_read_back?, [2, 2, 2]);
    assert_eq!(
        decode_setting_changed(&first)?,
        (APPEARANCE_NAMESPACE.to_owned(), COLOR_SCHEME_KEY.to_owned(), 2, "ssv".to_owned()),
    );

    assert_eq!(portal.set_color_scheme(ColorSchemePreference::Light)?, SwitchOutcome::Unchanged);
    assert_eq!(portal.set_color_scheme(ColorSchemePreference::Dark)?, SwitchOutcome::Changed);
    let (second, second_read_back) = receiver
        .recv_timeout(SIGNAL_TIMEOUT)
        .context("no SettingChanged arrived after switching light to dark")?;
    assert_eq!(second_read_back?, [1, 1, 1]);
    assert_eq!(
        decode_setting_changed(&second)?,
        (APPEARANCE_NAMESPACE.to_owned(), COLOR_SCHEME_KEY.to_owned(), 1, "ssv".to_owned()),
    );
    return Ok(())
}

/// Performs one complete private switch;
///  runs only inside the re-executed test process.
fn run_switch_role() -> anyhow::Result<()> {
    let portal = start_portal(ColorSchemePreference::Dark)?;
    let proxy = settings_proxy(&portal)?;
    assert_eq!(portal.set_color_scheme(ColorSchemePreference::Light)?, SwitchOutcome::Changed);
    assert_eq!(served_values(&proxy)?, [2, 2, 2]);
    return Ok(())
}

/// Confirms a runtime switch never connects to the session bus named by the environment.
///
/// A disposable decoy bus stands in for the host session bus.
///  This test re-executes itself
/// with `DBUS_SESSION_BUS_ADDRESS` naming the decoy,
///  so code that consults the environment
/// would connect there,
///  never to a real desktop.
///  Every connection to a bus is announced by a
/// `NameOwnerChanged` signal in order;
///  the sentinel connection made afterwards is the
/// positive control,
///  and it must be the first connection the decoy reports.
#[test]
fn runtime_switch_never_contacts_environment_session_bus() -> anyhow::Result<()> {
    if std::env::var_os(SWITCH_ROLE_VARIABLE).is_some() {
        return run_switch_role();
    }
    let decoy = PrivateBus::start()?;
    let watcher = zbus::blocking::connection::Builder::address(decoy.address())?.build()?;
    let bus = zbus::blocking::fdo::DBusProxy::new(&watcher)?;
    let mut connections = bus.receive_name_owner_changed()?;

    let status = Command::new(std::env::current_exe()?)
        .arg("--exact")
        .arg("appearance_portal::tests::runtime_switch_never_contacts_environment_session_bus")
        .arg("--nocapture")
        .env(SWITCH_ROLE_VARIABLE, "switch")
        .env("DBUS_SESSION_BUS_ADDRESS", decoy.address())
        .env_remove("DBUS_STARTER_ADDRESS")
        .env_remove("DBUS_STARTER_BUS_TYPE")
        .status()?;
    assert!(status.success(), "re-executed switch role failed: {status}");

    let sentinel = zbus::blocking::connection::Builder::address(decoy.address())?.build()?;
    let sentinel_name = sentinel
        .unique_name()
        .context("sentinel connection has no unique name")?
        .to_string();
    let first = connections
        .next()
        .context("decoy bus stopped before reporting the sentinel connection")?;
    let arguments = first.args()?;
    assert_eq!(
        arguments.name().to_string(),
        sentinel_name,
        "something connected to the environment session bus before the sentinel",
    );
    return Ok(())
}
