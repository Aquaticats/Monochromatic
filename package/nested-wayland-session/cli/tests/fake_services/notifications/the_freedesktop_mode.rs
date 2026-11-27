//! The mode that serves notifications alone:
//!  no Plasma job server,
//!  and a job verb that says how to get one.
//!
//! Split out of `notifications.rs` to keep that file under the code-line limit.

/// A table of bus values a request carries.
use std::collections::HashMap;

/// A private bus,
///  a fixture to drive,
///  and a client proxy.
use super::super::support::{
    err_message,
    error_reply,
    proxy,
    start,
};
/// The service mode that serves notifications alone.
use nested_wayland_session::notification_mode::NotificationMode;
/// A bus value the stale job request carries.
use zbus::Variant;
/// `--notifications freedesktop` serves no Plasma job server,
///  and its job verbs say how to get one.
#[test]
fn freedesktop_mode_has_no_job_server() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let job_proxy = proxy(
        &fixture.client,
        "org.kde.kuiserver",
        "/JobViewServer",
        "org.kde.JobViewServerV2",
    )?;
    let attempt = job_proxy.call::<_, _, zbus::OwnedObjectPath>(
        "requestView",
        &("org.example", 3_i32, HashMap::<&str, Variant<'_>>::new()),
    );
    let Err(error) = &attempt else {
        panic!("expected a bus error, got {attempt:?}");
    };
    let (name, _) =
        error_reply(error).unwrap_or_else(|| panic!("expected a bus error reply, got {error:?}"));
    assert_eq!(name, "org.freedesktop.DBus.Error.ServiceUnknown");
    let refusal = err_message(nested_wayland_session::control_job::run(
        Some(&fixture.services),
        &nested_wayland_session::protocol::JobCommand::List,
    ));
    assert_eq!(
        refusal,
        "no Plasma job service (start with --notifications kde)"
    );
    return Ok(());
}
