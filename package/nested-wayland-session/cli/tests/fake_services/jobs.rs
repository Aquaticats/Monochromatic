//! The fake Plasma job view server,
//!  driven by a real client that behaves like Qt's `KJob` tracker.

use std::collections::HashMap;

use super::support::{
    Fixture,
    field,
    ok_data,
    proxy,
    start,
    wait_until,
};
use nested_wayland_session::bus_test_client::Proxy;
use nested_wayland_session::{
    control_job::run,
    job_server::{
        JOB_VIEW_SERVER_PATH,
        KUISERVER_NAME,
    },
    job_view::JOB_VIEW_INTERFACE,
    notification_mode::NotificationMode,
    protocol::{
        JobCommand,
        Response,
    },
};
use zbus::{
    OwnedObjectPath,
    Variant,
};

/// The signals a job sends its client,
///  and that each reaches its own job only.
mod signals;

/// The server's older interfaces,
///  the log of its calls,
///  and how it types a capabilities argument.
mod older_interfaces;

/// Runs a job verb against the fixture's services.
fn verb(
    fixture: &Fixture,
    command: &JobCommand,
) -> Response {
    return run(Some(&fixture.services), command);
}

/// The first record of the `jobs` answer.
fn first_job(fixture: &Fixture) -> String {
    let payload = ok_data(verb(fixture, &JobCommand::List));
    return payload
        .split_once(' ')
        .map_or(String::new(), |(_, tail)| return tail.to_owned());
}

/// Requests a job view through the second interface version,
///  as Plasma-aware apps do.
fn request_view(
    fixture: &Fixture,
    capabilities: i32,
) -> anyhow::Result<(OwnedObjectPath, Proxy<'static>)> {
    let server = proxy(
        &fixture.client,
        KUISERVER_NAME,
        JOB_VIEW_SERVER_PATH,
        "org.kde.JobViewServerV2",
    )?;
    let hints = HashMap::from([
        ("application-display-name", Variant::from("Example Player")),
        ("application-icon-name", Variant::from("multimedia-player")),
        ("transient", Variant::Bool(true)),
    ]);
    let path: OwnedObjectPath =
        server.call("requestView", &("org.example.player", capabilities, hints))?;
    let job = proxy(
        &fixture.client,
        KUISERVER_NAME,
        path.as_str(),
        JOB_VIEW_INTERFACE,
    )?;
    return Ok((path, job));
}

/// Sends an `update` the way Qt does:
///  with the no-reply flag set.
fn update(
    job: &Proxy<'static>,
    properties: &HashMap<&str, Variant<'_>>,
) -> anyhow::Result<()> {
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

    update(
        &job,
        &HashMap::from([
            ("title", Variant::from("Analysing 3 tracks")),
            ("infoMessage", Variant::from("track 1 of 3")),
            ("percent", Variant::U32(10)),
            ("suspended", Variant::Bool(false)),
            ("descriptionField", Variant::from("ignored by Plasma")),
        ]),
    )?;
    wait_until("the first update", || {
        return field(&first_job(&fixture), "percent") == Some("10");
    });
    let updated = first_job(&fixture);
    assert_eq!(field(&updated, "title"), Some("Analysing%203%20tracks"));
    assert_eq!(field(&updated, "info"), Some("track%201%20of%203"));
    assert_eq!(field(&updated, "suspended"), Some("false"));

    // A later update without a title keeps the title; only the Pause flag and the bar move.
    update(
        &job,
        &HashMap::from([
            ("percent", Variant::U32(55)),
            ("suspended", Variant::Bool(true)),
        ]),
    )?;
    wait_until("the suspended flag", || {
        return field(&first_job(&fixture), "suspended") == Some("true");
    });
    let paused = first_job(&fixture);
    assert_eq!(field(&paused, "title"), Some("Analysing%203%20tracks"));
    assert_eq!(field(&paused, "percent"), Some("55"));

    job.call_noreply(
        "terminate",
        &(0_u32, "", HashMap::<&str, Variant<'_>>::new()),
    )?;
    wait_until("the job to end", || {
        return field(&first_job(&fixture), "terminated") == Some("true");
    });
    let ended = first_job(&fixture);
    assert_eq!(field(&ended, "code"), Some("0"));
    assert_eq!(field(&ended, "message"), Some(""));
    assert_eq!(field(&ended, "percent"), Some("55"));

    // Plasma unregisters the job object when it ends, so a later call has nowhere to go.
    let late = job.call::<_, _, ()>("update", &(HashMap::<&str, Variant<'_>>::new(),));
    assert!(
        late.is_err(),
        "the ended job object must be gone from the bus"
    );
    return Ok(());
}

/// A failed job records its error code and message.
#[test]
fn terminate_records_the_error() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let (_path, job) = request_view(&fixture, 1)?;
    job.call_noreply(
        "terminate",
        &(
            2_u32,
            "disk full; retry",
            HashMap::<&str, Variant<'_>>::new(),
        ),
    )?;
    wait_until("the job to end", || {
        return field(&first_job(&fixture), "terminated") == Some("true");
    });
    let ended = first_job(&fixture);
    assert_eq!(field(&ended, "code"), Some("2"));
    assert_eq!(field(&ended, "message"), Some("disk%20full%3B%20retry"));
    return Ok(());
}
