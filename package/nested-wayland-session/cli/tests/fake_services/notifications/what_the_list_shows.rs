//! What the fake notification server's list and log answers carry:
//!  the ids it hands out,
//!  a replacement that keeps its id,
//!  a clear that reuses none,
//!  and the capabilities and server information it advertises.
//!
//! Split out of `notifications.rs` to keep that file under the code-line limit.

/// A table of bus values a call carries.
use std::collections::HashMap;

/// A private bus and a fixture to drive.
use super::super::support::{
    field,
    ok_data,
    start,
};
/// The service mode that serves notifications.
use nested_wayland_session::notification_mode::NotificationMode;
/// The capability list the server advertises.
use nested_wayland_session::notification_service::CAPABILITIES;
/// The notification verbs this file runs against the fixture.
use nested_wayland_session::protocol::NotificationCommand;
/// A bus value a notification call carries.
use zbus::Variant;

/// The helpers the cases of this folder share with `notifications.rs`.
use super::{
    notifications_proxy,
    notify,
    records,
    verb,
};
/// A call returns an increasing id,
///  shows with its hints and actions,
///  and a replacement keeps the id.
#[test]
fn notify_and_replacement() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let client = notifications_proxy(&fixture)?;
    let hints = HashMap::from([
        ("urgency", Variant::U8(2)),
        ("value", Variant::I32(50)),
        ("transient", Variant::Bool(true)),
        ("category", Variant::from("transfer")),
    ]);
    let first = notify(&client, 0, "Analysis started", &["default", "Open"], &hints)?;
    assert_eq!(first, 1);
    let shown = ok_data(verb(&fixture, &NotificationCommand::List));
    assert!(shown.starts_with("1 id=1 app=Example%20App icon=dialog-information summary=Analysis%20started body=Body%20text actions=default=Open "), "{shown}");
    assert_eq!(
        field(&shown, "hints"),
        Some("category:s:transfer,transient:b:true,urgency:y:2,value:i:50")
    );
    assert_eq!(field(&shown, "expire"), Some("5000"));
    assert_eq!(field(&shown, "replaced"), Some("0"));

    // Replace it twice: the id is returned unchanged and the count follows.
    assert_eq!(
        notify(&client, first, "Analysis 40 percent", &[], &HashMap::new())?,
        first
    );
    assert_eq!(
        notify(&client, first, "Analysis done", &[], &HashMap::new())?,
        first
    );
    let second = notify(&client, 0, "Other", &[], &HashMap::new())?;
    assert_eq!(second, 2);

    let after_replacement = ok_data(verb(&fixture, &NotificationCommand::List));
    let list = records(&after_replacement);
    assert_eq!(list.len(), 2, "{after_replacement}");
    assert_eq!(field(&list[0], "summary"), Some("Analysis%20done"));
    assert_eq!(field(&list[0], "replaced"), Some("2"));
    assert_eq!(field(&list[0], "actions"), Some(""));
    assert_eq!(field(&list[1], "id"), Some("2"));

    let log = ok_data(verb(
        &fixture,
        &NotificationCommand::Log {
            count: 64,
        },
    ));
    let entries = records(&log);
    assert_eq!(entries.len(), 4, "{log}");
    let replaces: Vec<&str> = entries
        .iter()
        .filter_map(|entry| return field(entry, "replaces"))
        .collect();
    assert_eq!(replaces, ["0", "1", "1", "0"]);
    assert!(
        entries
            .iter()
            .all(|entry| return entry.contains(" notify id="))
    );
    let times: Vec<u64> = entries
        .iter()
        .map(|entry| {
            return entry
                .split(' ')
                .next()
                .and_then(|part| return part.strip_prefix("t="))
                .and_then(|digits| return digits.parse().ok())
                .unwrap_or(u64::MAX);
        })
        .collect();
    assert!(
        times.windows(2).all(|pair| return pair[0] <= pair[1]),
        "{times:?}"
    );
    let newest = ok_data(verb(
        &fixture,
        &NotificationCommand::Log {
            count: 1,
        },
    ));
    assert!(newest.starts_with("1 t="), "{newest}");
    assert_eq!(field(&newest, "id"), Some("2"));
    return Ok(());
}

/// `notification-clear` forgets the records but never reuses an id.
#[test]
fn clear_forgets_records_and_keeps_ids_unique() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let client = notifications_proxy(&fixture)?;
    assert_eq!(notify(&client, 0, "a", &[], &HashMap::new())?, 1);
    assert_eq!(ok_data(verb(&fixture, &NotificationCommand::Clear)), "");
    assert_eq!(ok_data(verb(&fixture, &NotificationCommand::List)), "0");
    assert_eq!(
        ok_data(verb(
            &fixture,
            &NotificationCommand::Log {
                count: 64
            }
        )),
        "0"
    );
    assert_eq!(notify(&client, 0, "b", &[], &HashMap::new())?, 2);
    return Ok(());
}

/// The capability list and server information are the documented ones.
#[test]
fn capabilities_and_server_information() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let client = notifications_proxy(&fixture)?;
    let capabilities: Vec<String> = client.call("GetCapabilities", &())?;
    assert_eq!(capabilities, CAPABILITIES);
    for required in ["actions", "body", "persistence"] {
        assert!(
            capabilities
                .iter()
                .any(|capability| return capability == required),
            "{required}"
        );
    }
    for unsupported in ["sound", "inline-reply", "inhibitions"] {
        assert!(
            !capabilities
                .iter()
                .any(|capability| return capability == unsupported),
            "{unsupported}"
        );
    }
    let (name, vendor, version, specification): (String, String, String, String) =
        client.call("GetServerInformation", &())?;
    assert_eq!(name, "monochromatic-nested-wayland-session");
    assert_eq!(vendor, "Monochromatic");
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
    assert_eq!(specification, "1.2");
    return Ok(());
}
