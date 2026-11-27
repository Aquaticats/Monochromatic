//! What the Plasma job server answers on its older interfaces,
//!  which calls the log records,
//!  and how it types a capabilities argument.
//!
//! Split out of `jobs.rs` to keep that file under the code-line limit.

/// A table of bus values an update or a request carries.
use std::collections::HashMap;

/// A private bus,
///  the services,
///  a client,
///  and signal subscriptions.
use super::super::support::{
    Signals,
    error_reply,
    field,
    ok_data,
    proxy,
    start,
    wait_until,
};
/// The well-known name and object path of the job view server.
use nested_wayland_session::job_server::{
    JOB_VIEW_SERVER_PATH,
    KUISERVER_NAME,
};
/// The service mode that serves the Plasma job view server.
use nested_wayland_session::notification_mode::NotificationMode;
/// The job verbs this file runs against the fixture.
use nested_wayland_session::protocol::JobCommand;
/// The bus object path a granted view is named by.
use zbus::{
    OwnedObjectPath,
    Variant,
};

/// The helpers the cases of this folder share with `jobs.rs`.
use super::{
    first_job,
    request_view,
    update,
    verb,
};
/// `update` and `terminate` are logged with the no-reply flag,
///  so a caller that waits is visible.
#[test]
fn the_log_shows_whether_the_caller_expected_a_reply() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let (_path, job) = request_view(&fixture, 3)?;
    update(&job, &HashMap::from([("percent", Variant::U32(1))]))?;
    // The positive control: an ordinary call that waits for a reply is logged with the flag clear.
    job.call::<_, _, ()>("update", &(HashMap::from([("percent", Variant::I32(2))]),))?;
    wait_until("both updates", || {
        return ok_data(verb(
            &fixture,
            &JobCommand::Log {
                count: 64,
            },
        ))
        .matches(" update id=")
        .count()
            == 2;
    });
    let log = ok_data(verb(
        &fixture,
        &JobCommand::Log {
            count: 64,
        },
    ));
    assert!(
        log.contains("update id=1 no-reply=true properties=percent:u:1"),
        "{log}"
    );
    assert!(
        log.contains("update id=1 no-reply=false properties=percent:i:2"),
        "{log}"
    );
    // An int32 percent still moves the bar, as Plasma's QVariant conversion does.
    assert_eq!(field(&first_job(&fixture), "percent"), Some("2"));
    assert!(log.contains("request-view id=1 api=2 entry=org.example.player capabilities=3 hints=application-display-name:s:Example%20Player"), "{log}");
    return Ok(());
}

/// The first interface version grants views too,
///  and `job-clear` keeps running jobs.
#[test]
fn first_interface_version_and_clear() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let old_server = proxy(
        &fixture.client,
        "org.kde.JobViewServer",
        JOB_VIEW_SERVER_PATH,
        "org.kde.JobViewServer",
    )?;
    let path: OwnedObjectPath =
        old_server.call("requestView", &("org.example.old", "folder", 3_i32))?;
    assert_eq!(path.as_str(), "/org/kde/notificationmanager/jobs/JobView_1");
    let log = ok_data(verb(
        &fixture,
        &JobCommand::Log {
            count: 64,
        },
    ));
    assert!(log.contains("request-view id=1 api=1 entry=org.example.old capabilities=3 hints=appIconName:s:folder"), "{log}");
    let (_second, ending) = request_view(&fixture, 0)?;
    ending.call_noreply(
        "terminate",
        &(0_u32, "", HashMap::<&str, Variant<'_>>::new()),
    )?;
    wait_until("job 2 to end", || {
        return ok_data(verb(&fixture, &JobCommand::List)).contains("terminated=true");
    });
    assert_eq!(ok_data(verb(&fixture, &JobCommand::Clear)), "");
    let remaining = ok_data(verb(&fixture, &JobCommand::List));
    assert!(remaining.starts_with("1 id=1 "), "{remaining}");
    assert_eq!(
        ok_data(verb(
            &fixture,
            &JobCommand::Log {
                count: 64
            }
        )),
        "0"
    );
    return Ok(());
}

/// The `org.kde.kuiserver` methods answer as Plasma 6.7.5's source does.
#[test]
fn kuiserver_answers_like_plasma() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let server = proxy(
        &fixture.client,
        KUISERVER_NAME,
        JOB_VIEW_SERVER_PATH,
        "org.kde.kuiserver",
    )?;
    let refusal = server.call::<_, _, ()>("registerService", &("org.example.service", "/Tracker"));
    let Err(error) = &refusal else {
        panic!("expected a bus error, got {refusal:?}");
    };
    let (name, message) =
        error_reply(error).unwrap_or_else(|| panic!("expected a bus error reply, got {error:?}"));
    assert_eq!(name, "org.freedesktop.DBus.Error.NotSupported");
    assert_eq!(
        message.as_deref(),
        Some("kuiserver proxying capabilities are deprecated and defunct.")
    );
    let contacts: Vec<String> = server.call("registeredJobContacts", &())?;
    assert!(contacts.is_empty());
    let required: bool = server.call("requiresJobTracker", &())?;
    assert!(!required);
    let urls = Signals::subscribe(&server, "jobUrlsChanged")?;
    server.call::<_, _, ()>("emitJobUrlsChanged", &())?;
    let (listed,): (Vec<String>,) = urls.next("jobUrlsChanged").body().deserialize()?;
    assert!(listed.is_empty());
    let log = ok_data(verb(
        &fixture,
        &JobCommand::Log {
            count: 64,
        },
    ));
    assert!(log.contains("kuiserver method=registerService"), "{log}");
    return Ok(());
}

/// A caller that sends an unsigned capabilities argument reaches no method,
///  as on Plasma.
#[test]
fn unsigned_capabilities_find_no_method() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let server = proxy(
        &fixture.client,
        KUISERVER_NAME,
        JOB_VIEW_SERVER_PATH,
        "org.kde.JobViewServerV2",
    )?;
    let attempt = server.call::<_, _, OwnedObjectPath>(
        "requestView",
        &(
            "org.example.player",
            3_u32,
            HashMap::<&str, Variant<'_>>::new(),
        ),
    );
    // zbus answers a mismatched signature with its own error name, Qt with "no such method";
    // either way the call fails and no job is granted.
    let Err(error) = &attempt else {
        panic!("expected the call to fail, got {attempt:?}");
    };
    assert!(error_reply(error).is_some(), "{attempt:?}");
    assert_eq!(ok_data(verb(&fixture, &JobCommand::List)), "0");
    return Ok(());
}
