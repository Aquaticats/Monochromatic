//! Controls for the post-commit actions: fields, working directory, environment, endings, order and bounds.

use super::*;
use crate::post_commit_template::parse_template;
use crate::test_support::{executable, fixture, remove};

/// A declared action from plain argument sources.
fn action(name: &str, argv: &[&str], timeout_seconds: u64) -> PostCommitAction {
    return PostCommitAction {
        name: String::from(name),
        argv: argv.iter().map(|source| { return parse_template(source).expect("template"); }).collect(),
        timeout_seconds,
    };
}

/// A program that records its arguments, working directory and one environment variable into `record`.
fn recorder(root: &Path, record: &Path) -> PathBuf {
    let program: PathBuf = root.join("record");
    executable(
        program.as_path(),
        format!(
            "#!/bin/sh\n{{ printf 'args=%s\\n' \"$*\"; printf 'cwd=%s\\n' \"$(pwd -P)\"; printf 'marker=%s\\n' \"$ACTION_MARKER\"; }} >> '{}'\n",
            record.display()
        )
        .as_bytes(),
    );
    return program;
}

/// The wrapper's environment for a run.
fn environment() -> Vec<(OsString, OsString)> {
    let mut variables: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    variables.push((OsString::from("ACTION_MARKER"), OsString::from("inherited")));
    return variables;
}

/// Run `actions` for a landing of `oid` in `top_level`.
fn run(actions: &[PostCommitAction], top_level: &Path, scratch: &Path) -> Vec<ActionReport> {
    let variables: Vec<(OsString, OsString)> = environment();
    return run_post_commit_actions(&PostCommitActionsRequest {
        actions,
        landed_oid: "0123456789abcdef0123456789abcdef01234567",
        top_level,
        environment: variables.as_slice(),
        scratch,
    });
}

/// An action starts once in the top level with its fields filled in and the wrapper's environment.
#[test]
fn an_action_runs_with_its_fields_in_the_top_level_with_the_environment() {
    let root: PathBuf = fixture("action-runs");
    let record: PathBuf = root.join("record.txt");
    let program: PathBuf = recorder(root.as_path(), record.as_path());
    let top: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    let declared = action("review", &[program.to_str().expect("UTF-8"), "--unit=u-{shortHead}", "{head}", "{topLevel}"], 30);
    let reports: Vec<ActionReport> = run(&[declared], top.as_path(), root.as_path());
    assert_eq!(reports, vec![ActionReport { name: String::from("review"), outcome: ActionOutcome::Succeeded }]);
    let seen: String = std::fs::read_to_string(&record).expect("recorded");
    let short: String = "0123456789abcdef0123456789abcdef01234567".chars().take(crate::post_commit_template::SHORT_HEAD_LENGTH).collect();
    assert!(seen.contains(&format!("args=--unit=u-{short} 0123456789abcdef0123456789abcdef01234567 {}", top.display())), "{seen}");
    assert!(seen.contains(&format!("cwd={}", top.display())), "{seen}");
    assert!(seen.contains("marker=inherited"), "{seen}");
    remove(root.as_path());
}

/// Shell syntax in an argument is text: nothing runs it.
#[test]
fn shell_syntax_in_an_argument_is_never_run() {
    let root: PathBuf = fixture("action-no-shell");
    let record: PathBuf = root.join("record.txt");
    let program: PathBuf = recorder(root.as_path(), record.as_path());
    let injected: PathBuf = root.join("injected");
    let text: String = format!("$(touch {}); `touch {}`", injected.display(), injected.display());
    let reports = run(&[action("literal", &[program.to_str().expect("UTF-8"), text.as_str()], 30)], root.as_path(), root.as_path());
    assert_eq!(reports[0].outcome, ActionOutcome::Succeeded);
    assert!(!injected.exists(), "an argument must never be run by a shell");
    assert!(std::fs::read_to_string(&record).expect("recorded").contains("$(touch"));
    remove(root.as_path());
}

/// A failing action, a missing program and an unset variable are reported, none stops the next action, and the next
/// one still runs.
#[test]
fn a_failure_is_reported_and_the_next_action_still_runs() {
    let root: PathBuf = fixture("action-failures");
    let record: PathBuf = root.join("record.txt");
    let program: PathBuf = recorder(root.as_path(), record.as_path());
    let failing: PathBuf = root.join("fails");
    executable(failing.as_path(), b"#!/bin/sh\nexit 3\n");
    let actions: Vec<PostCommitAction> = vec![
        action("fails", &[failing.to_str().expect("UTF-8")], 30),
        action("missing", &["/nonexistent/program"], 30),
        action("unset", &[program.to_str().expect("UTF-8"), "{env:CLI_GIT_NEVER_SET_VARIABLE}"], 30),
        action("last", &[program.to_str().expect("UTF-8"), "ran"], 30),
    ];
    let reports: Vec<ActionReport> = run(actions.as_slice(), root.as_path(), root.as_path());
    assert_eq!(reports[0].outcome, ActionOutcome::Failed(3));
    assert!(matches!(&reports[1].outcome, ActionOutcome::NotRun(reason) if reason.contains("could not start")), "{reports:?}");
    assert_eq!(
        reports[2].outcome,
        ActionOutcome::NotRun(String::from("the environment variable CLI_GIT_NEVER_SET_VARIABLE is not set"))
    );
    assert_eq!(reports[3].outcome, ActionOutcome::Succeeded);
    assert!(std::fs::read_to_string(&record).expect("recorded").contains("args=ran"));
    remove(root.as_path());
}

/// An action that outlives its bound is stopped and the run goes on.
#[test]
fn an_action_that_outlives_its_bound_is_stopped() {
    let root: PathBuf = fixture("action-timeout");
    let slow: PathBuf = root.join("slow");
    executable(slow.as_path(), b"#!/bin/sh\nexec sleep 30\n");
    let started: std::time::Instant = std::time::Instant::now();
    let reports = run(&[action("slow", &[slow.to_str().expect("UTF-8")], 1)], root.as_path(), root.as_path());
    assert!(matches!(&reports[0].outcome, ActionOutcome::NotRun(reason) if reason.contains("stopped")), "{reports:?}");
    assert!(started.elapsed() < Duration::from_secs(20), "the bound must end the wait");
    remove(root.as_path());
}

/// A notice names the action, what happened and that the commit stays landed; success is silent.
#[test]
fn notices_name_the_action_and_say_the_commit_stays_landed() {
    assert_eq!(notice("a", &ActionOutcome::Succeeded), None);
    let failed: String = notice("review", &ActionOutcome::Failed(3)).expect("notice");
    assert!(failed.contains("review") && failed.contains("status 3") && failed.contains("stays landed"), "{failed}");
    let signaled: String = notice("review", &ActionOutcome::Signaled(9)).expect("notice");
    assert!(signaled.contains("signal 9"), "{signaled}");
    let not_run: String = notice("review", &ActionOutcome::NotRun(String::from("why"))).expect("notice");
    assert!(not_run.contains("did not run: why") && not_run.contains("postCommit.actions"), "{not_run}");
}
