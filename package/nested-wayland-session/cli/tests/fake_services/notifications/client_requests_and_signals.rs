//! What a client asks the fake notification server to do,
//!  and the signals its requests raise:
//!  text that must stay on one line,
//!  a close with a reason,
//!  a client's own close,
//!  and an action invoked by its key.
//!
//! Split out of `notifications.rs` to keep that file under the code-line limit.

/// A table of bus values a call carries.
use std::collections::HashMap;

/// A private bus,
///  a fixture to drive,
///  and signal subscriptions.
use super::super::support::{
    Signals,
    err_message,
    field,
    ok_data,
    start,
};
/// The service mode that serves notifications.
use nested_wayland_session::notification_mode::NotificationMode;
/// The notification verbs this file runs against the fixture.
use nested_wayland_session::protocol::NotificationCommand;

/// The helpers the cases of this folder share with `notifications.rs`.
use super::{
    notifications_proxy,
    notify,
    records,
    verb,
};
/// Text chosen by the app cannot end the answer line or open a new field.
#[test]
fn hostile_text_stays_on_one_line() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let client = notifications_proxy(&fixture)?;
    let id = notify(
        &client,
        0,
        "done\nerr injected; id=99 x=1",
        &["a b", "L;1"],
        &HashMap::new(),
    )?;
    for payload in [
        ok_data(verb(&fixture, &NotificationCommand::List)),
        ok_data(verb(
            &fixture,
            &NotificationCommand::Log {
                count: 64,
            },
        )),
    ] {
        assert!(!payload.contains('\n'), "{payload:?}");
        assert_eq!(payload.matches("; ").count(), 0, "{payload:?}");
        assert_eq!(
            field(&payload, "summary"),
            Some("done%0Aerr%20injected%3B%20id%3D99%20x%3D1")
        );
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
        assert_eq!(
            ok_data(verb(
                &fixture,
                &NotificationCommand::Close {
                    id,
                    reason
                }
            )),
            ""
        );
        let message = closed.next("NotificationClosed");
        let (got_id, got_reason): (u32, u32) = message.body().deserialize()?;
        assert_eq!((got_id, got_reason), (id, reason));
    }
    assert!(ok_data(verb(&fixture, &NotificationCommand::List)).starts_with("0"));
    // A notification that is no longer visible cannot be closed again, and no signal leaves.
    let message = err_message(verb(
        &fixture,
        &NotificationCommand::Close {
            id: 1,
            reason: 2,
        },
    ));
    assert_eq!(message, "no visible notification with id 1");
    assert!(!closed.arrives_soon());
    let log = ok_data(verb(
        &fixture,
        &NotificationCommand::Log {
            count: 64,
        },
    ));
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
    let (unknown, unknown_reason): (u32, u32) =
        closed.next("NotificationClosed").body().deserialize()?;
    assert_eq!((unknown, unknown_reason), (4040, 3));
    assert!(ok_data(verb(&fixture, &NotificationCommand::List)).starts_with("0"));
    let log = ok_data(verb(
        &fixture,
        &NotificationCommand::Log {
            count: 64,
        },
    ));
    let entries = records(&log);
    let kinds: Vec<&str> = entries
        .iter()
        .filter_map(|entry| return entry.split(' ').nth(1))
        .collect();
    assert_eq!(
        kinds,
        ["notify", "close-call", "closed", "close-call", "closed"]
    );
    assert!(log.contains("close-call id=4040 known=false"), "{log}");
    return Ok(());
}

/// `notification-action` emits `ActionInvoked` for an offered key and refuses anything else.
#[test]
fn action_invocation_reaches_the_client() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let client = notifications_proxy(&fixture)?;
    let invoked = Signals::subscribe(&client, "ActionInvoked")?;
    let id = notify(
        &client,
        0,
        "Pick",
        &["default", "Open", "Action 1", "One"],
        &HashMap::new(),
    )?;
    for key in ["default", "Action 1"] {
        assert_eq!(
            ok_data(verb(
                &fixture,
                &NotificationCommand::Action {
                    id,
                    key: key.to_owned()
                }
            )),
            ""
        );
        let (got_id, got_key): (u32, String) =
            invoked.next("ActionInvoked").body().deserialize()?;
        assert_eq!((got_id, got_key.as_str()), (id, key));
    }
    let refusal = err_message(verb(
        &fixture,
        &NotificationCommand::Action {
            id,
            key: "snooze".to_owned(),
        },
    ));
    assert_eq!(
        refusal,
        "notification 1 offers no action snooze (offered: default, Action 1)"
    );
    let missing = err_message(verb(
        &fixture,
        &NotificationCommand::Action {
            id: 9,
            key: "default".to_owned(),
        },
    ));
    assert_eq!(missing, "no visible notification with id 9");
    assert!(!invoked.arrives_soon());
    // Invoking an action does not close the notification: the recorder leaves that to the test.
    assert!(ok_data(verb(&fixture, &NotificationCommand::List)).starts_with("1 "));
    let log = ok_data(verb(
        &fixture,
        &NotificationCommand::Log {
            count: 64,
        },
    ));
    assert!(log.contains("action-invoked id=1 key=Action%201"), "{log}");
    return Ok(());
}
