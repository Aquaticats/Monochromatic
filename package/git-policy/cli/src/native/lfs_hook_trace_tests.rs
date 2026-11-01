//! Controls for reading Git's trace2 event stream: which hook failure is blamed on the upload, and what cannot forge it.

use super::*;

/// The hook's command string as Git records it in the event stream.
const COMMAND: &str = HOOK_COMMAND;

/// One `child_start` event of a hook.
fn hook_start(sid: &str, id: i64, hook: &str, command: &str) -> String {
    return format!(
        "{{\"event\":\"child_start\",\"sid\":\"{sid}\",\"child_id\":{id},\"child_class\":\"hook\",\"hook_name\":\"{hook}\",\"argv\":[\"{command}\",\"origin\",\"url\"]}}"
    );
}

/// One `child_exit` event.
fn child_exit(sid: &str, id: i64, code: i64) -> String {
    return format!("{{\"event\":\"child_exit\",\"sid\":\"{sid}\",\"child_id\":{id},\"pid\":1,\"code\":{code}}}");
}

/// The first line every stream begins with.
fn start(sid: &str) -> String {
    return format!("{{\"event\":\"start\",\"sid\":\"{sid}\",\"argv\":[\"git\",\"push\"]}}");
}

/// Join event lines the way the file holds them.
fn stream(lines: &[String]) -> String {
    return lines.join("\n") + "\n";
}

/// A failing upload hook is found, with the status Git recorded, whatever language Git's own messages are in.
#[test]
fn a_failed_upload_hook_is_found_with_its_status() {
    let trace: String = stream(&[start("a-P1"), hook_start("a-P1", 1, "pre-push", COMMAND), child_exit("a-P1", 1, 7)]);
    assert_eq!(failed_hook_status(trace.as_str()), Some(7));
}

/// A passing hook, a hook that never exited and a stream without hooks blame nothing.
#[test]
fn a_passing_unfinished_or_absent_hook_blames_nothing() {
    let passing: String = stream(&[start("a-P1"), hook_start("a-P1", 1, "pre-push", COMMAND), child_exit("a-P1", 1, 0)]);
    assert_eq!(failed_hook_status(passing.as_str()), None);
    let unfinished: String = stream(&[start("a-P1"), hook_start("a-P1", 1, "pre-push", COMMAND)]);
    assert_eq!(failed_hook_status(unfinished.as_str()), None);
    assert_eq!(failed_hook_status(""), None);
    assert_eq!(failed_hook_status(stream(&[start("a-P1"), child_exit("a-P1", 0, 1)]).as_str()), None);
}

/// Another `pre-push` hook of the caller, or another hook kind, running the same ID range is never blamed on LFS.
#[test]
fn another_hook_is_not_blamed_on_the_upload() {
    let other_command: String = stream(&[start("a-P1"), hook_start("a-P1", 1, "pre-push", "their-check"), child_exit("a-P1", 1, 1)]);
    assert_eq!(failed_hook_status(other_command.as_str()), None);
    let other_hook: String = stream(&[start("a-P1"), hook_start("a-P1", 1, "pre-commit", COMMAND), child_exit("a-P1", 1, 1)]);
    assert_eq!(failed_hook_status(other_hook.as_str()), None);
}

/// A Git process the hook started writes events with a longer session ID; its failures are not the push's.
#[test]
fn events_of_nested_sessions_are_ignored() {
    let trace: String = stream(&[
        start("a-P1"),
        hook_start("a-P1", 1, "pre-push", COMMAND),
        hook_start("a-P1/b-P2", 5, "pre-push", COMMAND),
        child_exit("a-P1/b-P2", 5, 9),
        child_exit("a-P1", 1, 0),
    ]);
    assert_eq!(failed_hook_status(trace.as_str()), None);
}

/// Text that is not an event, or that is an event of another session, cannot add a failure.
#[test]
fn text_that_is_not_an_event_of_the_first_session_is_ignored() {
    let trace: String = stream(&[
        String::from("git-lfs: {\"event\":\"child_exit\",\"child_id\":1,\"code\":1}"),
        start("a-P1"),
        String::from("error: failed to push some refs"),
        hook_start("a-P1", 1, "pre-push", COMMAND),
        String::from("{\"event\":\"child_exit\",\"sid\":\"forged\",\"child_id\":1,\"code\":1}"),
        String::from("{\"event\":\"child_exit\",\"child_id\":1,\"code\":1}"),
        child_exit("a-P1", 1, 0),
    ]);
    assert_eq!(failed_hook_status(trace.as_str()), None);
}

/// A cut-off last line is skipped, and the first failure is the one reported.
#[test]
fn a_cut_off_line_is_skipped_and_the_first_failure_wins() {
    let mut trace: String = stream(&[
        start("a-P1"),
        hook_start("a-P1", 1, "pre-push", COMMAND),
        child_exit("a-P1", 1, 3),
        hook_start("a-P1", 2, "pre-push", COMMAND),
        child_exit("a-P1", 2, 4),
    ]);
    trace.push_str("{\"event\":\"child_ex");
    assert_eq!(failed_hook_status(trace.as_str()), Some(3));
}

/// A caller's own `GIT_TRACE2_EVENT` is kept: no trace file is made and no explanation follows.
#[test]
fn a_callers_trace_target_is_never_replaced() {
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from("GIT_TRACE2_EVENT"), OsString::from("/elsewhere"))];
    assert!(HookTrace::start(environment.as_slice()).is_none());
}

/// A trace file Git wrote is read back as the explanation, and an unwritten one explains nothing.
#[test]
fn the_explanation_comes_from_the_file_git_wrote() {
    use crate::test_support::{fixture, remove};
    let root = fixture("lfs-hook-trace-file");
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from("XDG_STATE_HOME"), root.clone().into_os_string())];
    let trace: HookTrace = HookTrace::start(environment.as_slice()).expect("a private directory exists");
    assert_eq!(trace.explanation(), "");
    let (name, path) = trace.entry();
    assert_eq!(name, OsString::from("GIT_TRACE2_EVENT"));
    let terminated: i64 = crate::lfs_pre_push::SIGNAL_STATUS_BASE + i64::from(libc::SIGTERM);
    let written: String =
        stream(&[start("a-P1"), hook_start("a-P1", 1, "pre-push", COMMAND), child_exit("a-P1", 1, terminated)]);
    std::fs::write(&path, written).expect("trace written");
    let explanation: String = trace.explanation();
    assert!(
        explanation.contains(&format!("exited with status {terminated} (that is signal {}", libc::SIGTERM)),
        "{explanation}"
    );
    assert!(explanation.contains("nothing was published"), "{explanation}");
    drop(trace);
    assert!(!std::path::Path::new(&path).exists(), "the trace file is removed with the value");
    remove(root.as_path());
}
