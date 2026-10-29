//! Focused tests for hosted-child environment isolation.

use std::ffi::{OsStr, OsString};

use super::*;

/// Returns explicit environment mutation stored on command builder.
fn environment_change(command: &Command, key: &str) -> Option<Option<OsString>> {
    return command
        .get_envs()
        .find(|(candidate, _value)| return *candidate == OsStr::new(key))
        .map(|(_key, value)| return value.map(OsStr::to_os_string));
}

/// Confirms graceful-shutdown deadline distinguishes pending and expired states.
#[test]
fn shutdown_deadline_expires_only_at_or_after_deadline() {
    let now = Instant::now();
    assert!(!shutdown_expired(None, now));
    assert!(!shutdown_expired(Some(now + Duration::from_secs(1)), now));
    assert!(shutdown_expired(Some(now), now));
}

/// Confirms nested socket and private bus replace inherited override variables.
#[test]
fn child_environment_isolates_wayland_and_private_bus() {
    let mut command = Command::new("app");
    command.env("WAYLAND_SOCKET", "9");
    command.env("DBUS_STARTER_ADDRESS", "host-address");
    command.env("DBUS_STARTER_BUS_TYPE", "session");
    configure_child_environment(
        &mut command,
        OsStr::new("wayland-nested"),
        Some("unix:path=/tmp/private-bus"),
        &[],
    );
    assert_eq!(
        environment_change(&command, "WAYLAND_DISPLAY"),
        Some(Some(OsString::from("wayland-nested"))),
    );
    assert_eq!(environment_change(&command, "WAYLAND_SOCKET"), Some(None));
    assert_eq!(
        environment_change(&command, "DBUS_SESSION_BUS_ADDRESS"),
        Some(Some(OsString::from("unix:path=/tmp/private-bus"))),
    );
    assert_eq!(environment_change(&command, "DBUS_STARTER_ADDRESS"), Some(None));
    assert_eq!(environment_change(&command, "DBUS_STARTER_BUS_TYPE"), Some(None));
}

/// Prevents clipboard libraries from falling back to the host X11 connection.
#[test]
fn child_environment_removes_host_x11_fallback() {
    // What: Command stores child-only environment overrides without mutating this process.
    // Why: Verify both inherited and explicit host display values using disposable state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const child = { env: { DISPLAY: ':host', XAUTHORITY: '/fixture/host-authority' } };
    // configureChildEnvironment(child);
    // ```
    let mut command = Command::new("app");
    command.env("DISPLAY", ":host");
    command.env("XAUTHORITY", "/fixture/host-authority");
    configure_child_environment(&mut command, OsStr::new("wayland-nested"), None, &[]);
    // Some(None) means explicitly removed, not merely absent from the override map.
    assert_eq!(environment_change(&command, "DISPLAY"), Some(None));
    assert_eq!(environment_change(&command, "XAUTHORITY"), Some(None));
}

/// Confirms omitted color scheme leaves inherited session bus untouched.
#[test]
fn child_environment_without_override_keeps_session_bus_inherited() {
    let mut command = Command::new("app");
    configure_child_environment(&mut command, OsStr::new("wayland-nested"), None, &[]);
    assert_eq!(environment_change(&command, "DBUS_SESSION_BUS_ADDRESS"), None);
    assert_eq!(environment_change(&command, "DBUS_STARTER_ADDRESS"), None);
    assert_eq!(environment_change(&command, "DBUS_STARTER_BUS_TYPE"), None);
}

/// Confirms the headless GPU variables are applied to the child:
/// a value is set,
/// `None` removes an inherited variable,
/// and an empty list leaves the environment alone.
#[test]
fn child_environment_applies_gpu_variables() {
    let mut command = Command::new("app");
    command.env("DRI_PRIME", "1");
    configure_child_environment(
        &mut command,
        OsStr::new("wayland-nested"),
        None,
        &[("LIBGL_ALWAYS_SOFTWARE", Some("1")), ("DRI_PRIME", None)],
    );
    assert_eq!(environment_change(&command, "LIBGL_ALWAYS_SOFTWARE"), Some(Some(OsString::from("1"))));
    assert_eq!(environment_change(&command, "DRI_PRIME"), Some(None));

    let mut untouched = Command::new("app");
    configure_child_environment(&mut untouched, OsStr::new("wayland-nested"), None, &[]);
    assert_eq!(environment_change(&untouched, "LIBGL_ALWAYS_SOFTWARE"), None);
    assert_eq!(environment_change(&untouched, "DRI_PRIME"), None);
}
