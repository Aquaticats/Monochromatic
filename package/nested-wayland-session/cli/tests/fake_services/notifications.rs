//! The fake `org.freedesktop.Notifications` server,
//!  driven by a real client on a real private bus.

use std::collections::HashMap;

use nested_wayland_session::{
    control_notification::run,
    notification_mode::NotificationMode,
    notification_service::{CAPABILITIES, NOTIFICATIONS_INTERFACE, NOTIFICATIONS_NAME, NOTIFICATIONS_PATH},
    protocol::NotificationCommand,
};
use super::support::{err_message, field, ok_data, proxy, start, Fixture, Signals};
use zbus::{blocking::Proxy, zvariant::Value};

/// Opens the client's proxy to the fake service by its well-known name.
fn notifications_proxy(fixture: &Fixture) -> anyhow::Result<Proxy<'static>> {
    return proxy(&fixture.client, NOTIFICATIONS_NAME, NOTIFICATIONS_PATH, NOTIFICATIONS_INTERFACE);
}

/// Calls `Notify` with every argument spelled out.
fn notify(
    client: &Proxy<'static>,
    replaces_id: u32,
    summary: &str,
    actions: &[&str],
    hints: &HashMap<&str, Value<'_>>,
) -> anyhow::Result<u32> {
    return Ok(client.call(
        "Notify",
        &("Example App", replaces_id, "dialog-information", summary, "Body text", actions, hints, 5000_i32),
    )?);
}

/// Runs a notification verb against the fixture's services.
fn verb(fixture: &Fixture, command: &NotificationCommand) -> nested_wayland_session::protocol::Response {
    return run(Some(&fixture.services), command);
}

/// Splits a `<count> <record>; <record>` payload into its records.
fn records(payload: &str) -> Vec<String> {
    let rest = payload.split_once(' ').map_or("", |(_, tail)| return tail);
    if rest.is_empty() {
        return Vec::new();
    }
    return rest.split("; ").map(str::to_owned).collect();
}

/// A call returns an increasing id,
///  shows with its hints and actions,
///  and a replacement keeps the id.
#[test]
fn notify_and_replacement() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let client = notifications_proxy(&fixture)?;
    let hints = HashMap::from([
        ("urgency", Value::U8(2)),
        ("value", Value::I32(50)),
        ("transient", Value::Bool(true)),
        ("category", Value::from("transfer")),
    ]);
    let first = notify(&client, 0, "Analysis started", &["default", "Open"], &hints)?;
    assert_eq!(first, 1);
    let shown = ok_data(verb(&fixture, &NotificationCommand::List));
    assert!(shown.starts_with("1 id=1 app=Example%20App icon=dialog-information summary=Analysis%20started body=Body%20text actions=default=Open "), "{shown}");
    assert_eq!(field(&shown, "hints"), Some("category:s:transfer,transient:b:true,urgency:y:2,value:i:50"));
    assert_eq!(field(&shown, "expire"), Some("5000"));
    assert_eq!(field(&shown, "replaced"), Some("0"));

    // Replace it twice: the id is returned unchanged and the count follows.
    assert_eq!(notify(&client, first, "Analysis 40 percent", &[], &HashMap::new())?, first);
    assert_eq!(notify(&client, first, "Analysis done", &[], &HashMap::new())?, first);
    let second = notify(&client, 0, "Other", &[], &HashMap::new())?;
    assert_eq!(second, 2);

    let after_replacement = ok_data(verb(&fixture, &NotificationCommand::List));
    let list = records(&after_replacement);
    assert_eq!(list.len(), 2, "{after_replacement}");
    assert_eq!(field(&list[0], "summary"), Some("Analysis%20done"));
    assert_eq!(field(&list[0], "replaced"), Some("2"));
    assert_eq!(field(&list[0], "actions"), Some(""));
    assert_eq!(field(&list[1], "id"), Some("2"));

    let log = ok_data(verb(&fixture, &NotificationCommand::Log { count: 64 }));
    let entries = records(&log);
    assert_eq!(entries.len(), 4, "{log}");
    let replaces: Vec<&str> = entries.iter().filter_map(|entry| return field(entry, "replaces")).collect();
    assert_eq!(replaces, ["0", "1", "1", "0"]);
    assert!(entries.iter().all(|entry| return entry.contains(" notify id=")));
    let times: Vec<u64> = entries
        .iter()
        .map(|entry| return entry.split(' ').next().and_then(|part| return part.strip_prefix("t=")).and_then(|digits| return digits.parse().ok()).unwrap_or(u64::MAX))
        .collect();
    assert!(times.windows(2).all(|pair| return pair[0] <= pair[1]), "{times:?}");
    let newest = ok_data(verb(&fixture, &NotificationCommand::Log { count: 1 }));
    assert!(newest.starts_with("1 t="), "{newest}");
    assert_eq!(field(&newest, "id"), Some("2"));
    return Ok(());
}

/// Text chosen by the app cannot end the answer line or open a new field.
#[test]
fn hostile_text_stays_on_one_line() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let client = notifications_proxy(&fixture)?;
    let id = notify(&client, 0, "done\nerr injected; id=99 x=1", &["a b", "L;1"], &HashMap::new())?;
    for payload in [
        ok_data(verb(&fixture, &NotificationCommand::List)),
        ok_data(verb(&fixture, &NotificationCommand::Log { count: 64 })),
    ] {
        assert!(!payload.contains('\n'), "{payload:?}");
        assert_eq!(payload.matches("; ").count(), 0, "{payload:?}");
        assert_eq!(field(&payload, "summary"), Some("done%0Aerr%20injected%3B%20id%3D99%20x%3D1"));
        assert_eq!(field(&payload, "actions"), Some("a%20b=L%3B1"));
    }
    assert_eq!(id, 1);
    return Ok(());
}

/// Control closes emit `NotificationClosed` with the reason asked for;
///  unknown ids are refused.
#[test]
fn close_reasons_reach_the_client() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let client = notifications_proxy(&fixture)?;
    let closed = Signals::subscribe(&client, "NotificationClosed")?;
    for reason in 1..=4_u32 {
        let id = notify(&client, 0, "temporary", &[], &HashMap::new())?;
        assert_eq!(ok_data(verb(&fixture, &NotificationCommand::Close { id, reason })), "");
        let message = closed.next("NotificationClosed");
        let (got_id, got_reason): (u32, u32) = message.body().deserialize()?;
        assert_eq!((got_id, got_reason), (id, reason));
    }
    assert!(ok_data(verb(&fixture, &NotificationCommand::List)).starts_with("0"));
    // A notification that is no longer visible cannot be closed again, and no signal leaves.
    let message = err_message(verb(&fixture, &NotificationCommand::Close { id: 1, reason: 2 }));
    assert_eq!(message, "no visible notification with id 1");
    assert!(!closed.arrives_soon());
    let log = ok_data(verb(&fixture, &NotificationCommand::Log { count: 64 }));
    assert_eq!(log.matches(" closed id=").count(), 4, "{log}");
    return Ok(());
}

/// A client's own `CloseNotification` removes the notification and is answered with reason 3,
///  for unknown ids too,
///  as Plasma does.
#[test]
fn client_close_is_announced_like_plasma() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let client = notifications_proxy(&fixture)?;
    let closed = Signals::subscribe(&client, "NotificationClosed")?;
    let id = notify(&client, 0, "temporary", &[], &HashMap::new())?;
    client.call::<_, _, ()>("CloseNotification", &(id,))?;
    let (got_id, reason): (u32, u32) = closed.next("NotificationClosed").body().deserialize()?;
    assert_eq!((got_id, reason), (id, 3));
    client.call::<_, _, ()>("CloseNotification", &(4040_u32,))?;
    let (unknown, unknown_reason): (u32, u32) = closed.next("NotificationClosed").body().deserialize()?;
    assert_eq!((unknown, unknown_reason), (4040, 3));
    assert!(ok_data(verb(&fixture, &NotificationCommand::List)).starts_with("0"));
    let log = ok_data(verb(&fixture, &NotificationCommand::Log { count: 64 }));
    let entries = records(&log);
    let kinds: Vec<&str> = entries.iter().filter_map(|entry| return entry.split(' ').nth(1)).collect();
    assert_eq!(kinds, ["notify", "close-call", "closed", "close-call", "closed"]);
    assert!(log.contains("close-call id=4040 known=false"), "{log}");
    return Ok(());
}

/// `notification-action` emits `ActionInvoked` for an offered key and refuses anything else.
#[test]
fn action_invocation_reaches_the_client() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let client = notifications_proxy(&fixture)?;
    let invoked = Signals::subscribe(&client, "ActionInvoked")?;
    let id = notify(&client, 0, "Pick", &["default", "Open", "Action 1", "One"], &HashMap::new())?;
    for key in ["default", "Action 1"] {
        assert_eq!(ok_data(verb(&fixture, &NotificationCommand::Action { id, key: key.to_owned() })), "");
        let (got_id, got_key): (u32, String) = invoked.next("ActionInvoked").body().deserialize()?;
        assert_eq!((got_id, got_key.as_str()), (id, key));
    }
    let refusal = err_message(verb(&fixture, &NotificationCommand::Action { id, key: "snooze".to_owned() }));
    assert_eq!(refusal, "notification 1 offers no action snooze (offered: default, Action 1)");
    let missing = err_message(verb(&fixture, &NotificationCommand::Action { id: 9, key: "default".to_owned() }));
    assert_eq!(missing, "no visible notification with id 9");
    assert!(!invoked.arrives_soon());
    // Invoking an action does not close the notification: the recorder leaves that to the test.
    assert!(ok_data(verb(&fixture, &NotificationCommand::List)).starts_with("1 "));
    let log = ok_data(verb(&fixture, &NotificationCommand::Log { count: 64 }));
    assert!(log.contains("action-invoked id=1 key=Action%201"), "{log}");
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
    assert_eq!(ok_data(verb(&fixture, &NotificationCommand::Log { count: 64 })), "0");
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
        assert!(capabilities.iter().any(|capability| return capability == required), "{required}");
    }
    for unsupported in ["sound", "inline-reply", "inhibitions"] {
        assert!(!capabilities.iter().any(|capability| return capability == unsupported), "{unsupported}");
    }
    let (name, vendor, version, specification): (String, String, String, String) =
        client.call("GetServerInformation", &())?;
    assert_eq!(name, "monochromatic-nested-wayland-session");
    assert_eq!(vendor, "Monochromatic");
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
    assert_eq!(specification, "1.2");
    return Ok(());
}

/// `--notifications freedesktop` serves no Plasma job server,
///  and its job verbs say how to get one.
#[test]
fn freedesktop_mode_has_no_job_server() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let job_proxy = proxy(&fixture.client, "org.kde.kuiserver", "/JobViewServer", "org.kde.JobViewServerV2")?;
    let attempt = job_proxy.call::<_, _, zbus::zvariant::OwnedObjectPath>("requestView", &("org.example", 3_i32, HashMap::<&str, Value<'_>>::new()));
    let Err(zbus::Error::MethodError(name, ..)) = attempt else {
        panic!("expected a method error, got {attempt:?}");
    };
    assert_eq!(name.as_str(), "org.freedesktop.DBus.Error.ServiceUnknown");
    let refusal = err_message(nested_wayland_session::control_job::run(
        Some(&fixture.services),
        &nested_wayland_session::protocol::JobCommand::List,
    ));
    assert_eq!(refusal, "no Plasma job service (start with --notifications kde)");
    return Ok(());
}
