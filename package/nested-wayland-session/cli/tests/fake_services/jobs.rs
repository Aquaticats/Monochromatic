//! The fake Plasma job view server,
//!  driven by a real client that behaves like Qt's `KJob` tracker.

use std::collections::HashMap;

use nested_wayland_session::{
    control_job::run,
    job_server::{JOB_VIEW_SERVER_PATH, KUISERVER_NAME},
    job_view::JOB_VIEW_INTERFACE,
    notification_mode::NotificationMode,
    protocol::{JobCommand, Response},
};
use super::support::{err_message, field, ok_data, proxy, start, wait_until, Fixture, Signals};
use zbus::{
    blocking::Proxy,
    zvariant::{OwnedObjectPath, Value},
};

/// Runs a job verb against the fixture's services.
fn verb(fixture: &Fixture, command: &JobCommand) -> Response {
    return run(Some(&fixture.services), command);
}

/// The first record of the `jobs` answer.
fn first_job(fixture: &Fixture) -> String {
    let payload = ok_data(verb(fixture, &JobCommand::List));
    return payload.split_once(' ').map_or(String::new(), |(_, tail)| return tail.to_owned());
}

/// Requests a job view through the second interface version,
///  as Plasma-aware apps do.
fn request_view(fixture: &Fixture, capabilities: i32) -> anyhow::Result<(OwnedObjectPath, Proxy<'static>)> {
    let server = proxy(&fixture.client, KUISERVER_NAME, JOB_VIEW_SERVER_PATH, "org.kde.JobViewServerV2")?;
    let hints = HashMap::from([
        ("application-display-name", Value::from("Example Player")),
        ("application-icon-name", Value::from("multimedia-player")),
        ("transient", Value::Bool(true)),
    ]);
    let path: OwnedObjectPath = server.call("requestView", &("org.example.player", capabilities, hints))?;
    let job = proxy(&fixture.client, KUISERVER_NAME, path.as_str(), JOB_VIEW_INTERFACE)?;
    return Ok((path, job));
}

/// Sends an `update` the way Qt does:
///  with the no-reply flag set.
fn update(job: &Proxy<'static>, properties: &HashMap<&str, Value<'_>>) -> anyhow::Result<()> {
    job.call_noreply("update", &(properties,))?;
    return Ok(());
}

/// A job goes from `requestView` through updates,
///  a suspended flag,
///  and termination.
#[test]
fn job_lifecycle_from_request_to_terminate() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let (path, job) = request_view(&fixture, 3)?;
    assert_eq!(path.as_str(), "/org/kde/notificationmanager/jobs/JobView_1");
    let requested = first_job(&fixture);
    assert_eq!(field(&requested, "id"), Some("1"));
    assert_eq!(field(&requested, "entry"), Some("org.example.player"));
    assert_eq!(field(&requested, "capabilities"), Some("3"));
    assert_eq!(field(&requested, "percent"), Some("0"));
    assert_eq!(field(&requested, "terminated"), Some("false"));

    update(&job, &HashMap::from([
        ("title", Value::from("Analysing 3 tracks")),
        ("infoMessage", Value::from("track 1 of 3")),
        ("percent", Value::U32(10)),
        ("suspended", Value::Bool(false)),
        ("descriptionField", Value::from("ignored by Plasma")),
    ]))?;
    wait_until("the first update", || return field(&first_job(&fixture), "percent") == Some("10"));
    let updated = first_job(&fixture);
    assert_eq!(field(&updated, "title"), Some("Analysing%203%20tracks"));
    assert_eq!(field(&updated, "info"), Some("track%201%20of%203"));
    assert_eq!(field(&updated, "suspended"), Some("false"));

    // A later update without a title keeps the title; only the Pause flag and the bar move.
    update(&job, &HashMap::from([("percent", Value::U32(55)), ("suspended", Value::Bool(true))]))?;
    wait_until("the suspended flag", || return field(&first_job(&fixture), "suspended") == Some("true"));
    let paused = first_job(&fixture);
    assert_eq!(field(&paused, "title"), Some("Analysing%203%20tracks"));
    assert_eq!(field(&paused, "percent"), Some("55"));

    job.call_noreply("terminate", &(0_u32, "", HashMap::<&str, Value<'_>>::new()))?;
    wait_until("the job to end", || return field(&first_job(&fixture), "terminated") == Some("true"));
    let ended = first_job(&fixture);
    assert_eq!(field(&ended, "code"), Some("0"));
    assert_eq!(field(&ended, "message"), Some(""));
    assert_eq!(field(&ended, "percent"), Some("55"));

    // Plasma unregisters the job object when it ends, so a later call has nowhere to go.
    let late = job.call::<_, _, ()>("update", &(HashMap::<&str, Value<'_>>::new(),));
    assert!(late.is_err(), "the ended job object must be gone from the bus");
    return Ok(());
}

/// A failed job records its error code and message.
#[test]
fn terminate_records_the_error() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let (_path, job) = request_view(&fixture, 1)?;
    job.call_noreply("terminate", &(2_u32, "disk full; retry", HashMap::<&str, Value<'_>>::new()))?;
    wait_until("the job to end", || return field(&first_job(&fixture), "terminated") == Some("true"));
    let ended = first_job(&fixture);
    assert_eq!(field(&ended, "code"), Some("2"));
    assert_eq!(field(&ended, "message"), Some("disk%20full%3B%20retry"));
    return Ok(());
}

/// The three user signals and `updateRequested` reach the client on the job's path.
#[test]
fn job_signals_reach_the_client() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let (path, job) = request_view(&fixture, 3)?;
    let suspend = Signals::subscribe(&job, "suspendRequested")?;
    let resume = Signals::subscribe(&job, "resumeRequested")?;
    let cancel = Signals::subscribe(&job, "cancelRequested")?;
    let refresh = Signals::subscribe(&job, "updateRequested")?;

    assert_eq!(ok_data(verb(&fixture, &JobCommand::Suspend { id: 1 })), "");
    let message = suspend.next("suspendRequested");
    assert_eq!(message.header().path().map(|object| return object.as_str().to_owned()), Some(path.as_str().to_owned()));
    assert_eq!(ok_data(verb(&fixture, &JobCommand::Resume { id: 1 })), "");
    resume.next("resumeRequested");
    assert_eq!(ok_data(verb(&fixture, &JobCommand::Refresh { id: 1 })), "");
    refresh.next("updateRequested");
    assert_eq!(ok_data(verb(&fixture, &JobCommand::Cancel { id: 1 })), "");
    cancel.next("cancelRequested");
    // Each signal reached only its own subscription.
    assert!(!suspend.arrives_soon() && !resume.arrives_soon() && !cancel.arrives_soon());

    let log = ok_data(verb(&fixture, &JobCommand::Log { count: 64 }));
    for name in ["suspendRequested", "resumeRequested", "updateRequested", "cancelRequested"] {
        assert!(log.contains(&format!("signal id=1 name={name}")), "{name}: {log}");
    }
    return Ok(());
}

/// A signal goes to its own job only,
///  unknown and ended jobs are refused.
#[test]
fn job_signals_are_per_job_and_refused_after_the_end() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let (_first_path, first) = request_view(&fixture, 3)?;
    let (_second_path, second) = request_view(&fixture, 3)?;
    let first_cancel = Signals::subscribe(&first, "cancelRequested")?;
    let second_cancel = Signals::subscribe(&second, "cancelRequested")?;
    assert_eq!(ok_data(verb(&fixture, &JobCommand::Cancel { id: 2 })), "");
    second_cancel.next("cancelRequested");
    assert!(!first_cancel.arrives_soon());
    assert_eq!(err_message(verb(&fixture, &JobCommand::Cancel { id: 9 })), "no job with id 9");
    second.call_noreply("terminate", &(1_u32, "canceled", HashMap::<&str, Value<'_>>::new()))?;
    wait_until("job 2 to end", || return ok_data(verb(&fixture, &JobCommand::List)).contains("terminated=true"));
    assert_eq!(
        err_message(verb(&fixture, &JobCommand::Suspend { id: 2 })),
        "job 2 has ended; its object is gone from the bus",
    );
    return Ok(());
}

/// `update` and `terminate` are logged with the no-reply flag,
///  so a caller that waits is visible.
#[test]
fn the_log_shows_whether_the_caller_expected_a_reply() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let (_path, job) = request_view(&fixture, 3)?;
    update(&job, &HashMap::from([("percent", Value::U32(1))]))?;
    // The positive control: an ordinary call that waits for a reply is logged with the flag clear.
    job.call::<_, _, ()>("update", &(HashMap::from([("percent", Value::I32(2))]),))?;
    wait_until("both updates", || return ok_data(verb(&fixture, &JobCommand::Log { count: 64 })).matches(" update id=").count() == 2);
    let log = ok_data(verb(&fixture, &JobCommand::Log { count: 64 }));
    assert!(log.contains("update id=1 no-reply=true properties=percent:u:1"), "{log}");
    assert!(log.contains("update id=1 no-reply=false properties=percent:i:2"), "{log}");
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
    let old_server = proxy(&fixture.client, "org.kde.JobViewServer", JOB_VIEW_SERVER_PATH, "org.kde.JobViewServer")?;
    let path: OwnedObjectPath = old_server.call("requestView", &("org.example.old", "folder", 3_i32))?;
    assert_eq!(path.as_str(), "/org/kde/notificationmanager/jobs/JobView_1");
    let log = ok_data(verb(&fixture, &JobCommand::Log { count: 64 }));
    assert!(log.contains("request-view id=1 api=1 entry=org.example.old capabilities=3 hints=appIconName:s:folder"), "{log}");
    let (_second, ending) = request_view(&fixture, 0)?;
    ending.call_noreply("terminate", &(0_u32, "", HashMap::<&str, Value<'_>>::new()))?;
    wait_until("job 2 to end", || return ok_data(verb(&fixture, &JobCommand::List)).contains("terminated=true"));
    assert_eq!(ok_data(verb(&fixture, &JobCommand::Clear)), "");
    let remaining = ok_data(verb(&fixture, &JobCommand::List));
    assert!(remaining.starts_with("1 id=1 "), "{remaining}");
    assert_eq!(ok_data(verb(&fixture, &JobCommand::Log { count: 64 })), "0");
    return Ok(());
}

/// The `org.kde.kuiserver` methods answer as Plasma 6.7.5's source does.
#[test]
fn kuiserver_answers_like_plasma() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let server = proxy(&fixture.client, KUISERVER_NAME, JOB_VIEW_SERVER_PATH, "org.kde.kuiserver")?;
    let refusal = server.call::<_, _, ()>("registerService", &("org.example.service", "/Tracker"));
    let Err(zbus::Error::MethodError(name, message, _)) = refusal else {
        panic!("expected a method error, got {refusal:?}");
    };
    assert_eq!(name.as_str(), "org.freedesktop.DBus.Error.NotSupported");
    assert_eq!(message.as_deref(), Some("kuiserver proxying capabilities are deprecated and defunct."));
    let contacts: Vec<String> = server.call("registeredJobContacts", &())?;
    assert!(contacts.is_empty());
    let required: bool = server.call("requiresJobTracker", &())?;
    assert!(!required);
    let urls = Signals::subscribe(&server, "jobUrlsChanged")?;
    server.call::<_, _, ()>("emitJobUrlsChanged", &())?;
    let (listed,): (Vec<String>,) = urls.next("jobUrlsChanged").body().deserialize()?;
    assert!(listed.is_empty());
    let log = ok_data(verb(&fixture, &JobCommand::Log { count: 64 }));
    assert!(log.contains("kuiserver method=registerService"), "{log}");
    return Ok(());
}

/// A caller that sends an unsigned capabilities argument reaches no method,
///  as on Plasma.
#[test]
fn unsigned_capabilities_find_no_method() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let server = proxy(&fixture.client, KUISERVER_NAME, JOB_VIEW_SERVER_PATH, "org.kde.JobViewServerV2")?;
    let attempt = server.call::<_, _, OwnedObjectPath>(
        "requestView",
        &("org.example.player", 3_u32, HashMap::<&str, Value<'_>>::new()),
    );
    // zbus answers a mismatched signature with its own error name, Qt with "no such method";
    // either way the call fails and no job is granted.
    assert!(matches!(attempt, Err(zbus::Error::MethodError(..))), "{attempt:?}");
    assert_eq!(ok_data(verb(&fixture, &JobCommand::List)), "0");
    return Ok(());
}
