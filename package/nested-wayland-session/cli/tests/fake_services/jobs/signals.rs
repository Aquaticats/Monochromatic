//! The signals a Plasma job sends its client,
//!  and that each one reaches its own job only.
//!
//! Split out of `jobs.rs` to keep that file under the code-line limit.

/// A table of bus values the terminate call carries.
use std::collections::HashMap;

/// A private bus,
///  the services,
///  a client,
///  and signal subscriptions.
use super::super::support::{
    Signals,
    err_message,
    ok_data,
    start,
    wait_until,
};
/// The service mode that serves the Plasma job view server.
use nested_wayland_session::notification_mode::NotificationMode;
/// The job verbs this file runs against the fixture.
use nested_wayland_session::protocol::JobCommand;
/// A bus value the terminate call carries.
use zbus::Variant;

/// The helpers the cases of this folder share with `jobs.rs`.
use super::{
    request_view,
    verb,
};

/// The three user signals and `updateRequested` reach the client on the job's path.
#[test]
fn job_signals_reach_the_client() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    let (path, job) = request_view(&fixture, 3)?;
    let suspend = Signals::subscribe(&job, "suspendRequested")?;
    let resume = Signals::subscribe(&job, "resumeRequested")?;
    let cancel = Signals::subscribe(&job, "cancelRequested")?;
    let refresh = Signals::subscribe(&job, "updateRequested")?;

    assert_eq!(
        ok_data(verb(
            &fixture,
            &JobCommand::Suspend {
                id: 1
            }
        )),
        ""
    );
    let message = suspend.next("suspendRequested");
    assert_eq!(
        message
            .header()
            .path()
            .map(|object| return object.as_str().to_owned()),
        Some(path.as_str().to_owned())
    );
    assert_eq!(
        ok_data(verb(
            &fixture,
            &JobCommand::Resume {
                id: 1
            }
        )),
        ""
    );
    resume.next("resumeRequested");
    assert_eq!(
        ok_data(verb(
            &fixture,
            &JobCommand::Refresh {
                id: 1
            }
        )),
        ""
    );
    refresh.next("updateRequested");
    assert_eq!(
        ok_data(verb(
            &fixture,
            &JobCommand::Cancel {
                id: 1
            }
        )),
        ""
    );
    cancel.next("cancelRequested");
    // Each signal reached only its own subscription.
    assert!(!suspend.arrives_soon() && !resume.arrives_soon() && !cancel.arrives_soon());

    let log = ok_data(verb(
        &fixture,
        &JobCommand::Log {
            count: 64,
        },
    ));
    for name in [
        "suspendRequested",
        "resumeRequested",
        "updateRequested",
        "cancelRequested",
    ] {
        assert!(
            log.contains(&format!("signal id=1 name={name}")),
            "{name}: {log}"
        );
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
    assert_eq!(
        ok_data(verb(
            &fixture,
            &JobCommand::Cancel {
                id: 2
            }
        )),
        ""
    );
    second_cancel.next("cancelRequested");
    assert!(!first_cancel.arrives_soon());
    assert_eq!(
        err_message(verb(
            &fixture,
            &JobCommand::Cancel {
                id: 9
            }
        )),
        "no job with id 9"
    );
    second.call_noreply(
        "terminate",
        &(1_u32, "canceled", HashMap::<&str, Variant<'_>>::new()),
    )?;
    wait_until("job 2 to end", || {
        return ok_data(verb(&fixture, &JobCommand::List)).contains("terminated=true");
    });
    assert_eq!(
        err_message(verb(
            &fixture,
            &JobCommand::Suspend {
                id: 2
            }
        )),
        "job 2 has ended; its object is gone from the bus",
    );
    return Ok(());
}
