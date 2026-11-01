//! The job answers keep hostile app text on one line.

/// Brings the formatters into scope.
use super::*;

/// Value summary used to build property lists.
use crate::{job_store::Termination, value_summary::ValueSummary};

/// Builds a running job whose text fields are hostile.
fn running_job() -> JobRecord {
    return JobRecord {
        id: 2,
        path: "/org/kde/notificationmanager/jobs/JobView_2".to_string(),
        desktop_entry: "org.example.app".to_string(),
        capabilities: 3,
        title: "Analysing\nok injected".to_string(),
        info_message: "track 3; of 9".to_string(),
        percent: 42,
        suspended: true,
        termination: None,
    };
}

/// A running job prints every documented field.
#[test]
fn running_job_line_has_the_documented_fields() {
    assert_eq!(
        format_job(&running_job()),
        "id=2 path=/org/kde/notificationmanager/jobs/JobView_2 entry=org.example.app \
         title=Analysing%0Aok%20injected info=track%203%3B%20of%209 percent=42 suspended=true \
         capabilities=3 terminated=false",
    );
}

/// A terminated job adds its code and message.
#[test]
fn terminated_job_adds_code_and_message() {
    let mut job = running_job();
    job.termination = Some(Termination { code: 2, message: "disk full".to_string() });
    assert!(format_job(&job).ends_with(" terminated=true code=2 message=disk%20full"));
    assert_eq!(format_job_list(&[]), "0");
    assert!(format_job_list(&[job.clone(), job]).starts_with("2 id=2 "));
}

/// Each event kind has its own record shape,
///  and the no-reply flag is visible.
#[test]
fn event_records_cover_every_kind() {
    let percent = ValueSummary { key: "percent".to_string(), signature: "u".to_string(), text: "5".to_string() };
    let request = Timed {
        at_ms: 1,
        event: JobEvent::RequestView {
            id: 1,
            api: 2,
            desktop_entry: "org.example.app".to_string(),
            capabilities: 3,
            hints: Vec::new(),
        },
    };
    assert_eq!(format_job_event(&request), "t=1 request-view id=1 api=2 entry=org.example.app capabilities=3 hints=");
    let update = Timed { at_ms: 2, event: JobEvent::Update { id: 1, no_reply: false, properties: vec![percent] } };
    assert_eq!(format_job_event(&update), "t=2 update id=1 no-reply=false properties=percent:u:5");
    let terminate = Timed {
        at_ms: 3,
        event: JobEvent::Terminate {
            id: 1,
            no_reply: true,
            code: 0,
            message: String::new(),
            hints: Vec::new(),
        },
    };
    assert_eq!(format_job_event(&terminate), "t=3 terminate id=1 no-reply=true code=0 message= hints=");
    let signal = Timed { at_ms: 4, event: JobEvent::Signal { id: 1, name: "cancelRequested".to_string() } };
    assert_eq!(format_job_event(&signal), "t=4 signal id=1 name=cancelRequested");
    let call = Timed {
        at_ms: 5,
        event: JobEvent::KuiserverCall { method: "registerService".to_string(), detail: "a b".to_string() },
    };
    assert_eq!(format_job_event(&call), "t=5 kuiserver method=registerService detail=a%20b");
    assert_eq!(format_job_event_list(&[&signal]), "1 t=4 signal id=1 name=cancelRequested");
}
