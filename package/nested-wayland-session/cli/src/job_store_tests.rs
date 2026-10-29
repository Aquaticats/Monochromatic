//! Numbering,
//!  property merging,
//!  and termination of the fake job store.

/// Brings the store and its types into scope.
use super::*;

/// Builds a request for the second interface version.
fn request(capabilities: i32) -> JobRequest {
    return JobRequest { api: 2, desktop_entry: "org.example.app".to_string(), capabilities, hints: Vec::new() };
}

/// Builds an update that changes the given properties.
fn update(id: u32, changes: JobChanges) -> UpdateCall {
    return UpdateCall { id, no_reply: true, changes, properties: Vec::new() };
}

/// Jobs are numbered from one and their paths end in the number.
#[test]
fn jobs_are_numbered_from_one_with_plasma_paths() {
    let mut store = JobStore::default();
    let first = store.request_view(1, request(3));
    let second = store.request_view(2, request(0));
    assert_eq!((first.id, second.id), (1, 2));
    assert_eq!(first.path, "/org/kde/notificationmanager/jobs/JobView_1");
    assert_eq!(store.jobs().len(), 2);
}

/// A partial update keeps the properties it leaves out.
#[test]
fn partial_updates_keep_earlier_properties() {
    let mut store = JobStore::default();
    let job = store.request_view(1, request(3));
    store.update(2, update(job.id, JobChanges {
        title: Some("Analysing".to_string()),
        percent: Some(10),
        ..JobChanges::default()
    }));
    store.update(3, update(job.id, JobChanges { suspended: Some(true), ..JobChanges::default() }));
    let seen = store.find(job.id).expect("the job exists");
    assert_eq!((seen.title.as_str(), seen.percent, seen.suspended), ("Analysing", 10, true));
    assert_eq!(seen.info_message, "");
}

/// Terminate records the code and message and keeps the final state.
#[test]
fn terminate_marks_the_job_ended() {
    let mut store = JobStore::default();
    let job = store.request_view(1, request(1));
    store.terminate(2, TerminateCall {
        id: job.id,
        no_reply: true,
        code: 2,
        message: "disk full".to_string(),
        hints: Vec::new(),
    });
    let ended = store.find(job.id).and_then(|seen| return seen.termination.clone());
    assert_eq!(ended, Some(Termination { code: 2, message: "disk full".to_string() }));
}

/// Clearing drops the log and ended jobs but keeps running jobs.
#[test]
fn clear_keeps_running_jobs() {
    let mut store = JobStore::default();
    let running = store.request_view(1, request(3));
    let ended = store.request_view(2, request(3));
    store.terminate(3, TerminateCall {
        id: ended.id,
        no_reply: true,
        code: 0,
        message: String::new(),
        hints: Vec::new(),
    });
    store.clear();
    assert!(store.last_events(10).is_empty());
    assert_eq!(store.jobs().iter().map(|job| return job.id).collect::<Vec<_>>(), [running.id]);
    assert_eq!(store.request_view(4, request(0)).id, 3);
}

/// An update for an unknown job is still logged,
///  so a late call stays visible.
#[test]
fn updates_for_unknown_jobs_are_logged() {
    let mut store = JobStore::default();
    store.update(1, update(9, JobChanges::default()));
    assert_eq!(store.last_events(10).len(), 1);
    assert!(store.find(9).is_none());
}
