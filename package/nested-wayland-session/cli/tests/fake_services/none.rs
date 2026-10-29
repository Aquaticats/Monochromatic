//! `--notifications none` keeps today's behavior:
//!  no notification or job service exists on the bus.

use std::{sync::Arc, time::Instant};

use nested_wayland_session::{
    control_job, control_notification,
    notification_mode::NotificationMode,
    plasma_services::PlasmaServices,
    private_bus::PrivateBus,
    protocol::{JobCommand, NotificationCommand},
};
use super::support::{err_message, proxy};
use zbus::blocking::connection::Builder;

/// Starting with mode `none` serves nothing,
///  and a client's call gets `ServiceUnknown`.
#[test]
fn none_mode_leaves_both_services_unknown() -> anyhow::Result<()> {
    let bus = Arc::new(PrivateBus::start()?);
    let services = PlasmaServices::start(NotificationMode::None, Arc::clone(&bus), Instant::now())?;
    assert!(services.is_none());
    let client = Builder::address(bus.address())?.build()?;
    for (name, path, interface, method) in [
        ("org.freedesktop.Notifications", "/org/freedesktop/Notifications", "org.freedesktop.Notifications", "GetCapabilities"),
        ("org.kde.kuiserver", "/JobViewServer", "org.kde.kuiserver", "requiresJobTracker"),
        ("org.kde.JobViewServer", "/JobViewServer", "org.kde.JobViewServer", "requestView"),
    ] {
        let target = proxy(&client, name, path, interface);
        // A proxy to an unowned name may fail to build or fail on the call; both are ServiceUnknown.
        let error = match target {
            Ok(built) => built.call::<_, _, ()>(method, &()).expect_err(name),
            Err(error) => zbus::Error::Failure(error.to_string()),
        };
        let text = error.to_string();
        assert!(text.contains("ServiceUnknown") || text.contains("not provided by any"), "{name}: {text}");
    }
    return Ok(());
}

/// Without a service every notification and job verb refuses and names the option to add.
#[test]
fn verbs_refuse_without_a_service() {
    for command in [
        NotificationCommand::List,
        NotificationCommand::Log { count: 64 },
        NotificationCommand::Clear,
        NotificationCommand::Close { id: 1, reason: 2 },
        NotificationCommand::Action { id: 1, key: "default".to_owned() },
    ] {
        let message = err_message(control_notification::run(None, &command));
        assert_eq!(message, "no notification service (start with --notifications freedesktop or --notifications kde)");
    }
    for command in [
        JobCommand::List,
        JobCommand::Log { count: 64 },
        JobCommand::Clear,
        JobCommand::Suspend { id: 1 },
        JobCommand::Resume { id: 1 },
        JobCommand::Cancel { id: 1 },
        JobCommand::Refresh { id: 1 },
    ] {
        let message = err_message(control_job::run(None, &command));
        assert_eq!(message, "no Plasma job service (start with --notifications kde)");
    }
}
